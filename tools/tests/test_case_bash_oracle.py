"""Collector unit tests. Never starts Bash or a host tool.

Collector pipe tests use real small private pipes plus fake process state. They
exercise backpressure/EOF/cleanup deterministically without stress or signals.
"""
import hashlib
import errno
import os
from pathlib import Path
import selectors
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import case_bash_oracle as oracle


class FakeProcess:
    def __init__(self, stdout=b'', stderr=b'', running=False, held=False):
        self.returncode = None if running else 0
        self.terminated = self.killed = 0
        self.waits = []
        self.writers = []
        for name, value in [('stdout', stdout), ('stderr', stderr)]:
            read, write = os.pipe()
            if value:
                # Test payloads fit the ordinary small pipe buffer; no producer thread.
                assert len(value) <= 4096
                assert os.write(write, value) == len(value)
            setattr(self, name, os.fdopen(read, 'rb', buffering=0))
            if held:
                self.writers.append(write)
            else:
                os.close(write)
        self.stdin = open(os.devnull, 'wb', buffering=0)

    def poll(self):
        return self.returncode

    def wait(self, timeout):
        self.waits.append(timeout)
        return self.returncode

    def terminate(self):
        self.terminated += 1
        self.returncode = -15

    def kill(self):
        self.killed += 1
        self.returncode = -9

    def close_writers(self):
        for fd in self.writers:
            os.close(fd)
        self.writers.clear()


class Clock:
    def __init__(self, step=.1):
        self.now = -step
        self.step = step

    def __call__(self):
        self.now += self.step
        return self.now


class EmptySelector:
    def __init__(self):
        self.keys = {}

    def register(self, file, events, data):
        self.keys[file] = selectors.SelectorKey(file, file.fileno(), events, data)

    def get_map(self):
        return self.keys

    def unregister(self, file):
        return self.keys.pop(file)

    def select(self, timeout):
        return []

    def close(self):
        self.keys.clear()


class ReadySelector(EmptySelector):
    def select(self, timeout):
        return [(key, key.events) for key in list(self.keys.values())]


class InputProcess(FakeProcess):
    """No child: empty output pipes, live fake state until stdin is closed."""
    def __init__(self):
        super().__init__(running=True)

    def poll(self):
        if self.returncode is None and self.stdin.closed:
            self.returncode = 0
        return self.returncode


