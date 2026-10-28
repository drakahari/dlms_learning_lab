#!/usr/bin/env python3
"""Fixed genuine11 before12 development capture/attestation, not learner input.

No source/profile/path/engine CLI. Prepare launches nothing. Capture requires
Root-reviewed exact current source/binary pins and the original genuine10 data.
SEALED_MANIFEST is filled only after an actual11 capture and independent review;
until then verification cannot authorize a future evaluator12 transition.
"""
import argparse
import copy
from collections import Counter
import hashlib
import json
import os
import re
from pathlib import Path

import case_bash_oracle as safe
import parameters_before11_verify as before10
from automation_before10_oracle import pinned_executable

ROOT = Path(__file__).resolve().parents[1]
HERE = '.build-artifacts/reviews/bash-numeric-validation/captures'
DESTINATION = HERE + '/public11-164-384-r1'
PRODUCER = 'target/debug/examples/scripting_before12_capture'
SOURCE = 'examples/scripting_before12_capture.rs'
INDEX = 'tests/fixtures/bash-before12-source-index.json'
INDEX_SHA = '26645b623e5ae983634ca591259ab54f3154bd333fe10c3b43cf7a0ac8f13d91'
PRIOR_SHA = '220cebac329a62adcc59e8338df80b94a07a383807d5832dd6905bbf332776a1'
SELF = 'tools/scripting_before12_verify.py'
TEST = 'tools/tests/test_scripting_before12_verify.py'
CAP = 16_000_000
BINARY_CAP = 512 * 1024 * 1024
TOTAL = 256 * 1024 * 1024
ATTEST_TOTAL = TOTAL + BINARY_CAP
SEALED_MANIFEST = "e303bdecb9f57a89debd77c8300d27b523c94630446bc918e4c5e4c171373168"  # Observed actual11 capture; independent Root review.
SEALED_ATTESTATION = "254d696e4ca90ec508b6034f015ae5710bf2184a30e068d9839a9f93d6523e48"
NEW = ['shell-optional-policy', 'shell-optional-report', 'shell-optional-handover']
DRAFTS = ['shell-handover', 'shell-command-plan']


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode()


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def strict(raw):
    def pairs(items):
        value = {}
        for k, v in items:
            safe.require(k not in value, 'Duplicate JSON key')
            value[k] = v
        return value
    return json.loads(raw, object_pairs_hook=pairs)


def contained(root, name):
    safe.require(isinstance(name, str), 'Contained fixed relative path')
    path = Path(name)
    safe.require(isinstance(name, str) and not path.is_absolute() and path.parts
                 and '..' not in path.parts and str(path) == name, 'Contained fixed relative path')
    return root / path


def index_value(raw):
    safe.require(sha(raw) == INDEX_SHA, 'Fixed52 authored input index changed')
    value = strict(raw)
    for key, expected in {'actual_evaluator_revision_required': 11, 'catalog_count': 164,
                           'script_count': 52, 'authored_cases': 384, 'separate_examples': 52,
                           'prior_profile_contexts': 139, 'expected_profile_contexts': 142}.items():
        safe.require(value[key] == expected and not isinstance(value[key], bool), 'Fixed actual11 scope: ' + key)
    safe.require(value['explicit_new_script_contexts'] == NEW and value['retained_draft_ids'] == DRAFTS,
                 'Only3 explicit new script fixtures and2 authentic drafts')
    safe.require(value['prior_actual10_manifest_sha256'] == PRIOR_SHA
                 and len(value['catalog']) == 164 and len(value['scripts']) == 52
                 and len(value['prior_profile_ids']) == 139 and len(set(value['prior_profile_ids'])) == 139
                 and len(value['non_script_absent_not_invented']) == 22,
                 'Exact authored/legacy identity counts')
    safe.require(sum(len(s['cases']) for s in value['scripts']) == 384, 'All384 authored cases')
    return value


def load_index(root):
    return index_value(safe.read_bytes(root / INDEX, CAP))

