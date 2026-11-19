"""Current Sieve registration, teaching and isolation guards; no mail execution."""
import csv
import hashlib
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage

IDS = ['lpic2-sieve-header-filing', 'lpic2-sieve-compound-filter',
       'lpic2-sieve-rule-order']


def read(name):
    return json.loads((ROOT / name).read_text())


class SieveLessons(unittest.TestCase):
    def test_registration_preserves_exact_predecessors_and_pack_limits(self):
        paths = coverage.registered_pack_paths(ROOT, 'current')
        self.assertEqual((len(paths), paths[-1]),
                         (128, 'scenarios/boot-entry-lessons-v53.json'))
        old = read('scenarios/boot-entry-lessons-v46.json')['scenarios']
        added = read('scenarios/sieve-filter-lessons-v1.json')['scenarios']
        self.assertEqual(len(old), 94)
        self.assertEqual([s['id'] for s in added], IDS)
        self.assertEqual(read('scenarios/boot-entry-lessons-v47.json')['scenarios'], old + added)
        self.assertLess((ROOT / paths[-1]).stat().st_size, 2_000_000)
        labs = teaching.catalog()
        self.assertEqual(len(labs), 482)
        self.assertEqual([s['id'] for s, _ in labs][467:470], IDS)
        unit = read('scenarios/sieve-filter-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'], IDS)
        self.assertEqual(unit['prerequisites'],
                         ['linux-local-mail-routing', 'linux-smtp-routing'])

    def test_all_three_screens_have_reviewed_subject_specific_content(self):
        rows = read('scenarios/presentation/lpic2.json')
        self.assertEqual(len(rows), 135)
        retained = json.dumps(rows[:120], sort_keys=True,
                              separators=(',', ':'), ensure_ascii=False).encode()
        self.assertEqual(hashlib.sha256(retained).hexdigest(),
                         'e82212f71ac743fa961182858613a9fc195c51c86776a0de19e13133bd1b39fb')
        for lab, row in zip(read('scenarios/sieve-filter-lessons-v1.json')['scenarios'], rows[120:123]):
            self.assertEqual(row['id'], lab['id'])
            self.assertEqual(row['source'], 'scenarios/sieve-filter-lessons-v1.json')
            self.assertEqual(row['status'], 'updated')
            for section in ['concepts', 'practice', 'requirements']:
                self.assertEqual(len(row[section]), 3)
            self.assertTrue(row['worked'])
            for phase in ['introduction', 'example', 'assignment']:
                self.assertGreater(len(row['review'][phase]), 60)
            self.assertTrue(all(p['action'] and p['observe'] for p in row['practice']))

    def test_behavior_cases_and_drafts_are_distinct_and_initially_ungraded(self):
        labs = read('scenarios/sieve-filter-lessons-v1.json')['scenarios']
        for lab in labs:
            definition = lab['goal']['definition']
            assignment, example = definition['assignment'], definition['example']
            self.assertEqual(len(assignment['cases']), 8)
            self.assertEqual(len(example['cases']), 4)
            self.assertNotEqual(assignment['reference'], example['reference'])
            self.assertNotEqual(assignment['cases'], example['cases'])
            self.assertNotEqual(assignment['starter'], assignment['reference'])
            for name, task in [('assignment', assignment), ('practice', example)]:
                scope = lab['initial']['simulation'][name]
                self.assertEqual(scope['draft'], task['starter'])
                self.assertFalse(scope['checked'])
                self.assertFalse(scope['running'])
                self.assertEqual(scope['results'], [])
            self.assertGreater(len({tuple(c['expected']) for c in assignment['cases']}), 1)
        self.assertEqual({c['id'] for c in labs[0]['goal']['definition']['assignment']['cases']},
                         {'staff', 'mixed', 'subject', 'prefix', 'suffix', 'missing', 'duplicate', 'empty'})
        self.assertIn('1024', json.dumps(labs[1]['goal']['definition']['assignment']))
        starter = labs[2]['goal']['definition']['assignment']['starter']
        self.assertEqual(starter.count('if header'), 2)
        self.assertTrue(starter.endswith('keep;\n'))

    def test_only_implemented_partial_scope_is_credited_and_core_has_no_host_bridge(self):
        with (ROOT / 'docs/SKILL_COVERAGE.csv').open() as f:
            rows = [r for r in csv.DictReader(f)
                    if r['blueprint'] == 'LPIC-2 v4.5' and r['parent'] == '211.2']
        self.assertTrue(rows)
        vacation = ['lpic2-sieve-vacation-safety', 'lpic2-sieve-vacation-routing-recovery']
        prior = IDS + ['lpic2-sieve-sender-domain', 'lpic2-sieve-recipient-recovery', 'lpic2-sieve-envelope-recipient', 'lpic2-sieve-redirect-recovery', 'lpic2-sieve-rejection-priority']
        self.assertEqual({r['reference'] for r in rows}, {'211.2','211.2.K1','211.2.K2','211.2.K3','211.2.T1','211.2.T2','211.2.T3'})
        for row in rows:
            if row['reference'] == '211.2.K3':
                self.assertEqual(row['status'], 'Supporting explanation')
                self.assertEqual(row['implemented_scenarios'], 'lpic2-procmail-sieve-migration')
                self.assertIn('not a procmail interpreter', row['remaining'])
                self.assertEqual(row['evidence_tests'], 'tests/procmail_migration.rs')
            else:
                self.assertEqual(row['status'], 'Partial practice')
                expected = vacation if row['reference'] == '211.2.T3' else prior + vacation
                if row['reference'] == '211.2':
                    expected = expected + ['lpic2-procmail-sieve-migration']
                self.assertEqual(row['implemented_scenarios'].split(';'), expected)
        for name in ['lex', 'parse', 'eval', 'workspace', 'source']:
            source = (ROOT / f'src/sieve_{name}.rs').read_text()
            for forbidden in ['std::fs', 'std::process', 'Command::new', 'TcpStream', 'UdpSocket', 'unsafe {']:
                self.assertNotIn(forbidden, source)

    def test_changed_prefix_or_tail_is_rejected_as_preserved_origin(self):
        lab = read('scenarios/sieve-filter-lessons-v1.json')['scenarios'][0]
        origin = 'scenarios/sieve-filter-lessons-v1.json'
        selected = 'scenarios/boot-entry-lessons-v47.json'
        self.assertTrue(teaching.same_preserved_origin(lab, origin, selected))
        original = Path.read_text
        for index in [0, 94]:
            bad = read(selected)
            bad['scenarios'][index]['title'] = 'Unreviewed replacement'

            def altered(path, *args, **kwargs):
                return json.dumps(bad) if path == ROOT / selected else original(path, *args, **kwargs)

            with patch.object(Path, 'read_text', altered), self.assertRaisesRegex(ValueError, 'Sieve consolidation changed'):
                teaching.same_preserved_origin(lab, origin, selected)
