#!/usr/bin/env python3
"""Fixed developer-only37-case Bash oracle. Never learner input or app dispatch.

Entry: MAIN tools/dev.py tool case_bash_oracle.py; fixed reviewed fixtures only.
The main runner owns the lock, sampler and process group. No child detaches.
"""
import argparse
from contextlib import contextmanager
from dataclasses import dataclass
import errno
import hashlib
import json
import os
from pathlib import Path
import re
import selectors
import stat
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = 'tests/fixtures/shell-case-oracle-inputs.json'
FIXTURE_SHA = 'e2979b112349c45cd083f0fc7e5dd49d7e3866bf762f60faf24ea9d232ff2cf3'
MANUAL = Path('/usr/share/doc/bash/bashref.html')
MANUAL_SHA = 'b91026acffb159bd1e842e3116a95539bf858e2b57cbfa74df9e3a53379177a4'
BASH = Path('/usr/bin/bash')
BASH_FLAGS = ['--noprofile', '--norc', '+O', 'extglob', '+o', 'posix']
BASELINE7 = '9a924e116ee7768053e361f6a651b956393d36c7df53c513924181de4fa486de'
INPUT_CAP = 1024 * 1024
DATA_CAP = 8 * 1024 * 1024
ROW_CAP = 2 * 1024 * 1024
FILE_CAP = 64 * 1024
FILE_TOTAL_CAP = 1024 * 1024
STREAM_CAP = 64 * 1024
VERSION_CAP = 8192
FILE_COUNT = 16
TIMEOUT = 2.0
DEADLINE = 120.0
HELD_PIPE_GRACE = .25
CORE = {
    'src/shell.rs': 'dc69fbf3d82d0854786d5d396be0a2bfec2968f247f1cdf257a357470929d0e7',
    'src/shell/arithmetic.rs': 'f425685c0e44f91e43e75c4f274ce1cdea58af7d2de0f6306c2cc24ec1da7c2e',
    'src/shell/capture.rs': '137a51db9d6faa9c9e79eb771e1eb149fe6491a65ad08f698ddf40395b7abf3c',
    'src/shell/execution.rs': 'e597382b8829f2370cb3f29c11ba3446fd1bfda6f3bb97bc2b3678654b819365',
    'src/shell/globbing.rs': 'd6cf355c640aa755b14fe46cd695a21bcc88a121c0286205ada14bfb50f6f3fe',
    'src/shell/preparation.rs': '4e418b84bb95cdba1c03055a3e9ca84e7de2480840f6cf43a2205663a3aabed1',
    'src/shell/substitution.rs': 'f89daee97e96bcce4de9148956e4991b5b74e2b368b4ba806b9bc79977f6b87e',
    'src/shell/text_tools.rs': 'deb064cd7ca459ffc0456adab6c91d8a0de6dd0574faaa809b75984830428540',
    'src/shell/values.rs': 'd0c7108fba65a05cc84b8d3f28f9d7b2a66a86f3771126086deabf890673154e',
}


class OracleError(RuntimeError):
    pass


def require(ok, message):
    if not ok:
        raise OracleError(message)


def signature(info):
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


@contextmanager
def ordinary(path, cap):
    """Same-FD type/size/read/hash; no component symlink, no FIFO-open hang."""
    path = Path(path)
    require(path.is_absolute() and '..' not in path.parts, 'Expected contained absolute ordinary path')
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
                    'Ordinary file size/type ceiling: ' + str(path))
            yield stream, before
            require(signature(os.fstat(stream.fileno())) == signature(before),
                    'File changed during same-FD observation: ' + str(path))
    finally:
        os.close(directory)


def read_bytes(path, cap):
    with ordinary(path, cap) as (stream, info):
        data = stream.read(info.st_size)
        require(len(data) == info.st_size and not stream.read(1), 'File grew/shrank during bounded read')
        return data


