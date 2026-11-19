#!/usr/bin/env python3
"""Thirty fixed builtin-only tracing comparisons; never learner source.

Observe executes only the reviewed static fixture in disposable project-disk
directories. Compare executes only the fixed in-process probe. No source/path CLI.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import resource
import stat
import subprocess
import tempfile
from native_reopen import inherited_lock, read_bytes, ordinary, pin
from automation_before10_oracle import pinned_executable

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = 'tests/fixtures/shell-tracing-reference.json'
FIXTURE_SHA = '7d165c919acf97f1afd85a0ca2156af32ed5b247bd7461c00857241f94c14a83'
FORMAT_FIXTURE = 'tests/fixtures/shell-tracing-format-reference.json'
FORMAT_SHA = 'b4d511186595da9838550c554ae806f85c9f87c27a34fcd26a75f3b79a6c7f5c'
EDGE_FIXTURE = 'tests/fixtures/shell-tracing-edge-reference.json'
EDGE_SHA = '98df423a3d5a7242d4ca761edd8fd6cac8f81d6dfa91f43d89cc68e492a0e0f9'
BASE = '.build-artifacts/reference-runs/shell-tracing'
PROBE = 'target/debug/examples/tracing_reference'
BASH_OBSERVATION_SHA = '74b165cd820820bd06cbe653cbca7da3042380a595225efb689f37b95e745fa8'  # Fill only after observing and independently reading the actual report.
FORMAT_OBSERVATION_SHA = 'dae6452bbae9b5bab6cd26c3df4ceeac91ba6f85659b0f6fea0d2f0cfa49ce0f'
EDGE_OBSERVATION_SHA = '330ecc46fc4d4f2d8c32aab12145870d300eee7211f6d1c736e7dab1d57f6e9c'


ENGINE_PATHS = ('Cargo.lock', 'Cargo.toml', 'examples/support/automation_digest.rs', 'examples/tracing_reference.rs', 'src/shell.rs', 'src/shell/arithmetic.rs', 'src/shell/capture.rs', 'src/shell/case.rs', 'src/shell/cron.rs', 'src/shell/execution.rs', 'src/shell/getopts.rs', 'src/shell/globbing.rs', 'src/shell/grep.rs', 'src/shell/literal_trim.rs', 'src/shell/loop_control.rs', 'src/shell/one_off.rs', 'src/shell/parameters.rs', 'src/shell/preparation.rs', 'src/shell/substitution.rs', 'src/shell/text_tools.rs', 'src/shell/tracing.rs', 'src/shell/values.rs', 'tests/fixtures/shell-tracing-edge-reference.json', 'tests/fixtures/shell-tracing-format-reference.json', 'tests/fixtures/shell-tracing-reference.json')

def require(ok, message):
    if not ok:
        raise ValueError(message)


def limits():
    resource.setrlimit(resource.RLIMIT_CPU, (2, 2))
    resource.setrlimit(resource.RLIMIT_AS, (256 * 1024 * 1024, 256 * 1024 * 1024))
    resource.setrlimit(resource.RLIMIT_FSIZE, (1024 * 1024, 1024 * 1024))
    resource.setrlimit(resource.RLIMIT_NOFILE, (32, 32))


def fixture():
    raw = read_bytes(ROOT / FIXTURE, 256 * 1024)
    require(hashlib.sha256(raw).hexdigest() == FIXTURE_SHA, 'Fixed recipes changed; review before repinning')
    data = json.loads(raw)
    require(data['schema_version'] == 1 and len(data['cases']) == 30, 'Closed tracing roster')
    for row in data['cases']:
        require(row['host_observation'] is None and row['args'] == [] and row['stdin'] == ''
                and row['env'] == {} and row['files'] == {}, 'Only reviewed static inputs')
        require(len(row['source']) < 1024, 'Bounded builtin-only recipe')
    return data['cases']


def formatting_fixture():
    raw = read_bytes(ROOT / FORMAT_FIXTURE, 256 * 1024)
    require(hashlib.sha256(raw).hexdigest() == FORMAT_SHA, 'Fixed formatting recipes changed')
    data = json.loads(raw)
    require(data['schema_version'] == 1 and [r['id'] for r in data['cases']]
            == ['ascii-punctuation', 'control-bytes', 'word-edges', 'assignment-quoting'], 'Closed formatting roster')
    for row in data['cases']:
        require(row['host_observation'] is None and row['args'] == [] and row['stdin'] == ''
                and row['env'] == {} and row['files'] == {} and len(row['source']) < 2048, 'Only fixed formatting recipes')
    return data['cases']


def edge_fixture():
    raw = read_bytes(ROOT / EDGE_FIXTURE, 256 * 1024)
    require(hashlib.sha256(raw).hexdigest() == EDGE_SHA, 'Fixed assignment/quote recipes changed')
    data = json.loads(raw)
    require(data['schema_version'] == 1 and [r['id'] for r in data['cases']]
            == ['empty-assignments', 'apostrophe-edges'], 'Closed edge roster')
    for row in data['cases']:
        require(row['host_observation'] is None and row['args'] == [] and row['stdin'] == ''
                and row['env'] == {} and row['files'] == {} and len(row['source']) < 1024, 'Only fixed edge recipes')
    return data['cases']


def fresh(root, name):
    require(not root.is_symlink() and root.resolve() == root, 'Canonical project-disk directory required')
    path = root / name; path.mkdir(mode=0o700)
    return path


def output(path, value):
    raw = (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()
    require(len(raw) <= 4 * 1024 * 1024, 'Comparison report limit')
    with path.open('xb') as f:
        f.write(raw); f.flush(); os.fsync(f.fileno())


def observation_rows(value, expected):
    require(isinstance(value, list) and len(expected) in [2, 4, 30, 36] and len(value) == len(expected), 'Exact reference/VM observation count required')
    require([r.get('id') for r in value] == [r['id'] for r in expected], 'Observation count/order differs')
    for row in value:
        require(set(row) == {'id', 'stdout', 'stderr', 'status', 'files', 'error'}
                and type(row['status']) is int and 0 <= row['status'] <= 255
                and row['error'] is None and isinstance(row['files'], dict)
                and isinstance(row['stdout'], str) and isinstance(row['stderr'], str), 'Incomplete/non-normal observation')


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--mode', choices=['observe', 'observe-format', 'observe-edges', 'compare'], required=True)
    args = parser.parse_args(); lock = inherited_lock()
    require((ROOT / '.git').is_dir() and Path(os.environ.get('CARGO_TARGET_DIR', '')) == ROOT / 'target', 'MAIN/default runner only')
    formatting = args.mode == 'observe-format'
    edges = args.mode == 'observe-edges'
    rows = edge_fixture() if edges else formatting_fixture() if formatting else fixture(); base = ROOT / BASE
    base.mkdir(parents=True, exist_ok=True)
    require(not base.is_symlink() and base.resolve() == base, 'No reference-root aliases')
    if args.mode in ['observe', 'observe-format', 'observe-edges']:
        run = fresh(base, 'edges-r1' if edges else 'format-r1' if formatting else 'bash-r1')
        bash = Path('/usr/bin/bash').resolve(strict=True)
        identity = pin(bash, 16 * 1024 * 1024)
        observations = []
        # Keep the exact observed recipes/collector and each actual row even if
        # a later fixed child fails. These files are diagnostics, never success.
        for name, source in [('observer.py', ROOT / 'tools/tracing_reference.py'), ('fixture.json', ROOT / (EDGE_FIXTURE if edges else FORMAT_FIXTURE if formatting else FIXTURE))]:
            with (run / name).open('xb') as saved:
                saved.write(read_bytes(source, 256 * 1024)); saved.flush(); os.fsync(saved.fileno())
        with (run / 'observations.jsonl').open('xb') as journal, pinned_executable(bash, dict(path=str(bash), **identity), 16 * 1024 * 1024) as (executable, fd):
            for n, row in enumerate(rows):
                work = fresh(run, str(n))
                env = dict(PATH='', LC_ALL='C', LANG='C', HOME=str(work), TMPDIR=str(work), PS4='+ ')
                try:
                    p = subprocess.run([executable, '--noprofile', '--norc', '-c', row['source'], 'fixed-tracing'],
                                       cwd=work, env=env, input=b'', capture_output=True, timeout=3,
                                       pass_fds=(lock, fd), preexec_fn=limits)
                except Exception as error:
                    output(run / 'owned-child-failure.json', dict(id=row['id'], diagnostic=str(error)[:1000], stdout=repr(getattr(error, 'stdout', b''))[:32768], stderr=repr(getattr(error, 'stderr', b''))[:32768]))
                    raise
                require(len(p.stdout) + len(p.stderr) <= 32768, 'Reference output limit')
                files = {}
                for path in work.iterdir():
                    require(stat.S_ISREG(path.lstat().st_mode) and path.stat().st_size <= 32768
                            and path.name not in ['.', '..'], 'Unexpected fixed reference artifact')
                    files['/work/' + path.name] = read_bytes(path, 32768).decode()
                observed = dict(id=row['id'], stdout=p.stdout.decode(), stderr=p.stderr.decode(), status=p.returncode, files=files, error=None)
                journal.write(json.dumps(observed, ensure_ascii=False).encode() + b'\n'); journal.flush(); os.fsync(journal.fileno())
                observations.append(observed)
        observation_rows(observations, rows)
        output(run / 'observations.json', dict(purpose='Actual fixed Bash tracing observations; no VM acceptance yet', bash=dict(path=str(bash), **identity), fixture_sha256=EDGE_SHA if edges else FORMAT_SHA if formatting else FIXTURE_SHA, observations=observations))
        print('Observed' + str(len(rows)) + 'fixed Bash recipes; VM comparison still pending'); return
    raw = read_bytes(base / 'bash-r1/observations.json', 4 * 1024 * 1024)
    require(BASH_OBSERVATION_SHA is not None and hashlib.sha256(raw).hexdigest() == BASH_OBSERVATION_SHA, 'Actual reference observation seal differs/pending')
    actual = json.loads(raw)
    require(actual['fixture_sha256'] == FIXTURE_SHA, 'Reference fixture context changed')
    observation_rows(actual['observations'], rows)
    format_raw = read_bytes(base / 'format-r1/observations.json', 4 * 1024 * 1024)
    require(FORMAT_OBSERVATION_SHA is not None and hashlib.sha256(format_raw).hexdigest() == FORMAT_OBSERVATION_SHA, 'Actual formatting seal differs/pending')
    format_actual = json.loads(format_raw); format_rows = formatting_fixture()
    require(format_actual['fixture_sha256'] == FORMAT_SHA, 'Formatting fixture context changed')
    observation_rows(format_actual['observations'], format_rows)
    rows += format_rows; actual['observations'] += format_actual['observations']
    edge_raw = read_bytes(base / 'edges-r1/observations.json', 4 * 1024 * 1024)
    require(EDGE_OBSERVATION_SHA is not None and hashlib.sha256(edge_raw).hexdigest() == EDGE_OBSERVATION_SHA, 'Actual edge seal differs/pending')
    edge_actual = json.loads(edge_raw); edge_rows = edge_fixture()
    require(edge_actual['fixture_sha256'] == EDGE_SHA, 'Edge fixture context changed')
    observation_rows(edge_actual['observations'], edge_rows)
    rows += edge_rows; actual['observations'] += edge_actual['observations']
    run = Path(tempfile.mkdtemp(prefix='compare-', dir=base))
    expected = [dict(path=name, **pin(ROOT/name, 256*1024)) for name in ENGINE_PATHS]
    retained = run / 'compiled-source'; retained.mkdir(mode=0o700)
    for row in expected:
        target = retained / row['path']; target.parent.mkdir(parents=True, exist_ok=True)
        with target.open('xb') as saved:
            saved.write(read_bytes(ROOT / row['path'], 256*1024)); saved.flush(); os.fsync(saved.fileno())
        require(pin(target, 256*1024) == {k: row[k] for k in ['bytes', 'sha256']}, 'Retained compare source changed')
    probe = ROOT / PROBE; identity = pin(probe, 1024 * 1024 * 1024)
    try:
        with pinned_executable(probe, dict(path=str(probe), **identity), 1024 * 1024 * 1024) as (executable, fd):
            # Retain exactly the descriptor that will execute, before advancing
            # source. Stream its debug sections rather than loading the ELF.
            with (run / 'probe-elf').open('xb') as saved:
                at = 0
                while at < identity['bytes']:
                    block = os.pread(fd, min(1024*1024, identity['bytes']-at), at)
                    require(bool(block), 'Reviewed probe shrank during retention')
                    saved.write(block); at += len(block)
                saved.flush(); os.fsync(saved.fileno())
            require(pin(run / 'probe-elf', 1024*1024*1024) == identity, 'Retained executed probe differs')
            p = subprocess.run([executable], cwd=ROOT, env=os.environ.copy(), capture_output=True, timeout=60, pass_fds=(lock, fd))
    except Exception as error:
        output(run / 'owned-probe-failure.json', dict(diagnostic=str(error)[:1000], stdout=repr(getattr(error, 'stdout', b''))[:32768], stderr=repr(getattr(error, 'stderr', b''))[:32768]))
        raise
    output(run / 'execution.json', dict(returncode=p.returncode, stdout=p.stdout[:4*1024*1024].decode(errors='replace'), stderr=p.stderr[:32768].decode(errors='replace')))
    require(p.returncode == 0 and not p.stderr and len(p.stdout) <= 4 * 1024 * 1024, 'Probe failed/output limit')
    vm = json.loads(p.stdout)
    require(vm['fixture_sha256'] == FIXTURE_SHA and vm['format_sha256'] == FORMAT_SHA and vm['edge_sha256'] == EDGE_SHA, 'Compiled probe fixture differs')
    require(vm['evaluator_revision'] == 16, 'Comparison requires actual evaluator16')
    require(vm['compiled_inputs'] == expected, 'Stale same-revision probe; compiled engine/source differs')
    observation_rows(vm['observations'], rows)
    failures = [r['id'] for r, reference in zip(vm['observations'], actual['observations']) if r != reference]
    report = dict(purpose='Fixed in-process VM versus observed Bash; not learner execution', fixture_sha256=FIXTURE_SHA,
                  probe=dict(path=str(probe), **identity), vm=vm, reference=actual, failures=failures)
    output(run / 'report.json', report)
    require(not failures, 'VM tracing mismatches retained: ' + ', '.join(failures))
    print('PASS36actual Bash versus VM tracing comparisons: ' + str(run))


if __name__ == '__main__':
    main()
