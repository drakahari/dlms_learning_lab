"""Prospective synthetic guards. No actual VM/progress/capture child is run.

Ordinary runner sets TMPDIR on project disk before these temporary fixtures.
Names/data below are explicitly synthetic, never historical observations.
"""
import copy
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
spec = importlib.util.spec_from_file_location('before12_verify', TOOLS / 'scripting_before12_verify.py')
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)
spec = importlib.util.spec_from_file_location('before12_dev', TOOLS / 'dev.py')
dev = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dev)


class Genuine11Guards(unittest.TestCase):
    def fixture(self):
        directory = tempfile.TemporaryDirectory(); self.addCleanup(directory.cleanup)
        return Path(directory.name)

    def revision(self, number):
        root = self.fixture(); (root / 'src').mkdir()
        (root / 'src/shell.rs').write_text('pub const EVALUATOR_REVISION: u32 = ' + str(number) + ';\n')
        return root

    def test_revision11_requests_require_real11_and_explicit_feature(self):
        for flag in [['--features', 'historical-evaluator11-capture'],
                     ['--features=historical-evaluator11-capture'],
                     ['-Fhistorical-evaluator11-capture'], ['-F=historical-evaluator11-capture'],
                     ['-F', 'other,dlms-learning-lab/historical-evaluator11-capture']]:
            with self.subTest(flag=flag):
                dev.historical_capture_preflight(['--example', 'scripting_before12_capture', *flag], self.revision(11))
                for later in [10, 12]:
                    with self.assertRaisesRegex(RuntimeError, 'actual public evaluator11'):
                        dev.historical_capture_preflight(flag, self.revision(later))
        with self.assertRaisesRegex(RuntimeError, 'requires --features'):
            dev.historical_capture_preflight(['--example=scripting_before12_capture'], self.revision(11))

    def test_preparation_refuses_non11_ambiguous_and_missing_current_source_revision(self):
        v.current11(self.revision(11))
        for revision in [10, 12]:
            with self.assertRaises(v.safe.OracleError): v.current11(self.revision(revision))
        root = self.revision(11)
        source = root / 'src/shell.rs'
        for raw in [b'', b'pub const EVALUATOR_REVISION: u32 = 11;\npub const EVALUATOR_REVISION: u32 = 12;\n', b'x' * (512 * 1024 + 1)]:
            source.write_bytes(raw)
            with self.assertRaises(v.safe.OracleError): v.current11(root)

    def test_defaults_and_test_filters_do_not_open_historical_sources(self):
        with patch.object(Path, 'open', side_effect=AssertionError('No source read needed')):
            for flags in [['--all-targets'], ['--examples'], ['--all-targets', '--', 'historical-evaluator11-capture']]:
                dev.historical_capture_preflight(flags, Path('/synthetic-unused'))
        with self.assertRaises(RuntimeError):
            dev.historical_capture_preflight(['--all-features'], self.revision(11))

    def test_closed_json_and_paths_reject_duplicate_traversal_and_aliases(self):
        with self.assertRaises(v.safe.OracleError): v.strict('{"a":1,"a":2}')
        for name in ['', '/', '/tmp/file', '../file', './file', 'x//file', 'x/../file', 3]:
            with self.subTest(name=name), self.assertRaises(v.safe.OracleError):
                v.contained(Path('/synthetic'), name)
        self.assertEqual(v.contained(Path('/synthetic'), 'input/data'), Path('/synthetic/input/data'))

    def test_approval_requires_exact_root_reviewed_source_and_binary_pins(self):
        control = {'root_reviewed': False, 'pins': [{'sha256': 'synthetic'}], 'actual_evaluator_revision': 11}
        v.approved(json.dumps({**control, 'root_reviewed': True}), control)
        for value in [control, {**control, 'root_reviewed': 1},
                      {**control, 'root_reviewed': True, 'extra': 'not approved'},
                      {**control, 'root_reviewed': True, 'pins': []}]:
            with self.subTest(value=value), self.assertRaises(v.safe.OracleError):
                v.approved(json.dumps(value), control)

    def test_exact_outcome_rejects_errors_stderr_wrong_status_and_changed_files(self):
        case = {'stdout': 'synthetic\n', 'stderr': '', 'status': 0, 'after': {'data': 'kept'}}
        outcome = {'stdout': 'synthetic\n', 'stderr': '', 'status': 0, 'files': {'data': 'kept'}, 'error': None}
        self.assertTrue(v.outcome(case, outcome))
        for key, bad in [('error', 'Unsupported'), ('stderr', 'wrong'), ('status', 1),
                         ('status', False), ('files', {}), ('stdout', 'different')]:
            self.assertFalse(v.outcome(case, {**outcome, key: bad}), key)
        self.assertFalse(v.outcome(case, {**outcome, 'unexpected': True}))

    def synthetic_profiles(self, drafts=False):
        #49 old scripts +90 old unrelated contexts; only3 executed scripts added.
        names = [*v.DRAFTS, *['synthetic-old-script-' + str(n) for n in range(47)], *v.NEW]
        result = {'stdout': 'synthetic\n', 'stderr': '', 'status': 0, 'files': {}, 'error': None}
        case = {'stdout': 'synthetic\n', 'stderr': '', 'status': 0, 'after': {}}
        rows = [{'id': name, 'source': 'synthetic source only', 'cases': [case],
                 'record': {'engine_revision': 11, 'source': 'synthetic source only', 'results': [result]},
                 'example_source': 'synthetic separate practice', 'example_case': case,
                 'example_outcome': result, 'example_revision': 11} for name in names]
        def attempt(row, revision):
            return {'scenario_version': 1, 'state': {'kind': 'Script', 'source': row['source'],
                    'run': {**row['record'], 'engine_revision': revision}}, 'step': 2, 'hints': 1,
                    'challenge': True, 'transcript': ['synthetic transcript'], 'visited': 5,
                    'workspace': {'checked': True, 'script_learning': {'version': 1, 'view': 'Assignment',
                    'example_source': row['example_source'], 'example_revision': revision,
                    'example_result': result}, 'script_history': {**row['record'], 'engine_revision': 9}}}
        old = {'schema_version': 1, 'clock': 200, 'favorites': ['synthetic-old-script-0'],
               'migration_receipts': {'workspace': True, 'shell': [7, 8, 9, 10]},
               'attempts': {r['id']: attempt(r, 10) for r in rows[:-3]}}
        for n in range(90): old['attempts']['synthetic-non-script-' + str(n)] = {'state': {'kind': 'Other'}, 'synthetic': n}
        if drafts:
            for name in v.DRAFTS:
                a = old['attempts'][name]; a['state']['source'] = 'synthetic unfinished assignment'; a['state']['run'] = None
                a['workspace']['script_learning'].update(example_source='synthetic unfinished practice', example_result=None)
        current = copy.deepcopy(old); current['migration_receipts']['shell'].append(11)
        current['clock'] += 50 if drafts else 52
        tick = old['clock']
        for row in rows:
            name = row['id']
            if drafts and name in v.DRAFTS: continue
            tick += 1
            if name in v.NEW:
                a = attempt(row, 11); a.update(step=0, hints=0, challenge=False, transcript=[], visited=tick)
                a['workspace'].pop('script_history'); current['attempts'][name] = a
            else:
                a = current['attempts'][name]
                a['workspace']['script_history_more'] = [copy.deepcopy(a['state']['run'])]
            a['state'] = {'kind': 'Script', 'source': row['source'], 'run': copy.deepcopy(row['record'])}
            a['workspace']['checked'] = False
            a['workspace']['script_learning'].update(example_revision=11, example_result=result)
        index = {'prior_profile_ids': sorted(old['attempts']),
                 'non_script_absent_not_invented': ['synthetic-absent-' + str(n) for n in range(22)]}
        return old, current, index, rows

    def test_actual_execution_only_profile_additions_preserve_original_records_and_metadata(self):
        for drafts in [False, True]:
            old, current, index, rows = self.synthetic_profiles(drafts)
            v.profile(old, current, index, rows, drafts)
            for mutate in [lambda p: p['attempts'].pop(v.NEW[0]),
                           lambda p: p['attempts'].update({'invented-cron-answer': {}}),
                           lambda p: p['attempts'][v.NEW[0]].update(hints=2),
                           lambda p: p['attempts']['synthetic-old-script-0']['workspace'].update(script_history_more=[]),
                           lambda p: p['attempts']['synthetic-non-script-0'].update(synthetic=999),
                           lambda p: p['attempts']['synthetic-old-script-0'].update(step=99),
                           lambda p: p.update(clock=999),
                           lambda p: p.update(migration_receipts={'workspace': True, 'shell': [11]})]:
                bad = copy.deepcopy(current); mutate(bad)
                with self.assertRaises(v.safe.OracleError): v.profile(old, bad, index, rows, drafts)

    def test_authentic_assignment_and_practice_drafts_must_remain_entirely_unchanged(self):
        old, current, index, rows = self.synthetic_profiles(True)
        v.profile(old, current, index, rows, True)
        for name in v.DRAFTS:
            bad = copy.deepcopy(current); bad['attempts'][name]['workspace']['script_learning']['example_result'] = rows[0]['example_outcome']
            with self.assertRaises(v.safe.OracleError): v.profile(old, bad, index, rows, True)
            bad = copy.deepcopy(current); bad['attempts'][name]['state']['run'] = rows[0]['record']
            with self.assertRaises(v.safe.OracleError): v.profile(old, bad, index, rows, True)

    def test_record_comparison_keeps_original_revision_source_results_and_duplicates(self):
        a = {'state': {'run': {'engine_revision': 10, 'source': 's', 'results': []}},
             'workspace': {'script_history': {'engine_revision': 9, 'source': 's', 'results': []},
                           'script_history_more': [{'engine_revision': 9, 'source': 's', 'results': []}]}}
        self.assertEqual(sum(v.records(a, True).values()), 3)
        self.assertEqual(sum(v.records(a, False).values()), 2)
        b = copy.deepcopy(a); b['state']['run']['engine_revision'] = 11
        self.assertNotEqual(v.records(a, True), v.records(b, True))

    def test_symlink_and_size_bounds_precede_evidence_reads(self):
        root = self.fixture(); file = root / 'file'; file.write_bytes(b'synthetic')
        (root / 'alias').symlink_to(file)
        for path, cap in [(root / 'alias', 100), (file, 1)]:
            with self.assertRaises((v.safe.OracleError, OSError)): v.safe.read_bytes(path, cap)

    def test_unset_or_incomplete_seal_pair_cannot_authorize_verification(self):
        # Keep this negative meaningful after a genuine pair is eventually
        # reviewed/sealed; a test must never demand erasing accepted evidence.
        for manifest, attestation in [(None, None), ('a' * 64, None), (None, 'b' * 64), ('guessed', 'b' * 64)]:
            with patch.multiple(v, SEALED_MANIFEST=manifest, SEALED_ATTESTATION=attestation):
                with self.assertRaises(v.safe.OracleError): v.require_seals()
        with patch.multiple(v, SEALED_MANIFEST='a' * 64, SEALED_ATTESTATION='b' * 64):
            v.require_seals()  # Shape only: no evidence or verification is invented.
        with self.assertRaises(v.safe.OracleError): v.verify_capture(Path('/synthetic-unused'), None)

    def test_capture_manifest_completion_and_output_byte_changes_fail_before_profile_reads(self):
        root = self.fixture(); folder = root / v.DESTINATION; folder.mkdir(parents=True)
        payload = b'synthetic output only'; (folder / 'payload.json').write_bytes(payload)
        manifest = {'actual_evaluator_revision': 11, 'catalog_contexts': 164, 'script_records': 52,
                    'authored_cases': 384, 'examples': 52, 'prior_actual10_manifest_sha256': v.PRIOR_SHA,
                    'source_index_sha256': v.INDEX_SHA,
                    'outputs': [{'file': 'payload.json', 'bytes': len(payload), 'sha256': v.sha(payload)}]}
        raw = v.canonical(manifest); pin = v.sha(raw)
        (folder / 'MANIFEST.json').write_bytes(raw); (folder / 'COMPLETE').write_text(pin + '\n')
        # Even an otherwise hash-consistent invented shape is not accepted.
        with self.assertRaisesRegex(v.safe.OracleError, 'closed10'):
            v.verify_capture(root, pin)
        (folder / 'payload.json').write_bytes(b'synthetic tamper')
        with self.assertRaisesRegex(v.safe.OracleError, 'output changed'):
            v.verify_capture(root, pin)
        (folder / 'COMPLETE').write_text('not complete')
        with self.assertRaises(v.safe.OracleError): v.verify_capture(root, pin)

    def test_source_index_is_independent_frozen_authored_input_not_current_implementation_counts(self):
        raw = (TOOLS.parent / v.INDEX).read_bytes(); index = v.index_value(raw)
        self.assertEqual(len(index['scripts']), 52)
        self.assertEqual(sum(len(s['cases']) for s in index['scripts']), 384)
        changed = copy.deepcopy(index); changed['authored_cases'] = 385
        with self.assertRaises(v.safe.OracleError): v.index_value(v.canonical(changed))

    def test_cli_is_closed_and_never_accepts_source_paths_or_engine_selection(self):
        with patch.object(v.safe, 'require_guard', side_effect=AssertionError('No child')):
            for args in [['--mode', 'capture', '--source', 'anything'], ['--mode', 'capture', '--output', '/tmp/x'],
                         ['--mode', 'capture', '--engine', '12'], ['--mo', 'capture']]:
                with self.subTest(args=args), self.assertRaises(SystemExit): v.main(args)