def pin(path, cap):
    digest = hashlib.sha256()
    with ordinary(path, cap) as (stream, info):
        remaining = info.st_size
        while remaining:
            block = stream.read(min(1024 * 1024, remaining))
            require(bool(block), 'File shrank during finite hash')
            digest.update(block)
            remaining -= len(block)
        require(not stream.read(1), 'File grew during finite hash')
        return {'path': str(path), 'bytes': info.st_size, 'sha256': digest.hexdigest()}


def private_dir(parent, name, existing=False):
    require(re.fullmatch(r'[A-Za-z0-9_-]{1,80}', name) is not None, 'Simple directory basename required')
    path = parent / name
    try:
        path.mkdir(mode=0o700)
    except FileExistsError:
        if not existing:
            raise
    info = path.lstat()
    require(stat.S_ISDIR(info.st_mode) and stat.S_IMODE(info.st_mode) == 0o700,
            'Expected private0700 directory; never chmod existing artifacts')
    require(path.resolve().parent == parent.resolve(), 'Directory escaped its parent')
    return path


@dataclass
class Budget:
    cap: int = DATA_CAP
    used: int = 0

    def charge(self, count):
        require(0 <= count <= self.cap - self.used, 'Total evidence byte ceiling; incomplete evidence retained')
        self.used += count

    def replace_reservation(self, old, new):
        require(0 <= old <= self.used and new >= 0, 'Invalid private-file byte reservation')
        require(self.used - old + new <= self.cap, 'Final private files exceed total evidence ceiling')
        self.used = self.used - old + new


def new_file(path):
    return os.fdopen(os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600), 'wb')


def write_new(path, data, budget, cap):
    require(len(data) <= cap, 'Per-file evidence ceiling before write')
    budget.charge(len(data))
    with new_file(path) as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())
    return pin(path, cap)


def json_new(path, value, budget, cap=ROW_CAP):
    # Encode incrementally; individual fixed strings are bounded before this call.
    count = 0
    with new_file(path) as stream:
        for part in json.JSONEncoder(ensure_ascii=False, separators=(',', ':')).iterencode(value):
            block = part.encode('utf-8')
            require(len(block) <= cap - count, 'JSON row ceiling; incomplete evidence retained')
            budget.charge(len(block))
            stream.write(block)
            count += len(block)
        require(count < cap, 'JSON row newline ceiling')
        budget.charge(1)
        stream.write(b'\n')
        stream.flush()
        os.fsync(stream.fileno())
    return pin(path, cap)


def safe_relative(name):
    require(isinstance(name, str) and name.startswith('/work/'), 'Only fixed /work-relative fixture files')
    relative = Path(name[len('/work/'):])
    require(bool(relative.parts) and not relative.is_absolute() and len(relative.parts) <= 4
            and all(p not in ('', '.', '..') and re.fullmatch(r'[A-Za-z0-9_.-]{1,64}', p)
                    for p in relative.parts), 'Invalid fixture relative path')
    return relative


def closed_environment(case_env, temporary):
    require(isinstance(case_env, dict) and len(case_env) <= 16, 'Fixture env bound')
    forbidden = {'HOME', 'PATH', 'ENV', 'SHELLOPTS', 'IFS', 'CDPATH', 'GLOBIGNORE',
                 'POSIXLY_CORRECT', 'LANG', 'LANGUAGE', 'TMPDIR', 'TMP', 'TEMP',
                 'SHELL', 'PROMPT_COMMAND', 'PS1', 'PS2', 'PS4', '_',
                 'GLIBC_TUNABLES', 'GCONV_PATH'}
    for key, value in case_env.items():
        require(re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]{0,63}', key) is not None
                and key not in forbidden and not key.startswith(('BASH', 'LC_', 'LD_', 'DYLD_')),
                'Startup/import/function/environment override forbidden')
        require(isinstance(value, str) and '\0' not in value and len(value.encode()) <= 4096,
                'Fixture environment value ceiling')
    # HOME is deliberately absent. Never copy os.environ or infer a user home.
    return {'PATH': '', 'LC_ALL': 'C', 'LANG': 'C', 'TMPDIR': str(temporary),
            'TMP': str(temporary), 'TEMP': str(temporary), **case_env}


