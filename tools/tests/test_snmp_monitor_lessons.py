"""Two authored monitoring tasks, real retained values and narrow objective reporting."""
import csv,hashlib,json,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory,verify_coverage
IDS=['ccna-snmp-link-monitor','ccna-snmp-counter-monitor']
def read(p):return json.loads((ROOT/p).read_text())
def pin(value):
 raw=json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();return len(raw),hashlib.sha256(raw).hexdigest()
class SnmpMonitorLessons(unittest.TestCase):
 def test_actual426_retained_tail_and100_presentations_are_exact(self):
  old=read('scenarios/boot-entry-lessons-v26.json')['scenarios'];new=read('scenarios/boot-entry-lessons-v27.json')['scenarios']
  self.assertEqual((len(old),len(new)),(53,55));self.assertEqual(new[:53],old);self.assertEqual([s['id']for s in new[53:]],IDS)
  self.assertEqual(pin(old),(523314,'de1536e742daecee03c7bc1030ee1cbf715f56d542670cfd7e5134ce28afaa83'))
  rows=read('scenarios/presentation/ccna.json');self.assertEqual(len(rows),113)
  self.assertEqual(pin(rows[:100]),(366833,'4b946750a4fb0ac265103b93d4ff8d854aba75e6017e835a4f8f61334334da84'))
  known={'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in read('scenarios/boot-entry-lessons-v36.json')['scenarios']}}
  self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v26.json',{s['id']:s for s in old},known))
  bad=json.loads(json.dumps(old));bad[0]['objective']+=' changed'
  with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v26.json',{s['id']:s for s in bad},known)
 def test_real_registration_and_prerequisites_do_not_raise_registry_limits(self):
  paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  labs=teaching_inventory.catalog();self.assertEqual(len(labs),482)
  unit=read('scenarios/snmp-monitor-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['ccna-transport-conversations','management-policy-foundations'])
  self.assertEqual([s['id']for s,_ in labs if s['unit']==unit['id']],IDS)
 def test_individual_teaching_matches_distinct_models_and_expected_actual_separation(self):
  rows={r['id']:r for r in read('scenarios/presentation/ccna.json')};lessons=read('scenarios/boot-entry-lessons-v27.json')['scenarios'][-2:]
  self.assertEqual([s['goal']['definition']['initial']['activity']for s in lessons],['link','traffic'])
  for s in lessons:
   r=rows[s['id']];d=s['goal']['definition'];self.assertEqual(r['status'],'updated');self.assertEqual(r['source'],'scenarios/boot-entry-lessons-v27.json');self.assertEqual(' '.join(r['requirements']),s['objective'])
   self.assertEqual((len(r['concepts']),len(r['practice']),len(r['requirements'])),(4,3,4));self.assertTrue(all(len(r['review'][v])>100 for v in ['introduction','example','assignment']))
   self.assertNotEqual(d['initial']['cases'],d['example']['cases']);self.assertNotEqual(d['initial']['id'],d['example']['id']);self.assertTrue(all(p['action']and p['observe']for p in r['practice']))
   self.assertIsNone(s['initial']['simulation']['assignment']['checked_fields']);self.assertIsNone(s['initial']['simulation']['assignment']['queried']);self.assertTrue(all(v.startswith(('@monitor-set ','@monitor-check'))for v in s['solution']))
  self.assertIn('Simple Network Management Protocol',rows[IDS[0]]['concepts'][0]['text']);self.assertIn('not an IPv4 address',rows[IDS[0]]['concepts'][1]['text'])
  traffic=rows[IDS[1]];self.assertIn('Expected for Sustained receive load',traffic['practice'][0]['observe']);self.assertIn('Actual query result',traffic['practice'][0]['observe'])
  self.assertEqual([r['value']for r in traffic['worked'][2:5]],['1,400,000,000 bits','70,000,000 bit/s','70% of 100 Mbit/s'])
 def test_objective_mapping_keeps_protocol_and_notification_exclusions(self):
  with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:r=next(r for r in csv.DictReader(f)if r['blueprint']=='CCNA v1.1'and r['reference']=='4.4')
  self.assertEqual(r['status'],'Partial practice');self.assertEqual(r['implemented_scenarios'].split(';'),IDS);self.assertEqual(r['evidence_tests'],'tests/snmp_content.rs')
  for word in ['SNMPv3','GETBULK','traps/informs','incomplete']:self.assertIn(word,r['remaining'])
  with(ROOT/'docs/CONTINUATION_COVERAGE.csv').open()as f:labs={r['lab_id']:r for r in csv.DictReader(f)}
  for id in IDS:self.assertEqual(labs[id]['evidence'],'SNMP_MONITOR_LESSONS.md')
if __name__=='__main__':unittest.main()
