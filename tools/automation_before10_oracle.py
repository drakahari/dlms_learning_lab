#!/usr/bin/env python3
"""Fixed public9-before10/empty-IFS and pipefail oracle. MAIN guarded tool only; never learner input.

Preparation hashes metadata only; capture requires an independently reviewed
APPROVED-ID.json. No command/script/source/binary/profile path argument exists.
Existing43 authored inputs and the prior fixed tools and all original historical observations remain untouched.
"""
import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import time

import case_bash_oracle as safe

ROOT = Path(__file__).resolve().parents[1]
HERE = '.build-artifacts/reviews/bash-safe-automation'
FIXED = 'tests/fixtures/automation-oracle-inputs.json'
AUTHORED = 'tests/fixtures/automation-before10-authored-inputs.json'
PROBE = 'target/debug/examples/automation_before10_capture'
SOURCE = 'examples/automation_before10_capture.rs'
DIGEST = 'examples/support/automation_digest.rs'
ATTEST = 'examples/support/automation_attestation.rs'
FIXED_SHA = '4031cb6800a7b8b46d93d4b6edd1db594335e4ebbea18cec5be80b980c56c508'
VM_BOUNDARIES = 'tests/fixtures/automation-vm-boundaries.json'
VM_SHA = 'd21c2e5470fdb4d40f7701b35617db725285bd72de0c3235a02982f668aee7a4'
BINARY_CAP = 1024 * 1024 * 1024
SOURCE_CAP = 2 * 1024 * 1024
PROFILE_CAP = 16_000_000
PROFILE_TOTAL_CAP = 64 * 1024 * 1024
TOTAL_PIN_CAP = 1536 * 1024 * 1024
TOTAL_OUTPUT_CAP = 128 * 1024 * 1024
VM_STREAM_CAP = 32 * 1024 * 1024
WHOLE_SECONDS = 300.0
VM_SECONDS = 180.0
PIN_PATHS = 'tests/fixtures/automation-before10-source-pins.json'


def require(ok, text):
    safe.require(ok, text)


def strict_json(data):
    def pairs(items):
        value = {}
        for key, item in items:
            require(key not in value, 'Duplicate JSON key')
            value[key] = item
        return value
    return json.loads(data, object_pairs_hook=pairs)


def capture_id(value):
    require(isinstance(value, str) and re.fullmatch(r'[A-Za-z0-9_-]{1,64}', value), 'Fixed simple capture ID required')
    return value


def load_fixed(root):
    raw = safe.read_bytes(root / FIXED, SOURCE_CAP)
    require(hashlib.sha256(raw).hexdigest() == FIXED_SHA, 'Reviewed fixed28 source changed')
    d = strict_json(raw)
    cases = d['cases']
    require(isinstance(cases, list) and len(cases) == 28, 'Exactly28 fixed sources required')
    seen = set()
    for row in cases:
        name = row['id']
        require(isinstance(name, str) and re.fullmatch(r'[a-z0-9-]{1,100}', name) and name not in seen, 'Fixed identity')
        seen.add(name)
        require(isinstance(row['source'], str) and 0 < len(row['source'].encode()) <= 4096 and '\0' not in row['source'], 'Fixed source bound')
        require(all(row[k] is None for k in ['actual_bash_5_3', 'actual_public9_before10', 'accepted_after10']), 'Input must never acquire fabricated observations')
        i = row['input']
        require(set(i) == {'args', 'stdin', 'env', 'files'}, 'Fixed input shape')
        require(len(i['args']) <= 16 and all(isinstance(v, str) and '\0' not in v and len(v.encode()) <= 256 for v in i['args']), 'Fixed args')
        require(isinstance(i['stdin'], str) and '\0' not in i['stdin'] and len(i['stdin'].encode()) <= 4096, 'Bounded fixed stdin')
        require(i['files'] == {'/work/keep.txt': 'retained\n'} or (name == 'read-redirect-capture-order' and i['files'] == {'/work/keep.txt': 'retained\n', '/work/input.txt': '  disk line  \n'}), 'Only reviewed disposable inputs')
        require(i['env'] == {}, 'No arbitrary environment')
        safe.closed_environment(i['env'], Path('/fixed-placeholder'))
    return raw, cases


