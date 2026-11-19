"""Guard tests for the fixed authored-project comparator; never launch Bash."""
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import debug_recovery_reference as ref
class DebugRecoveryReference(unittest.TestCase):
    def test_real_fixed_roster_and_exact_pins_are_closed(self):
        pack,fixtures=(ref.read_pinned(*p)for p in ref.PINS)
        self.assertEqual(tuple(r['id']for r in pack['scenarios']),ref.IDS)
        self.assertEqual(sum(len(v)for v in fixtures.values()),40)
        self.assertEqual([len(r['goal']['cases'])for r in pack['scenarios']],[12,12])
        self.assertTrue(all(not c['env']and not c['stdin']for cs in fixtures.values()for c in cs))
    def test_mutated_reference_and_symlink_refuse_before_any_child(self):
        with tempfile.TemporaryDirectory()as t,patch.object(ref,'ROOT',Path(t)),patch.object(ref.subprocess,'run')as child:
            p=Path(t)/'fixed.json';p.write_text('{}')
            sha=hashlib.sha256(p.read_bytes()).hexdigest()
            self.assertEqual(ref.read_pinned('fixed.json',2,sha),{})
            p.write_text('[]')
            with self.assertRaises(ValueError):ref.read_pinned('fixed.json',2,sha)
            target=Path(t)/'target.json';target.write_text('{}');p.unlink();p.symlink_to(target)
            with self.assertRaises(ValueError):ref.read_pinned('fixed.json',2,sha)
            child.assert_not_called()
    def test_no_source_path_profile_or_learner_selector(self):
        with patch.object(sys,'argv',['fixed','--source','bad']),patch.object(ref,'inherited_lock')as lock,patch.object(ref.subprocess,'run')as child:
            with self.assertRaises(ValueError):ref.main()
            lock.assert_not_called();child.assert_not_called()
