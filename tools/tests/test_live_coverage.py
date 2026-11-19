"""Validate the shipped coverage data as well as the verifier's isolated fixtures."""
from pathlib import Path
import sys
import unittest

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory
import verify_coverage


class CurrentCoverageRegistry(unittest.TestCase):
    def test_registered_curriculum_passes_the_actual_evidence_and_source_guards(self):
        report=verify_coverage.validate(ROOT,catalog='current')
        self.assertEqual(report['registered_labs'],len(teaching_inventory.catalog()))
        self.assertEqual(report['registered_labs'],report['authored_labs'])
        self.assertEqual(report['references'],report['partial']+report['supporting']+report['open'])