def load_vm(root):
    raw = safe.read_bytes(root / VM_BOUNDARIES, SOURCE_CAP)
    require(hashlib.sha256(raw).hexdigest() == VM_SHA, 'Reviewed VM-only inputs changed')
    cases = strict_json(raw)['cases']
    require(isinstance(cases, list) and len(cases) == 8, 'Exactly8 VM-only inputs')
    seen = set()
    for r in cases:
        require(r['id'] not in seen and re.fullmatch(r'[a-z0-9-]{1,100}', r['id']), 'VM-only identity'); seen.add(r['id'])
        require(r['run_bash'] is False and r['actual_public9_before10'] is None, 'VM-only observations remain unrun')
        require(isinstance(r['source'], str) and 0 < len(r['source'].encode()) <= 16384 and '\0' not in r['source'], 'VM-only source bound')
        i = r['input']; require(set(i) == {'args','stdin','env','files'} and i['args'] == [] and i['stdin'] == '' and i['files'] == {'/work/keep.txt':'retained\n'}, 'Closed VM-only input')
        require(isinstance(i['env'], dict) and set(i['env']) <= {'pad','large'} and all(isinstance(v,str) and len(v.encode()) <= 100000 and '\0' not in v for v in i['env'].values()) and sum(len(v.encode()) for v in i['env'].values()) <= 230000, 'VM-only bounded stress input; never forwarded to Bash')
    return raw, cases


def expected_inputs(root):
    data = strict_json(safe.read_bytes(root / PIN_PATHS, SOURCE_CAP))
    require(set(data) == {'compiled_files', 'original_engine9', 'frozen_inputs'}, 'Source pin-list shape')
    paths = data['compiled_files']
    require(isinstance(paths, list) and 16 <= len(paths) <= 200 and len(set(paths)) == len(paths), 'Bounded unique source path list')
    for p in paths:
        require(isinstance(p, str) and not Path(p).is_absolute() and '..' not in Path(p).parts
                and (p.startswith(('src/', 'scenarios/', 'tests/fixtures/')) or p in ('Cargo.toml', 'Cargo.lock')), 'Compiled path outside closed source inputs')
    actual_rs = {p.relative_to(root).as_posix() for p in (root / 'src').rglob('*.rs')
                 if 'ui' not in p.relative_to(root).parts and p.name not in ('main.rs', 'ui.rs', 'startup_trace.rs')}
    require({n for n in paths if n.startswith('src/')} == actual_rs,
            'Library module set changed; regenerate/review compile attestation before capture')
    for collection in ['original_engine9', 'frozen_inputs']:
        require(isinstance(data[collection], dict) and 1 <= len(data[collection]) <= 32, 'Bounded original identity list')
        for name, expected in data[collection].items():
            require(name in paths and isinstance(expected, str) and re.fullmatch('[0-9a-f]{64}', expected), 'Original identity shape')
            require(safe.pin(root / name, SOURCE_CAP)['sha256'] == expected, 'Original public9/historical evidence changed: ' + name)
    return data


def source_set(root):
    data = expected_inputs(root)
    names = list(data['compiled_files']) + [PIN_PATHS, SOURCE, DIGEST, ATTEST,
            'tools/automation_before10_oracle.py', 'tools/tests/test_automation_before10_oracle.py', 'tools/case_bash_oracle.py', 'tools/dev.py', 'tools/build_safety.py',
            '.build-artifacts/toolchains/READY.json']
    require(len(names) == len(set(names)), 'Duplicate source pin')
    paths = [(root / n, SOURCE_CAP) for n in names]
    paths += [(root / PROBE, BINARY_CAP), (safe.MANUAL, 8 * SOURCE_CAP), (safe.BASH.resolve(strict=True), 256 * SOURCE_CAP)]
    return paths


def pins(paths):
    require(len(paths) <= 256, 'Pin count ceiling')
    total = 0
    # Check each same-FD size before hashing any large candidate; no source copies.
    for path, cap in paths:
        with safe.ordinary(path, cap) as (_, info):
            total += info.st_size
            require(total <= TOTAL_PIN_CAP, 'Aggregate source/binary pin ceiling')
    actual_total, result = 0, []
    for path, cap in paths:
        actual = safe.pin(path, cap)
        require(0 <= actual['bytes'] <= cap, 'Actual same-FD pin size ceiling')
        actual_total += actual['bytes']
        require(actual_total <= TOTAL_PIN_CAP, 'Actual aggregate source/binary pin ceiling')
        result.append(actual)
    return result


