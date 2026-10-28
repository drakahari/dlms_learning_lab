#!/usr/bin/env python3
"""Fixed54 original +5 balanced Bash observations with one exact invalid-source amendment, never learner source or a VM engine.
Metadata preparation performs no Bash execution. Capture requires Root-reviewed
same-source approval. CLI accepts only a simple capture identity and prepare flag.
This source proposal is not executable through the runner until reviewed/registered.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import time
import case_bash_oracle as safe
from automation_before10_oracle import pinned_executable

ROOT = Path(__file__).resolve().parents[1]
HERE = '.build-artifacts/reviews/bash-parameter-defaults/bash-captures'
FIXED = {
    'tests/fixtures/bash-parameter-defaults-reference.json':
        ('84241e2ecf944ae7976bdaabab1ccdcc9e4ddab209d1febb446b48f1ff30cb7e', 35),
    'tests/fixtures/bash-colon-reference.json': ('19ec89a7cd22504084bbfb285cb42f0b4557c5d44a61126db884dcc40189a5a9', 6),
    'tests/fixtures/bash-cat-reference.json': ('63c9a584737293b40cd9e200e15ef3180efabf6d1a33fad75e236168edcf197c', 13),
    'tests/fixtures/bash-parameter-arithmetic-reference.json': ('4469272da8e4f83e843b519f35afcf0fc5c78274bb2a53f63a650ee307a02150', 5),
}
CAP = 2 * 1024 * 1024
TOTAL = 16 * 1024 * 1024
SORT = Path('/usr/bin/sort')
CAT = Path('/usr/bin/cat')
AMENDMENT = 'tests/fixtures/bash-parameter-arithmetic-amendment.json'
AMENDMENT_SHA = '6c8fe424bf6f1007e02f383e043039c2995c9c6b60ffe077f9181a374424872e'
PRIOR = HERE + '/parameters54-r1'
PRIOR_MANIFEST_SHA = '86b789a47debdccbbbcab130fe40cacd16ac44ca049d593b9bf8bf6a6de8bbad'
PRIOR_OBSERVATION_SHA = 'a88b399953a9bb660a1492cd565c8e34e009ebe85de3486379dea775833169ab'
INVALID_SOURCE_SHA = '593fae1c7c01f2ac3f1065cbeba04dd86c048d223a9c9ff26e7cf2a0562828f9'
INVALID_CASE_SHA = 'c283d6f39ff550bab7f28a88dee35499f16cdc05ddf7b6d80bc4cc8a0434149e'
INVALID_NAME = 'arithmetic ignores parentheses in unselected literal'


def strict(data):
    def pairs(items):
        value = {}
        for key, item in items:
            safe.require(key not in value, 'Duplicate JSON key')
            value[key] = item
        return value
    return json.loads(data, object_pairs_hook=pairs)


def identity(text):
    safe.require(isinstance(text, str) and re.fullmatch(r'[A-Za-z0-9_-]{1,64}', text),
                 'Simple fixed capture identity required')
    return text


def load_fixed(root):
    rows, inputs, names = [], [], set()
    for path, (expected, count) in FIXED.items():
        raw = safe.read_bytes(root / path, CAP)
        safe.require(hashlib.sha256(raw).hexdigest() == expected, 'Fixed reviewed inputs changed: ' + path)
        group = strict(raw)
        safe.require(isinstance(group, list) and len(group) == count, 'Exact fixed input count')
        for row in group:
            safe.require(set(row) == {'name', 'source', 'case'}, 'Closed fixed row shape')
            name, source, case = row['name'], row['source'], row['case']
            safe.require(isinstance(name, str) and 0 < len(name) <= 128 and name not in names, 'Unique fixed identity')
            names.add(name)
            safe.require(isinstance(source, str) and 0 < len(source.encode()) <= 16384 and '\0' not in source,
                         'Fixed finite source only')
            safe.require(set(case) == {'name', 'args', 'stdin', 'env', 'files', 'stdout', 'stderr', 'status', 'after'},
                         'Closed fixed case shape')
            safe.require(case['name'] == name and len(case['args']) <= 16 and all(
                isinstance(v, str) and '\0' not in v and len(v.encode()) <= 4096 for v in case['args']), 'Finite fixed argv')
            safe.require(isinstance(case['stdin'], str) and '\0' not in case['stdin'] and len(case['stdin'].encode()) <= 4096,
                         'Finite fixed standard input')
            safe.closed_environment(case['env'], Path('/fixed-placeholder'))
            safe.require(set(case['env']) <= {'value', 'destination', 'FILE'}, 'Only reviewed ordinary fixture names')
            for files in [case['files'], case['after']]:
                safe.require(isinstance(files, dict) and len(files) <= 8, 'Fixed file count')
                for name, text in files.items():
                    flat_relative(name)
                    safe.require(len(Path(name).parts) == 3 and isinstance(text, str) and '\0' not in text
                                 and len(text.encode()) <= 4096, 'Flat bounded disposable files')
            rows.append(row)
        inputs.append({'file': path, 'bytes': len(raw), 'sha256': expected})
    safe.require(len(rows) == 59, 'Closed59 fixed recipes')
    return rows, inputs


def flat_relative(name):
    safe.require(isinstance(name, str) and name.startswith('/work/'), 'Only reviewed virtual /work names')
    value = name[len('/work/'):]
    safe.require(value not in ('', '.', '..') and '/' not in value and '\0' not in value
                 and len(value.encode()) <= 250, 'One bounded flat disposable name')
    return Path(value)


def materialize(work, files, budget):
    total = sum(len(text.encode()) for text in files.values())
    safe.require(total <= 32768 and len(files) <= 8, 'Fixed initial aggregate/count')
    for name, text in files.items():
        safe.write_new(work / flat_relative(name), text.encode(), budget, 4096)
    return total


def final_files(work, allowed):
    import stat
    value, total = {}, 0
    with os.scandir(work) as entries:
        for entry in entries:
            safe.require(len(value) < 16 and entry.name in allowed, 'Only closed reviewed output names')
            info = entry.stat(follow_symlinks=False)
            safe.require(stat.S_ISREG(info.st_mode) and info.st_size <= 4096, 'Ordinary fixed output file only')
            path = work / flat_relative('/work/' + entry.name)
            raw = safe.read_bytes(path, 4096)
            total += len(raw)
            safe.require(total <= 65536, 'Fixed final aggregate')
            value['/work/' + entry.name] = raw.decode('utf-8', errors='strict')
    return dict(sorted(value.items())), total


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode()


def invalid_fingerprint(index, row):
    safe.require(index == 32 and row['name'] == INVALID_NAME
                 and digest(row['source'].encode()) == INVALID_SOURCE_SHA
                 and digest(canonical(row['case'])) == INVALID_CASE_SHA,
                 'Only exact original case032 source/input can use the amendment')


def load_amendment(root, rows):
    raw = safe.read_bytes(root / AMENDMENT, CAP)
    safe.require(digest(raw) == AMENDMENT_SHA, 'Immutable amendment source changed')
    amendment = strict(raw)
    safe.require(amendment['schema_version'] == 1 and amendment['original_index'] == 32
                 and amendment['name'] == INVALID_NAME
                 and amendment['source_sha256'] == INVALID_SOURCE_SHA
                 and amendment['case_canonical_sha256'] == INVALID_CASE_SHA,
                 'Closed amendment identity changed')
    safe.require(len(rows) == 59, 'Exact amended59 input order required')
    invalid_fingerprint(32, rows[32])
    safe.require(rows[32]['case']['stdout'] == amendment['original_expected_stdout'] == '8\n',
                 'Incorrect original expectation must remain explicit and unmodified')
    folder = root / PRIOR
    manifest_raw = safe.read_bytes(folder / 'MANIFEST.json', CAP)
    safe.require(digest(manifest_raw) == PRIOR_MANIFEST_SHA, 'Original failed-reference manifest changed')
    safe.require(safe.read_bytes(folder / 'COMPLETE', 65) == (PRIOR_MANIFEST_SHA + '\n').encode(),
                 'Original failed-reference COMPLETE changed')
    manifest = strict(manifest_raw)
    safe.require(manifest['case_rows'] == 54 and manifest['bash_only'] is True,
                 'Original54 observation scope changed')
    path = folder / 'case-032/OBSERVATION.json'
    pin = next(p for p in manifest['outputs'] if p['path'] == str(path))
    safe.require(pin['sha256'] == PRIOR_OBSERVATION_SHA, 'Original observation pin changed')
    observed_raw = safe.read_bytes(path, CAP)
    safe.require(len(observed_raw) == pin['bytes'] and digest(observed_raw) == PRIOR_OBSERVATION_SHA,
                 'Original invalid-source observation changed')
    previous = strict(observed_raw)
    invalid_fingerprint(previous['index'], previous)
    safe.require(previous['source'] == rows[32]['source'] and previous['case'] == rows[32]['case']
                 and previous['expected_matches'] is False, 'Original false expectation must stay false')
    actual = previous['actual_bash']
    safe.require(actual['status'] == 1 and actual['stdout'] == ''
                 and 'bad substitution: no closing' in actual['stderr']
                 and actual['files'] == rows[32]['case']['files'], 'Original real Syntax evidence differs')
    safe.require(amendment['actual_bash']['capture_manifest_sha256'] == PRIOR_MANIFEST_SHA
                 and amendment['actual_bash']['observation_sha256'] == PRIOR_OBSERVATION_SHA,
                 'Amendment no longer names the immutable real observation')
    return {'identity': {'amendment_sha256': AMENDMENT_SHA,
            'prior_manifest_sha256': PRIOR_MANIFEST_SHA, 'prior_observation_sha256': PRIOR_OBSERVATION_SHA,
            'index': 32, 'name': INVALID_NAME, 'source_sha256': INVALID_SOURCE_SHA,
            'case_canonical_sha256': INVALID_CASE_SHA}, 'original_actual': actual}


def original_expected(row):
    return {'stdout': row['case']['stdout'], 'stderr': row['case']['stderr'],
            'status': row['case']['status'], 'files': row['case']['after']}


def classify(index, row, actual, policy):
    # Preserve the old expected_matches value even for the incorrect original8.
    expected_matches = actual == original_expected(row)
    if index == 32:
        invalid_fingerprint(index, row)
        # Stable -c argv and two-line helper prelude make exact old diagnostics
        # available; do not waive arbitrary stderr/status/file changes.
        confirmed = not expected_matches and actual == policy['original_actual']
        return expected_matches, 'confirmed-invalid' if confirmed else 'unexpected-mismatch'
    return expected_matches, 'supported-match' if expected_matches else 'unexpected-mismatch'


def validation_summary(counts):
    safe.require(set(counts) == {'supported-match', 'confirmed-invalid', 'unexpected-mismatch'}
                 and all(isinstance(n, int) and not isinstance(n, bool) and n >= 0 for n in counts.values()),
                 'Closed observation partition required')
    return {'supported_matches': counts['supported-match'],
            'confirmed_invalid': counts['confirmed-invalid'],
            'unexpected_mismatches': counts['unexpected-mismatch'],
            'accepted_partition': counts == {'supported-match': 58, 'confirmed-invalid': 1,
                                             'unexpected-mismatch': 0}}


def source_set(root):
    paths = [(root / p, CAP) for p in [*FIXED, AMENDMENT,
        PRIOR + '/MANIFEST.json', PRIOR + '/COMPLETE', PRIOR + '/case-032/OBSERVATION.json',
        'tools/tests/test_parameter_bash_oracle.py',
        'tools/parameter_bash_oracle.py', 'tools/case_bash_oracle.py',
        'tools/automation_before10_oracle.py', 'tools/dev.py', 'tools/build_safety.py']]
    paths += [(safe.MANUAL.resolve(strict=True), 8 * CAP),
              (safe.BASH.resolve(strict=True), 256 * CAP), (SORT.resolve(strict=True), 256 * CAP),
              (CAT.resolve(strict=True), 256 * CAP)]
    return paths


def pins(paths):
    return [safe.pin(path, cap) for path, cap in paths]


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--capture-id', required=True, type=identity)
    parser.add_argument('--prepare', action='store_true')
    args = parser.parse_args(argv)
    root = ROOT.resolve(strict=True)
    safe.require((root / '.git').is_dir() and not (root / '.git').is_symlink(), 'MAIN checkout only')
    inherited = safe.require_guard(root)
    safe.require(os.environ.get('CARGO_TARGET_DIR') == str(root / 'target'), 'MAIN guarded target identity required')
    rows, inputs = load_fixed(root)
    policy = load_amendment(root, rows)
    allowed_files = {flat_relative(name).name for row in rows for files in [row['case']['files'], row['case']['after']] for name in files}
    paths = source_set(root)
    before = pins(paths)
    manual_pin = next(p for p in before if p['path'] == str(safe.MANUAL.resolve(strict=True)))
    safe.require(manual_pin['sha256'] == safe.MANUAL_SHA, 'Original GNU manual changed')
    head = safe.source_identity(root)
    from build_safety import owned_directory
    base = owned_directory(root, root / HERE)
    control = {'schema_version': 1, 'root_reviewed': False, 'capture_id': args.capture_id,
               'source_head': head, 'pins': before, 'inputs': inputs, 'amendment': policy['identity'],
               'claims': 'Bash-only fixed observations; no VM or learner source accepted'}
    if args.prepare:
        result = safe.json_new(base / ('PROPOSED-' + args.capture_id + '.json'), control, safe.Budget(CAP), CAP)
        print(json.dumps({'status': 'PROPOSED-ONLY', 'pin': result}))
        return 0
    approved = strict(safe.read_bytes(base / ('APPROVED-' + args.capture_id + '.json'), CAP))
    wanted = {**control, 'root_reviewed': True}
    safe.require(approved.get('root_reviewed') is True and approved == wanted, 'Exact independently reviewed approval required')
    out = safe.private_dir(base, args.capture_id)
    budget = safe.Budget(TOTAL)
    outputs = [safe.json_new(out / 'STARTED.json', control, budget, CAP)]
    began = time.monotonic()
    categories = {'supported-match': 0, 'confirmed-invalid': 0, 'unexpected-mismatch': 0}
    bash_path, sort_path, cat_path = safe.BASH.resolve(strict=True), SORT.resolve(strict=True), CAT.resolve(strict=True)
    bpin = next(p for p in before if p['path'] == str(bash_path))
    spin = next(p for p in before if p['path'] == str(sort_path))
    cpin = next(p for p in before if p['path'] == str(cat_path))
    with pinned_executable(bash_path, bpin, 256 * CAP) as (bash, bash_fd), \
         pinned_executable(sort_path, spin, 256 * CAP) as (sort, sort_fd), \
         pinned_executable(cat_path, cpin, 256 * CAP) as (cat, cat_fd):
        retained = (*inherited, bash_fd, sort_fd, cat_fd)
        version_dir = safe.private_dir(out, 'version')
        work, temp = safe.private_dir(version_dir, 'work'), safe.private_dir(version_dir, 'tmp')
        version = safe.capture_command([bash, '--noprofile', '--norc', '--version'], version_dir, work,
            safe.closed_environment({}, temp), b'', budget, safe.VERSION_CAP, inherited=retained)
        safe.quiet_group()
        safe.require(version['status'] == 0 and not version['stderr'] and
            re.match(r'^GNU bash, version 5\.3\.9\b', version['stdout']), 'Actual reviewed Bash5.3.9 required')
        outputs.append(safe.json_new(out / 'VERSION.json', version, budget))
        outputs += [safe.pin(version_dir / n, safe.VERSION_CAP) for n in ['stdout.raw', 'stderr.raw']]
        # PATH stays empty. Only the two reviewed utilities are available, through
        # retained/pinned descriptors. No general host command lookup is enabled.
        prelude = 'sort() { ' + sort + ' "$@"; }\ncat() { ' + cat + ' "$@"; }\n'
        for index, row in enumerate(rows):
            safe.require(time.monotonic() - began < 120, 'Whole fixed oracle deadline')
            folder = safe.private_dir(out, f'case-{index:03d}')
            work, temp = safe.private_dir(folder, 'work'), safe.private_dir(folder, 'tmp')
            initial = materialize(work, row['case']['files'], budget)
            observed = safe.capture_command([bash, *safe.BASH_FLAGS, '-c', prelude + row['source'],
                'fixed-parameter-' + str(index), *row['case']['args']], folder, work,
                safe.closed_environment(row['case']['env'], temp), row['case']['stdin'].encode(), budget,
                safe.STREAM_CAP, inherited=retained)
            safe.quiet_group()
            safe.require(observed['status'] >= 0, 'Unexpected signal termination')
            after, count = final_files(work, allowed_files)
            budget.replace_reservation(initial, count)
            observed['files'] = after
            expected_matches, classification = classify(index, row, observed, policy)
            categories[classification] += 1
            record = {'index': index, 'name': row['name'], 'source': row['source'],
                      'case': row['case'], 'actual_bash': observed, 'expected_matches': expected_matches, 'classification': classification,
                      'version': version['stdout'].splitlines()[0], 'utility_helper_source': prelude,
                      'virtual_machine_executed': False}
            outputs.append(safe.json_new(folder / 'OBSERVATION.json', record, budget, CAP))
            outputs += [safe.pin(folder / n, safe.STREAM_CAP) for n in ['stdout.raw', 'stderr.raw']]
            outputs += [safe.pin(work / flat_relative(name), safe.FILE_CAP) for name in after]
    safe.quiet_group()
    safe.require(pins(paths) == before and safe.source_identity(root) == head, 'Reviewed identity changed')
    for output in outputs:
        safe.require(safe.pin(Path(output['path']), TOTAL) == output, 'Retained observation changed')
    safe.require(time.monotonic() - began < 120, 'Whole oracle final deadline')
    partition = validation_summary(categories)
    manifest = {'schema_version': 1, 'source_head': head, 'pins': before, 'inputs': inputs, 'amendment': policy['identity'],
                'case_rows': 59, 'bash_only': True, 'partition': partition, 'outputs': outputs, 'no_vm_or_progress_produced': True}
    safe.publish(out, manifest, budget)
    print(json.dumps({'status': 'COMPLETE-OBSERVATIONS', 'path': str(out), 'rows': 59, **partition}))
    return 0 if partition['accepted_partition'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
