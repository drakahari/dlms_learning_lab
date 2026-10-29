#!/usr/bin/env python3
"""Fixed16 builtin-only GNU Bash/VM13 reference; eight exclusions are VM-only.

No learner source, arbitrary probe/worktree/file/engine selector or host engine
fallback. MAIN runner/common FD lock owns serial bounded development children.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import time
import case_bash_oracle as safe
import case_classes_bash_oracle as plumbing
from automation_before10_oracle import pinned_executable
from build_safety import owned_directory
from dev import build_target

ROOT = Path(__file__).resolve().parents[1]
WORKTREE = '.build-artifacts/worktrees/bash-loop13-b25'
BASE = 'b25b1f0bdf2ac26dbe3fd071a372a023e60acef9'
SOURCE_INVENTORY = '.build-artifacts/reviews/bash-loop-recovery/current13-applied-b25-r2/SOURCE.json'
INVENTORY_SHA = '112cc9932c3973e79dbb8c8f6e088e8e0e62880cce045c5e301ace5e0aa22fdc'
FIXED = 'tests/fixtures/shell-loop-control-fixed.json'
MIRROR = 'tools/tests/fixtures/loop-control24-reference.json'
FIXED_SHA = 'cbb429ee223420553da8d894ec9ea74e86cbcf92eb9e2b0ae26b4a1f4e941715'
PROBE_SOURCE = 'examples/loop_control_reference.rs'
PROBE_NAME = 'debug/examples/loop_control_reference'
PROBE_BEFORE_SHA = 'a90191a484d48542731446044835d7f2de13baba8975ce3afc91a0332ce12318'
SELF = 'tools/loop_control_bash_oracle.py'
TEST = 'tools/tests/test_loop_control_bash_oracle.py'
BASELINE = '.build-artifacts/reviews/bash-loop-recovery/captures/public12-171-420-r1'
BASELINE_SHA = '109d1c747c8bb9cb8f497abcc5a3e13be5de53de6447b37fccc728ec15aeb448'
ATTESTATION_SHA = '0f03d3474a70fa01295b545d6c14cec26ecdf8db758773cb951621e78e630f05'
OLD_ELF_SHA = '0a32601f83e105260b31ccbe88459e840553e4f99b1983b3346050f2bf7fcd03'
INFO = Path('/usr/share/info/bash.info.gz')
INFO_SHA = 'c2ab10cda098ad2e6a43ced7a02ff1545fd9d85f3dc05183bb1d72cd3fcf60b2'
CHECKED = '2026-10-29'
REFERENCES = ['https://www.gnu.org/software/bash/manual/html_node/Bourne-Shell-Builtins.html',
              'https://pubs.opengroup.org/onlinepubs/9799919799/utilities/break.html',
              'https://pubs.opengroup.org/onlinepubs/9799919799/utilities/continue.html']
OUTPUT = '.build-artifacts/reviews/bash-loop-recovery/bash-captures'
CAPTURE = 'loop16-r1'
CAP = 16_000_000
BINARY_CAP = 256 * 1024 * 1024
PIN_TOTAL = 640 * 1024 * 1024
EVIDENCE_TOTAL = 64 * 1024 * 1024
SOURCE_TOTAL = 512 * 1024 * 1024
STREAM = 128 * 1024
VM_STREAM = 1024 * 1024
DEADLINE = 180.0
ALLOWED_FILES = frozenset(('rows.txt', 'inner.txt', 'trace.txt'))
PROBE_SHA = '38c12cba3f961735764c92933e027961fd862e0b4e19cc7fc2440b143f554269'
PROBE_BYTES = 4005
ROW_PINS = {'first-break': '6167b91c84ea3173b7e9d8b481eb035cccf6e9a62ceb509e25a0fe14c16d0443', 'skip-one': '7296a12314b56351d5d18ea3a5ae53c477919890545d70de1665160e0b0e80b9', 'nested-local-break': 'd1421fdaf7cadfaebb33335030e6353bb152aaa4e92f702d399c9abf9721c15c', 'while-preserves-records': '262ca3037c88f8f0c09c156f3e073054c319fd738de12264016402c7dae51b53', 'break-restores-finite-input': '790aa4d07703c73272c06c9a43aca2f6c22a4569e20087cd71cf2bbc5f6a86fc', 'continue-restores-inner-input': '7734f2d8ef466f14c30ae3e99f9aef83d43352a3425011df7302da61f7333fbe', 'mixed-break-two': '07715c0460b2ec09c3b60480dc02688736b94969d5227f2dc47ddad5e0b62934', 'depth-exceeds-local-nesting': 'd737f67c2ff7e6852a976aef03269ac3153e0db2cd788dd32d92f0fa3a1d7bc4', 'quoted-expanded-head': '61b34da02637e4c7224d428739b4278c1b1eda1c3e8d0183cc6e69d43da2cc3a', 'case-selected-control': '35f0a63b00ebbd438f3e4e2168e46f7b72c7f118247cf0b233151ed3e887741b', 'control-short-circuits-remainder': '0189f9c7faac2dc7246fa8d271c5e81184eeaf49f0bd7f1aca8167f9eb1bebca', 'redirect-before-break': '5adecf28bcfbb12e12ba3e4f4f29fefb46eabff96154a77a2323f98c83c13173', 'own-function-loop': 'f34e09309efe8e6232603db2dfc558968767de13e94e8813d2f5d604363c6d96', 'own-capture-loop': '0cec47896a9c8433c438cc679bcc3a0ab66cebf42ee00ad2e11e982a274b48c3', 'return-cleanup-regression': '702dc8773ce4da969f01d2439535cbc64586a91b10934225f241c433d32b74ea', 'continue-status-zero': 'ecfeb5f92f0015e242a4e9e5b6d4a71a19edefd8fe08d5390cb0b8fc0ff1acd4', 'outside-loop': '1029e6a0fd291964629d1fa96da7961fa1a89685892f99f8810a0b0af4a71e48', 'caller-function-loop': 'd27c3d00d3e0989b9132e3178fc9ea8ac54b749ceb18742c2bd115acc0ba2c06', 'pipeline-control': '4406ec430f07e4099def4d9ba442e1b1427dbac61645733bb51e108a8c7c84e9', 'capture-parent-loop': 'be6934f9f5b3cb25d6a9e571021834986cf85159bdea1470e5a9d8154633b866', 'zero-count': '172b47d515e59589c64ed3d880721cd56d39cc9dec775d5fd25f503fe44be74d', 'noncanonical-count': 'd2f306c3070cc53f08d125d8945a419b8b344c1f4ae65748299a7fc6e439b0ad', 'excessive-count': 'be05f5590945a47a27565afedaff0d699544dd7f8fc75c79847932a99d9daac0', 'extra-operands': 'aa422134113846aa3a49b5e7c73e457aafd79e5188fc96b0593a8605b98659eb'}
CASE_IDS = ('first-break', 'skip-one', 'nested-local-break', 'while-preserves-records', 'break-restores-finite-input', 'continue-restores-inner-input', 'mixed-break-two', 'depth-exceeds-local-nesting', 'quoted-expanded-head', 'case-selected-control', 'control-short-circuits-remainder', 'redirect-before-break', 'own-function-loop', 'own-capture-loop', 'return-cleanup-regression', 'continue-status-zero', 'outside-loop', 'caller-function-loop', 'pipeline-control', 'capture-parent-loop', 'zero-count', 'noncanonical-count', 'excessive-count', 'extra-operands')
HOST_IDS = ('first-break', 'skip-one', 'nested-local-break', 'while-preserves-records', 'break-restores-finite-input', 'continue-restores-inner-input', 'mixed-break-two', 'depth-exceeds-local-nesting', 'quoted-expanded-head', 'case-selected-control', 'control-short-circuits-remainder', 'redirect-before-break', 'own-function-loop', 'own-capture-loop', 'return-cleanup-regression', 'continue-status-zero')
CORE = ('Cargo.toml', 'Cargo.lock', 'src/shell.rs', 'src/shell/arithmetic.rs', 'src/shell/capture.rs', 'src/shell/case.rs', 'src/shell/cron.rs', 'src/shell/execution.rs', 'src/shell/getopts.rs', 'src/shell/globbing.rs', 'src/shell/loop_control.rs', 'src/shell/parameters.rs', 'src/shell/preparation.rs', 'src/shell/substitution.rs', 'src/shell/text_tools.rs', 'src/shell/values.rs', 'examples/loop_control_reference.rs', 'examples/support/automation_digest.rs', 'tests/fixtures/shell-loop-control-fixed.json')


def digest(raw): return hashlib.sha256(raw).hexdigest()


def canonical(value): return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode()


strict = plumbing.strict
child = plumbing.child
directory = plumbing.directory



def safe_ref(name, *, requested=False):
    safe.require(isinstance(name, str) and len(name) <= 255
                 and re.fullmatch(r'refs/[A-Za-z0-9_./-]+', name)
                 and all(part and part not in ('.', '..') and not part.startswith('.')
                         and not part.endswith(('.', '.lock')) for part in name.split('/'))
                 and (not requested or name.startswith('refs/heads/')),
                 'Unsafe or unsupported symbolic source ref')
    return name


def git_line(raw):
    safe.require(raw and raw.isascii() and all(byte == 10 or 32 <= byte < 127 for byte in raw), 'Ordinary printable ASCII Git identity with LF only')
    text = raw.decode('ascii')
    if text.endswith('\n'): text = text[:-1]
    safe.require(text and '\n' not in text and '\r' not in text and text == text.strip(),
                 'Single canonical Git identity line')
    return text


def source_identity(root):
    """Local fixed-collector reader; old helpers and Git metadata stay immutable.

    Only an absent ordinary loose ref falls back to packed-refs. An alias,
    malformed loose value or open error must not become a silent fallback.
    """
    head = git_line(safe.read_bytes(root / '.git/HEAD', 4096))
    if re.fullmatch(r'[0-9a-f]{40}', head): return head
    safe.require(head.startswith('ref: '), 'Detached hash or symbolic HEAD required')
    requested = safe_ref(head[5:], requested=True)
    try:
        loose = safe.read_bytes(child(root / '.git', requested), 4096)
    except FileNotFoundError:
        pass
    else:
        value = git_line(loose)
        safe.require(re.fullmatch(r'[0-9a-f]{40}', value), 'Malformed ordinary loose source ref')
        return value  # Git loose-reference precedence; packed fallback is unused.
    raw = safe.read_bytes(root / '.git/packed-refs', 1024 * 1024)
    safe.require(raw.isascii() and all(byte == 10 or 32 <= byte < 127 for byte in raw),
                 'Packed refs allow printable ASCII and LF separators only')
    lines = raw.decode('ascii').split('\n')
    if lines and lines[-1] == '': lines.pop()  # One terminal LF is not an extra row.
    safe.require(len(lines) <= 4096, 'Packed-reference count bound')
    refs = {}; may_peel = False
    for line in lines:
        safe.require(len(line) <= 512, 'Packed-reference line bound')
        if not line or line.startswith('#'):
            may_peel = False
            continue
        if line.startswith('^'):
            safe.require(may_peel and re.fullmatch(r'\^[0-9a-f]{40}', line), 'Orphan, repeated or malformed peeled ref')
            may_peel = False
            continue
        match = re.fullmatch(r'([0-9a-f]{40}) (refs/[A-Za-z0-9_./-]+)', line)
        safe.require(match is not None, 'Malformed packed-reference record')
        value, name = match.groups(); safe_ref(name)
        safe.require(name not in refs, 'Duplicate packed-reference name')
        refs[name] = value; may_peel = True
    safe.require(requested in refs, 'HEAD source ref absent from bounded packed refs')
    return refs[requested]


def check_row(row):
    safe.require(isinstance(row, dict) and set(row) == {'id', 'partition', 'source', 'args', 'env', 'stdin', 'files', 'expected', 'note', 'vm_observation', 'bash_observation'}, 'Closed fixed row schema')
    safe.require(isinstance(row['id'], str) and row['id'] in ROW_PINS and row['args'] == [] and row['env'] == {}, 'Fixed ID and empty argv/environment')
    safe.require(isinstance(row['source'], str) and 0 < len(row['source'].encode()) <= 4096 and '\0' not in row['source'], 'Bounded fixed source')
    safe.require(isinstance(row['stdin'], str) and '\0' not in row['stdin'] and len(row['stdin'].encode()) <= 4096, 'Bounded finite stdin')
    safe.require(isinstance(row['files'], dict) and len(row['files']) <= 3, 'Closed initial virtual files')
    for name, text in row['files'].items():
        safe.require(name in {'/work/' + n for n in ALLOWED_FILES} and isinstance(text, str) and '\0' not in text and len(text.encode()) <= 4096, 'Only reviewed flat disposable files')
    safe.require(row['vm_observation'] is None and row['bash_observation'] is None, 'Predictions stay null; observations separate')
    safe.require(row['partition'] == ('supported' if row['id'] in HOST_IDS else 'honest_exclusion'), 'Explicit host/VM-only partition')
    safe.require(digest(canonical(row)) == ROW_PINS[row['id']], 'Exact reviewed source/input/prediction row; no source substitution')


def load_fixed(root):
    raw = safe.read_bytes(root / MIRROR, CAP)
    safe.require(digest(raw) == FIXED_SHA, 'Exact byte-identical developer fixture')
    value = strict(raw)
    safe.require(set(value) == {'status', 'supported_cases', 'honest_exclusions', 'cases', 'negative_redirect_plan'} and value['status'] == 'FIXED_SOURCE_ONLY_UNRUN_NO_REFERENCE_APPROVAL' and isinstance(value['negative_redirect_plan'], str) and value['supported_cases'] == 16 and value['honest_exclusions'] == 8, 'Closed24 fixture schema')
    rows = value['cases']
    safe.require(isinstance(rows, list) and tuple(r['id'] for r in rows) == CASE_IDS, 'Exact24 IDs/order without omissions or extras')
    for row in rows: check_row(row)
    return raw, rows


def clone_proof(root):
    raw = safe.read_bytes(root / SOURCE_INVENTORY, CAP)
    safe.require(digest(raw) == INVENTORY_SHA, 'Exact reviewed b25/13 full-source inventory')
    reviewed = strict(raw)
    safe.require(isinstance(reviewed, dict) and len(reviewed) == 898 and reviewed[PROBE_SOURCE]['sha256'] == PROBE_BEFORE_SHA, 'Closed898 source boundary and exact probe-only identity amendment')
    expected = {name: dict(pin) for name, pin in reviewed.items()}
    expected[PROBE_SOURCE] = {'bytes': PROBE_BYTES, 'sha256': PROBE_SHA}
    work = directory(root / WORKTREE)
    safe.require(plumbing.public_files(work) == set(expected), 'Complete source inventory, no unreviewed added/removed files')
    for name, pin in expected.items():
        actual = safe.pin(child(work, name), CAP)
        safe.require({k: actual[k] for k in ('bytes', 'sha256')} == pin, 'Worktree source drift: ' + name)
    safe.require(safe.read_bytes(work / FIXED, CAP) == safe.read_bytes(root / MIRROR, CAP), 'W compiled fixture and Main developer fixture identical')
    declaration = safe.read_bytes(work / 'src/shell.rs', CAP)
    safe.require(re.findall(rb'^pub const EVALUATOR_REVISION: u32 = ([0-9]+);$', declaration, re.MULTILINE) == [b'13'], 'Actual evaluator13 only')
    gitdir = root / '.git/worktrees' / work.name
    safe.require(safe.read_bytes(work / '.git', 4096).decode().strip() == 'gitdir: ' + str(gitdir) and safe.read_bytes(gitdir / 'HEAD', 4096).decode().strip() == BASE, 'Fixed detached same-project b25 checkout')
    target = build_target(root, work, None); directory(target)
    return work, target / PROBE_NAME, expected


def baseline_paths(root):
    folder = root / BASELINE
    raw = safe.read_bytes(folder / 'MANIFEST.json', CAP)
    safe.require(digest(raw) == BASELINE_SHA and digest(safe.read_bytes(folder / 'ATTESTATION.json', CAP)) == ATTESTATION_SHA and safe.read_bytes(folder / 'COMPLETE', 65) == (BASELINE_SHA + '\n').encode(), 'Genuine sealed evaluator12 required')
    manifest = strict(raw); a = strict(safe.read_bytes(folder / 'ATTESTATION.json', CAP))
    safe.require(a['actual12_manifest_sha256'] == BASELINE_SHA and a['records_regenerated'] is False and a['producer']['bytes'] == 113437256 and a['producer']['sha256'] == OLD_ELF_SHA, 'Exact genuine12 provenance, not generated under13')
    safe.require(manifest['actual_evaluator_revision'] == 12 and manifest['script_records'] == 55 and manifest['authored_cases'] == 420 and manifest['examples'] == 55, 'Retain old55/420/55 baseline')
    outputs = manifest['outputs']; safe.require(len(outputs) == 11 and len({p['file'] for p in outputs}) == 11, 'All11 old outputs')
    source = strict(safe.read_bytes(folder / 'SOURCE.json', CAP)); inputs = source['inputs']
    safe.require(source['actual_evaluator_revision'] == 12 and len(inputs) == 296 and len({p['file'] for p in inputs}) == 296, 'All296 retained source inputs')
    paths = [(folder / name, CAP) for name in ['MANIFEST.json', 'ATTESTATION.json', 'COMPLETE']]
    for base, rows in [(folder, outputs), (folder / 'compiled-source', inputs)]:
        for pin in rows:
            path = child(base, pin['file']); actual = safe.pin(path, CAP)
            safe.require({k: actual[k] for k in ('bytes', 'sha256')} == {k: pin[k] for k in ('bytes', 'sha256')}, 'Actual12 retained file changed')
            paths.append((path, CAP))
    kept = safe.pin(folder / 'actual12-producer', BINARY_CAP)
    safe.require(kept['bytes'] == 113437256 and kept['sha256'] == OLD_ELF_SHA, 'Actual12 retained ordinary ELF required')
    paths.append((folder / 'actual12-producer', BINARY_CAP))
    return paths


def pins(paths):
    safe.require(len(paths) <= 1600 and len({str(p) for p, _ in paths}) == len(paths), 'Bounded unique control inputs')
    total = 0
    for path, cap in paths:
        with safe.ordinary(path, cap) as (_, info): total += info.st_size
        safe.require(total <= PIN_TOTAL, 'Control inputs aggregate before hash')
    result = [safe.pin(path, cap) for path, cap in paths]
    safe.require(sum(p['bytes'] for p in result) <= PIN_TOTAL, 'Actual pin aggregate')
    return result


def source_paths(root, work, probe, reviewed):
    names = [SELF, TEST, MIRROR, 'tools/dev.py', 'tools/build_safety.py', 'tools/case_bash_oracle.py', 'tools/case_classes_bash_oracle.py', 'tools/automation_before10_oracle.py', SOURCE_INVENTORY]
    paths = [(root / name, CAP) for name in names]
    paths += [(work / name, CAP) for name in sorted(reviewed)]
    paths += [(probe, BINARY_CAP), (safe.BASH.resolve(strict=True), CAP), (safe.MANUAL.resolve(strict=True), 8 * CAP), (INFO.resolve(strict=True), CAP)]
    paths += baseline_paths(root)
    return paths


def approved(raw, control):
    value = strict(raw)
    safe.require(value == {**control, 'root_reviewed': True} and value.get('root_reviewed') is True, 'Exact Root-approved source/ELF/fixture/control required')


def outcome(value):
    safe.require(isinstance(value, dict) and set(value) == {'stdout', 'stderr', 'status', 'files', 'error'}, 'Actual full VM outcome')
    safe.require(all(isinstance(value[k], str) and '\0' not in value[k] and len(value[k].encode()) <= 32768 for k in ('stdout', 'stderr')) and type(value['status']) is int and 0 <= value['status'] <= 255, 'Bounded streams/status')
    safe.require(value['error'] is None or isinstance(value['error'], str) and len(value['error'].encode()) <= 1000, 'Bounded honest diagnostic')
    safe.require(isinstance(value['files'], dict) and set(value['files']) <= {'/work/' + n for n in ALLOWED_FILES} and all(isinstance(v, str) and '\0' not in v and len(v.encode()) <= 4096 for v in value['files'].values()), 'Closed virtual file outputs')
    return value


def vm_rows(text, work, rows):
    safe.require(len(text.encode()) <= VM_STREAM, 'Bounded fixed VM stream')
    value = strict(text)
    expected_identity = {name: safe.pin(work / name, CAP)['sha256'] for name in CORE}
    safe.require(isinstance(value, dict) and set(value) == {'fixture_id', 'engine_revision', 'manifest_dir', 'compiled_sources', 'fixture_utf8', 'outcomes'} and value['fixture_id'] == 'loop-control24-r1' and value['engine_revision'] == 13 and value['manifest_dir'] == str(work) and value['compiled_sources'] == expected_identity and value['fixture_utf8'].encode() == safe.read_bytes(work / FIXED, CAP), 'Actual compiled13 core/fixture/checkout identity; reject stale ELF')
    observations = value['outcomes']
    safe.require(isinstance(observations, list) and len(observations) == 24, 'All24 actual VM outcomes')
    result = []
    for row, observation in zip(rows, observations):
        safe.require(isinstance(observation, dict) and set(observation) == {'id', 'partition', 'outcome'} and observation['id'] == row['id'] and observation['partition'] == row['partition'], 'Exact VM order/source partition')
        result.append(outcome(observation['outcome']))
    return result


def predicted(row):
    safe.require(row['partition'] == 'supported' and row['expected'] is not None, 'Only supported predictions')
    return {**{key: row['expected'][key] for key in ('stdout', 'stderr', 'status')}, 'files': row['expected']['after'], 'error': None}


def comparison(expected, vm, bash):
    fields = ('stdout', 'stderr', 'status', 'files')
    by_field = {key: {'vm_matches_bash': vm[key] == bash[key], 'vm_matches_authored': vm[key] == expected[key], 'bash_matches_authored': bash[key] == expected[key]} for key in fields}
    passed = vm['error'] is None and all(all(row.values()) for row in by_field.values())
    return {'by_field': by_field, 'vm_error': vm['error'], 'passed': passed}


def exclusion(row, vm_out):
    safe.require(row['partition'] != 'supported', 'VM-only exclusion partition')
    return {'id': row['id'], 'partition': 'VM_ONLY_NOT_RUN_IN_BASH', 'actual_vm13': vm_out, 'actual_bash539': None,
            'honest_unsupported': isinstance(vm_out['error'], str) and 'Unsupported' in vm_out['error'] and vm_out['status'] != 0 and vm_out['files'] == row['files']}


def closed_files(work):
    directory(work); result = {}; total = 0
    with os.scandir(work) as entries:
        for entry in entries:
            safe.require(entry.name in ALLOWED_FILES and not entry.is_symlink() and entry.is_file(follow_symlinks=False), 'No unknown file/directory/symlink from child')
            raw = safe.read_bytes(work / entry.name, 4096); total += len(raw)
            safe.require(total <= 8192 and b'\0' not in raw, 'Final flat files aggregate/content')
            result['/work/' + entry.name] = raw.decode('utf-8', errors='strict')
    return dict(sorted(result.items()))


def run_child(out, name, command, row, budget, held, start, *, is_vm=False):
    safe.require(time.monotonic() - start < DEADLINE, 'Whole serial reference deadline')
    safe.require(isinstance(row['files'], dict) and len(row['files']) <= 3 and isinstance(row['stdin'], str) and '\0' not in row['stdin'] and len(row['stdin'].encode()) <= 4096, 'Closed child input')
    safe.require(all(path in {'/work/' + n for n in ALLOWED_FILES} and isinstance(text, str) and '\0' not in text and len(text.encode()) <= 4096 for path, text in row['files'].items()), 'No arbitrary fixture write path')
    folder = safe.private_dir(out, name); work = safe.private_dir(folder, 'work'); tmp = safe.private_dir(folder, 'tmp')
    for path, text in row['files'].items(): safe.write_new(work / path.removeprefix('/work/'), text.encode(), budget, 4096)
    environment = safe.closed_environment({}, tmp)
    result = safe.capture_command(command, folder, work, environment, row['stdin'].encode(), budget, VM_STREAM if is_vm else STREAM, timeout=30 if is_vm else 5, inherited=held, popen=lambda cmd, **kw: plumbing.bounded_popen(cmd, vm=is_vm, **kw))
    safe.quiet_group()
    # Preserve real child streams/status immediately even if later file admission
    # or comparison fails. Files and comparison have their own later records.
    outputs = [safe.json_new(folder / 'RESULT.json', {'kind': 'actual-fixed-loop-child-result', 'result': result, 'claims': 'Observed child only, not acceptance'}, budget, CAP)]
    outputs += [safe.pin(folder / n, VM_STREAM if is_vm else STREAM) for n in ['stdout.raw', 'stderr.raw']]
    files = closed_files(work)
    outputs.append(safe.json_new(folder / 'FILES.json', files, budget, CAP))
    return result, files, outputs


def retain_sources(root, work, out, reviewed, probe, executable_pin, before):
    budget = safe.Budget(SOURCE_TOTAL); expected = {p['path']: p for p in before}; retained = []
    bundle = safe.private_dir(out, 'compiled-source')
    main = [SELF, TEST, MIRROR, 'tools/dev.py', 'tools/build_safety.py', 'tools/case_bash_oracle.py', 'tools/case_classes_bash_oracle.py', 'tools/automation_before10_oracle.py']
    for label, base, names in [('worktree', work, sorted(reviewed)), ('main-tooling', root, main)]:
        destination = safe.private_dir(bundle, label)
        for name in names:
            plumbing.retained_source_parent(destination, name); raw = safe.read_bytes(child(base, name), CAP); wanted = expected[str(child(base, name))]
            safe.require(len(raw) == wanted['bytes'] and digest(raw) == wanted['sha256'], 'Source drift before retention')
            retained.append(safe.write_new(child(destination, name), raw, budget, CAP))
    with safe.ordinary(probe, BINARY_CAP) as (stream, info):
        safe.require(safe.pin(probe, BINARY_CAP) == executable_pin, 'Probe changed before retention'); budget.charge(info.st_size)
        with safe.new_file(out / 'vm13-producer') as target:
            left = info.st_size
            while left:
                block = stream.read(min(left, 1024 * 1024)); safe.require(block, 'Probe shrank'); target.write(block); left -= len(block)
            target.flush(); os.fsync(target.fileno())
    kept = safe.pin(out / 'vm13-producer', BINARY_CAP); safe.require(kept['sha256'] == executable_pin['sha256'], 'Retained probe differs'); retained.append(kept)
    return retained


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--mode', required=True, choices=['prepare', 'capture']); args = parser.parse_args(argv)
    root = ROOT.resolve(strict=True); inherited = plumbing.main_entry(root)
    _, rows = load_fixed(root); work, probe, reviewed = clone_proof(root)
    paths = source_paths(root, work, probe, reviewed); before = pins(paths); head = source_identity(root)
    safe.require(safe.pin(safe.MANUAL.resolve(strict=True), 8 * CAP)['sha256'] == safe.MANUAL_SHA and safe.pin(INFO.resolve(strict=True), CAP)['sha256'] == INFO_SHA, 'Original official manual identities')
    base = owned_directory(root, root / OUTPUT)
    control = {'schema_version': 1, 'root_reviewed': False, 'capture': CAPTURE, 'main_head': head, 'worktree': WORKTREE, 'worktree_base': BASE, 'expected_revision': 13, 'source_files': 898, 'pins': before, 'fixture_sha256': FIXED_SHA, 'host_rows': 16, 'VM_rows': 24, 'excluded_host_ids': [r['id'] for r in rows if r['partition'] != 'supported'], 'genuine12_manifest_sha256': BASELINE_SHA, 'references_checked': CHECKED, 'references': REFERENCES, 'scope': 'Fixed synthetic development comparison only; no learner source, host engine fallback, curriculum credit or mastery'}
    safe.require(not (base / CAPTURE).exists() and not (base / CAPTURE).is_symlink(), 'Fresh capture only; preserve all previous failures/observations')
    if args.mode == 'prepare':
        safe.json_new(base / (CAPTURE + '-PROPOSED.json'), control, safe.Budget(CAP), CAP); print(json.dumps({'status': 'PROPOSED-NO-CHILD-EXECUTION', 'host_rows': 16, 'VM_rows': 24})); return 0
    approved(safe.read_bytes(base / (CAPTURE + '-APPROVED.json'), CAP), control)
    out = safe.private_dir(base, CAPTURE); budget = safe.Budget(EVIDENCE_TOTAL); start = time.monotonic(); outputs = []
    executable_pin = next(p for p in before if p['path'] == str(probe)); bash = safe.BASH.resolve(strict=True); bash_pin = next(p for p in before if p['path'] == str(bash))
    with pinned_executable(probe, executable_pin, BINARY_CAP) as (vm, vfd), pinned_executable(bash, bash_pin, CAP) as (sh, bfd):
        held = (*inherited, vfd, bfd); empty = {'files': {}, 'stdin': ''}
        result, files, new = run_child(out, 'vm-outcomes', [vm], empty, budget, held, start, is_vm=True); outputs += new
        safe.require(result['status'] == 0 and not result['stderr'] and not files, 'Fixed in-process probe completed without host files'); actual = vm_rows(result['stdout'], work, rows)
        result, files, new = run_child(out, 'bash-version', [sh, '--noprofile', '--norc', '--version'], empty, budget, held, start); outputs += new
        safe.require(result['status'] == 0 and not result['stderr'] and not files and re.match(r'^GNU bash, version 5\.3\.9\b', result['stdout']), 'Original GNU Bash5.3.9 only')
        observations = []; excluded = []
        for index, (row, vm_out) in enumerate(zip(rows, actual)):
            if row['partition'] != 'supported':
                excluded_row = exclusion(row, vm_out)
                excluded.append(excluded_row)
                # Flat named JSON, no hidden pseudo-Bash receipt for exclusions.
                outputs.append(safe.json_new(out / ('excluded' + str(index) + '.json'), excluded_row, budget, CAP)); continue
            result, files, new = run_child(out, 'case' + str(index), [sh, *safe.BASH_FLAGS, '-c', row['source'], '--'], row, budget, held, start); outputs += new
            bash_out = {**result, 'files': files, 'error': None}; expected = predicted(row)
            observation = {'id': row['id'], 'source_sha256': digest(row['source'].encode()), 'case_sha256': digest(canonical({k: row[k] for k in ('args', 'env', 'stdin', 'files')})), 'authored': expected, 'actual_vm13': vm_out, 'actual_bash539': bash_out, 'comparison': comparison(expected, vm_out, bash_out)}
            observations.append(observation); outputs.append(safe.json_new(out / ('case' + str(index)) / 'COMPARISON.json', observation, budget, CAP))
    safe.require(time.monotonic() - start < DEADLINE, 'Whole reference deadline; partial observations retained')
    safe.require(pins(paths) == before and source_identity(root) == head, 'Source/ELF/manual/input changed during comparison')
    clone_proof(root)
    outputs += retain_sources(root, work, out, reviewed, probe, executable_pin, before)
    safe.require(pins(paths) == before, 'Source changed during retention')
    outputs.append(safe.json_new(out / 'OBSERVATIONS.json', {'supported': observations, 'VM_only': excluded}, budget, CAP))
    passed = sum(r['comparison']['passed'] for r in observations); excluded_pass = sum(r['honest_unsupported'] for r in excluded)
    accepted = len(observations) == 16 and len(excluded) == 8 and passed == 16 and excluded_pass == 8
    manifest = {**control, 'root_reviewed': True, 'matches': passed, 'mismatches': 16 - passed, 'VM_only_honest_exclusions': excluded_pass, 'accepted': accepted, 'producer': executable_pin, 'outputs': outputs, 'status': 'MATCHED16-SUPPORTED-8-VM-EXCLUSIONS' if accepted else 'COMPLETED-WITH-MISMATCHES-NOT-ACCEPTED', 'limits': {'Bash_seconds': 5, 'VM_seconds': 30, 'whole_seconds': DEADLINE, 'streams': STREAM, 'VM_stream': VM_STREAM, 'evidence_total': EVIDENCE_TOTAL, 'retained_source_total': SOURCE_TOTAL}, 'claims': 'Only these fixed builtin loop/file-input behaviors. Not full Bash conformance, new authored curriculum, learner mastery, native UI or other-platform acceptance.'}
    pin, warning = safe.publish(out, manifest, budget); print(json.dumps({'status': manifest['status'], 'matches': passed, 'mismatches': 16 - passed, 'VM_only_honest_exclusions': excluded_pass, 'manifest_sha256': pin['sha256'], 'durability_warning': warning})); return 0 if accepted else 1


if __name__ == '__main__':
    raise SystemExit(main())
