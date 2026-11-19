#!/usr/bin/env python3
"""Fixed, private-database RPM content comparison; never app/learner execution.

No selectors. Builds three tiny script-free fixtures with payloads ONLY inside a
fresh project-disk directory. Personal macros, plugins and transaction scripts
are excluded. Archive names/file lists/scripts are checked before installation.
"""
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import subprocess
import sys
import tempfile
import time

from automation_before10_oracle import pinned_executable
from case_bash_oracle import capture_command, closed_environment
from native_reopen import inherited_lock, pin, read_bytes
from startup_reference import Budget, fresh, output, strict

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = 'tests/fixtures/rpm-configuration-reference.json'
PROBE = 'target/debug/examples/rpm_configuration_probe'
PINS = {'tests/fixtures/rpm-configuration-reference.json': 'f96572a99fa64d7214a0313010a0686a7bea86a223259e738d1411b3c6cf447c', 'examples/rpm_configuration_probe.rs': '494cd446c836d47f955667b775cd11c39e94d3dd1d5480aa62f904a0ec8b407d', 'tests/support/rpm_configuration_fixture.rs': '5fafb3e2934146cd4898a5636424d9bf1e4567284ba47a33ff2e6e942263aac6', 'tests/support/rpm_metadata_fixture.rs': '26b8190959c220c45897f034589ed65408a3e3256280e16a930856c4430dad01', 'src/rpm_configuration.rs': '2362bfbe3f700dccb7479a524bec950abecbe3c5b35bbacd5b92037c1fe50ce7', 'src/rpm_packages.rs': 'd46d1d65065762c7033909c8af7616904fc9d06579acfcfa758c7558f6aa46da'}  # Reviewed current inputs, not historical evidence.
MACROS = ('/usr/lib/rpm/macros', '/usr/lib/rpm/platform/x86_64-linux/macros')
RCFILE = '/usr/lib/rpm/rpmrc'
SCRIPTS = ('PREIN', 'POSTIN', 'PREUN', 'POSTUN', 'PRETRANS', 'POSTTRANS',
           'PREUNTRANS', 'POSTUNTRANS', 'VERIFYSCRIPT', 'TRIGGERSCRIPTS',
           'FILETRIGGERSCRIPTS', 'TRANSFILETRIGGERSCRIPTS')


def require(ok, message):
    if not ok:
        raise ValueError(message)


def inputs():
    require(PINS and FIXTURE in PINS, 'Review fixed inputs before execution')
    for name, sha in PINS.items():
        require(hashlib.sha256(read_bytes(ROOT / name, 1024 * 1024)).hexdigest() == sha,
                'Fixed reviewed RPM input changed; no automatic pin refresh')
    value = strict(read_bytes(ROOT / FIXTURE, 32 * 1024))
    require(set(value) == {'schema_version', 'cases'} and value['schema_version'] == 1,
            'Closed fixture schema')
    rows = value['cases']
    require(len(rows) == 14 and len({r['id'] for r in rows}) == 14, 'Fixed fourteen cases')
    for row in rows:
        require(set(row) == {'id', 'kind', 'operation', 'recorded', 'current', 'incoming'}
                and re.fullmatch('[a-z-]{1,48}', row['id'])
                and row['kind'] in ('config', 'noreplace')
                and row['operation'] in ('upgrade', 'erase')
                and row['recorded'] == 'route=default\n'
                and row['current'] in (None, 'route=default\n', 'route=local\n', 'route=new\n')
                and row['incoming'] in (None, 'route=default\n', 'route=new\n'),
                'Only fixed inert configuration content, flags and operations')
    require([r['operation'] for r in rows] == ['upgrade'] * 10 + ['erase'] * 4,
            'Fixed ordered upgrade/erase roster')
    return rows


