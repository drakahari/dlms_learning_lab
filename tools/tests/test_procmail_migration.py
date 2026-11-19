"""Single real migration task, readonly context, exact predecessor and source bounds."""
import hashlib,json,sys,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
ID='lpic2-procmail-sieve-migration'
def read(n):return json.loads((ROOT/n).read_text())
class ProcmailMigration(unittest.TestCase):
 def test_selected_catalog_retains477_and_registers_one_migration(self):
  paths=coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
  old=read('scenarios/boot-entry-lessons-v50.json')['scenarios'];new=read('scenarios/procmail-migration-lessons-v1.json')['scenarios'];self.assertEqual((len(old),len(new)),(104,1));self.assertEqual(read('scenarios/boot-entry-lessons-v51.json')['scenarios'],old+new);self.assertEqual(read(paths[-1])['scenarios'][:105],old+new)
  labs=teaching.catalog();self.assertEqual(len(labs),482);self.assertEqual(labs[477][0]['id'],ID);self.assertLessEqual((ROOT/'src/pack.rs').stat().st_size,512000);self.assertLess((ROOT/paths[-1]).stat().st_size,2_000_000)
 def test_individual_views_preserve_requirements_and_teach_before_testing(self):
  rows=read('scenarios/presentation/lpic2.json');self.assertEqual(hashlib.sha256(json.dumps(rows[:130],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),'61ead6efb8262bb5f8364318685548adf15605e22e95c8a4205994e00a370a1f');self.assertEqual(len(rows),135);row=rows[130];lab=read('scenarios/procmail-migration-lessons-v1.json')['scenarios'][0]
  self.assertEqual(row['id'],ID);self.assertEqual(' '.join(row['requirements']),lab['objective']);self.assertEqual((len(row['concepts']),len(row['practice']),len(row['worked'])),(4,3,3))
  text=' '.join(c['text']for c in row['concepts']);
  for term in ['Procmail',':0','c means copy','INBOX','keep','stop','read-only']:self.assertIn(term,text)
  for view in ['introduction','example','assignment']:self.assertGreater(len(row['review'][view]),60)
  a,e=[lab['goal']['definition'][k]for k in ['assignment','example']];self.assertNotEqual(a['legacy_recipe'],e['legacy_recipe']);self.assertNotEqual(a['reference'],e['reference']);self.assertNotEqual(a['cases'],e['cases']);self.assertEqual((len(a['cases']),len(e['cases'])),(10,10));self.assertEqual(a['starter'],e['starter']);self.assertNotIn('fileinto',a['starter'])
  for k,t in [('assignment',a),('practice',e)]:self.assertEqual(lab['initial']['simulation'][k]['draft'],t['starter']);self.assertEqual(lab['initial']['simulation'][k]['results'],[])
 def test_changed_predecessor_or_tail_is_rejected_by_both_origin_checks(self):
  leaf='scenarios/procmail-migration-lessons-v1.json';selected='scenarios/boot-entry-lessons-v51.json';origin={s['id']:s for s in read(leaf)['scenarios']};lab=origin[ID];self.assertTrue(teaching.same_preserved_origin(lab,leaf,selected));original=Path.read_text
  for index in [0,104]:
   bad=read(selected);bad['scenarios'][index]['title']='Unreviewed replacement'
   def altered(path,*a,**kw):return json.dumps(bad)if path==ROOT/selected else original(path,*a,**kw)
   with patch.object(Path,'read_text',altered),self.assertRaisesRegex(ValueError,'Legacy-migration consolidation changed'):teaching.same_preserved_origin(lab,leaf,selected)
   with self.assertRaisesRegex(ValueError,'Legacy-migration consolidation changed'):coverage.preserved_authored_pack(leaf,origin,{selected:{s['id']:s for s in bad['scenarios']}})
 def test_context_is_explicitly_inert_and_native_plan_is_closed(self):
  text=(ROOT/'src/ui/sieve_page.rs').read_text();self.assertIn('Legacy procmail recipe · read only',text);self.assertIn('legacy recipe is never run',text)
  for name in ['src/sieve_workspace.rs','src/ui/sieve_page.rs','src/sieve_source.rs']:
   text=(ROOT/name).read_text()
   for forbidden in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','unsafe {']:self.assertNotIn(forbidden,text)
  plan=(ROOT/'src/ui/smoke/procmail_migration.rs').read_text();self.assertIn('const IDS: [&str; 1]',plan);self.assertIn('p.len(), 58',plan);self.assertIn('names.len(), 20',plan)