def fixed_root_guard(root):
    require(root == ROOT.resolve(strict=True) and (root / '.git').is_dir() and not (root / '.git').is_symlink(), 'Actual capture/preparation is MAIN checkout only')
    inherited = safe.require_guard(root)
    require(os.environ.get('CARGO_TARGET_DIR') == str(root / 'target'), 'Actual MAIN default target required')
    return inherited


def review_dir(root):
    # owned_directory verifies all directory components and the disk volume.
    from build_safety import owned_directory
    return owned_directory(root, root / HERE)


def prepare(root, identity):
    capture_id(identity)
    load_fixed(root); load_vm(root)
    sources = source_set(root)
    observed = pins(sources)
    require(next(x for x in observed if x['path'] == str(safe.MANUAL))['sha256'] == safe.MANUAL_SHA, 'Reviewed original manual changed')
    control = {'schema_version': 1, 'root_reviewed': False, 'capture_id': identity,
               'source_head': safe.source_identity(root), 'pins': observed,
               'claims': 'Metadata proposal only. No Bash/version/VM/outcome capture performed by prepare.'}
    path = review_dir(root) / ('PROPOSED-' + identity + '.json')
    pin = safe.json_new(path, control, safe.Budget(SOURCE_CAP), SOURCE_CAP)
    return {'status': 'PROPOSED-ONLY', 'path': str(path), 'sha256': pin['sha256']}


def approval(root, identity, sources):
    raw = safe.read_bytes(review_dir(root) / ('APPROVED-' + capture_id(identity) + '.json'), SOURCE_CAP)
    control = strict_json(raw)
    require(set(control) == {'schema_version', 'root_reviewed', 'capture_id', 'source_head', 'pins', 'claims'}, 'Closed approval fields')
    require(control['schema_version'] == 1 and control['root_reviewed'] is True and control['capture_id'] == identity, 'Actual independently reviewed control required')
    require(control['source_head'] == safe.source_identity(root), 'Reviewed actual MAIN source HEAD changed')
    actual = pins(sources)
    require(control['pins'] == actual, 'Reviewed source/binary/input/manual/Bash identities changed')
    return raw, control, actual


@contextmanager
def pinned_executable(path, expected, cap):
    # Hash, execute and final-check the SAME retained FD; no pathname reopen race.
    with safe.ordinary(path, cap) as (stream, info):
        digest = hashlib.sha256()
        require(stream.read(4) == b'\x7fELF', 'Fixed probe/reference executable must be ELF'); stream.seek(0)
        left = info.st_size
        while left:
            block = stream.read(min(left, 1024 * 1024))
            require(block, 'Executable shrank')
            digest.update(block)
            left -= len(block)
        require(not stream.read(1), 'Executable grew')
        require(expected == {'path': str(path), 'bytes': info.st_size, 'sha256': digest.hexdigest()}, 'Executable differs from reviewed pin')
        stream.seek(0)
        yield '/proc/self/fd/' + str(stream.fileno()), stream.fileno()


def vm_environment(root, temporary, profile):
    value = safe.closed_environment({}, temporary)
    value.update(CARGO_BUILD_JOBS='2', RUST_TEST_THREADS='2', CARGO_TARGET_DIR=str(root / 'target'),
                 DLMS_FIXED_AUTOMATION_CHILD='before10-public9-r1', DLMS_FIXED_AUTOMATION_OUTPUT=str(profile))
    return value


def authored_inputs(root):
    raw = safe.read_bytes(root / AUTHORED, SOURCE_CAP)
    authored = strict_json(raw)
    require(len(authored['catalog']) == 133 and len(authored['scripts']) == 43
            and sum(len(x['cases']) for x in authored['scripts']) == 282, 'Closed current authored scope')
    return raw, authored


def identity_rows(root, raw, before):
    lines = raw.splitlines()
    require(len(lines) == 1, 'Identity-only mode must produce exactly one row')
    value = strict_json(lines[0])
    require(value.get('kind') == 'identity' and value.get('revision') == 9 and value.get('compiled_manifest_dir') == str(root), 'Wrong linked revision/checkout')
    _, authored = authored_inputs(root)
    require(value.get('linked_catalog') == authored['catalog'] and value.get('linked_scripts') == authored['scripts'], 'Cached library catalog/script inputs differ')
    expected = expected_inputs(root)['compiled_files']
    wanted = [{'file': n, **{k: next(x[k] for x in before if x['path'] == str(root / n)) for k in ('bytes', 'sha256')}} for n in expected]
    require(value['compiled_inputs'] == wanted, 'Compiled inputs do not equal reviewed current source/data')
    for key, name in [('probe_source_sha256', SOURCE), ('digest_source_sha256', DIGEST), ('attestation_source_sha256', ATTEST)]:
        require(value[key] == next(x['sha256'] for x in before if x['path'] == str(root / name)), 'Cached probe executable/source mismatch')
    return value


