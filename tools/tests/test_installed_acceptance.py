"""Read-only candidate-boundary checks; actual lifecycle is an explicit guarded tool."""
import hashlib
import json
from pathlib import Path
import tempfile
import sys
import os
import signal
import time
from unittest.mock import patch
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from installed_acceptance import candidate, require, installation_base, clean_uninstalled, run_checked, bounded_capture


class InstalledCandidateInput(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='dlms-candidate-boundary-')
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.location = self.root / 'dist/candidate'
        self.location.mkdir(parents=True)
        for name in ['install.sh', 'uninstall.sh', 'dlms-learning-lab', 'INSTALL_FILES.txt']:
            (self.location / name).write_bytes(b'fixed inert test resource\n')
        self.manifest = self.location / 'BUILD_INFO.json'
        self.info = {'revision': 'a' * 40, 'binary_sha256': hashlib.sha256(
            (self.location / 'dlms-learning-lab').read_bytes()).hexdigest()}
        self.manifest.write_text(json.dumps(self.info))

    def test_matching_ordinary_candidate_is_identified_without_execution(self):
        self.assertEqual(candidate(self.root, self.location), (self.location, self.info))

    def test_foreign_candidate_and_changed_executable_are_rejected(self):
        with self.assertRaises(ValueError):
            candidate(self.root, self.root)
        (self.location / 'dlms-learning-lab').write_text('changed, never executed')
        with self.assertRaises(ValueError):
            candidate(self.root, self.location)

    def test_bad_identity_oversize_manifest_and_symlink_are_rejected(self):
        self.info['revision'] = 'latest'
        self.manifest.write_text(json.dumps(self.info))
        with self.assertRaises(ValueError):
            candidate(self.root, self.location)
        self.manifest.write_text(' ' * 1_000_001)
        with self.assertRaises(ValueError):
            candidate(self.root, self.location)
        self.manifest.unlink()
        self.manifest.symlink_to(self.location / 'INSTALL_FILES.txt')
        with self.assertRaises(ValueError):
            candidate(self.root, self.location)

    def test_acceptance_checks_remain_explicit_under_optimized_python(self):
        with self.assertRaises(ValueError):
            require(False, 'failed check')
        require(True, 'passing check')

    @unittest.skipUnless(os.name == 'posix', 'POSIX symlink fixture')
    def test_dist_and_installation_base_symlinks_cannot_authorize_external_locations(self):
        external = self.root / 'outside-project'
        external.mkdir()
        project = self.root / 'project'
        project.mkdir()
        (project / 'dist').symlink_to(self.location.parent, target_is_directory=True)
        with self.assertRaises(RuntimeError):
            candidate(project, self.location)
        (project / '.build-artifacts').mkdir()
        (project / '.build-artifacts/installations').symlink_to(external, target_is_directory=True)
        with self.assertRaises(RuntimeError):
            installation_base(project)
        self.assertEqual(list(external.iterdir()), [])

    def test_uninstall_cannot_claim_success_with_a_desktop_entry_left_behind(self):
        prefix, launcher, desktop = (self.root / p for p in ['application', 'launcher', 'desktop'])
        desktop.write_text('left by an incomplete uninstaller')
        with self.assertRaises(ValueError):
            clean_uninstalled(prefix, launcher, desktop)
        self.assertTrue(desktop.is_file())

    @unittest.skipUnless(os.name == 'posix', 'POSIX symlink fixture')
    def test_uninstall_cannot_hide_dangling_launcher_or_prefix_links(self):
        paths = [self.root / p for p in ['application', 'launcher', 'desktop']]
        for path in paths:
            path.symlink_to(self.root / 'missing-target')
            with self.assertRaises(ValueError):
                clean_uninstalled(*paths)
            self.assertTrue(path.is_symlink())
            path.unlink()
        clean_uninstalled(*paths)

    def test_oversize_timeout_and_failed_child_reports_are_refused_without_loading_output(self):
        log = self.root / 'never-created.log'
        for result in [(0, False, True, False), (0, True, False, False), (1, False, False, False), (0, False, False, True)]:
            with patch('installed_acceptance.bounded_capture', return_value=result) as mocked:
                with self.assertRaises(ValueError):
                    run_checked(['fixed-app', '--version'], log, self.root, {})
                self.assertEqual(mocked.call_args.kwargs['cap'], 32_768)
                self.assertFalse(log.exists())

    @unittest.skipUnless(sys.platform == 'linux' and hasattr(os, 'pidfd_open')
                         and hasattr(signal, 'pidfd_send_signal'), 'Linux own-child pidfd fixture')
    def test_descendant_held_stdout_does_not_block_timeout_cleanup(self):
        pidfile = self.root / 'own-fixture-child.pid'
        source = (
            "import subprocess,sys,time; from pathlib import Path; "
            "child=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)',sys.argv[1]]); "
            "Path(sys.argv[1]).write_text(str(child.pid)); "
            "print('fixture ready',flush=True); time.sleep(30)"
        )
        started = time.monotonic()
        try:
            result = bounded_capture([sys.executable, '-c', source, str(pidfile)],
                                     self.root / 'held-output.log', self.root,
                                     dict(os.environ), timeout=1, cap=32_768)
            self.assertTrue(pidfile.exists(), 'fixture never reached descendant-held pipe')
            self.assertTrue(result[1], result)
            self.assertLess(time.monotonic() - started, 5)
        finally:
            if pidfile.exists():
                # Bind an exact descriptor, then verify our unique fixture-path
                # marker before signaling. A reused PID must never be signaled.
                pid = int(pidfile.read_text())
                try:
                    descriptor = os.pidfd_open(pid)
                except ProcessLookupError:
                    pass
                else:
                    try:
                        expected = b'\0'.join([os.fsencode(sys.executable), b'-c',
                            b'import time; time.sleep(30)', os.fsencode(pidfile), b''])
                        try:
                            matches = Path(f'/proc/{pid}/cmdline').read_bytes() == expected
                        except FileNotFoundError:
                            matches = False
                        if matches:
                            signal.pidfd_send_signal(descriptor, signal.SIGTERM)
                    finally:
                        os.close(descriptor)
