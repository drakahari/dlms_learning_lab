#!/usr/bin/env python3
"""Development only: compare checked-in fixtures in disposable directories.
No learner input, arguments, imported scripts or personal files are accepted.
"""
import json
import pathlib
import shutil
import subprocess
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
BASH = shutil.which('bash')
PROBE = ROOT / 'target/debug/examples/shell_probe'
if not BASH:
    raise SystemExit('Bash unavailable: reference comparison NOT RUN')
fixtures = json.loads((ROOT / 'tests/fixtures/bash-reference.json').read_text())
failures = []
for fixture in fixtures:
    case = fixture['case']
    with tempfile.TemporaryDirectory(prefix='dlms-bash-reference-') as temporary:
        work = pathlib.Path(temporary)
        for name, text in case['files'].items():
            relative = name.removeprefix('/work/')
            assert not relative.startswith('/') and '..' not in pathlib.Path(relative).parts
            destination = work / relative
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(text.encode())
        result = subprocess.run([BASH, '--noprofile', '--norc', '-c', fixture['source'], 'fixture', *case['args']],
                                cwd=work, env={'PATH': '/usr/bin:/bin', 'LC_ALL': 'C', **case['env']},
                                input=case['stdin'].encode(), capture_output=True, timeout=2)
        after = {'/work/' + str(p.relative_to(work)): p.read_bytes().decode() for p in work.rglob('*') if p.is_file()}
        actual = json.loads(subprocess.run([str(PROBE)],input=json.dumps({'source': fixture['source'], 'case': case}),capture_output=True,text=True,check=True,timeout=2).stdout)
        expected = {'stdout': result.stdout.decode(), 'stderr': result.stderr.decode(), 'status': result.returncode, 'files': after, 'error': None}
        if actual != expected:
            failures.append({'name': fixture['name'], 'actual': actual, 'bash': expected})
print(json.dumps({'bash': subprocess.check_output([BASH, '--version'],text=True).splitlines()[0], 'fixtures': len(fixtures), 'failures': failures}, indent=2))
raise SystemExit(bool(failures))
