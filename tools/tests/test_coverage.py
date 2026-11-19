import copy
import csv
import json
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import objective_inventory as objective
import verify_coverage

class HostDnsSecurityContinuationTests(unittest.TestCase):
    def test_preserves_exact_site_predecessors_and_both_security_lessons(self):
        root = Path(__file__).resolve().parents[2]

        def pack(name):
            return {
                lab['id']: lab
                for lab in json.loads((root / name).read_text())['scenarios']
            }

        selected = 'scenarios/curriculum-continuation-v15.json'
        previous = 'scenarios/curriculum-continuation-v14.json'
        authored = 'scenarios/host-and-dns-security-lessons-v1.json'
        predecessor = pack(previous)
        additions = pack(authored)
        current = pack(selected)
        self.assertEqual(len(predecessor), 33)
        self.assertEqual(len(additions), 2)
        self.assertEqual(list(current.items()), list(predecessor.items()) + list(additions.items()))
        for origin, rows in ((previous, predecessor), (authored, additions)):
            self.assertTrue(verify_coverage.preserved_authored_pack(origin, rows, {selected: current}))
            modified = copy.deepcopy(rows)
            next(iter(modified.values()))['goal'] = 'Changed origin'
            with self.assertRaisesRegex(ValueError, 'Host and DNS security continuation'):
                verify_coverage.preserved_authored_pack(origin, modified, {selected: current})
        for index in (0, 32, 33, 34):
            modified = copy.deepcopy(current)
            list(modified.values())[index]['title'] = 'Changed lesson'
            with self.assertRaisesRegex(ValueError, 'Host and DNS security continuation'):
                verify_coverage.preserved_authored_pack(authored, additions, {selected: modified})
        reordered = dict(reversed(list(current.items())))
        with self.assertRaisesRegex(ValueError, 'Host and DNS security continuation'):
            verify_coverage.preserved_authored_pack(authored, additions, {selected: reordered})


class SiteConnectivityContinuationTests(unittest.TestCase):
    def test_preserves_exact_evidence_predecessors_and_both_site_lessons(self):
        root = Path(__file__).resolve().parents[2]

        def pack(name):
            return {
                lab['id']: lab
                for lab in json.loads((root / name).read_text())['scenarios']
            }

        selected = 'scenarios/curriculum-continuation-v14.json'
        previous = 'scenarios/curriculum-continuation-v13.json'
        authored = 'scenarios/site-connectivity-lessons-v1.json'
        predecessor = pack(previous)
        additions = pack(authored)
        current = pack(selected)
        self.assertEqual(len(predecessor), 31)
        self.assertEqual(len(additions), 2)
        self.assertEqual(list(current.items()), list(predecessor.items()) + list(additions.items()))
        for origin, rows in ((previous, predecessor), (authored, additions)):
            self.assertTrue(verify_coverage.preserved_authored_pack(origin, rows, {selected: current}))
            modified = copy.deepcopy(rows)
            next(iter(modified.values()))['goal'] = 'Changed origin'
            with self.assertRaisesRegex(ValueError, 'Site connectivity continuation'):
                verify_coverage.preserved_authored_pack(origin, modified, {selected: current})
        for index in (0, 30, 31, 32):
            modified = copy.deepcopy(current)
            list(modified.values())[index]['title'] = 'Changed lesson'
            with self.assertRaisesRegex(ValueError, 'Site connectivity continuation'):
                verify_coverage.preserved_authored_pack(authored, additions, {selected: modified})
        reordered = dict(reversed(list(current.items())))
        with self.assertRaisesRegex(ValueError, 'Site connectivity continuation'):
            verify_coverage.preserved_authored_pack(authored, additions, {selected: reordered})


class NetworkEvidenceContinuationTests(unittest.TestCase):
    def test_preserves_exact_staged_predecessors_and_both_evidence_review_lessons(self):
        root = Path(__file__).resolve().parents[2]

        def pack(name):
            return {
                lab['id']: lab
                for lab in json.loads((root / name).read_text())['scenarios']
            }

        selected = 'scenarios/curriculum-continuation-v13.json'
        previous = 'scenarios/curriculum-continuation-v12.json'
        authored = 'scenarios/network-evidence-review-lessons-v1.json'
        predecessor = pack(previous)
        additions = pack(authored)
        current = pack(selected)
        self.assertEqual(len(predecessor), 29)
        self.assertEqual(len(additions), 2)
        self.assertEqual(list(current.items()), list(predecessor.items()) + list(additions.items()))
        for origin, rows in ((previous, predecessor), (authored, additions)):
            self.assertTrue(verify_coverage.preserved_authored_pack(origin, rows, {selected: current}))
            modified = copy.deepcopy(rows)
            next(iter(modified.values()))['goal'] = 'Changed origin'
            with self.assertRaisesRegex(ValueError, 'Network evidence review continuation'):
                verify_coverage.preserved_authored_pack(origin, modified, {selected: current})
        for index in (0, 28, 29, 30):
            modified = copy.deepcopy(current)
            list(modified.values())[index]['title'] = 'Changed lesson'
            with self.assertRaisesRegex(ValueError, 'Network evidence review continuation'):
                verify_coverage.preserved_authored_pack(authored, additions, {selected: modified})
        reordered = dict(reversed(list(current.items())))
        with self.assertRaisesRegex(ValueError, 'Network evidence review continuation'):
            verify_coverage.preserved_authored_pack(authored, additions, {selected: reordered})


class StagedNetworkContinuationTests(unittest.TestCase):
    def test_preserves_exact_policy_predecessors_and_both_staged_lessons(self):
        root = Path(__file__).resolve().parents[2]

        def pack(name):
            return {
                lab['id']: lab
                for lab in json.loads((root / name).read_text())['scenarios']
            }

        selected = 'scenarios/curriculum-continuation-v12.json'
        previous = 'scenarios/curriculum-continuation-v11.json'
        authored = 'scenarios/staged-network-lessons-v1.json'
        predecessor = pack(previous)
        additions = pack(authored)
        current = pack(selected)
        self.assertEqual(len(predecessor), 27)
        self.assertEqual(len(additions), 2)
        self.assertEqual(list(current.items()), list(predecessor.items()) + list(additions.items()))
        for origin, rows in ((previous, predecessor), (authored, additions)):
            self.assertTrue(verify_coverage.preserved_authored_pack(origin, rows, {selected: current}))
            modified = copy.deepcopy(rows)
            next(iter(modified.values()))['goal'] = 'Changed origin'
            with self.assertRaisesRegex(ValueError, 'Staged network changes continuation'):
                verify_coverage.preserved_authored_pack(origin, modified, {selected: current})
        for index in (0, 26, 27, 28):
            modified = copy.deepcopy(current)
            list(modified.values())[index]['title'] = 'Changed lesson'
            with self.assertRaisesRegex(ValueError, 'Staged network changes continuation'):
                verify_coverage.preserved_authored_pack(authored, additions, {selected: modified})
        reordered = dict(reversed(list(current.items())))
        with self.assertRaisesRegex(ValueError, 'Staged network changes continuation'):
            verify_coverage.preserved_authored_pack(authored, additions, {selected: reordered})

