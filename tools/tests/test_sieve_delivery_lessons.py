"""Finite delivery-source admission and teaching guards; no host mail execution."""
import hashlib,json,sys,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
IDS=['lpic2-sieve-envelope-recipient','lpic2-sieve-redirect-recovery','lpic2-sieve-rejection-priority']
def read(n):return json.loads((ROOT/n).read_text())
class DeliveryLessons(unittest.TestCase):
 def test_selected_sources_preserve99_old_values_and_add_only_three_authored_tasks(self):
  paths=coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  old=read('scenarios/boot-entry-lessons-v48.json')['scenarios'];new=read('scenarios/sieve-delivery-lessons-v1.json')['scenarios'];self.assertEqual(len(old),99);self.assertEqual([s['id']for s in new],IDS);self.assertEqual(read('scenarios/boot-entry-lessons-v49.json')['scenarios'],old+new);self.assertLess((ROOT/paths[-1]).stat().st_size,2_000_000)
  labs=teaching.catalog();self.assertEqual(len(labs),482);self.assertEqual([s['id']for s,_ in labs][472:475],IDS)
  unit=read('scenarios/sieve-delivery-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['linux-sieve-address-routing'])
 def test_all125_prior_teaching_records_remain_exact_and_each_new_view_is_substantive(self):
  rows=read('scenarios/presentation/lpic2.json');self.assertEqual(len(rows),135)
  canonical=json.dumps(rows[:125],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();self.assertEqual(hashlib.sha256(canonical).hexdigest(),'b704e1d04cbead5e543fd0b7c1db068df9a00cd950d00b28aa19e6c90f660c5e')
  for row,lab in zip(rows[125:128],read('scenarios/sieve-delivery-lessons-v1.json')['scenarios']):
   self.assertEqual(row['id'],lab['id']);self.assertEqual(' '.join(row['requirements']),lab['objective'])
   for key in ['concepts','worked','practice']:self.assertEqual(len(row[key]),3)
   self.assertIn(row['status'],['pending','updated'])
   for phase in ['introduction','example','assignment']:self.assertGreater(len(row['review'][phase]),60)
 def test_different_worked_samples_and_intentional_assignment_errors_are_not_credited(self):
  for lab,n in zip(read('scenarios/sieve-delivery-lessons-v1.json')['scenarios'],[8,8,11]):
   d=lab['goal']['definition'];a,e=d['assignment'],d['example'];self.assertTrue(a['delivery_actions']and e['delivery_actions']);self.assertTrue(a['address_tests']and e['address_tests']);self.assertEqual(len(a['cases']),n);self.assertEqual(len(e['cases']),4);self.assertNotEqual(a['cases'],e['cases']);self.assertNotEqual(a['reference'],e['reference']);self.assertNotEqual(a['starter'],a['reference'])
   for scope,t in [('assignment',a),('practice',e)]:
    state=lab['initial']['simulation'][scope];self.assertEqual(state['draft'],t['starter']);self.assertFalse(state['checked']);self.assertFalse(state['running']);self.assertEqual(state['results'],[])
   self.assertTrue(all('envelope'in c for c in a['cases']))
  third=read('scenarios/sieve-delivery-lessons-v1.json')['scenarios'][2]['goal']['definition']['assignment'];self.assertTrue(any(c['headers'][-1]==['Subject','DISPATCH UPDATE']and c['expected']==['Updates']for c in third['cases']))
 def test_changed_old_or_added_values_fail_both_strict_origin_checks(self):
  leaf='scenarios/sieve-delivery-lessons-v1.json';selected='scenarios/boot-entry-lessons-v49.json';origin={s['id']:s for s in read(leaf)['scenarios']};lab=next(iter(origin.values()));self.assertTrue(teaching.same_preserved_origin(lab,leaf,selected));original=Path.read_text
  for at in [0,99]:
   bad=read(selected);bad['scenarios'][at]['title']='Changed unreviewed task'
   def altered(path,*a,**kw):return json.dumps(bad)if path==ROOT/selected else original(path,*a,**kw)
   with patch.object(Path,'read_text',altered),self.assertRaisesRegex(ValueError,'Delivery-filter consolidation changed'):teaching.same_preserved_origin(lab,leaf,selected)
   with self.assertRaisesRegex(ValueError,'Delivery-filter consolidation changed'):coverage.preserved_authored_pack(leaf,origin,{selected:{s['id']:s for s in bad['scenarios']}})
 def test_new_runtime_has_no_host_bridge_and_results_distinguish_effects(self):
  for n in ['src/sieve_envelope.rs','src/sieve_delivery_policy.rs','src/sieve_parse.rs','src/sieve_eval.rs','src/sieve_workspace.rs','src/ui/sieve_page.rs']:
   text=(ROOT/n).read_text()
   for forbidden in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','unsafe {']:self.assertNotIn(forbidden,text)
  page=(ROOT/'src/ui/sieve_page.rs').read_text();self.assertIn('Actual delivery envelope',page);self.assertIn('t.delivery_actions',page);self.assertIn('c.expected_description()',page);self.assertIn('r.description()',page)
  text=(ROOT/'src/sieve_workspace.rs').read_text();self.assertIn('skip_serializing_if = "BTreeSet::is_empty"',text);self.assertIn('Action::Run => s.start_for(t)',text)
 def test_native_cold_plan_uses_real_preferences_and_keeps_partial_repair(self):
  plan=(ROOT/'src/ui/smoke/reopen/sieve_delivery.rs').read_text().split('#[test]')[0];self.assertIn('replace("elsif header", "if header")',plan);self.assertIn('Op::ColdAppearance',plan);self.assertNotIn('Op::Style(',plan)

 def test_null_sender_explanation_does_not_conflate_sender_with_recipient(self):
  lab=read('scenarios/sieve-delivery-lessons-v1.json')['scenarios'][2]
  row=read('scenarios/presentation/lpic2.json')[127]
  for text in [lab['guide'][2]['why'],row['concepts'][2]['text']]:
   self.assertIn('A null return address does not remove the supplied delivery recipient',text)
   self.assertNotIn('A null sender is still a supplied delivery recipient',text)
