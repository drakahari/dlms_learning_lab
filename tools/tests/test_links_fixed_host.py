"""Prospective mock-only guards: no actual fixture, subprocess or host writes."""
import errno
import importlib.util
import json
import os
from pathlib import Path
import stat
from types import SimpleNamespace
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('links_fixed_host',ROOT/'tools/links_fixed_host.py')
module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)

def directory(**changes):
    fields=dict(st_mode=stat.S_IFDIR|0o700,st_uid=1000,st_dev=5,st_ino=9)
    fields.update(changes);return SimpleNamespace(**fields)

class FixedGuards(unittest.TestCase):
    def fixture(self,index=0):
        with patch.object(module.os,'fstat',return_value=directory()),patch.object(module.os,'getuid',return_value=1000),patch.object(module.os,'listdir',return_value=[]):
            return module.FixedFixture(index,73)

    def test_exact_eight_recipes_and_all_predictions_are_unobserved(self):
        fixture=json.loads((ROOT/'tests/fixtures/links-fixed-reference.json').read_text())
        self.assertIsNone(fixture['actual_comparison']);self.assertEqual(tuple(r['id']for r in fixture['cases']),module.IDS)
        self.assertEqual(len(module.NAMES),8);self.assertEqual(len(module.SYMBOLIC),8)
        for row in fixture['cases']+fixture['vm_only']:
            self.assertIsNone(row['vm_observation']);self.assertIsNone(row['linux_observation'])
        self.assertEqual(len(fixture['vm_only']),6)

    def test_index_is_closed_before_any_filesystem_observation(self):
        with patch.object(module.os,'fstat')as inspect:
            for value in [-1,8,True,'0',None]:
                with self.assertRaises(ValueError):module.FixedFixture(value,73)
            inspect.assert_not_called()

    def test_private_owned_newly_empty_descriptor_is_mandatory(self):
        for info,existing in [(directory(st_mode=stat.S_IFREG|0o600),[]),(directory(st_mode=stat.S_IFDIR|0o755),[]),(directory(st_uid=1001),[]),(directory(),['existing'])]:
            with patch.object(module.os,'fstat',return_value=info),patch.object(module.os,'getuid',return_value=1000),patch.object(module.os,'listdir',return_value=existing):
                with self.assertRaises(ValueError):module.FixedFixture(0,73)

    def test_paths_are_exact_not_just_inside_looking_and_have_a_finite_operation_budget(self):
        h=self.fixture()
        with patch.object(module.os,'fstat',return_value=directory()):
            self.assertEqual(h.path('item'),'item')
            for name in ['/etc/passwd','../item','./item','a/../../item','unknown','','item\x00','item/','alias/child']:
                with self.assertRaises(ValueError):h.path(name)
            h.operations=128
            with self.assertRaises(ValueError):h.path('item')
        h=self.fixture()
        with patch.object(module.os,'fstat',return_value=directory(st_ino=10)):
            with self.assertRaises(ValueError):h.path('item')

    def test_symbolic_targets_are_fixed_relative_values_before_any_mutator(self):
        h=self.fixture(2)
        with patch.object(module.os,'symlink')as create:
            for target,name in [('/personal','a/ref'),('../data','a/ref'),('unknown','a/ref'),('data','unknown')]:
                with self.assertRaises(ValueError):h.symbolic(target,name)
            create.assert_not_called()

    def test_only_three_fixed_syscall_errors_are_normalized(self):
        h=self.fixture()
        for number,kind in [(errno.ENOENT,'NotFound'),(errno.ENOTDIR,'NotDirectory'),(errno.EEXIST,'Exists')]:
            def operation():raise OSError(number,'fixed synthetic guard')
            self.assertEqual(h.error(operation),kind)
        def unexpected():raise OSError(errno.ELOOP,'excluded')
        with self.assertRaises(OSError):h.error(unexpected)
        with self.assertRaises(ValueError):h.error(lambda:None)

    def test_real_syscall_families_do_not_use_subprocess_shell_exec_or_network(self):
        import ast
        source=(ROOT/'tools/links_fixed_host.py').read_text();tree=ast.parse(source)
        forbidden={'subprocess','socket','ctypes','shutil','tempfile','multiprocessing'}
        imported=set()
        for node in ast.walk(tree):
            if isinstance(node,ast.Import):imported.update(a.name.split('.')[0]for a in node.names)
            if isinstance(node,ast.ImportFrom):imported.add(node.module.split('.')[0])
        self.assertFalse(imported&forbidden)
        self.assertIn('UNWIRED SOURCE PROTOTYPE',source)
        self.assertNotIn('sys.argv',source)

if __name__=='__main__':unittest.main()