class CredentialsVpnContinuationTests(unittest.TestCase):
    def test_preserves_exact_investigation_predecessors_and_both_policy_lessons(self):
        root = Path(__file__).resolve().parents[2]

        def pack(name):
            return {
                lab['id']: lab
                for lab in json.loads((root / name).read_text())['scenarios']
            }

        selected = 'scenarios/curriculum-continuation-v11.json'
        previous = 'scenarios/curriculum-continuation-v10.json'
        authored = 'scenarios/credentials-vpn-lessons-v1.json'
        predecessor = pack(previous)
        additions = pack(authored)
        current = pack(selected)
        self.assertEqual(len(predecessor), 25)
        self.assertEqual(len(additions), 2)
        self.assertEqual(list(current.items()), list(predecessor.items()) + list(additions.items()))
        for origin, rows in ((previous, predecessor), (authored, additions)):
            self.assertTrue(verify_coverage.preserved_authored_pack(origin, rows, {selected: current}))
            modified = copy.deepcopy(rows)
            next(iter(modified.values()))['goal'] = 'Changed origin'
            with self.assertRaisesRegex(ValueError, 'Credentials and VPN continuation'):
                verify_coverage.preserved_authored_pack(origin, modified, {selected: current})
        for index in (0, 24, 25, 26):
            modified = copy.deepcopy(current)
            list(modified.values())[index]['title'] = 'Changed lesson'
            with self.assertRaisesRegex(ValueError, 'Credentials and VPN continuation'):
                verify_coverage.preserved_authored_pack(authored, additions, {selected: modified})
        reordered = dict(reversed(list(current.items())))
        with self.assertRaisesRegex(ValueError, 'Credentials and VPN continuation'):
            verify_coverage.preserved_authored_pack(authored, additions, {selected: reordered})

class SecurityInvestigationContinuationTests(unittest.TestCase):
    def test_preserves_exact_desktop_predecessors_and_both_security_lessons(self):
        root = Path(__file__).resolve().parents[2]

        def pack(name):
            return {
                lab['id']: lab
                for lab in json.loads((root / name).read_text())['scenarios']
            }

        selected = 'scenarios/curriculum-continuation-v10.json'
        previous = 'scenarios/curriculum-continuation-v9.json'
        authored = 'scenarios/security-investigation-lessons-v1.json'
        predecessor = pack(previous)
        additions = pack(authored)
        current = pack(selected)
        self.assertEqual(len(predecessor), 23)
        self.assertEqual(len(additions), 2)
        self.assertEqual(list(current.items()), list(predecessor.items()) + list(additions.items()))
        for origin, rows in ((previous, predecessor), (authored, additions)):
            self.assertTrue(verify_coverage.preserved_authored_pack(origin, rows, {selected: current}))
            modified = copy.deepcopy(rows)
            next(iter(modified.values()))['goal'] = 'Changed origin'
            with self.assertRaisesRegex(ValueError, 'Security investigation continuation'):
                verify_coverage.preserved_authored_pack(origin, modified, {selected: current})
        for index in (0, 22, 23, 24):
            modified = copy.deepcopy(current)
            list(modified.values())[index]['title'] = 'Changed lesson'
            with self.assertRaisesRegex(ValueError, 'Security investigation continuation'):
                verify_coverage.preserved_authored_pack(authored, additions, {selected: modified})
        reordered = dict(reversed(list(current.items())))
        with self.assertRaisesRegex(ValueError, 'Security investigation continuation'):
            verify_coverage.preserved_authored_pack(authored, additions, {selected: reordered})

class DesktopContinuationTests(unittest.TestCase):
    def test_preserves_exact_https_predecessors_and_the_three_desktop_lessons(self):
        root = Path(__file__).resolve().parents[2]

        def pack(name):
            return {
                lab['id']: lab
                for lab in json.loads((root / name).read_text())['scenarios']
            }

        selected = 'scenarios/curriculum-continuation-v9.json'
        previous = 'scenarios/curriculum-continuation-v8.json'
        authored = 'scenarios/desktop-profile-lessons-v1.json'
        current = pack(selected)
        self.assertEqual(len(current), 23)
        for origin in (previous, authored):
            self.assertTrue(
                verify_coverage.preserved_authored_pack(origin, pack(origin), {selected: current})
            )
        for index in (0, 19, 20, 21, 22):
            changed = copy.deepcopy(current)
            list(changed.values())[index]['title'] = 'Changed content'
            with self.assertRaisesRegex(ValueError, 'Desktop profile continuation'):
                verify_coverage.preserved_authored_pack(authored, pack(authored), {selected: changed})
        changed = dict(reversed(list(current.items())))
        with self.assertRaisesRegex(ValueError, 'Desktop profile continuation'):
            verify_coverage.preserved_authored_pack(authored, pack(authored), {selected: changed})

class ApacheHttpsContinuationTests(unittest.TestCase):
    def test_preserves_exact_smart_predecessors_and_both_https_lessons(self):
        root = Path(__file__).resolve().parents[2]

        def pack(name):
            return {
                lab['id']: lab
                for lab in json.loads((root / name).read_text())['scenarios']
            }

        selected = 'scenarios/curriculum-continuation-v8.json'
        previous = 'scenarios/curriculum-continuation-v7.json'
        authored = 'scenarios/apache-https-lessons-v1.json'
        current = pack(selected)
        self.assertEqual(len(current), 20)
        for origin in (previous, authored):
            self.assertTrue(
                verify_coverage.preserved_authored_pack(origin, pack(origin), {selected: current})
            )
        for index in (0, 17, 18, 19):
            modified = copy.deepcopy(current)
            list(modified.values())[index]['title'] = 'Changed content'
            with self.assertRaisesRegex(ValueError, 'Apache HTTPS continuation'):
                verify_coverage.preserved_authored_pack(authored, pack(authored), {selected: modified})
        reordered = dict(reversed(list(current.items())))
        with self.assertRaisesRegex(ValueError, 'Apache HTTPS continuation'):
            verify_coverage.preserved_authored_pack(authored, pack(authored), {selected: reordered})

