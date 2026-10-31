"""Synthetic guards only. Never execute a probe, grep, Bash or learner input."""
import contextlib
import copy
import io
import json
import stat
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import shell_ere_reference as c

PROJECT = Path(__file__).resolve().parents[2]
# Named CURRENT QA extension, not inputs observed by the sealed r1 ELF.
# r1 COMPILED396/SOURCES409/collector/probe stay immutable. Fresh r1 capture
# must still refuse these paths until a separately reviewed new collector.
CURRENT_NATIVE_QA_INPUTS = (('src/ui/smoke/ere_evidence.rs', 81869, 'dfdad4ccab3a8457672d56e66a857e70e55a3cc9e2d629e289f3a346fe1daeff'), ('src/ui/tests/ere_evidence_native.rs', 30841, '1d5dc8f59e89802af17b2fcbb7c137ee5dfa650674e6b7800877412c2fc2bdc6'))

# Named CURRENT product-workspace extension; never sealed r1 compiled evidence.
CURRENT_STORAGE_WORKSPACE_INPUTS = (('src/partition_fs.rs', 34808, 'a769056e923a5dc932727bafaae52ed81e4d7c2e88f28dc867ab975406022ea9'), ('src/partition_fs_workspace.rs', 43668, '138c38aa3529c05136aa27d4caf8612e3c98a18b120ab30fe158370cddbf2a2e'), ('src/partition_fs_workspace/binding.rs', 16588, '7d4496e9ba9e4ba318bcd8176d8a6ade5d88b451258347ac855e748ce48dfd52'), ('src/ui/partition_fs_workspace.rs', 41812, '0623d6b4840158ef4d3a8f13baad663d7bda49d9d54321be426508fa74268cd5'), ('src/ui/tests/partition_fs_workspace.rs', 62096, 'aac1752f0289b6f99281423216383d4366286e8ae531d26fe7a265b38dd98441'))

# Fixed current authored Pack only; not observed by the sealed r1 probe.
CURRENT_AUTHORED_STORAGE_INPUTS = (('scenarios/disk-preparation-v1.json', 118384, '6971f6001d8cd9f3e58d28ef708ffecb6a5f4fdd1ad0f12b3d9606f07bf7dcb1'),)

# Two fixed conditional native QA modules; no sealed r1 observation claim.
CURRENT_STORAGE_NATIVE_INPUTS = (('src/ui/smoke/disk_preparation.rs', 159702, 'd97c9d25e9b8bd091b7ceccfda7424d6e41e6f6bb04a6d14e2cb3b3b9d0b3e68'), ('src/ui/tests/disk_preparation_native.rs', 15789, '76bf6dfe81b3b9b3a92a881521cb4448c55240576b898cacf9920d52ec45b6f5'))

CURRENT_LITERAL_PRODUCT_INPUTS = (('src/shell/literal_trim.rs', 9256, '9917307a858e06a7592811b49b466f9a5211a303cf915422a16520b83cea3f69'),)

# Exact unregistered authored Pack for CURRENT QA only; never a frozen r1 input.
CURRENT_LITERAL_AUTHORED_INPUTS = (('scenarios/shell-literal-normalization-draft.json', 41797, 'bd7149e93384eb8be68aac9cb68486fc1630248b13cab0ff04b78b393c7e1535'),)

