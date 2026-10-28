#!/usr/bin/env python3
"""Fixed two-process native reopen check; MAIN tools/dev.py tool only.
Never learner input, an arbitrary command runner, or a personal-profile operation.
Evidence is retained on failure. No global environment or desktop setting changes.
"""
import argparse
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import signal
import stat
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
PURPOSE = 'dlms-native-reopen-v1'
MARKER = 'REOPEN-SESSION.json'
LOG_CAP = 2 * 1024 * 1024
PROFILE_CAP = 16_000_000
JSON_CAP = 16 * 1024 * 1024
BINARY_CAP = 1024 * 1024 * 1024
EVIDENCE_CAP = 64 * 1024 * 1024
FILE_COUNT_CAP = 128
PHASE_TIMEOUT = 120
REQUIRED = {'write': 'PASS native reopen writer:', 'read': 'PASS native reopen reader:'}


class ReopenError(RuntimeError):
    pass


def require(ok, text):
    if not ok:
        raise ReopenError(text)


def signature(info):
    return info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns


@contextmanager
def ordinary(path, cap):
    """Component-wise no-follow open, same-FD finite type/read/hash observations."""
    path = Path(path)
    require(path.is_absolute() and '..' not in path.parts, 'Expected ordinary absolute path')
    directory = os.open('/', os.O_RDONLY | os.O_DIRECTORY)
    try:
        for part in path.parts[1:-1]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=directory)
            os.close(directory)
            directory = child
        fd = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=directory)
        with os.fdopen(fd, 'rb') as stream:
            before = os.fstat(stream.fileno())
            require(stat.S_ISREG(before.st_mode) and 0 <= before.st_size <= cap,
                    'Ordinary file type/size ceiling')
            yield stream, before
            require(signature(os.fstat(stream.fileno())) == signature(before),
                    'File changed during observation')
    finally:
        os.close(directory)


def read_bytes(path, cap):
    with ordinary(path, cap) as (stream, info):
        data = stream.read(info.st_size)
        require(len(data) == info.st_size and not stream.read(1), 'File grew or shrank')
        return data


def pin(path, cap):
    h = hashlib.sha256()
    with ordinary(path, cap) as (stream, info):
        left = info.st_size
        while left:
            data = stream.read(min(left, 1024 * 1024))
            require(bool(data), 'File shrank during pin')
            h.update(data)
            left -= len(data)
        require(not stream.read(1), 'File grew during pin')
        return {'bytes': info.st_size, 'sha256': h.hexdigest()}


def checked_directory(path):
    path = Path(path)
    require(path.is_absolute() and '..' not in path.parts, 'Expected canonical directory')
    here = Path('/')
    for part in path.parts[1:]:
        here /= part
        info = here.lstat()
        require(stat.S_ISDIR(info.st_mode), 'Directory component is a symlink or not a directory')
    require(path.resolve(strict=True) == path, 'Directory is not canonical')
    return path


def fresh_directory(path):
    path = Path(path)
    checked_directory(path.parent)
    require(not path.exists() and not path.is_symlink(), 'Fresh directory already occupied')
    path.mkdir(mode=0o700)
    require(path.stat().st_uid == os.getuid() and stat.S_IMODE(path.stat().st_mode) == 0o700,
            'Fresh directory ownership/permissions')
    return path


def exclusive(path, data):
    require(len(data) <= JSON_CAP, 'Evidence file ceiling')
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'wb') as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())


def json_bytes(value):
    return (json.dumps(value, indent=2) + '\n').encode()


def app_path(path, root=ROOT):
    path = Path(path).absolute()
    require(path.name == 'dlms-learning-lab' and path.is_relative_to(root),
            'Only this project Learning Lab executable is accepted')
    allowed = [root / 'target', root / '.build-artifacts' / 'targets', root / 'dist',
               root / '.build-artifacts' / 'installations']
    require(any(path.is_relative_to(base) for base in allowed), 'Executable outside project build roots')
    with ordinary(path, BINARY_CAP) as (stream, info):
        require(info.st_mode & 0o111 and not info.st_mode & 0o6000,
                'Expected unprivileged executable')
        require(stream.read(4) == b'\x7fELF', 'This fixed acceptance tool currently supports Linux ELF only')
    return path


