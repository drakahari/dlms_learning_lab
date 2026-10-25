#!/usr/bin/env python3
"""Fixed five-source Bash-only getopts followup. MAIN guarded developer tool.

No VM, catalog, progress, learner source or arbitrary executable/path admission.
Prepare only proposes pins; capture requires a separately approved exact pin set.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time

import case_bash_oracle as safe
from getopts_before9_oracle import capture_id, pinned_executable, strict_json

ROOT = Path(__file__).resolve().parents[1]
HERE = '.build-artifacts/reviews/getopts/scope-oracle'
FIXED = 'tests/fixtures/getopts-scope-oracle-inputs.json'
FIXED_SHA = 'c3b185f00d5be0913837cc5b00a354760b6748dbccf8d84cbbe295d7093c5806'
IDS = ('local-different-offset-fallthrough', 'local-different-offset-return',
       'local-repeat-declaration-one-frame', 'local-dynamic-helper-sharing',
       'unknown-within-cluster-progress')
SOURCE_CAP = 1024 * 1024
CONTROL_CAP = 32 * 1024
BASH_CAP = 16 * 1024 * 1024
PIN_TOTAL_CAP = 32 * 1024 * 1024
EVIDENCE_CAP = 1024 * 1024
STREAM_CAP = 16 * 1024
ROW_CAP = 32 * 1024
WHOLE_SECONDS = 30.0
CASE_SECONDS = 2.0


def load_fixed(root):
    raw = safe.read_bytes(root / FIXED, 16 * 1024)
    safe.require(hashlib.sha256(raw).hexdigest() == FIXED_SHA,
                 'Reviewed fixed5 bytes changed; no arbitrary source is accepted')
    value = strict_json(raw)
    safe.require(set(value) == {'schema_version', 'purpose', 'cases'}
                 and value['schema_version'] == 1, 'Closed fixed5 schema')
    rows = value['cases']
    safe.require(isinstance(rows, list) and len(rows) == 5
                 and tuple(row['id'] for row in rows) == IDS, 'Exactly five ordered reviewed IDs')
    for row in rows:
        safe.require(set(row) == {'id', 'purpose', 'source', 'input', 'actual_bash', 'expected'}
                     and row['actual_bash'] is None and row['expected'] is None,
                     'Input fixture is never an actual observation or expected oracle')
        source = row['source']
        safe.require(isinstance(source, str) and 0 < len(source.encode()) <= 4096
                     and '\0' not in source and '\\"' not in source
                     and '\\\\n' not in source and '\\n' in source,
                     'Fixed source bound/decoded quotation and format bytes')
        safe.require(row['input'] == {'args': [], 'stdin': '', 'env': {},
                                     'files': {'/work/keep.txt': 'retained\n'}},
                     'Only closed empty-stdin/private-keep-file fixture inputs')
    return raw, rows


def fixed_root_guard(root):
    safe.require(root == ROOT.resolve(strict=True) and (root / '.git').is_dir()
                 and not (root / '.git').is_symlink(), 'Preparation/capture require MAIN checkout')
    inherited = safe.require_guard(root)
    safe.require(os.environ.get('CARGO_TARGET_DIR') == str(root / 'target'),
                 'Actual MAIN default target required')
    return inherited


def review_dir(root):
    from build_safety import owned_directory
    return owned_directory(root, root / HERE)


def source_set(root):
    names = ['tools/getopts_scope_oracle.py', 'tools/tests/test_getopts_scope_oracle.py',
             'tools/case_bash_oracle.py', 'tools/getopts_before9_oracle.py',
             'tools/dev.py', 'tools/build_safety.py', FIXED, 'docs/GETOPTS_SCOPE_ORACLE.md']
    return [(root / name, SOURCE_CAP) for name in names] + [
        (safe.MANUAL.resolve(strict=True), 8 * SOURCE_CAP),
        (safe.BASH.resolve(strict=True), BASH_CAP)]


def pins(paths):
    safe.require(len(paths) <= 16 and len({str(p) for p, _ in paths}) == len(paths),
                 'Bounded unique pin list')
    preflight = 0
    for path, cap in paths:
        with safe.ordinary(path, cap) as (_, info):
            preflight += info.st_size
            safe.require(preflight <= PIN_TOTAL_CAP, 'Preflight aggregate pin ceiling')
    result, actual_bytes = [], 0
    for path, cap in paths:
        observed = safe.pin(path, cap)
        safe.require(0 <= observed['bytes'] <= cap, 'Actual same-FD individual pin ceiling')
        actual_bytes += observed['bytes']
        safe.require(actual_bytes <= PIN_TOTAL_CAP, 'Actual aggregate pin ceiling')
        result.append(observed)
    return result


def checked_pins(root):
    result = pins(source_set(root))
    manual = next(p for p in result if p['path'] == str(safe.MANUAL.resolve(strict=True)))
    safe.require(manual['sha256'] == safe.MANUAL_SHA, 'Reviewed primary manual changed')
    return result


def prepare(root, identity):
    capture_id(identity)
    load_fixed(root)
    control = {'schema_version': 1, 'root_reviewed': False, 'capture_id': identity,
               'source_head': safe.source_identity(root), 'pins': checked_pins(root),
               'claims': 'Metadata proposal only; no Bash/version/VM invocation.'}
    observed = safe.json_new(review_dir(root) / ('PROPOSED-' + identity + '.json'),
                             control, safe.Budget(CONTROL_CAP), CONTROL_CAP)
    return {'status': 'PROPOSED-ONLY', **observed}


def approval(root, identity):
    raw = safe.read_bytes(review_dir(root) / ('APPROVED-' + capture_id(identity) + '.json'),
                          CONTROL_CAP)
    control = strict_json(raw)
    safe.require(set(control) == {'schema_version', 'root_reviewed', 'capture_id',
                                  'source_head', 'pins', 'claims'}, 'Closed approval fields')
    safe.require(control['schema_version'] == 1 and control['root_reviewed'] is True
                 and control['capture_id'] == identity, 'Explicit independent approval required')
    safe.require(control['source_head'] == safe.source_identity(root), 'Actual reviewed HEAD changed')
    before = checked_pins(root)
    safe.require(control['pins'] == before, 'Reviewed helper/input/manual/Bash pins changed')
    return raw, control, before


def remaining(start):
    left = WHOLE_SECONDS - (time.monotonic() - start)
    safe.require(left > 0, 'Whole fixed5 deadline; incomplete evidence retained')
    return min(CASE_SECONDS, left)


def publish(out, manifest, budget, start):
    # Reuse no-clobber file/hash/sync helpers, with the deadline checked at the
    # last precommit boundary too. No later failure revokes a published COMPLETE.
    observed = safe.json_new(out / 'MANIFEST.json', manifest, budget, ROW_CAP)
    prepared = safe.write_new(out / 'COMPLETION-PREPARED',
                              (observed['sha256'] + '\n').encode(), budget, 65)
    safe.require(safe.pin(out / 'MANIFEST.json', ROW_CAP) == observed
                 and safe.pin(out / 'COMPLETION-PREPARED', 65) == prepared,
                 'Completion files changed before publication')
    safe.sync_dir(out)
    safe.sync_dir(out.parent)
    remaining(start)
    os.link(out / 'COMPLETION-PREPARED', out / 'COMPLETE')
    warning = None  # COMMITTED: only a durability warning is possible below.
    try:
        safe.sync_dir(out)
    except OSError as error:
        warning = 'COMMITTED; postpublication durability warning: ' + str(error)
    return observed, warning


def capture(root, identity, inherited):
    identity = capture_id(identity)
    started = time.monotonic()
    approved_raw, control, before = approval(root, identity)
    fixed_raw, rows = load_fixed(root)
    parent = safe.private_dir(review_dir(root), 'captures', existing=True)
    out = safe.private_dir(parent, identity)
    budget, outputs = safe.Budget(EVIDENCE_CAP), []
    outputs.append(safe.write_new(out / 'APPROVED.json', approved_raw, budget, CONTROL_CAP))
    outputs.append(safe.write_new(out / 'INPUTS.json', fixed_raw, budget, 16 * 1024))
    outputs.append(safe.json_new(out / 'STARTED.json', {'status': 'started-not-complete',
        'source_head': control['source_head'], 'pins': before}, budget, ROW_CAP))
    bash_path = safe.BASH.resolve(strict=True)
    expected = next(p for p in before if p['path'] == str(bash_path))
    with pinned_executable(bash_path, expected, BASH_CAP) as (bash, fd):
        folder = safe.private_dir(out, 'version')
        work = safe.private_dir(folder, 'work')
        temporary = safe.private_dir(folder, 'tmp')
        version = safe.capture_command([bash, '--noprofile', '--norc', '--version'],
            folder, work, safe.closed_environment({}, temporary), b'', budget,
            safe.VERSION_CAP, timeout=remaining(started), inherited=(*inherited, fd))
        safe.quiet_group()
        safe.require(version['status'] == 0 and not version['stderr']
                     and re.match(r'^GNU bash, version 5\.3\.9\b', version['stdout']),
                     'Actual reviewed Bash5.3.9 version required; failure retained')
        banner = version['stdout'].splitlines()[0]
        outputs += [safe.pin(folder / f, safe.VERSION_CAP) for f in ('stdout.raw', 'stderr.raw')]
        outputs.append(safe.json_new(out / 'VERSION.json', version, budget, ROW_CAP))
        for index, row in enumerate(rows):
            remaining(started)
            folder = safe.private_dir(out, f'fixed-{index:03}')
            work = safe.private_dir(folder, 'work')
            temporary = safe.private_dir(folder, 'tmp')
            initial = safe.materialize(work, row['input']['files'], budget)
            actual = safe.capture_command([bash, *safe.BASH_FLAGS, '-c', row['source'],
                'fixed-getopts-scope-' + str(index)], folder, work,
                safe.closed_environment({}, temporary), b'', budget, STREAM_CAP,
                timeout=remaining(started), inherited=(*inherited, fd))
            safe.quiet_group()
            safe.require(actual['status'] >= 0, 'Signaled Bash observation is incomplete')
            files, count = safe.final_files(work)
            budget.replace_reservation(initial, count)
            safe.require(files == row['input']['files'], 'Unexpected private fixture file changes')
            actual['files'] = files
            outputs += [safe.pin(folder / f, STREAM_CAP) for f in ('stdout.raw', 'stderr.raw')]
            outputs.append(safe.json_new(folder / 'OBSERVATION.json', {
                'id': row['id'], 'source': row['source'], 'input': row['input'],
                'source_sha256': hashlib.sha256(row['source'].encode()).hexdigest(),
                'actual_bash': actual, 'actual_bash_version': banner,
                'future_vm_expectation': None, 'claims': 'Raw fixed Bash evidence only; no VM/history/credit.'},
                budget, ROW_CAP))
            outputs += [safe.pin(work / safe.safe_relative(n), safe.FILE_CAP) for n in files]
    # No more children below this point. Failed/partial evidence is never removed.
    safe.quiet_group()
    remaining(started)
    safe.require(safe.source_identity(root) == control['source_head']
                 and checked_pins(root) == before, 'Pinned source/manual/Bash changed')
    for observed in outputs:
        safe.require(safe.pin(Path(observed['path']), EVIDENCE_CAP) == observed,
                     'Output changed before completion')
    remaining(started)
    manifest = {'schema_version': 1, 'status': 'complete-only-with-valid-COMPLETE',
        'source_head': control['source_head'], 'actual_bash_version': banner,
        'case_rows': len(rows), 'fixed_input_sha256': FIXED_SHA, 'pins': before,
        'outputs': [{'file': Path(p['path']).relative_to(out).as_posix(),
                     'bytes': p['bytes'], 'sha256': p['sha256']} for p in outputs],
        'limits': {'whole_seconds': WHOLE_SECONDS, 'process_seconds': CASE_SECONDS,
                   'stream_bytes': STREAM_CAP, 'version_stream_bytes': safe.VERSION_CAP,
                   'aggregate_evidence_bytes': EVIDENCE_CAP},
        'claims': 'Fixed five Bash observations only; no VM, new baseline, history, learner or curriculum claim.'}
    observed, warning = publish(out, manifest, budget, started)
    return {'status': 'COMMITTED', 'directory': str(out),
            'manifest_sha256': observed['sha256'], 'durability_warning': warning}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    choice = parser.add_mutually_exclusive_group(required=True)
    choice.add_argument('--prepare-id')
    choice.add_argument('--capture-id')
    args = parser.parse_args(argv)
    identity = capture_id(args.prepare_id if args.prepare_id is not None else args.capture_id)
    root = ROOT.resolve(strict=True)
    inherited = fixed_root_guard(root)  # No metadata preparation/child before actual guard.
    result = prepare(root, identity) if args.prepare_id is not None else capture(root, identity, inherited)
    try:
        print(json.dumps(result))  # COMMITTED output is optional, not a validation gate.
    except OSError:
        pass
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (safe.OracleError, OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        try:
            print('Fixed scope oracle aborted; incomplete evidence retained: ' + str(error), file=sys.stderr)
        except OSError:
            pass
        raise SystemExit(1)