class PoeContinuationConsolidationTests(unittest.TestCase):
    def test_exact_prior_detection_and_new_power_tasks_are_required(self):
        root=Path(__file__).resolve().parents[2]
        def pack(name):return {lab['id']:lab for lab in json.loads((root/name).read_text())['scenarios']}
        selected='scenarios/curriculum-continuation-v1.json';current=pack(selected)
        self.assertEqual(len(current),6)
        origins=['scenarios/security-detection-lessons-v2.json','scenarios/poe-lessons-v1.json','scenarios/auth-filter-lessons-v1.json','scenarios/ids-rule-lessons-v1.json']
        for origin in origins:
            source=pack(origin)
            self.assertTrue(verify_coverage.preserved_authored_pack(origin,source,{selected:current}))
            changed=copy.deepcopy(source);next(iter(changed.values()))['title']='Changed authored content'
            with self.assertRaises(ValueError):verify_coverage.preserved_authored_pack(origin,changed,{selected:current})
        for at in range(6):
            changed=copy.deepcopy(current);list(changed.values())[at]['title']='Changed consolidation value'
            with self.assertRaisesRegex(ValueError,'PoE continuation'):
                verify_coverage.preserved_authored_pack(origins[0],pack(origins[0]),{selected:changed})
        changed=copy.deepcopy(current);changed.pop(next(iter(changed)))
        with self.assertRaisesRegex(ValueError,'PoE continuation'):
            verify_coverage.preserved_authored_pack(origins[0],pack(origins[0]),{selected:changed})
        reordered=dict(reversed(list(current.items())))
        with self.assertRaisesRegex(ValueError,'PoE continuation'):
            verify_coverage.preserved_authored_pack(origins[0],pack(origins[0]),{selected:reordered})

class LdapClientAdministrationConsolidationTests(unittest.TestCase):
    def test_exact_two_client_and_ten_admin_values_remain_required(self):
        root = Path(__file__).resolve().parents[2]
        def pack(name):
            return {lab['id']: lab for lab in json.loads((root/name).read_text())['scenarios']}
        target = 'scenarios/ldap-administration-v3.json'
        selected = pack(target)
        self.assertEqual(len(selected), 12)
        for origin in ['scenarios/ldap-client-lessons-v1.json',
                       'scenarios/ldap-administration-v2.json',
                       'scenarios/ldap-change-lessons-v1.json',
                       'scenarios/ldap-utility-lessons-v1.json']:
            source = pack(origin)
            self.assertTrue(verify_coverage.preserved_authored_pack(origin, source, {target: selected}))
            changed = copy.deepcopy(source)
            next(iter(changed.values()))['title'] = 'Changed predecessor'
            with self.assertRaises(ValueError):
                verify_coverage.preserved_authored_pack(origin, changed, {target: selected})
        for index in [0, 2]:
            changed = copy.deepcopy(selected)
            list(changed.values())[index]['summary'] = 'Changed current value'
            with self.assertRaisesRegex(ValueError, 'exact predecessors'):
                verify_coverage.preserved_authored_pack('scenarios/ldap-client-lessons-v1.json',
                    pack('scenarios/ldap-client-lessons-v1.json'), {target: changed})

class AdvisoryPatchConsolidationTests(unittest.TestCase):
    def test_exact128_predecessors_and_two_patch_tasks_are_required(self):
        root=Path(__file__).resolve().parents[2]
        def pack(name):
            return {lab['id']:lab for lab in json.loads((root/name).read_text())['scenarios']}
        selected='scenarios/boot-entry-lessons-v63.json';current=pack(selected)
        for origin in ['scenarios/boot-entry-lessons-v62.json','scenarios/advisory-patch-lessons-v1.json']:
            source=pack(origin)
            self.assertTrue(verify_coverage.preserved_authored_pack(origin,source,{selected:current}))
            changed=copy.deepcopy(source);next(iter(changed.values()))['title']='Changed authored content'
            with self.assertRaises(ValueError):
                verify_coverage.preserved_authored_pack(origin,changed,{selected:current})
        for at in [0,127,128,129]:
            changed=copy.deepcopy(current);list(changed.values())[at]['title']='Changed consolidation value'
            with self.assertRaisesRegex(ValueError,'Advisory patch consolidation'):
                verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v62.json',
                    pack('scenarios/boot-entry-lessons-v62.json'),{selected:changed})

class FtpConsolidationTests(unittest.TestCase):
    def test_exact_prior121_and_three_new_values_are_required(self):
        root=Path(__file__).resolve().parents[2]
        def pack(name):
            labs=json.loads((root/name).read_text())['scenarios']
            return {lab['id']:lab for lab in labs}
        selected='scenarios/boot-entry-lessons-v60.json'
        current=pack(selected)
        for origin in ['scenarios/boot-entry-lessons-v59.json','scenarios/ftp-identity-lessons-v1.json']:
            source=pack(origin)
            self.assertTrue(verify_coverage.preserved_authored_pack(origin,source,{selected:current}))
            changed=copy.deepcopy(source);next(iter(changed.values()))['title']='Changed authored content'
            with self.assertRaises(ValueError):verify_coverage.preserved_authored_pack(origin,changed,{selected:current})
        for at in [0,121]:
            changed=copy.deepcopy(current);list(changed.values())[at]['title']='Changed consolidation value'
            with self.assertRaisesRegex(ValueError,'FTP identity consolidation'):
                verify_coverage.preserved_authored_pack('scenarios/boot-entry-lessons-v59.json',pack('scenarios/boot-entry-lessons-v59.json'),{selected:changed})


class VpnConsolidationTests(unittest.TestCase):
    def test_exact126_predecessors_and_two_vpn_tasks_are_required(self):
        root=Path(__file__).resolve().parents[2]
        def pack(name):return {lab['id']:lab for lab in json.loads((root/name).read_text())['scenarios']}
        selected='scenarios/boot-entry-lessons-v62.json';current=pack(selected)
        origins=['scenarios/boot-entry-lessons-v61.json','scenarios/vpn-point-lessons-v1.json']
        for origin in origins:
            source=pack(origin)
            self.assertTrue(verify_coverage.preserved_authored_pack(origin,source,{selected:current}))
            changed=copy.deepcopy(source);next(iter(changed.values()))['title']='Changed authored content'
            with self.assertRaises(ValueError):verify_coverage.preserved_authored_pack(origin,changed,{selected:current})
        for at in [0,125,126,127]:
            changed=copy.deepcopy(current);list(changed.values())[at]['title']='Changed consolidation value'
            with self.assertRaisesRegex(ValueError,'VPN consolidation'):
                verify_coverage.preserved_authored_pack(origins[0],pack(origins[0]),{selected:changed})
        changed=copy.deepcopy(current);changed.pop(next(iter(changed)))
        with self.assertRaisesRegex(ValueError,'VPN consolidation'):
            verify_coverage.preserved_authored_pack(origins[0],pack(origins[0]),{selected:changed})