def input10(root, index):
    # The original verifier remains byte-unchanged. It checks the immutable
    # genuine10 manifest/COMPLETE/all output hashes, never reruns its producer.
    safe.require(before10.EXPECTED == PRIOR_SHA and before10.CAPTURE == root / (
        '.build-artifacts/reviews/bash-parameter-defaults/captures/public10-154-350-r1'), 'Fixed genuine10 verifier identity')
    before10.verify()
    result = {}
    for role in ['original', 'drafts']:
        path = before10.CAPTURE / ('writer10-' + role + '.json')
        raw = safe.read_bytes(path, CAP)
        safe.require(sha(raw) == index['prior_' + role + '_sha256'], 'Original10 raw profile pin changed')
        value = strict(raw)
        safe.require(sorted(value['attempts']) == index['prior_profile_ids']
                     and value['migration_receipts'] == {'workspace': True, 'shell': [7, 8, 9, 10]},
                     'Authentic139 contexts/receipts unchanged')
        result[role] = (raw, value)
    return result


def outcome(expected, actual):
    return isinstance(actual, dict) and set(actual) == {'stdout', 'stderr', 'status', 'files', 'error'} \
        and actual['error'] is None and actual['stdout'] == expected['stdout'] \
        and actual['stderr'] == expected['stderr'] and actual['status'] == expected['status'] \
        and not isinstance(actual['status'], bool) and actual['files'] == expected['after']


def authored(raw, index):
    lines = raw.decode('utf-8', errors='strict').splitlines()
    safe.require(len(lines) == 52, 'All52 actual11 script/example observations required')
    rows = []
    for text, expected in zip(lines, index['scripts']):
        row = strict(text)
        safe.require(row['id'] == expected['id'] and row['source'] == expected['source']
                     and row['cases'] == expected['cases'] and row['example_source'] == expected['example_source']
                     and row['example_case'] == expected['example_case'] and row['example_revision'] == 11,
                     'Actual11 observation uses exact authored source/input/order')
        record = row['record']
        safe.require(set(record) == {'engine_revision', 'source', 'results'} and record['engine_revision'] == 11
                     and record['source'] == expected['source'] and len(record['results']) == len(expected['cases']),
                     'Actual11 record revision/source/case count')
        safe.require(all(outcome(c, r) for c, r in zip(expected['cases'], record['results']))
                     and outcome(expected['example_case'], row['example_outcome']),
                     'Actual11 outcomes must match every independently authored behavior')
        rows.append(row)
    return rows


def records(attempt, include_current):
    values = []
    if include_current and attempt.get('state', {}).get('run') is not None:
        values.append(attempt['state']['run'])
    work = attempt.get('workspace', {})
    if work.get('script_history') is not None:
        values.append(work['script_history'])
    values.extend(work.get('script_history_more', []))
    return Counter(canonical(v) for v in values)


def stable_metadata(attempt):
    value = copy.deepcopy(attempt)
    value.pop('state')
    work = value.get('workspace', {})
    for field in ['script_history', 'script_history_more', 'checked']:
        work.pop(field, None)
    for field in ['example_revision', 'example_result']:
        work.get('script_learning', {}).pop(field, None)
    return value


def profile(old, current, index, observations, drafts):
    expected_ids = set(index['prior_profile_ids']) | set(NEW)
    safe.require(set(current['attempts']) == expected_ids and len(expected_ids) == 142,
                 '139 authentic contexts plus only3 explicitly executed script fixtures')
    safe.require(set(current) == set(old), 'Preserve exact profile top-level fields')
    safe.require(current['favorites'] == old['favorites'] and current['schema_version'] == old['schema_version']
                 and current['migration_receipts'] == {'workspace': True, 'shell': [7, 8, 9, 10, 11]},
                 'Preserved favorites/schema and genuine11 receipt append')
    by_id = {row['id']: row for row in observations}
    skipped = set(DRAFTS) if drafts else set()
    safe.require(current['clock'] == old['clock'] + (50 if drafts else 52), 'Explicit executed fixture clock only')
    for name, before in old['attempts'].items():
        after = current['attempts'][name]
        if before['state']['kind'] != 'Script' or name in skipped:
            safe.require(after == before, 'Unchanged non-script/authentic draft: ' + name)
            continue
        row = by_id[name]
        safe.require(stable_metadata(after) == stable_metadata(before), 'All other learner metadata retained: ' + name)
        safe.require(records(after, False) == records(before, True), 'Every genuine6..10 record retained: ' + name)
        safe.require(after['state'] == {'kind': 'Script', 'source': row['source'], 'run': row['record']},
                     'Only actually executed11 reference supplies current credit: ' + name)
        work = after['workspace']
        safe.require(work['checked'] is False and work['script_learning']['example_revision'] == 11
                     and work['script_learning']['example_result'] == row['example_outcome'], 'Actually rerun11 example only')
    counter = old['clock']
    for row in observations:
        name = row['id']
        if name in skipped:
            continue
        counter += 1
        if name in NEW:
            a = current['attempts'][name]
            safe.require(a['state'] == {'kind': 'Script', 'source': row['source'], 'run': row['record']}
                         and a['step'] == 0 and a['hints'] == 0 and a['challenge'] is False
                         and a['transcript'] == [] and a['visited'] == counter,
                         'New synthetic script context follows real fixed execution only')
            work = a['workspace']
            safe.require(work['script_learning']['example_revision'] == 11
                         and work['script_learning']['example_source'] == row['example_source']
                         and work['script_learning']['example_result'] == row['example_outcome']
                         and not any(k.startswith(('cron_', 'etherchannel_', 'nfs_')) for k in work),
                         'No fabricated other-family metadata/mastery')
    safe.require(not set(index['non_script_absent_not_invented']) & set(current['attempts']), 'Never invent22 later non-script contexts')


