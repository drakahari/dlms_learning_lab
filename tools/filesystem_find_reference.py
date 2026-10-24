"""Fixed synthetic find comparator, imported by the existing allowlisted tool.

There is no executable CLI, learner script/path, external pack or arbitrary argv
entry point. GNU find is only a development reference, never a learner runtime.
"""
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import shlex
import stat
import tempfile
import time

import case_bash_oracle as finite

LEGACY_SHA = 'f69ea58baeebfc76aa1711d81ae1395ee3b023f7ec5b8abc72ec59125a2f55ea'
FIND_SHA = '42282afe4c538baba58dc8597e733a5378bb14d6fc418d9bb131b5a4609f7d83'
FIXTURE = 'tests/fixtures/filesystem-find-reference.json'
FIND = Path('/usr/bin/find')
INPUT_CAP = 128 * 1024
STREAM_CAP = 32 * 1024
PROBE_CAP = 1024 * 1024
DEADLINE = 90


def require(ok, message):
    finite.require(ok, message)


def safe_name(name):
    require(isinstance(name, str) and 0 < len(name.encode()) <= 1024
            and all(c.isprintable() for c in name), 'Fixed relative name bounds')
    relative = PurePosixPath(name)
    require(bool(relative.parts) and not relative.is_absolute() and name == relative.as_posix()
            and not set(relative.parts) & {'.', '..'} and len(relative.parts) <= 6
            and all(len(p.encode()) <= 255 for p in relative.parts), 'Canonical contained fixture entry required')
    return relative


def contained_start(start):
    require(isinstance(start, str) and 0 < len(start.encode()) <= 1024
            and all(c.isprintable() for c in start) and not start.startswith(('/', '-'))
            and start not in ('!', '('), 'One relative contained find start required')
    parts = []
    for part in start.split('/'):
        if part in ('', '.'):
            continue
        if part == '..':
            require(bool(parts), 'Find start escapes private fixture')
            parts.pop()
        else:
            require(len(part.encode()) <= 255 and len(parts) < 64, 'Find start component bounds')
            parts.append(part)
    return tuple(parts)


def pattern(value):
    require(isinstance(value, str) and len(value.encode()) <= 255
            and all(c.isprintable() for c in value), 'Fixed C-byte pattern bounds')
    escaped = False
    for c in value:
        if escaped:
            escaped = False
        elif c == '\\':
            escaped = True
        else:
            require(c not in '[]', 'Find brackets require literal escapes')
    require(not escaped, 'Unpaired pattern escape')


def validate_argv(argv, *, invalid_depth=False):
    require(isinstance(argv, list) and 2 <= len(argv) <= 37
            and all(isinstance(a, str) and '\0' not in a for a in argv)
            and sum(len(a.encode()) + 1 for a in argv) <= 4096
            and argv[0] == 'find', 'Fixed find argv bounds/head')
    contained_start(argv[1])
    i, predicates = 2, 0
    if i < len(argv) and argv[i] == '-maxdepth':
        require(i + 1 < len(argv) and 1 <= len(argv[i + 1]) <= 10
                and argv[i + 1].isascii() and argv[i + 1].isdecimal()
                and (int(argv[i + 1]) <= 2147483647
                     or (invalid_depth and argv[i + 1] in ('2147483648', '4294967295'))),
                'Fixed signed nonnegative depth subset or exact negative control')
        if invalid_depth:
            require(argv[i + 1] in ('2147483648', '4294967295'), 'Exact reviewed invalid-depth control required')
        i += 2
    require(not invalid_depth or (len(argv) >= 4 and argv[2] == '-maxdepth'), 'Invalid-depth control needs leading maxdepth')
    while i < len(argv):
        if argv[i] == '-print' and i + 1 == len(argv):
            return
        require(predicates < 16 and i + 1 < len(argv), 'Fixed predicate count/operand')
        if argv[i] == '-name':
            pattern(argv[i + 1])  # Pattern DATA, never an executable or path operand.
        elif argv[i] == '-type':
            require(argv[i + 1] in ('f', 'd'), 'Fixed type subset')
        else:
            raise finite.OracleError('Unsupported find operator/action/option')
        predicates += 1
        i += 2


def load_legacy(path):
    raw = finite.read_bytes(path, INPUT_CAP)
    require(hashlib.sha256(raw).hexdigest() == LEGACY_SHA, 'Original79 fixture bytes changed; review required')
    data = json.loads(raw)
    require(sum(len(f['commands']) for f in data[:8]) == 79
            and sum(len(f['commands']) for f in data) == 152,
            'Original79 prefix or subsequent73 permission comparisons changed')
    return data


