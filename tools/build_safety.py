"""Project-only build coordination. No machine configuration or unrelated processes."""
from contextlib import contextmanager
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import threading
import time

JOBS = 2
TEST_THREADS = 2
MIN_AVAILABLE = 6 * 1024 * 1024 * 1024
_ACTIVE = None


def project_root(cwd):
    common = subprocess.check_output(
        ['git', 'rev-parse', '--path-format=absolute', '--git-common-dir'],
        cwd=cwd, text=True).strip()
    return Path(common).resolve().parent


def disk_directory(path):
    path = Path(path).resolve()
    path.mkdir(parents=True, exist_ok=True, mode=0o700)
    if os.name == 'posix' and Path('/proc/self/mountinfo').exists():
        matches = []
        for line in Path('/proc/self/mountinfo').read_text().splitlines():
            left, right = line.split(' - ', 1)
            mount = Path(left.split()[4].replace('\\040', ' '))
            if path.is_relative_to(mount):
                matches.append((len(mount.parts), right.split()[0], right.split()[1]))
        if not matches:
            raise RuntimeError('Cannot establish artifact filesystem')
        _, kind, source = max(matches)
        if kind in {'tmpfs', 'ramfs', 'devtmpfs', 'overlay'}:
            raise RuntimeError(f'Artifact directory must be disk-backed, found {kind}')
        return {'path': str(path), 'filesystem': kind, 'source': source}
    # Native Windows/macOS acceptance must verify the chosen local volume there.
    return {'path': str(path), 'filesystem': 'not verified on this platform'}


def owned_directory(root, path):
    path = Path(path).resolve()
    if not path.is_relative_to(Path(root).resolve()):
        raise RuntimeError('Artifact path escaped the common project directory')
    disk_directory(path)
    return path


def group_alive(child):
    if os.name != 'posix':
        return child.poll() is None
    try:
        os.killpg(child.pid, 0)
    except ProcessLookupError:
        return False
    # Dead orphan zombies are not executing work and cannot be killed again.
    if Path('/proc').is_dir():
        uncertain = False
        for entry in Path('/proc').iterdir():
            if not entry.name.isdigit():
                continue
            try:
                fields = (entry / 'stat').read_bytes().rsplit(b')', 1)[1].split()
                if int(fields[2]) == child.pid and fields[0] != b'Z':
                    return True
            except (FileNotFoundError, ProcessLookupError):
                pass
            except (OSError, UnicodeError, ValueError, IndexError):
                uncertain = True
        return uncertain
    return True


def cleanup_child(child):
    for sig, grace in [(signal.SIGINT, 1), (signal.SIGTERM, 2), (getattr(signal, 'SIGKILL', signal.SIGTERM), 2)]:
        if not group_alive(child):
            break
        try:
            if os.name == 'posix':
                os.killpg(child.pid, sig)
            elif sig == signal.SIGINT:
                child.terminate()
            else:
                child.kill()
        except ProcessLookupError:
            pass
        deadline = time.monotonic() + grace
        while time.monotonic() < deadline and group_alive(child):
            time.sleep(.05)
    try:
        child.wait(timeout=1)
    except subprocess.TimeoutExpired:
        return False
    return not group_alive(child)


def snapshot():
    if not Path('/proc/meminfo').exists():
        return {'time': time.time(), 'available': None, 'platform_observation': 'unavailable'}
    mem = {}
    for line in Path('/proc/meminfo').read_text().splitlines():
        key, value = line.split(':', 1)
        if key in {'MemTotal', 'MemAvailable', 'SwapTotal', 'SwapFree'}:
            mem[key] = int(value.split()[0]) * 1024
    psi = {}
    pressure = Path('/proc/pressure/memory')
    if pressure.exists():
        for line in pressure.read_text().splitlines():
            parts = line.split()
            psi[parts[0]] = {k: float(v) for k, v in (p.split('=') for p in parts[1:])}
    counters = {}
    for line in Path('/proc/vmstat').read_text().splitlines():
        key, value = line.split()
        if key in {'pswpin', 'pswpout', 'oom_kill'}:
            counters[key] = int(value)
    return {'time': time.time(), 'available': mem['MemAvailable'],
            'memory': mem, 'pressure': psi, 'counters': counters}


