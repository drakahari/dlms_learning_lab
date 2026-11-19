"""Locale unit admission, retained content and narrowly bounded objective credit."""
import csv
import hashlib
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory
import verify_coverage

IDS = ['lpic1-locale-report-handover', 'lpic1-encoding-import-repair']


def read(name):
    return json.loads((ROOT / name).read_text())


class LocalizationLessons(unittest.TestCase):
    def test_all_415_predecessor_values_and_136_presentations_remain_exact(self):
        old = read('scenarios/boot-entry-lessons-v21.json')['scenarios']
        current = read('scenarios/boot-entry-lessons-v22.json')['scenarios']
        self.assertEqual((len(old), len(current)), (42, 44))
        self.assertEqual(current[:42], old)
        self.assertEqual([r['id'] for r in current[42:]], IDS)
        rows = read('scenarios/presentation/lpic1.json')
        self.assertEqual(len(rows), 154)
        before = json.dumps(rows[:136], sort_keys=True, separators=(',', ':'),
                            ensure_ascii=False).encode()
        # Pinned from the admitted415 candidate's retained actual source.
        self.assertEqual((len(before), hashlib.sha256(before).hexdigest()),
                         (588509, '1f2ece04c63f14f870dbc8f64594739a594b9601d72765e8b245306c8b5055aa'))

    def test_normal_catalog_registers_both_lessons_and_their_prerequisite_unit(self):
        paths = verify_coverage.registered_pack_paths(ROOT, 'current')
        self.assertEqual((len(paths), paths[-1]),
                         (128, 'scenarios/boot-entry-lessons-v53.json'))
        labs = teaching_inventory.catalog()
        self.assertEqual(len(labs), 482)
        unit = read('scenarios/localization-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'], IDS)
        self.assertEqual(unit['prerequisites'], ['linux-file-foundations'])
        for lab, _ in labs:
            if lab['id'] in IDS:
                self.assertEqual(lab['unit'], unit['id'])

    def test_individual_teaching_has_different_practice_inputs_and_real_actions(self):
        labs = read('scenarios/boot-entry-lessons-v22.json')['scenarios'][-2:]
        rows = {r['id']: r for r in read('scenarios/presentation/lpic1.json')}
        for lab in labs:
            with self.subTest(lab=lab['id']):
                definition = lab['goal']['definition']
                task, example = definition['initial'], definition['example']
                self.assertNotEqual(task['facts'], example['facts'])
                self.assertGreaterEqual(len(task['cases']), 3)
                self.assertGreaterEqual(len(example['cases']), 3)
                row = rows[lab['id']]
                self.assertEqual(row['status'], 'updated')
                self.assertGreaterEqual(len(row['concepts']), 4)
                self.assertGreaterEqual(len(row['worked']), 5)
                self.assertEqual(len(row['practice']), 3)
                for step in row['practice']:
                    self.assertGreater(len(step['action']), 60)
                    self.assertGreater(len(step['observe']), 60)
                self.assertTrue(all(len(row['review'][view]) > 60
                                    for view in ('introduction', 'example', 'assignment')))

    def test_locale_evidence_is_retained_and_unimplemented_utilities_are_not_credited(self):
        with (ROOT / 'docs/SKILL_COVERAGE.csv').open() as stream:
            rows = {r['reference']: r for r in csv.DictReader(stream)
                    if r['blueprint'] == 'LPIC-1 v5.0' and r['parent'] == '107.3'}
        for ref in ('107.3', '107.3.K1', '107.3.T4', '107.3.T5', '107.3.T6',
                    '107.3.T8', '107.3.T12', '107.3.T13', '107.3.T14',
                    '107.3.T15', '107.3.T16'):
            self.assertEqual(rows[ref]['status'], 'Partial practice')
            self.assertTrue(rows[ref]['implemented_scenarios'])
            self.assertIn('tests/localization_content.rs', rows[ref]['evidence_tests'])
        for ref in ('107.3.T1', '107.3.T9'):
            self.assertEqual(rows[ref]['status'], 'Open')
            self.assertEqual(rows[ref]['implemented_scenarios'], '')
            self.assertIn('/etc/timezone writes' if ref.endswith('T1') else 'tzselect', rows[ref]['remaining'])


if __name__ == '__main__':
    unittest.main()