def verify_vm(root, text, identity, authored, fixed, boundaries):
    require(len(text.encode()) <= VM_STREAM_CAP, 'VM output ceiling')
    lines = text.splitlines()
    require(len(lines) <= 4096 and all(len(x.encode()) <= safe.ROW_CAP * 2 for x in lines), 'VM bounded rows')
    rows = [strict_json(line) for line in lines]
    require(rows and rows[0] == identity and rows[-1].get('kind') == 'finished', 'VM incomplete/changed identity')
    final = rows[-1]
    require(final == {'kind': 'finished', 'actual_revision': 9, 'catalog_contexts': len(authored['catalog']), 'authored_scripts': 43,
            'authored_cases': 282, 'worked_examples': 43, 'fixed_automation_inputs': 28, 'vm_boundary_inputs': 8, 'slices': [1, 16, 32, 64],
            'claims': 'Actual public9 observations only; no emptyIFS/pipefail10 capability, learner execution, native acceptance or fabricated credit'}, 'VM finish scope mismatch')
    expected = {}
    for s in authored['scripts']:
        for index, _ in enumerate(s['cases']): expected[('authored-case', s['id'] + '/' + str(index))] = hashlib.sha256(s['source'].encode()).hexdigest()
        expected[('worked-example', s['id'])] = hashlib.sha256(s['example_source'].encode()).hexdigest()
    for s in authored['scripts']:
        if s['id'] in ('shell-handover', 'shell-capture-handover'):
            for index, _ in enumerate(s['cases']): expected[('unfinished-assignment', s['id'] + '/' + str(index))] = hashlib.sha256((s['source'] + '\nif').encode()).hexdigest()
            expected[('unfinished-example', s['id'])] = hashlib.sha256((s['example_source'] + '\nif').encode()).hexdigest()
    old8 = strict_json(safe.read_bytes(root / 'tests/fixtures/shell-v8-synthetic-writer-progress.json', PROFILE_CAP))
    script_map = {s['id']:s for s in authored['scripts']}
    for name, a in old8['attempts'].items():
        l = a.get('workspace',{}).get('script_learning')
        if l and l['example_source'] != script_map[name]['example_source']: expected[('imported-example',name)] = hashlib.sha256(l['example_source'].encode()).hexdigest()
    for s in fixed: expected[('fixed-automation-public9', s['id'])] = hashlib.sha256(s['source'].encode()).hexdigest()
    for s in boundaries: expected[('vm-boundary-public9', s['id'])] = hashlib.sha256(s['source'].encode()).hexdigest()
    observed = {}
    records, stops, profiles = {}, [], []
    for r in rows[1:-1]:
        kind = r.get('kind')
        if kind in ('authored-case', 'worked-example', 'fixed-automation-public9', 'vm-boundary-public9', 'unfinished-assignment', 'unfinished-example', 'imported-example'):
            key = (kind, r['id']); require(key in expected and r['source_sha256'] == expected[key] and r['actual_evaluator_revision'] == 9, 'Unknown VM input/revision')
            bucket = observed.setdefault(key, {})
            require(r['slice'] in (1, 16, 32, 64) and r['slice'] not in bucket and isinstance(r['ticks'], int) and r['ticks'] >= 0, 'Slice/tick identity')
            bucket[r['slice']] = r['actual']
        elif kind == 'authored-record':
            require(r['id'] not in records, 'Duplicate Record'); records[r['id']] = r
        elif kind == 'stopped-ungraded':
            require(r['record_created'] is False and r['ticks_before_stop'] > 0 and r['actual']['error'].startswith('Stopped'), 'Stop is ungraded and measured')
            stops.append(r['slice'])
        elif kind == 'profile-evidence': profiles.append(r)
        else: raise safe.OracleError('Unknown output row; no completion')
    require(set(observed) == set(expected) and all(set(v) == {1, 16, 32, 64} and all(o == v[1] for o in v.values()) for v in observed.values()), 'Missing/unstable actual observations')
    require(set(records) == {s['id'] for s in authored['scripts']} and sorted(stops) == [1, 16, 32, 64] and len(profiles) == 1, 'Missing records/Stop/profile evidence')
    require(profiles[0] == profile_evidence(len(authored['catalog'])), 'Profile evidence scope changed')
    for s in authored['scripts']:
        r = records[s['id']]
        require(r['scenario_version'] == s['version'] and r['cases'] == s['cases'], 'Authored input altered')
        require(r['record'] == {'engine_revision': 9, 'source': s['source'], 'results': [observed[('authored-case', s['id'] + '/' + str(i))][1] for i in range(len(s['cases']))]}, 'Record did not originate from actual observations')
    return rows, observed


