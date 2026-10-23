import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import tomllib
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import build_safety as safety


class BuildSafety(unittest.TestCase):
    def setUp(self):
        # A fixture repository gives lock tests their own domain while this suite
        # itself remains under the real project's outer heavy-work lock.
        self.temporary = tempfile.TemporaryDirectory(prefix='build-guard-test-')
        self.fixture = Path(self.temporary.name)
        subprocess.run(['git', 'init', '--quiet', str(self.fixture)], check=True)

    def tearDown(self):
        self.temporary.cleanup()

    def test_checked_in_defaults_bound_build_and_test_parallelism_separately(self):
        config = tomllib.loads((ROOT / '.cargo/config.toml').read_text())
        self.assertEqual(config['build']['jobs'], 2)
        self.assertEqual(config['env']['RUST_TEST_THREADS'], {'value': '2', 'force': True})

    def test_artifact_volume_is_verified_and_tmpfs_is_refused(self):
        if not Path('/proc/self/mountinfo').exists():
            self.skipTest('Linux filesystem observation')
        actual = safety.disk_directory(ROOT / '.build-artifacts')
        self.assertNotIn(actual['filesystem'], ['tmpfs', 'ramfs', 'devtmpfs', 'overlay'])
        with patch.object(safety.Path, 'read_text', return_value='1 0 0:1 / / rw - tmpfs tmpfs rw\n'):
            with self.assertRaisesRegex(RuntimeError, 'disk-backed'):
                safety.disk_directory(ROOT / '.build-artifacts')

    def test_guard_sets_disk_temporary_and_limits_only_for_its_process(self):
        before = {key: os.environ.get(key) for key in ['TMPDIR', 'TMP', 'TEMP', 'CARGO_BUILD_JOBS', 'RUST_TEST_THREADS']}
        with safety.workflow(self.fixture, 'unit-env') as session:
            with tempfile.TemporaryDirectory(prefix='safe-workflow-fixture-') as name:
                self.assertTrue(Path(name).is_relative_to(session.temporary))
            self.assertEqual(os.environ['CARGO_BUILD_JOBS'], '2')
            self.assertEqual(os.environ['RUST_TEST_THREADS'], '2')
            with safety.workflow(self.fixture, 'nested') as inner:
                self.assertIs(inner, session)
        self.assertEqual({key: os.environ.get(key) for key in before}, before)

    def test_other_process_refuses_shared_lock_without_starting_command(self):
        script = "import sys; sys.path.insert(0, sys.argv[1]); from build_safety import workflow;\nwith workflow(sys.argv[2], 'competing-unit'): print('UNEXPECTED acquired')"
        with safety.workflow(self.fixture, 'unit-lock'):
            child = subprocess.run([sys.executable, '-c', script, str(ROOT / 'tools'), str(self.fixture)], capture_output=True, text=True)
            self.assertNotEqual(child.returncode, 0)
            self.assertIn('heavy-work lock', child.stderr)
            self.assertNotIn('UNEXPECTED acquired', child.stdout)

    def test_initial_pressure_refuses_new_work_and_retains_observation(self):
        with patch.object(safety, 'snapshot', return_value={'time': 0, 'available': 1}):
            with self.assertRaisesRegex(RuntimeError, 'refused new work'):
                with safety.workflow(self.fixture, 'unit-refused'):
                    self.fail('Must not launch work')

    def test_sustained_mock_pressure_cancels_only_the_wrapper_owned_child(self):
        good = {'time': 0, 'available': 12 * 2**30}
        low = {'time': 1, 'available': 1}
        count = 0
        def observe():
            nonlocal count
            count += 1
            return good if count == 1 else low
        with patch.object(safety, 'snapshot', side_effect=observe):
            with self.assertRaisesRegex(RuntimeError, 'Sustained memory pressure'):
                with safety.workflow(self.fixture, 'unit-mocked-pressure') as session:
                    session.run([sys.executable, '-c', 'import time; time.sleep(20)'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        self.assertGreaterEqual(count, 4)

    def test_uncoordinated_heavy_command_is_rejected(self):
        with self.assertRaisesRegex(RuntimeError, 'requires the shared'):
            safety.run([sys.executable, '-c', 'raise RuntimeError("must not execute")'])

    def test_pressure_gate_uses_available_and_psi_not_free_memory_alone(self):
        self.assertFalse(safety.pressured({'available': 10 * 2**30, 'pressure': {'some': {'avg10': 0}, 'full': {'avg10': 0}}}))
        self.assertTrue(safety.pressured({'available': 10 * 2**30, 'pressure': {'full': {'avg10': 5}}}))
        self.assertTrue(safety.pressured({'available': 10 * 2**30, 'pressure': {'some': {'avg10': 15}}}))

    def test_artifact_symlink_cannot_escape_the_project_even_to_another_disk_folder(self):
        if os.name != 'posix':
            self.skipTest('Symlink privilege acceptance is platform-specific')
        with tempfile.TemporaryDirectory() as outside:
            (self.fixture / '.build-artifacts').symlink_to(outside, target_is_directory=True)
            with self.assertRaisesRegex(RuntimeError, 'escaped'):
                with safety.workflow(self.fixture, 'unit-escape'):
                    self.fail('Must not create external artifacts')

    def test_sampler_failure_fails_closed_without_unmonitored_work(self):
        count = 0
        def sample():
            nonlocal count
            count += 1
            if count == 1:
                return {'time': 0, 'available': 12 * 2**30}
            raise OSError('controlled sampler fixture failure')
        with patch.object(safety, 'snapshot', side_effect=sample):
            with self.assertRaisesRegex(RuntimeError, 'monitoring failed'):
                with safety.workflow(self.fixture, 'unit-monitor-error') as session:
                    session.run([sys.executable, '-c', 'import time; time.sleep(20)'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

    @unittest.skipUnless(os.name == 'posix', 'POSIX owned process group acceptance')
    def test_exited_leader_cannot_leave_running_descendant_and_release_lock_as_success(self):
        pidfile = self.fixture / 'own-child.pid'
        grandchild = "import signal,time,os;from pathlib import Path;signal.signal(signal.SIGINT,signal.SIG_IGN);signal.signal(signal.SIGTERM,signal.SIG_IGN);Path(%r).write_text(str(os.getpid()));time.sleep(20)" % str(pidfile)
        leader = "import subprocess,sys,time;from pathlib import Path;subprocess.Popen([sys.executable,'-c',%r]);\nwhile not Path(%r).exists():time.sleep(.01)" % (grandchild, str(pidfile))
        start = time.monotonic()
        with self.assertRaisesRegex(RuntimeError, 'active descendants'):
            with safety.workflow(self.fixture, 'unit-descendants') as session:
                session.run([sys.executable, '-c', leader], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        self.assertLess(time.monotonic() - start, 8)
        child = Path('/proc') / pidfile.read_text().strip()
        if child.exists():
            self.assertEqual((child / 'stat').read_text().rsplit(')', 1)[1].split()[0], 'Z')
        self.assertFalse((self.fixture / '.build-artifacts/unfinished-child.json').exists())

    def test_keyboard_interrupt_also_reaps_only_the_owned_command(self):
        with safety.workflow(self.fixture, 'unit-keyboard') as session:
            session.pressure.set()
            with patch.object(session, 'checkpoint', side_effect=[None, KeyboardInterrupt]):
                with self.assertRaises(KeyboardInterrupt):
                    session.run([sys.executable, '-c', 'import time; time.sleep(20)'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            session.pressure.clear()

    def test_cargo_flags_cannot_override_validated_target_jobs_or_config(self):
        import dev
        for flags in [['--target-dir', '/tmp/unsafe'], ['--target-dir=/tmp/unsafe'], ['--manifest-path=elsewhere/Cargo.toml'], ['--config', 'build.jobs=99'], ['-j99'], ['--jobs=99'], ['--', '--test-threads=99'], ['-C', '/tmp']]:
            with self.assertRaises(RuntimeError, msg=str(flags)):
                dev.validate_cargo_args(flags)
        dev.validate_cargo_args(['--offline', '--all-targets', '--', '-D', 'warnings'])

    def test_all_output_forms_are_contained_normalized_and_duplicates_rejected(self):
        import dev
        cwd = self.fixture / 'worktree'
        cwd.mkdir()
        for arguments in [['native_startup.py', '--output=/tmp/unsafe'], ['native_startup.py', '--output', '/tmp/unsafe'], ['native_startup.py', '--output=one', '--output', 'two']]:
            with self.assertRaises(RuntimeError):
                dev.tool_outputs(arguments, cwd, self.fixture)
        args = dev.tool_outputs(['native_startup.py', '--output', 'evidence'], cwd, self.fixture)
        self.assertEqual(args[-1], str(cwd / 'evidence'))
        args = dev.tool_outputs(['native_startup.py', '--output=evidence'], cwd, self.fixture)
        self.assertEqual(args[-1], '--output=' + str(cwd / 'evidence'))

    def test_output_abbreviations_do_not_bypass_path_validation(self):
        import dev
        for option in ['--o=/tmp/unsafe', '--out=/tmp/unsafe', '--outp', '--outpu=/tmp/unsafe']:
            with self.assertRaisesRegex(RuntimeError, 'Abbreviated'):
                dev.tool_outputs(['native_startup.py', option], self.fixture, self.fixture)

    def test_cleanup_failure_retains_a_blocking_marker(self):
        with self.assertRaisesRegex(OSError, 'controlled cleanup failure'):
            with safety.workflow(self.fixture, 'unit-cleanup-failure') as session:
                with patch.object(safety, 'cleanup_child', side_effect=OSError('controlled cleanup failure')):
                    session.run([sys.executable, '-c', 'pass'])
        self.assertTrue((self.fixture / '.build-artifacts/active-command.json').is_file())
        with self.assertRaisesRegex(RuntimeError, 'command marker'):
            with safety.workflow(self.fixture, 'unit-after-cleanup-failure'):
                self.fail('Must not launch more work')

    @unittest.skipUnless(os.name == 'posix', 'POSIX inherited lock/abrupt-death test')
    def test_abrupt_wrapper_death_cannot_release_permission_for_another_build(self):
        pidfile = self.fixture / 'abrupt-child.pid'
        child_code = "import os,time;from pathlib import Path;Path(%r).write_text(str(os.getpid()));time.sleep(20)" % str(pidfile)
        wrapper = "import sys;sys.path.insert(0,%r);from build_safety import workflow;\nwith workflow(%r,'unit-abrupt') as s:s.run([sys.executable,'-c',%r])" % (str(ROOT / 'tools'), str(self.fixture), child_code)
        process = subprocess.Popen([sys.executable, '-c', wrapper], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        pid = None
        try:
            deadline = time.monotonic() + 5
            while not pidfile.exists() and time.monotonic() < deadline:
                time.sleep(.01)
            self.assertTrue(pidfile.exists())
            pid = int(pidfile.read_text())
            process.kill(); process.wait(timeout=2)
            with self.assertRaisesRegex(RuntimeError, 'heavy-work lock|command marker'):
                with safety.workflow(self.fixture, 'unit-after-abrupt'):
                    self.fail('Must not launch while the owned child lives')
        finally:
            if process.poll() is None:
                process.kill(); process.wait(timeout=2)
            if pid is not None:
                try:
                    os.killpg(pid, 9)
                except ProcessLookupError:
                    pass
        self.assertTrue((self.fixture / '.build-artifacts/active-command.json').is_file())
