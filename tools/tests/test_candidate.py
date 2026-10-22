import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
spec = importlib.util.spec_from_file_location('candidate', ROOT / 'tools/build_candidate.py')
candidate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(candidate)


class CandidateResources(unittest.TestCase):
    def test_only_tracked_documentation_is_bundled(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'docs').mkdir()
            (root / 'docs/guide.md').write_text('public instructions')
            (root / 'docs/private.log').write_text('must never be bundled')
            with patch.object(candidate, 'ROOT', root), patch.object(candidate.subprocess, 'check_output', return_value=b'docs/guide.md\0'):
                candidate.copy_tracked_tree('docs', root / 'out')
            self.assertEqual((root / 'out/guide.md').read_text(), 'public instructions')
            self.assertFalse((root / 'out/private.log').exists())

    @unittest.skipIf(sys.platform == 'win32', 'link creation requires separate Windows privilege acceptance')
    def test_intermediate_source_link_is_rejected_even_when_target_is_inside_repository(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'local-data').mkdir()
            (root / 'local-data/guide.md').write_text('private progress')
            (root / 'docs').symlink_to(root / 'local-data', target_is_directory=True)
            with patch.object(candidate, 'ROOT', root), patch.object(candidate.subprocess, 'check_output', return_value=b'docs/guide.md\0'):
                with self.assertRaisesRegex(ValueError, 'source links'):
                    candidate.copy_tracked_tree('docs', root / 'out')
            self.assertFalse((root / 'out').exists())


if __name__ == '__main__':
    unittest.main()

class NativeProvenance(unittest.TestCase):
    def test_target_overrides_cannot_package_a_stale_host_binary(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            stale = root / 'target/release/dlms-learning-lab'
            stale.parent.mkdir(parents=True)
            stale.write_text('stale executable')
            target = 'test-native-target'
            expected = root / 'target/candidate-build' / target / 'release/dlms-learning-lab'
            def compiler(args, **kwargs):
                self.assertEqual(args[args.index('--target') + 1], target)
                self.assertEqual(Path(args[args.index('--target-dir') + 1]), root / 'target/candidate-build')
                self.assertNotIn('CARGO_BUILD_TARGET', kwargs['env'])
                self.assertNotIn('CARGO_TARGET_DIR', kwargs['env'])
                expected.parent.mkdir(parents=True)
                expected.write_text('fresh native executable')
            with patch.object(candidate, 'ROOT', root), patch.object(candidate.subprocess, 'run', side_effect=compiler):
                actual = candidate.compile_native('cargo', target, 'dlms-learning-lab', {'CARGO_BUILD_TARGET': 'other', 'CARGO_TARGET_DIR': '/elsewhere'})
            self.assertEqual(actual, expected)
            self.assertEqual(actual.read_text(), 'fresh native executable')
            self.assertEqual(stale.read_text(), 'stale executable')

    def test_content_identity_excludes_uncompiled_private_json(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'src').mkdir()
            (root / 'scenarios').mkdir()
            (root / 'src/pack.rs').write_text('include_str!("../scenarios/public.json")')
            (root / 'src/course.rs').write_text('include_str!("../scenarios/units.json")')
            for name in ['public.json', 'units.json', 'private-local-progress.json']:
                (root / 'scenarios' / name).write_text('{}')
            with patch.object(candidate, 'ROOT', root):
                result = candidate.embedded_content_hashes()
            self.assertEqual(set(result), {'public.json', 'units.json'})