def verify_capture(root, expected_manifest):
    safe.require(isinstance(expected_manifest, str) and len(expected_manifest) == 64, 'Actual reviewed11 manifest pin required')
    folder = root / DESTINATION
    raw = safe.read_bytes(folder / 'MANIFEST.json', CAP)
    safe.require(sha(raw) == expected_manifest and safe.read_bytes(folder / 'COMPLETE', 65) == (expected_manifest + '\n').encode(),
                 'Exact actual11 manifest and completion marker')
    manifest = strict(raw)
    for key, value in {'actual_evaluator_revision': 11, 'catalog_contexts': 164, 'script_records': 52,
                       'authored_cases': 384, 'examples': 52}.items():
        safe.require(manifest[key] == value and not isinstance(manifest[key], bool), 'Closed actual11 scope')
    safe.require(manifest['prior_actual10_manifest_sha256'] == PRIOR_SHA and manifest['source_index_sha256'] == INDEX_SHA,
                 'Actual11 provenance is linked to authentic10 and static authored inputs')
    seen = set()
    for pin in manifest['outputs']:
        name = pin['file']
        safe.require(name not in seen and len(seen) < 128, 'Unique bounded capture outputs')
        seen.add(name)
        observed = safe.pin(contained(folder, name), CAP)
        safe.require({k: observed[k] for k in ['bytes', 'sha256']} == {k: pin[k] for k in ['bytes', 'sha256']},
                     'Actual11 output changed: ' + name)
    required = {'SOURCE.json', 'input10-original.json', 'input10-drafts.json', 'actual11-authored.jsonl',
                'writer11-original.json', 'writer11-drafts.json',
                'original-profile/progress-v1.json', 'original-profile/progress-v1.before-shell-v11.json',
                'draft-profile/progress-v1.json', 'draft-profile/progress-v1.before-shell-v11.json'}
    safe.require(seen == required, 'Exact closed10 capture output identities')
    # After attestation, verify frozen input bytes, never a later evaluator source.
    frozen = folder / 'compiled-source'
    index = index_value(safe.read_bytes(frozen / INDEX if frozen.exists() else root / INDEX, CAP))
    if frozen.exists():
        source = strict(safe.read_bytes(folder / 'SOURCE.json', CAP))
        safe.require(source['actual_evaluator_revision'] == 11 and source['source_index_sha256'] == INDEX_SHA
                     and 1 <= len(source['inputs']) <= 1500, 'Frozen actual11 source identity')
        names = set()
        for pin in source['inputs']:
            name = pin['file']
            safe.require(name not in names, 'Unique frozen source input'); names.add(name)
            observed = safe.pin(contained(frozen, name), CAP)
            safe.require({k: observed[k] for k in ['bytes', 'sha256']} == {k: pin[k] for k in ['bytes', 'sha256']},
                         'Frozen source changed: ' + name)
    inputs = input10(root, index)
    observations = authored(safe.read_bytes(folder / 'actual11-authored.jsonl', CAP), index)
    for role, drafts in [('original', False), ('drafts', True)]:
        original_raw, original = inputs[role]
        safe.require(safe.read_bytes(folder / ('input10-' + role + '.json'), CAP) == original_raw, 'Actual10 inputs copied exactly')
        current_raw = safe.read_bytes(folder / ('writer11-' + role + '.json'), CAP)
        current = strict(current_raw)
        profile(original, current, index, observations, drafts)
        directory = folder / ('draft-profile' if drafts else 'original-profile')
        safe.require(safe.read_bytes(directory / 'progress-v1.json', CAP) == current_raw
                     and safe.read_bytes(directory / 'progress-v1.before-shell-v11.json', CAP) == original_raw,
                     'Current11 main/actual10 backup exact')
    return manifest


