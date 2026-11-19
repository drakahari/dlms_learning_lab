"""Guard checks only; no RPM/rpmbuild child is launched by these tests."""
import hashlib
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import rpm_configuration_reference as ref


class ConfigurationReference(unittest.TestCase):
    def test_fixed_fourteen_cases_have_no_expected_observations(self):
        rows = ref.inputs()
        self.assertEqual(len(rows), 14)
        self.assertEqual({r['operation'] for r in rows}, {'upgrade', 'erase'})
        self.assertTrue(all(set(r) == {'id', 'kind', 'operation', 'recorded',
                                     'current', 'incoming'} for r in rows))

    def test_changed_source_or_fixture_refuses_before_child(self):
        for target in ref.PINS:
            original = ref.read_bytes
            def changed(path, cap):
                return original(path, cap) + (b' ' if path == ROOT / target else b'')
            with self.subTest(target=target), patch.object(ref, 'read_bytes', side_effect=changed), \
                    patch.object(ref, 'capture_command') as child:
                with self.assertRaisesRegex(ValueError, 'input changed'):
                    ref.inputs()
                child.assert_not_called()

    def test_no_caller_selectors_even_with_inherited_lock(self):
        with patch.object(ref.sys, 'argv', ['tool', '/etc/never']), \
                patch.object(ref, 'inherited_lock') as lock, \
                patch.object(ref, 'capture_command') as child:
            with self.assertRaisesRegex(ValueError, 'no source/path/case selectors'):
                ref.main()
            lock.assert_not_called()
            child.assert_not_called()

    def test_spec_contains_only_private_leaf_payload_and_no_scriptlets(self):
        payload = ROOT / '.build-artifacts/reference-runs/fixture-test/payload'
        text = ref.spec('dlms-config-upgrade', 1, ref.inputs()[:10], payload)
        self.assertEqual(text.count('%config'), 10)
        self.assertIn('%config(noreplace)', text)
        for forbidden in ('%pre\n', '%post\n', '%trigger', 'curl ', 'wget ', 'sudo ', 'rm '):
            self.assertNotIn(forbidden, text)
        self.assertEqual(text.count('/usr/bin/printf'), 10)
        self.assertIn('AutoReqProv: no', text)

    def test_spec_rejects_foreign_paths_and_package_names(self):
        for path in (Path('/etc'), Path('/tmp'), ROOT / '.build-artifacts/reference-runs/../foreign',
                     ROOT / '.build-artifacts/reference-runs/bad\npath'):
            with self.assertRaises(ValueError):
                ref.spec('dlms-config-upgrade', 1, ref.inputs()[:10], path)
        with self.assertRaises(ValueError):
            ref.spec('other-package', 1, ref.inputs()[:10],
                     ROOT / '.build-artifacts/reference-runs/fixture/payload')

    def test_forged_fixture_schema_is_not_an_executable_source(self):
        raw = json.loads((ROOT / ref.FIXTURE).read_bytes())
        raw['cases'][0]['current'] = '$(touch /etc/never)'
        data = json.dumps(raw).encode()
        original = ref.read_bytes
        pins = dict(ref.PINS, **{ref.FIXTURE: hashlib.sha256(data).hexdigest()})
        with patch.object(ref, 'PINS', pins), patch.object(ref, 'read_bytes',
                side_effect=lambda p, c: data if p == ROOT / ref.FIXTURE else original(p, c)), \
                patch.object(ref, 'capture_command') as child:
            with self.assertRaisesRegex(ValueError, 'inert configuration'):
                ref.inputs()
            child.assert_not_called()