def profile_evidence(catalog_count):
    return {'kind':'profile-evidence','synthetic_only':True,'catalog_contexts':catalog_count,
            'current9_actual_records':43,'genuine6_history_records':33,'genuine7_history_records':37,
            'genuine8_history_records':40,'ungraded_assignment_drafts':['shell-handover','shell-capture-handover'],
            'ungraded_example_drafts':['shell-handover','shell-capture-handover'],'writer_receipts':[7,8,9],
            'before9_exact_original8':True,'before8_exact_original7':True,'repeat_write_exact':True,
            'preview_cancel_preserve_target':True,'restore_exact_current9_safety_copy':True,
            'reset_first_exact_original8':True,'occupied_whitespace_slot_rejected':True,'future10_not_implemented':True}


def historical(workspace):
    return ([workspace['script_history']] if workspace.get('script_history') is not None else []) + workspace.get('script_history_more', [])


def portable_projection(main):
    # The ONLY main-only key, per Progress/MainProgress's explicit serialization.
    value = dict(main)
    require('migration_receipts' in value, 'Main-file receipts required')
    del value['migration_receipts']
    return value


def recheck_projection(current, scripts, actual, pending):
    expected = strict_json(json.dumps(current))
    for name, s in scripts.items():
        a = expected['attempts'][name]
        results = [actual[('unfinished-assignment' if name in pending else 'authored-case', name + '/' + str(i))] for i in range(len(s['cases']))]
        a['state']['run'] = {'engine_revision':9, 'source':a['state']['source'], 'results':results}
        learning = a['workspace']['script_learning']
        if learning.get('example_result') is not None:
            learning['example_revision'] = 9
            kind = 'unfinished-example' if name in pending else ('imported-example' if learning['example_source'] != s['example_source'] else 'worked-example')
            learning['example_result'] = actual[(kind, name)]
    return expected


def verify_added_history_empty(old, current, catalog):
    original_ids = set(old['attempts'])
    current_ids = set(current['attempts'])
    require(len(original_ids) == 116 and len(catalog) == 133 and original_ids <= set(catalog)
            and current_ids == set(catalog), 'Exact old116/current133 context keys required')
    added = current_ids - original_ids
    require(len(added) == 17, 'Exactly17 added current contexts required')
    for name in sorted(added):
        workspace = current['attempts'][name].get('workspace')
        require(isinstance(workspace, dict) and historical(workspace) == [],
                'Added context must have empty prior history: ' + name)
    return added


