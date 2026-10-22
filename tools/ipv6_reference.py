#!/usr/bin/env python3
"""Development-only math comparison. No actual network operations or learner input."""
import ipaddress
import json
import pathlib
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[1]
ADDRESSES = ['::', 'ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff', '2001:db8:abcd:1234:ffff::abcd',
             'fe80::1234', 'fd55:1234:5678:9abc:def0:1234:5678:9abc', '3fff:fff:ffff:ffff::ffff']
fixtures = [{'address': address, 'prefix': prefix} for address in ADDRESSES for prefix in range(129)]
expected = [str(ipaddress.IPv6Network(f'{item["address"]}/{item["prefix"]}', strict=False)) for item in fixtures]
actual = json.loads(subprocess.run([str(ROOT / 'target/debug/examples/ipv6_probe')], input=json.dumps(fixtures),
                                   capture_output=True, text=True, check=True, timeout=3).stdout)
failures = [{'input': fixture, 'expected': wanted, 'actual': got} for fixture, wanted, got in zip(fixtures, expected, actual, strict=True) if wanted != got]
print(json.dumps({'prefix_comparisons': len(fixtures), 'failures': failures}, indent=2))
raise SystemExit(bool(failures))