def load_fixed(root):
    data = read_bytes(root / FIXTURE, INPUT_CAP)
    require(hashlib.sha256(data).hexdigest() == FIXTURE_SHA, 'Fixed original37 bytes changed')
    value = json.loads(data)
    cases = value['cases']
    require(isinstance(cases, list) and len(cases) == 37, 'Exactly37 reviewed input vectors required')
    seen = set()
    for row in cases:
        require(isinstance(row['name'], str) and len(row['name']) <= 128 and row['name'] not in seen,
                'Fixed case identity/uniqueness')
        seen.add(row['name'])
        require(isinstance(row['source'], str) and 0 < len(row['source'].encode()) <= 16384
                and '\0' not in row['source'], 'Fixed source ceiling')
        case = row['case']
        require(len(case['args']) <= 64 and all(isinstance(s, str) and '\0' not in s
                and len(s.encode()) <= 4096 for s in case['args']), 'Fixed argv ceiling')
        require(isinstance(case['stdin'], str) and '\0' not in case['stdin']
                and len(case['stdin'].encode()) <= FILE_CAP, 'Fixed stdin ceiling')
        closed_environment(case['env'], Path('/fixed-private-placeholder'))
        require(len(case['files']) <= FILE_COUNT, 'Initial file count')
        initial_bytes = 0
        for name, text in case['files'].items():
            safe_relative(name)
            require(isinstance(text, str) and '\0' not in text and len(text.encode()) <= FILE_CAP,
                    'Initial ordinary-file content ceiling')
            initial_bytes += len(text.encode())
        require(initial_bytes <= FILE_TOTAL_CAP, 'Initial file aggregate')
        require(row['actual_bash_observation'] is None and row['actual_evaluator7_before'] is None,
                'Original input proposal must remain unmodified; observations are separate')
    return data, cases


def materialize(work, files, budget):
    total = sum(len(text.encode('utf-8')) for text in files.values())
    require(total <= FILE_TOTAL_CAP and len(files) <= FILE_COUNT, 'Initial file aggregate/count')
    for name, text in files.items():
        relative = safe_relative(name)
        parent = work
        for part in relative.parts[:-1]:
            parent = private_dir(parent, part, existing=True)
        write_new(parent / relative.name, text.encode('utf-8'), budget, FILE_CAP)
    return total


def final_files(work):
    result, total, entries = {}, 0, 0

    def visit(directory, depth):
        nonlocal total, entries
        require(depth <= 4, 'Final fixture file depth')
        with os.scandir(directory) as children:
            for child in children:
                entries += 1
                require(entries <= 64, 'Final directory entry count')
                info = child.stat(follow_symlinks=False)
                path = directory / child.name
                require(not stat.S_ISLNK(info.st_mode), 'Final symlink forbidden')
                if stat.S_ISDIR(info.st_mode):
                    visit(path, depth + 1)
                    continue
                require(stat.S_ISREG(info.st_mode), 'Final special file forbidden')
                require(len(result) < FILE_COUNT and 0 <= info.st_size <= FILE_CAP
                        and total + info.st_size <= FILE_TOTAL_CAP,
                        'Final file size/count/aggregate before allocation')
                name = '/work/' + path.relative_to(work).as_posix()
                safe_relative(name)
                data = read_bytes(path, FILE_CAP)
                require(len(data) == info.st_size, 'Final file changed while inspecting')
                result[name] = data.decode('utf-8', errors='strict')
                total += len(data)

    visit(work, 0)
    return dict(sorted(result.items())), total


def terminate_direct_child(process):
    """Bounded direct child cleanup. Outer guard reaps any remaining descendants."""
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=1)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=1)
    else:
        process.wait(timeout=1)


