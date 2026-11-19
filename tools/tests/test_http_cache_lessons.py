"""Real authored cache content, retained430origins and narrow objective reporting."""
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
IDS=['lpic2-cache-shared-privacy','lpic2-cache-freshness-handover']
OLD=['lpic2-proxy-client-boundary','lpic2-proxy-connect-order']
def read(name):return json.loads((ROOT/name).read_text())
def pin(value):
    raw=json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
    return len(raw),hashlib.sha256(raw).hexdigest()
class HttpCacheLessons(unittest.TestCase):
    def test_actual430_tail_and_every_prior_lpic2_presentation_remain_exact(self):
        old=read('scenarios/boot-entry-lessons-v28.json')['scenarios'];new=read('scenarios/boot-entry-lessons-v29.json')['scenarios']
        self.assertEqual((len(old),len(new)),(57,59));self.assertEqual(new[:57],old);self.assertEqual([s['id']for s in new[-2:]],IDS)
        self.assertEqual(pin(old),(566404,'9e182ecfed50464effc078e6d22e67b3eb34c8329e131a3db25da89692cc0c1c'))
        entries=read('scenarios/presentation/lpic2.json');self.assertEqual(len(entries),135)
        self.assertEqual(pin(entries[:108]),(515477,'02e87305093abd223da663ff636cc81139e65540905b631f0097ff3c8c8be35e'))
        known={'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in read('scenarios/boot-entry-lessons-v36.json')['scenarios']}}
        self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v28.json',{s['id']:s for s in old},known))
        changed=json.loads(json.dumps(old));changed[-1]['objective']+=' changed'
        with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):
            verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v28.json',{s['id']:s for s in changed},known)
    def test_real_registry_unit_and_source_bound_unchecked_scopes(self):
        paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
        catalog=teaching_inventory.catalog();self.assertEqual(len(catalog),482)
        unit=read('scenarios/http-cache-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['linux-proxy-access-policy'])
        self.assertEqual([s['id']for s,_ in catalog if s['unit']==unit['id']],IDS)
        definitions=read('tests/fixtures/http-cache-definitions-preview-v1.json')
        for scenario,definition in zip(read('scenarios/boot-entry-lessons-v29.json')['scenarios'][-2:],definitions,strict=True):
            self.assertEqual(scenario['goal']['definition'],definition);session=scenario['initial']['simulation'];self.assertEqual(session['source'],definition)
            self.assertEqual(session['view'],'introduction');self.assertIsNone(session['assignment']['checked_stamp']);self.assertIsNone(session['practice']['checked_stamp'])
            self.assertNotEqual(definition['initial']['traces'],definition['example']['traces'])
            self.assertTrue(all(action.startswith('@http-cache ')for action in scenario['solution']))
        first=definitions[1]['initial']['traces'][0]['steps'][0]
        self.assertEqual((first['request_time'],first['origin']['response_time'],first['origin']['date'],first['origin']['age']),(10,10,0,10))
        example=definitions[1]['example']['traces'][0]['steps'][0]
        self.assertEqual((example['request_time'],example['origin']['date'],example['origin']['age']),(5,0,5))
    def test_individual_teaching_uses_real_controls_defines_new_terms_and_hides_independent_headers(self):
        entries={r['id']:r for r in read('scenarios/presentation/lpic2.json')}
        for scenario in read('scenarios/boot-entry-lessons-v29.json')['scenarios'][-2:]:
            e=entries[scenario['id']];self.assertEqual(e['status'],'updated');self.assertEqual(e['source'],'scenarios/boot-entry-lessons-v29.json')
            self.assertEqual(' '.join(e['requirements']),scenario['objective']);self.assertEqual((len(e['concepts']),len(e['practice'])),(4,3));self.assertGreaterEqual(len(e['worked']),4)
            self.assertTrue(all(not row['title'][0].isdigit()for row in e['concepts']))
            for view in ['introduction','example','assignment']:self.assertGreater(len(e['review'][view]),150)
            actions=' '.join(row['action']for row in e['practice']);self.assertIn('Run example traces',actions);self.assertIn('Fill worked headers',actions)
            for resource in scenario['goal']['definition']['initial']['resources']:
                self.assertNotIn(resource['reference'],scenario['objective'])
        self.assertIn('ETag is a label',entries[IDS[1]]['concepts'][2]['text'])
        self.assertIn('trace is a short sequence',entries[IDS[0]]['concepts'][3]['text'])
        self.assertIn('preceding lesson',entries[IDS[1]]['context'])
    def test_generic_cache_practice_does_not_become_squid_authentication_or_configuration_credit(self):
        with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='LPIC-2 v4.5'}
        parent=rows['208.3'];self.assertEqual(parent['status'],'Partial practice');self.assertEqual(parent['implemented_scenarios'].split(';'),OLD+IDS)
        self.assertIn('generic',parent['verified_scope']);self.assertIn('No Squid cache configuration',parent['remaining']);self.assertIn('authentication',parent['remaining']);self.assertIn('resource',parent['remaining'])
        self.assertEqual(rows['208.3.K3']['status'],'Open')
        for key in ['208.3.K1','208.3.K2','208.3.K4','208.3.T1','208.3.T2','208.3.T3']:
            self.assertEqual(rows[key]['implemented_scenarios'].split(';'),OLD)
        with(ROOT/'docs/CURRICULUM.csv').open()as f:row=next(r for r in csv.DictReader(f)if r['blueprint']=='LPIC-2 v4.5'and r['objective_id']=='208.3')
        self.assertEqual(row['status'],'Partial');self.assertEqual(row['implemented_scenarios'].split(';'),OLD+IDS)
if __name__=='__main__':unittest.main()
