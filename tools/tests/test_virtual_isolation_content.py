"""New current1.12 source/metadata checks; no private data or host runtime."""
from pathlib import Path
import hashlib
import json
import unittest
ROOT = Path(__file__).resolve().parents[2]

def read(name):
    return json.loads((ROOT / name).read_text())

class VirtualIsolationContent(unittest.TestCase):
    def test_all_110_earlier_ccna_teaching_values_stay_exact(self):
        notes = read('scenarios/presentation/ccna.json')
        self.assertEqual(len(notes), 113)
        raw = json.dumps(notes[:110], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        self.assertEqual(hashlib.sha256(raw).hexdigest(), '130b19e716f2c6078e9d74a3cbc27597efec808b791f796e0cac3578b61756cb')

    def test_selected_pack_preserves_old_values_and_adds_real_sources(self):
        before = read('scenarios/boot-entry-lessons-v43.json')['scenarios']
        after = read('scenarios/boot-entry-lessons-v44.json')['scenarios']
        new = read('scenarios/virtual-isolation-lessons-v1.json')['scenarios']
        self.assertEqual((len(before), len(after), len(new)), (85, 88, 3))
        self.assertEqual(after, before + new)
        self.assertLess((ROOT/'scenarios/boot-entry-lessons-v44.json').stat().st_size, 2_000_000)
        self.assertEqual([s['initial']['kind'] for s in new], ['VirtualIsolation', 'Switching', 'VirtualIsolation'])

    def test_three_screen_notes_are_specific_and_separate(self):
        entries = {e['id']: e for e in read('scenarios/presentation/ccna.json')}
        for s in read('scenarios/virtual-isolation-lessons-v1.json')['scenarios']:
            e = entries[s['id']]
            self.assertEqual(e['source'], 'scenarios/virtual-isolation-lessons-v1.json')
            self.assertEqual(set(e['review']), {'introduction', 'example', 'assignment'})
            self.assertEqual(len(e['concepts']), 3)
            self.assertEqual(len(e['worked']), 3)
            self.assertEqual(len(e['practice']), 3)
            self.assertTrue(e['context'] and e['requirements'])
            for step in e['practice']:
                self.assertGreater(len(step['action']), 40)
                self.assertGreater(len(step['observe']), 40)
            self.assertNotIn('Read the contract', json.dumps(e))
            self.assertNotIn('fresh virtual workspace', json.dumps(e))
            self.assertNotIn('virtual-net', s['objective'])

    def test_unit_retains_prerequisite_order_and_all_authored_members(self):
        fragment = read('scenarios/virtual-isolation-units-v1.json')
        self.assertEqual(fragment['version'], 1)
        self.assertEqual(len(fragment['units']), 1)
        u = fragment['units'][0]
        self.assertEqual(u['id'], 'ccna-virtual-isolation')
        self.assertEqual(u['track'], 'CCNA')
        self.assertEqual(u['prerequisites'], ['vlan-data-foundations', 'ipv4-static-foundations', 'ccna-campus-architectures'])
        self.assertEqual(u['labs'], [s['id'] for s in read('scenarios/virtual-isolation-lessons-v1.json')['scenarios']])

if __name__ == '__main__':
    unittest.main()
