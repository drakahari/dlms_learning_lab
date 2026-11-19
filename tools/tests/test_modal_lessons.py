"""Current modal lesson/content boundaries; never launches an editor or shell."""
import copy
import json
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
IDS=['lpic1-modal-setting-repair','lpic1-modal-record-maintenance','lpic1-modal-recovery']
def read(name):return json.loads((ROOT/name).read_text())
class ModalLessons(unittest.TestCase):
    def test_normal_catalog_registers_three_distinct_activities_and_exact_predecessor(self):
        registered=coverage.registered_pack_paths(ROOT,'current')
        self.assertEqual(len(registered),128)
        self.assertIn('scenarios/boot-entry-lessons-v53.json',registered)
        old=read('scenarios/boot-entry-lessons-v37.json')['scenarios']
        new=read('scenarios/modal-editing-lessons-v1.json')['scenarios']
        combined=read('scenarios/boot-entry-lessons-v38.json')['scenarios']
        self.assertEqual(len(old),72);self.assertEqual(combined,old+new)
        self.assertEqual([s['id']for s in new],IDS)
        self.assertEqual([s['unit']for s in new],['modal-file-editing']*3)
        self.assertEqual(new[0]['goal']['definition']['initial']['requirements'][0]['kind'],'Settings')
        self.assertEqual(new[1]['goal']['definition']['initial']['requirements'][0]['kind'],'Records')
        self.assertTrue(any(f['draft']is not None for f in new[2]['goal']['definition']['initial']['files'].values()))
    def test_each_lesson_teaches_its_own_tools_and_uses_separate_unchecked_work(self):
        labs,review=teaching.content();self.assertEqual(len(labs),482)
        for s in read('scenarios/modal-editing-lessons-v1.json')['scenarios']:
            d=s['goal']['definition'];row,_=review[s['id']]
            self.assertNotEqual(d['initial'],d['example'])
            self.assertEqual(''.join(' '.join(row['requirements']).split()),''.join(s['objective'].split()))
            self.assertEqual(row['status'],'updated')
            for view in ['introduction','example','assignment']:
                self.assertGreater(len(row['review'][view]),40);self.assertIn('novice',row['review'][view])
            for section in row['concepts']+row['practice']:
                self.assertFalse(section['title'][0].isdigit(),'Shared renderer supplies numbering')
            self.assertEqual(len(row['practice']),3)
            self.assertTrue(all(p['action']and p['observe']for p in row['practice']))
            self.assertFalse(s['initial']['simulation']['assignment']['checked'])
            self.assertFalse(s['initial']['simulation']['practice']['checked'])
        first=review[IDS[0]][0]
        intro=' '.join(p['text']for p in first['concepts'])
        for concept in ['buffer','NORMAL','INSERT','Escape','Backspace',':w','key=value']:
            self.assertIn(concept,intro)
    def test_source_authoring_is_declarative_and_matches_all_registered_definitions(self):
        authored=read('scenarios/authoring/modal-editing-v1.json')
        self.assertEqual(authored['schema_version'],1);self.assertEqual(len(authored['lessons']),3)
        labs=read('scenarios/modal-editing-lessons-v1.json')['scenarios']
        for source,s in zip(authored['lessons'],labs,strict=True):
            self.assertEqual(source['definition'],s['goal']['definition'])
            self.assertEqual(source['scenario']['objective'],s['objective'])
            self.assertEqual(source['scenario']['id'],s['id'])
            self.assertTrue(source['solution']);self.assertTrue(source['definition']['example_solution'])
            self.assertTrue(all(e['kind']in {'Key','Insert','Open','Check'}for e in source['solution']))
    def test_new_consolidation_pins_reject_changed_old_or_new_values(self):
        target='scenarios/boot-entry-lessons-v38.json'
        known={target:{s['id']:s for s in read(target)['scenarios']}}
        for origin in ['scenarios/boot-entry-lessons-v37.json','scenarios/modal-editing-lessons-v1.json']:
            pack={s['id']:s for s in read(origin)['scenarios']}
            self.assertTrue(coverage.preserved_authored_pack(origin,pack,known))
            bad=copy.deepcopy(pack);next(iter(bad.values()))['objective']+=' changed'
            with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):
                coverage.preserved_authored_pack(origin,bad,known)
    def test_core_and_native_workspace_have_no_host_execution_bridge(self):
        for name in ['src/modal_text.rs','src/modal_editing.rs','src/modal_workspace.rs','src/modal_source.rs','src/ui/modal_editor.rs','src/ui/modal_page.rs']:
            text=(ROOT/name).read_text()
            for forbidden in ['std::process','Command::new','std::fs','TcpStream','UnixStream','/usr/bin/vi','modal_editing_reference.py']:
                self.assertNotIn(forbidden,text,name)
        source=(ROOT/'src/modal_source.rs').read_text()
        self.assertIn('Unsupported', (ROOT/'src/modal_editing.rs').read_text())
        self.assertIn('modal-lab ',source)
    def test_current_issuer_map_keeps_unsupported_editor_scope_open(self):
        result=coverage.validate(ROOT,catalog='current')
        self.assertEqual(result['registered_labs'],482)
        self.assertEqual(result['references'],1240)
        import csv
        with (ROOT/'docs/SKILL_COVERAGE.csv').open()as f:
            rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='LPIC-1 v5.0'and r['parent']=='103.8'}
        self.assertEqual(rows['103.8']['status'],'Partial practice')
        for ref,status in [('103.8.K4','Supporting explanation'),('103.8.K5','Partial practice'),('103.8.T7','Partial practice')]:
            self.assertEqual(rows[ref]['status'],status)
            self.assertEqual(rows[ref]['implemented_scenarios'],'lpic1-editor-handover')
            self.assertIn('general motion operators',rows[ref]['remaining'])
        for ref in ['103.8.K1','103.8.K2','103.8.K3']:
            self.assertEqual(rows[ref]['status'],'Partial practice')
            self.assertEqual(rows[ref]['implemented_scenarios'].split(';'),IDS)
