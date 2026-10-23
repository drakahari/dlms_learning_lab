#!/usr/bin/env python3
"""Serial, two-job project builds/tests with disk-backed temporary files and memory evidence."""
import argparse
import os
from pathlib import Path
import shutil
import sys

from build_safety import JOBS, TEST_THREADS, project_root, disk_directory, owned_directory, workflow


def validate_cargo_args(arguments):
    forbidden = ('-j', '--jobs', '--test-threads', '--target-dir', '--manifest-path', '--config', '--out-dir', '--artifact-dir', '--lockfile-path', '-C', '-Z')
    if any(value.startswith(forbidden) for value in arguments):
        raise RuntimeError('Cargo job/test/path/config overrides are not accepted; use the reviewed runner options')


def tool_outputs(arguments, cwd, root):
    result = list(arguments)
    seen = False
    for index, value in enumerate(result):
        if value != '--output' and not value.startswith('--output='):
            if value.startswith('--') and '--output'.startswith(value.split('=', 1)[0]):
                raise RuntimeError('Abbreviated output options are not supported')
            continue
        if seen:
            raise RuntimeError('Duplicate output options are not supported')
        seen = True
        if value == '--output' and index + 1 >= len(result):
            raise RuntimeError('--output requires a path')
        raw = value.split('=', 1)[1] if '=' in value else result[index + 1]
        output = Path(raw)
        output = (cwd / output if not output.is_absolute() else output).resolve()
        if not output.is_relative_to(root):
            raise RuntimeError('New tool output must stay on the project disk volume')
        disk_directory(output.parent)
        if '=' in value:
            result[index] = '--output=' + str(output)
        else:
            result[index + 1] = str(output)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cwd', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--target-dir', type=Path)
    parser.add_argument('action', choices=['build', 'check', 'test', 'clippy', 'candidate', 'python-tests', 'native', 'tool'])
    parser.add_argument('args', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    root = project_root(args.cwd)
    if root != project_root(Path(__file__).resolve().parents[1]):
        raise RuntimeError('The selected worktree must belong to this project')
    cache = owned_directory(root, root / '.build-artifacts/toolchains')
    cargo_home, rustup_home = cache / 'cargo', cache / 'rustup'
    if (cache / 'READY.json').is_file() and cargo_home.is_dir() and rustup_home.is_dir():
        os.environ.update(CARGO_HOME=str(cargo_home), RUSTUP_HOME=str(rustup_home))
        os.environ['PATH'] = str(cargo_home / 'bin') + os.pathsep + os.environ.get('PATH', '')
    elif args.action not in ['python-tests', 'native', 'tool']:
        raise RuntimeError('Prepare the disk-backed toolchain cache described in docs/BUILD_WORKFLOW.md first')
    with workflow(args.cwd, args.action) as session:
        target = (args.target_dir or (root / 'target' if args.cwd.resolve() == root else root / '.build-artifacts/targets' / args.cwd.name)).resolve()
        if not target.is_relative_to(root):
            raise RuntimeError('Build target must stay in this project on its verified disk volume')
        disk_directory(target)
        os.environ['CARGO_TARGET_DIR'] = str(target)
        print(f'jobs={JOBS}; test threads={TEST_THREADS}; target={target}; temporary={session.temporary}; memory={session.log}', flush=True)
        if args.action == 'candidate':
            import build_candidate
            if args.args:
                raise RuntimeError('candidate uses its reviewed default output; extra arguments are not supported')
            build_candidate.build(root / 'dist')
        elif args.action == 'python-tests':
            session.run([sys.executable, '-m', 'unittest', 'discover', '-s', 'tools/tests', '-v'], cwd=args.cwd)
        elif args.action == 'tool':
            allowed = {'bash_reference.py', 'filesystem_reference.py', 'ipv4_route_reference.py', 'ipv6_reference.py', 'process_reference.py', 'storage_reference.py', 'native_startup.py', 'verify_coverage.py', 'verify_palettes.py'}
            if args.cwd.resolve() != root or args.target_dir:
                raise RuntimeError('Fixed reference tools currently require the main checkout/default target to preserve probe provenance')
            if not args.args or args.args[0] not in allowed:
                raise RuntimeError('Choose an existing fixed development tool, not a learner script')
            tool_args = tool_outputs(args.args, args.cwd.resolve(), root)
            session.run([sys.executable, str(root / 'tools' / tool_args[0]), *tool_args[1:]], cwd=args.cwd)
        elif args.action == 'native':
            if not args.args:
                raise RuntimeError('native requires the exact development or installed executable path')
            executable = Path(args.args[0])
            executable = (args.cwd / executable if not executable.is_absolute() else executable).resolve()
            import tempfile
            native_root = owned_directory(root, session.artifacts / 'native-runs')
            run_dir = Path(tempfile.mkdtemp(prefix='run-', dir=native_root))
            session.run([str(executable), *args.args[1:]], cwd=run_dir)
        else:
            validate_cargo_args(args.args)
            cargo = shutil.which('cargo')
            command = [cargo, args.action, '--jobs', str(JOBS), *args.args]
            if args.action == 'test':
                if '--' not in command:
                    command.append('--')
                command.append(f'--test-threads={TEST_THREADS}')
            session.run(command, cwd=args.cwd)
    return 0


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except (RuntimeError, OSError) as error:
        print(f'Safe workflow: {error}', file=sys.stderr)
        raise SystemExit(1)
