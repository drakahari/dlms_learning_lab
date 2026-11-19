"""Current opt-in authoritative query teaching; never invokes OS DNS tools."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
import verify_coverage
IDS={'lpic2-dns-dualstack','lpic2-dns-aliases','lpic2-dns-secondary'}
def read(p):return json.loads((ROOT/p).read_text())
class AuthoritativeQueries(unittest.TestCase):
    def test_original_six_tasks_and_goals_are_unchanged_except_explicit_optional_capability(self):
        before=read('scenarios/dns-forward-v1.json')['scenarios'];after=read('scenarios/dns-forward-v2.json')['scenarios']
        canonical=json.dumps(before,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual(hashlib.sha256(canonical).hexdigest(),'494747dd4454c8b06d1de2f8b13154ea2e3e6bc605c0ab787ccb88269f8fefe5')
        self.assertEqual(len(before),6)
        for old,current in zip(before,after):
            cleaned=copy.deepcopy(current)
            if old['id']in IDS:
                self.assertIn('lookup',cleaned['dns_teaching']);cleaned['dns_teaching'].pop('lookup')
                self.assertEqual(cleaned['sources'].pop(),'https://bind9.readthedocs.io/en/stable/manpages.html')
            self.assertEqual(cleaned,old)
        current={s['id']:s for s in after}
        self.assertTrue(verify_coverage.preserved_authored_pack('scenarios/dns-forward-v1.json',{s['id']:s for s in before},{'scenarios/dns-forward-v2.json':current}))
        broken=copy.deepcopy(current);broken[next(iter(IDS))]['objective']+=' changed'
        with self.assertRaises(ValueError):verify_coverage.preserved_authored_pack('scenarios/dns-forward-v1.json',{s['id']:s for s in before},{'scenarios/dns-forward-v2.json':broken})
    def test_three_lessons_teach_distinct_diagnostics_before_the_same_independent_repairs(self):
        labs,reviews=teaching.content();self.assertEqual(len(labs),482)
        overlays=read('scenarios/presentation/lpic2_dns_queries.json');self.assertEqual({s['id']for s in overlays},IDS)
        for s,_ in labs:
            if s['id']not in IDS:continue
            row,_=reviews[s['id']];self.assertEqual(' '.join(row['requirements']),s['objective'])
            self.assertEqual(row['source'],'scenarios/dns-forward-v2.json')
            self.assertIn('host -r',json.dumps(row['concepts']));self.assertIn('+norecurse',json.dumps(row['concepts']))
            self.assertTrue(any('Look up'in r['action']or 'Run dig'in r['action']for r in row['practice']))
            self.assertNotIn('workshop.test.',json.dumps(row['worked']))
        text=json.dumps(overlays)
        for word in ['NXDOMAIN','NOERROR','SERVFAIL','Secondary','Save zone','Reload primary'] :self.assertIn(word,text)
    def test_pure_model_actual_controls_and_legacy_progress_are_kept_separate(self):
        source=(ROOT/'src/dns_query_tools.rs').read_text()
        for forbidden in ['std::fs','std::process','Command::new','ToSocketAddrs','UdpSocket','TcpStream']:self.assertNotIn(forbidden,source)
        for required in ['.query(&question.name, question.kind)','MAX_COMMAND: usize = 512','MAX_OUTPUT: usize = 32_768','Served','host','+norecurse']:self.assertIn(required,source)
        ui=(ROOT/'src/ui/dns_workspace/lookup_controls.rs').read_text()
        self.assertIn('if plan.is_none()',ui);self.assertIn('input_valid',ui);self.assertIn('draft.output.clear()',ui)
        plan=(ROOT/'src/ui/smoke/dns_reverse/query_tools.rs').read_text()
        for required in ['Step::BeforeFill','Step::FillOnly','Step::SaveReadback','Step::Rows(false','"NXDOMAIN"','"SERVFAIL"']:self.assertIn(required,plan)
if __name__=='__main__':unittest.main()
