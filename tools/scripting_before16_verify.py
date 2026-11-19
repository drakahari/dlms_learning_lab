#!/usr/bin/env python3
"""Fixed genuine15 development evidence. No script, profile or engine selector.

Prepare launches nothing. Capture requires an independently reviewed exact control.
Verify reads retained evidence only; it never replays an old evaluator.
"""
import argparse
from collections import Counter
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess

from native_reopen import inherited_lock, read_bytes, ordinary, pin as file_pin
from automation_before10_oracle import pinned_executable

ROOT = Path(__file__).resolve().parents[1]
DESTINATION = '.build-artifacts/reviews/bash-tracing/captures/public15-255-549-r1'
REVIEW = '.build-artifacts/reviews/bash-tracing/before16-control.json'
APPROVAL = '.build-artifacts/reviews/bash-tracing/before16-approved.json'
ATTESTATION = '.build-artifacts/reviews/bash-tracing/before16-attestation.json'
RETAINED = '.build-artifacts/reviews/bash-tracing/retained15'
BINARY = 'target/debug/examples/scripting_before16_capture'
BINARY_CAP = 1024 * 1024 * 1024
INPUTS = ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'tools/dev.py',
          'examples/scripting_before16_capture.rs', 'examples/support/before16_inputs.rs',
          'tests/fixtures/bash-before16-source-index.json', 'tools/scripting_before16_verify.py',
          'tools/tests/test_scripting_before16_verify.py', 'tools/native_reopen.py',
          'tools/automation_before10_oracle.py', 'tools/case_bash_oracle.py']
SEALED_ATTESTATION = 'deef62466011be6685dd68088835c56f37b22e9845cde00a0d8c578cc25236ac'  # Independently read back actual15; never replay after advance.


def require(condition, message):
    if not condition:
        raise ValueError(message)


def strict(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'Duplicate JSON key')
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs)


def contained(root, name):
    require(isinstance(name, str) and name and not name.startswith('/')
            and all(p not in ['', '.', '..'] for p in name.split('/')), 'Closed relative path required')
    at = root
    for part in name.split('/'):
        at = at / part
        require(not at.is_symlink(), 'Symlink evidence path refused')
    return at


def pin(root, name, cap=16_000_000):
    path = contained(root, name)
    info = path.lstat()
    cargo_links = name == BINARY and info.st_nlink == 2
    if cargo_links:
        siblings = [
            p for p in path.parent.iterdir() if re.fullmatch(re.escape(path.name) + r'-[0-9a-f]{16}', p.name)
            and p.lstat().st_dev == info.st_dev and p.lstat().st_ino == info.st_ino]
        require(len(siblings) == 1 and not siblings[0].is_symlink(), 'Unexpected Cargo executable alias')
    require(stat.S_ISREG(info.st_mode) and (info.st_nlink == 1 or cargo_links) and info.st_size <= cap, 'Evidence file type/size refused')
    observed = file_pin(path, cap)
    return {'path': name, 'size': observed['bytes'], 'sha256': observed['sha256']}


def current15(root):
    raw = read_bytes(root / 'src/shell.rs', 512 * 1024)
    require(re.findall(rb'^pub const EVALUATOR_REVISION: u32 = ([0-9]+);$', raw, re.M) == [b'15'], 'Capture requires genuine current15')


def control(root):
    current15(root)
    inputs = list(INPUTS)
    text = (root / 'examples/support/before16_inputs.rs').read_text()
    names = re.findall(r'^\s*\("([^"\n]+)",\s*include_bytes!', text, re.M)
    require(len(names) == 493 and len(set(names)) == 493, 'Fixed compiled closure changed')
    inputs += names
    rows = [pin(root, name) for name in sorted(set(inputs))]
    return {'actual_evaluator_revision': 15, 'root_reviewed': False,
            'pins': rows, 'producer': pin(root, BINARY, BINARY_CAP),
            'destination': DESTINATION}


def approved(raw, expected):
    actual = strict(raw)
    require(actual.get('root_reviewed') is True, 'Independent exact-pin review required')
    require(actual == {**expected, 'root_reviewed': True}, 'Review does not match actual source/producer')


def outcome(case, observed):
    return (set(observed) == {'stdout', 'stderr', 'status', 'files', 'error'}
            and type(observed['status']) is int and observed['error'] is None
            and observed['stdout'] == case['stdout'] and observed['stderr'] == case['stderr']
            and observed['status'] == case['status'] and observed['files'] == case['after'])


def records(profile):
    result = Counter()
    for a in profile['attempts'].values():
        state, workspace = a['state'], a.get('workspace') or {}
        items = ([state['run']] if state.get('kind') == 'Script' and state.get('run') else [])
        items += ([workspace['script_history']] if workspace.get('script_history') else [])
        items += workspace.get('script_history_more', [])
        result.update(json.dumps(x, sort_keys=True, separators=(',', ':')) for x in items)
    return result


