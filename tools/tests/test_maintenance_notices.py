"""Maintenance lesson retention, teaching separation and conservative coverage.

Behavior, parser isolation and alternate solutions are exercised in Rust, not
reimplemented here. These checks guard declarative authoring/coverage boundaries.
"""
import csv
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
IDS = ['lpic2-maintenance-login-notices', 'lpic2-maintenance-restart-handover']


def read(name):
    return json.loads((ROOT / name).read_text())


class MaintenanceNoticeTeaching(unittest.TestCase):
    def test_retains_verified404_scenarios_and_individually_reviewed_presentations(self):
        old = read('scenarios/boot-entry-lessons-v17.json')['scenarios']
        new = read('scenarios/boot-entry-lessons-v18.json')['scenarios']
        self.assertEqual(len(old), 31)
        self.assertEqual(new[:31], old)
        self.assertEqual([r['id'] for r in new[31:]], IDS)
        rows = read('scenarios/presentation/lpic2.json')
        self.assertEqual(len(rows), 135)
        canonical = json.dumps(rows[:98], sort_keys=True, separators=(',', ':'), ensure_ascii=False)
        self.assertEqual(hashlib.sha256(canonical.encode()).hexdigest(),
                         '65d03ee75a44731896d281d908cab23ccfa33a47c335bcc01c52aae0f40c0ab7')
        prefix = (ROOT / 'src/course/metadata.rs').read_bytes().split(
            b'\n\npub(super) fn current406_index', 1)[0]
        self.assertEqual(hashlib.sha256(prefix).hexdigest(),
                         '307f7a2eb3e36e768ff071ebb391f19a4ef8115a825bbdd673f249adb5988ec2')
        for row in rows[98:100]:
            self.assertEqual(row['status'], 'updated')
            self.assertTrue(all('native' in value for value in row['review'].values()))

    def test_real_prerequisites_and_separate_unsolved_practice_assignment(self):
        unit = read('scenarios/maintenance-notice-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'], IDS)
        self.assertEqual(unit['prerequisites'], ['linux-local-mail-routing', 'linux-capacity-planning'])
        labs = read('scenarios/boot-entry-lessons-v18.json')['scenarios'][-2:]
        for lab in labs:
            self.assertEqual(lab['initial']['kind'], 'MaintenanceNotice')
            definition = lab['goal']['definition']
            self.assertNotEqual(definition['initial']['required'], definition['example']['required'])
            for mode in ('assignment', 'practice'):
                self.assertFalse(lab['initial']['simulation'][mode]['checked'])
                self.assertEqual(lab['initial']['simulation'][mode]['stdin'], '')
                self.assertIsNone(lab['initial']['simulation'][mode]['last_run'])
        first, second = [lab['goal']['definition']['initial'] for lab in labs]
        self.assertFalse(first['requires_restart'])
        self.assertEqual(first['starter'], [])
        self.assertTrue(second['requires_restart'])
        self.assertEqual(second['starter'][0]['kind'], 'schedule')
        self.assertNotEqual(second['starter'][0]['after_minutes'],
                            second['required']['starts'] - second['facts']['minute'])

    def test_teaches_actions_without_claiming_wall_stdout_or_linux_bulletin_grammar(self):
        rows = read('scenarios/presentation/lpic2.json')[98:100]
        for row in rows:
            concepts = ' '.join(c['text'] for c in row['concepts'])
            self.assertIn('not a Linux', concepts)
            self.assertIn('wall', concepts)
            self.assertIn('motd', concepts)
            self.assertEqual(len(row['practice']), 3)
            actions = ' '.join(p['action'] for p in row['practice'])
            self.assertIn('Save file', actions)
            self.assertIn('Fill worked notice', actions)
            self.assertIn('Check example notices', actions)
            self.assertIn('wrong announcement', ' '.join(p['observe'] for p in row['practice']))
            worked = ' '.join(w['explanation'] for w in row['worked'])
            self.assertIn('empty', worked)
            self.assertIn('separately', worked)
            self.assertNotIn('notice-action', str(row))
            self.assertEqual(len(row['requirements']), 3)

    def test_partial_map_does_not_credit_unimplemented_channels_or_restart_operation(self):
        with (ROOT / 'docs/SKILL_COVERAGE.csv').open(newline='') as f:
            rows = [r for r in csv.DictReader(f)
                    if r['blueprint'] == 'LPIC-2 v4.5' and r['parent'] == '206.3']
        partial = {'206.3', '206.3.K1', '206.3.K2', '206.3.T3', '206.3.T4', '206.3.T5'}
        self.assertEqual({r['reference'] for r in rows if r['status'] == 'Partial practice'}, partial)
        self.assertEqual({r['reference'] for r in rows if r['status'] == 'Supporting explanation'}, {'206.3.T1'})
        self.assertEqual({r['reference'] for r in rows if r['status'] == 'Open'}, {'206.3.T2', '206.3.T6'})
        for row in rows:
            self.assertIn('full objective remains incomplete', row['remaining'])
            self.assertIn('No issue.net/SSH', row['remaining'])
            if row['status'] == 'Open':
                self.assertEqual(row['implemented_scenarios'], '')
                self.assertEqual(row['evidence_tests'], '')
            else:
                self.assertEqual(row['implemented_scenarios'].split(';'), IDS)
