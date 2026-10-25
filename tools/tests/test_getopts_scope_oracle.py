"""Fixed5 collector guard tests; UNRUN at source handoff.

No test starts a process, Bash, VM, version probe or producer. Any fake actual
rows are ephemeral validator inputs, never retained authoritative observations.
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
import getopts_scope_oracle as oracle


@unittest.skipUnless(sys.platform.startswith('linux'), 'fixed Linux developer oracle')
class ScopeOracle(unittest.TestCase):
    def test_exact_five_decoded_sources_and_null_observations(self):
        raw, rows = oracle.load_fixed(ROOT)
        self.assertEqual(hashlib.sha256(raw).hexdigest(), oracle.FIXED_SHA)
        self.assertEqual(tuple(r['id'] for r in rows), oracle.IDS)
        for row in rows:
            self.assertNotIn('\\"', row['source'])
            self.assertNotIn('\\\\n', row['source'])
            self.assertIn('\\n', row['source'])
            self.assertIsNone(row['actual_bash'])
            self.assertIsNone(row['expected'])

    def test_changed_fixture_hash_rejected_before_json_or_child(self):
        raw = oracle.safe.read_bytes(ROOT / oracle.FIXED, 16384) + b' '
        with patch.object(oracle.safe, 'read_bytes', return_value=raw), \
             patch.object(oracle, 'strict_json', side_effect=AssertionError('must not parse')), \
             patch.object(oracle.safe, 'capture_command') as child:
            with self.assertRaisesRegex(oracle.safe.OracleError, 'fixed5'):
                oracle.load_fixed(ROOT)
            child.assert_not_called()

    def test_closed_fixture_rejects_input_source_shape_and_fake_observation(self):
        original = json.loads(oracle.safe.read_bytes(ROOT / oracle.FIXED, 16384))
        for change in ('stdin', 'files', 'args', 'env', 'actual_bash', 'escaped_source', 'extra_key'):
            value = json.loads(json.dumps(original))
            row = value['cases'][0]
            if change == 'stdin': row['input']['stdin'] = 'not empty'
            elif change == 'files': row['input']['files']['/work/extra'] = 'unrequested'
            elif change == 'args': row['input']['args'] = ['unrequested']
            elif change == 'env': row['input']['env'] = {'BASH_ENV': '/never-read'}
            elif change == 'actual_bash': row['actual_bash'] = {'stdout': 'invented'}
            elif change == 'escaped_source': row['source'] = row['source'].replace('"$rc"', '\\"$rc\\"')
            else: row['script_path'] = '/never-read'
            raw = json.dumps(value).encode()
            # Isolate secondary structural guards; production SHA is never overridden.
            with patch.object(oracle, 'FIXED_SHA', hashlib.sha256(raw).hexdigest()), \
                 patch.object(oracle.safe, 'read_bytes', return_value=raw), \
                 patch.object(oracle.safe, 'capture_command') as child:
                with self.assertRaises(oracle.safe.OracleError, msg=change): oracle.load_fixed(ROOT)
                child.assert_not_called()

    def test_no_path_binary_profile_source_or_ambiguous_cli(self):
        with patch.object(oracle, 'fixed_root_guard') as guard:
            for flags in (['--source', 'printf bad'], ['--binary', '/not-read'],
                          ['--profile', '/not-read'], ['--output', '/not-written'],
                          ['--capture', 'x'], ['--prepare-id', 'x', '--capture-id', 'y'],
                          ['--capture-id', '../escape'], ['--prepare-id', 'x/y']):
                with self.assertRaises((SystemExit, oracle.safe.OracleError)):
                    oracle.main(flags)
            guard.assert_not_called()

    def test_missing_guard_prevents_metadata_and_children(self):
        with patch.object(oracle, 'fixed_root_guard', side_effect=oracle.safe.OracleError('no guard')), \
             patch.object(oracle, 'prepare') as prepare, patch.object(oracle, 'capture') as capture, \
             patch.object(oracle.safe, 'capture_command') as child:
            for flag in ('--prepare-id', '--capture-id'):
                with self.assertRaises(oracle.safe.OracleError): oracle.main([flag, 'unit-only'])
            prepare.assert_not_called(); capture.assert_not_called(); child.assert_not_called()

    def test_worktree_and_nondefault_target_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / '.git').write_text('gitdir: elsewhere')
            with patch.object(oracle, 'ROOT', root), patch.object(oracle.safe, 'require_guard') as guard:
                with self.assertRaisesRegex(oracle.safe.OracleError, 'MAIN'): oracle.fixed_root_guard(root)
                guard.assert_not_called()
            (root / '.git').unlink(); (root / '.git').mkdir()
            with patch.object(oracle, 'ROOT', root), patch.object(oracle.safe, 'require_guard', return_value=(4,)), \
                 patch.dict(os.environ, {'CARGO_TARGET_DIR': str(root / 'other')}):
                with self.assertRaisesRegex(oracle.safe.OracleError, 'default'): oracle.fixed_root_guard(root)

    def test_actual_pin_total_rechecked_after_size_growth(self):
        @contextmanager
        def small_preflight(*_): yield None, SimpleNamespace(st_size=2)
        paths = [(Path('/fixed/one'), 8), (Path('/fixed/two'), 8)]
        with patch.object(oracle, 'PIN_TOTAL_CAP', 10), patch.object(oracle.safe, 'ordinary', small_preflight):
            with patch.object(oracle.safe, 'pin', side_effect=[{'bytes': 6}, {'bytes': 6}]):
                with self.assertRaisesRegex(oracle.safe.OracleError, 'Actual aggregate'): oracle.pins(paths)
            with patch.object(oracle.safe, 'pin', side_effect=[{'bytes': 4}, {'bytes': 6}]):
                self.assertEqual(oracle.pins(paths), [{'bytes': 4}, {'bytes': 6}])

    def test_prepare_only_metadata_is_false_reviewed_and_noclobber(self):
        raw, rows = oracle.load_fixed(ROOT)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            with patch.object(oracle, 'load_fixed', return_value=(raw, rows)), \
                 patch.object(oracle, 'review_dir', return_value=root), \
                 patch.object(oracle, 'checked_pins', return_value=[]), \
                 patch.object(oracle.safe, 'source_identity', return_value='a' * 40), \
                 patch.object(oracle.safe, 'capture_command') as child:
                result = oracle.prepare(root, 'unit-only')
                value = json.loads((root / 'PROPOSED-unit-only.json').read_bytes())
                self.assertFalse(value['root_reviewed'])
                self.assertEqual(result['status'], 'PROPOSED-ONLY')
                with self.assertRaises(FileExistsError): oracle.prepare(root, 'unit-only')
                child.assert_not_called()

    def test_approval_requires_reviewed_actual_id_head_and_exact_pins(self):
        good = {'schema_version': 1, 'root_reviewed': True, 'capture_id': 'unit-only',
                'source_head': 'a' * 40, 'pins': [], 'claims': 'unit-validator-input'}
        with patch.object(oracle, 'review_dir', return_value=Path('/fixed')), \
             patch.object(oracle.safe, 'source_identity', return_value='a' * 40), \
             patch.object(oracle, 'checked_pins', return_value=[]), \
             patch.object(oracle.safe, 'capture_command') as child:
            for key, value in [('root_reviewed', False), ('capture_id', 'other'),
                               ('source_head', 'b' * 40), ('pins', [{'changed': True}]), ('extra', True)]:
                with patch.object(oracle.safe, 'read_bytes', return_value=json.dumps({**good, key: value}).encode()):
                    with self.assertRaises(oracle.safe.OracleError): oracle.approval(Path('/fixed'), 'unit-only')
            child.assert_not_called()

    @contextmanager
    def fake_capture(self, *, banner='GNU bash, version 5.3.9 (FAKE UNIT INPUT ONLY)\n', quiet=None):
        # All files belong only to this test's temporary fixture and are removed
        # by its lifetime guard. No oracle evidence, process or ELF is created.
        raw, rows = oracle.load_fixed(ROOT)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            executable = root / 'fake-elf'
            executable.write_bytes(b'not-executed')
            before = [oracle.safe.pin(executable, 64)]
            control = {'source_head': 'a' * 40, 'pins': before}
            calls = []
            @contextmanager
            def pinned(*_): yield '/proc/self/fd/999', 999
            def child(command, folder, work, env, data, budget, stream_cap, **kwargs):
                calls.append((command, env, data, kwargs))
                text = banner if '--version' in command else 'FAKE UNIT INPUT ONLY\n'
                oracle.safe.write_new(folder / 'stdout.raw', text.encode(), budget, stream_cap)
                oracle.safe.write_new(folder / 'stderr.raw', b'', budget, stream_cap)
                return {'stdout': text, 'stderr': '', 'status': 0}
            with patch.object(oracle, 'approval', return_value=(b'{}\n', control, before)), \
                 patch.object(oracle, 'load_fixed', return_value=(raw, rows)), \
                 patch.object(oracle, 'review_dir', return_value=root), \
                 patch.object(oracle.safe, 'BASH', executable), \
                 patch.object(oracle, 'pinned_executable', pinned), \
                 patch.object(oracle.safe, 'capture_command', side_effect=child), \
                 patch.object(oracle.safe, 'quiet_group', side_effect=quiet), \
                 patch.object(oracle.safe, 'source_identity', return_value='a' * 40), \
                 patch.object(oracle, 'checked_pins', return_value=before):
                yield root, rows, calls, before

    def test_capture_uses_only_fixed_sources_empty_stdin_closed_env_and_fd(self):
        with self.fake_capture() as (root, rows, calls, _):
            result = oracle.capture(root, 'unit-only', (7,))
            self.assertEqual(result['status'], 'COMMITTED')
            out = Path(result['directory'])
            manifest = json.loads((out / 'MANIFEST.json').read_bytes())
            self.assertEqual(manifest['case_rows'], 5)
            self.assertEqual((out / 'COMPLETE').read_text().strip(), oracle.safe.pin(out / 'MANIFEST.json', oracle.ROW_CAP)['sha256'])
            self.assertEqual(len(calls), 6)
            for index, (command, env, data, kwargs) in enumerate(calls):
                self.assertEqual(command[0], '/proc/self/fd/999')
                self.assertEqual(kwargs['inherited'], (7, 999))
                self.assertLessEqual(kwargs['timeout'], 2)
                self.assertEqual(data, b'')
                self.assertNotIn('HOME', env); self.assertEqual(env['PATH'], '')
                self.assertEqual(env['LC_ALL'], 'C')
                if index:
                    self.assertEqual(command[-2], rows[index - 1]['source'])
                    observed = json.loads((out / f'fixed-{index - 1:03}' / 'OBSERVATION.json').read_bytes())
                    self.assertNotIn('actual_public8', observed)
                    self.assertIsNone(observed['future_vm_expectation'])

    def test_wrong_version_retains_partial_without_case_or_complete(self):
        with self.fake_capture(banner='GNU bash, version 5.2 (FAKE UNIT INPUT ONLY)\n') as (root, _, calls, _):
            with self.assertRaisesRegex(oracle.safe.OracleError, 'Bash5.3.9'): oracle.capture(root, 'unit-only', ())
            self.assertEqual(len(calls), 1)
            self.assertTrue((root / 'captures/unit-only/STARTED.json').exists())
            self.assertFalse((root / 'captures/unit-only/COMPLETE').exists())

    def test_uncertain_descendant_aborts_before_next_case_or_complete(self):
        with self.fake_capture(quiet=oracle.safe.OracleError('owned descendant')) as (root, _, calls, _):
            with self.assertRaisesRegex(oracle.safe.OracleError, 'descendant'): oracle.capture(root, 'unit-only', ())
            self.assertEqual(len(calls), 1)
            self.assertFalse((root / 'captures/unit-only/COMPLETE').exists())

    def test_changed_final_source_pins_and_output_block_completion(self):
        for change in ('pins', 'output'):
            with self.fake_capture() as (root, _, _, before):
                def changed(actual_root):
                    self.assertEqual(actual_root, root)
                    if change == 'pins': return []
                    (root / 'captures/unit-only/INPUTS.json').write_bytes(b'changed-unit-input')
                    return before
                with patch.object(oracle, 'checked_pins', side_effect=changed):
                    with self.assertRaises(oracle.safe.OracleError, msg=change): oracle.capture(root, 'unit-only', ())
                self.assertFalse((root / 'captures/unit-only/COMPLETE').exists())

    def test_deadline_after_presync_leaves_no_complete(self):
        with tempfile.TemporaryDirectory() as temporary:
            out = Path(temporary).resolve() / 'out'; out.mkdir()
            with patch.object(oracle, 'remaining', side_effect=oracle.safe.OracleError('deadline')):
                with self.assertRaisesRegex(oracle.safe.OracleError, 'deadline'):
                    oracle.publish(out, {'unit-validator': True}, oracle.safe.Budget(), 0)
            self.assertTrue((out / 'COMPLETION-PREPARED').exists())
            self.assertFalse((out / 'COMPLETE').exists())

    def test_committed_warning_counts_complete_and_existing_id_never_clobbered(self):
        with tempfile.TemporaryDirectory() as temporary:
            out = Path(temporary).resolve() / 'out'; out.mkdir()
            with patch.object(oracle, 'remaining', return_value=1), \
                 patch.object(oracle.safe, 'sync_dir', side_effect=[None, None, OSError('unit-sync')]):
                observed, warning = oracle.publish(out, {'unit-validator': True}, oracle.safe.Budget(), 0)
            old = (out / 'COMPLETE').read_bytes()
            self.assertEqual(old.strip().decode(), observed['sha256'])
            self.assertIn('COMMITTED', warning)
            with self.assertRaises(FileExistsError): oracle.publish(out, {}, oracle.safe.Budget(), 0)
            self.assertEqual((out / 'COMPLETE').read_bytes(), old)


if __name__ == '__main__':
    unittest.main()
