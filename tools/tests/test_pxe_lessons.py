"""Validate teaching context and closed PXE catalog provenance."""
import copy
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))

import teaching_inventory
import verify_coverage

PRIOR = "scenarios/curriculum-continuation-v4.json"
ADDED = "scenarios/pxe-lessons-v1.json"
SELECTED = "scenarios/curriculum-continuation-v5.json"


def rows(path):
    return json.loads((ROOT / path).read_text())["scenarios"]


class NetworkBootLessons(unittest.TestCase):
    def test_practice_teaches_the_actual_native_steps_without_overwriting_the_assignment(self):
        content = json.loads((ROOT / "scenarios/presentation/lpic2_pxe.json").read_text())
        labs = rows(ADDED)
        self.assertEqual([e["id"] for e in content], [lab["id"] for lab in labs])
        for entry, lab in zip(content, labs, strict=True):
            self.assertEqual(" ".join(entry["requirements"]), lab["objective"])
            self.assertEqual(len(entry["concepts"]), 3)
            self.assertEqual(len(entry["worked"]), 4)
            self.assertEqual(len(entry["practice"]), 3)
            self.assertNotIn("192.0.2.", json.dumps(entry["worked"]))
            self.assertNotIn("6.6.61", json.dumps(entry["practice"]))
            self.assertTrue(all(not step["commands"] for step in lab["guide"]))
            contexts = lab["initial"]["simulation"]
            self.assertNotEqual(contexts["assignment"], contexts["example"])
            self.assertFalse(contexts["assignment"]["work"]["checked"])
        self.assertIn("unconditional offer", content[0]["practice"][1]["action"])
        self.assertIn("Configuration paths tried", content[1]["practice"][0]["action"])
        self.assertIn("client-specific", content[1]["practice"][1]["title"])

    def test_aggregate_retains_exact_predecessors_and_origins(self):
        self.assertEqual(rows(SELECTED), rows(PRIOR) + rows(ADDED))
        self.assertLessEqual((ROOT / SELECTED).stat().st_size, 2_000_000)
        known = {SELECTED: {lab["id"]: lab for lab in rows(SELECTED)}}
        for origin in (PRIOR, ADDED, "scenarios/initramfs-lessons-v1.json",
                       "scenarios/dkms-lessons-v1.json", "scenarios/ids-rule-lessons-v1.json"):
            pack = {lab["id"]: lab for lab in rows(origin)}
            self.assertTrue(verify_coverage.preserved_authored_pack(origin, pack, known))
            for lab in pack.values():
                self.assertTrue(teaching_inventory.same_preserved_origin(lab, origin, SELECTED))
            changed = copy.deepcopy(pack)
            next(iter(changed.values()))["title"] += " changed"
            with self.assertRaises(ValueError):
                verify_coverage.preserved_authored_pack(origin, changed, known)
        additions = {lab["id"]: lab for lab in rows(ADDED)}
        for key in known[SELECTED]:
            corrupted = copy.deepcopy(known)
            corrupted[SELECTED][key]["title"] += " corrupted"
            with self.assertRaises(ValueError):
                verify_coverage.preserved_authored_pack(ADDED, additions, corrupted)
