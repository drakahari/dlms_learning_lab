"""Prove unsafe future fixture operands cannot reach a host reference command."""
import pathlib
import tempfile
import unittest
from unittest.mock import patch
import filesystem_reference as oracle


class ReferenceBoundary(unittest.TestCase):
    def test_escape_options_and_post_terminator_operands_never_run(self):
        with tempfile.TemporaryDirectory() as temporary:
            work = pathlib.Path(temporary).resolve()
            for argv in [
                ['cp', '--target-directory=/outside', 'source'],
                ['touch', '--', '--/../../outside'],
                ['rm', '-f', '--', '../outside'],
                ['cd', '../../outside'],
                ['ls', '/etc'],
                ['umask', '888'],
                ['bash', '-c', 'anything'],
            ]:
                with self.subTest(argv=argv), patch.object(oracle.subprocess, 'run') as run:
                    with self.assertRaises(ValueError):
                        oracle.run_checked(argv, work, work, 0o022, None)
                    run.assert_not_called()
            with patch.object(oracle.subprocess, 'run') as run:
                with self.assertRaises(ValueError):
                    oracle.run_checked(['cd', '-'], work, work, 0o022, work.parent)
                run.assert_not_called()

    def test_supported_options_and_literal_dash_names_stay_inside(self):
        with tempfile.TemporaryDirectory() as temporary:
            work = pathlib.Path(temporary).resolve()
            for argv in [['ls', '-ald', '.'], ['rm', '-f', '--', '-f'],
                         ['touch', '--', '-n'], ['cp', '--', '-', 'copy'], ['cd', '--', '-P']]:
                oracle.validate_argv(argv, work, work)


if __name__ == '__main__':
    unittest.main()
