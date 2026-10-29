"""Synthetic fixed-reference plumbing only. No Bash/VM/probe/Cargo child runs.

Small invented source/Outcome envelopes below are marked synthetic and confined
to new owned temporary fixtures; they never publish a capture, seal or mastery.
"""
import copy
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch
TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
import loop_control_bash_oracle as c
from build_safety import owned_directory


@unittest.skipUnless(os.name == 'posix', 'Linux development oracle; no other-platform acceptance')
class Guards(unittest.TestCase):
    def setUp(self):
        self.project = TOOLS.parent
        self.raw = (self.project / c.MIRROR).read_bytes()
        self.rows = c.strict(self.raw)['cases']
        parent = owned_directory(self.project, self.project / '.build-artifacts/reference-guard-tests')
        temporary = tempfile.TemporaryDirectory(prefix='loop16-', dir=parent)
        self.addCleanup(temporary.cleanup); self.root = Path(temporary.name)

    def put(self, name, raw, root=None):
        p = (root or self.root) / name; p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(raw if isinstance(raw, bytes) else raw.encode()); return p

    def clone_fixture(self):
        """898 small artificial source files, not measured code or history."""
        self.put(c.MIRROR, self.raw)  # Exact canonical mirror in this fake Main root.
        work = self.root / c.WORKTREE; work.mkdir(parents=True)
        names = list(c.CORE) + ['synthetic/' + str(n) + '.txt' for n in range(898 - len(c.CORE))]
        inventory = {}
        probe = (self.project / c.PROBE_SOURCE).read_bytes() if (self.project / c.PROBE_SOURCE).exists() else b'SYNTHETIC source-probe identity\n'
        for index, name in enumerate(names):
            if name == c.FIXED: raw = self.raw
            elif name == 'src/shell.rs': raw = b'pub const EVALUATOR_REVISION: u32 = 13;\n'
            elif name == c.PROBE_SOURCE: raw = probe
            else: raw = f'SYNTHETIC source {index}\n'.encode()
            self.put(name, raw, work); inventory[name] = {'bytes': len(raw), 'sha256': c.digest(raw)}
        # Synthetic before-image pin has its own explicit patched control, never
        # replaces any real b25 inventory or production proposal.
        raw = c.canonical(inventory); self.put(c.SOURCE_INVENTORY, raw)
        gitdir = self.root / '.git/worktrees' / work.name; gitdir.mkdir(parents=True)
        self.put('.git', 'gitdir: ' + str(gitdir) + '\n', work)
        self.put(str(gitdir.relative_to(self.root) / 'HEAD'), c.BASE + '\n')
        c.build_target(self.root, work, None).mkdir(parents=True)
        return work, c.digest(raw), c.digest(probe), len(probe)

    def clone_patch(self, sha, probe_sha, probe_bytes):
        return patch.multiple(c, INVENTORY_SHA=sha, PROBE_BEFORE_SHA=probe_sha, PROBE_SHA=probe_sha, PROBE_BYTES=probe_bytes)

    def test_fixed24_exact_order16_supported8_VM_only_and_null_predictions(self):
        self.put(c.MIRROR, self.raw); raw, rows = c.load_fixed(self.root)
        self.assertEqual(raw, self.raw); self.assertEqual(rows, self.rows)
        self.assertEqual(tuple(r['id'] for r in rows), c.CASE_IDS)
        self.assertEqual(tuple(r['id'] for r in rows if r['partition'] == 'supported'), c.HOST_IDS)
        self.assertEqual(len(c.HOST_IDS), 16)
        for row in rows:
            self.assertIsNone(row['vm_observation']); self.assertIsNone(row['bash_observation'])
        self.assertEqual((self.project / c.MIRROR).read_bytes(), self.raw)

    def test_all_modified_source_argv_environment_file_and_observation_rows_refused(self):
        mutations = {'source': 'eval "$1"', 'args': ['../../personal'], 'env': {'BASH_ENV': '/personal'}, 'stdin': 'altered', 'files': {'/work/../../personal': 'x'}, 'expected': {}, 'vm_observation': {}, 'bash_observation': {}}
        for key, value in mutations.items():
            row = copy.deepcopy(self.rows[0]); row[key] = value
            with self.assertRaises(c.safe.OracleError): c.check_row(row)
        with self.assertRaises(c.safe.OracleError): c.check_row({})

    def test_missing_extra_reordered_or_hash_tampered_fixture_refused_before_child(self):
        for rows in [self.rows[:-1], self.rows + self.rows[:1], list(reversed(self.rows))]:
            self.put(c.MIRROR, c.canonical({**c.strict(self.raw), 'cases': rows}))
            with patch.object(c.safe, 'capture_command') as child:
                with self.assertRaises(c.safe.OracleError): c.load_fixed(self.root)
                child.assert_not_called()

    def test_explicit_VM_only_rows_cannot_be_host_predictions(self):
        for row in self.rows:
            if row['partition'] != 'supported':
                with self.assertRaises(c.safe.OracleError): c.predicted(row)
                diagnostic = {'stdout': '', 'stderr': '', 'status': 2, 'files': row['files'], 'error': 'SYNTHETIC Unsupported only'}
                self.assertTrue(c.exclusion(row, diagnostic)['honest_unsupported'])
                self.assertIsNone(c.exclusion(row, diagnostic)['actual_bash539'])
                self.assertFalse(c.exclusion(row, {**diagnostic, 'status': 0})['honest_unsupported'])
                self.assertFalse(c.exclusion(row, {**diagnostic, 'files': {'/work/trace.txt': 'bad'}})['honest_unsupported'])
        self.assertEqual(len([r for r in self.rows if r['partition'] != 'supported']), 8)

    def test_main_default_lock_guard_and_wrong_target_refusal_unchanged(self):
        (self.root / '.git').mkdir()
        with patch.object(c.plumbing.safe, 'require_guard', return_value=(91,)) as guard:
            with patch.dict(os.environ, {'CARGO_TARGET_DIR': str(c.build_target(self.root, self.root, None))}):
                self.assertEqual(c.plumbing.main_entry(self.root), (91,))
            with patch.dict(os.environ, {'CARGO_TARGET_DIR': str(self.root / 'foreign')}):
                with self.assertRaises(c.safe.OracleError): c.plumbing.main_entry(self.root)
            self.assertEqual(guard.call_count, 2)

    def test_exact_full898_clone_and_exclusive_target_are_required(self):
        work, sha, probe_sha, probe_bytes = self.clone_fixture()
        with self.clone_patch(sha, probe_sha, probe_bytes):
            actual, probe, reviewed = c.clone_proof(self.root)
        self.assertEqual(actual, work); self.assertEqual(len(reviewed), 898)
        self.assertEqual(probe, c.build_target(self.root, work, None) / c.PROBE_NAME)
        self.assertNotEqual(probe, self.root / 'target' / c.PROBE_NAME)

    def test_missing_changed_extra_or_aliased_clone_source_fails(self):
        work, sha, probe_sha, probe_bytes = self.clone_fixture()
        path = work / 'synthetic/0.txt'; original = path.read_bytes()
        with self.clone_patch(sha, probe_sha, probe_bytes):
            path.unlink()
            with self.assertRaises((OSError, c.safe.OracleError)): c.clone_proof(self.root)
            path.write_bytes(original + b'changed')
            with self.assertRaises(c.safe.OracleError): c.clone_proof(self.root)
            path.write_bytes(original)
            other = self.put('src/extra.rs', b'unknown', work)
            with self.assertRaises(c.safe.OracleError): c.clone_proof(self.root)
            other.unlink(); path.unlink(); path.symlink_to(work / c.PROBE_SOURCE)
            with self.assertRaises((OSError, c.safe.OracleError)): c.clone_proof(self.root)

    def test_compiled_probe_wrong_revision_checkout_core_or_fixture_identity_fails(self):
        work, sha, probe_sha, probe_bytes = self.clone_fixture()
        payload = {'fixture_id': 'loop-control24-r1', 'engine_revision': 13, 'manifest_dir': str(work), 'compiled_sources': {n: c.safe.pin(work / n, c.CAP)['sha256'] for n in c.CORE}, 'fixture_utf8': self.raw.decode(), 'outcomes': []}
        for row in self.rows:
            value = c.predicted(row) if row['partition'] == 'supported' else {'stdout': '', 'stderr': '', 'status': 2, 'files': {}, 'error': 'SYNTHETIC Unsupported only'}
            payload['outcomes'].append({'id': row['id'], 'partition': row['partition'], 'outcome': value})
        self.assertEqual(len(c.vm_rows(c.canonical(payload).decode(), work, self.rows)), 24)
        for key, bad in [('engine_revision', 12), ('manifest_dir', str(self.root)), ('compiled_sources', {}), ('fixture_utf8', '{}'), ('outcomes', payload['outcomes'][:-1])]:
            changed = copy.deepcopy(payload); changed[key] = bad
            with self.assertRaises(c.safe.OracleError): c.vm_rows(c.canonical(changed).decode(), work, self.rows)

    def test_independent_four_fields_and_VM_diagnostic_each_prevent_false_match(self):
        expected = c.predicted(self.rows[0]); self.assertTrue(c.comparison(expected, expected, expected)['passed'])
        for key, value in [('stdout', 'wrong'), ('stderr', 'wrong'), ('status', 1), ('files', {'/work/trace.txt': 'wrong'}), ('error', 'Unsupported')]:
            vm = {**expected, key: value}; self.assertFalse(c.comparison(expected, vm, expected)['passed'])
            if key != 'error': self.assertFalse(c.comparison(expected, expected, {**expected, key: value})['passed'])

    def test_exact_approval_has_no_extra_omission_or_false_review_bypass(self):
        control = {'capture': c.CAPTURE, 'root_reviewed': False, 'pins': []}
        c.approved(c.canonical({**control, 'root_reviewed': True}), control)
        for bad in [control, {**control, 'root_reviewed': True, 'extra': True}, {'root_reviewed': True}]:
            with self.assertRaises(c.safe.OracleError): c.approved(c.canonical(bad), control)
        with self.assertRaises(c.safe.OracleError): c.strict(b'{"x":1,"x":2}')

    def test_flat_files_preserve_exact_text_and_reject_unknown_alias_directory_or_oversize(self):
        work = self.root / 'work'; work.mkdir(); self.put('rows.txt', 'one\nno-final-newline', work)
        self.assertEqual(c.closed_files(work), {'/work/rows.txt': 'one\nno-final-newline'})
        for name in ['unknown', 'inner.txt']:
            path = self.put(name, b'x' * (4097 if name == 'inner.txt' else 1), work)
            with self.assertRaises(c.safe.OracleError): c.closed_files(work)
            path.unlink()
        path = work / 'inner.txt'; path.symlink_to(work / 'rows.txt')
        with self.assertRaises(c.safe.OracleError): c.closed_files(work)
        path.unlink(); path.mkdir()
        with self.assertRaises(c.safe.OracleError): c.closed_files(work)

    def test_stripped_C_environment_and_startup_override_refusal(self):
        env = c.safe.closed_environment({}, self.root)
        self.assertEqual(env['PATH'], ''); self.assertEqual(env['LC_ALL'], 'C'); self.assertEqual(env['LANG'], 'C')
        self.assertFalse(set(env) & {'HOME', 'BASH_ENV', 'ENV', 'SHELLOPTS', 'CDPATH'})
        for override in [{'LC_ALL': 'C'}, {'BASH_ENV': '/outside'}, {'ENV': 'x'}]:
            with self.assertRaises(c.safe.OracleError): c.safe.closed_environment(override, self.root)

    def test_child_result_status_persisted_before_later_files_failure_no_fake_acceptance(self):
        out = c.safe.private_dir(self.root, 'out'); budget = c.safe.Budget(c.EVIDENCE_TOTAL)
        def simulated(command, folder, work, env, data, budget, cap, **kwargs):
            self.assertEqual(data, b''); self.assertEqual(kwargs['inherited'], (7, 8)); self.assertFalse(kwargs.get('start_new_session', False))
            self.assertEqual(cap, c.STREAM); self.assertEqual(kwargs['timeout'], 5)
            c.safe.write_new(folder / 'stdout.raw', b'partial diagnostic', budget, cap)
            c.safe.write_new(folder / 'stderr.raw', b'failure', budget, cap)
            (work / 'unknown').write_bytes(b'bad')
            return {'stdout': 'partial diagnostic', 'stderr': 'failure', 'status': 7}
        with patch.object(c.safe, 'capture_command', side_effect=simulated), patch.object(c.safe, 'quiet_group'), patch.object(c.safe, 'publish') as publish:
            with self.assertRaises(c.safe.OracleError): c.run_child(out, 'case0', ['synthetic-only-no-process'], {'files': {}, 'stdin': ''}, budget, (7, 8), c.time.monotonic())
            publish.assert_not_called()
        result = c.strict((out / 'case0/RESULT.json').read_bytes())
        self.assertEqual(result['result']['status'], 7); self.assertEqual(result['result']['stderr'], 'failure')
        self.assertEqual((out / 'case0/stdout.raw').read_bytes(), b'partial diagnostic')

    def test_invalid_write_path_or_expired_deadline_refused_before_child(self):
        out = c.safe.private_dir(self.root, 'out'); budget = c.safe.Budget(c.EVIDENCE_TOTAL)
        with patch.object(c.safe, 'capture_command') as launch:
            with self.assertRaises(c.safe.OracleError): c.run_child(out, 'bad', ['never'], {'files': {'/personal/x': 'bad'}, 'stdin': ''}, budget, (), c.time.monotonic())
            with self.assertRaises(c.safe.OracleError): c.run_child(out, 'late', ['never'], {'files': {}, 'stdin': ''}, budget, (), c.time.monotonic() - c.DEADLINE - 1)
            launch.assert_not_called()

    def test_arbitrary_cli_or_abbreviated_mode_refused_before_entry(self):
        for args in [['--mode', 'capture', '--source', 'anything'], ['--mo', 'prepare'], ['--mode', 'prepare', '--probe', '/outside'], ['--mode', 'other']]:
            with patch.object(c.plumbing, 'main_entry') as enter:
                with self.assertRaises(SystemExit): c.main(args)
                enter.assert_not_called()

    def test_local_HEAD_detached_loose_precedence_and_packed_fallback(self):
        a, b, z = 'a' * 40, 'b' * 40, 'c' * 40
        self.put('.git/HEAD', 'ref: refs/heads/main\n')
        loose = self.put('.git/refs/heads/main', b + '\n')
        self.put('.git/packed-refs', '# pack-refs with: peeled fully-peeled sorted\n' + a + ' refs/heads/main\n' + z + ' refs/tags/v1\n^' + b + '\n')
        self.assertEqual(c.source_identity(self.root), b)
        loose.unlink(); self.assertEqual(c.source_identity(self.root), a)
        self.put('.git/HEAD', z + '\n'); self.assertEqual(c.source_identity(self.root), z)

    def test_local_HEAD_unsafe_or_malformed_requested_or_loose_refs_refuse(self):
        for head in ['ref: ../outside', 'ref: /outside', 'ref: refs/heads/../outside',
                     'ref: refs/heads/main.lock', 'ref: refs/heads//main',
                     'ref: refs/heads/main\nextra', 'g' * 40, 'a' * 40 + '\0',
                     ' ' + 'a' * 40, 'ref: refs/tags/not-head']:
            self.put('.git/HEAD', head + '\n')
            with self.assertRaises(c.safe.OracleError): c.source_identity(self.root)
        self.put('.git/HEAD', 'ref: refs/heads/main\n')
        self.put('.git/packed-refs', 'a' * 40 + ' refs/heads/main\n')
        for text in ['', 'b' * 39, 'b' * 40 + '\nextra', 'ref: refs/heads/other']:
            self.put('.git/refs/heads/main', text)
            with self.assertRaises(c.safe.OracleError): c.source_identity(self.root)

    def test_local_packed_duplicates_malformed_records_peels_and_absence_refuse(self):
        a, b = 'a' * 40, 'b' * 40
        self.put('.git/HEAD', 'ref: refs/heads/main\n')
        records = [a + ' refs/heads/main\n' + b + ' refs/heads/main\n',
                   a + ' refs/heads/other\n' + b + ' refs/heads/other\n' + a + ' refs/heads/main\n',
                   a + ' refs/heads/main extra\n', a[:-1] + ' refs/heads/main\n',
                   a + ' refs/heads/../main\n', '^' + a + '\n' + b + ' refs/heads/main\n',
                   a + ' refs/tags/v1\n^' + b + '\n^' + a + '\n' + b + ' refs/heads/main\n',
                   a + ' refs/heads/other\n', a + '\0 refs/heads/main\n',
                   '# ' + 'x' * 513 + '\n', (a + ' refs/heads/main\n') * 4097]
        for text in records:
            self.put('.git/packed-refs', text)
            with self.assertRaises(c.safe.OracleError): c.source_identity(self.root)

    def test_local_HEAD_ref_and_packed_aliases_never_fall_back_or_follow(self):
        target = self.put('ordinary-hash.txt', 'a' * 40 + '\n')
        head = self.put('.git/HEAD', 'ref: refs/heads/main\n')
        self.put('.git/packed-refs', 'a' * 40 + ' refs/heads/main\n')
        head.unlink(); head.symlink_to(target)
        with self.assertRaises(OSError): c.source_identity(self.root)
        head.unlink(); head.write_text('ref: refs/heads/main\n')
        loose = self.root / '.git/refs/heads/main'; loose.parent.mkdir(parents=True)
        loose.symlink_to(target)
        with self.assertRaises(OSError): c.source_identity(self.root)
        loose.unlink(); packed = self.root / '.git/packed-refs'; packed.unlink(); packed.symlink_to(target)
        with self.assertRaises(OSError): c.source_identity(self.root)

    def test_local_packed_non_LF_control_separators_are_rejected_not_split_into_records(self):
        a, b = 'a' * 40, 'b' * 40
        self.put('.git/HEAD', 'ref: refs/heads/main\n')
        for separator in [b'\x0b', b'\x0c', b'\x1c', b'\x1d', b'\x1e', b'\r', b'\t', b'\0', b'\x7f']:
            text = a.encode() + b' refs/heads/main' + separator + b.encode() + b' refs/heads/other\n'
            self.put('.git/packed-refs', text)
            with self.assertRaises(c.safe.OracleError): c.source_identity(self.root)
            self.put('.git/packed-refs', b'# comment' + separator + b'\n' + a.encode() + b' refs/heads/main\n')
            with self.assertRaises(c.safe.OracleError): c.source_identity(self.root)
        self.put('.git/packed-refs', '# pack-refs with: peeled\n' + b + ' refs/tags/v1\n^' + a + '\n' + a + ' refs/heads/main\n')
        self.assertEqual(c.source_identity(self.root), a)

    def test_local_HEAD_and_loose_hash_only_remove_one_LF_never_other_control_bytes(self):
        a = 'a' * 40
        self.assertEqual(c.git_line(a.encode()), a)
        self.assertEqual(c.git_line(a.encode() + b'\n'), a)
        with self.assertRaises(c.safe.OracleError): c.git_line(a.encode() + b'\n\n')
        for byte in [*range(10), *range(11, 32), 127]:
            control = bytes([byte])
            for raw in [control + a.encode(), a.encode() + control, a[:20].encode() + control + a[20:].encode()]:
                self.put('.git/HEAD', raw + b'\n')
                with self.assertRaises(c.safe.OracleError): c.source_identity(self.root)
                self.put('.git/HEAD', 'ref: refs/heads/main\n')
                self.put('.git/refs/heads/main', raw + b'\n')
                with self.assertRaises(c.safe.OracleError): c.source_identity(self.root)
