"""No reference executables run here; synthetic fixtures test containment guards."""
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import file_backup_reference as reference


@unittest.skipUnless(sys.platform.startswith('linux'), 'fixed Fedora reference')
class BackupReferenceSafety(unittest.TestCase):
    def test_chmod_reference_accepts_only_eight_fixed_modes_and_owned_targets(self):
        self.assertEqual(len(reference.NUMERIC_MODES),8)
        for kind,mode in reference.NUMERIC_MODES:
            self.assertEqual(reference.chmod_argv('/proc/self/fd/99',kind,mode),
                             ['/proc/self/fd/99',mode,'mode-'+kind])
        for kind,mode in [('directory','77777'),('file','--reference=/private'),('../outside','755'),('directory','u+s')]:
            with self.assertRaises(reference.safe.OracleError):
                reference.chmod_argv('/proc/self/fd/99',kind,mode)
    def test_no_cli_source_profile_path_binary_or_abbreviations(self):
        with patch.object(reference, 'root_guard') as guard:
            for flags in [['--capture-id', 'x', '--source', 'tar bad'], ['--binary', '/bin/tar'],
                          ['--capture-id', 'x', '--output', '/tmp/escape'], ['--capture', 'x'],
                          ['--capture-id', '../outside'], ['--capture-id', 'x/y']]:
                with self.subTest(flags=flags), self.assertRaises((SystemExit, reference.safe.OracleError)):
                    reference.main(flags)
            guard.assert_not_called()

    def test_fixed_create_flags_cannot_be_replaced_with_shell_or_tar_escape(self):
        command = reference.create_argv('/proc/self/fd/99', 'base.tar', 'sequence.snar')
        self.assertEqual(command, ['/proc/self/fd/99', '--create', '--format=gnu', '--file=base.tar',
                                   '--listed-incremental=sequence.snar', '--directory=source', 'data'])
        for archive, snapshot in [('/private/a.tar', 'sequence.snar'),
                                  ('--checkpoint-action=exec=sh', 'sequence.snar'),
                                  ('base.tar', '../snapshot'), ('base.tar', '/private/snapshot')]:
            with self.assertRaises(reference.safe.OracleError):
                reference.create_argv('/proc/self/fd/99', archive, snapshot)

    def test_extraction_only_targets_ten_owned_roots_no_unlink_or_exec_flags(self):
        self.assertEqual(len(reference.CHAINS), 10)
        for name, chain in reference.CHAINS.items():
            for archive, incremental in chain:
                command = reference.extract_argv('retained-tar', archive, name, incremental)
                self.assertEqual(command[:4], ['retained-tar', '--extract', '--file=' + archive, '--directory=' + name])
                self.assertEqual(command[4:], ['--incremental'] if incremental else [])
        for destination in ['..', '/private', 'source', '--to-command=sh']:
            with self.assertRaises(reference.safe.OracleError):
                reference.extract_argv('retained-tar', 'base.tar', destination, True)
        self.assertEqual(reference.extract_argv('retained-tar','base.tar','partial_only',False,'data/.settings')[-2:], ['--','data/.settings'])
        for member in ['../private','/etc/passwd','data/../../private','--to-command=sh']:
            with self.assertRaises(reference.safe.OracleError):
                reference.extract_argv('retained-tar','base.tar','partial_only',False,member)

    def test_missing_shared_guard_or_foreign_target_stops_before_pins_or_children(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / '.git').mkdir()
            with patch.object(reference, 'ROOT', root), \
                 patch.object(reference.safe, 'require_guard', side_effect=reference.safe.OracleError('No shared lock')), \
                 patch.object(reference, 'source_pins') as pins, \
                 patch.object(reference.safe, 'capture_command') as child:
                with self.assertRaisesRegex(reference.safe.OracleError, 'shared lock'):
                    reference.main(['--capture-id', 'unit-only'])
                pins.assert_not_called(); child.assert_not_called()
            with patch.object(reference.safe, 'require_guard', return_value=(3,)), \
                 patch.dict(os.environ, {'CARGO_TARGET_DIR': str(root / 'other')}):
                with self.assertRaisesRegex(reference.safe.OracleError, 'target'):
                    reference.root_guard(root)

    def test_fixture_walk_never_follows_links_and_bounds_file_data(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            work = root / 'work'; work.mkdir(); (root / 'private').write_text('not to follow')
            (work / 'linked').symlink_to(root / 'private')
            with self.assertRaisesRegex(reference.safe.OracleError, 'symlinks'):
                reference.observation(work)
            # Exact newly-created synthetic unit link; no historical data removed.
            (work / 'linked').unlink()
            (work / 'large').write_bytes(b'x' * 4097)
            with self.assertRaises(reference.safe.OracleError): reference.observation(work)

    def test_modes_names_and_text_are_observed_without_assuming_owner_identity(self):
        with tempfile.TemporaryDirectory() as temporary:
            work = Path(temporary); work.chmod(0o750)
            (work / '.settings').write_text('same text\n'); (work / '.settings').chmod(0o640)
            value = reference.observation(work)
            self.assertEqual(value, {'': {'directory': True, 'mode': 0o750, 'text': None},
                                     '.settings': {'directory': False, 'mode': 0o640, 'text': 'same text\n'}})
