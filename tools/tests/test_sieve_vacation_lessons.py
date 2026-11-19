"""Current vacation registration, retained origins, content and simulation boundary."""
import hashlib,json,sys,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
IDS=['lpic2-sieve-vacation-safety','lpic2-sieve-vacation-routing-recovery']
def read(name):return json.loads((ROOT/name).read_text())
class VacationLessons(unittest.TestCase):
 def test_selected_current_is_exact475_plus_two_real_tasks(self):
  paths=coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  before=read('scenarios/boot-entry-lessons-v49.json')['scenarios'];added=read('scenarios/sieve-vacation-lessons-v1.json')['scenarios'];self.assertEqual(len(before),102);self.assertEqual([r['id']for r in added],IDS);self.assertEqual(read('scenarios/boot-entry-lessons-v50.json')['scenarios'],before+added)
  labs=teaching.catalog();self.assertEqual(len(labs),482);self.assertEqual([s['id']for s,_ in labs][475:477],IDS)
  self.assertLessEqual((ROOT/'src/pack.rs').stat().st_size,512000);self.assertLess((ROOT/paths[-1]).stat().st_size,2_000_000)
 def test_every_prior_teaching_value_is_exact_and_new_views_are_individually_authored(self):
  rows=read('scenarios/presentation/lpic2.json');self.assertEqual(len(rows),135)
  old=json.dumps(rows[:128],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();self.assertEqual(hashlib.sha256(old).hexdigest(),'f773a60e9911fbffa38ae87a392c937579759bc7f50610e4b53d0595d0a247bd')
  for row,lab in zip(rows[128:130],read('scenarios/sieve-vacation-lessons-v1.json')['scenarios']):
   self.assertEqual(row['id'],lab['id']);self.assertEqual(' '.join(row['requirements']),lab['objective']);self.assertEqual(len(row['concepts']),4)
   self.assertEqual(len(row['practice']),3);self.assertEqual(len(row['worked']),3)
   for phase in ['introduction','example','assignment']:self.assertGreater(len(row['review'][phase]),60)
   self.assertIn(row['status'],['pending','updated'])
 def test_timeline_experiments_are_separate_and_independent_starters_need_work(self):
  for lab,n in zip(read('scenarios/sieve-vacation-lessons-v1.json')['scenarios'],[11,10]):
   d=lab['goal']['definition'];a,e=d['assignment'],d['example'];self.assertNotEqual(a['reference'],e['reference']);self.assertNotEqual(a['cases'],e['cases']);self.assertEqual(len(a['cases']),n);self.assertEqual(len(e['cases']),n)
   for key,t in [('assignment',a),('practice',e)]:
    self.assertTrue(t['address_tests']and t['delivery_actions']);self.assertIn('vacation',t);self.assertNotEqual(t['starter'],t['reference'])
    arrivals=[c['arrival_seconds']for c in t['cases']];self.assertEqual(arrivals,sorted(arrivals));self.assertTrue(all(type(c['expected_reply'])is bool for c in t['cases']))
    s=lab['initial']['simulation'][key];self.assertEqual(s['draft'],t['starter']);self.assertEqual(s['results'],[]);self.assertFalse(s['checked']or s['running'])
  a=read('scenarios/sieve-vacation-lessons-v1.json')['scenarios'][0]['goal']['definition']['assignment'];self.assertEqual(a['cases'][0]['headers'],a['cases'][1]['headers']);self.assertEqual(a['cases'][0]['headers'],a['cases'][2]['headers']);self.assertEqual([c['expected_reply']for c in a['cases'][:3]],[True,False,True])
 def test_changed_predecessor_or_new_tail_fails_both_origin_checks(self):
  leaf='scenarios/sieve-vacation-lessons-v1.json';selected='scenarios/boot-entry-lessons-v50.json';origin={s['id']:s for s in read(leaf)['scenarios']};lab=next(iter(origin.values()));self.assertTrue(teaching.same_preserved_origin(lab,leaf,selected));original=Path.read_text
  for index in [0,102]:
   bad=read(selected);bad['scenarios'][index]['title']='Changed unreviewed task'
   def altered(path,*args,**kwargs):return json.dumps(bad)if path==ROOT/selected else original(path,*args,**kwargs)
   with patch.object(Path,'read_text',altered),self.assertRaisesRegex(ValueError,'Vacation-filter consolidation changed'):teaching.same_preserved_origin(lab,leaf,selected)
   with self.assertRaisesRegex(ValueError,'Vacation-filter consolidation changed'):coverage.preserved_authored_pack(leaf,origin,{selected:{s['id']:s for s in bad['scenarios']}})
 def test_unit_and_essential_teaching_match_actual_controls(self):
  unit=read('scenarios/sieve-vacation-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['linux-sieve-envelope-delivery'])
  rows=read('scenarios/presentation/lpic2.json')[128:130];self.assertEqual([r['id']for r in rows],IDS);first=' '.join(c['text']for c in rows[0]['concepts'])
  for term in ['require "vacation"',':days 2','172800','MAIL FROM','INBOX','Run filter','Stop']:self.assertIn(term,first)
  second=' '.join(c['text']for c in rows[1]['concepts']);self.assertIn('stop',second);self.assertIn(':handle',second)
  for row in rows:
   steps=' '.join(s['action']for s in row['practice']);self.assertIn('Fill worked example',steps);self.assertIn('Run filter',steps)
 def test_runtime_has_no_host_execution_bridge_and_native_stage_is_closed(self):
  for name in ['src/sieve_vacation_history.rs','src/sieve_vacation_policy.rs','src/sieve_parse.rs','src/sieve_eval.rs','src/sieve_workspace.rs','src/ui/sieve_page.rs']:
   source=(ROOT/name).read_text()
   for forbidden in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','unsafe {']:self.assertNotIn(forbidden,source)
  stage=(ROOT/'src/ui/smoke/sieve_vacation.rs').read_text();self.assertIn('const IDS: [&str; 2]',stage);self.assertIn('Op::Shot(tag("actual-reply"), "reply-body".into())',stage);self.assertIn('p.len(), 128',stage);self.assertIn('names.len(), 46',stage)