def validate_fixtures(data):
    require(isinstance(data, dict) and set(data) == {'schema_version', 'fixtures'}
            and type(data['schema_version']) is int and data['schema_version'] == 1 and isinstance(data['fixtures'], list)
            and 1 <= len(data['fixtures']) <= 12, 'Fixed fixture schema/count')
    count, names = 0, set()
    for fixture in data['fixtures']:
        require(isinstance(fixture, dict) and set(fixture) == {'name', 'entries', 'commands'}, 'Fixed fixture fields')
        name = fixture['name']
        require(isinstance(name, str) and 0 < len(name) <= 64 and name not in names
                and set(name) <= set('abcdefghijklmnopqrstuvwxyz0123456789-'), 'Fixed fixture identifier')
        names.add(name)
        entries = fixture['entries']
        require(isinstance(entries, dict) and len(entries) <= 64, 'Fixed entry count')
        total = 0
        for name, entry in entries.items():
            relative = safe_name(name)
            require(isinstance(entry, dict) and set(entry) in ({'mode'}, {'mode', 'text'})
                    and type(entry['mode']) is int and 0 <= entry['mode'] <= 0o777, 'Fixed entry shape/mode')
            if len(relative.parts) > 1:
                parent = relative.parent.as_posix()
                require(parent in entries and 'text' not in entries[parent], 'Declared directory parent required')
            if 'text' in entry:
                require(isinstance(entry['text'], str) and '\0' not in entry['text']
                        and len(entry['text'].encode()) <= 1024, 'Fixed file text bounds')
                total += len(entry['text'].encode())
        require(total <= 32 * 1024, 'Fixed file aggregate')
        commands = fixture['commands']
        require(isinstance(commands, list) and 1 <= len(commands) <= 16, 'Fixed query count')
        for command in commands:
            require(isinstance(command, dict) and set(command) == {'source', 'comparison', 'expected_paths'}
                    and isinstance(command['source'], str) and len(command['source'].encode()) <= 4096,
                    'Fixed query fields/source ceiling')
            argv = shlex.split(command['source'])
            require(command['comparison'] in ('complete', 'incomplete', 'lookup_error', 'unsupported', 'rejected'), 'Known comparison mode')
            validate_argv(argv, invalid_depth=command['comparison'] == 'rejected')
            paths = command['expected_paths']
            require(isinstance(paths, list) and len(paths) <= 66
                    and all(isinstance(p, str) and len(p.encode()) <= 2048
                            and all(c.isprintable() for c in p) for p in paths) and len(paths) == len(set(paths)), 'Independent path table bounds')
            count += 1
    require(count <= 96, 'Fixed command aggregate')
    return data['fixtures']


def load_fixed(root):
    raw = finite.read_bytes(root / FIXTURE, INPUT_CAP)
    require(hashlib.sha256(raw).hexdigest() == FIND_SHA, 'Reviewed find inputs changed; review required')
    return raw, validate_fixtures(json.loads(raw))


def materialize(work, entries, budget):
    for name in sorted(entries, key=lambda n: (len(PurePosixPath(n).parts), n)):
        entry, path = entries[name], work / safe_name(name)
        if 'text' in entry:
            finite.write_new(path, entry['text'].encode(), budget, 1024)
        else:
            path.mkdir(mode=0o700)
    for name in sorted(entries, key=lambda n: len(PurePosixPath(n).parts), reverse=True):
        (work / name).chmod(entries[name]['mode'])


def snapshot(work, entries):
    """Inspect known private entries; restore every temporarily raised mode.

    Read/find/inspection may change host atime and temporary chmod changes ctime.
    Neither is a promised comparison field. Namespace/type/inode/ownership/mode,
    mtime, size and contents must stay exact across the actual reference query.
    """
    info = work.stat(follow_symlinks=False)
    require(stat.S_ISDIR(info.st_mode), 'Private work root is an ordinary directory')
    result = {'.': {'directory': True, 'mode': info.st_mode & 0o777, 'device': info.st_dev,
                    'inode': info.st_ino, 'uid': info.st_uid, 'gid': info.st_gid, 'mtime_ns': info.st_mtime_ns}}
    saved = []
    try:
        for name in sorted(entries, key=lambda n: (len(PurePosixPath(n).parts), n)):
            path = work / safe_name(name)
            info = path.stat(follow_symlinks=False)
            require(stat.S_ISDIR(info.st_mode) or stat.S_ISREG(info.st_mode), 'Unexpected private entry type')
            directory = stat.S_ISDIR(info.st_mode)
            require(directory == ('text' not in entries[name]), 'Private entry kind changed')
            mode = info.st_mode & 0o777
            saved.append((path, mode))
            path.chmod(mode | (0o700 if directory else 0o400))
            row = {'directory': directory, 'mode': mode, 'device': info.st_dev, 'inode': info.st_ino,
                   'uid': info.st_uid, 'gid': info.st_gid, 'mtime_ns': info.st_mtime_ns}
            if not directory:
                row.update(bytes=info.st_size, text=finite.read_bytes(path, 1024).decode('utf-8'))
            result[name] = row
        actual = set()
        for folder, dirs, files in os.walk(work, followlinks=False):
            for name in dirs + files:
                path = Path(folder) / name
                require(not path.is_symlink(), 'Private fixture acquired a symlink')
                actual.add(path.relative_to(work).as_posix())
        require(actual == set(entries), 'Private namespace changed')
    finally:
        for path, mode in reversed(saved):
            path.chmod(mode)
    return result