def source_set(root):
    names = [SOURCE, INDEX, SELF, TEST, 'examples/support/automation_digest.rs', 'tests/support/actual10_evidence.rs',
             'tools/parameters_before11_verify.py', 'tools/case_bash_oracle.py', 'tools/automation_before10_oracle.py',
             'tools/native_reopen.py', 'tools/dev.py', 'tools/build_safety.py', 'docs/BUILD_WORKFLOW.md', 'Cargo.toml', 'Cargo.lock',
             'tests/support/actual9_evidence.rs', 'examples/automation_before10_capture.rs', 'examples/parameters_before11_capture.rs']
    names += sorted(p.relative_to(root).as_posix() for directory in ['src', 'scenarios'] for p in (root / directory).rglob('*') if p.is_file())
    return [(root / n, CAP) for n in names] + [(root / PRODUCER, BINARY_CAP)]


def pins(paths):
    rows = [safe.pin(p, cap) for p, cap in paths]
    safe.require(sum(p['bytes'] for p in rows) <= BINARY_CAP + TOTAL, 'Fixed source/binary aggregate')
    return rows


def current11(root):
    raw = safe.read_bytes(root / 'src/shell.rs', 512 * 1024)
    safe.require(re.findall(rb'^pub const EVALUATOR_REVISION: u32 = ([0-9]+);$', raw, re.MULTILINE) == [b'11'],
                 'New before12 capture requires actual current evaluator11 source; never relabel later semantics')


def approved(raw, control):
    value = strict(raw)
    safe.require(value.get('root_reviewed') is True and value == {**control, 'root_reviewed': True}, 'Exact Root-reviewed current11 control required')


def attest(root, manifest_sha, before, executable_pin):
    folder = root / DESTINATION
    destination = safe.private_dir(folder, 'compiled-source')
    budget = safe.Budget(ATTEST_TOTAL)
    source = strict(safe.read_bytes(folder / 'SOURCE.json', CAP))
    safe.require(source['actual_evaluator_revision'] == 11 and source['source_index_sha256'] == INDEX_SHA, 'Actual producer source statement')
    for pin in source['inputs']:
        name = pin['file']
        original = contained(root, name)
        raw = safe.read_bytes(original, CAP)
        safe.require(sha(raw) == pin['sha256'] and len(raw) == pin['bytes'], 'Source changed before attestation')
        target = contained(destination, name)
        parent = destination
        for component in Path(name).parts[:-1]:
            try:
                parent = safe.private_dir(parent, component)
            except FileExistsError:
                parent = safe.private_dir(parent, component, existing=True)
        safe.write_new(target, raw, budget, CAP)
    # Keep the actual11 executable itself; never rebuild an old producer under12.
    with safe.ordinary(root / PRODUCER, BINARY_CAP) as (stream, info):
        safe.require(safe.pin(root / PRODUCER, BINARY_CAP) == executable_pin, 'Actual11 producer changed')
        out = folder / 'actual11-producer'
        budget.charge(info.st_size)
        with safe.new_file(out) as target:
            left = info.st_size
            while left:
                data = stream.read(min(left, 1024 * 1024)); safe.require(data, 'Producer shrank'); target.write(data); left -= len(data)
            target.flush(); os.fsync(target.fileno())
        out.chmod(0o700)
        safe.require(safe.pin(out, BINARY_CAP)['sha256'] == executable_pin['sha256'], 'Retained actual11 producer differs')
    report = {'actual11_manifest_sha256': manifest_sha, 'producer': executable_pin, 'reviewed_pins': before,
              'records_regenerated': False, 'scope': 'Current11 actual capture; not yet sealed for future12 until Root independent review'}
    safe.json_new(folder / 'ATTESTATION.json', report, safe.Budget(CAP), CAP)