class ShellEreReferenceGuards(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix='dlms-ere24-guards-')
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.raw = (PROJECT / c.FIXTURE).read_bytes()
        self.fixture = c.strict(self.raw)
        self.put(c.FIXTURE, self.raw)

    def put(self, name, raw):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(raw.encode() if isinstance(raw, str) else raw)
        return path

    def expected(self):
        chain = self.root / '.build-artifacts/toolchains/rustup/toolchains/stable-x86_64-unknown-linux-gnu'
        paths = [str(self.root / n) for n in c.SOURCES] + [str(chain / 'bin/rustc'),
            str(chain / 'lib/rustlib/multirust-channel-manifest.toml'),
            str(self.root / '.build-artifacts/toolchains/READY.json'),
            str(Path(sys.executable).resolve(strict=True)), str(self.root / c.PROBE),
            str(c.shared.GREP), str(c.shared.MANUAL)]
        hashes = {str(self.root / n): sha for n, sha in c.CORE_PINS.items()}
        hashes[str(self.root / c.FIXTURE)] = c.FIXTURE_SHA
        hashes[str(self.root / c.PROBE_SOURCE)] = c.PROBE_SOURCE_SHA
        pins = [{'path': p, 'bytes': 1, 'sha256': hashes.get(p, '0' * 64)} for p in paths]
        with patch.object(c.safe, 'source_identity', return_value='a' * 40):
            return c.control(self.root, pins)

    def probe(self, expected=None):
        expected = expected or self.expected()
        outcomes = []
        for row in self.fixture['cases']:
            predicted = row['expected']
            outcomes.append({'id': row['id'], 'row_sha256': c.digest(c.canonical(row)),
                'outcome': {**predicted, 'files': {'/work/' + k: v for k, v in predicted['files'].items()},
                            'error': None}})
        return {'identity': c.identity(self.root, expected), 'outcomes': outcomes,
                'GNU_exclusions': self.fixture['vm_only']}

    def test_exact24_null_predictions_and_all_six_reason_only_rows(self):
        self.assertEqual(c.fixed(self.root), self.fixture)
        self.assertEqual(len(c.IDS), 24)
        self.assertEqual(len(self.fixture['vm_only']), 6)
        for row in self.fixture['cases']:
            self.assertIsNone(row['observed'])
        for field, value in [('cases', list(reversed(self.fixture['cases']))),
                             ('host_execution', 'fabricated'), ('vm_only', [])]:
            self.put(c.FIXTURE, c.canonical({**self.fixture, field: value}))
            with patch.object(c.safe, 'capture_command') as child:
                with self.assertRaises(c.safe.OracleError): c.fixed(self.root)
                child.assert_not_called()

    def test_contained_leaf_files_allow_literal_dash_but_refuse_escape_and_large_data(self):
        self.assertEqual(c.files({'-': 'one\n', '-records': 'two\n'}), {'-': 'one\n', '-records': 'two\n'})
        for value in [{'.': ''}, {'..': ''}, {'/absolute': ''}, {'a/b': ''}, {'bad\nname': ''},
                      {'a': '\0'}, {'a': 'x' * 4097}, {'a': '', 'b': '', 'c': ''}, {'a': 1}]:
            with self.assertRaises(c.safe.OracleError): c.files(value)

    def test_only_exact_supported_fixed_argv_reaches_held_gnu(self):
        rows = self.fixture['cases']
        for row in rows:
            self.assertEqual(c.host_argv('/proc/self/fd/71', row, rows), ['/proc/self/fd/71', *row['argv'][1:]])
        for bad in [dict(rows[0], argv=['sh', '-c', 'anything']),
                    dict(rows[0], stdin='changed'), self.fixture['vm_only'][0]]:
            with self.assertRaises(c.safe.OracleError): c.host_argv('/proc/self/fd/71', bad, rows)
        for binary in ['/bin/sh', '/usr/bin/grep', '/proc/self/fd/not-a-number']:
            with self.assertRaises(c.safe.OracleError): c.host_argv(binary, rows[0], rows)

    def test_actual_probe_identity_every_input_and_all_outcomes_required(self):
        expected = self.expected()
        good = self.probe(expected)
        self.assertEqual(len(c.parse_probe(c.canonical(good), self.root, self.fixture, expected)), 24)
        for field, value in [('engine_revision', 13), ('compiled_sources', {}), ('manifest_dir', '/foreign')]:
            bad = copy.deepcopy(good); bad['identity'][field] = value
            with self.assertRaises(c.safe.OracleError): c.parse_probe(c.canonical(bad), self.root, self.fixture, expected)
        for outcomes in [good['outcomes'][:-1], list(reversed(good['outcomes'])), good['outcomes'] + good['outcomes'][:1]]:
            with self.assertRaises(c.safe.OracleError): c.parse_probe(c.canonical({**good, 'outcomes': outcomes}), self.root, self.fixture, expected)
        for field, value in [('id', 'foreign'), ('row_sha256', '0' * 64)]:
            bad = copy.deepcopy(good); bad['outcomes'][0][field] = value
            with self.assertRaises(c.safe.OracleError): c.parse_probe(c.canonical(bad), self.root, self.fixture, expected)

    def test_probe_status_and_virtual_files_are_typed_bounded_and_errors_do_not_pass(self):
        expected = self.expected(); good = self.probe(expected)
        for field, value in [('status', True), ('files', {'/etc/personal': ''}), ('files', []),
                             ('stdout', 1), ('stdout', 'x' * 16385), ('error', 1)]:
            bad = copy.deepcopy(good); bad['outcomes'][0]['outcome'][field] = value
            with self.assertRaises(c.safe.OracleError): c.parse_probe(c.canonical(bad), self.root, self.fixture, expected)
        bad = copy.deepcopy(good); bad['outcomes'][0]['outcome']['error'] = 'Honest simulator limitation'
        actual = c.parse_probe(c.canonical(bad), self.root, self.fixture, expected)[0]['outcome']
        self.assertFalse(c.compared(self.fixture['cases'][0], actual,
            {k: self.fixture['cases'][0]['expected'][k] for k in ('stdout', 'stderr', 'status')}, {})['passed'])

    def test_full_stdout_stderr_status_files_and_prediction_are_independently_compared(self):
        row = self.fixture['cases'][-1]
        expected = row['expected']
        vm = {**expected, 'files': {'/work/' + k: v for k, v in expected['files'].items()}, 'error': None}
        host = {k: expected[k] for k in ('stdout', 'stderr', 'status')}
        self.assertTrue(c.compared(row, vm, host, row['files'])['passed'])
        for k, v in [('stdout', 'wrong'), ('stderr', 'failure'), ('status', 1)]:
            self.assertFalse(c.compared(row, vm, {**host, k: v}, row['files'])['passed'])
        self.assertFalse(c.compared(row, vm, host, {})['passed'])
        bothwrong = {**vm, 'stdout': 'same wrong text'}
        self.assertFalse(c.compared(row, bothwrong, {**host, 'stdout': 'same wrong text'}, row['files'])['passed'])

    def test_control_requires_all_exact_typed_source_compiler_executable_pins(self):
        good = self.expected(); self.assertEqual(len(c.check_control(self.root, good)), len(good['pins']))
        for pins in [good['pins'][:-1], good['pins'] + good['pins'][:1], [dict(good['pins'][0], bytes=True)] + good['pins'][1:]]:
            with self.assertRaises(c.safe.OracleError): c.check_control(self.root, {**good, 'pins': pins})
        for field, value in [('compiled_sources', {}), ('host_rows', True), ('main_head', 'wrong'), ('root', '/other')]:
            with self.assertRaises(c.safe.OracleError): c.check_control(self.root, {**good, field: value})
        bad = copy.deepcopy(good); bad['pins'][0]['sha256'] = 'f' * 64
        with self.assertRaises(c.safe.OracleError): c.check_control(self.root, bad)

    def test_exact_root_approval_and_duplicate_keys_are_required(self):
        expected = self.expected()
        c.shared.approve(c.canonical({**expected, 'root_reviewed': True}), expected)
        for bad in [expected, {**expected, 'root_reviewed': 1}, {**expected, 'root_reviewed': True, 'extra': 1}]:
            with self.assertRaises(c.safe.OracleError): c.shared.approve(c.canonical(bad), expected)
        with self.assertRaises(c.safe.OracleError): c.strict(b'{"status":0,"status":1}')

    def test_prepare_launches_no_children_and_missing_approval_refuses_capture(self):
        expected = self.expected()
        with patch.object(c.shared, 'root_guard', return_value=(91,)), \
             patch.object(c, 'source_pins', return_value=expected['pins']), \
             patch.object(c, 'control', return_value=expected), patch.object(c.safe, 'capture_command') as child:
            with contextlib.redirect_stdout(io.StringIO()): self.assertEqual(c.run(self.root, 'prepare'), 0)
            child.assert_not_called()
            proposed = self.root / c.BASE / (c.CAPTURE + '-PROPOSED.json')
            self.assertIs(c.strict(proposed.read_bytes())['root_reviewed'], False)
            self.assertFalse((self.root / c.BASE / c.CAPTURE).exists())
            with self.assertRaises(FileNotFoundError): c.run(self.root, 'capture')
            child.assert_not_called()
            with self.assertRaises(FileExistsError): c.run(self.root, 'prepare')

    def test_prior_capture_is_retained_and_never_overwritten(self):
        out = self.root / c.BASE / c.CAPTURE; out.mkdir(parents=True)
        diagnostic = out / 'diagnostic'; diagnostic.write_bytes(b'preserve')
        expected = self.expected()
        with patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c, 'source_pins', return_value=expected['pins']), \
             patch.object(c, 'control', return_value=expected), patch.object(c.safe, 'capture_command') as child:
            with self.assertRaises(c.safe.OracleError): c.run(self.root, 'prepare')
            child.assert_not_called()
        self.assertEqual(diagnostic.read_bytes(), b'preserve')

    def test_cli_has_no_source_binary_pattern_or_path_selector(self):
        with patch.object(c, 'run', return_value=0) as run:
            self.assertEqual(c.main(['--mode', 'prepare']), 0)
            for args in [['--mode', 'capture', '--source', 'anything'], ['--mode', 'capture', '--capture-id', 'other'],
                         ['--mode', 'capture', '--binary', '/bin/sh'], ['--mode', 'unknown']]:
                with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit): c.main(args)
            self.assertEqual(run.call_count, 1)

    def test_gnu_version_matches_exact_held_descriptor_status_and_framing(self):
        good = {'stdout': '71 (GNU grep) 3.12\nLicense information\n', 'stderr': '', 'status': 0}
        c.shared.require_gnu_version(good, '/proc/self/fd/71')
        for bad in [dict(good, status=True), dict(good, status=1), dict(good, stderr='warning'),
                    dict(good, stdout='grep (GNU grep) 3.12\n'), dict(good, stdout='72 (GNU grep) 3.12\n'),
                    dict(good, stdout='71 (GNU grep) 3.12\r\n')]:
            with self.assertRaises(c.safe.OracleError): c.shared.require_gnu_version(bad, '/proc/self/fd/71')

    def test_child_timeout_retains_unknown_status_and_no_completion(self):
        out = self.root / 'capture'; out.mkdir()
        with patch.object(c.safe, 'capture_command', side_effect=c.safe.OracleError('SYNTHETIC timeout')):
            with self.assertRaises(c.safe.OracleError):
                c.child(out, 'case0', ['/proc/self/fd/71'], b'', {}, c.safe.Budget(c.CAP), (), c.time.monotonic())
        status = c.strict((out / 'case0/STATUS.json').read_bytes())
        self.assertIs(status['complete'], False); self.assertIsNone(status['status'])
        self.assertFalse((out / 'COMPLETE').exists())

    def test_snapshot_preserves_final_lf_utf8_and_refuses_extra_symlink_or_hardalias(self):
        work = self.root / 'work'; work.mkdir()
        (work / '-').write_bytes('é\r\nlast'.encode())
        self.assertEqual(c.snapshot(work, {'-': 'é\r\nlast'}), {'-': 'é\r\nlast'})
        (work / 'extra').write_bytes(b'')
        with self.assertRaises(c.safe.OracleError): c.snapshot(work, {'-': 'é\r\nlast'})
        (work / 'extra').unlink(); (work / '-').unlink()
        (self.root / 'other').write_bytes(b'private synthetic')
        (work / '-').symlink_to(self.root / 'other')
        with self.assertRaises(OSError): c.snapshot(work, {'-': ''})
        (work / '-').unlink()
        import os
        os.link(self.root / 'other', work / '-')
        with self.assertRaises(c.safe.OracleError): c.snapshot(work, {'-': ''})

    def test_raw_result_complete_status_and_integer_agreement_are_required(self):
        folder = self.root / 'probe'; folder.mkdir()
        (folder / 'stdout.raw').write_bytes(b'data\n'); (folder / 'stderr.raw').write_bytes(b'')
        (folder / 'RESULT.json').write_bytes(c.canonical({'stdout': 'data\n', 'stderr': '', 'status': 0}))
        (folder / 'STATUS.json').write_bytes(c.canonical({'complete': True, 'status': 0}))
        self.assertEqual(c.retained_result(self.root, 'probe')['stdout'], 'data\n')
        for value in [{'complete': False, 'status': 0}, {'complete': True, 'status': True},
                      {'complete': True, 'status': 1}, {'complete': True, 'status': 0, 'extra': 1}]:
            (folder / 'STATUS.json').write_bytes(c.canonical(value))
            with self.assertRaises(c.safe.OracleError): c.retained_result(self.root, 'probe')
        (folder / 'STATUS.json').write_bytes(c.canonical({'complete': True, 'status': 0}))
        (folder / 'stdout.raw').write_bytes(b'changed\n')
        with self.assertRaises(c.safe.OracleError): c.retained_result(self.root, 'probe')

    def test_evidence_inventory_refuses_extra_files_and_symbolic_components(self):
        out = self.root / 'out'; out.mkdir(); (out / 'file').write_bytes(b'data')
        c.artifact_inventory(out, {'file'})
        with self.assertRaises(c.safe.OracleError): c.artifact_inventory(out, set())
        (out / 'link').symlink_to(self.root / c.FIXTURE)
        with self.assertRaises(c.safe.OracleError): c.artifact_inventory(out, {'file', 'link'})

    def test_six_exclusions_never_turn_reason_metadata_into_host_calls_or_passes(self):
        with patch.object(c.safe, 'capture_command') as child:
            for row in self.fixture['vm_only']:
                self.assertIn('reason', row)
                with self.assertRaises(c.safe.OracleError): c.host_argv('/proc/self/fd/71', row, self.fixture['cases'])
            child.assert_not_called()
        self.assertEqual(len(c.output_paths(self.fixture)), len(set(c.output_paths(self.fixture))))
        self.assertTrue(all('/work/' in p or not p.endswith('-records') for p in c.output_paths(self.fixture)))

    def test_readonly_verifier_checks_full_synthetic_retained_capture_and_rejects_tampering(self):
        # Synthetic provenance only: no actual evaluator/GNU observation is claimed.
        with patch.object(c, 'CORE_PINS', {}), patch.object(c, 'PROBE_SOURCE_SHA', c.digest(b'SYNTHETIC source')):
            expected = self.expected()
            table = {p['path']: p for p in expected['pins']}
            out = self.root / c.BASE / c.CAPTURE; out.mkdir(parents=True)
            source = out / 'source'; source.mkdir()
            for i, name in enumerate(c.SOURCES):
                raw = self.raw if name == c.FIXTURE else b'SYNTHETIC source'
                (source / f'file{i}.raw').write_bytes(raw)
                table[str(self.root / name)].update(bytes=len(raw), sha256=c.digest(raw))
            (source / 'INDEX.json').write_bytes(c.canonical(list(c.SOURCES)))
            for name, original in [('probe.elf', str(self.root / c.PROBE)), ('grep.elf', str(c.shared.GREP))]:
                raw = ('SYNTHETIC ' + name).encode(); (out / name).write_bytes(raw)
                table[original].update(bytes=len(raw), sha256=c.digest(raw))
            expected['compiled_sources'] = {n: table[str(self.root / n)]['sha256'] for n in c.COMPILED}
            c.check_control(self.root, expected)
            approved = c.canonical({**expected, 'root_reviewed': True})
            approval = self.root / c.BASE / (c.CAPTURE + '-APPROVED.json'); approval.write_bytes(approved)
            (out / 'CONTROL.json').write_bytes(approved)
            probe = self.probe(expected)
            observations = {
                'probe-identity': {'stdout': c.canonical(c.identity(self.root, expected)).decode(), 'stderr': '', 'status': 0},
                'probe-outcomes': {'stdout': c.canonical(probe).decode(), 'stderr': '', 'status': 0},
                'grep-version': {'stdout': '71 (GNU grep) 3.12\n', 'stderr': '', 'status': 0}}
            comparisons = []
            for i, (row, vm) in enumerate(zip(self.fixture['cases'], probe['outcomes'])):
                result = {k: row['expected'][k] for k in ('stdout', 'stderr', 'status')}
                observations[f'case{i}'] = result
                comparison = c.compared(row, vm['outcome'], result, row['files'])
                comparisons.append(comparison)
                (out / f'row{i}.json').write_bytes(c.canonical(comparison))
            for name, observed in observations.items():
                folder = out / name; folder.mkdir(); (folder / 'work').mkdir()
                for key in ('stdout', 'stderr'): (folder / (key + '.raw')).write_bytes(observed[key].encode())
                (folder / 'RESULT.json').write_bytes(c.canonical(observed))
                (folder / 'STATUS.json').write_bytes(c.canonical({'complete': True, 'status': observed['status']}))
                if name.startswith('case'):
                    for filename, text in self.fixture['cases'][int(name[4:])]['files'].items():
                        (folder / 'work' / filename).write_bytes(text.encode())
            manifest = {'schema_version': 1, 'control': expected, 'GNU_descriptor': '/proc/self/fd/71', 'accepted': True,
                'claims': expected['scope'], 'GNU_exclusions_executed': False, 'limits': {},
                'comparisons': comparisons, 'GNU_exclusions': self.fixture['vm_only'],
                'outputs': [c.evidence_pin(out, path) for path in c.output_paths(self.fixture)]}
            def seal():
                raw = c.canonical(manifest); (out / 'MANIFEST.json').write_bytes(raw)
                for name in ['COMPLETE', 'COMPLETION-PREPARED']:
                    (out / name).write_bytes((c.digest(raw) + '\n').encode())
            seal()
            with patch.object(c.shared, 'root_guard', return_value=()), patch.object(c.safe, 'capture_command') as child:
                self.assertEqual(c.verify(self.root)['status'], 'VERIFIED24-FIXED-UTILITY-COMPARISONS')
                child.assert_not_called()
                original = (out / 'case0/stdout.raw').read_bytes()
                (out / 'case0/stdout.raw').write_bytes(b'forged')
                with self.assertRaises(c.safe.OracleError): c.verify(self.root)
                (out / 'case0/stdout.raw').write_bytes(original)
                approval.write_bytes(c.canonical({**expected, 'root_reviewed': False}))
                with self.assertRaises(c.safe.OracleError): c.verify(self.root)
                approval.write_bytes(approved)
                manifest['accepted'] = False; seal()
                with self.assertRaises(c.safe.OracleError): c.verify(self.root)
                child.assert_not_called()


    def test_real_library_probe_identity_has_the_complete_closed396_input_superset(self):
        import re
        source = (PROJECT / c.PROBE_SOURCE).read_text()
        self.assertIn('use dlms_learning_lab::shell;', source)
        self.assertNotIn('#[path = "../src/shell.rs"]', source)
        self.assertNotIn('#[path = "../src/ere.rs"]', source)
        table = source.split('const COMPILED_INPUTS:', 1)[1].split('fn identity()', 1)[0]
        pairs = re.findall(r'\(\s*"([^"]+)",\s*include_bytes!\(\s*"([^"]+)"\s*\)\s*,?\s*\)', table)
        self.assertEqual(tuple(name for name, _ in pairs), c.COMPILED)
        self.assertTrue(all(include == '../' + name for name, include in pairs))
        self.assertEqual(len(c.COMPILED), 396)
        self.assertEqual(len(c.COMPILED), len(set(c.COMPILED)))
        self.assertEqual(len(c.SOURCES), 409)
        self.assertEqual(len(c.SOURCES), len(set(c.SOURCES)))
        actual = {p.relative_to(PROJECT).as_posix() for family in c.SOURCE_FAMILIES
                  for p in (PROJECT / family).rglob('*') if p.is_file()}
        listed = {name for name in c.COMPILED if Path(name).parts[0] in c.SOURCE_FAMILIES}
        extension = {name for name, _, _ in CURRENT_NATIVE_QA_INPUTS}
        self.assertEqual(len(extension), 2)
        self.assertTrue(listed.isdisjoint(extension))
        storage_extension = {name for name, _, _ in CURRENT_STORAGE_WORKSPACE_INPUTS}
        self.assertEqual(len(storage_extension), 5)
        self.assertTrue((listed | extension).isdisjoint(storage_extension))
        authored_extension = {name for name, _, _ in CURRENT_AUTHORED_STORAGE_INPUTS}
        self.assertEqual(len(authored_extension), 1)
        self.assertTrue((listed | extension | storage_extension).isdisjoint(authored_extension))
        storage_native_extension = {name for name, _, _ in CURRENT_STORAGE_NATIVE_INPUTS}
        self.assertEqual(len(storage_native_extension), 2)
        self.assertTrue((listed | extension | storage_extension | authored_extension).isdisjoint(storage_native_extension))
        literal_extension = {name for name, _, _ in CURRENT_LITERAL_PRODUCT_INPUTS}
        self.assertEqual(len(literal_extension), 1)
        self.assertTrue((listed | extension | storage_extension | authored_extension | storage_native_extension).isdisjoint(literal_extension))
        literal_authored_extension = {name for name, _, _ in CURRENT_LITERAL_AUTHORED_INPUTS}
        self.assertEqual(len(literal_authored_extension), 1)
        self.assertTrue((listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension).isdisjoint(literal_authored_extension))
        actual_before_literal = actual - literal_extension - literal_authored_extension
        # Project each complete predecessor by removing ALL fixed later sets.
        # The final equality still rejects every unknown, omitted or extra path.
        self.assertEqual(listed | extension, actual_before_literal - storage_extension - authored_extension - storage_native_extension)
        self.assertEqual(listed | extension | storage_extension, actual_before_literal - authored_extension - storage_native_extension)
        self.assertEqual(listed | extension | storage_extension | authored_extension, actual_before_literal - storage_native_extension)
        self.assertEqual(listed | extension | storage_extension | authored_extension | storage_native_extension, actual_before_literal)
        self.assertEqual(listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension, actual - literal_authored_extension)
        self.assertEqual(listed | extension | storage_extension | authored_extension | storage_native_extension | literal_extension | literal_authored_extension, actual)
        for name, size, sha in CURRENT_LITERAL_AUTHORED_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name, size, sha in CURRENT_LITERAL_PRODUCT_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name, size, sha in CURRENT_STORAGE_NATIVE_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name, size, sha in CURRENT_AUTHORED_STORAGE_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name, size, sha in CURRENT_STORAGE_WORKSPACE_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        for name, size, sha in CURRENT_NATIVE_QA_INPUTS:
            path = PROJECT / name
            info = path.lstat()
            self.assertTrue(stat.S_ISREG(info.st_mode) and info.st_nlink == 1)
            self.assertEqual(info.st_size, size)
            self.assertEqual(c.digest(path.read_bytes()), sha)
        self.assertTrue({'Cargo.toml', 'Cargo.lock', c.PROBE_SOURCE,
                         'examples/support/automation_digest.rs', c.FIXTURE} <= set(c.COMPILED))
        self.assertEqual(len(c.CORE_PINS), 18)
        self.assertEqual(c.CORE_PINS['src/shell/grep.rs'],
                         'f0b554b0fcbcbdf99e3898e7402f48749be2b01e110398ec32a46f26e1fd7863')

    def test_closed_project_roster_refuses_missing_extra_nested_and_alias_inputs(self):
        import os
        names = ('src/lib.rs', 'scenarios/fixed.json', 'assets/themes/fixed.json')
        for name in names:
            self.put(name, 'SYNTHETIC source, never compiled or executed')
        with patch.object(c, 'COMPILED', names), patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            extra = self.put('src/unreviewed.rs', 'SYNTHETIC extra')
            with self.assertRaises(c.safe.OracleError): c.project_roster(self.root)
            extra.unlink()
            directory = self.root / 'src/unreviewed'; directory.mkdir()
            with self.assertRaises(c.safe.OracleError): c.project_roster(self.root)
            directory.rmdir()
            target = self.root / names[0]; target.unlink()
            with self.assertRaises(c.safe.OracleError): c.project_roster(self.root)
            outside = self.put('outside', 'SYNTHETIC alias')
            target.symlink_to(outside)
            with self.assertRaises(c.safe.OracleError): c.project_roster(self.root)
            target.unlink(); os.link(outside, target)
            with self.assertRaises(c.safe.OracleError): c.project_roster(self.root)
            target.unlink(); self.put(names[0], 'SYNTHETIC restored')
            self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()

    def test_large_retained_snapshot_ignores_current_sources_docs_fixture_and_checks_every_file(self):
        # Reuse the original complete synthetic capture/negative assertions.
        # This creates no real evaluator/GNU observation or executable.
        self.put('src/lib.rs', 'DIFFERENT later current library')
        self.put('docs/SHELL_SUBSET.md', 'DIFFERENT later current docs')
        self.put(c.FIXTURE, 'DIFFERENT later current fixture')
        ordinary_read = c.safe.read_bytes
        def retained_only(path, cap):
            relative = Path(path).relative_to(self.root).as_posix()
            if relative == c.FIXTURE or Path(relative).parts[0] in (*c.SOURCE_FAMILIES, 'docs'):
                raise AssertionError('Verifier reopened later current source/docs/fixture: ' + relative)
            return ordinary_read(path, cap)
        with patch.object(c, 'CORE_PINS', {}), \
             patch.object(c, 'PROBE_SOURCE_SHA', c.digest(b'SYNTHETIC source')), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c, 'project_roster', side_effect=AssertionError('No current-source scan in verify')), \
             patch.object(c.safe, 'read_bytes', side_effect=retained_only), \
             patch.object(c.safe, 'capture_command') as child:
            self.test_readonly_verifier_checks_full_synthetic_retained_capture_and_rejects_tampering()
            out = self.root / c.BASE / c.CAPTURE
            manifest = c.strict((out / 'MANIFEST.json').read_bytes())
            # The old negative finishes accepted=False; restore that SYNTHETIC
            # premise before independent missing/tampered-source controls.
            manifest['accepted'] = True
            raw = c.canonical(manifest); (out / 'MANIFEST.json').write_bytes(raw)
            for name in ('COMPLETE', 'COMPLETION-PREPARED'):
                (out / name).write_bytes((c.digest(raw) + '\n').encode())
            self.assertGreater(len(c.output_paths(self.fixture)), 256)
            self.assertEqual(c.verify(self.root)['status'], 'VERIFIED24-FIXED-UTILITY-COMPARISONS')
            source = out / 'source' / f'file{c.SOURCES.index("src/lib.rs")}.raw'
            original = source.read_bytes(); source.unlink()
            with self.assertRaises((c.safe.OracleError, OSError)): c.verify(self.root)
            source.write_bytes(original + b'SYNTHETIC tampering')
            with self.assertRaises(c.safe.OracleError): c.verify(self.root)
            source.write_bytes(original)
            self.assertEqual(c.verify(self.root)['status'], 'VERIFIED24-FIXED-UTILITY-COMPARISONS')
            child.assert_not_called()


    def test_frozen_r1_prepare_refuses_each_named_current_qa_addition_before_any_child(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_NATIVE_QA_INPUTS:
                path = self.put(name, 'SYNTHETIC named later QA file')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                # Remove only exact newly-created disposable mock paths.
                path.unlink()
                parent = path.parent
                while parent != self.root / 'src':
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()


    def test_frozen_r1_prepare_refuses_each_new_storage_product_input_before_child_or_control(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_STORAGE_WORKSPACE_INPUTS:
                path = self.put(name, 'SYNTHETIC named later storage product file')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                # Remove only exact newly-created disposable mock paths.
                path.unlink()
                parent = path.parent
                while parent != self.root / 'src':
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()



    def test_frozen_r1_prepare_refuses_new_literal_product_before_child_or_control(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_LITERAL_PRODUCT_INPUTS:
                path = self.put(name, 'SYNTHETIC named later literal product file')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                # Remove only exact newly-created disposable mock paths.
                path.unlink()
                parent = path.parent
                while parent != self.root / 'src':
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()



    def test_frozen_r1_prepare_refuses_unregistered_storage_pack_before_child_or_control(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_AUTHORED_STORAGE_INPUTS:
                path = self.put(name, 'SYNTHETIC unregistered storage pack')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                path.unlink()  # Only this exact new disposable mock leaf; family already existed.
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()


    def test_frozen_r1_prepare_refuses_conditional_storage_native_modules_before_children(self):
        names = ('src/lib.rs', 'scenarios/base.json', 'assets/theme.txt')
        for name in names:
            self.put(name, 'SYNTHETIC original reviewed input')
        with patch.object(c, 'COMPILED', names), \
             patch.object(c.shared, 'root_guard', return_value=()), \
             patch.object(c.safe, 'capture_command') as child:
            self.assertEqual(c.project_roster(self.root), set(names))
            for name, _, _ in CURRENT_STORAGE_NATIVE_INPUTS:
                path = self.put(name, 'SYNTHETIC later storage native QA file')
                with self.assertRaises(c.safe.OracleError) as refused:
                    c.run(self.root, 'prepare')
                self.assertRegex(str(refused.exception),
                                 'No unreviewed project source directory|No omitted/extra/alias project source')
                child.assert_not_called()
                self.assertFalse((self.root / c.BASE).exists())
                path.unlink()
                parent = path.parent
                while parent != self.root / 'src':
                    previous = parent
                    parent = parent.parent
                    previous.rmdir()
                self.assertEqual(c.project_roster(self.root), set(names))
            child.assert_not_called()

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
