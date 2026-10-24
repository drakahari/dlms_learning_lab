"""Guard tests for fixed find references; never starts a subprocess or GNU find."""
import copy
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
import filesystem_find_reference as reference


class FindReferenceTests(unittest.TestCase):
    def fixed(self):
        return json.loads((ROOT / reference.FIXTURE).read_bytes())

    def rejected(self, argv):
        with self.assertRaises(reference.finite.OracleError):
            reference.validate_argv(argv)

    def test_exact_inputs_preserve_all152_current_and_original79_prefix_commands(self):
        raw, fixtures = reference.load_fixed(ROOT)
        self.assertEqual(hashlib.sha256(raw).hexdigest(), reference.FIND_SHA)
        self.assertEqual(len(fixtures), 6)
        self.assertEqual(sum(len(f['commands']) for f in fixtures), 32)
        self.assertEqual(sum(c['comparison'] == 'unsupported' for f in fixtures for c in f['commands']), 1)
        old = reference.load_legacy(ROOT / 'tests/fixtures/filesystem-reference.json')
        self.assertEqual(sum(len(f['commands']) for f in old), 152)
        self.assertEqual(sum(len(f['commands']) for f in old[:8]), 79)

    def test_argv_rejects_host_starts_escape_multiple_roots_and_shell_heads(self):
        for argv in [[], ['find'], ['sh', '-c', 'find .'], ['find', '/'], ['find', '/etc'],
                     ['find', '../'], ['find', 'dir/../../etc'], ['find', '-name', 'x'],
                     ['find', '.', '/etc'], ['find', '--', '.'], ['find', '.\0x']]:
            self.rejected(argv)
        for start in ['.', './docs/', 'docs/..', 'docs//deep', './-name', 'two words']:
            reference.validate_argv(['find', start, '-maxdepth', '1', '-type', 'f', '-print'])

    def test_actions_operators_links_formats_and_wrong_expression_positions_never_run(self):
        for tail in [['-exec', 'id', ';'], ['-execdir', 'rm', '{}', '+'], ['-delete'], ['-L'],
                     ['-type', 'l'], ['-type', 'f,d'], ['-o', '-print'], ['!', '-type', 'f'],
                     ['(', '-type', 'f', ')'], ['-prune'], ['-print0'], ['-printf', '%p'],
                     ['-print', '-type', 'f'], ['-print', '-print'], ['-type', 'f', '-maxdepth', '0']]:
            self.rejected(['find', '.', *tail])

    def test_pattern_data_is_not_treated_as_a_host_path_or_command(self):
        for pattern in ['*.log', '?', 'é', 'two words100%', '/etc/*', '-exec', r'\[x\]', r'\\']:
            reference.validate_argv(['find', '.', '-name', pattern])
        for pattern in ['[a]', ']', '\\', 'a\0b', 'x' * 256]:
            self.rejected(['find', '.', '-name', pattern])

    def test_decimal_predicate_count_and_command_byte_caps_are_strict(self):
        for depth in ['-1', '+1', '', '2147483648', '4294967295', '4294967296', '00000000000', '１２']:
            self.rejected(['find', '.', '-maxdepth', depth])
        reference.validate_argv(['find', '.', '-maxdepth', '2147483647'])
        reference.validate_argv(['find', '.', *(['-type', 'f'] * 16)])
        self.rejected(['find', '.', *(['-type', 'f'] * 17)])
        self.rejected(['find', 'a' * 4097])

    def test_exact_invalid_depth_controls_remain_safe_and_are_compared_as_errors(self):
        for value in ['2147483648', '4294967295']:
            argv = ['find', 'docs', '-maxdepth', value, '-type', 'f']
            self.rejected(argv)
            reference.validate_argv(argv, invalid_depth=True)
            command = {'comparison': 'rejected', 'expected_paths': []}
            actual = {'ok': False, 'output': '', 'error': 'find -maxdepth exceeds2147483647.'}
            native = {'status': 1, 'stdout': '', 'stderr': 'maxdepth: Numerical result out of range'}
            self.assertEqual(reference.compare_observation(command, actual, native), {'matched': True})
            with self.assertRaises(reference.finite.OracleError):
                reference.compare_observation(command, {'ok': True, 'output': '', 'error': None}, native)
        for argv in [['find', '.'], ['find', '.', '-maxdepth', '2147483647'],
                     ['find', '.', '-maxdepth', '4294967296'], ['find', '/', '-maxdepth', '4294967295'],
                     ['find', '.', '-maxdepth', '4294967295', '-exec', 'id', ';']]:
            with self.assertRaises(reference.finite.OracleError):
                reference.validate_argv(argv, invalid_depth=True)

    def test_long_listing_normalization_excludes_only_filesystem_dependent_directory_fields(self):
        import filesystem_reference as legacy
        conventional = 'drwx------ 2 alex staff 0 Jan 01 00:00 locked\n-rw------- 1 alex staff 7 Jan 01 00:00 note\n'
        btrfs = 'drwx------ 1 owner other 4096 Oct 24 12:00 locked\n-rw------- 1 owner other 7 Oct 24 12:00 note\n'
        normalize = legacy.normalize_long_listing
        self.assertEqual(normalize(conventional), normalize(btrfs))
        self.assertNotEqual(normalize(conventional), normalize(btrfs.replace('drwx------', 'drwxr-x---')))
        self.assertNotEqual(normalize(conventional), normalize(btrfs.replace('-rw------- 1', '-rw------- 2')))
        self.assertNotEqual(normalize(conventional), normalize(btrfs.replace('other 7', 'other 8')))
        self.assertNotEqual(normalize(conventional), normalize(btrfs.replace('locked', 'different')))

    def test_fixture_metadata_rejects_parent_symlink_shapes_unknown_fields_and_bounds(self):
        for mutate in [lambda d: d.update(schema_version=True),
                       lambda d: d.update(extra=True),
                       lambda d: d['fixtures'][0]['entries'].update({'../escape': {'mode': 0o600}}),
                       lambda d: d['fixtures'][0]['entries'].update({'link': {'mode': 0o600, 'symlink': '/etc'}}),
                       lambda d: d['fixtures'][0]['entries'].update({'bad': {'mode': 0o1000}}),
                       lambda d: d['fixtures'][0]['entries'].update({'bad': {'mode': 0o600, 'text': 'x' * 1025}}),
                       lambda d: d['fixtures'][0]['entries'].update({'missing/child': {'mode': 0o600}})]:
            data = self.fixed()
            mutate(data)
            with self.assertRaises(reference.finite.OracleError):
                reference.validate_fixtures(data)
        for name in ['.', '..', 'a/../b', '/absolute', 'a//b', 'line\nbreak']:
            with self.assertRaises(reference.finite.OracleError):
                reference.safe_name(name)

    def test_tampered_or_symlinked_fixed_file_is_rejected_before_reference_execution(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / reference.FIXTURE
            target.parent.mkdir(parents=True)
            target.write_bytes((ROOT / reference.FIXTURE).read_bytes() + b' ')
            with self.assertRaises(reference.finite.OracleError):
                reference.load_fixed(root)
            target.unlink()
            target.symlink_to(ROOT / reference.FIXTURE)
            with self.assertRaises(OSError):
                reference.load_fixed(root)

    def test_snapshot_restores_fixture_modes_and_detects_real_metadata_content_changes(self):
        entries = {'dir': {'mode': 0o400}, 'dir/item': {'mode': 0, 'text': 'sample'}}
        with tempfile.TemporaryDirectory() as temporary:
            work = Path(temporary)
            reference.materialize(work, entries, reference.finite.Budget())
            first = reference.snapshot(work, entries)
            self.assertEqual((work / 'dir').stat().st_mode & 0o777, 0o400)
            self.assertEqual(reference.snapshot(work, entries), first)
            (work / 'dir').chmod(0o700)
            (work / 'dir/item').chmod(0o600)
            (work / 'dir/item').write_text('changed')
            (work / 'dir/item').chmod(0)
            (work / 'dir').chmod(0o400)
            self.assertNotEqual(reference.snapshot(work, entries), first)
            # Permit only this tiny test fixture's standard TempDir cleanup.
            (work / 'dir').chmod(0o700)

    def test_preexisting_paths_and_link_entries_are_not_overwritten_or_followed(self):
        with tempfile.TemporaryDirectory() as temporary:
            work = Path(temporary)
            (work / 'keep').write_bytes(b'original')
            with self.assertRaises(FileExistsError):
                reference.materialize(work, {'keep': {'mode': 0o600, 'text': 'new'}}, reference.finite.Budget())
            self.assertEqual((work / 'keep').read_bytes(), b'original')
            (work / 'link').symlink_to('keep')
            with self.assertRaises(reference.finite.OracleError):
                reference.snapshot(work, {'keep': {'mode': 0o600, 'text': 'original'}, 'link': {'mode': 0o600, 'text': 'unknown'}})
            self.assertEqual((work / 'keep').read_bytes(), b'original')

    def test_complete_and_partial_comparisons_preserve_exact_root_spelling_as_sets(self):
        complete = {'comparison': 'complete', 'expected_paths': ['./docs/', './docs/x']}
        actual = {'ok': True, 'output': './docs/\n./docs/x\n', 'error': None}
        native = {'status': 0, 'stdout': './docs/x\n./docs/\n', 'stderr': ''}
        self.assertEqual(reference.compare_observation(complete, actual, native), {'matched': True})
        native['stdout'] = 'docs/\ndocs/x\n'
        with self.assertRaises(reference.finite.OracleError):
            reference.compare_observation(complete, actual, native)
        partial = {'comparison': 'incomplete', 'expected_paths': ['./safe']}
        step = {'ok': False, 'output': '', 'error': 'Search incomplete. Partial matches:\n./safe\nDiagnostics:\nPermission denied'}
        native = {'status': 1, 'stdout': './safe\n', 'stderr': 'denied'}
        self.assertEqual(reference.compare_observation(partial, step, native), {'matched': True})
        step['error'] = 'Search incomplete. Partial matches:\nDiagnostics:\ndenied'
        with self.assertRaises(reference.finite.OracleError):
            reference.compare_observation(partial, step, native)

    def test_readable_nonsearchable_boundary_does_not_fabricate_a_gnu_match(self):
        command = {'comparison': 'unsupported', 'expected_paths': ['./safe']}
        step = {'ok': False, 'output': '', 'error': 'Search incomplete. Partial matches:\n./safe\nDiagnostics:\nUnsupported descent'}
        native = {'status': 0, 'stdout': './readable/item\n./safe\n', 'stderr': ''}
        result = reference.compare_observation(command, step, native)
        self.assertIn('not compared as supported', result['boundary'])
        for mutated in [{'ok': True, 'output': './safe\n', 'error': None},
                        {'ok': False, 'output': '', 'error': 'Permission denied'}]:
            with self.assertRaises(reference.finite.OracleError):
                reference.compare_observation(command, mutated, native)

    def test_entrypoint_failed_preflight_starts_zero_legacy_or_new_children(self):
        import filesystem_reference as legacy
        with patch.object(legacy, 'BASH', '/fixed/not-executed/bash'):
            with patch.object(legacy.os, 'getuid', return_value=1000):
                with patch.object(sys, 'argv', ['filesystem_reference.py']):
                    with patch.object(reference, 'preflight', side_effect=reference.finite.OracleError('no actual MAIN guard')) as guard:
                        with patch.object(legacy.subprocess, 'run', side_effect=AssertionError('no legacy child may start')) as old_child:
                            with patch.object(reference.finite, 'capture_command', side_effect=AssertionError('no new child may start')) as new_child:
                                with patch.object(reference, 'load_legacy', side_effect=AssertionError('preflight must come first')):
                                    with self.assertRaises(reference.finite.OracleError):
                                        legacy.main()
                                guard.assert_called_once_with(legacy.ROOT, legacy.PROBE)
                                old_child.assert_not_called()
                                new_child.assert_not_called()

    def test_main_only_guard_rejection_precedes_any_subprocess(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / '.git').mkdir()
            probe = root / 'target/debug/examples/filesystem_probe'
            with patch.object(reference.finite, 'require_guard', side_effect=reference.finite.OracleError('no main guard')):
                with patch.object(reference.finite, 'capture_command', side_effect=AssertionError('must not launch')):
                    with self.assertRaises(reference.finite.OracleError):
                        reference.compare_fixed(root, probe, lambda _: {})


if __name__ == '__main__':
    unittest.main()
