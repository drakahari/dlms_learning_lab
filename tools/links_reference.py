#!/usr/bin/env python3
"""Fixed MAIN-only links8 Linux reference. Never learner data or arbitrary paths.

Prepare hashes only. Capture needs exact independent Root approval. Verify reads
retained evidence without running a probe or modifying any reference fixture.
"""
import argparse
from contextlib import ExitStack
import hashlib
import fcntl
import json
import os
from pathlib import Path
import re
import stat
import sys
import time
import case_bash_oracle as safe
from automation_before10_oracle import pinned_executable
from build_safety import owned_directory

ROOT=Path(__file__).resolve().parents[1]
BASE='.build-artifacts/reviews/links-reference/captures'
CAPTURE='links8-r2'
PROBE='target/debug/examples/links_reference'
FIXTURE='tests/fixtures/links-fixed-reference.json'
WORKER='tools/links_reference_worker.py'
ADAPTER='tools/links_fixed_host.py'
CAP=2*1024*1024
BINARY_CAP=256*1024*1024
EVIDENCE_CAP=512*1024*1024
DEADLINE=60.0
IDS=('shared-alias-independent-copy','unlink-one-then-last-name','move-relative-link-changes-parent','dangling-link-arrival-retirement','internal-directory-link-prefix','copy-follows-source-with-independent-data','fixed-file-errors-preserve-content','utf8-data-and-literal-names')
COMPILED=('src/links.rs','examples/links_reference.rs','examples/support/automation_digest.rs',FIXTURE,'Cargo.toml','Cargo.lock')
SOURCES=COMPILED+('tests/links_core.rs','docs/LINKS_SUBSET.md','docs/LINKS_REFERENCE.md',ADAPTER,WORKER,'tools/links_reference.py','tools/tests/test_links_fixed_host.py','tools/tests/test_links_reference.py','.cargo/config.toml','tools/dev.py','tools/case_bash_oracle.py','tools/automation_before10_oracle.py','tools/build_safety.py')
CORE_SHA='eb872623fa04ecd256d5fe5d7c9b1e9db14b319919e3fc34ac6ed5760068a56f'
FIXTURE_SHA='f463680464cea1fb468584970f972a894270620c55f28d38bbb4ac30b8b4c084'
PROBE_SOURCE_SHA='2eca3ed82bea6495fdb6da45470ed9ac9a3096de4e393e91512441c21482725a'