def simulated_paths(step):
    if step['ok']:
        require(step['error'] is None, 'Successful probe step has no error')
        return step['output'].splitlines()
    require(step['output'] == '' and isinstance(step['error'], str), 'Additive probe error field required')
    text = step['error']
    if not text.startswith('Search incomplete. Partial matches:\n'):
        return []
    paths, separator, _ = text[len('Search incomplete. Partial matches:\n'):].partition('Diagnostics:\n')
    require(bool(separator), 'Incomplete query must retain labeled diagnostics')
    return paths.splitlines()


def compare_observation(command, actual, native):
    mode = command['comparison']
    observed = simulated_paths(actual)
    require(len(observed) == len(set(observed)), 'Probe emitted duplicate paths')
    require(sorted(observed) == sorted(command['expected_paths']), 'Simulator differs from independent fixed path table')
    if mode == 'unsupported':
        require(not actual['ok'] and 'Unsupported descent' in actual['error'], 'Boundary must be explicitly unsupported/incomplete')
        return {'boundary': 'GNU behavior retained; not compared as supported', 'native_status': native['status']}
    if mode == 'rejected':
        require(not actual['ok'] and '-maxdepth' in actual['error']
                and 'Partial matches' not in actual['error'], 'Invalid depth must fail before traversal')
    complete = mode == 'complete'
    require(actual['ok'] == complete and (native['status'] == 0) == complete, 'Reference status/error mismatch')
    require(sorted(native['stdout'].splitlines()) == sorted(observed), 'Reference path set or entered root spelling differs')
    if complete:
        require(native['stderr'] == '', 'Successful reference has unexpected diagnostics')
    else:
        require(bool(native['stderr']) and bool(actual['error']), 'Failed traversal requires diagnostics')
    return {'matched': True}


def preflight(root, probe):
    """Guard every old/new subprocess, not only the later find comparisons."""
    root = root.resolve(strict=True)
    require((root / '.git').is_dir() and not (root / '.git').is_symlink()
            and probe == root / 'target/debug/examples/filesystem_probe', 'MAIN default probe only')
    inherited = finite.require_guard(root)
    require(os.getuid() != 0, 'Ordinary-user GNU permission comparisons required')
    require(not probe.is_symlink() and os.access(probe, os.X_OK), 'Actual rebuilt MAIN probe required')
    # Same-FD, no-follow check of every path component and ordinary file type/size.
    # Rechecked/pinned again around new comparisons after the legacy phase.
    finite.pin(probe, 1024 * 1024 * 1024)
    return root, inherited


