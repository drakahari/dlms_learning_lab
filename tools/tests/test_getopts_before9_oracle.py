"""Safety tests for fixed getopts collector. Never execute Bash, a VM or producer.

All fake output rows below are ephemeral validator inputs, not observed evidence.
Source/progress fixtures remain immutable; no installed learner path is accessed.
"""
from contextlib import contextmanager
from types import SimpleNamespace
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import getopts_before9_oracle as oracle


@unittest.skipUnless(sys.platform.startswith('linux'), 'fixed Linux developer oracle')
class FixedGetoptsSafety(unittest.TestCase):
    def test_actual_pin_aggregate_rechecked_after_preflight_size_growth(self):
        @contextmanager
        def small_first_pass(*_):
            yield None, SimpleNamespace(st_size=2)
        paths = [(Path('/fixed/one'), 8), (Path('/fixed/two'), 8)]
        with patch.object(oracle, 'TOTAL_PIN_CAP', 10), patch.object(oracle.safe, 'ordinary', small_first_pass):
            with patch.object(oracle.safe, 'pin', side_effect=[{'bytes': 6}, {'bytes': 6}]) as actual:
                with self.assertRaisesRegex(oracle.safe.OracleError, 'Actual aggregate'):
                    oracle.pins(paths)
                self.assertEqual(actual.call_count, 2)
            with patch.object(oracle.safe, 'pin', side_effect=[{'bytes': 4}, {'bytes': 6}]):
                self.assertEqual(oracle.pins(paths), [{'bytes': 4}, {'bytes': 6}])

    def test_fixed50_inputs_are_unobserved_and_original37_unchanged(self):
        _, rows = oracle.load_fixed(ROOT)
        self.assertEqual(len(rows), 50)
        self.assertEqual(hashlib.sha256(oracle.safe.read_bytes(ROOT / oracle.safe.FIXTURE, oracle.SOURCE_CAP)).hexdigest(), oracle.safe.FIXTURE_SHA)
        self.assertEqual(sum(x['group'] == 'prior-valid8' for x in rows), 8)
        self.assertTrue(all(x['actual_evaluator8_before9'] is None for x in rows))

    def test_changed_fixed_source_rejected_before_environment_or_child(self):
        data = oracle.safe.read_bytes(ROOT / oracle.FIXED, oracle.SOURCE_CAP) + b' '
        with patch.object(oracle.safe, 'read_bytes', return_value=data), patch.object(oracle.safe, 'capture_command') as child:
            with self.assertRaisesRegex(oracle.safe.OracleError, 'fixed50'): oracle.load_fixed(ROOT)
            child.assert_not_called()

    def test_duplicate_json_keys_are_rejected(self):
        with self.assertRaisesRegex(oracle.safe.OracleError, 'Duplicate'): oracle.strict_json(b'{"a":1,"a":2}')

    def test_capture_id_cannot_escape_or_accept_source(self):
        for name in ('../x', '/x', 'x/y', '', 'x' * 65, 'x;printf'):
            with self.assertRaises(oracle.safe.OracleError): oracle.capture_id(name)
        self.assertEqual(oracle.capture_id('before9-fixed-r1'), 'before9-fixed-r1')

    def test_parser_rejects_arbitrary_source_profile_binary_and_abbreviation(self):
        with patch.object(oracle, 'fixed_root_guard') as guard:
            for flags in (['--source', 'printf x'], ['--binary', '/bin/x'], ['--profile', '/tmp/x'], ['--capture', 'x'], ['--prepare-id', 'x', '--capture-id', 'y']):
                with self.assertRaises(SystemExit): oracle.main(flags)
            guard.assert_not_called()

    def test_missing_guard_prevents_preparation_capture_and_subprocesses(self):
        with patch.object(oracle, 'fixed_root_guard', side_effect=oracle.safe.OracleError('no guard')), patch.object(oracle, 'capture') as capture, patch.object(oracle, 'prepare') as prepare, patch.object(oracle.safe, 'capture_command') as child:
            with self.assertRaises(oracle.safe.OracleError): oracle.main(['--capture-id', 'x'])
            with self.assertRaises(oracle.safe.OracleError): oracle.main(['--prepare-id', 'x'])
            capture.assert_not_called(); prepare.assert_not_called(); child.assert_not_called()

    def test_worktree_and_target_override_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); (root / '.git').write_text('gitdir: other')
            with patch.object(oracle, 'ROOT', root), patch.object(oracle.safe, 'require_guard') as guard:
                with self.assertRaises(oracle.safe.OracleError): oracle.fixed_root_guard(root)
                guard.assert_not_called()
            (root / '.git').unlink(); (root / '.git').mkdir()
            with patch.object(oracle, 'ROOT', root), patch.object(oracle.safe, 'require_guard', return_value=(4,)), patch.dict(os.environ, {'CARGO_TARGET_DIR': str(root / 'other')}):
                with self.assertRaisesRegex(oracle.safe.OracleError, 'default target'): oracle.fixed_root_guard(root)

    def test_control_must_be_explicitly_reviewed_and_match_id_head_pins(self):
        control = {'schema_version': 1, 'root_reviewed': True, 'capture_id': 'x', 'source_head': 'a' * 40, 'pins': [], 'claims': 'metadata'}
        with patch.object(oracle, 'review_dir', return_value=Path('/fixed')), patch.object(oracle.safe, 'source_identity', return_value='a' * 40), patch.object(oracle, 'pins', return_value=[]):
            for key, val in [('root_reviewed', False), ('capture_id', 'y'), ('source_head', 'b' * 40), ('pins', [{'fake': True}])]:
                invalid = {**control, key: val}
                with patch.object(oracle.safe, 'read_bytes', return_value=json.dumps(invalid).encode()):
                    with self.assertRaises(oracle.safe.OracleError): oracle.approval(Path('/fixed'), 'x', [])
            with patch.object(oracle.safe, 'read_bytes', return_value=json.dumps(control).encode()):
                self.assertEqual(oracle.approval(Path('/fixed'), 'x', [])[1], control)

    def test_pinned_executable_reuses_same_fd_and_rejects_changed_pin(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / 'not-executed'; p.write_bytes(b'\x7fELFfixed-source-test')
            pin = oracle.safe.pin(p, 64)
            with oracle.pinned_executable(p, pin, 64) as (name, fd):
                self.assertEqual(name, '/proc/self/fd/' + str(fd))
                self.assertEqual(os.read(fd, 4), b'\x7fELF')
            with self.assertRaises(oracle.safe.OracleError):
                with oracle.pinned_executable(p, {**pin, 'sha256': '0' * 64}, 64): pass

    def test_executable_fifo_symlink_and_size_cannot_be_opened_as_program(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / 'file'; p.write_bytes(b'1234')
            link = Path(tmp) / 'alias'; link.symlink_to(p)
            for target, cap in ((link, 64), (p, 2)):
                with self.assertRaises((oracle.safe.OracleError, OSError)):
                    with oracle.pinned_executable(target, {}, cap): pass
            fifo = Path(tmp) / 'fifo'; os.mkfifo(fifo)
            with self.assertRaises(oracle.safe.OracleError):
                with oracle.pinned_executable(fifo, {}, 64): pass

    def test_vm_environment_omits_home_and_unrelated_host_variables(self):
        with patch.dict(os.environ, {'HOME': '/private', 'BASH_ENV': '/private/start', 'UNRELATED': 'secret'}):
            e = oracle.vm_environment(Path('/project'), Path('/temporary'), Path('/profile'))
        self.assertNotIn('HOME', e); self.assertNotIn('BASH_ENV', e); self.assertNotIn('UNRELATED', e)
        self.assertEqual(e['PATH'], ''); self.assertEqual(e['CARGO_TARGET_DIR'], '/project/target')

    def test_compiled_identity_rejects_stale_library_probe_or_catalog_bytes(self):
        data = {'kind': 'identity', 'revision': 8, 'compiled_manifest_dir': '/project',
                'compiled_inputs': [{'file': 'src/shell.rs', 'bytes': 1, 'sha256': 'a'}],
                'linked_catalog': {'fixed': 1}, 'linked_scripts': [],
                'probe_source_sha256': 'b', 'digest_source_sha256': 'c', 'attestation_source_sha256': 'd'}
        pins = [{'path': '/project/' + n, 'bytes': 1, 'sha256': h} for n, h in [('src/shell.rs', 'a'), (oracle.SOURCE, 'b'), (oracle.DIGEST, 'c'), (oracle.ATTEST, 'd')]]
        with patch.object(oracle, 'expected_inputs', return_value={'compiled_files': ['src/shell.rs']}), patch.object(oracle, 'authored_inputs', return_value=(b'', {'catalog': {'fixed': 1}, 'scripts': []})):
            self.assertEqual(oracle.identity_rows(Path('/project'), json.dumps(data), pins), data)
            for key, val in [('revision', 9), ('compiled_manifest_dir', '/other'), ('compiled_inputs', []), ('linked_catalog', {}), ('probe_source_sha256', 'z')]:
                with self.assertRaises(oracle.safe.OracleError): oracle.identity_rows(Path('/project'), json.dumps({**data, key: val}), pins)

    def test_vm_truncated_missing_or_duplicate_rows_cannot_finish(self):
        for text in ('', '{}', '{}\n{}', '{"kind":"identity"}\n{"kind":"finished"}'):
            with self.assertRaises((oracle.safe.OracleError, KeyError, ValueError)):
                oracle.verify_vm(ROOT, text, {}, {'scripts': []}, [])

    def test_profile_symlink_and_oversize_rejected_before_read(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); external = root / 'external'; external.write_bytes(b'x')
            folder = root / 'profile'; folder.mkdir(); (folder / 'link').symlink_to(external)
            with self.assertRaises(oracle.safe.OracleError): oracle.owned_outputs(folder, oracle.safe.Budget())
            (folder / 'link').unlink(); large = folder / 'too-large'
            with large.open('wb') as stream: stream.truncate(oracle.PROFILE_CAP + 1)
            with self.assertRaises(oracle.safe.OracleError): oracle.owned_outputs(folder, oracle.safe.Budget())

    def test_publication_never_clobbers_complete_and_precommit_failure_has_no_complete(self):
        with tempfile.TemporaryDirectory() as tmp:
            parent = Path(tmp); out = parent / 'result'; out.mkdir(); (out / 'COMPLETE').write_bytes(b'retained')
            with self.assertRaises(FileExistsError): oracle.safe.publish(out, {'test_only': True}, oracle.safe.Budget())
            self.assertEqual((out / 'COMPLETE').read_bytes(), b'retained')
            fail = parent / 'failure'; fail.mkdir()
            with patch.object(oracle.safe, 'sync_dir', side_effect=OSError('disk failure')):
                with self.assertRaises(OSError): oracle.safe.publish(fail, {}, oracle.safe.Budget())
            self.assertFalse((fail / 'COMPLETE').exists())

    def test_metadata_prepare_has_no_child_launch_and_requires_fresh_proposal(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            before = [{'path': str(oracle.safe.MANUAL), 'bytes': 1, 'sha256': oracle.safe.MANUAL_SHA}]
            with patch.object(oracle, 'load_fixed'), patch.object(oracle, 'source_set', return_value=[]), patch.object(oracle, 'pins', return_value=before), patch.object(oracle, 'review_dir', return_value=p), patch.object(oracle.safe, 'source_identity', return_value='a' * 40), patch.object(oracle.safe, 'capture_command') as child:
                result = oracle.prepare(p, 'fixed')
                self.assertEqual(result['status'], 'PROPOSED-ONLY')
                self.assertFalse(json.loads((p / 'PROPOSED-fixed.json').read_bytes())['root_reviewed'])
                with self.assertRaises(FileExistsError): oracle.prepare(p, 'fixed')
                child.assert_not_called()


if __name__ == '__main__': unittest.main()
