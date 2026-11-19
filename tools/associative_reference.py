#!/usr/bin/env python3
"""Fixed developer Bash5.3 builtin map recipes. Never accepts learner code or paths."""
import argparse
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import tempfile
import time
from native_reopen import inherited_lock,read_bytes,pin
from automation_before10_oracle import pinned_executable
from case_bash_oracle import capture_command
from startup_reference import limits,Budget,output,fresh,strict
ROOT=Path(__file__).resolve().parents[1]
FIXTURE='tests/fixtures/shell-associative-reference.json'
FIXTURE_SHA='2d8dba086746f90e9a24427b5c8b3c6feecce81b8aa633f07fa960a0d15ce6d3'
BASE='.build-artifacts/reference-runs/shell-associative'
PROBE='target/debug/examples/associative_reference'
OBSERVATIONS_SHA='18b0f588cb7cee12a4ecceebb61da3d3007ba434b87a872efcbb51311e5551e5'  # Actual fixed16 Bash observations, independently read and sealed.
OBSERVATION_SEAL_SHA='f1a52be0b217151a8414e20a27e8b0024fc4f1e8dd66657ac1ac66014886d76c'
CAP=4*1024*1024
INPUTS=(FIXTURE,'tools/associative_reference.py','tools/tests/test_associative_reference.py','tools/startup_reference.py','tools/native_reopen.py','tools/automation_before10_oracle.py','tools/case_bash_oracle.py','tools/dev.py','tools/build_safety.py')
SYSTEM={name:Path('/usr/bin')/name for name in ['bash']}
def require(ok,message):
    if not ok:raise ValueError(message)
def digest(p,cap=CAP):return dict(path=str(p),**pin(p,cap))
def key(c):return hashlib.sha256(json.dumps(c,sort_keys=True,separators=(',',':')).encode()).hexdigest()[:16]
def recipes():
    raw=read_bytes(ROOT/FIXTURE,256*1024);require(hashlib.sha256(raw).hexdigest()==FIXTURE_SHA,'Fixed recipes changed; exact review required')
    v=strict(raw);require(set(v)=={'schema_version','cases'}and v['schema_version']==1 and len(v['cases'])==16,'Closed16 recipe roster')
    require(len({r['id']for r in v['cases']})==16,'Duplicate recipe ID')
    for r in v['cases']:
        require(set(r)=={'id','source','files'}and re.fullmatch('[a-z][a-z-]{0,63}',r['id']),'Fixed recipe shape')
        require(isinstance(r['source'],str)and len(r['source'])<=2048 and '\0'not in r['source'],'Bounded reviewed source')
        require(len(r['files'])<=8,'Bounded sample inventory')
        for name,text in r['files'].items():
            require(isinstance(name,str)and re.fullmatch(r'[A-Za-z0-9_. -]{1,80}',name)and name not in ['.','..'],'Single fixture basename only')
            require(isinstance(text,str)and len(text)<=4096 and '\0'not in text,'Bounded UTF8 sample')
    return v['cases']
def bash53_version(actual):
    require(type(actual.get('status'))is int and actual['status']==0 and actual.get('stderr')==''
            and isinstance(actual.get('stdout'),str)and len(actual['stdout'])<=128
            and re.fullmatch(r'5\.3(?:\.[0-9]+)?(?:\([0-9]+\))?(?:-[A-Za-z0-9._-]+)?\n',actual['stdout']),
            'Reviewed Bash5.3 required; actual version evidence retained')
    return actual['stdout'].removesuffix('\n')
def engine_paths():
    text=(ROOT/'examples/support/associative_inputs.rs').read_text()
    names=re.findall(r'\(\s*"([^"\n]+)",\s*include_bytes!',text,re.M)
    require(len(names)==556 and len(set(names))==556,'Closed current19 source inventory')
    return names+['examples/associative_reference.rs','examples/support/associative_inputs.rs',FIXTURE]
def control(mode):
    paths=sorted(set(INPUTS)|(set(engine_paths())if mode=='compare'else set()))
    programs={'probe':digest(ROOT/PROBE,1024*1024*1024)}if mode=='compare'else{name:digest(p,16*1024*1024)for name,p in SYSTEM.items()}
    return dict(purpose='Fixed Bash5.3 associative '+mode,root_reviewed=False,fixture_sha256=FIXTURE_SHA,pins=[digest(ROOT/p)for p in paths],executables=programs)
def approval(base,mode):
    c=control(mode);a=strict(read_bytes(base/(mode+'-approved-'+key(c)+'.json'),CAP))
    require(a.get('root_reviewed')is True and a=={**c,'root_reviewed':True},'Independent exact source/executable approval required');return c
def snapshot(work):
    files={};total=0
    for p in sorted(work.iterdir()):
        info=p.lstat();require(stat.S_ISREG(info.st_mode)and info.st_nlink==1 and len(files)<64,'No fixture directories/symlinks/special files')
        raw=read_bytes(p,262144);total+=len(raw)+len(p.name);require(total<=262144,'Fixture memory bound');files['/work/'+p.name]=raw.decode('utf-8')
    return files
