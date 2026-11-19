"""Campus source/teaching preservation; no learner or reference host execution."""
import hashlib
import json
from collections import Counter
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
IDS=['ccna-collapsed-campus-uplinks','ccna-distribution-core-paths']
def read(name):return json.loads((ROOT/name).read_text())
class CampusLessons(unittest.TestCase):
    def test_selected_bounded_pack_adds_exactly_two_authorings_and_keeps_old_objects(self):
        paths=coverage.registered_pack_paths(ROOT,'current')
        self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
        before=read('scenarios/boot-entry-lessons-v40.json')['scenarios']
        added=read('scenarios/campus-architecture-lessons-v1.json')['scenarios']
        self.assertEqual(len(before),79);self.assertEqual([s['id']for s in added],IDS)
        self.assertEqual(read(paths[-1])['scenarios'],before+added+read('scenarios/leaf-spine-lessons-v1.json')['scenarios']+read('scenarios/samba-standalone-lessons-v1.json')['scenarios']+read('scenarios/virtual-isolation-lessons-v1.json')['scenarios']+read('scenarios/guest-driver-lessons-v1.json')['scenarios']+read('scenarios/smtp-routing-lessons-v1.json')['scenarios']+read('scenarios/sieve-filter-lessons-v1.json')['scenarios']+read('scenarios/sieve-address-lessons-v1.json')['scenarios']+read('scenarios/sieve-delivery-lessons-v1.json')['scenarios']+read('scenarios/sieve-vacation-lessons-v1.json')['scenarios']+read('scenarios/procmail-migration-lessons-v1.json')['scenarios']+read('scenarios/mailbox-access-lessons-v1.json')['scenarios']+read('scenarios/mailbox-administration-lessons-v1.json')['scenarios'])
        self.assertEqual(read('scenarios/authoring/campus-architecture-v1.json')['scenarios'],added)
        self.assertLess((ROOT/paths[-1]).stat().st_size,2_000_000)
        labs=teaching.catalog();self.assertEqual(len(labs),482)
        self.assertEqual(Counter(s['track']for s,_ in labs),{'CCNA':113,'LPIC-1':154,'LPIC-2':135,'Bash/Shell':80})
    def test_all_107_previous_ccna_teaching_values_and_other_tracks_are_retained(self):
        rows=read('scenarios/presentation/ccna.json');self.assertEqual(len(rows),113)
        self.assertEqual([r['id']for r in rows[107:109]],IDS)
        raw=json.dumps(rows[:107],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        # Actual retained pre-append values from verified452 main source, not an independent seal.
        self.assertEqual((len(raw),hashlib.sha256(raw).hexdigest()),(401812,'54eaedc5f8f2c98a494cb73c28bfa858b688cdb7a42d5c02a586e76846b77986'))
        unit=read('scenarios/campus-architecture-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'],IDS);self.assertEqual(unit['prerequisites'],['ccna-wired-device-paths'])
    def test_every_screen_has_specific_actions_without_populating_assignment_answers(self):
        rows={r['id']:r for r in read('scenarios/presentation/ccna.json')}
        for s in read('scenarios/campus-architecture-lessons-v1.json')['scenarios']:
            row=rows[s['id']];self.assertEqual(row['status'],'updated')
            self.assertEqual(''.join(' '.join(row['requirements']).split()),''.join(s['objective'].split()))
            self.assertEqual(len(row['concepts']),3);self.assertEqual(len(row['practice']),3)
            for view in ['introduction','example','assignment']:self.assertGreater(len(row['review'][view]),40)
            for step in row['practice']:self.assertIn('action',step);self.assertTrue(step['observe'])
            d=s['goal']['definition'];initial=s['initial']['simulation']
            self.assertNotEqual(d['assignment']['facts'],d['example']['facts'])
            for scope,key in [('assignment','assignment'),('practice','example')]:
                self.assertEqual(initial[scope]['draft'],d[key]['initial']);self.assertFalse(initial[scope]['checked']);self.assertNotEqual(initial[scope]['draft'],d[key]['reference'])
            self.assertGreaterEqual(len(d['assignment']['cases']),3)
    def test_model_and_workspace_have_no_host_execution_bridge_and_keep_bounded_native_controls(self):
        for name in ['src/network_architecture.rs','src/network_architecture_source.rs','src/network_architecture_workspace.rs','src/ui/network_architecture_page.rs']:
            source=(ROOT/name).read_text()
            for forbidden in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','ToSocketAddrs','unsafe {']:self.assertNotIn(forbidden,source)
        native=(ROOT/'src/ui/smoke/network_architectures.rs').read_text()
        for required in ['Op::Uncredited','Op::Fresh','progress::load','Duration::from_secs(15)','f.polls < 1200','smoke_control_visible']:self.assertIn(required,native)
if __name__=='__main__':unittest.main()