def verify_profiles(root, folder, authored, vm_text):
    original = safe.read_bytes(root / 'tests/fixtures/shell-v8-synthetic-writer-progress.json', PROFILE_CAP)
    original7 = safe.read_bytes(root / 'tests/fixtures/shell-v7-synthetic-writer-progress.json', PROFILE_CAP)
    old = strict_json(original)
    raw = safe.read_bytes(folder / 'writer9-original.json', PROFILE_CAP)
    current = strict_json(raw)
    require(set(current) == set(old) and set(current['attempts']) == set(authored['catalog']), 'Actual writer9 top-level fields/contexts')
    verify_added_history_empty(old, current, authored['catalog'])
    require(current['migration_receipts'] == {**old['migration_receipts'], 'shell': [7,8,9]}, 'Target-derived actual writer9 receipts')
    require(all(current[k] == old[k] for k in old if k not in ('attempts','migration_receipts')), 'Original profile metadata changed')
    rows = [strict_json(line) for line in vm_text.splitlines()]
    records = {r['id']: r['record'] for r in rows if r.get('kind') == 'authored-record'}
    actual = {(r['kind'],r['id']):r['actual'] for r in rows if r.get('slice') == 1 and 'actual' in r}
    scripts = {s['id']: s for s in authored['scripts']}
    pending = ('shell-handover', 'shell-capture-handover')
    for name, s in scripts.items():
        a = current['attempts'][name]
        require(a['state']['run'] == records[name], 'Current9 Record not from actual source/cases')
        require(a['state']['source'] == s['source'] + ('\nif' if name in pending else ''), 'Controlled assignment draft mismatch')
        require(a['scenario_version'] == s['version'], 'Current scenario version mismatch')
    counts = {6:0,7:0,8:0}
    for name, before in old['attempts'].items():
        a = current['attempts'][name]
        if before['state']['kind'] != 'Script':
            require(a == before, 'Unrelated original context changed: ' + name)
            continue
        w = before['workspace']; expected = historical(w)
        record = before['state'].get('run')
        require(record and record['engine_revision'] == 8, 'Missing genuine8 input Record')
        if record not in expected: expected.append(record)
        now = a['workspace']
        require(historical(now) == expected, 'Exact historical vectors changed: ' + name)
        for r in expected:
            require(r['engine_revision'] in counts, 'Unexpected historical revision'); counts[r['engine_revision']] += 1
        if 'script_learning' in w:
            wanted = strict_json(json.dumps(w['script_learning']))
            if name in pending: wanted['example_source'] += '\nif'
            require(now['script_learning'] == wanted, 'Imported example changed beyond explicit unfinished text: ' + name)
    require(counts == {6:33,7:37,8:40}, 'Genuine history count')
    require(safe.read_bytes(folder / 'progress.before-shell-v9.json', PROFILE_CAP) == original, 'Before9 raw backup is not exact original8')
    require(safe.read_bytes(folder / 'progress.before-shell-v8.json', PROFILE_CAP) == original7, 'Before8 slot was not preserved exact original7')
    require(safe.read_bytes(folder / 'progress-before-restore-1.json', PROFILE_CAP) == raw, 'Restore safety copy is not exact writer9 bytes')
    portable = strict_json(safe.read_bytes(folder / 'synthetic-current9-portable.json', PROFILE_CAP))
    require(portable_projection(current) == portable, 'Portable fields differ from actual current9 writer')
    # Independent exact recheck projection: every assignment result comes from
    # separately logged public9 observations; historical fields never change.
    expected = recheck_projection(current, scripts, actual, pending)
    restored = strict_json(safe.read_bytes(folder / 'progress.json', PROFILE_CAP))
    require(restored == expected, 'Explicit restore recheck differs from independently observed full-field projection')
    require(safe.read_bytes(folder / 'reset-first/progress.before-shell-v9.json', PROFILE_CAP) == original, 'Reset-first missed exact target backup')
    require(safe.read_bytes(folder / 'occupied-slot/progress.json', PROFILE_CAP) == original
            and safe.read_bytes(folder / 'occupied-slot/progress.before-shell-v9.json', PROFILE_CAP) == original + b'\n', 'Conflicting slot/target changed')


def owned_outputs(folder, budget):
    result, total, actual_total, entries = [], 0, 0, 0
    def visit(p, depth):
        nonlocal total, actual_total, entries
        require(depth <= 4, 'Profile directory depth')
        with os.scandir(p) as names:
            for entry in names:
                entries += 1; require(entries <= 64, 'Profile entry count')
                info = entry.stat(follow_symlinks=False)
                path = p / entry.name
                require(not stat.S_ISLNK(info.st_mode), 'Profile symlink forbidden')
                if stat.S_ISDIR(info.st_mode): visit(path, depth + 1)
                else:
                    require(stat.S_ISREG(info.st_mode) and 0 <= info.st_size <= PROFILE_CAP and total + info.st_size <= PROFILE_TOTAL_CAP, 'Profile aggregate/per-file bound before hashing')
                    total += info.st_size
                    actual = safe.pin(path, PROFILE_CAP)
                    require(0 <= actual['bytes'] <= PROFILE_CAP, 'Actual profile per-file bound')
                    actual_total += actual['bytes']
                    require(actual_total <= PROFILE_TOTAL_CAP, 'Actual profile aggregate bound')
                    result.append(actual)
    visit(folder, 0)
    budget.charge(actual_total)
    return result


