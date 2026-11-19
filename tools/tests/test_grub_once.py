import hashlib
import json
from pathlib import Path
import unittest

import teaching_inventory
import verify_coverage


ROOT = Path(__file__).resolve().parents[2]
ID = 'lpic2-grub-one-boot-recovery'


def load(name):
    return json.loads((ROOT / name).read_text())


class TemporaryGrubTeaching(unittest.TestCase):
    def test_retains_complete_removable_catalog_and_teaching_values(self):
        prior = load('scenarios/boot-entry-lessons-v6.json')['scenarios']
        active = load('scenarios/boot-entry-lessons-v7.json')['scenarios']
        self.assertEqual((len(prior), len(active)), (10, 11))
        self.assertEqual(active[:10], prior)
        target = 'scenarios/boot-entry-lessons-v36.json'
        latest = load(target)['scenarios']
        self.assertEqual(latest[:11], active)
        for version,rows in [(6,prior),(7,active)]:
            self.assertTrue(verify_coverage.preserved_authored_pack(
                f'scenarios/boot-entry-lessons-v{version}.json', {row['id']: row for row in rows},
                {target: {row['id']: row for row in latest}}))
        rows = load('scenarios/presentation/lpic2.json')
        pinned = json.dumps(rows[:91], sort_keys=True, separators=(',', ':'),
                            ensure_ascii=False).encode()
        self.assertEqual((len(pinned), hashlib.sha256(pinned).hexdigest()),
                         (420906, '8c0fd8842573b141fc1a483a9122fcb54c39da5f59d1c21e3b3eb3b75b4bab24'))
        self.assertEqual((len(rows), rows[91]['id']), (135, ID))
        self.assertEqual(len(teaching_inventory.catalog()), 482)

    def test_practice_teaches_temporary_root_and_target_without_exposing_assignment_line(self):
        lab = load('scenarios/boot-entry-lessons-v7.json')['scenarios'][-1]
        row = load('scenarios/presentation/lpic2.json')[91]
        self.assertEqual(lab['objective'], ' '.join(row['requirements']))
        self.assertEqual(row['status'], 'updated')
        d = lab['goal']['definition']
        self.assertNotEqual(d['initial']['required_root'], d['example']['required_root'])
        self.assertNotEqual(d['initial']['required_release'], d['example']['required_release'])
        self.assertNotIn(d['initial']['reference']['linux'], json.dumps(row))
        self.assertNotEqual(lab['initial']['simulation']['assignment']['input'],
                            d['initial']['reference'])
        self.assertIn('Linux can reach init', row['practice'][1]['observe'])
        self.assertIn('still requests multi-user', row['practice'][2]['observe'])
