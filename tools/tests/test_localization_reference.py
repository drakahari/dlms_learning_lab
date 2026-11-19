"""Fixed codec observer refuses executable arguments and imported environment."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import localization_reference as subject

class LocalizationReference(unittest.TestCase):
    def fixture(self):
        raw = (ROOT / subject.FIXTURE).read_bytes()
        self.assertEqual(hashlib.sha256(raw).hexdigest(), subject.FIXTURE_SHA)
        return json.loads(raw)

    def test_fixed_byte_fixture_and_roster_are_closed_before_any_process(self):
        value = self.fixture()
        self.assertIs(subject.validate_fixture(value), value)
        self.assertEqual(tuple(r['id'] for r in value['conversions']), subject.IDS)
        self.assertEqual(tuple(r['id'] for r in value['queries']), subject.QUERY_IDS)
        self.assertEqual(value['conversions'][3]['input'], list(range(256)))

    def test_flags_paths_scripts_and_unbounded_bytes_are_never_reference_recipes(self):
        for patch in ({'from': 'UTF-8//IGNORE'}, {'to': '/etc/passwd'},
                      {'from': ['utf8']}, {'input': [True]}, {'input': [-1]},
                      {'input': [256]}, {'input': [65] * 16385},
                      {'command': 'sh anything.sh'}, {'path': '../original'}):
            value = self.fixture()
            value['conversions'][0].update(patch)
            with self.subTest(patch=list(patch)), self.assertRaises(ValueError):
                subject.validate_fixture(value)

    def test_loader_startup_path_and_arbitrary_locale_imports_are_refused(self):
        for env in ({'GCONV_PATH': '/tmp'}, {'LD_PRELOAD': '/tmp/lib.so'},
                    {'PATH': '/usr/bin'}, {'BASH_ENV': 'file'},
                    {'LANG': '$(id)'}, {'LC_ALL': 'unknown'}, {'LANG': 4}):
            value = self.fixture()
            value['queries'][0]['env'] = env
            with self.subTest(env=env), self.assertRaises(ValueError):
                subject.validate_fixture(value)
        value = self.fixture()
        value['version'] = True
        with self.assertRaises(ValueError):
            subject.validate_fixture(value)

    def test_probe_uses_only_selected_original_core_and_fixed_fixture(self):
        source = (ROOT / 'examples/localization_reference.rs').read_text()
        for name in subject.SOURCES:
            self.assertIn('"' + name + '"', source)
        self.assertIn('localization::encoding::convert', source)
        self.assertNotIn('std::process', source)
        self.assertNotIn('std::fs', source)
