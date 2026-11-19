"""Current SSH teaching/bundling boundaries; never launch a host SSH utility."""
import hashlib
import json
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
IDS=['lpic1-ssh-server-trust','lpic2-ssh-key-authorization','lpic2-ssh-policy-handover']
def read(name):return json.loads((ROOT/name).read_text())
class SshAccess(unittest.TestCase):
    def test_three_authored_tasks_have_specific_teaching_separate_examples_and_unchanged_requirements(self):
        labs,review=teaching.content()
        self.assertEqual(len(labs),482)
        authored=read('scenarios/ssh-access-lessons-v1.json')['scenarios']
        self.assertEqual([s['id']for s in authored],IDS)
        for s in authored:
            row,_=review[s['id']]
            self.assertEqual(' '.join(row['requirements']),s['objective'])
            self.assertEqual(len(row['concepts']),4)
            self.assertEqual(len(row['practice']),3)
            d=s['goal']['definition']
            self.assertNotEqual(d['initial']['scope']['host'],d['example']['scope']['host'])
            self.assertEqual(s['initial']['simulation']['assignment']['world'],d['initial']['world'])
            self.assertFalse(s['initial']['simulation']['assignment']['checked'])
            for item in row['practice']:
                self.assertTrue(item['action']);self.assertTrue(item['observe'])
            self.assertIn('Save',json.dumps(row['practice']))
            self.assertNotIn('BEGIN OPENSSH PRIVATE',json.dumps(s))
    def test_consolidation_preserves_all69_preceding_values_and_catalog_limits(self):
        old=read('scenarios/boot-entry-lessons-v36.json')['scenarios']
        new=read('scenarios/boot-entry-lessons-v37.json')['scenarios']
        self.assertEqual(len(old),69);self.assertEqual(new[:69],old)
        self.assertEqual(new[69:],read('scenarios/ssh-access-lessons-v1.json')['scenarios'])
        registered=coverage.registered_pack_paths(ROOT,'current')
        self.assertEqual(len(registered),128)
        self.assertIn('scenarios/boot-entry-lessons-v53.json',registered)
        source=(ROOT/'src/pack.rs').read_text()
        self.assertIn('MAX_BUNDLED_PACK_FILES: usize = 128',source)
        for name in ['scenarios/boot-entry-lessons-v36.json','scenarios/ssh-access-lessons-v1.json']:
            before={s['id']:s for s in read(name)['scenarios']}
            current={'scenarios/boot-entry-lessons-v37.json':{s['id']:s for s in new}}
            self.assertTrue(coverage.preserved_authored_pack(name,before,current))
            broken=json.loads(json.dumps(before));broken[next(iter(broken))]['title']+=' changed'
            with self.assertRaises(ValueError):coverage.preserved_authored_pack(name,broken,current)
    def test_core_has_no_host_execution_or_network_and_native_route_uses_actual_controls(self):
        for path in ['src/ssh_identity.rs','src/ssh_policy.rs','src/ssh_key_access.rs','src/ssh_access.rs','src/ssh_workspace.rs']:
            source=(ROOT/path).read_text()
            for bad in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','ToSocketAddrs']:
                self.assertNotIn(bad,source)
        native=(ROOT/'src/ui/smoke/ssh_access.rs').read_text()
        for required in ['Op::Check(true, true)','Op::Check(false, false)','Op::Fresh','Op::Inert','Event::ApplyPolicy','click_visible_smoke_control','progress::load','Op::Style(1.5, true)','Op::Style(2., false)']:
            self.assertIn(required,native)
        page=(ROOT/'src/ui/ssh_page.rs').read_text()
        for required in ['Inspect active policy','Save permissions','Apply saved policy','All objective results','Check access','Check example']:
            self.assertIn(required,page)
