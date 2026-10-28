#!/usr/bin/env python3
"""Closed114 optional-input lesson comparisons; never learner source.

The original117 recipes stay byte-exact. Three absolute-destination guard rows
remain explicitly simulation-only. No arbitrary program/input/path CLI exists.
Preparation launches nothing; capture requires independently reviewed exact pins.
"""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import stat
import subprocess
import time

import case_bash_oracle as safe
from automation_before10_oracle import pinned_executable

ROOT = Path(__file__).resolve().parents[1]
FIXED = 'tests/fixtures/shell-optional-inputs-reference117.json'
FIXED_SHA = 'e27b80313df523058f9c8c12d2a02c39770564e0ffc21b57e226a72d8480a584'
ROWS_SHA = 'fe5919acb71f26ca1fdf2a08b7c94400b38388dab72a227cc29b975b5698eaae'
ADMITTED_SHA = '1a1ac1e49b253c2080e4b4f04ea866924c6268dba3b1dd4cbc88dd082dc4d741'
PACK = 'scenarios/shell-optional-inputs-v1.json'
PACK_SHA = '30d72ae7cb0bbeabcc9c4a1e0268103c50ce58355198f69afe7cd6e5a61d5678'
REVIEW = 'tests/fixtures/shell-optional-inputs-review.json'
REVIEW_SHA = '62ef4778223586cc9879f904c610a024f65f8c84fd4e8c683d277e40bb827aee'
PROGRAMS = {
    '13da8b8018a180075e4e10d6340128e2b259a453de211942660d074f5ec4b980',
    '14aecfb29bb8bdc0ae1268b5df0690c7430d62f2daa299438bf29508abcd947a',
    '14ed23eafc7fb0aa3a5225d9282e0b1863d9ebabb31bb9e2568b4c3f0a1eefcc',
    '25181ebf55eeecf41765ea3b9eec3f2b7afc6a46e97dfa6106ef9813f4e79e3c',
    '273f6598129bb0ea71fcb87311ba6193ebe221317a8d6deb0441c8f92684b97b',
    '3acad23ca31468e84c81a44033325d1a58f2af38a4228c0dc4b9e73df1052f4d',
    '4309f84a8a0c3dc4fcb1644eb6599898a266cc6b94294850567925e7cec3a237',
    '597b9263c61d8d9796061c4b9a2f8d2f1a1b0cb97205d27d7509db715662b4dc',
    '800a2739ae4a766f08711e6d064aa43c4e3079f38839d058c05a0b7d24b68a89',
    'a3d73f2cee06b8da05a5e81963634d1d878713685c7dda9cd61eb44b29d9a8e4',
    'ac8a7e46e3eb28077b0f13f4a5f9f1dfbaab325665ccb9544e4b4ca933297507',
    'b4d422168b9207588b8953d98fc0039a19023e051347c20f3d2a1404923d3bc9',
    'e0de570ffe440ed07c378b8189875ccffcae073ea69e3dea60d3cfb2fa8b81de',
    'e43dc904a61b85469c7a433395b3c5fb4b8f9610256dff5f0088e73d5165aade',
    'f8294a1fca2bf93b1a9d71ae5d8f13e6009d5362c2f51b4f47c276ed3e655dfb',
}
EXCLUDED = {
    'shell-optional-report/reference/guard00',
    'shell-optional-report/alternate1/guard00',
    'shell-optional-report/alternate2/guard00',
}
ABS_CASE_SHA = '6c088910e1728e9f6cc05ab4a0e44a567908c6d32f4089e9788781f45d21f9f6'
ENV_KEYS = {'TEAM', 'SUBTITLE', 'NOTICE', 'ROOM', 'CREDIT', 'NAME1', 'NAME2',
            'NOTE1', 'NOTE2', 'BASE_TEAM', 'BASE_NOTE', 'TEAM1', 'TEAM2'}
HERE = '.build-artifacts/reviews/bash-parameter-defaults/lesson-bash-captures'
PROBE = 'target/debug/examples/optional_inputs_reference'
PROBE_SOURCE = 'examples/optional_inputs_reference.rs'
SELF = 'tools/optional_inputs_bash_oracle.py'
TEST = 'tools/tests/test_optional_inputs_bash_oracle.py'
CAT = Path('/usr/bin/cat')
BASH_SHA = '379c1bfc53d08815975c647ad7da5d19d8e8a8566ce8097c50231c8b116e5e7b'
CAT_SHA = '73f19135b6cfb36142a19345557c48bdcc7d0d3b414b745be86e4458dfe5845b'
CAP = 2 * 1024 * 1024
TOTAL = 32 * 1024 * 1024
BINARY_CAP = 512 * 1024 * 1024
VM_STREAM = 1024 * 1024
WHOLE_SECONDS = 120


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode()


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def strict(raw):
    def pairs(items):
        value = {}
        for key, item in items:
            safe.require(key not in value, 'Duplicate JSON key')
            value[key] = item
        return value
    return json.loads(raw, object_pairs_hook=pairs)