def profile_checks(old, new, rows, drafts):
    require(new['favorites'] == old['favorites'], 'Favorites changed')
    expected_records = records(old)
    expected_ids = set(old['attempts'])
    for row in rows:
        name = row['id']; before = old['attempts'].get(name)
        expected_ids.add(name)
        if drafts and name in ['shell-command-plan', 'shell-handover']:
            require(before is not None and new['attempts'][name] == before, 'Unfinished draft changed')
            continue
        actual = new['attempts'][name]
        require(actual['state'] == dict(kind='Script', source=row['source'], run=row['record']), 'Current reference differs')
        learning = actual['workspace']['script_learning']
        require(learning['example_source'] == row['example_source'] and learning['example_revision'] == 15
                and learning['example_result'] == row['example_outcome'], 'Example context differs')
        expected_records[json.dumps(row['record'], sort_keys=True, separators=(',', ':'))] += 1
        if before is not None:
            def metadata(a):
                a = copy.deepcopy(a); a.pop('state')
                w = a.get('workspace') or {}
                for key in ['script_history', 'script_history_more', 'checked']:
                    w.pop(key, None)
                if w.get('script_learning'):
                    w['script_learning'].pop('example_revision', None)
                    w['script_learning'].pop('example_result', None)
                return a
            require(metadata(actual) == metadata(before), 'Learner metadata changed')
    require(set(new['attempts']) == expected_ids and len(expected_ids) == 156, 'Invented or missing attempt')
    for name, before in old['attempts'].items():
        if before['state']['kind'] != 'Script':
            require(new['attempts'][name] == before, 'Unrelated progress changed')
    require(records(new) == expected_records, 'Historical records dropped/relabeled/invented')


def inventory(root):
    folder = contained(root, DESTINATION)
    files = sorted(p for p in folder.rglob('*') if p.is_file())
    require(len(files) == 13, 'Incomplete/unexpected fixed capture inventory')
    total = 0
    pins = []
    for path in files:
        row = pin(root, path.relative_to(root).as_posix())
        total += row['size']; require(total <= 256 * 1024 * 1024, 'Capture total limit')
        pins.append(row)
    return pins


def validate(root):
    folder = contained(root, DESTINATION)
    manifest_raw = read_bytes(folder / 'MANIFEST.json', 16_000_000)
    manifest = strict(manifest_raw)
    require(read_bytes(folder / 'COMPLETE', 128).decode() == hashlib.sha256(manifest_raw).hexdigest() + '\n', 'Incomplete capture')
    require({k: manifest[k] for k in ['actual_evaluator_revision', 'catalog_contexts', 'script_records', 'authored_cases', 'examples', 'observations']}
            == dict(actual_evaluator_revision=15, catalog_contexts=255, script_records=66, authored_cases=549, examples=66, observations=615), 'Capture roster changed')
    rows = [strict(x) for x in read_bytes(folder / 'actual15-authored.jsonl', 16_000_000).splitlines()]
    obs = [strict(x) for x in read_bytes(folder / 'VM-OBSERVATIONS.jsonl', 16_000_000).splitlines()]
    require(len(rows) == 66 and len(obs) == 615, 'Missing actual VM observations')
    index = strict(read_bytes(contained(root, RETAINED + '/tests/fixtures/bash-before16-source-index.json'), 16_000_000))
    source = strict(read_bytes(folder / 'SOURCE.json', 16_000_000))
    require(source['actual_evaluator_revision'] == 15, 'Compiled revision differs')
    index_pin = pin(root, RETAINED + '/tests/fixtures/bash-before16-source-index.json')
    require(source['index_sha256'] == index_pin['sha256'], 'Compiled index differs')
    compiled_names = re.findall(r'^\s*\("([^"\n]+)",\s*include_bytes!',
                               contained(root, RETAINED + '/examples/support/before16_inputs.rs').read_text(), re.M)
    compiled_names += ['examples/scripting_before16_capture.rs', 'examples/support/before16_inputs.rs', 'tests/fixtures/bash-before16-source-index.json']
    require(len(compiled_names) == 496 and len(set(compiled_names)) == 496, 'Compiled binding inventory differs')
    expected_pins = []
    for name in compiled_names:
        row = pin(root, RETAINED + '/' + name)
        expected_pins.append({**row, 'path': name})
    require(source['inputs'] == expected_pins, 'Actual compiled source differs from retained reviewed source')
    require([r['id'] for r in rows] == [r['id'] for r in index['scripts']], 'Authored roster differs')
    cursor = 0
    for row in rows:
        expected = next(x for x in index['scripts'] if x['id'] == row['id'])
        require(all(row[k] == expected[k] for k in ['source', 'cases', 'example_source', 'example_case']), 'Declared fixture differs from compiled index')
        record = row['record']
        require(record['engine_revision'] == 15 and record['source'] == row['source'] and len(record['results']) == len(row['cases']), 'Record context differs')
        for n, (case, result) in enumerate(zip(row['cases'], record['results'])):
            require(outcome(case, result), 'Actual VM assignment failed')
            require(obs[cursor] == dict(id=row['id'], role='assignment', case=n, outcome=result), 'Observation order/content differs'); cursor += 1
        require(outcome(row['example_case'], row['example_outcome']), 'Actual VM example failed')
        require(obs[cursor] == dict(id=row['id'], role='example', case=0, outcome=row['example_outcome']), 'Example observation differs'); cursor += 1
    require(cursor == len(obs), 'Extra VM observations')
    for role, count in [('original', 66), ('draft', 64)]:
        p = manifest[role + '_profile']
        require(p['contexts'] == 156 and p['current_records'] == count and p['non_script_contexts_preserved'] == 90
                and p['all_prior_records_retained'] is True and len(p['added_script_ids']) == 6, 'Profile preservation summary differs')
        stem = 'drafts' if role == 'draft' else 'original'
        old = strict(read_bytes(folder / ('input14-' + stem + '.json'), 16_000_000))
        new = strict(read_bytes(folder / ('writer15-' + stem + '.json'), 16_000_000))
        require(hashlib.sha256(read_bytes(folder / ('input14-' + stem + '.json'), 16_000_000)).hexdigest()
                == index['prior']['writer14-' + stem + '.json'], 'Genuine prior profile differs')
        profile_checks(old, new, rows, role == 'draft')
    return inventory(root)


