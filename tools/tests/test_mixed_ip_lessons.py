"""Mixed-family task, explicit prerequisite additions and unchanged prior sources."""
import csv
import hashlib
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory
import verify_coverage


def read(path):
    return json.loads((ROOT / path).read_text())


def pin(value):
    raw = json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
    return len(raw), hashlib.sha256(raw).hexdigest()


class MixedIpLessons(unittest.TestCase):
    def test_all_prior63_sources_and142_teaching_values_remain_exact(self):
        before = read('scenarios/boot-entry-lessons-v31.json')['scenarios']
        now = read('scenarios/boot-entry-lessons-v32.json')['scenarios']
        self.assertEqual((len(before), len(now)), (63, 64))
        self.assertEqual(before, now[:63])
        self.assertEqual(pin(before), (641001, 'b10d4d2a2db1c1dd12d0cc47df627dd1fe0c1e3ddaf2a0b6dab43f04e8f1b6d5'))
        teaching = read('scenarios/presentation/lpic1.json')
        self.assertEqual(len(teaching), 154)
        self.assertEqual(pin(teaching[:142]), (621946, '0629ecc07370ff1bd4422e2187943c1685c92129582046f9b3d1fda7ae12e668'))
        known = {'scenarios/boot-entry-lessons-v36.json': {s['id']: s for s in read('scenarios/boot-entry-lessons-v36.json')['scenarios']}}
        self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v31.json', {s['id']: s for s in before}, known))
        broken = json.loads(json.dumps(before)); broken[0]['objective'] += ' changed'
        with self.assertRaisesRegex(ValueError, 'Changed preserved authored origin'):
            verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v31.json', {s['id']: s for s in broken}, known)

    def test_registered_task_and_teaching_match_actual_supported_controls(self):
        paths = verify_coverage.registered_pack_paths(ROOT, 'current')
        self.assertEqual((len(paths), paths[-1]), (128, 'scenarios/boot-entry-lessons-v53.json'))
        labs, reviews = teaching_inventory.content()
        self.assertEqual(len(labs), 482)
        s = next(s for s, _ in labs if s['id'] == 'lpic1-host-mixed-ip')
        self.assertEqual(s['prerequisites'], ['lpic1-host-handover'])
        unit = read('scenarios/mixed-ip-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'], [s['id']])
        self.assertEqual(unit['prerequisites'], ['linux-host-networking'])
        d = s['goal']['definition']; e = reviews[s['id']][0]
        self.assertEqual(len(d['requirements']), 13)
        self.assertEqual(len(d['example_actions']), 10)
        self.assertNotEqual(d['initial']['profile'], d['example']['profile'])
        self.assertEqual(' '.join(e['requirements']), s['objective'])
        self.assertEqual((len(e['concepts']), len(e['worked']), len(e['practice'])), (4, 7, 3))
        text = json.dumps(e, ensure_ascii=False)
        self.assertIn('Test address · simulator', text)
        self.assertNotIn('Test IPv6', text)
        self.assertNotIn('10.24.0.50', text)
        self.assertNotIn('2001:db8:24::50', text)
        self.assertIn('IPv6 has no broadcast', text)
        self.assertIn('subnet-router anycast', text)
        for view in ['introduction', 'example', 'assignment']:
            self.assertGreater(len(e['review'][view]), 40)

    def test_prerequisite_notes_have_a_closed_schema_and_only_two_opt_in_lessons(self):
        p = ROOT / 'scenarios/presentation/lpic1_foundations.json'
        self.assertLessEqual(p.stat().st_size, 8192)
        notes = read(p)
        self.assertEqual(set(notes), {'version', 'lessons'})
        self.assertEqual(notes['version'], 1)
        self.assertEqual([r['id'] for r in notes['lessons']], ['lpic1-host-address', 'lpic1-host-routing'])
        self.assertEqual({p.name for p in p.parent.glob('*.json')}, {'ccna.json', 'lpic1.json', 'lpic2.json', 'bash.json', 'lpic1_foundations.json', 'lpic1_direct_dns.json', 'lpic2_dns_queries.json'})
        _, effective = teaching_inventory.content()
        base = {r['id']: r for r in read('scenarios/presentation/lpic1.json')}
        for n in notes['lessons']:
            self.assertEqual(set(n), {'id', 'concepts', 'worked', 'introduction_review', 'sources'})
            self.assertEqual((len(n['concepts']), len(n['worked'])), (1, 1))
            self.assertTrue(all(u.startswith('https://www.rfc-editor.org/rfc/') for u in n['sources']))
            self.assertIn(n['introduction_review'], effective[n['id']][0]['review']['introduction'])
            for view in ['example', 'assignment']:
                self.assertEqual(effective[n['id']][0]['review'][view], base[n['id']]['review'][view])

    def test_scope_classification_stays_supporting_and_protocol_gaps_stay_open(self):
        with (ROOT / 'docs/SKILL_COVERAGE.csv').open() as f:
            rows = {r['reference']: r for r in csv.DictReader(f) if r['blueprint'] == 'LPIC-1 v5.0' and r['parent'] == '109.1'}
        self.assertEqual(rows['109.1.K2']['status'], 'Supporting explanation')
        self.assertIn('no separately graded', rows['109.1.K2']['remaining'])
        for key in ['109.1.K1', '109.1.K5', '109.1.K6', '109.1.T2', '109.1.T3']:
            self.assertEqual(rows[key]['status'], 'Partial practice')
            self.assertIn('lpic1-host-mixed-ip', rows[key]['implemented_scenarios'])
        self.assertIn('ND', rows['109.1.K6']['remaining'])
        self.assertIn('ICMP', rows['109.1']['remaining'])


if __name__ == '__main__':
    unittest.main()
