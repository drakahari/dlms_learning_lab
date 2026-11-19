"""Synthetic actual18 guard/readback fixtures; no producer, Bash or learner execution."""
import copy
import errno
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
TOOLS=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(TOOLS))
import scripting_before19_verify as v
spec=importlib.util.spec_from_file_location('before19_dev',TOOLS/'dev.py');dev=importlib.util.module_from_spec(spec);spec.loader.exec_module(dev)

class Genuine18Guards(unittest.TestCase):
    def directory(self):
        t=tempfile.TemporaryDirectory();self.addCleanup(t.cleanup);return Path(t.name)
    def revision(self,n):
        root=self.directory();(root/'src').mkdir();(root/'src/shell.rs').write_text(f'pub const EVALUATOR_REVISION: u32 = {n};\n');return root
    def control_fixture(self):
        root=self.directory();index=json.loads((v.ROOT/'tests/fixtures/bash-before19-source-index.json').read_text())
        for name in set(index['compiled'])|set(v.INPUTS)|{v.BINARY}:
            p=root/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b'unit-only-not-executable')
        (root/'src/shell.rs').write_text('pub const EVALUATOR_REVISION: u32 = 18;\n')
        (root/'examples/support/before19_inputs.rs').write_text('\n'.join(f'("{n}",include_bytes!("../../{n}")),'for n in index['compiled']))
        (root/'tests/fixtures/bash-before19-source-index.json').write_text(json.dumps(index))
        (root/'.git').mkdir();(root/v.REVIEW).parent.mkdir(parents=True);return root,index
    def test_actual18_feature_is_required_and_later_replay_refused(self):
        for flags in [['--features','historical-evaluator18-capture'],['--features=historical-evaluator18-capture'],['-Fhistorical-evaluator18-capture'],['--example=scripting_before19_capture','--features','historical-evaluator18-capture']]:
            dev.historical_capture_preflight(flags,self.revision(18))
            for n in [17,19]:
                with self.assertRaisesRegex(RuntimeError,'actual public evaluator18'):dev.historical_capture_preflight(flags,self.revision(n))
        with self.assertRaisesRegex(RuntimeError,'requires --features'):dev.historical_capture_preflight(['--example','scripting_before19_capture'],self.revision(18))
        with self.assertRaisesRegex(RuntimeError,'evaluator16'):dev.historical_capture_preflight(['--features','historical-evaluator16-capture'],self.revision(18))
    def test_exact_independent_approval_required_and_duplicate_json_refused(self):
        c={'actual_evaluator_revision':18,'root_reviewed':False,'pins':[{'path':'src/shell.rs','size':1,'sha256':'a'*64}]}
        v.approved(json.dumps({**c,'root_reviewed':True}).encode(),c)
        for bad in [c,{**c,'root_reviewed':1},{**c,'root_reviewed':True,'pins':[]}]:
            with self.assertRaises(ValueError):v.approved(json.dumps(bad).encode(),c)
        with self.assertRaises(ValueError):v.strict(b'{"nested":{"x":1,"x":2}}')
    def test_closed_current_source_inventory_includes_new_helpers_without_old_pin_edits(self):
        index=json.loads((v.ROOT/'tests/fixtures/bash-before19-source-index.json').read_text());names=index['compiled']
        self.assertEqual(len(names),551);self.assertEqual(len(set(names)),551)
        for name in ['src/cron.rs','src/fs_integrity.rs','src/environment_workspace.rs','src/startup_workspace.rs','src/shell/startup.rs','tests/support/actual17_evidence.rs','src/raid1.rs','src/raid1_workspace.rs','src/debian_packages/conffiles.rs','src/shell/file_lifecycle.rs']:self.assertIn(name,names)
        self.assertEqual(len(index['catalog']),277);self.assertEqual(len(index['scripts']),71);self.assertEqual(sum(len(r['cases'])for r in index['scripts']),608)
        self.assertEqual([r['id']for r in index['typed']],['lpic1-env-child-settings','lpic1-env-parent-note','lpic1-shell-startup-priority','lpic1-shell-startup-quiet'])
        self.assertTrue(all(t['goal_sha256']==v.canonical_hash(t['goal'])for t in index['typed']))
        prior=json.loads((v.ROOT/'tests/fixtures/bash-before18-source-index.json').read_text());self.assertEqual(index['scripts'][:68],prior['scripts']);self.assertEqual(index['new_scripts'],[r['id']for r in index['scripts'][68:]])
    def test_prepare_only_proposes_and_stale_capture_or_verify_cannot_launch(self):
        root,_=self.control_fixture()
        with patch.object(v,'ROOT',root),patch.object(v,'inherited_lock',return_value=123),patch.dict(os.environ,{'CARGO_TARGET_DIR':str(root/'target')}),patch.object(v.subprocess,'run')as run:
            with patch.object(sys,'argv',['tool','--mode','prepare']):v.main()
            proposed=json.loads((root/v.REVIEW).read_text());self.assertFalse(proposed['root_reviewed']);run.assert_not_called()
            with patch.object(sys,'argv',['tool','--mode','capture']):
                with self.assertRaises(OSError):v.main()
            (root/v.APPROVAL).write_text(json.dumps({**proposed,'root_reviewed':True,'pins':[]}))
            with patch.object(sys,'argv',['tool','--mode','capture']):
                with self.assertRaisesRegex(ValueError,'Review does not match'):v.main()
            with patch.object(sys,'argv',['tool','--mode','verify']),patch.object(v,'SEALED_ATTESTATION',None):
                with self.assertRaisesRegex(ValueError,'not been independently sealed'):v.main()
            run.assert_not_called();self.assertFalse((root/v.RETAINED).exists());self.assertFalse((root/v.DESTINATION).exists())
    def test_unknown_cli_source_profile_or_evaluator_selector_refused_before_lock(self):
        for option in ['--source','--profile','--engine','--output','--cwd']:
            with patch.object(sys,'argv',['tool','--mode','prepare',option,'anything']),patch.object(v,'inherited_lock')as lock,patch.object(v.subprocess,'run')as run:
                with self.assertRaises(SystemExit):v.main()
                lock.assert_not_called();run.assert_not_called()
    def test_duplicate_missing_and_aliased_source_closures_refused(self):
        root,index=self.control_fixture();base=root/'examples/support/before19_inputs.rs';raw=base.read_text();valid=v.control(root);self.assertEqual(valid['actual_evaluator_revision'],18)
        for changed in [raw+'\n'+raw.splitlines()[0], '\n'.join(raw.splitlines()[1:])]:
            base.write_text(changed)
            with self.assertRaisesRegex(ValueError,'compiled closure'):v.control(root)
        base.write_text(raw);p=root/'src/shell.rs';p.unlink();p.symlink_to(root/'Cargo.lock')
        with self.assertRaises(OSError) as failure:v.control(root)
        self.assertEqual(failure.exception.errno,errno.ELOOP)
    def test_inherited_profile_adds_zero_scripts_and_preserves_every_record_and_unrelated_context(self):
        oldrecord={'engine_revision':17,'source':'script','results':[]};newrecord={**oldrecord,'engine_revision':18}
        learning={'example_source':'example','example_revision':17,'example_result':{'status':0}}
        a={'state':{'kind':'Script','source':'script','run':oldrecord},'workspace':{'script_learning':learning},'visited':4}
        old={'favorites':['ctx-2'],'attempts':{'script':a,**{f'ctx-{n}':{'state':{'kind':'Filesystem','marker':n}}for n in range(157)}}}
        new=copy.deepcopy(old);b=new['attempts']['script'];b['state']['run']=newrecord;b['workspace']['script_history']=oldrecord;b['workspace']['script_learning']['example_revision']=18
        row={'id':'script','source':'script','record':newrecord,'example_source':'example','example_outcome':{'status':0}}
        v.profile_checks(old,new,[row],False)
        for mutate in [lambda p:p['attempts'].update({'invented':a}),lambda p:p['attempts']['ctx-3']['state'].update(marker=999),lambda p:p['attempts']['script']['workspace'].pop('script_history'),lambda p:p.update(favorites=[])]:
            bad=copy.deepcopy(new);mutate(bad)
            with self.assertRaises(ValueError):v.profile_checks(old,bad,[row],False)
        with self.assertRaisesRegex(ValueError,'No new Script attempts'):v.profile_checks(old,new,[{**row,'id':'missing'}],False)
    def test_typed_values_exports_file_preservation_and_selection_are_independent_of_flags(self):
        out={'stdout':'','stderr':'','status':0,'files':{'/work/keep':'x'},'error':None};c={'input':{'stdout':'','stderr':'','status':0,'after':out['files']},'parent':{'APP':{'value':'oak','exported':True}},'child':{'stdout':'oak\n','stderr':'','status':0,'after':out['files']},'id':'c','label':'case','present':['~/.bash_profile','~/.profile'],'system_profile':'BASE=system','invocation':{'kind':'InteractiveLogin','noprofile':False}}
        r={'id':'c','label':'case','checks':[[True,'actual checks']],'parent':{'selection':['/etc/profile','~/.bash_profile'],'output':out,'variables':{'parent':c['parent'],'child':{'APP':'oak'}},'command_finished':True},'child':{**out,'stdout':'oak\n'},'error':None,'passed':True}
        self.assertTrue(v.startup_case(c,r))
        for mutate in [lambda x:x['parent']['variables']['parent']['APP'].update(value='hardcoded'),lambda x:x['parent'].update(command_finished=False),lambda x:x['child'].update(files={}),lambda x:x['parent'].update(selection=['~/.profile'])]:
            bad=copy.deepcopy(r);mutate(bad)
            with self.assertRaisesRegex(ValueError,'contradict'):v.startup_case(c,bad)
        bad=copy.deepcopy(r);bad['parent']['variables']['parent']['APP']['exported']=1
        with self.assertRaises(ValueError):v.startup_case(c,bad)
    def test_environment_numeric_flags_cannot_impersonate_boolean_checks(self):
        def fixture(value):
            files={'/work/keep.txt':'retained'}
            c=dict(input=dict(name=value,args=[],stdin='',env={'IN':value},files=files,stdout='',stderr='',status=0,after=files),parent={'IN':dict(value=value,exported=True),'OUT':dict(value=value,exported=True)},child=dict(name=value+'-child',args=[],stdin='',env={'IN':value,'OUT':value},files=files,stdout=value+'\n',stderr='',status=0,after=files))
            r=dict(parent=dict(stdout='',stderr='',status=0,files=files,error=None),bindings=dict(parent=c['parent'],child=c['child']['env']),child=dict(stdout=value+'\n',stderr='',status=0,files=files,error=None),diagnostic=None)
            return c,r
        pairs=[fixture(value)for value in ['oak','elm']];source='OUT="$IN"; export OUT'
        tasks={role:dict(id='fixture-'+role,instructions='Synthetic guard fixture',starter=source,child_source='printf',cases=[c for c,_ in pairs])for role in ['assignment','example']}
        definition=dict(id='fixture',**tasks);goal=dict(kind='Environment',definition=definition)
        records={role:dict(revision=18,task=task,source=source,results=[r for _,r in pairs])for role,task in tasks.items()}
        state=dict(kind='Environment',simulation=dict(definition=definition,view='assignment',**{role:dict(source=source,result=record)for role,record in records.items()}))
        obs={**records,'assignment_verified_in_process':True,'example_verified_in_process':True,'assignment_checks':[[True,'actual']for _ in pairs],'example_checks':[[True,'actual']for _ in pairs]}
        row=dict(goal=goal,goal_sha256=v.canonical_hash(goal),state=state,phase='passed',current_solved=True,observations=obs)
        v.typed_results(row,dict(goal=goal))
        for role in ['assignment','example']:
            bad=copy.deepcopy(row);bad['observations'][role+'_checks'][0][0]=1
            with self.assertRaisesRegex(ValueError,'objective results'):v.typed_results(bad,dict(goal=goal))

    def test_separate_new_profiles_cannot_invent_legacy_credit_or_overwrite_independent_drafts(self):
        root=self.directory();folder=root/'capture';folder.mkdir()
        ids=['shell-stage-report','shell-cleanup-preview','shell-recover-publication']
        rows=[dict(id=id,source='synthetic '+id,record=dict(engine_revision=18,source='synthetic '+id,results=[]),example_source='separate '+id,example_outcome=dict(stdout='',stderr='',status=0,files={},error=None))for id in ids]
        reports=[dict(role=role,contexts=3,loaded_current_solved=role=='passed',repeat_save_stable=True)for role in ['passed','drafts']]
        observations=[];profiles={}
        for role in ['passed','drafts']:
            p=dict(schema_version=1,clock=300,favorites=[ids[0]],attempts={},migration_receipts=dict(shell=[18],workspace=True))
            for id in ids:
                row=next(r for r in rows if r['id']==id)
                state=dict(kind='Script',source=row['source'],run=row['record'])if role=='passed'else dict(kind='Script',source=f"# unfinished {id} assignment\nKEY='",run=None)
                learning=dict(version=1,view='Introduction',example_source=row['example_source']if role=='passed'else f"# separate unfinished {id} practice\nVALUE='",example_revision=18)
                if role=='passed':learning['example_result']=row['example_outcome']
                p['attempts'][id]=dict(state=state,workspace=dict(script_learning=learning))
                observations.append(dict(id=id,role=role,state=state,learning=learning,current_solved=role=='passed'))
            profiles[role]=p
        data=dict(actual_evaluator_revision=18,reports=reports,observations=observations)
        manifest=dict(new_script_profile=dict(contexts_per_profile=3,observations=6,reports=reports))
        def publish():
            (folder/'NEW-SCRIPT-OBSERVATIONS.json').write_text(json.dumps(data))
            for role,p in profiles.items():
                raw=json.dumps(p);(folder/f'writer18-new-{role}.json').write_text(raw)
                d=folder/f'new-{role}';d.mkdir(exist_ok=True);(d/'progress-v1.json').write_text(raw)
        publish();v.validate_new_scripts(root,folder,dict(new_scripts=ids),manifest,rows)
        original=copy.deepcopy(profiles);original_data=copy.deepcopy(data)
        mutations=[lambda:profiles['passed']['migration_receipts'].update(shell=[7,18]),lambda:profiles['drafts']['attempts'][ids[0]]['state'].update(run=rows[0]['record']),lambda:data['observations'][0].update(current_solved=1),lambda:profiles['passed']['attempts'][ids[0]]['workspace'].update(script_history=rows[0]['record'])]
        for mutate in mutations:
            profiles=copy.deepcopy(original);data=copy.deepcopy(original_data);mutate();publish()
            with self.assertRaises(ValueError):v.validate_new_scripts(root,folder,dict(new_scripts=ids),manifest,rows)

    def test_outcome_rejects_missing_extra_or_fabricated_runtime_fields(self):
        case={'stdout':'x\n','stderr':'','status':0,'after':{'/work/a':'retained'}};out={'stdout':'x\n','stderr':'','status':0,'files':case['after'],'error':None}
        self.assertTrue(v.outcome(case,out))
        for bad in [None,{},dict(out,status=False),dict(out,error='limit'),dict(out,files={}),dict(out,extra=True)]:self.assertFalse(v.outcome(case,bad))

if __name__=='__main__':unittest.main()
