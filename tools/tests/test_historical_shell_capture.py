"""Unrun prospective guards: no compiler/probe/VM/process child is started."""
import importlib.util
from contextlib import ExitStack
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch, MagicMock
import sys

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
spec = importlib.util.spec_from_file_location('historical_capture_dev', TOOLS / 'dev.py')
dev = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dev)


class HistoricalCaptureGuards(unittest.TestCase):
    def fixture(self, revision=9, extra=''):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        (root / 'src').mkdir()
        (root / 'src/shell.rs').write_text(f'pub const EVALUATOR_REVISION: u32 = {revision};\n' + extra)
        return root

    def test_explicit_revision9_requires_feature_and_accepts_closed_forms(self):
        root = self.fixture()
        for flags in [
            ['--features', 'historical-evaluator9-capture'],
            ['--features=historical-evaluator9-capture'],
            ['-Fhistorical-evaluator9-capture'],
            ['-F=historical-evaluator9-capture'],
            ['-F', 'historical-evaluator9-capture'],
            ['--features', 'other,dlms-learning-lab/historical-evaluator9-capture'],
            ['--all-features'],
        ]:
            with self.subTest(flags=flags):
                dev.historical_capture_preflight(['--example', 'automation_before10_capture', *flags], root)
        with self.assertRaisesRegex(RuntimeError, 'requires --features'):
            dev.historical_capture_preflight(['--example=automation_before10_capture'], root)

    def test_revision10_rejects_explicit_target_feature_and_all_features(self):
        root = self.fixture(10)
        for flags in [
            ['--example', 'automation_before10_capture'],
            ['--example=automation_before10_capture'],
            ['--features', 'other historical-evaluator9-capture'],
            ['--features=dlms-learning-lab/historical-evaluator9-capture'],
            ['-Fhistorical-evaluator9-capture'],
            ['-F=historical-evaluator9-capture'],
            ['--all-features'],
        ]:
            with self.subTest(flags=flags), self.assertRaisesRegex(RuntimeError, 'actual public evaluator9'):
                dev.historical_capture_preflight(flags, root)

    def test_ordinary_all_targets_and_test_filter_do_not_select_historical_example(self):
        root = self.fixture(10)
        with patch.object(Path, 'open', side_effect=AssertionError('No historical source admission needed')):
            for flags in [['--all-targets'], ['--examples'], ['--features', 'other'],
                          ['--all-targets', '--', 'historical-evaluator9-capture']]:
                dev.historical_capture_preflight(flags, root)

    def test_ambiguous_missing_and_oversized_revision_declarations_fail(self):
        root = self.fixture(9, 'pub const EVALUATOR_REVISION: u32 = 10;\n')
        for raw in [b'', b'// public9', (root / 'src/shell.rs').read_bytes(), b'x' * (512 * 1024 + 1)]:
            (root / 'src/shell.rs').write_bytes(raw)
            with self.subTest(length=len(raw)), self.assertRaises(RuntimeError):
                dev.historical_capture_preflight(['--all-features'], root)

    def test_missing_option_values_fail_without_a_child(self):
        root = self.fixture()
        for flags in [['--features'], ['-F'], ['--example']]:
            with self.subTest(flags=flags), self.assertRaises(RuntimeError):
                dev.historical_capture_preflight(flags, root)

    def test_actual_entry_rejects_before_compiler_selection_or_any_child(self):
        root = self.fixture(10)
        session = MagicMock()
        session.temporary = root / 'tmp'
        session.log = root / 'memory.jsonl'
        context = MagicMock()
        context.__enter__.return_value = session
        with ExitStack() as stack:
            stack.enter_context(patch.object(sys, 'argv', ['dev.py', '--cwd', str(root), 'test', '--all-features']))
            stack.enter_context(patch.object(dev, 'project_root', return_value=root))
            stack.enter_context(patch.object(dev, 'owned_directory', side_effect=lambda root, path: path))
            stack.enter_context(patch.object(dev, 'workflow', return_value=context))
            stack.enter_context(patch.object(dev, 'disk_directory'))
            stack.enter_context(patch.object(dev, 'build_target', return_value=root / 'target'))
            # A private cache marker allows entry into the ordinary Cargo branch;
            # no global environment persists beyond this synthetic test.
            cache = root / '.build-artifacts/toolchains'
            (cache / 'cargo').mkdir(parents=True)
            (cache / 'rustup').mkdir()
            (cache / 'READY.json').write_text('{}')
            stack.enter_context(patch.dict(dev.os.environ, {}, clear=False))
            compiler = stack.enter_context(patch.object(dev.shutil, 'which'))
            with self.assertRaisesRegex(RuntimeError, 'actual public evaluator9'):
                dev.main()
            compiler.assert_not_called()
            session.run.assert_not_called()
            sys.argv = ['dev.py', '--cwd', str(root), 'test', '--all-targets',
                        '-F=historical-evaluator9-capture']
            with self.assertRaisesRegex(RuntimeError, 'actual public evaluator9'):
                dev.main()
            compiler.assert_not_called()
            session.run.assert_not_called()


if __name__ == '__main__':
    unittest.main()