def digest(raw):return hashlib.sha256(raw).hexdigest()
def canonical(value):return json.dumps(value,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()
def strict(raw):
    def pairs(items):
        out={}
        for key,value in items:
            safe.require(key not in out,'Duplicate JSON key');out[key]=value
        return out
    safe.require(len(raw)<=CAP,'JSON bound')
    return json.loads(raw,object_pairs_hook=pairs)

def fixture(raw):
    safe.require(digest(raw)==FIXTURE_SHA,'Exact fixed8+6 fixture')
    value=strict(raw);safe.require(value['schema_version']==1 and value['actual_comparison']is None,'Unobserved fixed schema')
    safe.require(tuple(x['id']for x in value['cases'])==IDS and len(value['vm_only'])==6,'Exact fixed row inventory')
    safe.require(all(x['vm_observation']is None and x['linux_observation']is None for x in value['cases']+value['vm_only']),'Never fabricate observations')
    return value

def root_guard(root):
    safe.require((root/'.git').is_dir() and not(root/'.git').is_symlink(),'MAIN only')
    inherited=safe.require_guard(root)
    safe.require(os.environ.get('CARGO_TARGET_DIR')==str(root/'target'),'Exclusive MAIN default target')
    safe.require(os.getuid()!=0 and os.geteuid()==os.getuid(),'Ordinary owner only')
    return inherited

def source_pins(root):
    safe.require(safe.pin(root/'src/links.rs',CAP)['sha256']==CORE_SHA,'Reviewed pure inode core changed')
    safe.require(safe.pin(root/'examples/links_reference.rs',CAP)['sha256']==PROBE_SOURCE_SHA,'Reviewed fixed pure probe changed')
    fixture(safe.read_bytes(root/FIXTURE,CAP))
    chain=root/'.build-artifacts/toolchains/rustup/toolchains/stable-x86_64-unknown-linux-gnu'
    paths=[(root/name,CAP)for name in SOURCES]+[(chain/'bin/rustc',128*1024*1024),(chain/'lib/rustlib/multirust-channel-manifest.toml',CAP),(root/'.build-artifacts/toolchains/READY.json',CAP),(Path(sys.executable).resolve(strict=True),128*1024*1024),(root/PROBE,BINARY_CAP)]
    pins=[safe.pin(path,cap)for path,cap in paths]
    safe.require(len({p['path']for p in pins})==len(pins),'Unique exact source/compiler/executable pins')
    return pins

def control(root,pins):
    return {'schema_version':1,'root':str(root),'capture':CAPTURE,'root_reviewed':False,'pins':pins,
        'compiled_inputs':[{ 'path':name,'sha256':next(p['sha256']for p in pins if p['path']==str(root/name))}for name in COMPILED],
        'python_path':str(Path(sys.executable).resolve(strict=True)),'python_version':sys.version,
        'platform':{'system':os.uname().sysname,'release':os.uname().release,'machine':os.uname().machine},
        'host_rows':8,'vm_only':6,'scope':'Exact original inode/path fixtures only; no CLI/permission/fullPOSIX/curriculum/native/platform acceptance',
        'compiler_scope':'Cached rustc/channel manifest/READY+Cargo.lock and interpreter/kernel identity; not a complete toolchain/stdlib reproducibility audit'}

def check_control(value,root):
    safe.require(type(value)is dict and type(value.get('schema_version'))is int and value.get('schema_version')==1 and value.get('root')==str(root) and value.get('capture')==CAPTURE and type(value.get('host_rows'))is int and value.get('host_rows')==8 and type(value.get('vm_only'))is int and value.get('vm_only')==6,'Fixed approved control scope')
    pins=value.get('pins');safe.require(type(pins)is list and len(pins)==len(SOURCES)+5,'Complete source/compiler/ELF inventory')
    table={p['path']:p for p in pins};safe.require(len(table)==len(pins),'Unique approved pin inventory')
    chain=root/'.build-artifacts/toolchains/rustup/toolchains/stable-x86_64-unknown-linux-gnu'
    expected={str(root/name)for name in SOURCES}|{str(chain/'bin/rustc'),str(chain/'lib/rustlib/multirust-channel-manifest.toml'),str(root/'.build-artifacts/toolchains/READY.json'),str(root/PROBE),value['python_path']}
    safe.require(set(table)==expected,'No source/compiler pin omissions or arbitrary paths')
    for pin in pins:
        safe.require(set(pin)=={'path','bytes','sha256'} and type(pin['bytes'])is int and 0<=pin['bytes']<=BINARY_CAP and re.fullmatch('[0-9a-f]{64}',pin['sha256']),'Closed typed ordinary pin')
    safe.require(table[str(root/'src/links.rs')]['sha256']==CORE_SHA and table[str(root/FIXTURE)]['sha256']==FIXTURE_SHA and table[str(root/'examples/links_reference.rs')]['sha256']==PROBE_SOURCE_SHA,'Exact original model/fixture/probe identities')
    compiled=[{'path':name,'sha256':table[str(root/name)]['sha256']}for name in COMPILED]
    safe.require(canonical(value['compiled_inputs'])==canonical(compiled),'Exact six compiled-input pins')
    return value

def approve(raw,expected):
    actual=check_control(strict(raw),Path(expected['root']))
    safe.require(actual.get('root_reviewed')is True and canonical(actual)==canonical({**expected,'root_reviewed':True}),'Exact independent literal approval; no other field changes')
    return actual

def parse_probe(raw,expected,rows):
    value=strict(raw)
    safe.require(set(value)=={'schema_version','core_version','compiled_inputs','cases','vm_only'} and type(value['schema_version'])is int and value['schema_version']==1 and type(value['core_version'])is int and value['core_version']==1,'Closed pure probe result')
    safe.require(canonical(value['compiled_inputs'])==canonical(expected['compiled_inputs']),'Actual six compiled-source identities')
    safe.require(tuple(x['id']for x in value['cases'])==IDS,'All eight actual typed rows')
    for predicted,observed in zip(rows['cases'],value['cases']):
        safe.require(set(observed)=={'id','observations'} and same_shape(observed['observations'],predicted['expected_observations']),'Closed actual observation field types')
    safe.require(set(value['vm_only'])=={x['id']for x in rows['vm_only']},'All six VM-only results')
    return value

def same_shape(actual,expected):
    if type(actual)is not type(expected):return False
    if type(expected)is dict:return set(actual)==set(expected) and all(same_shape(actual[k],v)for k,v in expected.items())
    if type(expected)is str:return len(actual.encode())<=8192
    return type(expected)in (bool,int) and (type(expected)is not int or 0<=actual<=8192)

def compare(predicted,vm,host):
    wanted=predicted['expected_observations']
    return {'id':predicted['id'],'VM':vm,'Linux':host,'expected':wanted,'passed':same_shape(vm,wanted) and same_shape(host,wanted) and canonical(vm)==canonical(host)==canonical(wanted)}

def retain_fd(fd,expected,out,name,budget,cap):
    at=0;h=hashlib.sha256()
    with safe.new_file(out/name)as stream:
        while at<expected['bytes']:
            block=os.pread(fd,min(1024*1024,expected['bytes']-at),at);safe.require(bool(block),'Retained executable shrank');budget.charge(len(block));stream.write(block);h.update(block);at+=len(block)
        safe.require(not os.pread(fd,1,at) and h.hexdigest()==expected['sha256'],'Held executable/source bytes changed')
        stream.flush();os.fsync(stream.fileno())
    return safe.pin(out/name,cap)

def sealed_source(root,name,expected):
    # Tiny approved code snapshots only; fixture/output/build artifacts stay disk-backed.
    raw=safe.read_bytes(root/name,65536)
    safe.require(len(raw)==expected['bytes'] and digest(raw)==expected['sha256'],'Held approved source bytes')
    safe.require(hasattr(os,'memfd_create') and hasattr(fcntl,'F_ADD_SEALS'),'Linux source sealing unavailable; no unsafe fallback')
    fd=os.memfd_create('dlms-fixed-links-source',os.MFD_ALLOW_SEALING|os.MFD_CLOEXEC)
    try:
        at=0
        while at<len(raw):
            n=os.write(fd,raw[at:]);safe.require(n>0,'Source snapshot write');at+=n
        flags=fcntl.F_SEAL_WRITE|fcntl.F_SEAL_GROW|fcntl.F_SEAL_SHRINK|fcntl.F_SEAL_SEAL
        fcntl.fcntl(fd,fcntl.F_ADD_SEALS,flags)
        safe.require(fcntl.fcntl(fd,fcntl.F_GET_SEALS)&flags==flags,'All source mutation seals')
        out=fd;fd=-1;return out
    finally:
        if fd>=0:os.close(fd)

def open_directory(path):
    safe.require(path.is_absolute() and '..'not in path.parts,'Canonical fixed directory')
    fd=os.open('/',os.O_RDONLY|os.O_DIRECTORY|os.O_CLOEXEC)
    try:
        for name in path.parts[1:]:
            child=os.open(name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=fd);os.close(fd);fd=child
        out=fd;fd=-1;return out
    finally:
        if fd>=0:os.close(fd)

def new_private_dir(parent,name):
    safe.require(re.fullmatch('[A-Za-z0-9_-]{1,80}',name)is not None,'Fixed private directory basename')
    fd=open_directory(parent)
    try:
        os.mkdir(name,0o700,dir_fd=fd)
        child=os.open(name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=fd)
        try:
            info=os.fstat(child);safe.require(info.st_uid==os.getuid() and stat.S_IMODE(info.st_mode)==0o700,'Fresh private directory owner/mode')
        finally:os.close(child)
    finally:os.close(fd)
    return parent/name

def owned_base(root):
    # Walk/create only the four literal artifact components through directory FDs.
    # Never normalize away an alias before deciding whether it was acceptable.
    fd=open_directory(root)
    try:
        for name in BASE.split('/'):
            try:os.mkdir(name,0o700,dir_fd=fd)
            except FileExistsError:pass
            child=os.open(name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=fd)
            info=os.fstat(child)
            if info.st_uid!=os.getuid() or stat.S_IMODE(info.st_mode)&0o022:
                os.close(child);raise ValueError('Artifact parent owner/write permissions differ')
            os.close(fd);fd=child
    finally:os.close(fd)
    base=root/BASE
    safe.require(owned_directory(root,base)==base,'Verified disk base must remain the exact canonical fixed path')
    return base

def bounded_names(fd):
    # dup/scandir can share the caller's directory-stream offset. Open the same
    # directory relative to the held descriptor with a new open description.
    scan=os.open('.',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=fd)
    try:
        names=[]
        with os.scandir(scan)as entries:
            for entry in entries:
                safe.require(len(names)<16,'Unexpected retained directory entry count');names.append(entry.name)
        return sorted(names)
    finally:os.close(scan)

def inventory_fixture(folder,index,expected=None):
    # Parent/verify never follows a final symlink. Adapter source provides the
    # exact allowed-path/target tables; do not interpret operations from JSON.
    # Tables are duplicated as fixed literals, with source/mock checks for parity.
    names=(('item','alias','copy'),('item','alias'),('a','b','a/data','b/data','a/ref','b/ref'),('later','latest'),('real','real/data','shortcut','shortcut/data','shortcut/'),('original','view','copy'),('item','existing','item/child','item/','absent'),('-odd name','alias name','view name'))
    targets=({}, {}, {'a/ref':'data','b/ref':'data'},{'latest':'later'},{'shortcut':'real'},{'view':'original'},{},{'view name':'-odd name'})
    result=[];total=0;groups={}
    def walk(fd,prefix,depth):
        nonlocal total
        safe.require(depth<=2,'Fixed final depth')
        for name in bounded_names(fd):
            path=prefix+name;safe.require(path in names[index] and '/'not in name and name not in ('.','..'),'Unexpected retained fixture entry')
            info=os.stat(name,dir_fd=fd,follow_symlinks=False);safe.require(info.st_uid==os.getuid(),'Retained fixture owner')
            if stat.S_ISDIR(info.st_mode):
                safe.require(stat.S_IMODE(info.st_mode)==0o700,'Private retained directory');result.append({'path':path,'kind':'Directory'})
                child=os.open(name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=fd)
                try:walk(child,path+'/',depth+1)
                finally:os.close(child)
            elif stat.S_ISLNK(info.st_mode):
                target=os.readlink(name,dir_fd=fd);safe.require(targets[index].get(path)==target,'Retained symbolic target');result.append({'path':path,'kind':'SymbolicLink','target':target})
            else:
                safe.require(stat.S_ISREG(info.st_mode) and info.st_size<=8192 and 1<=info.st_nlink<=2,'Retained regular bounds')
                child=os.open(name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=fd)
                try:
                    now=os.fstat(child);safe.require((now.st_dev,now.st_ino)==(info.st_dev,info.st_ino),'Retained regular inode')
                    raw=os.read(child,8193);safe.require(len(raw)==info.st_size,'Retained bytes')
                finally:os.close(child)
                total+=len(raw);safe.require(total<=65536,'Retained data total')
                groups.setdefault((info.st_dev,info.st_ino),[]).append((len(result),info.st_nlink))
                result.append({'path':path,'kind':'Regular','bytes':len(raw),'sha256':digest(raw)})
            safe.require(len(result)<=16,'Retained entry bound')
    fd=open_directory(folder)
    try:walk(fd,'',0)
    finally:os.close(fd)
    for group in groups.values():
        safe.require(all(links==len(group)for _,links in group),'Every hardlink belongs to this fixture')
        label=min(result[i]['path']for i,_ in group)
        for i,links in group:result[i].update(links=links,alias_group=label)
    if expected is not None:safe.require(canonical(result)==canonical(expected),'Actual final fixture differs from worker evidence')
    return result

def child(command,folder,work,env,data,budget,inherited,timeout,started,stream_cap):
    remaining=DEADLINE-(time.monotonic()-started);safe.require(remaining>0,'Whole fixed reference deadline')
    try:
        result=safe.capture_command(command,folder,work,env,data,budget,stream_cap,timeout=min(timeout,remaining),inherited=inherited)
    except Exception as error:
        safe.json_new(folder/'STATUS.json',{'status':None,'complete':False,'error':str(error)[:2000]},budget,CAP);raise
    safe.json_new(folder/'STATUS.json',{'status':result['status'],'complete':True},budget,CAP)
    safe.json_new(folder/'RESULT.json',result,budget,CAP);safe.quiet_group()
    return result

def expected_outputs():
    paths=['probe/stdout.raw','probe/stderr.raw','probe/STATUS.json','probe/RESULT.json','probe.elf','python.elf','source/INDEX.json']
    for i in range(len(SOURCES)):paths.append(f'source/file{i}.raw')
    for i in range(8):paths += [f'case{i}/{name}'for name in ('stdout.raw','stderr.raw','STATUS.json','RESULT.json','PACKET.json','COMPARISON.json')]
    return sorted(paths)

def retained_result(folder,stream_cap):
    result=strict(safe.read_bytes(folder/'RESULT.json',CAP))
    safe.require(type(result)is dict and set(result)=={'stdout','stderr','status'} and type(result['stdout'])is str and type(result['stderr'])is str and type(result['status'])is int,'Closed retained process result')
    for name in ('stdout','stderr'):
        safe.require(safe.read_bytes(folder/(name+'.raw'),stream_cap)==result[name].encode('utf-8'),'Retained raw '+name+' differs from RESULT')
    status=strict(safe.read_bytes(folder/'STATUS.json',CAP))
    safe.require(type(status)is dict and set(status)=={'status','complete'} and status['complete']is True and type(status['status'])is int and status['status']==result['status'],'Retained STATUS differs from complete RESULT')
    return result

def retained_approval(root,expected):
    raw=safe.read_bytes(root/BASE/(CAPTURE+'-APPROVED.json'),CAP)
    return approve(raw,{**expected,'root_reviewed':False})

def verify(root):
    out=root/BASE/CAPTURE;raw=safe.read_bytes(out/'MANIFEST.json',CAP);m=strict(raw)
    safe.require(safe.read_bytes(out/'COMPLETE',65)==(digest(raw)+'\n').encode(),'Complete manifest marker')
    safe.require(m['status']=='MATCHED8-6-EXCLUSIONS' and m['matches']==8 and m['vm_only']==6,'All comparisons actually matched')
    expected=check_control(m['control'],root);safe.require(m['owner_acceptance']is None and expected['root_reviewed']is True and expected['capture']==CAPTURE and expected['root']==str(root),'Retained approved scope')
    safe.require(m['approval_sha256']==digest(canonical(expected)),'Approved normalized control pin')
    outputs=m['outputs'];safe.require([x['relative']for x in outputs]==expected_outputs(),'Every ordinary output retained, no missing rows')
    retained_approval(root,expected)
    for item in outputs:
        name=item['relative'];p=out/name;pin=safe.pin(p,BINARY_CAP if name in ('probe.elf','python.elf')else CAP)
        safe.require(pin['bytes']==item['bytes'] and pin['sha256']==item['sha256'],'Every retained output hash: '+name)
    names=strict(safe.read_bytes(out/'source/INDEX.json',CAP));safe.require(names==list(SOURCES),'Exact retained source list')
    pins={p['path']:p for p in expected['pins']}
    for i,name in enumerate(SOURCES):
        p=safe.pin(out/f'source/file{i}.raw',CAP);safe.require((p['bytes'],p['sha256'])==(pins[str(root/name)]['bytes'],pins[str(root/name)]['sha256']),'Retained compiled/helper/source bytes')
    for name,original in [('probe.elf',str(root/PROBE)),('python.elf',expected['python_path'])]:
        p=safe.pin(out/name,BINARY_CAP);safe.require((p['bytes'],p['sha256'])==(pins[original]['bytes'],pins[original]['sha256']),'Retained actual executable identity')
        with safe.ordinary(out/name,BINARY_CAP)as(stream,_):safe.require(stream.read(4)==b'\x7fELF','Retained ELF magic')
    fr=fixture(safe.read_bytes(out/f'source/file{SOURCES.index(FIXTURE)}.raw',CAP))
    probe=retained_result(out/'probe',65536);safe.require(probe['status']==0 and probe['stderr']=='','Retained probe process')
    actual=parse_probe(probe['stdout'].encode(),expected,fr)
    for i,row in enumerate(fr['cases']):
        result=retained_result(out/f'case{i}',8192);safe.require(result['status']==0 and result['stderr']=='','Retained worker status')
        host=strict(result['stdout'].encode());safe.require(set(host)=={'schema_version','index','result','inventory'} and type(host['schema_version'])is int and host['schema_version']==1 and type(host['index'])is int and host['index']==i and set(host['result'])=={'id','observations'} and host['result']['id']==row['id'],'Retained worker shape/order')
        comparison=compare(row,actual['cases'][i]['observations'],host['result']['observations']);safe.require(comparison['passed'],'Recomputed complete comparison')
        safe.require(strict(safe.read_bytes(out/f'case{i}/COMPARISON.json',CAP))==comparison,'Exact stored comparison')
        inventory_fixture(out/f'case{i}/fixture',i,host['inventory'])
    for row in fr['vm_only']:safe.require(actual['vm_only'][row['id']]==row['expected_error_kind'],'All six actual VM exclusions')
    print(json.dumps({'status':'COMPLETE-VERIFIED-NOT-OWNER-ACCEPTED','manifest_sha256':digest(raw),'matches':8,'vm_only':6}))
    return 0

def run(root,mode):
    inherited=root_guard(root)
    if mode=='verify':return verify(root)
    safe.require(mode in ('prepare','capture'),'Closed modes')
    fixed=fixture(safe.read_bytes(root/FIXTURE,CAP));pins=source_pins(root);expected=control(root,pins)
    base=owned_base(root);safe.require(not(base/CAPTURE).exists() and not(base/CAPTURE).is_symlink(),'Fresh capture only; no artifact deletion')
    if mode=='prepare':
        safe.json_new(base/(CAPTURE+'-PROPOSED.json'),expected,safe.Budget(CAP),CAP)
        print(json.dumps({'status':'PROPOSED-NO-CHILD-OR-REFERENCE-SYSCALLS','host_rows':8,'VM_only':6,'pins':len(pins)}));return 0
    approved_path=base/(CAPTURE+'-APPROVED.json');approved_raw=safe.read_bytes(approved_path,CAP);approved=approve(approved_raw,expected)
    out=new_private_dir(base,CAPTURE);budget=safe.Budget(EVIDENCE_CAP);started=time.monotonic()
    by_path={p['path']:p for p in pins}
    try:
        with ExitStack()as stack:
            vm,vfd=stack.enter_context(pinned_executable(root/PROBE,by_path[str(root/PROBE)],BINARY_CAP))
            py,pfd=stack.enter_context(pinned_executable(Path(expected['python_path']),by_path[expected['python_path']],128*1024*1024))
            worker_fd=sealed_source(root,WORKER,by_path[str(root/WORKER)]);stack.callback(os.close,worker_fd)
            adapter_fd=sealed_source(root,ADAPTER,by_path[str(root/ADAPTER)]);stack.callback(os.close,adapter_fd)
            approval,cinfo=stack.enter_context(safe.ordinary(approved_path,CAP))
            safe.require(approval.read()==approved_raw,'Same held approved control');approval.seek(0)
            # Preserve the exact actual code/executables even when a later pair
            # mismatches, times out or fails. Never overwrite prior evidence.
            retain_fd(vfd,by_path[str(root/PROBE)],out,'probe.elf',budget,BINARY_CAP)
            retain_fd(pfd,by_path[expected['python_path']],out,'python.elf',budget,128*1024*1024)
            source=new_private_dir(out,'source')
            for i,name in enumerate(SOURCES):
                raw=safe.read_bytes(root/name,CAP);pin=by_path[str(root/name)]
                safe.require(len(raw)==pin['bytes'] and digest(raw)==pin['sha256'],'Retain only approved source bytes')
                safe.write_new(source/f'file{i}.raw',raw,budget,CAP)
            safe.json_new(source/'INDEX.json',list(SOURCES),budget,CAP)
            safe.require(source_pins(root)==pins,'Input drift before any child')
            probe_folder=new_private_dir(out,'probe');work=new_private_dir(probe_folder,'work');tmp=new_private_dir(probe_folder,'tmp')
            result=child([vm],probe_folder,work,safe.closed_environment({},tmp),b'',budget,(*inherited,vfd),3,started,65536)
            safe.require(result['status']==0 and result['stderr']=='','Actual fixed pure probe failed')
            safe.require(not any(work.iterdir()),'Pure probe unexpectedly wrote files')
            observed=parse_probe(result['stdout'].encode(),expected,fixed);comparisons=[]
            for i,row in enumerate(fixed['cases']):
                folder=new_private_dir(out,f'case{i}');fixture_dir=new_private_dir(folder,'fixture');work=new_private_dir(folder,'run');tmp=new_private_dir(folder,'tmp')
                fd=open_directory(fixture_dir)
                try:
                    packet={'schema':1,'index':i,'collector_pid':os.getpid(),'fixture_fd':fd,'adapter_fd':adapter_fd,'worker_fd':worker_fd,'python_fd':pfd,'control_fd':approval.fileno(),'lock_fd':inherited[0]}
                    safe.json_new(folder/'PACKET.json',packet,budget,CAP)
                    data=canonical(packet);safe.require(len(data)<=4096,'Fixed worker packet bound')
                    result=child([py,'-I','-S','-B',f'/proc/self/fd/{worker_fd}'],folder,work,safe.closed_environment({},tmp),data,budget,(*inherited,pfd,worker_fd,adapter_fd,approval.fileno(),fd),5,started,8192)
                    safe.require(result['status']==0 and result['stderr']=='','Actual fixed worker failed; retain fixture')
                    actual=strict(result['stdout'].encode());safe.require(set(actual)=={'schema_version','index','result','inventory'} and type(actual['schema_version'])is int and actual['schema_version']==1 and type(actual['index'])is int and actual['index']==i,'Closed real worker result')
                    safe.require(set(actual['result'])=={'id','observations'} and actual['result']['id']==row['id'],'Actual fixed worker identity')
                    inventory_fixture(fixture_dir,i,actual['inventory']);safe.require(not any(work.iterdir()),'Unexpected files outside fixture descriptor')
                    comparison=compare(row,observed['cases'][i]['observations'],actual['result']['observations']);comparisons.append(comparison);safe.json_new(folder/'COMPARISON.json',comparison,budget,CAP)
                finally:os.close(fd)
            exclusions=all(observed['vm_only'][row['id']]==row['expected_error_kind']for row in fixed['vm_only'])
            safe.require(exclusions and all(c['passed']for c in comparisons),'Actual mismatches retained; never publish accepted completion')
        safe.require(time.monotonic()-started<DEADLINE and source_pins(root)==pins,'Whole deadline or source/compiler/ELF drift')
        outputs=[]
        for name in expected_outputs():
            pin=safe.pin(out/name,BINARY_CAP if name in ('probe.elf','python.elf')else CAP);outputs.append({'relative':name,'bytes':pin['bytes'],'sha256':pin['sha256']})
        manifest={'schema_version':1,'status':'MATCHED8-6-EXCLUSIONS','matches':8,'vm_only':6,'owner_acceptance':None,'control':approved,'approval_sha256':digest(canonical(approved)),'outputs':outputs,'comparisons':comparisons,'limits':{'probe_seconds':3,'worker_seconds':5,'whole_seconds':DEADLINE,'worker_stdout_stderr_bytes':8192,'worker_memory_bytes':256*1024*1024,'worker_file_bytes':8192,'worker_operations':128},'claims':'Fixed typed/Linux observations only; no CLI/fullPOSIX/curriculum/platform/novice acceptance'}
        pin,warning=safe.publish(out,manifest,budget)
    except Exception as error:
        safe.json_new(out/'FAILED.json',{'status':'INCOMPLETE-NOT-ACCEPTED','error':str(error)[:2000]},budget,CAP);raise
    print(json.dumps({'status':'MATCHED-NOT-INDEPENDENTLY-ACCEPTED','manifest_sha256':pin['sha256'],'durability_warning':warning}));return 0

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__,allow_abbrev=False);parser.add_argument('--mode',required=True,choices=['prepare','capture','verify']);args=parser.parse_args(argv)
    return run(ROOT.resolve(strict=True),args.mode)
if __name__=='__main__':raise SystemExit(main())
