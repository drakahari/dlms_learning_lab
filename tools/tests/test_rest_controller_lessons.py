"""Authored resources/teaching only; no real HTTP, process or network execution."""
import hashlib
import json
from pathlib import Path
import sys
import unittest

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory
import verify_coverage

IDS=['ccna-rest-change-request','ccna-rest-profile-handover']
def read(path):return json.loads((ROOT/path).read_text())
def pin(value):
    raw=json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
    return hashlib.sha256(raw).hexdigest()

class RestControllerLessons(unittest.TestCase):
    def test_previous66_authored_scenarios_remain_exact_and_current_registry_is_bounded(self):
        old=read('scenarios/boot-entry-lessons-v34.json')['scenarios']
        current=read('scenarios/boot-entry-lessons-v36.json')['scenarios']
        self.assertEqual((len(old),len(current)),(66,69))
        self.assertEqual(current[:66],old)
        self.assertEqual(pin(old),'723eb6b492341e07c8e58db8189733d9d2aa0317d05b46747ecd426b6c651729')
        self.assertEqual(pin(read('scenarios/presentation/ccna.json')[:102]),'6d86b4455e8ea1ce576d7de052f101bd44c737cb07a3f4c1ca0766ad486033f0')
        known={'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in current}}
        self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v34.json',{s['id']:s for s in old},known))
        bad=json.loads(json.dumps(old));bad[0]['objective']+=' changed'
        with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):
            verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v34.json',{s['id']:s for s in bad},known)
        paths=verify_coverage.registered_pack_paths(ROOT,'current')
        self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
        self.assertEqual(len(teaching_inventory.catalog()),482)

    def test_prerequisites_separate_worlds_and_authored_drafts_match_real_sources(self):
        labs={s['id']:s for s,_ in teaching_inventory.catalog()}
        previews=read('tests/fixtures/rest-controller-preview-v1.json')['scenarios']
        self.assertEqual([s['id']for s in previews],IDS)
        for preview in previews:
            if preview['id']==IDS[0]:preview['prerequisites']=['ccna-json-inventory-handover']
            self.assertEqual(preview,labs[preview['id']])
            d=preview['goal']['definition']
            self.assertNotEqual(d['initial']['api'],d['example']['api'])
            self.assertNotEqual(d['initial']['initial'],d['example']['initial'])
            self.assertEqual(preview['initial']['simulation']['assignment']['changes'],[])
            self.assertFalse(preview['initial']['simulation']['assignment']['checked'])
        unit=read('scenarios/rest-controller-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'],IDS)
        self.assertEqual(unit['prerequisites'],['structured-network-inventory'])

    def test_each_teaching_screen_has_concrete_actions_and_independent_requirements(self):
        labs={s['id']:s for s,_ in teaching_inventory.catalog()}
        entries={e['id']:e for e in read('scenarios/presentation/ccna.json')}
        for lab in IDS:
            e=entries[lab]
            self.assertEqual(' '.join(e['requirements']),labs[lab]['objective'])
            self.assertEqual((len(e['concepts']),len(e['practice'])),(4,3))
            self.assertGreaterEqual(len(e['worked']),3)
            for view in ['introduction','example','assignment']:self.assertGreater(len(e['review'][view]),40)
            text=json.dumps(e)
            for term in ['Send request','Check network','PATCH','PUT']:self.assertIn(term,text)
            for bad in ['Read the contract','fresh virtual workspace','local scalar']:self.assertNotIn(bad,text)
        # The first worked resource is team on demo, not the graded staff resource.
        self.assertTrue(any('/profiles/team' in w['value']for w in entries[IDS[0]]['worked']))
        self.assertFalse(any('/profiles/staff' in w['value']for w in entries[IDS[0]]['worked']))

    def test_model_scope_and_resume_do_not_deserialize_response_as_authority(self):
        model=(ROOT/'src/rest_controller.rs').read_text()
        workspace=(ROOT/'src/rest_workspace.rs').read_text()
        self.assertIn('pub const MAX_BODY: usize = 4096',model)
        self.assertIn('pub const MAX_PROFILES: usize = 12',model)
        start=model.index('pub struct Response')
        response=model[model.rfind('#[derive',0,start):model.index('enum Target')]
        self.assertNotIn('Deserialize',response)
        self.assertIn('pub changes: Vec<api::Request>',workspace)
        self.assertIn('!response.changed',workspace)
        self.assertIn('spec.project(store)?',model)
        for forbidden in ['std::process','std::fs','TcpStream','UdpSocket','Command::new']:self.assertNotIn(forbidden,model+workspace)

if __name__=='__main__':unittest.main()
