#!/usr/bin/env python3
"""Fixed development VERSION observations via constant NVR framing; no learner input.

Only MAIN tools/dev.py may invoke this tool. No shell, RPM transaction, option,
filename, macro, DB or full-EVR comparison is admitted. Actual execution must
remain in the default execution sandbox; a refusal is not permission to bypass it.
"""
import argparse
from collections import Counter
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import sys
import time

import case_bash_oracle as safe

ROOT = Path(__file__).resolve().parents[1]
VERSION_FIXED = 'tests/fixtures/rpmsort-version-inputs.json'
VERSION_SHA = '70bd89690c9b0cbe29f9dc26b9aab4cd04330053009c5bf2c624089c3777d88f'
FIXED = 'tests/fixtures/rpmsort-version-nvr-inputs.json'
FIXED_SHA = '36143d9912286d9fb40585df0e05c4b026c0ffbc8539a24565912336555a1f17'
ELF = Path('/usr/bin/rpmsort')
ELF_PIN = {'path': str(ELF), 'bytes': 15832,
           'sha256': '54bc254cc76ebe6340bf26e841fe0ad14f243419e2b1f4061a992d2eac0c19aa'}
# Identity was read from ELF notes, never obtained by invoking an option.
PACKAGE = {'name': 'rpm', 'version': '6.0.2-1.fc44', 'architecture': 'x86_64',
           'build_id': 'c556addbd0c3965e0dda0648b924a619f6438896'}
HERE = '.build-artifacts/reviews/rpm-package-foundation/version-oracle'
PROVENANCE = {
    '.build-artifacts/reviews/rpm-package-foundation/rpmsort-contract-r1/CONTRACT.md':
        '7fe2206465c1b2be27ad627fa8b28ec5dd54c153b2407773f39ab621fa309a64',
    '.build-artifacts/reviews/rpm-package-foundation/rpmsort-startup-audit-r1/ELF-AND-WRAPPER-AUDIT.json':
        'c2d7e18b58070d21ab2b39adfd1334f0e0b118e005d8020a9216b6305310f5fe',
}
SOURCE_FILES = (
    'tools/rpm_version_oracle.py', 'tools/tests/test_rpm_version_oracle.py',
    'tools/case_bash_oracle.py', 'tools/build_safety.py', 'tools/dev.py',
    'tools/tests/test_case_bash_oracle.py', FIXED, VERSION_FIXED,
    'docs/RPM_VERSION_REFERENCE.md', 'src/rpm_packages.rs',
    'tests/rpm_packages.rs', 'tests/fixtures/rpm-package-vectors.json',
    'docs/RPM_PACKAGE_SUBSET.md',
)
FILE_CAP = 1024 * 1024
PIN_CAP = 8 * 1024 * 1024
FIXTURE_CAP = 16 * 1024
STREAM_CAP = 4096
EVIDENCE_CAP = 512 * 1024
MANIFEST_CAP = 128 * 1024
ROW_CAP = 16 * 1024
WHOLE_SECONDS = 30.0
CHILD_SECONDS = 2.0
ORDERS = ('a-b', 'b-a')
URLS = [
    {'url': 'https://rpm.org/docs/6.0.x/man/rpmsort.1', 'checked': '2026-10-26'},
    {'url': 'https://rpm.org/docs/6.0.x/man/rpm-version.7', 'checked': '2026-10-26'},
]


def strict_json(raw):
    def unique(pairs):
        out = {}
        for key, value in pairs:
            safe.require(key not in out, 'Duplicate JSON key')
            out[key] = value
        return out
    return json.loads(raw, object_pairs_hook=unique)


def identity(value):
    safe.require(isinstance(value, str) and re.fullmatch(r'[A-Za-z0-9_-]{1,64}', value),
                 'Simple fixed capture ID required')
    return value


def remaining(start):
    left = WHOLE_SECONDS - (time.monotonic() - start)
    safe.require(left > 0, 'Whole rpmsort deadline; incomplete evidence retained')
    return min(CHILD_SECONDS, left)


def main_guard(root):
    safe.require(root == ROOT.resolve(strict=True) and (root / '.git').is_dir()
                 and not (root / '.git').is_symlink(), 'Fixed oracle requires MAIN')
    inherited = safe.require_guard(root)
    safe.require(os.environ.get('CARGO_TARGET_DIR') == str(root / 'target'),
                 'MAIN default target required')
    return inherited


