"""Keep reporting corrections separate from new labs and whole-objective credit."""
import csv
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory


class ScopeReconciliation(unittest.TestCase):
    def test_six_parents_name_registered_practice_and_leave_explicit_gaps(self):
        expected = {
            ('CCNA v1.1', '3.5'): 4, ('CCNA v1.1', '4.5'): 4,
            ('CCNA v1.1', '4.7'): 3, ('LPIC-1 v5.0', '101.3'): 3,
            ('LPIC-1 v5.0', '109.2'): 5, ('LPIC-2 v4.5', '210.1'): 2,
        }
        labs = {lab['id'] for lab, _ in teaching_inventory.catalog()}
        with (ROOT / 'docs/CURRICULUM.csv').open() as f:
            curriculum = {(r['blueprint'], r['objective_id']): r for r in csv.DictReader(f)}
        with (ROOT / 'docs/SKILL_COVERAGE.csv').open() as f:
            skills = {(r['blueprint'], r['reference']): r for r in csv.DictReader(f)}
        for key, count in expected.items():
            with self.subTest(key=key):
                parent, skill = curriculum[key], skills[key]
                self.assertEqual(parent['status'], 'Partial')
                self.assertEqual(skill['status'], 'Partial practice')
                self.assertEqual(skill['kind'], 'objective')
                ids = parent['implemented_scenarios'].split(';')
                self.assertEqual(len(set(ids)), count)
                self.assertTrue(set(ids) <= labs)
                self.assertEqual(skill['implemented_scenarios'], parent['implemented_scenarios'])
                self.assertEqual(skill['remaining'], parent['gap_or_fidelity_limit'])
                self.assertIn('no ', skill['remaining'].lower())
                self.assertGreater(len(skill['verified_scope']), 60)
                self.assertTrue(skill['source_sha256'])
                for path in skill['evidence_tests'].split(';'):
                    self.assertTrue((ROOT / path).is_file(), path)
        self.assertEqual(len(labs), 482)