def retain_inputs(run,c):
    for row in c['pins']:
        source=Path(row['path']);name=source.relative_to(ROOT);dest=run/'approved-source'/name;dest.parent.mkdir(parents=True,exist_ok=True)
        raw=read_bytes(source,CAP);require(len(raw)==row['bytes']and hashlib.sha256(raw).hexdigest()==row['sha256'],'Approved input changed before retention')
        with dest.open('xb')as f:f.write(raw);f.flush();os.fsync(f.fileno())
        require(pin(dest,CAP)=={k:row[k]for k in ['bytes','sha256']},'Retained input differs')
def retain_executable(run,name,fd,meta):
    folder=run/'executables';folder.mkdir(exist_ok=True);dest=folder/name
    with dest.open('xb')as f:
        at=0
        while at<meta['bytes']:
            raw=os.pread(fd,min(1024*1024,meta['bytes']-at),at);require(bool(raw),'Held executable shrank');f.write(raw);at+=len(raw)
        f.flush();os.fsync(f.fileno())
    require(pin(dest,16*1024*1024)=={k:meta[k]for k in ['bytes','sha256']},'Retained held utility differs')
def sealed_reference(base):
    require(OBSERVATIONS_SHA is not None and OBSERVATION_SEAL_SHA is not None,'Actual independent reference seal pending')
    folder=base/'bash-r1';raw=read_bytes(folder/'observations.json',CAP);sealraw=read_bytes(folder/'INDEPENDENT_OBSERVATION_SEAL.json',CAP)
    require(hashlib.sha256(raw).hexdigest()==OBSERVATIONS_SHA and hashlib.sha256(sealraw).hexdigest()==OBSERVATION_SEAL_SHA,'Sealed reference/independent receipt changed')
    seal=strict(sealraw);controlraw=read_bytes(folder/'control.json',CAP);c=strict(controlraw);approved=strict(read_bytes(folder/'approval.json',CAP))
    require(approved=={**c,'root_reviewed':True},'Retained exact observation approval differs')
    require(seal['observations_sha256']==OBSERVATIONS_SHA and seal['fixture_sha256']==FIXTURE_SHA and seal['control_sha256']==hashlib.sha256(controlraw).hexdigest(),'Independent seal binding differs')
    value=strict(raw);require(value['fixture_sha256']==FIXTURE_SHA,'Reference fixture binding');return value
def observe(base,rows,lock):
    c=approval(base,'observe');run=fresh(base,'bash-r1');output(run/'control.json',c);retain_inputs(run,c)
    approved_raw=read_bytes(base/('observe-approved-'+key(c)+'.json'),CAP)
    with (run/'approval.json').open('xb')as f:f.write(approved_raw);f.flush();os.fsync(f.fileno())
    observed=[];budget=Budget();started=time.monotonic()
    with ExitStack()as stack:
        held={name:stack.enter_context(pinned_executable(Path(meta['path']),meta,16*1024*1024))for name,meta in c['executables'].items()}
        for name,(_,fd)in held.items():retain_executable(run,name,fd,c['executables'][name])
        version=fresh(run,'version');version_work=fresh(version,'work');version_capture=fresh(version,'capture')
        version_source='printf \'%s\\n\' "$BASH_VERSION"\n'
        with (version/'source.bash').open('x')as f:f.write(version_source);f.flush();os.fsync(f.fileno())
        version_env=dict(PATH='',LC_ALL='C',LANG='C',HOME=str(version_work),TMPDIR=str(version_work),TMP=str(version_work),TEMP=str(version_work))
        version_actual=capture_command([held['bash'][0],'--noprofile','--norc','-c',version_source,'fixed-associative-version'],version_capture,version_work,version_env,b'',budget,1024,timeout=3,inherited=(lock,held['bash'][1]),popen=lambda *a,**kw:subprocess.Popen(*a,preexec_fn=limits,**kw))
        output(version/'execution.json',version_actual)
        bash_version=bash53_version(version_actual)
        for r in rows:
            require(time.monotonic()-started<60,'Whole reference time bound');case=fresh(run,r['id']);work=fresh(case,'work');capture=fresh(case,'capture')
            for name,text in r['files'].items():
                with (work/name).open('x')as f:f.write(text)
            # Exactly the reviewed builtin-only source; no utility wrappers,
            # learner arguments, startup files, command selectors or PATH lookup.
            source=r['source'];output(case/'recipe.json',r)
            with (case/'source.bash').open('x')as f:f.write(source);f.flush();os.fsync(f.fileno())
            env=dict(PATH='',LC_ALL='C',LANG='C',HOME=str(work),TMPDIR=str(work),TMP=str(work),TEMP=str(work))
            actual=capture_command([held['bash'][0],'--noprofile','--norc','-c',source,'fixed-associative'],capture,work,env,b'',budget,65536,timeout=3,inherited=tuple([lock]+[fd for _,fd in held.values()]),popen=lambda *a,**kw:subprocess.Popen(*a,preexec_fn=limits,**kw))
            output(case/'execution.json',actual);require(actual['status']==0 and not actual['stderr'],'Reference process failed/limited; actual streams retained')
            observed.append(dict(id=r['id'],status=actual['status'],stdout=actual['stdout'],stderr=actual['stderr'],files=snapshot(work)))
    output(run/'observations.json',dict(purpose='Actual fixed Bash5.3 builtin recipes; no learner execution',fixture_sha256=FIXTURE_SHA,bash_version=bash_version,observations=observed))
    print('Actual16builtin reference recipes retained; independent raw readback/seal required: '+str(run))
