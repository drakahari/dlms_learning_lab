"""Check the authored EFI lesson and its exact preserved predecessor."""
import hashlib
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory
import verify_coverage

ID = 'lpic1-grub-efi-registration'

def read(path):
    return json.loads((ROOT / path).read_text())

def digest(value):
    data = json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
    return len(data), hashlib.sha256(data).hexdigest()

class EfiLoaderTeaching(unittest.TestCase):
    def test_preserves_all_eight_boot_lessons_and_125_presentation_values(self):
        old = read('scenarios/boot-entry-lessons-v4.json')['scenarios']
        new = read('scenarios/boot-entry-lessons-v5.json')['scenarios']
        rows = read('scenarios/presentation/lpic1.json')
        self.assertEqual((len(old), len(new), len(rows)), (8, 9, 154))
        self.assertEqual(old, new[:8])
        self.assertEqual(digest(old), (60824, '7c27eeb1704b2dff4437c43dc968acf870a97c438ec52a47ab363908a01db7d1'))
        self.assertEqual(digest(rows[:125]), (539186, 'b2763ea3f8190c143f7c36e50e9240df0d0b4301b43303db03b98ce0d03fa1e4'))
        latest = read('scenarios/boot-entry-lessons-v36.json')['scenarios']
        known = {'scenarios/boot-entry-lessons-v36.json': {lab['id']: lab for lab in latest}}
        self.assertTrue(verify_coverage.preserved_authored_pack(
            'scenarios/boot-entry-lessons-v4.json', {lab['id']:lab for lab in old}, known))

    def test_teaches_partition_relative_paths_before_independent_firmware_repair(self):
        lab = read('scenarios/boot-entry-lessons-v5.json')['scenarios'][-1]
        row = read('scenarios/presentation/lpic1.json')[125]
        self.assertEqual((lab['id'],row['id']), (ID,ID))
        self.assertEqual(lab['objective'], ' '.join(row['requirements']))
        task = lab['goal']['definition']['initial']
        example = lab['goal']['definition']['example']
        self.assertNotEqual(task['required_esp'], example['required_esp'])
        self.assertNotEqual(task['required_root'], example['required_root'])
        self.assertNotEqual(task['archive_entry'], example['archive_entry'])
        self.assertEqual(lab['solution'], task['reference'])
        self.assertEqual(task['reference'][-1], '@efi-check')
        self.assertTrue(all(command in [worked['value'] for worked in row['worked']] for command in example['reference'][:2]))
        for command in task['reference'][:2]:
            self.assertNotIn(command, json.dumps(row, ensure_ascii=False))
        self.assertEqual(len(row['practice']), 4)
        self.assertIn('archive still starts', row['practice'][2]['observe'])
        self.assertIn('single quotes', row['concepts'][2]['text'])
        paths = verify_coverage.registered_pack_paths(ROOT, 'current')
        self.assertEqual((len(paths),paths[-1]), (128,'scenarios/boot-entry-lessons-v53.json'))
        self.assertEqual(len(teaching_inventory.catalog()),482)
