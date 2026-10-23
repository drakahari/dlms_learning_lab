#!/usr/bin/env python3
"""Repeat app-only startup/readback with private synthetic profiles and capped logs.
Development tool, never a learner runner. No backend or desktop-setting overrides.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import time

MAX_LOG = 2_000_000


def stop_child(process):
    if process.poll() is None:
        process.terminate()
        try:
            process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=5)


def launch(command, logfile, cwd, env, timeout, cap=MAX_LOG, on_timeout=None):
    """Drain a single child's output, retain at most cap bytes; always reap on interruption."""
    total = [0]
    with logfile.open('xb') as log:
        process = subprocess.Popen(command, cwd=cwd, env=env, stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT)

        def drain():
            while chunk := process.stdout.read(8192):
                keep = max(0, min(len(chunk), cap - total[0]))
                if keep:
                    log.write(chunk[:keep])
                total[0] += len(chunk)

        reader = threading.Thread(target=drain, daemon=True)
        reader.start()
        timed_out = False
        try:
            try:
                process.wait(timeout=timeout)
            except subprocess.TimeoutExpired:
                timed_out = True
                if on_timeout:
                    on_timeout(process)
        finally:
            stop_child(process)
            reader.join(timeout=5)
            process.stdout.close()
        return process.returncode, timed_out, total[0] > cap


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--count', type=int, default=20)
    parser.add_argument('--timeout', type=int, default=12)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--no-protocol', action='store_false', dest='protocol', default=True)
    parser.add_argument('--no-lifecycle', action='store_false', dest='lifecycle', default=True)
    args = parser.parse_args()
    if not 1 <= args.count <= 100 or not 1 <= args.timeout <= 60:
        parser.error('count must be1..100 and timeout1..60 seconds')
    app = args.app.resolve(strict=True)
    if args.output:
        output = args.output.absolute()
        # A fresh private directory, exclusive file creation and a retained report FD
        # prevent replacing unrelated files or following existing output symlinks.
        if output.exists() or output.is_symlink():
            parser.error('Output must be a fresh directory that does not exist')
        output.mkdir(mode=0o700, parents=True, exist_ok=False)
    else:
        output = Path(tempfile.mkdtemp(prefix='dlms-native-startup-'))
    env = dict(os.environ, DLMS_STARTUP_TRACE='1', DLMS_SMOKE_STAGE='startup', WAYLAND_DEBUG='client')
    if not args.protocol:
        env.pop('WAYLAND_DEBUG', None)
    if not args.lifecycle:
        env.pop('DLMS_STARTUP_TRACE', None)
    version_log = output / 'version.log'
    code, timeout, truncated = launch([str(app), '--version'], version_log, output, env, 5, cap=4096)
    if code or timeout or truncated:
        parser.error('Could not read a bounded application build identity')
    identity = next((line for line in version_log.read_text().splitlines()
                     if line.startswith('DLMS Learning Lab ')), None)
    if identity is None:
        parser.error('Expected the Learning Lab executable')
    rows = []
    with (output / 'results.json').open('x', encoding='utf-8') as report_file:
        for attempt in range(args.count):
            logfile = output / f'launch-{attempt:03}.log'
            start = time.monotonic()

            def sample_stack(process):
                # Only this child; respect debugger permissions and disable incidental
                # configuration, Python auto-load and debuginfod network access.
                try:
                    launch(['gdb', '--batch', '-nx', '-nh',
                            '-iex', 'set auto-load off', '-iex', 'set debuginfod enabled off',
                            '-p', str(process.pid), '-ex', 'set pagination off',
                            '-ex', 'thread apply all bt', '-ex', 'detach'],
                           output / f'launch-{attempt:03}-stack.log', output, env, 8)
                except FileNotFoundError:
                    pass

            code, timed_out, truncated = launch([str(app), '--ui-smoke'], logfile, output,
                                               env, args.timeout, on_timeout=sample_stack)
            with logfile.open('rb') as log:
                text = log.read(MAX_LOG).decode(errors='replace')
            row = {'attempt': attempt + 1, 'seconds': round(time.monotonic() - start, 3),
                   'timeout': timed_out, 'exit': code, 'log_truncated': truncated,
                   'wayland_protocol_observed': 'xdg_wm_base' in text if args.protocol else None,
                   'application_ready': 'application-ready' in text if args.lifecycle else None,
                   'update_completed': 'update-end' in text if args.lifecycle else None,
                   'capture': 'Captured startup-library:' in text,
                   'passed': code == 0 and not timed_out and not truncated and 'PASS native startup:' in text}
            rows.append(row)
            report = {'build': identity, 'protocol_logging': args.protocol, 'lifecycle_logging': args.lifecycle,
                      'launches': rows, 'passed': sum(r['passed'] for r in rows),
                      'scope': 'native synthetic-profile startup/readback; no curriculum, novice or assistive-technology acceptance'}
            report_file.seek(0)
            report_file.truncate()
            json.dump(report, report_file, indent=2)
            report_file.write('\n')
            report_file.flush()
            print(json.dumps(row), flush=True)
            if not row['passed']:
                break
    print(f"Evidence: {output}; {sum(r['passed'] for r in rows)}/{len(rows)} passed")
    return 0 if len(rows) == args.count and all(r['passed'] for r in rows) else 1


if __name__ == '__main__':
    raise SystemExit(main())
