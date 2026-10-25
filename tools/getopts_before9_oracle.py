#!/usr/bin/env python3
"""Fixed public8-before9/getopts oracle. MAIN guarded tool only; never learner input.

Preparation hashes metadata only; capture requires an independently reviewed
APPROVED-ID.json. No command/script/source/binary/profile path argument exists.
Existing37 input/tool and all original historical observations remain untouched.
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
HERE = '.build-artifacts/reviews/getopts'
FIXED = 'tests/fixtures/getopts-oracle-inputs.json'
AUTHORED = 'tests/fixtures/getopts-before9-authored-inputs.json'
PROBE = 'target/debug/examples/getopts_before9_capture'
SOURCE = 'examples/getopts_before9_capture.rs'
DIGEST = 'examples/support/getopts_digest.rs'
ATTEST = 'examples/support/getopts_attestation.rs'
FIXED_SHA = '90f4440e790c5669dd8cffa662e78e04f2b0613bf5bab4cfd1f4bc80d0635bde'
BINARY_CAP = 1024 * 1024 * 1024
SOURCE_CAP = 2 * 1024 * 1024
PROFILE_CAP = 16_000_000
TOTAL_PIN_CAP = 1536 * 1024 * 1024
TOTAL_OUTPUT_CAP = 128 * 1024 * 1024
VM_STREAM_CAP = 32 * 1024 * 1024
WHOLE_SECONDS = 300.0
VM_SECONDS = 180.0
PIN_PATHS = 'tests/fixtures/getopts-before9-source-pins.json'


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
    require(hashlib.sha256(raw).hexdigest() == FIXED_SHA, 'Reviewed fixed50 source changed')
    d = strict_json(raw)
    cases = d['cases']
    require(isinstance(cases, list) and len(cases) == 50, 'Exactly50 fixed sources required')
    seen = set()
    for row in cases:
        name = row['id']
        require(isinstance(name, str) and re.fullmatch(r'[a-z0-9-]{1,100}', name) and name not in seen, 'Fixed identity')
        seen.add(name)
        require(isinstance(row['source'], str) and 0 < len(row['source'].encode()) <= 4096 and '\0' not in row['source'], 'Fixed source bound')
        require(all(row[k] is None for k in ['actual_bash_5_3', 'actual_evaluator8_before9', 'accepted_after9']), 'Input must never acquire fabricated observations')
        i = row['input']
        require(set(i) == {'args', 'stdin', 'env', 'files'}, 'Fixed input shape')
        require(len(i['args']) <= 16 and all(isinstance(v, str) and '\0' not in v and len(v.encode()) <= 256 for v in i['args']), 'Fixed args')
        require(i['stdin'] == '' and i['files'] == {'/work/keep.txt': 'retained\n'}, 'Only reviewed disposable inputs')
        require(i['env'] == {} or (name == 'prior8-opterr-fixture-env' and i['env'] == {'OPTERR': '0'}), 'No arbitrary environment')
        safe.closed_environment(i['env'], Path('/fixed-placeholder'))
    return raw, cases


def expected_inputs(root):
    data = strict_json(safe.read_bytes(root / PIN_PATHS, SOURCE_CAP))
    require(set(data) == {'compiled_files', 'original_engine8', 'frozen_inputs'}, 'Source pin-list shape')
    paths = data['compiled_files']
    require(isinstance(paths, list) and 16 <= len(paths) <= 200 and len(set(paths)) == len(paths), 'Bounded unique source path list')
    for p in paths:
        require(isinstance(p, str) and not Path(p).is_absolute() and '..' not in Path(p).parts
                and (p.startswith(('src/', 'scenarios/', 'tests/fixtures/')) or p in ('Cargo.toml', 'Cargo.lock')), 'Compiled path outside closed source inputs')
    actual_rs = {p.relative_to(root).as_posix() for p in (root / 'src').rglob('*.rs')
                 if 'ui' not in p.relative_to(root).parts and p.name not in ('main.rs', 'ui.rs', 'startup_trace.rs')}
    require({n for n in paths if n.startswith('src/')} == actual_rs,
            'Library module set changed; regenerate/review compile attestation before capture')
    for collection in ['original_engine8', 'frozen_inputs']:
        require(isinstance(data[collection], dict) and 1 <= len(data[collection]) <= 32, 'Bounded original identity list')
        for name, expected in data[collection].items():
            require(name in paths and isinstance(expected, str) and re.fullmatch('[0-9a-f]{64}', expected), 'Original identity shape')
            require(safe.pin(root / name, SOURCE_CAP)['sha256'] == expected, 'Original evaluator8/historical evidence changed: ' + name)
    return data


def source_set(root):
    data = expected_inputs(root)
    names = list(data['compiled_files']) + [PIN_PATHS, SOURCE, DIGEST, ATTEST,
            'tools/getopts_before9_oracle.py', 'tools/case_bash_oracle.py', 'tools/dev.py', 'tools/build_safety.py',
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
    load_fixed(root)
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
                 DLMS_FIXED_GETOPTS_CHILD='before9-public8-r1', DLMS_FIXED_GETOPTS_OUTPUT=str(profile))
    return value


def authored_inputs(root):
    raw = safe.read_bytes(root / AUTHORED, SOURCE_CAP)
    authored = strict_json(raw)
    require(len(authored['catalog']) == 116 and len(authored['scripts']) == 40
            and sum(len(x['cases']) for x in authored['scripts']) == 246, 'Closed current authored scope')
    return raw, authored


def identity_rows(root, raw, before):
    lines = raw.splitlines()
    require(len(lines) == 1, 'Identity-only mode must produce exactly one row')
    value = strict_json(lines[0])
    require(value.get('kind') == 'identity' and value.get('revision') == 8 and value.get('compiled_manifest_dir') == str(root), 'Wrong linked revision/checkout')
    _, authored = authored_inputs(root)
    require(value.get('linked_catalog') == authored['catalog'] and value.get('linked_scripts') == authored['scripts'], 'Cached library catalog/script inputs differ')
    expected = expected_inputs(root)['compiled_files']
    wanted = [{'file': n, **{k: next(x[k] for x in before if x['path'] == str(root / n)) for k in ('bytes', 'sha256')}} for n in expected]
    require(value['compiled_inputs'] == wanted, 'Compiled inputs do not equal reviewed current source/data')
    for key, name in [('probe_source_sha256', SOURCE), ('digest_source_sha256', DIGEST), ('attestation_source_sha256', ATTEST)]:
        require(value[key] == next(x['sha256'] for x in before if x['path'] == str(root / name)), 'Cached probe executable/source mismatch')
    return value


def verify_vm(root, text, identity, authored, fixed):
    require(len(text.encode()) <= VM_STREAM_CAP, 'VM output ceiling')
    lines = text.splitlines()
    require(len(lines) <= 4096 and all(len(x.encode()) <= safe.ROW_CAP * 2 for x in lines), 'VM bounded rows')
    rows = [strict_json(line) for line in lines]
    require(rows and rows[0] == identity and rows[-1].get('kind') == 'finished', 'VM incomplete/changed identity')
    final = rows[-1]
    require(final == {'kind': 'finished', 'actual_revision': 8, 'catalog_contexts': 116, 'authored_scripts': 40,
            'authored_cases': 246, 'worked_examples': 40, 'fixed_getopts_inputs': 50, 'slices': [1, 16, 32, 64],
            'claims': 'Actual public8 observations only; no getopts9 capability, learner execution, native acceptance or fabricated credit'}, 'VM finish scope mismatch')
    expected = {}
    for s in authored['scripts']:
        for index, _ in enumerate(s['cases']): expected[('authored-case', s['id'] + '/' + str(index))] = hashlib.sha256(s['source'].encode()).hexdigest()
        expected[('worked-example', s['id'])] = hashlib.sha256(s['example_source'].encode()).hexdigest()
    for s in fixed: expected[('fixed-getopts-public8', s['id'])] = hashlib.sha256(s['source'].encode()).hexdigest()
    observed = {}
    records, stops, profiles = {}, [], []
    for r in rows[1:-1]:
        kind = r.get('kind')
        if kind in ('authored-case', 'worked-example', 'fixed-getopts-public8'):
            key = (kind, r['id']); require(key in expected and r['source_sha256'] == expected[key] and r['actual_evaluator_revision'] == 8, 'Unknown VM input/revision')
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
    for s in authored['scripts']:
        r = records[s['id']]
        require(r['scenario_version'] == s['version'] and r['cases'] == s['cases'], 'Authored input altered')
        require(r['record'] == {'engine_revision': 8, 'source': s['source'], 'results': [observed[('authored-case', s['id'] + '/' + str(i))][1] for i in range(len(s['cases']))]}, 'Record did not originate from actual observations')
    return rows, observed


def verify_profiles(root, folder, authored, vm_text):
    original = safe.read_bytes(root / 'tests/fixtures/shell-v7-synthetic-writer-progress.json', PROFILE_CAP)
    old = strict_json(original)
    raw = safe.read_bytes(folder / 'writer8-original.json', PROFILE_CAP)
    current = strict_json(raw)
    require(set(current['attempts']) == set(authored['catalog']) and len(current['attempts']) == 116,
            'Actual writer8 missing supplied contexts')
    require(current['migration_receipts']['shell'] == [7, 8], 'Target-derived actual writer8 receipts')
    require(current['favorites'] == old['favorites'] and current['clock'] == old['clock'], 'Original profile metadata changed')
    records = {r['id']: r['record'] for r in map(strict_json, vm_text.splitlines()) if r.get('kind') == 'authored-record'}
    for s in authored['scripts']:
        a = current['attempts'][s['id']]
        require(a['state']['run'] == records[s['id']], 'Current8 Record not from actual source/cases')
        wanted = s['source'] + ('\nif' if s['id'] in ('shell-handover', 'shell-capture-handover') else '')
        require(a['state']['source'] == wanted, 'Controlled assignment draft mismatch')
    six = seven = 0
    for name, before in old['attempts'].items():
        a = current['attempts'][name]
        if before['state']['kind'] != 'Script':
            require(a == before, 'Unrelated original context changed: ' + name)
            continue
        w = before.get('workspace', {})
        expected = ([w['script_history']] if w.get('script_history') is not None else []) + w.get('script_history_more', [])
        record = before['state'].get('run')
        require(record and record['engine_revision'] == 7, 'Missing genuine7 input Record')
        if record not in expected: expected.append(record)
        now = a['workspace']; actual = ([now['script_history']] if now.get('script_history') is not None else []) + now.get('script_history_more', [])
        require(actual == expected, 'Exact historical vectors changed: ' + name)
        six += sum(r['engine_revision'] == 6 for r in actual)
        seven += sum(r['engine_revision'] == 7 for r in actual)
        if 'script_learning' in w:
            require(now['script_learning'] == w['script_learning'], 'Separate imported example changed before explicit recheck: ' + name)
    require((six, seven) == (33, 37), 'Genuine history count')
    require(safe.read_bytes(folder / 'progress.before-shell-v8.json', PROFILE_CAP) == original, 'Before8 raw backup is not exact original7')
    require(safe.read_bytes(folder / 'progress-before-restore-1.json', PROFILE_CAP) == raw, 'Restore safety copy is not exact writer8 bytes')
    portable = strict_json(safe.read_bytes(folder / 'synthetic-current8-portable.json', PROFILE_CAP))
    require('migration_receipts' not in portable, 'Portable serialization imported writer receipt')
    projected = dict(current); projected.pop('migration_receipts')
    require(projected == portable, 'Portable fields differ from actual current8 writer')
    restored = strict_json(safe.read_bytes(folder / 'progress.json', PROFILE_CAP))
    require(set(restored['attempts']) == set(current['attempts'])
            and restored['migration_receipts']['shell'] == [7, 8], 'Restored contexts/actual-target receipts differ')
    for name, before in current['attempts'].items():
        now = restored['attempts'][name]
        if before['state']['kind'] != 'Script':
            require(now == before, 'Restore changed unrelated context: ' + name)
            continue
        require(now['state']['source'] == before['state']['source'], 'Restore lost assignment draft: ' + name)
        for field in ('command', 'script_history', 'script_history_more'):
            require(now['workspace'].get(field) == before['workspace'].get(field), 'Restore lost draft/history: ' + name)
        require(now['workspace']['script_learning']['example_source'] == before['workspace']['script_learning']['example_source'],
                'Restore lost independent example draft: ' + name)
    require(safe.read_bytes(folder / 'reset-first/progress.before-shell-v8.json', PROFILE_CAP) == original, 'Reset-first missed exact target backup')
    require(safe.read_bytes(folder / 'occupied-slot/progress.json', PROFILE_CAP) == original
            and safe.read_bytes(folder / 'occupied-slot/progress.before-shell-v8.json', PROFILE_CAP) == original + b'\n', 'Conflicting slot/target changed')


def owned_outputs(folder, budget):
    result, total, entries = [], 0, 0
    def visit(p, depth):
        nonlocal total, entries
        require(depth <= 4, 'Profile directory depth')
        with os.scandir(p) as names:
            for entry in names:
                entries += 1; require(entries <= 64, 'Profile entry count')
                info = entry.stat(follow_symlinks=False)
                path = p / entry.name
                require(not stat.S_ISLNK(info.st_mode), 'Profile symlink forbidden')
                if stat.S_ISDIR(info.st_mode): visit(path, depth + 1)
                else:
                    require(stat.S_ISREG(info.st_mode) and info.st_size <= PROFILE_CAP and total + info.st_size <= 64 * 1024 * 1024, 'Profile aggregate/per-file bound before hashing')
                    total += info.st_size; result.append(safe.pin(path, PROFILE_CAP))
    visit(folder, 0)
    budget.charge(total)
    return result


def capture(root, name, inherited):
    name = capture_id(name); started = time.monotonic(); sources = source_set(root)
    approved_raw, control, before = approval(root, name, sources)
    fixed_raw, fixed = load_fixed(root)
    authored_raw, authored = authored_inputs(root)
    parent = safe.private_dir(review_dir(root), 'captures', existing=True)
    out = safe.private_dir(parent, name); budget = safe.Budget(TOTAL_OUTPUT_CAP); outputs = []
    outputs.append(safe.write_new(out / 'APPROVED.json', approved_raw, budget, SOURCE_CAP))
    outputs.append(safe.write_new(out / 'FIXED-INPUTS.json', fixed_raw, budget, SOURCE_CAP))
    outputs.append(safe.write_new(out / 'AUTHORED-INPUTS.json', authored_raw, budget, SOURCE_CAP))
    outputs.append(safe.json_new(out / 'STARTED.json', {'status': 'started-not-complete', 'pins': before, 'source_head': control['source_head']}, budget, SOURCE_CAP))
    profile = safe.private_dir(out, 'vm-profile')
    vm_pin = next(p for p in before if p['path'] == str(root / PROBE))
    bash_path = safe.BASH.resolve(strict=True); bash_pin = next(p for p in before if p['path'] == str(bash_path))
    with pinned_executable(root / PROBE, vm_pin, BINARY_CAP) as (executable, fd), pinned_executable(bash_path, bash_pin, 256 * SOURCE_CAP) as (bash, bash_fd):
        folder = safe.private_dir(out, 'identity'); temporary = safe.private_dir(folder, 'tmp')
        observed = safe.capture_command([executable, '--identity'], folder, root, vm_environment(root, temporary, profile), b'', budget, SOURCE_CAP, timeout=10, inherited=(*inherited, fd))
        safe.quiet_group(); require(observed['status'] == 0 and not observed['stderr'], 'Identity probe failed')
        identity = identity_rows(root, observed['stdout'], before)
        outputs += [safe.pin(folder / f, SOURCE_CAP) for f in ('stdout.raw', 'stderr.raw')]
        folder = safe.private_dir(out, 'vm'); temporary = safe.private_dir(folder, 'tmp')
        observed = safe.capture_command([executable], folder, root, vm_environment(root, temporary, profile), b'', budget, VM_STREAM_CAP, timeout=VM_SECONDS, inherited=(*inherited, fd))
        safe.quiet_group(); require(observed['status'] == 0 and not observed['stderr'], 'Actual8 capture incomplete; retained raw failure')
        _, native = verify_vm(root, observed['stdout'], identity, authored, fixed)
        outputs += [safe.pin(folder / f, VM_STREAM_CAP) for f in ('stdout.raw', 'stderr.raw')]
        verify_profiles(root, profile, authored, observed['stdout'])
        outputs += owned_outputs(profile, budget)
        folder = safe.private_dir(out, 'version'); work = safe.private_dir(folder, 'work'); temporary = safe.private_dir(folder, 'tmp')
        version = safe.capture_command([bash, '--noprofile', '--norc', '--version'], folder, work, safe.closed_environment({}, temporary), b'', budget, safe.VERSION_CAP, inherited=(*inherited, bash_fd))
        safe.quiet_group(); require(version['status'] == 0 and not version['stderr'] and re.match(r'^GNU bash, version 5\.3\.9\b', version['stdout']), 'Reviewed actual Bash5.3.9 banner required')
        outputs += [safe.pin(folder / f, safe.VERSION_CAP) for f in ('stdout.raw', 'stderr.raw')]
        for i, r in enumerate(fixed):
            require(time.monotonic() - started < WHOLE_SECONDS, 'Whole capture deadline')
            folder = safe.private_dir(out, 'fixed-' + str(i).zfill(3)); work = safe.private_dir(folder, 'work'); temporary = safe.private_dir(folder, 'tmp')
            initial = safe.materialize(work, r['input']['files'], budget)
            actual = safe.capture_command([bash, *safe.BASH_FLAGS, '-c', r['source'], 'fixed-getopts-' + str(i), *r['input']['args']], folder, work,
                    safe.closed_environment(r['input']['env'], temporary), b'', budget, safe.STREAM_CAP,
                    timeout=min(safe.TIMEOUT, max(.001, WHOLE_SECONDS - (time.monotonic() - started))), inherited=(*inherited, bash_fd))
            safe.quiet_group(); require(actual['status'] >= 0, 'Signaled Bash case is incomplete')
            after, count = safe.final_files(work); budget.replace_reservation(initial, count); actual['files'] = after
            # Difference is evidence, not a regression decision or future9 expected result.
            result = {'id': r['id'], 'source': r['source'], 'input': r['input'], 'actual_bash': actual,
                      'actual_public8': native[('fixed-getopts-public8', r['id'])][1],
                      'actual_bash_version': version['stdout'].splitlines()[0], 'comparison': 'unclassified-before9', 'getopts9_acceptance': None}
            outputs += [safe.pin(folder / f, safe.STREAM_CAP) for f in ('stdout.raw', 'stderr.raw')]
            outputs.append(safe.json_new(folder / 'OBSERVATION.json', result, budget))
            outputs += [safe.pin(work / safe.safe_relative(n), safe.FILE_CAP) for n in after]
    # No more child launches below this boundary.
    safe.quiet_group(); require(time.monotonic() - started < WHOLE_SECONDS, 'Whole deadline before final verification')
    require(safe.source_identity(root) == control['source_head'] and pins(sources) == before, 'Pinned source/binaries changed during capture')
    for p in outputs: require(safe.pin(Path(p['path']), TOTAL_OUTPUT_CAP) == p, 'Output changed before publication')
    manifest = {'schema_version': 1, 'status': 'complete-only-with-valid-COMPLETE', 'source_head': control['source_head'],
        'actual_evaluator_revision': 8, 'catalog_contexts': 116, 'authored_scripts': 40, 'authored_cases': 246,
        'fixed_getopts_inputs': 50, 'pins': before, 'outputs': [{'file': Path(p['path']).relative_to(out).as_posix(), 'bytes': p['bytes'], 'sha256': p['sha256']} for p in outputs],
        'constraints': {'whole_seconds': WHOLE_SECONDS, 'vm_seconds': VM_SECONDS, 'bash_case_seconds': safe.TIMEOUT, 'total_output_bytes': TOTAL_OUTPUT_CAP},
        'claims': 'Controlled actual public8 and Bash observations; genuine6/7 history retained, new8 reference Records actually executed. No evaluator9/curriculum/native/learner-data claim.'}
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
        try: print('Fixed getopts capture aborted; partial evidence retained, no COMPLETE claimed: ' + str(error), file=sys.stderr)
        except OSError: pass
        raise SystemExit(1)
