"""DKMS teaching/origin regressions; no compiler or learner command execution."""
import copy
import json
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory
import verify_coverage
import native_reopen

def rows(name):return json.loads((ROOT/name).read_text())['scenarios']
class DkmsLessons(unittest.TestCase):
    def test_teaches_per_kernel_lifecycle_and_explicit_scope_before_independent_work(self):
        content=json.loads((ROOT/'scenarios/presentation/lpic2_dkms.json').read_text())
        labs=rows('scenarios/dkms-lessons-v1.json')
        self.assertEqual([r['id']for r in content],[s['id']for s in labs])
        for entry,lab in zip(content,labs,strict=True):
            self.assertEqual(' '.join(entry['requirements']),lab['objective'])
            self.assertGreaterEqual(len(entry['concepts']),4)
            self.assertGreaterEqual(len(entry['worked']),4)
            self.assertEqual(len(entry['practice']),3)
            text=' '.join(c['text']for c in entry['concepts'])
            for word in ('Added','Built','Installed','running','-k','Run'):
                self.assertIn(word,text)
            d=lab['goal']['definition'];self.assertNotEqual(d['assignment']['facts'],d['example']['facts'])
            self.assertFalse(lab['initial']['simulation']['assignment']['work']['checked'])
            self.assertEqual(lab['initial']['simulation']['view'],'introduction')
            self.assertEqual(lab['initial']['simulation']['assignment']['work']['command_draft'],'')
            self.assertNotIn('dkms-workspace ',json.dumps(entry))
            self.assertTrue(all(not g['commands']for g in lab['guide']))
        self.assertIn('outside the simulator',content[1]['concepts'][3]['text'])
    def test_current_aggregate_retains_all_prior_eight_records_and_only_two_additions(self):
        before=rows('scenarios/curriculum-continuation-v2.json');added=rows('scenarios/dkms-lessons-v1.json')
        selected='scenarios/curriculum-continuation-v3.json';self.assertEqual(rows(selected),before+added)
        self.assertEqual(len(rows(selected)),10);self.assertLessEqual((ROOT/selected).stat().st_size,2000000)
        known={selected:{s['id']:s for s in rows(selected)}}
        for origin in ['scenarios/curriculum-continuation-v2.json','scenarios/dkms-lessons-v1.json','scenarios/kernel-build-lessons-v1.json','scenarios/curriculum-continuation-v1.json','scenarios/poe-lessons-v1.json','scenarios/ids-rule-lessons-v1.json']:
            pack={s['id']:s for s in rows(origin)};self.assertTrue(verify_coverage.preserved_authored_pack(origin,pack,known))
            for lab in pack.values():self.assertTrue(teaching_inventory.same_preserved_origin(lab,origin,selected))
            wrong=copy.deepcopy(pack);next(iter(wrong.values()))['title']='Changed'
            with self.assertRaises(ValueError):verify_coverage.preserved_authored_pack(origin,wrong,known)
        for identity in known[selected]:
            wrong=copy.deepcopy(known);wrong[selected][identity]['title']='Corrupt selected record'
            with self.assertRaises(ValueError):verify_coverage.preserved_authored_pack('scenarios/dkms-lessons-v1.json',{s['id']:s for s in added},wrong)
    def test_native_workspace_discloses_results_and_only_applies_worked_values_in_practice(self):
        source=(ROOT/'src/ui/dkms_page.rs').read_text()
        for text in ('Actual console results','Not checked','Needs repair','Show worked answer','if practice','fitting_input','Check configuration'):
            self.assertIn(text,source)
        engine=(ROOT/'src/dkms.rs').read_text()
        for forbidden in ('std::process','Command::new','std::fs','std::net','unsafe {'):
            self.assertNotIn(forbidden,engine)
        self.assertIn('outside this subset',engine)
        self.assertIn('core::execute', (ROOT/'src/dkms_workspace.rs').read_text())
        self.assertNotIn('Read the contract',json.dumps(rows('scenarios/dkms-lessons-v1.json')))
    def test_dkms_cold_case_is_closed_and_legacy_marker_is_unchanged(self):
        root=Path('/fixed-project/.build-artifacts/tmp/native-reopen-dkms')
        legacy=native_reopen.session_marker(root,root.parent,'a'*32)
        marker=native_reopen.session_marker(root,root.parent,'a'*32,'dkms')
        self.assertEqual(marker.pop('case'),'dkms');self.assertEqual(marker,legacy)
        for value in ['dkms /host','dkms;sh','dkms-arbitrary','real-dkms']:
            with self.assertRaises(native_reopen.ReopenError):native_reopen.session_marker(root,root.parent,'a'*32,value)
