"""Credit existing bounded hands-on work without inheriting whole-family coverage."""
import csv,json,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory
class LinuxExistingMapping(unittest.TestCase):
 def test_real_registered_scenarios_support_only_explicit_narrow_slices(self):
  labs={s['id']:s for s,_ in teaching_inventory.catalog()}
  with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='LPIC-2 v4.5'}
  for key,ids,evidence in [('201.1',['lpic1-module-device-driver','lpic1-module-index-repair','lpic2-module-options-reload'],'tests/kernel_module_lessons.rs'),('203.2',['lpic1-fs-offline-accounting','lpic1-fs-retained-recovery'],'tests/fs_integrity_lessons.rs')]:
   r=rows[key];self.assertEqual(r['status'],'Partial practice');self.assertEqual(r['implemented_scenarios'].split(';'),ids);self.assertEqual(r['evidence_tests'],evidence);self.assertIn('Mapping correction only',r['verified_scope']);self.assertIn('incomplete',r['remaining'])
   teaching={r['id']:r for name in teaching_inventory.SHARDS for r in json.loads((ROOT/f'scenarios/presentation/{name}.json').read_text())}
   for id in ids:
    self.assertTrue(labs[id]['solution'])
    entry=teaching[id];self.assertEqual(entry['status'],'updated')
    for field in ['concepts','worked','practice','requirements']:self.assertTrue(entry[field],(id,field))
    for view in ['introduction','example','assignment']:self.assertTrue(entry['review'][view],(id,view))
  for key in ['203.2.K1','203.2.T3']:self.assertEqual(rows[key]['implemented_scenarios'],rows['203.2']['implemented_scenarios'])
  for key in ['201.1.K1','201.1.T1','201.1.T2','201.1.T3','201.1.T4','201.1.T5','203.2.K2','203.2.K3','203.2.K4','203.2.T4','203.2.T5','203.2.T6','203.2.T7']:self.assertEqual(rows[key]['status'],'Open',key)
 def test_whole_objective_and_capacity_inheritance_are_not_claimed(self):
  with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows=[r for r in csv.DictReader(f)if r['blueprint']=='LPIC-2 v4.5'and r['parent']in ['201.1','203.2']]
  for r in rows:self.assertNotIn('lpic2-capacity-inodes',r['implemented_scenarios']);self.assertNotIn('Complete',r['status'])
  with(ROOT/'docs/CURRICULUM.csv').open()as f:rows={r['objective_id']:r for r in csv.DictReader(f)if r['blueprint']=='LPIC-2 v4.5'}
  for key in ['201.1','203.2']:self.assertEqual(rows[key]['status'],'Partial');self.assertIn('incomplete',rows[key]['gap_or_fidelity_limit'])
 def test_service_practice_gains_only_its_taught_slices_and_keeps_the_remaining_bridge_open(self):
  with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='LPIC-1 v5.0'and r['parent']=='109.1'}
  self.assertEqual(len(rows),11)
  for key in ['109.1.K3','109.1.T1']:
   r=rows[key];self.assertEqual(r['status'],'Partial practice');self.assertEqual(r['implemented_scenarios'].split(';'),['lpic1-service-client-targets','lpic1-services-file-repair']);self.assertEqual(r['evidence_tests'],'tests/service_name_content.rs');self.assertIn('ICMP',r['remaining']);self.assertIn('full listed',r['remaining']);self.assertNotIn('Complete',r['status'])
  for key in ['109.1.K4','109.1.T4']:
   r=rows[key];self.assertEqual(r['status'],'Partial practice');self.assertEqual(r['implemented_scenarios'].split(';'),['lpic1-service-client-targets','lpic1-services-file-repair','lpic1-protocol-round-trips']);self.assertEqual(r['evidence_tests'],'tests/service_name_content.rs;tests/protocol_evidence_content.rs');self.assertIn('ICMP error',r['remaining']);self.assertIn('Linux firewall',r['remaining']);self.assertNotIn('Complete',r['status'])
  for key in ['109.1','109.1.K1','109.1.K5','109.1.K6','109.1.T2','109.1.T3']:
   r=rows[key];self.assertEqual(r['status'],'Partial practice');self.assertIn('lpic1-host-mixed-ip',r['implemented_scenarios']);self.assertNotIn('Complete',r['status'])
  r=rows['109.1.K2'];self.assertEqual(r['status'],'Supporting explanation');self.assertEqual(r['implemented_scenarios'],'lpic1-host-routing');self.assertIn('classification',r['remaining'])
if __name__=='__main__':unittest.main()
