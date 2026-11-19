"""Current time-zone admission with exact predecessor and explicit fidelity limits."""
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

IDS = ['lpic1-timezone-system-display', 'lpic1-timezone-process-override']


def read(name):
    return json.loads((ROOT / name).read_text())


class TimezoneLessons(unittest.TestCase):
    def test_retains_every_417_predecessor_and_138_admitted_presentations(self):
        old = read('scenarios/boot-entry-lessons-v22.json')['scenarios']
        current = read('scenarios/boot-entry-lessons-v23.json')['scenarios']
        self.assertEqual((len(old), len(current)), (44, 46))
        self.assertEqual(current[:44], old)
        self.assertEqual([r['id'] for r in current[44:]], IDS)
        rows = read('scenarios/presentation/lpic1.json')
        self.assertEqual(len(rows), 154)
        raw = json.dumps(rows[:138], sort_keys=True, separators=(',', ':'),
                         ensure_ascii=False).encode()
        # Canonical readback of actual retained417R1 source, not a refreshed prefix.
        self.assertEqual((len(raw), hashlib.sha256(raw).hexdigest()),
                         (599860, '5d3a88cb45101df63f7dd37caa69703235508c2522e43b3e77048a7914020ca7'))

    def test_current_registration_and_explicit_prerequisite_order(self):
        paths = verify_coverage.registered_pack_paths(ROOT, 'current')
        self.assertEqual((len(paths), paths[-1]),
                         (128, 'scenarios/boot-entry-lessons-v53.json'))
        labs = teaching_inventory.catalog()
        self.assertEqual(len(labs), 482)
        unit = read('scenarios/timezone-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'], IDS)
        self.assertEqual(unit['prerequisites'], ['linux-localization-reports'])
        source = (ROOT / 'src/course/metadata.rs').read_text() if (ROOT / 'src/course/metadata.rs').exists() else (ROOT / 'src/course.rs').read_text()
        self.assertIn('timezone-units-v1.json', source)
        self.assertIn('current419_index', source)
        self.assertEqual([s['id'] for s, _ in labs if s['unit'] == unit['id']], IDS)

    def test_both_lessons_teach_distinct_examples_and_policy_verification(self):
        rows = {r['id']: r for r in read('scenarios/presentation/lpic1.json')}
        labs = read('scenarios/boot-entry-lessons-v23.json')['scenarios'][-2:]
        for lab in labs:
            with self.subTest(lab=lab['id']):
                d = lab['goal']['definition']
                self.assertNotEqual(d['initial']['facts'], d['example']['facts'])
                self.assertGreaterEqual(len(d['initial']['cases']), 3)
                self.assertGreaterEqual(len(d['example']['cases']), 3)
                row = rows[lab['id']]
                self.assertEqual(row['status'], 'updated')
                self.assertGreaterEqual(len(row['concepts']), 5)
                self.assertGreaterEqual(len(row['worked']), 5)
                self.assertEqual(len(row['practice']), 3)
                self.assertTrue(all(p['action'] and p['observe'] for p in row['practice']))
                self.assertTrue(all(len(row['review'][v]) > 60
                                    for v in ('introduction', 'example', 'assignment')))
                self.assertEqual(lab['objective'], ' '.join(row['requirements']))

    def test_only_supported_time_zone_slices_receive_partial_credit(self):
        with (ROOT / 'docs/SKILL_COVERAGE.csv').open() as f:
            rows = {r['reference']: r for r in csv.DictReader(f)
                    if r['blueprint'] == 'LPIC-1 v5.0'}
        for ref in ('107.3.K2', '107.3.T2', '107.3.T3', '107.3.T7',
                    '107.3.T10', '107.3.T11', '108.1.K3', '108.1.T1',
                    '108.1.T3', '108.1.T6', '108.1.T8'):
            r = rows[ref]
            self.assertEqual(r['status'], 'Partial practice')
            self.assertEqual(r['implemented_scenarios'].split(';'), IDS)
            self.assertIn('tests/timezone_content.rs', r['evidence_tests'])
            self.assertIn('2026', r['remaining'])
        for ref in ('107.3.T1', '107.3.T9', '108.1.K1', '108.1.K2', '108.1.T2', '108.1.T7'):
            self.assertEqual(rows[ref]['status'], 'Open')
            self.assertFalse(rows[ref]['implemented_scenarios'])


if __name__ == '__main__':
    unittest.main()