def load_versions(root):
    raw = safe.read_bytes(root / VERSION_FIXED, FIXTURE_CAP)
    safe.require(hashlib.sha256(raw).hexdigest() == VERSION_SHA,
                 'Reviewed fixed20 VERSION input bytes changed')
    value = strict_json(raw)
    safe.require(set(value) == {'schema_version', 'scope', 'source_urls', 'cases', 'observations'}
                 and value['schema_version'] == 1
                 and value['scope'] == 'FIXED_VERSION_COMPONENT_SORT_ONLY'
                 and value['source_urls'] == URLS and value['observations'] == [],
                 'Closed component-only fixture; no invented observations')
    rows = value['cases']
    safe.require(isinstance(rows, list) and len(rows) == 20, 'Exactly20 reviewed pairs')
    for n, row in enumerate(rows, 1):
        safe.require(isinstance(row, dict)
                     and set(row) == {'id', 'a', 'b', 'docs_expected_order', 'meaning'}
                     and row['id'] == f'component-{n:02}'
                     and type(row['docs_expected_order']) is int
                     and row['docs_expected_order'] in (-1, 0, 1)
                     and isinstance(row['meaning'], str) and len(row['meaning']) <= 256,
                     'Fixed ordered pair/expectation identity')
        for key in ('a', 'b'):
            safe.require(isinstance(row[key], str)
                         and re.fullmatch(r'[A-Za-z0-9._+~^]{1,128}', row[key]),
                         'Only reviewed VERSION labels; no flags, EVR, path or source')
    return raw, rows


def load_fixed(root):
    raw = safe.read_bytes(root / FIXED, FIXTURE_CAP)
    safe.require(hashlib.sha256(raw).hexdigest() == FIXED_SHA,
                 'Reviewed fixed20 NVR input bytes changed')
    value = strict_json(raw)
    safe.require(set(value) == {'schema_version', 'scope', 'source_urls',
                                'version_fixture_sha256', 'framing', 'cases', 'observations'}
                 and value['schema_version'] == 1
                 and value['scope'] == 'VERSION_VIA_FIXED_NVR_SORT'
                 and value['source_urls'] == URLS
                 and value['version_fixture_sha256'] == VERSION_SHA
                 and value['framing'] == {'name': 'dlms-oracle', 'release': '1'}
                 and value['observations'] == [], 'Closed fixed NVR framing and provenance')
    _, versions = load_versions(root)
    rows = value['cases']
    safe.require(isinstance(rows, list) and len(rows) == 20, 'Exactly20 framed pairs')
    for row, version in zip(rows, versions):
        safe.require(isinstance(row, dict)
                     and set(row) == set(version) | {'nvr_a', 'nvr_b'}
                     and type(row['docs_expected_order']) is int
                     and {key: row[key] for key in version} == version,
                     'Original VERSION pairs and authored expectations remain exact')
        for key in ('a', 'b'):
            safe.require(row['nvr_' + key] == 'dlms-oracle-' + version[key] + '-1',
                         'Only the fixed identical NAME/RELEASE wrapper is admitted')
        safe.require(len((row['nvr_a'] + '\n' + row['nvr_b'] + '\n').encode('ascii')) <= 258,
                     'Original stdin ceiling also applies to NVR framing')
    return raw, rows


def source_pins(root, start):
    paths = [(root / name, FILE_CAP) for name in (*SOURCE_FILES, *PROVENANCE)]
    paths.append((ELF, FILE_CAP))
    safe.require(len(paths) == len(set(str(p) for p, _ in paths)) <= 16,
                 'Closed unique bounded pin set')
    total = 0
    for path, cap in paths:
        remaining(start)
        with safe.ordinary(path, cap) as (_, info):
            total += info.st_size
            safe.require(total <= PIN_CAP, 'Preflight aggregate pin ceiling')
    actual_total, pins = 0, []
    for path, cap in paths:
        remaining(start)
        observed = safe.pin(path, cap)
        safe.require(0 <= observed['bytes'] <= cap, 'Actual per-file pin bound')
        actual_total += observed['bytes']
        safe.require(actual_total <= PIN_CAP, 'Actual aggregate pin ceiling')
        if path == ELF:
            safe.require(observed == ELF_PIN, 'Exact reviewed rpmsort ELF changed')
        if path.is_relative_to(root):
            expected = PROVENANCE.get(path.relative_to(root).as_posix())
            safe.require(expected is None or observed['sha256'] == expected,
                         'Reviewed startup/manual contract provenance changed')
        pins.append(observed)
    remaining(start)
    return pins


