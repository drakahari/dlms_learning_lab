"""Individual teaching/source checks; execution/alternatives are tested in Rust."""
import hashlib,json,unittest
from pathlib import Path
R=Path(__file__).resolve().parents[2]
IDS=['lpic1-layout-log-growth','lpic1-layout-media-policy']
def read(n):return json.loads((R/n).read_text())
class LayoutTeaching(unittest.TestCase):
 def test_preserves_394_catalog_sources_and_all_131_lpic1_reviews(self):
  old=read('scenarios/boot-entry-lessons-v13.json')['scenarios'];new=read('scenarios/boot-entry-lessons-v14.json')['scenarios']
  self.assertEqual((len(old),len(new)),(21,23));self.assertEqual(old,new[:21]);self.assertEqual([s['id']for s in new[21:]],IDS)
  rows=read('scenarios/presentation/lpic1.json');self.assertEqual(len(rows),154)
  self.assertEqual(hashlib.sha256(json.dumps(rows[:131],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),'a2053f926dd83da88558644f6e5a7e25905689e30c0ba2bce63e95a99d303873')
  self.assertEqual([r['id']for r in rows[131:133]],IDS)
 def test_teaches_units_actions_constraints_and_recovery_before_independent_work(self):
  labs=read('scenarios/boot-entry-lessons-v14.json')['scenarios'][-2:];rows=[next(r for r in read('scenarios/presentation/lpic1.json') if r['id']==id)for id in IDS]
  for lab,row in zip(labs,rows):
   self.assertEqual(row['id'],lab['id']);self.assertEqual(' '.join(row['requirements']),lab['objective'])
   self.assertEqual(len(row['practice']),3);self.assertTrue(all(p['action']and p['observe']for p in row['practice']))
   text=' '.join(c['text']for c in row['concepts'])+' '+str(row['worked'])
   for term in ['GPT','partition','Reset practice','ext4']:self.assertIn(term,text)
   d=lab['goal']['definition'];self.assertNotEqual(d['initial'],d['lesson']['fixture']);self.assertEqual(d['plan']['layout']['version'],1)
   self.assertTrue(any('Expected' in w['label']for w in row['worked']))
   self.assertFalse(lab['initial']['simulation']['checked']);self.assertNotIn('worked',str(row['requirements']).lower())
  text=' '.join(c['text']for c in rows[0]['concepts'])+' '+str(rows[0]['worked'])
  for term in ['512-byte','2048 sectors','− 1','unallocated','reserved GPT']:self.assertIn(term,text)
  self.assertIn('Separate fixed filesystems already isolate',' '.join(c['text']for c in rows[1]['concepts']))
  self.assertIn('not a measurement of I/O performance',' '.join(c['text']for c in rows[1]['concepts']))
 def test_unit_follows_actual_preparation_and_challenges_different_relationships(self):
  unit=read('scenarios/layout-planning-units-v1.json')['units'][0]
  self.assertEqual(unit['prerequisites'],['linux-disk-preparation']);self.assertEqual(unit['labs'],IDS)
  a,b=read('scenarios/boot-entry-lessons-v14.json')['scenarios'][-2:]
  self.assertEqual(a['prerequisites'],['lpic1-disk-delivery']);self.assertEqual(b['prerequisites'],[IDS[0]])
  rules=lambda s:s['goal']['definition']['plan']['layout']['rules']
  self.assertEqual([r['kind']for r in rules(a)],['adjacent_free']);self.assertEqual({r['kind']for r in rules(b)},{'adjacent_free','separate_disks'})

 def test_coverage_marks_only_the_verified_planning_slice(self):
  import csv
  with (R/'docs/SKILL_COVERAGE.csv').open()as f:rows=[r for r in csv.DictReader(f)if r['blueprint']=='LPIC-1 v5.0'and r['parent']=='102.1']
  covered={r['reference']for r in rows if r['status']=='Partial practice'}
  self.assertEqual(covered,{'102.1','102.1.K1','102.1.K2','102.1.T1','102.1.T6','102.1.T8'})
  for r in rows:
   if r['reference']in covered:
    self.assertEqual(r['implemented_scenarios'].split(';'),IDS)
    for term in ['/boot','ESP','LVM','No real','mount','swap activation']:self.assertIn(term,r['remaining'])
   else:self.assertEqual(r['status'],'Open')
