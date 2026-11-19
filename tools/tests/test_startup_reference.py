"""Fixed collector guards only. Never executes Bash or a learner program."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
TOOLS=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('fixed_startup',TOOLS/'startup_reference.py')
v=importlib.util.module_from_spec(spec);spec.loader.exec_module(v)
class StartupReferenceGuards(unittest.TestCase):
    def test_fixed18_are_bounded_and_distinguish_document_selection_from_real_sources(self):
        rows=v.fixtures();self.assertEqual(len(rows),18)
        self.assertEqual(len({r['id']for r in rows}),18)
        self.assertTrue(all(set(r['files'])<=set(v.PATHS)for r in rows))
        self.assertIn('explicit-exit',{r['id']for r in rows})
        self.assertIn('opterr-invocation-reset',{r['id']for r in rows})
    def test_changed_fixture_refuses_before_any_program_or_child(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);p=root/v.FIXTURE;p.parent.mkdir(parents=True);p.write_text('{}')
            with patch.object(v,'ROOT',root),patch.object(v,'pinned_executable')as held,patch.object(v,'capture_command')as child:
                with self.assertRaisesRegex(ValueError,'fixture changed'):v.fixtures()
                held.assert_not_called();child.assert_not_called()
    def test_observer_sources_only_fixed_disposable_names_without_env_interpolation(self):
        row=v.fixtures()[0];source=v.script(row)
        self.assertIn("trap '__dlms_observe' EXIT",source)
        self.assertIn('  __dlms_status=$?',source)
        self.assertIn('. ./startup-0.sh\n. ./startup-1.sh\n',source)
        self.assertNotIn('/etc/profile',source);self.assertNotIn('~/.bash_profile',source)
        self.assertNotIn('eval ',source);self.assertNotIn('\\"',source)
        self.assertIn('} > business.stdout 2> business.stderr',source)
        row=copy.deepcopy(row);row['env']={'SOURCE':'$(must-stay-data)'}
        self.assertEqual(v.script(row),source)
    def test_protocol_distinguishes_unset_empty_export_and_nonzero_status(self):
        raw='\0'.join(['7','A','1','1','','B','0','1',''])+'\0'
        status,values=v.decode_protocol(raw,['A','B']);self.assertEqual(status,7)
        self.assertEqual(values,{'A':dict(value='',exported=True),'B':dict(value=None,exported=True)})
        for bad in [raw[:-1],raw+'extra\0',raw.replace('B\0','C\0')]:
            with self.assertRaises(ValueError):v.decode_protocol(bad,['A','B'])
    def test_roster_and_selection_cannot_pass_a_partial_or_forged_result(self):
        recipes=v.fixtures();good=[dict(id=r['id'],selection=r['expected_selection'],stdout='',stderr='',status=0,files={'/work/keep.txt':'Retained fixture bytes.\n'},error=None,variables={n:dict(value=None,exported=False)for n in r['observe']})for r in recipes]
        v.observation_rows(good,recipes)
        for bad in [good[:-1],good+good[:1],[good[0]]+good[:-1]]:
            with self.assertRaises(ValueError):v.observation_rows(bad,recipes)
        for key,value in [('status',True),('error','unsupported'),('selection',['/etc/shadow']),('stdout',False),('files',{})]:
            bad=copy.deepcopy(good);bad[0][key]=value
            with self.assertRaises(ValueError):v.observation_rows(bad,recipes)
    def test_approval_requires_exact_current_control_without_launch(self):
        expected=dict(root_reviewed=False,pins=['synthetic'],executable={'sha256':'fixed'})
        with tempfile.TemporaryDirectory()as d:
            base=Path(d);path=base/('observe-approved-'+v.control_key(expected)+'.json')
            with patch.object(v,'control',return_value=expected),patch.object(v,'capture_command')as child:
                with self.assertRaises(FileNotFoundError):v.approved(base,'observe')
                for value in [expected,{**expected,'root_reviewed':1},{**expected,'root_reviewed':True,'extra':True},{**expected,'root_reviewed':True,'pins':['changed']}]:
                    path.write_text(json.dumps(value))
                    with self.assertRaises(ValueError):v.approved(base,'observe')
                path.write_text(json.dumps({**expected,'root_reviewed':True}));self.assertEqual(v.approved(base,'observe'),expected);child.assert_not_called()
    def test_unsealed_reference_cannot_launch_vm(self):
        with patch.object(v,'OBSERVATIONS_SHA',None),patch.object(v,'approved')as approved,patch.object(v,'capture_command')as child:
            with self.assertRaisesRegex(ValueError,'seal pending'):v.compare(Path('unused'),v.fixtures(),0)
            approved.assert_not_called();child.assert_not_called()
    def test_limits_are_child_only_and_budget_cumulative(self):
        budget=v.Budget();budget.charge(v.CAP)
        with self.assertRaises(ValueError):budget.charge(1)
        with self.assertRaises(ValueError):budget.charge(-1)
        with patch.object(v.resource,'setrlimit')as limit:
            v.limits();self.assertEqual(limit.call_count,4)
            self.assertIn((v.resource.RLIMIT_CPU,(2,2)),[c.args for c in limit.call_args_list])
    def test_existing_evidence_is_never_overwritten(self):
        with tempfile.TemporaryDirectory()as d:
            root=Path(d);path=root/'observations.json';v.output(path,{'synthetic':True});before=path.read_bytes()
            with self.assertRaises(FileExistsError):v.output(path,{'replace':True})
            self.assertEqual(path.read_bytes(),before);v.fresh(root,'run')
            with self.assertRaises(FileExistsError):v.fresh(root,'run')
if __name__=='__main__':unittest.main()
