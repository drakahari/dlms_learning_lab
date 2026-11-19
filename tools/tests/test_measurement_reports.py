"""Teaching/registration retention; behavior is tested independently in Rust."""
import hashlib,json,unittest
from pathlib import Path
R=Path(__file__).resolve().parents[2]
IDS=['shell-counter-rates','shell-cpu-share','shell-measurement-recovery']
def read(n):return json.loads((R/n).read_text())
class MeasurementTeaching(unittest.TestCase):
 def test_preserves_every_old_scenario_and_presentation(self):
  old=read('scenarios/boot-entry-lessons-v14.json')['scenarios'];new=read('scenarios/boot-entry-lessons-v15.json')['scenarios']
  self.assertEqual(len(old),23);self.assertEqual(new[:23],old);self.assertEqual([s['id']for s in new[23:]],IDS)
  rows=read('scenarios/presentation/bash.json');self.assertEqual(len(rows),80)
  self.assertEqual(hashlib.sha256(json.dumps(rows[:77],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest(),'997bff349741bc71a3d5537fb2baf148e887ca6c05b87bb55efc6693b9732180')
  self.assertEqual([r['id']for r in rows[77:]],IDS)
 def test_teaches_interval_semantics_and_names_actual_practice_controls(self):
  rows=read('scenarios/presentation/bash.json')[-3:]
  for row in rows:
   self.assertTrue(all(set(w)=={'label','value','explanation'}for w in row['worked']))
   self.assertEqual(len(row['practice']),3);self.assertGreaterEqual(len(row['concepts']),5)
   text=' '.join(c['text']for c in row['concepts'])
   self.assertGreater(len(text),1300)
   practice=' '.join(str(p)for p in row['practice'])
   for term in ['Run example','Expected preview','Actual']:self.assertIn(term,practice)
   self.assertIn('compare',practice.lower())
   self.assertNotIn('automatically compare',practice.lower())
   for view in ['introduction','example','assignment']:self.assertTrue(row['review'][view])
   self.assertGreaterEqual(len(row['requirements']),6)
   self.assertNotIn('Show worked answer',' '.join(row['requirements']))
  self.assertIn('Expand Expected preview',rows[0]['worked'][-1]['explanation'])
  self.assertIn('Multiply before dividing',rows[0]['concepts'][3]['text'])
  self.assertIn('decrease alone does not prove a reboot',rows[1]['concepts'][4]['text'])
  self.assertIn('portable',rows[2]['concepts'][4]['text'])
 def test_unit_keeps_prerequisite_order_and_supported_scope(self):
  unit=read('scenarios/measurement-report-units-v1.json')['units'][0]
  self.assertEqual(unit['id'],'shell-measurement-reports');self.assertEqual(unit['labs'],IDS)
  self.assertIn('shell-log-correlation',unit['prerequisites'])
  labs=read('scenarios/boot-entry-lessons-v15.json')['scenarios'][-3:]
  for lab in labs:
   self.assertEqual(lab['unit'],unit['id']);self.assertEqual(len(lab['goal']['cases']),12)
   self.assertNotEqual(lab['initial']['source'],lab['teaching']['example'])
  for fixture in ['rates','cpu','recovery']:
   f=read(f'tests/fixtures/shell-measurement-{fixture}-reference.json')
   self.assertNotEqual(f['source'],f['alternate']);self.assertEqual(len(f['cases']),12)