def capture_command(command, folder, work, env, data, budget, stream_cap,
                    timeout=TIMEOUT, inherited=(), clock=time.monotonic,
                    popen=subprocess.Popen, select_factory=selectors.DefaultSelector):
    """Finite nonblocking separate streams, including leader-exit/held-pipe failure.

    No process-group detach or killpg here. A failure immediately aborts the tool;
    the MAIN guard cleans only its owned group, including any orphan pipe-holder.
    """
    require(len(data) <= FILE_CAP, 'Bound stdin before launch')
    deadline = clock() + timeout
    process = popen(command, cwd=work, env=env, stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, bufsize=0,
                    pass_fds=tuple(inherited), start_new_session=False)
    selector = None
    streams, counts, offset = {}, {'stdout': 0, 'stderr': 0}, 0
    exited_at = None
    try:
        selector = select_factory()
        for name in ['stdout', 'stderr']:
            pipe = getattr(process, name)
            os.set_blocking(pipe.fileno(), False)
            selector.register(pipe, selectors.EVENT_READ, name)
            streams[name] = new_file(folder / (name + '.raw'))
        os.set_blocking(process.stdin.fileno(), False)
        if data:
            selector.register(process.stdin, selectors.EVENT_WRITE, 'stdin')
        else:
            process.stdin.close()
        while True:
            now = clock()
            code = process.poll()
            if code is not None:
                if not any(key.data != 'stdin' for key in selector.get_map().values()):
                    break
                if exited_at is None:
                    exited_at = now
                require(now - exited_at < HELD_PIPE_GRACE, 'Exited child left an open output pipe; abort for guard-owned cleanup')
            require(now < deadline, 'Fixed Bash command timeout; abort for guard-owned cleanup')
            for key, _ in selector.select(min(.05, deadline - now)):
                if key.data == 'stdin':
                    try:
                        offset += os.write(key.fileobj.fileno(), data[offset:offset + 8192])
                    except BlockingIOError:
                        # A readiness notification can become stale. Retain the
                        # same offset and return to bounded selector/deadline
                        # polling rather than treating EAGAIN as an input error.
                        continue
                    except BrokenPipeError:
                        offset = len(data)
                    if offset == len(data):
                        selector.unregister(key.fileobj)
                        key.fileobj.close()
                    continue
                try:
                    block = os.read(key.fileobj.fileno(), 8192)
                except BlockingIOError:
                    continue
                if not block:
                    selector.unregister(key.fileobj)
                    continue
                name = key.data
                require(len(block) <= stream_cap - counts[name], 'Output ceiling; partial streams retained, not accepted')
                budget.charge(len(block))
                streams[name].write(block)
                counts[name] += len(block)
        process.wait(timeout=1)
        for output in streams.values():
            output.flush()
            os.fsync(output.fileno())
        return {'stdout': read_bytes(folder / 'stdout.raw', stream_cap).decode('utf-8', errors='strict'),
                'stderr': read_bytes(folder / 'stderr.raw', stream_cap).decode('utf-8', errors='strict'),
                'status': process.returncode}
    finally:
        try:
            terminate_direct_child(process)
        finally:
            if selector is not None:
                selector.close()
            for output in streams.values():
                output.close()
            for name in ['stdin', 'stdout', 'stderr']:
                pipe = getattr(process, name)
                if pipe is not None:
                    pipe.close()


def group_members(group, exclude, proc=Path('/proc')):
    """Only numeric process stat/group/state; no command lines or unrelated signals."""
    found, examined = [], 0
    with os.scandir(proc) as entries:
        for entry in entries:
            if not entry.name.isdecimal() or int(entry.name) == exclude:
                continue
            examined += 1
            require(examined <= 131072, 'Cannot establish bounded owned-group quiescence')
            try:
                # procfs stat reports st_size=0 despite its finite generated
                # bytes. Keep a same-FD bounded read, without using size as EOF.
                with ordinary(proc / entry.name / 'stat', 4096) as (stream, _):
                    data = stream.read(4097)
                require(len(data) <= 4096, 'Process stat byte ceiling')
                fields = data.rpartition(b')')[2].split()
            except OSError as error:
                if error.errno in (errno.ENOENT, errno.ESRCH):
                    continue  # ordinary exit during the scan
                raise  # permission/I/O uncertainty is never a quiet group
            require(len(fields) >= 3 and fields[0] in (b'R', b'S', b'D', b'Z', b'T', b't', b'X', b'x', b'K', b'W', b'P', b'I'),
                    'Uncertain process stat while checking owned group')
            if int(fields[2]) == group and fields[0] not in (b'Z', b'X', b'x'):
                found.append(int(entry.name))
    return found