def limits():
    resource.setrlimit(resource.RLIMIT_CPU, (60, 60))
    resource.setrlimit(resource.RLIMIT_AS, (1024 * 1024 * 1024,) * 2)
    resource.setrlimit(resource.RLIMIT_FSIZE, (8 * 1024 * 1024,) * 2)
    resource.setrlimit(resource.RLIMIT_NOFILE, (128, 128))
    # Child-only privilege floor; no machine policy is changed.
    import ctypes
    require(ctypes.CDLL(None, use_errno=True).prctl(38, 1, 0, 0, 0) == 0,
            'Child no-new-privileges failed')


def spec(name, version, rows, payload):
    require(name in ('dlms-config-upgrade', 'dlms-config-erase') and version in (1, 2),
            'Fixed development package identity')
    require(re.fullmatch(r'/[A-Za-z0-9_./-]+', str(payload)) and payload == payload.resolve() and payload.is_relative_to(
        ROOT / '.build-artifacts/reference-runs'), 'Private plain package path')
    commands, files = [], []
    for row in rows:
        text = row['recorded'] if version == 1 else row['incoming']
        require(text in ('route=default\n', 'route=new\n'), 'Fixed inert payload')
        target = str(payload / (row['id'] + '.conf'))
        commands.append(f"/usr/bin/printf '%s\\n' '{text.rstrip()}' > \"%{{buildroot}}{target}\"")
        flag = '%config(noreplace)' if row['kind'] == 'noreplace' else '%config'
        files.append(flag + ' ' + target)
    return (f'Name: {name}\nVersion: {version}\nRelease: 1\nSummary: Private content fixture\n'
            'License: MIT\nBuildArch: noarch\nAutoReqProv: no\n'
            '%description\nFixed disposable development data, never distributed.\n'
            '%prep\n%build\n%install\n'
            f'/usr/bin/mkdir -p "%{{buildroot}}{payload}"\n' + '\n'.join(commands)
            + '\n%files\n%defattr(0644,-,-,0755)\n' + '\n'.join(files) + '\n')


