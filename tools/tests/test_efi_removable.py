"""Verify the LPIC-2 recovery lesson and unchanged predecessor values."""
import hashlib
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory
import verify_coverage

ID = 'lpic2-efi-removable-recovery'

def read(path):
    return json.loads((ROOT / path).read_text())

def fingerprint(value):
    data = json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
    return len(data), hashlib.sha256(data).hexdigest()

class RemovableRecoveryTeaching(unittest.TestCase):
    def test_preserves_nine_boot_lessons_and_90_lpic2_presentations(self):
        previous = read('scenarios/boot-entry-lessons-v5.json')['scenarios']
        current = read('scenarios/boot-entry-lessons-v6.json')['scenarios']
        rows = read('scenarios/presentation/lpic2.json')
        self.assertEqual((len(previous),len(current),len(rows)),(9,10,135))
        self.assertEqual(current[:9],previous)
        self.assertEqual(fingerprint(previous),(72219,'be418f8ce8c2d330d408a14b6b694eaaa103f17451219d9de0df1e439bfa628c'))
        self.assertEqual(fingerprint(rows[:90]),(416461,'52510080b8442e4dfd9bb3b36c948130c298615ab058c3fd78a11b7e15d84d2e'))
        target = 'scenarios/boot-entry-lessons-v36.json'
        integrated = read(target)['scenarios']
        self.assertEqual(integrated[:10],current)
        known = {target:{lab['id']:lab for lab in integrated}}
        for version,source in [(5,previous),(6,current)]:
            self.assertTrue(verify_coverage.preserved_authored_pack(
                f'scenarios/boot-entry-lessons-v{version}.json',{lab['id']:lab for lab in source},known))
        self.assertEqual(verify_coverage.registered_pack_paths(ROOT,'current')[-1],'scenarios/boot-entry-lessons-v53.json')
        self.assertEqual(len(teaching_inventory.catalog()),482)

    def test_worked_copy_demonstrates_a_distinct_root_failure_before_the_repair(self):
        lab = read('scenarios/boot-entry-lessons-v6.json')['scenarios'][-1]
        row = read('scenarios/presentation/lpic2.json')[90]
        self.assertEqual((lab['id'],row['id']),(ID,ID))
        self.assertEqual(lab['objective'],' '.join(row['requirements']))
        assignment = lab['goal']['definition']['initial']
        workshop = lab['goal']['definition']['example']
        self.assertEqual(assignment['reference'],lab['solution'])
        self.assertNotEqual(assignment['media'],workshop['media'])
        self.assertNotEqual(assignment['required_root'],workshop['required_root'])
        self.assertNotEqual(assignment['required_release'],workshop['required_release'])
        self.assertNotEqual(assignment['archive_entry'],workshop['archive_entry'])
        self.assertIn('/workshop/copied-boot',workshop['reference'][0])
        self.assertIn('/workshop/recovery-boot',workshop['reference'][1])
        self.assertEqual(workshop['reference'][2],'@efi-check')
        self.assertEqual([work['value'] for work in row['worked'][1:]],workshop['reference'][:2])
        self.assertNotIn(assignment['reference'][0],json.dumps(row,ensure_ascii=False))
        self.assertIn('root UUID',row['concepts'][1]['text'])
        self.assertIn('Test recovery paths',row['practice'][0]['action'])