def compare_fixed(root, probe, make_tree):
    root, inherited = preflight(root, probe)
    raw, fixtures = load_fixed(root)
    load_legacy(root / 'tests/fixtures/filesystem-reference.json')
    paths = [(root / name, INPUT_CAP) for name in [FIXTURE, 'tests/fixtures/filesystem-reference.json',
             'tools/filesystem_reference.py', 'tools/filesystem_find_reference.py', 'tools/case_bash_oracle.py',
             'examples/filesystem_probe.rs', 'src/filesystem.rs', 'src/filesystem/find.rs']]
    paths += [(probe, 1024 * 1024 * 1024), (FIND, 64 * 1024 * 1024)]
    before = [finite.pin(path, cap) for path, cap in paths]
    temporary = Path(os.environ['TMPDIR']).resolve(strict=True)
    out = Path(tempfile.mkdtemp(prefix='filesystem-find-reference-', dir=temporary))
    out.chmod(0o700)
    budget = finite.Budget(cap=16 * 1024 * 1024)
    outputs = [finite.write_new(out / 'INPUTS.json', raw, budget, INPUT_CAP)]
    outputs.append(finite.json_new(out / 'STARTED.json', {'status': 'not-complete', 'source': finite.source_identity(root), 'pins': before}, budget))
    env = {'LC_ALL': 'C', 'LANG': 'C', 'PATH': '', 'TZ': 'UTC', 'TMPDIR': str(temporary), 'TMP': str(temporary), 'TEMP': str(temporary)}
    version_dir = finite.private_dir(out, 'version')
    version = finite.capture_command([str(FIND), '--version'], version_dir, version_dir, env, b'', budget, 4096, timeout=2, inherited=inherited)
    require(version['status'] == 0 and version['stdout'].startswith('find (GNU findutils) ') and not version['stderr'], 'Actual GNU find identity required')
    outputs.extend(finite.pin(version_dir / name, 4096) for name in ('stdout.raw', 'stderr.raw'))
    finite.quiet_group()
    failures, count = [], 0
    deadline = time.monotonic() + DEADLINE
    for index, fixture in enumerate(fixtures):
        require(time.monotonic() < deadline, 'Fixed reference total deadline')
        folder = finite.private_dir(out, f'fixture-{index:02}')
        work = finite.private_dir(folder, 'work')
        materialize(work, fixture['entries'], budget)
        initial = make_tree(fixture['entries'])
        initial.update(previous=None, clock=0, modified={})
        commands = [c['source'] for c in fixture['commands']]
        probe_dir = finite.private_dir(folder, 'probe')
        data = json.dumps({'tree': initial, 'commands': commands}, ensure_ascii=False).encode()
        require(len(data) <= 64 * 1024, 'Fixed probe stdin ceiling')
        simulated = finite.capture_command([str(probe)], probe_dir, work, env, data, budget, PROBE_CAP, timeout=3, inherited=inherited)
        finite.quiet_group()
        outputs.extend(finite.pin(probe_dir / name, PROBE_CAP) for name in ('stdout.raw', 'stderr.raw'))
        require(simulated['status'] == 0 and not simulated['stderr'], 'Actual MAIN probe failed')
        steps = json.loads(simulated['stdout'])
        require(isinstance(steps, list) and len(steps) == len(commands), 'Probe step shape/count')
        baseline = snapshot(work, fixture['entries'])
        for j, (command, step) in enumerate(zip(fixture['commands'], steps, strict=True)):
            require(time.monotonic() < deadline, 'Fixed reference total deadline')
            argv = shlex.split(command['source'])
            validate_argv(argv, invalid_depth=command['comparison'] == 'rejected')
            require(step['tree'] == initial, 'Query mutated simulated state/metadata')
            command_dir = finite.private_dir(folder, f'query-{j:02}')
            native = finite.capture_command([str(FIND), *argv[1:]], command_dir, work, env, b'', budget, STREAM_CAP, timeout=2, inherited=inherited)
            finite.quiet_group()
            outputs.extend(finite.pin(command_dir / name, STREAM_CAP) for name in ('stdout.raw', 'stderr.raw'))
            after = snapshot(work, fixture['entries'])
            require(after == baseline, 'GNU query mutated private namespace/content/metadata')
            row = {'fixture': fixture['name'], 'command': command, 'actual_simulator': step, 'actual_GNU': native,
                   'metadata_before': baseline, 'metadata_after': after, 'metadata_unchanged': True}
            try:
                row['comparison'] = compare_observation(command, step, native)
            except finite.OracleError as error:
                row['failure'] = str(error)
                failures.append({'fixture': fixture['name'], 'source': command['source'], 'reason': str(error)})
            outputs.append(finite.json_new(command_dir / 'OBSERVATION.json', row, budget, PROBE_CAP))
            count += 1
    after = [finite.pin(path, cap) for path, cap in paths]
    require(after == before, 'Fixed source, GNU binary or MAIN probe changed')
    summary = {'queries': count, 'legacy_cases_preserved': 152, 'original_prefix_preserved': 79, 'failures': failures, 'evidence': str(out), 'version': version['stdout'].splitlines()[0]}
    outputs.append(finite.json_new(out / 'REPORT.json', summary, budget, PROBE_CAP))
    if not failures:
        manifest = {'status': 'complete', 'summary': summary, 'pins_before': before, 'pins_after': after,
                    'outputs': outputs, 'deadline_seconds': DEADLINE, 'stdout_stderr_each_bytes': STREAM_CAP,
                    'simulator_effort_ceiling': 1048576, 'binary_identity': 'MAIN rebuilt fixed probe; no worktree/shared target',
                    'metadata_scope': 'namespace/type/inode/owner/group/mode/mtime/content; host atime/ctime excluded'}
        _, warning = finite.publish(out, manifest, budget)
        summary['durability_warning'] = warning
    return summary