def identity(text):
    safe.require(isinstance(text, str) and re.fullmatch(r'[A-Za-z0-9_-]{1,64}', text),
                 'Simple fixed capture identity required')
    return text


def flat(name):
    safe.require(isinstance(name, str) and name.startswith('/work/'), 'Fixed virtual /work prefix required')
    tail = name[6:]
    safe.require(tail not in ('', '.', '..') and '/' not in tail and '\\' not in tail
                 and len(tail.encode()) <= 250
                 and not any(ord(c) < 32 or ord(c) == 127 for c in tail),
                 'One flat non-control disposable filename required')
    return tail


def check_case(row, excluded=False):
    safe.require(set(row) == {'id', 'role', 'program_role', 'lab_id', 'source', 'case'}, 'Closed fixed row shape')
    safe.require(isinstance(row['source'], str) and digest(row['source'].encode()) in PROGRAMS,
                 'Only15 independently reviewed fixed programs')
    c = row['case']
    safe.require(set(c) == {'name', 'args', 'stdin', 'env', 'files', 'stdout', 'stderr', 'status', 'after'},
                 'Closed case shape')
    safe.require(isinstance(c['args'], list) and len(c['args']) <= 3 and c['stdin'] == '', 'Fixed finite argv/empty stdin')
    for arg in c['args']:
        safe.require(isinstance(arg, str) and len(arg.encode()) <= 256
                     and not any(ord(x) < 32 or ord(x) == 127 for x in arg), 'Finite non-control argv')
        safe.require(excluded or ('/' not in arg or arg == './data.txt'), 'Outside-CWD argv forbidden before launch')
        safe.require(arg not in ('.', '..') and '\\' not in arg, 'Dot/parent/platform path forbidden')
    safe.closed_environment(c['env'], Path('/fixed-private-placeholder'))
    safe.require(set(c['env']) <= ENV_KEYS, 'Only13 reviewed data variables')
    for scope in ('files', 'after'):
        safe.require(isinstance(c[scope], dict) and len(c[scope]) <= 8, 'Fixed flat file count')
        total = 0
        for name, text in c[scope].items():
            flat(name)
            safe.require(isinstance(text, str) and '\0' not in text and len(text.encode()) <= 4096, 'Fixed UTF-8 file ceiling')
            total += len(text.encode())
        safe.require(total <= 32768, 'Fixed file aggregate')
    safe.require(c['status'] in (0, 2) and not isinstance(c['status'], bool), 'Normal fixed status only')
    safe.require(all(isinstance(c[k], str) and len(c[k].encode()) <= 4096 for k in ('stdout', 'stderr')), 'Fixed expected streams')
    if excluded:
        safe.require(row['id'] in EXCLUDED and digest(canonical(c)) == ABS_CASE_SHA
                     and c['args'] == ['data.txt', '/work/report.txt'], 'Only exact3 original absolute guards excluded')


def load_fixed(root):
    raw = safe.read_bytes(root / FIXED, CAP)
    safe.require(digest(raw) == FIXED_SHA, 'Original117 bytes changed')
    group = strict(raw)
    safe.require(set(group) == {'status', 'pack_sha256', 'review_sha256', 'rows'}
                 and group['pack_sha256'] == PACK_SHA and group['review_sha256'] == REVIEW_SHA, 'Original117 closed provenance')
    rows = group['rows']
    safe.require(isinstance(rows, list) and len(rows) == 117 and digest(canonical(rows)) == ROWS_SHA, 'Exact117 input order')
    safe.require(len({r['id'] for r in rows}) == 117, 'Unique117 identities')
    admitted, excluded, programs = [], [], set()
    for index, row in enumerate(rows):
        omit = row['id'] in EXCLUDED
        check_case(row, omit)
        programs.add(digest(row['source'].encode()))
        if omit:
            excluded.append({'original_index': index, 'id': row['id'], 'case_sha256': digest(canonical(row['case'])),
                             'reason': 'Absolute destination outside CWD; simulation-only, no host comparison'})
        else:
            admitted.append(row)
    safe.require(programs == PROGRAMS and len(excluded) == 3 and {x['id'] for x in excluded} == EXCLUDED,
                 'Exact15 programs and3 explicit exclusions')
    safe.require(len(admitted) == 114 and digest(canonical(admitted)) == ADMITTED_SHA, 'Exact114, no silent omission/remapping')
    safe.require(Counter(r['role'] for r in admitted) == {'assignment': 102, 'ungraded-practice': 9, 'noncredit-guard': 3},
                 'Complete9 assignment programs and separate ungraded examples/guards')
    for path, sha in [(PACK, PACK_SHA), (REVIEW, REVIEW_SHA)]:
        safe.require(digest(safe.read_bytes(root / path, CAP)) == sha, 'Authored source changed: ' + path)
    return raw, admitted, excluded


