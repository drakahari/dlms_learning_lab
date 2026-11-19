"""Authored local mail teaching, independent work and exact prior content."""
import hashlib
import json
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[2]
IDS=['lpic1-mail-team-alias','lpic1-mail-personal-forward','lpic1-mail-index-recovery']


def read(path):
    return json.loads((ROOT/path).read_text())


def digest(value):
    data=json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
    return len(data),hashlib.sha256(data).hexdigest()


class LocalMailTeaching(unittest.TestCase):
    def test_preserves_all_fourteen_predecessors_and_128_presentations(self):
        old=read('scenarios/boot-entry-lessons-v10.json')['scenarios']
        new=read('scenarios/boot-entry-lessons-v11.json')['scenarios']
        self.assertEqual((len(old),len(new)),(14,17))
        self.assertEqual(old,new[:14])
        self.assertEqual(digest(old),(121006, 'db5e9fb874a77d5011932cd6e186f9ac162d2537970622e47bd67f6c22722d23'))
        rows=read('scenarios/presentation/lpic1.json')
        self.assertEqual(len(rows),154)
        self.assertEqual(digest(rows[:128]),(551076, '49815fe3e7af83d8a9c723199d9d16eef0923f615901bd55394f5052460ce701'))
        self.assertEqual([row['id']for row in rows[128:131]],IDS)

    def test_worked_files_use_different_users_and_require_real_saved_and_indexed_state(self):
        lessons=read('scenarios/boot-entry-lessons-v11.json')['scenarios'][-3:]
        rows=read('scenarios/presentation/lpic1.json')[128:131]
        for lesson,row in zip(lessons,rows):
            definition=lesson['goal']['definition']
            task,example=definition['initial'],definition['example']
            self.assertEqual((lesson['id'],row['id']),(task['id'],task['id']))
            self.assertEqual(lesson['objective'],' '.join(row['requirements']))
            self.assertEqual(task['instructions'],lesson['objective'])
            self.assertTrue(set(task['users']).isdisjoint(example['users']))
            self.assertGreaterEqual(len(task['probes']),3)
            self.assertEqual(
                json.loads(lesson['solution'][0].removeprefix('LOCAL MAIL WORK\n')),
                task['reference'],
            )
            self.assertIn('unsupported',lesson['simplifications'])
            self.assertIn('immediately',lesson['simplifications'])
            self.assertEqual(len(row['practice']),3)
            self.assertNotIn('Read the contract',json.dumps(row))
            self.assertNotIn('fresh virtual workspace',json.dumps(row))
            self.assertNotIn('local scalar',json.dumps(row))
        self.assertEqual(lessons[1]['goal']['definition']['initial']['focus'],{'Forward':'alex'})
        handover=lessons[2]['goal']['definition']['initial']
        self.assertNotEqual(handover['initial']['aliases'],handover['initial_index'])
        self.assertIn('loopa',handover['initial']['aliases'])
        self.assertIn('retired',handover['initial_index'])
        self.assertEqual(handover['reference']['forwards'],handover['initial']['forwards'])
        self.assertIn('Run delivery test',rows[2]['practice'][0]['action'])
        self.assertIn('current lookup',rows[2]['practice'][0]['action'])
        self.assertEqual(
            read('scenarios/local-mail-units-v1.json')['units'][0]['labs'],IDS,
        )
