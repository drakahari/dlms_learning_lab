"""Synthetic guard fixtures; never invokes a historical producer or learner work."""
import copy
import importlib.util
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
spec = importlib.util.spec_from_file_location('before17', TOOLS / 'scripting_before17_verify.py')
v = importlib.util.module_from_spec(spec); spec.loader.exec_module(v)
spec = importlib.util.spec_from_file_location('before17_dev', TOOLS / 'dev.py')
dev = importlib.util.module_from_spec(spec); spec.loader.exec_module(dev)


class Genuine16Guards(unittest.TestCase):
    def fixture(self):
        directory = tempfile.TemporaryDirectory(); self.addCleanup(directory.cleanup)
        return Path(directory.name)

    def revision(self, n):
        root = self.fixture(); (root / 'src').mkdir()
        (root / 'src/shell.rs').write_text('pub const EVALUATOR_REVISION: u32 = ' + str(n) + ';\n')
        return root

    def test_actual16_feature_required_and_later_replay_refused(self):
        for flags in [['--features', 'historical-evaluator16-capture'], ['--features=historical-evaluator16-capture'],
                      ['-Fhistorical-evaluator16-capture'], ['-F=historical-evaluator16-capture'],
                      ['-F', 'other,dlms-learning-lab/historical-evaluator16-capture']]:
            dev.historical_capture_preflight(['--example=scripting_before17_capture', *flags], self.revision(16))
            for n in [15, 17]:
                with self.assertRaisesRegex(RuntimeError, 'actual public evaluator16'):
                    dev.historical_capture_preflight(flags, self.revision(n))
        with self.assertRaisesRegex(RuntimeError, 'requires --features'):
            dev.historical_capture_preflight(['--example', 'scripting_before17_capture'], self.revision(16))

    def test_normal_default_checks_do_not_select_old_evaluators(self):
        with patch.object(Path, 'open', side_effect=AssertionError('No source read needed')):
            for flags in [['--all-targets'], ['--examples'], ['--', 'historical-evaluator16-capture']]:
                dev.historical_capture_preflight(flags, Path('/synthetic'))
        with self.assertRaises(RuntimeError): dev.historical_capture_preflight(['--all-features'], self.revision(16))

    def test_revision_declaration_must_be_unique_exact_and_bounded(self):
        v.current16(self.revision(16))
        for n in [15, 17]:
            with self.assertRaises(ValueError): v.current16(self.revision(n))
        root = self.revision(16)
        for text in ['', 'pub const EVALUATOR_REVISION: u32 = 16;\n' * 2]:
            (root / 'src/shell.rs').write_text(text)
            with self.assertRaises(ValueError): v.current16(root)

    def test_paths_and_file_pins_refuse_aliases_and_large_inputs(self):
        root = self.fixture()
        for name in ['', '/', '../x', './x', 'x//a', 'x/../a', 1]:
            with self.assertRaises(ValueError): v.contained(root, name)
        (root / 'a').write_bytes(b'synthetic')
        (root / 'alias').symlink_to(root / 'a')
        with self.assertRaises(ValueError): v.pin(root, 'alias')
        with self.assertRaises(ValueError): v.pin(root, 'a', 2)
        self.assertEqual(v.pin(root, 'a')['size'], 9)

    def test_exact_control_cannot_self_approve_or_change_source(self):
        import json
        c = dict(actual_evaluator_revision=16, root_reviewed=False, pins=['synthetic'])
        v.approved(json.dumps({**c, 'root_reviewed': True}), c)
        for bad in [c, {**c, 'root_reviewed': 1}, {**c, 'root_reviewed': True, 'extra': True},
                    {**c, 'root_reviewed': True, 'pins': []}]:
            with self.assertRaises(ValueError): v.approved(json.dumps(bad), c)
        with self.assertRaises(ValueError): v.strict('{"a":1,"a":2}')

    def test_outcome_requires_every_stream_status_and_file_without_error(self):
        case = dict(stdout='yes\n', stderr='', status=0, after={'/work/data': 'kept'})
        actual = dict(stdout='yes\n', stderr='', status=0, files={'/work/data': 'kept'}, error=None)
        self.assertTrue(v.outcome(case, actual))
        for key, bad in [('stdout', ''), ('stderr', 'error'), ('status', False), ('status', 1), ('files', {}), ('error', 'unsupported')]:
            self.assertFalse(v.outcome(case, {**actual, key: bad}))
        self.assertFalse(v.outcome(case, {**actual, 'extra': True}))

    def test_record_inventory_detects_relabeling_and_missing_history(self):
        r = dict(engine_revision=16, source='synthetic', results=[])
        p = {'attempts': {'a': {'state': dict(kind='Script', run=r),
                              'workspace': {'script_history': r, 'script_history_more': [r]}}}}
        original = v.records(p)
        self.assertEqual(sum(original.values()), 3)
        q = copy.deepcopy(p); q['attempts']['a']['state']['run']['engine_revision'] = 17
        self.assertNotEqual(v.records(q), original)
        q = copy.deepcopy(p); q['attempts']['a']['workspace']['script_history_more'] = []
        self.assertNotEqual(v.records(q), original)

    def test_existing_capture_and_retained_source_are_not_overwritten(self):
        root = self.fixture(); path = root / 'retained.json'
        v.create(path, {'synthetic': True}); before = path.read_bytes()
        with self.assertRaises(FileExistsError): v.create(path, {'different': True})
        self.assertEqual(path.read_bytes(), before)

    def test_only_the_exact_cargo_binary_allows_one_hashed_hardlink(self):
        root = self.fixture(); path = root / v.BINARY; path.parent.mkdir(parents=True)
        path.write_bytes(b'synthetic executable bytes')
        alias = path.with_name(path.name + '-0123456789abcdef')
        os.link(path, alias)
        self.assertEqual(v.pin(root, v.BINARY)['size'], path.stat().st_size)
        third = path.with_name('unexpected'); os.link(path, third)
        with self.assertRaises(ValueError): v.pin(root, v.BINARY)
        other = root / 'other'; other.write_bytes(b'not a compiler output')
        os.link(other, root / 'other-link')
        with self.assertRaises(ValueError): v.pin(root, 'other')

    def test_producer_binds_its_actual_self_include_list_and_static_index(self):
        source = (TOOLS.parent / 'examples/scripting_before17_capture.rs').read_text()
        self.assertIn('COMPILED.iter().chain(EVIDENCE_BINDINGS)', source)
        for name in ['scripting_before17_capture.rs', 'support/before17_inputs.rs', '../tests/fixtures/bash-before17-source-index.json']:
            self.assertIn('include_bytes!("' + name + '")', source)

    def test_static_index_preserves_prior_scripts_and_names_the_actual_input_writer(self):
        import json
        root = TOOLS.parent
        prior = json.loads((root / 'tests/fixtures/bash-before16-source-index.json').read_text())
        current = json.loads((root / 'tests/fixtures/bash-before17-source-index.json').read_text())
        self.assertEqual(current['catalog'][:255], prior['catalog'])
        self.assertEqual(current['scripts'][:66], prior['scripts'])
        self.assertEqual(set(current['prior']), {'writer15-original.json', 'writer15-drafts.json'})
        self.assertEqual((len(current['catalog']), len(current['scripts'])), (257, 68))
        self.assertEqual(sum(len(r['cases']) for r in current['scripts']), 573)
        authored = json.loads((root / 'scenarios/shell-tracing-lessons-v1.json').read_text())['scenarios']
        for row, lab in zip(current['scripts'][66:], authored, strict=True):
            self.assertEqual(row, dict(id=lab['id'], version=lab['version'], source=lab['solution'][0],
                                      cases=lab['goal']['cases'], example_source=lab['teaching']['example'],
                                      example_case=lab['teaching']['fixture']))
        verifier = (TOOLS / 'scripting_before17_verify.py').read_text()
        self.assertIn("index['prior']['writer15-' + stem + '.json']", verifier)
        self.assertNotIn("index['prior']['writer14-' + stem + '.json']", verifier)


if __name__ == '__main__':
    unittest.main()
