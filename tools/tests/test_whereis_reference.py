"""Source-only proposal; any future child/security functions are mocked."""
import copy
from contextlib import ExitStack
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock,patch
import whereis_reference as ref
import generate_whereis_bindings as gen
class WhereisReference(unittest.TestCase):
    def test_fixture_closed_classification_roster_and_no_script_path_or_extra_fields(self):
        v=ref.fixture();self.assertEqual(len(v['cases']),24)
        self.assertEqual(sum(c['classification']=='supported'for c in v['cases']),18)
        self.assertEqual(sum(c['classification']=='unsupported'for c in v['cases']),6)
        for update in [{'schema_version':True},{'script':'id'},{'cases':v['cases'][:-1]}]:
            bad=copy.deepcopy(v);bad.update(update)
            with self.assertRaises(ValueError):ref.validate_fixture(bad)
        for field,value in [('directory','/scan/../personal'),('name','$(id)'),('classification','supported-extra'),('command','id')]:
            bad=copy.deepcopy(v);bad['cases'][0][field]=value
            with self.assertRaises(ValueError):ref.validate_fixture(bad)
    def test_fixed_mutations_cannot_target_outside_or_execute_anything(self):
        v=ref.fixture()
        for mutation in [{'op':'run','script':'id'},{'op':'remove','path':'/home/private'},{'op':'chmod','path':'/scan','mode':True}]:
            bad=copy.deepcopy(v);bad['cases'][0]['mutations']=[mutation]
            with self.assertRaises(ValueError):ref.validate_fixture(bad)
    def test_fixed_node_validation_rejects_symlink_fields_and_wrong_text_kind(self):
        for n in [{'path':'/scan/a','directory':False,'mode':0o600,'text':'x','target':'/outside'},
                  {'path':'/scan/a','directory':True,'mode':0o700,'text':'x'},
                  {'path':'/scan/a','directory':False,'mode':True,'text':'x'},
                  {'path':'/scan/../a','directory':False,'mode':0o600,'text':'x'},
                  {'path':'/scan/a','directory':False,'mode':0o600,'text':'\0'}]:
            with self.assertRaises(ValueError):ref.valid_node(n)
    def test_changed_fixture_is_refused_before_any_child(self):
        with tempfile.TemporaryDirectory()as tmp:
            root=Path(tmp);p=root/ref.FIXTURE;p.parent.mkdir(parents=True);p.write_text('{}')
            with patch.object(ref,'ROOT',root),patch.object(ref,'capture_restricted')as child:
                with self.assertRaises(ValueError):ref.fixture()
                child.assert_not_called()
    def test_environment_removes_path_not_just_empty_and_excludes_all_personal_imports(self):
        with tempfile.TemporaryDirectory()as tmp,patch.dict(os.environ,{'PATH':'/personal/bin','HOME':'/personal','MANPATH':'/personal/man','WHEREIS_DEBUG':'all','LD_PRELOAD':'bad','BASH_ENV':'/personal/start'}):
            env=ref.environment(Path(tmp))
            self.assertEqual(set(env),{'LC_ALL','LANG','TMPDIR','TMP','TEMP'})
            self.assertEqual(env['LC_ALL'],'C')
            for key in('PATH','HOME','MANPATH','WHEREIS_DEBUG','LD_PRELOAD','BASH_ENV'):self.assertNotIn(key,env)
    def test_restriction_uses_unchanged_shared_child_guard_with_parent_refusal(self):
        self.assertIs(ref.capture_restricted,ref.shared.capture_restricted)
        calls=[];g=ref.shared.ChildGuard(7,10,Mock(),pid=lambda:10,write=lambda *a:calls.append('write'),close=lambda *a:calls.append('close'),apply_limits=lambda:calls.append('limits'),apply_core=lambda:calls.append('core'))
        with self.assertRaises(RuntimeError):g()
        self.assertEqual(calls,[]);g.prctl.assert_not_called()
    def test_child_set_get_limits_core_proof_close_call_order_is_fully_mocked(self):
        events=[];values=iter((0,1))
        def prctl(*a):events.append(('prctl',a));return next(values)
        def write(fd,b):events.append(('write',fd,b));return 1
        g=ref.shared.ChildGuard(7,10,prctl,pid=lambda:11,write=write,close=lambda fd:events.append(('close',fd)),apply_limits=lambda:events.append(('limits',)),apply_core=lambda:events.append(('core',)))
        g()
        self.assertEqual(events,[('prctl',(38,1,0,0,0)),('prctl',(39,0,0,0,0)),('limits',),('core',),('write',7,b'N'),('close',7)])
    def test_failed_security_readback_never_proves_or_invokes_limits(self):
        for replies in((-1,),(0,0),(0,-1)):
            fn=Mock(side_effect=replies);write=Mock();limits=Mock();close=Mock()
            g=ref.shared.ChildGuard(7,10,fn,pid=lambda:11,write=write,close=close,apply_limits=limits,apply_core=Mock())
            with self.assertRaises(RuntimeError):g()
            write.assert_not_called();limits.assert_not_called();close.assert_called_once_with(7)
    def test_no_match_status0_and_name_colon_are_not_which_status1(self):
        files=Mock(folder=Path('/fixed/fixture'),rows={})
        c={'name':'atlas'}
        self.assertEqual(ref.normalized({'status':0,'stderr':'','stdout':'atlas:\n'},files,c),{'status':0,'paths':[],'stdout':'atlas:\n'})
        for status in(1,127,-9,True):
            with self.assertRaises(ValueError):ref.normalized({'status':status,'stderr':'','stdout':'atlas:\n'},files,c)
    def test_normalizer_preserves_multi_order_without_crediting_unsupported(self):
        files=Mock(folder=Path('/fixed/fixture'),rows={'/scan/man/atlas.1':{},'/scan/man/atlas.8':{}})
        raw={'status':0,'stderr':'','stdout':'atlas: /fixed/fixture/scan/man/atlas.8 /fixed/fixture/scan/man/atlas.1\n'}
        n=ref.normalized(raw,files,{'name':'atlas'})
        self.assertEqual(n['paths'],['/scan/man/atlas.8','/scan/man/atlas.1'])
        self.assertEqual(n['stdout'],'atlas: /scan/man/atlas.8 /scan/man/atlas.1\n')
    def test_diagnostics_outside_paths_duplicates_and_bad_line_format_are_not_silently_normalized(self):
        files=Mock(folder=Path('/fixed/fixture'),rows={'/scan/bin/atlas':{}})
        for a in [
            {'status':0,'stderr':'error','stdout':'atlas:\n'},
            {'status':0,'stderr':'','stdout':'atlas: /outside\n'},
            {'status':0,'stderr':'','stdout':'atlas: /fixed/fixture/scan/bin/atlas /fixed/fixture/scan/bin/atlas\n'},
            {'status':0,'stderr':'','stdout':'atlas:'},
            {'status':0,'stderr':'','stdout':'atlas:\nextra\n'},
        ]:
            with self.assertRaises(ValueError):ref.normalized(a,files,{'name':'atlas'})
    def test_literal_argv_uses_one_absolute_fixture_dir_no_shell_or_path_fallback(self):
        files=Mock();files.path.return_value=Path('/fixed/fixture/scan/bin')
        c={'category':'binary','directory':'/scan/bin','name':'atlas'}
        self.assertEqual(ref.argv('/proc/self/fd/9',files,c),['/proc/self/fd/9','-b','-B','/fixed/fixture/scan/bin','-f','atlas'])
        files.path.assert_called_once_with('/scan/bin')
    def test_zero_mode_bytes_are_read_from_held_fd_without_permission_restoration(self):
        case=ref.fixture()['cases'][0]
        with tempfile.TemporaryDirectory()as tmp,ExitStack()as stack:
            files=ref.Files(Path(tmp),case,stack);first=files.snapshot();second=files.snapshot();third=files.snapshot()
            self.assertEqual(second,third)
            self.assertEqual(first,second);self.assertEqual(first['/scan/bin/atlas']['mode'],0)
            self.assertEqual(first['/scan/bin/atlas']['sha256'],ref.digest(b'inert fixture bytes\n'))
            self.assertEqual(os.fstat(files.rows['/scan/bin/atlas']['fd']).st_mode&0o777,0)
    def test_held_fd_replacement_and_changed_name_roster_refuse(self):
        case=ref.fixture()['cases'][0]
        with tempfile.TemporaryDirectory()as tmp,ExitStack()as stack:
            files=ref.Files(Path(tmp),case,stack)
            old=files.rows['/scan/bin/atlas']['fd'];files.rows['/scan/bin/atlas']['fd']=files.rows['/scan/man']['fd']
            with self.assertRaises(ValueError):files.snapshot()
            files.rows['/scan/bin/atlas']['fd']=old
            fd=os.open('unreviewed',os.O_CREAT|os.O_EXCL|os.O_WRONLY,0o600,dir_fd=files.rows['/scan/bin']['fd']);os.close(fd)
            with self.assertRaises(ValueError):files.snapshot()
    def test_root_and_path_traversal_are_rejected(self):
        case=ref.fixture()['cases'][0]
        with tempfile.TemporaryDirectory()as tmp,ExitStack()as stack:
            files=ref.Files(Path(tmp),case,stack)
            for p in('/scan/../outside','/work/../outside','/outside'):
                with self.assertRaises(ValueError):files.path(p)
    def test_prepare_controls_and_failure_never_launch_a_child_or_security_callback(self):
        with tempfile.TemporaryDirectory()as tmp:
            root=Path(tmp);(root/'.git').mkdir();(root/ref.BASE).parent.mkdir(parents=True)
            with patch.dict(os.environ,{'CARGO_TARGET_DIR':str(root/'target')}),patch.object(ref,'ROOT',root),patch.object(ref,'fixture',return_value={}),patch.object(ref,'inherited_lock',return_value=9),patch.object(ref,'control',return_value={'fixed':'not-observed'}),patch('sys.argv',['fixed','--mode','prepare']),patch.object(ref,'capture_restricted')as child,patch.object(ref.shared,'load_prctl')as security:
                ref.main();self.assertEqual(json.loads((root/ref.BASE/'CONTROL.json').read_text()),{'fixed':'not-observed'});child.assert_not_called();security.assert_not_called()
    def test_bad_control_does_not_leave_new_attempt(self):
        with tempfile.TemporaryDirectory()as tmp:
            root=Path(tmp);(root/'.git').mkdir();(root/ref.BASE).parent.mkdir(parents=True)
            with patch.dict(os.environ,{'CARGO_TARGET_DIR':str(root/'target')}),patch.object(ref,'ROOT',root),patch.object(ref,'fixture',return_value={}),patch.object(ref,'inherited_lock',return_value=9),patch.object(ref,'control',side_effect=ValueError('invalid')),patch('sys.argv',['fixed','--mode','prepare']),patch.object(ref,'capture_restricted')as child:
                with self.assertRaises(ValueError):ref.main()
                self.assertFalse((root/ref.BASE).exists());child.assert_not_called()
    def test_control_approval_is_exact_and_not_observation_acceptance(self):
        c={'program':{'sha256':'p'},'probe':{'sha256':'r'}}
        args=Mock(approved_control='wrong',approved_whereis='p',approved_probe='r')
        with patch.object(ref,'capture_restricted')as child:
            with self.assertRaises(ValueError):ref.approve(args,c,b'control','observe')
            child.assert_not_called()
        args.approved_control=ref.digest(b'control');ref.approve(args,c,b'control','observe')
        args.approved_probe='wrong'
        with self.assertRaises(ValueError):ref.approve(args,c,b'control','compare')
    def test_source_binding_generator_and_parser_close_all_roots_plus_six_extras(self):
        with tempfile.TemporaryDirectory()as tmp:
            root=Path(tmp);names=sorted(set(ref.EXTRAS)|{'src/one.rs','scenarios/two.json','assets/three.bin'})
            for p in names:
                dest=root/p;dest.parent.mkdir(parents=True,exist_ok=True);dest.write_text('fixed')
            (root/ref.BINDINGS).write_bytes(gen.render(root))
            with patch.object(ref,'ROOT',root):
                self.assertEqual(ref.binding_names(),names)
                (root/'src/new.rs').write_text('not-bound')
                with self.assertRaises(ValueError):ref.binding_names()
    def test_source_commitment_is_canonical_rows_and_not_fixture_echo(self):
        rows=[{'path':'src/b.rs','sha256':'b'*64,'bytes':2},{'path':'src/a.rs','sha256':'a'*64,'bytes':1}]
        encoded=json.dumps(sorted(rows,key=lambda p:p['path']),sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual(ref.binding_commitment(rows),ref.digest(encoded))
        with self.assertRaises(ValueError):ref.binding_commitment(rows+[rows[0]])
    def synthetic_rows(self):
        host=[];model=[]
        for i,id in enumerate(ref.IDS):
            classification='supported'if i<18 else'unsupported'
            n={'status':0,'paths':[],'stdout':'atlas:\n'}
            host.append({'id':id,'classification':classification,'normalized':n})
            model.append({'id':id,'classification':classification,**n}if i<18 else{'id':id,'classification':classification,'error_kind':'Unsupported'})
        return host,model
    def test_unsupported_rows_are_honest_refusals_never_six_extra_utility_matches(self):
        host,model=self.synthetic_rows();rows=ref.comparison_rows(host,model)
        self.assertEqual(sum(r['exact_match']is True for r in rows),18)
        self.assertEqual(sum(r['exact_match']is None for r in rows),6)
        self.assertTrue(all(r['unsupported_honest']for r in rows[18:]))
        model[18]['error_kind']='Invalid';rows=ref.comparison_rows(host,model)
        self.assertFalse(rows[18]['unsupported_honest'])
    def test_supported_model_error_or_mismatch_retained_as_failure_not_hidden_by_class_change(self):
        host,model=self.synthetic_rows();model[0]={'id':ref.IDS[0],'classification':'supported','error_kind':'Unsupported'}
        self.assertFalse(ref.comparison_rows(host,model)[0]['exact_match'])
        model[0]['classification']='unsupported'
        with self.assertRaises(ValueError):ref.comparison_rows(host,model)
if __name__=='__main__':unittest.main()

