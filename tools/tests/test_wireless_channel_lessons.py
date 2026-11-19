"""Two distinct channel-planning lessons; no real-RF or whole-topic credit."""
import csv
import hashlib
import json
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory
import verify_coverage
IDS=['ccna-wireless-neighbour-channels','ccna-wireless-multisurvey-plan']
def read(name):return json.loads((ROOT/name).read_text())
class WirelessChannelLessons(unittest.TestCase):
    def test_retains_all_46_previous_scenarios_and_93_ccna_presentations(self):
        old=read('scenarios/boot-entry-lessons-v23.json')['scenarios'];new=read('scenarios/boot-entry-lessons-v24.json')['scenarios']
        self.assertEqual((len(old),len(new)),(46,48));self.assertEqual(new[:46],old);self.assertEqual([s['id']for s in new[46:]],IDS)
        rows=read('scenarios/presentation/ccna.json');self.assertEqual(len(rows),113)
        raw=json.dumps(rows[:93],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        # Exact canonical values read from admitted419R1's retained actual source.
        self.assertEqual((len(raw),hashlib.sha256(raw).hexdigest()),(332473,'28dbc03231cee0c2a10e7658304ec10e663b78432c5b8377152e709864ff9ec1'))
    def test_flat_registration_order_and_source_ids_are_real(self):
        paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
        labs=teaching_inventory.catalog();self.assertEqual(len(labs),482)
        unit=read('scenarios/wireless-channel-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['ipv4-foundations','ccna-wired-device-paths'])
        for s,_ in labs:
            if s['id']in IDS:
                self.assertEqual(s['unit'],unit['id']);self.assertEqual(s['id'],s['goal']['definition']['initial']['id']);self.assertEqual(s['id'],s['initial']['simulation']['assignment']['task_id'])
    def test_teaching_uses_distinct_situations_real_actions_and_individual_review(self):
        rows={r['id']:r for r in read('scenarios/presentation/ccna.json')};labs=read('scenarios/boot-entry-lessons-v24.json')['scenarios'][-2:]
        for s in labs:
            d=s['goal']['definition'];self.assertNotEqual(d['initial']['network'],d['example']['network'])
            r=rows[s['id']];self.assertEqual(r['status'],'updated');self.assertGreaterEqual(len(r['concepts']),5);self.assertEqual(len(r['practice']),3);self.assertGreaterEqual(len(r['worked']),5)
            self.assertEqual(s['objective'],' '.join(r['requirements']));self.assertTrue(all(len(r['review'][view])>60 for view in ('introduction','example','assignment')))
            self.assertTrue(all(step['action']and step['observe']for step in r['practice']));self.assertTrue(all('explanation'in w for w in r['worked']));self.assertTrue(r['goal'])
        first=labs[0]['goal']['definition']['initial']['network'];second=labs[1]['goal']['definition']['initial']['network']
        self.assertEqual(sum(r['managed']for r in first['radios']),1);self.assertEqual(sum(r['managed']for r in second['radios']),4);self.assertEqual((len(first['surveys']),len(second['surveys'])),(1,2))
    def test_channel_slice_is_preserved_without_claiming_complete_wireless_coverage(self):
        with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='CCNA v1.1'}
        r=rows['1.11.a'];self.assertEqual(r['status'],'Partial practice');self.assertEqual(r['implemented_scenarios'].split(';'),IDS)
        self.assertIn('tests/wireless_channel_content.rs',r['evidence_tests']);self.assertIn('throughput',r['remaining'])
        parent=rows['1.11'];self.assertEqual(parent['status'],'Partial practice')
        self.assertEqual(parent['implemented_scenarios'].split(';'),IDS+['ccna-wlan-ssid-repair','ccna-wlan-wpa2-personal','ccna-wlan-wpa3-handover'])
        self.assertIn('tests/wireless_channel_content.rs',parent['evidence_tests']);self.assertIn('no other bands',parent['remaining'])
        self.assertEqual(rows['1.11.c']['status'],'Supporting explanation')
        self.assertEqual(rows['2.6']['status'],'Partial practice');self.assertEqual(rows['2.6']['implemented_scenarios'].split(';'),['ccna-wlc-campus-forwarding','ccna-flexconnect-local-vlan'])
if __name__=='__main__':unittest.main()
