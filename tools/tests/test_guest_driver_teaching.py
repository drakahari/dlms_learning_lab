"""Closed authored content and preserved accepted presentation; no learner execution."""
import hashlib,json,sys,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import teaching_inventory as teaching
class GuestDriverTeaching(unittest.TestCase):
    def test_prior151_lpic1_presentations_are_exact_and_new_views_are_substantial(self):
        rows=json.loads((ROOT/'scenarios/presentation/lpic1.json').read_text())
        old=json.dumps(rows[:151],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        self.assertEqual((len(old),hashlib.sha256(old).hexdigest()),(674414,'b5da74536d7351ad167e898ad0601744fcea76e01130403b7454005357d9fe47'))
        self.assertEqual(len(rows),154)
        self.assertEqual([r['id']for r in rows[151:]],['lpic1-guest-driver-migration','lpic1-guest-object-recovery','lpic1-guest-image-handover'])
        for r in rows[151:]:
            self.assertGreaterEqual(len(r['concepts']),3);self.assertGreaterEqual(len(r['worked']),4);self.assertEqual(len(r['practice']),3);self.assertEqual(len(r['requirements']),5)
            for step in r['practice']:self.assertTrue(step['action']);self.assertTrue(step['observe'])
            for view in ['introduction','example','assignment']:self.assertGreater(len(r['review'][view]),100)
    def test_consolidation_requires_exact_old88_and_exact_three_additions(self):
        old=json.loads((ROOT/'scenarios/boot-entry-lessons-v44.json').read_text())['scenarios']
        new=json.loads((ROOT/'scenarios/guest-driver-lessons-v1.json').read_text())['scenarios']
        combined=json.loads((ROOT/'scenarios/boot-entry-lessons-v45.json').read_text())['scenarios']
        self.assertLess((ROOT/'scenarios/boot-entry-lessons-v45.json').stat().st_size,2_000_000)
        self.assertEqual((len(old),len(new),len(combined)),(88,3,91));self.assertEqual(combined,old+new)
        for lab in new:self.assertTrue(teaching.same_preserved_origin(lab,'scenarios/guest-driver-lessons-v1.json','scenarios/boot-entry-lessons-v45.json'))
        altered=json.loads(json.dumps(combined));altered[-1]['title']='Changed unreviewed copy'
        real=Path.read_text
        def changed(p,*a,**kw):return json.dumps({'schema_version':1,'scenarios':altered})if p==ROOT/'scenarios/boot-entry-lessons-v45.json'else real(p,*a,**kw)
        with patch.object(Path,'read_text',changed):
            with self.assertRaisesRegex(ValueError,'consolidation changed'):teaching.same_preserved_origin(new[-1],'scenarios/guest-driver-lessons-v1.json','scenarios/boot-entry-lessons-v45.json')
