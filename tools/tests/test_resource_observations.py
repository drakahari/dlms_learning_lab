"""Recorded-host lesson retention and teaching contract; behavior has Rust tests."""
import hashlib
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
IDS = ['lpic2-resource-cpu-intervals', 'lpic2-resource-swap-activity',
       'lpic2-resource-restart-handover']


def read(name):
    return json.loads((ROOT / name).read_text())


class ResourceObservationTeaching(unittest.TestCase):
    def test_retains_previous_pack_and_every_previous_presentation(self):
        old = read('scenarios/boot-entry-lessons-v15.json')['scenarios']
        new = read('scenarios/boot-entry-lessons-v16.json')['scenarios']
        self.assertEqual(len(old), 26)
        self.assertEqual(new[:26], old)
        self.assertEqual([s['id'] for s in new[26:]], IDS)
        rows = read('scenarios/presentation/lpic2.json')
        self.assertEqual(len(rows), 135)
        prefix = json.dumps(rows[:93], sort_keys=True, separators=(',', ':'),
                            ensure_ascii=False).encode()
        self.assertEqual(hashlib.sha256(prefix).hexdigest(),
                         'de51e122505d163c32aa554f0ece10ea2ccbeb99041332fa60c29d3028562878')
        self.assertEqual([r['id'] for r in rows[93:96]], IDS)

    def test_teaches_measurement_before_the_independent_report(self):
        rows = read('scenarios/presentation/lpic2.json')[93:96]
        for row in rows:
            self.assertGreaterEqual(len(row['concepts']), 5)
            self.assertGreaterEqual(len(row['worked']), 4)
            self.assertEqual(len(row['practice']), 3)
            self.assertEqual(len(row['requirements']), 3)
            self.assertTrue(all(set(w) == {'label', 'value', 'explanation'}
                                for w in row['worked']))
            text = ' '.join(c['text'] for c in row['concepts'])
            self.assertIn('boot', text)
            self.assertIn('interval', text)
            practice = ' '.join(p['action'] for p in row['practice'])
            self.assertIn('vmstat', practice)
            self.assertIn('Check', practice)
            self.assertNotIn('resource-action', practice)
        self.assertIn('us + sy + gu', str(rows[0]))
        self.assertIn('not simply free', str(rows[1]))
        self.assertIn('does not continue through', str(rows[2]))
        self.assertIn('different kinds of evidence', str(rows[2]))

    def test_order_and_real_workspace_do_not_replace_the_task_with_a_quiz(self):
        unit = read('scenarios/resource-observation-units-v1.json')['units'][0]
        self.assertEqual(unit['id'], 'linux-resource-observations')
        self.assertEqual(unit['labs'], IDS)
        self.assertIn('linux-processes', unit['prerequisites'])
        labs = read('scenarios/boot-entry-lessons-v16.json')['scenarios'][-3:]
        for lab in labs:
            self.assertEqual(lab['unit'], unit['id'])
            self.assertEqual(lab['initial']['kind'], 'ResourceObservations')
            definition = lab['goal']['definition']
            self.assertNotEqual(definition['initial']['recording'],
                                definition['example']['recording'])
            draft = lab['initial']['simulation']['assignment']
            self.assertFalse(draft['checked'])
            self.assertTrue(all(not c['value'] and c['evidence'] is None
                                for c in draft['claims']))
            self.assertTrue(all(op.startswith('resource-action ') for op in lab['solution']))

    def test_contract_discloses_replay_and_unimplemented_diagnosis(self):
        text = (ROOT / 'docs/RESOURCE_OBSERVATION_LESSONS.md').read_text()
        for term in ['finite recording replay', 'not a live host',
                     'broader fixed-inventory consistency guard',
                     'stricter than live', 'capacity', '200.2',
                     'No real host monitoring reference was run']:
            self.assertIn(term, text)

    def test_coverage_credits_only_the_assessed_measurement_slice(self):
        with (ROOT / 'docs/SKILL_COVERAGE.csv').open() as stream:
            rows = [r for r in csv.DictReader(stream)
                    if r['blueprint'] == 'LPIC-2 v4.5' and r['parent'] == '200.1']
        partial = {'200.1', '200.1.K1', '200.1.K2', '200.1.K3',
                   '200.1.T3', '200.1.T14', '200.1.T17'}
        self.assertEqual({r['reference'] for r in rows
                          if r['status'] == 'Partial practice'}, partial)
        for row in rows:
            if row['reference'] in partial:
                self.assertEqual(row['implemented_scenarios'].split(';'), IDS)
                self.assertIn('No iostat/iotop/sar', row['remaining'])
                self.assertIn('Capacity200.2 has separate finite volume-planning practice', row['remaining'])
                self.assertNotIn('lpic2-capacity-', row['implemented_scenarios'])
            else:
                self.assertEqual(row['status'], 'Open')
                self.assertEqual(row['implemented_scenarios'], '')
                self.assertEqual(row['evidence_tests'], '')
