#!/usr/bin/env python3
"""Fixed developer-only codec/locale comparison. Never executes learner input.
MAIN runner, one inherited lock, isolated disk fixtures and pinned executables.
This is new selected-input evidence, not a historical seal or compiler closure.
"""
import argparse
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import stat
import time

from automation_before10_oracle import pinned_executable
from case_bash_oracle import capture_command, closed_environment, require_guard
from native_reopen import pin, read_bytes
from startup_reference import Budget, fresh, output

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = 'tests/fixtures/localization-reference-v1.json'
FIXTURE_SHA = '97231e8b204e93264f9e23602f7fc7a39ca5583f80545ca877380a2e9c86d566'
SOURCES = ('Cargo.toml', 'Cargo.lock', 'src/lib.rs', 'src/localization.rs',
           'src/localization/encoding.rs', 'src/cron_workspace.rs',
           'examples/localization_reference.rs', 'examples/support/whereis_digest.rs', FIXTURE)
IDS = ('empty', 'ascii-copy', 'accented-latin1', 'every-latin1-byte',
       'latin1-roundtrip', 'multibyte-copy', 'invalid-utf8-prefix', 'incomplete-utf8',
       'unrepresentable-euro', 'unrepresentable-accent', 'invalid-ascii',
       'overlong-utf8', 'surrogate-utf8', 'multibyte-byte-offset',
       'earlier-unrepresentable-than-invalid', 'invalid-after-accent')
QUERY_IDS = ('c-default', 'utf8-base', 'german-number',
             'german-with-machine-numbers', 'all-overrides-categories',
             'empty-all-falls-back')
ENCODINGS = {'ascii': 'ASCII', 'latin1': 'ISO-8859-1', 'utf8': 'UTF-8'}
LOCALES = {'', 'C', 'C.UTF-8', 'en_US.UTF-8', 'de_DE.UTF-8'}
TOOLS = ('tools/localization_reference.py', 'tools/dev.py', 'tools/build_safety.py',
         'tools/automation_before10_oracle.py', 'tools/case_bash_oracle.py',
         'tools/native_reopen.py', 'tools/startup_reference.py')

def require(ok, why):
    if not ok:
        raise ValueError(why)

def validate_fixture(value):
    require(isinstance(value, dict) and set(value) == {'version', 'conversions', 'queries'}
            and type(value['version']) is int and value['version'] == 1, 'Closed fixture version')
    require(isinstance(value['conversions'], list) and len(value['conversions']) == 16
            and isinstance(value['queries'], list) and len(value['queries']) == 6, 'Exact fixture roster')
    for row, expected in zip(value['conversions'], IDS):
        require(isinstance(row, dict) and set(row) == {'id', 'input', 'from', 'to'}
                and row['id'] == expected, 'Exact inert conversion recipe')
        require(isinstance(row['from'], str) and isinstance(row['to'], str)
                and row['from'] in ENCODINGS and row['to'] in ENCODINGS, 'Fixed codec names only')
        require(isinstance(row['input'], list) and len(row['input']) <= 16384
                and all(type(b) is int and 0 <= b <= 255 for b in row['input']), 'Bound byte input')
    for row, expected in zip(value['queries'], QUERY_IDS):
        require(isinstance(row, dict) and set(row) == {'id', 'env'} and row['id'] == expected, 'Exact locale recipe')
        env = row['env']
        require(isinstance(env, dict) and len(env) <= 4
                and set(env) <= {'LANG', 'LC_ALL', 'LC_CTYPE', 'LC_NUMERIC'}
                and all(isinstance(v, str) and v in LOCALES for v in env.values()),
                'Only fixed locale values; no loader, startup, path or function imports')
    return value

def normalize_locale(name):
    return name.lower().replace('utf-8', 'utf8')

def source_pins():
    return [{'path': name, **pin(ROOT / name, 4 * 1024 * 1024)} for name in SOURCES]

def execute(command, folder, work, env, budget, inherited, deadline, stream_cap=65536):
    remaining = deadline - time.monotonic()
    require(remaining > 0, 'Whole fixed-reference deadline')
    return capture_command(command, folder, work, env, b'', budget, stream_cap,
                           timeout=min(3, remaining), inherited=inherited)

