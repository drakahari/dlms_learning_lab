"""Only integrated, taught protocol evidence earns narrow practical credit."""
import hashlib
import json
import sys
import unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory,verify_coverage

def read(n):return json.loads((ROOT/n).read_text())
def pin(v):
 raw=json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
 return len(raw),hashlib.sha256(raw).hexdigest()

class ProtocolEvidenceLessons(unittest.TestCase):
 def test_exact437_candidate_tail_and143_teaching_values_are_preserved(self):
  old=read('scenarios/boot-entry-lessons-v32.json')['scenarios'];now=read('scenarios/boot-entry-lessons-v33.json')['scenarios']
  self.assertEqual((len(old),len(now)),(64,65));self.assertEqual(now[:64],old)
  self.assertEqual(pin(old),(655387,'595e364959ed8f52499a99bd1712ebda4c54a8b9529d8d7cb4fb657d4a59cc16'))
  teaching=read('scenarios/presentation/lpic1.json');self.assertEqual(len(teaching),154)
  self.assertEqual(pin(teaching[:143]),(629347,'e5c5840ae1857912a6d287fe68196fd93b61e080c3f65046dee4d13b112bf86e'))
  known={'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in read('scenarios/boot-entry-lessons-v36.json')['scenarios']}}
  self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v32.json',{s['id']:s for s in old},known))
  bad=json.loads(json.dumps(old));bad[0]['objective']+=' changed'
  with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v32.json',{s['id']:s for s in bad},known)
 def test_real_registry_and_case_rosters_bind_distinct_practice_and_assignment(self):
  paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  labs=teaching_inventory.catalog();self.assertEqual(len(labs),482)
  s=next(s for s,_ in labs if s['id']=='lpic1-protocol-round-trips');self.assertEqual(s['prerequisites'],['lpic1-host-mixed-ip','lpic1-service-client-targets'])
  unit=read('scenarios/protocol-evidence-units-v1.json')['units'][0];self.assertEqual(unit['labs'],[s['id']]);self.assertEqual(unit['prerequisites'],['linux-mixed-ip-foundations','linux-service-names'])
  fixture=read('tests/fixtures/protocol-evidence-preview-v1.json')['scenarios'][0];fixture['prerequisites']=s['prerequisites'];self.assertEqual(s,fixture)
  d=s['goal']['definition'];self.assertNotEqual(d['assignment']['transport']['hosts'],d['example']['transport']['hosts'])
  for t in [d['assignment'],d['example']]:
   self.assertEqual((len(t['transport']['exchanges']),len(t['echoes']),len(t['requirements'])),(3,2,5))
   self.assertEqual(t['editable_routers'],['R1']);self.assertFalse(t['editable_listeners']or t['editable_exchanges'])
   self.assertEqual(sum(r['kind']=='echo_receive'for r in t['requirements']),2)
   self.assertTrue(all('port'not in q for q in t['echoes']))
 def test_each_teaching_view_names_real_actions_without_exposing_assignment_repairs(self):
  s=read('scenarios/boot-entry-lessons-v33.json')['scenarios'][-1];e=next(e for e in read('scenarios/presentation/lpic1.json')if e['id']==s['id'])
  self.assertEqual(e['id'],s['id']);self.assertEqual(' '.join(e['requirements']),s['objective']);self.assertEqual(e['status'],'updated')
  self.assertEqual((len(e['concepts']),len(e['worked']),len(e['practice'])),(5,4,3))
  text=json.dumps(e,ensure_ascii=False)
  for term in ['connection request (SYN)','separate message (datagram)','identifier','sequence','no TCP/UDP service ports','Check configuration','Fill worked settings','Reset example only']:self.assertIn(term,text)
  self.assertIn('9443',' '.join(w['value']for w in e['worked']));self.assertNotIn('10.80.20.20 eq 8443',text)
  for view in ['introduction','example','assignment']:self.assertGreater(len(e['review'][view]),40)
  for bad in ['Read the contract','fresh virtual workspace','source-held endpoints']:self.assertNotIn(bad,text)
 def test_new_optional_fields_keep_ports_out_of_echo_and_reports_out_of_progress(self):
  core=(ROOT/'src/transport_core/echo.rs').read_text();ws=(ROOT/'src/transport_workspace.rs').read_text()
  self.assertIn('skip_serializing_if = "Vec::is_empty"',ws);self.assertIn('pub echoes: Vec<core::echo::Query>',ws)
  self.assertIn('MAX_PAYLOAD: usize = 1024',core);self.assertIn('MAX_CASES: usize = 8',core)
  self.assertIn('sequence::<_,u8,1024>',core);self.assertIn('sequence::<_,Query,8>',core)
  initial=read('tests/fixtures/protocol-evidence-preview-v1.json')['scenarios'][0]['initial']['simulation']
  for name in ['assignment','practice']:
   self.assertIsNone(initial[name]['tested']);self.assertEqual(set(initial[name]),{'draft','tested'})

if __name__=='__main__':unittest.main()
