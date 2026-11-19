"""Verify the fixed offline tool cannot accept learner commands/profiles."""
import contextlib
import io
from pathlib import Path
import sys
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import nm_bond_keyfile_reference as n
from case_bash_oracle import OracleError

class OfflineKeyfiles(unittest.TestCase):
    def test_closed_fixtures_and_offline_only_command_prefix(self):
        self.assertEqual(n.BASE,('--offline','connection','add'))
        self.assertEqual([name for name,_ in n.FIXTURES],['unbound-port','bound-port','controller'])
        for _,properties in n.FIXTURES:
            self.assertNotIn('up',properties)
            self.assertNotIn('reload',properties)
            self.assertNotIn('load',properties)
            self.assertNotIn('save',properties)
            self.assertEqual(properties[properties.index('ipv4.method')+1],'disabled')
            self.assertEqual(properties[properties.index('ipv6.method')+1],'disabled')
            self.assertTrue(any(value.startswith('20000000-0000-4000-8000-') for value in properties))

    def test_arguments_are_rejected_before_guard_binary_or_process_access(self):
        for args in [['--profile','/etc/NetworkManager/system-connections/real'],['echo','hello'],['--output','/tmp/elsewhere']]:
            with self.subTest(args=args), patch.object(n,'require_guard') as guard,patch.object(n,'capture_command') as capture,contextlib.redirect_stderr(io.StringIO()):
                with self.assertRaises(SystemExit):n.main(args)
                guard.assert_not_called();capture.assert_not_called()

    def test_failed_commands_and_unknown_fixture_never_receive_acceptance(self):
        with self.assertRaises(RuntimeError):n.verify('unbound-port',dict(status=1,stdout='',stderr='rejected'))
        with self.assertRaises(OracleError):n.verify('learner-profile',dict(status=0,stdout='',stderr=''))

    def test_unbound_and_normalized_bound_shapes_are_distinguished(self):
        unbound='[connection]\nautoconnect=false\n[ipv4]\nmethod=disabled\n[ipv6]\nmethod=disabled\n'
        n.verify('unbound-port',dict(status=0,stdout=unbound))
        bound='[connection]\nautoconnect=false\ncontroller=lab-bond\nport-type=bond\n[ethernet]\n[bond-port]\n'
        n.verify('bound-port',dict(status=0,stdout=bound))
        with self.assertRaises(OracleError):n.verify('bound-port',dict(status=0,stdout=bound+'[ipv4]\nmethod=disabled\n'))
        with self.assertRaises(OracleError):n.verify('unbound-port',dict(status=0,stdout=unbound.replace('autoconnect=false','autoconnect=true')))

    def test_controller_dictionary_is_not_a_canned_options_string(self):
        text='[connection]\nautoconnect-ports=1\n[bond]\nmode=active-backup\nmiimon=100\nprimary=labnic0\n[ipv4]\nmethod=disabled\n[ipv6]\nmethod=disabled\n'
        n.verify('controller',dict(status=0,stdout=text))
        for wrong in [text.replace('miimon=100','miimon=0'),text.replace('autoconnect-ports=1','autoconnect-ports=0'),text.replace('mode=active-backup','mode=802.3ad'),text.replace('mode=active-backup','options=mode=active-backup\nmode=active-backup')]:
            with self.subTest(wrong=wrong),self.assertRaises(OracleError):n.verify('controller',dict(status=0,stdout=wrong))
