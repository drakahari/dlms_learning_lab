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

    def test_fixed_environment_gui_can_select_review_without_changing_legacy_argv_or_lock(self):
        fake,_=self.fake_pipe(b'controlled review output\n')
        with tempfile.TemporaryDirectory() as temporary, patch.object(reopen.subprocess,'Popen',return_value=fake) as popen:
            root=Path(temporary)
            result=reopen.capture(root/'dlms-learning-lab','write',root/'log',root,{},73,1,review=True)
            self.assertEqual(result['exit'],0)
            self.assertEqual(popen.call_args.args[0],[str(root/'dlms-learning-lab'),'--developer','--presentation-review'])
            self.assertEqual(popen.call_args.kwargs['pass_fds'],(73,))
        with patch.object(reopen.subprocess,'Popen') as popen:
            with self.assertRaises(reopen.ReopenError):reopen.capture(Path('/fixed'),'version',Path('/unused'),Path('/'),{},73,1,review=True)
            popen.assert_not_called()

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



class EnvironmentReplayGuards(unittest.TestCase):
    def fixture(self, revision=19):
        expected={'progress':{'attempts':{'lpic1-env-child-settings':{'state':{
            'kind':'Environment','simulation':{'assignment':{'source':'retained source',
            'result':{'revision':revision,'source':'retained source',
                      'task':{'id':'lpic1-env-child-settings'}}}}}}}}}
        initial={'evaluator_revision':revision,'saved_record_verified':False,
                 'current_assignment_credit':False,'drafts_preserved':True}
        replay={'observed':copy.deepcopy(expected),'record_verified':True,
                'current_assignment_credit':True}
        return initial, expected, replay

    def test_revision_matches_the_actual_writer_instead_of_hardcoded17(self):
        for revision in [17,19]:
            self.assertEqual(reopen.verify_environment_replay(*self.fixture(revision)),revision)

    def test_mismatched_revision_premature_credit_corrupt_drafts_and_fake_replay_are_refused(self):
        for value in [16,17,18,20,True,'19',19.0,None]:
            initial,expected,replay=self.fixture();initial['evaluator_revision']=value
            with self.assertRaises(reopen.ReopenError):reopen.verify_environment_replay(initial,expected,replay)
        for key,value in [('saved_record_verified',True),('saved_record_verified',0),
                          ('current_assignment_credit',True),('current_assignment_credit',0),
                          ('drafts_preserved',False),('drafts_preserved',1)]:
            initial,expected,replay=self.fixture();initial[key]=value
            with self.assertRaises(reopen.ReopenError):reopen.verify_environment_replay(initial,expected,replay)
        for key,value in [('record_verified',False),('record_verified',1),
                          ('current_assignment_credit',False),('observed',{})]:
            initial,expected,replay=self.fixture();replay[key]=value
            with self.assertRaises(reopen.ReopenError):reopen.verify_environment_replay(initial,expected,replay)
        for invalid in [None,{}, {'progress':{}},self.fixture(True)[1],self.fixture(16)[1]]:
            initial,_,replay=self.fixture()
            with self.assertRaises(reopen.ReopenError):reopen.verify_environment_replay(initial,invalid,replay)
        for target,key,value in [('state','kind','Script'),('record','source','different'),('task','id','other')]:
            initial,expected,replay=self.fixture();state=expected['progress']['attempts']['lpic1-env-child-settings']['state'];record=state['simulation']['assignment']['result']
            obj={'state':state,'record':record,'task':record['task']}[target];obj[key]=value
            with self.assertRaises(reopen.ReopenError):reopen.verify_environment_replay(initial,expected,replay)


