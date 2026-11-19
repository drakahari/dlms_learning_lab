"""Synthetic checks only: never run a child or alter parent limits/umask."""
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import Mock, patch
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import recursive_tree_reference as ref


class RecursiveReference(unittest.TestCase):
    def test_pinned_roster_and_contained_paths(self):
        f = ref.fixture()
        self.assertEqual((len(f['cases']), len(f['excluded'])), (8, 3))
        self.assertEqual(len({c['id'] for c in f['cases']}), 8)
        for p in ('src/sub', 'src words', '-out', '--recursive'):
            self.assertTrue(ref.safe_relative(p))
        for p in ('../private', '/home/private', 'src/../x', '.', 'a//b', 'a\nrm', '$(id)'):
            self.assertFalse(ref.safe_relative(p))

    def test_child_guard_refuses_parent_before_any_effect(self):
        effect = Mock()
        guard = ref.ChildGuard(7, 10, effect, pid=lambda: 10, write=effect,
                               close=effect, apply_limits=effect, apply_core=effect)
        with self.assertRaises(RuntimeError):
            guard()
        effect.assert_not_called()

    def test_literal_fixtures_and_snapshots_do_not_change_global_umask(self):
        f = ref.fixture()
        with tempfile.TemporaryDirectory() as tmp, patch.object(os, 'umask', side_effect=AssertionError('parent umask')):
            work = Path(tmp)
            ref.make_files(work, f, f['cases'][0])
            snap = ref.snapshot(work)
            self.assertEqual(len(snap), 10)
            self.assertEqual(snap['src/.hidden']['text'], 'hidden without LF')
            self.assertEqual(snap['src/sub/data']['mode'], 0o444)
            self.assertTrue(snap['src/empty']['directory'])

    def test_snapshot_refuses_symlink_escape(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp)
            (p / 'alias').symlink_to('/etc/passwd')
            with self.assertRaises(ValueError):
                ref.snapshot(p)

    def test_capture_invalid_mask_or_expired_deadline_never_launches(self):
        with patch.object(ref.subprocess, 'Popen') as launch:
            for mask, deadline in ((0o777, 1e20), (0o022, 0)):
                with self.assertRaises(ValueError):
                    ref.capture([], None, None, {}, None, (), mask, 32768, deadline)
            launch.assert_not_called()

    def test_environment_does_not_inherit_personal_or_loader_values(self):
        with patch.dict(os.environ, {'HOME': '/personal', 'LD_PRELOAD': '/private/loader'}):
            e = ref.closed_environment({}, Path('/fixed'))
            self.assertNotIn('HOME', e)
            self.assertNotIn('LD_PRELOAD', e)
            self.assertEqual(e['PATH'], '')

    def test_stale_or_mislabeled_compiled_inventory_refuses_before_execution(self):
        raw=b'pub const INPUTS:&[(&str,&[u8])]=&[("src/one.rs",include_bytes!("../../src/one.rs"))];'
        with patch.object(ref, 'read_bytes', return_value=raw), patch.object(ref, 'names', return_value=['src/one.rs']):
            ref.validate_bindings()
        for bad in (raw.replace(b'../../src/one.rs', b'../../src/other.rs'), raw):
            with patch.object(ref, 'read_bytes', return_value=bad), patch.object(ref, 'names', return_value=['src/one.rs','src/two.rs']), patch.object(ref.subprocess,'Popen') as launch:
                with self.assertRaises(ValueError):
                    ref.validate_bindings()
                launch.assert_not_called()
