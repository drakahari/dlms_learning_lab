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
spec = importlib.util.spec_from_file_location('before13_verify', TOOLS / 'scripting_before13_verify.py')
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)
spec = importlib.util.spec_from_file_location('before13_dev', TOOLS / 'dev.py')
dev = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dev)


class Genuine12Guards(unittest.TestCase):
    def fixture(self):
        directory = tempfile.TemporaryDirectory(); self.addCleanup(directory.cleanup)
        return Path(directory.name)

    def revision(self, number):
        root = self.fixture(); (root / 'src').mkdir()
        (root / 'src/shell.rs').write_text('pub const EVALUATOR_REVISION: u32 = ' + str(number) + ';\n')
        return root

    def test_revision12_requests_require_real12_and_explicit_feature(self):
        for flag in [['--features', 'historical-evaluator12-capture'],
                     ['--features=historical-evaluator12-capture'],
                     ['-Fhistorical-evaluator12-capture'], ['-F=historical-evaluator12-capture'],
                     ['-F', 'other,dlms-learning-lab/historical-evaluator12-capture']]:
            with self.subTest(flag=flag):
                dev.historical_capture_preflight(['--example', 'scripting_before13_capture', *flag], self.revision(12))
                for later in [11, 13]:
                    with self.assertRaisesRegex(RuntimeError, 'actual public evaluator12'):
                        dev.historical_capture_preflight(flag, self.revision(later))
        with self.assertRaisesRegex(RuntimeError, 'requires --features'):
            dev.historical_capture_preflight(['--example=scripting_before13_capture'], self.revision(12))

    def test_preparation_refuses_non12_ambiguous_and_missing_current_source_revision(self):
        v.current12(self.revision(12))
        for revision in [11, 13]:
            with self.assertRaises(v.safe.OracleError): v.current12(self.revision(revision))
        root = self.revision(12)
        source = root / 'src/shell.rs'
        for raw in [b'', b'pub const EVALUATOR_REVISION: u32 = 12;\npub const EVALUATOR_REVISION: u32 = 13;\n', b'x' * (512 * 1024 + 1)]:
            source.write_bytes(raw)
            with self.assertRaises(v.safe.OracleError): v.current12(root)

    def test_defaults_and_test_filters_do_not_open_historical_sources(self):
        with patch.object(Path, 'open', side_effect=AssertionError('No source read needed')):
            for flags in [['--all-targets'], ['--examples'], ['--all-targets', '--', 'historical-evaluator12-capture']]:
                dev.historical_capture_preflight(flags, Path('/synthetic-unused'))
        with self.assertRaises(RuntimeError):
            dev.historical_capture_preflight(['--all-features'], self.revision(12))

    def test_closed_json_and_paths_reject_duplicate_traversal_and_aliases(self):
        with self.assertRaises(v.safe.OracleError): v.strict('{"a":1,"a":2}')
        for name in ['', '/', '/tmp/file', '../file', './file', 'x//file', 'x/../file', 3]:
            with self.subTest(name=name), self.assertRaises(v.safe.OracleError):
                v.contained(Path('/synthetic'), name)
        self.assertEqual(v.contained(Path('/synthetic'), 'input/data'), Path('/synthetic/input/data'))

    def test_approval_requires_exact_root_reviewed_source_and_binary_pins(self):
        control = {'root_reviewed': False, 'pins': [{'sha256': 'synthetic'}], 'actual_evaluator_revision': 12}
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
        #52 actual prior scripts +90 unrelated contexts; only3 actually executed numeric-script fixtures added.
        names = [*v.DRAFTS, *['synthetic-old-script-' + str(n) for n in range(50)], *v.NEW]
        result = {'stdout': 'synthetic\n', 'stderr': '', 'status': 0, 'files': {}, 'error': None}
        case = {'stdout': 'synthetic\n', 'stderr': '', 'status': 0, 'after': {}}
        rows = [{'id': name, 'source': 'synthetic source only', 'cases': [case],
                 'record': {'engine_revision': 12, 'source': 'synthetic source only', 'results': [result]},
                 'example_source': 'synthetic separate practice', 'example_case': case,
                 'example_outcome': result, 'example_revision': 12} for name in names]
        def attempt(row, revision):
            return {'scenario_version': 1, 'state': {'kind': 'Script', 'source': row['source'],
                    'run': {**row['record'], 'engine_revision': revision}}, 'step': 2, 'hints': 1,
                    'challenge': True, 'transcript': ['synthetic transcript'], 'visited': 5,
                    'workspace': {'checked': True, 'script_learning': {'version': 1, 'view': 'Assignment',
                    'example_source': row['example_source'], 'example_revision': revision,
                    'example_result': result}, 'script_history': {**row['record'], 'engine_revision': 9}}}
        old = {'schema_version': 1, 'clock': 200, 'favorites': ['synthetic-old-script-0'],
               'migration_receipts': {'workspace': True, 'shell': [7, 8, 9, 10, 11]},
               'attempts': {r['id']: attempt(r, 11) for r in rows[:-3]}}
        for n in range(90): old['attempts']['synthetic-non-script-' + str(n)] = {'state': {'kind': 'Other'}, 'synthetic': n}
        if drafts:
            for name in v.DRAFTS:
                a = old['attempts'][name]; a['state']['source'] = 'synthetic unfinished assignment'; a['state']['run'] = None
                a['workspace']['script_learning'].update(example_source='synthetic unfinished practice', example_result=None)
        current = copy.deepcopy(old); current['migration_receipts']['shell'].append(12)
        current['clock'] += 53 if drafts else 55
        tick = old['clock']
        for row in rows:
            name = row['id']
            if drafts and name in v.DRAFTS: continue
            tick += 1
            if name in v.NEW:
                a = attempt(row, 12); a.update(step=0, hints=0, challenge=False, transcript=[], visited=tick)
                a['workspace'].pop('script_history'); current['attempts'][name] = a
            else:
                a = current['attempts'][name]
                a['workspace']['script_history_more'] = [copy.deepcopy(a['state']['run'])]
            a['state'] = {'kind': 'Script', 'source': row['source'], 'run': copy.deepcopy(row['record'])}
            a['workspace']['checked'] = False
            a['workspace']['script_learning'].update(example_revision=12, example_result=result)
        index = {'prior_profile_ids': sorted(old['attempts']),
                 'scripts': [{'id':r['id'],'version':1} for r in rows],
                 'non_script_absent_not_invented': ['synthetic-absent-' + str(n) for n in range(26)]}
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
        b = copy.deepcopy(a); b['state']['run']['engine_revision'] = 12
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
        manifest = {'actual_evaluator_revision': 12, 'catalog_contexts': 171, 'script_records': 55,
                    'authored_cases': 420, 'examples': 55, 'prior_actual11_manifest_sha256': v.PRIOR_SHA,
                    'source_index_sha256': v.INDEX_SHA,
                    'outputs': [{'file': 'payload.json', 'bytes': len(payload), 'sha256': v.sha(payload)}]}
        raw = v.canonical(manifest); pin = v.sha(raw)
        (folder / 'MANIFEST.json').write_bytes(raw); (folder / 'COMPLETE').write_text(pin + '\n')
        # Even an otherwise hash-consistent invented shape is not accepted.
        with self.assertRaisesRegex(v.safe.OracleError, 'closed12'):
            v.verify_capture(root, pin)
        (folder / 'payload.json').write_bytes(b'synthetic tamper')
        with self.assertRaisesRegex(v.safe.OracleError, 'output changed'):
            v.verify_capture(root, pin)
        (folder / 'COMPLETE').write_text('not complete')
        with self.assertRaises(v.safe.OracleError): v.verify_capture(root, pin)

    def test_source_index_is_independent_frozen_authored_input_not_current_implementation_counts(self):
        raw = (TOOLS.parent / v.INDEX).read_bytes(); index = v.index_value(raw)
        self.assertEqual(len(index['scripts']), 55)
        self.assertEqual(sum(len(s['cases']) for s in index['scripts']), 420)
        changed = copy.deepcopy(index); changed['authored_cases'] = 421
        with self.assertRaises(v.safe.OracleError): v.index_value(v.canonical(changed))

    def test_cli_is_closed_and_never_accepts_source_paths_or_engine_selection(self):
        with patch.object(v.safe, 'require_guard', side_effect=AssertionError('No child')):
            for args in [['--mode', 'capture', '--source', 'anything'], ['--mode', 'capture', '--output', '/tmp/x'],
                         ['--mode', 'capture', '--engine', '12'], ['--mo', 'capture']]:
                with self.subTest(args=args), self.assertRaises(SystemExit): v.main(args)


    def test_retained_dotcargo_config_is_exact_and_other_dotdirs_aliases_refuse(self):
        root = self.fixture()
        path = v.retained_parent(root, '.cargo/config.toml')
        self.assertEqual(path, root / '.cargo/config.toml')
        self.assertEqual((root / '.cargo').stat().st_mode & 0o777, 0o700)
        self.assertEqual(v.retained_parent(root, '.cargo/config.toml'), path)
        for name in ['.cargo/other.toml', 'src/.cargo/config.toml', '.other/config.toml', '.github/workflow.yml']:
            with self.subTest(name=name), self.assertRaises(v.safe.OracleError):
                v.retained_parent(root, name)
        other = self.fixture(); target = self.fixture(); (other / '.cargo').symlink_to(target)
        with self.assertRaises(v.safe.OracleError): v.retained_parent(other, '.cargo/config.toml')
        # The shared old helper remains strict; only the new collector has the
        # explicitly reviewed fixed .cargo/config.toml retention exception.
        with self.assertRaises(v.safe.OracleError): v.safe.private_dir(root, '.cargo', existing=True)

    def test_index_derivation_retains_authentic_prior_ids_and_only_absent_new_scripts(self):
        index = v.load_index(TOOLS.parent)
        self.assertEqual(len(index['prior_script_ids']), 52)
        self.assertEqual(len(index['prior_non_script_ids']), 90)
        self.assertEqual(len(set(index['prior_profile_ids']) | set(v.NEW)), 145)
        self.assertTrue(set(v.NEW).isdisjoint(index['prior_profile_ids']))
        self.assertEqual(index['retained_draft_ids'], v.DRAFTS)
        self.assertEqual(len(index['non_script_absent_not_invented']), 26)

    def test_complete_case_stream_retains_all_actual_record_outcomes_order_and_errors(self):
        outcome = {'stdout': 'synthetic', 'stderr': '', 'status': 0, 'files': {}, 'error': None}
        records=[]
        for n in range(55):
            size = 8 if n < 35 else 7
            records.append({'id': 'synthetic-' + str(n), 'record': {'results': [outcome] * size},
                            'example_outcome': outcome})
        self.assertEqual(sum(len(r['record']['results']) for r in records), 420)
        lines=[]
        for row in records:
            lines.extend({'id':row['id'],'role':'assignment','case':n,'outcome':o}
                         for n,o in enumerate(row['record']['results']))
            lines.append({'id':row['id'],'role':'example','case':0,'outcome':outcome})
        index={'scripts':[{'cases':[None]*len(r['record']['results'])} for r in records]}
        raw=lambda rows: b'\n'.join(v.canonical(row) for row in rows)+b'\n'
        v.case_observations(raw(lines),index,records)
        for changed in [lines[:-1], list(reversed(lines)), [*lines,lines[0]]]:
            with self.assertRaises(v.safe.OracleError):v.case_observations(raw(changed),index,records)
        bad=copy.deepcopy(lines);bad[0]['outcome']['error']='synthetic unobserved failure'
        with self.assertRaises(v.safe.OracleError):v.case_observations(raw(bad),index,records)


    def test_new_numeric_context_cannot_claim_foreign_workspace_or_wrong_source_version(self):
        old,current,index,rows=self.synthetic_profiles()
        for field in ['one_off_session','physical_link_draft','file_backup_draft']:
            bad=copy.deepcopy(current);bad['attempts'][v.NEW[0]]['workspace'][field]={}
            with self.subTest(field=field),self.assertRaises(v.safe.OracleError):
                v.profile(old,bad,index,rows,False)
        bad=copy.deepcopy(current);bad['attempts'][v.NEW[0]]['scenario_version']=99
        with self.assertRaises(v.safe.OracleError):v.profile(old,bad,index,rows,False)


    def retained_fixture(self):
        # Synthetic source retention only: no saved learner progress, producer,
        # capture/attestation seal, VM outcome or subprocess is constructed.
        root = self.fixture()
        current = root / v.INDEX; current.parent.mkdir(parents=True)
        raw_index = (TOOLS.parent / v.INDEX).read_bytes(); current.write_bytes(raw_index)
        folder = root / v.DESTINATION; folder.mkdir(parents=True)
        return root, folder, raw_index

    def test_final_retention_refuses_entire_missing_tree_dangling_alias_and_file(self):
        root, folder, raw_index = self.retained_fixture()
        # The exact root index exists: this is the precise former false fallback.
        self.assertEqual(v.index_value(raw_index), v.source_index(root, folder))
        with self.assertRaisesRegex(v.safe.OracleError, 'compiled-source directory is required'):
            v.source_index(root, folder, require_retained=True)
        frozen = folder / 'compiled-source'
        frozen.symlink_to(folder / 'nonexistent-retained', target_is_directory=True)
        for required in [False, True]:
            with self.assertRaisesRegex(v.safe.OracleError, 'ordinary directory'):
                v.source_index(root, folder, require_retained=required)
        frozen.rename(folder / 'synthetic-dangling-alias')
        frozen.write_bytes(b'synthetic regular file is not a retained tree')
        with self.assertRaisesRegex(v.safe.OracleError, 'ordinary directory'):
            v.source_index(root, folder, require_retained=True)

    def test_complete_retained_inputs_use_frozen_index_even_without_current_root_index(self):
        root, folder, raw_index = self.retained_fixture()
        frozen = folder / 'compiled-source'; frozen.mkdir(mode=0o700)
        index_path = frozen / v.INDEX; index_path.parent.mkdir(parents=True)
        index_path.write_bytes(raw_index)
        data = frozen / 'synthetic-source.rs'; data.write_bytes(b'fixed synthetic source bytes')
        inputs = [{'file':v.INDEX, 'bytes':len(raw_index), 'sha256':v.sha(raw_index)},
                  {'file':'synthetic-source.rs', 'bytes':data.stat().st_size, 'sha256':v.sha(data.read_bytes())}]
        (folder / 'SOURCE.json').write_text(json.dumps({'actual_evaluator_revision':12,
            'source_index_sha256':v.INDEX_SHA, 'inputs':inputs}))
        expected = v.index_value(raw_index)
        self.assertEqual(v.source_index(root, folder, require_retained=True), expected)
        (root / v.INDEX).rename(root / 'synthetic-current-index-moved')
        self.assertEqual(v.source_index(root, folder, require_retained=True), expected)
        data.write_bytes(b'tampered retained source')
        with self.assertRaisesRegex(v.safe.OracleError, 'Frozen source changed'):
            v.source_index(root, folder, require_retained=True)
        data.rename(frozen / 'synthetic-source-moved.rs')
        with self.assertRaises((v.safe.OracleError, OSError)):
            v.source_index(root, folder, require_retained=True)
        frozen.rename(folder / 'synthetic-entire-tree-moved')
        with self.assertRaisesRegex(v.safe.OracleError, 'compiled-source directory is required'):
            v.source_index(root, folder, require_retained=True)

    def test_final_verify_route_explicitly_requires_retention_without_launching_child(self):
        root = self.fixture(); (root / '.git').mkdir()
        with patch.multiple(v, ROOT=root, SEALED_MANIFEST='a'*64, SEALED_ATTESTATION='b'*64), \
             patch.dict(v.os.environ, {'CARGO_TARGET_DIR':str(root/'target')}), \
             patch.object(v.safe, 'require_guard', return_value=()), \
             patch.object(v, 'verify_capture', side_effect=AssertionError('stop before any synthetic attestation')) as checked:
            with self.assertRaisesRegex(AssertionError, 'stop before'):
                v.main(['--mode','verify'])
            checked.assert_called_once_with(root,'a'*64,require_retained=True)
