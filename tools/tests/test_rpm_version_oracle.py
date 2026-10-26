"""Fixed rpmsort guard tests. No real child or package utility is ever started.

Pipe tests reuse fake process objects; full collector tests replace its only
child boundary and retained executable context. Fixture observations stay empty.
"""
from contextlib import contextmanager
import hashlib
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import rpm_version_oracle as oracle
from test_case_bash_oracle import FakeProcess, Clock, EmptySelector


@unittest.skipUnless(sys.platform.startswith('linux'), 'fixed Linux development oracle')
class RpmVersionOracle(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='rpm-oracle-guards-')
        self.root = Path(self.temp.name).resolve()
        self.raw, self.rows = oracle.load_fixed(ROOT)

    def tearDown(self):
        self.temp.cleanup()  # Only fresh tiny fixtures owned by this test.

    def test_exact_twenty_reviewed_pairs_and_no_invented_observations(self):
        self.assertEqual(hashlib.sha256(self.raw).hexdigest(), oracle.FIXED_SHA)
        self.assertEqual(len(self.rows), 20)
        self.assertEqual(self.rows[2]['a'], '1.01')
        self.assertEqual(self.rows[7]['a'], '1.0^git1')
        self.assertEqual(self.rows[-1]['b'], '1.0.0')
        self.assertEqual(json.loads(self.raw)['observations'], [])

    def test_changed_fixture_rejected_before_parse_or_child(self):
        with patch.object(oracle.safe, 'read_bytes', return_value=self.raw + b' '), \
             patch.object(oracle, 'strict_json', side_effect=AssertionError('do not parse')), \
             patch.object(oracle.safe, 'capture_command') as child:
            with self.assertRaisesRegex(oracle.safe.OracleError, 'fixed20'):
                oracle.load_fixed(ROOT)
            child.assert_not_called()

    def test_secondary_fixture_guards_reject_paths_options_evr_unicode_and_fake_rows(self):
        for text in ('--help', '1:2', '1-2', '/tmp/never', 'a b', 'x\n', 'é', '', 'x' * 129):
            value = json.loads(self.raw)
            value['cases'][0]['a'] = text
            raw = json.dumps(value).encode()
            with patch.object(oracle, 'FIXED_SHA', hashlib.sha256(raw).hexdigest()), \
                 patch.object(oracle.safe, 'read_bytes', return_value=raw):
                with self.assertRaises(oracle.safe.OracleError, msg=repr(text)):
                    oracle.load_fixed(ROOT)
        for change in ('observation', 'extra', 'count', 'order', 'boolean'):
            value = json.loads(self.raw)
            if change == 'observation': value['observations'] = [{'invented': True}]
            elif change == 'extra': value['cases'][0]['program'] = '/never'
            elif change == 'count': value['cases'].pop()
            elif change == 'order': value['cases'].reverse()
            else: value['cases'][0]['docs_expected_order'] = True
            raw = json.dumps(value).encode()
            with patch.object(oracle, 'FIXED_SHA', hashlib.sha256(raw).hexdigest()), \
                 patch.object(oracle.safe, 'read_bytes', return_value=raw):
                with self.assertRaises(oracle.safe.OracleError): oracle.load_fixed(ROOT)

    def test_duplicate_json_keys_fail_closed(self):
        with self.assertRaisesRegex(oracle.safe.OracleError, 'Duplicate'):
            oracle.strict_json(b'{"scope":1,"scope":2}')

    def test_no_caller_program_source_path_flags_or_self_test_admitted(self):
        with patch.object(oracle, 'main_guard') as guard:
            for args in ([], ['--source', 'x'], ['--binary', '/never'], ['--output', '/never'],
                         ['--version'], ['--self-test'], ['--capture', 'x'],
                         ['--capture-id', '../escape'], ['--capture-id', 'x/y'],
                         ['--capture-id', 'ok', '--', 'extra'],
                         ['--capture-id', 'a', '--capture-id', 'b']):
                with self.assertRaises((SystemExit, oracle.safe.OracleError)):
                    oracle.main(args)
            guard.assert_not_called()

    def test_main_guard_failure_prevents_all_capture(self):
        with patch.object(oracle, 'main_guard', side_effect=oracle.safe.OracleError('no lock')), \
             patch.object(oracle, 'capture') as capture:
            with self.assertRaisesRegex(oracle.safe.OracleError, 'no lock'):
                oracle.main(['--capture-id', 'guard-test'])
            capture.assert_not_called()

    def test_worktree_or_foreign_target_refused_before_children(self):
        (self.root / '.git').write_text('gitdir: worktree')
        with patch.object(oracle, 'ROOT', self.root), patch.object(oracle.safe, 'require_guard') as guard:
            with self.assertRaisesRegex(oracle.safe.OracleError, 'MAIN'):
                oracle.main_guard(self.root)
            guard.assert_not_called()
        (self.root / '.git').unlink(); (self.root / '.git').mkdir()
        with patch.object(oracle, 'ROOT', self.root), \
             patch.object(oracle.safe, 'require_guard', return_value=(9,)), \
             patch.dict(os.environ, {'CARGO_TARGET_DIR': '/foreign'}):
            with self.assertRaisesRegex(oracle.safe.OracleError, 'default target'):
                oracle.main_guard(self.root)

    def test_environment_never_inherits_personal_macro_loader_proxy_or_home(self):
        hostile = {'HOME': '/personal', 'RPM_CONFIGDIR': '/personal', 'RPMRC': '/personal',
                   'LD_PRELOAD': '/never', 'http_proxy': 'http://never', 'PATH': '/user/bin'}
        with patch.dict(os.environ, hostile):
            env = oracle.environment(self.root)
        self.assertEqual(set(env), {'HOME', 'PATH', 'LC_ALL', 'LANG', 'TMPDIR', 'TMP', 'TEMP',
                                    'XDG_CONFIG_HOME', 'XDG_CACHE_HOME', 'XDG_DATA_HOME'})
        self.assertEqual(env['PATH'], '')
        self.assertEqual(env['LC_ALL'], 'C')
        self.assertEqual(env['TMP'], env['TMPDIR'])
        for key in ('HOME', 'TMPDIR', 'XDG_CONFIG_HOME', 'XDG_CACHE_HOME', 'XDG_DATA_HOME'):
            path = Path(env[key]); self.assertTrue(path.is_relative_to(self.root))
            self.assertEqual(path.stat().st_mode & 0o777, 0o700)
            self.assertEqual(list(path.iterdir()), [])

    def test_remaining_budget_is_positive_and_never_resets_child_deadline(self):
        with patch.object(oracle.time, 'monotonic', return_value=29.75):
            self.assertEqual(oracle.remaining(0), .25)
        with patch.object(oracle.time, 'monotonic', return_value=31):
            with self.assertRaisesRegex(oracle.safe.OracleError, 'Whole'):
                oracle.remaining(0)

    def test_actual_pin_aggregate_growth_rejected_after_small_stat_preflight(self):
        @contextmanager
        def ordinary(path, cap):
            class Info: st_size = 1
            yield None, Info()
        with patch.object(oracle, 'SOURCE_FILES', ('one', 'two')), \
             patch.object(oracle, 'PROVENANCE', {}), patch.object(oracle, 'PIN_CAP', 10), \
             patch.object(oracle, 'remaining', return_value=2), \
             patch.object(oracle.safe, 'ordinary', ordinary), \
             patch.object(oracle.safe, 'pin', side_effect=lambda p, cap: {
                 'path': str(p), 'bytes': 6, 'sha256': '0' * 64}):
            with self.assertRaisesRegex(oracle.safe.OracleError, 'Actual aggregate'):
                oracle.source_pins(self.root, 0)

    def test_elf_pin_and_provenance_cannot_auto_repin(self):
        @contextmanager
        def ordinary(path, cap):
            class Info: st_size = 1
            yield None, Info()
        with patch.object(oracle, 'SOURCE_FILES', ()), patch.object(oracle, 'PROVENANCE', {}), \
             patch.object(oracle, 'remaining', return_value=2), \
             patch.object(oracle.safe, 'ordinary', ordinary), \
             patch.object(oracle.safe, 'pin', return_value={'path': str(oracle.ELF),
                                                         'bytes': 1, 'sha256': '0' * 64}):
            with self.assertRaisesRegex(oracle.safe.OracleError, 'ELF changed'):
                oracle.source_pins(self.root, 0)

    def test_changed_private_provenance_is_not_accepted_as_new_review(self):
        @contextmanager
        def ordinary(path, cap):
            class Info: st_size = 1
            yield None, Info()
        with patch.object(oracle, 'SOURCE_FILES', ()), \
             patch.object(oracle, 'PROVENANCE', {'audit.json': '1' * 64}), \
             patch.object(oracle, 'remaining', return_value=2), \
             patch.object(oracle.safe, 'ordinary', ordinary), \
             patch.object(oracle.safe, 'pin', side_effect=lambda p, cap: {
                 'path': str(p), 'bytes': 1, 'sha256': '0' * 64}):
            with self.assertRaisesRegex(oracle.safe.OracleError, 'provenance changed'):
                oracle.source_pins(self.root, 0)

    def test_retained_executable_fd_and_path_change_rejected_without_execution(self):
        path = self.root / 'fake-elf'; path.write_bytes(b'\x7fELFtiny-not-executable')
        path.chmod(0o755)
        pin = oracle.safe.pin(path, 100)
        with patch.object(oracle, 'ELF', path), patch.object(oracle, 'ELF_PIN', pin):
            with oracle.executable(pin) as (name, fd):
                self.assertEqual(name, '/proc/self/fd/' + str(fd))
                self.assertEqual(os.fstat(fd).st_ino, path.stat().st_ino)
            with self.assertRaises(oracle.safe.OracleError):
                with oracle.executable({**pin, 'sha256': '0' * 64}): pass
            with self.assertRaises(oracle.safe.OracleError):
                with oracle.executable(pin):
                    path.write_bytes(b'\x7fELFtiny-CHANGED-exe')

    def test_strict_orders_are_observations_and_mismatches_do_not_rewrite_oracle(self):
        row = self.rows[6]
        good = {'stdout': '1.0~rc1\n1.0\n', 'stderr': '', 'status': 0}
        bad = {'stdout': '1.0\n1.0~rc1\n', 'stderr': '', 'status': 0}
        for order in oracle.ORDERS:
            self.assertTrue(oracle.observation(row, order, good)['strict_sort_compatible'])
            self.assertFalse(oracle.observation(row, order, bad)['strict_sort_compatible'])
        self.assertEqual(oracle.load_fixed(ROOT)[0], self.raw)

    def test_both_tie_orders_never_claim_equality_or_strict_compatibility(self):
        row = self.rows[2]
        for text in ('1.01\n1.1\n', '1.1\n1.01\n'):
            r = oracle.observation(row, 'a-b', {'stdout': text, 'stderr': '', 'status': 0})
            self.assertIsNone(r['strict_sort_compatible'])
            self.assertIsNone(r['comparator_equality_observed'])

    def test_malformed_or_failed_process_result_is_not_an_observation(self):
        for actual in ({'stdout': '1.0\n1.0~rc1\n', 'stderr': '', 'status': -9},
                       {'stdout': '1.0\n1.0~rc1\n', 'stderr': 'error', 'status': 0},
                       {'stdout': '1.0\n', 'stderr': '', 'status': 0},
                       {'stdout': '1.0\nextra\n', 'stderr': '', 'status': 0},
                       {'stdout': '1.0\n1.0~rc1', 'stderr': '', 'status': 0},
                       {'stdout': '1.0\r\n1.0~rc1\r\n', 'stderr': '', 'status': 0}):
            with self.assertRaises(oracle.safe.OracleError):
                oracle.observation(self.rows[6], 'a-b', actual)

    def fake_capture(self, *, fail=False, drift=False, deadline=False):
        base = self.root / 'oracle'; base.mkdir(mode=0o700)
        outputs = []
        @contextmanager
        def executable(_): yield '/fake/held-fd', 19
        def child(command, folder, work, env, data, budget, cap, **kw):
            outputs.append((command, env, data, kw))
            self.assertEqual(command, ['/fake/held-fd'])
            self.assertEqual(kw['inherited'], (7, 19))
            self.assertLessEqual(kw['timeout'], 2)
            self.assertGreater(kw['timeout'], 0)
            self.assertLessEqual(len(data), 258)
            labels = data.decode().splitlines()
            row = self.rows[(len(outputs) - 1) // 2]
            text = ('\n'.join([row['a'], row['b']] if row['docs_expected_order'] <= 0
                              else [row['b'], row['a']]) + '\n')
            oracle.safe.write_new(folder / 'stdout.raw', text.encode(), budget, cap)
            oracle.safe.write_new(folder / 'stderr.raw', b'', budget, cap)
            if fail: raise oracle.safe.OracleError('fake child failed after output')
            self.assertEqual(sorted(labels), sorted([row['a'], row['b']]))
            return {'stdout': text, 'stderr': '', 'status': 0}
        pins = [{'path': '/fake/source', 'bytes': 1, 'sha256': '0' * 64}]
        if drift: pins_after = [{**pins[0], 'sha256': '1' * 64}]
        else: pins_after = pins
        with patch.object(oracle, 'load_fixed', return_value=(self.raw, self.rows)), \
             patch.object(oracle, 'source_pins', side_effect=[pins, pins_after]), \
             patch.object(oracle.safe, 'source_identity', return_value='0' * 40), \
             patch('build_safety.owned_directory', return_value=base), \
             patch.object(oracle, 'executable', executable), \
             patch.object(oracle.safe, 'capture_command', side_effect=child), \
             patch.object(oracle.safe, 'quiet_group'), \
             patch.object(oracle, 'remaining', side_effect=(oracle.safe.OracleError('expired')
                                                          if deadline else lambda _: .5)):
            result = oracle.capture(self.root, 'fixed-test', (7,))
        return result, outputs

    def test_full_fake_collection_uses_exact_argv_stdin_env_forty_children_and_complete(self):
        result, seen = self.fake_capture()
        self.assertEqual(len(seen), 40)
        out = self.root / 'oracle/captures/fixed-test'
        manifest = oracle.safe.pin(out / 'MANIFEST.json', oracle.MANIFEST_CAP)
        self.assertEqual((out / 'COMPLETE').read_text(), manifest['sha256'] + '\n')
        value = json.loads((out / 'MANIFEST.json').read_bytes())
        self.assertEqual(value['child_count'], 40)
        self.assertIsNone(value['comparator_equality_observed'])
        self.assertTrue(value['strict_cases_compatible'])
        for output in value['outputs']:
            self.assertEqual(oracle.safe.pin(Path(output['path']), oracle.MANIFEST_CAP), output)
        with self.assertRaises(FileExistsError):
            oracle.safe.private_dir(out.parent, out.name)
        self.assertEqual(oracle.load_fixed(ROOT)[0], self.raw)

    def test_failed_child_retains_partial_output_without_complete_or_next_child(self):
        with self.assertRaisesRegex(oracle.safe.OracleError, 'fake child failed'):
            self.fake_capture(fail=True)
        out = self.root / 'oracle/captures/fixed-test'
        self.assertTrue((out / 'component-01-a-b/stdout.raw').exists())
        self.assertFalse((out / 'component-01-b-a').exists())
        self.assertFalse((out / 'COMPLETE').exists())

    def test_changed_source_after_children_refuses_complete_and_retains_raws(self):
        with self.assertRaisesRegex(oracle.safe.OracleError, 'identity changed'):
            self.fake_capture(drift=True)
        out = self.root / 'oracle/captures/fixed-test'
        self.assertTrue((out / 'component-20-b-a/stdout.raw').exists())
        self.assertFalse((out / 'COMPLETE').exists())

    def test_expired_whole_deadline_prevents_first_child(self):
        with self.assertRaisesRegex(oracle.safe.OracleError, 'expired'):
            self.fake_capture(deadline=True)
        out = self.root / 'oracle/captures/fixed-test'
        self.assertTrue((out / 'STARTED.json').exists())
        self.assertFalse((out / 'component-01-a-b').exists())
        self.assertFalse((out / 'COMPLETE').exists())

    def test_expired_last_boundary_retains_manifest_but_never_complete(self):
        out = oracle.safe.private_dir(self.root, 'publish')
        with patch.object(oracle, 'remaining', side_effect=oracle.safe.OracleError('expired')):
            with self.assertRaisesRegex(oracle.safe.OracleError, 'expired'):
                oracle.publish(out, {'status': 'fake'}, oracle.safe.Budget(1024), 0)
        self.assertTrue((out / 'MANIFEST.json').exists())
        self.assertTrue((out / 'COMPLETION-PREPARED').exists())
        self.assertFalse((out / 'COMPLETE').exists())

    def test_fake_pipe_stream_limit_and_aggregate_limit_retain_partial_files(self):
        for count in (False, True):
            folder = oracle.safe.private_dir(self.root, 'aggregate' if count else 'stream')
            work = oracle.safe.private_dir(folder, 'work')
            process = FakeProcess(stdout=b'0123456789')
            with self.assertRaisesRegex(oracle.safe.OracleError, 'ceiling'):
                oracle.safe.capture_command(['fake-only'], folder, work, {}, b'',
                    oracle.safe.Budget(5 if count else 100), 100 if count else 5,
                    popen=lambda *a, **kw: process)
            self.assertTrue((folder / 'stdout.raw').exists())
            self.assertTrue(process.stdin.closed)
            self.assertTrue(process.stdout.closed)

    def test_fake_running_child_timeout_and_held_pipe_are_reaped_without_real_signals(self):
        for held in (False, True):
            folder = oracle.safe.private_dir(self.root, 'held' if held else 'timeout')
            work = oracle.safe.private_dir(folder, 'work')
            process = FakeProcess(running=not held, held=held)
            try:
                with self.assertRaises(oracle.safe.OracleError):
                    oracle.safe.capture_command(['fake-only'], folder, work, {}, b'',
                        oracle.safe.Budget(100), 100, clock=Clock(step=1), timeout=2,
                        popen=lambda *a, **kw: process, select_factory=EmptySelector)
                self.assertTrue(process.waits)
                self.assertTrue(process.stdout.closed)
                self.assertEqual(process.terminated, 0 if held else 1)
            finally:
                process.close_writers()



if __name__ == '__main__':
    unittest.main()
