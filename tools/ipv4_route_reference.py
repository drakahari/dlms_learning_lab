#!/usr/bin/env python3
"""Development-only arithmetic comparison: fixed fixtures, no network or learner input."""
import ipaddress
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
ADDRESSES = ['0.0.0.0', '255.255.255.255', '192.168.10.77', '10.20.255.127',
             '172.16.254.129', '192.0.2.1', '198.51.100.254', '203.0.113.128']
fixtures = [{'address': address, 'prefix': prefix} for address in ADDRESSES for prefix in range(33)]
expected = []
for item in fixtures:
    network = ipaddress.IPv4Network(f'{item["address"]}/{item["prefix"]}', strict=False)
    expected.append({'network': str(network), 'last': str(network.broadcast_address)})
actual = json.loads(subprocess.run([str(ROOT / 'target/debug/examples/ipv4_route_probe')],
                                   input=json.dumps(fixtures), capture_output=True,
                                   text=True, check=True, timeout=3).stdout)
failures = [{'input': fixture, 'expected': wanted, 'actual': got}
            for fixture, wanted, got in zip(fixtures, expected, actual, strict=True) if wanted != got]
print(json.dumps({'prefix_comparisons': len(fixtures), 'failures': failures}, indent=2))
raise SystemExit(bool(failures))
