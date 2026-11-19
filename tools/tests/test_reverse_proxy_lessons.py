"""Authored reverse-routing scope and exact verified432 preservation."""
import csv
import hashlib
import json
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory
import verify_coverage
IDS=['lpic2-proxy-uri-mapping','lpic2-proxy-private-paths']
def read(name):return json.loads((ROOT/name).read_text())
def pin(value):
    raw=json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
    return len(raw),hashlib.sha256(raw).hexdigest()
class ReverseProxyLessons(unittest.TestCase):
    def test_actual432_sources_and_all_prior_teaching_values_remain_exact(self):
        before=read('scenarios/boot-entry-lessons-v29.json')['scenarios'];after=read('scenarios/boot-entry-lessons-v30.json')['scenarios']
        self.assertEqual((len(before),len(after)),(59,61));self.assertEqual(after[:59],before)
        self.assertEqual(pin(before),(596866,'f6e947858f6961ec6b2dcc85772065d548a76174cadd41c2fc064301f41d1630'))
        entries=read('scenarios/presentation/lpic2.json');self.assertEqual(len(entries),135)
        self.assertEqual(pin(entries[:110]),(526561,'5c8bfb485ab174106eb0788a7fc8c6b58a0eabaed15a47a41d834e1be6341c7b'))
        known={'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in read('scenarios/boot-entry-lessons-v36.json')['scenarios']}}
        self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v29.json',{s['id']:s for s in before},known))
        damaged=json.loads(json.dumps(before));damaged[-1]['objective']+=' changed'
        with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):
            verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v29.json',{s['id']:s for s in damaged},known)
    def test_real_catalog_and_new_unit_use_fresh_separate_source_bound_tasks(self):
        paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
        catalog=teaching_inventory.catalog();self.assertEqual(len(catalog),482)
        unit=read('scenarios/nginx-routing-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS)
        self.assertEqual(unit['prerequisites'],['linux-proxy-access-policy','static-web-services'])
        self.assertEqual([s['id']for s,_ in catalog if s['unit']==unit['id']],IDS)
        proposals=read('tests/fixtures/nginx-routing-preview-v1.json')['scenarios']
        for n,s in enumerate(read('scenarios/boot-entry-lessons-v30.json')['scenarios'][-2:]):
            self.assertEqual(s,proposals[n]);self.assertEqual(s['id'],IDS[n])
            definition=s['goal']['definition'];state=s['initial']['simulation'];self.assertEqual(state['source'],definition)
            self.assertEqual(state['view'],'introduction');self.assertNotEqual(definition['initial']['cases'],definition['example']['cases'])
            self.assertNotEqual(definition['initial']['required_config'],definition['example']['required_config'])
            for scope in ['assignment','practice']:self.assertIsNone(state[scope]['checked_stamp']);self.assertEqual(state[scope]['world']['saved'],state[scope]['world']['applied'])
            self.assertNotEqual(state['assignment']['editor'],definition['initial']['required_config'])
            self.assertTrue(all(a.startswith('@reverse-proxy ')for a in s['solution']))
    def test_individual_three_view_teaching_names_real_controls_and_preserves_reasoning(self):
        entries={e['id']:e for e in read('scenarios/presentation/lpic2.json')}
        for s in read('scenarios/boot-entry-lessons-v30.json')['scenarios'][-2:]:
            e=entries[s['id']];self.assertEqual(e['status'],'updated');self.assertEqual(e['source'],'scenarios/boot-entry-lessons-v30.json')
            self.assertEqual(' '.join(e['requirements']),s['objective']);self.assertEqual((len(e['concepts']),len(e['practice'])),(4,3));self.assertGreaterEqual(len(e['worked']),4)
            for view in ['introduction','example','assignment']:self.assertGreater(len(e['review'][view]),150)
            actions=' '.join(x['action']for x in e['practice'])
            for control in ['Save configuration','Check syntax','Apply saved configuration','Check routing','Request','Fill worked configuration']:self.assertIn(control,actions)
            self.assertNotIn(s['goal']['definition']['initial']['required_config'],s['objective'])
        first=' '.join(x['text']for x in entries[IDS[0]]['concepts']);self.assertIn('replace /charts/ with /v1/',first);self.assertIn('keep the entire client path',first)
        second=' '.join(x['text']for x in entries[IDS[1]]['concepts']);self.assertIn('longest',second);self.assertIn('exact',second);self.assertIn('prefix is a string',entries[IDS[1]]['concepts'][1]['title'].lower())
    def test_narrow_reverse_proxy_credit_does_not_claim_full_nginx_server_or_platform(self):
        with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='LPIC-2 v4.5'}
        for key in ['208.4','208.4.K1','208.4.K2']:
            self.assertEqual(rows[key]['status'],'Partial practice');self.assertEqual(rows[key]['implemented_scenarios'].split(';'),IDS)
            self.assertIn('normalized',rows[key]['verified_scope']);self.assertIn('No real Nginx',rows[key]['remaining'])
        for key in ['208.4.K3','208.4.T1','208.4.T2']:
            self.assertEqual(rows[key]['status'],'Open');self.assertFalse(rows[key]['implemented_scenarios'])
        with(ROOT/'docs/CURRICULUM.csv').open()as f:row=next(r for r in csv.DictReader(f)if r['blueprint']=='LPIC-2 v4.5'and r['objective_id']=='208.4')
        self.assertEqual(row['status'],'Partial');self.assertEqual(row['implemented_scenarios'].split(';'),IDS)
if __name__=='__main__':unittest.main()
