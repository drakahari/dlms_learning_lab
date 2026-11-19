"""Closed reference-tool guards. No Bash, VM or subprocess is launched."""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
spec = importlib.util.spec_from_file_location('tracing_reference_guard', TOOLS / 'tracing_reference.py')
v = importlib.util.module_from_spec(spec); spec.loader.exec_module(v)


class TracingReferenceGuards(unittest.TestCase):
    def fixture(self):
        folder = tempfile.TemporaryDirectory(); self.addCleanup(folder.cleanup)
        return Path(folder.name)

    def test_exact_thirty_fixed_inputs_have_no_observations_or_external_commands(self):
        rows = v.fixture(); self.assertEqual(len(rows), 30)
        self.assertEqual(len({r['id'] for r in rows}), 30)
        for r in rows:
            self.assertIsNone(r['host_observation'])
            for forbidden in ['cat ', 'eval ', 'exec ', 'source ', '$(', '`', '/home/', '/tmp/', 'while ', 'for ']:
                self.assertNotIn(forbidden, r['source'])
            self.assertEqual((r['args'], r['env'], r['files'], r['stdin']), ([], {}, {}, ''))

    def test_recipe_changes_are_rejected_before_any_child(self):
        with patch.object(v, 'read_bytes', return_value=b'{"changed":true}'), patch.object(v.subprocess, 'run', side_effect=AssertionError('No child')):
            with self.assertRaisesRegex(ValueError, 'Fixed recipes changed'): v.fixture()

    def test_existing_observation_directory_cannot_be_replaced(self):
        root = self.fixture(); p = v.fresh(root, 'evidence'); (p / 'sentinel').write_text('retained')
        with self.assertRaises(FileExistsError): v.fresh(root, 'evidence')
        self.assertEqual((p / 'sentinel').read_text(), 'retained')

    def test_aliases_and_oversize_reports_are_refused(self):
        root = self.fixture(); alias = root / 'alias'; alias.symlink_to(root, target_is_directory=True)
        with self.assertRaises(ValueError): v.fresh(alias, 'new')
        with self.assertRaises(ValueError): v.output(root / 'large', {'text': 'x' * (4 * 1024 * 1024)})
        self.assertFalse((root / 'large').exists())

    def test_reports_are_exclusive_and_preserve_actual_values(self):
        root = self.fixture(); path = root / 'report'
        actual = {'stdout': '', 'stderr': '+ false\n', 'status': 1}
        v.output(path, actual); self.assertEqual(json.loads(path.read_text()), actual)
        with self.assertRaises(FileExistsError): v.output(path, {'changed': True})
        self.assertEqual(json.loads(path.read_text()), actual)

    def test_truncated_references_cannot_pass_zip_comparison(self):
        expected = [{'id': str(n)} for n in range(30)]
        observations = [dict(id=str(n), stdout='', stderr='', status=0, files={}, error=None) for n in range(30)]
        v.observation_rows(observations, expected)
        for bad in [[], observations[:-1], list(reversed(observations)),
                    [{**observations[0], 'status': False}, *observations[1:]],
                    [{**observations[0], 'error': 'unsupported'}, *observations[1:]],
                    [{**observations[0], 'extra': True}, *observations[1:]]]:
            with self.assertRaises(ValueError): v.observation_rows(bad, expected)


if __name__ == '__main__':
    unittest.main()
