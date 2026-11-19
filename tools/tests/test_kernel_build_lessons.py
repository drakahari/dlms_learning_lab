"""Focused teaching/provenance checks; never run a compiler or learner command."""
import copy
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory
import verify_coverage

def rows(name):
    return json.loads((ROOT / name).read_text())['scenarios']

class KernelBuildLessons(unittest.TestCase):
    def test_teaches_saved_inputs_module_choices_and_distinct_practice_before_assignment(self):
        content = json.loads((ROOT/'scenarios/presentation/lpic2_kernel_build.json').read_text())
        lessons = rows('scenarios/kernel-build-lessons-v1.json')
        self.assertEqual([r['id'] for r in content], [s['id'] for s in lessons])
        for entry, lab in zip(content, lessons, strict=True):
            self.assertEqual(' '.join(entry['requirements']), lab['objective'])
            self.assertGreaterEqual(len(entry['concepts']), 4)
            self.assertGreaterEqual(len(entry['worked']), 4)
            self.assertGreaterEqual(len(entry['practice']), 3)
            text = ' '.join(c['text'] for c in entry['concepts'])
            for word in ('saved', 'module', 'image', 'initramfs'):
                self.assertIn(word, text)
            definition = lab['goal']['definition']
            self.assertNotEqual(definition['assignment']['sources'], definition['example']['sources'])
            self.assertFalse(lab['initial']['simulation']['assignment']['work']['checked'])
            self.assertEqual(lab['initial']['simulation']['view'], 'introduction')
            self.assertNotIn('kernel-workspace ', json.dumps(entry))
            self.assertTrue(all(not g['commands'] for g in lab['guide']))
            self.assertEqual(len(lab['guide']), len(lab['steps']))
        boot = lessons[1]['goal']['definition']
        self.assertNotEqual(boot['assignment']['root']['uuid'], boot['example']['root']['uuid'])
        self.assertIn('/choose-staged-image', boot['assignment']['initial_entry'])

    def test_selected_aggregate_retains_exact_previous_six_and_two_authored_additions(self):
        before = rows('scenarios/curriculum-continuation-v1.json')
        added = rows('scenarios/kernel-build-lessons-v1.json')
        selected = 'scenarios/curriculum-continuation-v2.json'
        self.assertEqual(rows(selected), before + added)
        self.assertEqual(len(rows(selected)), 8)
        self.assertLessEqual((ROOT/selected).stat().st_size, 2000000)
        known = {selected: {s['id']:s for s in rows(selected)}}
        for origin in ['scenarios/curriculum-continuation-v1.json', 'scenarios/kernel-build-lessons-v1.json', 'scenarios/poe-lessons-v1.json', 'scenarios/ids-rule-lessons-v1.json']:
            pack = {s['id']:s for s in rows(origin)}
            self.assertTrue(verify_coverage.preserved_authored_pack(origin, pack, known))
            for lab in pack.values():
                self.assertTrue(teaching_inventory.same_preserved_origin(lab, origin, selected))
            wrong = copy.deepcopy(pack)
            next(iter(wrong.values()))['title'] = 'Changed'
            with self.assertRaises(ValueError):
                verify_coverage.preserved_authored_pack(origin, wrong, known)
        for identity in known[selected]:
            wrong = copy.deepcopy(known)
            wrong[selected][identity]['title'] = 'Corrupted selected record'
            with self.assertRaises(ValueError):
                verify_coverage.preserved_authored_pack('scenarios/kernel-build-lessons-v1.json', {s['id']:s for s in added}, wrong)

    def test_native_workspace_keeps_expected_and_actual_separate_and_never_fills_assignment(self):
        code = (ROOT/'src/ui/kernel_build_page.rs').read_text()
        for text in ('Actual build outputs', 'Actual result', 'Unchecked', 'Needs attention', 'Show worked answer', 'if practice', 'configuration_input', 'fitting_input'):
            self.assertIn(text, code)
        self.assertTrue('.id_salt(("kernel-worked",&scenario.id))' in ''.join(code.split()), "Worked disclosure must be salted by scenario identity")
        self.assertNotIn('std::process', code)
        self.assertNotIn('std::fs', code)

if __name__ == '__main__':
    unittest.main()