@unittest.skipUnless(sys.platform.startswith('linux'), 'fixed Linux developer oracle')
class CaseOracle(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='case-oracle-source-tests-')
        self.root = Path(self.temp.name).resolve()

    def tearDown(self):
        # Only these unit tests' own newly created fixtures, never oracle evidence.
        self.temp.cleanup()

    def process_fixture(self, process, **kwargs):
        folder = oracle.private_dir(self.root, 'capture')
        work = oracle.private_dir(folder, 'work')
        seen = []

        def fake_popen(command, **options):
            seen.append((command, options))
            return process

        result = oracle.capture_command(['never-executed-fake'], folder, work,
            {'PATH': ''}, kwargs.pop('data', b''), oracle.Budget(), kwargs.pop('stream_cap', 4096),
            popen=fake_popen, **kwargs)
        return result, seen

    def test_exact_original_37_fixture_copy_is_not_an_observation(self):
        raw, rows = oracle.load_fixed(ROOT)
        self.assertEqual(hashlib.sha256(raw).hexdigest(), oracle.FIXTURE_SHA)
        self.assertEqual(len(rows), 37)
        self.assertTrue(all(r['actual_bash_observation'] is None for r in rows))
        self.assertTrue(all(r['actual_evaluator7_before'] is None for r in rows))

    def test_fixed_input_size_and_hash_rejected_before_json_is_trusted(self):
        directory = self.root / 'tests/fixtures'
        directory.mkdir(parents=True)
        path = self.root / oracle.FIXTURE
        path.write_bytes(b'x' * (oracle.INPUT_CAP + 1))
        with patch.object(oracle.json, 'loads', side_effect=AssertionError('must not parse')):
            with self.assertRaises(oracle.OracleError):
                oracle.load_fixed(self.root)
        path.write_text('{}')
        with patch.object(oracle.json, 'loads', side_effect=AssertionError('must not parse')):
            with self.assertRaises(oracle.OracleError):
                oracle.load_fixed(self.root)

    def test_closed_environment_omits_home_and_forbids_startup_and_import_overrides(self):
        with patch.dict(os.environ, {'HOME': '/personal-do-not-copy', 'BASH_ENV': '/never-read'}):
            env = oracle.closed_environment({'LABEL': 'fixed value'}, self.root)
        self.assertNotIn('HOME', env)
        self.assertNotIn('BASH_ENV', env)
        self.assertEqual(env['PATH'], '')
        self.assertEqual(env['LC_ALL'], 'C')
        self.assertEqual(env['LABEL'], 'fixed value')
        for key in ['HOME', 'PATH', 'BASH_ENV', 'ENV', 'SHELLOPTS', 'BASHOPTS',
                    'BASH_FUNC_x%%', 'BASH_COMPAT', 'LC_ALL', 'POSIXLY_CORRECT', 'TMPDIR',
                    'LD_PRELOAD', 'LD_AUDIT', 'DYLD_LIBRARY_PATH', 'GLIBC_TUNABLES']:
            with self.assertRaises(oracle.OracleError, msg=key):
                oracle.closed_environment({key: 'bad'}, self.root)

    def test_no_source_path_or_abbreviated_cli_is_accepted(self):
        for args in [['--source', 'not-a-script'], ['--capture', 'ambiguous'],
                     ['--capture-id', '../escape']]:
            with patch.object(oracle, 'require_guard', side_effect=AssertionError('must not reach guard')):
                with self.assertRaises((SystemExit, oracle.OracleError)):
                    oracle.main(args)

    def test_same_fd_bounds_and_symlink_parent_leaf_rejection(self):
        original = self.root / 'original'
        original.write_bytes(b'unchanged')
        (self.root / 'alias').symlink_to(original)
        with self.assertRaises(OSError):
            oracle.read_bytes(self.root / 'alias', 100)
        (self.root / 'directory').mkdir()
        (self.root / 'directory-link').symlink_to(self.root / 'directory')
        with self.assertRaises(OSError):
            oracle.read_bytes(self.root / 'directory-link/leaf', 100)
        with self.assertRaises(oracle.OracleError):
            oracle.read_bytes(original, 3)
        with self.assertRaises(oracle.OracleError):
            with oracle.ordinary(original, 100) as (stream, _):
                original.write_bytes(b'now longer')
                stream.read(1)

    def test_fixture_paths_and_retained_file_maps_are_complete_and_private(self):
        work = oracle.private_dir(self.root, 'work')
        budget = oracle.Budget()
        self.assertEqual(oracle.materialize(work, {'/work/keep.txt': 'keep\n'}, budget), 5)
        (work / 'trace.txt').write_text('ab')
        files, total = oracle.final_files(work)
        self.assertEqual(files, {'/work/keep.txt': 'keep\n', '/work/trace.txt': 'ab'})
        self.assertEqual(total, 7)
        self.assertEqual(work.stat().st_mode & 0o777, 0o700)
        for name in ['/etc/hosts', '/work/../outside', '/work//absolute', '/work/.']:
            with self.assertRaises(oracle.OracleError, msg=name):
                oracle.safe_relative(name)
        with self.assertRaises(FileExistsError):
            oracle.private_dir(self.root, 'work')
        self.assertEqual((work / 'keep.txt').read_text(), 'keep\n')

    def test_final_oversized_unknown_symlink_and_count_boundaries_fail(self):
        work = oracle.private_dir(self.root, 'work')
        big = work / 'big.txt'
        big.write_bytes(b'x' * (oracle.FILE_CAP + 1))
        with self.assertRaises(oracle.OracleError):
            oracle.final_files(work)
        big.unlink()
        (work / 'link').symlink_to(self.root / 'outside')
        with self.assertRaises(oracle.OracleError):
            oracle.final_files(work)
        (work / 'link').unlink()
        for i in range(oracle.FILE_COUNT + 1):
            (work / str(i)).write_text('x')
        with self.assertRaises(oracle.OracleError):
            oracle.final_files(work)

    def test_capped_json_aggregate_and_no_clobber_preserve_exact_existing_bytes(self):
        path = self.root / 'record.json'
        with self.assertRaises(oracle.OracleError):
            oracle.json_new(path, {'message': 'x' * 64}, oracle.Budget(32), cap=100)
        self.assertFalse((self.root / 'COMPLETE').exists())
        path.write_bytes(b'existing exact bytes')
        with self.assertRaises(FileExistsError):
            oracle.write_new(path, b'new', oracle.Budget(), 10)
        self.assertEqual(path.read_bytes(), b'existing exact bytes')

    def test_nonblocking_separate_streams_and_guard_group_and_fd_inheritance(self):
        process = FakeProcess(b'left\n', b'right\n')
        result, seen = self.process_fixture(process, inherited=(42,))
        self.assertEqual(result, {'stdout': 'left\n', 'stderr': 'right\n', 'status': 0})
        self.assertFalse(seen[0][1]['start_new_session'])
        self.assertEqual(seen[0][1]['pass_fds'], (42,))
        self.assertNotIn('shell', seen[0][1])
        self.assertEqual(process.terminated, 0)
        self.assertTrue(process.stdout.closed and process.stderr.closed)

    def test_partial_stdin_writes_advance_only_the_written_prefix(self):
        process = InputProcess()
        with patch.object(oracle.os, 'write', side_effect=[2, 1, 3]) as writes:
            result, _ = self.process_fixture(process, data=b'abcdef', select_factory=ReadySelector)
        self.assertEqual([call.args[1] for call in writes.call_args_list],
                         [b'abcdef', b'cdef', b'def'])
        self.assertEqual(result, {'stdout': '', 'stderr': '', 'status': 0})
        self.assertTrue(process.stdin.closed)
        self.assertEqual(process.terminated, 0)

    def test_stdin_eagain_retries_the_same_offset_before_and_after_partial_write(self):
        process = InputProcess()
        again = BlockingIOError(errno.EAGAIN, 'deterministic readiness race')
        with patch.object(oracle.os, 'write', side_effect=[again, 2, again, 4]) as writes:
            result, _ = self.process_fixture(process, data=b'abcdef', select_factory=ReadySelector)
        self.assertEqual([call.args[1] for call in writes.call_args_list],
                         [b'abcdef', b'abcdef', b'cdef', b'cdef'])
        self.assertEqual(result['status'], 0)
        self.assertTrue(process.stdin.closed)
        self.assertEqual(process.terminated, 0)

    def test_stdin_early_epipe_closes_input_without_retrying_the_unconsumed_tail(self):
        process = InputProcess()
        with patch.object(oracle.os, 'write',
                          side_effect=[2, BrokenPipeError(errno.EPIPE, 'fixed early close')]) as writes:
            result, _ = self.process_fixture(process, data=b'abcdef', select_factory=ReadySelector)
        self.assertEqual([call.args[1] for call in writes.call_args_list], [b'abcdef', b'cdef'])
        self.assertEqual(result['status'], 0)
        self.assertTrue(process.stdin.closed)
        self.assertEqual(process.terminated, 0)

    def test_repeated_stdin_eagain_remains_deadline_bounded_and_cleans_its_fake_child(self):
        process = InputProcess()
        with patch.object(oracle.os, 'write',
                          side_effect=BlockingIOError(errno.EAGAIN, 'always blocked')):
            with self.assertRaisesRegex(oracle.OracleError, 'timeout'):
                self.process_fixture(process, data=b'abcdef', select_factory=ReadySelector,
                                     clock=Clock(step=.4), timeout=1)
        self.assertEqual(process.terminated, 1)
        self.assertTrue(process.stdin.closed and process.stdout.closed and process.stderr.closed)

    def test_version_or_fixture_output_ceiling_is_failure_not_successful_truncation(self):
        process = FakeProcess(b'1234')
        with self.assertRaisesRegex(oracle.OracleError, 'Output ceiling'):
            self.process_fixture(process, stream_cap=3)
        self.assertFalse((self.root / 'capture/COMPLETE').exists())
        self.assertTrue(process.stdout.closed)

    def test_exited_leader_held_pipe_aborts_without_blocking_or_starting_another_child(self):
        process = FakeProcess(held=True)
        try:
            with self.assertRaisesRegex(oracle.OracleError, 'open output pipe'):
                self.process_fixture(process, clock=Clock(), select_factory=EmptySelector)
            self.assertTrue(process.stdout.closed and process.stderr.closed)
            self.assertEqual(process.terminated, 0)  # leader already exited; guard owns descendants
            self.assertFalse((self.root / 'capture/COMPLETE').exists())
        finally:
            process.close_writers()

    def test_timeout_cleans_only_its_direct_fake_child_with_finite_wait(self):
        process = FakeProcess(running=True, held=True)
        try:
            with self.assertRaisesRegex(oracle.OracleError, 'timeout'):
                self.process_fixture(process, clock=Clock(step=1), select_factory=EmptySelector, timeout=2)
            self.assertEqual(process.terminated, 1)
            self.assertEqual(process.waits, [1])
        finally:
            process.close_writers()

    def test_cleanup_escalates_only_its_child_and_keyboardinterrupt_still_closes_pipes(self):
        process = FakeProcess(running=True)
        with patch.object(process, 'wait', side_effect=[subprocess.TimeoutExpired('fake', 1), -9]):
            oracle.terminate_direct_child(process)
        self.assertEqual((process.terminated, process.killed), (1, 1))
        for stream in [process.stdin, process.stdout, process.stderr]:
            stream.close()
        process = FakeProcess(b'payload', running=True)
        with patch.object(oracle.os, 'read', side_effect=KeyboardInterrupt):
            with self.assertRaises(KeyboardInterrupt):
                self.process_fixture(process)
        self.assertEqual(process.terminated, 1)
        self.assertTrue(process.stdout.closed and process.stderr.closed)

    def test_proc_group_parser_handles_parentheses_and_uncertainty_without_signals(self):
        proc = oracle.private_dir(self.root, 'proc')
        for pid, group, state in [(11, 5, 'S'), (12, 5, 'Z'), (13, 9, 'S')]:
            child = oracle.private_dir(proc, str(pid))
            (child / 'stat').write_bytes(f'{pid} (name ) with spaces) {state} 1 {group} 0 0\n'.encode())
        self.assertEqual(oracle.group_members(5, exclude=99, proc=proc), [11])
        self.assertEqual(oracle.group_members(5, exclude=11, proc=proc), [])
        (proc / '11/stat').write_bytes(b'malformed')
        with self.assertRaises(oracle.OracleError):
            oracle.group_members(5, exclude=99, proc=proc)

    def test_completion_publish_precommit_failure_and_existing_marker_never_overwrite(self):
        capture = oracle.private_dir(self.root, 'capture')
        with patch.object(oracle, 'sync_dir', side_effect=OSError('precommit sync failure')):
            with self.assertRaises(OSError):
                oracle.publish(capture, {'actual': 'fixture'}, oracle.Budget())
        self.assertFalse((capture / 'COMPLETE').exists())
        self.assertTrue((capture / 'COMPLETION-PREPARED').exists())
        second = oracle.private_dir(self.root, 'second')
        (second / 'COMPLETE').write_bytes(b'prior marker retained')
        with self.assertRaises(FileExistsError):
            oracle.publish(second, {'actual': 'new'}, oracle.Budget())
        self.assertEqual((second / 'COMPLETE').read_bytes(), b'prior marker retained')

    def test_postcommit_durability_warning_keeps_verified_complete_and_all_work_folders(self):
        capture = oracle.private_dir(self.root, 'capture')
        for i in range(37):
            oracle.private_dir(capture, f'case-{i:03d}')
        with patch.object(oracle, 'sync_dir', side_effect=[None, None, OSError('after publication')]):
            manifest, warning = oracle.publish(capture, {'actual_rows': 37}, oracle.Budget())
        self.assertIn('COMMITTED', warning)
        self.assertEqual((capture / 'COMPLETE').read_bytes(), (manifest['sha256'] + '\n').encode())
        self.assertEqual(len(list(capture.glob('case-*'))), 37)
        self.assertEqual((capture / 'COMPLETE').stat().st_ino, (capture / 'COMPLETION-PREPARED').stat().st_ino)


if __name__ == '__main__':
    unittest.main()