def create(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x') as f:
        json.dump(data, f, indent=2); f.write('\n'); f.flush(); os.fsync(f.fileno())


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--mode', choices=['prepare', 'capture', 'verify'], required=True)
    args = parser.parse_args()
    lock = inherited_lock()
    require((ROOT / '.git').is_dir() and Path(os.environ.get('CARGO_TARGET_DIR', '')) == ROOT / 'target', 'MAIN/default runner only')
    if args.mode == 'verify':
        require(SEALED_ATTESTATION is not None, 'Actual capture has not been independently sealed')
        raw = read_bytes(contained(ROOT, ATTESTATION), 16_000_000)
        require(hashlib.sha256(raw).hexdigest() == SEALED_ATTESTATION, 'Attestation changed')
        sealed = strict(raw)
        require(sealed['outputs'] == validate(ROOT), 'Retained capture bytes changed')
        require(sealed['producer'] == pin(ROOT, sealed['producer']['path'], BINARY_CAP), 'Retained producer changed')
        for row in sealed['retained_inputs']:
            require(row == pin(ROOT, row['path']), 'Retained compiled source/control changed')
        print('PASS retained genuine15 evidence; no replay'); return
    c = control(ROOT)
    if args.mode == 'prepare':
        create(contained(ROOT, REVIEW), c)
        print('Prepared exact genuine15 proposal; no VM executed; independent review required'); return
    approved(read_bytes(contained(ROOT, APPROVAL), 16_000_000), c)
    require(not contained(ROOT, DESTINATION).exists(), 'Preserve existing capture; no overwrite')
    retained = contained(ROOT, RETAINED)
    require(not retained.exists(), 'Preserve retained source and producer')
    retained.mkdir(mode=0o700)
    for row in [*c['pins'], c['producer']]:
        source = contained(ROOT, row['path'])
        dest = contained(retained, row['path']); dest.parent.mkdir(parents=True, exist_ok=True)
        with ordinary(source, BINARY_CAP) as (stream, info), dest.open('xb') as output:
            copied = 0
            while chunk := stream.read(1024 * 1024):
                copied += len(chunk); require(copied <= row['size'], 'Input grew during retention'); output.write(chunk)
            output.flush(); os.fsync(output.fileno())
        require(pin(retained, row['path'], BINARY_CAP) == row, 'Retained source/producer differs')
    env = dict(os.environ, DLMS_FIXED_CAPTURE15='before16-r1')
    expected = dict(path=str(contained(ROOT, BINARY)), bytes=c['producer']['size'], sha256=c['producer']['sha256'])
    with pinned_executable(contained(ROOT, BINARY), expected, BINARY_CAP) as (executable, descriptor):
        subprocess.run([executable], cwd=ROOT, env=env, check=True, timeout=240, pass_fds=(lock, descriptor))
    outputs = validate(ROOT)
    retained_producer = pin(ROOT, RETAINED + '/' + BINARY, BINARY_CAP)
    retained_inputs = [pin(ROOT, RETAINED + '/' + row['path']) for row in c['pins']]
    create(contained(ROOT, ATTESTATION), {'purpose': 'Actual genuine15 execution; independent sealing still required', 'control': c, 'outputs': outputs, 'producer': retained_producer, 'retained_inputs': retained_inputs})
    print('Actual capture validates; retain producer/source and obtain independent seal before semantics advance')


if __name__ == '__main__':
    main()
