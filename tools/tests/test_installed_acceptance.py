"""Read-only candidate-boundary checks; actual lifecycle is an explicit guarded tool."""
import hashlib
import copy
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
from installed_acceptance import candidate, require, installation_base, clean_uninstalled, run_checked, bounded_capture, review_sources, upgrade_identity


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

    def review_info(self):
        return dict(self.info, revision='a' * 40 + '-dirty',
                    pilot_profile='Separate ccna-presentation-pilot subdirectory; originals never written',
                    pilot_catalog={'labs': 279, 'units': 84, 'all_existing_review_labs': True},
                    pilot_source_sha256={name: 'b' * 64 for name in
                        ('src/main.rs', 'src/lib.rs', 'src/pack.rs', 'src/course.rs', 'scenarios/task.json')},
                    bundled_content={'task.json': 'b' * 64})

    def test_dirty_review_candidate_requires_explicit_profile_catalog_and_source_closure(self):
        info = self.review_info()
        self.manifest.write_text(json.dumps(info))
        pin = hashlib.sha256(self.manifest.read_bytes()).hexdigest()
        self.assertEqual(candidate(self.root, self.location, pin), (self.location, info))
        for rejected_pin in (None, 'c' * 64, 'bad'):
            with self.assertRaises(ValueError):
                candidate(self.root, self.location, rejected_pin)
        for field in ('pilot_profile', 'pilot_catalog', 'pilot_source_sha256'):
            changed = dict(info)
            changed.pop(field)
            self.manifest.write_text(json.dumps(changed))
            with self.assertRaises(ValueError, msg=field):
                candidate(self.root, self.location, hashlib.sha256(self.manifest.read_bytes()).hexdigest())

    def test_dirty_review_source_closure_rejects_malformed_names_hashes_and_limits(self):
        for name in ('/src/main.rs', 'src/../outside', 'src//bad', 'src/./bad',
                     'src/back\\slash', 'src/line\nfeed', 'tools/foreign.py', 'src/' + 'x' * 260):
            info = self.review_info()
            info['pilot_source_sha256'][name] = 'b' * 64
            with self.assertRaises(ValueError, msg=repr(name)):
                review_sources(info)
        for value in ('bad', 'B' * 64, 123, None):
            info = self.review_info()
            info['pilot_source_sha256']['src/main.rs'] = value
            with self.assertRaises(ValueError):
                review_sources(info)
        for sources in ({}, [], None, {f'src/file{i}.rs': 'b' * 64 for i in range(2001)}):
            info = dict(self.review_info(), pilot_source_sha256=sources)
            with self.assertRaises(ValueError):
                review_sources(info)
        info = self.review_info()
        del info['pilot_source_sha256']['scenarios/task.json']
        with self.assertRaises(ValueError):
            review_sources(info)
        for catalog in ({}, {'labs': True, 'units': 1, 'all_existing_review_labs': True},
                        {'labs': 2, 'units': 3, 'all_existing_review_labs': True},
                        {'labs': 279, 'units': 84, 'all_existing_review_labs': False}):
            with self.assertRaises(ValueError):
                review_sources(dict(self.review_info(), pilot_catalog=catalog))

    def test_review_upgrade_requires_both_changed_source_and_changed_executable(self):
        old = self.review_info()
        new = copy.deepcopy(old)
        with self.assertRaises(ValueError):
            upgrade_identity(old, new)
        new['binary_sha256'] = 'c' * 64
        with self.assertRaises(ValueError):
            upgrade_identity(old, new)
        new['pilot_source_sha256']['src/main.rs'] = 'd' * 64
        self.assertEqual(upgrade_identity(old, new), 'different-review-source-and-binary')
        new['binary_sha256'] = old['binary_sha256']
        with self.assertRaises(ValueError):
            upgrade_identity(old, new)

    def test_source_addition_or_deletion_alone_does_not_establish_review_upgrade(self):
        old = self.review_info()
        for change in ('addition', 'deletion'):
            new = copy.deepcopy(old)
            new['binary_sha256'] = 'c' * 64
            if change == 'addition':
                new['pilot_source_sha256']['src/new.rs'] = 'd' * 64
            else:
                old['pilot_source_sha256']['src/extra.rs'] = 'd' * 64
            with self.assertRaises(ValueError):
                upgrade_identity(old, new)
            new['pilot_source_sha256']['src/main.rs'] = 'e' * 64
            self.assertEqual(upgrade_identity(old, new), 'different-review-source-and-binary')

    def test_clean_upgrade_keeps_different_revision_and_executable_requirements(self):
        old = dict(self.info)
        new = dict(self.info, binary_sha256='c' * 64)
        with self.assertRaises(ValueError):
            upgrade_identity(old, new)
        new['revision'] = 'b' * 40
        self.assertEqual(upgrade_identity(old, new), 'different-revision')
        new['binary_sha256'] = old['binary_sha256']
        with self.assertRaises(ValueError):
            upgrade_identity(old, new)

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
