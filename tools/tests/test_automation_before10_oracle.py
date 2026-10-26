"""Safety tests for fixed automation collector. Never execute Bash, a VM or producer.

All fake output rows below are ephemeral validator inputs, not observed evidence.
Source/progress fixtures remain immutable; no installed learner path is accessed.
"""
from contextlib import contextmanager
from types import SimpleNamespace
import hashlib
import json
import os
import stat
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import automation_before10_oracle as oracle


@unittest.skipUnless(sys.platform.startswith('linux'), 'fixed Linux developer oracle')
class FixedAutomationSafety(unittest.TestCase):
    def test_actual_pin_aggregate_rechecked_after_preflight_size_growth(self):
        @contextmanager
        def small_first_pass(*_):
            yield None, SimpleNamespace(st_size=2)
        paths = [(Path('/fixed/one'), 8), (Path('/fixed/two'), 8)]
        with patch.object(oracle, 'TOTAL_PIN_CAP', 10), patch.object(oracle.safe, 'ordinary', small_first_pass):
            with patch.object(oracle.safe, 'pin', side_effect=[{'bytes': 6}, {'bytes': 6}]) as actual:
                with self.assertRaisesRegex(oracle.safe.OracleError, 'Actual aggregate'):
                    oracle.pins(paths)
                self.assertEqual(actual.call_count, 2)
            with patch.object(oracle.safe, 'pin', side_effect=[{'bytes': 4}, {'bytes': 6}]):
                self.assertEqual(oracle.pins(paths), [{'bytes': 4}, {'bytes': 6}])

    def test_fixed28_inputs_are_unobserved_and_original37_unchanged(self):
        _, rows = oracle.load_fixed(ROOT)
        self.assertEqual(len(rows), 28)
        self.assertEqual(hashlib.sha256(oracle.safe.read_bytes(ROOT / oracle.safe.FIXTURE, oracle.SOURCE_CAP)).hexdigest(), oracle.safe.FIXTURE_SHA)
        self.assertTrue(all(x['actual_public9_before10'] is None for x in rows))

    def test_changed_fixed_source_rejected_before_environment_or_child(self):
        data = oracle.safe.read_bytes(ROOT / oracle.FIXED, oracle.SOURCE_CAP) + b' '
        with patch.object(oracle.safe, 'read_bytes', return_value=data), patch.object(oracle.safe, 'capture_command') as child:
            with self.assertRaisesRegex(oracle.safe.OracleError, 'fixed28'): oracle.load_fixed(ROOT)
            child.assert_not_called()

    def test_duplicate_json_keys_are_rejected(self):
        with self.assertRaisesRegex(oracle.safe.OracleError, 'Duplicate'): oracle.strict_json(b'{"a":1,"a":2}')

    def test_capture_id_cannot_escape_or_accept_source(self):
        for name in ('../x', '/x', 'x/y', '', 'x' * 65, 'x;printf'):
            with self.assertRaises(oracle.safe.OracleError): oracle.capture_id(name)
        self.assertEqual(oracle.capture_id('before9-fixed-r1'), 'before9-fixed-r1')

    def test_parser_rejects_arbitrary_source_profile_binary_and_abbreviation(self):
        with patch.object(oracle, 'fixed_root_guard') as guard:
            for flags in (['--source', 'printf x'], ['--binary', '/bin/x'], ['--profile', '/tmp/x'], ['--capture', 'x'], ['--prepare-id', 'x', '--capture-id', 'y']):
                with self.assertRaises(SystemExit): oracle.main(flags)
            guard.assert_not_called()

    def test_missing_guard_prevents_preparation_capture_and_subprocesses(self):
        with patch.object(oracle, 'fixed_root_guard', side_effect=oracle.safe.OracleError('no guard')), patch.object(oracle, 'capture') as capture, patch.object(oracle, 'prepare') as prepare, patch.object(oracle.safe, 'capture_command') as child:
            with self.assertRaises(oracle.safe.OracleError): oracle.main(['--capture-id', 'x'])
            with self.assertRaises(oracle.safe.OracleError): oracle.main(['--prepare-id', 'x'])
            capture.assert_not_called(); prepare.assert_not_called(); child.assert_not_called()

    def test_worktree_and_target_override_are_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); (root / '.git').write_text('gitdir: other')
            with patch.object(oracle, 'ROOT', root), patch.object(oracle.safe, 'require_guard') as guard:
                with self.assertRaises(oracle.safe.OracleError): oracle.fixed_root_guard(root)
                guard.assert_not_called()
            (root / '.git').unlink(); (root / '.git').mkdir()
            with patch.object(oracle, 'ROOT', root), patch.object(oracle.safe, 'require_guard', return_value=(4,)), patch.dict(os.environ, {'CARGO_TARGET_DIR': str(root / 'other')}):
                with self.assertRaisesRegex(oracle.safe.OracleError, 'default target'): oracle.fixed_root_guard(root)

    def test_control_must_be_explicitly_reviewed_and_match_id_head_pins(self):
        control = {'schema_version': 1, 'root_reviewed': True, 'capture_id': 'x', 'source_head': 'a' * 40, 'pins': [], 'claims': 'metadata'}
        with patch.object(oracle, 'review_dir', return_value=Path('/fixed')), patch.object(oracle.safe, 'source_identity', return_value='a' * 40), patch.object(oracle, 'pins', return_value=[]):
            for key, val in [('root_reviewed', False), ('capture_id', 'y'), ('source_head', 'b' * 40), ('pins', [{'fake': True}])]:
                invalid = {**control, key: val}
                with patch.object(oracle.safe, 'read_bytes', return_value=json.dumps(invalid).encode()):
                    with self.assertRaises(oracle.safe.OracleError): oracle.approval(Path('/fixed'), 'x', [])
            with patch.object(oracle.safe, 'read_bytes', return_value=json.dumps(control).encode()):
                self.assertEqual(oracle.approval(Path('/fixed'), 'x', [])[1], control)

    def test_pinned_executable_reuses_same_fd_and_rejects_changed_pin(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / 'not-executed'; p.write_bytes(b'\x7fELFfixed-source-test')
            pin = oracle.safe.pin(p, 64)
            with oracle.pinned_executable(p, pin, 64) as (name, fd):
                self.assertEqual(name, '/proc/self/fd/' + str(fd))
                self.assertEqual(os.read(fd, 4), b'\x7fELF')
            with self.assertRaises(oracle.safe.OracleError):
                with oracle.pinned_executable(p, {**pin, 'sha256': '0' * 64}, 64): pass

    def test_executable_fifo_symlink_and_size_cannot_be_opened_as_program(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp) / 'file'; p.write_bytes(b'1234')
            link = Path(tmp) / 'alias'; link.symlink_to(p)
            for target, cap in ((link, 64), (p, 2)):
                with self.assertRaises((oracle.safe.OracleError, OSError)):
                    with oracle.pinned_executable(target, {}, cap): pass
            fifo = Path(tmp) / 'fifo'; os.mkfifo(fifo)
            with self.assertRaises(oracle.safe.OracleError):
                with oracle.pinned_executable(fifo, {}, 64): pass

    def test_vm_environment_omits_home_and_unrelated_host_variables(self):
        with patch.dict(os.environ, {'HOME': '/private', 'BASH_ENV': '/private/start', 'UNRELATED': 'secret'}):
            e = oracle.vm_environment(Path('/project'), Path('/temporary'), Path('/profile'))
        self.assertNotIn('HOME', e); self.assertNotIn('BASH_ENV', e); self.assertNotIn('UNRELATED', e)
        self.assertEqual(e['PATH'], ''); self.assertEqual(e['CARGO_TARGET_DIR'], '/project/target')

    def test_compiled_identity_rejects_stale_library_probe_or_catalog_bytes(self):
        data = {'kind': 'identity', 'revision': 9, 'compiled_manifest_dir': '/project',
                'compiled_inputs': [{'file': 'src/shell.rs', 'bytes': 1, 'sha256': 'a'}],
                'linked_catalog': {'fixed': 1}, 'linked_scripts': [],
                'probe_source_sha256': 'b', 'digest_source_sha256': 'c', 'attestation_source_sha256': 'd'}
        pins = [{'path': '/project/' + n, 'bytes': 1, 'sha256': h} for n, h in [('src/shell.rs', 'a'), (oracle.SOURCE, 'b'), (oracle.DIGEST, 'c'), (oracle.ATTEST, 'd')]]
        with patch.object(oracle, 'expected_inputs', return_value={'compiled_files': ['src/shell.rs']}), patch.object(oracle, 'authored_inputs', return_value=(b'', {'catalog': {'fixed': 1}, 'scripts': []})):
            self.assertEqual(oracle.identity_rows(Path('/project'), json.dumps(data), pins), data)
            for key, val in [('revision', 8), ('compiled_manifest_dir', '/other'), ('compiled_inputs', []), ('linked_catalog', {}), ('probe_source_sha256', 'z')]:
                with self.assertRaises(oracle.safe.OracleError): oracle.identity_rows(Path('/project'), json.dumps({**data, key: val}), pins)

    def test_vm_truncated_missing_or_duplicate_rows_cannot_finish(self):
        for text in ('', '{}', '{}\n{}', '{"kind":"identity"}\n{"kind":"finished"}'):
            with self.assertRaises((oracle.safe.OracleError, KeyError, ValueError)):
                oracle.verify_vm(ROOT, text, {}, {'scripts': [], 'catalog':{}}, [], [])

    def test_profile_symlink_and_oversize_rejected_before_read(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp); external = root / 'external'; external.write_bytes(b'x')
            folder = root / 'profile'; folder.mkdir(); (folder / 'link').symlink_to(external)
            with self.assertRaises(oracle.safe.OracleError): oracle.owned_outputs(folder, oracle.safe.Budget())
            (folder / 'link').unlink(); large = folder / 'too-large'
            with large.open('wb') as stream: stream.truncate(oracle.PROFILE_CAP + 1)
            with self.assertRaises(oracle.safe.OracleError): oracle.owned_outputs(folder, oracle.safe.Budget())

    def test_publication_never_clobbers_complete_and_precommit_failure_has_no_complete(self):
        with tempfile.TemporaryDirectory() as tmp:
            parent = Path(tmp); out = parent / 'result'; out.mkdir(); (out / 'COMPLETE').write_bytes(b'retained')
            with self.assertRaises(FileExistsError): oracle.safe.publish(out, {'test_only': True}, oracle.safe.Budget())
            self.assertEqual((out / 'COMPLETE').read_bytes(), b'retained')
            fail = parent / 'failure'; fail.mkdir()
            with patch.object(oracle.safe, 'sync_dir', side_effect=OSError('disk failure')):
                with self.assertRaises(OSError): oracle.safe.publish(fail, {}, oracle.safe.Budget())
            self.assertFalse((fail / 'COMPLETE').exists())

    def test_metadata_prepare_has_no_child_launch_and_requires_fresh_proposal(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            before = [{'path': str(oracle.safe.MANUAL), 'bytes': 1, 'sha256': oracle.safe.MANUAL_SHA}]
            with patch.object(oracle, 'load_fixed'), patch.object(oracle, 'load_vm'), patch.object(oracle, 'source_set', return_value=[]), patch.object(oracle, 'pins', return_value=before), patch.object(oracle, 'review_dir', return_value=p), patch.object(oracle.safe, 'source_identity', return_value='a' * 40), patch.object(oracle.safe, 'capture_command') as child:
                result = oracle.prepare(p, 'fixed')
                self.assertEqual(result['status'], 'PROPOSED-ONLY')
                self.assertFalse(json.loads((p / 'PROPOSED-fixed.json').read_bytes())['root_reviewed'])
                with self.assertRaises(FileExistsError): oracle.prepare(p, 'fixed')
                child.assert_not_called()



    def test_vm_only_boundaries_cannot_be_forwarded_to_bash(self):
        _, rows = oracle.load_vm(ROOT)
        self.assertEqual(len(rows), 8)
        self.assertTrue(all(r['run_bash'] is False and r['actual_public9_before10'] is None for r in rows))
        self.assertEqual(next(r for r in rows if r['id']=='pending-live-memory-limit')['input']['env']['pad'], 'x'*100000)
        raw = oracle.safe.read_bytes(ROOT / oracle.VM_BOUNDARIES, oracle.SOURCE_CAP)+b' '
        with patch.object(oracle.safe, 'read_bytes', return_value=raw), patch.object(oracle.safe,'capture_command') as child:
            with self.assertRaisesRegex(oracle.safe.OracleError,'VM-only inputs changed'): oracle.load_vm(ROOT)
            child.assert_not_called()

    def test_fixed_finite_stdin_preserves_whitespace_backslash_cr_utf8_bytes(self):
        _, rows = oracle.load_fixed(ROOT); by_id={r['id']:r for r in rows}
        self.assertEqual(by_id['read-leading-trailing']['input']['stdin'].encode(), b'  alpha\t \n')
        self.assertEqual(by_id['read-cr-utf8']['input']['stdin'].encode(), ' é\r\n'.encode())
        self.assertEqual(by_id['read-partial-final']['input']['stdin'], '  final\t ')
        self.assertTrue(all(r['input']['env']=={} for r in rows))
        self.assertFalse(any('expected' in r for r in rows))
        # Closed fixture source has ordinary quotes and a single printf escape.
        self.assertIn('"$row"', by_id['read-leading-trailing']['source'])
        self.assertNotIn('\\"$row', by_id['read-leading-trailing']['source'])

    def test_all43_authored_inputs_are_exact_current_pack_metadata(self):
        _, data=oracle.authored_inputs(ROOT)
        self.assertEqual(len(data['scripts']),43)
        self.assertEqual(sum(len(s['cases']) for s in data['scripts']),282)
        raw_packs={}
        for name in oracle.expected_inputs(ROOT)['compiled_files']:
            if name.startswith('scenarios/') and name!='scenarios/units-v1.json':
                pack=json.loads((ROOT/name).read_bytes())
                for s in pack['scenarios']: raw_packs[s['id']]=s
        for s in data['scripts']:
            p=raw_packs[s['id']]
            self.assertEqual(p['solution'],[s['source']])
            self.assertEqual(p['goal']['cases'],s['cases'])
            self.assertEqual(p['teaching']['example'],s['example_source'])
            self.assertEqual(p['teaching']['fixture'],s['example_case'])
        self.assertEqual(data['catalog'],{n:s['version'] for n,s in raw_packs.items()})
        units=json.loads((ROOT/'scenarios/units-v1.json').read_bytes())
        self.assertEqual(units['version'],1)
        self.assertEqual(len(units['units']),33)

    def test_main_portable_projection_removes_only_receipts_and_retains_all_other_keys(self):
        main={'schema_version':1,'favorites':['fixed'],'clock':3,'attempts':{'fixed':{'draft':'unfinished'}},'migration_receipts':{'shell':[7,8,9]}}
        expected={k:v for k,v in main.items() if k!='migration_receipts'}
        self.assertEqual(oracle.portable_projection(main),expected)
        self.assertIn('migration_receipts',main)
        for k in ('clock','favorites','attempts','schema_version'):
            changed=dict(expected);changed[k]=None
            self.assertNotEqual(oracle.portable_projection(main),changed)
        with self.assertRaises(oracle.safe.OracleError):oracle.portable_projection(expected)

    def test_real8_fixture_has_exact6_7_8_history_and_unfinished_separate_examples(self):
        old=json.loads((ROOT/'tests/fixtures/shell-v8-synthetic-writer-progress.json').read_bytes())
        counts={6:0,7:0,8:0}
        for a in old['attempts'].values():
            if a['state']['kind']!='Script':continue
            records=oracle.historical(a['workspace'])+[a['state']['run']]
            for r in records:counts[r['engine_revision']]+=1
        self.assertEqual(counts,{6:33,7:37,8:40})
        self.assertEqual(old['migration_receipts']['shell'],[7,8])
        for name in ('shell-arguments','shell-capture-preview'):
            l=old['attempts'][name]['workspace']['script_learning']
            self.assertEqual(l['example_revision'],7)
            self.assertIn('unfinished example',l['example_source'])

    def test_profile_summary_cannot_be_relabelled_or_claim_future_capability(self):
        row=oracle.profile_evidence(133)
        self.assertEqual(row['writer_receipts'],[7,8,9])
        self.assertTrue(row['future10_not_implemented'])
        self.assertEqual(row['genuine8_history_records'],40)
        self.assertEqual(row['current9_actual_records'],43)
        self.assertEqual(len(row['ungraded_example_drafts']),2)

    def test_recheck_projection_is_exact_current_result_only_and_preserves_history(self):
        # Ephemeral validator values, never actual observations or retained credit.
        fixture={'source':'assignment','cases':[{}],'example_source':'example'}
        outcome={'stdout':'new','stderr':'','status':0,'files':{},'error':None}
        current={'clock':4,'favorites':['fixed'],'migration_receipts':{'shell':[7,8,9]},'attempts':{'fixed':{'state':{'kind':'Script','source':'assignment','run':{'unchanged':'old result'}},'workspace':{'command':'unfinished terminal','script_history':{'engine_revision':6,'source':'old6'},'script_history_more':[{'engine_revision':7,'source':'old7'},{'engine_revision':8,'source':'old8'}],'script_learning':{'example_source':'example','example_revision':7,'example_result':{'old':True}}}}}}
        before=json.loads(json.dumps(current))
        actual={('authored-case','fixed/0'):outcome,('worked-example','fixed'):outcome}
        expected=oracle.recheck_projection(current,{'fixed':fixture},actual,())
        self.assertEqual(current,before)
        self.assertEqual(expected['attempts']['fixed']['state']['run'],{'engine_revision':9,'source':'assignment','results':[outcome]})
        self.assertEqual(expected['attempts']['fixed']['workspace']['script_learning'],{'example_source':'example','example_revision':9,'example_result':outcome})
        for key in ('script_history','script_history_more','command'):
            self.assertEqual(expected['attempts']['fixed']['workspace'][key],before['attempts']['fixed']['workspace'][key])
        self.assertEqual(expected['favorites'],before['favorites'])
        for kind in ('unfinished','imported'):
            altered=json.loads(json.dumps(before));a=altered['attempts']['fixed'];a['state']['source']='assignment\nif' if kind=='unfinished' else 'assignment';a['workspace']['script_learning']['example_source']='example\nif' if kind=='unfinished' else 'different original draft'
            observations={('unfinished-assignment' if kind=='unfinished' else 'authored-case','fixed/0'):outcome,('unfinished-example' if kind=='unfinished' else 'imported-example','fixed'):outcome}
            projected=oracle.recheck_projection(altered,{'fixed':fixture},observations,('fixed',) if kind=='unfinished' else ())
            self.assertEqual(projected['attempts']['fixed']['workspace']['script_learning']['example_source'],a['workspace']['script_learning']['example_source'])
            with self.assertRaises(KeyError):oracle.recheck_projection(altered,{'fixed':fixture},{},('fixed',) if kind=='unfinished' else ())

    def test_reference_forwards_exact_reviewed_stdin_and_closed_environment_without_process(self):
        _, rows=oracle.load_fixed(ROOT)
        row=next(r for r in rows if r['id']=='read-leading-trailing')
        with patch.object(oracle.safe,'capture_command',return_value={'mock':True}) as child, patch.object(oracle.time,'monotonic',return_value=100.):
            result=oracle.capture_reference(row,0,'/proc/self/fd/4',Path('/capture'),Path('/work'),Path('/private-tmp'),oracle.safe.Budget(),100.,(3,4))
        self.assertEqual(result,{'mock':True})
        args,kw=child.call_args
        self.assertEqual(args[4],b'  alpha\t \n')
        self.assertEqual(args[0],['/proc/self/fd/4',*oracle.safe.BASH_FLAGS,'-c',row['source'],'fixed-automation-0'])
        self.assertEqual(args[3],{'PATH':'','LC_ALL':'C','LANG':'C','TMPDIR':'/private-tmp','TMP':'/private-tmp','TEMP':'/private-tmp'})
        self.assertEqual(kw,{'timeout':2.0,'inherited':(3,4)})

    def test_profile_actual_bytes_rechecked_and_charged_after_small_stat_growth(self):
        # Deterministic borrowed metadata mocks; no files, subprocess or stress.
        info=SimpleNamespace(st_mode=stat.S_IFREG,st_size=2)
        entries=[SimpleNamespace(name=n,stat=lambda **_:info) for n in ('one','two')]
        @contextmanager
        def small_scan(_):
            yield entries
        with patch.object(oracle,'PROFILE_TOTAL_CAP',10), patch.object(oracle.os,'scandir',small_scan):
            budget=oracle.safe.Budget(100)
            with patch.object(oracle.safe,'pin',side_effect=[{'bytes':6},{'bytes':6}]) as actual:
                with self.assertRaisesRegex(oracle.safe.OracleError,'Actual profile aggregate'):
                    oracle.owned_outputs(Path('/synthetic-profile'),budget)
                self.assertEqual(actual.call_count,2)
                self.assertEqual(budget.used,0)
            budget=oracle.safe.Budget(100)
            with patch.object(oracle.safe,'pin',side_effect=[{'bytes':4},{'bytes':6}]):
                self.assertEqual(oracle.owned_outputs(Path('/synthetic-profile'),budget),[{'bytes':4},{'bytes':6}])
                self.assertEqual(budget.used,10)  # actual bytes, not stat total4.

    def test_every_child_launch_refuses_expired_whole_budget_and_caps_short_remaining(self):
        # Models pin preflight consuming the budget; never spawns any process.
        args=(['/fixed/probe'],Path('/capture'),Path('/work'),{},b'',oracle.safe.Budget(),32)
        for elapsed in (300.,301.):
            with patch.object(oracle.time,'monotonic',return_value=elapsed), patch.object(oracle.safe,'capture_command') as child:
                with self.assertRaisesRegex(oracle.safe.OracleError,'deadline before child'):
                    oracle.capture_child(*args,started=0.,timeout=180.,inherited=())
                child.assert_not_called()
        # Identity10/VM180/version2/reference2 all receive the same remaining1s.
        for cap in (10.,180.,2.):
            with patch.object(oracle.time,'monotonic',return_value=299.), patch.object(oracle.safe,'capture_command',return_value={'mock':True}) as child:
                self.assertEqual(oracle.capture_child(*args,started=0.,timeout=cap,inherited=(4,)),{'mock':True})
                self.assertEqual(child.call_args.kwargs,{'timeout':1.,'inherited':(4,)})
        with patch.object(oracle.time,'monotonic',return_value=1.), patch.object(oracle.safe,'capture_command') as child:
            oracle.capture_child(*args,started=0.,timeout=2.,inherited=())
            self.assertEqual(child.call_args.kwargs['timeout'],2.)

    def test_added_contexts_require_empty_history_and_exact_current_keyset(self):
        # Ephemeral checker inputs only; no new outcomes or saved credit are invented.
        old=json.loads((ROOT/'tests/fixtures/shell-v8-synthetic-writer-progress.json').read_bytes())
        _,authored=oracle.authored_inputs(ROOT)
        current=json.loads(json.dumps(old))
        added=set(authored['catalog'])-set(old['attempts'])
        self.assertEqual(len(added),17)
        for name in added: current['attempts'][name]={'workspace':{}}
        self.assertEqual(oracle.verify_added_history_empty(old,current,authored['catalog']),added)
        first=sorted(added)[0]
        record=old['attempts']['shell-arguments']['state']['run']
        for field,value in [('script_history',record),('script_history_more',[record])]:
            bad=json.loads(json.dumps(current));bad['attempts'][first]['workspace'][field]=value
            with self.assertRaisesRegex(oracle.safe.OracleError,'empty prior history'):
                oracle.verify_added_history_empty(old,bad,authored['catalog'])
        bad=json.loads(json.dumps(current));del bad['attempts'][first]['workspace']
        with self.assertRaises(oracle.safe.OracleError):oracle.verify_added_history_empty(old,bad,authored['catalog'])
        bad=json.loads(json.dumps(current));del bad['attempts']['shell-arguments'];bad['attempts']['foreign']={'workspace':{}}
        with self.assertRaisesRegex(oracle.safe.OracleError,'context keys'):
            oracle.verify_added_history_empty(old,bad,authored['catalog'])
        self.assertTrue(old==json.loads((ROOT/'tests/fixtures/shell-v8-synthetic-writer-progress.json').read_bytes()))

    def test_profile_recheck_observations_include_real_idless_stop_without_indexing_it(self):
        # This is the exact genuine stopped row shape from retained actual R2.
        # Named outcomes below are ephemeral checker inputs, not observed credit.
        stop = {'actual': {'error': 'Stopped: this run was not graded',
                'files': {'/work/case-child.txt': 'entered'}, 'status': 2, 'stderr': '', 'stdout': ''},
                'kind': 'stopped-ungraded', 'record_created': False, 'slice': 1,
                'source': "held=$(printf entered > case-child.txt; while true; do printf ''; done)",
                'ticks_before_stop': 3}
        self.assertNotIn('id',stop)
        kinds=('authored-case','worked-example','fixed-automation-public9','vm-boundary-public9',
               'unfinished-assignment','unfinished-example','imported-example')
        named=[];expected={}
        for index,kind in enumerate(kinds):
            identity='fixed/0' if kind=='authored-case' else 'fixed'
            outcome={'stdout':'validator-'+str(index),'stderr':'','status':0,'files':{},'error':None}
            for size in (1,16,32,64):named.append({'kind':kind,'id':identity,'slice':size,'actual':outcome})
            expected[(kind,identity)]=outcome
        # Metadata must never become recheck outcomes; verify_vm validates it first.
        rows=[{'kind':'identity'},*named,{'kind':'authored-record'},stop,
              {'kind':'profile-evidence'},{'kind':'finished'}]
        before=json.loads(json.dumps(rows))
        actual=oracle.profile_observations(rows)
        self.assertEqual(actual,expected)
        self.assertEqual(len(actual),7)
        self.assertEqual(rows,before)
        # Exercise the same pure recheck path called by verify_profiles after indexing.
        current={'attempts':{'fixed':{'state':{'source':'assignment','run':None},'workspace':{
            'script_learning':{'example_source':'example','example_revision':7,'example_result':{'old':True}}}}}}
        scripts={'fixed':{'source':'assignment','cases':[{}],'example_source':'example'}}
        projected=oracle.recheck_projection(current,scripts,actual,())
        self.assertEqual(projected['attempts']['fixed']['state']['run']['results'],[expected[('authored-case','fixed/0')]])
        self.assertEqual(projected['attempts']['fixed']['workspace']['script_learning']['example_result'],expected[('worked-example','fixed')])
        # Malformed known named rows must fail even for a slice not used by recheck.
        base={'kind':'authored-case','id':'fixed/0','slice':1,'actual':expected[('authored-case','fixed/0')]}
        for change,message in [({'id':None},'missing ID'),({'id':''},'missing ID'),
                               ({'slice':0},'invalid slice'),({'slice':True},'invalid slice')]:
            with self.assertRaisesRegex(oracle.safe.OracleError,message):oracle.profile_observations([{**base,**change},stop])
        for size in (1,16):
            bad={k:v for k,v in {**base,'slice':size}.items() if k!='id'}
            with self.assertRaisesRegex(oracle.safe.OracleError,'missing ID'):oracle.profile_observations([bad,stop])
        bad={k:v for k,v in base.items() if k!='slice'}
        with self.assertRaisesRegex(oracle.safe.OracleError,'invalid slice'):oracle.profile_observations([bad,stop])
        bad={k:v for k,v in base.items() if k!='actual'}
        with self.assertRaisesRegex(oracle.safe.OracleError,'missing actual'):oracle.profile_observations([bad,stop])
        with self.assertRaisesRegex(oracle.safe.OracleError,'Unknown VM row'):oracle.profile_observations([stop,{'kind':'unrecognized','slice':1,'actual':{}}])
        with self.assertRaisesRegex(oracle.safe.OracleError,'Duplicate named'):oracle.profile_observations([base,stop,base])

if __name__ == '__main__': unittest.main()

