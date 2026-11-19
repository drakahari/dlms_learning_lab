"""Command construction only; no test/build subprocess."""
from pathlib import Path
import tempfile
import unittest
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import dev

class FocusedPythonRunner(unittest.TestCase):
    def test_default_keeps_the_inclusive_suite_and_focused_selects_one_existing_file(self):
        root=Path(__file__).resolve().parents[2]
        base=dev.python_test_command([],root)
        self.assertEqual(base[-3:],['-s','tools/tests','-v'])
        self.assertEqual(dev.python_test_command(['--pattern','test_find_size_reference.py'],root),base+['-p','test_find_size_reference.py'])

    def test_unknown_flags_wildcards_paths_and_missing_files_never_fall_back_to_full_suite(self):
        root=Path(__file__).resolve().parents[2]
        for args in [['--help'],['--pattern'],['--pattern','test_*.py'],['--pattern','../private.py'],['--pattern','test_missing_unregistered.py'],['--pattern','test_find_size_reference.py','--other']]:
            with self.assertRaises(RuntimeError):dev.python_test_command(args,root)

    def test_alias_is_refused_even_when_its_target_is_another_project_test(self):
        with tempfile.TemporaryDirectory()as tmp:
            root=Path(tmp);folder=root/'tools/tests';folder.mkdir(parents=True)
            (folder/'test_target.py').write_text('')
            (folder/'test_alias.py').symlink_to('test_target.py')
            with self.assertRaises(RuntimeError):dev.python_test_command(['--pattern','test_alias.py'],root)
