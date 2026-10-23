#!/usr/bin/env python3
"""Controlled Linux candidate lifecycle fixture; use tools/dev.py tool, never learner input."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import itertools
import selectors
import subprocess
import time
import tempfile

from build_safety import owned_directory

ROOT = Path(__file__).resolve().parents[1]


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open('rb') as stream:
        while block := stream.read(1024 * 1024):
            digest.update(block)
    return digest.hexdigest()


def candidate(root, value):
    distribution = owned_directory(root, root / 'dist')
    location = Path(value).resolve(strict=True)
    if not location.is_relative_to(distribution) or not location.is_dir():
        raise ValueError('Candidate must be an existing project dist directory')
    manifest = location / 'BUILD_INFO.json'
    if manifest.is_symlink() or not manifest.is_file() or manifest.stat().st_size > 1_000_000:
        raise ValueError('Expected a bounded ordinary BUILD_INFO.json')
    info = json.loads(manifest.read_text())
    if not re.fullmatch(r'[0-9a-f]{40}', info.get('revision', '')):
        raise ValueError('Expected an exact source revision')
    for name in ['install.sh', 'uninstall.sh', 'dlms-learning-lab', 'INSTALL_FILES.txt']:
        path = location / name
        if path.is_symlink() or not path.is_file():
            raise ValueError('Missing ordinary candidate installation resource: ' + name)
    if sha(location / 'dlms-learning-lab') != info.get('binary_sha256'):
        raise ValueError('Candidate executable does not match its build manifest')
    return location, info


def require(condition, message):
    if not condition:
        raise ValueError(message)


def installation_base(root):
    return owned_directory(root, root / '.build-artifacts/installations')


def clean_uninstalled(prefix, launcher, desktop):
    for path in [prefix, launcher, desktop]:
        require(not path.exists() and not path.is_symlink(),
                'Uninstall left an application resource: ' + str(path))


def bounded_capture(command, logfile, cwd, env, timeout, cap):
    """Linux nonblocking capture; no reader thread can hold close() after timeout.

    Children inherit the guard's owned group. Any descendant left after failure
    is terminated by that outer guard, without detaching a second process group.
    """
    deadline = time.monotonic() + timeout
    process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                               stderr=subprocess.STDOUT, bufsize=0)
    selector = selectors.DefaultSelector()
    total, eof, timed_out, incomplete = 0, False, False, False
    exited_at = None
    try:
        os.set_blocking(process.stdout.fileno(), False)
        selector.register(process.stdout, selectors.EVENT_READ)
        with logfile.open('xb') as output:
            while True:
                now = time.monotonic()
                code = process.poll()
                if code is not None:
                    if eof:
                        break
                    if exited_at is None:
                        exited_at = now
                    if now - exited_at >= 0.25:
                        incomplete = True
                        break
                if now >= deadline:
                    timed_out = True
                    break
                for key, _ in selector.select(min(0.05, deadline - now)):
                    try:
                        block = os.read(key.fileobj.fileno(), 8192)
                    except BlockingIOError:
                        continue
                    if not block:
                        eof = True
                        selector.unregister(key.fileobj)
                        continue
                    retained = max(0, min(len(block), cap - total))
                    if retained:
                        output.write(block[:retained])
                    total += len(block)
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=1)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=1)
        selector.close()
        process.stdout.close()
    return process.returncode, timed_out, total > cap, incomplete


def run_checked(command, logfile, cwd, env):
    status = bounded_capture([str(x) for x in command], logfile, cwd, env, 60, cap=32_768)
    require(not any(status),
            'Installation/identity capture failed (exit/timeout/truncated/open-pipe): '
            + repr(status) + '; ' + str(logfile))
    return logfile.read_text(errors='replace').strip()


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--candidate', type=Path, required=True)
    parser.add_argument('--previous', type=Path, required=True)
    parser.add_argument('--labs', type=int, required=True)
    parser.add_argument('--units', type=int, required=True)
    args = parser.parse_args()
    if platform.system() != 'Linux':
        parser.error('This fixture verifies only the Linux installer; other desktop acceptance is separate')
    if not 1 <= args.labs <= 1000 or not 1 <= args.units <= args.labs:
        parser.error('Expected explicit positive catalog/unit counts')
    if os.environ.get('CARGO_BUILD_JOBS') != '2' or os.environ.get('RUST_TEST_THREADS') != '2':
        parser.error('Run through the main tools/dev.py shared workflow')
    temporary = Path(os.environ.get('TMPDIR', '/')).resolve()
    if not temporary.is_relative_to(ROOT / '.build-artifacts'):
        parser.error('Use the project disk-backed temporary workflow')
    current, info = candidate(ROOT, args.candidate)
    archive = current.with_name(current.name + '.tar.gz')
    if archive.is_symlink() or not archive.is_file():
        parser.error('Expected the ordinary archive beside the candidate')
    previous, old_info = candidate(ROOT, args.previous)
    if old_info['revision'] == info['revision']:
        parser.error('Upgrade evidence requires a different previous source revision')
    # The parent dev.py owns the single shared lock, memory observer and process
    # group. All child commands are fixed project installers or static app flags.
    base = installation_base(ROOT)
    acceptance = Path(tempfile.mkdtemp(prefix='verified-' + info['revision'][:12] + '-', dir=base))
    prefix, bindir, menu = (acceptance / part for part in ['application', 'bin', 'desktop'])
    data = acceptance / 'synthetic-data'
    data.mkdir()
    original_labs = json.loads((ROOT / 'scenarios/starter-v1.json').read_text())['scenarios']
    progress = {'schema_version': 1, 'clock': 9, 'favorites': ['ccna-ipv4'], 'attempts': {
        s['id']: {'scenario_version': s['version'], 'state': s['initial'], 'step': 0,
                  'challenge': False, 'hints': 0, 'transcript': [], 'visited': i + 1}
        for i, s in enumerate(original_labs)}}
    progress_path = data / 'dlmslearninglab/progress-v1.json'
    progress_path.parent.mkdir()
    original = (json.dumps(progress, indent=2) + '\n').encode()
    progress_path.write_bytes(original)
    env = {**os.environ, 'XDG_DATA_HOME': str(data)}

    counter = itertools.count()

    def call(command):
        text = run_checked(command, acceptance / f'command-{next(counter):03}.log', acceptance, env)
        require(progress_path.read_bytes() == original, 'Synthetic progress bytes changed')
        return text

    def install(source):
        call(['sh', source / 'install.sh', '--prefix', prefix, '--bin-dir', bindir,
              '--desktop-dir', menu])

    install(previous)
    require(json.loads((prefix / 'BUILD_INFO.json').read_text())['revision'] == old_info['revision'], 'Previous build identity differs')
    require(sha(prefix / 'dlms-learning-lab') == old_info['binary_sha256'], 'Previous installed binary differs')
    require(old_info['revision'] in call([bindir / 'dlms-learning-lab', '--version']), 'Previous executable identity differs')
    install(current)
    require(json.loads((prefix / 'BUILD_INFO.json').read_text()) == info, 'Installed manifest differs')
    binary = prefix / 'dlms-learning-lab'
    require(sha(binary) == info['binary_sha256'], 'Installed binary hash differs')
    version = call([bindir / 'dlms-learning-lab', '--version'])
    require(info['revision'] in version, 'Installed version identity differs')
    contents = call([bindir / 'dlms-learning-lab', '--validate-content'])
    require(contents == f'Validated {args.labs} authored labs and {args.units} units', 'Installed catalog differs: ' + contents)
    call(['desktop-file-validate', menu / 'org.dlms.LearningLab.desktop'])
    call(['sh', prefix / 'uninstall.sh'])
    clean_uninstalled(prefix, bindir / 'dlms-learning-lab', menu / 'org.dlms.LearningLab.desktop')
    install(current)
    require(call([bindir / 'dlms-learning-lab', '--version']) == version, 'Reinstalled identity differs')
    require(call([bindir / 'dlms-learning-lab', '--validate-content']) == contents, 'Reinstalled content differs')
    report = {'revision': info['revision'], 'candidate': str(current),
              'app': str(bindir / 'dlms-learning-lab'), 'binary_sha256': sha(binary),
              'archive_sha256': sha(archive),
              'previous_revision': old_info['revision'], 'catalog': contents,
              'synthetic_original_nine_bytes_sha256': hashlib.sha256(original).hexdigest(),
              'checks': ['upgrade', 'outside-checkout identity/content/hash', 'desktop validation',
                         'clean uninstall/reinstall', 'exact synthetic progress retention'],
              'limits': ['native GUI and launches checked separately', 'not personal progress',
                         'not novice/screen-reader/physical-device acceptance',
                         'not Windows/macOS runtime acceptance']}
    (acceptance / 'acceptance.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