def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args(argv)
    inherited = require_guard(ROOT)
    destination = args.output.absolute()
    require(destination.is_relative_to(ROOT / '.build-artifacts')
            and destination.parent.resolve() == destination.parent,
            'Evidence only in canonical project artifacts, never personal or RAM paths')
    run = fresh(destination.parent, destination.name)
    raw = read_bytes(ROOT / FIXTURE, 128 * 1024)
    require(hashlib.sha256(raw).hexdigest() == FIXTURE_SHA, 'Reviewed fixed fixture bytes changed')
    fixture = validate_fixture(json.loads(raw))
    before = source_pins()
    tool_pins = {name: pin(ROOT / name, 4 * 1024 * 1024) for name in TOOLS}
    programs = {}
    for name, path in [('probe', ROOT / 'target/debug/examples/localization_reference'),
                       ('iconv', Path('/usr/bin/iconv')), ('locale', Path('/usr/bin/locale'))]:
        info = path.lstat()
        require(stat.S_ISREG(info.st_mode) and info.st_mode & 0o111
                and not info.st_mode & 0o6000, 'Fixed ordinary unprivileged executable required: ' + name)
        programs[name] = {'path': str(path), **pin(path, 512 * 1024 * 1024)}
    output(run / 'CONTROL.json', {'scope': 'Selected new-capability sources, not full compiler closure or independent seal',
                                 'sources': before, 'tools': tool_pins, 'programs': programs, 'fixture_sha256': FIXTURE_SHA})
    budget = Budget()
    deadline = time.monotonic() + 60
    env = closed_environment({}, run)
    results, skipped = [], []
    with ExitStack() as stack:
        executables = {}
        for name, record in programs.items():
            executables[name] = stack.enter_context(pinned_executable(Path(record['path']), record, 512 * 1024 * 1024))
        def call(name, args, directory, child_env=env, cap=65536):
            program, fd = executables[name]
            return execute([program, *args], directory, directory, child_env, budget,
                           (*inherited, fd), deadline, cap)
        observed = call('probe', [], fresh(run, 'probe'), cap=512 * 1024)
        require(observed['status'] == 0 and not observed['stderr'], 'In-process fixed probe failed')
        model = json.loads(observed['stdout'])
        require(model['purpose'] == 'localization-capability-reference-v1'
                and model['manifest_dir'] == str(ROOT) and model['sources'] == before,
                'Compiled selected inputs differ from current model; rebuild through MAIN runner')
        require([r['id'] for r in model['conversions']] == list(IDS)
                and [r['id'] for r in model['queries']] == list(QUERY_IDS), 'Exact compiled observation roster')
        for n, (case, expected) in enumerate(zip(fixture['conversions'], model['conversions'])):
            directory = fresh(run, 'codec-' + str(n))
            source = directory / 'input.raw'
            with source.open('xb') as stream:
                stream.write(bytes(case['input']))
            result = call('iconv', ['-f', ENCODINGS[case['from']], '-t', ENCODINGS[case['to']],
                                    '-o', str(directory / 'output.raw'), str(source)], directory)
            require(read_bytes(source, 16384) == bytes(case['input']), 'Reference input changed')
            path = directory / 'output.raw'
            actual = list(read_bytes(path, 32768)) if path.exists() else []
            ok = actual == expected['bytes'] and (result['status'] != 0) == expected['failed'] and result['stdout'] == ''
            record = {'id': case['id'], 'passed': ok, 'expected': expected,
                      'actual_bytes': actual, 'actual_status': result['status'], 'stderr': result['stderr']}
            output(directory / 'comparison.json', record)
            results.append(record)
        available = call('locale', ['-a'], fresh(run, 'available'))
        require(available['status'] == 0 and not available['stderr'], 'Cannot inspect available reference locales')
        names = {normalize_locale(n) for n in available['stdout'].splitlines()}
        for n, (case, expected) in enumerate(zip(fixture['queries'], model['queries'])):
            missing = sorted({v for v in case['env'].values() if v and normalize_locale(v) not in names})
            if missing:
                skipped.append({'id': case['id'], 'reason': 'Reference locales unavailable; no package installed', 'locales': missing})
                continue
            directory = fresh(run, 'locale-' + str(n))
            child_env = dict(env)
            child_env.pop('LC_ALL')
            child_env.update(case['env'])  # Closed four-key fixture schema only.
            for key in ('charmap', 'decimal_point'):
                result = call('locale', [key], fresh(directory, key), child_env)
                actual = result['stdout'].rstrip('\n')
                record = {'id': case['id'] + '/' + key, 'passed': result['status'] == 0
                          and not result['stderr'] and actual == expected[key],
                          'expected': expected[key], 'actual': result}
                output(directory / (key + '.json'), record)
                results.append(record)
    require(source_pins() == before, 'Selected source changed during comparison')
    require({name: pin(ROOT / name, 4 * 1024 * 1024) for name in TOOLS} == tool_pins, 'Reference harness changed during comparison')
    require(any(r['id'] == 'c-default/charmap' for r in results), 'No actual C-locale reference')
    report = {'status': 'PASS' if all(r['passed'] for r in results) else 'MISMATCHES',
              'checks': len(results), 'passed': sum(r['passed'] for r in results), 'skipped': skipped,
              'results': results, 'limits': 'Fixed development codec/locale queries only; no systemd service/config runtime, full compiler closure, independent seal or learner execution.'}
    output(run / 'REPORT.json', report)
    print(json.dumps({k: v for k, v in report.items() if k != 'results'}, indent=2))
    require(report['status'] == 'PASS', 'Reference mismatch; retain diagnostics and fix before authoring')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
