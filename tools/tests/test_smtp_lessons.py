"""SMTP source/content/coverage guards; never a host MTA or learner execution."""
import csv, hashlib, json, sys, unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
IDS=['lpic2-smtp-destination-role','lpic2-smtp-relay-boundary','lpic2-smtp-queue-recovery']
def read(p):return json.loads((ROOT/p).read_text())
class SmtpLessons(unittest.TestCase):
 def test_actual_registered_catalog_and_exact_predecessors_fit_unchanged_limits(self):
  paths=coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  before=read('scenarios/boot-entry-lessons-v45.json')['scenarios'];added=read('scenarios/smtp-routing-lessons-v1.json')['scenarios'];self.assertEqual(len(before),91);self.assertEqual([s['id']for s in added],IDS);self.assertEqual(read('scenarios/boot-entry-lessons-v46.json')['scenarios'],before+added);self.assertLess((ROOT/paths[-1]).stat().st_size,2_000_000)
  labs=teaching.catalog();self.assertEqual(len(labs),482);self.assertEqual([s['id']for s,_ in labs][464:467],IDS)
  unit=read('scenarios/smtp-routing-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['linux-local-mail-routing','linux-host-networking'])
 def test_preserved117_presentations_and_each_new_screen_have_subject_specific_review(self):
  rows=read('scenarios/presentation/lpic2.json');self.assertEqual(len(rows),135);old=json.dumps(rows[:117],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode();self.assertEqual(hashlib.sha256(old).hexdigest(),'4ee752dde716e41099b81a56e38896bf9260b81d33eca568be2d97fd488f2afe')
  for lab,row in zip(read('scenarios/smtp-routing-lessons-v1.json')['scenarios'],rows[117:120]):
   self.assertEqual(lab['id'],row['id']);self.assertEqual(' '.join(row['requirements']),lab['objective']);self.assertEqual(row['source'],'scenarios/smtp-routing-lessons-v1.json');self.assertEqual(row['status'],'updated')
   for part in ['concepts','worked','practice']:self.assertEqual(len(row[part]),3)
   setting=row['worked'][0];self.assertEqual(setting['label'],next(iter(lab['goal']['definition']['assignment']['editable'].values()))[0]);self.assertNotIn(' = ',setting['value'])
   self.assertTrue(all('→'not in w['label']for w in row['worked']))
   for phase in ['introduction','example','assignment']:self.assertGreater(len(row['review'][phase]),60)
   self.assertIn('Save configuration',row['practice'][0]['action']);self.assertIn('postfix reload',row['practice'][1]['action']);self.assertIn('Check configuration',row['practice'][2]['action'])
 def test_faults_and_positive_negative_cases_are_real_and_practice_is_not_the_assignment(self):
  labs=read('scenarios/smtp-routing-lessons-v1.json')['scenarios']
  for lab,field in zip(labs,['mydestination','mynetworks','relayhost']):
   d=lab['goal']['definition'];a=d['assignment'];e=d['example'];self.assertEqual(len(a['probes']),9);self.assertEqual(len(a['queued']),2);self.assertNotEqual(a['facts'],e['facts']);self.assertEqual(next(iter(a['editable'].values())),[field]);self.assertEqual({p['expected']['kind']for p in a['probes']},{'Delivery','Reject'})
   for scope in ['assignment','practice']:self.assertFalse(lab['initial']['simulation'][scope]['checked']);self.assertTrue(all(m['status']=={'kind':'Queued'}for m in lab['initial']['simulation'][scope]['runtime']['messages'].values()))
   self.assertNotEqual(a['facts']['initial_files'][d['reference'][0]['node']],d['reference'][0]['text'])
  self.assertIn('mydestination = edge.dispatch.test, localhost, team.dispatch.test',labs[0]['goal']['definition']['assignment']['facts']['initial_files']['edge']);self.assertIn('mynetworks = 0.0.0.0/0',labs[1]['goal']['definition']['assignment']['facts']['initial_files']['edge']);self.assertIn('relayhost = [edge.dispatch.test]',labs[2]['goal']['definition']['assignment']['facts']['initial_files']['transit'])
 def test_only_implemented_configuration_scope_gets_partial_credit(self):
  with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='LPIC-2 v4.5'and r['parent']=='211.1'}
  for key,row in rows.items():
   self.assertEqual(row['status'],'Partial practice'if key in {'211.1','211.1.K1','211.1.T1','211.1.T2'}else'Open');self.assertIn('no TLS',row['remaining'])
   if row['status']=='Partial practice':self.assertEqual(row['implemented_scenarios'].split(';'),IDS)
  for p in ['src/smtp_config.rs','src/smtp_routing.rs','src/smtp_queue.rs','src/smtp_equivalence.rs','src/smtp_workspace.rs','src/smtp_console.rs','src/smtp_source.rs','src/ui/smtp_page.rs']:
   source=(ROOT/p).read_text()
   for forbidden in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','unsafe {']:self.assertNotIn(forbidden,source)
 def test_changed_prefix_or_tail_cannot_disguise_itself_as_a_preserved_origin(self):
  lab=read('scenarios/smtp-routing-lessons-v1.json')['scenarios'][0];origin='scenarios/smtp-routing-lessons-v1.json';selected='scenarios/boot-entry-lessons-v46.json';self.assertTrue(teaching.same_preserved_origin(lab,origin,selected));original=Path.read_text
  for index in [0,91]:
   bad=read(selected);bad['scenarios'][index]['title']='An unreviewed changed title'
   def altered(path,*a,**kw):return json.dumps(bad)if path==ROOT/selected else original(path,*a,**kw)
   with patch.object(Path,'read_text',altered),self.assertRaisesRegex(ValueError,'SMTP consolidation changed'):teaching.same_preserved_origin(lab,origin,selected)
