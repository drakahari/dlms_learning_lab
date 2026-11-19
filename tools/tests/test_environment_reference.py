"""Closed collector guards, with no Bash, probe or learner execution."""
import copy
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

TOOLS=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('fixed_environment',TOOLS/'environment_reference.py')
v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)

class EnvironmentReferenceGuards(unittest.TestCase):
    def test_fixed28_sources_are_unobserved_and_bounded(self):
        rows=v.fixtures()
        self.assertEqual(len(rows),28)
        self.assertTrue(all(r['host_observation'] is None for r in rows))
        self.assertEqual({r['id']for r in rows if r['files']},{'literal-wildcard','declaration-no-globbing'})
        for row in rows:
            if row['files']:self.assertIn('/work/ROOM=a.txt',row['files'])

    def test_changed_recipe_refused_before_executable_or_child(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);p=root/v.FIXTURE;p.parent.mkdir(parents=True);p.write_text('{}')
            with patch.object(v,'ROOT',root),patch.object(v,'pinned_executable')as held,patch.object(v,'capture_command')as child:
                with self.assertRaisesRegex(ValueError,'fixture changed'):v.fixtures()
                held.assert_not_called();child.assert_not_called()

    def test_observer_group_retains_scope_and_quotes_values(self):
        recipe=v.fixtures()[0];source=v.script(recipe)
        self.assertTrue(source.startswith('{\n'+recipe['source']))
        self.assertIn('} > business.stdout 2> business.stderr',source)
        self.assertIn('"$__dlms_name" "$__dlms_set" "$__dlms_export" "$__dlms_value"',source)
        self.assertNotIn('\\"',source)
        self.assertNotIn('eval ',source)

    def test_protocol_distinguishes_empty_unset_exported_and_status(self):
        # Build with joins so Python octal escapes cannot alter flag bytes.
        source='\0'.join(['1','A','1','1','','B','0','1',''])+'\0'
        status,values=v.decode_protocol(source,['A','B'])
        self.assertEqual(status,1);self.assertEqual(values,{'A':dict(value='',exported=True),'B':dict(value=None,exported=True)})
        flags=source.split('\0');flags[3]='9'
        for bad in [source[:-1],source+'extra\0',source.replace('B\0','C\0'),'\0'.join(flags)]:
            self.assertNotEqual(bad,source)
            with self.assertRaises(ValueError):v.decode_protocol(bad,['A','B'])

    def test_roster_cannot_pass_truncated_zip_or_duplicate_results(self):
        recipes=v.fixtures()
        good=[dict(id=r['id'],stdout='',stderr='',status=0,files={},error=None,
                   variables={n:dict(value=None,exported=False)for n in r['observe']})for r in recipes]
        v.observation_rows(good,recipes)
        for bad in [good[:-1],good+good[:1],[good[0]]+good[0:-1]]:
            with self.assertRaises(ValueError):v.observation_rows(bad,recipes)
        for key,value in [('status',True),('error','unsupported')]:
            bad=copy.deepcopy(good);bad[0][key]=value
            with self.assertRaises(ValueError):v.observation_rows(bad,recipes)

    def test_output_and_existing_run_are_preserved(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);p=root/'report.json';v.output(p,{'synthetic':True});before=p.read_bytes()
            with self.assertRaises(FileExistsError):v.output(p,{'other':True})
            self.assertEqual(p.read_bytes(),before)
            v.fresh(root,'run')
            with self.assertRaises(FileExistsError):v.fresh(root,'run')

    def test_budget_is_cumulative_and_cannot_reset_per_stream(self):
        budget=v.Budget();budget.charge(v.CAP-1);budget.charge(1)
        with self.assertRaises(ValueError):budget.charge(1)
        with self.assertRaises(ValueError):budget.charge(-1)

    def test_control_approval_binds_all_keys_and_program_without_launch(self):
        expected=dict(root_reviewed=False,pins=['synthetic'],executable={'sha256':'fixed'})
        with tempfile.TemporaryDirectory()as d:
            base=Path(d);target=base/('observe-approved-'+v.control_key(expected)+'.json')
            with patch.object(v,'control',return_value=expected),patch.object(v,'capture_command')as child:
                for value in [expected,{**expected,'root_reviewed':1},{**expected,'root_reviewed':True,'extra':True}]:
                    target.write_text(__import__('json').dumps(value))
                    with self.assertRaises(ValueError):v.approved(base,'observe')
                target.write_text(__import__('json').dumps({**expected,'root_reviewed':True}))
                self.assertEqual(v.approved(base,'observe'),expected);child.assert_not_called()

if __name__=='__main__':unittest.main()
