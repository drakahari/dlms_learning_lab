"""Current authored handover boundaries. Never launches Bash or an editor."""
import copy
import csv
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage

ID = 'lpic1-editor-handover'


def read(name):
    return json.loads((ROOT / name).read_text())


class EditorHandoverLessons(unittest.TestCase):
    def test_registered_consolidation_is_the_exact_predecessor_plus_one_distinct_composition(self):
        paths = coverage.registered_pack_paths(ROOT, 'current')
        self.assertEqual(len(paths), 128)
        self.assertIn('scenarios/boot-entry-lessons-v53.json', paths)
        old = read('scenarios/boot-entry-lessons-v38.json')['scenarios']
        new = read('scenarios/editor-handover-lessons-v1.json')['scenarios']
        self.assertEqual(len(old), 75)
        self.assertEqual([s['id'] for s in new], [ID])
        self.assertEqual(read('scenarios/boot-entry-lessons-v39.json')['scenarios'], old + new)
        self.assertEqual(new[0]['initial']['kind'], 'EditorHandover')
        unit = read('scenarios/editor-handover-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'], [ID])
        self.assertEqual(unit['prerequisites'], ['modal-file-editing', 'linux-shell-environment'])

    def test_authored_examples_and_starters_retain_real_work_and_supplied_variable_roles(self):
        authored = read('scenarios/authoring/editor-handover-v1.json')['lessons'][0]
        lab = read('scenarios/editor-handover-lessons-v1.json')['scenarios'][0]
        self.assertEqual(authored['definition'], lab['goal']['definition'])
        self.assertEqual(authored['scenario']['objective'], lab['objective'])
        d = lab['goal']['definition']
        self.assertNotEqual(d['setup']['assignment'], d['setup']['example'])
        self.assertNotEqual(d['editing']['initial'], d['editing']['example'])
        starter = lab['initial']['simulation']
        self.assertIsNone(starter['setup']['assignment']['result'])
        self.assertIsNone(starter['assignment_launch'])
        self.assertNotIn('export EDITOR', starter['setup']['assignment']['source'])
        self.assertFalse(starter['files']['assignment']['checked'])
        supplied = d['setup']['assignment']['cases']
        self.assertEqual(len(supplied), 3)
        self.assertEqual({c['input']['env']['PREFERRED_EDITOR'] for c in supplied}, {'vi', 'vim'})
        self.assertNotIn('EDITOR', supplied[0]['input']['env'])
        self.assertEqual(supplied[1]['input']['env']['EDITOR'], 'nano')
        self.assertEqual(supplied[2]['input']['env']['EDITOR'], '')
        for c in supplied:
            self.assertEqual(c['input']['files'], {})
            self.assertEqual(c['child']['env']['EDITOR'], c['input']['env']['PREFERRED_EDITOR'])
            self.assertTrue(c['parent']['EDITOR']['exported'])
            self.assertEqual(c['child']['env']['TEAM'], c['input']['env']['TEAM'])

    def test_consolidation_pins_refuse_changed_prior_or_new_objects(self):
        target = 'scenarios/boot-entry-lessons-v39.json'
        known = {target: {s['id']: s for s in read(target)['scenarios']}}
        for origin in ['scenarios/boot-entry-lessons-v38.json', 'scenarios/editor-handover-lessons-v1.json']:
            objects = {s['id']: s for s in read(origin)['scenarios']}
            self.assertTrue(coverage.preserved_authored_pack(origin, objects, known))
            changed = copy.deepcopy(objects)
            next(iter(changed.values()))['objective'] += ' altered'
            with self.assertRaisesRegex(ValueError, 'Changed preserved authored origin'):
                coverage.preserved_authored_pack(origin, changed, known)

    def test_presentation_has_the_actual_controls_and_preserves_all_requirements(self):
        labs, entries = teaching.content()
        self.assertEqual(len(labs), 482)
        lab = next(lab for lab, _ in labs if lab['id'] == ID)
        row, _ = entries[ID]
        self.assertEqual(''.join(' '.join(row['requirements']).split()), ''.join(lab['objective'].split()))
        intro = ' '.join(c['text'] for c in row['concepts'])
        for term in ['PREFERRED_EDITOR', 'TEAM', 'export', 'inherited', 'nano', 'Emacs']:
            self.assertIn(term, intro)
        self.assertEqual(len(row['practice']), 3)
        practice = ' '.join(p['action'] for p in row['practice'])
        for control in ['Run setup', 'Fill worked setup', 'Open handover file with selected editor', 'Check example']:
            self.assertIn(control, practice)
        self.assertTrue(all(p['observe'] for p in row['practice']))
        self.assertTrue(all(not c['title'][0].isdigit() for c in row['concepts'] + row['practice']))
        self.assertEqual([r['label'] for r in row['worked'][:2]], ['Supplied editor', 'Supplied team'])

    def test_issuer_credit_is_bounded_practice_and_support_not_complete_editor_coverage(self):
        with (ROOT / 'docs/SKILL_COVERAGE.csv').open() as stream:
            rows = {r['reference']: r for r in csv.DictReader(stream)
                    if r['blueprint'] == 'LPIC-1 v5.0' and r['parent'] == '103.8'}
        for ref in ['103.8.K5', '103.8.T7']:
            self.assertEqual(rows[ref]['status'], 'Partial practice')
            self.assertEqual(rows[ref]['implemented_scenarios'], ID)
            self.assertIn('VISUAL', rows[ref]['remaining'])
        self.assertEqual(rows['103.8.K4']['status'], 'Supporting explanation')
        self.assertIn('no nano or Emacs interaction', rows['103.8.K4']['verified_scope'])
        self.assertEqual(rows['103.8']['status'], 'Partial practice')
        self.assertIn('general motion operators', rows['103.8']['remaining'])

    def test_new_runtime_composition_has_no_host_execution_bridge(self):
        for name in ['src/editor_handover.rs', 'src/editor_handover_source.rs', 'src/ui/editor_handover_page.rs']:
            text = (ROOT / name).read_text()
            for forbidden in ['std::process', 'Command::new', 'std::fs', 'TcpStream', 'UnixStream', '/usr/bin/vi', 'modal_editing_reference.py']:
                self.assertNotIn(forbidden, text, name)
        text = (ROOT / 'src/ui/editor_handover_page.rs').read_text()
        self.assertIn('job.runner.advance(16)', text)
        self.assertIn('job.runner.stop()', text)
        self.assertNotIn('env::run(', text, 'The GUI must retain cooperative Run/Stop')
