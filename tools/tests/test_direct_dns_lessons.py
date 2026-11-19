"""Current source-only expansion checks, no dig or host DNS execution."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage as coverage
def read(name):return json.loads((ROOT/name).read_text())
class DirectDnsLessons(unittest.TestCase):
    def test_only_readonly_capability_changes_existing_scenario_values(self):
        old=read('scenarios/linux-host-lessons-v1.json')['scenarios'];new=read('scenarios/linux-host-lessons-v2.json')['scenarios']
        raw=json.dumps(old,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual(hashlib.sha256(raw).hexdigest(),'72a6db11847b81f60eaefb3c0e37ec25e7324101e3e67dc8d9544a3db65c25eb')
        self.assertEqual(len(old),4);self.assertEqual(len(new),4)
        stripped=copy.deepcopy(new);d=next(s for s in stripped if s['id']=='lpic1-host-resolver')['goal']['definition']
        self.assertEqual(d.pop('direct_dns'),{'initial':{'servers':['203.0.113.53','203.0.113.54']},'example':{'servers':['10.73.0.53','10.73.0.54']}})
        self.assertEqual(stripped,old)
        known={'scenarios/linux-host-lessons-v2.json':{s['id']:s for s in new}}
        self.assertTrue(coverage.preserved_authored_pack('scenarios/linux-host-lessons-v1.json',{s['id']:s for s in old},known))
        for field in ['objective','version','solution']:
            bad=copy.deepcopy(known);bad['scenarios/linux-host-lessons-v2.json']['lpic1-host-resolver'][field]='tampered'
            with self.assertRaises(ValueError):coverage.preserved_authored_pack('scenarios/linux-host-lessons-v1.json',{s['id']:s for s in old},bad)
    def test_actual_registration_and_three_screen_teaching_preserve_assignment(self):
        labs,review=teaching.content();self.assertEqual(len(labs),482)
        lab=next(s for s,_ in labs if s['id']=='lpic1-host-resolver');entry,path=review[lab['id']]
        self.assertEqual(path,'scenarios/presentation/lpic1_direct_dns.json')
        self.assertEqual(' '.join(entry['requirements']),lab['objective'])
        self.assertTrue(coverage.registered_pack_paths(ROOT,'current').count('scenarios/linux-host-lessons-v2.json')==1)
        for term in ['+short','final dot','A asks for an IPv4 address','getent','Run command','Actual results','Save resolver']:
            self.assertIn(term,json.dumps(entry))
        self.assertTrue(all('203.0.113' not in x['value'] for x in entry['worked']))
        self.assertEqual(len(entry['practice']),2)
        for view in ['introduction','example','assignment']:self.assertGreater(len(entry['review'][view]),80)
    def test_subset_has_no_external_dispatch_and_unknown_facts_stay_unknown(self):
        code=(ROOT/'src/host_direct_dns.rs').read_text()
        for text in ['std::fs','std::process','Command::new','TcpStream','UdpSocket','ToSocketAddrs']:self.assertNotIn(text,code)
        for text in ['MAX_COMMAND: usize = 512','no utility exit status is inferred','No NXDOMAIN, NODATA or external lookup is inferred','unsupported','world.probe(query.server)']:self.assertIn(text,code)
if __name__=='__main__':unittest.main()
