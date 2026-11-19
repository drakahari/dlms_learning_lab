"""Authored GRUB teaching, retained origins and strict current registration only."""
import copy
import csv
import hashlib
import json
import re
from pathlib import Path
import sys
import unittest
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory
import verify_coverage

class GrubTeaching(unittest.TestCase):
    def test_two_distinct_assignments_have_separate_worked_values_and_preserve_all_old_teaching(self):
        data=json.loads((ROOT/'scenarios/boot-entry-lessons-v2.json').read_text())['scenarios']
        old=json.loads((ROOT/'scenarios/boot-entry-lessons-v1.json').read_text())['scenarios']
        self.assertEqual(data[:3],old)
        rows=json.loads((ROOT/'scenarios/presentation/lpic1.json').read_text())
        self.assertEqual(len(rows),154)
        # Actual copied376R4LPIC1presentation, checked against its BUILD_INFO.
        # This is an immutable baseline value, not regenerated from current data.
        canonical=json.dumps(rows[:120],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(canonical),hashlib.sha256(canonical).hexdigest()),(511261,'e8bddcb8bb9fc76a655591cb1491404e6ec4dbe18473a6308a71153bae1d8dee'))
        self.assertEqual([s['id']for s in data[3:]],['lpic1-grub-default-fallback','lpic1-grub-recovery-entry'])
        for lab,row in zip(data[3:],rows[120:]):
            self.assertEqual(row['id'],lab['id']);self.assertEqual(row['source'],'scenarios/boot-entry-lessons-v2.json')
            self.assertTrue(all(not re.match(r'^\d+\.',c['title'])for c in row['concepts']))
            self.assertEqual(' '.join(row['requirements']),lab['objective'])
            d=lab['goal']['definition'];self.assertEqual(d['initial']['instructions'],lab['objective'])
            self.assertNotEqual(d['initial']['reference'],d['example']['reference'])
            self.assertNotEqual(d['initial']['reference'].splitlines()[0],d['example']['reference'].splitlines()[0])
            self.assertTrue(set(d['initial']['menu']['preserve']).isdisjoint(d['example']['menu']['preserve']))
            self.assertEqual(lab['initial']['simulation']['assignment']['source'],d['initial']['starter']);self.assertIsNone(lab['initial']['simulation']['assignment']['checked'])
            teaching=json.dumps({key:row[key]for key in ['concepts','worked','practice','requirements']})
            self.assertNotIn(d['initial']['reference'],teaching)
            self.assertIn(d['example']['reference'],[r['value']for r in row['worked']])
            self.assertIn('Test menu · simulation',' '.join(p['action']for p in row['practice']))
            self.assertTrue(any('root' in c['text']and'UUID'in c['text']for c in row['concepts']))
        self.assertNotIn('set default=spare',json.dumps(rows[120]['worked']))

    def test_source_origin_alias_requires_the_complete_identical_three_original_tasks(self):
        origin='scenarios/boot-entry-lessons-v1.json';target='scenarios/boot-entry-lessons-v36.json'
        old=json.loads((ROOT/origin).read_text())['scenarios'];current=json.loads((ROOT/target).read_text())['scenarios']
        known={target:{s['id']:s for s in current}};by_id={s['id']:s for s in old}
        self.assertTrue(verify_coverage.preserved_authored_pack(origin,by_id,known))
        for lab in old:
            self.assertTrue(teaching_inventory.same_preserved_origin(lab,origin,target))
            wrong=copy.deepcopy(lab);wrong['objective']+=' changed'
            self.assertFalse(teaching_inventory.same_preserved_origin(wrong,origin,target))
        changed=copy.deepcopy(known);changed[target][old[0]['id']]['goal']={'forged':True}
        with self.assertRaisesRegex(ValueError,'differs from its bundled scenario'):
            verify_coverage.preserved_authored_pack(origin,by_id,changed)
        self.assertFalse(verify_coverage.preserved_authored_pack('scenarios/unrelated.json',by_id,known))

    def test_menu_generation_and_bios_claim_only_eight_narrow_partial_references(self):
        with (ROOT/'docs/SKILL_COVERAGE.csv').open() as f:
            rows=[r for r in csv.DictReader(f) if r['blueprint']=='LPIC-1 v5.0' and r['parent']=='102.2']
        partial=[r for r in rows if r['status']!='Open']
        self.assertEqual({r['reference']for r in partial},{'102.2','102.2.K1','102.2.K2','102.2.K3','102.2.T1','102.2.T2','102.2.T3','102.2.T4'})
        self.assertEqual([r['reference']for r in rows if r['status']=='Open'],['102.2.K4'])
        menu=['lpic1-grub-default-fallback','lpic1-grub-recovery-entry'];settings=['lpic1-grub-persistent-default','lpic1-grub-argument-scope'];bios=['lpic1-grub-bios-install'];efi=['lpic1-grub-efi-registration']
        menu_tests=['tests/grub_menu.rs','tests/grub_menu_content.rs'];settings_tests=['tests/grub_settings.rs','tests/grub_settings_content.rs'];bios_tests=['tests/bios_loader.rs','tests/bios_loader_workspace.rs','tests/bios_loader_content.rs'];efi_tests=['tests/efi_loader.rs','tests/efi_fat_profile.rs','tests/efi_loader_content.rs']
        expected={'102.2':(menu+settings+bios+efi,menu_tests+settings_tests+bios_tests+efi_tests),'102.2.K1':(menu+efi,menu_tests+efi_tests),'102.2.K3':(menu+settings,menu_tests+settings_tests),'102.2.T1':(menu+settings,menu_tests+settings_tests),'102.2.T3':(settings,settings_tests)}
        for ref in ['102.2.K2','102.2.T2']:expected[ref]=(bios+efi,bios_tests+efi_tests)
        expected['102.2.T4']=(bios,bios_tests)
        for r in partial:
            self.assertEqual(r['status'],'Partial practice')
            labs,tests=expected[r['reference']]
            self.assertEqual(r['implemented_scenarios'].split(';'),labs)
            self.assertEqual(r['evidence_tests'].split(';'),tests)
            for limit in ['GRUB Legacy','grub-mkconfig','firmware','installation','interactive loader']:self.assertIn(limit,r['remaining'])

    def test_normal_registry_and_unit_are_integrated_without_raising_caps(self):
        paths=verify_coverage.registered_pack_paths(ROOT,'current')
        self.assertEqual(len(paths),128);self.assertEqual(paths[-1],'scenarios/boot-entry-lessons-v53.json')
        self.assertNotIn('scenarios/boot-entry-lessons-v1.json',paths)
        labs=teaching_inventory.catalog();self.assertEqual(len(labs),482)
        unit=json.loads((ROOT/'scenarios/grub-menu-units-v1.json').read_text())['units'][0]
        self.assertEqual(unit['prerequisites'],['linux-boot-entry-recovery'])
        self.assertEqual(unit['labs'],['lpic1-grub-default-fallback','lpic1-grub-recovery-entry'])
        self.assertEqual([s['id']for s,_ in labs if s.get('unit')==unit['id']],unit['labs'])

if __name__=='__main__':unittest.main()
