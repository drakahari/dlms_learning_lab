"""Leaf/spine source/presentation guards; no host commands or runtime acceptance."""
import hashlib,json,sys,unittest
from pathlib import Path
from collections import Counter
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
ID='ccna-leaf-spine-rack-expansion'
def read(p):return json.loads((ROOT/p).read_text())
class LeafSpineLessons(unittest.TestCase):
 def test_one_registered_authored_task_preserves_81_previous_objects_and_limits(self):
  paths=coverage.registered_pack_paths(ROOT,'current')
  self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  before=read('scenarios/boot-entry-lessons-v41.json')['scenarios'];added=read('scenarios/leaf-spine-lessons-v1.json')['scenarios']
  self.assertEqual(len(before),81);self.assertEqual([s['id']for s in added],[ID])
  self.assertEqual(read('scenarios/boot-entry-lessons-v42.json')['scenarios'],before+added)
  self.assertEqual(read(paths[-1])['scenarios'],before+added+read('scenarios/samba-standalone-lessons-v1.json')['scenarios']+read('scenarios/virtual-isolation-lessons-v1.json')['scenarios']+read('scenarios/guest-driver-lessons-v1.json')['scenarios']+read('scenarios/smtp-routing-lessons-v1.json')['scenarios']+read('scenarios/sieve-filter-lessons-v1.json')['scenarios']+read('scenarios/sieve-address-lessons-v1.json')['scenarios']+read('scenarios/sieve-delivery-lessons-v1.json')['scenarios']+read('scenarios/sieve-vacation-lessons-v1.json')['scenarios']+read('scenarios/procmail-migration-lessons-v1.json')['scenarios']+read('scenarios/mailbox-access-lessons-v1.json')['scenarios']+read('scenarios/mailbox-administration-lessons-v1.json')['scenarios'])
  self.assertEqual(read('scenarios/authoring/leaf-spine-v1.json')['scenarios'],added)
  self.assertLess((ROOT/paths[-1]).stat().st_size,2_000_000)
  labs=teaching.catalog();self.assertEqual(len(labs),482)
  self.assertEqual(Counter(s['track']for s,_ in labs),{'CCNA':113,'LPIC-1':154,'LPIC-2':135,'Bash/Shell':80})
 def test_all_previous_teaching_values_retained_and_each_new_view_is_specific(self):
  rows=read('scenarios/presentation/ccna.json');self.assertEqual(len(rows),113)
  raw=json.dumps(rows[:109],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
  self.assertEqual((len(raw),hashlib.sha256(raw).hexdigest()),(409548,'9fd470ad1c28421bcd7559414c5f98b1b106d08764215f9763bdfd3253b88dbc'))
  row=rows[109];s=read('scenarios/leaf-spine-lessons-v1.json')['scenarios'][0]
  self.assertEqual(row['id'],ID);self.assertEqual(' '.join(row['requirements']),s['objective'])
  self.assertEqual(len(row['concepts']),3);self.assertEqual(len(row['practice']),3)
  for phase in ['introduction','example','assignment']:self.assertGreater(len(row['review'][phase]),40)
  self.assertIn('Check answers',row['practice'][0]['action']);self.assertIn('Guide Spine 1 / p3',row['practice'][1]['action'])
  self.assertIn('Unplug',row['practice'][2]['action']);self.assertIn('not a claim of fully disjoint',' '.join(r['explanation']for r in row['worked']))
 def test_complete_source_bound_case_matrix_and_separate_ungraded_drafts(self):
  s=read('scenarios/leaf-spine-lessons-v1.json')['scenarios'][0];d=s['goal']['definition'];a=d['assignment'];e=d['example']
  self.assertEqual((len(a['facts']['nodes']),len(a['facts']['wires']),len(a['editable']),len(a['pairs'])),(7,12,3,3))
  self.assertEqual([c['minimum_paths']for c in a['cases']],[3,2,2,2])
  self.assertEqual([c['minimum_paths']for c in e['cases']],[2,1,1])
  self.assertNotEqual(a['facts'],e['facts'])
  for scope,key in [('assignment','assignment'),('practice','example')]:
   self.assertEqual(s['initial']['simulation'][scope]['draft'],d[key]['initial']);self.assertFalse(s['initial']['simulation'][scope]['checked'])
  self.assertEqual(a['initial']['connections'][:9],a['reference']['connections'][:9])
  self.assertTrue(all(c['b']is None for c in a['initial']['connections'][9:]))
  self.assertEqual(read('scenarios/leaf-spine-units-v1.json')['units'][0]['prerequisites'],['ccna-campus-architectures'])
 def test_new_count_observation_is_opt_in_and_bounded_capture_keeps_limits(self):
  for p in ['src/network_architecture.rs','src/network_architecture_workspace.rs','src/ui/network_architecture_page.rs']:
   text=(ROOT/p).read_text()
   for forbidden in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','unsafe {']:self.assertNotIn(forbidden,text)
  text=(ROOT/'src/ui/network_architecture_page.rs').read_text()
  self.assertIn('t.facts.design == core::Design::SpineLeaf',text);self.assertIn('Actual distinct internal spine alternatives',text)
  self.assertIn('filter(|(_, r)| !r.passed)',text)
  text=(ROOT/'src/ui/smoke/network_architectures.rs').read_text()
  for required in ['Duration::from_secs(15)','f.polls < 1200','smoke_control_visible','leaf-spine-batch','Op::Choose("pair".into(), "2".into())']:self.assertIn(required,text)
if __name__=='__main__':unittest.main()
