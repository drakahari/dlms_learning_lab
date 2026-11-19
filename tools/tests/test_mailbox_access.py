"""Closed mail-access content, exact predecessor, honest subset and alias guards."""
import hashlib,json,sys,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
IDS=['lpic2-dovecot-secure-access','lpic2-imap-mailbox-recovery','lpic2-pop-transaction-recovery']
LEAF='scenarios/mailbox-access-lessons-v1.json';BEFORE='scenarios/boot-entry-lessons-v51.json';SELECTED='scenarios/boot-entry-lessons-v52.json'
def read(name):return json.loads((ROOT/name).read_text())
class MailboxAccess(unittest.TestCase):
 def test_registered_catalog_retains_the_complete_predecessor_and_three_real_tasks(self):
  paths=coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  before=read(BEFORE)['scenarios'];added=read(LEAF)['scenarios'];self.assertEqual((len(before),len(added)),(105,3));self.assertEqual(read(SELECTED)['scenarios'],before+added)
  labs=teaching.catalog();self.assertEqual(len(labs),482);self.assertEqual([s['id']for s,_ in labs[478:481]],IDS);self.assertLess((ROOT/SELECTED).stat().st_size,2_000_000);self.assertLess((ROOT/'src/pack.rs').stat().st_size,512000)
  unit=read('scenarios/mailbox-access-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['linux-smtp-routing','linux-host-networking'])
 def test_retained_factory_body_and_existing_presentation_values_stay_exact(self):
  text=(ROOT/'src/pack_retained_mail.rs').read_text();body=text[text.index('fn accepted475_texts()'):text.index('\npub(super) fn texts()')]
  self.assertEqual((len(body.encode()),hashlib.sha256(body.encode()).hexdigest()),(7758,'db16f83300d240639071b0393b64f04379042625c7b20571b84ca4f367099f96'))
  old=(ROOT/'src/pack_retained.rs').read_bytes();self.assertEqual(hashlib.sha256(old).hexdigest(),'4fc300967370e6200c67c7598c180bde3ef95ee482a0bed3e194af2f99449c4c')
  rows=read('scenarios/presentation/lpic2.json');self.assertEqual(len(rows),135)
  # All131 old objects, including the accepted migration, are an unchanged prefix.
  prefix=read('scenarios/procmail-migration-lessons-v1.json')['scenarios'][0];self.assertEqual(rows[130]['id'],prefix['id'])
  self.assertEqual(hashlib.sha256(json.dumps(rows[:130],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),'61ead6efb8262bb5f8364318685548adf15605e22e95c8a4205994e00a370a1f')
  # Complete131-entry prefix independently read back from the verified478 retained inputs.
  self.assertEqual(hashlib.sha256(json.dumps(rows[:131],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),'2cab576d13026c24033c4f3dac6ec5bcf6231bc93f0533cabdf3813cbdb80030')
 def test_each_view_teaches_the_actual_controls_without_copying_assignment_values(self):
  rows={r['id']:r for r in read('scenarios/presentation/lpic2.json')}
  for s in read(LEAF)['scenarios']:
   row=rows[s['id']];self.assertEqual(row['status'],'updated');self.assertEqual(' '.join(row['requirements']),s['objective']);self.assertEqual((len(row['concepts']),len(row['practice']),len(row['worked'])),(4,3,4))
   for view in ['introduction','example','assignment']:self.assertGreater(len(row['review'][view]),60)
   a,e=[s['goal']['definition'][key]for key in['assignment','example']];self.assertNotEqual(a['facts']['accounts'],e['facts']['accounts']);self.assertNotEqual(a['facts']['certificates'],e['facts']['certificates']);self.assertEqual(s['initial']['simulation']['assignment']['input'],'');self.assertFalse(s['initial']['simulation']['assignment']['checked'])
   self.assertTrue(s['guide'][1]['commands']);self.assertTrue(all(not c['simulator_only']for g in s['guide']for c in g['commands']))
   if a['editable_config']:self.assertNotEqual(a['saved'],s['goal']['definition']['worked_config'])
   self.assertEqual(s['steps'][0],'Understand the task');self.assertIn('no full daemon or protocol conformance',s['simplifications']);self.assertIn('no MIME, APPEND, SEARCH',s['simplifications'])
  imap=read(LEAF)['scenarios'][1]['goal']['definition'];self.assertFalse(imap['assignment']['editable_config']);self.assertIsNone(imap['worked_config'])
  left={m['uid']for b in imap['assignment']['facts']['mailboxes'].values()for m in b['INBOX']['messages']};right={m['uid']for b in imap['example']['facts']['mailboxes'].values()for m in b['INBOX']['messages']};self.assertTrue(left.isdisjoint(right))
  pop=read(LEAF)['scenarios'][2]['goal']['definition'];self.assertTrue(any(r['kind']=='pop_committed'for r in pop['assignment']['requirements']))
 def test_changed_predecessor_or_new_tail_is_refused_without_relaxing_alias_guards(self):
  origin={s['id']:s for s in read(LEAF)['scenarios']};lab=origin[IDS[0]];self.assertTrue(teaching.same_preserved_origin(lab,LEAF,SELECTED));self.assertTrue(coverage.preserved_authored_pack(LEAF,origin,{SELECTED:{s['id']:s for s in read(SELECTED)['scenarios']}}));original=Path.read_text
  for index in[0,105]:
   changed=read(SELECTED);changed['scenarios'][index]['title']='Unreviewed replacement'
   def altered(path,*a,**kw):return json.dumps(changed)if path==ROOT/SELECTED else original(path,*a,**kw)
   with patch.object(Path,'read_text',altered),self.assertRaisesRegex(ValueError,'Mailbox-access consolidation changed'):teaching.same_preserved_origin(lab,LEAF,SELECTED)
   with self.assertRaisesRegex(ValueError,'Mailbox-access consolidation changed'):coverage.preserved_authored_pack(LEAF,origin,{SELECTED:{s['id']:s for s in changed['scenarios']}})
 def test_new_models_expose_no_host_execution_primitive_and_keep_limits_explicit(self):
  for name in['src/mailbox_config.rs','src/mailbox_access.rs','src/mailbox_integrity.rs','src/mailbox_protocol.rs','src/mailbox_protocol/imap.rs','src/mailbox_workspace.rs','src/mailbox_source.rs']:
   text=(ROOT/name).read_text()
   for forbidden in['std::fs','std::process','Command::new','TcpStream','UdpSocket','unsafe {']:self.assertNotIn(forbidden,text)
  protocol=(ROOT/'src/mailbox_protocol.rs').read_text();self.assertIn('pub const MAX_COMMAND: usize = 2048;',protocol);self.assertIn('pub const MAX_COMMANDS: u16 = 128;',protocol);self.assertIn('pub const MAX_OUTPUT: usize = 64 * 1024;',protocol)
  self.assertIn('X-SIMULATOR',(ROOT/'src/mailbox_protocol/imap.rs').read_text());self.assertIn('pop_committed',(ROOT/'src/mailbox_integrity.rs').read_text())