class DhcpCaseGuards(unittest.TestCase):
    def test_user_mount_case_is_closed_and_old_markers_remain_exact(self):
        root=Path('/controlled/.build-artifacts/tmp/native-reopen-mount')
        original=reopen.session_marker(root,root.parent,'d'*32)
        self.assertNotIn('case',original)
        self.assertEqual(reopen.session_marker(root,root.parent,'d'*32,'user-mounts'),dict(original,case='user-mounts'))
        for bad in ['mount','UserMounts','../user-mounts','user-mounts;anything']:
            with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,root.parent,'d'*32,bad)

    def test_environment_case_is_explicit_closed_and_old_marker_wire_stays_exact(self):
        root=Path('/controlled/.build-artifacts/tmp/native-reopen-env')
        original=reopen.session_marker(root,root.parent,'c'*32)
        self.assertNotIn('case',original)
        self.assertEqual(reopen.session_marker(root,root.parent,'c'*32,'environment'),dict(original,case='environment'))
        for bad in ['Environment','env','../environment','environment;anything']:
            with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,root.parent,'c'*32,bad)

    def test_nfs_case_is_explicit_closed_marker_and_old_script_wire_stays_exact(self):
        root = Path('/controlled/.build-artifacts/tmp/native-reopen-nfs')
        original = reopen.session_marker(root, root.parent, 'b' * 32)
        self.assertNotIn('case', original)
        self.assertEqual(reopen.session_marker(root, root.parent, 'b' * 32, 'nfs'),
                         dict(original, case='nfs'))
        for case in ['NFS', 'nfs-any-path', '../nfs', 'nfs;anything']:
            with self.assertRaises(reopen.ReopenError):
                reopen.session_marker(root, root.parent, 'b' * 32, case)
    def test_default_marker_is_old_wire_and_only_closed_dhcp_case_adds_one_field(self):
        root = Path('/controlled/.build-artifacts/tmp/native-reopen-case')
        expected = {'version': 1, 'purpose': reopen.PURPOSE, 'root': str(root),
                    'temporary_root': str(root.parent), 'nonce': 'a' * 32}
        self.assertEqual(reopen.session_marker(root, root.parent, 'a' * 32), expected)
        self.assertEqual(reopen.session_marker(root, root.parent, 'a' * 32, 'script'), expected)
        self.assertEqual(reopen.session_marker(root, root.parent, 'a' * 32, 'dhcpv4'),
                         dict(expected, case='dhcpv4'))
        for case, nonce in [('arbitrary', 'a' * 32), ('dhcpv4', 'short')]:
            with self.assertRaises(reopen.ReopenError):
                reopen.session_marker(root, root.parent, nonce, case)

    def test_unknown_or_abbreviated_case_starts_no_child_and_creates_no_evidence(self):
        for args in [['--case', 'arbitrary'], ['--cas', 'dhcpv4']]:
            with patch('sys.argv', ['native_reopen.py', '--app', '/fixed'] + args), patch.object(reopen.subprocess, 'Popen') as popen, patch.object(reopen, 'fresh_directory') as fresh:
                with self.assertRaises(SystemExit): reopen.main()
                popen.assert_not_called()
                fresh.assert_not_called()

    def prepared(self, session):
        observed, raw, prefs = RawObservationGuards.fixture()
        data = session / 'data' / 'dlmslearninglab'; data.mkdir(parents=True)
        observed['progress_path'] = str(data / 'progress-v1.json')
        observed['preferences_path'] = str(data / 'preferences-v1.json')
        (data / 'progress-v1.json').write_bytes(raw)
        (data / 'preferences-v1.json').write_bytes(prefs)
        (session / 'reader-progress.raw').write_bytes(raw)
        (session / 'reader-preferences.raw').write_bytes(prefs)
        (session / 'reader-repaired.json').write_text(json.dumps(observed))
        return observed, raw, prefs

    def test_reader_post_exit_keeps_all_raw_fields_receipts_and_exact_snapshot_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            writer, raw, prefs = self.prepared(root)
            original_observation = (root / 'reader-repaired.json').read_bytes()
            self.assertEqual(reopen.verify_repaired_reader(root, writer), writer)
            path = Path(writer['progress_path'])
            for key in ['migration_receipts', 'attempts']:
                changed = json.loads(raw)
                changed[key] = {} if key == 'attempts' else {'workspace': False, 'shell': [8]}
                path.write_text(json.dumps(changed))
                with self.assertRaises(reopen.ReopenError): reopen.verify_repaired_reader(root, writer)
                path.write_bytes(raw)
            path.write_bytes(raw + b'\n')
            with self.assertRaises(reopen.ReopenError): reopen.verify_repaired_reader(root, writer)
            path.write_bytes(raw)
            Path(writer['preferences_path']).write_bytes(prefs + b'\n')
            with self.assertRaises(reopen.ReopenError): reopen.verify_repaired_reader(root, writer)
            self.assertEqual((root / 'reader-repaired.json').read_bytes(), original_observation)

    def test_repaired_portable_projection_cannot_be_changed_without_raw_bytes(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); writer, raw, prefs = self.prepared(root)
            # Only MainProgress's documented top-level receipt is nonportable.
            portable = json.loads(raw); del portable['migration_receipts']
            self.assertEqual(portable, writer['progress'])
            self.assertEqual(reopen.verify_repaired_reader(root, writer), writer)
            for key in ['clock', 'attempts', 'favorites', 'unknown_new_field']:
                tampered = copy.deepcopy(writer)
                tampered['progress'][key] = ['shell-case-publication'] if key == 'favorites' else 'changed-only-portable'
                self.assertNotEqual(tampered['progress'], writer['progress'], 'negative must actually change portable data')
                (root / 'reader-repaired.json').write_text(json.dumps(tampered))
                with self.assertRaises(reopen.ReopenError): reopen.verify_repaired_reader(root, writer)
                self.assertEqual(Path(writer['progress_path']).read_bytes(), raw)
                self.assertEqual((root / 'reader-progress.raw').read_bytes(), raw)
                self.assertEqual(Path(writer['preferences_path']).read_bytes(), prefs)
            (root / 'reader-repaired.json').write_text(json.dumps(writer))
            self.assertEqual(reopen.verify_repaired_reader(root, writer), writer)

    def test_repaired_observation_cannot_drop_keys_change_private_identity_or_escape_paths(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); writer, raw, prefs = self.prepared(root)
            original = copy.deepcopy(writer)
            candidates = []
            missing = copy.deepcopy(writer); del missing['migration_receipts']; candidates.append(missing)
            extra = copy.deepcopy(writer); extra['ignore_me'] = True; candidates.append(extra)
            moved = copy.deepcopy(writer); moved['progress_path'] = str(root / 'outside'); candidates.append(moved)
            changed = copy.deepcopy(writer); changed['raw_progress']['unknown'] = True; candidates.append(changed)
            for bad in candidates:
                (root / 'reader-repaired.json').write_text(json.dumps(bad))
                with self.assertRaises(reopen.ReopenError): reopen.verify_repaired_reader(root, writer)
            self.assertEqual(writer, original)
            self.assertEqual(Path(writer['progress_path']).read_bytes(), raw)
            self.assertEqual(Path(writer['preferences_path']).read_bytes(), prefs)


