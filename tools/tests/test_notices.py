import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('notices', ROOT / 'tools/dependency_notices.py')
notices = importlib.util.module_from_spec(spec)
spec.loader.exec_module(notices)


class Notices(unittest.TestCase):
    def test_filtered_target_inventory_has_no_private_path_and_excludes_unused_packages(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            crate_bytes = b'controlled original crate source'
            checksum = hashlib.sha256(crate_bytes).hexdigest()
            (root / 'Cargo.lock').write_text('[[package]]\nname="sample"\nversion="1.0.0"\nchecksum="' + checksum + '"\n')
            source = root / 'registry/src/index/sample-1.0.0'
            source.mkdir(parents=True)
            archive = root / 'registry/cache/index/sample-1.0.0.crate'
            archive.parent.mkdir(parents=True)
            archive.write_bytes(crate_bytes)
            (source / 'LICENSE').write_text('original copyright and license')
            package = {'name': 'sample', 'version': '1.0.0', 'source': 'registry+https://example.invalid/index',
                       'manifest_path': str(source / 'Cargo.toml'), 'license': 'MIT', 'repository': 'https://example.invalid/sample'}
            missing = {**package, 'name': 'unused', 'version': '2.0.0'}
            with patch.object(notices, 'ROOT', root), patch.object(notices, 'CACHE', root / 'cache'):
                result = notices.collect({'packages': [package, missing]}, root / 'out',
                                         dependencies={('sample', '1.0.0')}, target='test-target')
            self.assertEqual(result['unresolved'], [])
            self.assertEqual(len(result['packages']), 1)
            self.assertEqual(result['scope'], 'normal/build dependency graph for test-target')
            self.assertNotIn(temporary, json.dumps(result))
            self.assertEqual((root / 'out/sample-1.0.0/LICENSE').read_text(), 'original copyright and license')

    def test_changed_notice_and_path_escape_fail_without_copying_external_data(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / 'source'
            source.mkdir()
            revision = 'a' * 40
            (source / '.cargo_vcs_info.json').write_text(json.dumps({'git': {'sha1': revision}}))
            package = {'repository': 'https://github.com/example/project'}
            key = 'example/project/' + revision
            cached = root / 'cache' / key
            cached.mkdir(parents=True)
            (cached / 'LICENSE').write_text('changed')
            manifest = {key: {'notices': [{'file': 'LICENSE', 'sha256': hashlib.sha256(b'original').hexdigest()}]}}
            with patch.object(notices, 'CACHE', root / 'cache'):
                with self.assertRaisesRegex(ValueError, 'Changed/missing'):
                    notices.upstream(package, source, False, manifest)
                manifest[key]['notices'][0]['file'] = '../../outside'
                with self.assertRaisesRegex(ValueError, 'Unsafe cached'):
                    notices.upstream(package, source, False, manifest)

    def test_nonempty_output_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            sentinel = root / 'private.json'
            sentinel.write_text('retain')
            with self.assertRaisesRegex(ValueError, 'empty notice output'):
                notices.collect({'packages': []}, root)
            self.assertEqual(sentinel.read_text(), 'retain')


class BackportNotices(unittest.TestCase):
    def make_fixture(self, root):
        shutil.copytree(ROOT / 'vendor', root / 'vendor')
        (root / 'Cargo.lock').write_text('[[package]]\nname="objc2-foundation"\nversion="0.2.2"\n')
        return {'name': 'objc2-foundation', 'version': '0.2.2', 'source': None, 'license': 'MIT',
                'repository': 'https://github.com/madsmtm/objc2',
                'manifest_path': str(root / 'vendor/objc2-foundation/Cargo.toml')}

    def test_exact_backport_delivers_upstream_notice_original_patch_and_patched_source(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package = self.make_fixture(root)
            with patch.object(notices, 'ROOT', root), patch.object(notices, 'CACHE', root / 'cache'):
                result = notices.collect({'packages': [package]}, root / 'out')
            self.assertEqual(result['unresolved'], [])
            self.assertNotIn(temporary, json.dumps(result))
            record, = result['packages']
            sources = [record['source_archive'], record['backport']['patch'], *record['notices'],
                       *record['backport']['resulting_sources']]
            for source in sources:
                self.assertEqual(notices.digest(root / 'out' / source['file']), source['sha256'])
            self.assertIn('vendored-sources/objc2-foundation-0.2.2/crate/src/thread.rs',
                          [source['file'] for source in sources])

    def test_tampering_is_rejected_even_outside_selected_target(self):
        for name in ('objc2-foundation-0.2.2.crate', 'dispatch2.patch',
                     'objc2-foundation/LICENSE.upstream.txt', 'objc2-foundation/PROVENANCE.json',
                     'objc2-foundation/src/thread.rs', 'objc2-foundation/README.md',
                     'objc2-foundation/unexpected'):
            with self.subTest(path=name), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                package = self.make_fixture(root)
                (root / 'vendor' / name).write_text('{"tampered": true}')
                with patch.object(notices, 'ROOT', root), patch.object(notices, 'CACHE', root / 'cache'):
                    with self.assertRaisesRegex(ValueError, 'Backport|Unexpected backport'):
                        notices.collect({'packages': [package]}, root / 'out', dependencies=set())
                self.assertEqual(list((root / 'out').iterdir()), [])

    def test_symlink_and_unreviewed_local_dependency_rejected(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            package = self.make_fixture(root)
            source = root / 'vendor/objc2-foundation/README.md'
            outside = root / 'same-contents'
            source.rename(outside)
            with patch.object(notices, 'ROOT', root), patch.object(notices, 'CACHE', root / 'cache'):
                with self.assertRaisesRegex(ValueError, 'Unreviewed local dependency'):
                    notices.collect({'packages': [{**package, 'name': 'other'}]}, root / 'out',
                                    dependencies=set())
                try:
                    source.symlink_to(outside)
                except OSError as exc:
                    self.skipTest(f'Host does not allow symlinks: {exc}')
                with self.assertRaisesRegex(ValueError, 'Unexpected backport'):
                    notices.collect({'packages': [package]}, root / 'out')


if __name__ == '__main__':
    unittest.main()
