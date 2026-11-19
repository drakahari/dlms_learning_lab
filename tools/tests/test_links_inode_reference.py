"""Private unexecuted guards. No real child/prctl/limits; owned disk fixtures only."""
import copy
from contextlib import ExitStack
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock,patch
import links_inode_reference as ref
import generate_links_inode_bindings as gen
class LinksInodeReference(unittest.TestCase):
    def test_closed_fixture_roster_and_no_arbitrary_path_target_or_query(self):
        v=ref.fixture();self.assertEqual(len(v['entries']),10);self.assertEqual(len(v['queries']),8)
        for k,x in [('schema_version',True),('script','id')]:
            bad=copy.deepcopy(v);bad[k]=x
            with self.assertRaises(ValueError):ref.validate_fixture(bad)
        for p in('/outside','../../personal','/work/../else'):
            bad=copy.deepcopy(v);bad['entries'][3]['target']=p
            with self.assertRaises(ValueError):ref.validate_fixture(bad)
        bad=copy.deepcopy(v);bad['queries'][0]['args']=['-l','/home/private']
        with self.assertRaises(ValueError):ref.validate_fixture(bad)
        bad=copy.deepcopy(v);bad['entries'][0]['mode']=True
        with self.assertRaises(ValueError):ref.validate_fixture(bad)
    def test_environment_omits_personal_path_locale_loader_and_profiles(self):
        with tempfile.TemporaryDirectory()as tmp,patch.dict(os.environ,{'PATH':'/personal/bin','HOME':'/personal','LD_PRELOAD':'bad','BASH_ENV':'/personal/start','QUOTING_STYLE':'shell'}):
            env=ref.environment(Path(tmp))
            self.assertEqual(set(env),{'LC_ALL','LANG','TMPDIR','TMP','TEMP'});self.assertEqual(env['LC_ALL'],'C')
            self.assertNotIn('PATH',env);self.assertNotIn('HOME',env);self.assertNotIn('QUOTING_STYLE',env)
    def test_unchanged_shared_nnp_refuses_parent_before_any_security_limit_or_proof_call(self):
        self.assertIs(ref.capture_restricted,ref.shared.capture_restricted)
        prctl=Mock();write=Mock();close=Mock();limits=Mock()
        guard=ref.shared.ChildGuard(7,10,prctl,pid=lambda:10,write=write,close=close,apply_limits=limits,apply_core=Mock())
        with self.assertRaises(RuntimeError):guard()
        prctl.assert_not_called();write.assert_not_called();close.assert_not_called();limits.assert_not_called()
    def test_child_call_order_and_failed_readback_are_entirely_mocked(self):
        events=[];answers=iter((0,1))
        def prctl(*a):events.append(('prctl',a));return next(answers)
        def write(fd,b):events.append(('write',fd,b));return 1
        guard=ref.shared.ChildGuard(7,10,prctl,pid=lambda:11,write=write,close=lambda fd:events.append(('close',fd)),apply_limits=lambda:events.append(('limits',)),apply_core=lambda:events.append(('core',)))
        guard();self.assertEqual(events,[('prctl',(38,1,0,0,0)),('prctl',(39,0,0,0,0)),('limits',),('core',),('write',7,b'N'),('close',7)])
        write=Mock();limits=Mock();close=Mock()
        guard=ref.shared.ChildGuard(7,10,Mock(side_effect=(0,0)),pid=lambda:11,write=write,close=close,apply_limits=limits,apply_core=Mock())
        with self.assertRaises(RuntimeError):guard()
        write.assert_not_called();limits.assert_not_called();close.assert_called_once_with(7)
    def test_raw_uint64_tokens_and_cross_directory_identity_are_not_float_or_fake_model_numbers(self):
        labels={};n='9007199254740993';other='9007199254740994'
        a=ref.listing({'status':0,'stderr':'','stdout':n+' alias\n'+other+' copy\n'+n+' original\n'},labels)
        b=ref.listing({'status':0,'stderr':'','stdout':n+' pin\n'},labels)
        self.assertEqual(a['parsed'][0]['inode_raw'],n)
        self.assertEqual(a['normalized']['entries'][0]['identity'],a['normalized']['entries'][2]['identity'])
        self.assertNotEqual(a['normalized']['entries'][0]['identity'],a['normalized']['entries'][1]['identity'])
        self.assertEqual(b['normalized']['entries'][0]['identity'],a['normalized']['entries'][0]['identity'])
    def test_raw_numbers_are_bound_to_captured_entry_identity_not_just_relation_partition(self):
        q={'id':'default-work'};metadata={'/work/original':{'ino':9007199254740993},'/work/alias':{'ino':9007199254740993}}
        good=ref.listing({'status':0,'stderr':'','stdout':'9007199254740993 alias\n9007199254740993 original\n'}, {})
        ref.bind_listing_to_inodes(good,metadata,q)
        fake=ref.listing({'status':0,'stderr':'','stdout':'17 alias\n17 original\n'}, {})
        self.assertEqual(fake['normalized']['entries'],good['normalized']['entries'])
        with self.assertRaises(ValueError):ref.bind_listing_to_inodes(fake,metadata,q)
    def test_spacing_order_and_name_quotes_are_retained_not_sorted_or_stripped(self):
        a=ref.listing({'status':0,'stderr':'','stdout':' 9 zeta\n10 alpha\n'}, {})
        self.assertEqual([r['name']for r in a['parsed']],['zeta','alpha'])
        self.assertEqual(a['parsed'][0]['padding'],1);self.assertEqual(a['normalized']['relative_columns'],[0,0])
        extra=ref.listing({'status':0,'stderr':'','stdout':'  9 zeta\n10 alpha\n'}, {})
        self.assertEqual(extra['normalized']['relative_columns'],[1,0])
        quoted=ref.listing({'status':0,'stderr':'','stdout':"9 'name with space'\n"}, {})
        self.assertEqual(quoted['parsed'][0]['name'],"'name with space'")
    def test_empty_success_is_distinct_from_error_signal_boolean_and_noncanonical_inode(self):
        self.assertEqual(ref.listing({'status':0,'stderr':'','stdout':''},{})['normalized'],{'status':0,'entries':[],'relative_columns':[]})
        for status in(1,127,-9,True):
            self.assertIn('error',ref.listing({'status':status,'stderr':'','stdout':''},{})['normalized'])
        for raw in('00 name\n','01 name\n','0 name\n','-1 name\n','7 name','7 name\nextra'):
            self.assertIn('error',ref.listing({'status':0,'stderr':'','stdout':raw},{})['normalized'])
    def test_snapshot_three_times_keeps_real_hard_links_symlink_text_zero_mode_and_bytes(self):
        with tempfile.TemporaryDirectory()as tmp,ExitStack()as stack:
            files=ref.Files(Path(tmp),ref.fixture(),stack)
            a=files.snapshot();b=files.snapshot();c=files.snapshot();self.assertEqual(a,b);self.assertEqual(b,c)
            self.assertEqual(a['/work/original']['ino'],a['/work/alias']['ino'])
            self.assertEqual(a['/work/original']['ino'],a['/out/pin']['ino'])
            self.assertEqual(a['/work/original']['nlink'],3);self.assertNotEqual(a['/work/original']['ino'],a['/work/copy']['ino'])
            self.assertEqual(a['/work/original']['mode'],0);self.assertEqual(a['/work/shortcut']['target'],'original')
            self.assertNotEqual(a['/work/shortcut']['ino'],a['/work/original']['ino'])
    def test_descriptor_swap_unexpected_names_and_changed_symlink_inode_refuse(self):
        with tempfile.TemporaryDirectory()as tmp,ExitStack()as stack:
            files=ref.Files(Path(tmp),ref.fixture(),stack)
            old=files.rows['/work/original']['fd'];files.rows['/work/original']['fd']=files.rows['/work/copy']['fd']
            with self.assertRaises(ValueError):files.snapshot()
            files.rows['/work/original']['fd']=old
            parent=files.rows['/out']['fd'];os.unlink('portal',dir_fd=parent);os.symlink('../empty','portal',dir_fd=parent)
            with self.assertRaises(ValueError):files.snapshot()
    def test_literal_argv_never_uses_shell_and_contains_only_known_generated_paths(self):
        files=Mock();files.path.side_effect=lambda p:Path('/fixed/fixture')/p[1:]
        q={'args':['-i','--','-dir room']}
        self.assertEqual(ref.argv('/proc/self/fd/7',files,q),['/proc/self/fd/7','-i','--','-dir room'])
        self.assertEqual(ref.argv('/proc/self/fd/7',files,{'args':['-i','/out/portal']}),['/proc/self/fd/7','-i','/fixed/fixture/out/portal'])
    def test_prepare_writes_no_child_or_nnp_and_bad_control_leaves_no_attempt(self):
        for valid in(True,False):
            with tempfile.TemporaryDirectory()as tmp:
                root=Path(tmp);(root/'.git').mkdir();(root/ref.BASE).parent.mkdir(parents=True)
                control=Mock(return_value={'fixed':'not-observed'})if valid else Mock(side_effect=ValueError('bad'))
                with patch.dict(os.environ,{'CARGO_TARGET_DIR':str(root/'target')}),patch.object(ref,'ROOT',root),patch.object(ref,'fixture',return_value={}),patch.object(ref,'inherited_lock',return_value=9),patch.object(ref,'control',control),patch('sys.argv',['fixed','--mode','prepare']),patch.object(ref,'capture_restricted')as child,patch.object(ref.shared,'load_prctl')as nnp:
                    if valid:ref.main();self.assertEqual(json.loads((root/ref.BASE/'CONTROL.json').read_text()),{'fixed':'not-observed'})
                    else:
                        with self.assertRaises(ValueError):ref.main()
                        self.assertFalse((root/ref.BASE).exists())
                    child.assert_not_called();nnp.assert_not_called()
    def test_control_program_and_probe_approval_are_distinct(self):
        c={'program':{'sha256':'p'},'probe':{'sha256':'r'}};args=Mock(approved_control='wrong',approved_ls='p',approved_probe='r')
        with self.assertRaises(ValueError):ref.approve(args,c,b'control','observe')
        args.approved_control=ref.digest(b'control');ref.approve(args,c,b'control','observe')
        args.approved_probe='wrong'
        with self.assertRaises(ValueError):ref.approve(args,c,b'control','compare')
    def test_source_binding_generator_parser_and_canonical_commitment_are_complete(self):
        with tempfile.TemporaryDirectory()as tmp:
            root=Path(tmp);names=sorted(set(ref.EXTRAS)|{'src/one.rs','scenarios/two.json','assets/three.dat'})
            for p in names:
                out=root/p;out.parent.mkdir(parents=True,exist_ok=True);out.write_text('fixed')
            (root/ref.BINDINGS).write_bytes(gen.render(root))
            with patch.object(ref,'ROOT',root):
                self.assertEqual(ref.binding_names(),names)
                (root/'src/unbound.rs').write_text('new')
                with self.assertRaises(ValueError):ref.binding_names()
        rows=[{'path':'src/z.rs','bytes':1,'sha256':'a'*64},{'path':'src/a.rs','bytes':2,'sha256':'b'*64}]
        encoded=json.dumps(sorted(rows,key=lambda r:r['path']),sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual(ref.binding_commitment(rows),ref.digest(encoded))
    def synthetic(self):
        host=[];model=[]
        for i,id in enumerate(ref.IDS):
            classification='supported'if i<6 else'excluded'
            raw={'status':0,'stdout':'','stderr':''}
            host.append({'id':id,'classification':classification,'actual':raw,'listing':ref.listing(raw,{})if i<6 else None})
            model.append({'id':id,'classification':classification,**raw}if i<6 else{'id':id,'classification':classification,'error':'Outside this subset','honest_exclusion':True})
        return host,model
    def test_excluded_file_and_options_never_inflate_six_relationship_matches(self):
        host,model=self.synthetic();rows=ref.comparison_rows(host,model)
        self.assertEqual(sum(r['exact_relationship_and_relative_columns']is True for r in rows),6)
        self.assertEqual(sum(r['exact_relationship_and_relative_columns']is None for r in rows),2)
        self.assertTrue(all(r['honest_exclusion']for r in rows[6:]))
        model[6]['honest_exclusion']=False;self.assertFalse(ref.comparison_rows(host,model)[6]['honest_exclusion'])
    def test_supported_model_error_and_spacing_difference_are_retained_failures(self):
        host,model=self.synthetic();model[0]={'id':ref.IDS[0],'classification':'supported','error':'bad'}
        self.assertFalse(ref.comparison_rows(host,model)[0]['exact_relationship_and_relative_columns'])
        model[0]['classification']='excluded'
        with self.assertRaises(ValueError):ref.comparison_rows(host,model)
    def test_wrong_actual_observation_digest_refuses_before_probe_or_attempt_creation(self):
        with tempfile.TemporaryDirectory()as tmp:
            base=Path(tmp);(base/'observed').mkdir();(base/'observed/OBSERVATIONS.json').write_text('{}')
            control={'program':{'sha256':'p'},'probe':{'sha256':'r'}}
            args=Mock(approved_control=ref.digest(b'control'),approved_ls='p',approved_probe='r',approved_observations='wrong')
            with patch.object(ref.os,'geteuid',return_value=1000),patch.object(ref,'capture_restricted')as child:
                with self.assertRaises(ValueError):ref.compare(args,base,control,b'control',9)
                child.assert_not_called();self.assertFalse((base/'compared').exists())
if __name__=='__main__':unittest.main()