def main():
    require(len(sys.argv) == 1, 'Fixed comparison has no source/path/case selectors')
    lock = inherited_lock()
    require(Path.cwd() == ROOT and Path(os.environ.get('CARGO_TARGET_DIR', '')) == ROOT / 'target',
            'MAIN/default-target workflow only')
    rows = inputs()  # Refuse changed content before any child.
    base = ROOT / '.build-artifacts/reference-runs'
    require(base.is_dir() and base.resolve() == base, 'Canonical project-disk directory')
    run = Path(tempfile.mkdtemp(prefix='rpm-configuration-', dir=base))
    payload = fresh(run, 'payload'); db = fresh(run, 'database'); temp = fresh(run, 'temp')
    env = closed_environment({}, temp)
    env.update(HOME=str(fresh(run, 'home')), SOURCE_DATE_EPOCH='1700000000', OMP_NUM_THREADS='1', PATH='/usr/bin:/bin')
    system = {name: Path('/usr/bin', name).resolve(strict=True) for name in ('rpm', 'rpmbuild')}
    system['probe'] = ROOT / PROBE
    identities = {name: {'path': str(path), **pin(path, 512 * 1024 * 1024 if name == 'probe' else 16 * 1024 * 1024)}
                  for name, path in system.items()}
    config = {name: pin(Path(name), 2 * 1024 * 1024) for name in (*MACROS, RCFILE)}
    output(run / 'INPUTS.json', {'reviewed': PINS, 'executables': identities,
                               'rpm_configuration': config, 'cases': rows})
    for name in (*PINS, 'tools/rpm_configuration_reference.py'):
        target = run / 'reviewed' / name
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(read_bytes(ROOT / name, 1024 * 1024))
    common = ['--rcfile', RCFILE, '--macros', ':'.join(MACROS), '--noplugins',
              '--define', f'_tmppath {temp}']
    budget = Budget(); started = time.monotonic(); sequence = 0
    with ExitStack() as stack:
        held = {name: stack.enter_context(pinned_executable(path, identities[name],
                    512 * 1024 * 1024 if name == 'probe' else 16 * 1024 * 1024))
                for name, path in system.items()}
        def command(name, args, timeout=30):
            nonlocal sequence
            require(time.monotonic() - started < 180, 'Whole fixed reference time bound')
            sequence += 1
            folder = fresh(run, f'command-{sequence:02}')
            output(folder / 'argv.json', [identities[name]['path'], *args])
            result = capture_command([held[name][0], *args], folder, run, env, b'', budget,
                                     128 * 1024, timeout=timeout,
                                     inherited=(lock, held[name][1]),
                                     popen=lambda *a, **k: subprocess.Popen(*a, preexec_fn=limits, **k))
            output(folder / 'result.json', result)
            require(result['status'] == 0, 'Reference command failed; retain diagnostics')
            return result['stdout']
        version = command('rpm', [*common, '--version'])
        model = strict(command('probe', [], 5))
        require([r['id'] for r in model] == [r['id'] for r in rows], 'Exact model result roster')
        archives = {}
        for name, v, selected in [('dlms-config-upgrade', 1, rows[:10]),
                                  ('dlms-config-upgrade', 2, rows[:10]),
                                  ('dlms-config-erase', 1, rows[10:])]:
            top = fresh(run, f'build-{name}-{v}')
            for directory in ('BUILD', 'BUILDROOT', 'RPMS', 'SOURCES', 'SPECS', 'SRPMS'):
                fresh(top, directory)
            source = top / 'SPECS/fixture.spec'
            source.write_text(spec(name, v, selected, payload))
            command('rpmbuild', [*common, '--define', f'_topdir {top}', '--define', '_smp_build_ncpus 1',
                                '--define', '_smp_build_nthreads 1', '--define', '_smp_nthreads_max 1',
                                '--define', '_smp_ncpus_max 1',
                                '--define', '__os_install_post %{nil}', '--define', '__spec_install_post %{nil}',
                                '--define', '_build_id_links none', '--define', '_buildhost training.invalid',
                                '--noclean', '-bb', str(source)])
            archive = top / f'RPMS/noarch/{name}-{v}-1.noarch.rpm'
            pin(archive, 1024 * 1024)
            listing = command('rpm', [*common, '-qpl', str(archive)]).splitlines()
            require(set(listing) == {str(payload / (r['id'] + '.conf')) for r in selected}
                    and len(listing) == len(selected), 'No unexpected package payload path')
            script_query = ''.join(f'[%{{{tag}}}]' for tag in SCRIPTS)
            require(command('rpm', [*common, '-qp', '--queryformat', script_query, str(archive)]) == '',
                    'No package scripts/triggers permitted')
            archives[name, v] = archive
        transaction = [*common, '--dbpath', str(db), '--noscripts', '--notriggers', '--nodeps']
        command('rpm', [*transaction, '-i', str(archives['dlms-config-upgrade', 1]),
                        str(archives['dlms-config-erase', 1])])
        for row in rows:
            path = payload / (row['id'] + '.conf')
            require(path.is_file() and not path.is_symlink(), 'Ordinary private installed file')
            if row['current'] is None:
                path.unlink()  # Exact fresh fixture leaf only; never existing artifacts.
            else:
                path.write_text(row['current'])
        command('rpm', [*transaction, '-U', str(archives['dlms-config-upgrade', 2])])
        command('rpm', [*transaction, '-e', 'dlms-config-erase'])
        observed = []
        for row, expected in zip(rows, model, strict=True):
            def text(suffix):
                path = payload / (row['id'] + '.conf' + suffix)
                return read_bytes(path, 4096).decode() if path.exists() else None
            actual = dict(id=row['id'], file=text(''), rpmnew=text('.rpmnew'), rpmsave=text('.rpmsave'))
            observed.append(dict(id=row['id'], actual=actual, model=expected, passed=actual == expected))
        output(run / 'report.json', {'version': version, 'observations': observed,
            'passed': all(r['passed'] for r in observed),
            'scope': 'Fixed content only; not ownership, scripts, dependencies or full RPM conformance'})
        print(json.dumps({'output': str(run), 'cases': len(rows), 'passed': all(r['passed'] for r in observed)}))
        return 0 if all(r['passed'] for r in observed) else 1


if __name__ == '__main__':
    raise SystemExit(main())
