"""Current source/content checks only; no real getent, DNS or subprocess."""
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
ID='lpic1-client-name-investigation'
def read(path):return json.loads((ROOT/path).read_text())
def pin(value):return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
class ClientDiagnosticLessons(unittest.TestCase):
    def test_all_previous68_tail_values_and145_teaching_values_are_preserved(self):
        old=read('scenarios/boot-entry-lessons-v35.json')['scenarios'];current=read('scenarios/boot-entry-lessons-v36.json')['scenarios']
        self.assertEqual((len(old),len(current)),(68,69));self.assertEqual(current[:68],old)
        self.assertEqual(pin(old),'560a52d0a1586e4707a07f55b38edba1859b186bd90966001ede0e9d17c0f7f2')
        self.assertEqual(pin(read('scenarios/presentation/lpic1.json')[:145]),'caa289917c5536c26a680ade34d63cd6f23e9c93430fd598e9511613cf8b0d61')
        self.assertEqual(len(teaching_inventory.catalog()),482)
        self.assertEqual(verify_coverage.registered_pack_paths(ROOT,'current')[-1],'scenarios/boot-entry-lessons-v53.json')
        unit=read('scenarios/client-diagnostic-units-v1.json')['units'][0]
        self.assertEqual(unit['labs'],[ID]);self.assertEqual(unit['prerequisites'],['linux-resolver-search'])
    def test_separate_examples_and_explicit_readonly_diagnostics_teach_before_assessment(self):
        lab=next(s for s,_ in teaching_inventory.catalog() if s['id']==ID)
        preview=read('tests/fixtures/client-diagnostic-preview-v1.json')['scenarios'][0]
        self.assertEqual(lab['goal'],preview['goal']);self.assertEqual(lab['initial'],preview['initial'])
        self.assertEqual(lab['prerequisites'],['lpic1-dns-absolute-precedence'])
        entry=next(r for r in read('scenarios/presentation/lpic1.json') if r['id']==ID)
        self.assertEqual(' '.join(entry['requirements']),lab['objective'])
        self.assertEqual((len(entry['concepts']),len(entry['practice'])),(5,3))
        text=json.dumps(entry)
        for term in ['getent ahostsv4','canonical','NOTFOUND=return','Run command','Run lookups','systemd-resolved','status 2']:self.assertIn(term,text)
        for view in ['introduction','example','assignment']:self.assertGreater(len(entry['review'][view]),100)
        d=lab['goal']['definition'];self.assertNotEqual(d['initial']['starter'],d['example']['starter'])
        self.assertTrue(all('report' not in w['value'] for w in entry['worked']))
        self.assertEqual(lab['initial']['simulation']['assignment']['diagnostic'],{'input':'','sample':0})
    def test_actual_subset_is_isolated_bounded_and_has_no_saved_output_authority(self):
        model=(ROOT/'src/client_diagnostic.rs').read_text();workspace=(ROOT/'src/client_names_workspace.rs').read_text()
        for forbidden in ['std::process','std::fs','TcpStream','UdpSocket','Command::new','ToSocketAddrs']:self.assertNotIn(forbidden,model+workspace)
        self.assertIn('pub const MAX_COMMAND: usize = 512',model);self.assertIn('pub const MAX_OUTPUT: usize = 2048',model)
        start=workspace.index('pub struct DiagnosticDraft');end=workspace.index('impl Scope',start)
        self.assertNotIn('stdout',workspace[start:end]);self.assertNotIn('resolution',workspace[start:end])
        self.assertIn('self.tested = None',workspace)
    def test_granular_mapping_does_not_invent_remote_utility_or_daemon_coverage(self):
        with (ROOT/'docs/SKILL_COVERAGE.csv').open() as source:
            rows=list(csv.DictReader(source))
        selected={r['reference']:r for r in rows if r['blueprint'].startswith('LPIC-1') and r['parent']=='109.4'}
        self.assertEqual(selected['109.4.T4']['status'],'Partial practice')
        self.assertEqual(selected['109.4.T4']['implemented_scenarios'],'lpic1-host-resolver')
        self.assertIn('positive-A',selected['109.4.T4']['verified_scope'])
        self.assertIn('negative DNS',selected['109.4.T4']['remaining'])
        for ref in ['109.4.K1','109.4.T5']:
            self.assertEqual(selected[ref]['status'],'Partial practice');self.assertEqual(selected[ref]['implemented_scenarios'],'lpic1-host-resolver')
            self.assertIn('positive',selected[ref]['verified_scope'])
            self.assertIn('negative DNS',selected[ref]['remaining'])
        self.assertEqual(selected['109.4.K5']['status'],'Supporting explanation')
        for ref in ['109.4.K2','109.4.K3','109.4.K4','109.4.T1','109.4.T2','109.4.T3','109.4.T6']:
            self.assertEqual(selected[ref]['status'],'Partial practice');self.assertIn(ID,selected[ref]['implemented_scenarios'])
if __name__=='__main__':unittest.main()
