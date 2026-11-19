"""Fixed-composition guard tests; never launch a host script or utility."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import measurement_scripts_reference as ref


class MeasurementScriptsReference(unittest.TestCase):
    def test_fixed_three_sources_cover_all_seventy_five_bounded_authoring_observations(self):
        values=ref.fixtures()
        self.assertEqual([v[0]for v in values],['rates','cpu','recovery'])
        for (label,value,raw),(_,path,sha)in zip(values,ref.FIXTURES):
            self.assertEqual(hashlib.sha256(raw).hexdigest(),sha)
            self.assertEqual(len(value['cases']),12)
            self.assertNotEqual(value['source'],value['alternate'])
            self.assertIn('cut -d , -f',value['source'])
        self.assertIn('records=("${records[@]}" "$entry")',values[2][1]['source'])
        self.assertNotIn('records=',values[2][1]['alternate'])
        self.assertEqual(values[2][1]['example_case']['status'],1)

    def test_no_source_path_case_or_learner_argument_is_accepted_before_any_child(self):
        with patch.object(sys, 'argv', ['fixed', '--source', 'untrusted']), \
                patch.object(ref, 'inherited_lock') as lock, \
                patch.object(ref, 'capture_command') as child:
            with self.assertRaisesRegex(ValueError, 'selector'):
                ref.main()
            lock.assert_not_called(); child.assert_not_called()

    def test_changed_or_aliased_fixture_refuses_without_running_a_child(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / ref.FIXTURES[0][1]
            path.parent.mkdir(parents=True)
            path.write_text('{}')
            with patch.object(ref, 'ROOT', root), patch.object(ref, 'capture_command') as child:
                with self.assertRaisesRegex(ValueError, 'measurement fixture changed'):
                    ref.fixtures()
                path.unlink()
                outside = root / 'unchosen.json'; outside.write_text('{}')
                path.symlink_to(outside)
                with self.assertRaises((ValueError, OSError, RuntimeError)):
                    ref.fixtures()
                child.assert_not_called()

    def test_case_schema_rejects_environment_escape_argv_and_unbounded_data(self):
        _, value, _ = ref.fixtures()[0]
        for key, bad in [('schema_version', True), ('schema_version', 2),
                         ('source', ''), ('source', 'x' * 16_385), ('extra', None),
                         ('cases', [])]:
            changed = copy.deepcopy(value); changed[key] = bad
            with self.assertRaises(ValueError, msg=key):
                ref.validate(changed)
        for key, bad in [('env', {'BASH_ENV': '/personal/startup'}), ('stdin', 'unreviewed'),
                         ('args', ['a'] * 9), ('args', ['bad\0value']), ('status', True),
                         ('files', {'/work/../escape': ''}), ('files', {'/work/.': ''}),
                         ('files', {'/work/..': ''}), ('files', {'/etc/secret': ''}),
                         ('files', {'/work/a': 'x' * 8193}), ('after', {'/work/a/b': ''}),
                         ('stdout', 'x' * 32_769), ('extra', 1)]:
            changed = copy.deepcopy(value); changed['cases'][0][key] = bad
            with self.assertRaises(ValueError, msg=key):
                ref.validate(changed)
        changed = copy.deepcopy(value); changed['cases'][1]['name'] = changed['cases'][0]['name']
        with self.assertRaises(ValueError):
            ref.validate(changed)

    def test_snapshot_accepts_only_small_ordinary_flat_outputs_and_never_reads_aliases(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'service.conf').write_text('log_level=info\n')
            self.assertEqual(ref.snapshot(root), {'/work/service.conf': 'log_level=info\n'})
            (root / 'alias').symlink_to(root / 'service.conf')
            with self.assertRaisesRegex(ValueError, 'symlink'):
                ref.snapshot(root)
            (root / 'alias').unlink()
            (root / 'folder').mkdir()
            with self.assertRaisesRegex(ValueError, 'directory'):
                ref.snapshot(root)