def pressured(sample):
    return (sample.get('available') is not None and sample['available'] < MIN_AVAILABLE
            or sample.get('pressure', {}).get('full', {}).get('avg10', 0) >= 5
            or sample.get('pressure', {}).get('some', {}).get('avg10', 0) >= 15)


class Session:
    def __init__(self, cwd, label):
        self.root = project_root(cwd)
        self.artifacts = owned_directory(self.root, self.root / '.build-artifacts')
        self.volume = disk_directory(self.artifacts)
        self.temporary = owned_directory(self.root, self.artifacts / 'tmp')
        self.logs = owned_directory(self.root, self.artifacts / 'logs')
        stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S.%fZ')
        self.log = self.logs / f'{stamp}-{label}-memory.jsonl'
        self.stop = threading.Event()
        self.pressure = threading.Event()
        self.monitor_error = None
        self.lock = None
        self.launching = False
        self.termination_signal = None

    def checkpoint(self):
        if self.monitor_error is not None:
            raise RuntimeError(f'Memory monitoring failed; no additional work launched. Evidence: {self.log}; {self.monitor_error}')
        if self.pressure.is_set():
            raise RuntimeError(f'Sustained memory pressure; no additional work launched. Evidence: {self.log}')

    def observe(self):
        streak = 0
        try:
            with self.log.open('x') as output:
                while True:
                    sample = snapshot()
                    output.write(json.dumps(sample) + '\n')
                    output.flush()
                    streak = streak + 1 if pressured(sample) else 0
                    if streak >= 3:
                        self.pressure.set()
                    if self.stop.wait(1):
                        break
        except Exception as error:
            self.monitor_error = str(error)
            self.pressure.set()

    def run(self, args, **kwargs):
        self.checkpoint()
        with self.log.with_name(self.log.name.replace('-memory.jsonl', '-commands.jsonl')).open('a') as record:
            record.write(json.dumps({'argv': [str(a) for a in args], 'cargo_jobs': os.environ.get('CARGO_BUILD_JOBS'), 'test_threads': os.environ.get('RUST_TEST_THREADS'), 'temporary': os.environ.get('TMPDIR'), 'target': os.environ.get('CARGO_TARGET_DIR')}) + '\n')
        active = self.artifacts / 'active-command.json'
        active.write_text(json.dumps({'wrapper_pid': os.getpid(), 'child_pid': None, 'state': 'starting'}) + '\n')
        child = None
        try:
            inherited = {'pass_fds': (self.lock.fileno(),)} if os.name == 'posix' else {}
            self.launching = True
            try:
                child = subprocess.Popen(args, start_new_session=os.name == 'posix', **inherited, **kwargs)
            finally:
                self.launching = False
            if self.termination_signal is not None:
                # A termination during Popen must wait until the returned child
                # is assigned, so the finally block can reap that exact group.
                signal.raise_signal(self.termination_signal)
            active.write_text(json.dumps({'wrapper_pid': os.getpid(), 'child_pid': child.pid, 'state': 'running'}) + '\n')
            while child.poll() is None:
                if self.pressure.wait(.25):
                    self.checkpoint()
            if group_alive(child):
                raise RuntimeError('Development command left active descendants; cleaning only its process group')
            if child.returncode:
                raise subprocess.CalledProcessError(child.returncode, args)
            self.checkpoint()
        finally:
            cleaned = child is None
            try:
                if child is not None:
                    cleaned = cleanup_child(child)
            except BaseException:
                cleaned = False
                raise
            finally:
                if cleaned:
                    active.unlink(missing_ok=True)
                else:
                    (self.artifacts / 'unfinished-child.json').write_text(json.dumps({'pid': child.pid, 'message': 'Owned development process group did not exit; inspect before starting any more work'}) + '\n')
                    # active-command.json remains even if writing the extra report fails.
            if not cleaned:
                raise RuntimeError('Owned command could not be reaped; further work blocked by command marker')


