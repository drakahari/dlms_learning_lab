"""Private unexecuted guard proposal. All child/prctl/limit effects mocked; no host tool runs."""
import copy
import ctypes
from contextlib import ExitStack
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock,patch
import file_index_reference as ref

class FileIndexReference(unittest.TestCase):
    def guard(self,results=(0,1),pid=11,write_result=1):
        events=[];answers=iter(results)
        def prctl(*args):events.append(('prctl',args));return next(answers)
        def write(fd,b):events.append(('write',fd,b));return write_result
        g=ref.ChildGuard(77,10,prctl,pid=lambda:pid,write=write,close=lambda fd:events.append(('close',fd)),apply_limits=lambda:events.append(('limits',)),apply_core=lambda:events.append(('core',)))
        return g,events
    def test_parent_refuses_before_any_prctl_limit_or_proof_effect(self):
        g,events=self.guard(pid=10)
        with self.assertRaises(RuntimeError):g()
        self.assertEqual(events,[])
    def test_child_call_order_exact_arguments_proof_close_and_no_parent_security_call(self):
        g,events=self.guard();g()
        self.assertEqual(events,[('prctl',(38,1,0,0,0)),('prctl',(39,0,0,0,0)),('limits',),('core',),('write',77,b'N'),('close',77)])
    def test_set_failure_closes_proof_without_get_limits_or_write(self):
        g,events=self.guard(results=(-1,))
        with self.assertRaises(RuntimeError):g()
        self.assertEqual(events,[('prctl',(38,1,0,0,0)),('close',77)])
    def test_get_failure_closes_proof_without_limits_or_write(self):
        for value in (0,-1,2):
            g,events=self.guard(results=(0,value))
            with self.assertRaises(RuntimeError):g()
            self.assertEqual(events[-1],('close',77));self.assertFalse(any(e[0]in('limits','write')for e in events))
    def test_limit_failure_never_proves_or_reaches_exec(self):
        g,events=self.guard();g.apply_limits=Mock(side_effect=RuntimeError('mock limits failed'))
        with self.assertRaises(RuntimeError):g()
        self.assertEqual(events[-1],('close',77));self.assertFalse(any(e[0]in('core','write')for e in events))
    def test_short_proof_write_is_failure_and_closes(self):
        g,events=self.guard(write_result=0)
        with self.assertRaises(RuntimeError):g()
        self.assertEqual(events[-1],('close',77))
    def test_libc_binding_initializes_fixed_abi_without_invoking_prctl(self):
        fn=Mock();lib=Mock(prctl=fn)
        with patch.object(ref.ctypes,'CDLL',return_value=lib)as load:
            held,actual=ref.load_prctl();load.assert_called_once_with(None,use_errno=True)
        self.assertIs(held,lib);self.assertIs(actual,fn);fn.assert_not_called()
        self.assertEqual(fn.argtypes,[ctypes.c_int,ctypes.c_ulong,ctypes.c_ulong,ctypes.c_ulong,ctypes.c_ulong]);self.assertEqual(fn.restype,ctypes.c_int)
    def test_proof_is_exact_nonblocking_receipt_not_missing_or_extra_or_parent(self):
        ref.verify_proof(b'N',10,11)
        for marker,child in[(b'',11),(b'NN',11),(b'X',11),(b'N',10),(b'N',None),(b'N',False),(b'N',-1)]:
            with self.assertRaises(ValueError):ref.verify_proof(marker,10,child)
    def test_fixture_closed_roster_unknown_script_boolean_and_unsafe_paths(self):
        original=ref.fixture();self.assertEqual(tuple(p['id']for p in original['phases']),ref.PHASES)
        for key,bad in [('schema_version',True),('source','unreviewed'),('directories',['/'])]:
            v=copy.deepcopy(original);v[key]=bad
            with self.assertRaises(ValueError):ref.validate_fixture(v)
        for key,bad in[('path','/scan/../personal'),('mode',True),('mode',0o755),('source','true')]:
            v=copy.deepcopy(original);v['files'][0][key]=bad
            with self.assertRaises(ValueError):ref.validate_fixture(v)
        v=copy.deepcopy(original);v['phases'][1]['mutations'][0]['path']='/scan/active/unreadable.txt'
        with self.assertRaises(ValueError):ref.validate_fixture(v)
        v=copy.deepcopy(original);v['phases'][0]['queries'][0]['literal']='*'
        with self.assertRaises(ValueError):ref.validate_fixture(v)
    def test_changed_fixture_is_refused_before_child_without_fallback(self):
        with tempfile.TemporaryDirectory()as tmp:
            root=Path(tmp);p=root/ref.FIXTURE;p.parent.mkdir(parents=True);p.write_text('{}')
            with patch.object(ref,'ROOT',root),patch.object(ref,'capture_restricted')as child:
                with self.assertRaises(ValueError):ref.fixture()
                child.assert_not_called()
    def test_closed_environment_never_uses_personal_or_global_index_values(self):
        with tempfile.TemporaryDirectory()as tmp,patch.dict(os.environ,{'HOME':'/personal','LOCATE_PATH':'/global.db','PATH':'/personal/bin','LD_PRELOAD':'bad','BASH_ENV':'/personal/startup'}):
            env=ref.closed_environment({},Path(tmp));self.assertEqual(env['PATH'],'');self.assertEqual(env['LC_ALL'],'C');self.assertNotIn('HOME',env);self.assertNotIn('LOCATE_PATH',env);self.assertNotIn('LD_PRELOAD',env);self.assertNotIn('BASH_ENV',env)
    def test_snapshot_reads_held_zero_mode_leaf_without_restoring_permissions(self):
        v=ref.fixture()
        with tempfile.TemporaryDirectory()as tmp,ExitStack()as stack:
            folder=Path(tmp);files=ref.Files(folder,v,stack);files.config(v['phases'][0]['prunes']);first=files.snapshot();second=files.snapshot()
            self.assertEqual(first,second);self.assertEqual(first['/scan/active/unreadable.txt']['mode'],0)
            self.assertEqual(os.fstat(files.rows['/scan/active/unreadable.txt']['fd']).st_mode&0o777,0)
            self.assertEqual(first['/scan/active/unreadable.txt']['sha256'],ref.digest(b'unreadable inert text\n'))
    def test_fixed_mutations_preserve_original_removed_inode_and_copy_data(self):
        v=ref.fixture()
        with tempfile.TemporaryDirectory()as tmp,ExitStack()as stack:
            files=ref.Files(Path(tmp),v,stack);files.config(v['phases'][0]['prunes']);before=files.snapshot()
            for m in v['phases'][1]['mutations']:files.mutate(m)
            after=files.snapshot();self.assertFalse(after['/scan/active/old.txt']['active']);self.assertEqual(after['/scan/active/old.txt']['nlink'],0);self.assertEqual(after['/scan/active/old.txt']['ino'],before['/scan/active/old.txt']['ino']);self.assertEqual(after['/scan/active/new.txt']['sha256'],after['/scan/source.txt']['sha256'])
            self.assertEqual(after['/scan/active/unreadable.txt'],before['/scan/active/unreadable.txt'])
    def test_status127_bool_signal_and_extra_output_are_never_normalized_to_no_match(self):
        v=ref.fixture();files=Mock(folder=Path('/private/fixture'))
        found={'status':0,'stderr':'','stdout':'/private/fixture/scan/archive\n/private/fixture/scan/archives\n'}
        self.assertEqual(ref.normalized(found,files,v)['paths'],['/scan/archive','/scan/archives'])
        for actual in[{'status':127,'stderr':'','stdout':''},{'status':True,'stderr':'','stdout':''},{'status':-9,'stderr':'','stdout':''},{'status':1,'stderr':'error','stdout':''},{'status':0,'stderr':'','stdout':'/outside\n'},{'status':0,'stderr':'','stdout':'/private/fixture/scan'},{'status':0,'stderr':'','stdout':'/private/fixture/scan\n/private/fixture/scan\n'}]:
            with self.assertRaises(ValueError):ref.normalized(actual,files,v)
        self.assertEqual(ref.normalized({'status':1,'stderr':'','stdout':''},files,v),{'status':'NoMatch','paths':[],'stdout':''})
    def test_binding_commitment_is_canonical_actual_row_order_not_property_order(self):
        rows=[{'path':'src/z.rs','bytes':2,'sha256':'b'*64},{'path':'src/a.rs','bytes':1,'sha256':'a'*64}]
        expected=b'[{"bytes":1,"path":"src/a.rs","sha256":"'+b'a'*64+b'"},{"bytes":2,"path":"src/z.rs","sha256":"'+b'b'*64+b'"}]'
        self.assertEqual(ref.binding_commitment(rows),ref.digest(expected));self.assertEqual(ref.binding_commitment(list(reversed(rows))),ref.digest(expected))
        with self.assertRaises(ValueError):ref.binding_commitment(rows+[rows[0]])
    def test_wrong_fd_or_symlink_and_database_alias_are_refused_not_hidden(self):
        v=ref.fixture()
        with tempfile.TemporaryDirectory()as tmp,ExitStack()as stack:
            files=ref.Files(Path(tmp),v,stack);files.config(v['phases'][0]['prunes']);row=files.rows['/scan/archive'];old=row['fd'];row['fd']=files.rows['/scan/archives']['fd']
            with self.assertRaises(ValueError):files.snapshot()
            row['fd']=old;db=files.path('/work/private.db');db.symlink_to(files.path('/work/index.conf'))
            with self.assertRaises((ValueError,OSError)):ref.db_snapshot(files)
    def test_prepare_and_approval_controls_do_not_create_or_authorize_a_child(self):
        c={'programs':{'plocate':{'sha256':'p'},'updatedb':{'sha256':'u'}},'probe':{'sha256':'r'}};args=Mock(approved_control='wrong',approved_plocate='p',approved_updatedb='u',approved_probe='r')
        with patch.object(ref,'capture_restricted')as child,patch.object(ref,'load_prctl')as security:
            with self.assertRaises(ValueError):ref.approve(args,c,b'control','observe')
            child.assert_not_called();security.assert_not_called()
    def test_prepare_only_writes_control_without_loading_prctl_or_launching(self):
        with tempfile.TemporaryDirectory()as tmp:
            root=Path(tmp);(root/'.git').mkdir();(root/ref.BASE).parent.mkdir(parents=True)
            with patch.dict(os.environ,{'CARGO_TARGET_DIR':str(root/'target')}),patch.object(ref,'ROOT',root),patch.object(ref,'fixture',return_value={}),patch.object(ref,'inherited_lock',return_value=11),patch.object(ref,'control',return_value={'fixed':'unexecuted'}),patch('sys.argv',['fixed','--mode','prepare']),patch.object(ref,'capture_restricted')as capture,patch.object(ref,'load_prctl')as security,patch.object(ref.subprocess,'Popen')as process:
                ref.main();self.assertEqual(json.loads((root/ref.BASE/'CONTROL.json').read_text()),{'fixed':'unexecuted'});capture.assert_not_called();security.assert_not_called();process.assert_not_called()
    def test_binding_roster_closes_all_three_source_roots_and_six_exact_includes(self):
        with tempfile.TemporaryDirectory()as tmp:
            root=Path(tmp);names=sorted(set(ref.EXTRAS)|{'src/one.rs','scenarios/two.json','assets/three.dat'})
            for p in names:
                out=root/p;out.parent.mkdir(parents=True,exist_ok=True);out.write_text('fixed')
            text='\n'.join('("'+p+'", include_bytes!("'+os.path.relpath(p,'examples/support')+'")),'for p in names);(root/ref.BINDINGS).write_text(text)
            with patch.object(ref,'ROOT',root):
                self.assertEqual(ref.binding_names(),names)
                formatted='\n'.join('(\n "'+p+'",\n include_bytes!("'+os.path.relpath(p,'examples/support')+'"),\n), ' for p in names)
                (root/ref.BINDINGS).write_text(formatted)
                self.assertEqual(ref.binding_names(),names)
                (root/ref.BINDINGS).write_text(text)
                (root/'scenarios/new-unbound.json').write_text('{}')
                with self.assertRaises(ValueError):ref.binding_names()
                (root/'scenarios/new-unbound.json').unlink();(root/ref.BINDINGS).write_text(text.replace('include_bytes!("../../src/one.rs")','include_bytes!("../../assets/three.dat")'))
                with self.assertRaises(ValueError):ref.binding_names()

    def test_fixture_path_traversal_and_changed_root_mode_refuse_before_walking(self):
        v=ref.fixture()
        with tempfile.TemporaryDirectory() as tmp,ExitStack() as stack:
            files=ref.Files(Path(tmp),v,stack)
            for path in ('/scan/../outside','/scan/../../outside','/work/../outside','/other'):
                with self.assertRaises(ValueError):files.path(path)
            os.fchmod(files.rootfd,0o750)
            with self.assertRaises(ValueError):files.snapshot()
            os.fchmod(files.rootfd,0o700) # Restore this test-owned temporary directory only.
    def test_invalid_control_does_not_leave_a_prepared_attempt_or_launch(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);(root/'.git').mkdir();(root/ref.BASE).parent.mkdir(parents=True)
            with patch.dict(os.environ,{'CARGO_TARGET_DIR':str(root/'target')}),patch.object(ref,'ROOT',root),patch.object(ref,'fixture',return_value={}),patch.object(ref,'inherited_lock',return_value=11),patch.object(ref,'control',side_effect=ValueError('invalid source')),patch('sys.argv',['fixed','--mode','prepare']),patch.object(ref,'capture_restricted') as capture:
                with self.assertRaises(ValueError):ref.main()
                self.assertFalse((root/ref.BASE).exists());capture.assert_not_called()

    def test_deeper_and_near_prefix_roster_preserves_five_phases_and_seventeen_queries(self):
        v=ref.fixture();self.assertEqual(len(v['files']),7);self.assertEqual(len(v['directories']),6)
        self.assertIn('/scan/active/nested',v['directories']);self.assertIn('/scan/active-other',v['directories'])
        rows={r['path']:r for r in v['files']};self.assertEqual(rows['/scan/active/nested/deep.txt']['mode'],0o600);self.assertEqual(rows['/scan/active-other/sibling.txt']['mode'],0o600)
        self.assertEqual(rows['/scan/active/unreadable.txt']['mode'],0)
        self.assertEqual(tuple(p['id']for p in v['phases']),ref.PHASES);self.assertEqual(sum(len(p['queries'])for p in v['phases']),17);self.assertEqual(sum(p['rebuild']for p in v['phases']),3)
        for removed in('/scan/active/nested','/scan/active-other'):
            wrong=copy.deepcopy(v);wrong['directories'].remove(removed)
            with self.assertRaises(ValueError):ref.validate_fixture(wrong)
    def test_normalizer_retains_nonflat_deep_before_sibling_order_without_sorting(self):
        # Synthetic guard data only, not an actual R2 utility observation.
        v=ref.fixture();files=Mock(folder=Path('/private/fixture'))
        paths=['/scan/active','/scan/active-other','/scan/active/nested','/scan/active/old.txt','/scan/active/nested/deep.txt','/scan/active-other/sibling.txt']
        raw={'status':0,'stderr':'','stdout':''.join('/private/fixture'+p+'\n'for p in paths)}
        actual=ref.normalized(raw,files,v);self.assertEqual(actual['paths'],paths);self.assertEqual(actual['stdout'],''.join(p+'\n'for p in paths));self.assertLess(actual['paths'].index('/scan/active/nested/deep.txt'),actual['paths'].index('/scan/active-other/sibling.txt'))
if __name__=='__main__':unittest.main()
