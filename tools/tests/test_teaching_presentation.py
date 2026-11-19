"""Focused inventory and review-resource checks; never execute learner work."""
import re
import ast
import hashlib
import json
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import build_candidate
import teaching_inventory

class TeachingPresentation(unittest.TestCase):
    def test_poe_teaches_real_controls_and_keeps_worked_plans_separate(self):
        path=ROOT/'scenarios/presentation/ccna_poe.json'
        self.assertLessEqual(path.stat().st_size,20000)
        rows=json.loads(path.read_text())
        labs=json.loads((ROOT/'scenarios/poe-lessons-v1.json').read_text())['scenarios']
        self.assertEqual([r['id'] for r in rows],[s['id'] for s in labs])
        self.assertEqual({r['id'] for r in rows},{'ccna-poe-compatible-ports','ccna-poe-budget-recovery'})
        for row,lab in zip(rows,labs,strict=True):
            self.assertEqual(' '.join(row['requirements']),lab['objective'])
            self.assertEqual(row['source'],'scenarios/poe-lessons-v1.json')
            self.assertGreaterEqual(len(row['concepts']),5)
            self.assertEqual(len(row['practice']),3)
            self.assertGreaterEqual(len(row['worked']),5)
            concepts=' '.join(c['text']for c in row['concepts'])
            for phrase in ('source','class','reservation','consumption','listed order','data-only'):
                self.assertIn(phrase,concepts)
            actions=' '.join(p['action']for p in row['practice'])
            for phrase in ('Connect to','Disconnected','PoE enabled','Check example'):
                self.assertIn(phrase,actions)
            d=lab['goal']['definition']
            self.assertNotEqual(d['assignment']['facts'],d['example']['facts'])
            self.assertNotEqual(d['assignment']['id'],d['example']['id'])
            self.assertNotEqual(d['assignment']['facts']['devices'],d['example']['facts']['devices'])
            self.assertFalse(lab['initial']['simulation']['assignment']['checked'])
            self.assertTrue(all(not g['commands']for g in lab['guide']))
            # No internal authoring helper is taught as a real equipment command.
            self.assertNotIn('poe-workspace ',json.dumps(row))

    def test_continuation_aggregate_preserves_the_previous_security_detection_records(self):
        old=json.loads((ROOT/'scenarios/security-detection-lessons-v2.json').read_text())['scenarios']
        current=json.loads((ROOT/'scenarios/curriculum-continuation-v1.json').read_text())['scenarios']
        self.assertEqual(current[:len(old)],old)
        self.assertEqual(len(current),len(old)+2)
        self.assertLessEqual((ROOT/'scenarios/curriculum-continuation-v1.json').stat().st_size,2000000)

    def test_port_probes_expand_only_two_existing_lessons_without_changing_independent_work(self):
        original=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        raw=json.dumps(original,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual(hashlib.sha256(raw).hexdigest(),'34146f695bd7e4a74259d90b2f5e09261241851957e503671159d92702d5f4a9')
        rows=json.loads((ROOT/'scenarios/presentation/lpic2_port_probes.json').read_text())
        allowed={'lpic2-exposure-client-policy','lpic2-exposure-listener-retirement'}
        self.assertEqual(len(rows),2)
        self.assertEqual({r['id']for r in rows},allowed)
        originals={r['id']:r for r in original}
        labs={r['id']:r for r in json.loads((ROOT/'scenarios/service-exposure-lessons-v1.json').read_text())['scenarios']}
        _,reviews=teaching_inventory.content()
        current={r['id']:r for r in json.loads((ROOT/'scenarios/presentation/lpic2_telnet_transport.json').read_text())}
        for row in rows:
            lab=labs[row['id']]
            self.assertEqual(row['requirements'],originals[row['id']]['requirements'])
            self.assertEqual(' '.join(row['requirements']),lab['objective'])
            self.assertEqual(row['goal'],originals[row['id']]['goal'])
            self.assertEqual((len(row['concepts']),len(row['practice'])),(5,3))
            concepts=' '.join(c['text']for c in row['concepts'])
            for text in ('-n uses','-v shows','-z tests','status 0','status 1','no port observation'):
                self.assertIn(text,concepts)
            practice=' '.join(p['action']for p in row['practice'])
            for text in ('Server terminal','Observer terminal','Run command','Observe from','Check example','nc -n -v -z 192.0.2.10'):
                self.assertIn(text,practice)
            self.assertEqual(lab['goal']['definition']['example']['initial']['server'],'192.0.2.10')
            self.assertNotEqual(lab['goal']['definition']['assignment']['initial']['server'],'192.0.2.10')
            # The retained nc expansion is still tested above; the newer closed
            # overlay is now the active renderer and must match in its entirety.
            self.assertEqual(current[row['id']]['requirements'],row['requirements'])
            self.assertEqual(reviews[row['id']][0],current[row['id']])
            self.assertEqual(reviews[row['id']][1],'scenarios/presentation/lpic2_telnet_transport.json')

    def test_telnet_transport_teaches_client_modes_without_changing_independent_requirements(self):
        base={r['id']:r for r in json.loads((ROOT/'scenarios/presentation/lpic2_port_probes.json').read_text())}
        path=ROOT/'scenarios/presentation/lpic2_telnet_transport.json'
        self.assertLessEqual(path.stat().st_size,20000)
        rows=json.loads(path.read_text())
        self.assertEqual(len(rows),2)
        self.assertEqual({r['id']for r in rows},set(base))
        for row in rows:
            self.assertEqual(row['requirements'],base[row['id']]['requirements'])
            self.assertEqual(row['goal'],base[row['id']]['goal'])
            self.assertEqual((len(row['concepts']),len(row['practice']),len(row['worked'])),(6,3,9))
            text=' '.join(c['text']for c in row['concepts'])
            for phrase in ('no exit status yet','port 23','personal files','quit'):
                self.assertIn(phrase,text)
            worked=' '.join(r['value']+' '+r['explanation']for r in row['worked'])
            for phrase in ('telnet -c 192.0.2.10 443','Escape to local prompt','Ctrl+]','trace-file option'):
                self.assertIn(phrase,worked)
            actions=' '.join(r['action']for r in row['practice'])
            for phrase in ('Observer terminal','telnet -c 192.0.2.10 443','Escape to local prompt','quit','Run command','Check example'):
                self.assertIn(phrase,actions)
            self.assertNotIn('telnet', ' '.join(row['requirements']).lower())

    def test_ids_rules_preserve158_values_and_teach_editor_replay_and_independent_requirements(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        before=json.dumps(rows[:158],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual(hashlib.sha256(before).hexdigest(),'198c8517fbc7e03c8d5c27d322f6aaf232ebb63a2046f9fb9c444d3591a46c6a')
        self.assertEqual(len(rows),160)
        labs=json.loads((ROOT/'scenarios/ids-rule-lessons-v1.json').read_text())['scenarios']
        self.assertEqual([r['id']for r in rows[158:]],[s['id']for s in labs])
        for row,lab in zip(rows[158:],labs,strict=True):
            self.assertEqual(row['source'],'scenarios/ids-rule-lessons-v1.json')
            self.assertEqual(' '.join(row['requirements']),lab['objective'])
            self.assertEqual(len(row['requirements']),4)
            self.assertGreaterEqual(len(row['concepts']),4)
            self.assertEqual(len(row['practice']),3)
            self.assertEqual(len(row['worked']),3)
            for item in row['concepts']+row['practice']:
                self.assertIsNone(re.match(r'^\d+\. ',item['title']))
            actions=' '.join(p['action']for p in row['practice'])
            for label in ('Save virtual rule','Replay record set','Record set','Check example'):
                self.assertIn(label,actions)
            d=lab['goal']['definition']
            self.assertNotEqual(d['assignment']['rule_path'],d['example']['rule_path'])
            self.assertNotEqual(d['assignment']['initial_rule'],d['example']['initial_rule'])
            self.assertFalse(lab['initial']['simulation']['assignment']['checked'])
            self.assertEqual(lab['initial']['simulation']['assignment']['history'],[])
            self.assertTrue(all(not g['commands']for g in lab['guide']))
            self.assertTrue(any('Snort itself permits omission' in c['text']for c in row['concepts']))

    def test_auth_filters_preserve156_values_and_teach_actual_separate_workspaces(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:156],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual(hashlib.sha256(old).hexdigest(),'4680e71ab4a0ddae4a081e6b8d05eb990e303c7c13af59d23318f8d0ceef0c77')
        self.assertEqual(len(rows),160)
        labs=json.loads((ROOT/'scenarios/auth-filter-lessons-v1.json').read_text())['scenarios']
        self.assertEqual([r['id']for r in rows[156:158]],[s['id']for s in labs])
        for row,lab in zip(rows[156:158],labs,strict=True):
            self.assertEqual(row['status'],'updated')
            self.assertEqual(row['source'],'scenarios/auth-filter-lessons-v1.json')
            self.assertEqual(' '.join(row['requirements']),lab['objective'])
            self.assertEqual((len(row['concepts']),len(row['practice'])),(4,3))
            self.assertGreaterEqual(len(row['worked']),4)
            for item in row['concepts']+row['practice']:
                self.assertIsNone(re.match(r'^\d+\. ',item['title']))
            actions=' '.join(p['action']for p in row['practice'])
            for label in ['Save virtual filter','Run command','Log sample','Check example']:self.assertIn(label,actions)
            d=lab['goal']['definition']
            self.assertNotEqual(d['assignment']['filter_path'],d['example']['filter_path'])
            self.assertNotEqual(d['assignment']['log_path'],d['example']['log_path'])
            self.assertFalse(lab['initial']['simulation']['assignment']['checked'])
            self.assertEqual(lab['initial']['simulation']['assignment']['history'],[])
            for view in ['introduction','example','assignment']:self.assertGreater(len(row['review'][view]),100)

    def test_patch_preserves154_verified_values_and_teaches_actual_rpm_workspaces(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        # Independently read back from the retained verified501 candidate inputs.
        before=json.dumps(rows[:154],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),
            (759221,'b185f3b103114ddea36077e2b6a614bfc52c1f655429b54940aefea1d24f7d6e'))
        self.assertEqual(len(rows),160)
        labs=json.loads((ROOT/'scenarios/advisory-patch-lessons-v1.json').read_text())['scenarios']
        self.assertEqual([r['id']for r in rows[154:156]],[s['id']for s in labs])
        for row,lab in zip(rows[154:156],labs,strict=True):
            self.assertEqual(row['status'],'updated')
            self.assertEqual(row['source'],'scenarios/advisory-patch-lessons-v1.json')
            self.assertEqual(' '.join(row['requirements']),lab['objective'])
            self.assertGreaterEqual(len(row['concepts']),5)
            self.assertEqual(len(row['practice']),3)
            self.assertGreaterEqual(len(row['worked']),4)
            self.assertTrue(all(len(row['review'][v])>100 for v in ('introduction','example','assignment')))
            actions=' '.join(p['action']for p in row['practice'])
            for label in ('Run command','Check example','Inspect package data'):self.assertIn(label,actions)
            d=lab['goal']['definition'];example=lab['rpm_packages_teaching']['definition']
            self.assertTrue(set(d['archive_paths']).isdisjoint(example['archive_paths']))
            self.assertTrue(set(d['initial']['catalog']['archives']).isdisjoint(example['initial']['catalog']['archives']))
            self.assertTrue(d['patch_policy']['advisories'])
            self.assertNotIn('allow_metadata_restore',d)

    def test_vpn_preserves_all152_values_and_has_specific_three_screen_teaching(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        before=json.dumps(rows[:152],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),(748942,'52875c708d8cc6d9d2f48813e88e38ab9b68acb84b42fb0a664e9856e9ea3e29'))
        labs=json.loads((ROOT/'scenarios/vpn-point-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(len(rows),160)
        self.assertEqual([r['id']for r in rows[152:154]],[s['id']for s in labs])
        for row,lab in zip(rows[152:154],labs):
            self.assertEqual(row['status'],'updated')
            self.assertEqual(row['source'],'scenarios/vpn-point-lessons-v1.json')
            self.assertEqual((len(row['concepts']),len(row['practice'])),(3,3))
            self.assertGreaterEqual(len(row['worked']),3)
            self.assertEqual(' '.join(row['requirements']),lab['objective'])
            for view in ('introduction','example','assignment'):self.assertGreater(len(row['review'][view]),100)
            actions=' '.join(p['action']for p in row['practice'])
            for label in ('Save profile','Apply profiles','Test path','Check example'):self.assertIn(label,actions)
            d=lab['goal']['definition']
            self.assertNotEqual(d['assignment']['initial']['supplied']['peers'],d['example']['initial']['supplied']['peers'])
            self.assertFalse(lab['initial']['simulation']['assignment']['checked'])
            self.assertEqual(lab['initial']['simulation']['assignment']['history'],[])

    def test_service_exposure_preserves150_values_and_teaches_each_real_workspace(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:150],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(738426,'3d7beb77132de6dd82688d617fe8295e535874c2b44aa7c63c35f4b8837702e8'))
        labs=json.loads((ROOT/'scenarios/service-exposure-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(len(rows),160)
        self.assertEqual([r['id']for r in rows[150:152]],[s['id']for s in labs])
        for row,lab in zip(rows[150:152],labs):
            self.assertEqual(row['status'],'updated')
            self.assertEqual((len(row['concepts']),len(row['practice'])),(4,3))
            self.assertGreaterEqual(len(row['worked']),4)
            self.assertEqual(' '.join(row['requirements']),lab['objective'])
            self.assertLess(len(row['goal']),160)
            self.assertNotEqual(row['goal'],lab['objective'])
            for view in ('introduction','example','assignment'):self.assertGreater(len(row['review'][view]),100)
            self.assertIn('Server terminal',' '.join(p['action']for p in row['practice']))
            self.assertIn('Observer terminal',' '.join(p['action']for p in row['practice']))
            d=lab['goal']['definition']
            self.assertNotEqual(d['assignment']['initial']['server'],d['example']['initial']['server'])
            self.assertEqual(lab['initial']['simulation']['assignment']['history'],[])
            self.assertFalse(lab['initial']['simulation']['assignment']['checked'])
        self.assertIn('whole Admin range',rows[150]['requirements'][0])
        self.assertIn('firewall-only',rows[151]['requirements'][0])

    def test_ftp_unit_preserves147_prior_values_and_reviews_each_three_screen_lesson(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:147],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(721848,'1e90010927f653b283da6076a0214198f3a9600b8978b1f9875608c1e5d9ea79'))
        self.assertEqual(len(rows),160)
        labs=json.loads((ROOT/'scenarios/ftp-identity-lessons-v1.json').read_text())['scenarios']
        self.assertEqual([r['id']for r in rows[147:150]],['lpic2-ftp-local-jail','lpic2-ftp-upload-rights','lpic2-ftp-chroot-handover'])
        for row,lab in zip(rows[147:150],labs):
            self.assertEqual(row['status'],'updated')
            self.assertEqual((len(row['concepts']),len(row['practice'])),(4,3))
            self.assertGreaterEqual(len(row['worked']),4)
            self.assertEqual(' '.join(row['requirements']),lab['objective'])
            for view in ('introduction','example','assignment'):self.assertGreater(len(row['review'][view]),100)
            d=lab['goal']['definition']
            self.assertNotEqual(d['assignment']['initial']['server'],d['example']['initial']['server'])
            self.assertNotEqual(d['assignment']['initial']['accounts'][0]['user'],d['example']['initial']['accounts'][0]['user'])
            self.assertFalse(lab['initial']['simulation']['assignment']['checked'])
            self.assertEqual(lab['initial']['simulation']['assignment']['history'],[])
            self.assertIn('Server terminal',' '.join(p['action']for p in row['practice']))
        self.assertIn('only exact names',rows[147]['concepts'][1]['text'])
        self.assertIn('local_umask',rows[148]['concepts'][2]['text'])
        self.assertIn('NOT confined',rows[149]['concepts'][0]['text'])

    def test_ipv6_administration_preserves_all143_verified_values_and_complete_three_screen_content(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:143],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        # Exact canonical values from retained verified490; no private fixture required.
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(699582, '1f00d0766929e45b4ee8f5582b7db91415e3031ac7ca24de1773e6d360b30b9f'))
        ids=['lpic2-router-ipv6-scope-recovery','lpic2-router-ipv6-ra-recovery','lpic2-router-ipv6-saved-settings']
        self.assertEqual([r['id']for r in rows[143:146]],ids)
        for r in rows[143:146]:
            self.assertEqual(r['status'],'updated')
            self.assertEqual((len(r['concepts']),len(r['practice']),len(r['requirements'])),(4,3,3))
            self.assertTrue(r['worked'])
            self.assertTrue(all(len(r['review'][v])>100 for v in ('introduction','example','assignment')))
            self.assertNotEqual(r['goal'],r['id'])
        self.assertIn('link-local',' '.join(c['text']for c in rows[143]['concepts']))
        self.assertIn('does not create a received message',rows[144]['worked'][1]['explanation'])
        self.assertIn('not a complete Linux reboot',' '.join(c['text']for c in rows[145]['concepts']))

    def test_ipv6_startup_retains146_prior_values_and_has_distinct_taught_practice(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:146],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(714915, '58ee27a290db18a3d6ef5cd0678d8fcb91e102cd6281ddad2e9985f0ff51c1b5'))
        self.assertEqual(len(rows),160)
        r=rows[146]
        self.assertEqual(r['id'],'lpic2-ipv6-startup-policy-recovery')
        self.assertEqual((len(r['concepts']),len(r['practice']),len(r['requirements'])),(5,4,5))
        self.assertGreaterEqual(len(r['worked']),4)
        actions=' '.join(p['action']for p in r['practice'])
        for label in ('Run traffic test','Save virtual file','Run startup test','Check example'):
            self.assertIn(label,actions)
        lab=json.loads((ROOT/r['source']).read_text())['scenarios'][0]
        d=lab['goal']['definition']
        self.assertNotEqual(d['assignment']['initial']['router']['host']['profile']['ipv6'],d['example']['initial']['router']['host']['profile']['ipv6'])
        self.assertEqual(lab['initial']['simulation']['assignment']['editor'],d['assignment']['initial']['saved_filter'])
        self.assertFalse(lab['initial']['simulation']['assignment']['checked'])
        self.assertEqual(' '.join(r['requirements']),lab['objective'])

    def test_inventory_follows_selected_registry_not_unselected_historical_includes(self):
        with tempfile.TemporaryDirectory() as folder:
            root=Path(folder);(root/'src').mkdir();(root/'scenarios').mkdir()
            source='''fn bundled_texts() -> Vec<&'static str> { vec![include_str!("../scenarios/first.json"),] }
pub fn bundled() -> Result<Vec<Scenario>,String> { load_catalog(&bundled_texts()) }
fn accepted_texts() -> Vec<&'static str> { let mut texts=bundled_texts();texts.push(include_str!("../scenarios/second.json"));texts }
pub fn ccna_pilot() -> Result<Vec<Scenario>,String> { load_bundled_catalog(&accepted_texts()) }
fn accepted999_texts() -> Vec<&'static str> { vec![include_str!("../scenarios/first.json"),include_str!("../scenarios/ignored.json"),] }
'''
            (root/'src/pack.rs').write_text(source)
            for name,identity in [('first','one'),('second','two'),('ignored','not-registered')]:
                (root/f'scenarios/{name}.json').write_text(json.dumps({'scenarios':[{'id':identity}]}))
            with patch.object(teaching_inventory,'ROOT',root):
                self.assertEqual([(s['id'],p)for s,p in teaching_inventory.catalog()],
                                 [('one','scenarios/first.json'),('two','scenarios/second.json')])
                (root/'scenarios/second.json').write_text(json.dumps({'scenarios':[{'id':'one'}]}))
                with self.assertRaisesRegex(ValueError,'Duplicate registered lab ID'):
                    teaching_inventory.catalog()
                (root/'src/pack.rs').write_text(source.replace('texts.push(include_str!("../scenarios/second.json"))','texts.push(include_str!("../scenarios/first.json"))'))
                with self.assertRaises(ValueError):
                    teaching_inventory.catalog()

    def test_cow_unit_preserves_all_61_prior_lpic2_presentation_values(self):
        rows = json.loads((ROOT / 'scenarios/presentation/lpic2.json').read_text())
        self.assertEqual(len(rows), 160)
        # Canonical value digest from the immutable verified326 candidate;
        # presentation indentation changes do not hide a changed old lesson.
        before = json.dumps(rows[:61], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        self.assertEqual((len(before), hashlib.sha256(before).hexdigest()),
                         (274185, '409fe07d99265ad75d1a3a1867847e05af5f5797da1d9672d9f838287e089e4e'))
        self.assertEqual([r['id'] for r in rows[61:64]],
                         ['lpic2-lvm-saved-view', 'lpic2-lvm-cow-budget', 'lpic2-lvm-record-recovery'])
        for row in rows[61:64]:
            self.assertGreaterEqual(len(row['requirements']), 4)
            self.assertEqual(len(row['practice']), 3)
            self.assertTrue(any('Record' in step['action'] for step in row['practice']))

    def test_ipv6_unit_preserves_all140_preceding_values_and_teaches_separate_work(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        # Read from the retained, source-hashed pre-unit feedback snapshot.
        before=json.dumps(rows[:140],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),
                         (682063,'79535cbf21dd5f624c00c6f56a55d7ed67c59539d07fe636e5bd0adbd0ec9d01'))
        self.assertEqual([r['id']for r in rows[140:142]],
                         ['lpic2-ipv6-control-header-policy','lpic2-ipv6-tracked-service-policy'])
        scenarios=json.loads((ROOT/'scenarios/linux-v6-lessons-v1.json').read_text())['scenarios']
        for row,source in zip(rows[140:142],scenarios):
            self.assertEqual(row['status'],'updated')
            self.assertEqual(len(row['practice']),3)
            self.assertEqual(len(row['concepts']),4)
            self.assertTrue(all(step['action']and step['observe']for step in row['practice']))
            self.assertGreaterEqual(len(row['requirements']),5)
            self.assertNotIn('→',json.dumps(row,ensure_ascii=False))
            self.assertEqual(source['initial']['simulation']['assignment']['input'],'')
            self.assertFalse(source['initial']['simulation']['assignment']['checked'])
            definition=source['goal']['definition']
            self.assertNotEqual(definition['example']['facts'],definition['assignment']['facts'])
            self.assertNotEqual(definition['example']['cases'],definition['assignment']['cases'])
            self.assertNotIn('independent novice acceptance passed',str(row['review']).lower())

    def test_services_router_capstone_keeps_all142_prior_rows_and_teaches_saved_vs_installed(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        before=json.dumps(rows[:142],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual(hashlib.sha256(before).hexdigest(),'aea468aecbb182bd8bb3446db934737d7af65216b3db45d3dcce64e604e8a4af')
        row=rows[142];self.assertEqual(row['id'],'lpic2-router-service-name-recovery')
        self.assertEqual(row['status'],'updated');self.assertEqual(len(row['practice']),3)
        self.assertEqual(len(row['concepts']),4);self.assertTrue(all(c['text'] for c in row['concepts']))
        self.assertIn('Save virtual file',row['practice'][0]['action']);self.assertIn('ip6tables -S',row['practice'][1]['action'])
        self.assertIn('old',row['practice'][2]['action']);self.assertNotIn('independent novice acceptance passed',str(row['review']).lower())
        source=json.loads((ROOT/'scenarios/linux-services-router-lessons-v1.json').read_text())['scenarios'][0]
        self.assertEqual(' '.join(row['requirements']),source['objective'])
        d=source['goal']['definition'];self.assertNotEqual(d['example']['required_services'],d['assignment']['required_services'])
        self.assertNotEqual(d['example']['router']['facts'],d['assignment']['router']['facts'])
        self.assertEqual(source['initial']['simulation']['assignment']['router']['input'],'')
        self.assertFalse(source['initial']['simulation']['assignment']['router']['checked'])

    def test_reference_unit_preserves_all_64_verified_lpic2_values(self):
        rows = json.loads((ROOT / 'scenarios/presentation/lpic2.json').read_text())
        # Independent canonical digest read from verified329 candidate T132739Z.
        before = json.dumps(rows[:64], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        self.assertEqual((len(before), hashlib.sha256(before).hexdigest()),
                         (281786, 'faabfc5a5fe5c5186a706dc0a4770226966f9ad72d57bc91df39d32a20884063'))
        self.assertEqual([r['id'] for r in rows[64:66]],
                         ['lpic2-lvm-reference-repair', 'lpic2-lvm-reference-handover'])
        for row in rows[64:66]:
            self.assertEqual(len(row['requirements']), 5)
            self.assertEqual(len(row['practice']), 3)
            self.assertTrue(any('Save' in step['action'] for step in row['practice']))

    def test_config_unit_preserves_all66_verified_presentation_values(self):
        rows = json.loads((ROOT / 'scenarios/presentation/lpic2.json').read_text())
        before = json.dumps(rows[:66], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        self.assertEqual((len(before), hashlib.sha256(before).hexdigest()),
                         (286916, 'bc9e6dc34c7580915e3db8b67bf93af22a25cd956d34d13203ebd18767c13351'))
        self.assertEqual([r['id'] for r in rows[66:68]],
                         ['lpic2-lvm-filter-policy', 'lpic2-lvm-devices-file'])
        for row in rows[66:68]:
            self.assertEqual(len(row['requirements']),5)
            self.assertEqual(len(row['practice']),3)
            self.assertTrue(all(step['action'] and step['observe'] for step in row['practice']))

    def test_mount_unit_preserves_all68_verified_presentation_values(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        # Exact canonical values from verified333T175513Z; no historical digest
        # changes. Its original binary/archive and readback receipt are retained.
        before=json.dumps(rows[:68],sort_keys=True,separators=(',', ':'),ensure_ascii=False).encode()
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),
                         (292216, 'c5dc2a1aab46ee3351c3753bc219e58d8a7af3bbc5f07c9dce84ea5bc3683666'))
        self.assertEqual([r['id']for r in rows[68:70]],
                         ['lpic2-lvm-mount-repair','lpic2-lvm-mounted-handover'])
        for row in rows[68:70]:
            self.assertEqual(len(row['requirements']),4)
            self.assertEqual(len(row['practice']),3)
            self.assertTrue(all(step['action']and step['observe']for step in row['practice']))
            self.assertTrue(any('Save configuration'in step['action']for step in row['practice']))

    def test_secure_ports_preserve_all74_verified346_values_and_teach_actual_actions(self):
        rows=json.loads((ROOT/'scenarios/presentation/ccna.json').read_text())
        before=json.dumps(rows[:74],sort_keys=True,separators=(',', ':'),ensure_ascii=False).encode()
        # Read from the immutable346T234326Z resource, not a rewritten prefix.
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),(238273,'42f3c2e649e50370c18f47cb5f9779d488c7657485cbfdbc2909da8cfbf1fcf2'))
        self.assertEqual([r['id']for r in rows[74:77]],['ccna-security-identity','ccna-security-observe','ccna-security-recover'])
        for row in rows[74:77]:
            self.assertEqual(len(row['requirements']),3)
            self.assertEqual(len(row['practice']),3)
            self.assertGreaterEqual(len(row['concepts']),4)
            self.assertTrue(all(p['action'] and p['observe'] for p in row['practice']))
            self.assertTrue(any('Apply settings' in p['action'] for p in row['practice']))
            self.assertTrue(any('Send' in p['action'] and 'Check work' in p['action'] for p in row['practice']) or
                            (any('Send' in p['action'] for p in row['practice']) and any('Check work' in p['action'] for p in row['practice'])))
            self.assertTrue(any('not' in c['text'] for c in row['concepts']))

    def test_autofs_preserves_all70_verified335_presentation_values(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        before=json.dumps(rows[:70],sort_keys=True,separators=(',', ':'),ensure_ascii=False).encode()
        # Independently read from the immutable335T185236Zartifact, not a changed
        # current prefix. Earlier61/64/66/68digests remain unmodified.
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),(298010, '09b4fe40248647a147a614059ea216db3d43eb1e730f72cc415466ac50796758'))
        self.assertEqual([r['id']for r in rows[70:72]],['lpic2-autofs-indirect-repair','lpic2-autofs-direct-conversion'])
        for row in rows[70:72]:
            self.assertEqual(len(row['requirements']),4)
            self.assertEqual(len(row['practice']),3)
            self.assertTrue(all(s['action']and s['observe']for s in row['practice']))
            self.assertTrue(any('Save file'in s['action']for s in row['practice']))
            self.assertGreaterEqual(len(row['concepts']),4)

    def test_assembly_preserves_all72_verified337_presentation_values(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        # Canonical values retained before this batch and independently checkable
        # against immutable337T210025Z resources. Historical digests stay exact.
        before=json.dumps(rows[:72],sort_keys=True,separators=(',', ':'),ensure_ascii=False).encode()
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),(308653,'357b468b2ecd0d4bf685f27664347597f32d753d18f9d4dc6de29f04eff07bfd'))
        self.assertEqual([r['id']for r in rows[72:73]],['lpic2-raid1-assembly-policy'])
        row=rows[72]
        self.assertEqual(len(row['requirements']),5)
        self.assertEqual(len(row['practice']),3)
        self.assertGreaterEqual(len(row['concepts']),4)
        self.assertTrue(all(s['action']and s['observe']for s in row['practice']))
        self.assertTrue(any('Save file'in s['action']for s in row['practice']))

    def test_bridge_unit_keeps_all76_verified341_values_and_teaches_true_isolation(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        before=json.dumps(rows[:76],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),
                         (330924,'5398e44ada4fe294ec0c242d8bef1c10e42f9e1d6e9129fa81adf3b5c68e563d'))
        self.assertEqual([r['id']for r in rows[76:77]],['lpic2-bridge-lan-commission'])
        row=rows[76]
        self.assertEqual(len(row['concepts']),4)
        self.assertEqual(len(row['practice']),3)
        self.assertEqual(len(row['requirements']),4)
        self.assertTrue(any('isolat'in c['text']for c in row['concepts']))
        self.assertTrue(any('10.250.0.50/24'in step['action']for step in row['practice']))
        self.assertIn('all three physical cases',' '.join(row['requirements']))

    def test_pam_unit_keeps_all77_verified342_values_and_teaches_before_testing(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        before=json.dumps(rows[:77],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),
                         (336938,'6476f109d1a7a695cc9364838c3d396a8b8b9457a7c9e629a81762e4ee73993c'))
        self.assertEqual([r['id']for r in rows[77:78]],['lpic2-pam-auth-bypass'])
        row=rows[77]
        self.assertEqual(len(row['concepts']),5)
        self.assertEqual(len(row['practice']),3)
        self.assertEqual(len(row['requirements']),5)
        self.assertTrue(any('supplied'in c['text']for c in row['concepts']))
        self.assertTrue(any('Test stack'in step['action']for step in row['practice']))
        self.assertTrue(all(step['action']and step['observe']for step in row['practice']))

    def test_bond_capstone_keeps_all75_verified340_presentation_values(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        before=json.dumps(rows[:75],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),
                         (324930,'fd88cdb21c2bff9774a7235d6714be4caae6cd832492a366beb7e6a8c7ab85e0'))
        self.assertEqual([r['id']for r in rows[75:76]],['lpic2-bonded-vlan-handover'])
        row=rows[75]
        self.assertEqual(len(row['concepts']),4)
        self.assertEqual(len(row['practice']),3)
        text=json.dumps(row)
        self.assertIn('autoconnect-ports=1',text)
        self.assertIn('autoconnect=false',text)
        self.assertIn('bound',text)

    def test_vlan_keeps_all73_verified338_presentation_values_and_teaches_tags(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        # Actual canonical values read from immutable338T163747Z resources.
        before=json.dumps(rows[:73],sort_keys=True,separators=(',', ':'),ensure_ascii=False).encode()
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),(313868,'c592ff16ab11a5a47337f8ac0c72174380c072e2feb972942137d784873e95cd'))
        self.assertEqual([r['id']for r in rows[73:75]],['lpic2-vlan-profile-commission','lpic2-vlan-return-handover'])
        for row in rows[73:75]:
            self.assertGreaterEqual(len(row['concepts']),4)
            self.assertEqual(len(row['practice']),3)
            self.assertTrue(any('Test startup profiles'in step['action']for step in row['practice']))
            self.assertTrue(any('return'in c['text']for c in row['concepts']))
            self.assertIn('audit.nmconnection',' '.join(row['requirements']))

    def test_aging_preserves_prior_presentation_and_teaches_separate_bounded_practice(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic1.json').read_text())
        before=json.dumps(rows[:114],sort_keys=True,separators=(',', ':'),ensure_ascii=False).encode()
        # Actual canonical values read from immutable343T224545Z resources.
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),(479801,'16d80d0ba23d3c87ae25ca79ed84e4d063d53d2b3591a21bef2b5ee8e5796015'))
        ids=['lpic1-aging-rotation','lpic1-aging-grace','lpic1-aging-contract-reset']
        self.assertEqual([r['id']for r in rows[114:117]],ids)
        for row in rows[114:117]:
            self.assertGreaterEqual(len(row['concepts']),4)
            self.assertEqual(len(row['practice']),3)
            self.assertGreaterEqual(len(row['worked']),3)
            self.assertTrue(any('Run command'in r['action']for r in row['practice']))
            self.assertTrue(any('Check aging policy'in r['action']for r in row['practice']))
            self.assertTrue(all(not re.match(r'^\d+\.',c['title'])for c in row['concepts']))
        labs=json.loads((ROOT/'scenarios/aging-lessons-v1.json').read_text())['scenarios']
        self.assertEqual([s['id']for s in labs],ids)
        for lab in labs:
            self.assertEqual(lab['initial']['kind'],'Aging')
            state=lab['initial']['simulation'];d=lab['goal']['definition']
            self.assertEqual(state['origin'],lab['id'])
            self.assertNotIn('definition',state)
            self.assertEqual(state['assignment']['world'],d['initial']['initial'])
            self.assertEqual(state['practice']['world'],d['example']['initial'])
            self.assertEqual(state['assignment']['input'],'')
            self.assertFalse(state['assignment']['checked'])
            self.assertNotEqual(d['initial']['target'],d['example']['target'])
            self.assertTrue(d['example_worked'])
            for task in (d['initial'],d['example']):
                self.assertGreaterEqual(len(task['cases']),3)
                self.assertLessEqual(len(task['cases']),8)
                self.assertEqual(len({c['day']for c in task['cases']}),len(task['cases']))
                target=next(e['record']for e in task['initial']['entries']if e['login']==task['target'])
                self.assertNotEqual(target,task['expected'])

    def test_dhcp_lessons_preserve_old77_presentations_and_review_all_three_screens(self):
        rows = json.loads((ROOT / 'scenarios/presentation/ccna.json').read_text())
        before = json.dumps(rows[:77], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        self.assertEqual((len(before), hashlib.sha256(before).hexdigest()),
                         (253506, '0a1316597894a875d5b5e396cc828bea2c07a36b5cfd87d8653e425776a705ed'))
        self.assertEqual([r['id'] for r in rows[77:79]],
                         ['ccna-dhcp-server-boundary', 'ccna-dhcp-client-mac-check'])
        for row in rows[77:79]:
            self.assertEqual(row['status'], 'updated')
            self.assertGreaterEqual(len(row['concepts']), 3)
            self.assertGreaterEqual(len(row['practice']), 3)
            self.assertTrue(row['worked'])
            for view in ('introduction', 'example', 'assignment'):
                self.assertGreater(len(row['review'][view]), 100)
        self.assertIn('not authentication', rows[78]['concepts'][2]['title'])
        self.assertIn('\nenabled', rows[78]['worked'][1]['value'])

    def test_arp_lessons_preserve_old79_and_review_distinct_practical_screens(self):
        rows = json.loads((ROOT / 'scenarios/presentation/ccna.json').read_text())
        before = json.dumps(rows[:79], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        self.assertEqual((len(before), hashlib.sha256(before).hexdigest()),
                         (262045, 'd17a671c9bd28f3b26efc76b4269a905af553eef7e31dea6f96e09633f7d94d7'))
        ids = ['ccna-arp-sender-binding', 'ccna-arp-static-policy', 'ccna-arp-field-consistency']
        self.assertEqual([r['id'] for r in rows[79:82]], ids)
        for row in rows[79:82]:
            self.assertEqual(row['status'], 'updated')
            self.assertGreaterEqual(len(row['concepts']), 4)
            self.assertGreaterEqual(len(row['practice']), 3)
            self.assertGreaterEqual(len(row['worked']), 3)
            for view in ('introduction', 'example', 'assignment'):
                self.assertGreater(len(row['review'][view]), 100)
        pack = json.loads((ROOT / 'scenarios/arp-inspection-lessons-v1.json').read_text())['scenarios']
        self.assertEqual([s['id'] for s in pack], ids)
        for source in pack:
            self.assertEqual(source['initial']['kind'], 'ArpInspection')
            definition = source['goal']['definition']
            self.assertNotEqual(definition['initial'], definition['example'])
            for task in (definition['initial'], definition['example']):
                self.assertGreaterEqual(len(task['cases']), 3)
                self.assertLessEqual(len(task['cases']), 8)
            self.assertNotIn('definition', source['initial']['simulation'])
        self.assertIn('not proof', rows[79]['concepts'][0]['title'])
        self.assertIn('fallback', rows[80]['concepts'][2]['title'])
        self.assertIn('reply', rows[81]['concepts'][2]['title'])

    def test_management_preserves_old82_and_teaches_distinct_policy_tasks(self):
        rows = json.loads((ROOT / 'scenarios/presentation/ccna.json').read_text())
        before = json.dumps(rows[:82], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        self.assertEqual((len(before), hashlib.sha256(before).hexdigest()),
                         (274848, 'e487b77e20127db0239e99b1350422d3fe41faf8295b78aa15aac43daa567f08'))
        self.assertEqual([r['id'] for r in rows[82:84]],
                         ['ccna-management-protocols', 'ccna-management-source-policy'])
        for row in rows[82:84]:
            self.assertEqual(row['status'], 'updated')
            self.assertEqual(len(row['concepts']), 4)
            self.assertEqual(len(row['practice']), 3)
            self.assertGreaterEqual(len(row['worked']), 3)
            for view in ('introduction', 'example', 'assignment'):
                self.assertGreater(len(row['review'][view]), 100)
        self.assertIn('protocol', rows[82]['practice'][0]['title'])
        self.assertIn('Inspect trial', rows[82]['practice'][1]['action'])
        self.assertIn('every supplied line', rows[83]['concepts'][2]['title'])

    def test_local_access_preserves_old84_and_teaches_identity_levels_and_retirement(self):
        rows = json.loads((ROOT / 'scenarios/presentation/ccna.json').read_text())
        before = json.dumps(rows[:84], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        # Exact canonical values from immutable356 T083709Z, not a changed prefix.
        self.assertEqual((len(before), hashlib.sha256(before).hexdigest()),
                         (285028, '65acac30095f84d91b780477441e1e1981137ab564a825489854dddadb35f96f'))
        self.assertEqual([r['id'] for r in rows[84:86]],
                         ['ccna-local-account-setup', 'ccna-local-account-handover'])
        for row in rows[84:86]:
            self.assertEqual(row['status'], 'updated')
            self.assertEqual(len(row['concepts']), 4)
            self.assertEqual(len(row['practice']), 3)
            self.assertEqual(len(row['worked']), 4)
            self.assertEqual(len(row['requirements']), 3)
            for view in ('introduction', 'example', 'assignment'):
                self.assertGreater(len(row['review'][view]), 100)
            self.assertIn('Inspect trial', ' '.join(p['action'] for p in row['practice']))
            self.assertIn('Check configuration', ' '.join(p['action'] for p in row['practice']))
        self.assertIn('case-sensitive', rows[84]['concepts'][2]['text'])
        self.assertIn('level 1', rows[84]['practice'][2]['action'])
        self.assertIn('surplus', rows[85]['practice'][2]['action'])

    def test_ldap_clients_preserve_old78_and_teach_real_query_controls(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        before=json.dumps(rows[:78],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),
                         (342455,'bff48acc67142bc3e67138d57fb720d4eca67e7d6905d081bebddd73ca682e74'))
        self.assertEqual([r['id']for r in rows[78:80]],['lpic2-ldap-client-base','lpic2-ldap-contact-query'])
        for r in rows[78:80]:
            self.assertEqual(r['status'],'updated')
            self.assertEqual(len(r['concepts']),4)
            self.assertEqual(len(r['practice']),3)
            self.assertGreaterEqual(len(r['worked']),4)
            self.assertEqual(len(r['requirements']),4)
            for view in ('introduction','example','assignment'):
                self.assertGreater(len(r['review'][view]),100)
            actions=' '.join(p['action']for p in r['practice'])
            for label in ('Apply configuration','Run search','Check example'):
                self.assertIn(label,actions)
        self.assertIn('uid description mail',rows[79]['practice'][1]['action'])
        self.assertIn('mail absent',' '.join(w['value']for w in rows[79]['worked']))

    def test_current_flat_registry_retains_depth_guard_and_ordered_358_prefix(self):
        source=(ROOT/'src/pack.rs').read_text()
        current=teaching_inventory.registered_pack_paths(ROOT,'teaching-review')
        self.assertEqual(len(current),128)
        self.assertEqual(current[-1], 'scenarios/curriculum-continuation-v19.json')
        self.assertEqual(current[-2], 'scenarios/boot-entry-lessons-v63.json')
        detection=json.loads((ROOT/'scenarios/security-detection-lessons-v2.json').read_text())['scenarios']
        poe=json.loads((ROOT/'scenarios/poe-lessons-v1.json').read_text())['scenarios']
        kernel=json.loads((ROOT/'scenarios/kernel-build-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v1.json').read_text())['scenarios'],detection+poe)
        dkms=json.loads((ROOT/'scenarios/dkms-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v2.json').read_text())['scenarios'],detection+poe+kernel)
        initramfs=json.loads((ROOT/'scenarios/initramfs-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v3.json').read_text())['scenarios'],detection+poe+kernel+dkms)
        pxe=json.loads((ROOT/'scenarios/pxe-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v4.json').read_text())['scenarios'],detection+poe+kernel+dkms+initramfs)
        prior = detection + poe + kernel + dkms + initramfs + pxe
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v5.json').read_text())['scenarios'], prior)
        iscsi = json.loads((ROOT/'scenarios/iscsi-lessons-v1.json').read_text())['scenarios']
        through_iscsi = prior + iscsi
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v6.json').read_text())['scenarios'], through_iscsi)
        smart = json.loads((ROOT/'scenarios/ata-smart-lessons-v1.json').read_text())['scenarios']
        through_smart = through_iscsi + smart
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v7.json').read_text())['scenarios'], through_smart)
        https = json.loads((ROOT/'scenarios/apache-https-lessons-v1.json').read_text())['scenarios']
        through_https = through_smart + https
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v8.json').read_text())['scenarios'], through_https)
        desktop = json.loads((ROOT/'scenarios/desktop-profile-lessons-v1.json').read_text())['scenarios']
        through_desktop = through_https + desktop
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v9.json').read_text())['scenarios'], through_desktop)
        security = json.loads((ROOT/'scenarios/security-investigation-lessons-v1.json').read_text())['scenarios']
        through_security = through_desktop + security
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v10.json').read_text())['scenarios'], through_security)
        credentials = json.loads((ROOT/'scenarios/credentials-vpn-lessons-v1.json').read_text())['scenarios']
        through_credentials = through_security + credentials
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v11.json').read_text())['scenarios'], through_credentials)
        staged = json.loads((ROOT/'scenarios/staged-network-lessons-v1.json').read_text())['scenarios']
        through_staged = through_credentials + staged
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v12.json').read_text())['scenarios'], through_staged)
        evidence = json.loads((ROOT/'scenarios/network-evidence-review-lessons-v1.json').read_text())['scenarios']
        through_evidence = through_staged + evidence
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v13.json').read_text())['scenarios'], through_evidence)
        site = json.loads((ROOT/'scenarios/site-connectivity-lessons-v1.json').read_text())['scenarios']
        through_site = through_evidence + site
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v14.json').read_text())['scenarios'], through_site)
        security = json.loads((ROOT/'scenarios/host-and-dns-security-lessons-v1.json').read_text())['scenarios']
        through_security = through_site + security
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v15.json').read_text())['scenarios'], through_security)
        gateways = json.loads((ROOT/'scenarios/ccna-vlan-gateway-lessons-v1.json').read_text())['scenarios']
        through_gateways = through_security + gateways
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v16.json').read_text())['scenarios'], through_gateways)
        exports = json.loads((ROOT/'scenarios/nfs-export-operations-v1.json').read_text())['scenarios']
        through_exports = through_gateways + exports
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v17.json').read_text())['scenarios'], through_exports)
        routes = json.loads((ROOT/'scenarios/ccna-route-reading-lessons-v1.json').read_text())['scenarios']
        through_routes = through_exports + routes
        self.assertEqual(json.loads((ROOT/'scenarios/curriculum-continuation-v18.json').read_text())['scenarios'], through_routes)
        broadcasts = json.loads((ROOT/'scenarios/ccna-ospf-broadcast-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(json.loads((ROOT/current[-1]).read_text())['scenarios'], through_routes + broadcasts)
        original=json.loads((ROOT/'scenarios/auth-filter-lessons-v1.json').read_text())['scenarios']
        additions=json.loads((ROOT/'scenarios/ids-rule-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(detection,original+additions)
        self.assertEqual([s['id']for s in detection],['lpic2-auth-filter-detection','lpic2-auth-filter-exceptions','lpic2-ids-header-direction','lpic2-ids-content-specificity'])
        client=json.loads((ROOT/'scenarios/ldap-client-lessons-v1.json').read_text())['scenarios']
        administration=json.loads((ROOT/'scenarios/ldap-administration-v2.json').read_text())['scenarios']
        self.assertEqual(json.loads((ROOT/'scenarios/ldap-administration-v3.json').read_text())['scenarios'],client+administration)
        # The extension retains the exact original three authored tasks.
        boot_before=json.loads((ROOT/'scenarios/boot-entry-lessons-v1.json').read_text())['scenarios']
        boot_now=json.loads((ROOT/current[-2]).read_text())['scenarios']
        self.assertEqual(boot_now[:3],boot_before)
        # Preserve the exact old114+2 consolidation and then its full116objects.
        v55=json.loads((ROOT/'scenarios/boot-entry-lessons-v55.json').read_text())['scenarios']
        v6=json.loads((ROOT/'scenarios/linux-v6-lessons-v1.json').read_text())['scenarios']
        prior=json.loads((ROOT/'scenarios/boot-entry-lessons-v56.json').read_text())['scenarios']
        self.assertEqual(prior,v55+v6)
        added=json.loads((ROOT/'scenarios/linux-services-router-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(len(added),1)
        v57=json.loads((ROOT/'scenarios/boot-entry-lessons-v57.json').read_text())['scenarios']
        self.assertEqual(v57,prior+added)
        administration=json.loads((ROOT/'scenarios/linux-v6-admin-lessons-v1.json').read_text())['scenarios']
        self.assertEqual([r['id']for r in administration],['lpic2-router-ipv6-scope-recovery','lpic2-router-ipv6-ra-recovery','lpic2-router-ipv6-saved-settings'])
        v58=json.loads((ROOT/'scenarios/boot-entry-lessons-v58.json').read_text())['scenarios']
        self.assertEqual(v58,v57+administration)
        startup=json.loads((ROOT/'scenarios/linux-v6-startup-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(len(startup),1)
        v59=json.loads((ROOT/'scenarios/boot-entry-lessons-v59.json').read_text())['scenarios']
        self.assertEqual(v59,v58+startup)
        ftp=json.loads((ROOT/'scenarios/ftp-identity-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(len(ftp),3)
        v60=json.loads((ROOT/'scenarios/boot-entry-lessons-v60.json').read_text())['scenarios']
        self.assertEqual(v60,v59+ftp)
        exposure=json.loads((ROOT/'scenarios/service-exposure-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(len(exposure),2)
        v61=json.loads((ROOT/'scenarios/boot-entry-lessons-v61.json').read_text())['scenarios']
        self.assertEqual(v61,v60+exposure)
        vpn=json.loads((ROOT/'scenarios/vpn-point-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(len(vpn),2)
        v62=json.loads((ROOT/'scenarios/boot-entry-lessons-v62.json').read_text())['scenarios']
        self.assertEqual(v62,v61+vpn)
        patch=json.loads((ROOT/'scenarios/advisory-patch-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(len(patch),2)
        self.assertEqual(boot_now,v62+patch)

        self.assertEqual([s['id']for s in boot_now[3:5]],['lpic1-grub-default-fallback','lpic1-grub-recovery-entry'])
        previous=[]
        for path in current[:-2]:
            if path=='scenarios/kernel-module-curriculum-v2.json':
                previous.extend(['scenarios/kernel-module-lessons-v1.json','scenarios/kernel-module-names-lessons-v1.json'])
            elif path=='scenarios/ldap-administration-v3.json':
                previous.extend(['scenarios/ldap-client-lessons-v1.json','scenarios/ldap-administration-v2.json'])
            else:
                previous.append(path)
        self.assertEqual(len(previous),128)
        self.assertEqual(previous[-2:],['scenarios/ldap-administration-v2.json','scenarios/ipv6-interface-id-v1.json'])
        self.assertEqual(len(previous[:-3]),125)
        self.assertEqual(previous[-3:],['scenarios/ldap-client-lessons-v1.json','scenarios/ldap-administration-v2.json','scenarios/ipv6-interface-id-v1.json'])
        self.assertEqual(previous[-6:-3],['scenarios/management-policy-lessons-v1.json','scenarios/local-access-lessons-v1.json','scenarios/aaa-access-lessons-v1.json'])
        # The consolidated tail preserves every old change/utility object and
        # their order; the separate new task is the final373rd scenario.
        old=[]
        for name in ('ldap-change-lessons-v1.json','ldap-utility-lessons-v1.json'):
            old.extend(json.loads((ROOT/'scenarios'/name).read_text())['scenarios'])
        self.assertEqual(json.loads((ROOT/'scenarios/ldap-administration-v2.json').read_text())['scenarios'],old)
        self.assertIn('load_bundled_catalog(&accepted360_texts())',source)
        # Historical helper/source paths remain; no increase of traversal limits.
        self.assertIn('len(visiting) >= 64',(ROOT/'tools/verify_coverage.py').read_text())
        self.assertIn('maximum_paths = 128 if source_owned else 100',(ROOT/'tools/verify_coverage.py').read_text())

    def test_aaa_decisions_preserve_old86_and_teach_methods_before_independent_work(self):
        rows=json.loads((ROOT/'scenarios/presentation/ccna.json').read_text())
        before=json.dumps(rows[:86],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        # Canonical values from immutable358 T101716Z, never a rewritten prefix.
        self.assertEqual((len(before),hashlib.sha256(before).hexdigest()),
                         (295278,'8bcf7fd592df71eb4db4d65175b6f46fef6aa533d786bd469513ae61e02e5513'))
        self.assertEqual([r['id']for r in rows[86:88]],['ccna-aaa-method-recovery','ccna-aaa-authorization-recovery'])
        for r in rows[86:88]:
            self.assertEqual(r['status'],'updated')
            self.assertEqual((len(r['concepts']),len(r['worked']),len(r['practice']),len(r['requirements'])),(4,5,3,4))
            for view in ('introduction','example','assignment'):self.assertGreater(len(r['review'][view]),100)
            self.assertIn('Inspect trial',' '.join(p['action']for p in r['practice']))
            self.assertIn('Check configuration',' '.join(p['action']for p in r['practice']))
        self.assertIn('wrong word as an outage',rows[86]['concepts'][1]['text'])
        self.assertIn('local EXEC authorization',rows[87]['concepts'][0]['text'])

    def test_inventory_covers_the_registered_catalog_and_every_original_requirement(self):
        labs, reviews = teaching_inventory.content()
        self.assertEqual(len(labs), 543)
        self.assertEqual([lab["id"] for lab,_ in labs[507:509]],["ccna-poe-compatible-ports","ccna-poe-budget-recovery"])
        self.assertEqual([lab["id"] for lab,_ in labs[509:511]],["lpic2-kernel-root-build","lpic2-kernel-staged-handover"])
        self.assertEqual([lab["id"] for lab,_ in labs[511:513]],["lpic2-dkms-kernel-recovery","lpic2-dkms-version-handover"])
        management=[lab for lab,_ in labs if lab["unit"]=="management-policy-foundations"]
        self.assertEqual([lab["id"] for lab in management],["ccna-management-protocols","ccna-management-source-policy"])
        for lab in management:
            self.assertEqual(lab["initial"]["kind"],"ManagementPolicy")
            d=lab["goal"]["definition"]
            self.assertNotEqual(d["initial"]["network"],d["example"]["network"])
            self.assertEqual(len(d["initial"]["supplied"]["lines"]),16)
            self.assertEqual(len(d["example"]["supplied"]["lines"]),3)
            self.assertFalse(lab["initial"]["simulation"]["assignment"]["checked"])
            self.assertNotEqual(d["initial"]["starter"],d["initial"]["reference"])
            self.assertTrue(reviews[lab["id"]][0]["concepts"])
        swaps = [lab for lab, _ in labs if lab['unit'] == 'linux-swap-policy']
        self.assertEqual([lab['id'] for lab in swaps], ['lpic1-swap-prepared-activation', 'lpic1-swap-replacement-handover'])
        for lab in swaps:
            self.assertTrue(lab['initial']['storage']['inventory']['swap']['areas'])
            self.assertTrue(lab['storage_teaching']['fixture']['inventory']['swap']['areas'])
            self.assertTrue(reviews[lab['id']][0]['worked'])
            self.assertNotIn('swapon -p', lab['objective'])
        dual = [lab for lab, _ in labs if lab['unit'] == 'linux-host-ipv6']
        self.assertEqual([lab['id'] for lab in dual], ['lpic2-host-ipv6-addition', 'lpic2-host-ipv6-gateway-scope', 'lpic2-host-dual-stack-handover'])
        for lab in dual:
            d = lab['goal']['definition']
            self.assertIsNotNone(d['initial']['profile']['ipv6'])
            self.assertIsNotNone(d['example']['profile']['ipv6'])
            self.assertNotEqual(d['initial'], d['example'])
            self.assertTrue(d['requirements'])
            self.assertTrue(reviews[lab['id']][0]['worked'])
            self.assertNotIn('definition', lab['initial']['simulation'])
        recovery = [lab for lab, _ in labs if lab['unit'] == 'rpm-payload-recovery']
        self.assertEqual([lab['id'] for lab in recovery], ['lpic1-rpm-metadata-preserve', 'lpic1-rpm-mixed-recovery'])
        for lab in recovery:
            self.assertTrue(lab['goal']['definition']['allow_metadata_restore'])
            self.assertTrue(lab['rpm_packages_teaching']['definition']['allow_metadata_restore'])
            self.assertNotIn('definition', lab['initial'])
            self.assertTrue(reviews[lab['id']][0]['worked'])
            self.assertNotIn('--restore queue-agent', lab['objective'])
        triage = [lab for lab, _ in labs if lab['unit'] == 'shell-log-correlation']
        self.assertEqual([lab['id'] for lab in triage], ['shell-log-owner-triage'])
        self.assertEqual(len(triage[0]['goal']['cases']), 12)
        self.assertIn('at most 8 rows per input', triage[0]['objective'])
        self.assertNotIn('Storage team', triage[0]['initial']['source'])
        modules = [lab for lab, _ in labs if lab['initial']['kind'] == 'KernelModules']
        self.assertEqual([lab['id'] for lab in modules], ['lpic1-module-device-driver','lpic1-module-index-repair','lpic2-module-options-reload','lpic2-module-name-handover'])
        named = [lab for lab in modules if lab['goal']['definition']['assignment']['target'].get('request_names')]
        self.assertEqual([lab['id'] for lab in named], ['lpic2-module-name-handover'])
        self.assertEqual(named[0]['goal']['definition']['assignment']['target']['request_names'], ['capture_current','capture_legacy'])
        self.assertNotIn('alias capture_current lab_sensor', named[0]['initial']['simulation']['assignment']['config_draft'])
        for lab in modules:
            initial = lab['initial']['simulation']
            self.assertNotIn('definition', initial)
            self.assertEqual(initial['origin'], lab['id'])
            self.assertNotEqual(lab['goal']['definition']['assignment']['target']['module'], lab['goal']['definition']['example']['target']['module'])
            self.assertEqual(initial['assignment']['role'], 'assignment')
            self.assertEqual(initial['practice']['role'], 'practice')
            self.assertTrue(reviews[lab['id']][0]['worked'])
        sized = [lab for lab, _ in labs if lab['unit'] == 'linux-size-evidence']
        self.assertEqual([lab['id'] for lab in sized], ['lpic1-find-sized-incident','lpic1-find-rounded-evidence'])
        self.assertEqual([lab['goal']['definition']['assignment']['task']['criterion']['unit'] for lab in sized], ['bytes','kibibytes'])
        self.assertTrue(all(lab['goal']['definition']['assignment']['task']['kind']=='collect' for lab in sized))
        trees = [lab for lab, _ in labs if lab['unit'] == 'linux-recursive-handovers']
        self.assertEqual([lab['id'] for lab in trees], ['lpic1-tree-private-stage', 'lpic1-tree-verified-retirement'])
        self.assertTrue(all(lab['initial']['kind'] == 'RecursiveTree' for lab in trees))
        for lab in trees:
            initial = lab['initial']['simulation']
            self.assertEqual(initial['origin'], lab['id'])
            self.assertNotIn('definition', initial)
            self.assertEqual(initial['assignment']['role'], 'assignment')
            self.assertEqual(initial['practice']['role'], 'practice')
            self.assertNotEqual(initial['assignment']['tree']['cwd'], initial['practice']['tree']['cwd'])
            self.assertEqual(len(reviews[lab['id']][0]['requirements']), 4)
        transport = [lab for lab, _ in labs if lab['unit'] == 'ccna-transport-conversations']
        self.assertEqual([lab['id'] for lab in transport], ['ccna-transport-port-pairing', 'ccna-transport-return-policy'])
        self.assertEqual([len(lab['goal']['definition']['assignment']['requirements']) for lab in transport], [3, 6])
        self.assertTrue(all(lab['initial']['kind'] == 'Transport' for lab in transport))
        self.assertTrue(all('definition' not in lab['initial']['simulation'] for lab in transport))
        support = [lab for lab, _ in labs if lab['unit'] == 'linux-application-support-discovery']
        self.assertEqual([lab['id'] for lab in support], ['lpic1-whereis-support-handover'])
        self.assertEqual(support[0]['prerequisites'], ['lpic1-command-permission-search','lpic1-fhs-locate-active-files'])
        self.assertEqual(support[0]['goal']['requirements'][0]['kind'], 'WhereisDeployment')
        self.assertEqual(len(support[0]['terminal_teaching']['example']),8)
        self.assertNotIn('definition', support[0]['initial']['tree'])
        indexed = [lab for lab, _ in labs if lab['unit'] == 'linux-indexed-file-discovery']
        self.assertEqual([lab['id'] for lab in indexed], ['lpic1-file-index-handover'])
        self.assertEqual(indexed[0]['initial']['kind'], 'FileIndex')
        self.assertEqual(indexed[0]['prerequisites'], ['lpic1-fhs-locate-active-files','lpic1-search-incomplete-intake'])
        self.assertNotIn('definition', indexed[0]['initial']['simulation'])
        self.assertEqual(len(indexed[0]['goal']['definition']['teaching']['try_steps']),8)
        discovery = [lab for lab, _ in labs if lab['unit'] == 'linux-command-location-diagnosis']
        self.assertEqual([lab['id'] for lab in discovery], ['lpic1-command-permission-search'])
        self.assertEqual(discovery[0]['prerequisites'], ['lpic1-env-tool-path','lpic1-traversal'])
        self.assertEqual(discovery[0]['initial']['kind'], 'Filesystem')
        self.assertEqual(len(discovery[0]['terminal_teaching']['example']), 6)
        self.assertEqual(discovery[0]['goal']['requirements'][0]['kind'],'CommandLocation')
        defaults = [lab for lab, _ in labs if lab['unit'] == 'linux-account-defaults']
        self.assertEqual([lab['id'] for lab in defaults], ['lpic1-skeleton-handover'])
        self.assertEqual(defaults[0]['prerequisites'], ['lpic1-account-onboarding', 'lpic1-fhs-locate-active-files'])
        self.assertEqual(len(defaults[0]['initial']['simulation']['policy']['editable_skeleton_leaves']), 3)
        self.assertEqual(len(defaults[0]['goal']['requirements']), 11)
        self.assertEqual(len(defaults[0]['accounts_teaching']['template_teaching']['assignment']), 3)
        path_labs = [lab for lab, _ in labs if lab['unit'] == 'linux-command-search']
        self.assertEqual([lab['id'] for lab in path_labs], ['lpic1-env-tool-path'])
        self.assertEqual(path_labs[0]['prerequisites'], ['lpic1-env-parent-note', 'lpic1-fhs-locate-active-files'])
        self.assertEqual(len(path_labs[0]['goal']['definition']['assignment']['cases']), 6)
        self.assertEqual(len(path_labs[0]['goal']['definition']['example']['cases']), 2)
        self.assertEqual(path_labs[0]['initial']['kind'], 'Environment')
        listeners = [lab for lab, _ in labs if lab['unit'] == 'linux-listener-audit']
        self.assertEqual([lab['id'] for lab in listeners], ['lpic1-listener-audit'])
        self.assertEqual(listeners[0]['prerequisites'], ['lpic1-process-handover', 'lpic1-host-handover'])
        self.assertEqual(len(listeners[0]['goal']['requirements']), 12)
        self.assertEqual(len(listeners[0]['process_teaching']['example']), 8)
        self.assertEqual(listeners[0]['initial']['kind'], 'Processes')
        recovery = [lab for lab, _ in labs if lab['unit'] == 'shell-content-backups']
        self.assertEqual([lab['id']for lab in recovery],['shell-backup-content-set','shell-restore-validated-set'])
        self.assertEqual(recovery[0]['prerequisites'],['shell-managed-config'])
        self.assertEqual(recovery[1]['prerequisites'],['shell-backup-content-set'])
        self.assertEqual([len(lab['goal']['cases'])for lab in recovery],[12,12])
        self.assertTrue(all(lab['initial']['kind']=='Script' and lab['teaching']['example']!=lab['initial']['source']for lab in recovery))
        self.assertTrue(all(lab['goal']['cases'][0]['files']!=lab['teaching']['fixture']['files']for lab in recovery))
        managed = [lab for lab, _ in labs if lab['unit'] == 'shell-managed-configuration']
        self.assertEqual([lab['id'] for lab in managed], ['shell-managed-config'])
        self.assertEqual(managed[0]['prerequisites'],
                         ['shell-recovery-ledger', 'shell-recover-publication', 'shell-map-reconcile'])
        self.assertEqual(len(managed[0]['goal']['cases']), 12)
        self.assertEqual(managed[0]['initial']['kind'], 'Script')
        locations = [lab for lab, _ in labs if lab['unit'] == 'linux-file-locations']
        self.assertEqual([lab['id'] for lab in locations],
                         ['lpic1-fhs-application-layout', 'lpic1-fhs-locate-active-files'])
        self.assertEqual(locations[0]['prerequisites'],
                         ['lpic1-file-handover', 'lpic1-search-release-set'])
        self.assertEqual(locations[1]['prerequisites'], ['lpic1-fhs-application-layout'])
        self.assertTrue(all(lab['initial']['kind'] == 'Filesystem' for lab in locations))
        self.assertTrue(all(len(lab['terminal_teaching']['example']) == 8 for lab in locations))
        libraries = [lab for lab, _ in labs if lab['unit'] == 'linux-shared-library-resolution']
        self.assertEqual([lab['id'] for lab in libraries],
                         ['lpic1-library-search', 'lpic1-library-cache-handover'])
        self.assertEqual(libraries[0]['prerequisites'],
                         ['lpic1-env-child-settings', 'lpic1-links-handover'])
        self.assertEqual(libraries[1]['prerequisites'], ['lpic1-library-search'])
        self.assertTrue(all(lab['initial']['kind'] == 'SharedLibraries' for lab in libraries))
        self.assertTrue(all(reviews[lab['id']][0]['source'] ==
                            'scenarios/shared-library-lessons-v1.json' for lab in libraries))
        maps=[lab for lab,_ in labs if lab['unit']=='shell-associative-maps']
        self.assertEqual([lab['id']for lab in maps],['shell-map-lookup','shell-map-reconcile'])
        self.assertEqual([len(lab['goal']['cases'])for lab in maps],[12,12])
        self.assertTrue(all(lab['initial']['kind']=='Script' and lab['teaching']['example']!=lab['initial']['source']for lab in maps))
        self.assertEqual(maps[0]['prerequisites'],['shell-array-priority'])
        self.assertEqual(maps[1]['prerequisites'],['shell-map-lookup','shell-text-capstone','shell-preserve-lines'])
        conffiles=[lab for lab,_ in labs if lab['unit']=='debian-conffile-upgrades']
        self.assertEqual([lab['id']for lab in conffiles],['lpic1-conffile-retain','lpic1-conffile-adopt','lpic1-conffile-handover'])
        self.assertTrue(all(lab['initial']['kind']=='DebianPackages' for lab in conffiles))
        self.assertTrue(all(lab['goal']['definition']['requirements'][0]['kind']=='ConfigurationUpgrade' for lab in conffiles))
        shell_startup=[lab for lab,_ in labs if lab['unit']=='linux-shell-startup-files']
        self.assertEqual([lab['id']for lab in shell_startup],['lpic1-shell-startup-priority','lpic1-shell-startup-quiet'])
        self.assertTrue(all(lab['initial']['kind']=='StartupFiles' for lab in shell_startup))
        self.assertEqual([len(lab['goal']['definition']['initial']['cases'])for lab in shell_startup],[6,5])
        self.assertTrue(all('WORKSHOP' in str(lab['goal']['definition']['example_worked']) and 'APP_NAME' not in str(lab['goal']['definition']['example_worked']) for lab in shell_startup))
        integrity=[lab for lab,_ in labs if lab['unit']=='linux-filesystem-consistency']
        self.assertEqual([lab['id']for lab in integrity],['lpic1-fs-offline-accounting','lpic1-fs-retained-recovery'])
        self.assertTrue(all(lab['initial']['kind']=='FsIntegrity' for lab in integrity))
        self.assertTrue(all(len(lab['goal']['definition']['assignment']['cases'])==3 for lab in integrity))
        automatic=[lab for lab,_ in labs if lab['unit']=='linux-automatic-ethernet']
        self.assertEqual([lab['id']for lab in automatic],['lpic1-nm-dhcp-startup','lpic1-nm-dhcp-local-only'])
        self.assertTrue(all(lab['initial']['kind']=='NmProfiles' for lab in automatic))
        self.assertTrue(all(len(lab['goal']['definition']['initial']['cases'])==3 for lab in automatic))
        startup = {lab['id'] for lab, _ in labs if lab['unit'] == 'linux-startup-policy'}
        self.assertEqual(startup, {'lpic1-startup-current-future',
                                  'lpic1-startup-default-target',
                                  'lpic1-startup-install-links'})
        qos = {lab['id'] for lab, _ in labs if lab['unit'] == 'ccna-qos-hop'}
        self.assertEqual(qos, {'ccna-qos-mark-boundary', 'ccna-qos-queue-bursts',
                               'ccna-qos-trusted-handover'})
        selection = [lab for lab, _ in labs if lab['unit'] == 'backup-selection']
        self.assertEqual([lab['id'] for lab in selection], ['lpic2-backup-directory-selection'])
        self.assertEqual(selection[0]['prerequisites'], ['lpic2-backup-recovery-handover'])
        dhcp = [lab for lab, _ in labs if lab['unit'] == 'linux-dhcp-configuration']
        self.assertEqual([lab['id'] for lab in dhcp], ['lpic2-dhcp-dynamic-ranges', 'lpic2-dhcp-option-scope'])
        self.assertEqual(dhcp[1]['prerequisites'], ['lpic2-dhcp-dynamic-ranges'])
        nm = [lab for lab, _ in labs if lab['unit'] == 'linux-persistent-networking']
        self.assertEqual([lab['id'] for lab in nm], ['lpic1-nm-persistent-profile', 'lpic1-nm-autoconnect-choice', 'lpic1-nm-uplink-handover'])
        self.assertEqual(nm[1]['prerequisites'], ['lpic1-nm-persistent-profile'])
        self.assertEqual(nm[2]['prerequisites'], ['lpic1-nm-autoconnect-choice'])
        removable = [lab for lab, _ in labs if lab['unit'] == 'linux-removable-startup']
        self.assertEqual([lab['id'] for lab in removable], ['lpic1-removable-startup-policy'])
        names = [lab for lab, _ in labs if lab['unit'] == 'linux-client-name-sources']
        self.assertEqual([lab['id'] for lab in names], ['lpic1-local-host-alias', 'lpic1-name-source-fallback'])
        self.assertEqual(names[0]['prerequisites'], ['lpic1-host-resolver'])
        self.assertEqual(names[1]['prerequisites'], ['lpic1-local-host-alias'])
        search = [lab for lab, _ in labs if lab['unit'] == 'linux-resolver-search']
        self.assertEqual([lab['id'] for lab in search], ['lpic1-dns-search-suffixes', 'lpic1-dns-absolute-precedence'])
        self.assertEqual(search[0]['prerequisites'], ['lpic1-name-source-fallback'])
        self.assertEqual(search[1]['prerequisites'], ['lpic1-dns-search-suffixes'])
        provider = [lab for lab, _ in labs if lab['unit'] == 'debian-package-file-search']
        self.assertEqual([lab['id'] for lab in provider], ['lpic1-package-file-provider'])
        self.assertEqual(provider[0]['prerequisites'], ['lpic1-package-handover'])
        self.assertEqual(provider[0]['goal']['definition']['requirements'][0],
                         {'kind': 'InstalledFileProvider', 'path': '/usr/bin/ledger-check'})
        source_tasks=[lab for lab,_ in labs if lab['unit']=='debian-source-metadata']
        self.assertEqual([lab['id'] for lab in source_tasks],['lpic1-package-source-selection','lpic1-package-source-recovery'])
        self.assertEqual(source_tasks[0]['prerequisites'],['lpic1-package-file-provider'])
        self.assertEqual(source_tasks[1]['prerequisites'],['lpic1-package-source-selection'])
        self.assertTrue(all(lab['initial']['kind']=='DebianSources' for lab in source_tasks))
        tracing = [lab for lab, _ in labs if lab['unit'] == 'shell-tracing-diagnosis']
        self.assertEqual([lab['id'] for lab in tracing], ['shell-trace-delivery', 'shell-trace-fallback'])
        self.assertEqual(tracing[0]['prerequisites'], ['shell-capstone'])
        self.assertEqual(tracing[1]['prerequisites'], ['shell-trace-delivery'])
        self.assertTrue(all(lab['initial']['kind'] == 'Script' for lab in tracing))
        environment=[lab for lab,_ in labs if lab['unit']=='linux-shell-environment']
        self.assertEqual([lab['id'] for lab in environment],['lpic1-env-child-settings','lpic1-env-parent-note'])
        self.assertEqual(environment[0]['prerequisites'],['lpic1-file-navigation','shell-arguments'])
        self.assertEqual(environment[1]['prerequisites'],['lpic1-env-child-settings'])
        self.assertTrue(all(lab['initial']['kind']=='Environment' for lab in environment))
        mounts=[lab for lab,_ in labs if lab['unit']=='linux-removable-callers']
        self.assertEqual([lab['id']for lab in mounts],['lpic1-removable-personal-session','lpic1-removable-shared-handover'])
        self.assertTrue(all(lab['initial']['kind']=='UserMounts'for lab in mounts))
        conflicts=[lab for lab,_ in labs if lab['unit']=='debian-package-conflicts']
        self.assertEqual([lab['id']for lab in conflicts],['lpic1-package-conflict-retirement','lpic1-package-conflict-upgrade'])
        self.assertTrue(all(lab['initial']['kind']=='DebianPackages' for lab in conflicts))
        self.assertEqual(set(reviews), {lab['id'] for lab, _ in labs})
        self.assertFalse(any(r['status'] in ('pending', 'blocked') for r, _ in reviews.values()))
        for name, text in teaching_inventory.documents().items():
            self.assertEqual((ROOT / name).read_text(), text, name)

    def test_review_resources_include_only_declared_ordinary_json(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'src/ui').mkdir(parents=True)
            (root / 'scenarios/presentation').mkdir(parents=True)
            (root / 'src/ui/lesson_presentation.rs').write_text('include_str!("../../scenarios/presentation/public.json")')
            (root / 'scenarios/presentation/public.json').write_text('[]')
            private = root / 'scenarios/presentation/local-progress.json'
            private.write_text('not bundled')
            with patch.object(build_candidate, 'ROOT', root):
                sources = build_candidate.presentation_sources()
            self.assertEqual([p.name for p in sources], ['public.json'])
            self.assertEqual(private.read_text(), 'not bundled')
            (root / 'scenarios/presentation/public.json').unlink()
            (root / 'scenarios/presentation/public.json').symlink_to(private)
            with patch.object(build_candidate, 'ROOT', root), self.assertRaises(ValueError):
                build_candidate.presentation_sources()

    def test_review_candidate_records_metadata_module_unit_sources_and_exact_membership(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'src/course').mkdir(parents=True)
            (root / 'scenarios').mkdir()
            (root / 'src/pack.rs').write_text('include_str!("../scenarios/lab.json")')
            (root / 'src/course.rs').write_text('include_str!("../scenarios/base.json")')
            (root / 'src/course/metadata.rs').write_text(
                'include_str!("../../scenarios/base.json");'
                'include_str!("../../scenarios/extra.json")')
            (root / 'scenarios/lab.json').write_text('{}')
            import json
            for name in ('base', 'extra'):
                (root / 'scenarios' / (name + '.json')).write_text(json.dumps(
                    {'version': 1, 'units': [{'id': name}]}))
            with patch.object(build_candidate, 'ROOT', root), patch.object(
                    teaching_inventory, 'catalog', return_value=[
                        ({'unit': 'base'}, 'scenarios/lab.json'),
                        ({'unit': 'extra'}, 'scenarios/lab.json')]):
                self.assertEqual(build_candidate.review_catalog_identity(),
                                 {'labs': 2, 'units': 2, 'all_existing_review_labs': True})
                self.assertEqual(set(build_candidate.embedded_content_hashes()),
                                 {'lab.json', 'base.json', 'extra.json'})
                (root / 'scenarios/extra.json').write_text(json.dumps(
                    {'version': 1, 'units': [{'id': 'unregistered'}]}))
                with self.assertRaisesRegex(ValueError, 'differs'):
                    build_candidate.review_catalog_identity()
                (root / 'scenarios/extra.json').write_text(json.dumps(
                    {'version': 1, 'units': [{'id': 'base'}]}))
                with self.assertRaisesRegex(ValueError, 'Duplicate'):
                    build_candidate.review_catalog_identity()

    def test_review_package_explicitly_includes_the_shared_library_guides(self):
        tree = ast.parse((ROOT / 'tools/build_candidate.py').read_text())
        lists = [node.iter for node in ast.walk(tree) if isinstance(node, ast.For)
                 and isinstance(node.iter, ast.Tuple)
                 and any(isinstance(item, ast.Constant) and item.value ==
                         'TEACHING_PRESENTATION.md' for item in node.iter.elts)]
        self.assertEqual(len(lists), 1)
        names = [ast.literal_eval(item) for item in lists[0].elts]
        self.assertEqual(len(names), len(set(names)))
        for name in ('SHARED_LIBRARIES_SUBSET.md', 'SHARED_LIBRARIES_LESSONS.md', 'FHS_LESSONS.md', 'SHELL_MANAGED_CONFIG_LESSON.md', 'LISTENER_AUDIT_LESSON.md', 'PATH_LOOKUP_SUBSET.md', 'PATH_LOOKUP_LESSON.md', 'LVM_MOUNT_MODEL.md', 'LVM_MOUNT_LESSONS.md'):
            self.assertIn(name, names)
            self.assertTrue((ROOT / 'docs' / name).is_file())
            self.assertNotIn('..', Path(name).parts)

    def test_management_guides_are_in_the_actual_review_copy_loop(self):
        tree = ast.parse((ROOT / 'tools/build_candidate.py').read_text())
        lists = []
        for node in ast.walk(tree):
            if isinstance(node, ast.For) and isinstance(node.iter, ast.Tuple):
                values = [v.value for v in node.iter.elts if isinstance(v, ast.Constant) and isinstance(v.value, str)]
                copies = [v for v in ast.walk(node) if isinstance(v, ast.Call) and isinstance(v.func, ast.Attribute) and v.func.attr == 'copyfile']
                if 'TEACHING_PRESENTATION.md' in values and copies:
                    self.assertIn('LINUX_IPV6_STARTUP_SUBSET.md', values)
                    lists.append(values)
        self.assertEqual(len(lists), 1)
        for name in ('MANAGEMENT_POLICY_SUBSET.md', 'MANAGEMENT_POLICY_LESSONS.md', 'LOCAL_ACCESS_SUBSET.md', 'LOCAL_ACCESS_LESSONS.md', 'AAA_METHODS_SUBSET.md', 'AAA_DECISIONS_LESSONS.md', 'LDAP_QUERY_SUBSET.md', 'LDAP_CLIENT_LESSONS.md'):
            self.assertIn(name, lists[0])
            self.assertTrue((ROOT / 'docs' / name).is_file())

    def test_review_readmes_use_real_lines_and_state_assisted_acceptance_limits(self):
        tree = ast.parse((ROOT / 'tools/build_candidate.py').read_text())
        messages = [n.value for n in ast.walk(tree) if isinstance(n, ast.Constant) and isinstance(n.value, str) and n.value.startswith('LOCAL REVIEW ONLY.')]
        self.assertEqual(len(messages), 2)
        for message in messages:
            self.assertGreaterEqual(message.count('\n'), 3)
            self.assertNotIn('\\n', message)
        self.assertTrue(any('No independent novice' in message for message in messages))


class DirectoryChangePresentation(unittest.TestCase):
    def test_preserves_all80_prior_lpic2_lessons_and_matches_actual_actions(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:80],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(354805,'471dd848a54683f44af658e22c2aba09cf235b3e373cc4fe7f371efa5f2fc50c'))
        self.assertEqual([r['id']for r in rows[80:82]],['lpic2-ldap-person-import','lpic2-ldap-contact-maintenance'])
        for row in rows[80:82]:
            self.assertEqual(row['status'],'updated')
            self.assertEqual(len(row['concepts']),4)
            self.assertEqual(len(row['worked']),4)
            self.assertEqual(len(row['practice']),3)
            self.assertEqual(row['source'],'scenarios/ldap-change-lessons-v1.json')
            for step in row['practice']:
                self.assertTrue(step['title'].strip())
            self.assertEqual(len(row['requirements']),4)
            for view in ('introduction','example','assignment'):
                self.assertGreater(len(row['review'][view]),100)
            actions=' '.join(p['action']for p in row['practice'])
            for label in ('Apply LDIF','Sample inputs','Check example'):
                self.assertIn(label,actions)
        self.assertIn('not a Linux command',(ROOT/'src/ldap_change_workspace.rs').read_text())
        source=(ROOT/'tools/build_candidate.py').read_text()
        for name in ('LDAP_CHANGE_SUBSET.md','LDAP_CHANGE_LESSONS.md'):
            self.assertIn("'"+name+"'",source)


class DirectoryUtilityPresentation(unittest.TestCase):
    def test_retained82_values_and_two_specific_three_phase_lessons(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:82],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(369738,'12f582df4742962e62d558a1e94770f2023414c1573c30738b1e5ea91e757bd9'))
        self.assertEqual([r['id']for r in rows[82:84]],['lpic2-ldap-import-command','lpic2-ldap-stopped-update'])
        labs={s['id']:s for s in json.loads((ROOT/'scenarios/ldap-utility-lessons-v1.json').read_text())['scenarios']}
        for row in rows[82:84]:
            lab=labs[row['id']];d=lab['goal']['definition']
            self.assertEqual(row['status'],'updated')
            self.assertEqual(lab['objective'],' '.join(row['requirements']))
            for key in ('concepts','worked','practice','requirements'):self.assertGreaterEqual(len(row[key]),3)
            for phase in ('introduction','example','assignment'):self.assertGreater(len(row['review'][phase]),100)
            values=' '.join(w['value']for w in row['worked'])
            self.assertIn('dc=workshop-tools',values);self.assertNotIn('dc=orchard',values)
            self.assertTrue(all(p['title']and p['action']and p['observe']for p in row['practice']))
            self.assertIn('Run command',' '.join(p['action']for p in row['practice']))
            self.assertIn('Check example',' '.join(p['action']for p in row['practice']))
            self.assertNotEqual(d['initial']['reference'],d['example']['reference'])
            self.assertEqual(len(d['initial']['cases']),3)
    def test_internal_profile_envelope_is_not_shown_as_a_linux_command(self):
        ui=(ROOT/'src/ui/ldap_changes_page.rs').read_text()
        self.assertIn('Sample credentials',ui)
        self.assertIn('Draft::parse(&task.reference)',ui)
        self.assertRegex(ui,r'monospace\s*\(\s*ui\s*,\s*&p\.command')
        self.assertIn('.lock_focus(false)',ui)
        self.assertIn('if t.utility.is_some()',ui)


class DirectoryRetirementPresentation(unittest.TestCase):
    def test_retained84_values_and_specific_three_phase_retirement_work(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:84],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(379621,'5f0c29bbe7c8b38c4f4c90390a45b51dd1708c8bfdbf277911aa6ce6291fa034'))
        self.assertEqual([r['id']for r in rows[84:85]],['lpic2-ldap-account-retirement'])
        row=rows[84]
        lab=json.loads((ROOT/'scenarios/ldap-utility-lessons-v1.json').read_text())['scenarios'][2]
        self.assertEqual(lab['objective'],' '.join(row['requirements']))
        self.assertEqual(lab['goal']['definition']['initial']['utility']['input_kind'],'dn_list')
        for key in ('concepts','worked','practice','requirements'):self.assertEqual(len(row[key]),3)
        values=' '.join(w['value']for w in row['worked'])
        self.assertIn('dc=training-retirement',values);self.assertNotIn('dc=retirement,',values)
        for phase in ('introduction','example','assignment'):self.assertGreater(len(row['review'][phase]),100)
        self.assertIn('one DN per line',row['concepts'][1]['text'])
        self.assertIn('Run command',' '.join(p['action']for p in row['practice']))
        self.assertIn('Check example',' '.join(p['action']for p in row['practice']))
        ui=(ROOT/'src/ui/ldap_changes_page.rs').read_text()
        self.assertIn('DN retirement list',ui)
        self.assertIn('is_dn_list',ui)


class ProtectedPasswordPresentation(unittest.TestCase):
    def test_retained85_values_and_separate_protected_password_teaching(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:85],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        # Current post367admission baseline, not the old package's frozen metadata.
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(384818,'09e99cb85befd343c15dd361f49d717ed5a6476b247d78cc7df1c3c55594ac37'))
        self.assertEqual([r['id']for r in rows[85:86]],['lpic2-ldap-protected-password'])
        row=rows[85]
        lab=json.loads((ROOT/'scenarios/ldap-utility-lessons-v1.json').read_text())['scenarios'][3]
        self.assertEqual(lab['objective'],' '.join(row['requirements']))
        self.assertEqual([len(row[k])for k in ('concepts','worked','practice','requirements')],[4,4,3,4])
        for phase in ('introduction','example','assignment'):self.assertGreater(len(row['review'][phase]),100)
        d=lab['goal']['definition'];initial=d['initial'];example=d['example']
        self.assertNotEqual(initial['actor'],example['actor'])
        self.assertNotEqual(initial['uri'],example['uri'])
        self.assertNotEqual(initial['starter'],initial['reference'])
        self.assertEqual(len(initial['cases']),3)
        self.assertNotEqual(initial['cases'][0]['policy'],initial['cases'][1]['policy'])
        self.assertEqual(initial['starter']['bind_profile'],'replacement')
        self.assertEqual(initial['reference']['bind_profile'],'current')
        for task in (initial,example):
            self.assertNotIn('sample_password',json.dumps(task['starter']))
            self.assertNotIn('sample_password',json.dumps(task['reference']))
        ui=(ROOT/'src/ui/ldap_password_page.rs').read_text()
        for text in ('Expected','Actual','No current results','All sample results','Fill worked example'):
            self.assertIn(text,ui)
        self.assertIn('if draft != s.draft',ui)
        self.assertIn('s.edit(draft)',ui)


class DirectoryWriteDelegationPresentation(unittest.TestCase):
    def test_retained86_values_and_three_phase_write_policy_requirements(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:86],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(390687,'0b7a3cf7605af401ddddbc0d00ed06ba365cefb52ec5ec450fab418b7bba6976'))
        self.assertEqual([r['id']for r in rows[86:87]],['lpic2-ldap-contact-delegation'])
        row=rows[86];lab=json.loads((ROOT/'scenarios/ldap-utility-lessons-v1.json').read_text())['scenarios'][4]
        self.assertEqual(lab['objective'],' '.join(row['requirements']))
        self.assertEqual([len(row[k])for k in ('concepts','worked','practice','requirements')],[4,3,3,4])
        d=lab['goal']['definition'];self.assertNotEqual(d['initial']['reference'],d['example']['reference'])
        self.assertIn('delegation-workshop',d['example']['reference'])
        self.assertNotIn('delegation-workshop',d['initial']['reference'])
        self.assertEqual(sum(len(c['probes'])for c in d['initial']['cases']),43)
        for phase in ('introduction','example','assignment'):self.assertGreater(len(row['review'][phase]),100)
        self.assertIn('Run request',' '.join(p['action']for p in row['practice']))
        self.assertIn('Check example',' '.join(p['action']for p in row['practice']))
        ui=(ROOT/'src/ui/ldap_access_page.rs').read_text()
        for label in ('Before','After','Task requires','Actual','No current result','Write policy needs attention'):
            self.assertIn(label,ui)
        self.assertIn('s.edit(draft)',ui)

class DirectoryConfigurationDeploymentPresentation(unittest.TestCase):
    def test_all87_prior_values_and_real_configuration_task_are_preserved(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:87],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(396133,'0a535c936f13426d01f3698230acf63ada9a56300ecdd66bc0a8c6048bd1cffb'))
        self.assertEqual([r['id']for r in rows[87:88]],['lpic2-ldap-config-deployment'])
        row=rows[87];lab=json.loads((ROOT/'scenarios/ldap-utility-lessons-v1.json').read_text())['scenarios'][5]
        self.assertEqual(lab['objective'],' '.join(row['requirements']))
        self.assertEqual([len(row[k])for k in ('concepts','worked','practice','requirements')],[4,3,3,4])
        d=lab['goal']['definition'];self.assertNotEqual(d['initial']['configuration']['target'],d['example']['configuration']['target'])
        self.assertNotEqual(d['initial']['reference'],d['example']['reference'])
        self.assertEqual(sum(len(c['probes'])for c in d['initial']['cases']),43)
        self.assertIn('olcSuffix',' '.join(c['text']for c in row['concepts']))
        self.assertIn('Apply and try request',' '.join(c['action']for c in row['practice']))
        for phase in ('introduction','example','assignment'):self.assertGreater(len(row['review'][phase]),100)
        ui=(ROOT/'src/ui/ldap_access_page.rs').read_text()
        for label in ('LDIF configuration change','Supplied configuration entries','Inspect supplied access values','Apply and try request','Actual configuration values','Check configuration'):
            self.assertIn(label,ui)

class DirectorySchemaPresentation(unittest.TestCase):
    def test_all88_prior_values_and_actual_schema_workspace_teaching_are_preserved(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic2.json').read_text())
        old=json.dumps(rows[:88],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(402872,'b4933287dba61e9092b45f302dd3ab60ddb7dba47e9a6ec8caad78799cf104e1'))
        self.assertEqual([r['id']for r in rows[88:89]],['lpic2-ldap-schema-extension'])
        row=rows[88];lab=json.loads((ROOT/'scenarios/ldap-utility-lessons-v1.json').read_text())['scenarios'][6]
        self.assertEqual(lab['objective'],' '.join(row['requirements']))
        self.assertEqual([len(row[k])for k in ('concepts','worked','practice','requirements')],[4,3,3,5])
        d=lab['goal']['definition'];self.assertEqual(lab['initial']['kind'],'LdapSchema')
        self.assertNotEqual(d['initial']['schema_name'],d['example']['schema_name'])
        self.assertNotEqual(d['initial']['oid_branch'],d['example']['oid_branch'])
        for task in (d['initial'],d['example']):
            self.assertEqual(sum(len(c['probes'])for c in task['cases']),33)
            self.assertNotEqual(task['starter'],task['reference'])
        self.assertIn('MUST does not itself limit',' '.join(c['text']for c in row['concepts']))
        self.assertIn('Apply schema and try request',' '.join(c['action']for c in row['practice']))
        ui=(ROOT/'src/ui/ldap_schema_page.rs').read_text()
        for label in ('Schema LDIF editor','Existing sample entries','Apply schema and try request','Actual result:','Actual stored entries','Check schema','Failing requests:','First issue'):
            self.assertIn(label,ui)


class DirectorySchemaConsumerPresentation(unittest.TestCase):
    def test_preserves_all89_prior_values_and_separate_consumer_practice(self):
        rows = json.loads((ROOT / 'scenarios/presentation/lpic2.json').read_text())
        old = json.dumps(rows[:89], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        # Verified371R2 actual copied presentation, checked against BUILD_INFO.
        self.assertEqual((len(old), hashlib.sha256(old).hexdigest()),
                         (409769, '2a91c8cad3f25fe7623612d3d916074aa823184110f3810f6ec601a996781ec7'))
        self.assertEqual([r['id'] for r in rows[89:93]],
                         ['lpic2-ldap-schema-consumer', 'lpic2-efi-removable-recovery',
                          'lpic2-grub-one-boot-recovery', 'lpic2-rescue-startup-repair'])
        row = rows[89]
        lab = json.loads((ROOT / 'scenarios/ldap-utility-lessons-v1.json').read_text())['scenarios'][7]
        self.assertEqual(lab['objective'], ' '.join(row['requirements']))
        self.assertEqual(row['status'], 'updated')
        self.assertEqual([len(row[k]) for k in ('concepts', 'worked', 'practice', 'requirements')], [4, 4, 3, 5])
        self.assertIn('case-exact does not mean byte-for-byte', row['concepts'][1]['text'])
        self.assertIn('keeps Undefined', row['concepts'][3]['text'])
        self.assertIn('names and user IDs change', row['requirements'][4])
        for phase in ('introduction', 'example', 'assignment'):
            self.assertGreater(len(row['review'][phase]), 100)
        ui = (ROOT / 'src/ui/ldap_schema_query_page.rs').read_text()
        for label in ('Schema LDIF editor', 'Apply schema and run filter', 'Check configuration'):
            self.assertIn(label, ui)
        workspace = (ROOT / 'src/ldap_schema_query_workspace.rs').read_text()
        self.assertIn('Changed names and IDs', workspace)
