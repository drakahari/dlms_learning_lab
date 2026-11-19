#!/usr/bin/env python3
"""Fixed nmcli --offline serialization checks, MAIN runner only.

No learner input, profile reading, D-Bus, activation, discovery or arbitrary paths.
This validates three constant keyfile fixtures, not runtime bridge forwarding/STP.
"""
import argparse
import configparser
import json
from pathlib import Path
import tempfile
import time

from automation_before10_oracle import pinned_executable
from case_bash_oracle import (Budget, capture_command, closed_environment,
                              pin, quiet_group, require_guard, require)
from native_reopen import inherited_lock

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path('/usr/bin/nmcli')
# Property/value lists are development constants. Never accept imported profiles,
# interface names, argv, stdin or shell source from a learner or a CLI argument.
BASE = ('--offline', 'connection', 'add')
FIXTURES = (
    ('unbound-port', ('type', 'ethernet', 'con-name', 'offline-unbound',
                      'ifname', 'labnic0', 'connection.uuid',
                      '30000000-0000-4000-8000-000000000001',
                      'connection.autoconnect', 'false',
                      'ipv4.method', 'disabled', 'ipv6.method', 'disabled')),
    ('bound-port', ('type', 'ethernet', 'con-name', 'offline-bound',
                    'ifname', 'labnic1', 'connection.uuid',
                    '30000000-0000-4000-8000-000000000002',
                    'connection.autoconnect', 'false',
                    'connection.controller', 'lab-bridge', 'connection.port-type', 'bridge',
                    'ipv4.method', 'disabled', 'ipv6.method', 'disabled')),
    ('controller', ('type', 'bridge', 'con-name', 'offline-controller',
                    'ifname', 'lab-bridge', 'connection.uuid',
                    '30000000-0000-4000-8000-000000000003',
                    'connection.autoconnect', 'true', 'connection.autoconnect-ports', '1',
                    'bridge.stp', 'true', 'bridge.vlan-filtering', 'false',
                    'ipv4.method', 'manual', 'ipv4.addresses', '10.200.0.20/24', 'ipv6.method', 'disabled')),
)


def verify(name, actual):
    require(name in {n for n,_ in FIXTURES}, 'Fixed offline fixture only')
    if actual['status'] != 0:
        raise RuntimeError(f'{name}: offline fixture rejected; retained diagnostics, no acceptance')
    parser = configparser.ConfigParser(interpolation=None, strict=False)
    parser.read_string(actual['stdout'])
    if name == 'unbound-port':
        require(parser['ipv4']['method'] == parser['ipv6']['method'] == 'disabled', 'Expected disabled IP on unbound fixture')
    elif name == 'controller':
        require(parser['ipv4']['method'] == 'manual' and parser['ipv4']['address1'] == '10.200.0.20/24', 'Manual bridge address normalization mismatch')
        require(parser['ipv6']['method'] == 'disabled', 'Fixed controller IPv6 must be disabled')
    else:
        require('ipv4' not in parser and 'ipv6' not in parser, 'Bound port IP normalization mismatch')
    connection = parser['connection']
    if name == 'unbound-port':
        require('controller' not in connection and 'port-type' not in connection, 'Unbound candidate unexpectedly bound')
        require(connection['autoconnect'] == 'false', 'Fixed port must retain autoconnect=false')
    elif name == 'bound-port':
        require(connection['controller'] == 'lab-bridge' and connection['port-type'] == 'bridge', 'Fixed bound port mismatch')
        require(connection['autoconnect'] == 'false', 'Fixed port must retain autoconnect=false')
    elif name == 'controller':
        require(connection['autoconnect-ports'] == '1', 'Controller must start matching ports')
        require(connection['type'] == 'bridge', 'Controller type mismatch')
        require(parser.has_section('bridge'), 'Expected bridge settings group')
        require(parser.getboolean('bridge','stp',fallback=True), 'This fixed fixture selects the NM enabled STP default')
        require(not parser.getboolean('bridge','vlan-filtering',fallback=False), 'This fixed fixture disables VLAN filtering')
    else:
        raise RuntimeError('Fixed fixture only')


def main(argv=None):
    argparse.ArgumentParser(description=__doc__, allow_abbrev=False).parse_args(argv)
    require_guard(ROOT)
    lock = inherited_lock(ROOT)
    binary = BINARY.resolve(strict=True)
    identity = pin(binary, 32 * 1024 * 1024)
    root = ROOT / '.build-artifacts/references'
    root.mkdir(mode=0o700, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix='nm-bridge-offline-', dir=root))
    budget = Budget(cap=1024 * 1024)
    environment = closed_environment({}, run)
    for key in ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME'):
        environment[key] = str(run / key.lower())
    started = time.monotonic()
    rows = []
    with pinned_executable(binary, identity, 32 * 1024 * 1024) as (executable, fd):
        folder = run / 'version'; folder.mkdir(mode=0o700)
        version = capture_command([executable, '--version'], folder, run, environment,
                                  b'', budget, 8192, timeout=3, inherited=(lock, fd))
        require(version['status'] == 0, 'nmcli version failed')
        for name, properties in FIXTURES:
            require(time.monotonic() - started < 30, 'Whole fixed comparison time bound')
            folder = run / name; folder.mkdir(mode=0o700)
            command = [executable, *BASE, *properties]
            require(command[1:4] == ['--offline', 'connection', 'add'], 'Only fixed offline serialization is allowed')
            actual = capture_command(command, folder, run, environment, b'', budget,
                                     65536, timeout=5, inherited=(lock, fd))
            (folder / 'result.json').write_text(json.dumps(actual, indent=2) + '\n')
            verify(name, actual)
            rows.append({'name': name, 'status': actual['status']})
            quiet_group()
    require(pin(binary, 32 * 1024 * 1024) == identity, 'Observed nmcli binary changed')
    result = dict(status='FIXED-OFFLINE-KEYFILE-FIXTURES-VERIFIED', binary=identity,
                  version=version['stdout'].strip(), cases=rows, output=str(run),
                  scope='Serialization/normalization only. No daemon, host profiles, interfaces, packets, activation, failover or learner execution.')
    (run / 'acceptance.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
