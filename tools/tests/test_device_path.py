"""Distinct wired-device tasks, visible teaching and exact predecessor retention."""
import hashlib
import json
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[2]
IDS=['ccna-device-path-patching','ccna-device-path-broadcast-boundary']

def read(name):
    return json.loads((ROOT/name).read_text())

class WiredDeviceTeaching(unittest.TestCase):
    def test_old_scenarios_and_all_89_ccna_presentations_remain_identical(self):
        old=read('scenarios/boot-entry-lessons-v11.json')['scenarios']
        current=read('scenarios/boot-entry-lessons-v12.json')['scenarios']
        self.assertEqual((len(old),len(current)),(17,19))
        self.assertEqual(current[:17],old)
        canonical=lambda value:json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual(hashlib.sha256(canonical(old)).hexdigest(), OLD_PACK)
        rows=read('scenarios/presentation/ccna.json')
        self.assertEqual(len(rows),113)
        self.assertEqual(hashlib.sha256(canonical(rows[:89])).hexdigest(),OLD_CCNA)
        self.assertEqual([row['id']for row in rows[89:91]],IDS)

    def test_each_lab_teaches_its_real_controls_with_separate_worked_facts(self):
        lessons=read('scenarios/boot-entry-lessons-v12.json')['scenarios'][-2:]
        rows=[r for r in read('scenarios/presentation/ccna.json')if r['id'] in IDS]
        for lesson,row in zip(lessons,rows):
            definition=lesson['goal']['definition'];task=definition['initial'];example=definition['example']
            self.assertEqual(task['id'],lesson['id'])
            self.assertEqual(task['instructions'],lesson['objective'])
            self.assertEqual(' '.join(row['requirements']),lesson['objective'])
            self.assertNotEqual(task['facts']['network'],example['facts']['network'])
            self.assertEqual({probe['kind']for probe in task['probes']},{'Local','Separate','Routed'})
            self.assertEqual(len(task['probes']),4)
            self.assertFalse(lesson['initial']['simulation']['assignment']['checked'])
            self.assertEqual(json.loads(lesson['solution'][0].removeprefix('DEVICE PATH WIRING\n'))['cables'],task['reference'])
            self.assertEqual(len(row['practice']),3)
            self.assertTrue(all(step['action']and step['observe']for step in row['practice']))
            self.assertGreaterEqual(len(row['concepts']),4)
            self.assertIn('Check paths',' '.join(step['action']for step in row['practice']))
            self.assertNotIn('fresh virtual workspace',json.dumps(row))
        self.assertEqual(read('scenarios/device-path-units-v1.json')['units'][0]['labs'],IDS)
        self.assertEqual(lessons[1]['prerequisites'],[IDS[0]])
        patch,boundary=[lesson['goal']['definition']['initial']for lesson in lessons]
        self.assertNotEqual(patch['initial'][0]['b'],patch['reference'][0]['b'])
        self.assertEqual(boundary['initial'][:-1],boundary['reference'][:-1])
        self.assertTrue(boundary['initial'][-1]['up'])
        self.assertFalse(boundary['reference'][-1]['up'])

OLD_PACK = '21f5759f7dd17db7110adbab3cb4af2c87072a989a8a95cb55d0a8266968e585'
OLD_CCNA = '9d7b1d43ba569fbd9032a0aab557356bef78a231fdda5403ce9fc30b75eaca8b'