class ServiceExposureConsolidationTests(unittest.TestCase):
    def test_exact124_predecessors_and_two_exposure_tasks_are_required(self):
        root=Path(__file__).resolve().parents[2]
        def pack(name):
            return {lab['id']:lab for lab in json.loads((root/name).read_text())['scenarios']}
        selected='scenarios/boot-entry-lessons-v61.json'
        current=pack(selected)
        origins=['scenarios/boot-entry-lessons-v60.json','scenarios/service-exposure-lessons-v1.json']
        for origin in origins:
            source=pack(origin)
            self.assertTrue(verify_coverage.preserved_authored_pack(origin,source,{selected:current}))
            changed=copy.deepcopy(source);next(iter(changed.values()))['title']='Changed authored content'
            with self.assertRaises(ValueError):verify_coverage.preserved_authored_pack(origin,changed,{selected:current})
        for at in [0,123,124,125]:
            changed=copy.deepcopy(current);list(changed.values())[at]['title']='Changed consolidation value'
            with self.assertRaisesRegex(ValueError,'Exposure consolidation'):
                verify_coverage.preserved_authored_pack(origins[0],pack(origins[0]),{selected:changed})
        changed=copy.deepcopy(current);changed.pop(next(iter(changed)))
        with self.assertRaisesRegex(ValueError,'Exposure consolidation'):
            verify_coverage.preserved_authored_pack(origins[0],pack(origins[0]),{selected:changed})


def lpi(version='5.0'):
    return f'''<p>Exam Objectives Version: {version}</p>
    <h4 id="sample">103.1 Example domain</h4>
    <table><tr><td>Weight</td><td>2</td></tr><tr><td>Description</td><td>Required sample behavior.</td></tr></table>
    <b>Key Knowledge Areas:</b><ul><li>First skill.</li></ul><ul><li>Second skill.</li></ul>
    <b>The following is a partial list of the used files, terms and utilities:</b>
    <ul><li>example-one</li><li>example-two</li></ul>
    <h4 id="removed">104.4 Removed</h4><h3>Next domain</h3>'''


class ObjectiveTests(unittest.TestCase):
    def test_lpi_separate_lists_terms_and_removed_marker(self):
        items = objective.lpi_items(lpi(), '5.0')
        self.assertEqual([i['reference'] for i in items], ['103.1', '103.1.K1', '103.1.K2', '103.1.T1', '103.1.T2'])
        self.assertTrue(all(i['anchor'] == 'sample' for i in items))
        self.assertEqual(items[1]['normalized_sha256'], objective.digest('First skill.'))
        self.assertFalse(any('skill' in json.dumps(i) for i in items))

    def test_description_only_or_weight_changes_invalidate_parent_fingerprint(self):
        before = objective.lpi_items(lpi(), '5.0')
        after = objective.lpi_items(lpi().replace('Required sample behavior.', 'A different required behavior.'), '5.0')
        self.assertNotEqual(before[0]['normalized_sha256'], after[0]['normalized_sha256'])
        self.assertNotEqual(before[0]['description_sha256'], after[0]['description_sha256'])
        self.assertEqual(before[1:], after[1:])
        weighted = objective.lpi_items(lpi().replace('<td>2</td>', '<td>4</td>'), '5.0')
        self.assertEqual(weighted[0]['weight'], 4)
        self.assertNotEqual(before[0]['normalized_sha256'], weighted[0]['normalized_sha256'])

    def test_version_missing_sections_and_broken_lists_fail(self):
        for source in [lpi('4.5'), lpi().replace('Key Knowledge Areas:', 'Unknown areas'), lpi().replace('</ul>', '', 1), lpi().replace('<li>First skill.</li>', '<li></li>')]:
            with self.assertRaises(ValueError):
                objective.lpi_items(source, '5.0')

    def test_cisco_wrapped_lines_and_footer_are_not_lost_or_merged(self):
        text = '''CCNA Exam v1.1 (200-301)
20% 1.0 Domain
    1.1 Parent starts
        then continues here
        1.1.a Child category
2024 Cisco Systems, Inc. This document is Cisco Public. Page 1
\f  1.2 Another parent
'''
        items = objective.cisco_items(text)
        self.assertEqual([i['reference'] for i in items], ['1.1', '1.1.a', '1.2'])
        self.assertEqual(items[0]['normalized_sha256'], objective.digest('Parent starts then continues here'))
        with self.assertRaises(ValueError):
            objective.cisco_items(text + ' 1.1 Duplicate\n')
        with self.assertRaises(ValueError):
            objective.cisco_items('CCNA Exam v1.1 (200-301)\n1.1.a Orphan\n')


class CoverageTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root / 'docs').mkdir()
        (self.root / 'tests').mkdir()
        (self.root / 'scenarios').mkdir()
        (self.root / 'src').mkdir()
        self.registry(['../scenarios/sample.json'])
        (self.root / 'tests/example.rs').write_text('// fixture only')
        (self.root / 'scenarios/sample.json').write_text(json.dumps({'schema_version': 1, 'scenarios': [{'id': 'sample'}]}))
        items = [objective.item('1.1', '1.1', 'objective', 0, 'Example'), objective.item('1.1.a', '1.1', 'subobjective', 0, 'Child')]
        (self.root / 'docs/OBJECTIVE_SOURCES.json').write_text(json.dumps({'schema_version': 1, 'sources': [{'blueprint': 'Sample v1', 'url': 'https://example.test/objectives', 'items': items}]}))
        self.parent('Planned', 'None')
        self.rows = [dict(blueprint='Sample v1', reference=i['reference'], parent='1.1', kind=i['kind'], source_sha256=i['normalized_sha256'], source_url='https://example.test/objectives', source_position='0', status='Open', implemented_scenarios='', verified_scope='', evidence_tests='', remaining='Required model is not implemented.', planned_assignment='Investigate sample state') for i in items]

    def registry(self, paths):
        entries = ''.join('        include_str!("' + path + '"),\n' for path in paths)
        (self.root / 'src/pack.rs').write_text(
            'pub fn bundled() -> Result<Vec<Scenario>, String> {\n'
            '    load_catalog(&[\n' + entries + '    ])\n}\n')

    def pack(self, name, ids):
        (self.root / 'scenarios' / name).write_text(json.dumps(
            {'schema_version': 1, 'scenarios': [{'id': item} for item in ids]}))

    def parent(self, status, labs):
        with (self.root / 'docs/CURRICULUM.csv').open('w', newline='') as stream:
            writer = csv.DictWriter(stream, ['blueprint', 'objective_id', 'status', 'implemented_scenarios'])
            writer.writeheader()
            writer.writerow(dict(blueprint='Sample v1', objective_id='1.1', status=status, implemented_scenarios=labs))

    def write(self, rows):
        with (self.root / 'docs/SKILL_COVERAGE.csv').open('w', newline='') as f:
            writer = csv.DictWriter(f, list(self.rows[0]));writer.writeheader();writer.writerows(rows)

    def test_open_and_scoped_partial_claims_validate(self):
        self.write(self.rows)
        self.assertEqual(verify_coverage.validate(self.root)['open'], 2)
        rows = copy.deepcopy(self.rows)
        rows[0].update(status='Partial practice', implemented_scenarios='sample', verified_scope='One deterministic sample only.', evidence_tests='tests/example.rs')
        self.write(rows)
        self.parent('Partial', 'sample')
        self.assertEqual(verify_coverage.validate(self.root)['partial'], 1)

    def test_csv_surplus_or_missing_columns_cannot_silently_change_scope(self):
        self.write(self.rows)
        for name in ['CURRICULUM.csv', 'SKILL_COVERAGE.csv']:
            path = self.root / 'docs' / name
            original = path.read_text()
            for corrupt in [lambda line: line + ',unquoted extra field', lambda line: line.rsplit(',', 1)[0]]:
                lines = original.splitlines()
                lines[1] = corrupt(lines[1])
                path.write_text('\n'.join(lines) + '\n')
                with self.assertRaisesRegex(ValueError, 'CSV columns'):
                    verify_coverage.validate(self.root)
            path.write_text(original)

    def test_duplicate_parent_objective_is_rejected(self):
        self.write(self.rows)
        path = self.root / 'docs/CURRICULUM.csv'
        source = path.read_text()
        for blueprint in ['Sample v1', 'Sample v2 upcoming']:
            lines = source.splitlines()
            row = lines[1].replace('Sample v1', blueprint)
            path.write_text(source + (row + '\n') * (2 if 'upcoming' in blueprint else 1))
            with self.assertRaisesRegex(ValueError, 'Duplicate parent'):
                verify_coverage.validate(self.root)

    def test_distinct_future_parent_does_not_inherit_current_coverage(self):
        self.write(self.rows)
        path = self.root / 'docs/CURRICULUM.csv'
        source = path.read_text()
        path.write_text(source + source.splitlines()[1].replace('Sample v1', 'Sample v2 upcoming') + '\n')
        result = verify_coverage.validate(self.root)
        self.assertEqual(result['references'], 2)
        self.assertEqual(result['current_objectives'], 1)
        self.assertEqual(result['open'], 2)

    def test_parent_status_and_credited_labs_must_match_scoped_current_evidence(self):
        rows = copy.deepcopy(self.rows)
        rows[0].update(status='Partial practice', implemented_scenarios='sample', verified_scope='Bounded sample.', evidence_tests='tests/example.rs')
        self.write(rows)
        for status, labs in [('Planned', 'None'), ('Partial', 'None'), ('Partial', 'other')]:
            self.parent(status, labs)
            with self.assertRaisesRegex(ValueError, 'Parent planning'):
                verify_coverage.validate(self.root)
        self.parent('Partial', 'sample')
        self.assertEqual(verify_coverage.validate(self.root)['partial'], 1)

    def test_missing_duplicate_wrong_source_or_inherited_claim_is_rejected(self):
        variants = [self.rows[:-1], self.rows + self.rows[:1]]
        for change in [dict(source_url='https://example.test/unrelated'), dict(parent='9.9'), dict(source_position='3'), dict(source_sha256='0' * 64), dict(implemented_scenarios='sample'), dict(status='Complete'), dict(remaining='')]:
            rows = copy.deepcopy(self.rows);rows[0].update(change);variants.append(rows)
        for rows in variants:
            self.write(rows)
            with self.assertRaises(ValueError):
                verify_coverage.validate(self.root)

    def test_missing_lab_and_unsafe_or_absent_test_evidence_is_rejected(self):
        for lab, path in [('absent', 'tests/example.rs'), ('sample', 'tests/absent.rs'), ('sample', 'tests/../../outside'), ('sample', '/tmp/outside')]:
            rows = copy.deepcopy(self.rows)
            rows[0].update(status='Partial practice', implemented_scenarios=lab, verified_scope='Bounded example.', evidence_tests=path)
            self.write(rows)
            with self.assertRaises(ValueError):
                verify_coverage.validate(self.root)

    def test_unregistered_drafts_are_reported_separately_and_never_credited(self):
        self.write(self.rows)
        ids = ['draft-selective', 'draft-levels', 'draft-branch', 'draft-handover']
        self.pack('backup-drafts-v1.json', ids)
        result = verify_coverage.validate(self.root)
        self.assertEqual(result['authored_labs'], 1)
        self.assertEqual(result['registered_labs'], 1)
        self.assertEqual(result['registered_pack_paths'], ['scenarios/sample.json'])
        self.assertEqual(result['unregistered_draft_ids'], sorted(ids))
        for item in ids:
            rows = copy.deepcopy(self.rows)
            rows[0].update(status='Partial practice', implemented_scenarios=item,
                           verified_scope='This is still an unregistered draft.',
                           evidence_tests='tests/example.rs')
            self.write(rows)
            self.parent('Partial', item)
            with self.assertRaisesRegex(ValueError, 'unregistered'):
                verify_coverage.validate(self.root)

    def test_unit_metadata_does_not_consume_the_draft_pack_budget(self):
        scenario_dir = self.root / 'scenarios'
        unit = dict(id='sample-unit', track='CCNA', title='Sample',
                    summary='Fixture', prerequisites=[], labs=['sample'])
        (scenario_dir / 'sample-units.json').write_text(json.dumps(
            {'version': 1, 'units': [unit]}))
        for index in range(100):
            self.pack(f'draft-{index}.json', [f'draft-{index}'])
        labs, drafts, _, units = verify_coverage.catalog_inventory(self.root)
        self.assertEqual(list(labs), ['sample'])
        self.assertEqual(len(drafts), 100)
        self.assertEqual(units, ['scenarios/sample-units.json'])
        self.pack('draft-overflow.json', ['overflow'])
        with self.assertRaisesRegex(ValueError, 'Unregistered data inventory capacity'):
            verify_coverage.catalog_inventory(self.root)

    def test_metadata_budget_is_bounded_across_index_files(self):
        for part, count in enumerate((128, 129)):
            units = [
                dict(id=f'unit-{part}-{index}', track='CCNA', title='Sample',
                     summary='Fixture', prerequisites=[], labs=['sample'])
                for index in range(count)
            ]
            (self.root / f'scenarios/units-{part}.json').write_text(json.dumps(
                {'version': 1, 'units': units}))
        with self.assertRaisesRegex(ValueError, 'Unit metadata inventory capacity'):
            verify_coverage.catalog_inventory(self.root)

    def test_only_static_registered_paths_gain_evidence_after_an_explicit_append(self):
        self.write(self.rows)
        self.pack('second-v1.json', ['second'])
        self.registry(['../scenarios/sample.json', '../scenarios/second-v1.json'])
        rows = copy.deepcopy(self.rows)
        rows[0].update(status='Partial practice', implemented_scenarios='second',
                       verified_scope='Registered example only.', evidence_tests='tests/example.rs')
        self.write(rows)
        self.parent('Partial', 'second')
        result = verify_coverage.validate(self.root)
        self.assertEqual(result['authored_labs'], 2)
        self.assertEqual(result['unregistered_draft_ids'], [])

    def test_unknown_dynamic_empty_or_commented_registry_cannot_gain_credit(self):
        self.write(self.rows)
        path = self.root / 'src/pack.rs'
        original = path.read_text()
        variants = [
            original.replace('load_catalog(&[', 'other_loader(&['),
            original.replace('include_str!("../scenarios/sample.json")', 'include_str!(PACK_PATH)'),
            original.replace('include_str!("../scenarios/sample.json")', 'include_str!(concat!("../scenarios/", "sample.json"))'),
            original.replace('    load_catalog(&[', '    if offline { return load_catalog(&[]); } load_catalog(&['),
            'pub fn bundled() -> Result<Vec<Scenario>, String> { load_catalog(&[]) }',
            '/* ' + original + ' */',
            '// ' + original.replace('\n', '\n// '),
            original + original,
            '#[cfg(feature="other")]\n' + original,
            original + '\nfn extra() {}\n',
            original.replace('include_str!', 'unknown_macro!'),
            original + '\0',
            original + '/* unterminated',
        ]
        for source in variants:
            path.write_text(source)
            with self.assertRaises(ValueError, msg=source):
                verify_coverage.validate(self.root)

    def test_comments_raw_strings_and_escaped_literals_do_not_forge_registry_entries(self):
        self.write(self.rows)
        path = self.root / 'src/pack.rs'
        original = path.read_text()
        fake = original.replace('sample.json', 'unregistered.json')
        quoted = 'const TEXT: &str = r###"' + fake + '"###;\n' + 'const URL: &str = "https://example.test/\\"quoted\\"";\nconst SLASH: char = \'/\'; const QUOTE: char = \'"\';\n'
        # Nested comments, line comments and fake raw-string function headers
        # remain inert. Only the actual literal function list is counted.
        path.write_text('/* outer /* nested */ ' + fake + ' */\n' + quoted +
                        original.replace('        include_str!',
                                         '        // include_str!("../scenarios/ignored.json"),\n        include_str!'))
        self.assertEqual(verify_coverage.validate(self.root)['authored_labs'], 1)

    def test_duplicate_missing_unsafe_or_control_registered_paths_are_rejected(self):
        self.write(self.rows)
        variants = [
            ['../scenarios/sample.json', '../scenarios/sample.json'],
            ['../scenarios/missing.json'], ['../scenarios/../outside.json'],
            ['/tmp/outside.json'], ['../scenarios/nested/sample.json'],
            ['../scenarios/./sample.json'], ['../scenarios/Sample.json'],
            ['../scenarios/sample.json\\escape'], ['../scenarios/sample\n.json'],
            ['../scenarios/sample\r.json'], ['../scenarios/sample\t.json'],
        ]
        for paths in variants:
            self.registry(paths)
            with self.assertRaises(ValueError, msg=paths):
                verify_coverage.validate(self.root)

    def test_registered_or_unregistered_symlink_never_reads_outside_project(self):
        self.write(self.rows)
        outside = self.root / 'outside.json'
        outside.write_text(json.dumps({'schema_version': 1, 'scenarios': [{'id': 'outside'}]}))
        linked = self.root / 'scenarios/linked.json'
        linked.symlink_to(outside)
        with self.assertRaisesRegex(ValueError, 'unsafe'):
            verify_coverage.validate(self.root)
        self.registry(['../scenarios/linked.json'])
        with self.assertRaisesRegex(ValueError, 'unsafe'):
            verify_coverage.validate(self.root)

    def test_duplicate_ids_and_malformed_pack_data_remain_rejected(self):
        self.write(self.rows)
        self.pack('duplicate-v1.json', ['sample'])
        with self.assertRaisesRegex(ValueError, 'Duplicate authored ID'):
            verify_coverage.validate(self.root)
        draft = self.root / 'scenarios/duplicate-v1.json'
        malformed = [
            {'schema_version': 2, 'scenarios': [{'id': 'draft'}]},
            {'schema_version': True, 'scenarios': [{'id': 'draft'}]},
            {'schema_version': 1, 'scenarios': []},
            {'schema_version': 1, 'scenarios': [{'id': 'draft'}, {'id': 'draft'}]},
            {'schema_version': 1, 'scenarios': [{'id': 'not safe'}]},
            {'schema_version': 1, 'scenarios': [{'id': 'draft'}], 'extra': 'unknown'},
        ]
        for value in malformed:
            draft.write_text(json.dumps(value))
            with self.assertRaises(ValueError):
                verify_coverage.validate(self.root)
        draft.write_text('{"schema_version":1,"schema_version":1,"scenarios":[{"id":"draft"}]}')
        with self.assertRaisesRegex(ValueError, 'Duplicate pack JSON key'):
            verify_coverage.validate(self.root)


    def modern_registry(self):
        source = '''
fn bundled_texts() -> Vec<&'static str> {
    vec![include_str!("../scenarios/sample.json"),]
}
pub fn bundled() -> Result<Vec<Scenario>, String> { load_catalog(&bundled_texts()) }
fn accepted_texts() -> Vec<&'static str> {
    let mut texts = bundled_texts();
    texts.push(include_str!("../scenarios/second.json"));
    texts
}
pub fn ccna_pilot() -> Result<Vec<Scenario>, String> { load_catalog(&accepted_texts()) }
'''
        (self.root / 'src/pack.rs').write_text(source)
        self.pack('second.json', ['second'])
        self.write(self.rows)
        return source

    def test_only_explicit_review_bundle_loader_can_resolve101_to128_static_files(self):
        def source(n, loader='load_bundled_catalog'):
            paths = ','.join(f'include_str!("../scenarios/part-{i}.json")' for i in range(n)) + ','
            return ("fn accepted_texts() -> Vec<&'static str> { vec![" + paths + "] }\n"
                    "pub fn ccna_pilot() -> Result<Vec<Scenario>, String> { " + loader + "(&accepted_texts()) }")
        for n in (101,128):
            (self.root / 'src/pack.rs').write_text(source(n))
            self.assertEqual(len(verify_coverage.registered_pack_paths(self.root,'teaching-review')),n)
        for n, loader in ((129,'load_bundled_catalog'),(101,'load_catalog'),(101,'other_loader')):
            (self.root / 'src/pack.rs').write_text(source(n,loader))
            with self.assertRaises(ValueError):
                verify_coverage.registered_pack_paths(self.root,'teaching-review')
        (self.root / 'src/pack.rs').write_text(source(101).replace('ccna_pilot','bundled'))
        with self.assertRaises(ValueError):
            verify_coverage.registered_pack_paths(self.root,'bundled')

    def test_selected_modern_registry_preserves_default_and_resolves_review_chain(self):
        self.modern_registry()
        default = verify_coverage.validate(self.root)
        review = verify_coverage.validate(self.root, 'teaching-review')
        self.assertEqual(default['catalog_profile'], 'bundled')
        self.assertEqual(default['authored_labs'], 1)
        self.assertEqual(default['unregistered_draft_ids'], ['second'])
        self.assertEqual(review['registry_entrypoint'], 'ccna_pilot')
        self.assertEqual(review['registered_pack_paths'],
                         ['scenarios/sample.json', 'scenarios/second.json'])
        self.assertEqual(review['authored_labs'], 2)
        self.assertEqual(review['unregistered_draft_ids'], [])
        with self.assertRaises(ValueError):
            verify_coverage.validate(self.root, 'latest')

    def test_unreachable_helpers_and_unit_lists_never_grant_lab_credit(self):
        source = self.modern_registry()
        self.pack('experiment.json', ['experiment'])
        source += '''
fn accepted999_texts() -> Vec<&'static str> {
    let mut texts = accepted_texts();
    texts.push(include_str!("../scenarios/experiment.json")); texts
}
'''
        (self.root / 'src/pack.rs').write_text(source)
        self.unit_document('notes.json', ['experiment'])
        result = verify_coverage.validate(self.root, 'teaching-review')
        self.assertEqual(result['authored_labs'], 2)
        self.assertEqual(result['unregistered_draft_ids'], ['experiment'])
        self.assertEqual(result['unit_index_paths'], ['scenarios/notes.json'])
        self.rows[0].update(status='Partial practice', implemented_scenarios='experiment',
                            verified_scope='Unregistered experiments cannot earn credit.',
                            evidence_tests='tests/example.rs')
        self.parent('Partial', 'experiment'); self.write(self.rows)
        with self.assertRaisesRegex(ValueError, 'unregistered'):
            verify_coverage.validate(self.root, 'teaching-review')

    def test_modern_registry_refuses_dynamic_conditional_and_wrong_signatures(self):
        source = self.modern_registry()
        variants = [
            source.replace('vec![include_str!("../scenarios/sample.json"),]', 'vec![]'),
            source.replace('texts.push(', 'texts.extend('),
            source.replace('texts.push(', 'if true { texts.push(').replace('texts\n}', 'texts }\n}'),
            source.replace('include_str!("../scenarios/second.json")', 'include_str!(PATH)'),
            source.replace('include_str!("../scenarios/second.json")',
                           'include_str!(concat!("../scenarios/", "second.json"))'),
            source.replace('include_str!("../scenarios/second.json")',
                           'include_str!(r"../scenarios/second.json")'),
            source.replace('fn accepted_texts()', '#[cfg(test)]\nfn accepted_texts()'),
            source.replace("fn accepted_texts() -> Vec<&'static str>", 'fn accepted_texts() -> String'),
            source.replace('accepted_texts()', 'unknown_texts()'),
            source.replace('    texts\n}', '    texts.clear(); texts\n}'),
            source.replace('    texts\n}', '    return texts;\n}'),
            source.replace('    texts\n}', '    load_catalog(&texts); texts\n}'),
            source.replace('    texts\n}', '    texts = bundled_texts(); texts\n}'),
            source.replace('pub fn ccna_pilot()', 'pub fn other_pilot()'),
            source + "\nfn accepted_texts() -> Vec<&'static str> { vec![] }\n",
        ]
        for value in variants:
            (self.root / 'src/pack.rs').write_text(value)
            with self.assertRaises(ValueError, msg=value):
                verify_coverage.validate(self.root, 'teaching-review')

    def test_modern_registry_bounds_cycles_depth_and_literal_paths(self):
        source = self.modern_registry()
        variants = [source.replace('let mut texts = bundled_texts();',
                                   'let mut texts = accepted_texts();')]
        variants += [source.replace('../scenarios/second.json', path) for path in
                     ('../scenarios/sample.json', '../scenarios/../escape.json',
                      '/tmp/escape.json', '../scenarios/missing.json')]
        chain = source.replace('load_catalog(&accepted_texts())', 'load_catalog(&accepted65_texts())')
        for n in range(1, 66):
            previous = 'accepted_texts' if n == 1 else f'accepted{n-1}_texts'
            chain += f"\nfn accepted{n}_texts() -> Vec<&'static str> {{ let mut texts = {previous}(); texts }}\n"
        variants.append(chain)
        for value in variants:
            (self.root / 'src/pack.rs').write_text(value)
            with self.assertRaises(ValueError):
                verify_coverage.validate(self.root, 'teaching-review')

    def test_modern_fake_headers_nested_functions_and_quoted_braces_remain_inert(self):
        source = self.modern_registry()
        fake = source.replace('sample.json', 'forged.json')
        quoted = 'const NOTE: &str = r###"' + fake + '"###;\n'
        nested = 'mod unrelated {\n' + source + '\n}\n'
        (self.root / 'src/pack.rs').write_text('/* outer /* nested */ } */\n' + quoted + nested + source)
        result = verify_coverage.validate(self.root, 'teaching-review')
        self.assertEqual(result['authored_labs'], 2)
        self.assertEqual(result['registered_pack_paths'],
                         ['scenarios/sample.json', 'scenarios/second.json'])

    def unit_document(self, filename, labs=None):
        document = {'version': 1, 'units': [{'id': 'sample-unit', 'track': 'CCNA',
                    'title': 'Sample unit', 'summary': 'Sample metadata only.',
                    'prerequisites': [], 'labs': labs or ['sample']}]}
        (self.root / 'scenarios' / filename).write_text(json.dumps(document))
        return document

    def test_unit_documents_are_bounded_reporting_metadata_not_scenario_packs(self):
        self.modern_registry(); self.unit_document('notes.json')
        result = verify_coverage.validate(self.root, 'teaching-review')
        self.assertEqual(result['unit_index_paths'], ['scenarios/notes.json'])
        self.assertEqual(result['authored_labs'], 2)
        source = (self.root / 'src/pack.rs').read_text().replace('second.json', 'notes.json')
        (self.root / 'src/pack.rs').write_text(source)
        with self.assertRaisesRegex(ValueError, 'Malformed scenario pack'):
            verify_coverage.validate(self.root, 'teaching-review')

    def test_unit_documents_refuse_unknown_versions_bad_fields_duplicates_and_aliases(self):
        self.modern_registry(); document = self.unit_document('notes.json')
        variants = []
        for key, value in [('version', True), ('version', 2), ('extra', 1), ('units', [])]:
            d = copy.deepcopy(document); d[key] = value; variants.append(d)
        for key, value in [('labs', []), ('labs', ['../escape']), ('labs', ['sample', 'sample']),
                           ('prerequisites', ['x'] * 65), ('title', 'x' * 161),
                           ('summary', ''), ('track', 'new-track'), ('id', 'bad id'),
                           ('extra', 'unsupported')]:
            d = copy.deepcopy(document); d['units'][0][key] = value; variants.append(d)
        d = copy.deepcopy(document); d['units'] *= 2; variants.append(d)
        path = self.root / 'scenarios/notes.json'
        for value in variants:
            path.write_text(json.dumps(value))
            with self.assertRaises(ValueError):
                verify_coverage.validate(self.root, 'teaching-review')
        path.write_text('{"version":1,"version":1,"units":[]}')
        with self.assertRaisesRegex(ValueError, 'Duplicate pack JSON key'):
            verify_coverage.validate(self.root, 'teaching-review')
        path.unlink(); path.symlink_to(self.root / 'scenarios/second.json')
        with self.assertRaisesRegex(ValueError, 'unsafe'):
            verify_coverage.validate(self.root, 'teaching-review')



