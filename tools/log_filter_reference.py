#!/usr/bin/env python3
"""Fixed ASCII regex comparison; MAIN guarded developer tool, never learner input.

No selectors. Python evaluates twelve SHA-pinned authored fixtures, and a held
fixed probe reports the in-process model's observations. This is not an actual
fail2ban-regex or daemon comparison and not a sealed historical baseline.
"""
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import re
import resource
import subprocess
import sys
import tempfile

from automation_before10_oracle import pinned_executable
from case_bash_oracle import capture_command, closed_environment
from native_reopen import inherited_lock, pin, read_bytes
from startup_reference import Budget, fresh, output, strict

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = 'tests/fixtures/log-filter-reference.json'
PROBE = 'target/debug/examples/log_filter_probe'
PINS = {'tests/fixtures/log-filter-reference.json': 'e4430c20814f101afbbf73c134df418904666cf5fc706578b961b4dff71fd0e8', 'examples/log_filter_probe.rs': 'd4d72529c651634e09f63e5325c5aa9103c9c8c7155de07386408bf50b85cc0c', 'src/log_filter.rs': '24cc08368c6a93cf351d4fa0d66869c73fde35867a592c52f3616d11fd7469cf', 'src/ere.rs': 'e0543af81bb9388b71a1edd1fcfa770efcba12eab3847046079a36b95fb3db7d', 'src/ere/cooperative.rs': 'd71ff8619a5ce77a0a55ff337785600cd1e926cb007cfc5ce3e0c18de7304d62'}  # Explicit reviewed current source pins populated before first run.


def require(ok, message):
    if not ok:
        raise ValueError(message)


def inputs():
    require(PINS and FIXTURE in PINS, 'Review fixed inputs before execution')
    for name, expected in PINS.items():
        require(hashlib.sha256(read_bytes(ROOT / name, 1024 * 1024)).hexdigest() == expected,
                'Fixed reviewed log-filter input changed; no automatic refresh')
    value = strict(read_bytes(ROOT / FIXTURE, 32 * 1024))
    require(set(value) == {'schema_version', 'cases'} and value['schema_version'] == 1,
            'Closed fixture schema')
    rows = value['cases']
    require(len(rows) == 12 and len({r['id'] for r in rows}) == 12,
            'Fixed twelve definitions')
    for row in rows:
        require(set(row) == {'id', 'filter', 'lines'}
                and re.fullmatch('[a-z-]{1,48}', row['id'])
                and isinstance(row['filter'], str) and row['filter'].isascii()
                and 0 < len(row['filter']) < 2048
                and isinstance(row['lines'], list) and len(row['lines']) == 4
                and all(isinstance(s, str) and s.isascii() and len(s) <= 512
                        and not any(ord(c) < 32 for c in s) for s in row['lines']),
                'Only fixed short ASCII definitions and inert messages')
    return rows


def expected(case):
    # This independent reference uses Python's actual named capture, not the
    # model's literal-address substitution or its ERE implementation.
    values = {}
    for line in case['filter'].splitlines():
        if '=' in line:
            key, value = line.split('=', 1)
            values[key.strip()] = value.strip()
    pattern = values['failregex'].replace('<HOST>', r'(?P<host>[0-9]+(?:\.[0-9]+){3})')
    failure = re.compile(pattern, re.ASCII)
    ignore = re.compile(values['ignoreregex'], re.ASCII) if values.get('ignoreregex') else None
    rows = []
    for number, line in enumerate(case['lines'], 1):
        match = failure.search(line)
        valid = False
        if match and match.group('host') is not None:
            try:
                ipaddress.IPv4Address(match.group('host'))
                valid = True
            except ValueError:
                pass
        disposition = 'ignored' if ignore and ignore.search(line) else 'matched' if valid else 'missed'
        rows.append(dict(line=number, text=line, disposition=disposition))
    return dict(id=case['id'], rows=rows)


def limits():
    resource.setrlimit(resource.RLIMIT_CPU, (5, 5))
    resource.setrlimit(resource.RLIMIT_AS, (1024 * 1024 * 1024,) * 2)
    resource.setrlimit(resource.RLIMIT_FSIZE, (1024 * 1024,) * 2)
    resource.setrlimit(resource.RLIMIT_NOFILE, (64, 64))
    import ctypes
    require(ctypes.CDLL(None, use_errno=True).prctl(38, 1, 0, 0, 0) == 0,
            'Child no-new-privileges failed')


def main():
    require(len(sys.argv) == 1, 'Fixed comparison has no source/path/case selectors')
    lock = inherited_lock()
    require(Path.cwd() == ROOT and Path(os.environ.get('CARGO_TARGET_DIR', '')) == ROOT / 'target',
            'MAIN/default-target workflow only')
    cases = inputs()  # Refuse any changed input before child/reference evaluation.
    reference = [expected(case) for case in cases]
    base = ROOT / '.build-artifacts/reference-runs'
    require(base.is_dir() and base.resolve() == base, 'Canonical project-disk directory')
    run = Path(tempfile.mkdtemp(prefix='log-filter-', dir=base))
    temp = fresh(run, 'temp')
    env = closed_environment({}, temp)
    env['HOME'] = str(fresh(run, 'home'))
    probe = ROOT / PROBE
    identity = {'path': str(probe), **pin(probe, 512 * 1024 * 1024)}
    output(run / 'INPUTS.json', {'reviewed': PINS, 'probe': identity,
                               'python_version': sys.version, 'cases': cases})
    for name in (*PINS, 'tools/log_filter_reference.py'):
        target = run / 'reviewed' / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(read_bytes(ROOT / name, 1024 * 1024))
    folder = fresh(run, 'model')
    with pinned_executable(probe, identity, 512 * 1024 * 1024) as held:
        result = capture_command([held[0]], folder, run, env, b'', Budget(),
                                 64 * 1024, timeout=5, inherited=(lock, held[1]),
                                 popen=lambda *a, **k: subprocess.Popen(*a, preexec_fn=limits, **k))
    output(folder / 'result.json', result)
    require(result['status'] == 0, 'Fixed probe failed; retain diagnostics')
    model = strict(result['stdout'])
    require(isinstance(model, list) and [r['id'] for r in model] == [r['id'] for r in cases],
            'Exact model roster')
    observations = [dict(id=a['id'], model=a, reference=b, passed=a == b)
                    for a, b in zip(model, reference, strict=True)]
    passed = all(row['passed'] for row in observations)
    output(run / 'report.json', {'observations': observations, 'passed': passed,
                                'scope': 'Twelve fixed ASCII common-subset filters /48lines against Python re. Not actual Fail2ban, daemon, general regex conformance or independent seal.'})
    print(json.dumps({'output': str(run), 'definitions': 12, 'lines': 48, 'passed': passed}))
    return 0 if passed else 1


if __name__ == '__main__':
    raise SystemExit(main())
