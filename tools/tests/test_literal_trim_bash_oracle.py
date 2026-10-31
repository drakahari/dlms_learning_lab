"""Synthetic guards only: no Bash/probe/build/prepare/reference child execution."""
import contextlib
import copy
import io
import json
import os
import stat
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import literal_trim_bash_oracle as c
PROJECT=Path(__file__).resolve().parents[2]
MIRROR=Path(__file__).parent/'fixtures/literal-trim-reference.json'

# CURRENT QA allowance only; frozen literal32 COMPILED/SOURCES are unchanged.
CURRENT_LITERAL_AUTHORED_INPUTS = (('scenarios/shell-literal-normalization-draft.json', 41797, 'bd7149e93384eb8be68aac9cb68486fc1630248b13cab0ff04b78b393c7e1535'),)

class LiteralTrimGuards(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='dlms-literal-mock-')
        self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name)
        self.raw=MIRROR.read_bytes();self.fixture=c.fixed_bytes(self.raw)
    def put(self,name,raw):
        p=self.root/name;p.parent.mkdir(parents=True,exist_ok=True)
        p.write_bytes(raw if isinstance(raw,bytes) else raw.encode());return p
    def result(self,name='phase'):
        f=self.root/name;f.mkdir()
        for stream,text in [('stdout','out\n'),('stderr','err\n')]:
            (f/(stream+'.raw')).write_text(text)
        (f/'RESULT.json').write_text(json.dumps({'stdout':'out\n','stderr':'err\n','status':3}))
        (f/'STATUS.json').write_text(json.dumps({'complete':True,'status':3}))
        return f
    def test_exact_fixture_null_observations_and_partition(self):
        self.assertEqual(len(self.fixture['cases']),32);self.assertEqual(len(self.fixture['vm_only']),8)
        self.assertEqual(c.digest(self.raw),c.FIXTURE_SHA)
        self.assertTrue(all(r['observed'] is None for r in self.fixture['cases']))
        self.assertTrue(all(r['observed'] is None and r['executed'] is False for r in self.fixture['vm_only']))
        self.assertEqual(self.raw,(PROJECT/c.FIXTURE).read_bytes())
    def test_any_source_args_files_or_expected_mutation_rejected(self):
        for field,value in [('source','echo arbitrary'),('args',['../../personal']),('files',{'/tmp/no':'x'}),('expected',{})]:
            v=copy.deepcopy(self.fixture);v['cases'][0][field]=value
            with self.assertRaises(c.safe.OracleError):c.fixed_bytes(c.canonical(v))
    def test_argv_is_fixed_source_and_literal_data_never_eval(self):
        row=next(r for r in self.fixture['cases'] if r['id']=='command-looking-data')
        argv=c.host_argv('/proc/self/fd/17',row,self.fixture['cases'])
        self.assertEqual(argv,['/proc/self/fd/17','--noprofile','--norc','-c',row['source'],'--','pre$(not-code);*.log'])
        for binary in ['/bin/bash','bash','/proc/self/fd/2/else']:
            with self.assertRaises(c.safe.OracleError):c.host_argv(binary,row,self.fixture['cases'])
        changed=copy.deepcopy(row);changed['source']='echo bad'
        with self.assertRaises(c.safe.OracleError):c.host_argv('/proc/self/fd/17',changed,self.fixture['cases'])
    def test_flat_names_and_bounded_data_only(self):
        for n in ['/home/private','../x','a/b','.','..','\0','a\n']:
            with self.assertRaises(c.safe.OracleError):c.files({n:'x'})
        for v in [{'a':'x'*4097},{'a':'\0'},{str(i):'' for i in range(4)}]:
            with self.assertRaises(c.safe.OracleError):c.files(v)
    def test_quoted_bare_newline_utf8_scope_and_original_value_rows_present(self):
        ids={r['id'] for r in self.fixture['cases']}
        self.assertTrue({'bare-default-ifs','bare-result-glob','quoted-result-no-glob','literal-newline-data','utf8-prefix','utf8-suffix','function-positional-scope','local-name-scope','source-value-not-mutated'}<=ids)
        self.assertEqual(next(r for r in self.fixture['cases'] if r['id']=='literal-newline-data')['expected']['stdout'],'<a\nb\n>\n')
    def test_all_four_outcome_fields_and_error_are_independent(self):
        row=self.fixture['cases'][0];want=row['expected']
        vm={**want,'files':{'/work/'+n:t for n,t in want['files'].items()},'error':None}
        host={k:want[k] for k in ['stdout','stderr','status']}
        self.assertTrue(c.compared(row,vm,host,want['files'])['passed'])
        for key,bad in [('stdout','wrong'),('stderr','wrong'),('status',9),('files',{'keep':'wrong'}),('error','Unsupported')]:
            other=copy.deepcopy(vm);other[key]=bad
            self.assertFalse(c.compared(row,other,host,want['files'])['passed'])
    def test_raw_stream_and_status_verification_not_just_result_json(self):
        f=self.result();self.assertEqual(c.retained_result(self.root,'phase')['status'],3)
        (f/'stdout.raw').write_text('different')
        with self.assertRaises(c.safe.OracleError):c.retained_result(self.root,'phase')
        (f/'stdout.raw').write_text('out\n');(f/'stderr.raw').write_text('different')
        with self.assertRaises(c.safe.OracleError):c.retained_result(self.root,'phase')
    def test_incomplete_or_wrong_status_cannot_be_accepted(self):
        f=self.result()
        for status in [{'complete':False,'status':3},{'complete':True,'status':0}]:
            (f/'STATUS.json').write_text(json.dumps(status))
            with self.assertRaises(c.safe.OracleError):c.retained_result(self.root,'phase')
    def test_literal_root_approval_only_no_extra_keys(self):
        expected={'root_reviewed':False,'pins':[{'path':'fixed','sha256':'0'*64,'bytes':0}]}
        approved={**expected,'root_reviewed':True}
        c.shared.approve(c.canonical(approved),expected)
        for v in [expected,{**approved,'extra':True},{**approved,'pins':[]}]:
            with self.assertRaises(c.safe.OracleError):c.shared.approve(c.canonical(v),expected)
    def test_prepare_has_no_child_or_held_executable_execution(self):
        base=self.root/c.BASE;base.mkdir(parents=True)
        with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'source_pins',return_value=[]),patch.object(c,'control',return_value={'root_reviewed':False}),patch.object(c,'check_control'),patch.object(c.shared,'owned_directory',return_value=base),patch.object(c.safe,'capture_command') as child,patch.object(c.shared,'pinned_executable') as held,contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(c.run(self.root,'prepare'),0)
            child.assert_not_called();held.assert_not_called()
            self.assertTrue((base/(c.CAPTURE+'-PROPOSED.json')).is_file())
            self.assertFalse((base/c.CAPTURE).exists())
    def test_capture_refuses_without_exact_approval_before_child(self):
        base=self.root/c.BASE;base.mkdir(parents=True)
        with patch.object(c.shared,'root_guard',return_value=()),patch.object(c,'fixed',return_value=self.fixture),patch.object(c,'source_pins',return_value=[]),patch.object(c,'control',return_value={'root_reviewed':False}),patch.object(c,'check_control'),patch.object(c.shared,'owned_directory',return_value=base),patch.object(c.safe,'capture_command') as child:
            with self.assertRaises(FileNotFoundError):c.run(self.root,'capture')
            child.assert_not_called();self.assertFalse((base/c.CAPTURE).exists())
    def test_git_detached_loose_packed_lf_contract(self):
        h='a'*40;self.put('.git/HEAD',h+'\n');self.assertEqual(c.source_identity(self.root),h)
        self.put('.git/HEAD','ref: refs/heads/main\n');self.put('.git/packed-refs',h+' refs/heads/main\n^'+'b'*40+'\n')
        self.assertEqual(c.source_identity(self.root),h)
        self.put('.git/refs/heads/main','c'*40+'\n');self.assertEqual(c.source_identity(self.root),'c'*40)
    def test_git_nonlf_control_duplicate_unsafe_and_alias_refused(self):
        self.put('.git/HEAD','ref: refs/heads/main\n')
        for raw in [('a'*40+' refs/heads/main'+sep+'b'*40+' refs/heads/other\n') for sep in ['\v','\f','\r','\x1c','\x1d','\x1e']]+['a'*40+' refs/heads/main\n'+'b'*40+' refs/heads/main\n']:
            self.put('.git/packed-refs',raw)
            with self.assertRaises(c.safe.OracleError):c.source_identity(self.root)
        for raw in ['a'*40+'\v',' '+ 'a'*40+'\n']:
            self.put('.git/HEAD',raw)
            with self.assertRaises(c.safe.OracleError):c.source_identity(self.root)
        self.put('.git/HEAD','ref: refs/heads/../main\n')
        with self.assertRaises(c.safe.OracleError):c.source_identity(self.root)
    def test_real_library_closed_source_superset_and_no_stub_module(self):
        text=(PROJECT/c.PROBE_SOURCE).read_text()
        self.assertIn('use dlms_learning_lab::shell;',text)
        self.assertNotIn('#[path = "../src/shell.rs"]',text)
        self.assertEqual(len(c.COMPILED),len(set(c.COMPILED)))
        actual={p.relative_to(PROJECT).as_posix() for fam in c.SOURCE_FAMILIES for p in (PROJECT/fam).rglob('*') if p.is_file()}
        listed={n for n in c.COMPILED if Path(n).parts[0] in c.SOURCE_FAMILIES}
        authored_extension = {name for name, _, _ in CURRENT_LITERAL_AUTHORED_INPUTS}
        self.assertEqual(len(authored_extension), 1)
        self.assertTrue(listed.isdisjoint(authored_extension))
        # Retain the entire fixed predecessor and reject every unknown path.
        self.assertEqual(actual-authored_extension,listed)
        self.assertEqual(actual,listed|authored_extension)
        for name, size, sha in CURRENT_LITERAL_AUTHORED_INPUTS:
            path=PROJECT/name;info=path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink==1)
            self.assertEqual(info.st_size,size)
            self.assertEqual(c.digest(path.read_bytes()),sha)
        self.assertTrue({'src/shell/literal_trim.rs','src/ere/cooperative.rs','docs/SHELL_SUBSET.md',c.PROBE_SOURCE,c.FIXTURE}<=set(c.COMPILED))
        self.assertEqual(c.digest((PROJECT/c.PROBE_SOURCE).read_bytes()),c.PROBE_SOURCE_SHA)
    def test_source_roster_refuses_omission_extra_alias_and_empty_directory(self):
        names=('src/lib.rs','scenarios/fixed.json','assets/fixed.txt')
        for n in names:self.put(n,'SYNTHETIC')
        with patch.object(c,'COMPILED',names):
            self.assertEqual(c.project_roster(self.root),set(names))
            extra=self.put('src/extra.rs','x')
            with self.assertRaises(c.safe.OracleError):c.project_roster(self.root)
            extra.unlink();directory=self.root/'src/extra';directory.mkdir()
            with self.assertRaises(c.safe.OracleError):c.project_roster(self.root)
            directory.rmdir();(self.root/'src/lib.rs').unlink();(self.root/'src/lib.rs').symlink_to(self.root/'scenarios/fixed.json')
            with self.assertRaises(c.safe.OracleError):c.project_roster(self.root)
    def test_bash_version_identity_not_error_or_unrelated_binary(self):
        c.require_bash_version({'status':0,'stderr':'','stdout':'GNU bash, version 5.3.9(1)-release (x86_64-pc-linux-gnu)\n'},'/proc/self/fd/9')
        for result in [{'status':1,'stderr':'','stdout':'GNU bash, version 5.3.9\n'},{'status':0,'stderr':'error','stdout':'GNU bash, version 5.3.9\n'},{'status':0,'stderr':'','stdout':'other 5.3\n'}]:
            with self.assertRaises(c.safe.OracleError):c.require_bash_version(result,'/proc/self/fd/9')
    def test_reason_only_exclusions_have_no_output_process_slots(self):
        paths=c.output_paths(self.fixture)
        self.assertIn('case31/STATUS.json',paths);self.assertNotIn('case32/STATUS.json',paths)
        self.assertFalse(any(e['id'] in p for e in self.fixture['vm_only'] for p in paths))
    def test_parent14_seals_are_fixed_and_retained_source_inputs(self):
        self.assertEqual(len(c.PARENT14_SEALS),3)
        self.assertTrue(set(c.PARENT14_SEALS)<=set(c.SOURCES))
        for name,sha in c.PARENT14_SEALS.items():self.assertEqual(c.digest((PROJECT/name).read_bytes()),sha)


    def test_frozen_r1_prepare_refuses_literal_content_pack_before_control_destination_or_subject(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        for name, size, sha in CURRENT_LITERAL_AUTHORED_INPUTS:
            raw = (PROJECT / name).read_bytes()
            self.assertEqual(len(raw), size)
            self.assertEqual(c.digest(raw), sha)
            added = self.put(name, raw)
            with patch.object(c, 'COMPILED', names), \
                 patch.object(c.shared, 'root_guard', return_value=()), \
                 patch.object(c, 'fixed', return_value=self.fixture), \
                 patch.object(c, 'control') as control, \
                 patch.object(c.shared, 'owned_directory') as destination, \
                 patch.object(c.shared, 'pinned_executable') as held, \
                 patch.object(c.safe, 'capture_command') as child:
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'Bounded closed source inventory|No unreviewed project source directory|No omitted/extra/alias project source')
                control.assert_not_called()
                destination.assert_not_called()
                held.assert_not_called()
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                # Only the exact newly-created private mock leaf is removed.
                added.unlink()
                self.assertEqual(c.project_roster(self.root), set(names))

if __name__=='__main__':unittest.main()