@contextmanager
def termination_cleanup():
    """Unwind the runner's own cleanup on a catchable wrapper termination.

    SIGKILL remains uncatchable: the inherited lock and marker still fail closed.
    Handlers are local to this CLI workflow and restored before returning.
    """
    if os.name != 'posix' or threading.current_thread() is not threading.main_thread():
        yield
        return
    previous = {sig: signal.getsignal(sig) for sig in (signal.SIGTERM, signal.SIGHUP)}

    def interrupted(signum, _frame):
        if _ACTIVE is not None and _ACTIVE.launching:
            if _ACTIVE.termination_signal is None:
                _ACTIVE.termination_signal = signum
            return
        # Repeated termination must not interrupt bounded child/group teardown.
        for sig in previous:
            signal.signal(sig, signal.SIG_IGN)
        raise SystemExit(128 + signum)

    try:
        for sig in previous:
            signal.signal(sig, interrupted)
        yield
    finally:
        for sig, handler in previous.items():
            signal.signal(sig, handler)


@contextmanager
def workflow(cwd, label='development'):
    global _ACTIVE
    if _ACTIVE is not None:
        if project_root(cwd) != _ACTIVE.root:
            raise RuntimeError('Nested workflow belongs to another project')
        _ACTIVE.checkpoint()
        yield _ACTIVE
        return
    session = Session(cwd, label)
    lock = (session.artifacts / 'heavy-work.lock').open('a+b')
    session.lock = lock
    try:
        if os.name == 'posix':
            import fcntl
            try:
                fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as error:
                raise RuntimeError('Another project build/test owns the heavy-work lock; retry after it finishes') from error
        else:
            import msvcrt
            if lock.tell() == 0:
                lock.write(b'\0'); lock.flush()
            lock.seek(0)
            try:
                msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
            except OSError as error:
                raise RuntimeError('Another project build/test owns the heavy-work lock') from error
        if any((session.artifacts / name).exists() for name in ['unfinished-child.json', 'active-command.json']):
            raise RuntimeError('An earlier owned command may still be running; inspect the command marker before retrying')
        first = snapshot()
        if pressured(first):
            session.log.write_text(json.dumps(first) + '\n')
            raise RuntimeError(f'Memory headroom/pressure gate refused new work; evidence: {session.log}')
        overrides = {'CARGO_BUILD_JOBS': str(JOBS), 'RUST_TEST_THREADS': str(TEST_THREADS),
                     'TMPDIR': str(session.temporary), 'TMP': str(session.temporary), 'TEMP': str(session.temporary)}
        saved = {key: os.environ.get(key) for key in overrides}
        prior_temp = tempfile.tempdir
        os.environ.update(overrides)
        tempfile.tempdir = None
        thread = threading.Thread(target=session.observe, daemon=True)
        thread.start()
        _ACTIVE = session
        with termination_cleanup():
            try:
                yield session
                session.checkpoint()
            finally:
                _ACTIVE = None
                session.stop.set()
                thread.join()
                tempfile.tempdir = prior_temp
                for key, value in saved.items():
                    if value is None:
                        os.environ.pop(key, None)
                    else:
                        os.environ[key] = value
    finally:
        lock.close()


def run(args, **kwargs):
    if _ACTIVE is None:
        raise RuntimeError('Heavy command requires the shared project workflow')
    return _ACTIVE.run(args, **kwargs)


def checkpoint():
    if _ACTIVE is not None:
        _ACTIVE.checkpoint()


def capture(args, **kwargs):
    if _ACTIVE is None:
        raise RuntimeError('Captured development command requires the shared workflow')
    with tempfile.TemporaryFile(dir=_ACTIVE.temporary) as output:
        run(args, stdout=output, **kwargs)
        if output.tell() > 32 * 1024 * 1024:
            raise RuntimeError('Development metadata exceeds the supported 32 MiB capture')
        output.seek(0)
        return output.read().decode()