if __name__ == '__main__':
    unittest.main()


class HandoverReplayGuards(unittest.TestCase):
    def fixture(self, revision=19):
        expected={'progress':{'attempts':{'lpic1-editor-handover':{'state':{
            'kind':'EditorHandover','simulation':{'setup':{'assignment':{
                'source':'retained source','result':{'revision':revision,'source':'retained source',
                'task':{'id':'lpic1-editor-handover-assignment'}}},'example':{'result':None}},
                'assignment_launch':{'editor':'vi'},'example_launch':None,
                'files':{'assignment':{'checked':True}}}}}}}}
        initial={'evaluator_revision':revision,'saved_record_verified':False,
                 'current_assignment_credit':False,'drafts_preserved':True}
        replay={'observed':copy.deepcopy(expected),'record_verified':True,'current_assignment_credit':True}
        return initial,expected,replay

    def test_same_binary_replay_and_closed_case_preserve_old_marker_shape(self):
        self.assertEqual(reopen.verify_editor_handover_replay(*self.fixture()),19)
        root=Path('/controlled/.build-artifacts/tmp/native-reopen-handover')
        original=reopen.session_marker(root,root.parent,'d'*32)
        self.assertNotIn('case',original)
        self.assertEqual(reopen.session_marker(root,root.parent,'d'*32,'editor-handover'),dict(original,case='editor-handover'))
        for bad in ['handover','../editor-handover','EditorHandover','editor-handover;anything']:
            with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,root.parent,'d'*32,bad)

    def test_premature_credit_changed_observations_missing_drafts_and_bad_revision_refuse(self):
        for revision in [True,16,'19',None]:
            with self.assertRaises(reopen.ReopenError):reopen.verify_editor_handover_replay(*self.fixture(revision))
        for key,value in [('evaluator_revision',18),('saved_record_verified',True),
                          ('current_assignment_credit',True),('drafts_preserved',False)]:
            initial,expected,replay=self.fixture();initial[key]=value
            with self.assertRaises(reopen.ReopenError):reopen.verify_editor_handover_replay(initial,expected,replay)
        for key,value in [('record_verified',False),('current_assignment_credit',1),('observed',{})]:
            initial,expected,replay=self.fixture();replay[key]=value
            with self.assertRaises(reopen.ReopenError):reopen.verify_editor_handover_replay(initial,expected,replay)
        for field,value in [('assignment_launch',None),('example_launch',{'editor':'vi'})]:
            initial,expected,replay=self.fixture();expected['progress']['attempts']['lpic1-editor-handover']['state']['simulation'][field]=value
            with self.assertRaises(reopen.ReopenError):reopen.verify_editor_handover_replay(initial,expected,replay)
        for invalid in [None,{}, {'progress':{}}]:
            initial,expected,replay=self.fixture()
            with self.assertRaises(reopen.ReopenError):reopen.verify_editor_handover_replay(initial,invalid,replay)

