"""SOURCE ONLY guard tests. No subprocess, Bash, GUI or compiler is started here."""
import importlib.util
import copy
from contextlib import contextmanager
import io
import stat
from types import SimpleNamespace
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('native_reopen', Path(__file__).resolve().parents[1] / 'native_reopen.py')
reopen = importlib.util.module_from_spec(spec)
spec.loader.exec_module(reopen)


@unittest.skipUnless(os.name == 'posix' and Path('/proc/self/fd').is_dir(), 'fixed Linux tool')
class ReopenGuards(unittest.TestCase):
    def test_child_environment_is_private_and_does_not_copy_startup_or_loader_overrides(self):
        session = Path('/fixed-project/.build-artifacts/tmp/native-reopen-test')
        env = reopen.child_environment(session, 'read', {'HOME': '/private-home', 'PATH': '/untrusted', 'LD_PRELOAD': '/bad.so', 'DLMS_SMOKE_STAGE': 'case-lessons', 'XDG_DATA_HOME': '/personal', 'WAYLAND_DISPLAY': 'wayland-0', 'TMPDIR': '/fixed-project/.build-artifacts/tmp'})
        self.assertEqual(env['XDG_DATA_HOME'], str(session / 'data'))
        self.assertEqual(env['DLMS_NATIVE_REOPEN_PHASE'], 'read')
        self.assertEqual(env['WAYLAND_DISPLAY'], 'wayland-0')
        for key in ['HOME', 'PATH', 'LD_PRELOAD', 'DLMS_SMOKE_STAGE']:
            self.assertNotIn(key, env)
        with self.assertRaises(reopen.ReopenError):
            reopen.child_environment(session, 'arbitrary', {})

    def test_exclusive_write_preserves_unknown_existing_bytes_and_symlinks(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / 'existing'; target.write_bytes(b'keep')
            with self.assertRaises(FileExistsError):
                reopen.exclusive(target, b'replace')
            self.assertEqual(target.read_bytes(), b'keep')
            link = root / 'link'; link.symlink_to(target)
            with self.assertRaises(FileExistsError):
                reopen.exclusive(link, b'replace')
            self.assertEqual(target.read_bytes(), b'keep')

    def test_fresh_output_never_reuses_directory_or_symlink(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); child = root / 'fresh'
            reopen.fresh_directory(child)
            self.assertEqual(child.stat().st_mode & 0o777, 0o700)
            with self.assertRaises(reopen.ReopenError):
                reopen.fresh_directory(child)
            alias = root / 'alias'; alias.symlink_to(child)
            with self.assertRaises(reopen.ReopenError):
                reopen.fresh_directory(alias)

    def test_ordinary_reads_refuse_symlink_components_and_finite_ceiling(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); f = root / 'file'; f.write_bytes(b'12345')
            self.assertEqual(reopen.read_bytes(f, 5), b'12345')
            with self.assertRaises(reopen.ReopenError):
                reopen.read_bytes(f, 4)
            link = root / 'alias'; link.symlink_to(f)
            with self.assertRaises(OSError):
                reopen.read_bytes(link, 5)
            directory = root / 'dir'; directory.mkdir(); (directory / 'x').write_bytes(b'x')
            alias = root / 'dir-link'; alias.symlink_to(directory)
            with self.assertRaises(OSError):
                reopen.read_bytes(alias / 'x', 5)

    def test_hash_is_complete_and_bounded(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); f = root / 'file'; f.write_bytes(b'abc')
            self.assertEqual(reopen.pin(f, 3), {'bytes': 3, 'sha256': 'ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad'})
            with self.assertRaises(reopen.ReopenError):
                reopen.pin(f, 2)

    def test_executable_is_never_an_outside_program_or_symlink(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); target = root / 'target'; target.mkdir()
            app = target / 'dlms-learning-lab'; app.write_bytes(b'\x7fELFfake'); app.chmod(0o700)
            self.assertEqual(reopen.app_path(app, root), app)
            outside = root / 'dlms-learning-lab'; outside.write_bytes(b'\x7fELFfake'); outside.chmod(0o700)
            with self.assertRaises(reopen.ReopenError):
                reopen.app_path(outside, root)
            alias = target / 'another'; alias.symlink_to(app)
            with self.assertRaises(reopen.ReopenError):
                reopen.app_path(alias, root)
            app.chmod(0o4700)
            with self.assertRaises(reopen.ReopenError):
                reopen.app_path(app, root)

    def test_actual_installed_root_is_precisely_allowed_but_other_artifact_roots_are_not(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            installed = root / '.build-artifacts' / 'installations' / 'controlled-install' / 'bin'
            installed.mkdir(parents=True)
            app = installed / 'dlms-learning-lab'; app.write_bytes(b'\x7fELFsmall'); app.chmod(0o700)
            self.assertEqual(reopen.app_path(app, root), app)
            unrelated = root / '.build-artifacts' / 'reviews'; unrelated.mkdir()
            outside = unrelated / 'dlms-learning-lab'; outside.write_bytes(b'\x7fELFsmall'); outside.chmod(0o700)
            with self.assertRaises(reopen.ReopenError): reopen.app_path(outside, root)
            sibling = root / '.build-artifacts' / 'installations-other'; sibling.mkdir()
            wrong = sibling / 'dlms-learning-lab'; wrong.write_bytes(b'\x7fELFsmall'); wrong.chmod(0o700)
            with self.assertRaises(reopen.ReopenError): reopen.app_path(wrong, root)
            alias = installed / 'aliased'; alias.mkdir()
            (alias / 'dlms-learning-lab').symlink_to(app)
            with self.assertRaises(OSError): reopen.app_path(alias / 'dlms-learning-lab', root)

    def test_one_gib_binary_limit_covers_observed_debug_metadata_without_large_allocation(self):
        self.assertEqual(reopen.BINARY_CAP, 1024 * 1024 * 1024)
        observed_size = 379_733_232
        seen = []
        @contextmanager
        def fake_ordinary(path, cap):
            seen.append(cap)
            self.assertLess(observed_size, cap)
            yield io.BytesIO(b'\x7fELF'), SimpleNamespace(st_mode=stat.S_IFREG | 0o700, st_size=observed_size)
        root = Path('/controlled-project')
        app = root / 'target/debug/dlms-learning-lab'
        with patch.object(reopen, 'ordinary', side_effect=fake_ordinary):
            self.assertEqual(reopen.app_path(app, root), app)
        self.assertEqual(seen, [reopen.BINARY_CAP])

    def test_inventory_rejects_file_count_without_reading_all_large_bodies(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            for n in range(3): (root / str(n)).write_bytes(b'x')
            with patch.object(reopen, 'FILE_COUNT_CAP', 2):
                with self.assertRaises(reopen.ReopenError): reopen.inventory(root)
            with patch.object(reopen, 'EVIDENCE_CAP', 2):
                with self.assertRaises(reopen.ReopenError): reopen.inventory(root)

    def test_inventory_rejects_symlinks_instead_of_following_them(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); target = root / 'target'; target.write_bytes(b'x')
            (root / 'alias').symlink_to(target)
            with self.assertRaises(reopen.ReopenError): reopen.inventory(root)

    def test_unknown_launch_role_is_rejected_before_any_subprocess(self):
        with tempfile.TemporaryDirectory() as temporary, patch.object(reopen.subprocess, 'Popen') as popen:
            root = Path(temporary)
            with self.assertRaises(reopen.ReopenError): reopen.capture(root / 'fake', 'arbitrary', root / 'log', root, {}, 3, 1)
            popen.assert_not_called()

    def test_child_cleanup_stops_only_the_supplied_fake_child(self):
        class Fake:
            status = None
            signals = []
            def poll(self): return self.status
            def send_signal(self, sig): self.signals.append(sig)
            def wait(self, timeout): self.status = 0; return 0
        child = Fake(); reopen.stop_own_child(child)
        self.assertEqual(child.status, 0)
        self.assertEqual(len(child.signals), 1)

    def test_abbreviated_and_arbitrary_options_are_not_accepted(self):
        with patch.object(reopen.os, 'environ', {}), patch('sys.argv', ['native_reopen.py', '--app', '/fixed', '--command', 'anything']):
            with self.assertRaises(SystemExit): reopen.main()

@unittest.skipUnless(os.name == 'posix' and Path('/proc/self/fd').is_dir(), 'fixed Linux tool')
class FixedCaptureGuards(unittest.TestCase):
    @staticmethod
    def fake_pipe(data=b'', hold=False, running=False):
        read, write = os.pipe()
        if data: os.write(write, data)
        if not hold: os.close(write)
        class Fake:
            stdout = os.fdopen(read, 'rb')
            pid = 123456
            status = None if running else 0
            signals = []
            def poll(self): return self.status
            def wait(self, timeout): self.status = 0; return 0
            def send_signal(self, sig): self.signals.append(sig); self.status = 0
        return Fake(), write if hold else None

    def test_fixed_argv_lock_forwarding_and_eof_without_starting_a_subprocess(self):
        fake, _ = self.fake_pipe(b'controlled output\n')
        with tempfile.TemporaryDirectory() as temporary, patch.object(reopen.subprocess, 'Popen', return_value=fake) as popen:
            root = Path(temporary)
            result = reopen.capture(root / 'dlms-learning-lab', 'read', root / 'log', root, {}, 73, 1)
            self.assertEqual(result['exit'], 0)
            self.assertEqual((root / 'log').read_bytes(), b'controlled output\n')
            self.assertEqual(popen.call_args.args[0], [str(root / 'dlms-learning-lab'), '--developer'])
            self.assertEqual(popen.call_args.kwargs['pass_fds'], (73,))
            self.assertNotIn('start_new_session', popen.call_args.kwargs)

    def test_excess_log_is_bounded_and_failure_does_not_become_success(self):
        fake, _ = self.fake_pipe(b'0123456789abcdef')
        with tempfile.TemporaryDirectory() as temporary, patch.object(reopen.subprocess, 'Popen', return_value=fake), patch.object(reopen, 'LOG_CAP', 8):
            root = Path(temporary)
            with self.assertRaises(reopen.ReopenError): reopen.capture(root / 'fake', 'write', root / 'log', root, {}, 73, 1)
            self.assertEqual((root / 'log').read_bytes(), b'01234567')

    def test_timeout_cleans_only_supplied_fake_child_and_retains_log(self):
        fake, write = self.fake_pipe(hold=True, running=True)
        try:
            with tempfile.TemporaryDirectory() as temporary, patch.object(reopen.subprocess, 'Popen', return_value=fake):
                root = Path(temporary)
                with self.assertRaises(reopen.ReopenError): reopen.capture(root / 'fake', 'read', root / 'log', root, {}, 73, .001)
                self.assertEqual(len(fake.signals), 1)
                self.assertTrue((root / 'log').exists())
        finally: os.close(write)

    def test_held_pipe_after_leader_exit_is_not_unbounded(self):
        fake, write = self.fake_pipe(hold=True)
        clock = [0.0]
        def step(): clock[0] += .3; return clock[0]
        try:
            with tempfile.TemporaryDirectory() as temporary, patch.object(reopen.subprocess, 'Popen', return_value=fake), patch.object(reopen.time, 'monotonic', side_effect=step):
                root = Path(temporary)
                with self.assertRaises(reopen.ReopenError): reopen.capture(root / 'fake', 'read', root / 'log', root, {}, 73, 120)
                self.assertEqual(fake.signals, [])
        finally: os.close(write)

    def test_complete_is_no_clobber_and_a_failed_precommit_never_publishes_it(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with patch.object(reopen.os, 'fsync', side_effect=OSError('controlled failure')):
                with self.assertRaises(OSError): reopen.publish_manifest(root, {'passed': True})
            self.assertFalse((root / 'COMPLETE').exists())
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); (root / 'COMPLETE').write_bytes(b'preserve existing')
            with self.assertRaises(FileExistsError): reopen.publish_manifest(root, {'passed': True})
            self.assertEqual((root / 'COMPLETE').read_bytes(), b'preserve existing')


class RawObservationGuards(unittest.TestCase):
    @staticmethod
    def fixture():
        # Hand-authored pending-only synthetic wire data: no successful Record/credit.
        portable = {'schema_version': 1, 'clock': 1, 'favorites': [], 'attempts': {
            'shell-case-publication': {'scenario_version': 1, 'state': {'kind': 'Script', 'source': "printf 'unfinished", 'run': None},
                'step': 0, 'challenge': False, 'hints': 0, 'transcript': [], 'visited': 1}}}
        receipts = {'workspace': True, 'shell': [8]}
        raw_json = copy.deepcopy(portable)
        raw_json['migration_receipts'] = copy.deepcopy(receipts)
        preferences = {'version': 1, 'theme': 'dlms-light', 'text_scale': 1.5}
        observed = {'progress': portable, 'raw_progress': raw_json,
                    'migration_receipts': receipts, 'preferences': preferences}
        raw = json.dumps(raw_json, separators=(',', ':')).encode()
        prefs = json.dumps(preferences).encode()
        return observed, raw, prefs

    def test_full_raw_and_explicit_receipts_succeed_despite_portable_serialization_omission(self):
        expected, raw, prefs = self.fixture()
        self.assertNotIn('migration_receipts', expected['progress'])
        self.assertIn('migration_receipts', json.loads(raw))
        reopen.verify_writer_saved_bytes(expected, raw, prefs, raw, prefs)
        self.assertEqual(json.loads(raw)['migration_receipts'], expected['migration_receipts'])

    def test_every_post_exit_byte_change_in_provenance_drafts_identity_format_or_preferences_fails(self):
        expected, raw, prefs = self.fixture()
        receipt = json.loads(raw); receipt['migration_receipts']['workspace'] = False
        draft = json.loads(raw); draft['attempts']['shell-case-publication']['state']['source'] += 'changed'
        identity = json.loads(raw); identity['favorites'] = ['shell-case-interface']
        for changed in [json.dumps(receipt).encode(), json.dumps(draft).encode(), json.dumps(identity).encode(), raw + b'\n']:
            with self.assertRaises(reopen.ReopenError): reopen.verify_writer_saved_bytes(expected, changed, prefs, raw, prefs)
        changed_prefs = dict(expected['preferences'], text_scale=2.0)
        with self.assertRaises(reopen.ReopenError): reopen.verify_writer_saved_bytes(expected, raw, json.dumps(changed_prefs).encode(), raw, prefs)
        self.assertEqual(json.loads(raw), expected['raw_progress'])

    def test_full_json_and_loaded_receipt_checks_cannot_be_masked_by_matching_snapshot_bytes(self):
        expected, raw, prefs = self.fixture()
        receipt = json.loads(raw); receipt['migration_receipts']['workspace'] = False
        extra = json.loads(raw); extra['unknown_new_field'] = True
        draft = json.loads(raw); draft['attempts']['shell-case-publication']['state']['source'] += 'changed'
        for changed in [receipt, extra, draft]:
            both = json.dumps(changed).encode()
            with self.assertRaises(reopen.ReopenError): reopen.verify_writer_saved_bytes(expected, both, prefs, both, prefs)
        mismatched = copy.deepcopy(expected); mismatched['migration_receipts']['workspace'] = False
        self.assertEqual(json.loads(raw), mismatched['raw_progress'], 'full raw guard must already pass in this independent receipt check')
        with self.assertRaises(reopen.ReopenError): reopen.verify_writer_saved_bytes(mismatched, raw, prefs, raw, prefs)
        changed_prefs = json.dumps(dict(expected['preferences'], text_scale=2.0)).encode()
        with self.assertRaises(reopen.ReopenError): reopen.verify_writer_saved_bytes(expected, raw, changed_prefs, raw, changed_prefs)


if __name__ == '__main__':
    unittest.main()
