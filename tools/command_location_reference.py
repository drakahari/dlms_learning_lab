#!/usr/bin/env python3
"""Fixed owner-permission lookup comparison, never a learner execution route.

Prepare launches nothing. Observe requires exact reviewed source/ELF pins.
Compare separately requires an approved completed observation and fixed probe.
Only project-disk inert fixtures and held Bash/which executables are used.
"""
import argparse
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import stat
import subprocess
import sys
import time

from automation_before10_oracle import pinned_executable
from case_bash_oracle import capture_command, closed_environment
from native_reopen import inherited_lock, pin, read_bytes
from startup_reference import Budget, fresh, limits, output, strict

ROOT = Path(__file__).resolve().parents[1]
BASE = '.build-artifacts/reference-runs/command-location-r2'
FIXTURE = 'tests/fixtures/command-location-reference-v1.json'
FIXTURE_SHA = '6bec8c151f8d74bac2b2e7053742080c5bad0043a8aefb0dbfa16a52c482af33'
PROBE = 'target/debug/examples/command_location_reference'
BINDINGS = 'examples/support/command_location_bindings.rs'
BODY = {'preferred': b'Inert preferred artifact\n', 'fallback': b'Inert fallback artifact\n'}
IDS = ('preferred-not-executable', 'preferred-restored', 'neither-file-executable',
       'execute-without-read', 'preferred-search-denied', 'both-search-denied',
       'directory-is-not-tool', 'reversed-search-order', 'type-all-bypasses-hash')
FIELDS = {'id', 'preferred_mode', 'fallback_mode', 'preferred_dir_mode',
          'fallback_dir_mode', 'preferred_directory', 'reverse_path', 'preload_hash'}
TOOL_INPUTS = ('tools/command_location_reference.py', 'tools/dev.py',
               'tools/build_safety.py', 'tools/native_reopen.py',
               'tools/automation_before10_oracle.py', 'tools/case_bash_oracle.py',
               'tools/startup_reference.py', 'tools/tests/test_command_location_reference.py')


def require(ok, message):
    if not ok:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def reference_limits():
    limits()
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))


def validate(value):
    require(isinstance(value, dict) and set(value) == {'schema_version', 'cases'}
            and type(value['schema_version']) is int and value['schema_version'] == 1,
            'Closed fixed fixture schema')
    rows = value['cases']
    require(isinstance(rows, list) and len(rows) == 9
            and tuple(r.get('id') for r in rows if isinstance(r, dict)) == IDS,
            'Exact nine ordered reference states')
    for row in rows:
        require(set(row) == FIELDS, 'Closed reference fields; no source/path/argv selector')
        for key in ('preferred_mode', 'fallback_mode', 'preferred_dir_mode', 'fallback_dir_mode'):
            require(type(row[key]) is int and row[key] in (0, 0o100, 0o600, 0o700),
                    'Only reviewed ordinary owner permission values')
        require(all(type(row[k]) is bool for k in ('preferred_directory', 'reverse_path', 'preload_hash')),
                'Actual boolean reference flags')
        require(row['preload_hash'] == (row['id'] == IDS[-1]), 'Only the fixed hash-bypass case')
    return rows


def fixture():
    raw = read_bytes(ROOT / FIXTURE, 16 * 1024)
    require(digest(raw) == FIXTURE_SHA, 'Fixed fixture changed; review new bytes first')
    return validate(strict(raw))


def compiled_inputs():
    text = read_bytes(ROOT / BINDINGS, 512 * 1024).decode()
    pairs = re.findall(r'\(\s*"([A-Za-z0-9_./-]+)"\s*,\s*include_bytes!\(\s*"([A-Za-z0-9_./-]+)"\s*\)\s*,?\s*\)', text)
    actual = {p.relative_to(ROOT).as_posix() for family in ('src', 'scenarios', 'assets')
              for p in (ROOT / family).rglob('*') if p.is_file()}
    actual.update(('Cargo.toml', 'Cargo.lock', 'examples/command_location_reference.rs',
                   'examples/support/automation_digest.rs', BINDINGS, FIXTURE))
    require(0 < len(pairs) <= 1200 and len(pairs) == len({n for n, _ in pairs})
            and {n for n, _ in pairs} == actual, 'Fixed probe compiled-source closure differs')
    result = []
    for name, included in pairs:
        require(included == '../../' + name and '..' not in Path(name).parts,
                'No source path alias or escape')
        path = ROOT / name
        require(path.resolve() == path, 'Canonical ordinary compiled input')
        result.append({'path': name, **pin(path, 4 * 1024 * 1024)})
    return result


