"""Guards for the fixed comparison; these tests never launch the probe."""
import hashlib
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import log_filter_reference as ref


class LogFilterReferenceGuards(unittest.TestCase):
    def test_exact_roster_and_fixed_comparisons(self):
        cases = ref.inputs()
        self.assertEqual(len(cases), 12)
        self.assertEqual(sum(len(c['lines']) for c in cases), 48)
        ordinary = ref.expected(cases[0])
        self.assertEqual([r['disposition'] for r in ordinary['rows']],
                         ['matched', 'missed', 'matched', 'missed'])
        self.assertEqual(ref.expected(cases[2])['rows'][1]['disposition'], 'matched')
        self.assertEqual(ref.expected(cases[3])['rows'][2]['disposition'], 'ignored')
        self.assertTrue(all(r['disposition'] == 'ignored'
                            for r in ref.expected(cases[4])['rows']))

    def test_changed_sources_or_fixture_refuse_before_any_child(self):
        read = ref.read_bytes
        for name in ref.PINS:
            def changed(path, cap):
                return read(path, cap) + (b' ' if path == ROOT / name else b'')
            with self.subTest(name=name), patch.object(ref, 'read_bytes', side_effect=changed), \
                    patch.object(ref, 'capture_command') as child:
                with self.assertRaisesRegex(ValueError, 'input changed'):
                    ref.inputs()
                child.assert_not_called()

    def test_caller_selectors_refuse_before_lock_or_child(self):
        for arg in ['/etc/passwd', '--fixture', '--source', '--case', '127.0.0.1']:
            with self.subTest(arg=arg), patch.object(ref.sys, 'argv', ['tool', arg]), \
                    patch.object(ref, 'inherited_lock') as lock, \
                    patch.object(ref, 'capture_command') as child:
                with self.assertRaisesRegex(ValueError, 'no source/path/case selectors'):
                    ref.main()
                lock.assert_not_called()
                child.assert_not_called()

    def test_unknown_or_oversized_fixture_shape_is_not_a_selector(self):
        source = json.loads((ROOT / ref.FIXTURE).read_bytes())
        for change in ['path', 'line', 'count']:
            value = json.loads(json.dumps(source))
            if change == 'path':
                value['cases'][0]['path'] = '/etc/passwd'
            elif change == 'line':
                value['cases'][0]['lines'][0] = 'x' * 513
            else:
                value['cases'].pop()
            raw = json.dumps(value).encode()
            read = ref.read_bytes
            pins = dict(ref.PINS, **{ref.FIXTURE: hashlib.sha256(raw).hexdigest()})
            with self.subTest(change=change), patch.object(ref, 'PINS', pins), \
                    patch.object(ref, 'read_bytes', side_effect=lambda p, c:
                                 raw if p == ROOT / ref.FIXTURE else read(p, c)), \
                    patch.object(ref, 'capture_command') as child:
                with self.assertRaises(ValueError):
                    ref.inputs()
                child.assert_not_called()
