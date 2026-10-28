"""Admission/filesystem/comparison guards; no Bash, VM process or host command.

All temporary evidence belongs to the project disk, never system /tmp.
"""
import copy
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import call, patch

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
import optional_inputs_bash_oracle as oracle
from build_safety import owned_directory


class Guards(unittest.TestCase):
    def setUp(self):
        self.root = TOOLS.parent
        self.original = oracle.strict((self.root / oracle.FIXED).read_bytes())
        self.rows = self.original['rows']
        self.row = next(r for r in self.rows if r['id'] == 'shell-optional-report/reference/assignment00')
        parent = owned_directory(self.root, self.root / '.build-artifacts/reference-guard-tests')
        self.temporary = tempfile.TemporaryDirectory(prefix='optional-', dir=parent)
        self.addCleanup(self.temporary.cleanup)
        self.work = Path(self.temporary.name)

    def test_exact117_114_15_and3_exclusions_are_not_remapped(self):
        raw, rows, excluded = oracle.load_fixed(self.root)
        self.assertEqual(hashlib.sha256(raw).hexdigest(), oracle.FIXED_SHA)
        self.assertEqual((len(self.rows), len(rows), len(excluded)), (117, 114, 3))
        self.assertEqual({r['id'] for r in excluded}, oracle.EXCLUDED)
        self.assertEqual(len({r['source'] for r in rows}), 15)
        self.assertEqual(rows, [r for r in self.rows if r['id'] not in oracle.EXCLUDED])
        for row in self.rows:
            if row['id'] in oracle.EXCLUDED:
                self.assertEqual(row['case']['args'], ['data.txt', '/work/report.txt'])
                with self.assertRaisesRegex(oracle.safe.OracleError, 'Outside-CWD'):
                    oracle.check_case(row)

    def test_omission_reorder_mutation_or_unknown_program_rejected_without_child(self):
        original = (self.root / oracle.FIXED).read_bytes()
        for changed in [self.rows[:-1], list(reversed(self.rows)), [*self.rows, self.rows[0]]]:
            value = {**self.original, 'rows': changed}
            with patch.object(oracle.safe, 'read_bytes', return_value=oracle.canonical(value)), \
                 patch.object(oracle, 'bounded_popen') as child:
                with self.assertRaises(oracle.safe.OracleError):
                    oracle.load_fixed(self.root)
                child.assert_not_called()
        self.assertEqual((self.root / oracle.FIXED).read_bytes(), original)
        row = copy.deepcopy(self.row)
        row['source'] = 'eval "$1"\n'
        with self.assertRaisesRegex(oracle.safe.OracleError, 'reviewed fixed programs'):
            oracle.check_case(row)

    def test_paths_controls_and_parent_argv_fail_before_launch(self):
        for arg in ['/etc/passwd', '../outside', 'sub/file', '.', '..', 'a\\b', 'line\nname', 'a\0b']:
            row = copy.deepcopy(self.row)
            row['case']['args'] = [arg]
            with patch.object(oracle, 'bounded_popen') as child:
                with self.assertRaises(oracle.safe.OracleError):
                    oracle.check_case(row)
                child.assert_not_called()
        row = copy.deepcopy(self.row)
        row['case']['args'] = ['./data.txt']
        oracle.check_case(row)

    def test_flat_mapping_never_creates_absolute_host_or_parent_paths(self):
        for name in ['/elsewhere/x', '/work/../x', '/work/a/b', '/work/.', '/work/', '/work/a\\b', '/work/a\n']:
            with self.assertRaises(oracle.safe.OracleError):
                oracle.flat(name)
        self.assertEqual(oracle.flat('/work/- input.txt'), '- input.txt')
        self.assertEqual(oracle.flat('/work/daily summary.txt'), 'daily summary.txt')

    def test_exclusive_ordinary_materialization_and_unchanged_existing_file(self):
        budget = oracle.safe.Budget(65536)
        initial = {'/work/- input.txt': 'Café\n', '/work/keep.txt': 'retain'}
        self.assertEqual(oracle.materialize(self.work, initial, budget), len('Café\nretain'.encode()))
        self.assertEqual(oracle.final_files(self.work, {'- input.txt', 'keep.txt'})[0], initial)
        with self.assertRaises(FileExistsError):
            oracle.materialize(self.work, initial, budget)
        self.assertEqual((self.work / 'keep.txt').read_text(), 'retain')

    def test_symlink_and_directory_outputs_rejected_without_target_read(self):
        target = self.work / 'outside'
        target.write_text('preserve')
        work = self.work / 'work'
        work.mkdir()
        (work / 'link').symlink_to(target)
        with self.assertRaises(oracle.safe.OracleError):
            oracle.final_files(work, {'link'})
        self.assertEqual(target.read_text(), 'preserve')
        other = self.work / 'other'
        other.mkdir()
        (other / 'directory').mkdir()
        with self.assertRaises(oracle.safe.OracleError):
            oracle.final_files(other, {'directory'})

    def test_existing_or_symlink_owned_case_directories_refuse(self):
        first = oracle.safe.private_dir(self.work, 'case-000')
        with self.assertRaises(FileExistsError):
            oracle.safe.private_dir(self.work, 'case-000')
        (self.work / 'case-001').symlink_to(first, target_is_directory=True)
        with self.assertRaises(FileExistsError):
            oracle.safe.private_dir(self.work, 'case-001')
        for name in ['../outside', '/absolute', 'case/name']:
            with self.assertRaises(oracle.safe.OracleError):
                oracle.safe.private_dir(self.work, name)

    def test_startup_environment_is_not_inherited_and_unknown_keys_refuse(self):
        with patch.dict(os.environ, {'BASH_ENV': '/private/startup', 'PATH': '/private/bin',
                                     'BASH_FUNC_unsafe%%': '() { :; }', 'HOME': '/private/home'}):
            value = oracle.safe.closed_environment({'TEAM': '$(printf unexpected)'}, self.work)
        self.assertEqual(value['PATH'], '')
        self.assertNotIn('HOME', value)
        self.assertNotIn('BASH_ENV', value)
        self.assertNotIn('BASH_FUNC_unsafe%%', value)
        self.assertEqual(value['TEAM'], '$(printf unexpected)')
        for key in ['PATH', 'BASH_ENV', 'IFS', 'SHELLOPTS', 'UNREVIEWED']:
            row = copy.deepcopy(self.row)
            row['case']['env'][key] = 'data'
            with self.assertRaises(oracle.safe.OracleError):
                oracle.check_case(row)

    def test_exact_same_source_approval_only(self):
        control = {'root_reviewed': False, 'capture_id': 'fixed', 'pins': [{'sha256': 'a'}]}
        oracle.approved(oracle.canonical({**control, 'root_reviewed': True}), control)
        for bad in [control, {**control, 'root_reviewed': 1}, {**control, 'root_reviewed': True, 'extra': 1},
                    {**control, 'root_reviewed': True, 'pins': []}]:
            with self.assertRaises(oracle.safe.OracleError):
                oracle.approved(oracle.canonical(bad), control)

    def test_duplicate_json_and_arbitrary_capture_identity_refuse(self):
        with self.assertRaises(oracle.safe.OracleError):
            oracle.strict('{"a":1,"a":2}')
        for value in ['../capture', '/capture', 'capture/name', '', 'x' * 65]:
            with self.assertRaises(oracle.safe.OracleError):
                oracle.identity(value)

    def test_actual_vm_identity_order_source_and_case_fingerprints_required(self):
        _, rows, _ = oracle.load_fixed(self.root)
        values = [{'id': row['id'], 'source_sha256': oracle.digest(row['source'].encode()),
                   'case_sha256': oracle.digest(oracle.canonical(row['case'])),
                   'outcome': {**oracle.expected(row), 'error': None}} for row in rows]
        text = '\n'.join(oracle.canonical(v).decode() for v in values)
        self.assertEqual(oracle.vm_rows(text, rows), [oracle.expected(r) for r in rows])
        for key, replacement in [('id', 'other'), ('source_sha256', '0' * 64), ('case_sha256', '0' * 64)]:
            changed = copy.deepcopy(values)
            changed[0][key] = replacement
            with self.assertRaises(oracle.safe.OracleError):
                oracle.vm_rows('\n'.join(oracle.canonical(v).decode() for v in changed), rows)
        with self.assertRaises(oracle.safe.OracleError):
            oracle.vm_rows(text.split('\n', 1)[1], rows)
        for key, replacement in [('error', 'Unsupported syntax'), ('status', False),
                                 ('files', {'/outside': 'unexpected'})]:
            changed = copy.deepcopy(values)
            changed[0]['outcome'][key] = replacement
            with self.assertRaises(oracle.safe.OracleError):
                oracle.vm_rows('\n'.join(oracle.canonical(v).decode() for v in changed), rows)

    def test_state_stdout_stderr_and_status_mismatches_never_become_pass(self):
        expected = oracle.expected(self.row)
        self.assertTrue(oracle.comparison(self.row, expected, expected)['passed'])
        for key, value in [('stdout', 'wrong'), ('stderr', 'unexpected'), ('status', 9), ('files', {})]:
            broken = {**expected, key: value}
            self.assertFalse(oracle.comparison(self.row, expected, broken)['passed'])
            self.assertFalse(oracle.comparison(self.row, broken, expected)['passed'])
            self.assertFalse(oracle.comparison(self.row, broken, broken)['passed'])

    def test_child_limits_and_group_policy_are_fixed_without_launching(self):
        with patch.object(oracle.subprocess, 'Popen') as child:
            oracle.bounded_popen(['/fixed/descriptor'], env={}, pass_fds=(3,), start_new_session=False)
            setup = child.call_args.kwargs['preexec_fn']
            self.assertFalse(child.call_args.kwargs['start_new_session'])
            with patch.object(oracle.os, 'umask') as mask, patch.object(oracle.resource, 'setrlimit') as limits:
                setup()
                mask.assert_called_once_with(0o077)
                self.assertEqual(limits.call_count, 5)
                self.assertIn(call(oracle.resource.RLIMIT_CPU, (2, 2)), limits.call_args_list)
                self.assertIn(call(oracle.resource.RLIMIT_AS, (256 * 1024 * 1024,) * 2), limits.call_args_list)

    def test_no_arbitrary_source_input_or_executable_cli(self):
        for extra in [['--source', 'echo unsafe'], ['--fixture', '/outside'], ['--bash', '/other']]:
            with patch.object(oracle, 'bounded_popen') as child, patch('sys.stderr'):
                with self.assertRaises(SystemExit):
                    oracle.main(['--capture-id', 'fixed', *extra])
                child.assert_not_called()

    def test_capture_adapter_keeps_empty_stdin_finite_streams_and_existing_group(self):
        with patch.object(oracle.safe, 'capture_command', return_value={'status': 0}) as capture:
            oracle.capture_child(['/fixed'], self.work, self.work, {}, oracle.safe.Budget(), inherited=(3,))
            args, kwargs = capture.call_args
            self.assertEqual(args[4], b'')
            self.assertEqual(args[6], oracle.safe.STREAM_CAP)
            self.assertEqual(kwargs['timeout'], oracle.safe.TIMEOUT)
            self.assertEqual(kwargs['inherited'], (3,))
            self.assertIn('popen', kwargs)


if __name__ == '__main__':
    unittest.main()