class SambaReopenScope(unittest.TestCase):
    def test_new_fixed_samba_case_keeps_legacy_marker_and_private_tool_boundaries(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-test');temporary=root.parent;nonce='c'*32
        marker=reopen.session_marker(root,temporary,nonce,'samba')
        self.assertEqual(marker['case'],'samba')
        self.assertNotIn('case',reopen.session_marker(root,temporary,nonce))
        with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,nonce,'arbitrary-samba-host')
        source=(Path(__file__).resolve().parents[2]/'src/ui/smoke/reopen/samba.rs').read_text()
        for text in ['assert_eq!(loaded, expected)','assert_eq!(simulation.assignment.editor, ASSIGNMENT)','assert_eq!(simulation.practice.editor, EXAMPLE)','simulation.assignment.last_trial.is_none()','repair(&mut p, &d.assignment, true)','self.samba_flow(ctx, plan(session.role == Role::Read))']:
            self.assertIn(text,source)

class VirtualIsolationReopenScope(unittest.TestCase):
    def test_fixed_two_kind_case_keeps_legacy_marker_and_private_boundaries(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-test'); temporary=root.parent; nonce='d'*32
        marker=reopen.session_marker(root,temporary,nonce,'virtual')
        self.assertEqual(marker['case'],'virtual')
        self.assertNotIn('case',reopen.session_marker(root,temporary,nonce))
        with self.assertRaises(reopen.ReopenError):
            reopen.session_marker(root,temporary,nonce,'arbitrary-virtual-host')
        source=(Path(__file__).resolve().parents[2]/'src/ui/smoke/reopen/virtual_isolation.rs').read_text()
        for text in ['assert_eq!(loaded, expected)','expected_partial(&d.assignment)','expected_partial(&d.example)','simulation.assignment.actual.is_none()','self.virtual_flow(ctx, plan(session.role == Role::Read))','assert!(!simulation.assignment.checked && !simulation.practice.checked)']:
            self.assertIn(text,source)
        self.assertIn('ccna-virtual-host-resilience',source)
        self.assertIn('ccna-vrf-overlap-repair',source)

class GuestDriverReopenScope(unittest.TestCase):
    def test_fixed_guest_case_preserves_legacy_wire_and_rejects_unknown_case(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-test');temporary=root.parent;nonce='e'*32
        self.assertEqual(reopen.session_marker(root,temporary,nonce,'guest-drivers')['case'],'guest-drivers')
        self.assertNotIn('case',reopen.session_marker(root,temporary,nonce))
        with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,nonce,'host-kernel')
        source=(Path(__file__).resolve().parents[2]/'src/ui/smoke/reopen/guest_driver.rs').read_text()
        for text in ['assert_eq!(loaded, expected)','verify_partial(self)','self.guest_flow(ctx, plan(session.role == Role::Read))','scope.runtime, partial(t, &method[0])']:
            self.assertIn(text,source)

class SmtpRoutingReopenScope(unittest.TestCase):
    def test_fixed_mail_case_preserves_legacy_wire_and_requires_raw_two_scope_readback(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-test');temporary=root.parent;nonce='f'*32
        self.assertEqual(reopen.session_marker(root,temporary,nonce,'smtp-routing')['case'],'smtp-routing')
        self.assertNotIn('case',reopen.session_marker(root,temporary,nonce))
        with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,nonce,'real-smtp')
        source=(Path(__file__).resolve().parents[2]/'src/ui/smoke/reopen/smtp.rs').read_text()
        for text in ['assert_eq!(loaded, expected)','verify_partial(self)','self.smtp_flow(ctx, plan(session.role == Role::Read))','scope.runtime, partial(t)','scope.drafts, drafts','writer-progress.raw','reader-loaded.json']:
            self.assertIn(text,source)

class SieveFiltersReopenScope(unittest.TestCase):
    def test_fixed_filter_case_preserves_legacy_wire_and_requires_actual_drafts_results_readback(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-test');temporary=root.parent;nonce='f'*32
        self.assertEqual(reopen.session_marker(root,temporary,nonce,'sieve-filters')['case'],'sieve-filters')
        self.assertNotIn('case',reopen.session_marker(root,temporary,nonce))
        with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,nonce,'real-sieve')
        source=(Path(__file__).resolve().parents[2]/'src/ui/smoke/reopen/sieve.rs').read_text()
        for text in ['assert_eq!(loaded, expected)','verify_partial(self)','self.sieve_flow(ctx, plan(session.role == Role::Read))','scope.draft, draft(t, practice)','scope.results, expected','writer-progress.raw','reader-loaded.json']:
            self.assertIn(text,source)

class AddressFilterColdGuards(unittest.TestCase):
    def test_address_case_is_closed_and_preserves_real_two_scope_results_before_repair(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-address-test');temporary=root.parent;nonce='a'*32
        self.assertEqual(reopen.session_marker(root,temporary,nonce,'sieve-addresses')['case'],'sieve-addresses')
        with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,nonce,'real-sieve-addresses')
        source=(Path(__file__).resolve().parents[2]/'src/ui/smoke/reopen/sieve_addresses.rs').read_text()
        for text in ['assert_eq!(loaded, expected)','verify_partial(self)','scope.draft, draft(t, practice)','scope.results, expected','writer-progress.raw','reader-loaded.json','t.parse(&scope.draft)']:
            self.assertIn(text,source)
        implementation, regression = source.split('#[test]', 1)
        self.assertNotIn('Op::Style',implementation)
        self.assertIn('assert!(!p.iter().any(|o| matches!(o, Op::Style(..))))', regression)

class DeliveryFilterColdGuards(unittest.TestCase):
    def test_delivery_case_is_closed_and_preserves_refusal_conflict_results_before_repairs(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-delivery-test');temporary=root.parent;nonce='d'*32
        self.assertEqual(reopen.session_marker(root,temporary,nonce,'sieve-delivery')['case'],'sieve-delivery')
        self.assertNotIn('case',reopen.session_marker(root,temporary,nonce))
        with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,nonce,'real-mail-delivery')
        source=(Path(__file__).resolve().parents[2]/'src/ui/smoke/reopen/sieve_delivery.rs').read_text()
        for text in ['assert_eq!(loaded, expected)','verify_partial(self)','scope.draft, draft(t, practice)','scope.results, expected','writer-progress.raw','reader-loaded.json','t.parse(&scope.draft)']:
            self.assertIn(text,source)
        implementation,regression=source.split('#[test]',1)
        self.assertNotIn('Op::Style',implementation)
        self.assertIn('assert!(!p.iter().any(|o| matches!(o, Op::Style(..))))',regression)

class VacationFilterColdGuards(unittest.TestCase):
    def test_vacation_case_is_closed_and_replays_real_partial_timeline_before_repair(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-vacation-test');temporary=root.parent;nonce='b'*32
        self.assertEqual(reopen.session_marker(root,temporary,nonce,'sieve-vacation')['case'],'sieve-vacation')
        with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,nonce,'real-vacation')
        source=(Path(__file__).resolve().parents[2]/'src/ui/smoke/reopen/sieve_vacation.rs').read_text()
        for text in ['assert_eq!(loaded, expected)','verify_partial(self)','scope.draft, draft(t, practice)','scope.results, expected','writer-progress.raw','reader-loaded.json','t.parse(&scope.draft)','last_sent: 0']:
            self.assertIn(text,source)
        implementation,regression=source.split('#[test]',1)
        self.assertNotIn('Op::Style',implementation)
        self.assertIn('assert!(!p.iter().any(|o| matches!(o, Op::Style(..))))',regression)

class MailboxColdGuards(unittest.TestCase):
    def test_mailbox_case_preserves_real_pending_transactions_and_raw_state(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-mailbox-test');temporary=root.parent;nonce='b'*32
        self.assertEqual(reopen.session_marker(root,temporary,nonce,'mailbox-access')['case'],'mailbox-access')
        with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,nonce,'host-mailbox')
        source=(Path(__file__).resolve().parents[2]/'src/ui/smoke/reopen/mailbox.rs').read_text()
        for text in ['assert_eq!(loaded, expected)','verify_partial(self)','scope.draft, draft(source, d, practice)','scope.runtime.pop_committed.is_empty()','scope.input, "QUIT"','writer-progress.raw','reader-loaded.json','scope.runtime.pop_committed[user].contains(&uid)']:
            self.assertIn(text,source)
        implementation,regression=source.split('#[test]',1)
        self.assertNotIn('Op::Style',implementation)
        self.assertIn('Op::ColdAppearance',regression)

class MailAdministrationColdGuards(unittest.TestCase):
    def test_administration_case_is_closed_and_checks_exact_partial_work_before_repair(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-admin-test')
        self.assertEqual(reopen.session_marker(root,root.parent,'b'*32,'mailbox-administration')['case'],'mailbox-administration')
        with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,root.parent,'b'*32,'host-administration')
        source=(Path(__file__).resolve().parents[2]/'src/ui/smoke/reopen/mailbox_admin.rs').read_text()
        for text in ['assert_eq!(loaded,expected)', 'assert_eq!(scope,&expected)', 'verify_partial(self)', 'writer-progress.raw', 'reader-loaded.json', 'Op::Click("run-admin".into())', 'assert!(progress::current_solved(a,source))']:
            self.assertIn(text,source)
        implementation,regression=source.split('#[test]',1)
        self.assertNotIn('Op::Style',implementation)
        self.assertIn('Op::ColdAppearance',regression)
        self.assertIn('Op::NotRun(practice)',implementation)

class LinuxRouterColdGuards(unittest.TestCase):
    def test_router_case_is_explicit_and_preserves_legacy_marker_shape(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-router-test')
        before=reopen.session_marker(root,root.parent,'b'*32)
        selected=reopen.session_marker(root,root.parent,'b'*32,'linux-router')
        self.assertEqual(selected.pop('case'),'linux-router')
        self.assertEqual(selected,before)
        for unknown in ['host-router','iptables','linux-router-host']:
            with self.assertRaises(reopen.ReopenError):
                reopen.session_marker(root,root.parent,'b'*32,unknown)

class FlowReopenMarkers(unittest.TestCase):
    def test_fixed_flow_case_keeps_legacy_marker_and_refuses_unknown_near_names(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-test')
        base=reopen.session_marker(root,root.parent,'c'*32,'script')
        selected=reopen.session_marker(root,root.parent,'c'*32,'linux-flow')
        self.assertEqual(selected.pop('case'),'linux-flow')
        self.assertEqual(selected,base)
        for bad in ['host-flow','linux-flow-host','linux-nat']:
            with self.assertRaises(reopen.ReopenError):
                reopen.session_marker(root,root.parent,'c'*32,bad)

class IPv6ReopenMarkers(unittest.TestCase):
    def test_fixed_ipv6_case_keeps_legacy_shape_and_rejects_similar_host_names(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-ipv6-test')
        base=reopen.session_marker(root,root.parent,'d'*32,'script')
        selected=reopen.session_marker(root,root.parent,'d'*32,'linux-v6')
        self.assertEqual(selected.pop('case'),'linux-v6')
        self.assertEqual(selected,base)
        for bad in ['host-ipv6','linux-v6-host','ip6tables','linux-ipv6']:
            with self.assertRaises(reopen.ReopenError):
                reopen.session_marker(root,root.parent,'d'*32,bad)

class ServiceRouterReopenMarkers(unittest.TestCase):
    def test_fixed_services_router_case_preserves_legacy_shape_and_rejects_host_aliases(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-services-test')
        base=reopen.session_marker(root,root.parent,'e'*32,'script')
        selected=reopen.session_marker(root,root.parent,'e'*32,'services-router')
        self.assertEqual(selected.pop('case'),'services-router')
        self.assertEqual(selected,base)
        for bad in ['service-router','linux-services-router','services-router-host','getent']:
            with self.assertRaises(reopen.ReopenError):
                reopen.session_marker(root,root.parent,'e'*32,bad)

class IPv6AdministrationReopenTests(unittest.TestCase):
    def test_new_case_is_explicit_and_legacy_marker_remains_unchanged(self):
        root=Path('/home/example/session')
        temporary=Path('/home/example/tmp')
        legacy=reopen.session_marker(root,temporary,'a'*32)
        self.assertNotIn('case',legacy)
        added=reopen.session_marker(root,temporary,'b'*32,'v6-startup')
        self.assertEqual(added['case'],'v6-startup')
        with self.assertRaises(reopen.ReopenError):
            reopen.session_marker(root,temporary,'b'*32,'v6-startup-arbitrary')
        current=reopen.session_marker(root,temporary,'a'*32,'v6-admin')
        self.assertEqual(current['case'],'v6-admin')
        self.assertEqual({k:v for k,v in current.items() if k!='case'},legacy)
        with self.assertRaises(reopen.ReopenError):
            reopen.session_marker(root,temporary,'a'*32,'v6-admin-arbitrary')

class FtpIdentityReopenTests(unittest.TestCase):
    def test_fixed_ftp_case_preserves_legacy_marker_and_rejects_selectors(self):
        root=Path('/home/example/session');temporary=Path('/home/example/tmp')
        legacy=reopen.session_marker(root,temporary,'c'*32)
        marker=reopen.session_marker(root,temporary,'c'*32,'ftp-identity')
        self.assertEqual(marker.pop('case'),'ftp-identity')
        self.assertEqual(marker,legacy)
        with patch.object(reopen.subprocess,'Popen')as child:
            for case in ['ftp','FTP','ftp-identity-arbitrary','ftp-identity /etc/vsftpd.conf']:
                with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,'c'*32,case)
            child.assert_not_called()


class ServiceExposureReopenTests(unittest.TestCase):
    def test_fixed_exposure_case_preserves_legacy_marker_and_refuses_arbitrary_selectors(self):
        root=Path('/home/example/session');temporary=Path('/home/example/tmp')
        legacy=reopen.session_marker(root,temporary,'d'*32)
        marker=reopen.session_marker(root,temporary,'d'*32,'service-exposure')
        self.assertEqual(marker.pop('case'),'service-exposure')
        self.assertEqual(marker,legacy)
        with patch.object(reopen.subprocess,'Popen')as child:
            for case in ['exposure','ServiceExposure','service-exposure-arbitrary','service-exposure 192.0.2.1']:
                with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,'d'*32,case)
            child.assert_not_called()

class VpnPointReopenTests(unittest.TestCase):
    def test_fixed_vpn_case_preserves_legacy_marker_and_refuses_arbitrary_selectors(self):
        root=Path('/home/example/session');temporary=Path('/home/example/tmp')
        legacy=reopen.session_marker(root,temporary,'e'*32)
        marker=reopen.session_marker(root,temporary,'e'*32,'vpn-point')
        self.assertEqual(marker.pop('case'),'vpn-point')
        self.assertEqual(marker,legacy)
        with patch.object(reopen.subprocess,'Popen')as child:
            for case in ['vpn','VPN','vpn-point-arbitrary','vpn-point /etc/openvpn/client.conf']:
                with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,'e'*32,case)
            child.assert_not_called()


class RpmPatchReopenTests(unittest.TestCase):
    def test_fixed_patch_case_keeps_legacy_marker_and_refuses_arbitrary_selectors(self):
        root=Path('/home/example/session');temporary=Path('/home/example/tmp')
        legacy=reopen.session_marker(root,temporary,'f'*32)
        marker=reopen.session_marker(root,temporary,'f'*32,'rpm-patch')
        self.assertEqual(marker.pop('case'),'rpm-patch')
        self.assertEqual(marker,legacy)
        with patch.object(reopen.subprocess,'Popen') as child:
            for case in ['rpm','RPM','rpm-patch-arbitrary','rpm-patch /etc/passwd']:
                with self.assertRaises(reopen.ReopenError):reopen.session_marker(root,temporary,'f'*32,case)
            child.assert_not_called()


class AuthFilterReopenTests(unittest.TestCase):
    def test_fixed_auth_filter_case_is_separate_and_rejects_path_or_command_selectors(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-auth')
        temporary=root.parent
        legacy=reopen.session_marker(root,temporary,'a'*32)
        marker=reopen.session_marker(root,temporary,'a'*32,'auth-filter')
        self.assertEqual(marker.pop('case'),'auth-filter')
        self.assertEqual(marker,legacy)
        for case in ['auth-filter /etc/passwd','auth-filter;cat','auth-filter-arbitrary','fail2ban']:
            with self.subTest(case=case):
                with self.assertRaises(reopen.ReopenError):
                    reopen.session_marker(root,temporary,'a'*32,case)

class IdsRuleReopenTests(unittest.TestCase):
    def test_fixed_ids_rule_case_is_separate_and_rejects_path_or_command_selectors(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-ids')
        temporary=root.parent
        legacy=reopen.session_marker(root,temporary,'a'*32)
        marker=reopen.session_marker(root,temporary,'a'*32,'ids-rule')
        self.assertEqual(marker.pop('case'),'ids-rule')
        self.assertEqual(marker,legacy)
        for case in ['ids-rule /etc/passwd','ids-rule;cat','ids-rule-arbitrary','snort']:
            with self.subTest(case=case):
                with self.assertRaises(reopen.ReopenError):
                    reopen.session_marker(root,temporary,'a'*32,case)
