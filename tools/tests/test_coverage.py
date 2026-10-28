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


if __name__ == '__main__':
    unittest.main()
