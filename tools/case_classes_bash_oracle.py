#!/usr/bin/env python3
"""Closed class48 developer reference. No learner input, path or probe CLI.

Entry stays MAIN/default-target guarded. Only this NEW collector may observe the
one reviewed W-exclusive ELF; old tools/guards and Cargo targets are unchanged.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import time
try:
    import resource
except ImportError:
    resource = None

import case_bash_oracle as safe
from automation_before10_oracle import pinned_executable
from build_safety import owned_directory
from dev import build_target

ROOT = Path(__file__).resolve().parents[1]
WORKTREE = '.build-artifacts/worktrees/bash-case-validation-164'
FIXED = 'tests/fixtures/shell-case-class-inputs-v12.json'
FIXED_SHA = 'cd2a901fb1cd7850016674596e4b5e1fc720664ef8d0b4f5fd12e9f86d6414bf'
PROBE_SOURCE = 'examples/case_classes_reference.rs'
PROBE_NAME = 'debug/examples/case_classes_reference'
SELF = 'tools/case_classes_bash_oracle.py'
TEST = 'tools/tests/test_case_classes_bash_oracle.py'
VM_MANIFEST = '.build-artifacts/reviews/bash-numeric-validation/case-classes-source-r2/MANIFEST.json'
VM_MANIFEST_SHA = 'a33615013bb7dbec2b59d859ffafe33adc2b8c0b52ddc8ac7a6a5499cab36383'
BASE = '821d58da56083b98f1fdb9b396a3715f577ec9c5'
BASELINE = '.build-artifacts/reviews/bash-numeric-validation/captures/public11-164-384-r1'
BASELINE_SHA = 'e303bdecb9f57a89debd77c8300d27b523c94630446bc918e4c5e4c171373168'
ATTESTATION_SHA = '254d696e4ca90ec508b6034f015ae5710bf2184a30e068d9839a9f93d6523e48'
OUTPUT = '.build-artifacts/reviews/bash-numeric-validation/bash-captures'
CAPTURE = 'classes48-r2'  # New attempt requires a reviewed source revision; no overwrite or arbitrary ID.
CAP = 16_000_000
BINARY_CAP = 256 * 1024 * 1024
TOTAL = 512 * 1024 * 1024
VM_STREAM = 2 * 1024 * 1024
STREAM = 4096
DEADLINE = 120.0
CORE = ('Cargo.toml', 'Cargo.lock', 'src/shell.rs', 'src/shell/arithmetic.rs',
        'src/shell/capture.rs', 'src/shell/case.rs', 'src/shell/cron.rs',
        'src/shell/execution.rs', 'src/shell/getopts.rs', 'src/shell/globbing.rs',
        'src/shell/parameters.rs', 'src/shell/preparation.rs', 'src/shell/substitution.rs',
        'src/shell/text_tools.rs', 'src/shell/values.rs', PROBE_SOURCE,
        'examples/support/automation_digest.rs', FIXED)
# Exact reviewed source hashes and ordered case IDs; never learned from a caller or child.
PROGRAMS = frozenset(('16e4b48a4331c32f29ab1c3aad79243547f281ee89138d7fc714ab76a3928218', '2316ede68dbe6a8d0308aaf3089485067691bcea046f5b701b2e055f661a4c42', '2a382fa49436d5739ff960175fb66ae6e09f1e5832d901a3afe2d9dd4e9ca6d1', '2f4db3f847209c5aa2fbc18ce02d30ce76ac1bef65c2303506839f310bf36401', '35e1f013f5931aeaaa05448be0ac69663d98d163b49804837036b858718786cf', '3ad58a3b7a57e7b06958d60013b6bde07ef7499fdc42df67e1b001baa312385d', '3d6b0abd8878cae5dbc3df5ad4ee706fab0ecc2b9436ed6301796c7213cbdd74', '4d58b552d3a36ae1d51e99fed0142ed346c51bcf8fa931c210a4e00b86c0ebff', '4db137a5d836a0bcf68728f334e519052bbd2866689819bf8e6cfee90b2030d0', '5210b91d9b21fedc74762674757e433658108c46deebfa54219f2fc23cf4473a', '5d40afb5e0d470cda2bfef351ebc33586b4f62e34146c84af679cbb3a55bec74', '60c8c7f0d2cf80544e68ecf28244d6cd433852363cd44f8f8918407a943e8e36', '65a2cd6985daf5798480ed46d7e5d7d959664d7cf5064dccddff977fec99aef8', '75363c2520c21e8a1c106dd137a40a866159ca24ff3679f828759b6b91b06c50', '78a8614e118a538262670edc9890ce27fe943bf5646ff5e685b42539bf76cc1f', '7ba416e0b955f6f456b3dcdbada5c629e4f12b72dbc2b9663dfc1936da24773f', '7dfb92ad5c2a92cfd04a03ad450aca7482df6848957ba38613f6d7e50ae0ef7f', '8eb92d7bdaf650db4fb310b68b75693de52921aa105684276b0f456354979355', '9445917cbdd7b27d51659e33ef62142a4022fece4ef2f1004b1d96acc6079dce', '96700a5ec1c4ff81ad2bb8b6f4250cfd48c733b2af2438bc4db71e9d8b802c53', '9d8d7008ff98bc7d52a5fa7f1833602d8981c0b404325dadfb87bfcd698f7041', 'bc82e00a2fde50713a1882f97b03d28f65045b3230d44077e712c1ab9f62254e', 'bf20a3628ead1c6425ba3f71cd977ae06cc8a69f233dcbbb754ac1f1c026588b', 'd17065e0968659305c751cb7bb30db56616bc68372f2c676af9a59e59f1b919b', 'd390f2dc1fe8eb19b5ece7ce1e152b484c81c913b45edf6bf14d7da8beded71b', 'e71a5597e1bb0078b707bbf5155cd7670e55b035e40f7e26bdb1ecb2c16a3e43', 'fc8e6714678cc25d550875e31273116a679058b1bc0cb01423a0586c67a26670', 'fcc818b3bf71725c5aaef163eae73e504e69744dc66b31796131ebfe667da36b'))
CASE_IDS = ('digit', 'digit-low', 'digit-high', 'digit-no-letter', 'digit-no-empty', 'digit-no-two', 'negation-number', 'negation-letter', 'negation-empty', 'non-digit-tail', 'all-digits', 'ascii-list', 'ascii-list-no', 'dash-first', 'dash-last', 'close-first', 'negated-close', 'adjacent', 'adjacent-no', 'quoted-class', 'quoted-class-not-digit', 'quoted-hyphen', 'quoted-hyphen-literal', 'quoted-open', 'empty-before-star', 'two-digit', 'leading-zero', 'single-zero', 'negated-close-no', 'negated-close-yes', 'close-dash', 'quoted-caret', 'quoted-open-member', 'two-ranges-C', 'two-ranges-no-G', 'across-mask-word', 'across-mask-word-no', 'nonascii-one-class', 'nonascii-two-classes', 'nonascii-anywhere', 'nonascii-one-question', 'nonascii-two-questions', 'whole-value-newline', 'not-a-filename', 'double-quoted-literal', 'double-quoted-not-class', 'negated-range', 'range-upper')


def digest(raw): return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode()


def strict(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            safe.require(key not in result, 'Duplicate JSON key')
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs)


def child(root, name):
    safe.require(isinstance(name, str), 'Fixed relative source name')
    path = Path(name)
    safe.require(bool(path.parts) and not path.is_absolute() and '..' not in path.parts
                 and str(path) == name, 'Contained canonical source name')
    return root / path


def directory(path):
    path = Path(path)
    safe.require(path.is_absolute(), 'Canonical absolute directory')
    at = Path('/')
    for name in path.parts[1:]:
        at /= name
        info = at.lstat()
        safe.require(stat.S_ISDIR(info.st_mode) and not stat.S_ISLNK(info.st_mode), 'No directory aliases')
    return path


def main_entry(root):
    safe.require((root / '.git').is_dir() and not (root / '.git').is_symlink(), 'MAIN checkout only')
    inherited = safe.require_guard(root)
    safe.require(os.environ.get('CARGO_TARGET_DIR') == str(build_target(root, root, None)),
                 'Main/default target only; old guard policy is unchanged')
    return inherited


def check_case(row):
    safe.require(set(row) == {'id', 'source', 'args', 'env', 'stdin', 'files', 'proposed_stdout',
                             'actual_bash', 'actual_vm11', 'actual_future_vm'}, 'Fixed row schema')
    safe.require(isinstance(row['source'], str) and 0 < len(row['source'].encode()) <= 4096
                 and digest(row['source'].encode()) in PROGRAMS, 'Only reviewed builtin-only programs')
    safe.require(row['env'] == {'LC_ALL': 'C'} and row['stdin'] == '' and row['files'] == {},
                 'No startup environment, stdin or host files')
    safe.require(isinstance(row['args'], list) and len(row['args']) == 1
                 and isinstance(row['args'][0], str) and '\0' not in row['args'][0]
                 and len(row['args'][0].encode()) <= 128, 'One fixed quoted subject, never a filename')
    safe.require(row['proposed_stdout'] in ('match\n', 'miss\n')
                 and all(row[k] is None for k in ('actual_bash', 'actual_vm11', 'actual_future_vm')),
                 'Authored predictions only; never fabricate or rewrite observations')


def load_fixed(root):
    raw = safe.read_bytes(root / FIXED, CAP)
    safe.require(digest(raw) == FIXED_SHA, 'Frozen class48 fixture changed')
    value = strict(raw)
    safe.require(set(value) == {'scope', 'capability', 'original_evaluator', 'proposed_future_evaluator',
                               'rows', 'additional_vm_only_negative_categories', 'no_execution_until'}, 'Fixed fixture schema')
    safe.require(value['original_evaluator'] == 11 and value['proposed_future_evaluator'] == 12,
                 'No evaluator relabeling')
    rows = value['rows']
    safe.require(isinstance(rows, list) and len(rows) == 48
                 and tuple(r['id'] for r in rows) == CASE_IDS, 'Exact48 case IDs/order, no omissions or extras')
    safe.require(len({r['source'] for r in rows}) == 28, 'Exactly28 distinct fixed programs')
    for row in rows: check_case(row)
    return rows


def public_files(root):
    ignored = {'.git', '.build-artifacts', 'target', 'dist', 'local-data', 'recordings',
               'screenshots', '.agents', '.aws', '.codex', '__pycache__'}
    result = set()
    directories = 0
    def walk(path, depth=0):
        nonlocal directories
        directories += 1
        safe.require(directories <= 2048 and depth <= 32, 'Bounded source directory walk')
        with os.scandir(path) as entries:
            for entry in entries:
                name = entry.name
                if name in ignored or name == '.env' or name.startswith('.env.') or name.endswith(('.log', '.tmp', '.pyc')):
                    continue  # Never open personal/ignored artifacts.
                info = entry.stat(follow_symlinks=False)
                safe.require(not stat.S_ISLNK(info.st_mode), 'No public source aliases')
                full = path / name
                if stat.S_ISDIR(info.st_mode): walk(full, depth + 1)
                else:
                    safe.require(stat.S_ISREG(info.st_mode), 'Public source must be ordinary files')
                    result.add(full.relative_to(root).as_posix())
                    safe.require(len(result) <= 2048, 'Bounded public source inventory')
    walk(root)
    return result


def clone_proof(root):
    work = directory(root / WORKTREE)
    raw = safe.read_bytes(root / VM_MANIFEST, CAP)
    safe.require(digest(raw) == VM_MANIFEST_SHA, 'Exact independently reviewed VM12 source manifest')
    manifest = strict(raw)
    safe.require(manifest['base'] == BASE and manifest['owned_count'] == 21 and manifest['protected_count'] == 840,
                 'Original21/840 clone boundary is fixed, not derived from current implementation')
    inventory_path = (root / VM_MANIFEST).parent / 'SOURCE-INVENTORY.json'
    inventory_raw = safe.read_bytes(inventory_path, CAP)
    safe.require(digest(inventory_raw) == manifest['inventory']['sha256'], 'Reviewed full clone inventory changed')
    inventory = strict(inventory_raw)
    safe.require(len(inventory['owned']) == 21 and len(inventory['protected']) == 840, 'Exact21/840 inventory')
    expected = {}
    for item in inventory['owned']:
        safe.require(item['path'] not in expected, 'Duplicate reviewed source name')
        expected[item['path']] = item['after']
    for item in inventory['protected']:
        safe.require(item['path'] not in expected, 'Duplicate protected source name')
        expected[item['path']] = {k: item[k] for k in ('bytes', 'sha256')}
    safe.require(PROBE_SOURCE not in expected and public_files(work) == set(expected) | {PROBE_SOURCE},
                 'Full reviewed clone plus ONLY the new fixed probe; data/catalog unchanged')
    for name, wanted in expected.items():
        actual = safe.pin(child(work, name), CAP)
        safe.require({k: actual[k] for k in ('bytes', 'sha256')} == wanted, 'W source drift: ' + name)
    safe.require(safe.read_bytes(work / FIXED, CAP) == safe.read_bytes(root / FIXED, CAP), 'Main/W fixed input bytes identical')
    declaration = safe.read_bytes(work / 'src/shell.rs', CAP)
    safe.require(re.findall(rb'^pub const EVALUATOR_REVISION: u32 = ([0-9]+);$', declaration, re.MULTILINE) == [b'12'],
                 'Actual W evaluator12 required')
    gitdir = root / '.git/worktrees' / work.name
    safe.require(safe.read_bytes(work / '.git', 4096).decode().strip() == 'gitdir: ' + str(gitdir),
                 'Fixed same-project worktree metadata only')
    ref = 'refs/heads/codex/bash-case-validation-164'
    safe.require(safe.read_bytes(gitdir / 'HEAD', 4096).decode().strip() == 'ref: ' + ref, 'Fixed W branch')
    head = safe.read_bytes(root / '.git' / ref, 4096).decode().strip()
    safe.require(re.fullmatch('[0-9a-f]{40}', head) is not None, 'Actual W HEAD required')
    target = build_target(root, work, None)
    directory(target)
    return work, target / PROBE_NAME, expected, {'head': head, 'gitdir': str(gitdir), 'base': BASE,
        'owned': 21, 'protected': 840, 'new_probe': 1, 'data_and_old_source_unchanged': True}


def baseline(root):
    folder = root / BASELINE
    safe.require(digest(safe.read_bytes(folder / 'MANIFEST.json', CAP)) == BASELINE_SHA
                 and digest(safe.read_bytes(folder / 'ATTESTATION.json', CAP)) == ATTESTATION_SHA
                 and safe.read_bytes(folder / 'COMPLETE', 65) == (BASELINE_SHA + '\n').encode(),
                 'Preserve the genuine independently sealed evaluator11 evidence')


def pins(paths):
    safe.require(len(paths) <= 1024, 'Bounded source pin count')
    seen, total = set(), 0
    for path, cap in paths:
        safe.require(str(path) not in seen, 'Duplicate control source'); seen.add(str(path))
        with safe.ordinary(path, cap) as (_, info):
            total += info.st_size
            safe.require(total <= TOTAL, 'Bounded source/ELF aggregate before hashing')
    result, actual_total = [], 0
    for path, cap in paths:
        row = safe.pin(path, cap); actual_total += row['bytes']; result.append(row)
        safe.require(actual_total <= TOTAL, 'Bounded actual source/ELF aggregate')
    return result


def source_paths(root, work, probe, reviewed):
    main = [SELF, TEST, FIXED, 'tools/dev.py', 'tools/build_safety.py',
            'tools/case_bash_oracle.py', 'tools/automation_before10_oracle.py', VM_MANIFEST,
            str(Path(VM_MANIFEST).parent / 'SOURCE-INVENTORY.json'),
            BASELINE + '/MANIFEST.json', BASELINE + '/ATTESTATION.json', BASELINE + '/COMPLETE']
    paths = [(root / name, CAP) for name in main]
    paths += [(work / name, CAP) for name in sorted(set(reviewed) | {PROBE_SOURCE})]
    paths += [(probe, BINARY_CAP), (safe.BASH.resolve(strict=True), CAP), (safe.MANUAL.resolve(strict=True), 8 * CAP)]
    return paths


def approved(raw, control):
    value = strict(raw)
    safe.require(value.get('root_reviewed') is True and value == {**control, 'root_reviewed': True},
                 'Exact independently reviewed current source/input/ELF control required')


def identity(value, work, rows):
    sources = {name: safe.pin(work / name, CAP)['sha256'] for name in CORE}
    safe.require(value == {'kind': 'fixed-case-classes-reference', 'engine_revision': 12,
        'rows': len(rows), 'programs': 28, 'input_sha256': FIXED_SHA, 'manifest_dir': str(work),
        'compiled_sources': sources}, 'Actual fixed W12 ELF/compiled core/fixture identity')


def case_input(row): return {key: row[key] for key in ('args', 'env', 'stdin', 'files')}


def authored(row): return {'stdout': row['proposed_stdout'], 'stderr': '', 'status': 0, 'files': {}, 'error': None}


def vm_rows(text, rows):
    safe.require(len(text.encode()) <= VM_STREAM, 'Bounded VM stream')
    lines = text.splitlines(); safe.require(len(lines) == 48, 'All48 VM observations required')
    observed = []
    for line, row in zip(lines, rows):
        value = strict(line)
        safe.require(set(value) == {'id', 'source_sha256', 'case_sha256', 'outcome'} and value['id'] == row['id']
                     and value['source_sha256'] == digest(row['source'].encode())
                     and value['case_sha256'] == digest(canonical(case_input(row))), 'Exact fixed VM source/input/order')
        out = value['outcome']
        safe.require(set(out) == {'stdout', 'stderr', 'status', 'files', 'error'}
                     and all(isinstance(out[k], str) and len(out[k].encode()) <= 32768 for k in ('stdout', 'stderr'))
                     and type(out['status']) is int and 0 <= out['status'] <= 255
                     and (out['error'] is None or isinstance(out['error'], str) and len(out['error'].encode()) <= 1000)
                     and isinstance(out['files'], dict) and len(out['files']) <= 64, 'Bounded actual VM Outcome shape')
        safe.require(all(isinstance(k, str) and len(k.encode()) <= 256 and '\0' not in k
                         and isinstance(v, str) and len(v.encode()) <= 262144 for k, v in out['files'].items())
                     and sum(len(k.encode()) + len(v.encode()) for k, v in out['files'].items()) <= 262144,
                     'Bounded virtual file result')
        observed.append(out)  # A diagnostic is retained as a failed comparison, never silently dropped.
    return observed


def comparison(row, vm, bash):
    expected = authored(row)
    return {'vm_matches_authored': vm == expected, 'bash_matches_authored': bash == expected,
            'vm_matches_bash': vm == bash, 'passed': vm == bash == expected}


def empty_work(path):
    directory(path)
    with os.scandir(path) as entries:
        safe.require(not any(True for _ in entries), 'Unexpected child filesystem change; never omit entries')
    return {}


def bounded_popen(command, *, vm=False, **kwargs):
    safe.require(resource is not None and 'preexec_fn' not in kwargs, 'POSIX fixed child setup only')
    def limits():
        os.umask(0o077)
        resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
        resource.setrlimit(resource.RLIMIT_CPU, (10, 10) if vm else (2, 2))
        memory = (512 if vm else 256) * 1024 * 1024
        resource.setrlimit(resource.RLIMIT_AS, (memory, memory))
        resource.setrlimit(resource.RLIMIT_NOFILE, (64, 64))
        resource.setrlimit(resource.RLIMIT_FSIZE, (65536, 65536))
    return subprocess.Popen(command, preexec_fn=limits, **kwargs)


def capture_child(command, folder, work, env, budget, inherited, *, vm=False):
    return safe.capture_command(command, folder, work, env, b'', budget, VM_STREAM if vm else STREAM,
        timeout=30 if vm else 2, inherited=inherited,
        popen=lambda cmd, **kw: bounded_popen(cmd, vm=vm, **kw))


def retained_source_parent(destination, name):
    # child enforces a canonical relative source name before any mkdir. Only
    # the two actually reviewed top-level dotted directories receive this local
    # exception; all other directories keep safe.private_dir's old grammar.
    child(destination, name)
    directory(destination)
    parent = destination
    parts = Path(name).parts[:-1]
    safe.require(len(parts) <= 32, 'Bounded reviewed source directory depth')
    for position, component in enumerate(parts):
        if component in ('.cargo', '.github'):
            safe.require(position == 0, 'Only reviewed top-level dotted source directories')
            path = parent / component
            try:
                path.mkdir(mode=0o700)
            except FileExistsError:
                pass
            directory(path)  # lstat every ancestor, including this exact child.
            safe.require(stat.S_IMODE(path.lstat().st_mode) == 0o700,
                         'Expected private0700 source directory; never chmod existing evidence')
            safe.require(path.resolve().parent == parent.resolve(), 'Source directory escaped its parent')
            parent = path
        else:
            try:
                parent = safe.private_dir(parent, component)
            except FileExistsError:
                parent = safe.private_dir(parent, component, existing=True)
    return parent


def retain_child_result(folder, result, files, budget):
    # Called immediately after a complete bounded child and empty-CWD check.
    # Preserve its real return status even if later source retention fails.
    safe.require(set(result) == {'stdout', 'stderr', 'status'}
                 and type(result['status']) is int and files == {}, 'Actual closed child result shape')
    return safe.json_new(folder / 'RESULT.json',
        {'kind': 'fixed-class48-child-observation', 'result': {**result, 'files': files},
         'claims': 'Observed child result only; not COMPLETE or acceptance'}, budget, CAP)


def snapshot_sources(root, work, out, reviewed, probe, executable_pin, before, budget):
    bundle = safe.private_dir(out, 'compiled-source')
    expected = {p['path']: p for p in before}
    retained = []
    for label, base, names in [('worktree', work, sorted(set(reviewed) | {PROBE_SOURCE})),
                               ('main-tooling', root, [SELF, TEST, FIXED, 'tools/dev.py', 'tools/build_safety.py',
                                'tools/case_bash_oracle.py', 'tools/automation_before10_oracle.py'])]:
        destination = safe.private_dir(bundle, label)
        for name in names:
            target = child(destination, name)
            retained_source_parent(destination, name)
            raw = safe.read_bytes(child(base, name), CAP)
            wanted = expected[str(child(base, name))]
            safe.require(len(raw) == wanted['bytes'] and digest(raw) == wanted['sha256'], 'Source changed before retention')
            retained.append(safe.write_new(target, raw, budget, CAP))
    with safe.ordinary(probe, BINARY_CAP) as (stream, info):
        safe.require(safe.pin(probe, BINARY_CAP) == executable_pin, 'Producer changed before retention')
        budget.charge(info.st_size)
        with safe.new_file(out / 'actual12-producer') as target:
            left = info.st_size
            while left:
                block = stream.read(min(left, 1024 * 1024)); safe.require(block, 'Producer shrank'); target.write(block); left -= len(block)
            target.flush(); os.fsync(target.fileno())
        (out / 'actual12-producer').chmod(0o700)
        kept = safe.pin(out / 'actual12-producer', BINARY_CAP)
        safe.require(kept['sha256'] == executable_pin['sha256'], 'Retained producer differs')
        retained.append(kept)
    return retained


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--mode', required=True, choices=['prepare', 'capture'])
    args = parser.parse_args(argv)
    root = ROOT.resolve(strict=True)
    inherited = main_entry(root)
    baseline(root)
    rows = load_fixed(root)
    work, probe, reviewed, clone = clone_proof(root)
    paths = source_paths(root, work, probe, reviewed); before = pins(paths)
    head = safe.source_identity(root)
    safe.require(safe.pin(safe.MANUAL.resolve(strict=True), 8 * CAP)['sha256'] == safe.MANUAL_SHA, 'Original GNU manual required')
    base = owned_directory(root, root / OUTPUT)
    control = {'schema_version': 1, 'root_reviewed': False, 'capture': CAPTURE, 'main_head': head,
        'fixed48_sha256': FIXED_SHA, 'VM12_manifest_sha256': VM_MANIFEST_SHA, 'clone': clone, 'pins': before,
        'baseline11_sha256': BASELINE_SHA, 'actual_expected_vm': 12, 'rows': 48, 'programs': 28,
        'scope': 'Synthetic fixed developer observations only; never learner progress or host execution of learner source'}
    safe.require(not (base / CAPTURE).exists(), 'Preserve all previous capture evidence')
    if args.mode == 'prepare':
        safe.json_new(base / (CAPTURE + '-PROPOSED.json'), control, safe.Budget(CAP), CAP)
        print(json.dumps({'status': 'PROPOSED-NO-EXECUTION', 'rows': 48, 'programs': 28})); return 0
    approved(safe.read_bytes(base / (CAPTURE + '-APPROVED.json'), CAP), control)
    out = safe.private_dir(base, CAPTURE); budget = safe.Budget(TOTAL); start = time.monotonic()
    bash = safe.BASH.resolve(strict=True)
    executable_pin = next(p for p in before if p['path'] == str(probe))
    bash_pin = next(p for p in before if p['path'] == str(bash))
    outputs = []
    with pinned_executable(probe, executable_pin, BINARY_CAP) as (vm, vfd), pinned_executable(bash, bash_pin, CAP) as (sh, bfd):
        held = (*inherited, vfd, bfd)
        def run(name, command, *, is_vm=False):
            safe.require(time.monotonic() - start < DEADLINE, 'Whole reference deadline; partial evidence retained')
            folder = safe.private_dir(out, name); cwd = safe.private_dir(folder, 'work'); tmp = safe.private_dir(folder, 'tmp')
            # The existing closed environment itself fixes LC_ALL/LANG=C and
            # deliberately refuses caller-supplied locale/startup overrides.
            result = capture_child(command, folder, cwd, safe.closed_environment({}, tmp), budget, held, vm=is_vm)
            safe.quiet_group()
            files = empty_work(cwd)
            outputs.append(retain_child_result(folder, result, files, budget))
            outputs.extend(safe.pin(folder / n, VM_STREAM if is_vm else STREAM) for n in ['stdout.raw', 'stderr.raw'])
            return result
        result = run('vm-identity', [vm, '--identity'], is_vm=True)
        safe.require(result['status'] == 0 and not result['stderr'], 'Fixed VM identity completed')
        identity(strict(result['stdout']), work, rows)
        result = run('vm-outcomes', [vm], is_vm=True)
        safe.require(result['status'] == 0 and not result['stderr'], 'Fixed VM observations completed')
        actual_vm = vm_rows(result['stdout'], rows)
        result = run('bash-version', [sh, '--noprofile', '--norc', '--version'])
        safe.require(result['status'] == 0 and not result['stderr']
                     and re.match(r'^GNU bash, version 5\.3\.9\b', result['stdout']), 'Original Bash5.3.9 required')
        observations = []
        for n, (row, actual) in enumerate(zip(rows, actual_vm)):
            result = run(f'case{n:02}', [sh, *safe.BASH_FLAGS, '-c', row['source'], '--', *row['args']])
            bash_result = {**result, 'files': {}, 'error': None}
            observation = {'id': row['id'], 'source_sha256': digest(row['source'].encode()),
                'case_sha256': digest(canonical(case_input(row))), 'authored': authored(row),
                'actual_vm12': actual, 'actual_bash539': bash_result, 'comparison': comparison(row, actual, bash_result)}
            observations.append(observation)
            outputs.append(safe.json_new(out / f'case{n:02}' / 'COMPARISON.json', observation, budget, CAP))
    safe.require(time.monotonic() - start < DEADLINE, 'Whole reference deadline exceeded')
    safe.require(pins(paths) == before and safe.source_identity(root) == head, 'Source/input/binary changed during capture')
    _, _, _, after_clone = clone_proof(root); safe.require(after_clone == clone, 'W identity changed during capture')
    outputs += snapshot_sources(root, work, out, reviewed, probe, executable_pin, before, budget)
    safe.require(pins(paths) == before, 'Source changed during retention')
    outputs.append(safe.json_new(out / 'OBSERVATIONS.json', observations, budget, CAP))
    passed = sum(r['comparison']['passed'] for r in observations)
    manifest = {**control, 'root_reviewed': True, 'observations': 48, 'matches': passed,
        'mismatches': 48 - passed, 'accepted': passed == 48,
        'status': 'MATCHED-ALL48' if passed == 48 else 'COMPLETED-WITH-MISMATCHES-NOT-ACCEPTED',
        'comparisons': {k: sum(r['comparison'][k] for r in observations) for k in
                        ['vm_matches_authored', 'bash_matches_authored', 'vm_matches_bash']},
        'producer': executable_pin, 'outputs': outputs,
        'limits': {'Bash_seconds': 2, 'VM_seconds': 30, 'whole_seconds': 120, 'Bash_stream_bytes': STREAM, 'VM_stream_bytes': VM_STREAM},
        'claims': 'Case matching only; file maps are intentionally empty. Not file-operation fidelity, authored curriculum, mastery, native or other-platform acceptance.'}
    pin, warning = safe.publish(out, manifest, budget)
    print(json.dumps({'status': manifest['status'], 'manifest_sha256': pin['sha256'], 'matches': passed,
                      'mismatches': 48 - passed, 'durability_warning': warning}))
    return 0 if passed == 48 else 1


if __name__ == '__main__':
    raise SystemExit(main())
