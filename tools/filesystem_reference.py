#!/usr/bin/env python3
"""Development-only GNU comparisons: checked-in literals, disposable fixtures only.
No learner input, imported packs, CLI script arguments or personal paths accepted.
Bash is a reference for fixed wrapper/cd/umask commands, never the learner engine.
"""
import json
import os
import pathlib
import shlex
import shutil
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
PROBE = ROOT / 'target/debug/examples/filesystem_probe'
FIXTURES = ROOT / 'tests/fixtures/filesystem-reference.json'
BASH = shutil.which('bash')
COMMANDS = {'pwd', 'cd', 'umask', 'ls', 'cat', 'touch', 'mkdir', 'cp', 'mv', 'rm', 'rmdir'}
if not BASH or os.name != 'posix' or os.getuid() == 0:
    raise SystemExit('Ordinary-user POSIX/Bash reference environment unavailable: NOT RUN')


def tree(entries):
    nodes = {p: {'directory': True, 'mode': 0o755, 'owner': 'root', 'group': 'root'} for p in ['/', '/work']}
    nodes['/work'].update(owner='alex', group='staff')
    files = {}
    for name, item in entries.items():
        path = '/work/' + name
        nodes[path] = {'directory': 'text' not in item, 'mode': item['mode'], 'owner': 'alex', 'group': 'staff'}
        if 'text' in item:
            files[path] = item['text']
    return {'nodes': nodes, 'files': files, 'users': {'root': ['root'], 'alex': ['staff']},
            'user': 'alex', 'home': '/work', 'cwd': '/work', 'umask': 0o022}


def snapshot(work):
    # Inspect our disposable fixture as its owner, restoring all permissions before
    # the next reference command. No production or learner paths enter this tool.
    files, modes, saved = {}, {}, []
    try:
        for directory, dirs, names in os.walk(work):
            for name in dirs + names:
                path = pathlib.Path(directory) / name
                key = '/work/' + str(path.relative_to(work))
                mode = path.stat().st_mode & 0o777
                modes[key] = mode
                saved.append((path, mode))
                path.chmod(mode | (0o700 if path.is_dir() else 0o400))
                if path.is_file():
                    files[key] = path.read_text()
    finally:
        for path, mode in reversed(saved):
            path.chmod(mode)
    return files, modes


failures = []
count = 0
long_rows = 0
for fixture in json.loads(FIXTURES.read_text()):
    initial = tree(fixture['entries'])
    simulated = json.loads(subprocess.run([str(PROBE)], input=json.dumps({'tree': initial, 'commands': fixture['commands']}),
                                         capture_output=True, text=True, check=True, timeout=3).stdout)
    with tempfile.TemporaryDirectory(prefix='dlms-files-reference-') as temporary:
        work = pathlib.Path(temporary).resolve()
        for name, entry in fixture['entries'].items():
            relative = pathlib.Path(name)
            assert not relative.is_absolute() and '..' not in relative.parts
            target = work / relative
            if 'text' in entry:
                target.write_text(entry['text'])
            else:
                target.mkdir()
            target.chmod(entry['mode'])
        cwd, previous, mask = work, None, 0o022
        for command, actual in zip(fixture['commands'], simulated, strict=True):
            argv = shlex.split(command)
            assert argv[0] in COMMANDS
            # Whitelist literal operands inside this disposable root before invoking utilities.
            for operand in argv[1:]:
                if operand.startswith('-'):
                    continue
                assert not pathlib.Path(operand).is_absolute()
                candidate = (cwd / operand).resolve()
                assert candidate == work or work in candidate.parents
            if argv[0] == 'cd':
                args = argv[1:]
                if args and args[0] == '--':
                    args = args[1:]
                target = str(work) if not args else str(previous) if args == ['-'] and previous else args[0]
                result = subprocess.run([BASH, '--noprofile', '--norc', '-c', 'cd -- "$1" && pwd -P', 'fixture', target],
                                        cwd=cwd, env={'PATH': '/usr/bin:/bin', 'HOME': str(work), 'LC_ALL': 'C'},
                                        stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=2)
                output = ''
                if result.returncode == 0:
                    previous, cwd = cwd, pathlib.Path(result.stdout.rstrip('\n'))
                    if args == ['-']:
                        output = '/work' + str(cwd)[len(str(work)):] + '\n'
            else:
                result = subprocess.run([BASH, '--noprofile', '--norc', '-c', 'umask "$1"; shift; "$@"', 'fixture', f'{mask:03o}', *argv],
                                        cwd=cwd, env={'PATH': '/usr/bin:/bin', 'HOME': str(work), 'LC_ALL': 'C'},
                                        stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=2)
                output = result.stdout.replace(str(work), '/work')
                if argv[0] == 'umask' and len(argv) == 2 and result.returncode == 0:
                    mask = int(argv[1], 8)
            files, modes = snapshot(work)
            expected = {'ok': result.returncode == 0, 'output': output if result.returncode == 0 else '',
                        'files': files, 'modes': modes, 'cwd': '/work' + str(cwd)[len(str(work)):], 'umask': mask}
            state = actual['tree']
            observed = {'ok': actual['ok'], 'output': actual['output'], 'files': state['files'],
                        'modes': {p: n['mode'] for p, n in state['nodes'].items() if p.startswith('/work/')},
                        'cwd': state['cwd'], 'umask': state['umask']}
            if argv[0] == 'ls' and any(a.startswith('-') and 'l' in a for a in argv[1:]):
                # Owner names, timestamps, directory byte sizes and SELinux/ACL
                # suffixes deliberately differ. Compare the modeled fields only.
                def long_fields(text):
                    rows = [row.split(maxsplit=8) for row in text.splitlines() if not row.startswith('total ')]
                    return [(r[0][:10], r[1], r[4] if r[0][0] != 'd' else None, r[8]) for r in rows]
                expected['output'] = long_fields(expected['output'])
                observed['output'] = long_fields(observed['output'])
                long_rows += 1
            count += 1
            if expected != observed:
                failures.append({'fixture': fixture['name'], 'command': command, 'gnu': expected, 'simulated': observed})
print(json.dumps({'fixtures': count, 'long_listing_subset_checks': long_rows, 'failures': failures}, indent=2))
raise SystemExit(bool(failures))
