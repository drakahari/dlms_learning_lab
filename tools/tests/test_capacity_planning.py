"""Planning teaching/registration retention; typed behavior has independent Rust tests."""
import csv
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
IDS = ['lpic2-capacity-trend-plan', 'lpic2-capacity-import-handover']


def read(name):
    return json.loads((ROOT / name).read_text())


def canonical(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                     ensure_ascii=False).encode()).hexdigest()


class CapacityPlanningTeaching(unittest.TestCase):
    def test_preserves_the_actual_verified402_inputs_and_adds_only_two_tasks(self):
        old = read('scenarios/boot-entry-lessons-v16.json')['scenarios']
        new = read('scenarios/boot-entry-lessons-v17.json')['scenarios']
        self.assertEqual(len(old), 29)
        self.assertEqual(canonical(old), 'c29c2ed9068bd1a33a3f83cc98f96f0cc8bb74cd273bb288e88562b36d0b6f4b')
        self.assertEqual(new[:29], old)
        self.assertEqual([s['id'] for s in new[29:]], IDS)
        rows = read('scenarios/presentation/lpic2.json')
        self.assertEqual(len(rows), 135)
        self.assertEqual(canonical(rows[:96]), '40e1fc433627b455741b66b2aae5a0d6b7a195bd6c2dd80b13597a1c09b0a64f')
        self.assertEqual([r['id'] for r in rows[96:98]], IDS)
        prefix = (ROOT / 'src/course/metadata.rs').read_bytes().split(b'\npub(super) fn current404_index', 1)[0]
        self.assertEqual(hashlib.sha256(prefix).hexdigest(),
                         '0d81fe2d7ea1d295c6cfa8c64b04ba8a7bbafd637163f0f76818ea4175bc9e05')

    def test_unit_uses_actual_prerequisites_and_separate_saved_planning_workspaces(self):
        unit = read('scenarios/capacity-planning-units-v1.json')
        self.assertEqual(unit['version'], 1)
        self.assertEqual(unit['units'][0]['labs'], IDS)
        self.assertEqual(unit['units'][0]['prerequisites'],
                         ['linux-resource-observations', 'linux-capacity-handover'])
        for lab in read('scenarios/boot-entry-lessons-v17.json')['scenarios'][-2:]:
            self.assertEqual(lab['initial']['kind'], 'CapacityPlanning')
            self.assertEqual(lab['unit'], 'linux-capacity-planning')
            definition = lab['goal']['definition']
            self.assertNotEqual(definition['initial']['series'], definition['example']['series'])
            self.assertEqual(len(definition['initial']['demands']), 3)
            self.assertEqual(definition['reference'][-1], {'kind': 'test'})
            self.assertEqual(definition['example_reference'][-1], {'kind': 'test'})
            self.assertFalse(lab['initial']['simulation']['assignment']['checked'])
            self.assertFalse(lab['initial']['simulation']['practice']['checked'])
        first, second = read('scenarios/boot-entry-lessons-v17.json')['scenarios'][-2:]
        self.assertIsNone(first['initial']['simulation']['assignment']['inputs']['offer'])
        self.assertEqual(second['initial']['simulation']['assignment']['inputs']['offer'],
                         'bulk-expansion', 'Retain the intentional inherited-plan error')

    def test_teaches_stock_growth_units_delivery_and_cases_beside_actual_controls(self):
        rows = read('scenarios/presentation/lpic2.json')[96:98]
        for row in rows:
            self.assertGreaterEqual(len(row['concepts']), 6)
            self.assertGreaterEqual(len(row['worked']), 5)
            self.assertEqual(len(row['practice']), 3)
            self.assertEqual(len(row['requirements']), 3)
            self.assertTrue(all(set(w) == {'label', 'value', 'explanation'} for w in row['worked']))
            action = ' '.join(p['action'] for p in row['practice'])
            self.assertIn('Growth window', action)
            self.assertIn('Order day', action)
            self.assertIn('Reserve', action)
            self.assertIn('test', action.lower())
            self.assertNotIn('plan-action', action)
            text = ' '.join(c['text'] for c in row['concepts'])
            self.assertIn('projection', text.lower())
            self.assertIn('workload', text.lower())
            self.assertIn('day', text)
        self.assertIn('cleanup', str(rows[1]))
        self.assertIn('not proof that usage will keep falling', str(rows[1]))

    def test_native_graph_and_inputs_use_existing_controls_and_distinct_result_states(self):
        text = (ROOT / 'src/ui/capacity_planning_page.rs').read_text()
        for label in ['Growth window start', 'Growth window end', 'Order day',
                      'Keep free', 'Test example plan', 'Test plan',
                      'Plan not tested', 'Plan needs attention', 'Projection limited']:
            self.assertIn(label, text)
        self.assertIn('controls::singleline_fitting', text)
        self.assertIn('.char_limit(24)', text)
        self.assertIn('controls::popup_content_clip', text)
        self.assertIn('A passing workload alone does not pass the plan', text)
        self.assertIn('Solid circles:', text)
        self.assertIn('Dashed line:', text)
        self.assertIn('Squares:', text)

    def test_documentation_keeps_simulation_and_acceptance_limits_explicit(self):
        text = (ROOT / 'docs/CAPACITY_PLANNING_LESSONS.md').read_text()
        for term in ['fictional credits', 'every** day', 'before delivery',
                     'not trusted cached', 'not a Linux command',
                     '200.2', 'physical-device', 'novice acceptance']:
            self.assertIn(term, text)


class CapacityPlanningCoverage(unittest.TestCase):
    def test_only_the_actual_volume_planning_references_receive_partial_credit(self):
        with (ROOT / 'docs/SKILL_COVERAGE.csv').open(newline='') as f:
            rows = [r for r in csv.DictReader(f)
                    if r['blueprint'] == 'LPIC-2 v4.5' and r['parent'] == '200.2']
        partial = {'200.2', '200.2.K2', '200.2.K3', '200.2.K4', '200.2.T2', '200.2.T3'}
        self.assertEqual({r['reference'] for r in rows if r['status'] == 'Partial practice'}, partial)
        for row in rows:
            self.assertIn('No infrastructure collection', row['remaining'])
            self.assertIn('full objective remains incomplete', row['remaining'])
            if row['reference'] in partial:
                self.assertEqual(row['implemented_scenarios'].split(';'), IDS)
                self.assertIn('tests/capacity_planning_content.rs', row['evidence_tests'])
            else:
                self.assertEqual(row['status'], 'Open')
                self.assertEqual(row['implemented_scenarios'], '')
                self.assertEqual(row['evidence_tests'], '')
