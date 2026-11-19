"""Fixed vi observer guards only; no host editor launched by these tests."""
from pathlib import Path
import sys
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import modal_editing_reference as ref
class ModalReference(unittest.TestCase):
    def test_recipes_are_closed_ascii_data_and_never_accept_arbitrary_commands(self):
        rows=ref.recipes();self.assertEqual(len(rows),10)
        for row in rows:
            text=ref.script(row).decode('ascii')
            self.assertNotIn(':!',text);self.assertNotIn('source ',text)
            self.assertTrue(text.endswith(':'+row[-1]+'\r'))
            self.assertNotIn("call ",text)
            self.assertIn("i~",ref.script(row,True).decode("ascii"))
        for bad in [('evil','x',':!whoami','wq'),('evil','x','x','w /personal/file'),('unknown','x','x','wq')]:
            with self.assertRaises(ValueError):ref.script(bad)
    def test_argv_disables_startup_plugins_modelines_history_and_swap(self):
        args=ref.argv('/held/reference')
        for flag in ['-N','-u','-U','-i','-n','--noplugin','--cmd']:self.assertIn(flag,args)
        self.assertNotIn('-es',args)
        self.assertIn('nomodeline',ref.PRELUDE);self.assertIn('noexrc',ref.PRELUDE)
        self.assertEqual(args[-1],'work.txt')
        self.assertEqual(ref.argv('/held/reference',True),['/held/reference','--version'])
    def test_direct_or_root_execution_refuses_before_any_editor_child(self):
        with patch.object(ref,'inherited_lock',side_effect=RuntimeError('missing guard')),patch.object(ref,'capture_command')as child:
            with self.assertRaises(RuntimeError):ref.observe()
            child.assert_not_called()
        with patch.object(ref,'inherited_lock',return_value=7),patch.object(ref.os,'geteuid',return_value=0),patch.object(ref,'capture_command')as child:
            with self.assertRaisesRegex(ValueError,'root'):ref.observe()
            child.assert_not_called()
    def test_environment_has_no_personal_startup_or_loader_variables(self):
        env=ref.closed_environment({},Path('/private/disposable'))
        for key in ['HOME','VIMINIT','EXINIT','SHELL','LD_PRELOAD','PYTHONPATH']:self.assertNotIn(key,env)
        self.assertEqual(env['PATH'],'');self.assertEqual(env['LC_ALL'],'C')
    def test_app_buffer_does_not_import_or_launch_the_reference_tool(self):
        source=(ref.ROOT/'src/modal_text.rs').read_text()
        for bad in ['std::fs','std::process','Command::new','TcpStream','modal_editing_reference.py']:self.assertNotIn(bad,source)

    def test_cursor_probe_is_separate_bounded_and_never_normalizes_written_bytes(self):
        self.assertEqual(ref.cursor_from_marker(b'abc\nxy~z\n'),[2,3])
        self.assertEqual(ref.cursor_from_marker(b'~\n'),[1,1])
        for raw in [b'abc',b'~~',b'\t~',b'x'*16384+b'~']:
            with self.assertRaises(ValueError):ref.cursor_from_marker(raw)
        case=ref.recipes()[-1]
        self.assertTrue(ref.script(case).endswith(b':q!\r'))
        self.assertTrue(ref.script(case,True).endswith(b':wq\r'))

    def test_compare_requires_runner_and_rejects_changed_observations_before_probe(self):
        with patch.object(ref,'inherited_lock',side_effect=RuntimeError('missing guard')),patch.object(ref,'capture_command')as child:
            with self.assertRaises(RuntimeError):ref.compare()
            child.assert_not_called()
        with patch.object(ref,'read_bytes',return_value=b'{}'),patch.object(ref,'capture_command')as child:
            with self.assertRaisesRegex(ValueError,'observation changed'):ref.reference()
            child.assert_not_called()

    def test_navigation_profile_is_closed_and_original_capture_stays_separate(self):
        self.assertEqual(len(ref.recipes()),10)
        self.assertEqual(len(ref.recipes("navigation")),21)
        self.assertEqual(ref.REFERENCE_SHA,"f8815ade637b4f2688eb3bcfcdeb9d706cc0d55dd68faec2183b2bb34205a20e")
        self.assertEqual(len({r[0] for r in ref.recipes()+ref.recipes("navigation")}),31)
        for row in ref.recipes("navigation"):
            self.assertLessEqual(len(row[1]),1024)
            self.assertNotIn("~",row[1]+row[2])
            self.assertTrue(ref.script(row).endswith(b":wq\r"))
            self.assertNotIn(b":!",ref.script(row))
        with self.assertRaises(ValueError):ref.recipes("caller")

    def test_insertion_profile_uses_only_fixed_private_editor_keys(self):
        self.assertEqual(len(ref.recipes("insertion")),8)
        for row in ref.recipes("insertion"):
            self.assertTrue(ref.script(row).endswith(b":wq\r"))
            self.assertNotIn(b":!",ref.script(row))
            self.assertNotIn("~",row[1]+row[2])
        with self.assertRaises(ValueError):ref.script(("caller","abc","iBAD","wq"))
