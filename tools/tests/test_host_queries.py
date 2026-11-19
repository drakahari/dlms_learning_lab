"""Current host-query content checks only, never host utility/network execution."""
import json
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
class HostQueries(unittest.TestCase):
    def test_existing_assignment_and_separate_worked_host_query(self):
        labs,review=teaching.content();self.assertEqual(len(labs),482)
        lab=next(s for s,_ in labs if s['id']=='lpic1-host-resolver');row,_=review[lab['id']]
        self.assertEqual(' '.join(row['requirements']),lab['objective'])
        self.assertEqual(len(row['concepts']),4);self.assertEqual(len(row['practice']),2)
        self.assertIn('without @',row['concepts'][1]['text']);self.assertIn('host -t A',row['concepts'][1]['text'])
        self.assertTrue(any(w['value']=='host -t A inventory.lab.test. 10.73.0.53'for w in row['worked']))
        self.assertTrue(all('203.0.113'not in w['value']for w in row['worked']))
        self.assertIn('https://bind9.readthedocs.io/en/stable/manpages.html',row['prerequisite_sources'])
    def test_capability_is_explicit_positive_a_only_not_an_os_command_bridge(self):
        source=(ROOT/'src/host_direct_dns.rs').read_text()
        for forbidden in ['std::fs','std::process','Command::new','ToSocketAddrs','UdpSocket','TcpStream']:self.assertNotIn(forbidden,source)
        for required in ['Tool::Host','Using domain server:','has address','query type','No NXDOMAIN, NODATA or external lookup is inferred','no utility exit status is inferred']:self.assertIn(required,source)
        scope=(ROOT/'src/host_workspace.rs').read_text().split('pub struct Scope',1)[1].split('impl Scope',1)[0]
        self.assertNotIn('Tool',scope);self.assertNotIn('Observation',scope)
    def test_native_plan_uses_both_actual_utilities_and_keeps_independent_state_checks(self):
        source=(ROOT/'src/ui/smoke/host_lessons/direct_dns.rs').read_text()
        for required in ['dig @10.73.0.53','host -t A inventory.lab.test. 10.73.0.53','host -t A missing.lab.test. 10.73.0.53','host -4 -t A service.lab.test. 203.0.113.53','Op::Solved(false)','Op::Solved(true)','Op::Example(true)','Op::Save']:self.assertIn(required,source)
if __name__=='__main__':unittest.main()
