"""Fixed reference guard checks only; no real daemon or service operation."""
import hashlib
from pathlib import Path
import sys
import unittest
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).resolve().parents[1]))
import linux_logging_reference as r
class LinuxLoggingReference(unittest.TestCase):
    def test_fixed_policies_events_and_model_pin(self):
        self.assertEqual(len(r.FRAGMENTS),5);self.assertEqual(len(r.EVENTS),36)
        name,size,sha=r.CORE_PIN;raw=(r.ROOT/name).read_bytes()
        self.assertEqual((len(raw),hashlib.sha256(raw).hexdigest()),(size,sha))
        self.assertEqual(r.expected('duplicates','daemon',3),['general','general'])
        self.assertEqual(r.expected('private','authpriv',7),['private'])
        self.assertEqual(r.expected('private','mail',7),[])
        for lines in r.FRAGMENTS.values():
            for line in lines:self.assertIn(line.split()[1],['general','private','service'])
    def test_no_source_config_path_or_process_selector(self):
        with patch.object(sys,'argv',['fixed','--config','bad']),patch.object(r,'inherited_lock')as lock,patch.object(r.subprocess,'Popen')as child,patch.object(r.subprocess,'run')as syntax:
            with self.assertRaises(ValueError):r.main()
            lock.assert_not_called();child.assert_not_called();syntax.assert_not_called()
    def test_unknown_policy_never_gets_fallback_success(self):
        with self.assertRaises(ValueError):r.expected('not-supplied','daemon',3)

    def test_model_symlink_refuses_before_reference_children_or_output(self):
        import tempfile
        with tempfile.TemporaryDirectory()as t:
            root=Path(t);source=root/'src';source.mkdir();target=root/'target';target.write_text('synthetic')
            (source/'model.rs').symlink_to(target)
            with patch.object(r,'ROOT',root),patch.object(r,'CORE_PIN',('src/model.rs',9,hashlib.sha256(b'synthetic').hexdigest())),patch.object(r,'inherited_lock',return_value=3),patch.object(sys,'argv',['fixed']),patch.object(r.subprocess,'Popen')as child,patch.object(r.subprocess,'run')as syntax:
                with self.assertRaises(ValueError):r.main()
                child.assert_not_called();syntax.assert_not_called();self.assertFalse((root/'.build-artifacts').exists())
