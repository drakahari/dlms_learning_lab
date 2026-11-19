"""Fixed timezone observer rejects executable recipes and arbitrary environment."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import unittest

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))
import timezone_reference as subject

class TimezoneReference(unittest.TestCase):
    def fixture(self):
        raw=(ROOT/subject.FIXTURE).read_bytes()
        self.assertEqual(hashlib.sha256(raw).hexdigest(),subject.FIXTURE_SHA)
        return json.loads(raw)

    def test_fixed_roster_includes_real_transition_edges_and_empty_override(self):
        value=self.fixture()
        self.assertIs(subject.validate_fixture(value),value)
        self.assertEqual(tuple(r['id']for r in value['cases']),subject.IDS)
        self.assertEqual(len(value['cases']),30)
        cases={r['id']:r for r in value['cases']}
        self.assertEqual(cases['berlin-spring-after']['epoch']-cases['berlin-spring-before']['epoch'],1)
        self.assertEqual(cases['new-york-autumn-after']['epoch']-cases['new-york-autumn-before']['epoch'],1)
        self.assertEqual(cases['empty-tz-means-utc']['tz'],'')

    def test_scripts_paths_options_and_environment_injection_refuse(self):
        for patch in ({'configured':'../../etc/localtime'}, {'configured':['UTC']},
                      {'tz':'/etc/passwd'}, {'tz':':../private'}, {'tz':True},
                      {'epoch':True}, {'epoch':1767225599}, {'epoch':1798761600},
                      {'epoch':'1767225600;id'}, {'utc':1}, {'format':'%F;$(id)'},
                      {'command':'sh script.sh'}, {'env':{'TZDIR':'/tmp'}},
                      {'env':{'LD_PRELOAD':'/tmp/library'}}):
            value=self.fixture();value['cases'][0].update(patch)
            with self.subTest(patch=patch),self.assertRaises(ValueError):subject.validate_fixture(value)
        value=self.fixture();value['version']=True
        with self.assertRaises(ValueError):subject.validate_fixture(value)
        value=self.fixture();value['cases'].pop()
        with self.assertRaises(ValueError):subject.validate_fixture(value)

    def test_reference_children_receive_only_closed_c_locale_and_supplied_tz(self):
        env=subject.closed_environment({'TZ':'Europe/Berlin'},ROOT/'.build-artifacts/tmp')
        self.assertEqual(set(env),{'PATH','LC_ALL','LANG','TMPDIR','TMP','TEMP','TZ'})
        self.assertEqual(env['PATH'],'')
        self.assertEqual((env['LC_ALL'],env['LANG'],env['TZ']),('C','C','Europe/Berlin'))
        self.assertNotIn('HOME',env)

    def test_probe_embeds_the_fixed_source_inventory_and_runs_the_actual_parser(self):
        source=(ROOT/'examples/timezone_reference.rs').read_text()
        for name in subject.SOURCES:self.assertIn('"'+name+'"',source)
        self.assertIn('world.command(&raw)',source)
        self.assertIn('assert_eq!',source)
        self.assertNotIn('std::process',source)
        self.assertNotIn('std::fs',source)

if __name__=='__main__':unittest.main()
