"""Fixed map-oracle guard fixtures only; no Bash/probe execution."""
import copy
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
TOOLS=Path(__file__).resolve().parents[1];sys.path.insert(0,str(TOOLS))
import associative_reference as v
import startup_reference
class AssociativeOracleGuards(unittest.TestCase):
    def directory(self):
        t=tempfile.TemporaryDirectory();self.addCleanup(t.cleanup);return Path(t.name)
    def fixture_root(self):
        root=self.directory();(root/'.git').mkdir();p=root/v.FIXTURE;p.parent.mkdir(parents=True);p.write_bytes((v.ROOT/v.FIXTURE).read_bytes());return root
    def test_fixed16_recipes_and_complete_current_closure(self):
        rows=v.recipes();self.assertEqual(len(rows),16);self.assertEqual(len(v.engine_paths()),559);self.assertEqual(len(set(v.engine_paths())),559)
        for name in ['src/shell/associative.rs','src/shell/capture.rs','src/shell/execution.rs','src/environment_workspace.rs','src/startup_workspace.rs','src/cron.rs','src/fs_integrity.rs']:self.assertIn(name,v.engine_paths())
        self.assertTrue(all('/work/'not in r['source']and '/tmp/'not in r['source']for r in rows))
    def test_only_held_bash_and_exact_version_metadata_are_used(self):
        self.assertEqual(v.SYSTEM,{'bash':Path('/usr/bin/bash')})
        valid=dict(status=0,stdout='5.3.9(1)-release\n',stderr='')
        self.assertEqual(v.bash53_version(valid),'5.3.9(1)-release')
        for change in [dict(status=False),dict(status=-9),dict(stderr='unexpected'),dict(stdout='5.2.9(1)-release\n'),dict(stdout='5.30.1\n'),dict(stdout='5.3.9\nextra\n'),dict(stdout=b'5.3.9\n')]:
            with self.assertRaises(ValueError):v.bash53_version({**valid,**change})
    def test_supported_recipes_do_not_pretend_to_oracle_unsupported_diagnostics(self):
        rows=v.recipes()
        for row in rows:
            self.assertNotIn('local ',row['source']);self.assertNotIn('export ',row['source'])
            self.assertNotIn('set -x',row['source']);self.assertNotIn('declare -A m=(',row['source'])
            self.assertNotIn('"${m[@]}"',row['source']);self.assertNotIn('${!m',row['source'])
        self.assertEqual({r['id']for r in rows},{'fresh-map','same-type-redeclaration','quoted-literal-data','dynamic-key-is-data','numeric-string-keys','missing-and-empty-values','replacement-retains-count','plain-write-and-read-key-zero','read-writes-key-zero','loop-writes-key-zero','global-function-write','capture-isolates-map-bindings','pipeline-isolates-key-zero','assignment-context-value','function-failure-keeps-write','utf-eight-key-data'})
    def test_mutated_recipe_refused_before_any_execution(self):
        root=self.fixture_root();p=root/v.FIXTURE;p.write_bytes(p.read_bytes()+b' ')
        with patch.object(v,'ROOT',root),patch.object(v,'capture_command')as run:
            with self.assertRaisesRegex(ValueError,'Fixed recipes changed'):v.recipes()
            run.assert_not_called()
    def test_snapshot_only_reads_owned_flat_ordinary_files(self):
        work=self.directory();(work/'a').write_text('retain');self.assertEqual(v.snapshot(work),{'/work/a':'retain'})
        (work/'alias').symlink_to(work/'a')
        with self.assertRaises(ValueError):v.snapshot(work)
        (work/'alias').unlink();(work/'directory').mkdir()
        with self.assertRaises(ValueError):v.snapshot(work)
    def test_fixed_subprocess_limits_are_child_only_and_bounded(self):
        with patch.object(startup_reference.resource,'setrlimit')as limit:v.limits()
        self.assertEqual(limit.call_count,4);self.assertIn(((startup_reference.resource.RLIMIT_CPU,(2,2)),),[(c.args,)for c in limit.call_args_list])
        self.assertEqual([c.args[1]for c in limit.call_args_list],[(2,2),(256*1024*1024,)*2,(1024*1024,)*2,(32,32)])
    def test_unknown_cli_selectors_refused_before_lock_or_execution(self):
        for option in ['--source','--profile','--binary','--output','--case']:
            with patch.object(sys,'argv',['tool','--mode','prepare-observe',option,'anything']),patch.object(v,'inherited_lock')as lock,patch.object(v,'capture_command')as run:
                with self.assertRaises(SystemExit):v.main()
                lock.assert_not_called();run.assert_not_called()
    def test_prepare_has_no_execution_or_implicit_approval(self):
        root=self.fixture_root();c=dict(root_reviewed=False,fixture_sha256=v.FIXTURE_SHA,pins=[],executables={})
        with patch.object(v,'ROOT',root),patch.object(v,'inherited_lock',return_value=123),patch.dict(os.environ,{'CARGO_TARGET_DIR':str(root/'target')}),patch.object(v,'control',return_value=c),patch.object(v,'capture_command')as run,patch.object(sys,'argv',['tool','--mode','prepare-observe']):v.main()
        files=list((root/v.BASE).glob('observe-control-*.json'));self.assertEqual(len(files),1);self.assertEqual(json.loads(files[0].read_text()),c);self.assertFalse(any((root/v.BASE).glob('*approved*')));run.assert_not_called()
    def test_unapproved_observe_and_unsealed_compare_cannot_run(self):
        root=self.directory();rows=v.recipes()
        with patch.object(v,'approval',side_effect=ValueError('approval pending')),patch.object(v,'capture_command')as run:
            with self.assertRaises(ValueError):v.observe(root,rows,123)
            run.assert_not_called();self.assertFalse((root/'bash-r1').exists())
        with patch.object(v,'OBSERVATIONS_SHA',None),patch.object(v,'capture_command')as run:
            with self.assertRaisesRegex(ValueError,'reference seal pending'):v.compare(root,rows,123)
            run.assert_not_called()
    def test_observation_seal_binds_actual_control_approval_fixture_and_bytes(self):
        import hashlib
        base=self.directory();folder=base/'bash-r1';folder.mkdir()
        c=dict(root_reviewed=False,fixture_sha256=v.FIXTURE_SHA,pins=[],executables={});v.output(folder/'control.json',c);v.output(folder/'approval.json',{**c,'root_reviewed':True})
        observations=dict(fixture_sha256=v.FIXTURE_SHA,observations=[]);v.output(folder/'observations.json',observations);obs=hashlib.sha256((folder/'observations.json').read_bytes()).hexdigest()
        seal=dict(observations_sha256=obs,fixture_sha256=v.FIXTURE_SHA,control_sha256=hashlib.sha256((folder/'control.json').read_bytes()).hexdigest());v.output(folder/'INDEPENDENT_OBSERVATION_SEAL.json',seal);sealed=hashlib.sha256((folder/'INDEPENDENT_OBSERVATION_SEAL.json').read_bytes()).hexdigest()
        with patch.object(v,'OBSERVATIONS_SHA',obs),patch.object(v,'OBSERVATION_SEAL_SHA',sealed):
            self.assertEqual(v.sealed_reference(base),observations)
            (folder/'approval.json').write_text('{}')
            with self.assertRaises(ValueError):v.sealed_reference(base)
        with patch.object(v,'OBSERVATIONS_SHA',obs),patch.object(v,'OBSERVATION_SEAL_SHA',None):
            with self.assertRaises(ValueError):v.sealed_reference(base)
    def test_approved_input_retention_is_exact_and_change_refuses(self):
        root=self.directory();run=self.directory();file=root/'source.py';file.write_bytes(b'fixed-source');c=dict(pins=[v.digest(file)])
        with patch.object(v,'ROOT',root):v.retain_inputs(run,c)
        self.assertEqual((run/'approved-source/source.py').read_bytes(),b'fixed-source');file.write_bytes(b'changed')
        with patch.object(v,'ROOT',root):
            with self.assertRaises(ValueError):v.retain_inputs(self.directory(),c)
    def test_normalization_preserves_status_outputs_files_and_honestly_compares_exact_stderr(self):
        r=dict(id='case',outcome=dict(stdout='report\n',stderr='diagnostic',status=1,files={'/work/a':'retained'},error=None));n=v.normalize(r);self.assertEqual(n,dict(id='case',stdout='report\n',status=1,stderr='diagnostic',files={'/work/a':'retained'}))
        for change in [dict(status=True),dict(error='unsupported'),dict(extra='bad'),dict(files={'/tmp/personal':'no'})]:
            bad=copy.deepcopy(r);bad['outcome'].update(change)
            with self.assertRaises(ValueError):v.normalize(bad)
if __name__=='__main__':unittest.main()