class NetworkTransferConsolidationTests(unittest.TestCase):
    def test_current_consolidation_preserves_both_origins_and_rejects_changes(self):
        root=Path(__file__).resolve().parents[2]
        names=['scenarios/boot-entry-lessons-v39.json','scenarios/network-transfers-lessons-v1.json','scenarios/boot-entry-lessons-v40.json']
        packs={n:{s['id']:s for s in json.loads((root/n).read_text())['scenarios']}for n in names}
        self.assertEqual(list(packs[names[2]].values()),list(packs[names[0]].values())+list(packs[names[1]].values()))
        for origin in names[:2]:
            self.assertTrue(verify_coverage.preserved_authored_pack(origin,packs[origin],{names[2]:packs[names[2]]}))
        # Mutation cannot pass by sharing the same forged values in both maps.
        forged=copy.deepcopy(packs)
        identity=next(iter(forged[names[0]]))
        forged[names[0]][identity]['title']='Changed original objective'
        forged[names[2]][identity]['title']='Changed original objective'
        with self.assertRaisesRegex(ValueError,'exact predecessor'):
            verify_coverage.preserved_authored_pack(names[0],forged[names[0]],{names[2]:forged[names[2]]})
        for mutate in ['drop','reorder','tail']:
            current=copy.deepcopy(packs[names[2]])
            if mutate=='drop':current.pop(next(iter(current)))
            elif mutate=='reorder':current=dict(reversed(list(current.items())))
            else:current[next(reversed(current))]['title']='Unreviewed transfer'
            with self.assertRaisesRegex(ValueError,'exact predecessor'):
                verify_coverage.preserved_authored_pack(names[1],packs[names[1]],{names[2]:current})


