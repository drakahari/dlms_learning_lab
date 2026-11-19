"""Standalone sharing lesson/source guards. No host Samba or learner execution."""
import json,hashlib,sys,unittest
from collections import Counter
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
IDS=['lpic2-smb-identity-access','lpic2-smb-team-policy','lpic2-smb-change-handover']
def read(p):return json.loads((ROOT/p).read_text())
class SambaShareLessons(unittest.TestCase):
 def test_registered_distinct_unit_keeps_exact_old_objects_and_pack_limits(self):
  paths=coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  before=read('scenarios/boot-entry-lessons-v42.json')['scenarios'];added=read('scenarios/samba-standalone-lessons-v1.json')['scenarios']
  self.assertEqual(len(before),82);self.assertEqual([s['id']for s in added],IDS);self.assertEqual(read(paths[-1])['scenarios'],before+added+read('scenarios/virtual-isolation-lessons-v1.json')['scenarios']+read('scenarios/guest-driver-lessons-v1.json')['scenarios']+read('scenarios/smtp-routing-lessons-v1.json')['scenarios']+read('scenarios/sieve-filter-lessons-v1.json')['scenarios']+read('scenarios/sieve-address-lessons-v1.json')['scenarios']+read('scenarios/sieve-delivery-lessons-v1.json')['scenarios']+read('scenarios/sieve-vacation-lessons-v1.json')['scenarios']+read('scenarios/procmail-migration-lessons-v1.json')['scenarios']+read('scenarios/mailbox-access-lessons-v1.json')['scenarios']+read('scenarios/mailbox-administration-lessons-v1.json')['scenarios'])
  self.assertEqual(read('scenarios/boot-entry-lessons-v43.json')['scenarios'],before+added)
  self.assertEqual(read('scenarios/authoring/samba-standalone-v1.json')['scenarios'],added);self.assertLess((ROOT/paths[-1]).stat().st_size,2_000_000)
  labs=teaching.catalog();self.assertEqual(len(labs),482);self.assertEqual(Counter(s['track']for s,_ in labs),{'CCNA':113,'LPIC-1':154,'LPIC-2':135,'Bash/Shell':80})
  unit=read('scenarios/samba-standalone-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['local-account-lifecycle','linux-access-policy'])
 def test_114_previous_lpic2_presentations_exact_and_new_three_views_are_specific(self):
  rows=read('scenarios/presentation/lpic2.json');self.assertEqual(len(rows),135)
  raw=json.dumps(rows[:114],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();self.assertEqual((len(raw),hashlib.sha256(raw).hexdigest()),(548979,'b810c5b78e2498928fd528d8b43f94f0114e418975d46c6679e6f172d9dadaa0'))
  for s,row in zip(read('scenarios/samba-standalone-lessons-v1.json')['scenarios'],rows[114:]):
   self.assertEqual(row['id'],s['id']);self.assertEqual(' '.join(row['requirements']),s['objective']);self.assertEqual(len(row['concepts']),3);self.assertEqual(len(row['practice']),3)
   for phase in ['introduction','example','assignment']:self.assertGreater(len(row['review'][phase]),40)
   self.assertIn('Run',row['practice'][0]['action']);self.assertIn('Check answers',row['practice'][2]['action']);self.assertTrue(row['worked']);self.assertIn('simulator',s['simplifications'])
 def test_source_bound_trial_matrix_matches_paths_not_only_outputs_and_starts_unchecked(self):
  for s in read('scenarios/samba-standalone-lessons-v1.json')['scenarios']:
   d=s['goal']['definition'];a=d['assignment'];e=d['example'];self.assertEqual(len(a['cases']),12);self.assertEqual(len(e['cases']),12);self.assertNotEqual(a['initial']['server'],e['initial']['server'])
   for view,key in [('assignment','assignment'),('practice','example')]:
    v=s['initial']['simulation'][view];self.assertEqual(v['world'],d[key]['initial']);self.assertEqual(v['editor'],v['world']['files']['files'][v['world']['configuration_path']]);self.assertFalse(v['checked']);self.assertNotEqual(v['world'],d[key]['reference'])
   kinds=Counter(c['expected']['kind']for c in a['cases']);self.assertGreater(kinds['Read'],2);self.assertGreater(kinds['Replaced'],0);self.assertGreater(kinds['Denied'],2)
   for c in a['cases']:
    if c['expected']['kind']!='Denied':self.assertTrue(c['expected']['path'].startswith('/'));self.assertNotEqual(c['expected']['path'],a['initial']['configuration_path'])
 def test_native_subset_has_no_host_execution_and_worked_fill_is_example_only(self):
  for p in ['src/samba_share_config.rs','src/samba_share.rs','src/samba_share_workspace.rs','src/samba_share_source.rs','src/ui/samba_share_page.rs']:
   s=(ROOT/p).read_text()
   for bad in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','unsafe {']:self.assertNotIn(bad,s)
  ui=(ROOT/'src/ui/samba_share_page.rs').read_text();self.assertIn('if practice {',ui);self.assertIn('Fill worked guide configuration',ui);self.assertIn('a.last_trial = None',ui);self.assertIn('filter(|c| !c.passed)',ui);self.assertIn('controls::singleline_fitting',ui)
  native=(ROOT/'src/ui/smoke/samba_shares.rs').read_text()
  for required in ['Duration::from_secs(15)','f.polls < 1200','f.popup_reveals <= 20','smoke_control_visible','Op::Trial(false)','Op::Trial(true)','Op::Text("payload"']:self.assertIn(required,native)
if __name__=='__main__':unittest.main()
