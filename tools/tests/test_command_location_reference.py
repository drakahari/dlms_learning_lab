"""Fixed-fixture guards only; never launch Bash, which or a model probe."""
import copy
import os
from pathlib import Path
import tempfile
import unittest
from contextlib import ExitStack
from unittest.mock import patch
import command_location_reference as ref


class CommandLocationReference(unittest.TestCase):
    def test_exact_fixture_is_closed_and_untrusted_script_path_and_modes_refuse(self):
        rows = ref.fixture()
        self.assertEqual(tuple(r['id'] for r in rows), ref.IDS)
        for key, bad in [('schema_version', True), ('schema_version', 2), ('cases', []), ('source', 'unreviewed')]:
            value={'schema_version':1,'cases':copy.deepcopy(rows)}
            value[key]=bad
            with self.assertRaises(ValueError):ref.validate(value)
        for key,bad in [('script','touch personal'),('PATH','/usr/bin'),('preferred_mode',True),
                        ('preferred_mode',0o777),('reverse_path',1),('id','../escape'),('preload_hash',True)]:
            value={'schema_version':1,'cases':copy.deepcopy(rows)}
            value['cases'][0][key]=bad
            with self.assertRaises(ValueError,msg=key):ref.validate(value)

    def test_changed_fixture_and_alias_are_rejected_without_child(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);p=root/ref.FIXTURE;p.parent.mkdir(parents=True);p.write_text('{}')
            with patch.object(ref,'ROOT',root),patch.object(ref,'capture_command') as child:
                with self.assertRaises(ValueError):ref.fixture()
                p.unlink();other=root/'unchosen';other.write_text('{}');p.symlink_to(other)
                with self.assertRaises((ValueError,OSError,RuntimeError)):ref.fixture()
                child.assert_not_called()

    def test_environment_is_generated_private_and_does_not_copy_startup_or_loader_settings(self):
        row=ref.fixture()[0]
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);work=root/ref.BASE/'work';work.mkdir(parents=True)
            for name in ('work','preferred','fallback'):(work/name).mkdir()
            with patch.object(ref,'ROOT',root),patch.dict(os.environ,{'BASH_ENV':'/personal/startup','LD_PRELOAD':'escape','PATH':'/personal/bin'}):
                env=ref.generated_environment(work,row)
                self.assertEqual(env['PATH'],str(work/'preferred')+':'+str(work/'fallback'))
                self.assertEqual(env['HOME'],str(work/'work'))
                self.assertNotIn('BASH_ENV',env);self.assertNotIn('LD_PRELOAD',env)
                self.assertEqual(env['LC_ALL'],'C')
                with self.assertRaises(ValueError):ref.generated_environment(root,{'reverse_path':False})
                (work/'preferred').rmdir();(work/'preferred').symlink_to(work/'fallback',target_is_directory=True)
                with self.assertRaises(ValueError):ref.generated_environment(work,{'reverse_path':False})

    def test_held_snapshot_retains_inaccessible_modes_and_bytes_without_permission_restore(self):
        rows=ref.fixture()
        with tempfile.TemporaryDirectory() as tmp:
            folder=Path(tmp)
            with ExitStack() as stack:
                work,held=ref.materialize(folder,rows[5],stack)
                self.assertEqual(os.fstat(held['preferred'][1]).st_mode&0o777,0)
                before=ref.snapshot(held);after=ref.snapshot(held)
                self.assertEqual(before,after)
                self.assertEqual(after['preferred']['parent_mode'],0)
                self.assertEqual(after['preferred']['sha256'],ref.digest(ref.BODY['preferred']))
                # Test-owned cleanup only, AFTER proving both snapshots retained 000.
                for _,parent,_,_ in held.values():os.fchmod(parent,0o700)
                fd=held['fallback'][0];os.pwrite(fd,b'X',0)
                with self.assertRaises(ValueError):ref.snapshot(held)

    def test_directory_fixture_uses_retained_identity_not_filesystem_specific_link_count(self):
        row=ref.fixture()[6]
        with tempfile.TemporaryDirectory() as tmp:
            with ExitStack() as stack:
                _,held=ref.materialize(Path(tmp),row,stack)
                before=ref.snapshot(held)
                self.assertTrue(before['preferred']['directory'])
                self.assertGreaterEqual(before['preferred']['nlink'],1)
                self.assertEqual(before,ref.snapshot(held))
                fd,parent,directory,original=held['preferred']
                held['preferred']=(held['fallback'][0],parent,directory,original)
                with self.assertRaises(ValueError):ref.snapshot(held)
                held['preferred']=(fd,parent,directory,original)

    def test_normalization_never_turns_failed_execution_into_missing_or_drops_extra_output(self):
        work=Path('/contained/fixture')
        for status in (-9,None,True,126,127,255):
            with self.assertRaises(ValueError,msg=str(status)):ref.normalized({'status':status,'stdout':''},work)
        missing=ref.normalized({'status':1,'stdout':''},work)
        self.assertEqual(missing,{'status':'Missing','paths':[],'stdout':''})
        found=ref.normalized({'status':0,'stdout':str(work/'preferred/report')+'\n'},work)
        self.assertEqual(found['paths'],['/preferred/report'])
        for status,stdout in [(0,''),(1,str(work/'preferred/report')+'\n'),(0,'/personal/tool\n'),
                              (0,str(work/'preferred/report')),(0,str(work/'preferred/report')+'\nextra\n')]:
            with self.assertRaises(ValueError):ref.normalized({'status':status,'stdout':stdout},work)

    def test_missing_or_mismatched_approval_and_root_execution_refuse_before_child(self):
        from types import SimpleNamespace
        args=SimpleNamespace(approved_control=None,approved_bash=None,approved_which=None)
        value={'programs':{'bash':{'sha256':'a'*64},'which':{'sha256':'b'*64}}}
        with patch.object(ref,'capture_command') as child:
            with self.assertRaises(ValueError):ref.approvals(args,value,b'fixed')
            args.approved_control=ref.digest(b'fixed');args.approved_bash='a'*64;args.approved_which='b'*64
            ref.approvals(args,value,b'fixed')
            with patch.object(ref.os,'geteuid',return_value=0):
                with self.assertRaisesRegex(ValueError,'nonroot'):ref.observe(args,Path('/unused'),value,b'fixed',3)
            child.assert_not_called()

    def test_incomplete_probe_bindings_refuse_before_child_or_reference_execution(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);p=root/ref.BINDINGS;p.parent.mkdir(parents=True);p.write_text('pub const COMPILED: &[(&str,&[u8])] = &[];')
            for family in ('src','scenarios','assets'):(root/family).mkdir()
            with patch.object(ref,'ROOT',root),patch.object(ref,'capture_command') as child:
                with self.assertRaisesRegex(ValueError,'closure'):ref.compiled_inputs()
                child.assert_not_called()


if __name__=='__main__':unittest.main()
