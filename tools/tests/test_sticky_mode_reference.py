"""Fixed chmod guards; no host child or real prctl/limit invocation."""
import copy
from contextlib import ExitStack
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock,patch
import sticky_mode_reference as ref
import generate_sticky_mode_bindings as gen
class StickyModeReference(unittest.TestCase):
    def test_exact_seventeen_case_fixture_and_no_source_path_script_selector(self):
        v=ref.fixture();self.assertEqual(len(v['cases']),17);self.assertEqual(v['initial_mode'],0o1770)
        for key,value in [('schema_version',True),('initial_mode',True),('initial_mode',0o777),('script','id'),('path','/personal')]:
            bad=copy.deepcopy(v);bad[key]=value
            with self.assertRaises(ValueError):ref.validate_fixture(bad)
        for key,value in [('spec','u+r; id'),('id','foreign'),('classification','supported')]:
            bad=copy.deepcopy(v);bad['cases'][16][key]=value
            with self.assertRaises(ValueError):ref.validate_fixture(bad)
        bad=copy.deepcopy(v);bad['cases'][0],bad['cases'][1]=bad['cases'][1],bad['cases'][0]
        with self.assertRaises(ValueError):ref.validate_fixture(bad)
    def test_environment_has_only_C_and_private_temp_no_personal_loader_or_shell_context(self):
        with tempfile.TemporaryDirectory()as tmp,patch.dict(os.environ,{'PATH':'/personal/bin','HOME':'/personal','BASH_ENV':'/personal/start','LD_PRELOAD':'bad','PYTHONPATH':'bad','POSIXLY_CORRECT':'1'}):
            env=ref.environment(Path(tmp));self.assertEqual(set(env),{'LC_ALL','LANG','TMPDIR','TMP','TEMP'});self.assertEqual(env['LC_ALL'],'C')
    def test_shared_capture_identity_and_parent_callback_refusal_are_mocked(self):
        self.assertIs(ref.capture_restricted,ref.shared.capture_restricted)
        prctl=Mock();write=Mock();limits=Mock();core=Mock();close=Mock()
        g=ref.shared.ChildGuard(7,10,prctl,pid=lambda:10,write=write,close=close,apply_limits=limits,apply_core=core)
        with self.assertRaises(RuntimeError):g()
        prctl.assert_not_called();write.assert_not_called();limits.assert_not_called();core.assert_not_called();close.assert_not_called()
    def test_child_order_and_set_get_failures_never_write_a_success_proof(self):
        events=[];answers=iter((0,1))
        def prctl(*a):events.append(('prctl',a));return next(answers)
        def write(fd,b):events.append(('write',fd,b));return 1
        g=ref.shared.ChildGuard(7,10,prctl,pid=lambda:11,write=write,close=lambda fd:events.append(('close',fd)),apply_limits=lambda:events.append(('limits',)),apply_core=lambda:events.append(('core',)))
        g();self.assertEqual(events,[('prctl',(38,1,0,0,0)),('prctl',(39,0,0,0,0)),('limits',),('core',),('write',7,b'N'),('close',7)])
        for results in [(-1,), (0,0)]:
            writer=Mock();limits=Mock();core=Mock()
            g=ref.shared.ChildGuard(7,10,Mock(side_effect=results),pid=lambda:11,write=writer,close=Mock(),apply_limits=limits,apply_core=core)
            with self.assertRaises(RuntimeError):g()
            writer.assert_not_called();limits.assert_not_called();core.assert_not_called()
    def test_held_subject_three_snapshots_and_mode_only_changes_without_old_file_restore(self):
        with tempfile.TemporaryDirectory()as tmp,ExitStack()as stack:
            subject=ref.Subject(Path(tmp),stack);a=subject.snapshot();self.assertEqual(a,subject.snapshot());self.assertEqual(a,subject.snapshot())
            self.assertEqual(a['subject']['mode'],0o1770);self.assertEqual(a['subject']['uid'],os.geteuid())
            os.fchmod(subject.fd,0o775);b=subject.snapshot();self.assertEqual(b['subject']['mode'],0o775)
            for key in ('dev','ino','uid','gid','nlink','size','mtime_ns'):self.assertEqual(a['subject'][key],b['subject'][key])
            self.assertEqual(a['parent'],b['parent'])
    def test_subject_descriptor_swap_replacement_or_extra_names_are_not_accepted(self):
        with tempfile.TemporaryDirectory()as tmp,ExitStack()as stack:
            base=Path(tmp);(base/'one').mkdir(mode=0o700);(base/'two').mkdir(mode=0o700)
            one=ref.Subject(base/'one',stack);two=ref.Subject(base/'two',stack)
            original=one.fd;one.fd=two.fd
            with self.assertRaises(ValueError):one.snapshot()
            one.fd=original;os.mkdir('extra',0o700,dir_fd=one.parent)
            with self.assertRaises(ValueError):one.snapshot()
    def test_explicit_argv_uses_only_exact_operator_and_generated_subject(self):
        subject=Mock();subject.path=Path('/private/fixture/subject')
        self.assertEqual(ref.argv('/proc/self/fd/7',subject,{'id':'remove-sticky','spec':'-t'}),['/proc/self/fd/7','--','-t','/private/fixture/subject'])
        for q in [{'id':'foreign','spec':'777'},{'id':'remove-sticky','spec':'-t /personal'}]:
            with self.assertRaises(ValueError):ref.argv('/proc/self/fd/7',subject,q)
    def test_prepare_launches_no_child_and_invalid_control_leaves_no_attempt(self):
        v=ref.fixture()
        for valid in (True,False):
            with tempfile.TemporaryDirectory()as tmp:
                root=Path(tmp);(root/'.git').mkdir();(root/'.build-artifacts/reference-runs').mkdir(parents=True)
                capture=Mock();c={'prepared':True}
                with patch.object(ref,'ROOT',root),patch.object(ref,'fixture',return_value=ref.validate_fixture(copy.deepcopy(v))),patch.object(ref,'inherited_lock',return_value=9),patch.object(ref,'control',side_effect=None if valid else ValueError('bad pin'),return_value=c),patch.object(ref,'capture_restricted',capture),patch.dict(os.environ,{'CARGO_TARGET_DIR':str(root/'target')}),patch('sys.argv',['tool','--mode','prepare']):
                    if valid:ref.main();self.assertEqual(json.loads((root/ref.BASE/'CONTROL.json').read_text()),c)
                    else:
                        with self.assertRaises(ValueError):ref.main()
                        self.assertFalse((root/ref.BASE).exists())
                capture.assert_not_called()
    def test_exact_approval_requires_original_chmod_and_compare_probe(self):
        c={'program':{'sha256':'chmod-hash'},'probe':{'sha256':'probe-hash'}};raw=b'control'
        args=Mock(approved_control=ref.digest(raw),approved_chmod='chmod-hash',approved_probe='probe-hash');ref.approve(args,c,raw,'compare')
        args.approved_chmod='wrong'
        with self.assertRaises(ValueError):ref.approve(args,c,raw,'observe')
        args.approved_chmod='chmod-hash';args.approved_probe='wrong'
        with self.assertRaises(ValueError):ref.approve(args,c,raw,'compare')
    def test_complete_binding_roster_and_include_paths_reject_missing_duplicate_or_alias(self):
        with tempfile.TemporaryDirectory()as tmp:
            root=Path(tmp)
            for base in ('src','scenarios','assets'):(root/base).mkdir()
            (root/'src/one.rs').write_text('one');p=root/ref.BINDINGS;p.parent.mkdir(parents=True);p.write_bytes(gen.render(root))
            with patch.object(ref,'ROOT',root):
                names=ref.binding_names();self.assertEqual(set(names),set(ref.EXTRAS)|{'src/one.rs'})
                raw=p.read_text();p.write_text(raw.replace('"src/one.rs", include_bytes!("../../src/one.rs")','"src/one.rs", include_bytes!("/personal")'))
                with self.assertRaises(ValueError):ref.binding_names()
                p.write_text(raw.replace('    ("src/one.rs", include_bytes!("../../src/one.rs")),\n',''))
                with self.assertRaises(ValueError):ref.binding_names()
                p.write_text(raw.replace('];','    ("src/one.rs", include_bytes!("../../src/one.rs")),\n];'))
                with self.assertRaises(ValueError):ref.binding_names()
    def test_comparison_never_promotes_failure_signal_boolean_wrong_mode_or_hardcoded_case(self):
        host=[{'id':p[0],'classification':'supported'if i<15 else'excluded','actual':{'status':0,'stdout':'','stderr':''},'initial_mode':0o1770,'mode':0o770}for i,p in enumerate(ref.SPECS)]
        model=[{'id':p[0],'classification':'supported'if i<15 else'excluded','ok':True,'stdout':'','initial_mode':0o1770,'mode':0o770}if i<15 else {'id':p[0],'classification':'excluded','ok':False,'honest_exclusion':True,'error':'Unsupported'}for i,p in enumerate(ref.SPECS)]
        rows=ref.comparison_rows(host,model);self.assertEqual(sum(r['exact_mode_and_success']is True for r in rows),15);self.assertEqual(sum(r.get('honest_exclusion')is True for r in rows),2)
        for actual in [{'status':127,'stdout':'','stderr':''},{'status':-9,'stdout':'','stderr':''},{'status':0,'stdout':'','stderr':'warning'}]:
            bad=copy.deepcopy(host);bad[0]['actual']=actual;self.assertFalse(ref.comparison_rows(bad,model)[0]['exact_mode_and_success'])
        bad=copy.deepcopy(host);bad[0]['actual']['status']=True
        with self.assertRaises(ValueError):ref.comparison_rows(bad,model)
        for value in (0o777,True):
            bad=copy.deepcopy(model);bad[0]['mode']=value;self.assertFalse(ref.comparison_rows(host,bad)[0]['exact_mode_and_success'])
        bad=copy.deepcopy(model);bad[15]['ok']=True;self.assertFalse(ref.comparison_rows(host,bad)[15]['honest_exclusion'])
        bad=copy.deepcopy(model);bad.reverse()
        with self.assertRaises(ValueError):ref.comparison_rows(host,bad)
    def test_cli_has_no_arbitrary_script_case_source_path_or_executable_arguments(self):
        for option in ('--script','--source','--case','--program','--path'):
            with patch('sys.argv',['tool','--mode','prepare',option,'foreign']),patch.object(ref,'inherited_lock')as lock:
                with self.assertRaises(SystemExit):ref.main()
                lock.assert_not_called()
if __name__=='__main__':unittest.main()
