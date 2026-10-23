#!/usr/bin/env python3
"""Development-only fstab parser comparison, never a learner or mount runner.

Only fixed checked-in synthetic text is accepted. findmnt reads a private table
with canonicalization/evaluation disabled. Nothing is mounted or written outside
that disposable directory. This compares fields, not kernel mount semantics.
"""
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / 'tests/fixtures/storage-reference.json'
PROBE = ROOT / 'target/debug/examples/storage_probe'
FINDMNT = Path('/usr/bin/findmnt')
OPTIONS = ['--fstab', '--nocanonicalize', '--list', '--json',
           '--output', 'SOURCE,TARGET,FSTYPE,OPTIONS,FREQ,PASSNO']


def validate(fixtures):
    if not isinstance(fixtures, list) or not 1 <= len(fixtures) <= 16:
        raise ValueError('Only bounded checked-in storage fixtures are accepted')
    for item in fixtures:
        if not isinstance(item, dict) or set(item) != {'name', 'configuration'}:
            raise ValueError('Unexpected fixture fields')
        if not isinstance(item['name'], str) or not re.fullmatch('[a-z-]{1,64}', item['name']):
            raise ValueError('Invalid fixture name')
        text = item['configuration']
        if not isinstance(text, str) or len(text.encode()) > 16384 or '\0' in text:
            raise ValueError('Invalid fixture text')


def reference_argv(table):
    # The sole variable argument is a path created by TemporaryDirectory below;
    # no fixture text, source path, command or learner script becomes an argument.
    return [str(FINDMNT), *OPTIONS, '--tab-file', str(table)]


def run(argv, *, work, text=None):
    result = subprocess.run(argv, input=text, text=True, cwd=work,
                            env={'PATH': '/usr/bin:/bin', 'LC_ALL': 'C', 'HOME': str(work)},
                            capture_output=True, timeout=2, check=False)
    if len(result.stdout.encode()) + len(result.stderr.encode()) > 65536:
        raise ValueError('Fixed reference output exceeded its bound')
    if result.returncode or result.stderr:
        raise ValueError(f'Reference/probe failed: {result.returncode}: {result.stderr[:256]}')
    return result.stdout


def normalize(row):
    source = row['source']
    # Independent policy equation over the observed option tokens; not a claim
    # that findmnt exercised ro/rw, kernel mounting, or automatic boot policy.
    ro, automatic = False, True
    for option in row['options'].split(','):
        if option == 'ro':
            ro = True
        elif option == 'rw':
            ro = False
        elif option == 'noauto':
            automatic = False
        elif option == 'auto':
            automatic = True
        elif option != 'defaults':
            raise ValueError('Observed option outside fixed comparison subset')
    return {'source': source, 'target': row['target'], 'fstype': row['fstype'],
            'read_only': ro, 'automatic': automatic,
            'freq': int(row['freq']), 'passno': int(row['passno'])}


def main():
    import sys
    if len(sys.argv) != 1:
        raise ValueError('This development tool accepts no arguments or external fixture paths')
    fixtures = json.loads(FIXTURES.read_text())
    validate(fixtures)
    with tempfile.TemporaryDirectory(prefix='dlms-storage-reference-') as directory:
        work = Path(directory)
        os.chmod(work, 0o700)
        version = run([str(FINDMNT), '--version'], work=work).strip()
        table = work / 'sample.fstab'
        for item in fixtures:
            table.write_text(item['configuration'])
            table.chmod(0o600)
            observed = json.loads(run(reference_argv(table), work=work))['filesystems']
            expected = [normalize(row) for row in observed]
            native = json.loads(run([str(PROBE)], work=work, text=item['configuration']))
            if native != {'entries': expected}:
                raise AssertionError(f"{item['name']}: native={native}, reference={expected}")
    print(f'{len(fixtures)} fixed storage field comparisons passed; {version}. No mounting/evaluation/kernel acceptance.')


if __name__ == '__main__':
    main()
