#!/usr/bin/env python3
"""Eight fixed developer fixtures. No learner command, script or path selector.

Preparation launches nothing; observation requires exact source/executable pins.
This root-reviewed comparison is not an independently reviewed historical seal.
"""
import argparse
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import time

from automation_before10_oracle import pinned_executable
from case_bash_oracle import capture_command, closed_environment
from file_index_reference import ChildGuard, load_prctl, verify_proof, retain_fd
from generate_recursive_tree_bindings import names
from native_reopen import inherited_lock, pin, read_bytes
from startup_reference import Budget, fresh, output, strict

ROOT = Path(__file__).resolve().parents[1]
BASE = '.build-artifacts/reference-runs/recursive-tree-r1'
FIXTURE = 'tests/fixtures/recursive-tree-reference-v1.json'
FIXTURE_SHA = '8659860e6ba444d62c7dde04d07dea5cb2d65300689166b00450672aa05694d5'
PROBE = 'target/debug/examples/recursive_tree_reference'
TOOLS = ('tools/recursive_tree_reference.py', 'tools/generate_recursive_tree_bindings.py',
         'tools/tests/test_recursive_tree_reference.py', 'tools/dev.py', 'tools/build_safety.py',
         'tools/native_reopen.py', 'tools/automation_before10_oracle.py',
         'tools/case_bash_oracle.py', 'tools/file_index_reference.py', 'tools/startup_reference.py')


def require(ok, message):
    if not ok:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def fixture():
    raw = read_bytes(ROOT / FIXTURE, 32768)
    require(sha(raw) == FIXTURE_SHA, 'Fixed reviewed fixture changed')
    value = strict(raw)
    require(set(value) == {'schema_version', 'entries', 'cases', 'excluded'}
            and value['schema_version'] == 1 and len(value['entries']) == 10
            and len(value['cases']) == 8 and len(value['excluded']) == 3, 'Fixed roster')
    return value


def safe_relative(p):
    return (isinstance(p, str) and re.fullmatch(r'[A-Za-z0-9 ._-]{1,80}(?:/[A-Za-z0-9 ._-]{1,80}){0,5}', p)
            and all(c not in ('.', '..') for c in p.split('/')))


def control():
    fixture()
    # Source advances must refuse before launching even the model probe.
    validate_bindings()
    inputs = [{'path': p, **pin(ROOT / p, 4 * 1024 * 1024)} for p in names()]
    programs = {}
    for n in ('cp', 'rm'):
        p = Path('/usr/bin') / n
        s = p.lstat()
        require(stat.S_ISREG(s.st_mode) and stat.S_IMODE(s.st_mode) == 0o755,
                'Original ordinary0755 GNU executable only')
        programs[n] = {'path': str(p), **pin(p, 16 * 1024 * 1024)}
    return dict(purpose='fixed-recursive-tree-v1', compiled_inputs=inputs,
                tools={p: pin(ROOT / p, 4 * 1024 * 1024) for p in TOOLS},
                programs=programs, probe={'path': str(ROOT / PROBE), **pin(ROOT / PROBE, 1024**3)})


def validate_bindings():
    text = read_bytes(ROOT / 'examples/support/recursive_tree_bindings.rs', 256 * 1024).decode()
    rows = re.findall(r'\("([^"\n]+)",include_bytes!\("([^"\n]+)"\)\)', text)
    require(len(rows) == len(set(p for p, _ in rows)) and
            all(os.path.normpath('examples/support/' + rel) == p for p, rel in rows)
            and sorted(p for p, _ in rows) == names(), 'Complete current compile-time input bindings required')


def capture(command, folder, work, env, budget, inherited, mask, cap, deadline):
    """Restriction changes occur only after fork; Popen umask is child-only."""
    require(mask in (0o022, 0o077), 'Closed child mask')
    remaining = deadline - time.monotonic()
    require(remaining > 0, 'Whole90second deadline')
    parent = os.getpid()
    lib, prctl = load_prctl()
    reader, writer = os.pipe2(os.O_CLOEXEC | os.O_NONBLOCK)
    state = {'open': True, 'pid': None}
    guard = ChildGuard(writer, parent, prctl)

    def factory(*args, **kw):
        try:
            child = subprocess.Popen(*args, preexec_fn=guard, umask=mask, **kw)
            state['pid'] = child.pid
            return child
        finally:
            os.close(writer)
            state['open'] = False
    try:
        result = capture_command(command, folder, work, env, b'', budget, cap,
                                 timeout=min(3, remaining), inherited=(*inherited, writer), popen=factory)
        verify_proof(os.read(reader, 2), parent, state['pid'])
        output(folder / 'restriction.json', dict(child_pid=state['pid'], child_umask=mask,
               no_new_privs_child_proof=True, timeout_seconds=min(3, remaining)))
        return result
    finally:
        if state['open']:
            os.close(writer)
        os.close(reader)
        del lib


def no_acl(path):
    require(not any(n in os.listxattr(path, follow_symlinks=False)
                    for n in ('system.posix_acl_access', 'system.posix_acl_default')),
            'Fixture must not inherit POSIX ACLs; do not modify host settings')


