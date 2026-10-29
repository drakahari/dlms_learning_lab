"""Mock-only protocol guards. No probe, host fixture, sealed FD or child is run.

Synthetic JSON below tests admission, never claims actual inode observations.
"""
import copy
import importlib.util
import io
import json
from pathlib import Path
import resource
import sys
import unittest
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'tools'))

def load(name):
    spec=importlib.util.spec_from_file_location(name,ROOT/'tools'/f'{name}.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
collector=load('links_reference')
worker=load('links_reference_worker')


def synthetic_control():
    root=Path('/fixed/repo');chain=root/'.build-artifacts/toolchains/rustup/toolchains/stable-x86_64-unknown-linux-gnu'
    names=[str(root/name)for name in collector.SOURCES]+[str(chain/'bin/rustc'),str(chain/'lib/rustlib/multirust-channel-manifest.toml'),str(root/'.build-artifacts/toolchains/READY.json'),'/fixed/python3',str(root/collector.PROBE)]
    pins=[{'path':name,'bytes':1,'sha256':'0'*64}for name in names]
    for pin in pins:
        if pin['path']==str(root/'src/links.rs'):pin['sha256']=collector.CORE_SHA
        if pin['path']==str(root/collector.FIXTURE):pin['sha256']=collector.FIXTURE_SHA
        if pin['path']==str(root/'examples/links_reference.rs'):pin['sha256']=collector.PROBE_SOURCE_SHA
    table={p['path']:p for p in pins}
    return {'schema_version':1,'root':str(root),'capture':collector.CAPTURE,'root_reviewed':False,'pins':pins,
        'compiled_inputs':[{'path':name,'sha256':table[str(root/name)]['sha256']}for name in collector.COMPILED],
        'python_path':'/fixed/python3','python_version':'fixed synthetic version','platform':{'system':'Linux','release':'fixed','machine':'fixed'},'host_rows':8,'vm_only':6,'scope':'synthetic admission guard','compiler_scope':'synthetic guard only'}

def packet():
    return {'schema':1,'index':0,'collector_pid':12345,'fixture_fd':5,'adapter_fd':6,'worker_fd':7,'python_fd':8,'control_fd':9,'lock_fd':10}

class ProtocolGuards(unittest.TestCase):
    def test_literal_exact_approval_and_no_bool_integer_equivalence(self):
        expected=synthetic_control();good={**expected,'root_reviewed':True}
        self.assertEqual(collector.approve(collector.canonical(good),expected),good)
        for field,value in [('root_reviewed',False),('root_reviewed',1),('host_rows',7),('vm_only',5),('capture','other'),('extra','unknown')]:
            bad=copy.deepcopy(good);bad[field]=value
            with self.assertRaises(collector.safe.OracleError):collector.approve(collector.canonical(bad),expected)

    def test_every_source_compiler_and_executable_pin_is_required_and_typed(self):
        expected=synthetic_control();root=Path(expected['root']);collector.check_control(expected,root)
        for mode in ['missing','duplicate','extra','changed-core','boolean-bytes','wrong-compiled']:
            bad=copy.deepcopy(expected)
            if mode=='missing':bad['pins'].pop()
            elif mode=='duplicate':bad['pins'][1]=bad['pins'][0]
            elif mode=='extra':bad['pins'].append({'path':'/personal/unknown','bytes':1,'sha256':'0'*64})
            elif mode=='changed-core':bad['pins'][0]['sha256']='f'*64
            elif mode=='boolean-bytes':bad['pins'][0]['bytes']=True
            else:bad['compiled_inputs'][0]['sha256']='f'*64
            with self.assertRaises(collector.safe.OracleError):collector.check_control(bad,root)

    def test_directory_creation_is_relative_to_nofollow_opened_parent_and_never_chmods(self):
        from types import SimpleNamespace
        info=SimpleNamespace(st_uid=1000,st_mode=0o40700)
        with patch.object(collector,'open_directory',return_value=20),patch.object(collector.os,'mkdir')as mkdir,patch.object(collector.os,'open',return_value=21)as opened,patch.object(collector.os,'fstat',return_value=info),patch.object(collector.os,'getuid',return_value=1000),patch.object(collector.os,'close')as close:
            self.assertEqual(collector.new_private_dir(Path('/fixed/owned'),'fixture'),Path('/fixed/owned/fixture'))
        self.assertEqual(mkdir.call_args.kwargs,{'dir_fd':20});self.assertTrue(opened.call_args.args[1]&collector.os.O_NOFOLLOW)
        self.assertEqual([c.args[0]for c in close.call_args_list],[21,20])
        with patch.object(collector,'open_directory')as opened:
            for name in ['../other','/absolute','a/b','.hidden']:
                with self.assertRaises(collector.safe.OracleError):collector.new_private_dir(Path('/fixed'),name)
            opened.assert_not_called()

    def test_strict_json_rejects_duplicate_keys_in_both_parent_and_worker(self):
        for module in [collector,worker]:
            with self.assertRaises((ValueError,RuntimeError)):module.strict(b'{"schema":1,"schema":1}')
            with self.assertRaises((ValueError,RuntimeError)):module.strict(b' '* (module.CAP+1))

    def test_packet_has_no_path_source_selector_and_only_distinct_bounded_fds(self):
        p=packet();self.assertEqual(worker.packet(collector.canonical(p)),p)
        with self.assertRaises(ValueError):worker.packet(b' '*4097)
        for field,value in [('schema',True),('index',True),('index',8),('fixture_fd',1),('adapter_fd',5),('worker_fd',128),('path','/personal'),('source','unreviewed')]:
            bad={**p,field:value}
            with self.assertRaises(ValueError):worker.packet(collector.canonical(bad))

    def test_worker_root_is_only_the_exact_owned_fixed_cwd_hierarchy(self):
        root=Path('/fixed/repo')
        for i in range(8):self.assertEqual(worker.fixed_root(root/worker.BASE/worker.CAPTURE/f'case{i}'/'run',i),root)
        for path,index in [(Path('/tmp'),0),(root/'run',0),(root/worker.BASE/worker.CAPTURE/'case8/run',0),(root/'other'/worker.CAPTURE/'case0/run',0)]:
            with self.assertRaises(ValueError):worker.fixed_root(path,index)

    def test_worker_resource_limits_only_reduce_its_own_process_limits(self):
        with patch.object(worker.resource,'getrlimit',return_value=(resource.RLIM_INFINITY,resource.RLIM_INFINITY)),patch.object(worker.resource,'setrlimit')as setlimit:
            worker.limits()
        self.assertEqual(setlimit.call_count,4)
        values=dict(c.args for c in setlimit.call_args_list)
        self.assertEqual(values[resource.RLIMIT_AS],(256*1024*1024,256*1024*1024));self.assertEqual(values[resource.RLIMIT_CPU],(2,2));self.assertEqual(values[resource.RLIMIT_NOFILE],(128,128))
        with patch.object(worker.resource,'getrlimit',return_value=(32,32)),patch.object(worker.resource,'setrlimit')as setlimit:
            worker.limits()
        self.assertEqual(dict(c.args for c in setlimit.call_args_list),{resource.RLIMIT_AS:(32,32),resource.RLIMIT_CPU:(2,2),resource.RLIMIT_FSIZE:(32,32),resource.RLIMIT_NOFILE:(32,32)})

    def test_source_is_checked_before_memfd_and_all_four_seals_are_mandatory(self):
        data=b'fixed synthetic source';expected={'bytes':len(data),'sha256':collector.digest(data)}
        with patch.object(collector.safe,'read_bytes',return_value=data),patch.object(collector.os,'memfd_create')as create:
            with self.assertRaises(collector.safe.OracleError):collector.sealed_source(Path('/fixed/repo'),collector.WORKER,{**expected,'sha256':'0'*64})
            create.assert_not_called()
        flags=collector.fcntl.F_SEAL_WRITE|collector.fcntl.F_SEAL_GROW|collector.fcntl.F_SEAL_SHRINK|collector.fcntl.F_SEAL_SEAL
        with patch.object(collector.safe,'read_bytes',return_value=data),patch.object(collector.os,'memfd_create',return_value=27),patch.object(collector.os,'write',return_value=len(data)),patch.object(collector.os,'close'),patch.object(collector.fcntl,'fcntl',side_effect=[0,flags])as seal:
            self.assertEqual(collector.sealed_source(Path('/fixed/repo'),collector.WORKER,expected),27)
            self.assertEqual(seal.call_args_list[0].args,(27,collector.fcntl.F_ADD_SEALS,flags))
        with patch.object(collector.safe,'read_bytes',return_value=data),patch.object(collector.os,'memfd_create',return_value=27),patch.object(collector.os,'write',return_value=len(data)),patch.object(collector.os,'close')as close,patch.object(collector.fcntl,'fcntl',side_effect=[0,0]):
            with self.assertRaises(collector.safe.OracleError):collector.sealed_source(Path('/fixed/repo'),collector.WORKER,expected)
            close.assert_called_once_with(27)

    def test_prepare_never_creates_a_fixture_seals_code_or_launches_a_child(self):
        root=Path('/fixed/repo');expected=synthetic_control()
        with patch.object(collector,'root_guard',return_value=(10,)),patch.object(collector,'source_pins',return_value=[]),patch.object(collector,'control',return_value=expected),patch.object(collector,'fixture',return_value={}),patch.object(collector.safe,'read_bytes',return_value=b'fixed'),patch.object(collector,'owned_base',return_value=root/collector.BASE),patch.object(Path,'exists',return_value=False),patch.object(Path,'is_symlink',return_value=False),patch.object(collector.safe,'json_new')as write,patch.object(collector,'new_private_dir')as make,patch.object(collector,'sealed_source')as seal,patch.object(collector,'child')as child,patch('builtins.print'):
            self.assertEqual(collector.run(root,'prepare'),0)
        write.assert_called_once();make.assert_not_called();seal.assert_not_called();child.assert_not_called()

    def test_unapproved_capture_stops_before_any_fixture_source_seal_or_child(self):
        root=Path('/fixed/repo');expected=synthetic_control()
        with patch.object(collector,'root_guard',return_value=(10,)),patch.object(collector,'source_pins',return_value=[]),patch.object(collector,'control',return_value=expected),patch.object(collector,'fixture',return_value={}),patch.object(collector.safe,'read_bytes',return_value=collector.canonical(expected)),patch.object(collector,'owned_base',return_value=root/collector.BASE),patch.object(Path,'exists',return_value=False),patch.object(Path,'is_symlink',return_value=False),patch.object(collector,'new_private_dir')as make,patch.object(collector,'sealed_source')as seal,patch.object(collector,'child')as child:
            with self.assertRaises(collector.safe.OracleError):collector.run(root,'capture')
        make.assert_not_called();seal.assert_not_called();child.assert_not_called()

    def test_missing_runtime_status_is_retained_before_a_failure_propagates(self):
        with patch.object(collector.safe,'capture_command',side_effect=RuntimeError('fixed timeout')),patch.object(collector.safe,'json_new')as write:
            with self.assertRaises(RuntimeError):collector.child([],Path('/fixed/folder'),Path('/fixed/run'),{},b'',object(),(),5,collector.time.monotonic(),8192)
        self.assertEqual(write.call_args.args[0],Path('/fixed/folder/STATUS.json'));self.assertIsNone(write.call_args.args[1]['status']);self.assertFalse(write.call_args.args[1]['complete'])

    def test_closed_probe_protocol_and_mismatch_are_distinct_not_canned_success(self):
        expected=synthetic_control();fr=json.loads((ROOT/collector.FIXTURE).read_text())
        # Synthetic protocol objects only; no accepted or persisted VM observation.
        value={'schema_version':1,'core_version':1,'compiled_inputs':expected['compiled_inputs'],'cases':[{'id':r['id'],'observations':r['expected_observations']}for r in fr['cases']],'vm_only':{r['id']:r['expected_error_kind']for r in fr['vm_only']}}
        collector.parse_probe(collector.canonical(value),expected,fr)
        bad=copy.deepcopy(value);bad['cases'].pop()
        with self.assertRaises(collector.safe.OracleError):collector.parse_probe(collector.canonical(bad),expected,fr)
        bad=copy.deepcopy(value);bad['compiled_inputs'][0]['sha256']='f'*64
        with self.assertRaises(collector.safe.OracleError):collector.parse_probe(collector.canonical(bad),expected,fr)
        row=fr['cases'][0];vm=row['expected_observations'];wrong={**vm,'same_alias':False}
        self.assertTrue(collector.compare(row,vm,vm)['passed']);self.assertFalse(collector.compare(row,vm,wrong)['passed'])
        self.assertFalse(collector.compare(row,vm,{**vm,'same_alias':1})['passed'])

    def test_every_retained_output_path_is_closed_and_unique(self):
        paths=collector.expected_outputs();self.assertEqual(len(paths),74);self.assertEqual(len(set(paths)),74)
        self.assertTrue(all(not Path(p).is_absolute() and '..'not in Path(p).parts for p in paths))
        for i in range(8):self.assertIn(f'case{i}/COMPARISON.json',paths);self.assertIn(f'case{i}/STATUS.json',paths)
        self.assertIn('probe.elf',paths);self.assertIn('python.elf',paths);self.assertIn('source/INDEX.json',paths)
        self.assertEqual(len([p for p in paths if p.startswith('source/file')]),19)

    def test_verify_rejects_missing_manifest_outputs_before_reading_fake_observations(self):
        expected={**synthetic_control(),'root_reviewed':True}
        manifest={'status':'MATCHED8-6-EXCLUSIONS','matches':8,'vm_only':6,'owner_acceptance':None,'control':expected,'approval_sha256':collector.digest(collector.canonical(expected)),'outputs':[]}
        raw=collector.canonical(manifest)
        def read(path,cap):
            if path.name=='MANIFEST.json':return raw
            if path.name=='COMPLETE':return (collector.digest(raw)+'\n').encode()
            raise AssertionError('No fake actual output may be read')
        with patch.object(collector.safe,'read_bytes',side_effect=read),patch.object(collector.safe,'pin')as pin:
            with self.assertRaises(collector.safe.OracleError):collector.verify(Path('/fixed/repo'))
        pin.assert_not_called()

    def test_cli_has_no_worker_path_source_fixture_or_abbreviated_mode_selectors(self):
        for args in [[],['--mode','capture','--source','unknown'],['--worker'],['--mo','prepare'],['--mode','unknown']]:
            with patch.object(collector,'run')as run,patch('sys.stderr',new=io.StringIO()):
                with self.assertRaises(SystemExit):collector.main(args)
            run.assert_not_called()

class RetainedConsistencyGuards(unittest.TestCase):
    def test_probe_and_all_eight_workers_require_exact_raw_streams(self):
        result={'stdout':'fixed UTF-8 café\n','stderr':'','status':0}
        for folder in ['probe']+[f'case{i}'for i in range(8)]:
            cap=65536 if folder=='probe'else 8192
            base=Path('/fixed/evidence')/folder
            files={'RESULT.json':collector.canonical(result),'STATUS.json':collector.canonical({'status':0,'complete':True}),'stdout.raw':result['stdout'].encode(),'stderr.raw':b''}
            def read(path,bound):
                self.assertEqual(path.parent,base)
                if path.suffix=='.raw':self.assertEqual(bound,cap)
                return files[path.name]
            with patch.object(collector.safe,'read_bytes',side_effect=read):
                self.assertEqual(collector.retained_result(base,cap),result)
                for stream in ('stdout','stderr'):
                    original=files[stream+'.raw'];files[stream+'.raw']=original+b'changed'
                    with self.assertRaises(collector.safe.OracleError):collector.retained_result(base,cap)
                    files[stream+'.raw']=original

    def test_partial_or_contradictory_status_and_untyped_result_never_verify(self):
        result={'stdout':'','stderr':'','status':0}
        files={'RESULT.json':collector.canonical(result),'STATUS.json':collector.canonical({'status':0,'complete':True}),'stdout.raw':b'','stderr.raw':b''}
        with patch.object(collector.safe,'read_bytes',side_effect=lambda path,cap:files[path.name]):
            for status in [{'status':None,'complete':False},{'status':0,'complete':False},{'status':1,'complete':True},{'status':True,'complete':True},{'status':0,'complete':1},{'status':0,'complete':True,'extra':'unknown'}]:
                files['STATUS.json']=collector.canonical(status)
                with self.assertRaises(collector.safe.OracleError):collector.retained_result(Path('/fixed/probe'),65536)
            files['STATUS.json']=collector.canonical({'status':0,'complete':True})
            for bad in [{**result,'status':True},{**result,'stdout':None},{**result,'extra':'unknown'}]:
                files['RESULT.json']=collector.canonical(bad)
                with self.assertRaises(collector.safe.OracleError):collector.retained_result(Path('/fixed/probe'),65536)

    def test_verifier_requires_the_fixed_root_approval_not_only_manifest_control(self):
        expected={**synthetic_control(),'root_reviewed':True};root=Path(expected['root'])
        fixed=root/collector.BASE/(collector.CAPTURE+'-APPROVED.json')
        with patch.object(collector.safe,'read_bytes',return_value=collector.canonical(expected))as read:
            self.assertEqual(collector.retained_approval(root,expected),expected)
            read.assert_called_once_with(fixed,collector.CAP)
        for field,value in [('root_reviewed',False),('root_reviewed',1),('scope','different Root approval'),('capture','different-capture')]:
            altered={**expected,field:value}
            with patch.object(collector.safe,'read_bytes',return_value=collector.canonical(altered)):
                with self.assertRaises(collector.safe.OracleError):collector.retained_approval(root,expected)
        with patch.object(collector.safe,'read_bytes',side_effect=collector.safe.OracleError('Missing ordinary approval')):
            with self.assertRaises(collector.safe.OracleError):collector.retained_approval(root,expected)


class IndependentDirectoryCursorGuards(unittest.TestCase):
    def test_both_scanners_open_fresh_relative_description_and_close_only_their_own_fd(self):
        import contextlib
        from types import SimpleNamespace
        for module in [collector,worker]:
            rows=[SimpleNamespace(name='item'),SimpleNamespace(name='alias')]
            with patch.object(module.os,'open',return_value=81)as opened, \
                 patch.object(module.os,'scandir',return_value=contextlib.nullcontext(iter(rows)))as scanned, \
                 patch.object(module.os,'close')as closed:
                self.assertEqual(module.bounded_names(20),['alias','item'])
                opened.assert_called_once_with('.',module.os.O_RDONLY|module.os.O_DIRECTORY|module.os.O_NOFOLLOW|module.os.O_CLOEXEC,dir_fd=20)
                scanned.assert_called_once_with(81)
                closed.assert_called_once_with(81)

    def test_both_scanners_close_fresh_fd_on_error_and_keep_original_sixteen_entry_bound(self):
        import contextlib
        from types import SimpleNamespace
        for module in [collector,worker]:
            with patch.object(module.os,'open',return_value=81), \
                 patch.object(module.os,'scandir',side_effect=OSError('SYNTHETIC scan failure')), \
                 patch.object(module.os,'close')as closed:
                with self.assertRaises(OSError):module.bounded_names(20)
                closed.assert_called_once_with(81)
            rows=[SimpleNamespace(name=str(i))for i in range(17)]
            with patch.object(module.os,'open',return_value=81), \
                 patch.object(module.os,'scandir',return_value=contextlib.nullcontext(iter(rows))), \
                 patch.object(module.os,'close')as closed:
                with self.assertRaises((ValueError,collector.safe.OracleError)):module.bounded_names(20)
                closed.assert_called_once_with(81)

    def test_mocked_consumed_borrowed_cursor_yields_full_alias_inventory_repeatedly(self):
        # Mocked directory/file descriptions only. A scan of borrowed20 is empty;
        # each fresh81 has all three entries. No actual fixture/child is created.
        import contextlib
        from types import SimpleNamespace
        names=['alias','copy','item']
        content={'alias':b'updated\n','copy':b'seed\n','item':b'updated\n'}
        datafds={'alias':82,'copy':83,'item':84}
        info={name:SimpleNamespace(st_uid=1000,st_mode=0o100600,st_size=len(content[name]),
            st_nlink=1 if name=='copy'else 2,st_dev=7,st_ino=11 if name=='copy'else 10)for name in names}
        def opened(name,flags,*,dir_fd):
            self.assertEqual(dir_fd,20)
            if name=='.':
                self.assertEqual(flags,worker.os.O_RDONLY|worker.os.O_DIRECTORY|worker.os.O_NOFOLLOW|worker.os.O_CLOEXEC)
                return 81
            return datafds[name]
        def scanned(fd):
            rows=[]if fd==20 else[SimpleNamespace(name=name)for name in names]
            self.assertIn(fd,[20,81])
            return contextlib.nullcontext(iter(rows))
        def fstat(fd):return next(info[name]for name in names if datafds[name]==fd)
        def read(fd,bound):
            self.assertEqual(bound,8193)
            return next(content[name]for name in names if datafds[name]==fd)
        expected=None
        for module in [worker,collector]:
            with patch.object(module.os,'open',side_effect=opened)as callopen, \
                 patch.object(module.os,'scandir',side_effect=scanned)as callscan, \
                 patch.object(module.os,'close'),patch.object(module.os,'getuid',return_value=1000), \
                 patch.object(module.os,'stat',side_effect=lambda name,**kwargs:info[name]), \
                 patch.object(module.os,'fstat',side_effect=fstat),patch.object(module.os,'read',side_effect=read), \
                 patch.object(collector,'open_directory',return_value=20):
                for _ in range(2):
                    actual=worker.inventory(20,set(names),{})if module is worker else collector.inventory_fixture(Path('/fixed/fixture'),0,expected)
                    self.assertEqual([x['path']for x in actual],names)
                    self.assertEqual([x['links']for x in actual],[2,1,2])
                    self.assertEqual([x['alias_group']for x in actual],['alias','copy','alias'])
                    self.assertEqual([x['sha256']for x in actual],[collector.digest(content[name])for name in names])
                    if expected is None:expected=actual
                    self.assertEqual(actual,expected)
                self.assertEqual([c.args[0]for c in callscan.call_args_list],[81,81])
                self.assertEqual(sum(c.args[0]=='.'for c in callopen.call_args_list),2)

if __name__=='__main__':unittest.main()
