"""Fixed-query synthetic guards; no real executable or privilege calls."""
import copy
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch,Mock
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import find_size_reference as ref

class SizeReference(unittest.TestCase):
    def test_exact_fixture_and_query_grammar(self):
        f=ref.fixture();self.assertEqual(len(f['cases']),31)
        for row in f['cases']:ref.validate_query(row['args'])
        for args in [['/home','-type','f'],['.','-type','f','-exec','sh'],['.','-type','f','-delete'],['.','-type','f','-size','$(id)'],['.','-type','d','-size','1k'],['.','-type','f','-fprint','/tmp/output']]:
            with self.assertRaises(ValueError):ref.validate_query(args)

    def test_membership_preserves_finite_names_but_does_not_claim_readdir_order(self):
        f=ref.fixture()
        self.assertEqual(ref.path_members('./n1\n./n0\n',f),['./n0','./n1'])
        for text in ['/home/private\n','./../private\n','./n1','./n1\n./n1\n','./unknown\n']:
            with self.assertRaises(ValueError):ref.path_members(text,f)

    def test_owned_small_fixtures_have_real_byte_sizes_and_unchanged_snapshots(self):
        with tempfile.TemporaryDirectory()as tmp:
            work=Path(tmp);ref.make_files(work,ref.fixture());a=ref.snapshot(work)
            self.assertEqual(a['n0']['text'],'');self.assertEqual((work/'utf8').stat().st_size,2)
            self.assertEqual((work/'n1025').stat().st_size,1025)
            self.assertEqual(a,ref.snapshot(work))

    def test_parent_restrictions_never_apply_and_expired_capture_launches_nothing(self):
        effect=Mock();g=ref.ChildGuard(7,10,effect,pid=lambda:10,write=effect,close=effect,apply_limits=effect,apply_core=effect)
        with self.assertRaises(RuntimeError):g()
        effect.assert_not_called()
        with patch.object(ref.subprocess,'Popen')as child:
            with self.assertRaises(ValueError):ref.capture([],None,None,{},None,(),0o022,32768,0)
            child.assert_not_called()

    def test_complete_binding_and_missing_input_guards(self):
        data=b'("src/x.rs",include_bytes!("../../src/x.rs"))'
        with patch.object(ref,'read_bytes',return_value=data),patch.object(ref,'names',return_value=['src/x.rs']):ref.validate_bindings()
        with patch.object(ref,'read_bytes',return_value=data),patch.object(ref,'names',return_value=['src/x.rs','src/missing.rs']):
            with self.assertRaises(ValueError):ref.validate_bindings()
