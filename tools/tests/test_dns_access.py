"""Authored DNS access teaching; native and state grading have separate evidence."""
import hashlib
import json
from pathlib import Path
import sys
import unittest
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory
import verify_coverage
IDS = ['lpic2-dns-recursion-boundary', 'lpic2-dns-secondary-transfer-policy', 'lpic2-dns-access-handover']
def read(path):
    return json.loads((ROOT / path).read_text())
class DnsAccessTeaching(unittest.TestCase):
    def test_retains_all_409_predecessors_and_103_presentation_values(self):
        old = read('scenarios/boot-entry-lessons-v19.json')['scenarios']
        new = read('scenarios/boot-entry-lessons-v20.json')['scenarios']
        self.assertEqual((len(old), len(new)), (36, 39))
        self.assertEqual(new[:36], old)
        self.assertEqual([s['id'] for s in new[36:]], IDS)
        rows = read('scenarios/presentation/lpic2.json')
        self.assertEqual(len(rows), 135)
        raw = json.dumps(rows[:103], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        self.assertEqual((len(raw), hashlib.sha256(raw).hexdigest()), (486810, 'c6fba95fc2f89665612463e2c130de7f0231c1218312bc2db163bc9b4972ba0e'))
        self.assertEqual(verify_coverage.registered_pack_paths(ROOT, 'current')[-1], 'scenarios/boot-entry-lessons-v53.json')
        self.assertEqual(len(teaching_inventory.catalog()), 482)

    def test_every_lesson_teaches_then_practices_different_facts_before_independent_work(self):
        labs = read('scenarios/boot-entry-lessons-v20.json')['scenarios'][-3:]
        by_id = {r['id']: r for r in read('scenarios/presentation/lpic2.json')}
        rows = [by_id[identifier] for identifier in IDS]
        for lab, row in zip(labs, rows):
            self.assertEqual((lab['id'], row['id']), (IDS[labs.index(lab)], lab['id']))
            self.assertEqual(lab['objective'], ' '.join(row['requirements']))
            self.assertEqual(len(row['concepts']), 4)
            self.assertEqual(len(row['practice']), 3)
            self.assertEqual(len(row['requirements']), 3)
            task, example = lab['goal']['definition']['initial'], lab['goal']['definition']['example']
            self.assertNotEqual(task['facts'], example['facts'])
            self.assertGreaterEqual(len(task['cases']), 6)
            self.assertTrue(any(case['cache'] == 'SuppliedWarm' for case in task['cases']))
            saved = example['reference'][0]['config']
            self.assertEqual(row['worked'][0]['value'], saved)
            self.assertNotIn(task['reference'][0]['config'], json.dumps(row))
            actions = ' '.join(p['action'] for p in row['practice'])
            self.assertIn('Save configuration', actions)
            self.assertIn('Reload configuration', actions)
            self.assertIn('Check example access', actions)
            self.assertNotIn('dns-policy', json.dumps(row))

    def test_unit_follows_dns_and_records_advisory_prerequisites(self):
        unit = read('scenarios/dns-access-units-v1.json')['units'][0]
        self.assertEqual(unit['id'], 'linux-dns-access')
        self.assertEqual(unit['labs'], IDS)
        labs = read('scenarios/boot-entry-lessons-v20.json')['scenarios'][-3:]
        self.assertIn('lpic2-dns-handover', labs[0]['prerequisites'])
        self.assertIn(IDS[0], labs[1]['prerequisites'])
        self.assertIn(IDS[1], labs[2]['prerequisites'])
