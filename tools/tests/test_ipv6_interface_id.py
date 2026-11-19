"""Authored-source consolidation and teaching facts; never run learner work."""
import copy
import hashlib
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
import teaching_inventory
from verify_coverage import registered_pack_paths


class InterfaceIdTeaching(unittest.TestCase):
    def test_new_task_and_consolidated_packs_preserve_all_old_values_and_limits(self):
        old = json.loads((ROOT / 'scenarios/ipv6-static-v1.json').read_text())['scenarios']
        new = json.loads((ROOT / 'scenarios/ipv6-interface-id-v1.json').read_text())['scenarios']
        self.assertEqual(len(old), 6)
        self.assertEqual(len(new), 1)
        paths = registered_pack_paths(ROOT, 'teaching-review')
        self.assertEqual(len(paths), 128)
        self.assertIn('scenarios/ipv6-static-v1.json', paths)
        self.assertIn('scenarios/ipv6-interface-id-v1.json', paths)
        self.assertIn('scenarios/ldap-administration-v2.json', paths)
        previous = []
        for name in ('ldap-change-lessons-v1.json', 'ldap-utility-lessons-v1.json'):
            previous += json.loads((ROOT / 'scenarios' / name).read_text())['scenarios']
            self.assertNotIn('scenarios/' + name, paths)
        self.assertEqual(json.loads((ROOT / 'scenarios/ldap-administration-v2.json').read_text())['scenarios'], previous)
        lab = new[-1]
        self.assertEqual(lab['id'], 'ccna-ipv6-modified-eui64')
        self.assertIsNone(lab['initial']['topology']['devices']['Client']['interfaces']['lan']['address'])
        self.assertEqual(lab['goal']['requirements'][0]['kind'], 'ModifiedEui64')
        self.assertNotEqual(lab['goal']['requirements'][0]['mac'], lab['ipv6_teaching']['outcomes'][0]['mac'])
        rows = json.loads((ROOT / 'scenarios/presentation/ccna.json').read_text())
        self.assertEqual(len(rows),113)
        row = next(row for row in rows if row["id"] == "ccna-ipv6-modified-eui64")
        self.assertEqual(row['source'], 'scenarios/ipv6-interface-id-v1.json')
        self.assertEqual(' '.join(row['requirements']), lab['objective'])
        self.assertIn('Apply and test example', ' '.join(r['action'] for r in row['practice']))
        self.assertNotIn('2001:db8:74:0:425:96ff:feab:cdef', json.dumps({k: row[k] for k in ('concepts', 'worked', 'practice', 'requirements')}))
        self.assertIn('modern', row['concepts'][-1]['text'].lower())
        self.assertIn('without embedding a permanent MAC', row['concepts'][-1]['text'])
        # Old values include authored origin; no invisible prose/requirement edits.
        before = json.dumps(rows[:88], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        # Pin independently to the shipped372 copied presentation with BUILD_INFO.
        # The permanent digest is populated from that verified artifact below.
        self.assertEqual((len(before), hashlib.sha256(before).hexdigest()), OLD_CCNA)

    def test_origin_alias_requires_a_complete_unchanged_known_scenario(self):
        for name in ('ldap-change-lessons-v1.json', 'ldap-utility-lessons-v1.json'):
            origin = 'scenarios/' + name
            old = json.loads((ROOT / origin).read_text())['scenarios'][0]
            bundled = 'scenarios/ldap-administration-v2.json'
            self.assertTrue(teaching_inventory.same_preserved_origin(old, origin, bundled))
            for field in ('objective', 'version', 'goal', 'initial'):
                wrong = copy.deepcopy(old)
                wrong[field] = 'changed'
                self.assertFalse(teaching_inventory.same_preserved_origin(wrong, origin, bundled))
            self.assertFalse(teaching_inventory.same_preserved_origin(old, '../private.json', bundled))
            self.assertFalse(teaching_inventory.same_preserved_origin(old, origin, 'scenarios/unrelated.json'))
            self.assertFalse(teaching_inventory.same_preserved_origin({'id': 'unknown'}, origin, bundled))

OLD_CCNA = (306830, 'f54671df666bbc0f635e694a4dff1638bea2655a7e909a0a0e1a532ed0a9102c')

class ConsolidatedCoverage(unittest.TestCase):
    def test_only_frozen_complete_origins_are_excluded_from_duplicate_draft_credit(self):
        from verify_coverage import preserved_authored_pack
        container = 'scenarios/ldap-administration-v2.json'
        target = json.loads((ROOT / container).read_text())['scenarios']
        known = {container: {s['id']: s for s in target}}
        for name in ('ldap-change-lessons-v1.json', 'ldap-utility-lessons-v1.json'):
            origin = 'scenarios/' + name
            original = json.loads((ROOT / origin).read_text())['scenarios']
            pack = {s['id']: s for s in original}
            self.assertTrue(preserved_authored_pack(origin, pack, known))
            self.assertFalse(preserved_authored_pack('scenarios/duplicate-v1.json', pack, known))
            self.assertFalse(preserved_authored_pack(origin, pack, {}))
            changed = copy.deepcopy(pack)
            next(iter(changed.values()))['objective'] += ' changed'
            with self.assertRaisesRegex(ValueError, 'Changed preserved authored origin'):
                preserved_authored_pack(origin, changed, known)
            changed = copy.deepcopy(known)
            changed[container][original[0]['id']]['version'] += 1
            with self.assertRaisesRegex(ValueError, 'differs from its bundled scenario'):
                preserved_authored_pack(origin, pack, changed)


class CurrentCatalogReporting(unittest.TestCase):
    def test_current_and_review_profiles_match_without_relaxing_legacy_bounds(self):
        current = registered_pack_paths(ROOT, 'current')
        self.assertEqual(current, registered_pack_paths(ROOT, 'teaching-review'))
        self.assertEqual(len(current), 128)
        legacy = registered_pack_paths(ROOT, 'bundled')
        self.assertLessEqual(len(legacy), 100)
        with self.assertRaises(ValueError):
            registered_pack_paths(ROOT, 'unknown-profile')
