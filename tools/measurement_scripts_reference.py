#!/usr/bin/env python3
"""Fixed authoring comparison, MAIN runner only; never executes learner scripts.

Runs exactly three reviewed canonical/alternate twelve-case fixtures and separate
examples. This is a composition check, not a new interpreter or migration baseline.
"""
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile
import time

from automation_before10_oracle import pinned_executable
from case_bash_oracle import capture_command, closed_environment
from native_reopen import inherited_lock, read_bytes, pin
from startup_reference import Budget, fresh, limits, output, strict

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = (('rates', 'tests/fixtures/shell-measurement-rates-reference.json', 'c2d51aac2b082fe03f9a0d96b5b8f414f79fe4edfdda21ec4a62c9df72b76181'), ('cpu', 'tests/fixtures/shell-measurement-cpu-reference.json', 'f74b328fbd9f66200b24401e7d54a662e618ad7762cc954bce859d0a3294297e'), ('recovery', 'tests/fixtures/shell-measurement-recovery-reference.json', '217bd02ceb11ac0b41cab827d7ea8617728d147d551ac652c5a934d7c1211697'))
CAP = 256 * 1024
CASE_FIELDS = {'name', 'args', 'stdin', 'env', 'files', 'stdout', 'stderr', 'status', 'after'}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def file_map(files):
    require(isinstance(files, dict) and len(files) <= 8, 'Bounded flat fixture file map')
    total = 0
    for name, text in files.items():
        require(isinstance(name, str) and re.fullmatch(r'/work/[A-Za-z0-9_.-]{1,80}', name)
                and name[6:] not in ('.', '..'), 'Only flat ordinary /work fixture names')
        require(isinstance(text, str) and '\0' not in text and len(text.encode()) <= 8192,
                'Bounded UTF8 fixture text')
        total += len(text.encode())
    require(total <= 32 * 1024, 'Fixture aggregate bound')


def validate(value):
    require(isinstance(value, dict) and set(value) == {
        'schema_version', 'source', 'alternate', 'cases', 'example_source', 'example_case'
    } and type(value['schema_version']) is int and value['schema_version'] == 1,
            'Fixed authoring fixture schema')
    for key in ('source', 'alternate', 'example_source'):
        require(isinstance(value[key], str) and 0 < len(value[key].encode()) <= 16_384
                and '\0' not in value[key], 'Bounded reviewed source')
    require(isinstance(value['cases'], list) and len(value['cases']) == 12,
            'Exactly twelve fixed cases per measurement program')
    rows = [*value['cases'], value['example_case']]
    names = set()
    for case in rows:
        require(isinstance(case, dict) and set(case) == CASE_FIELDS, 'Closed case fields')
        require(isinstance(case['name'], str) and 0 < len(case['name']) < 120
                and case['name'] not in names, 'Distinct bounded case names')
        names.add(case['name'])
        require(isinstance(case['args'], list) and len(case['args']) <= 8
                and all(isinstance(s, str) and '\0' not in s and len(s.encode()) <= 256
                        for s in case['args']), 'Bounded fixed argv')
        require(case['env'] == {} and case['stdin'] == '', 'No caller environment or input')
        require(type(case['status']) is int and 0 <= case['status'] <= 255,
                'Exact ordinary exit status')
        require(all(isinstance(case[k], str) and '\0' not in case[k]
                    and len(case[k].encode()) <= 32_768 for k in ('stdout', 'stderr')),
                'Bounded expected streams')
        file_map(case['files']); file_map(case['after'])
    return value


def fixtures():
    values=[]
    for label,name,sha in FIXTURES:
        raw=read_bytes(ROOT/name,CAP)
        require(hashlib.sha256(raw).hexdigest()==sha,'Reviewed measurement fixture changed; no source selector or automatic pin refresh')
        values.append((label,validate(strict(raw)),raw))
    return values


def snapshot(work):
    files = {}
    for path in sorted(work.iterdir()):
        info = path.lstat()
        require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1,
                'No reference symlink, directory or special-file result')
        require(len(files) < 8, 'Reference result file count')
        files['/work/' + path.name] = read_bytes(path, 8192).decode('utf-8')
    file_map(files)
    return files


