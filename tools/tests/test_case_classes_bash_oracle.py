"""Synthetic admission/plumbing guards. Never run Bash, VM, Cargo or a probe.

Fake child observations below are deliberately incorrect test plumbing in new
owned temporary directories. Fixed public actual_* fields stay null; no genuine
historical capture or personal progress is constructed, changed or overwritten.
"""
import copy
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import MagicMock, patch

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
import case_classes_bash_oracle as c
from build_safety import owned_directory


@unittest.skipUnless(os.name == 'posix', 'Fixed Linux reference guards; not other-platform acceptance')
class Guards(unittest.TestCase):
    def setUp(self):
        self.project = TOOLS.parent
        self.raw = (self.project / c.FIXED).read_bytes()
        self.fixed = c.strict(self.raw)
        self.rows = self.fixed['rows']
        parent = owned_directory(self.project, self.project / '.build-artifacts/reference-guard-tests')
        temporary = tempfile.TemporaryDirectory(prefix='class48-', dir=parent)
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)

    def put(self, name, raw, root=None):
        p = (root or self.root) / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(raw if isinstance(raw, bytes) else raw.encode())
        return p

    def clone_fixture(self, revision=12):
        """861 tiny SYNTHETIC source files, not measured source or progress."""
        work = self.root / c.WORKTREE; work.mkdir(parents=True)
        owned_names = [n for n in c.CORE if n != c.PROBE_SOURCE]
        owned_names += ['synthetic-owned/' + str(n) for n in range(21 - len(owned_names))]
        owned, protected = [], []
        for n, name in enumerate(owned_names):
            if name == c.FIXED: raw = self.raw
            elif name == 'src/shell.rs': raw = f'pub const EVALUATOR_REVISION: u32 = {revision};\n'.encode()
            else: raw = f'SYNTHETIC ONLY source {n}\n'.encode()
            path = self.put(name, raw, work)
            owned.append({'path': name, 'after': {'bytes': len(raw), 'sha256': c.digest(raw)}})
        for n in range(840):
            name = f'synthetic-protected/{n:04}.txt'; raw = f'SYNTHETIC protected {n}\n'.encode()
            self.put(name, raw, work); protected.append({'path': name, 'bytes': len(raw), 'sha256': c.digest(raw)})
        self.put(c.PROBE_SOURCE, 'synthetic fixed probe source; never compiled', work)
        self.put(c.FIXED, self.raw)
        inventory = c.canonical({'owned': owned, 'protected': protected})
        folder = Path(c.VM_MANIFEST).parent
        self.put(str(folder / 'SOURCE-INVENTORY.json'), inventory)
        manifest = c.canonical({'base': c.BASE, 'owned_count': 21, 'protected_count': 840,
                               'inventory': {'sha256': c.digest(inventory)}})
        self.put(c.VM_MANIFEST, manifest)
        gitdir = self.root / '.git/worktrees' / work.name
        gitdir.mkdir(parents=True)
        self.put('.git', 'gitdir: ' + str(gitdir) + '\n', work)
        self.put(str(gitdir.relative_to(self.root) / 'HEAD'), 'ref: refs/heads/codex/bash-case-validation-164\n')
        self.put('.git/refs/heads/codex/bash-case-validation-164', 'a' * 40 + '\n')
        c.build_target(self.root, work, None).mkdir(parents=True)
        return work, c.digest(manifest)

    def vm_lines(self, wrong=False):
        return '\n'.join(c.canonical({'id': r['id'], 'source_sha256': c.digest(r['source'].encode()),
            'case_sha256': c.digest(c.canonical(c.case_input(r))),
            'outcome': ({**c.authored(r), 'stdout': 'SYNTHETIC mismatch only\n'} if wrong else c.authored(r))}).decode()
            for r in self.rows) + '\n'

    def test_fixed48_schema_order28_programs_and_null_observations(self):
        self.put(c.FIXED, self.raw)
        self.assertEqual(c.load_fixed(self.root), self.rows)
        self.assertEqual(len(c.PROGRAMS), 28)
        self.assertEqual(tuple(r['id'] for r in self.rows), c.CASE_IDS)
        for row in self.rows:
            self.assertTrue(all(row[k] is None for k in ('actual_bash', 'actual_vm11', 'actual_future_vm')))
        self.assertEqual((self.project / c.FIXED).read_bytes(), self.raw)

    def test_missing_reordered_duplicate_extra_or_modified_inputs_refuse_before_child(self):
        for rows in [self.rows[:-1], list(reversed(self.rows)), [*self.rows, self.rows[0]]]:
            self.put(c.FIXED, c.canonical({**self.fixed, 'rows': rows}))
            with patch.object(c, 'bounded_popen') as launch:
                with self.assertRaises(c.safe.OracleError): c.load_fixed(self.root)
                launch.assert_not_called()
        for key, bad in [('source', 'eval "$1"'), ('env', {'BASH_ENV': '/personal'}),
                         ('stdin', 'unexpected'), ('files', {'/outside': 'x'}), ('actual_bash', {})]:
            row = copy.deepcopy(self.rows[0]); row[key] = bad
            with self.assertRaises(c.safe.OracleError): c.check_case(row)

    def test_quoted_subject_data_is_not_a_host_filename_or_executable_source(self):
        # These exact fixed subjects are admitted only by the byte-pinned recipe;
        # source never reads/executes subject data or uses it as printf format.
        for row in self.rows: c.check_case(row)
        self.assertTrue(any('/' in r['args'][0] and '\n' in r['args'][0] for r in self.rows))
        for bad in [['x', 'extra'], ['a\0b'], ['x' * 129], '/not-a-list']:
            row = copy.deepcopy(self.rows[0]); row['args'] = bad
            with self.assertRaises(c.safe.OracleError): c.check_case(row)

    def test_main_default_guard_is_retained_and_wrong_target_cannot_enter(self):
        (self.root / '.git').mkdir()
        with patch.object(c.safe, 'require_guard', return_value=(123,)) as guard:
            with patch.dict(os.environ, {'CARGO_TARGET_DIR': str(c.build_target(self.root, self.root, None))}):
                self.assertEqual(c.main_entry(self.root), (123,))
            with patch.dict(os.environ, {'CARGO_TARGET_DIR': str(self.root / 'foreign-target')}):
                with self.assertRaises(c.safe.OracleError): c.main_entry(self.root)
            self.assertEqual(guard.call_count, 2)
        other = self.root / 'ordinary-worktree'; other.mkdir()
        self.put('.git', 'not-a-main-directory', other)
        with patch.object(c.safe, 'require_guard') as guard:
            with self.assertRaises(c.safe.OracleError): c.main_entry(other)
            guard.assert_not_called()

    def test_full21_840_clone_and_computed_exclusive_target_are_required(self):
        work, expected = self.clone_fixture()
        with patch.object(c, 'VM_MANIFEST_SHA', expected):
            w, probe, reviewed, identity = c.clone_proof(self.root)
        self.assertEqual(w, work); self.assertEqual(len(reviewed), 861)
        self.assertEqual(probe, c.build_target(self.root, work, None) / c.PROBE_NAME)
        self.assertEqual((identity['owned'], identity['protected'], identity['new_probe']), (21,840,1))

    def test_extra_missing_changed_or_alias_source_cannot_be_hidden_in_clone(self):
        work, expected = self.clone_fixture()
        with patch.object(c, 'VM_MANIFEST_SHA', expected):
            path = self.put('src/unknown.rs', 'unreviewed extra', work)
            with self.assertRaises(c.safe.OracleError): c.clone_proof(self.root)
            path.unlink()  # Only this synthetic test's exact new file.
            source = work / 'src/shell.rs'; raw = source.read_bytes(); source.write_bytes(raw+b'changed')
            with self.assertRaises(c.safe.OracleError): c.clone_proof(self.root)
            source.write_bytes(raw); alias=work/'synthetic-alias';alias.symlink_to(source)
            with self.assertRaises(c.safe.OracleError): c.clone_proof(self.root)

    def test_same_shape_review_does_not_authorize_wrong_revision_or_worktree_metadata(self):
        work, expected = self.clone_fixture(revision=11)
        with patch.object(c, 'VM_MANIFEST_SHA', expected):
            with self.assertRaisesRegex(c.safe.OracleError, 'evaluator12'): c.clone_proof(self.root)
        # Wrong branch/pointer is independently refused on an otherwise exact clone.
        self.root = self.root / 'second-valid-clone'; self.root.mkdir()
        work, expected = self.clone_fixture()
        with patch.object(c, 'VM_MANIFEST_SHA', expected):
            meta = work / '.git'; original = meta.read_bytes()
            meta.write_text('gitdir: /outside\n')
            with self.assertRaisesRegex(c.safe.OracleError, 'metadata'): c.clone_proof(self.root)
            meta.write_bytes(original)
            gitdir = self.root / '.git/worktrees' / work.name
            (gitdir / 'HEAD').write_text('ref: refs/heads/other\n')
            with self.assertRaisesRegex(c.safe.OracleError, 'branch'): c.clone_proof(self.root)

    def test_source_directory_symlink_fifo_and_deep_walk_fail_without_open_hangs(self):
        file=self.put('ordinary','data'); (self.root/'alias').symlink_to(file)
        with self.assertRaises((OSError,c.safe.OracleError)): c.safe.read_bytes(self.root/'alias',100)
        with self.assertRaises(c.safe.OracleError): c.safe.read_bytes(file,1)
        fifo=self.root/'fifo';os.mkfifo(fifo)
        with self.assertRaises(c.safe.OracleError): c.safe.read_bytes(fifo,100)
        walk=self.root/'walk';walk.mkdir();at=walk
        for _ in range(34): at/= 'd';at.mkdir()
        with self.assertRaises(c.safe.OracleError): c.public_files(walk)

    def test_exact_source_binary_approval_and_duplicate_json_keys_fail_closed(self):
        control={'root_reviewed':False,'pins':[{'sha256':'synthetic'}],'rows':48}
        c.approved(c.canonical({**control,'root_reviewed':True}),control)
        for value in [control,{**control,'root_reviewed':1},{**control,'root_reviewed':True,'extra':True},
                      {**control,'root_reviewed':True,'pins':[]}]:
            with self.assertRaises(c.safe.OracleError): c.approved(c.canonical(value),control)
        with self.assertRaises(c.safe.OracleError):c.strict('{"root_reviewed":true,"root_reviewed":false}')

    def test_compiled_identity_rejects_stale_core_wrong_origin_version_or_fixture(self):
        work=self.root/'work';work.mkdir()
        for name in c.CORE:self.put(name,'synthetic source '+name,work)
        value={'kind':'fixed-case-classes-reference','engine_revision':12,'rows':48,'programs':28,
               'input_sha256':c.FIXED_SHA,'manifest_dir':str(work),
               'compiled_sources':{name:c.safe.pin(work/name,c.CAP)['sha256'] for name in c.CORE}}
        c.identity(value,work,self.rows)
        for key,bad in [('engine_revision',11),('manifest_dir','/other'),('rows',47),('input_sha256','0'*64),('compiled_sources',{})]:
            with self.assertRaises(c.safe.OracleError):c.identity({**value,key:bad},work,self.rows)

    def test_all_vm_rows_match_fixed_inputs_and_errors_remain_failed_observations(self):
        text=self.vm_lines();out=c.vm_rows(text,self.rows);self.assertEqual(len(out),48)
        for lines in [text.splitlines()[:-1],list(reversed(text.splitlines())),[*text.splitlines(),text.splitlines()[0]]]:
            with self.assertRaises(c.safe.OracleError):c.vm_rows('\n'.join(lines),self.rows)
        values=[c.strict(line) for line in text.splitlines()]
        values[0]['outcome']['error']='Unsupported synthetic negative'
        outcomes=c.vm_rows('\n'.join(c.canonical(v).decode() for v in values),self.rows)
        self.assertFalse(c.comparison(self.rows[0],outcomes[0],c.authored(self.rows[0]))['passed'])
        values[0]['source_sha256']='bad'
        with self.assertRaises(c.safe.OracleError):c.vm_rows('\n'.join(c.canonical(v).decode() for v in values),self.rows)

    def test_all_four_outcome_fields_and_vm_diagnostic_participate_in_comparison(self):
        row=self.rows[0]; good=c.authored(row)
        self.assertTrue(c.comparison(row,good,good)['passed'])
        for key,bad in [('stdout','miss\n'),('stderr','unexpected'),('status',2),('files',{'/work/x':'unexpected'}),('error','Unsupported')]:
            wrong={**good,key:bad}
            self.assertFalse(c.comparison(row,wrong,good)['passed'],key)
            self.assertFalse(c.comparison(row,good,wrong)['passed'],key)

    def test_unexpected_file_directory_and_symlink_changes_are_never_omitted(self):
        work=self.root/'work';work.mkdir();self.assertEqual(c.empty_work(work),{})
        p=work/'unexpected';p.mkdir()
        with self.assertRaises(c.safe.OracleError):c.empty_work(work)
        p.rmdir();p.symlink_to(self.put('outside','preserve'))
        with self.assertRaises(c.safe.OracleError):c.empty_work(work)
        self.assertEqual((self.root/'outside').read_text(),'preserve')

    def test_child_limits_are_only_deferred_setup_and_environment_has_no_startup_leak(self):
        fake_resource=MagicMock()
        with (patch.object(c,'resource',fake_resource),
              patch.object(c.subprocess,'Popen',return_value='fake') as launch,
              patch.object(c.os,'umask') as umask):
            c.bounded_popen(['synthetic-unexecuted'],vm=True,cwd=self.root)
            self.assertFalse(fake_resource.setrlimit.called);umask.assert_not_called()
            launch.call_args.kwargs['preexec_fn']()
            self.assertEqual(fake_resource.setrlimit.call_count,5);umask.assert_called_once_with(0o077)
        with self.assertRaises(c.safe.OracleError):c.bounded_popen(['unexecuted'],preexec_fn=lambda:None)
        tmp=self.root/'tmp';tmp.mkdir()
        env=c.safe.closed_environment({},tmp)
        self.assertEqual(env['PATH'],'');self.assertFalse({'HOME','BASH_ENV','ENV','SHELLOPTS','LD_PRELOAD'} & set(env))
        self.assertEqual((env['LC_ALL'], env['LANG']), ('C', 'C'))
        for override in ({'LC_ALL': 'C'}, {'LC_ALL': 'en_US.UTF-8'}, {'BASH_ENV': '/outside'}):
            with self.assertRaises(c.safe.OracleError):
                c.safe.closed_environment(override, tmp)

    def test_closed_cli_refuses_all_arbitrary_source_probe_path_and_cwd_selectors(self):
        with patch.object(c,'main_entry',side_effect=AssertionError('No admission/child')):
            for args in [['--mode','capture','--source','anything'],['--mode','capture','--probe','/outside'],
                         ['--mode','capture','--cwd','/outside'],['--mode','capture','--output','/tmp/x'],
                         ['--mode','capture','--capture-id','other'],['--mo','capture']]:
                with self.assertRaises(SystemExit):c.main(args)

    def test_retained_fd_helper_rejects_non_elf_and_boundaries_without_execution(self):
        path=self.put('not-executable','synthetic ordinary bytes')
        with self.assertRaises(c.safe.OracleError):
            with c.pinned_executable(path,c.safe.pin(path,100),100):pass
        with self.assertRaises(c.safe.OracleError):c.child(self.root,'../outside')
        with self.assertRaises(c.safe.OracleError):c.child(self.root,'/outside')
        with self.assertRaises(c.safe.OracleError):c.child(self.root,'./alias')


    def test_capture_delegates_bounded_stream_stdin_timeout_and_shared_fd(self):
        with patch.object(c.safe, 'capture_command', return_value={'synthetic': True}) as capture:
            self.assertEqual(c.capture_child(['synthetic-not-executed'], self.root, self.root,
                {'PATH': ''}, 'budget', (7,), vm=False), {'synthetic': True})
            call = capture.call_args
            self.assertEqual(call.args[4:7], (b'', 'budget', c.STREAM))
            self.assertEqual(call.kwargs['timeout'], 2)
            self.assertEqual(call.kwargs['inherited'], (7,))
            c.capture_child(['synthetic-not-executed'], self.root, self.root, {}, 'budget', (7,), vm=True)
            self.assertEqual(capture.call_args.args[6], c.VM_STREAM)
            self.assertEqual(capture.call_args.kwargs['timeout'], 30)

    def test_retention_is_complete_and_preapproved_and_never_clobbers(self):
        work = self.root / 'work'; work.mkdir()
        for name in (*c.CORE, 'protected-source.txt'):
            self.put(name, 'SYNTHETIC unexecuted source ' + name, work)
        for name in [c.SELF,c.TEST,c.FIXED,'tools/dev.py','tools/build_safety.py',
                     'tools/case_bash_oracle.py','tools/automation_before10_oracle.py']:
            self.put(name, 'SYNTHETIC tooling ' + name)
        paths = [p for p in work.rglob('*') if p.is_file()]
        paths += [self.root / n for n in [c.SELF,c.TEST,c.FIXED,'tools/dev.py','tools/build_safety.py',
                  'tools/case_bash_oracle.py','tools/automation_before10_oracle.py']]
        before = c.pins([(p,c.CAP) for p in paths])
        producer = self.put('producer', b'\x7fELF' + b'SYNTHETIC never executed')
        out = self.root / 'capture'; out.mkdir()
        reviewed = {p.relative_to(work).as_posix(): {} for p in paths if p.is_relative_to(work)
                    and p.relative_to(work).as_posix() != c.PROBE_SOURCE}
        kept = c.snapshot_sources(self.root,work,out,reviewed,producer,c.safe.pin(producer,c.BINARY_CAP),
                                  before,c.safe.Budget(c.TOTAL))
        self.assertEqual(len(kept), len(paths)+1)
        self.assertEqual((out/'actual12-producer').read_bytes(), producer.read_bytes())
        for pin in kept:
            self.assertEqual(c.safe.pin(Path(pin['path']),c.BINARY_CAP),pin)
        with self.assertRaises((FileExistsError,c.safe.OracleError)):
            c.snapshot_sources(self.root,work,out,reviewed,producer,c.safe.pin(producer,c.BINARY_CAP),
                               before,c.safe.Budget(c.TOTAL))
        drift = self.root / 'drift-capture'; drift.mkdir()
        (work/'protected-source.txt').write_text('changed since Root approval')
        with self.assertRaisesRegex(c.safe.OracleError,'Source changed'):
            c.snapshot_sources(self.root,work,drift,reviewed,producer,c.safe.pin(producer,c.BINARY_CAP),
                               before,c.safe.Budget(c.TOTAL))

    def test_completion_of_mismatch_evidence_is_never_acceptance(self):
        outcomes = c.vm_rows(self.vm_lines(wrong=True),self.rows)
        comparisons = [c.comparison(row,out,c.authored(row)) for row,out in zip(self.rows,outcomes)]
        self.assertEqual(sum(v['passed'] for v in comparisons),0)
        self.assertTrue(all(not v['vm_matches_authored'] and v['bash_matches_authored'] for v in comparisons))
        self.assertTrue(all(v['actual_bash'] is None and v['actual_future_vm'] is None for v in self.rows))