@contextmanager
def executable(expected):
    # Same regular nofollow FD is hashed, retained for exec, then checked at exit.
    with safe.ordinary(ELF, FILE_CAP) as (stream, info):
        safe.require(stat.S_IMODE(info.st_mode) == 0o755,
                     'Reviewed ordinary executable0755 mode required')
        safe.require(stream.read(4) == b'\x7fELF', 'Fixed executable must be ELF')
        stream.seek(0)
        digest = hashlib.sha256()
        left = info.st_size
        while left:
            block = stream.read(min(left, 65536))
            safe.require(bool(block), 'Executable shrank')
            digest.update(block)
            left -= len(block)
        safe.require(not stream.read(1), 'Executable grew')
        observed = {'path': str(ELF), 'bytes': info.st_size, 'sha256': digest.hexdigest()}
        safe.require(observed == expected == ELF_PIN, 'Retained ELF differs from reviewed identity')
        yield '/proc/self/fd/' + str(stream.fileno()), stream.fileno()
        # Rehash the retained FD too, in addition to before/after path pins.
        stream.seek(0)
        digest = hashlib.sha256()
        left = info.st_size
        while left:
            block = stream.read(min(left, 65536))
            safe.require(bool(block), 'Retained executable shrank')
            digest.update(block)
            left -= len(block)
        safe.require(not stream.read(1) and digest.hexdigest() == expected['sha256'],
                     'Retained ELF changed during capture')


def environment(folder):
    # Explicit fresh user directories: never inherit caller config/loader/env.
    home = safe.private_dir(folder, 'home')
    temporary = safe.private_dir(folder, 'tmp')
    xdg = {key: str(safe.private_dir(folder, name)) for key, name in (
        ('XDG_CONFIG_HOME', 'config'), ('XDG_CACHE_HOME', 'cache'), ('XDG_DATA_HOME', 'data'))}
    return {'PATH': '', 'LC_ALL': 'C', 'LANG': 'C', 'HOME': str(home),
            'TMPDIR': str(temporary), 'TMP': str(temporary), 'TEMP': str(temporary), **xdg}


def observation(row, order, actual):
    safe.require(order in ORDERS and set(actual) == {'stdout', 'stderr', 'status'},
                 'Closed result shape')
    safe.require(type(actual['status']) is int and actual['status'] == 0
                 and actual['stderr'] == '', 'rpmsort must exit0 without diagnostics')
    raw = actual['stdout']
    safe.require(isinstance(raw, str) and len(raw.encode()) <= STREAM_CAP
                 and raw.endswith('\n') and '\r' not in raw,
                 'Bounded exact newline-terminated output')
    labels = raw[:-1].split('\n')
    safe.require(Counter(labels) == Counter([row['nvr_a'], row['nvr_b']]),
                 'Output must be exact input multiset; no added/dropped lines')
    expectation = row['docs_expected_order']
    # Sort output cannot establish comparator equality, regardless of tie order.
    expected = ([row['nvr_a'], row['nvr_b']] if expectation < 0
                else [row['nvr_b'], row['nvr_a']])
    compatible = None if expectation == 0 else labels == expected
    return {'id': row['id'], 'input_order': order,
            'version_a': row['a'], 'version_b': row['b'],
            'nvr_a': row['nvr_a'], 'nvr_b': row['nvr_b'], 'stdout': raw,
            'stderr': actual['stderr'], 'status': actual['status'],
            'docs_expected_order': expectation, 'strict_sort_compatible': compatible,
            'comparator_equality_observed': None,
            'claim': 'Tie/equality pending' if expectation == 0 else 'VERSION via identical NAME/RELEASE framing only'}


def publish(out, manifest, budget, start):
    observed = safe.json_new(out / 'MANIFEST.json', manifest, budget, MANIFEST_CAP)
    prepared = safe.write_new(out / 'COMPLETION-PREPARED',
                              (observed['sha256'] + '\n').encode(), budget, 65)
    safe.require(safe.pin(out / 'MANIFEST.json', MANIFEST_CAP) == observed
                 and safe.pin(out / 'COMPLETION-PREPARED', 65) == prepared,
                 'Publication files changed')
    safe.sync_dir(out)
    safe.sync_dir(out.parent)
    remaining(start)
    os.link(out / 'COMPLETION-PREPARED', out / 'COMPLETE')
    warning = None  # COMMITTED: later sync failure is only a durability warning.
    try:
        safe.sync_dir(out)
    except OSError as error:
        warning = 'COMMITTED; postpublication durability warning: ' + str(error)
    return observed, warning


