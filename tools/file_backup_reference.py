#!/usr/bin/env python3
"""Fixed GNU tar comparisons in private fixtures, never learner paths or commands.

No source/path/binary options. All eight restore chains and fixture bytes are
defined below and in the compiled fixed probe. No Bash or shell=True is used.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import time

import case_bash_oracle as safe
from automation_before10_oracle import pinned_executable

ROOT = Path(__file__).resolve().parents[1]
TAR = Path('/usr/bin/tar')
MANUAL = Path('/usr/share/info/tar.info-1.gz')
MANUAL_SHA = 'e2a5577834435166482dee180f55f8192ed21081a7ac5f7f25aeab532ee56365'
PROBE = 'target/debug/examples/file_backup_probe'
SOURCES = ['src/file_backup.rs', 'src/engine.rs', 'src/lib.rs', 'examples/file_backup_probe.rs',
           'examples/support/automation_digest.rs', 'tools/file_backup_reference.py',
           'tools/case_bash_oracle.py', 'tools/automation_before10_oracle.py',
           'tools/build_safety.py', 'tools/dev.py', 'Cargo.toml', 'Cargo.lock']
CHAINS = {
    'base': [('base.tar', True)],
    'sequential': [('base.tar', True), ('day1.tar', True)],
    'overlay': [('base.tar', True), ('day1.tar', False)],
    'delta_alone': [('day1.tar', True)],
    'reverse': [('day1.tar', True), ('base.tar', True)],
    'sequential_day2': [('base.tar', True), ('day1.tar', True), ('day2.tar', True)],
    'differential': [('base.tar', True), ('differential.tar', True)],
    'missing_middle': [('base.tar', True), ('day2.tar', True)],
}


def identity(value):
    safe.require(isinstance(value, str) and re.fullmatch(r'[A-Za-z0-9_-]{1,64}', value),
                 'Simple capture identity only')
    return value


def root_guard(root):
    safe.require((root / '.git').is_dir() and not (root / '.git').is_symlink(), 'MAIN only')
    lock = safe.require_guard(root)
    safe.require(os.environ.get('CARGO_TARGET_DIR') == str(root / 'target'), 'MAIN target only')
    return lock


def source_pins(root):
    pins = [safe.pin(root / path, 2 * 1024 * 1024) for path in SOURCES]
    pins += [safe.pin(root / PROBE, 256 * 1024 * 1024),
             safe.pin(TAR, 4 * 1024 * 1024), safe.pin(MANUAL, 2 * 1024 * 1024)]
    safe.require(pins[-1]['sha256'] == MANUAL_SHA, 'Original primary manual changed')
    return pins


def observation(directory):
    """Only inspect our newly created, bounded fixture; never follow symlinks."""
    result, total = {}, 0
    pending = [(directory, '')]
    while pending:
        location, relative = pending.pop()
        info = location.lstat()
        safe.require(stat.S_ISDIR(info.st_mode), 'Ordinary fixture directory required')
        result[relative] = {'directory': True, 'mode': stat.S_IMODE(info.st_mode), 'text': None}
        with os.scandir(location) as children:
            for child in children:
                safe.require(len(result) + len(pending) < 256, 'Fixture entry bound')
                info = child.stat(follow_symlinks=False)
                name = str(Path(relative) / child.name)
                if stat.S_ISDIR(info.st_mode):
                    pending.append((Path(child.path), name))
                else:
                    safe.require(stat.S_ISREG(info.st_mode), 'No fixture symlinks/special files')
                    raw = safe.read_bytes(Path(child.path), 4096)
                    total += len(raw)
                    safe.require(total <= 32768, 'Fixture byte bound')
                    result[name] = {'directory': False, 'mode': stat.S_IMODE(info.st_mode),
                                    'text': raw.decode('utf-8', errors='strict')}
    return dict(sorted(result.items()))


def create_argv(executable, archive, snapshot):
    safe.require(archive in {'base.tar', 'day1.tar', 'day2.tar', 'differential.tar'}, 'Fixed archive only')
    safe.require(snapshot in {'sequence.snar', 'branch.snar'}, 'Fixed snapshot only')
    return [executable, '--create', '--format=gnu', '--file=' + archive,
            '--listed-incremental=' + snapshot, '--directory=source', 'data']


def extract_argv(executable, archive, destination, incremental):
    safe.require(archive in {'base.tar', 'day1.tar', 'day2.tar', 'differential.tar'}, 'Fixed archive only')
    safe.require(destination in CHAINS, 'Fixed restore directory only')
    return [executable, '--extract', '--file=' + archive,
            '--directory=' + destination, *(['--incremental'] if incremental else [])]


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--capture-id', required=True, type=identity)
    args = parser.parse_args(argv)
    root = ROOT.resolve(strict=True)
    inherited = root_guard(root)
    before, head = source_pins(root), safe.source_identity(root)
    from build_safety import owned_directory
    base = owned_directory(root, root / '.build-artifacts/reviews/file-backup/reference')
    out = safe.private_dir(base, args.capture_id)
    work, temporary = safe.private_dir(out, 'work'), safe.private_dir(out, 'tmp')
    budget, outputs = safe.Budget(16 * 1024 * 1024), []
    started = time.monotonic()
    tar_pin = next(p for p in before if p['path'] == str(TAR))
    probe_pin = next(p for p in before if p['path'] == str(root / PROBE))
    with pinned_executable(TAR, tar_pin, 4 * 1024 * 1024) as (tar, tar_fd), \
         pinned_executable(root / PROBE, probe_pin, 256 * 1024 * 1024) as (probe, probe_fd):
        descriptors = (*inherited, tar_fd, probe_fd)
        serial = 0

        def command(command):
            nonlocal serial
            safe.require(time.monotonic() - started < 120, 'Whole fixed comparison deadline')
            folder = safe.private_dir(out, f'command-{serial:03d}')
            serial += 1
            row = safe.capture_command(command, folder, work, safe.closed_environment({}, temporary),
                                       b'', budget, 65536, inherited=descriptors)
            safe.quiet_group()
            outputs.extend(safe.pin(folder / name, 65536) for name in ['stdout.raw', 'stderr.raw'])
            safe.require(row['status'] == 0 and not row['stderr'], 'Fixed reference command failed; raw evidence retained')
            return row['stdout']

        version = command([tar, '--version'])
        safe.require(version.startswith('tar (GNU tar) 1.35\n'), 'Actual GNU tar1.35 required')
        model = json.loads(command([probe]))
        safe.require(model['kind'] == 'fixed-backup-model-comparison' and model['revision'] == 1
                     and model['compiled_manifest_dir'] == str(root), 'Current fixed probe identity required')
        for key, path in [('model_sha256', 'src/file_backup.rs'), ('probe_sha256', 'examples/file_backup_probe.rs'),
                          ('digest_sha256', 'examples/support/automation_digest.rs')]:
            safe.require(model[key] == next(p for p in before if p['path'] == str(root / path))['sha256'],
                         'Stale compiled model/probe refused')
        source = safe.private_dir(work, 'source')
        data = safe.private_dir(source, 'data')
        notes = safe.private_dir(data, 'notes')
        data.chmod(0o750); notes.chmod(0o750)
        for relative, text in [('notes/keep.txt', 'keep\n'), ('notes/change.txt', 'before\n'),
                               ('notes/remove.txt', 'obsolete\n'), ('.settings', 'private=true\n')]:
            safe.write_new(data / relative, text.encode(), budget, 4096)
            (data / relative).chmod(0o640)
        command(create_argv(tar, 'base.tar', 'sequence.snar'))
        snapshot = safe.read_bytes(work / 'sequence.snar', 65536)
        safe.write_new(work / 'branch.snar', snapshot, budget, 65536)
        (data / 'notes/change.txt').write_text('updated\n')
        (data / 'notes/remove.txt').unlink()  # Exact disposable fixture file only.
        safe.write_new(data / 'notes/new.txt', b'new\n', budget, 4096)
        (data / 'notes/new.txt').chmod(0o640)
        (data / '.settings').chmod(0o600)
        command(create_argv(tar, 'day1.tar', 'sequence.snar'))
        (data / '.settings').write_text('private=still\n')
        command(create_argv(tar, 'day2.tar', 'sequence.snar'))
        command(create_argv(tar, 'differential.tar', 'branch.snar'))
        for filename in ['base.tar', 'day1.tar', 'day2.tar', 'differential.tar', 'sequence.snar', 'branch.snar']:
            outputs.append(safe.pin(work / filename, 1024 * 1024))
        actual = {}
        for name, chain in CHAINS.items():
            safe.private_dir(work, name)
            for archive, incremental in chain:
                command(extract_argv(tar, archive, name, incremental))
            actual[name] = observation(work / name / 'data')
        safe.require(set(model['observations']) == set(CHAINS), 'Every fixed comparison required')
        report = {'actual_tar_version': version.splitlines()[0], 'source_head': head,
                  'compared': 8, 'matches': sum(actual[n] == model['observations'][n] for n in CHAINS),
                  'gnu': actual, 'model': model['observations'],
                  'scope': 'Names, text, directory types and modes only; no binary-format/ctime/ownership conformance'}
        outputs.append(safe.json_new(out / 'COMPARISON.json', report, budget, 2 * 1024 * 1024))
    safe.require(source_pins(root) == before and safe.source_identity(root) == head, 'Source identity changed during reference')
    for output in outputs:
        safe.require(safe.pin(Path(output['path']), 2 * 1024 * 1024) == output, 'Observation changed')
    safe.publish(out, {'pins': before, 'outputs': outputs, 'comparison': report['matches'], 'fixed_cases': 8}, budget)
    print(json.dumps({'path': str(out), 'matches': report['matches'], 'compared': 8}))
    return 0 if report['matches'] == 8 else 1


if __name__ == '__main__':
    raise SystemExit(main())
