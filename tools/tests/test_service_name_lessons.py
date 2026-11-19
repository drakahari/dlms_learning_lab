"""Authored service-name tasks, preserving all434 prior sources and teaching."""
import hashlib,json,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory,verify_coverage
IDS=['lpic1-service-client-targets','lpic1-services-file-repair']
def read(n):return json.loads((ROOT/n).read_text())
def pin(v):
 raw=json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();return len(raw),hashlib.sha256(raw).hexdigest()
class ServiceNamesLessons(unittest.TestCase):
 def test_exact_prior61_sources_and_all_prior_teaching_are_preserved(self):
  old=read('scenarios/boot-entry-lessons-v30.json')['scenarios'];new=read('scenarios/boot-entry-lessons-v31.json')['scenarios'];self.assertEqual((len(old),len(new)),(61,63));self.assertEqual(old,new[:61]);self.assertEqual(pin(old),(619636,'09478f2f388c963e49cbcf2693bd7c5e45ccd53a9c4e14cbcf9b6b59721ba36e'))
  known={'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in read('scenarios/boot-entry-lessons-v36.json')['scenarios']}};self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v30.json',{s['id']:s for s in old},known));bad=json.loads(json.dumps(old));bad[0]['objective']+=' changed'
  with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v30.json',{s['id']:s for s in bad},known)
 def test_registered_complete_tasks_and_separate_examples_have_real_cases(self):
  paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'));catalog=teaching_inventory.catalog();self.assertEqual(len(catalog),482);unit=read('scenarios/service-name-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['linux-host-networking'])
  proposal=read('tests/fixtures/service-name-preview-v1.json')['scenarios'];actual=read('scenarios/boot-entry-lessons-v31.json')['scenarios'][-2:];self.assertEqual(proposal,actual)
  for n,s in enumerate(actual):
   d=s['goal']['definition'];state=s['initial']['simulation'];self.assertEqual(state['source'],d);self.assertNotEqual(d['initial']['profile'],d['example']['profile']);self.assertEqual(len(d['initial']['requests']),6 if n==0 else 4)
   for key,t in [('assignment',d['initial']),('practice',d['example'])]:self.assertIsNone(state[key]['checked_stamp']);self.assertEqual(state[key]['editor'],t['initial_database']);self.assertEqual(state[key]['clients'],t['initial_clients'])
   self.assertTrue(all(a.startswith('@service-clients ')for a in s['solution']))
 def test_each_screen_teaches_the_actual_tools_without_exposing_assignment_answers(self):
  entries={e['id']:e for e in read('scenarios/presentation/lpic1.json')};self.assertEqual(len(entries),154)
  for s in read('scenarios/boot-entry-lessons-v31.json')['scenarios'][-2:]:
   e=entries[s['id']];self.assertEqual(' '.join(e['requirements']),s['objective']);self.assertEqual((len(e['concepts']),len(e['practice'])),(4,3));self.assertTrue(e['worked']);self.assertEqual(e['source'],'scenarios/boot-entry-lessons-v31.json');self.assertTrue(all(len(e['review'][k])>40 for k in ['introduction','example','assignment']));self.assertIn('getent',' '.join(c['text'] for c in e['concepts']));self.assertIn('Check clients',' '.join(c['action'] for c in e['practice']));self.assertNotIn('Read the contract',json.dumps(e));self.assertNotIn('fresh virtual workspace',json.dumps(e))
if __name__=='__main__':unittest.main()
