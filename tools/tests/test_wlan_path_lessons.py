"""Integrated AP/controller lessons: actual retained values, distinct tasks and honest scope."""
import csv
import hashlib
import json
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory, verify_coverage
IDS=['ccna-wlc-campus-forwarding','ccna-flexconnect-local-vlan']
def read(name):return json.loads((ROOT/name).read_text())
def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
class WlanPathLessons(unittest.TestCase):
    def test_exact_verified424_tail_and_98_teaching_values_remain(self):
        old=read('scenarios/boot-entry-lessons-v25.json')['scenarios'];new=read('scenarios/boot-entry-lessons-v26.json')['scenarios']
        self.assertEqual((len(old),len(new)),(51,53));self.assertEqual(new[:51],old)
        self.assertEqual([s['id']for s in new[51:]],IDS)
        raw=canonical(new[:51]);self.assertEqual((len(raw),hashlib.sha256(raw).hexdigest()),(503194,'054f5284815c422480fe05042f7ff1d366da27425ce6ae75ea9ed47c5bd05faf'))
        rows=read('scenarios/presentation/ccna.json');self.assertEqual(len(rows),113)
        raw=canonical(rows[:98]);self.assertEqual((len(raw),hashlib.sha256(raw).hexdigest()),(356546,'30791f56c0458d52a7e4d17e0da73f44f92509556da9a5e56d8e8c9d9b5d5a43'))
        self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v25.json',{s['id']:s for s in old},{'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in read('scenarios/boot-entry-lessons-v36.json')['scenarios']}}))
        changed=json.loads(json.dumps(old));changed[0]['objective']+=' changed'
        with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):
            verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v25.json',{s['id']:s for s in changed},{'scenarios/boot-entry-lessons-v36.json':{s['id']:s for s in read('scenarios/boot-entry-lessons-v36.json')['scenarios']}})
    def test_normal_registry_and_unit_prerequisites(self):
        paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
        labs=teaching_inventory.catalog();self.assertEqual(len(labs),482)
        unit=read('scenarios/wlan-path-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['vlan-data-foundations','ccna-wlan-personal-security'])
        self.assertEqual([s['id']for s,_ in labs if s['unit']==unit['id']],IDS)
    def test_individually_reviewed_teaching_separates_worked_sites_and_full_assignments(self):
        rows={r['id']:r for r in read('scenarios/presentation/ccna.json')}
        for s in read('scenarios/boot-entry-lessons-v26.json')['scenarios'][-2:]:
            r=rows[s['id']];d=s['goal']['definition']
            self.assertEqual(r['status'],'updated');self.assertEqual(r['source'],'scenarios/boot-entry-lessons-v26.json')
            self.assertEqual(' '.join(r['requirements']),s['objective'])
            self.assertEqual((len(r['concepts']),len(r['practice']),len(r['requirements'])),(4,3,4))
            self.assertGreaterEqual(len(r['worked']),5)
            self.assertNotEqual(d['initial']['id'],d['example']['id']);self.assertNotEqual(d['initial']['initial'],d['example']['initial'])
            self.assertTrue(all(len(r['review'][v])>100 for v in ['introduction','example','assignment']))
            self.assertTrue(all(p['action']and p['observe']for p in r['practice']))
            self.assertIn('Check paths',r['practice'][1]['action']);self.assertIn('Fill worked configuration',r['practice'][1]['action'])
            self.assertIsNone(s['initial']['simulation']['assignment']['checked_fields'])
            self.assertTrue(all(c.startswith(('@ap-edit ','@ap-check'))for c in s['solution']))
    def test_objective_credit_is_narrow_and_keeps_exclusions(self):
        with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='CCNA v1.1'}
        for ref in ['2.6','2.7']:
            r=rows[ref];self.assertEqual(r['status'],'Partial practice');self.assertEqual(r['implemented_scenarios'].split(';'),IDS)
            for term in ['CAPWAP','other AP modes','LAG','incomplete']:self.assertIn(term,r['remaining'])
            self.assertIn('tests/wlan_path_content.rs',r['evidence_tests'])
        with(ROOT/'docs/CONTINUATION_COVERAGE.csv').open()as f:labs={r['lab_id']:r for r in csv.DictReader(f)}
        for id in IDS:self.assertEqual(labs[id]['evidence'],'WLAN_PATH_LESSONS.md')
if __name__=='__main__':unittest.main()