def capture(root, name, inherited):
    name = identity(name)
    start = time.monotonic()
    fixed_raw, rows = load_fixed(root)
    before = source_pins(root, start)
    head = safe.source_identity(root)
    from build_safety import owned_directory
    base = owned_directory(root, root / HERE)
    parent = safe.private_dir(base, 'captures', existing=True)
    out = safe.private_dir(parent, name)
    budget, outputs, observations = safe.Budget(EVIDENCE_CAP), [], []
    outputs.append(safe.write_new(out / 'INPUTS.json', fixed_raw, budget, FIXTURE_CAP))
    outputs.append(safe.json_new(out / 'STARTED.json', {
        'status': 'INCOMPLETE', 'source_head': head, 'source_pins': before,
        'elf_package_metadata': PACKAGE, 'argv': [str(ELF)],
        'startup_evidence': 'Trusted development executable; wrapper audit is qualified, not a transitive proof',
        'claims': 'Fixed VERSION via identical NAME/RELEASE stdio only; no VM/model, package transactions, learner input or fullEVR proof; transitive startup effects not proven',
    }, budget, ROW_CAP))
    with executable(ELF_PIN) as (program, fd):
        for row in rows:
            for order in ORDERS:
                timeout = remaining(start)  # Always positive before child launch.
                folder = safe.private_dir(out, row['id'] + '-' + order)
                work = safe.private_dir(folder, 'work')
                env = environment(folder)
                labels = ([row['nvr_a'], row['nvr_b']] if order == 'a-b'
                          else [row['nvr_b'], row['nvr_a']])
                data = ('\n'.join(labels) + '\n').encode('ascii')
                safe.require(len(data) <= 258, 'Fixed stdin byte ceiling')
                outputs.append(safe.write_new(folder / 'stdin.raw', data, budget, 258))
                safe.quiet_group()
                # Recheck after setup too: no stale timeout crosses whole deadline.
                timeout = min(timeout, remaining(start))
                actual = safe.capture_command([program], folder, work, env, data, budget,
                    STREAM_CAP, timeout=timeout, inherited=(*inherited, fd))
                safe.quiet_group()
                safe.require(not any(work.iterdir()) and all(
                    not any(Path(env[key]).iterdir()) for key in (
                        'HOME', 'TMPDIR', 'XDG_CONFIG_HOME', 'XDG_CACHE_HOME', 'XDG_DATA_HOME')),
                    'Unexpected private-directory side effect; incomplete evidence retained')
                outputs += [safe.pin(folder / key, STREAM_CAP)
                            for key in ('stdout.raw', 'stderr.raw')]
                result = observation(row, order, actual)
                observations.append(result)
                outputs.append(safe.json_new(folder / 'OBSERVATION.json', result, budget, ROW_CAP))
    safe.quiet_group()
    safe.require(len(observations) == 40, 'All40 fixed observations required')
    safe.require(source_pins(root, start) == before and safe.source_identity(root) == head,
                 'Source/fixture/ELF identity changed; incomplete capture retained')
    # Independently recheck every retained output, charging ACTUAL bytes as well as
    # the streaming budget. Pin equality prevents later silent output replacement.
    total = 0
    for observed in outputs:
        remaining(start)
        actual = safe.pin(Path(observed['path']), MANIFEST_CAP)
        total += actual['bytes']
        safe.require(total <= EVIDENCE_CAP and actual == observed,
                     'Retained output identity/actual aggregate changed')
    safe.quiet_group()
    manifest = {'schema_version': 1, 'capture_id': name, 'source_head': head,
                'source_pins': before, 'outputs': outputs, 'elf_package_metadata': PACKAGE,
                'observations': observations, 'child_count': 40,
                'evidence_bytes_before_manifest': budget.used,
                'scope': '20VERSION pairs via fixed identical NAME/RELEASE labels, both orders; no EVR composition or equality proof',
                'strict_cases_compatible': all(r['strict_sort_compatible'] is not False
                                               for r in observations),
                'comparator_equality_observed': None}
    published, warning = publish(out, manifest, budget, start)
    return {'status': 'COMPLETE', 'capture_id': name, 'manifest': published,
            'strict_cases_compatible': manifest['strict_cases_compatible'],
            'durability_warning': warning}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--capture-id', type=identity, required=True)
    values = sys.argv[1:] if argv is None else argv
    args = parser.parse_args(values)
    safe.require(sum(v == '--capture-id' or v.startswith('--capture-id=') for v in values) == 1,
                 'Exactly one capture ID; duplicate flags are not admitted')
    root = ROOT.resolve(strict=True)
    inherited = main_guard(root)
    print(json.dumps(capture(root, args.capture_id, inherited), sort_keys=True))
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OSError, ValueError, safe.OracleError) as error:
        print('rpm_version_oracle: ' + str(error), file=sys.stderr)
        raise SystemExit(1)
