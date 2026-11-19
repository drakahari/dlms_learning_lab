"""Narrow, actually authored host diagnosis; no credit for absent utilities."""
import csv
import hashlib
import json
import sys
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory,verify_coverage
ID='lpic1-echo-path-diagnosis'
def read(n):return json.loads((ROOT/n).read_text())
def pin(v):
 raw=json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
 return len(raw),hashlib.sha256(raw).hexdigest()
class EchoDiagnosisLessons(unittest.TestCase):
 def test_genuine438_source_and_teaching_prefixes_remain_exact(self):
  old=read('scenarios/boot-entry-lessons-v33.json')['scenarios'];current=read('scenarios/boot-entry-lessons-v34.json')['scenarios']
  self.assertEqual((len(old),len(current)),(65,66));self.assertEqual(current[:65],old)
  self.assertEqual(pin(old),(674270,'ac9845239c2e790d17f553940dbb31a494f48a2ae886ff0814d3ce86fe8547a6'))
  rows=read('scenarios/presentation/lpic1.json');self.assertEqual(len(rows),154)
  self.assertEqual(pin(rows[:144]),(636511,'5a386bc69dc2f8c4229e832e146c1add14bf39d1a6c4a0d2ffee1dee7300e548'))
  known={'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in read('scenarios/boot-entry-lessons-v36.json')['scenarios']}}
  self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v33.json',{s['id']:s for s in old},known))
  changed=json.loads(json.dumps(old));changed[0]['objective']+=' modified'
  with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v33.json',{s['id']:s for s in changed},known)
 def test_registered_task_has_real_prerequisites_and_independent_worlds(self):
  paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  labs=teaching_inventory.catalog();self.assertEqual(len(labs),482);s=next(s for s,_ in labs if s['id']==ID)
  self.assertEqual(s['prerequisites'],['lpic1-host-mixed-ip','lpic1-protocol-round-trips','lpic1-listener-audit'])
  fixture=read('tests/fixtures/echo-diagnosis-preview-v1.json')['scenarios'][0];fixture['prerequisites']=s['prerequisites'];self.assertEqual(s,fixture)
  u=read('scenarios/echo-diagnosis-units-v1.json')['units'][0];self.assertEqual(u['labs'],[ID]);self.assertEqual(u['prerequisites'],['linux-protocol-evidence','linux-listener-audit'])
  d=s['goal']['definition'];self.assertNotEqual(d['initial']['profile'],d['example']['profile'])
  for scope in ['initial','example']:
   exercise=d['echo'][scope];self.assertEqual(len(exercise['required_replies']),4)
   self.assertTrue(all(any(p['address']==a and p['responds']for p in exercise['plan']['peers'])for a in exercise['required_replies']))
   silent=[p for p in exercise['plan']['peers']if not p['responds']];self.assertEqual(len(silent),1);self.assertNotIn(silent[0]['address'],exercise['required_replies'])
  self.assertEqual(s['initial']['simulation']['assignment']['history'],[])
 def test_teaching_names_actual_controls_and_does_not_reveal_assignment_commands(self):
  e=next(e for e in read('scenarios/presentation/lpic1.json')if e['id']==ID)
  self.assertEqual(e['status'],'updated');self.assertEqual((len(e['concepts']),len(e['worked']),len(e['practice'])),(4,4,3))
  text=json.dumps(e,ensure_ascii=False)
  for needed in ['Run command','Insert worked action','Next worked action','Check example','Reset example only','no real network','/32','/20','silent']:self.assertIn(needed,text)
  worked=' '.join(w['value']for w in e['worked']);self.assertIn('10.110.64.0–10.110.79.255',worked);self.assertNotIn('10.96.19.10/20',text);self.assertNotIn('ip -4 route replace default via 10.96.16.1',text)
  for view in ['introduction','example','assignment']:self.assertGreater(len(e['review'][view]),40)
 def test_only_implemented_ip_ping_and_mask_slices_gain_partial_evidence(self):
  with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='LPIC-1 v5.0'}
  for ref in ['109.3','109.3.K1','109.3.K2','109.3.K3','109.3.T1','109.3.T4','109.1.K1','109.1.T3']:
   r=rows[ref];self.assertEqual(r['status'],'Partial practice');self.assertIn(ID,r['implemented_scenarios'].split(';'));self.assertIn('tests/echo_diagnosis_content.rs',r['evidence_tests'])
  for ref in ['109.3.K4','109.3.T2',*[f'109.3.T{i}'for i in range(5,14)]]:self.assertEqual(rows[ref]['status'],'Open',ref)
  self.assertEqual(rows['109.3.T3']['implemented_scenarios'],'lpic1-listener-audit')
  self.assertIn('no real packets',rows['109.3.T4']['remaining'].lower())
if __name__=='__main__':unittest.main()
