"""Report actual existing SSH/Telnet practice without inventing access-method coverage."""
import csv,json,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory
IDS=['ccna-management-protocols','ccna-management-source-policy']
class ManagementAccessMapping(unittest.TestCase):
    def test_mapping_reuses_registered_unchanged_lessons_and_names_missing_access_methods(self):
        labs={s['id']:s for s,_ in teaching_inventory.catalog()};self.assertEqual(len(labs),482)
        original=json.loads((ROOT/'scenarios/management-policy-lessons-v1.json').read_text())['scenarios']
        self.assertEqual([s['id']for s in original],IDS)
        for s in original:self.assertEqual(labs[s['id']],s)
        with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='CCNA v1.1'}
        r=rows['2.8'];self.assertEqual(r['status'],'Partial practice');self.assertEqual(r['implemented_scenarios'].split(';'),IDS)
        for word in ['HTTP/HTTPS','console','TACACS+/RADIUS','cloud','incomplete']:self.assertIn(word,r['remaining'])
        self.assertIn('Mapping correction only',r['verified_scope'])
        self.assertEqual(rows['4.4']['status'],'Partial practice');self.assertEqual(rows['4.4']['implemented_scenarios'].split(';'),['ccna-snmp-link-monitor','ccna-snmp-counter-monitor'])
        with(ROOT/'docs/CURRICULUM.csv').open()as f:parent=next(r for r in csv.DictReader(f)if r['blueprint']=='CCNA v1.1'and r['objective_id']=='2.8')
        self.assertEqual(parent['status'],'Partial');self.assertEqual(parent['implemented_scenarios'].split(';'),IDS)
        self.assertEqual(parent['gap_or_fidelity_limit'],r['remaining'])
    def test_historical_retention_and_current_registration_are_both_asserted(self):
        text=(ROOT/'tests/management_lessons.rs').read_text()
        for word in ['pack::accepted356()','course::accepted356(catalog())','pack::current()','course::current(&current)','(445, 176)','catalog().len(), 356','previous.len(), units.len()), (125, 126)']:self.assertIn(word,text)
if __name__=='__main__':unittest.main()