def normalize(row):
    o=row['outcome'];require(isinstance(o,dict)and set(o)=={'stdout','stderr','status','files','error'}and o['error']is None,'Supported fixed recipe failed or outcome shape changed')
    require(type(o['status'])is int and 0<=o['status']<=255 and isinstance(o['stdout'],str)and isinstance(o['stderr'],str),'Typed fixed outcome streams/status')
    require(isinstance(o['files'],dict)and len(o['files'])<=64 and all(isinstance(n,str)and n.startswith('/work/')and '/'not in n[6:]and isinstance(t,str)for n,t in o['files'].items()),'Only flat virtual reference files')
    return dict(id=row['id'],status=o['status'],stdout=o['stdout'],stderr=o['stderr'],files=o['files'])
def compare(base,rows,lock):
    reference=sealed_reference(base)
    c=approval(base,'compare');run=Path(tempfile.mkdtemp(prefix='compare-'+key(c)+'-',dir=base));output(run/'control.json',c)
    compiled=set(engine_paths())
    controller=dict(pins=[row for row in c['pins']if Path(row['path']).relative_to(ROOT).as_posix()not in compiled])
    controller_root=fresh(run,'controller-source');retain_inputs(controller_root,controller)
    for name in engine_paths():
        p=run/'compiled-source'/name;p.parent.mkdir(parents=True,exist_ok=True)
        with p.open('xb')as f:f.write(read_bytes(ROOT/name,CAP));f.flush();os.fsync(f.fileno())
    meta=c['executables']['probe']
    with pinned_executable(Path(meta['path']),meta,1024*1024*1024)as(exe,fd):
        with (run/'probe-elf').open('xb')as f:
            at=0
            while at<meta['bytes']:
                b=os.pread(fd,min(1024*1024,meta['bytes']-at),at);require(bool(b),'Probe shrank');f.write(b);at+=len(b)
            f.flush();os.fsync(f.fileno())
        require(pin(run/'probe-elf',1024*1024*1024)=={k:meta[k]for k in ['bytes','sha256']},'Retained probe differs')
        result=capture_command([exe],fresh(run,'capture'),ROOT,os.environ.copy(),b'',Budget(),CAP,timeout=60,inherited=(lock,fd))
    output(run/'execution.json',result);require(result['status']==0 and not result['stderr'],'Probe failed; actual streams retained');vm=strict(result['stdout'])
    require(vm['evaluator']==19 and vm['fixture_sha256']==FIXTURE_SHA,'Actual19 fixture required')
    expected=[dict(path=n,**pin(ROOT/n,CAP))for n in engine_paths()];require(vm['source_bindings']==expected,'Stale compiled closure')
    require([r['id']for r in vm['outcomes']]==[r['id']for r in rows]==[r['id']for r in reference['observations']],'Reference/probe order differs')
    normalized=[normalize(r)for r in vm['outcomes']]
    failures=[a['id']for a,b in zip(normalized,reference['observations'],strict=True)if a!=b]
    output(run/'report.json',dict(purpose='Exact supported Bash5.3 stdout/stderr/status/file maps; no unsupported-diagnostic or general conformance claim',actual_vm=vm,reference=reference,normalized=normalized,failures=failures))
    require(not failures,'Mismatches retained: '+','.join(failures));print('PASS16actual Bash5.3 associative comparisons: '+str(run))
def main():
    p=argparse.ArgumentParser(description=__doc__,allow_abbrev=False);p.add_argument('--mode',choices=['prepare-observe','observe','prepare-compare','compare'],required=True);a=p.parse_args();lock=inherited_lock()
    require((ROOT/'.git').is_dir()and not(ROOT/'.git').is_symlink()and Path(os.environ.get('CARGO_TARGET_DIR',''))==ROOT/'target','MAIN/default inherited runner only')
    rows=recipes();base=ROOT/BASE;base.mkdir(parents=True,exist_ok=True);require(base.resolve()==base and not base.is_symlink(),'Project-disk canonical output only')
    if a.mode.startswith('prepare-'):
        mode=a.mode.removeprefix('prepare-');c=control(mode);output(base/(mode+'-control-'+key(c)+'.json'),c);print('PROPOSED ONLY: '+mode+' '+key(c));return
    if a.mode=='observe':observe(base,rows,lock)
    else:compare(base,rows,lock)
if __name__=='__main__':main()