def capture_child(command, folder, work, env, data, budget, stream_cap, *, started, timeout, inherited):
    # Recompute immediately before EVERY fixed child, including identity/version.
    remaining = WHOLE_SECONDS - (time.monotonic() - started)
    require(remaining > 0, 'Whole capture deadline before child launch')
    return safe.capture_command(command, folder, work, env, data, budget, stream_cap,
            timeout=min(timeout, remaining), inherited=inherited)


def capture_reference(row, index, bash, folder, work, temporary, budget, started, inherited):
    # Internal call site receives only the hash-admitted fixed28 row, never CLI source.
    return capture_child([bash, *safe.BASH_FLAGS, '-c', row['source'], 'fixed-automation-' + str(index), *row['input']['args']],
            folder, work, safe.closed_environment(row['input']['env'], temporary), row['input']['stdin'].encode(),
            budget, safe.STREAM_CAP, started=started, timeout=safe.TIMEOUT, inherited=inherited)


def capture(root, name, inherited):
    name = capture_id(name); started = time.monotonic(); sources = source_set(root)
    approved_raw, control, before = approval(root, name, sources)
    fixed_raw, fixed = load_fixed(root)
    vm_raw, boundaries = load_vm(root)
    authored_raw, authored = authored_inputs(root)
    parent = safe.private_dir(review_dir(root), 'captures', existing=True)
    out = safe.private_dir(parent, name); budget = safe.Budget(TOTAL_OUTPUT_CAP); outputs = []
    outputs.append(safe.write_new(out / 'APPROVED.json', approved_raw, budget, SOURCE_CAP))
    outputs.append(safe.write_new(out / 'FIXED-INPUTS.json', fixed_raw, budget, SOURCE_CAP))
    outputs.append(safe.write_new(out / 'AUTHORED-INPUTS.json', authored_raw, budget, SOURCE_CAP))
    outputs.append(safe.write_new(out / 'VM-ONLY-INPUTS.json', vm_raw, budget, SOURCE_CAP))
    outputs.append(safe.json_new(out / 'STARTED.json', {'status': 'started-not-complete', 'pins': before, 'source_head': control['source_head']}, budget, SOURCE_CAP))
    profile = safe.private_dir(out, 'vm-profile')
    vm_pin = next(p for p in before if p['path'] == str(root / PROBE))
    bash_path = safe.BASH.resolve(strict=True); bash_pin = next(p for p in before if p['path'] == str(bash_path))
    with pinned_executable(root / PROBE, vm_pin, BINARY_CAP) as (executable, fd), pinned_executable(bash_path, bash_pin, 256 * SOURCE_CAP) as (bash, bash_fd):
        folder = safe.private_dir(out, 'identity'); temporary = safe.private_dir(folder, 'tmp')
        observed = capture_child([executable, '--identity'], folder, root, vm_environment(root, temporary, profile), b'', budget, SOURCE_CAP, started=started, timeout=10, inherited=(*inherited, fd))
        safe.quiet_group(); require(observed['status'] == 0 and not observed['stderr'], 'Identity probe failed')
        identity = identity_rows(root, observed['stdout'], before)
        outputs += [safe.pin(folder / f, SOURCE_CAP) for f in ('stdout.raw', 'stderr.raw')]
        folder = safe.private_dir(out, 'vm'); temporary = safe.private_dir(folder, 'tmp')
        observed = capture_child([executable], folder, root, vm_environment(root, temporary, profile), b'', budget, VM_STREAM_CAP, started=started, timeout=VM_SECONDS, inherited=(*inherited, fd))
        safe.quiet_group(); require(observed['status'] == 0 and not observed['stderr'], 'Actual9 capture incomplete; retained raw failure')
        vm_rows, native = verify_vm(root, observed['stdout'], identity, authored, fixed, boundaries)
        outputs += [safe.pin(folder / f, VM_STREAM_CAP) for f in ('stdout.raw', 'stderr.raw')]
        verify_profiles(root, profile, authored, observed['stdout'])
        outputs += owned_outputs(profile, budget)
        folder = safe.private_dir(out, 'version'); work = safe.private_dir(folder, 'work'); temporary = safe.private_dir(folder, 'tmp')
        version = capture_child([bash, '--noprofile', '--norc', '--version'], folder, work, safe.closed_environment({}, temporary), b'', budget, safe.VERSION_CAP, started=started, timeout=safe.TIMEOUT, inherited=(*inherited, bash_fd))
        safe.quiet_group(); require(version['status'] == 0 and not version['stderr'] and re.match(r'^GNU bash, version 5\.3\.9\b', version['stdout']), 'Reviewed actual Bash5.3.9 banner required')
        outputs += [safe.pin(folder / f, safe.VERSION_CAP) for f in ('stdout.raw', 'stderr.raw')]
        for i, r in enumerate(fixed):
            require(time.monotonic() - started < WHOLE_SECONDS, 'Whole capture deadline')
            folder = safe.private_dir(out, 'fixed-' + str(i).zfill(3)); work = safe.private_dir(folder, 'work'); temporary = safe.private_dir(folder, 'tmp')
            initial = safe.materialize(work, r['input']['files'], budget)
            actual = capture_reference(r, i, bash, folder, work, temporary, budget, started, (*inherited, bash_fd))
            safe.quiet_group(); require(actual['status'] >= 0, 'Signaled Bash case is incomplete')
            after, count = safe.final_files(work); budget.replace_reservation(initial, count); actual['files'] = after
            # Difference is evidence, not a regression decision or future10 expected result.
            result = {'id': r['id'], 'source': r['source'], 'input': r['input'], 'actual_bash': actual,
                      'actual_public9': native[('fixed-automation-public9', r['id'])][1],
                      'actual_bash_version': version['stdout'].splitlines()[0], 'comparison': 'unclassified-before10', 'future10_acceptance': None}
            outputs += [safe.pin(folder / f, safe.STREAM_CAP) for f in ('stdout.raw', 'stderr.raw')]
            outputs.append(safe.json_new(folder / 'OBSERVATION.json', result, budget))
            outputs += [safe.pin(work / safe.safe_relative(n), safe.FILE_CAP) for n in after]
    # No more child launches below this boundary.
    safe.quiet_group(); require(time.monotonic() - started < WHOLE_SECONDS, 'Whole deadline before final verification')
    require(safe.source_identity(root) == control['source_head'] and pins(sources) == before, 'Pinned source/binaries changed during capture')
    for p in outputs: require(safe.pin(Path(p['path']), TOTAL_OUTPUT_CAP) == p, 'Output changed before publication')
    manifest = {'schema_version': 1, 'status': 'complete-only-with-valid-COMPLETE', 'source_head': control['source_head'],
        'actual_evaluator_revision': 9, 'catalog_contexts': len(authored['catalog']), 'authored_scripts': 43, 'authored_cases': 282, 'worked_examples': 43,
        'fixed_automation_inputs': 28, 'vm_boundary_inputs': 8, 'vm_rows':len(vm_rows), 'observed_input_vectors':len(native), 'pins': before, 'outputs': [{'file': Path(p['path']).relative_to(out).as_posix(), 'bytes': p['bytes'], 'sha256': p['sha256']} for p in outputs],
        'constraints': {'whole_seconds': WHOLE_SECONDS, 'vm_seconds': VM_SECONDS, 'vm_per_stream_bytes':VM_STREAM_CAP, 'bash_case_seconds': safe.TIMEOUT, 'total_output_bytes': TOTAL_OUTPUT_CAP},
        'claims': 'Controlled actual public9 and Bash5.3.9 observations; genuine6/7/8 history retained, new9 reference Records actually executed. No evaluator10/curriculum/native/learner-data claim.'}
    require(time.monotonic() - started < WHOLE_SECONDS, 'Whole deadline before COMPLETE')
    committed, warning = safe.publish(out, manifest, budget)
    return {'status': 'COMMITTED', 'directory': str(out), 'manifest_sha256': committed['sha256'], 'durability_warning': warning}


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    mode = p.add_mutually_exclusive_group(required=True)
    mode.add_argument('--prepare-id'); mode.add_argument('--capture-id')
    args = p.parse_args(argv)
    root = ROOT.resolve(strict=True); inherited = fixed_root_guard(root)
    result = prepare(root, args.prepare_id) if args.prepare_id else capture(root, args.capture_id, inherited)
    try: print(json.dumps(result))
    except OSError: pass  # Post-COMPLETE/PROPOSED diagnostics cannot undo committed files.
    return 0


if __name__ == '__main__':
    try: raise SystemExit(main())
    except (safe.OracleError, OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        try: print('Fixed automation capture aborted; partial evidence retained, no COMPLETE claimed: ' + str(error), file=sys.stderr)
        except OSError: pass
        raise SystemExit(1)