def control():
    fixture()
    return {'purpose': 'fixed-command-location-reference-v1', 'root': str(ROOT),
            'fixture_sha256': FIXTURE_SHA, 'compiled_inputs': compiled_inputs(),
            'tools': {n: pin(ROOT / n, 4 * 1024 * 1024) for n in TOOL_INPUTS},
            'programs': {n: {'path': str(Path('/usr/bin', n).resolve(strict=True)),
                             **pin(Path('/usr/bin', n).resolve(strict=True), 16 * 1024 * 1024)}
                         for n in ('bash', 'which')}}


def generated_environment(work, row):
    """Only this audited materializer supplies PATH/HOME; no caller env overrides."""
    require(work.resolve() == work and work.is_relative_to(ROOT / BASE), 'Private canonical fixture root')
    dirs = ['fallback', 'preferred'] if row['reverse_path'] else ['preferred', 'fallback']
    require(all((work / d).is_dir() and not (work / d).is_symlink() for d in dirs),
            'Actual fixed private PATH directories')
    home = work / 'work'
    require(home.is_dir() and not home.is_symlink(), 'Contained generated HOME')
    env = closed_environment({}, work)
    env['PATH'] = ':'.join(str(work / d) for d in dirs)
    env['HOME'] = str(home)
    return env


