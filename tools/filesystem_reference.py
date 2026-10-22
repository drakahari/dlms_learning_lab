#!/usr/bin/env python3
"""Development-only GNU comparisons: checked-in literals, disposable fixtures only.
No learner input, imported packs, CLI script arguments or personal paths accepted.
Bash is a reference for fixed wrapper/cd/umask commands, never the learner engine.
"""
import json
import os
import pathlib
import re
import shlex
import shutil
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
PROBE = ROOT / 'target/debug/examples/filesystem_probe'
FIXTURES = ROOT / 'tests/fixtures/filesystem-reference.json'
BASH = shutil.which('bash')
COMMANDS = {'pwd', 'cd', 'umask', 'ls', 'cat', 'touch', 'mkdir', 'cp', 'mv', 'rm', 'rmdir', 'chmod'}


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


def validate_argv(argv, cwd, work):
    if len(argv) > 12 or not argv or argv[0] not in COMMANDS:
        raise ValueError('Reference command is not whitelisted')
    if cwd != work and work not in cwd.resolve().parents:
        raise ValueError('Reference cwd escapes disposable fixture')
    operands, options = [], True
    arguments = argv[1:]
    if argv[0] == 'chmod':
        if arguments and arguments[0] == '--':
            arguments = arguments[1:]
            options = False
        if len(arguments) != 2:
            raise ValueError('Reference chmod requires one mode and one path')
        mode, operand = arguments
        numeric = 1 <= len(mode) <= 4 and all(c in '01234567' for c in mode) and int(mode, 8) <= 0o777
        symbolic = len(mode) <= 512 and len(mode.split(',')) <= 32 and re.fullmatch(r'[ugoa]+[-+=][rwx]*(?:,[ugoa]+[-+=][rwx]*)*', mode) is not None
        if not (numeric or symbolic):
            raise ValueError('Reference chmod mode outside the declared subset')
        arguments = [operand]
    for arg in arguments:
        if options and arg == '--':
            options = False
            continue
        if options and arg.startswith('-') and arg != '-':
            allowed = ((argv[0] == 'ls' and len(arg) > 1 and all(c in 'ald' for c in arg[1:]))
                       or (argv[0] == 'pwd' and arg in ('-P', '-L'))
                       or (argv[0] == 'rm' and arg == '-f'))
            if not allowed:
                raise ValueError('Reference option is outside the declared subset')
            continue
        operands.append(arg)
    for operand in operands:
        candidate = (cwd / operand).resolve()
        if pathlib.Path(operand).is_absolute() or (candidate != work and work not in candidate.parents):
            raise ValueError('Reference operand escapes disposable fixture')
    if argv[0] == 'umask' and (len(argv) > 2 or (len(argv) == 2 and (not argv[1] or len(argv[1]) > 4 or any(c not in '01234567' for c in argv[1]) or int(argv[1], 8) > 0o777))):
        raise ValueError('Reference mask is outside the declared subset')


def run_checked(argv, cwd, work, mask, previous):
    validate_argv(argv, cwd, work)
    if previous is not None and previous != work and work not in previous.resolve().parents:
        raise ValueError('Reference previous directory escapes disposable fixture')
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
    return result, output, cwd, previous, mask


def main():
    if not BASH or os.name != 'posix' or os.getuid() == 0:
        raise SystemExit('Ordinary-user POSIX/Bash reference environment unavailable: NOT RUN')
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
                result, output, cwd, previous, mask = run_checked(argv, cwd, work, mask, previous)
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
    return bool(failures)


if __name__ == '__main__':
    raise SystemExit(main())
