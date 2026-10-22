"""Disposable installer fixtures; never touches the real user profile."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class LinuxInstallation(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='dlms-install-test-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.package = self.root / 'candidate [1]'
        self.package.mkdir()
        self.prefix = self.root / 'apps' / 'Learning Lab'
        self.bindir = self.root / 'bin'
        self.menu = self.root / 'applications'
        self.env = {**os.environ, 'HOME': str(self.root / 'home'), 'XDG_DATA_HOME': str(self.root / 'data')}
        self.progress = self.root / 'data' / 'dlmslearninglab' / 'progress-v1.json'
        self.progress.parent.mkdir(parents=True)
        self.progress.write_text('learner data, never an installation resource')
        for file in ['install.sh', 'uninstall.sh']:
            shutil.copyfile(ROOT / 'packaging/linux' / file, self.package / file)
        (self.package / 'resources').mkdir()
        (self.package / 'resources/notice.txt').write_text('notice')
        self.version('version one')

    def version(self, text):
        binary = self.package / 'dlms-learning-lab'
        binary.write_text('#!/bin/sh\nprintf "%s\\n" "' + text + '"\n')
        binary.chmod(0o755)
        paths = sorted(p.relative_to(self.package).as_posix() for p in self.package.rglob('*') if p.is_file())
        if 'INSTALL_FILES.txt' not in paths:
            paths.append('INSTALL_FILES.txt')
        (self.package / 'INSTALL_FILES.txt').write_text('\n'.join(sorted(paths)) + '\n')

    def install(self, **changes):
        args = {'--prefix': self.prefix, '--bin-dir': self.bindir, '--desktop-dir': self.menu}
        args.update(changes)
        return subprocess.run(['sh', str(self.package / 'install.sh'), *[str(x) for pair in args.items() for x in pair]],
                              env=self.env, capture_output=True, text=True, timeout=5)

    def uninstall(self):
        return subprocess.run(['sh', str(self.prefix / 'uninstall.sh')], env=self.env, capture_output=True, text=True, timeout=5)

    def test_clean_install_upgrade_uninstall_retains_progress_and_added_files(self):
        self.assertEqual(self.install().returncode, 0)
        self.assertEqual(subprocess.check_output([self.bindir / 'dlms-learning-lab'], text=True).strip(), 'version one')
        self.version('version two')
        self.assertEqual(self.install().returncode, 0)
        self.assertEqual(subprocess.check_output([self.bindir / 'dlms-learning-lab'], text=True).strip(), 'version two')
        added = self.prefix / 'my-backup.json'
        added.write_text('personal backup')
        self.assertNotEqual(self.install().returncode, 0)
        self.assertEqual(self.uninstall().returncode, 0)
        self.assertTrue(added.is_file())
        self.assertTrue(self.progress.is_file())
        self.assertFalse((self.bindir / 'dlms-learning-lab').exists())
        self.assertFalse((self.menu / 'org.dlms.LearningLab.desktop').exists())

    def test_dangling_desktop_link_and_unrelated_launcher_are_retained(self):
        self.menu.mkdir()
        external = self.root / 'must-not-be-created'
        entry = self.menu / 'org.dlms.LearningLab.desktop'
        entry.symlink_to(external)
        self.assertNotEqual(self.install().returncode, 0)
        self.assertFalse(external.exists())
        self.assertFalse(self.prefix.exists())
        entry.unlink()
        self.bindir.mkdir()
        launcher = self.bindir / 'dlms-learning-lab'
        launcher.write_text('unrelated program')
        self.assertNotEqual(self.install().returncode, 0)
        self.assertEqual(launcher.read_text(), 'unrelated program')

    def test_intermediate_symlink_never_deletes_external_notice(self):
        self.assertEqual(self.install().returncode, 0)
        resources = self.prefix / 'resources'
        shutil.rmtree(resources)
        outside = self.root / 'other'
        outside.mkdir()
        sentinel = outside / 'notice.txt'
        sentinel.write_text('external sentinel')
        resources.symlink_to(outside, target_is_directory=True)
        self.assertNotEqual(self.uninstall().returncode, 0)
        self.assertEqual(sentinel.read_text(), 'external sentinel')
        self.assertTrue((self.bindir / 'dlms-learning-lab').is_symlink())
        self.assertNotEqual(self.install().returncode, 0)
        self.assertEqual(sentinel.read_text(), 'external sentinel')

    def test_overlapping_and_field_code_paths_are_rejected_before_changes(self):
        for args in [{'--bin-dir': self.prefix}, {'--desktop-dir': self.prefix / 'menu'},
                     {'--prefix': self.bindir / 'app'}, {'--prefix': self.root / 'lab%f'},
                     {'--prefix': self.root / 'apps/../bin/app'}]:
            with self.subTest(args=args):
                self.assertNotEqual(self.install(**args).returncode, 0)
                self.assertFalse((self.bindir / 'dlms-learning-lab').exists())
        self.assertTrue(self.progress.exists())

    def test_corrupt_manifest_is_validated_before_any_removal(self):
        self.assertEqual(self.install().returncode, 0)
        manifest = self.prefix / 'INSTALL_FILES.txt'
        manifest.write_text('dlms-learning-lab\n../outside\n')
        self.assertNotEqual(self.uninstall().returncode, 0)
        self.assertTrue((self.prefix / 'dlms-learning-lab').exists())
        self.assertTrue((self.bindir / 'dlms-learning-lab').is_symlink())
        self.assertTrue(self.progress.exists())


if __name__ == '__main__':
    unittest.main()
