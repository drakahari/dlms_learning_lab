"""Persistent-settings teaching and exact preexisting-content preservation."""
import copy
import hashlib
import json
import sys
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory
import verify_coverage
IDS=['lpic1-grub-persistent-default','lpic1-grub-argument-scope']
def canonical(v):return json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
class PersistentGrubTeaching(unittest.TestCase):
    def test_admitted378_origin_and_all_prior_presentations_are_exact(self):
        old=json.loads((ROOT/'scenarios/boot-entry-lessons-v2.json').read_text())['scenarios']
        current=json.loads((ROOT/'scenarios/boot-entry-lessons-v3.json').read_text())['scenarios']
        self.assertEqual(current[:5],old)
        # Exact canonical objects read against Ready378R1 BUILD_INFO, not regenerated.
        self.assertEqual((len(canonical(old)),hashlib.sha256(canonical(old)).hexdigest()),(36809,'59d89cea00a6f499afb64ec8f51620a7414b4e03dd00e14d8e42894dbd4fbdad'))
        rows=json.loads((ROOT/'scenarios/presentation/lpic1.json').read_text())
        self.assertEqual(len(rows),154)
        self.assertEqual((len(canonical(rows[:122])),hashlib.sha256(canonical(rows[:122])).hexdigest()),(522971,'e453d0020dec6257f982911bf715d268bfe5fe40089812b79d661b60c5674c80'))
        known={'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in json.loads((ROOT/'scenarios/boot-entry-lessons-v36.json').read_text())['scenarios']}}
        for origin in ['scenarios/boot-entry-lessons-v1.json','scenarios/boot-entry-lessons-v2.json']:
            original=json.loads((ROOT/origin).read_text())['scenarios'];self.assertTrue(verify_coverage.preserved_authored_pack(origin,{s['id']:s for s in original},known))
            for s in original:self.assertTrue(teaching_inventory.same_preserved_origin(s,origin,'scenarios/boot-entry-lessons-v36.json'))
        changed=copy.deepcopy(known);changed['scenarios/boot-entry-lessons-v36.json'][old[0]['id']]['objective']+=' changed'
        with self.assertRaisesRegex(ValueError,'differs from its bundled scenario'):verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v2.json',{s['id']:s for s in old},changed)
    def test_introductions_practice_and_assignments_teach_actual_distinct_workflows(self):
        labs=json.loads((ROOT/'scenarios/boot-entry-lessons-v3.json').read_text())['scenarios'][5:]
        rows=json.loads((ROOT/'scenarios/presentation/lpic1.json').read_text())[122:124]
        self.assertEqual([s['id']for s in labs],IDS)
        for s,r in zip(labs,rows):
            self.assertEqual(s['id'],r['id']);self.assertEqual(' '.join(r['requirements']),s['objective'])
            d=s['goal']['definition'];a=d['initial'];e=d['example']
            self.assertEqual(a['instructions'],s['objective']);self.assertEqual(s['initial']['simulation']['view'],'introduction')
            self.assertNotIn('generated',s['initial']['simulation']['assignment'])
            self.assertNotEqual(a['reference'],e['reference']);self.assertNotEqual(a['required_root'],e['required_root']);self.assertNotEqual(a['generator']['templates'],e['generator']['templates'])
            self.assertNotEqual(a['starter'],a['reference']);self.assertNotEqual(e['starter'],e['reference'])
            self.assertEqual(json.loads(s['solution'][0].split('\n',1)[1]),{'source':a['reference'],'generate':True})
            actions=' '.join(p['action']for p in r['practice']);self.assertIn('Generate menu · simulation',actions);self.assertIn('Test generated menu · simulation',actions);self.assertIn('Inspect generated menu',actions)
            self.assertEqual(len(r['practice']),4);self.assertEqual(len(r['concepts']),3)
            self.assertIn(e['reference'],[w['value']for w in r['worked']]);self.assertNotIn(json.dumps(a['reference']),json.dumps(r))
            self.assertIn('normal',json.dumps(r['concepts']));self.assertIn('recovery',json.dumps(r['concepts']))
    def test_normal_registry_prerequisites_and_capacity_remain_bounded(self):
        paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual(len(paths),128);self.assertEqual(paths[-1],'scenarios/boot-entry-lessons-v53.json')
        labs=teaching_inventory.catalog();self.assertEqual(len(labs),482)
        u=json.loads((ROOT/'scenarios/grub-settings-units-v1.json').read_text())['units'][0];self.assertEqual(u['labs'],IDS);self.assertEqual(u['prerequisites'],['linux-grub-menu-recovery'])
        self.assertEqual([s['id']for s,_ in labs if s['unit']==u['id']],IDS)
        self.assertIn('16 kernel arguments',(ROOT/'src/grub_settings.rs').read_text())
if __name__=='__main__':unittest.main()