def materialize(work, files, budget):
    total = sum(len(v.encode()) for v in files.values())
    safe.require(total <= 32768 and len(files) <= 8, 'Initial fixed file aggregate')
    for name, text in files.items():
        safe.write_new(work / flat(name), text.encode(), budget, 4096)
    return total


def final_files(work, allowed):
    out, total = {}, 0
    with os.scandir(work) as entries:
        for entry in entries:
            safe.require(len(out) < 16 and entry.name in allowed, 'Unexpected fixture output name')
            info = entry.stat(follow_symlinks=False)
            safe.require(stat.S_ISREG(info.st_mode) and info.st_size <= 4096, 'Only finite ordinary output files')
            text = safe.read_bytes(work / flat('/work/' + entry.name), 4096)
            total += len(text)
            safe.require(total <= 65536, 'Final file aggregate')
            out['/work/' + entry.name] = text.decode('utf-8', errors='strict')
    return dict(sorted(out.items())), total


def bounded_popen(command, *, vm=False, **kwargs):
    # Child-only resource limits; never alter the desktop or detach from the
    # MAIN guard's owned process group. All commands/sources were fixed above.
    def limits():
        os.umask(0o077)
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
        resource.setrlimit(resource.RLIMIT_CPU, (10, 10) if vm else (2, 2))
        memory = (512 if vm else 256) * 1024 * 1024
        resource.setrlimit(resource.RLIMIT_AS, (memory, memory))
        resource.setrlimit(resource.RLIMIT_NOFILE, (64, 64))
        resource.setrlimit(resource.RLIMIT_FSIZE, (65536, 65536))
    safe.require('preexec_fn' not in kwargs, 'No caller-provided child setup')
    return subprocess.Popen(command, preexec_fn=limits, **kwargs)


def capture_child(command, folder, work, env, budget, *, inherited, vm=False):
    return safe.capture_command(command, folder, work, env, b'', budget,
        VM_STREAM if vm else safe.STREAM_CAP, timeout=10 if vm else safe.TIMEOUT,
        inherited=inherited, popen=lambda cmd, **kw: bounded_popen(cmd, vm=vm, **kw))


def expected(row):
    c = row['case']
    return {'stdout': c['stdout'], 'stderr': c['stderr'], 'status': c['status'], 'files': c['after']}


def vm_rows(text, rows):
    safe.require(len(text.encode()) <= VM_STREAM, 'VM stream bound')
    lines = text.splitlines()
    safe.require(len(lines) == 114, 'All114 actual VM results required')
    actual = []
    for line, row in zip(lines, rows):
        value = strict(line)
        safe.require(set(value) == {'id', 'source_sha256', 'case_sha256', 'outcome'}
                     and value['id'] == row['id']
                     and value['source_sha256'] == digest(row['source'].encode())
                     and value['case_sha256'] == digest(canonical(row['case'])), 'VM must use exact fixed source/case/order')
        result = value['outcome']
        safe.require(set(result) == {'stdout', 'stderr', 'status', 'files', 'error'} and result['error'] is None,
                     'Supported fixed VM programs must not produce simulator diagnostics')
        safe.require(isinstance(result['status'], int) and not isinstance(result['status'], bool)
                     and all(isinstance(result[k], str) and len(result[k].encode()) <= safe.STREAM_CAP
                             for k in ('stdout', 'stderr'))
                     and isinstance(result['files'], dict) and len(result['files']) <= 16,
                     'Bounded actual VM result types')
        for name, data in result['files'].items():
            flat(name)
            safe.require(isinstance(data, str) and '\0' not in data and len(data.encode()) <= 4096,
                         'Actual VM file content bound')
        actual.append({k: result[k] for k in ('stdout', 'stderr', 'status', 'files')})
    return actual


def comparison(row, vm, bash):
    authored = expected(row)
    return {'vm_matches_authored': vm == authored, 'bash_matches_authored': bash == authored,
            'vm_matches_bash': vm == bash, 'passed': vm == bash == authored}


