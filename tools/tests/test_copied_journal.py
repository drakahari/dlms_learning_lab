"""Check authored copied-journal teaching, distinct practice and prior values."""
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
ID = "lpic1-copied-journal-incident"


def read(path):
    return json.loads((ROOT / path).read_text())


def canonical(value):
    data = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()
    return len(data), hashlib.sha256(data).hexdigest()


class CopiedJournalTeaching(unittest.TestCase):
    def test_preserves_all_thirteen_predecessors_and_127_presentations(self):
        old = read("scenarios/boot-entry-lessons-v9.json")["scenarios"]
        new = read("scenarios/boot-entry-lessons-v10.json")["scenarios"]
        self.assertEqual((len(old), len(new)), (13, 14))
        self.assertEqual(old, new[:13])
        self.assertEqual(
            canonical(old),
            (115703, 'b1f42d2644ef1b94a6631b5b5c8901f113a6d2a154c4c7ad91c1a3feaa586fdd'),
        )
        rows = read("scenarios/presentation/lpic1.json")
        self.assertEqual(len(rows), 154)
        self.assertEqual(canonical(rows[:127]), (547878, 'fed4d7030279c278007ac726b22f511e626ec4716ad0a8d3e463dbc8543c136f'))

    def test_essential_instruction_precedes_independent_query(self):
        row = next(row for row in read("scenarios/presentation/lpic1.json") if row["id"] == ID)
        lesson = read("scenarios/boot-entry-lessons-v10.json")["scenarios"][-1]
        self.assertEqual((row["id"], lesson["id"]), (ID, ID))
        self.assertEqual(lesson["objective"], " ".join(row["requirements"]))
        definition = lesson["goal"]["definition"]
        example, assignment = definition["example"], definition["initial"]
        teaching = json.dumps([row["concepts"], row["worked"], row["practice"]])
        for term in ["-D", "-b", "-u", "-p", "-o cat", "32-character", "more urgent"]:
            self.assertIn(term, teaching)
        self.assertIn(example["expected_boot"], teaching)
        self.assertNotIn(assignment["expected_boot"], teaching)
        self.assertNotEqual(example["expected_unit"], assignment["expected_unit"])
        self.assertNotEqual(example["expected_priority"], assignment["expected_priority"])
        self.assertNotEqual(example["archive"]["directory"], assignment["archive"]["directory"])
        self.assertEqual(len(row["practice"]), 3)
        self.assertIn("copied", lesson["simplifications"])
        self.assertIn("No binary journal decoding", lesson["simplifications"])
