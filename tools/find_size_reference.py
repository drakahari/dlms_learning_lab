#!/usr/bin/env python3
"""Thirty-one fixed read-only developer queries. No learner command, script or path selector.

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
from generate_find_size_bindings import names
from native_reopen import inherited_lock, pin, read_bytes
from startup_reference import Budget, fresh, output, strict

ROOT = Path(__file__).resolve().parents[1]
BASE = '.build-artifacts/reference-runs/file-size-r1'
FIXTURE = 'tests/fixtures/find-size-reference-v1.json'
FIXTURE_SHA = 'f8e20440c75457685c0315bf5e6a33ddbcc040ad30ea79bb26e24b0e1b708dd1'
PROBE = 'target/debug/examples/find_size_reference'
TOOLS = ('tools/find_size_reference.py', 'tools/generate_find_size_bindings.py',
         'tools/tests/test_find_size_reference.py', 'tools/recursive_tree_reference.py', 'tools/dev.py', 'tools/build_safety.py',
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
    require(set(value) == {'schema_version', 'files', 'cases', 'excluded'}
            and value['schema_version'] == 1 and len(value['files']) == 10
            and len(value['cases']) == 31 and len(value['excluded']) == 2, 'Fixed roster')
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
    for n in ('find',):
        p = Path('/usr/bin') / n
        s = p.lstat()
        require(stat.S_ISREG(s.st_mode) and stat.S_IMODE(s.st_mode) == 0o755,
                'Original ordinary0755 GNU executable only')
        programs[n] = {'path': str(p), **pin(p, 16 * 1024 * 1024)}
    return dict(purpose='fixed-file-size-v1', compiled_inputs=inputs,
                tools={p: pin(ROOT / p, 4 * 1024 * 1024) for p in TOOLS},
                programs=programs, probe={'path': str(ROOT / PROBE), **pin(ROOT / PROBE, 1024**3)})


def validate_bindings():
    text = read_bytes(ROOT / 'examples/support/find_size_bindings.rs', 256 * 1024).decode()
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


def make_files(work, f):
    no_acl(work)
    for name,text in f['files'].items():
        require(re.fullmatch('[a-z0-9]{1,16}',name) and isinstance(text,str) and len(text.encode())<=8192,'Bounded fixed file')
        p=work/name
        fd=os.open(p,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600)
        try:
            raw=text.encode();require(os.write(fd,raw)==len(raw),'Complete inert write')
        finally:os.close(fd)
        no_acl(p)


def validate_query(args):
    require(isinstance(args,list) and 1<=len(args)<=12 and args[0]=='.','One relative fixed root')
    i=1;file_test=False
    while i<len(args):
        flag=args[i]
        if flag=='-print' and i==len(args)-1:break
        require(i+1<len(args),'Complete fixed predicate')
        value=args[i+1]
        if flag=='-type':require(value=='f','Ordinary files only');file_test=True
        elif flag=='-size':require(re.fullmatch(r'[+-]?[0-9]{1,20}[bckwMG]?',value) is not None,'Fixed decimal size')
        elif flag=='-maxdepth':require(value=='1' or value=='0','Fixed bounded depth')
        elif flag=='-name':require(value=='n*','One fixed literal pattern')
        else:raise ValueError('Unsupported reference predicate; never execute unknown flags')
        i+=2
    require(file_test,'No unmodeled directory-size comparison')


def path_members(text,f):
    require(isinstance(text,str) and len(text.encode())<=32768,'Bounded real path output')
    rows=[]
    for line in text.splitlines(keepends=True):
        require(line.endswith('\n') and line.startswith('./') and line[2:-1] in f['files'],'Exact finite relative result')
        rows.append(line[:-1])
    require(len(rows)==len(set(rows)),'No duplicate observations')
    return sorted(rows)


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
    for case in f['cases']:validate_query(case['args'])
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
        require([r['id'] for r in model['rows']] == [r['id'] for r in f['cases']], 'Ordered31 outcomes')
        rows = []
        version_dir=fresh(run,'version')
        version=capture([held['find'][0],'--version'],version_dir,version_dir,closed_environment({},version_dir),budget,(lock,held['find'][1]),0o022,8192,deadline)
        output(version_dir/'actual.json',version)
        require(version['status']==0 and not version['stderr'] and version['stdout'].startswith('find (GNU findutils) '),'Actual held GNU version')
        for case, m in zip(f['cases'], model['rows'], strict=True):
            folder = fresh(run, case['id'])
            work = fresh(folder, 'files')
            identity = work.stat()
            make_files(work, f)
            before=snapshot(work)
            output(folder / 'before.json', before)
            exe, fd = held['find']
            argv = [exe, *case['args']]
            env = closed_environment({}, work)
            output(folder / 'launch.json', dict(argv=argv, env=env))
            actual = capture(argv, folder, work, env, budget, (lock, fd), 0o022, 32768, deadline)
            output(folder / 'actual.json', actual)
            require((work.stat().st_dev, work.stat().st_ino) == (identity.st_dev, identity.st_ino),
                    'Original private fixture root unchanged')
            after = snapshot(work)
            output(folder / 'after.json', after)
            require(before==after,'Read-only GNU query altered fixture data or modes')
            model_members=path_members(m['stdout'],f) if m['ok'] is True else None
            host_members=path_members(actual['stdout'],f)
            rows.append(dict(id=case['id'],exact=(actual['status']==0 and actual['stderr']=='' and m['ok'] is True and host_members==model_members),actual=actual,model=m,host_members=host_members,model_members=model_members))
    require(control() == c, 'Source/executables changed during comparison')
    output(run / 'COMPARISON.json', dict(control_sha256=sha(raw), rows=rows, excluded=model['excluded'],
           limits='Thirty-one ordinary GNU size queries only; model exclusions are not GNU comparisons. '
                  'No directory sizes, allocated blocks/sparse files, time predicates, find mutations, links, privileges or full Findutils claim. '
                  'Root-reviewed fixed source; not an independently reviewed historical seal.'))
    require(all(r['exact'] for r in rows), 'Actual mismatch retained; no equivalence credit')
    print('Thirty-one fixed GNU/model path-membership comparisons match; two model-only exclusions.')


if __name__ == '__main__':
    main()
