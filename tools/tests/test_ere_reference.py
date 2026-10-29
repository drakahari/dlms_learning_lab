"""Guard-only synthetic reports/mocks; never execute GNU/probe or award ERE acceptance."""
import contextlib
import copy
import io
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import ere_reference as c
PROJECT=Path(__file__).resolve().parents[2]

class EreReferenceGuards(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='dlms-ere-guards-')
        self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        self.raw=(PROJECT/c.FIXTURE).read_bytes()
        self.rows=c.strict(self.raw)['cases']
        self.put(c.FIXTURE,self.raw)
    def put(self,name,raw):
        path=self.root/name;path.parent.mkdir(parents=True,exist_ok=True)
        path.write_bytes(raw.encode() if isinstance(raw,str) else raw)
        return path
    def expected(self):
        return {'root_reviewed':False,'compiled_sources':{name:'SYNTHETIC-'+name for name in c.COMPILED},'pins':[]}
    def vm(self):
        expected=self.expected()
        identity={'kind':'fixed-ere-reference-r1','manifest_dir':str(self.root),'fixture_sha256':c.CORE_PINS[c.FIXTURE],
                  'cases':42,'host_cases':34,'VM_only':8,'compiled_sources':expected['compiled_sources']}
        outcomes=[]
        for row in self.rows:
            excluded=row['partition']=='excluded'
            outcomes.append({'id':row['id'],'partition':row['partition'],
                'pattern_sha256':c.digest(row['pattern'].encode()),'line_sha256':c.digest(row['line'].encode()),
                'matched':None if excluded else row['expected_match'],
                'error_kind':row['expected_error_kind'] if excluded else None,
                'diagnostic':'SYNTHETIC guard diagnostic' if excluded else None})
        return {'identity':identity,'outcomes':outcomes}
    def test_exact42_null_predictions_and_modified_rows_are_refused(self):
        self.assertEqual(c.load_fixed(self.root),self.rows)
        self.assertEqual(sum(r['partition']=='supported' for r in self.rows),34)
        self.assertEqual(sum(r['partition']=='excluded' for r in self.rows),8)
        for row in self.rows:
            self.assertIsNone(row['vm_observation']);self.assertIsNone(row['gnu_observation'])
        for key,value in [('pattern','.*'),('line','substituted'),('vm_observation',{}),('partition','supported-extra'),('extra',True)]:
            row={**self.rows[0],key:value}
            with self.assertRaises(c.safe.OracleError):c.check_row(row)
        for rows in [self.rows[:-1],self.rows+self.rows[:1],list(reversed(self.rows))]:
            self.put(c.FIXTURE,c.canonical({**c.strict(self.raw),'cases':rows}))
            with patch.object(c.safe,'capture_command') as child:
                with self.assertRaises(c.safe.OracleError):c.load_fixed(self.root)
                child.assert_not_called()
    def test_only_fixed_supported_patterns_become_argv_and_all_exclusions_stay_VM_only(self):
        for row in self.rows:
            if row['partition']=='supported':
                self.assertEqual(c.host_argv('/proc/self/fd/91',row),['/proc/self/fd/91','-E','-q','--',row['pattern']])
            else:
                with self.assertRaises(c.safe.OracleError):c.host_argv('/proc/self/fd/91',row)
        with self.assertRaises(c.safe.OracleError):c.host_argv('/bin/sh',{**self.rows[0],'pattern':'$(not-a-script)'})
    def test_approval_requires_exact_true_not_integer_or_omitted_extra_pins(self):
        control=self.expected();c.approve(c.canonical({**control,'root_reviewed':True}),control)
        for raw in [control,{**control,'root_reviewed':1},{**control,'root_reviewed':True,'extra':1},{'root_reviewed':True},
                    {**control,'root_reviewed':True,'pins':[{'sha256':'changed'}]}]:
            with self.assertRaises(c.safe.OracleError):c.approve(c.canonical(raw),control)
        with self.assertRaises(c.safe.OracleError):c.strict(b'{"a":1,"a":2}')
    def test_prepare_has_no_process_and_missing_approval_never_launches(self):
        with patch.object(c,'root_guard',return_value=(91,)),patch.object(c,'source_pins',return_value=[]),patch.object(c,'control',return_value=self.expected()),patch.object(c.safe,'capture_command') as child:
            with contextlib.redirect_stdout(io.StringIO()):self.assertEqual(c.run(self.root,'prepare'),0)
            child.assert_not_called()
            proposed=self.root/c.BASE/(c.CAPTURE+'-PROPOSED.json')
            self.assertFalse(c.strict(proposed.read_bytes())['root_reviewed'])
            self.assertFalse((self.root/c.BASE/c.CAPTURE).exists())
            with self.assertRaises(FileNotFoundError):c.run(self.root,'capture')
            child.assert_not_called()
    def test_existing_capture_and_prepare_proposal_are_not_overwritten(self):
        base=self.root/c.BASE;base.mkdir(parents=True)
        (base/c.CAPTURE).mkdir()
        sentinel=base/c.CAPTURE/'existing-diagnostics';sentinel.write_bytes(b'SYNTHETIC preserved')
        with patch.object(c,'root_guard',return_value=()),patch.object(c,'source_pins',return_value=[]),patch.object(c,'control',return_value=self.expected()),patch.object(c.safe,'capture_command') as child:
            with self.assertRaises(c.safe.OracleError):c.run(self.root,'prepare')
            child.assert_not_called()
        self.assertEqual(sentinel.read_bytes(),b'SYNTHETIC preserved')
    def test_Main_default_target_and_actual_shared_lock_are_required(self):
        (self.root/'.git').mkdir()
        with patch.object(c.safe,'require_guard',return_value=(91,)) as lock:
            with patch.dict(os.environ,{'CARGO_TARGET_DIR':str(self.root/'target')}):self.assertEqual(c.root_guard(self.root),(91,))
            with patch.dict(os.environ,{'CARGO_TARGET_DIR':str(self.root/'foreign-target')}):
                with self.assertRaises(c.safe.OracleError):c.root_guard(self.root)
            self.assertEqual(lock.call_count,2)
        (self.root/'.git').rmdir();(self.root/'.git').write_text('gitdir: worktree')
        with patch.object(c.safe,'require_guard') as lock:
            with self.assertRaises(c.safe.OracleError):c.root_guard(self.root)
            lock.assert_not_called()
    def test_actual_compiled_module_probe_fixture_Cargo_identity_and_every_row_are_required(self):
        payload=self.vm();expected=self.expected()
        self.assertEqual(len(c.parse_vm(c.canonical(payload),self.root,self.rows,expected)),42)
        for key,value in [('manifest_dir','/unrelated'),('compiled_sources',{}),('fixture_sha256','wrong'),('cases',41)]:
            bad=copy.deepcopy(payload);bad['identity'][key]=value
            with self.assertRaises(c.safe.OracleError):c.parse_vm(c.canonical(bad),self.root,self.rows,expected)
        for outcomes in [payload['outcomes'][:-1],list(reversed(payload['outcomes']))]:
            bad={**payload,'outcomes':outcomes}
            with self.assertRaises(c.safe.OracleError):c.parse_vm(c.canonical(bad),self.root,self.rows,expected)
        for key,value in [('line_sha256','wrong'),('partition','excluded'),('matched',1),('diagnostic','unpaired error')]:
            bad=copy.deepcopy(payload);bad['outcomes'][0][key]=value
            with self.assertRaises(c.safe.OracleError):c.parse_vm(c.canonical(bad),self.root,self.rows,expected)
    def test_mismatch_GNU_errors_streams_and_VM_errors_cannot_be_boolean_success(self):
        row=next(r for r in self.rows if r['partition']=='supported' and r['expected_match'])
        vm={'matched':True,'error_kind':None,'diagnostic':None}
        gnu={'status':0,'stdout':'','stderr':''}
        self.assertTrue(c.compare(row,vm,gnu)['passed'])
        for bad in [{**gnu,'status':1},{**gnu,'status':2},{**gnu,'stderr':'diagnostic'},{**gnu,'stdout':'unexpected'}]:
            self.assertFalse(c.compare(row,vm,bad)['passed'])
        self.assertFalse(c.compare(row,{**vm,'matched':False},gnu)['passed'])
        self.assertFalse(c.compare(row,{**vm,'error_kind':'Limit','matched':None},gnu)['passed'])
        no=next(r for r in self.rows if r['partition']=='supported' and not r['expected_match'])
        self.assertTrue(c.compare(no,{**vm,'matched':False},{**gnu,'status':1})['passed'])
        self.assertFalse(c.compare(no,{**vm,'matched':False},{**gnu,'status':2})['passed'])
    def test_VM_only_exclusions_require_honest_diagnostic_and_never_host_observation(self):
        for row in self.rows:
            if row['partition']=='excluded':
                vm={'matched':None,'error_kind':row['expected_error_kind'],'diagnostic':'SYNTHETIC'}
                self.assertTrue(c.excluded(row,vm))
                self.assertFalse(c.excluded(row,{**vm,'matched':False}))
                self.assertFalse(c.excluded(row,{**vm,'diagnostic':''}))
    def test_child_receives_closed_C_environment_finite_limits_and_no_shell(self):
        result={'stdout':'','stderr':'','status':1}
        def fake(command,folder,work,env,data,budget,cap,**kw):
            self.assertEqual(command,['/proc/self/fd/91','-E','-q','--','^fixed$'])
            self.assertEqual(set(env),{'PATH','LC_ALL','LANG','HOME','TMPDIR','TMP','TEMP'})
            self.assertEqual(env['LC_ALL'],'C');self.assertEqual(env['HOME'],str(work))
            self.assertEqual(data,b'fixed\n');self.assertEqual(cap,c.STREAM)
            self.assertEqual(kw,{'timeout':2,'inherited':(92,)})
            return result
        with patch.object(c.safe,'capture_command',side_effect=fake) as child:
            out=c.safe.private_dir(self.root,'capture')
            actual=c.run_child(out,'case0',['/proc/self/fd/91','-E','-q','--','^fixed$'],b'fixed\n',c.safe.Budget(c.CAP),(92,),c.time.monotonic())
            self.assertEqual(actual,result);self.assertEqual(child.call_count,1)
        self.assertTrue((out/'case0/STATUS.json').is_file())
    def test_source_pins_cover_transitive_project_helpers_and_no_caller_selectors_exist(self):
        for name in ['tools/case_bash_oracle.py','tools/automation_before10_oracle.py','tools/build_safety.py','tools/dev.py']:
            self.assertIn(name,c.SOURCES)
        self.assertEqual(set(c.CORE_PINS),{'src/ere.rs','tests/ere.rs','docs/ERE_SUBSET.md',c.FIXTURE})
        with patch.object(c,'run') as run,contextlib.redirect_stderr(io.StringIO()):
            for args in [['--mode','capture','--pattern','.*'],['--mode','capture','--probe','/bin/sh'],['--mode','prepare','--worktree','elsewhere'],['--mode','capture','--output','/tmp/result']]:
                with self.assertRaises(SystemExit):c.main(args)
            run.assert_not_called()
    def test_ordinary_reader_refuses_aliased_fixture_before_child(self):
        actual=self.root/c.FIXTURE;actual.unlink()
        other=self.put('SYNTHETIC-fixture.json',self.raw);actual.symlink_to(other)
        with patch.object(c.safe,'capture_command') as child:
            with self.assertRaises(OSError):c.load_fixed(self.root)
            child.assert_not_called()

    def test_timeout_retains_partial_streams_and_honestly_unknown_status(self):
        out=c.safe.private_dir(self.root,'timed')
        def failure(command,folder,*args,**kwargs):
            (folder/'stdout.raw').write_bytes(b'SYNTHETIC partial stdout')
            (folder/'stderr.raw').write_bytes(b'SYNTHETIC partial diagnostic')
            raise c.safe.OracleError('SYNTHETIC bounded child timeout')
        with patch.object(c.safe,'capture_command',side_effect=failure):
            with self.assertRaises(c.safe.OracleError):
                c.run_child(out,'case0',['/proc/self/fd/91'],b'',c.safe.Budget(c.CAP),(),c.time.monotonic())
        status=c.strict((out/'case0/STATUS.json').read_bytes())
        self.assertIsNone(status['status']);self.assertFalse(status['complete'])
        self.assertIn('timeout',status['error'])
        self.assertEqual((out/'case0/stderr.raw').read_bytes(),b'SYNTHETIC partial diagnostic')
        self.assertFalse((out/'COMPLETE').exists())
    def test_retained_executable_is_same_descriptor_bounded_exact_bytes(self):
        raw=b'\x7fELF SYNTHETIC guard fixture, never executable'
        path=self.put('SYNTHETIC.elf',raw)
        expected=c.safe.pin(path,4096)
        with c.safe.ordinary(path,4096) as (stream,info):
            pinned=c.retain_executable(stream.fileno(),expected,self.root,'copy.elf',c.safe.Budget(4096))
        self.assertEqual(pinned['sha256'],expected['sha256'])
        self.assertEqual((self.root/'copy.elf').read_bytes(),raw)
        with c.safe.ordinary(path,4096) as (stream,info):
            with self.assertRaises(c.safe.OracleError):
                c.retain_executable(stream.fileno(),expected,self.root,'capped.elf',c.safe.Budget(1))


    def test_GNU_version_uses_exact_held_descriptor_basename_not_fixed_grep_name(self):
        version={'status':0,'stderr':'','stdout':'7 (GNU grep) 3.12\nSYNTHETIC guard banner, no executable run.\n'}
        c.require_gnu_version(version,'/proc/self/fd/7')
        c.require_gnu_version({**version,'stdout':version['stdout'].replace('7 (GNU','95 (GNU',1)},'/proc/self/fd/95')
        self.assertEqual(c.CAPTURE,'ere34-r2')
        # The failed original has a different immutable destination; the retry
        # cannot overwrite its proposal, diagnostics or observations.
        self.assertNotEqual(c.CAPTURE,'ere34-r1')
    def test_GNU_version_wrong_identity_status_diagnostic_and_framing_are_refused(self):
        good={'status':0,'stderr':'','stdout':'7 (GNU grep) 3.12\nSYNTHETIC guard banner.\n'}
        for bad in [{**good,'status':1},{**good,'status':2},{**good,'status':False},
                    {**good,'stderr':'diagnostic'},
                    *[{**good,'stdout':v} for v in ['grep (GNU grep) 3.12\n','8 (GNU grep) 3.12\n',
                      '7 (GNU other) 3.12\n','7 (GNU grep) 3.12 extra\n','7 (GNU grep) 3.12',
                      '7 (GNU grep) 3.12\r\n','7 (GNU grep) 3.12\n\x00',
                      '7 (GNU grep) 3.12\n\x1b[0m\n','\n7 (GNU grep) 3.12\n']]]:
            with self.assertRaises(c.safe.OracleError):c.require_gnu_version(bad,'/proc/self/fd/7')
        for executable in ['/usr/bin/grep','/proc/self/fd/not-a-number','/proc/self/fd/7/extra','/proc/self/fd/7\n']:
            with self.assertRaises(c.safe.OracleError):c.require_gnu_version(good,executable)

if __name__=='__main__':unittest.main()