def make_files(work, f, case):
    no_acl(work)
    for e in f['entries']:
        require(safe_relative(e['path']), 'Closed contained fixture path')
        p = work / e['path']
        if e['directory']:
            p.mkdir(mode=0o700)
        else:
            raw = e['text'].encode()
            fd = os.open(p, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
            try:
                require(os.write(fd, raw) == len(raw), 'Complete inert write')
            finally:
                os.close(fd)
        no_acl(p)
    # Restrict source directories only after their known children are populated.
    for e in reversed(f['entries']):
        os.chmod(work / e['path'], case['modes'].get(e['path'], e['mode']), follow_symlinks=False)


def snapshot(work):
    result = {}
    for p in work.rglob('*'):
        rel = p.relative_to(work).as_posix()
        s = p.lstat()
        require(safe_relative(rel) and len(result) < 64 and s.st_uid == os.geteuid()
                and (stat.S_ISDIR(s.st_mode) or (stat.S_ISREG(s.st_mode) and s.st_nlink == 1)),
                'Bounded owned ordinary fixture, no aliases/special files')
        no_acl(p)
        result[rel] = dict(directory=p.is_dir(), mode=stat.S_IMODE(s.st_mode),
                           caller_owned=s.st_uid == os.geteuid(), caller_group=s.st_gid == os.getegid(),
                           text=None if p.is_dir() else read_bytes(p, 8192).decode())
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--mode', choices=('prepare', 'compare'), required=True)
    parser.add_argument('--approved-control')
    args = parser.parse_args()
    lock = inherited_lock(ROOT)
    require((ROOT / '.git').is_dir() and Path(os.environ.get('CARGO_TARGET_DIR', '')) == ROOT / 'target'
            and os.geteuid() != 0, 'Ordinary MAIN/default-target runner only')
    c = control()
    base = ROOT / BASE
    if args.mode == 'prepare':
        require(base.parent.resolve() == base.parent, 'Canonical project disk')
        base.mkdir(mode=0o700)
        output(base / 'CONTROL.json', c)
        print('Prepared only: no child executed.')
        return
    raw = read_bytes(base / 'CONTROL.json', 1024 * 1024)
    require(base.resolve() == base and strict(raw) == c and args.approved_control == sha(raw),
            'Exact source/executable control approval required before any child')
    run = fresh(base, 'compared')
    source = fresh(run, 'source')
    for p, expected in [(r['path'], r) for r in c['compiled_inputs']] + list(c['tools'].items()):
        data = read_bytes(ROOT / p, 4 * 1024 * 1024)
        require(sha(data) == expected['sha256'] and len(data) == expected['bytes'], 'Retained source identity')
        dest = source / p
        dest.parent.mkdir(parents=True, exist_ok=True)
        with dest.open('xb') as out:
            out.write(data)
    f = fixture()
    budget = Budget()
    deadline = time.monotonic() + 90
    with ExitStack() as stack:
        held = {}
        for n, meta in {**c['programs'], 'probe': c['probe']}.items():
            held[n] = stack.enter_context(pinned_executable(Path(meta['path']), meta, 1024**3))
            retain_fd(run / ('retained-' + n), held[n][1], meta)
        probe_dir = fresh(run, 'probe')
        a = capture([held['probe'][0]], probe_dir, probe_dir, closed_environment({}, probe_dir), budget,
                    (lock, held['probe'][1]), 0o022, 1024 * 1024, deadline)
        output(probe_dir / 'actual.json', a)
        require(a['status'] == 0 and not a['stderr'], 'Actual probe failed; evidence retained')
        model = strict(a['stdout'])
        require(model['purpose'] == c['purpose'] and model['compiled_inputs'] == c['compiled_inputs'],
                'Actual compiled source differs from current approved inputs')
        require([r['id'] for r in model['rows']] == [r['id'] for r in f['cases']], 'Ordered8 outcomes')
        rows = []
        for case, m in zip(f['cases'], model['rows'], strict=True):
            folder = fresh(run, case['id'])
            work = fresh(folder, 'files')
            identity = work.stat()
            make_files(work, f, case)
            output(folder / 'before.json', snapshot(work))
            require(case['program'] in ('cp', 'rm') and len(case['args']) <= 4
                    and all(safe_relative(s) for s in case['args']), 'Only pinned literal operands/options')
            exe, fd = held[case['program']]
            argv = [exe, *case['args']]
            env = closed_environment({}, work)
            output(folder / 'launch.json', dict(argv=argv, env=env, mask=case['mask']))
            actual = capture(argv, folder, work, env, budget, (lock, fd), case['mask'], 32768, deadline)
            output(folder / 'actual.json', actual)
            require((work.stat().st_dev, work.stat().st_ino) == (identity.st_dev, identity.st_ino),
                    'Original private fixture root unchanged')
            after = snapshot(work)
            output(folder / 'after.json', after)
            rows.append(dict(id=case['id'], exact=(actual['status'] == 0 and actual['stdout'] == ''
                        and actual['stderr'] == '' and m['ok'] is True and after == m['view']),
                        actual=actual, model=m))
    require(control() == c, 'Source/executables changed during comparison')
    output(run / 'COMPARISON.json', dict(control_sha256=sha(raw), rows=rows, excluded=model['excluded'],
           limits='Eight ordinary same-UID/GID GNU cases only; model exclusions are not GNU comparisons. '
                  'No links, ACLs, preservation flags, partial-error equivalence or full coreutils claim. '
                  'Root-reviewed fixed source; not an independently reviewed historical seal.'))
    require(all(r['exact'] for r in rows), 'Actual mismatch retained; no equivalence credit')
    print('Eight fixed GNU/model state comparisons match; three explicit model-only exclusions.')


if __name__ == '__main__':
    main()