def require_seals():
    safe.require(all(isinstance(value, str) and re.fullmatch(r'[0-9a-f]{64}', value)
                     for value in [SEALED_MANIFEST, SEALED_ATTESTATION]),
                 'No genuine11 capture has been sealed yet; do not advance evaluator')


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--mode', required=True, choices=['prepare', 'capture', 'verify'])
    args = parser.parse_args(argv)
    root = ROOT.resolve(strict=True)
    safe.require((root / '.git').is_dir() and not (root / '.git').is_symlink(), 'MAIN checkout only')
    inherited = safe.require_guard(root)
    safe.require(os.environ.get('CARGO_TARGET_DIR') == str(root / 'target'), 'MAIN guarded target required')
    if args.mode == 'verify':
        require_seals()
        verify_capture(root, SEALED_MANIFEST)
        attestation_raw = safe.read_bytes(root / DESTINATION / 'ATTESTATION.json', CAP)
        safe.require(sha(attestation_raw) == SEALED_ATTESTATION, 'Reviewed actual11 attestation changed')
        report = strict(attestation_raw)
        safe.require(report['actual11_manifest_sha256'] == SEALED_MANIFEST and report['records_regenerated'] is False,
                     'Exact actual11 provenance')
        retained = safe.pin(root / DESTINATION / 'actual11-producer', BINARY_CAP)
        safe.require({k: retained[k] for k in ['bytes', 'sha256']} == {k: report['producer'][k] for k in ['bytes', 'sha256']},
                     'Original actual11 executable changed')
        print(json.dumps({'status': 'SEALED-ACTUAL11-VERIFIED', 'manifest_sha256': SEALED_MANIFEST}))
        return 0
    current11(root)
    index = load_index(root)
    input10(root, index)
    paths = source_set(root); before = pins(paths); head = safe.source_identity(root)
    from build_safety import owned_directory
    base = owned_directory(root, root / HERE)
    safe.require(not (root / DESTINATION).exists(), 'Preserve existing genuine capture; no overwrite')
    control = {'schema_version': 1, 'root_reviewed': False, 'source_head': head, 'pins': before,
               'actual_evaluator_revision': 11, 'index_sha256': INDEX_SHA, 'prior_actual10_manifest_sha256': PRIOR_SHA,
               'profile_contexts': 142, 'new_script_contexts': NEW, 'claims': 'Synthetic actual11 observations, never personal progress or learner mastery'}
    if args.mode == 'prepare':
        safe.json_new(base / 'PROPOSED-before12-r2.json', control, safe.Budget(CAP), CAP)
        print(json.dumps({'status': 'PROPOSED-NO-EXECUTION'})); return 0
    approved(safe.read_bytes(base / 'APPROVED-before12-r2.json', CAP), control)
    run = safe.private_dir(base, 'before12-run-r2'); budget = safe.Budget(TOTAL)
    temporary = safe.private_dir(run, 'tmp')
    environment = safe.closed_environment({}, temporary)
    environment.update(DLMS_FIXED_CAPTURE11='before12-r1', CARGO_TARGET_DIR=str(root / 'target'))
    executable_pin = next(p for p in before if p['path'] == str(root / PRODUCER))
    with pinned_executable(root / PRODUCER, executable_pin, BINARY_CAP) as (executable, fd):
        result = safe.capture_command([executable], run, root, environment, b'', budget, 65536,
            timeout=90, inherited=(*inherited, fd))
    safe.quiet_group()
    safe.require(result['status'] == 0 and not result['stderr'] and result['stdout'].startswith('PASS genuine actual11 before12:'),
                 'Actual current11 producer did not finish; preserve partial evidence')
    safe.require(pins(paths) == before and safe.source_identity(root) == head, 'Compiled source/binary changed during actual11 capture')
    manifest_sha = sha(safe.read_bytes(root / DESTINATION / 'MANIFEST.json', CAP))
    verify_capture(root, manifest_sha)
    attest(root, manifest_sha, before, executable_pin)
    print(json.dumps({'status': 'ACTUAL11-CAPTURED-REVIEW-AND-SEAL-REQUIRED', 'manifest_sha256': manifest_sha,
                      'records': 52, 'cases': 384, 'examples': 52, 'contexts': 142, 'draft_current_records': 50}))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
