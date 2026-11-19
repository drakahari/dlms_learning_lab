"""Authored printing progression and honest partial objective reporting."""
import hashlib
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
IDS = ['lpic1-print-paused-queue', 'lpic1-print-held-request',
       'lpic1-print-destination-handover']


class PrintQueueLessons(unittest.TestCase):
    def test_additive_source_keeps_all_412_predecessor_tail_values(self):
        def labs(name):
            return json.loads((ROOT / 'scenarios' / name).read_text())['scenarios']
        old = labs('boot-entry-lessons-v20.json')
        current = labs('boot-entry-lessons-v21.json')
        self.assertEqual(len(old), 39)
        self.assertEqual(current[:39], old)
        self.assertEqual([r['id'] for r in current[39:]], IDS)

    def test_all_admitted412_lpic1_presentations_are_retained(self):
        rows = json.loads((ROOT / 'scenarios/presentation/lpic1.json').read_text())
        self.assertEqual(len(rows), 154)
        raw = json.dumps(rows[:133], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        self.assertEqual((len(raw), hashlib.sha256(raw).hexdigest()), (573119, 'b8956130097e23c6b892ebbdb64eaf57324757462253ce26f0ac3b1e497c0849'))

    def test_each_worked_command_uses_only_the_different_example_and_has_a_reason(self):
        labs = json.loads((ROOT / 'scenarios/boot-entry-lessons-v21.json').read_text())['scenarios'][-3:]
        rows = {r['id']: r for r in json.loads((ROOT / 'scenarios/presentation/lpic1.json').read_text())}
        for lab in labs:
            with self.subTest(lab=lab['id']):
                definition = lab['goal']['definition']
                independent, example = definition['initial'], definition['example']
                self.assertEqual(independent['id'], lab['id'])
                self.assertNotEqual(independent['facts'], example['facts'])
                row = rows[lab['id']]
                self.assertEqual(row['status'], 'updated')
                self.assertEqual([r['value'] for r in row['worked']], example['reference'])
                self.assertTrue(all(len(r['explanation']) > 35 for r in row['worked']))
                self.assertEqual(len(row['practice']), 3)
                self.assertTrue(all(len(r['action']) > 60 and len(r['observe']) > 60
                                    for r in row['practice']))
                self.assertEqual(len(example['cases']), 4)
                self.assertEqual(len(independent['cases']), 4)

    def test_real_printer_setup_and_cups_config_files_stay_open(self):
        with (ROOT / 'docs/SKILL_COVERAGE.csv').open() as file:
            rows = {r['reference']: r for r in csv.DictReader(file)
                    if r['blueprint'] == 'LPIC-1 v5.0' and r['parent'] == '108.4'}
        for ref in ('108.4', '108.4.K2', '108.4.K3', '108.4.K4', '108.4.T1', '108.4.T3'):
            self.assertEqual(rows[ref]['status'], 'Partial practice')
            self.assertEqual(rows[ref]['implemented_scenarios'].split(';'), IDS)
            self.assertIn('LPD transport', rows[ref]['remaining'])
            self.assertIn('tests/print_queue_content.rs', rows[ref]['evidence_tests'])
        for ref in ('108.4.K1', '108.4.T2'):
            self.assertEqual(rows[ref]['status'], 'Open')
            self.assertEqual(rows[ref]['implemented_scenarios'], '')


if __name__ == '__main__':
    unittest.main()
