"""Boot teaching data and source-only consolidation; execution tests are in Rust."""
import copy
import hashlib
import json
import sys
import unittest
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory
import verify_coverage


class BootEntryTeaching(unittest.TestCase):
    def test_three_distinct_tasks_keep_workshop_and_assignment_separate(self):
        labs = json.loads((ROOT / 'scenarios/boot-entry-lessons-v1.json').read_text())['scenarios']
        self.assertEqual([s['id'] for s in labs], ['lpic1-boot-root-identity', 'lpic1-boot-initramfs-match', 'lpic1-boot-recovery-handoff'])
        rows = json.loads((ROOT / 'scenarios/presentation/lpic1.json').read_text())
        self.assertEqual(len(rows), 154)
        old = json.dumps(rows[:117], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        # Actual admitted372 copied LPIC1 presentation; no regenerated baseline.
        self.assertEqual((len(old), hashlib.sha256(old).hexdigest()), (492433, '3117d7fabdb51d3d31e1d7ddcdd46537a27fdbfb92df727e1e3830ff9504045c'))
        for lab, row in zip(labs, rows[117:120]):
            self.assertEqual(row['id'], lab['id'])
            self.assertEqual(' '.join(row['requirements']), lab['objective'])
            d = lab['goal']['definition']
            self.assertNotEqual(d['initial']['reference'], d['example']['reference'])
            self.assertNotEqual(d['initial']['required_root'], d['example']['required_root'])
            self.assertIsNone(lab['initial']['simulation']['assignment']['checked'])
            self.assertEqual(lab['initial']['simulation']['assignment']['source'], d['initial']['starter'])
            self.assertNotEqual(d['initial']['starter'], d['initial']['reference'])
            self.assertIn('Test entry · simulation', ' '.join(p['action'] for p in row['practice']))
            self.assertIn('kernel', row['concepts'][2]['text'])
            self.assertNotIn(d['initial']['reference'], json.dumps({k: row[k] for k in ('requirements', 'worked', 'practice', 'concepts')}))

    def test_only_complete_unchanged_module_sources_can_use_the_consolidation(self):
        container = 'scenarios/kernel-module-curriculum-v2.json'
        current = json.loads((ROOT / container).read_text())['scenarios']
        original = []
        for name, count in [('kernel-module-lessons-v1.json', 3), ('kernel-module-names-lessons-v1.json', 1)]:
            origin = 'scenarios/' + name
            pack = json.loads((ROOT / origin).read_text())['scenarios']
            self.assertEqual(len(pack), count)
            original += pack
            for lab in pack:
                self.assertTrue(teaching_inventory.same_preserved_origin(lab, origin, container))
                wrong = copy.deepcopy(lab)
                wrong['goal'] = {'forged': True}
                self.assertFalse(teaching_inventory.same_preserved_origin(wrong, origin, container))
            known = {container: {s['id']: s for s in current}}
            by_id = {s['id']: s for s in pack}
            self.assertTrue(verify_coverage.preserved_authored_pack(origin, by_id, known))
            self.assertFalse(verify_coverage.preserved_authored_pack('scenarios/unrelated.json', by_id, known))
            changed = copy.deepcopy(by_id)
            next(iter(changed.values()))['version'] += 1
            with self.assertRaisesRegex(ValueError, 'Changed preserved authored origin'):
                verify_coverage.preserved_authored_pack(origin, changed, known)
            changed = copy.deepcopy(known)
            changed[container][pack[0]['id']]['objective'] += ' altered'
            with self.assertRaisesRegex(ValueError, 'differs from its bundled scenario'):
                verify_coverage.preserved_authored_pack(origin, by_id, changed)
        self.assertEqual(current, original)
        paths = verify_coverage.registered_pack_paths(ROOT, 'current')
        self.assertEqual(len(paths), 128)
        self.assertIn(container, paths)
        self.assertEqual(paths[-1], 'scenarios/boot-entry-lessons-v53.json')
        self.assertNotIn('scenarios/kernel-module-lessons-v1.json', paths)
        self.assertNotIn('scenarios/kernel-module-names-lessons-v1.json', paths)
