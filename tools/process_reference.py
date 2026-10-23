#!/usr/bin/env python3
"""Development-only Linux observations of this script's own disposable children.

No learner input, arbitrary PID, imported script, user path or command is accepted.
Never linked to the application. Only the children created below are signaled.
Raw ps identities and dynamic PIDs are deliberately not printed or stored.
"""
import json
import os
import pathlib
import shutil
import signal
import subprocess
import sys
import tempfile
import time

if len(sys.argv) != 1 or sys.platform != 'linux':
    raise SystemExit('This fixed development fixture takes no arguments and requires Linux.')
SLEEP = shutil.which('sleep')
PS = shutil.which('ps')
RENICE = shutil.which('renice')
if not all((SLEEP, PS, RENICE)):
    raise SystemExit('Reference tools unavailable: NOT RUN')
checks = []
unavailable = []
children = []
ENV = {'PATH': '/usr/bin:/bin', 'LC_ALL': 'C'}


def require(value, label):
    if not value:
        raise RuntimeError('Controlled process observation failed: ' + label)
    checks.append(label)


with tempfile.TemporaryDirectory(prefix='dlms-process-reference-') as temporary:
    work = pathlib.Path(temporary)

    def child():
        process = subprocess.Popen([SLEEP, '15'], cwd=work, env=ENV,
                                   stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,
                                   stderr=subprocess.DEVNULL)
        children.append(process)
        if process.pid <= 1:
            raise RuntimeError('Invalid owned child identity')
        return process

    def snapshot(process):
        if process not in children:
            raise RuntimeError('Only owned children may be inspected')
        result = subprocess.run([PS, '-p', str(process.pid), '-o', 'stat=,ni=,comm='],
                                cwd=work, env=ENV, capture_output=True, text=True, timeout=1)
        fields = result.stdout.split()
        if result.returncode or len(fields) != 3:
            raise RuntimeError('Owned child snapshot unavailable')
        return fields[0], int(fields[1]), fields[2]

    def wait_state(process, state):
        deadline = time.monotonic() + 1
        while time.monotonic() < deadline:
            if snapshot(process)[0].startswith(state):
                return
            time.sleep(0.01)
        raise RuntimeError('Owned child state did not settle')

    try:
        process = child()
        wait_state(process, 'S')
        state, old, comm = snapshot(process)
        require(comm == 'sleep', 'ps executable name from owned child')
        os.kill(process.pid, signal.SIGSTOP)
        wait_state(process, 'T')
        require(process.poll() is None, 'SIGSTOP retains a live process')
        os.kill(process.pid, signal.SIGCONT)
        wait_state(process, 'S')
        require(process.poll() is None, 'SIGCONT resumes sleeping child')
        if old < 19:
            target = old + 1
            result = subprocess.run([RENICE, '--priority', str(target), '-p', str(process.pid)],
                                    cwd=work, env=ENV, capture_output=True, text=True, timeout=1)
            require(result.returncode == 0 and snapshot(process)[1] == target,
                    'absolute renice changes owned-child nice value')
        else:
            unavailable.append('niceness increase: child already at ceiling19')
        os.kill(process.pid, signal.SIGSTOP)
        wait_state(process, 'T')
        os.kill(process.pid, signal.SIGTERM)
        time.sleep(0.05)
        require(process.poll() is None and snapshot(process)[0].startswith('T'),
                'SIGTERM remains pending while stopped')
        os.kill(process.pid, signal.SIGCONT)
        require(process.wait(timeout=1) == -signal.SIGTERM,
                'SIGCONT permits pending default termination')
        process = child()
        wait_state(process, 'S')
        os.kill(process.pid, signal.SIGSTOP)
        wait_state(process, 'T')
        os.kill(process.pid, signal.SIGKILL)
        require(process.wait(timeout=1) == -signal.SIGKILL,
                'SIGKILL terminates a stopped owned child immediately')
    finally:
        for process in children:
            if process.poll() is None:
                process.kill()
            process.wait(timeout=2)

print(json.dumps({'checks': checks, 'passed': len(checks), 'unavailable': unavailable,
                  'scope': 'fixed owned-child development observations only; not app runtime or complete process conformance'}, indent=2))
