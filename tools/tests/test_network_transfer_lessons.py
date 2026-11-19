"""Current transfer composition/source guards; no client, GUI or shell is run."""
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
IDS=['ccna-tftp-recovery-copy','ccna-ftp-archive-channel','ccna-transfer-file-handover']
def read(name):return json.loads((ROOT/name).read_text())
class NetworkTransferLessons(unittest.TestCase):
    def test_current_registry_adds_only_three_authored_tasks_without_raising_capacity(self):
        paths=coverage.registered_pack_paths(ROOT,'current')
        self.assertEqual((len(paths),paths[-1]),(128,'scenarios/boot-entry-lessons-v53.json'))
        before=read('scenarios/boot-entry-lessons-v39.json')['scenarios']
        added=read('scenarios/network-transfers-lessons-v1.json')['scenarios']
        current=read('scenarios/boot-entry-lessons-v40.json')['scenarios']
        self.assertEqual(read(paths[-1])['scenarios'],current+read('scenarios/campus-architecture-lessons-v1.json')['scenarios']+read('scenarios/leaf-spine-lessons-v1.json')['scenarios']+read('scenarios/samba-standalone-lessons-v1.json')['scenarios']+read('scenarios/virtual-isolation-lessons-v1.json')['scenarios']+read('scenarios/guest-driver-lessons-v1.json')['scenarios']+read('scenarios/smtp-routing-lessons-v1.json')['scenarios']+read('scenarios/sieve-filter-lessons-v1.json')['scenarios']+read('scenarios/sieve-address-lessons-v1.json')['scenarios']+read('scenarios/sieve-delivery-lessons-v1.json')['scenarios']+read('scenarios/sieve-vacation-lessons-v1.json')['scenarios']+read('scenarios/procmail-migration-lessons-v1.json')['scenarios']+read('scenarios/mailbox-access-lessons-v1.json')['scenarios']+read('scenarios/mailbox-administration-lessons-v1.json')['scenarios'])
        self.assertEqual(len(before),76)
        self.assertEqual([s['id']for s in added],IDS)
        self.assertEqual(current,before+added)
        labs=teaching.catalog()
        self.assertEqual(len(labs),482)
        self.assertEqual(Counter(s['track']for s,_ in labs),{'CCNA':113,'LPIC-1':154,'LPIC-2':135,'Bash/Shell':80})
        unit=read('scenarios/network-transfers-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'],IDS)
        self.assertEqual(unit['prerequisites'],['ccna-transport-conversations'])
    def test_all104_preceding_ccna_presentation_values_remain_exact(self):
        rows=read('scenarios/presentation/ccna.json')
        self.assertEqual(len(rows),113)
        self.assertEqual([r['id']for r in rows[104:107]],IDS)
        raw=json.dumps(rows[:104],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        # Actual canonical values from verified449's retained selected source;
        # not an independent seal or a requirement for private files at test time.
        self.assertEqual((len(raw),hashlib.sha256(raw).hexdigest()),(387598,'cad1b98f06b5f455b250be472275e46ef151c8744d169bc557bf435f1f98285d'))
    def test_each_lesson_has_its_own_reviewed_teaching_and_separate_source_bound_tasks(self):
        authored=read('scenarios/authoring/network-transfers-v1.json')
        self.assertEqual(authored['schema_version'],1)
        labs=read('scenarios/network-transfers-lessons-v1.json')['scenarios']
        rows={r['id']:r for r in read('scenarios/presentation/ccna.json')}
        for a,s in zip(authored['lessons'],labs,strict=True):
            self.assertEqual(a['definition'],s['goal']['definition'])
            self.assertEqual(a['presentation'],rows[s['id']])
            row=rows[s['id']]
            self.assertEqual(row['status'],'updated')
            self.assertEqual(''.join(' '.join(row['requirements']).split()),''.join(s['objective'].split()))
            self.assertEqual(len(row['practice']),3)
            for view in ['introduction','example','assignment']:self.assertGreater(len(row['review'][view]),40)
            for step in row['practice']:self.assertTrue(step['action']);self.assertTrue(step['observe'])
            d=s['goal']['definition']
            self.assertNotEqual(d['assignment'],d['example'])
            self.assertEqual(len(d['assignment']['cases']),3)
            self.assertFalse(s['initial']['simulation']['assignment']['checked'])
            self.assertFalse(s['initial']['simulation']['practice']['checked'])
            self.assertEqual(s['initial']['simulation']['assignment']['history'],[])
            self.assertEqual(s['initial']['simulation']['practice']['history'],[])
    def test_new_transfer_core_and_native_workspace_have_no_host_client_bridge(self):
        for name in ['src/network_transfer.rs','src/network_transfer_workspace.rs','src/network_transfer_source.rs','src/ui/network_transfer_page.rs']:
            source=(ROOT/name).read_text()
            for forbidden in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','ToSocketAddrs','unsafe {']:
                self.assertNotIn(forbidden,source)
        native=(ROOT/'src/ui/smoke/network_transfers.rs').read_text()
        for required in ['Op::Actual(false)','Op::Check(false, false)','Op::Uncredited','Op::Fresh','progress::load','Duration::from_secs(15)','f.polls < 1200','smoke_control_visible']:
            self.assertIn(required,native)
if __name__=='__main__':unittest.main()
