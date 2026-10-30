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
            root = Path(temporary).resolve()
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
            root = Path(temporary).resolve()
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
    def test_worktree_cannot_reuse_main_candidate_target(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            with patch.object(candidate, 'ROOT', root / 'worktree'), patch.object(candidate, 'project_root', return_value=root), patch.object(candidate, 'safe_run') as run:
                with self.assertRaisesRegex(ValueError, 'main checkout'):
                    candidate.compile_native('cargo', 'test-native-target', 'app', {})
                run.assert_not_called()
            self.assertFalse((root / 'target').exists())

    def test_target_overrides_cannot_package_a_stale_host_binary(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            stale = root / 'target/release/dlms-learning-lab'
            stale.parent.mkdir(parents=True)
            stale.write_text('stale executable')
            target = 'test-native-target'
            expected = root / 'target/candidate-build' / target / 'release/dlms-learning-lab'
            def compiler(args, **kwargs):
                self.assertEqual(args[args.index('--target') + 1], target)
                self.assertEqual(Path(args[args.index('--target-dir') + 1]), root / 'target/candidate-build')
                self.assertEqual(args[args.index('--jobs') + 1], '2')
                self.assertEqual(kwargs['env']['CARGO_BUILD_JOBS'], '2')
                self.assertEqual(kwargs['env']['RUST_TEST_THREADS'], '2')
                self.assertNotIn('CARGO_BUILD_TARGET', kwargs['env'])
                self.assertNotIn('CARGO_TARGET_DIR', kwargs['env'])
                expected.parent.mkdir(parents=True)
                expected.write_text('fresh native executable')
            with patch.object(candidate, 'ROOT', root), patch.object(candidate, 'safe_run', side_effect=compiler), patch.object(candidate, 'project_root', return_value=root):
                actual = candidate.compile_native('cargo', target, 'dlms-learning-lab', {'CARGO_BUILD_TARGET': 'other', 'CARGO_TARGET_DIR': '/elsewhere'})
            self.assertEqual(actual, expected)
            self.assertEqual(actual.read_text(), 'fresh native executable')
            self.assertEqual(stale.read_text(), 'stale executable')

    def test_content_identity_excludes_uncompiled_private_json(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            (root / 'src').mkdir()
            (root / 'scenarios').mkdir()
            (root / 'src/pack.rs').write_text('include_str!("../scenarios/public.json")')
            (root / 'src/course.rs').write_text('include_str!("../scenarios/units.json")')
            for name in ['public.json', 'units.json', 'private-local-progress.json']:
                (root / 'scenarios' / name).write_text('{}')
            with patch.object(candidate, 'ROOT', root):
                result = candidate.embedded_content_hashes()
            self.assertEqual(set(result), {'public.json', 'units.json'})


class CandidateReadme(unittest.TestCase):
    VERSION = '0.1.0'
    REVISION = 'b' * 40
    PLATFORMS = (
        ('Linux', 'x86_64-unknown-linux-gnu', 'dlms-learning-lab', '',
         ('install.sh', 'uninstall.sh'), './dlms-learning-lab'),
        ('Windows', 'x86_64-pc-windows-msvc', 'dlms-learning-lab.exe', '',
         ('Install.ps1',), '.\\dlms-learning-lab.exe'),
        ('Darwin', 'aarch64-apple-darwin',
         'DLMS Learning Lab.app/Contents/MacOS/dlms-learning-lab',
         'DLMS Learning Lab.app/Contents/Resources/',
         ('INSTALL.txt', 'DLMS Learning Lab.app/Contents/Info.plist'),
         'DLMS Learning Lab.app'),
    )

    def stage(self, root, layout):
        _, _, binary, prefix, extras, _ = layout
        stage = root / 'extracted candidate'
        stage.mkdir()
        for relative in (binary, *extras, *(prefix + name for name in (
            'BUILD_INFO.json', 'docs/LEARNER_GUIDE.md', 'docs/INSTALLATION.md',
            'LICENSE', 'NOTICE',
        ))):
            path = stage / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('tiny disposable resource; never executed', encoding='utf-8')
        return stage

    def test_relative_instructions_match_all_native_layouts_and_never_copy_old_repository_paths(self):
        import re
        for layout in self.PLATFORMS:
            system, target, binary, prefix, extras, launch = layout
            with self.subTest(system=system), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                repository = root / 'repository'
                repository.mkdir()
                old = ('$HOME/Documents/dlms_learning_lab/dist/'
                       'dlms-learning-lab-0.1.0-g5233a151bfe3-old-candidate/dlms-learning-lab\n')
                (repository / 'README.md').write_text(old)
                stage = self.stage(root, layout)
                before = {path.relative_to(stage): path.read_bytes()
                          for path in stage.rglob('*') if path.is_file()}
                with patch.object(candidate, 'ROOT', repository), \
                     patch.object(candidate, 'command') as command, \
                     patch.object(candidate, 'safe_run') as run, \
                     patch.object(candidate, 'compile_native') as compiler, \
                     patch.object(candidate, 'collect') as notices:
                    candidate.write_candidate_readme(stage, system, self.VERSION,
                                                     self.REVISION, target)
                    command.assert_not_called()
                    run.assert_not_called()
                    compiler.assert_not_called()
                    notices.assert_not_called()
                text = (stage / 'README.md').read_text(encoding='utf-8')
                self.assertIn(self.VERSION, text)
                self.assertIn(self.REVISION, text)
                self.assertIn(target, text)
                self.assertIn(launch, text)
                self.assertTrue((stage / binary).is_file())
                for relative in extras:
                    self.assertTrue((stage / relative).is_file())
                for obsolete in ('$HOME', '/home/', 'Documents/', 'g5233a151bfe3',
                                 './dist/', 'cargo build', 'target/debug'):
                    self.assertNotIn(obsolete, text)
                links = re.findall(r'\]\(<([^>]+)>\)', text)
                self.assertEqual(len(links), 5)
                for relative in links:
                    self.assertFalse(Path(relative).is_absolute())
                    self.assertNotIn('..', Path(relative).parts)
                    self.assertTrue((stage / relative).is_file(), relative)
                    self.assertTrue(relative.startswith(prefix))
                self.assertIn('separate application-data folder', text)
                self.assertIn('development milestone candidate', text)
                self.assertIn('Building a package does not establish runtime', text)
                self.assertEqual((repository / 'README.md').read_text(), old)
                self.assertEqual({name: (stage / name).read_bytes() for name in before}, before)

    def test_missing_or_wrong_platform_launch_entry_cannot_generate_misleading_readme(self):
        for layout in self.PLATFORMS:
            system, target, binary, _, _, _ = layout
            with self.subTest(system=system), tempfile.TemporaryDirectory() as temporary:
                stage = self.stage(Path(temporary), layout)
                (stage / binary).unlink()  # This test's own tiny fixture only.
                (stage / 'wrong-platform-executable').write_text('never run')
                with self.assertRaisesRegex(ValueError, 'entry missing'):
                    candidate.write_candidate_readme(stage, system, self.VERSION,
                                                     self.REVISION, target)
                self.assertFalse((stage / 'README.md').exists())
        with tempfile.TemporaryDirectory() as temporary:
            stage = self.stage(Path(temporary), self.PLATFORMS[0])
            with self.assertRaisesRegex(ValueError, 'platform does not match'):
                candidate.write_candidate_readme(stage, 'Windows', self.VERSION,
                                                 self.REVISION, 'x86_64-unknown-linux-gnu')
            self.assertFalse((stage / 'README.md').exists())

    def test_existing_candidate_readme_and_invalid_identity_are_retained(self):
        with tempfile.TemporaryDirectory() as temporary:
            stage = self.stage(Path(temporary), self.PLATFORMS[0])
            for version, revision, target in (
                ('0.1.0\nother', self.REVISION, 'x86_64-unknown-linux-gnu'),
                (self.VERSION, 'not-a-reviewed-revision', 'x86_64-unknown-linux-gnu'),
                (self.VERSION, self.REVISION, '../outside/linux'),
            ):
                with self.assertRaisesRegex(ValueError, 'ordinary build identity'):
                    candidate.write_candidate_readme(stage, 'Linux', version, revision, target)
                self.assertFalse((stage / 'README.md').exists())
            path = stage / 'README.md'
            path.write_text('retain this existing candidate exactly')
            with self.assertRaises(FileExistsError):
                candidate.write_candidate_readme(stage, 'Linux', self.VERSION,
                                                 self.REVISION, 'x86_64-unknown-linux-gnu')
            self.assertEqual(path.read_text(), 'retain this existing candidate exactly')

    @unittest.skipIf(sys.platform == 'win32', 'link creation requires separate Windows privilege acceptance')
    def test_launch_or_intermediate_resource_alias_is_refused_before_readme_write(self):
        for relative in ['dlms-learning-lab', 'install.sh', 'docs']:
            with self.subTest(relative=relative), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                stage = self.stage(root, self.PLATFORMS[0])
                path = stage / relative
                retained = root / 'retained fixture'
                path.rename(retained)
                path.symlink_to(retained, target_is_directory=retained.is_dir())
                with self.assertRaisesRegex(ValueError, 'resource links'):
                    candidate.write_candidate_readme(stage, 'Linux', self.VERSION,
                                                     self.REVISION, 'x86_64-unknown-linux-gnu')
                self.assertFalse((stage / 'README.md').exists())
                self.assertTrue(retained.exists())