def main():
    require(len(sys.argv) == 1, 'Fixed comparator has no source/path/case selector')
    lock = inherited_lock()
    require((ROOT / '.git').is_dir() and not (ROOT / '.git').is_symlink()
            and Path(os.environ.get('CARGO_TARGET_DIR', '')) == ROOT / 'target',
            'MAIN/default-target inherited workflow only')
    values = fixtures()
    system = {name: Path('/usr/bin', name).resolve(strict=True)
              for name in ('bash', 'cat', 'grep', 'cut')}
    identities = {name: {'path': str(path), **pin(path, 16 * 1024 * 1024)}
                  for name, path in system.items()}
    base = ROOT / '.build-artifacts/reference-runs'
    require(base.is_dir() and base.resolve() == base and not base.is_symlink(),
            'Existing canonical project-disk reference directory')
    run = Path(tempfile.mkdtemp(prefix='measurement-scripts-', dir=base))
    for label, _, raw in values:
        (run / ('fixed-' + label + '-inputs.json')).write_bytes(raw)
    retained = {}
    for name in ('tools/measurement_scripts_reference.py', 'tools/native_reopen.py',
                 'tools/automation_before10_oracle.py', 'tools/case_bash_oracle.py',
                 'tools/startup_reference.py', 'tools/dev.py', 'tools/build_safety.py'):
        data = read_bytes(ROOT / name, CAP)
        target = run / 'reviewed-tools' / name
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open('xb') as stream:
            stream.write(data); stream.flush(); os.fsync(stream.fileno())
        retained[name] = {'bytes': len(data), 'sha256': hashlib.sha256(data).hexdigest()}
    output(run / 'INPUTS.json', {'fixture_sha256': {label:sha for label,_,sha in FIXTURES},
           'tools': retained,
           'executables': identities, 'purpose': 'Fixed authoring composition; not an evaluator baseline'})
    rows = [(label + '-' + variant + f'-{i:02}', value[variant], case)
            for label,value,_ in values
            for variant in ('source','alternate') for i,case in enumerate(value['cases'])]
    rows.extend((label+'-example',value['example_source'],value['example_case'])for label,value,_ in values)
    observed = []
    budget = Budget()
    started = time.monotonic()
    with ExitStack() as stack:
        held = {name: stack.enter_context(pinned_executable(system[name], meta, 16 * 1024 * 1024))
                for name, meta in identities.items()}
        version_folder = fresh(run, 'bash-version')
        version = capture_command([held['bash'][0], '--version'], version_folder, run,
                                  closed_environment({}, run), b'', budget, 8192, timeout=3,
                                  inherited=(lock, held['bash'][1]))
        output(run / 'BASH_VERSION.json', version)
        require(version['status'] == 0 and not version['stderr'], 'Reference Bash version failed')
        header = ''.join(f'{name}() {{ (exec -a {name} /proc/self/fd/{held[name][1]} "$@"); }}\n'
                         for name in ('cat', 'grep', 'cut'))
        for identity, source, case in rows:
            require(time.monotonic() - started < 90, 'Whole comparison time bound')
            folder = fresh(run, identity)
            work = fresh(folder, 'work'); capture = fresh(folder, 'capture')
            for name, text in case['files'].items():
                with (work / name[6:]).open('xb') as stream:
                    stream.write(text.encode()); stream.flush(); os.fsync(stream.fileno())
            output(folder / 'case.json', case)
            with (folder / 'source.bash').open('x') as stream:
                stream.write(header + source)
            actual = capture_command(
                [held['bash'][0], '--noprofile', '--norc', '-c', header + source,
                 'fixed-measurement-scripts', *case['args']], capture, work,
                closed_environment({}, work), b'', budget, 65_536, timeout=3,
                inherited=tuple([lock] + [fd for _, fd in held.values()]),
                popen=lambda *args, **kwargs: subprocess.Popen(*args, preexec_fn=limits, **kwargs))
            output(folder / 'execution.json', actual)
            require(type(actual['status']) is int and 0 <= actual['status'] <= 255,
                    'Reference failed, timed out or was signaled; diagnostics retained')
            result = {key: actual[key] for key in ('stdout', 'stderr', 'status')}
            result['files'] = snapshot(work)
            expected = {key: case[key] for key in ('stdout', 'stderr', 'status')}
            expected['files'] = case['after']
            observed.append({'id': identity, 'name': case['name'], 'actual': result,
                             'expected': expected, 'passed': result == expected})
            output(folder / 'observation.json', observed[-1])
    report = {'purpose': 'Actual fixed Bash composition, not learner execution or full conformance',
              'fixture_sha256': {label:sha for label,_,sha in FIXTURES}, 'executables': identities,
              'bash_version': version['stdout'],
              'observations': observed, 'passed': all(r['passed'] for r in observed)}
    output(run / 'report.json', report)
    print(json.dumps({'output': str(run), 'cases': len(observed), 'passed': report['passed']}))
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