def materialize(folder, row, stack):
    work = fresh(folder, 'fixture')
    fresh(work, 'work')
    held = {}
    for name in ('preferred', 'fallback'):
        branch = fresh(work, name)
        directory = row['preferred_directory'] and name == 'preferred'
        path = branch / 'report'
        if directory:
            path.mkdir(mode=0o700)
            fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        else:
            fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            require(os.write(fd, BODY[name]) == len(BODY[name]), 'Complete inert artifact write')
            os.fsync(fd)
        stack.callback(os.close, fd)
        branch_fd = os.open(branch, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        stack.callback(os.close, branch_fd)
        os.fchmod(fd, row[name + '_mode'])
        os.fchmod(branch_fd, row[name + '_dir_mode'])
        held[name] = (fd, branch_fd, directory, (os.fstat(fd), os.fstat(branch_fd)))
    return work, held


def snapshot(held):
    result = {}
    for name, (fd, parent, directory, original) in held.items():
        info = os.fstat(fd)
        require((stat.S_ISDIR(info.st_mode) if directory else stat.S_ISREG(info.st_mode))
                and (info.st_nlink >= 1 if directory else info.st_nlink == 1) and info.st_uid == os.geteuid()
                and (info.st_dev,info.st_ino,info.st_nlink) ==
                    (original[0].st_dev,original[0].st_ino,original[0].st_nlink)
                and (os.fstat(parent).st_dev,os.fstat(parent).st_ino,os.fstat(parent).st_nlink) ==
                    (original[1].st_dev,original[1].st_ino,original[1].st_nlink),
                'Original owned inert artifact descriptor')
        body = b'' if directory else os.pread(fd, 1024, 0)
        require(directory or body == BODY[name], 'Inert artifact bytes changed')
        result[name] = {'directory': directory, 'device':info.st_dev,'inode':info.st_ino,
                        'nlink':info.st_nlink,'parent_inode':os.fstat(parent).st_ino, 'mode': stat.S_IMODE(info.st_mode),
                        'parent_mode': stat.S_IMODE(os.fstat(parent).st_mode),
                        'file_mtime_ns':info.st_mtime_ns,'file_ctime_ns':info.st_ctime_ns,
                        'parent_mtime_ns':os.fstat(parent).st_mtime_ns,'parent_ctime_ns':os.fstat(parent).st_ctime_ns,
                        'bytes': len(body), 'sha256': digest(body)}
    return result


def normalized(actual, work):
    require(type(actual.get('status')) is int and actual['status'] in (0, 1),
            'Reference could not run, was signaled or returned unexpected status; never call that Missing')
    stdout = actual['stdout']
    require(isinstance(stdout, str) and len(stdout.encode()) <= 32768, 'Bound actual path output')
    paths = []
    for line in stdout.splitlines(keepends=True):
        require(line.endswith('\n') and line[:-1] in
                (str(work / 'preferred/report'), str(work / 'fallback/report')),
                'No dropped output, aliases, outside path or malformed line')
        paths.append(line[len(str(work)):-1])
    require((actual['status'] == 0) == bool(paths), 'Success must contain paths; failed query must be empty')
    return {'status': 'Found' if actual['status'] == 0 else 'Missing',
            'paths': paths, 'stdout': ''.join(p + '\n' for p in paths)}


def approvals(args, value, raw):
    require(args.approved_control == digest(raw), 'Exact independent control approval required')
    for name in ('bash', 'which'):
        require(getattr(args, 'approved_' + name) == value['programs'][name]['sha256'],
                'Exact reviewed ' + name + ' ELF approval required')


def retain_fd(path, fd, expected, cap):
    """Keep the exact held executable, without reopening its pathname or executing the copy."""
    size=os.fstat(fd).st_size
    require(size==expected['bytes'] and 0<size<=cap,'Bound held executable retention')
    h=hashlib.sha256();at=0
    with path.open('xb') as stream:
        while at<size:
            block=os.pread(fd,min(1024*1024,size-at),at)
            require(block,'Held executable shrank')
            stream.write(block);h.update(block);at+=len(block)
        stream.flush();os.fsync(stream.fileno())
    require(h.hexdigest()==expected['sha256'],'Retained held executable differs')


def retain_inputs(run, value):
    folder=fresh(run,'reviewed-inputs')
    entries={r['path']:{k:r[k] for k in ('bytes','sha256')} for r in value['compiled_inputs']}
    entries.update(value['tools'])
    for name,expected in entries.items():
        data=read_bytes(ROOT/name,4*1024*1024)
        require({'bytes':len(data),'sha256':digest(data)}==expected,'Source differs while retaining: '+name)
        dest=folder/name;dest.parent.mkdir(parents=True,exist_ok=True)
        with dest.open('xb') as stream:
            stream.write(data);stream.flush();os.fsync(stream.fileno())


def observe(args, base, value, raw, lock):
    approvals(args, value, raw)
    require(os.geteuid() != 0, 'Owned permission comparisons require an ordinary nonroot user')
    run = fresh(base, 'observed')
    retain_inputs(run,value)
    binary_folder=fresh(run,'executables')
    budget = Budget()
    rows = []
    started = time.monotonic()
    with ExitStack() as stack:
        held = {n: stack.enter_context(pinned_executable(Path(v['path']), v, 16 * 1024 * 1024))
                for n, v in value['programs'].items()}
        for name,(_,fd) in held.items():
            retain_fd(binary_folder/name,fd,value['programs'][name],16*1024*1024)
        for case in fixture():
            for mode in ('Which', 'TypeAllPaths'):
                require(time.monotonic() - started < 90, 'Whole reference deadline')
                folder = fresh(run, case['id'] + '-' + mode)
                with ExitStack() as files:
                    work, handles = materialize(folder, case, files)
                    before = snapshot(handles)
                    env = generated_environment(work, case)
                    source = ('hash -p "$2" "$1"; type -a -p "$1"' if case['preload_hash']
                              else 'type -a -p "$1"')
                    command = ([held['which'][0], 'report'] if mode == 'Which' else
                               [held['bash'][0], '--noprofile', '--norc', '-c', source,
                                'fixed-location-query', 'report', str(work / 'fallback/report')])
                    output(folder / 'launch.json', {'command': command, 'environment': env,
                           'case': case, 'before': before, 'mode': mode})
                    capture = fresh(folder, 'capture')
                    actual = capture_command(command, capture, work / 'work', env, b'', budget,
                                             32768, timeout=3, inherited=(lock, held['bash'][1], held['which'][1]),
                                             popen=lambda *a, **k: subprocess.Popen(*a, preexec_fn=reference_limits, **k))
                    output(folder / 'actual.json', actual)
                    after = snapshot(handles)
                    require(before == after, 'Read-only query changed fixture modes/bytes')
                    observed = normalized(actual, work)
                    rows.append({'id': case['id'], 'mode': mode, 'actual': actual,
                                 'normalized': observed, 'before': before, 'after': after})
    require(control() == value, 'Source or executable changed during observation')
    output(run / 'OBSERVATIONS.json', {'control_sha256': digest(raw), 'observations': rows,
           'limits': 'Only owned UID cases; not foreign UID/group, aliases, full type/which, whereis or artifact execution'})
    print(json.dumps({'status': 'ACTUAL-UNSEALED', 'output': str(run), 'queries': len(rows),
                      'observations_sha256': pin(run / 'OBSERVATIONS.json', 4 * 1024 * 1024)['sha256']}))


def compare(args, base, value, raw, lock):
    observations = read_bytes(base / 'observed/OBSERVATIONS.json', 4 * 1024 * 1024)
    require(args.approved_observation == digest(observations), 'Independent actual raw-readback approval required')
    original = strict(observations)
    require(original['control_sha256'] == digest(raw) and len(original['observations']) == 18,
            'Complete observed fixed table and exact control')
    probe = ROOT / PROBE
    identity = {'path': str(probe), **pin(probe, 1024 * 1024 * 1024)}
    require(args.approved_probe == identity['sha256'], 'Exact reviewed compiled probe approval required')
    run = fresh(base, 'compared')
    with pinned_executable(probe, identity, 1024 * 1024 * 1024) as held:
        retain_fd(run/'retained-probe',held[1],identity,1024*1024*1024)
        actual = capture_command([held[0]], run, run, closed_environment({}, run), b'',
                                 Budget(), 512 * 1024, timeout=10, inherited=(lock, held[1]),
                                 popen=lambda *a, **k: subprocess.Popen(*a, preexec_fn=reference_limits, **k))
    output(run / 'PROBE.json', actual)
    require(actual['status'] == 0 and not actual['stderr'], 'Fixed probe failed; diagnostics retained')
    probe_result = strict(actual['stdout'])
    require(probe_result['purpose'] == value['purpose'] and probe_result['compiled_manifest_dir'] == str(ROOT)
            and probe_result['fixture_sha256'] == FIXTURE_SHA
            and probe_result['compiled_inputs'] == value['compiled_inputs'], 'Actual compiled source/fixture closure differs')
    rows = probe_result['observations']
    require(len(rows) == 18, 'Exact actual model observation count')
    comparisons = []
    for host, model in zip(original['observations'], rows):
        require((host['id'], host['mode']) == (model['id'], model['mode']), 'Ordered host/model identity mismatch')
        expected = {k: model[k] for k in ('status', 'paths', 'stdout')}
        comparisons.append({'id': host['id'], 'mode': host['mode'], 'host': host['normalized'],
                            'model': expected, 'passed': host['normalized'] == expected})
    output(run / 'COMPARISON.json', {'comparisons': comparisons, 'probe': identity,
                                   'passed': all(r['passed'] for r in comparisons)})
    require(all(r['passed'] for r in comparisons), 'Actual comparison mismatch; retained, not accepted')
    print(json.dumps({'status': 'FIXED-COMPARISON-PASS', 'queries': len(comparisons), 'output': str(run)}))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=('prepare', 'observe', 'compare'), required=True)
    for name in ('control', 'bash', 'which', 'observation', 'probe'):
        parser.add_argument('--approved-' + name)
    args = parser.parse_args()
    lock = inherited_lock()
    require((ROOT / '.git').is_dir() and not (ROOT / '.git').is_symlink()
            and Path(os.environ.get('CARGO_TARGET_DIR', '')) == ROOT / 'target', 'MAIN/default-target shared runner only')
    parent = ROOT / '.build-artifacts/reference-runs'
    require(parent.resolve() == parent and parent.is_dir(), 'Existing project-disk reference root')
    base = ROOT / BASE
    if args.mode == 'prepare':
        value = control()
        base.mkdir(mode=0o700)
        output(base / 'CONTROL.json', value)
        print(json.dumps({'status': 'PROPOSED-NO-EXECUTION', 'control': pin(base / 'CONTROL.json', 4 * 1024 * 1024)}))
        return
    raw = read_bytes(base / 'CONTROL.json', 4 * 1024 * 1024)
    value = strict(raw)
    require(value == control(), 'Prepared source/executable control changed; nothing launched')
    if args.mode == 'observe':
        observe(args, base, value, raw, lock)
    else:
        compare(args, base, value, raw, lock)


if __name__ == '__main__':
    main()
