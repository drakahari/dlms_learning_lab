import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location('storage_reference', Path(__file__).parents[1] / 'storage_reference.py')
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)


class StorageReferenceTests(unittest.TestCase):
    def test_reference_uses_only_fixed_parser_flags_and_one_private_table(self):
        path = Path('/tmp/owned-fixture/sample.fstab')
        self.assertEqual(M.reference_argv(path), ['/usr/bin/findmnt', '--fstab', '--nocanonicalize', '--list', '--json', '--output', 'SOURCE,TARGET,FSTYPE,OPTIONS,FREQ,PASSNO', '--tab-file', str(path)])
        self.assertNotIn('--evaluate', M.OPTIONS)

    def test_fixture_boundary_rejects_external_program_or_path_fields(self):
        for item in [None, [], [{'name':'bad','configuration':'ok','command':'mount'}], [{'name':'bad','configuration':'x' * 16385}], [{'name':'bad','configuration':'\0'}]]:
            with self.assertRaises(ValueError):
                M.validate(item)

    def test_policy_equation_does_not_let_defaults_erase_explicit_policy(self):
        row={'source':'UUID=AAAA','target':'/data','fstype':'xfs','options':'ro,noauto,defaults','freq':0,'passno':2}
        got=M.normalize(row)
        self.assertTrue(got['read_only'])
        self.assertFalse(got['automatic'])
        self.assertEqual(got['source'],'UUID=AAAA')


if __name__ == '__main__':
    unittest.main()
