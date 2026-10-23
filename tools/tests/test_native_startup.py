"""Controlled development fixtures for diagnostic output/process boundaries."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import json

spec = importlib.util.spec_from_file_location('native_startup', Path(__file__).resolve().parents[1] / 'native_startup.py')
startup = importlib.util.module_from_spec(spec)
spec.loader.exec_module(startup)


class StartupBoundaries(unittest.TestCase):
    def test_late_success_after_deadline_is_a_failed_launch_and_stops_the_series(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'fresh-output'
            def late(command, logfile, *args, **kwargs):
                if '--version' in command:
                    logfile.write_text('DLMS Learning Lab controlled-test\n')
                    return 0, False, False
                logfile.write_text('Captured startup-library: 10x10\nPASS native startup: late\n')
                return 0, True, False
            argv = ['native_startup.py', '--app', sys.executable, '--count', '2', '--output', str(output)]
            with patch.object(sys, 'argv', argv), patch.object(startup, 'launch', side_effect=late):
                self.assertEqual(startup.main(), 1)
            report = json.loads((output / 'results.json').read_text())
            self.assertEqual(len(report['launches']), 1)
            self.assertTrue(report['launches'][0]['timeout'])
            self.assertEqual(report['passed'], 0)

    def test_large_child_output_is_drained_but_retained_bytes_are_capped(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            code, timeout, truncated = startup.launch(
                [sys.executable, '-c', 'print("x" * 100000)'], root / 'log', root, {}, 5, cap=4096)
            self.assertEqual(code, 0)
            self.assertFalse(timeout)
            self.assertTrue(truncated)
            self.assertEqual((root / 'log').stat().st_size, 4096)

    def test_timeout_terminates_and_reaps_only_its_child(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            children = []
            _, timed_out, _ = startup.launch(
                [sys.executable, '-c', 'import time; time.sleep(10)'], root / 'log', root, {}, .1,
                on_timeout=children.append)
            self.assertTrue(timed_out)
            self.assertEqual(len(children), 1)
            self.assertIsNotNone(children[0].poll())

    def test_interruption_during_stack_sampling_still_reaps_the_child(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            children = []
            def interrupt(process):
                children.append(process)
                raise KeyboardInterrupt
            with self.assertRaises(KeyboardInterrupt):
                startup.launch([sys.executable, '-c', 'import time; time.sleep(10)'],
                               root / 'log', root, {}, .1, on_timeout=interrupt)
            self.assertIsNotNone(children[0].poll())

    def test_occupied_or_symlink_log_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target = root / 'existing'
            target.write_bytes(b'preserve this data')
            with self.assertRaises(FileExistsError):
                startup.launch([sys.executable, '-c', 'raise RuntimeError'], target, root, {}, 1)
            if sys.platform != 'win32':
                link = root / 'link'
                link.symlink_to(target)
                with self.assertRaises(FileExistsError):
                    startup.launch([sys.executable, '-c', 'raise RuntimeError'], link, root, {}, 1)
            self.assertEqual(target.read_bytes(), b'preserve this data')


if __name__ == '__main__':
    unittest.main()
