import hashlib
import json
from pathlib import Path
import unittest

import teaching_inventory
import verify_coverage


ROOT = Path(__file__).resolve().parents[2]
ID = 'lpic2-rescue-startup-repair'


def read(path):
    return json.loads((ROOT / path).read_text())


def pin(value):
    raw = json.dumps(value, sort_keys=True, separators=(',', ':'),
                     ensure_ascii=False).encode()
    return len(raw), hashlib.sha256(raw).hexdigest()


class RescueStartupTeaching(unittest.TestCase):
    def test_retains_all_eleven_boot_lessons_and_ninety_two_lpic2_presentations(self):
        old = read('scenarios/boot-entry-lessons-v7.json')['scenarios']
        latest_path = 'scenarios/boot-entry-lessons-v36.json'
        current = read(latest_path)['scenarios']
        self.assertEqual((len(old), len(current)), (11, 69))
        self.assertEqual(current[:11], old)
        self.assertEqual(pin(old), (92160, '85d3b3284b24cc72ce4d06af59215c84d72117ba217330f239bbae230fb5b919'))
        self.assertTrue(verify_coverage.preserved_authored_pack(
            'scenarios/boot-entry-lessons-v7.json', {row['id']: row for row in old},
            {latest_path: {row['id']: row for row in current}}))
        rows = read('scenarios/presentation/lpic2.json')
        self.assertEqual((len(rows), rows[92]['id']), (135, ID))
        self.assertEqual(pin(rows[:92]), (424774, '3c339559da0a547bfd41ed3e474c77883904a6fbe32dbd8ca23b29cd5f532fc1'))
        self.assertEqual(len(teaching_inventory.catalog()), 482)

    def test_teaches_actual_startup_choice_with_distinct_repair_and_no_assignment_answer(self):
        lab = read('scenarios/boot-entry-lessons-v8.json')['scenarios'][-1]
        row = next(r for r in read('scenarios/presentation/lpic2.json') if r['id'] == ID)
        self.assertEqual(lab['objective'], ' '.join(row['requirements']))
        assignment = lab['goal']['definition']
        example = lab['unit_manager_teaching']
        self.assertEqual(assignment['fresh']['recovery_target'], 'rescue.target')
        self.assertEqual(assignment['initial']['default_target'], 'multi-user.target')
        self.assertTrue(assignment['preservation']['default_target'])
        self.assertNotEqual(assignment['initial']['inventory']['units'],
                            example['definition']['initial']['inventory']['units'])
        self.assertEqual(example['solution'][:2],
                         [{'kind': 'Startup', 'target': None},
                          {'kind': 'Startup', 'target': 'rescue.target'}])
        self.assertIn('MODE=workshop-ready', json.dumps(row))
        self.assertNotIn('MODE=ready-east', json.dumps(row))
