"""Source teaching review for the two bounded JSON-network inventories."""
import hashlib,json,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
IDS=['ccna-json-port-records','ccna-json-inventory-handover']
def read(name):return json.loads((ROOT/name).read_text())
class JsonInventoryTeaching(unittest.TestCase):
    def test_exact_previous_scenarios_and_91_presentations_are_retained(self):
        old=read('scenarios/boot-entry-lessons-v12.json')['scenarios']
        new=read('scenarios/boot-entry-lessons-v13.json')['scenarios']
        self.assertEqual((len(old),len(new)),(19,21));self.assertEqual(old,new[:19])
        rows=read('scenarios/presentation/ccna.json')
        self.assertEqual(len(rows),113)
        self.assertEqual(hashlib.sha256(json.dumps(rows[:91],sort_keys=True,separators=(',',':')).encode()).hexdigest(),'d4f1eff6b06aa808eb106cf2edbaca5c3b68b00a8d8a440c1468d8e1e45ad2be')
        # Inspect this frozen unit's original two rows, not later catalog additions.
        self.assertEqual([r['id']for r in rows[91:93]],IDS)
    def test_essential_types_ownership_and_apply_are_taught_before_independent_task(self):
        lessons=read('scenarios/boot-entry-lessons-v13.json')['scenarios'][-2:]
        rows=read('scenarios/presentation/ccna.json')[91:93]
        for lesson,row in zip(lessons,rows):
            self.assertEqual(row['id'],lesson['id']);self.assertEqual(' '.join(row['requirements']),lesson['objective'])
            concepts=' '.join(c['text']for c in row['concepts'])
            for term in ['Curly braces','Double quotes','colon','commas','Square brackets','array','true or false','integer','null','Apply inventory','Check network']:
                self.assertIn(term,concepts)
            self.assertEqual(len(row['practice']),3)
            self.assertTrue(all(step['action']and step['observe']for step in row['practice']))
            d=lesson['goal']['definition'];self.assertNotEqual(d['initial']['inventory'],d['example']['inventory'])
            self.assertEqual(json.loads(lesson['solution'][0].removeprefix('JSON INVENTORY\n')),json.loads(d['initial']['reference']))
            self.assertEqual(row['worked'][0]['value'],d['example']['reference'])
            self.assertIn('Expected',row['worked'][1]['label'])
            self.assertIn('actual results',row['worked'][1]['explanation'])
            self.assertFalse(lesson['initial']['simulation']['assignment']['checked'])
            self.assertNotEqual(d['initial']['starter'],d['initial']['reference'])
            self.assertEqual(len(lesson['steps']),len(lesson['guide']))
        self.assertIn('same p1 can appear on two different switches',' '.join(c['text']for c in rows[1]['concepts']).lower())
    def test_unit_prerequisites_and_two_distinct_behavior_tasks(self):
        unit=read('scenarios/json-inventory-units-v1.json')['units'][0]
        self.assertEqual(unit['prerequisites'],['vlan-data-foundations']);self.assertEqual(unit['labs'],IDS)
        lessons=read('scenarios/boot-entry-lessons-v13.json')['scenarios'][-2:]
        a,b=[l['goal']['definition']['initial']for l in lessons]
        self.assertEqual((len(a['inventory']['editable']),len(b['inventory']['editable'])),(1,2))
        self.assertEqual((len(a['requirements']),len(b['requirements'])),(3,8))
        self.assertEqual(lessons[1]['prerequisites'],[IDS[0]])
        self.assertEqual(json.loads(a['starter'])['switches'][0]['ports'][0]['up'],'true')
        self.assertIsInstance(json.loads(a['initial_applied'])['switches'][0]['ports'][0]['up'],bool)
