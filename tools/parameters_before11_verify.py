#!/usr/bin/env python3
"""Closed actual10 attestation/verification. No learner source or path arguments."""
import argparse
import hashlib
import json
import os
from pathlib import Path
from native_reopen import ordinary as ordinary_fd

ROOT = Path(__file__).resolve().parents[1]
CAPTURE = ROOT / '.build-artifacts/reviews/bash-parameter-defaults/captures/public10-154-350-r1'
EXPECTED = '220cebac329a62adcc59e8338df80b94a07a383807d5832dd6905bbf332776a1'
PRODUCER = ROOT / 'target/debug/examples/parameters_before11_capture'
CAP = 16_000_000


def digest(path, cap=CAP):
    total = 0
    h = hashlib.sha256()
    with ordinary_fd(path, cap) as (stream, before):
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            total += len(chunk)
            if total > cap:
                raise ValueError('Streaming byte bound')
            h.update(chunk)
        after = os.fstat(stream.fileno())
    if (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns) != (
            after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns) or total != before.st_size:
        raise ValueError('File changed during observation')
    return {'bytes': total, 'sha256': h.hexdigest()}


def read(path):
    with ordinary_fd(path, CAP) as (stream, info):
        data = stream.read(info.st_size + 1)
        if len(data) != info.st_size:
            raise ValueError('Evidence changed while reading')
        return data


def child(root, name):
    p = Path(name)
    if p.is_absolute() or not p.parts or '..' in p.parts or str(p) != name:
        raise ValueError('Contained relative evidence name required')
    return root / p


def verify():
    if digest(CAPTURE / 'MANIFEST.json')['sha256'] != EXPECTED:
        raise ValueError('Immutable actual10 manifest changed')
    if read(CAPTURE / 'COMPLETE') != (EXPECTED + '\n').encode():
        raise ValueError('Missing exact completion marker')
    manifest = json.loads(read(CAPTURE / 'MANIFEST.json'))
    for key, value in {'actual_evaluator_revision': 10, 'catalog_contexts': 154,
                       'profile_contexts': 139, 'script_records': 49,
                       'authored_cases': 350, 'examples': 49, 'parameter_proposals': 35}.items():
        if manifest[key] != value:
            raise ValueError('Fixed actual10 scope changed: ' + key)
    seen = set()
    for row in manifest['outputs']:
        if row['file'] in seen or len(seen) >= 128:
            raise ValueError('Duplicate/excessive output identities')
        seen.add(row['file'])
        if digest(child(CAPTURE, row['file'])) != {k: row[k] for k in ('bytes', 'sha256')}:
            raise ValueError('Historical output changed: ' + row['file'])
    source = json.loads(read(CAPTURE / 'SOURCE.json'))
    if source['actual_evaluator_revision'] != 10 or not 1 <= len(source['inputs']) <= 1000:
        raise ValueError('Source closure shape')
    return manifest, source


def attest():
    manifest, source = verify()
    destination = CAPTURE / 'compiled-source'
    if destination.exists():
        raise ValueError('Preserve existing attestation; no overwrite')
    # Current source matching is required only at capture time. Historical
    # verification remains valid after Main advances to evaluator11.
    for row in source['inputs']:
        if digest(child(ROOT, row['file'])) != {k: row[k] for k in ('bytes', 'sha256')}:
            raise ValueError('Source changed after compilation/capture: ' + row['file'])
    producer = digest(PRODUCER, 1024 * 1024 * 1024)
    destination.mkdir(mode=0o700)
    total = 0
    for row in source['inputs']:
        src = child(ROOT, row['file'])
        dst = child(destination, row['file'])
        dst.parent.mkdir(parents=True, exist_ok=True)
        total += row['bytes']
        if total > 256 * 1024 * 1024:
            raise ValueError('Source bundle byte bound')
        with ordinary_fd(src, CAP) as (inp, _), dst.open('xb') as out:
            for chunk in iter(lambda: inp.read(1024 * 1024), b''):
                out.write(chunk)
        if digest(dst) != {k: row[k] for k in ('bytes', 'sha256')}:
            raise ValueError('Copied source changed')
    binary = CAPTURE / 'actual10-producer'
    with ordinary_fd(PRODUCER, 1024 * 1024 * 1024) as (inp, _), binary.open('xb') as out:
        for chunk in iter(lambda: inp.read(1024 * 1024), b''):
            out.write(chunk)
    binary.chmod(0o700)
    if digest(binary, 1024 * 1024 * 1024) != producer:
        raise ValueError('Copied producer changed')
    report = {'manifest_sha256': EXPECTED, 'producer': producer,
              'source_files': len(source['inputs']), 'source_bytes': total,
              'scope': manifest['scope'], 'records_regenerated': False}
    with (CAPTURE / 'ATTESTATION.json').open('x') as stream:
        json.dump(report, stream, indent=2)
        stream.write('\n')
    return report


def main():
    p = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    p.add_argument('action', choices=('attest', 'verify'))
    args = p.parse_args()
    if os.name != 'posix' or not Path('/proc/self/fd').is_dir():
        raise RuntimeError('Fixed Fedora retained-evidence tool; not other-platform acceptance')
    report = attest() if args.action == 'attest' else verify()[0]
    print(json.dumps({'status': 'PASS immutable actual10 evidence',
                      'manifest_sha256': EXPECTED,
                      'scope': report['scope']}, indent=2))


if __name__ == '__main__':
    main()