def inherited_lock(root=ROOT):
    """Carry the MAIN runner's already inherited lock through the GUI children."""
    lock = (root / '.build-artifacts/heavy-work.lock').stat()
    for entry in Path('/proc/self/fd').iterdir():
        try:
            fd = int(entry.name)
            info = os.fstat(fd)
        except (ValueError, OSError):
            continue
        if fd > 2 and (info.st_dev, info.st_ino) == (lock.st_dev, lock.st_ino):
            return fd
    raise ReopenError('Use MAIN tools/dev.py; inherited heavy-work lock is missing')


def child_environment(session, phase, source=None):
    source = os.environ if source is None else source
    require(phase in REQUIRED, 'Fixed write/read phase only')
    # No HOME, loader/import overrides, shell startup paths, arbitrary DLMS flags
    # or renderer selection is introduced. Existing display bindings are needed.
    keys = ('WAYLAND_DISPLAY', 'DISPLAY', 'XDG_RUNTIME_DIR', 'XAUTHORITY',
            'DBUS_SESSION_BUS_ADDRESS', 'LANG', 'LC_ALL', 'LC_CTYPE', 'TMPDIR', 'TMP', 'TEMP')
    env = {key: source[key] for key in keys if key in source}
    env.update(XDG_DATA_HOME=str(session / 'data'), XDG_CONFIG_HOME=str(session / 'config'),
               XDG_CACHE_HOME=str(session / 'cache'), DLMS_NATIVE_REOPEN_ROOT=str(session),
               DLMS_NATIVE_REOPEN_PHASE=phase, RUST_BACKTRACE='1')
    return env


def stop_own_child(process):
    for sig, grace in [(signal.SIGINT, 1), (signal.SIGTERM, 2), (signal.SIGKILL, 2)]:
        if process.poll() is not None:
            return
        process.send_signal(sig)
        try:
            process.wait(timeout=grace)
        except subprocess.TimeoutExpired:
            pass
    require(process.poll() is not None, 'Owned GUI child did not exit; outer runner must block/clean it')


def capture(app, role, logfile, cwd, env, lock_fd, timeout):
    """Fixed identity or GUI argv, nonblocking output, bounded timeout/held pipe.
    No setsid/group escape: MAIN guard owns the entire group, including on failure.
    """
    require(role in ('version', 'write', 'read'), 'Fixed launch role only')
    command = [str(app), '--version' if role == 'version' else '--developer']
    cap = 4096 if role == 'version' else LOG_CAP
    process = None
    started = time.monotonic()
    with os.fdopen(os.open(logfile, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), 'wb') as log, selectors.DefaultSelector() as selector:
        try:
            process = subprocess.Popen(command, cwd=cwd, env=env, stdin=subprocess.DEVNULL,
                                       stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                       pass_fds=(lock_fd,))
            os.set_blocking(process.stdout.fileno(), False)
            selector.register(process.stdout, selectors.EVENT_READ)
            total = 0
            ended = None
            while selector.get_map():
                require(time.monotonic() - started < timeout, 'Native phase timed out')
                if process.poll() is not None:
                    ended = ended or time.monotonic()
                    require(time.monotonic() - ended < .5, 'Owned child exited but stdout stayed open')
                for key, _ in selector.select(.05):
                    try:
                        chunk = os.read(key.fd, 8192)
                    except BlockingIOError:
                        continue
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    keep = min(len(chunk), max(0, cap - total))
                    log.write(chunk[:keep])
                    total += len(chunk)
                    require(total <= cap, 'Native log ceiling exceeded')
            left = max(.01, timeout - (time.monotonic() - started))
            code = process.wait(timeout=left)
            log.flush()
            os.fsync(log.fileno())
            require(code == 0, f'Native {role} exited {code}; evidence retained')
            return {'role': role, 'pid': process.pid, 'exit': code, 'bytes': total,
                    'seconds': round(time.monotonic() - started, 3)}
        finally:
            if process is not None:
                stop_own_child(process)
                process.stdout.close()


def inventory(session):
    checked_directory(session)
    files = {}
    total = 0
    visited = 0
    pending = [session]
    while pending:
        directory = pending.pop()
        with os.scandir(directory) as entries:
            for entry in entries:
                visited += 1
                require(visited <= FILE_COUNT_CAP, 'Evidence entry count ceiling')
                path = Path(entry.path)
                require(len(path.relative_to(session).parts) <= 12, 'Evidence depth ceiling')
                info = entry.stat(follow_symlinks=False)
                require(not stat.S_ISLNK(info.st_mode), 'Evidence symlink refused')
                if stat.S_ISDIR(info.st_mode):
                    pending.append(path)
                    continue
                require(stat.S_ISREG(info.st_mode), 'Nonordinary evidence refused')
                total += info.st_size
                require(total <= EVIDENCE_CAP, 'Evidence aggregate ceiling')
                files[str(path.relative_to(session))] = pin(path, JSON_CAP)
    return dict(sorted(files.items()))


