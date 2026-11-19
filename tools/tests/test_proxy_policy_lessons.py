"""Authored proxy tasks, retained actual origins and honest limited coverage."""
import csv,hashlib,json,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory,verify_coverage
IDS=['lpic2-proxy-client-boundary','lpic2-proxy-connect-order']
def read(name):return json.loads((ROOT/name).read_text())
def pin(value):
 raw=json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();return len(raw),hashlib.sha256(raw).hexdigest()
class ProxyPolicyLessons(unittest.TestCase):
 def test_exact_verified428_origins_remain_unchanged(self):
  old=read('scenarios/boot-entry-lessons-v27.json')['scenarios'];new=read('scenarios/boot-entry-lessons-v28.json')['scenarios']
  self.assertEqual((len(old),len(new)),(55,57));self.assertEqual(new[:55],old);self.assertEqual([s['id']for s in new[-2:]],IDS)
  self.assertEqual(pin(old),(546387,'2eb6ecf36f447707966ccea2b691d7feef4508e50d32b22e51b1452dfeb1991f'))
  rows=read('scenarios/presentation/lpic2.json');self.assertEqual(len(rows),135)
  self.assertEqual(pin(rows[:106]),(504820,'36ba487951c6e152efbd90d73aae6131c2f8de9ff5990ef155923742772bd21e'))
  known={'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in read('scenarios/boot-entry-lessons-v36.json')['scenarios']}}
  self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v27.json',{s['id']:s for s in old},known))
  bad=json.loads(json.dumps(old));bad[-1]['objective']+=' changed'
  with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v27.json',{s['id']:s for s in bad},known)
 def test_real_current_registration_and_prerequisites_do_not_raise_pack_budgets(self):
  paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  labs=teaching_inventory.catalog();self.assertEqual(len(labs),482)
  unit=read('scenarios/proxy-policy-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['static-web-services','linux-host-networking'])
  self.assertEqual([s['id']for s,_ in labs if s['unit']==unit['id']],IDS)
 def test_individual_teaching_preserves_source_workflows_and_hides_assignment_answers(self):
  rows={r['id']:r for r in read('scenarios/presentation/lpic2.json')};labs=read('scenarios/boot-entry-lessons-v28.json')['scenarios'][-2:]
  for s in labs:
   e=rows[s['id']];d=s['goal']['definition'];a=s['initial']['simulation'];self.assertEqual(e['status'],'updated');self.assertEqual(e['source'],'scenarios/boot-entry-lessons-v28.json');self.assertEqual(' '.join(e['requirements']),s['objective'])
   self.assertEqual((len(e['concepts']),len(e['practice'])),(4,3));self.assertTrue(e['worked']);self.assertEqual(a['source'],d);self.assertNotEqual(d['initial']['cases'],d['example']['cases']);self.assertNotEqual(d['initial']['initial_config'],d['initial']['required_policy'])
   self.assertEqual(len(d['initial']['cases']),7);self.assertIsNone(a['assignment']['checked_stamp']);self.assertIsNone(a['practice']['checked_stamp']);self.assertEqual(a['view'],'introduction')
   self.assertIn('Save configuration',e['practice'][1]['action']);self.assertIn('Apply saved policy',e['practice'][1]['action']);self.assertIn('Request',e['practice'][2]['action'])
   for view in ['introduction','example','assignment']:self.assertGreater(len(e['review'][view]),100)
   self.assertTrue(all(v.startswith('@proxy-policy ')for v in s['solution']))
 def test_coverage_is_only_readonly_acl_policy_practice_not_caching_or_authentication(self):
  with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='LPIC-2 v4.5'}
  for key in ['208.3','208.3.K1','208.3.K2','208.3.K4','208.3.T1','208.3.T2','208.3.T3']:
   r=rows[key];self.assertEqual(r['status'],'Partial practice');self.assertEqual(r['implemented_scenarios'].split(';'),IDS+['lpic2-cache-shared-privacy','lpic2-cache-freshness-handover']if key=='208.3'else IDS);self.assertEqual(r['evidence_tests'],'tests/proxy_policy_content.rs;tests/http_cache_content.rs'if key=='208.3'else'tests/proxy_policy_content.rs');self.assertIn('incomplete',r['remaining']);self.assertIn('cach',r['remaining']);self.assertIn('auth',r['remaining'])
  self.assertEqual(rows['208.3.K3']['status'],'Open')
  with(ROOT/'docs/CURRICULUM.csv').open()as f:r=next(r for r in csv.DictReader(f)if r['blueprint']=='LPIC-2 v4.5'and r['objective_id']=='208.3')
  self.assertEqual(r['status'],'Partial');self.assertEqual(r['implemented_scenarios'].split(';'),IDS+['lpic2-cache-shared-privacy','lpic2-cache-freshness-handover'])
if __name__=='__main__':unittest.main()
