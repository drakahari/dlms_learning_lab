"""Check the authored teaching sequence and closed catalog provenance."""
import copy
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))

import native_reopen
import teaching_inventory
import verify_coverage

PRIOR = "scenarios/curriculum-continuation-v3.json"
ADDED = "scenarios/initramfs-lessons-v1.json"
SELECTED = "scenarios/curriculum-continuation-v4.json"


def rows(path):
    return json.loads((ROOT / path).read_text())["scenarios"]


class InitramfsLessons(unittest.TestCase):
    def test_teaching_distinguishes_saved_fragment_image_and_selected_entry(self):
        content = json.loads((ROOT / "scenarios/presentation/lpic2_initramfs.json").read_text())
        labs = rows(ADDED)
        self.assertEqual([entry["id"] for entry in content], [lab["id"] for lab in labs])
        for entry, lab in zip(content, labs, strict=True):
            self.assertEqual(" ".join(entry["requirements"]), lab["objective"])
            self.assertEqual(len(entry["worked"]), 4)
            intro = " ".join(row["text"] for row in entry["concepts"])
            for term in ('hostonly="no"', "omit_drivers", "--kver", "lsinitrd"):
                self.assertIn(term, intro)
            self.assertNotIn("6.6.61", json.dumps([entry["worked"], entry["practice"]]))
            self.assertNotIn("initramfs-workspace ", json.dumps(entry))
            self.assertTrue(all(not item["commands"] for item in lab["guide"]))
            scopes = lab["initial"]["simulation"]
            self.assertNotEqual(scopes["assignment"], scopes["example"])
            for name in ("assignment", "example"):
                self.assertFalse(scopes[name]["work"]["checked"])
                self.assertEqual(scopes[name]["work"]["command_draft"], "")
        self.assertIn("empty", content[0]["practice"][0]["observe"])
        self.assertIn("remains unused", content[1]["practice"][1]["observe"])

    def test_current_aggregate_admits_only_exact_predecessors_and_additions(self):
        self.assertEqual(rows(SELECTED), rows(PRIOR) + rows(ADDED))
        self.assertLessEqual((ROOT / SELECTED).stat().st_size, 2_000_000)
        known = {SELECTED: {lab["id"]: lab for lab in rows(SELECTED)}}
        for origin in (PRIOR, ADDED, "scenarios/dkms-lessons-v1.json",
                       "scenarios/kernel-build-lessons-v1.json", "scenarios/poe-lessons-v1.json",
                       "scenarios/ids-rule-lessons-v1.json"):
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

    def test_native_reopen_accepts_only_the_fixed_case_name(self):
        root = Path("/fixed-project/.build-artifacts/tmp/native-reopen-initramfs")
        token = "a" * 32
        legacy = native_reopen.session_marker(root, root.parent, token)
        marker = native_reopen.session_marker(root, root.parent, token, "initramfs")
        self.assertEqual(marker.pop("case"), "initramfs")
        self.assertEqual(marker, legacy)
        for invalid in ("initramfs /host", "initramfs;sh", "real-initramfs"):
            with self.assertRaises(native_reopen.ReopenError):
                native_reopen.session_marker(root, root.parent, token, invalid)