def verify_writer_saved_bytes(expected, raw, preferences, snapshot, preferences_snapshot):
    """Compare full raw data, not Progress's intentionally nonportable projection.
    Byte identity catches late writes/format changes; no fields are ignored.
    """
    require(raw == snapshot, 'Writer progress bytes changed after the pre-exit snapshot')
    require(preferences == preferences_snapshot, 'Writer appearance bytes changed after the pre-exit snapshot')
    raw_json = json.loads(raw)
    require(raw_json == expected['raw_progress'], 'Full raw writer JSON differs from observation')
    require(raw_json.get('migration_receipts') == expected['migration_receipts'],
            'Loaded provenance differs from full raw main-file provenance')
    require(json.loads(preferences) == expected['preferences'], 'Saved writer appearance differs')


def publish_manifest(session, report):
    exclusive(session / 'MANIFEST.json', json_bytes(report))
    # Prepare/fsync first; publishing the hard link is the no-clobber commit boundary.
    # COMPLETE records validation, not a power-loss durability guarantee.
    ready = session / '.COMPLETE-ready'
    exclusive(ready, (pin(session / 'MANIFEST.json', JSON_CAP)['sha256'] + '\n').encode())
    directory = os.open(session, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(directory)
    finally:
        os.close(directory)
    os.link(ready, session / 'COMPLETE', follow_symlinks=False)


def session_marker(session, temporary, nonce, case='script'):
    require(case in ('script', 'dhcpv4', 'nfs'), 'Fixed script/dhcpv4/nfs case only')
    require(re.fullmatch(r'[0-9a-f]{32}', nonce) is not None, 'Fixed marker nonce')
    marker = {'version': 1, 'purpose': PURPOSE, 'root': str(session),
              'temporary_root': str(temporary), 'nonce': nonce}
    # Preserve the exact legacy marker shape when the new case is not requested.
    if case in ('dhcpv4', 'nfs'):
        marker['case'] = case
    return marker


def verify_repaired_reader(session, writer):
    expected = json.loads(read_bytes(session / 'reader-repaired.json', JSON_CAP))
    require(set(expected) == set(writer), 'Repaired reader observation shape differs')
    # MainProgress flattens the identical Progress serialization and adds ONLY
    # this documented nonportable field; no learning field may be ignored.
    require(isinstance(expected['raw_progress'], dict), 'Repaired raw progress must be an object')
    portable = dict(expected['raw_progress'])
    portable.pop('migration_receipts', None)
    require(expected['progress'] == portable, 'Repaired portable progress differs from raw projection')
    for key in ('progress_path', 'preferences_path'):
        require(expected[key] == writer[key], 'Repaired reader changed private paths')
    path = Path(writer['progress_path'])
    preferences = Path(writer['preferences_path'])
    require(path.is_relative_to(session / 'data') and path.name == 'progress-v1.json',
            'Repaired reader profile escaped private XDG')
    require(preferences == path.with_name('preferences-v1.json'),
            'Repaired reader preferences path differs')
    verify_writer_saved_bytes(expected, read_bytes(path, PROFILE_CAP),
                              read_bytes(preferences, 4096),
                              read_bytes(session / 'reader-progress.raw', PROFILE_CAP),
                              read_bytes(session / 'reader-preferences.raw', 4096))
    return expected


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--app', required=True, type=Path)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--case', choices=('script', 'dhcpv4', 'nfs'), default='script')
    args = parser.parse_args()
    require(os.name == 'posix' and Path('/proc/self/fd').is_dir(), 'Fixed Linux native acceptance only')
    temporary = checked_directory(Path(os.environ.get('TMPDIR', '')))
    require(temporary == ROOT / '.build-artifacts/tmp', 'Use MAIN runner project artifact TMPDIR')
    # Runner verified the volume and holds this descriptor; propagate it to children.
    lock_fd = inherited_lock()
    app = app_path(args.app)
    before = pin(app, BINARY_CAP)
    if args.output:
        output = args.output.absolute()
        require(output.is_relative_to(temporary) and output.parent == temporary,
                'Output must be a fresh direct child of runner TMPDIR')
    else:
        output = temporary / ('native-reopen-' + os.urandom(12).hex())
    require(re.fullmatch(r'native-reopen-[A-Za-z0-9-]{1,82}', output.name) is not None, 'Fixed bounded fresh session name required')
    session = fresh_directory(output)
    for name in ('data', 'config', 'cache', 'write', 'read'):
        fresh_directory(session / name)
    marker = session_marker(session, temporary, os.urandom(16).hex(), args.case)
    exclusive(session / MARKER, json_bytes(marker))
    exclusive(session / 'STARTED.json', json_bytes({'binary': before, 'marker': marker}))
    version = capture(app, 'version', session / 'version.log', session,
                      child_environment(session, 'write'), lock_fd, 5)
    identity = read_bytes(session / 'version.log', 4096).decode()
    require(any(line.startswith('DLMS Learning Lab ') for line in identity.splitlines()),
            'Expected Learning Lab build identity')
    phases = []
    for role in ('write', 'read'):
        require(pin(app, BINARY_CAP) == before, 'Executable changed before GUI launch')
        inventory(session)
        phases.append(capture(app, role, session / f'{role}.log', session / role,
                              child_environment(session, role), lock_fd, PHASE_TIMEOUT))
        require(REQUIRED[role] in read_bytes(session / f'{role}.log', LOG_CAP).decode(),
                'Native phase did not reach its fixed completion marker')
        if role == 'write':
            expected = json.loads(read_bytes(session / 'writer-observed.json', JSON_CAP))
            progress_path = Path(expected['progress_path'])
            preferences_path = Path(expected['preferences_path'])
            require(progress_path.is_relative_to(session / 'data') and progress_path.name == 'progress-v1.json', 'Observed profile path escaped private XDG')
            require(preferences_path == progress_path.with_name('preferences-v1.json'), 'Unexpected preferences path')
            raw = read_bytes(progress_path, PROFILE_CAP)
            prefs = read_bytes(preferences_path, 4096)
            snapshot = read_bytes(session / 'writer-progress.raw', PROFILE_CAP)
            preferences_snapshot = read_bytes(session / 'writer-preferences.raw', 4096)
            verify_writer_saved_bytes(expected, raw, prefs, snapshot, preferences_snapshot)
        else:
            loaded = json.loads(read_bytes(session / 'reader-loaded.json', JSON_CAP))
            require(loaded == expected, 'Second process did not load the exact writer observation')
            if args.case in ('dhcpv4', 'nfs'):
                verify_repaired_reader(session, expected)
    require(len(phases) == 2 and phases[0]['pid'] != phases[1]['pid'], 'Expected two distinct GUI processes')
    require(pin(app, BINARY_CAP) == before, 'Executable changed during native acceptance')
    files = inventory(session)
    report = {'purpose': PURPOSE, 'binary': before, 'version': version, 'phases': phases,
              'files': files, 'scope': 'Two real GUI processes share a tool-created private synthetic profile; actual UI/VM completion, unfinished assignment/example drafts, favorite and appearance. Not novice/accessibility/other-platform acceptance.'}
    if args.case == 'dhcpv4':
        report['case'] = 'dhcpv4'
        report['scope'] = ('Two real normal GUI processes share one fresh private XDG profile. '
                           'Actual DHCP assignment/example exchanges, distinct unfinished policy drafts, '
                           'current leases, favorite and appearance survive cold load, then actual field repairs '
                           'and checks. No injected State, host DHCP, novice or other-platform acceptance.')
    elif args.case == 'nfs':
        report['case'] = 'nfs'
        report['scope'] = ('Two real normal GUI processes share one fresh private XDG profile. '
                           'Actual NFS editor/Save/Apply/Mount/Read operations, separate practice, '
                           'distinct unfinished exports drafts, current mounts/files, favorite and appearance '
                           'survive cold load. Reader repairs through native editors and rechecks both goals. '
                           'No injected State, host NFS, novice or other-platform acceptance.')
    publish_manifest(session, report)
    try:
        print(f'PASS native cold-process reopen: {session}', flush=True)
    except OSError:
        pass  # Publication already committed; broken diagnostic output cannot undo it.
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