def quiet_group():
    require(not group_members(os.getpgrp(), os.getpid()), 'Owned descendants remain; abort before another fixture or completion')


def require_guard(root):
    require(sys.platform.startswith('linux') and os.getpgrp() == os.getpid(), 'Fixed Linux tool requires MAIN owned process group')
    require(Path.cwd().resolve() == root and os.environ.get('CARGO_BUILD_JOBS') == '2'
            and os.environ.get('RUST_TEST_THREADS') == '2', 'Use MAIN tools/dev.py with2jobs/2testthreads')
    from build_safety import disk_directory
    disk_directory(root / '.build-artifacts')
    temp = Path(os.environ.get('TMPDIR', '/')).resolve(strict=True)
    require(temp.is_relative_to(root / '.build-artifacts')
            and all(os.environ.get(k) == str(temp) for k in ['TMPDIR', 'TMP', 'TEMP']),
            'Guard-owned disk temporary environment required')
    marker = root / '.build-artifacts/active-command.json'
    until = time.monotonic() + .5
    while True:
        try:
            value = json.loads(read_bytes(marker, 16384))
        except (json.JSONDecodeError, OracleError):
            require(time.monotonic() < until, 'Uncertain guard marker during bounded spawn transition')
            time.sleep(.01)
            continue
        if value.get('state') == 'running':
            break
        require(value.get('state') == 'starting' and time.monotonic() < until,
                'No actual MAIN guard active-command state')
        time.sleep(.01)  # bounded state polling for spawn->marker publication
    require(value.get('child_pid') == os.getpid() and value.get('wrapper_pid') == os.getppid(),
            'Oracle is not the actual MAIN guard child')
    lock = (root / '.build-artifacts/heavy-work.lock').stat(follow_symlinks=False)
    require(stat.S_ISREG(lock.st_mode), 'Actual shared lock must be an ordinary file')
    descriptors = []
    with os.scandir('/proc/self/fd') as entries:
        for entry in entries:
            require(entry.name.isdecimal() and int(entry.name) < 65536, 'Unexpected inherited FD shape')
            try:
                info = os.fstat(int(entry.name))
            except OSError as error:
                if error.errno == errno.EBADF:
                    continue
                raise
            if (info.st_dev, info.st_ino) == (lock.st_dev, lock.st_ino):
                descriptors.append(int(entry.name))
    require(len(descriptors) == 1, 'Expected exactly one actual inherited shared-lock FD')
    return tuple(descriptors)


def source_identity(root):
    head = read_bytes(root / '.git/HEAD', 4096).decode().strip()
    if head.startswith('ref: '):
        ref = head[5:]
        require(re.fullmatch(r'refs/heads/[A-Za-z0-9_./-]+', ref) and '..' not in Path(ref).parts,
                'Unexpected main source ref')
        head = read_bytes(root / '.git' / ref, 4096).decode().strip()
    require(re.fullmatch(r'[0-9a-f]{40}', head) is not None, 'Expected actual source HEAD')
    return head


