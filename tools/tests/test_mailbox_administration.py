"""Current authored capstone, exact predecessor and honest scope. No host execution."""
import hashlib,json,sys,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
ID='lpic2-mailbox-administration-recovery';BEFORE='scenarios/boot-entry-lessons-v52.json';LEAF='scenarios/mailbox-administration-lessons-v1.json';SELECTED='scenarios/boot-entry-lessons-v53.json'
def read(n):return json.loads((ROOT/n).read_text())
def digest(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
class MailAdministration(unittest.TestCase):
 def test_one_registered_assignment_retains_the_entire_481_catalog_and_134_teaching_values(self):
  paths=coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,SELECTED));labs=teaching.catalog();self.assertEqual(len(labs),482);self.assertEqual(labs[-1][0]['id'],ID);self.assertEqual(digest([s for s,_ in labs[:481]]),'d125f063c782d73142e3bb886bdd74826f1ea88e62a277bfe83f407af503af1b')
  old=read(BEFORE)['scenarios'];new=read(LEAF)['scenarios'];self.assertEqual((len(old),len(new)),(108,1));self.assertEqual(read(SELECTED)['scenarios'],old+new);self.assertLess((ROOT/SELECTED).stat().st_size,2_000_000);self.assertLessEqual((ROOT/'src/pack.rs').stat().st_size,512000)
  rows=read('scenarios/presentation/lpic2.json');self.assertEqual(len(rows),135);self.assertEqual(digest(rows[:134]),'2c53943a539de193c7d4935055094278e5e04f6b241732e3462261c4ea0bee76');self.assertEqual(rows[-1]['id'],ID)
  unit=read('scenarios/mailbox-administration-units-v1.json')['units'][0];self.assertEqual(unit['labs'],[ID]);self.assertEqual(unit['prerequisites'],['linux-mailbox-access'])
 def test_retained_478_factory_is_verbatim_without_altering_the_475_body(self):
  text=(ROOT/'src/pack_retained_mail.rs').read_text();body=text[text.index('fn accepted478_texts()'):text.index('\npub(super) fn texts478()')];self.assertEqual((len(body.encode()),hashlib.sha256(body.encode()).hexdigest()),(7757,'786781ee4f8d1ba409cd9aa0ce2e952c7ccb70adfb8d563566f0bf7af4a67b45'))
  body=text[text.index('fn accepted475_texts()'):text.index('\npub(super) fn texts()')];self.assertEqual(hashlib.sha256(body.encode()).hexdigest(),'db16f83300d240639071b0393b64f04379042625c7b20571b84ca4f367099f96')
 def test_all_three_views_use_actual_controls_and_distinct_practice_without_claiming_courier_administration(self):
  s=read(LEAF)['scenarios'][0];p=read('scenarios/presentation/lpic2.json')[-1];self.assertEqual(s['steps'][0],'Understand the task');self.assertEqual(' '.join(p['requirements']),s['objective']);self.assertEqual((len(p['concepts']),len(p['practice']),len(p['worked'])),(4,3,4));self.assertIn('Courier',p['concepts'][3]['text']);self.assertIn('not Courier administration',p['concepts'][3]['text']);self.assertNotIn('Reset only the example',' '.join(r['action']for r in p['practice']))
  d=s['goal']['definition'];self.assertNotEqual(d['assignment']['facts']['accounts'],d['example']['facts']['accounts']);self.assertNotEqual(d['assignment']['saved'],d['worked_config']);self.assertEqual(s['initial']['simulation']['assignment']['input'],'');self.assertFalse(s['initial']['simulation']['assignment']['checked']);self.assertEqual(s['goal']['kind'],'MailboxAdministration')
  commands=[c['command']for g in s['guide']for c in g['commands']];self.assertTrue(all(c.startswith(('doveconf ','doveadm '))for c in commands));self.assertFalse(any(c.startswith('doveconf -n')for c in commands));self.assertIn('full/default dumps',s['simplifications']);self.assertIn('UIDs',p['practice'][2]['action'])
 def test_changed_predecessor_or_new_assignment_is_rejected_by_both_alias_guards(self):
  origin={s['id']:s for s in read(LEAF)['scenarios']};lab=origin[ID];self.assertTrue(teaching.same_preserved_origin(lab,LEAF,SELECTED));self.assertTrue(coverage.preserved_authored_pack(LEAF,origin,{SELECTED:{s['id']:s for s in read(SELECTED)['scenarios']}}));original=Path.read_text
  for at in[0,108]:
   changed=read(SELECTED);changed['scenarios'][at]['title']='Unreviewed replacement'
   def altered(path,*a,**kw):return json.dumps(changed)if path==ROOT/SELECTED else original(path,*a,**kw)
   with patch.object(Path,'read_text',altered),self.assertRaisesRegex(ValueError,'Mail-administration consolidation changed'):teaching.same_preserved_origin(lab,LEAF,SELECTED)
   with self.assertRaisesRegex(ValueError,'Mail-administration consolidation changed'):coverage.preserved_authored_pack(LEAF,origin,{SELECTED:{s['id']:s for s in changed['scenarios']}})
 def test_new_utility_is_bounded_in_process_and_never_uses_host_execution(self):
  text=(ROOT/'src/mailbox_admin.rs').read_text()
  for forbidden in['std::fs','std::process','Command::new','TcpStream','UdpSocket','unsafe {']:self.assertNotIn(forbidden,text)
  for required in['line.len() > 2048','words.len() > 64','q.predicates.len() > 16','result.len() > 16','out.text.len() > 32 * 1024','let mut scratch = r.clone()','r.validate(f)?']:self.assertIn(required,text)