if __name__ == '__main__':
    unittest.main()

class HistoricalOriginBudgetTests(unittest.TestCase):
    def test_only_exact_preserved_origins_are_exempt_and_real_drafts_keep_the_byte_cap(self):
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);(root/'scenarios').mkdir()
            # No large fixture allocation: exercise accounting with bounded fake
            # read receipts. Real reads retain the2MB _catalog_document guard.
            for i in range(17):(root/'scenarios'/f'copy-{i:02}.json').touch()
            def document(*args):
                name=args[1]
                return {'schema_version':1,'scenarios':[{'id':name}]},1_999_999
            def scenarios(doc,name):return {s['id']:s for s in doc['scenarios']}
            with patch.object(verify_coverage,'registered_pack_paths',return_value=[]),patch.object(verify_coverage,'_catalog_document',side_effect=document)as read,patch.object(verify_coverage,'_scenario_labs',side_effect=scenarios):
                with patch.object(verify_coverage,'preserved_authored_pack',return_value=True)as exact:
                    result=verify_coverage.catalog_inventory(root)
                    self.assertEqual(result[1],[]);self.assertEqual(exact.call_count,17)
                    self.assertTrue(all(c.args[2]==2_000_000 for c in read.call_args_list))
                with patch.object(verify_coverage,'preserved_authored_pack',return_value=False):
                    with self.assertRaisesRegex(ValueError,'Unregistered data byte capacity exceeded'):
                        verify_coverage.catalog_inventory(root)
                with patch.object(verify_coverage,'preserved_authored_pack',side_effect=ValueError('Changed preserved authored origin')):
                    with self.assertRaisesRegex(ValueError,'Changed preserved authored origin'):
                        verify_coverage.catalog_inventory(root)
