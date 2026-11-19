"""Authored BIOS teaching and preserved source origins; no external execution."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory
import verify_coverage
ID='lpic1-grub-bios-install'
def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
class BiosLoaderTeaching(unittest.TestCase):
    def test_every_admitted380_boot_object_and_presentation_is_exact(self):
        old=json.loads((ROOT/'scenarios/boot-entry-lessons-v3.json').read_text())['scenarios']
        now=json.loads((ROOT/'scenarios/boot-entry-lessons-v4.json').read_text())['scenarios']
        self.assertEqual((len(old),len(now)),(7,8));self.assertEqual(now[:7],old)
        self.assertEqual((len(canonical(old)),hashlib.sha256(canonical(old)).hexdigest()),(52250,'f43afb791578d4d74ce4e321ad80fd8749396ab2f468dc01bb9d4ef6d3dbc066'))
        rows=json.loads((ROOT/'scenarios/presentation/lpic1.json').read_text());self.assertEqual(len(rows),154)
        self.assertEqual((len(canonical(rows[:124])),hashlib.sha256(canonical(rows[:124])).hexdigest()),(534240,'98a9c1dfa36bb9beae12ba34ac85ed4f41ea15fd3df02dcd067403b69a609559'))
        target='scenarios/boot-entry-lessons-v36.json';known={target:{s['id']:s for s in json.loads((ROOT/target).read_text())['scenarios']}}
        for origin in ['scenarios/boot-entry-lessons-v1.json','scenarios/boot-entry-lessons-v2.json','scenarios/boot-entry-lessons-v3.json']:
            source=json.loads((ROOT/origin).read_text())['scenarios'];self.assertTrue(verify_coverage.preserved_authored_pack(origin,{s['id']:s for s in source},known))
            for lab in source:self.assertTrue(teaching_inventory.same_preserved_origin(lab,origin,target))
        bad=copy.deepcopy(known);bad[target][old[0]['id']]['objective']+=' changed'
        with self.assertRaisesRegex(ValueError,'differs from its bundled scenario'):verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v3.json',{s['id']:s for s in old},bad)
    def test_teaching_uses_distinct_workshop_real_controls_and_meaningful_failure(self):
        s=json.loads((ROOT/'scenarios/boot-entry-lessons-v4.json').read_text())['scenarios'][-1];r=json.loads((ROOT/'scenarios/presentation/lpic1.json').read_text())[124]
        self.assertEqual((s['id'],r['id']),(ID,ID));self.assertEqual(s['objective'],' '.join(r['requirements']))
        d=s['goal']['definition'];a=d['initial'];e=d['example'];self.assertNotEqual(a['reference'],e['reference']);self.assertNotEqual(a['required_root'],e['required_root']);self.assertNotEqual(a['facts']['boot_disk'],e['facts']['boot_disk'])
        self.assertEqual(s['solution'],a['reference']);self.assertEqual(s['solution'][-1],'@bios-check');self.assertEqual(s['initial']['simulation']['view'],'introduction')
        self.assertEqual(s['initial']['simulation']['assignment']['current'],a['initial']);self.assertEqual(s['initial']['simulation']['practice']['current'],e['initial'])
        self.assertEqual(len(r['concepts']),3);self.assertEqual(len(r['worked']),3);self.assertEqual(len(r['practice']),3)
        actions=' '.join(v['action'] for v in r['practice']);self.assertIn('Run command',actions);self.assertIn('Insert command',actions);self.assertIn('Test boot chain',actions);self.assertIn('grub-install /dev/vdc',actions)
        self.assertIn(e['reference'][0],[v['value'] for v in r['worked']]);self.assertNotIn(a['reference'][0],json.dumps(r));self.assertNotIn('grub-install',s['objective'])
        self.assertIn('No actual GRUB',s['simplifications']);self.assertEqual(a['preserve'],['/dev/vdb']);self.assertEqual(e['preserve'],['/dev/vdd'])
    def test_native_registry_and_prerequisites_are_integrated_without_capacity_increase(self):
        paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual(len(paths),128);self.assertEqual(paths[-1],'scenarios/boot-entry-lessons-v53.json')
        labs=teaching_inventory.catalog();self.assertEqual(len(labs),482)
        u=json.loads((ROOT/'scenarios/bios-loader-units-v1.json').read_text())['units'][0];self.assertEqual(u['labs'],[ID]);self.assertEqual(u['prerequisites'],['linux-grub-persistent-configuration','mbr-layouts']);self.assertEqual([s['id']for s,_ in labs if s.get('unit')==u['id']],[ID])
if __name__=='__main__':unittest.main()