def source_set(root):
    names = [FIXED, PACK, REVIEW, SELF, TEST, PROBE_SOURCE, 'examples/support/automation_digest.rs',
             'tools/case_bash_oracle.py', 'tools/automation_before10_oracle.py', 'tools/dev.py',
             'tools/build_safety.py', 'Cargo.toml', 'Cargo.lock']
    names += sorted(p.relative_to(root).as_posix() for p in (root / 'src').rglob('*.rs'))
    return [(root / p, CAP) for p in names] + [(root / PROBE, BINARY_CAP),
        (safe.MANUAL.resolve(strict=True), 8 * CAP), (safe.BASH.resolve(strict=True), CAP), (CAT.resolve(strict=True), CAP)]


def pins(paths):
    result = [safe.pin(p, cap) for p, cap in paths]
    safe.require(sum(p['bytes'] for p in result) <= BINARY_CAP + 64 * CAP, 'Source/binary aggregate pin bound')
    return result


def approved(raw, control):
    value = strict(raw)
    safe.require(value.get('root_reviewed') is True and value == {**control, 'root_reviewed': True},
                 'Exact independent same-source approval required')
    return value


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--capture-id', required=True, type=identity)
    parser.add_argument('--prepare', action='store_true')
    args = parser.parse_args(argv)
    root = ROOT.resolve(strict=True)
    safe.require((root / '.git').is_dir() and not (root / '.git').is_symlink(), 'MAIN checkout only')
    inherited = safe.require_guard(root)
    safe.require(os.environ.get('CARGO_TARGET_DIR') == str(root / 'target'), 'MAIN default target required')
    fixed_raw, rows, excluded = load_fixed(root)
    paths = source_set(root)
    before = pins(paths)
    by_path = {p['path']: p for p in before}
    bash_path, cat_path = safe.BASH.resolve(strict=True), CAT.resolve(strict=True)
    safe.require(by_path[str(bash_path)]['sha256'] == BASH_SHA
                 and by_path[str(cat_path)]['sha256'] == CAT_SHA
                 and by_path[str(safe.MANUAL.resolve(strict=True))]['sha256'] == safe.MANUAL_SHA,
                 'Original reviewed Bash/cat/manual identities required')
    head = safe.source_identity(root)
    from build_safety import owned_directory
    base = owned_directory(root, root / HERE)
    control = {'schema_version': 1, 'root_reviewed': False, 'capture_id': args.capture_id,
               'source_head': head, 'pins': before, 'original117_sha256': FIXED_SHA,
               'admitted114_sha256': ADMITTED_SHA, 'excluded': excluded,
               'claims': 'Fixed114 real Bash/VM comparisons;3 absolute guards remain simulation-only'}
    if args.prepare:
        pin = safe.json_new(base / ('PROPOSED-' + args.capture_id + '.json'), control, safe.Budget(CAP), CAP)
        print(json.dumps({'status': 'PROPOSED-ONLY-NO-EXECUTION', 'pin': pin}))
        return 0
    approval_raw = safe.read_bytes(base / ('APPROVED-' + args.capture_id + '.json'), CAP)
    approved(approval_raw, control)
    out = safe.private_dir(base, args.capture_id)
    budget = safe.Budget(TOTAL)
    outputs = [safe.json_new(out / 'STARTED.json', control, budget, CAP),
               safe.write_new(out / 'ORIGINAL117.json', fixed_raw, budget, CAP),
               safe.write_new(out / 'APPROVED.json', approval_raw, budget, CAP)]
    began = time.monotonic()
    with pinned_executable(root / PROBE, by_path[str(root / PROBE)], BINARY_CAP) as (probe, pfd), \
         pinned_executable(bash_path, by_path[str(bash_path)], CAP) as (bash, bfd), \
         pinned_executable(cat_path, by_path[str(cat_path)], CAP) as (cat, cfd):
        retained = (*inherited, pfd, bfd, cfd)
        folder = safe.private_dir(out, 'vm-identity')
        work, temporary = safe.private_dir(folder, 'work'), safe.private_dir(folder, 'tmp')
        value = capture_child([probe, '--identity'], folder, work, safe.closed_environment({}, temporary), budget,
                              inherited=retained, vm=True)
        safe.quiet_group()
        safe.require(value['status'] == 0 and not value['stderr'] and strict(value['stdout']) == {
            'kind': 'fixed-optional-inputs-reference', 'engine_revision': 11, 'original_rows': 117,
            'admitted_rows': 114, 'unique_programs': 15, 'original117_sha256': FIXED_SHA}, 'Actual fixed evaluator11 probe identity')
        outputs += [safe.pin(folder / n, VM_STREAM) for n in ('stdout.raw', 'stderr.raw')]
        folder = safe.private_dir(out, 'vm-outcomes')
        work, temporary = safe.private_dir(folder, 'work'), safe.private_dir(folder, 'tmp')
        value = capture_child([probe], folder, work, safe.closed_environment({}, temporary), budget,
                              inherited=retained, vm=True)
        safe.quiet_group()
        safe.require(value['status'] == 0 and not value['stderr'], 'Fixed VM probe completed')
        vm = vm_rows(value['stdout'], rows)
        outputs += [safe.pin(folder / n, VM_STREAM) for n in ('stdout.raw', 'stderr.raw')]
        folder = safe.private_dir(out, 'version')
        work, temporary = safe.private_dir(folder, 'work'), safe.private_dir(folder, 'tmp')
        version = capture_child([bash, '--noprofile', '--norc', '--version'], folder, work,
            safe.closed_environment({}, temporary), budget, inherited=retained)
        safe.quiet_group()
        safe.require(version['status'] == 0 and not version['stderr']
                     and re.match(r'^GNU bash, version 5\.3\.9\b', version['stdout']), 'Actual original Bash5.3.9 required')
        outputs += [safe.pin(folder / n, safe.STREAM_CAP) for n in ('stdout.raw', 'stderr.raw')]
        safe.require(re.fullmatch(r'/proc/self/fd/[0-9]+', cat), 'Only retained executable descriptor helper')
        # Empty PATH; this sole reviewed function delegates argv/stdin/status to
        # original GNU cat. No general host executable lookup or script wrapper.
        prelude = 'cat() { ' + cat + ' "$@"; }\n'
        passed = 0
        for index, (row, simulated) in enumerate(zip(rows, vm)):
            safe.require(time.monotonic() - began < WHOLE_SECONDS, 'Whole114 deadline')
            check_case(row)  # Admission again immediately before each child.
            folder = safe.private_dir(out, f'case-{index:03d}')
            work, temporary = safe.private_dir(folder, 'work'), safe.private_dir(folder, 'tmp')
            initial = materialize(work, row['case']['files'], budget)
            observed = capture_child([bash, *safe.BASH_FLAGS, '-c', prelude + row['source'],
                'fixed-optional-' + str(index), *row['case']['args']], folder, work,
                safe.closed_environment(row['case']['env'], temporary), budget, inherited=retained)
            safe.quiet_group()
            safe.require(observed['status'] >= 0, 'Unexpected child signal; retain diagnostic evidence')
            allowed = {flat(n) for scope in ('files', 'after') for n in row['case'][scope]}
            after, total = final_files(work, allowed)
            budget.replace_reservation(initial, total)
            observed['files'] = after
            result = comparison(row, simulated, observed)
            passed += result['passed']
            record = {'index': index, 'id': row['id'], 'source': row['source'], 'case': row['case'],
                      'actual_vm11': simulated, 'actual_bash': observed, **result,
                      'utility_helper_source': prelude, 'learner_input_accepted': False}
            outputs.append(safe.json_new(folder / 'OBSERVATION.json', record, budget, CAP))
            outputs += [safe.pin(folder / n, safe.STREAM_CAP) for n in ('stdout.raw', 'stderr.raw')]
            outputs += [safe.pin(work / flat(name), 4096) for name in after]
    safe.quiet_group()
    safe.require(pins(paths) == before and safe.source_identity(root) == head, 'Reviewed source/executable identity changed')
    safe.require(time.monotonic() - began < WHOLE_SECONDS, 'Whole114 final deadline')
    for pin in outputs:
        safe.require(safe.pin(Path(pin['path']), TOTAL) == pin, 'Retained observation changed')
    manifest = {'schema_version': 1, 'source_head': head, 'pins': before, 'original_rows': 117,
                'compared_rows': 114, 'unique_programs': 15, 'excluded_simulation_only': excluded,
                'matches': passed, 'mismatches': 114 - passed, 'accepted': passed == 114,
                'outputs': outputs, 'no_learner_source_or_progress': True}
    safe.publish(out, manifest, budget)
    print(json.dumps({'status': 'COMPLETE-OBSERVATIONS', 'path': str(out), 'compared': 114,
                      'simulation_only': 3, 'matches': passed, 'mismatches': 114 - passed}))
    return 0 if passed == 114 else 1


if __name__ == '__main__':
    raise SystemExit(main())
