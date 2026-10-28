"""Closed developer oracle guards. No process or Bash is executed by these tests."""
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
import parameter_bash_oracle as oracle


@unittest.skipUnless(sys.platform.startswith('linux'), 'fixed Fedora developer reference')
class ParameterOracleGuards(unittest.TestCase):
    def test_exact_59_fixed_sources_are_inputs_not_observations(self):
        rows, pins = oracle.load_fixed(ROOT)
        self.assertEqual(len(rows), 59)
        self.assertEqual(len(pins), 4)
        self.assertEqual(len({r['name'] for r in rows}), 59)
        self.assertTrue(all(set(r) == {'name', 'source', 'case'} for r in rows))
        self.assertIn('cat -- --\n', [r['source'] for r in rows])
        for p in pins:
            self.assertEqual(hashlib.sha256((ROOT / p['file']).read_bytes()).hexdigest(), p['sha256'])

    def test_changed_source_fails_before_parse_or_child(self):
        raw = (ROOT / next(iter(oracle.FIXED))).read_bytes() + b' '
        with patch.object(oracle.safe, 'read_bytes', return_value=raw), \
             patch.object(oracle, 'strict', side_effect=AssertionError('No parsing')), \
             patch.object(oracle.safe, 'capture_command') as child:
            with self.assertRaisesRegex(oracle.safe.OracleError, 'inputs changed'):
                oracle.load_fixed(ROOT)
            child.assert_not_called()

    def test_duplicate_keys_and_paths_outside_flat_work_namespace_fail(self):
        with self.assertRaises(oracle.safe.OracleError):
            oracle.strict('{"source":"a","source":"b"}')
        for p in ['/etc/passwd', '/work/../file', '/work/a/b', '/work/.', '/work/', '/work/..']:
            with self.subTest(path=p), self.assertRaises(oracle.safe.OracleError):
                oracle.flat_relative(p)
        self.assertEqual(oracle.flat_relative('/work/- shift notes.txt'), Path('- shift notes.txt'))

    def test_cli_cannot_supply_program_binary_profile_path_or_abbreviation(self):
        with patch.object(oracle.safe, 'require_guard') as guard:
            for flags in [['--source', 'echo bad'], ['--binary', '/bin/bash'], ['--profile', '/private'],
                          ['--capture', 'x'], ['--capture-id', '../escape'],
                          ['--capture-id', 'x', '--output', '/tmp/x']]:
                with self.subTest(flags=flags), self.assertRaises((SystemExit, oracle.safe.OracleError)):
                    oracle.main(flags)
            guard.assert_not_called()

    def setup_control(self, root):
        (root / '.git').mkdir()
        base = root / oracle.HERE
        base.mkdir(parents=True)
        return base

    def test_prepare_is_metadata_only_exclusive_and_never_self_approves(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            base = self.setup_control(root)
            with patch.object(oracle, 'ROOT', root), \
                 patch.object(oracle.safe, 'require_guard', return_value=(7,)), \
                 patch.dict(os.environ, {'CARGO_TARGET_DIR': str(root / 'target')}), \
                 patch.object(oracle, 'load_fixed', return_value=([], [])), \
                 patch.object(oracle, 'load_amendment', return_value={'identity': {'unit_test': True}}), \
                 patch.object(oracle, 'source_set', return_value=[]), \
                 patch.object(oracle, 'pins', return_value=[{'path': str(oracle.safe.MANUAL.resolve()),
                                                          'sha256': oracle.safe.MANUAL_SHA}]), \
                 patch.object(oracle.safe, 'source_identity', return_value='a' * 40), \
                 patch.object(oracle.safe, 'capture_command') as child:
                oracle.main(['--capture-id', 'unit-only', '--prepare'])
                data = json.loads((base / 'PROPOSED-unit-only.json').read_bytes())
                self.assertFalse(data['root_reviewed'])
                self.assertFalse((base / 'APPROVED-unit-only.json').exists())
                with self.assertRaises(FileExistsError):
                    oracle.main(['--capture-id', 'unit-only', '--prepare'])
                child.assert_not_called()

    def test_approval_requires_exact_review_head_inputs_and_pins_before_child(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            base = self.setup_control(root)
            pins = [{'path': str(oracle.safe.MANUAL.resolve()), 'sha256': oracle.safe.MANUAL_SHA}]
            good = {'schema_version': 1, 'root_reviewed': True, 'capture_id': 'unit-only',
                    'source_head': 'a' * 40, 'pins': pins, 'inputs': [], 'amendment': {'unit_test': True},
                    'claims': 'Bash-only fixed observations; no VM or learner source accepted'}
            with patch.object(oracle, 'ROOT', root), \
                 patch.object(oracle.safe, 'require_guard', return_value=(7,)), \
                 patch.dict(os.environ, {'CARGO_TARGET_DIR': str(root / 'target')}), \
                 patch.object(oracle, 'load_fixed', return_value=([], [])), \
                 patch.object(oracle, 'load_amendment', return_value={'identity': {'unit_test': True}}), \
                 patch.object(oracle, 'source_set', return_value=[]), \
                 patch.object(oracle, 'pins', return_value=pins), \
                 patch.object(oracle.safe, 'source_identity', return_value='a' * 40), \
                 patch.object(oracle.safe, 'capture_command') as child:
                for key, value in [('root_reviewed', False), ('capture_id', 'other'),
                                   ('source_head', 'b' * 40), ('pins', []), ('inputs', ['other']),
                                   ('extra', True)]:
                    (base / 'APPROVED-unit-only.json').write_text(json.dumps({**good, key: value}))
                    with self.subTest(key=key), self.assertRaises(oracle.safe.OracleError):
                        oracle.main(['--capture-id', 'unit-only'])
                child.assert_not_called()
                self.assertFalse((base / 'unit-only').exists())

    def test_foreign_target_or_missing_guard_refuses_before_fixtures(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            self.setup_control(root)
            with patch.object(oracle, 'ROOT', root), \
                 patch.object(oracle.safe, 'require_guard', return_value=(7,)), \
                 patch.dict(os.environ, {'CARGO_TARGET_DIR': str(root / 'other')}), \
                 patch.object(oracle, 'load_fixed') as load:
                with self.assertRaises(oracle.safe.OracleError):
                    oracle.main(['--capture-id', 'unit-only', '--prepare'])
                load.assert_not_called()
            with patch.object(oracle, 'ROOT', root), \
                 patch.object(oracle.safe, 'require_guard', side_effect=oracle.safe.OracleError('No lock')), \
                 patch.object(oracle, 'load_fixed') as load:
                with self.assertRaisesRegex(oracle.safe.OracleError, 'No lock'):
                    oracle.main(['--capture-id', 'unit-only', '--prepare'])
                load.assert_not_called()

    def test_final_files_refuse_symlink_and_unreviewed_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'known').write_text('retained')
            self.assertEqual(oracle.final_files(root, {'known'})[0], {'/work/known': 'retained'})
            (root / 'alias').symlink_to(root / 'known')
            with self.assertRaises(oracle.safe.OracleError): oracle.final_files(root, {'known', 'alias'})
            with self.assertRaises(oracle.safe.OracleError): oracle.final_files(root, {'known'})


    def fixed_policy(self):
        # Unit expectation for classification, never a new observed Bash record.
        import copy
        rows, _ = oracle.load_fixed(ROOT)
        row = copy.deepcopy(rows[32])
        actual = {'stdout': '', 'stderr': "fixed-parameter-32: line 3: bad substitution: no closing `}' in  ${number:-\n",
                  'status': 1, 'files': copy.deepcopy(row['case']['files'])}
        return rows, row, actual, {'original_actual': copy.deepcopy(actual)}

    def test_case032_stays_false_raw_expected_matches_and_has_only_the_exact_invalid_classification(self):
        import copy
        _, row, actual, policy = self.fixed_policy()
        before = copy.deepcopy((row, actual, policy))
        self.assertEqual(oracle.classify(32, row, actual, policy), (False, 'confirmed-invalid'))
        self.assertEqual((row, actual, policy), before)
        self.assertEqual(row['case']['stdout'], '8\n')

    def test_exception_cannot_move_to_a_different_source_input_name_or_index(self):
        import copy
        _, row, actual, policy = self.fixed_policy()
        variants = []
        changed = copy.deepcopy(row); changed['source'] += ' '; variants.append(changed)
        changed = copy.deepcopy(row); changed['case']['args'] = ['other']; variants.append(changed)
        changed = copy.deepcopy(row); changed['name'] += ' other'; variants.append(changed)
        for changed in variants:
            with self.assertRaises(oracle.safe.OracleError): oracle.classify(32, changed, actual, policy)
        # Same bad status in another fixed recipe is an unexpected mismatch.
        self.assertEqual(oracle.classify(31, row, actual, policy), (False, 'unexpected-mismatch'))

    def test_changed_status_stderr_stdout_or_files_is_unexpected_even_for_exact_case032(self):
        import copy
        _, row, actual, policy = self.fixed_policy()
        for key, value in [('status', 2), ('stderr', 'a different syntax error'),
                           ('stdout', '8\n'), ('files', {})]:
            changed = {**copy.deepcopy(actual), key: value}
            with self.subTest(key=key):
                self.assertEqual(oracle.classify(32, row, changed, policy), (False, 'unexpected-mismatch'))
        self.assertEqual(oracle.classify(32, row, oracle.original_expected(row), policy),
                         (True, 'unexpected-mismatch'))

    def test_supported_case_requires_every_output_component_and_cannot_borrow_the_exception(self):
        import copy
        rows, _, bad, policy = self.fixed_policy()
        row = rows[0]; actual = oracle.original_expected(row)
        self.assertEqual(oracle.classify(0, row, actual, policy), (True, 'supported-match'))
        for key, value in [('status', 1), ('stderr', 'error'), ('stdout', 'wrong'), ('files', {})]:
            changed = {**copy.deepcopy(actual), key: value}
            with self.subTest(key=key):
                self.assertEqual(oracle.classify(0, row, changed, policy), (False, 'unexpected-mismatch'))
        self.assertEqual(oracle.classify(0, row, bad, policy), (False, 'unexpected-mismatch'))

    def test_changed_amendment_fails_before_parse_or_child_execution(self):
        with patch.object(oracle.safe, 'read_bytes', return_value=b'changed amendment'), \
             patch.object(oracle, 'strict', side_effect=AssertionError('No parsing')), \
             patch.object(oracle.safe, 'capture_command') as child:
            with self.assertRaisesRegex(oracle.safe.OracleError, 'amendment source changed'):
                oracle.load_amendment(ROOT, [])
            child.assert_not_called()

    def test_partition_accepts_only_58_supported_one_confirmed_invalid_and_zero_unexpected(self):
        good = {'supported-match': 58, 'confirmed-invalid': 1, 'unexpected-mismatch': 0}
        self.assertTrue(oracle.validation_summary(good)['accepted_partition'])
        for changed in [dict(good, **{'supported-match': 57}),
                        dict(good, **{'confirmed-invalid': 0}),
                        {'supported-match': 57, 'confirmed-invalid': 1, 'unexpected-mismatch': 1},
                        {'supported-match': 59, 'confirmed-invalid': 0, 'unexpected-mismatch': 0}]:
            self.assertFalse(oracle.validation_summary(changed)['accepted_partition'])
        with self.assertRaises(oracle.safe.OracleError): oracle.validation_summary({'waive': 59})

    def test_approval_must_include_current_amendment_identity(self):
        # This guard is independent of any private recording or real process.
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary); base = self.setup_control(root)
            pins = [{'path': str(oracle.safe.MANUAL.resolve()), 'sha256': oracle.safe.MANUAL_SHA}]
            good = {'schema_version': 1, 'root_reviewed': True, 'capture_id': 'unit-only',
                    'source_head': 'a' * 40, 'pins': pins, 'inputs': [],
                    'amendment': {'old': 'identity'},
                    'claims': 'Bash-only fixed observations; no VM or learner source accepted'}
            (base / 'APPROVED-unit-only.json').write_text(json.dumps(good))
            with patch.object(oracle, 'ROOT', root), \
                 patch.object(oracle.safe, 'require_guard', return_value=(7,)), \
                 patch.dict(os.environ, {'CARGO_TARGET_DIR': str(root / 'target')}), \
                 patch.object(oracle, 'load_fixed', return_value=([], [])), \
                 patch.object(oracle, 'load_amendment', return_value={'identity': {'new': 'identity'}}), \
                 patch.object(oracle, 'source_set', return_value=[]), \
                 patch.object(oracle, 'pins', return_value=pins), \
                 patch.object(oracle.safe, 'source_identity', return_value='a' * 40), \
                 patch.object(oracle.safe, 'capture_command') as child:
                with self.assertRaises(oracle.safe.OracleError): oracle.main(['--capture-id', 'unit-only'])
                child.assert_not_called()
                self.assertFalse((base / 'unit-only').exists())

    def test_source_pins_include_original_failed_observation_complete_and_new_control_fixture(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            fake = root / 'metadata'; fake.write_text('not executable; metadata-only test')
            with patch.object(oracle.safe, 'MANUAL', fake), patch.object(oracle.safe, 'BASH', fake), \
                 patch.object(oracle, 'SORT', fake), patch.object(oracle, 'CAT', fake):
                paths = {str(p) for p, _ in oracle.source_set(root)}
            for name in [oracle.AMENDMENT, oracle.PRIOR + '/MANIFEST.json',
                         oracle.PRIOR + '/COMPLETE', oracle.PRIOR + '/case-032/OBSERVATION.json',
                         'tests/fixtures/bash-parameter-arithmetic-reference.json',
                         'tools/tests/test_parameter_bash_oracle.py']:
                self.assertIn(str(root / name), paths)
