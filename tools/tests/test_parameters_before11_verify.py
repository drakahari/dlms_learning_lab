"""Synthetic guard fixtures only; never historical outcomes or learner progress."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
spec = importlib.util.spec_from_file_location('before11_verify', TOOLS/'parameters_before11_verify.py')
v = importlib.util.module_from_spec(spec)
spec.loader.exec_module(v)


@unittest.skipUnless(os.name == 'posix', 'Fixed Linux retained-evidence tool')
class HistoricalEvidenceGuards(unittest.TestCase):
    def fixture(self):
        t = tempfile.TemporaryDirectory(); self.addCleanup(t.cleanup)
        root = Path(t.name)
        source = root/'src/shell.rs'; source.parent.mkdir(); source.write_text('synthetic source only\n')
        capture = root/'capture'; capture.mkdir()
        producer = root/'fake-executable'; producer.write_bytes(b'not executable: copied data only')
        source_data = {'actual_evaluator_revision': 10, 'inputs': [dict(file='src/shell.rs', **v.digest(source))]}
        (capture/'SOURCE.json').write_text(json.dumps(source_data))
        (capture/'payload.json').write_text('{"synthetic_guard_fixture":true}')
        outputs = [dict(file=p.name, **v.digest(p)) for p in [capture/'SOURCE.json', capture/'payload.json']]
        manifest = dict(scope='Synthetic unit fixture; not actual evaluator evidence',
                        actual_evaluator_revision=10,catalog_contexts=154,profile_contexts=139,
                        script_records=49,authored_cases=350,examples=49,parameter_proposals=35,outputs=outputs)
        raw = json.dumps(manifest).encode(); h = hashlib.sha256(raw).hexdigest()
        (capture/'MANIFEST.json').write_bytes(raw); (capture/'COMPLETE').write_text(h+'\n')
        return root,capture,producer,h

    def test_contained_names_refuse_absolute_parent_and_noncanonical_paths(self):
        for name in ['/tmp/file','../outside','folder/../file','./file','folder//file','']:
            with self.subTest(name=name), self.assertRaises(ValueError): v.child(Path('/controlled'),name)
        self.assertEqual(v.child(Path('/controlled'),'folder/file'),Path('/controlled/folder/file'))

    def test_complete_marker_manifest_and_every_record_byte_are_required(self):
        root,c,p,h=self.fixture()
        with patch.multiple(v,ROOT=root,CAPTURE=c,PRODUCER=p,EXPECTED=h):
            v.verify()
            for name in ['COMPLETE','MANIFEST.json','payload.json','SOURCE.json']:
                path=c/name; original=path.read_bytes(); path.write_bytes(original+b' ')
                with self.subTest(name=name), self.assertRaises((ValueError,RuntimeError)): v.verify()
                path.write_bytes(original)
            v.verify()

    def test_attestation_is_exclusive_and_history_survives_new_current_source(self):
        root,c,p,h=self.fixture()
        with patch.multiple(v,ROOT=root,CAPTURE=c,PRODUCER=p,EXPECTED=h):
            v.attest()
            self.assertEqual((c/'compiled-source/src/shell.rs').read_bytes(),(root/'src/shell.rs').read_bytes())
            self.assertEqual((c/'actual10-producer').read_bytes(),p.read_bytes())
            (root/'src/shell.rs').write_text('later source; no historical regeneration\n')
            v.verify()
            with self.assertRaises(ValueError):v.attest()
            self.assertFalse(json.loads((c/'ATTESTATION.json').read_text())['records_regenerated'])

    def test_capture_time_source_change_refuses_before_copying_or_attesting(self):
        root,c,p,h=self.fixture()
        (root/'src/shell.rs').write_text('changed after proposed capture')
        with patch.multiple(v,ROOT=root,CAPTURE=c,PRODUCER=p,EXPECTED=h):
            with self.assertRaises(ValueError):v.attest()
        self.assertFalse((c/'ATTESTATION.json').exists())
        self.assertFalse((c/'compiled-source').exists())

    def test_symlinks_and_oversized_reads_never_follow_or_return_the_target(self):
        root,c,p,h=self.fixture();(root/'alias').symlink_to(p)
        with self.assertRaises((ValueError,RuntimeError,OSError)):v.digest(root/'alias')
        with self.assertRaises((ValueError,RuntimeError)):v.digest(p,1)
