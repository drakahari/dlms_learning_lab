"""Three distinct WLAN tasks; preserve retained source and individual teaching review."""
import csv,hashlib,json,sys,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory,verify_coverage
IDS=['ccna-wlan-ssid-repair','ccna-wlan-wpa2-personal','ccna-wlan-wpa3-handover']
def read(name):return json.loads((ROOT/name).read_text())
class WlanProfileLessons(unittest.TestCase):
    def test_exact_48_old_objects_and_95_retained_ccna_presentations_are_preserved(self):
        old=read('scenarios/boot-entry-lessons-v24.json')['scenarios'];new=read('scenarios/boot-entry-lessons-v25.json')['scenarios']
        self.assertEqual((len(old),len(new)),(48,51));self.assertEqual(new[:48],old);self.assertEqual([s['id']for s in new[48:]],IDS)
        rows=read('scenarios/presentation/ccna.json');self.assertEqual(len(rows),113)
        raw=json.dumps(rows[:95],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        # Actual canonical values from verified421R1's retained selected source.
        self.assertEqual((len(raw),hashlib.sha256(raw).hexdigest()),(342946,'7115781697e885a31cce8367bb4586419928bfe4656edfee0c51a88cb6118185'))
    def test_main_registration_and_prerequisites_select_the_real_unit_in_a_flat_registry(self):
        paths=verify_coverage.registered_pack_paths(ROOT,'current');self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
        labs=teaching_inventory.catalog();self.assertEqual(len(labs),482)
        unit=read('scenarios/wlan-profile-units-v1.json')['units'][0];self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['ccna-wireless-channel-foundations'])
        for s in read('scenarios/boot-entry-lessons-v25.json')['scenarios'][-3:]:
            self.assertEqual(s['unit'],unit['id']);self.assertEqual(s['id'],s['goal']['definition']['initial']['id']);self.assertEqual(s['initial']['kind'],'WlanProfiles')
    def test_distinct_tasks_teach_new_concepts_and_supply_contextual_guided_actions(self):
        lessons=read('scenarios/boot-entry-lessons-v25.json')['scenarios'][-3:];rows={r['id']:r for r in read('scenarios/presentation/ccna.json')}
        masks=[]
        for s in lessons:
            d=s['goal']['definition'];self.assertNotEqual(d['initial']['network'],d['example']['network']);self.assertNotEqual(d['initial']['id'],d['example']['id'])
            r=rows[s['id']];self.assertEqual(r['source'],'scenarios/boot-entry-lessons-v25.json');self.assertEqual(' '.join(r['requirements']),s['objective']);self.assertGreaterEqual(len(r['concepts']),4);self.assertEqual(len(r['practice']),3)
            self.assertTrue(all(p['action']and p['observe']for p in r['practice']));self.assertIn('Check configuration',r['practice'][1]['action']);self.assertIn('Fill worked configuration',r['practice'][1]['action'])
            self.assertTrue(all(w['explanation']and w['value']for w in r['worked']));self.assertIsNone(s['initial']['simulation']['assignment']['last_check'])
            self.assertEqual(r['status'],'updated');self.assertTrue(all(len(r['review'][v])>60 for v in ['introduction','example','assignment']))
            masks.append(set(d['initial']['policies'][0]['editable']))
        self.assertEqual(masks,[{'enabled','ssid'},{'authentication','management_protection','passphrase'},{'authentication','management_protection'}])
        self.assertIn('wireless LAN (WLAN)',rows[IDS[0]]['concepts'][0]['text']);self.assertIn('pre-shared key (PSK)',' '.join(c['text']for c in rows[IDS[1]]['concepts']));self.assertIn('Simultaneous Authentication of Equals',' '.join(c['text']for c in rows[IDS[2]]['concepts']))
    def test_configuration_compatibility_does_not_claim_crypto_or_real_association(self):
        lessons=read('scenarios/boot-entry-lessons-v25.json')['scenarios'][-3:]
        for s in lessons:
            for term in ['No RF','actual association','encryption','hex keys']:self.assertIn(term,s['simplifications'])
            self.assertTrue(all(c.startswith(('@wlan-set ','@wlan-check'))for c in s['solution']))
            self.assertTrue(all('cisco.com/'in u for u in s['sources']))
        self.assertEqual([len(s['goal']['definition']['initial']['cases'])for s in lessons],[4,4,5])
        self.assertEqual([s['goal']['definition']['initial']['reference'][0]['configuration']['authentication']for s in lessons],['wpa2_psk','wpa2_psk','wpa3_sae'])
    def test_objective_credit_is_per_lesson_narrow_and_not_complete(self):
        with(ROOT/'docs/SKILL_COVERAGE.csv').open()as f:rows={r['reference']:r for r in csv.DictReader(f)if r['blueprint']=='CCNA v1.1'}
        exact={'1.11.b':IDS[:1],'1.11.d':IDS[1:],'2.9':IDS,'5.9':IDS[1:],'5.10':IDS[1:2]}
        for ref,ids in exact.items():
            r=rows[ref];self.assertEqual(r['status'],'Partial practice');self.assertEqual(r['implemented_scenarios'].split(';'),ids)
            self.assertIn('tests/wlan_profile_content.rs',r['evidence_tests']);self.assertIn('No cryptography',r['remaining']);self.assertIn('incomplete',r['remaining'])
        self.assertEqual(rows['2.6']['status'],'Partial practice');self.assertEqual(rows['2.7']['status'],'Partial practice')
        with(ROOT/'docs/CONTINUATION_COVERAGE.csv').open()as f:by_id={r['lab_id']:r for r in csv.DictReader(f)}
        self.assertNotIn('5.10',by_id[IDS[0]]['objective']);self.assertNotIn('5.10',by_id[IDS[2]]['objective'])
        self.assertIn('5.10',by_id[IDS[1]]['objective'])
if __name__=='__main__':unittest.main()
