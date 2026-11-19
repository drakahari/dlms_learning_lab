"""Address-filter registration/teaching/origin guards; no mail or learner execution."""
import hashlib
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
IDS=['lpic2-sieve-sender-domain','lpic2-sieve-recipient-recovery']
def read(p):return json.loads((ROOT/p).read_text())
class AddressLessons(unittest.TestCase):
 def test_selected_catalog_is_exact470_plus_two_distinct_authored_address_tasks(self):
  paths=coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  before=read('scenarios/boot-entry-lessons-v47.json')['scenarios'];added=read('scenarios/sieve-address-lessons-v1.json')['scenarios']
  self.assertEqual(len(before),97);self.assertEqual([s['id']for s in added],IDS);self.assertEqual(read('scenarios/boot-entry-lessons-v48.json')['scenarios'],before+added);self.assertLess((ROOT/paths[-1]).stat().st_size,2_000_000)
  labs=teaching.catalog();self.assertEqual(len(labs),482);self.assertEqual([s['id']for s,_ in labs][470:472],IDS)
  unit=read('scenarios/sieve-address-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['linux-sieve-filters'])
 def test_all123_old_presentations_stay_exact_and_both_new_teaching_reviews_are_specific(self):
  rows=read('scenarios/presentation/lpic2.json');self.assertEqual(len(rows),135)
  old=json.dumps(rows[:123],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();self.assertEqual(hashlib.sha256(old).hexdigest(),'468215f850896a7a4a7b98ff527d57ba0955aeee151460e00f9d80a675a9920a')
  for lab,row in zip(read('scenarios/sieve-address-lessons-v1.json')['scenarios'],rows[123:]):
   self.assertEqual(lab['id'],row['id']);self.assertEqual(row['status'],'updated');self.assertEqual(' '.join(row['requirements']),lab['objective']);self.assertEqual(row['source'],'scenarios/sieve-address-lessons-v1.json')
   for section in ['concepts','worked','practice']:self.assertEqual(len(row[section]),3)
   for phase in ['introduction','example','assignment']:self.assertGreater(len(row['review'][phase]),60)
  self.assertIn('either To or Cc',rows[124]['practice'][1]['observe']);self.assertNotIn('Both actual recipient fields are required',json.dumps(rows[123:]))
 def test_profiles_and_counterexamples_are_real_with_independent_ungraded_worked_inputs(self):
  for lab,n in zip(read('scenarios/sieve-address-lessons-v1.json')['scenarios'],[11,12]):
   definition=lab['goal']['definition'];assignment=definition['assignment'];example=definition['example'];self.assertTrue(assignment['address_tests']and example['address_tests']);self.assertEqual(len(assignment['cases']),n);self.assertEqual(len(example['cases']),4);self.assertNotEqual(assignment['cases'],example['cases']);self.assertNotEqual(assignment['starter'],assignment['reference']);self.assertNotEqual(assignment['reference'],example['reference'])
   self.assertGreater(len({tuple(c['expected'])for c in assignment['cases']}),1)
   for scope,task in [('assignment',assignment),('practice',example)]:
    initial=lab['initial']['simulation'][scope];self.assertEqual(initial['draft'],task['starter']);self.assertFalse(initial['checked']);self.assertFalse(initial['running']);self.assertEqual(initial['results'],[])
  recipient=read('scenarios/sieve-address-lessons-v1.json')['scenarios'][1]['goal']['definition']['assignment'];split=next(c for c in recipient['cases']if c['id']=='split-parts');self.assertEqual(split['expected'],['INBOX']);self.assertEqual(len(split['headers']),2)
  quoted=next(c for c in recipient['cases']if c['id']=='quoted-only');self.assertEqual(quoted['expected'],['INBOX'])
 def test_changed_predecessor_or_additions_are_refused_by_both_strict_origin_checks(self):
  source='scenarios/sieve-address-lessons-v1.json';selected='scenarios/boot-entry-lessons-v48.json';lab=read(source)['scenarios'][0];self.assertTrue(teaching.same_preserved_origin(lab,source,selected));original=Path.read_text
  origin={s['id']:s for s in read(source)['scenarios']}
  for index in [0,97]:
   bad=read(selected);bad['scenarios'][index]['title']='Unreviewed changed task'
   def altered(path,*a,**kw):return json.dumps(bad)if path==ROOT/selected else original(path,*a,**kw)
   with patch.object(Path,'read_text',altered),self.assertRaisesRegex(ValueError,'Address-filter consolidation changed'):teaching.same_preserved_origin(lab,source,selected)
   with self.assertRaisesRegex(ValueError,'Address-filter consolidation changed'):coverage.preserved_authored_pack(source,origin,{selected:{s['id']:s for s in bad['scenarios']}})
 def test_scope_has_no_host_bridge_and_native_runs_choose_the_source_task_profile(self):
  for name in ['src/sieve_address.rs','src/sieve_parse.rs','src/sieve_eval.rs','src/sieve_workspace.rs','src/ui/sieve_page.rs']:
   text=(ROOT/name).read_text()
   for forbidden in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','unsafe {']:self.assertNotIn(forbidden,text)
  page=(ROOT/'src/ui/sieve_page.rs').read_text();self.assertIn('s.start_for(t)',page)
  source=(ROOT/'src/sieve_workspace.rs').read_text();self.assertIn('skip_serializing_if = "is_false"',source);self.assertIn('Action::Run => s.start_for(t)',source)