def sync_dir(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def publish(folder, manifest, budget):
    """All precommit failures lack COMPLETE. Publication itself is the commit."""
    manifest_pin = json_new(folder / 'MANIFEST.json', manifest, budget)
    prepared = write_new(folder / 'COMPLETION-PREPARED',
                         (manifest_pin['sha256'] + '\n').encode(), budget, 65)
    require(pin(folder / 'MANIFEST.json', ROW_CAP) == manifest_pin, 'Manifest changed before publication')
    require(pin(folder / 'COMPLETION-PREPARED', 65) == prepared, 'Prepared marker changed')
    sync_dir(folder)
    sync_dir(folder.parent)
    os.link(folder / 'COMPLETION-PREPARED', folder / 'COMPLETE')  # atomic no clobber
    # COMMITTED. No postpublication exception/output failure revokes this marker.
    warning = None
    try:
        sync_dir(folder)
    except OSError as error:
        warning = f'COMMITTED; postpublication directory durability warning: {error}'
    return manifest_pin, warning


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--capture-id', required=True)
    args = parser.parse_args(argv)
    require(re.fullmatch(r'[A-Za-z0-9_-]{1,64}', args.capture_id) is not None, 'Simple fixed capture ID required')
    root = ROOT.resolve(strict=True)
    inherited = require_guard(root)
    start = time.monotonic()
    head = source_identity(root)
    source, cases = load_fixed(root)
    bash = BASH.resolve(strict=True)
    manual = MANUAL.resolve(strict=True)
    sources = [(root / 'tools/case_bash_oracle.py', INPUT_CAP),
               (root / 'tools/dev.py', INPUT_CAP), (root / 'tools/build_safety.py', INPUT_CAP),
               (root / FIXTURE, INPUT_CAP), (manual, 8 * INPUT_CAP), (bash, 256 * INPUT_CAP)]
    sources += [(root / path, INPUT_CAP) for path in CORE]
    before = [pin(path, cap) for path, cap in sources]
    require(before[4]['sha256'] == MANUAL_SHA, 'Original installed manual changed; review new identity')
    for path, sha in CORE.items():
        require(next(p for p in before if p['path'] == str(root / path))['sha256'] == sha,
                'Original public7 evaluator changed before decision oracle')
    from build_safety import disk_directory
    base = root / '.build-artifacts/reviews/shell-case'
    require(base.is_dir() and not base.is_symlink() and base.resolve().is_relative_to(root),
            'Expected contained existing project-disk review directory')
    disk_directory(base)
    parent = private_dir(base, 'bash-oracle-captures', existing=True)
    out = private_dir(parent, args.capture_id)
    budget = Budget()
    outputs = []
    outputs.append(json_new(out / 'STARTED.json', {'status': 'started-not-complete', 'head': head,
        'before7_baseline_manifest': BASELINE7, 'pins_before': before}, budget))
    outputs.append(write_new(out / 'INPUTS.json', source, budget, INPUT_CAP))
    outputs.append(write_new(out / 'COLLECTOR.py', read_bytes(root / 'tools/case_bash_oracle.py', INPUT_CAP), budget, INPUT_CAP))
    version_dir = private_dir(out, 'version')
    work = private_dir(version_dir, 'work')
    temporary = private_dir(version_dir, 'tmp')
    version = capture_command([str(bash), '--noprofile', '--norc', '--version'], version_dir,
                              work, closed_environment({}, temporary), b'', budget, VERSION_CAP,
                              inherited=inherited)
    quiet_group()
    require(version['status'] == 0 and not version['stderr'], 'Actual Bash version request failed')
    lines = version['stdout'].splitlines()
    require(bool(lines), 'Actual Bash version banner is empty')
    first_line = lines[0]
    require(first_line.startswith('GNU bash, version '), 'Unexpected actual Bash banner; review identity')
    outputs += [pin(version_dir / name, VERSION_CAP) for name in ['stdout.raw', 'stderr.raw']]
    outputs.append(json_new(out / 'VERSION.json', {'actual': version, 'binary': before[5],
        'locale': 'C', 'HOME_environment': 'omitted; collector performs no home lookup',
        'invocation_flags': BASH_FLAGS, 'options': 'GNU non-POSIX mode; extglob explicitly off; no startup/import environment'}, budget))
    require(re.match(r'^GNU bash, version 5\.3\.9\b', first_line) is not None,
            'Actual version differs from reviewed5.3.9 identity; VERSION retained, review before cases')
    for index, row in enumerate(cases):
        require(time.monotonic() - start < DEADLINE, 'Whole fixed oracle deadline')
        folder = private_dir(out, f'case-{index:03d}')
        work = private_dir(folder, 'work')
        temporary = private_dir(folder, 'tmp')
        initial = materialize(work, row['case']['files'], budget)
        env = closed_environment(row['case']['env'], temporary)
        began = time.monotonic()
        observed = capture_command([str(bash), *BASH_FLAGS, '-c', row['source'],
                                    'fixed-case-' + str(index), *row['case']['args']], folder, work,
                                   env, row['case']['stdin'].encode(), budget, STREAM_CAP,
                                   timeout=min(TIMEOUT, max(.001, DEADLINE - (time.monotonic() - start))),
                                   inherited=inherited)
        quiet_group()
        require(observed['status'] >= 0, 'Unexpected signal termination; retain incomplete oracle')
        after, after_bytes = final_files(work)
        budget.replace_reservation(initial, after_bytes)
        observed['files'] = after
        record = {'index': index, 'name': row['name'], 'category': row['category'],
            'elapsed_seconds': time.monotonic() - began,
            'source': row['source'], 'case': row['case'],
            'source_sha256': hashlib.sha256(row['source'].encode()).hexdigest(),
            'input_sha256': hashlib.sha256(json.dumps(row['case'], sort_keys=True, separators=(',', ':')).encode()).hexdigest(),
            'actual_bash_observation': observed, 'fixture_expectations_non_authoritative': True,
            'actual_version': first_line, 'locale': 'C', 'virtual_machine_executed': False}
        outputs += [pin(folder / name, STREAM_CAP) for name in ['stdout.raw', 'stderr.raw']]
        outputs.append(json_new(folder / 'OBSERVATION.json', record, budget))
        for name in after:
            outputs.append(pin(work / safe_relative(name), FILE_CAP))
    # No more children can be launched past this point.
    quiet_group()
    require(time.monotonic() - start < DEADLINE, 'Whole oracle deadline before completion')
    require(source_identity(root) == head and [pin(path, cap) for path, cap in sources] == before,
            'Source/input/manual/Bash changed during observation')
    for output in outputs:
        require(pin(Path(output['path']), DATA_CAP) == output, 'Retained output changed before completion')
    require(time.monotonic() - start < DEADLINE, 'Whole oracle deadline after final pin/output verification')
    portable_outputs = [{'file': Path(p['path']).relative_to(out).as_posix(),
                         'bytes': p['bytes'], 'sha256': p['sha256']} for p in outputs]
    manifest = {'schema_version': 1, 'status': 'complete-only-with-valid-COMPLETE', 'source_head': head,
        'original_input_sha256': FIXTURE_SHA, 'before7_baseline_manifest': BASELINE7,
        'actual_bash_version': first_line, 'case_rows': len(cases), 'pins': before, 'outputs': portable_outputs,
        'collector_bytes_before_manifest': budget.used, 'data_cap': DATA_CAP,
        'per_stream_cap': STREAM_CAP, 'per_version_stream_cap': VERSION_CAP,
        'per_file_cap': FILE_CAP, 'files_per_case': FILE_COUNT, 'file_total_per_case': FILE_TOTAL_CAP,
        'per_process_seconds': TIMEOUT, 'whole_seconds': DEADLINE, 'HOME': 'omitted',
        'claims': 'Actual fixed Bash observations only; no VM8, learner execution, mastery, curriculum or conformance claim.'}
    digest, warning = publish(out, manifest, budget)
    # COMMITTED: writes to terminal are optional diagnostics, never validation gates.
    try:
        print(json.dumps({'status': 'COMMITTED', 'directory': str(out), 'manifest_sha256': digest['sha256'],
                          'durability_warning': warning, 'actual_bash_version': first_line, 'cases': 37}))
    except OSError:
        pass
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (OracleError, OSError, ValueError, subprocess.SubprocessError) as error:
        # No automatic retry/cleanup/deletion. The outer guard cleans owned group.
        try:
            print(f'Fixed oracle aborted; partial evidence retained, no completion claimed: {error}', file=sys.stderr)
        except OSError:
            pass
        raise SystemExit(1)
