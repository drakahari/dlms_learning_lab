#!/usr/bin/env python3
"""Fixed developer index comparison; never accepts learner input or executes it."""
import argparse
from contextlib import ExitStack
import ctypes
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import stat
import subprocess
import time
from automation_before10_oracle import pinned_executable
from case_bash_oracle import capture_command, closed_environment
from native_reopen import inherited_lock, pin, read_bytes
from startup_reference import Budget, fresh, limits, output, strict
ROOT=Path(__file__).resolve().parents[1]
BASE='.build-artifacts/reference-runs/file-index-r2'
FIXTURE='tests/fixtures/file-index-reference-v1.json'
FIXTURE_SHA='d184f88255c19528ee234d161c8d77003c04a20a7c85b39966912c83b9489e7f'
PROBE='target/debug/examples/file_index_reference'
BINDINGS='examples/support/file_index_bindings.rs'
EXTRAS=('Cargo.toml','Cargo.lock','examples/file_index_reference.rs',BINDINGS,'examples/support/file_index_digest.rs',FIXTURE)
TOOLS=('tools/file_index_reference.py','tools/tests/test_file_index_reference.py','tools/dev.py','tools/build_safety.py','tools/native_reopen.py','tools/automation_before10_oracle.py','tools/case_bash_oracle.py','tools/startup_reference.py')
PHASES=('initial','stale-live','rebuilt-live','config-only','config-rebuilt')
CAP=4*1024*1024
PR_SET_NO_NEW_PRIVS=38
PR_GET_NO_NEW_PRIVS=39

def require(ok,message):
    if not ok:raise ValueError(message)
def digest(raw):return hashlib.sha256(raw).hexdigest()
def safe_name(p):return isinstance(p,str) and re.fullmatch(r'/scan(?:/[A-Za-z0-9_.-]{1,80}){0,4}',p) is not None and all(v not in ('.','..') for v in p.split('/')[1:])

def validate_fixture(v):
    require(isinstance(v,dict) and set(v)=={'schema_version','directories','files','phases'} and type(v['schema_version'])is int and v['schema_version']==1,'Closed fixture fields/version')
    require(v['directories']==['/scan','/scan/active','/scan/active/nested','/scan/active-other','/scan/archive','/scan/archives'],'Fixed directory roster/order')
    require(isinstance(v['files'],list) and len(v['files'])==7,'Seven inert leaves')
    paths=set(v['directories'])
    for f in v['files']:
        require(set(f)=={'path','text','mode'} and safe_name(f['path']) and f['path']not in paths and str(Path(f['path']).parent)in v['directories'],'Fixed ordinary leaf path')
        require(type(f['mode'])is int and f['mode']in(0,0o600) and isinstance(f['text'],str) and len(f['text'].encode())<=8192 and '\0'not in f['text'],'Bounded inert leaf contents/mode')
        paths.add(f['path'])
    require(isinstance(v['phases'],list) and tuple(p['id']for p in v['phases'])==PHASES,'Closed five-phase roster/order')
    require(tuple(p['rebuild']for p in v['phases'])==(True,False,True,False,True),'Fixed rebuild positions')
    require([p['prunes']for p in v['phases']]==[['/scan/archive']]*3+[[],[]],'Fixed config transitions')
    mutations=[];count=0
    for p in v['phases']:
        require(set(p)=={'id','prunes','mutations','rebuild','queries'} and type(p['rebuild'])is bool,'Closed phase fields')
        require(isinstance(p['queries'],list)and 1<=len(p['queries'])<=5,'Bounded phase queries')
        ids=set()
        for q in p['queries']:
            require(set(q)=={'id','literal','existing'} and re.fullmatch('[a-z][a-z-]{0,40}',q['id']) and q['id']not in ids and type(q['existing'])is bool,'Unique typed query')
            require(isinstance(q['literal'],str)and re.fullmatch(r'[A-Za-z0-9/._-]{1,80}',q['literal']),'Only inert literal query')
            ids.add(q['id']);count+=1
        require(isinstance(p['mutations'],list)and len(p['mutations'])<=2,'Bounded fixed mutations')
        mutations.extend(p['mutations'])
    require(mutations==[{'op':'remove','path':'/scan/active/old.txt'},{'op':'copy','from':'/scan/source.txt','path':'/scan/active/new.txt'}] and count==17,'Exact fixed mutation/query count')
    return v

def fixture():
    raw=read_bytes(ROOT/FIXTURE,65536);require(digest(raw)==FIXTURE_SHA,'Fixed reviewed fixture bytes changed');return validate_fixture(strict(raw))

def load_prctl():
    # Initialize libc and fixed signatures in the parent, never load libraries after fork.
    lib=ctypes.CDLL(None,use_errno=True)
    fn=lib.prctl;fn.argtypes=[ctypes.c_int,ctypes.c_ulong,ctypes.c_ulong,ctypes.c_ulong,ctypes.c_ulong];fn.restype=ctypes.c_int
    return lib,fn

def core_limit():resource.setrlimit(resource.RLIMIT_CORE,(0,0))
class ChildGuard:
    """Only the forked disposable child may touch limits/security/proof pipe."""
    def __init__(self,writer,parent_pid,prctl,pid=os.getpid,write=os.write,close=os.close,apply_limits=limits,apply_core=core_limit):
        self.writer=writer;self.parent_pid=parent_pid;self.prctl=prctl;self.pid=pid;self.write=write;self.close=close;self.apply_limits=apply_limits;self.apply_core=apply_core
    def __call__(self):
        # This check MUST precede any limits, prctl or proof operation. Tests mock every effect.
        if self.pid()==self.parent_pid:raise RuntimeError('Child restriction callback refused in parent')
        try:
            if self.prctl(PR_SET_NO_NEW_PRIVS,1,0,0,0)!=0:raise RuntimeError('Cannot restrict child privilege gains')
            if self.prctl(PR_GET_NO_NEW_PRIVS,0,0,0,0)!=1:raise RuntimeError('Child no_new_privs readback failed')
            self.apply_limits();self.apply_core()
            if self.write(self.writer,b'N')!=1:raise RuntimeError('Child restriction proof write failed')
        finally:self.close(self.writer)

def verify_proof(marker,parent,child):
    require(marker==b'N'and type(child)is int and child>0 and child!=parent,'Exact one-byte child-only restriction proof required')

def capture_restricted(command,folder,work,env,budget,inherited,deadline=None):
    # No subprocess is created by import/prepare; this is called only after exact observe/compare approval.
    parent=os.getpid();lib,prctl=load_prctl();reader,writer=os.pipe2(os.O_CLOEXEC|os.O_NONBLOCK)
    state={'writer_open':True,'child_pid':None};guard=ChildGuard(writer,parent,prctl)
    def factory(*args,**kw):
        try:
            process=subprocess.Popen(*args,preexec_fn=guard,**kw);state['child_pid']=process.pid;return process
        finally:
            if state['writer_open']:os.close(writer);state['writer_open']=False
    try:
        remaining=3 if deadline is None else deadline-time.monotonic();require(remaining>0,'Whole reference deadline before child launch');timeout=min(3,remaining)
        actual=capture_command(command,folder,work,env,b'',budget,32768,timeout=timeout,inherited=tuple(inherited)+(writer,),popen=factory)
        # Writer was closed before exec and by parent after Popen. No post-exec process can forge it.
        try:marker=os.read(reader,2)
        except BlockingIOError:raise ValueError('Restriction proof pipe remained open; no blocking read permitted')
        verify_proof(marker,parent,state['child_pid'])
        output(folder/'restriction.json',{'parent_pid':parent,'child_pid':state['child_pid'],'marker_hex':marker.hex(),'timeout_seconds':timeout,'set_then_get_then_limits_then_core_then_write_close':True,'scope':'Child-before-exec; inheritance relies on Linux documented no_new_privs, not invented post-exec observation'})
        return actual
    finally:
        if state['writer_open']:os.close(writer)
        os.close(reader)
        del lib

def binding_names():
    text=read_bytes(ROOT/BINDINGS,256*1024).decode();pairs=re.findall(r'\(\s*"([^"\n]+)"\s*,\s*include_bytes!\(\s*"([^"\n]+)"\s*\)\s*,?\s*\)',text);names=[p for p,_ in pairs]
    require(all(not rel.startswith('/')and os.path.normpath('examples/support/'+rel)==p for p,rel in pairs),'Binding name must identify its actual included source')
    expected=set(EXTRAS)
    for base in ('src','scenarios','assets'):
        for path in (ROOT/base).rglob('*'):
            require(not path.is_symlink(),'Source closure contains symlink')
            if path.is_file():expected.add(path.relative_to(ROOT).as_posix())
            else:require(path.is_dir(),'Unexpected source kind')
    require(len(names)==len(set(names))and set(names)==expected and len(names)<=1024,'Complete exact current source plus six extras required')
    return sorted(names)

def control():
    compiled=[{'path':p,**pin(ROOT/p,4*1024*1024)}for p in binding_names()]
    programs={n:{'path':str(Path('/usr/bin')/n),**pin(Path('/usr/bin')/n,16*1024*1024)}for n in ('plocate','updatedb')}
    modes={}
    for n,m in programs.items():
        info=Path(m['path']).stat();require(stat.S_ISREG(info.st_mode)and not(info.st_mode&stat.S_ISUID),'No setuid reference executable')
        require(stat.S_IMODE(info.st_mode)==(0o2755 if n=='plocate'else 0o755),'Exact reviewed reference executable mode')
        modes[n]={'mode':stat.S_IMODE(info.st_mode),'uid':info.st_uid,'gid':info.st_gid}
    probe=ROOT/PROBE
    return {'purpose':'fixed-file-index-reference-v1','root':str(ROOT),'fixture_sha256':FIXTURE_SHA,'compiled_inputs':compiled,'tools':{p:pin(ROOT/p,4*1024*1024)for p in TOOLS},'programs':programs,'program_modes':modes,'probe':{'path':str(probe),**pin(probe,1024*1024*1024)},'restriction':'child-only PR_SET_NO_NEW_PRIVS=1/readback; never parent'}

def binding_commitment(rows):
    require(len(rows)<=1024 and len(rows)==len({r['path']for r in rows})and all(set(r)=={'path','bytes','sha256'}and isinstance(r['path'],str)and type(r['bytes'])is int and 0<=r['bytes']<=4*1024*1024 and isinstance(r['sha256'],str)and re.fullmatch('[0-9a-f]{64}',r['sha256'])for r in rows),'Unique closed bounded compiled pins');ordered=sorted(rows,key=lambda r:r['path']);return digest(json.dumps(ordered,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode('utf-8'))

def retain_inputs(run,c):
    folder=fresh(run,'reviewed-inputs');items={r['path']:{k:r[k]for k in('bytes','sha256')}for r in c['compiled_inputs']};items.update(c['tools'])
    for p,expected in items.items():
        raw=read_bytes(ROOT/p,4*1024*1024);require({'bytes':len(raw),'sha256':digest(raw)}==expected,'Retained source differs: '+p);dest=folder/p;dest.parent.mkdir(parents=True,exist_ok=True)
        with dest.open('xb')as f:f.write(raw);f.flush();os.fsync(f.fileno())

def retain_fd(dest,fd,meta):
    size=os.fstat(fd).st_size;require(0<size<=1024*1024*1024 and size==meta['bytes'],'Bound held executable')
    h=hashlib.sha256();at=0
    with dest.open('xb')as f:
        while at<size:
            b=os.pread(fd,min(1024*1024,size-at),at);require(b,'Held ELF shrank');f.write(b);h.update(b);at+=len(b)
        f.flush();os.fsync(f.fileno())
    require(h.hexdigest()==meta['sha256'],'Retained ELF mismatch')

class Files:
    def __init__(self,folder,v,stack):
        self.folder=folder;self.rows={};self.active={};self.stack=stack;self.rootfd=os.open(folder,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);stack.callback(os.close,self.rootfd);self.rootstat=os.fstat(self.rootfd);require(re.fullmatch(r'/[A-Za-z0-9/._-]+',str(folder))is not None,'Fixture parent path must fit the disclosed ASCII config grammar');require(folder.resolve()==folder and not folder.is_symlink()and stat.S_ISDIR(self.rootstat.st_mode)and self.rootstat.st_uid==os.geteuid()and stat.S_IMODE(self.rootstat.st_mode)==0o700,'Canonical owned private fixture root')
        self.add_dir('/scan');self.add_dir('/work')
        for p in v['directories'][1:]:self.add_dir(p)
        for f in v['files']:self.add_file(f['path'],f['text'].encode(),f['mode'])
        self.add_file('/work/index.conf',b'',0o600)
    def verify_root(self):
        now=self.folder.lstat();held=os.fstat(self.rootfd)
        require(self.folder.resolve()==self.folder and stat.S_ISDIR(now.st_mode) and (now.st_dev,now.st_ino,now.st_mode,now.st_uid,now.st_gid)==(held.st_dev,held.st_ino,held.st_mode,held.st_uid,held.st_gid) and (held.st_dev,held.st_ino,held.st_mode,held.st_uid,held.st_gid)==(self.rootstat.st_dev,self.rootstat.st_ino,self.rootstat.st_mode,self.rootstat.st_uid,self.rootstat.st_gid),'Original private fixture root changed')
    def path(self,p):
        self.verify_root();require(p in('/work','/work/index.conf','/work/private.db') or safe_name(p),'Canonical fixed fixture namespace');return self.folder/p[1:]
    def parentfd(self,p):
        name=str(Path(p).parent);row=None if name=='/'else self.rows[name];fd=self.rootfd if row is None else row['fd'];old=self.rootstat if row is None else row['original'];now=os.fstat(fd);require(stat.S_ISDIR(now.st_mode)and now.st_uid==os.geteuid()and(now.st_dev,now.st_ino)==(old.st_dev,old.st_ino),'Held original parent directory');return fd
    def add_dir(self,p):
        self.path(p);parent=self.parentfd(p);os.mkdir(Path(p).name,0o700,dir_fd=parent);fd=os.open(Path(p).name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=parent);self.stack.callback(os.close,fd);self.rows[p]={'fd':fd,'directory':True,'original':os.fstat(fd)};self.active[p]=True
    def add_file(self,p,raw,mode):
        require(p not in self.rows and len(raw)<=8192,'Exclusive bounded inert file');fd=os.open(Path(p).name,os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=self.parentfd(p));self.stack.callback(os.close,fd);require(os.write(fd,raw)==len(raw),'Complete inert write');os.fsync(fd);os.fchmod(fd,mode);self.rows[p]={'fd':fd,'directory':False,'original':os.fstat(fd)};self.active[p]=True
    def config(self,prunes):
        self.identity('/work/index.conf');text='PRUNEPATHS = "'+' '.join(str(self.path(p))for p in prunes)+'"\n';fd=self.rows['/work/index.conf']['fd'];os.ftruncate(fd,0);require(os.pwrite(fd,text.encode(),0)==len(text.encode()),'Complete known config write');os.fsync(fd)
    def mutate(self,m):
        p=m['path'];require(p in('/scan/active/old.txt','/scan/active/new.txt'),'Fixed leaf only')
        if m['op']=='remove':
            require(self.active[p],'Remove known active original');self.identity(p);os.unlink(Path(p).name,dir_fd=self.parentfd(p));self.active[p]=False
        else:
            require(m=={'op':'copy','from':'/scan/source.txt','path':'/scan/active/new.txt'},'Fixed copy only');self.identity(m['from']);source=self.rows[m['from']]['fd'];raw=os.pread(source,8193,0);require(len(raw)<=8192,'Bound known copy');self.add_file(p,raw,0o600)
    def identity(self,p):
        r=self.rows[p];s=os.fstat(r['fd']);o=r['original'];require((s.st_dev,s.st_ino)==(o.st_dev,o.st_ino)and s.st_uid==os.geteuid(),'Original held owned inode')
        require((stat.S_ISDIR(s.st_mode)if r['directory']else stat.S_ISREG(s.st_mode))and (s.st_nlink>=1 if r['directory']else s.st_nlink==(1 if self.active[p]else 0)),'Known type/link count')
        if self.active[p]:
            q=self.path(p).lstat();require((q.st_dev,q.st_ino,q.st_mode,q.st_nlink)==(s.st_dev,s.st_ino,s.st_mode,s.st_nlink),'Known path/FD identity')
        else:require(not self.path(p).exists()and not self.path(p).is_symlink(),'Removed leaf not replaced')
        return s
    def snapshot(self):
        self.verify_root();rows={}
        for p,r in self.rows.items():
            s=self.identity(p);b=b''if r['directory']else os.pread(r['fd'],8193,0);require(len(b)<=8192,'Bound original inert bytes')
            parent=os.fstat(self.parentfd(p))
            rows[p]={'active':self.active[p],'directory':r['directory'],'dev':s.st_dev,'ino':s.st_ino,'nlink':s.st_nlink,'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid,'mtime_ns':s.st_mtime_ns,'ctime_ns':s.st_ctime_ns,'parent_dev':parent.st_dev,'parent_ino':parent.st_ino,'parent_mode':stat.S_IMODE(parent.st_mode),'bytes':len(b),'sha256':digest(b)}
        allowed={p[1:]for p,on in self.active.items()if on}|{'work/private.db'}
        for p in self.folder.rglob('*'):require(not p.is_symlink()and p.relative_to(self.folder).as_posix()in allowed,'Unexpected fixture path')
        return rows

def db_snapshot(files):
    p=files.path('/work/private.db');parent=files.parentfd('/work/private.db');fd=os.open('private.db',os.O_RDONLY|os.O_NOFOLLOW,dir_fd=parent)
    try:
        s=os.fstat(fd);q=p.lstat();require((q.st_dev,q.st_ino)==(s.st_dev,s.st_ino)and stat.S_ISREG(s.st_mode)and s.st_nlink==1 and s.st_uid==os.geteuid()and 0<s.st_size<=1024*1024,'Bound original private DB')
        blocks=[];at=0
        while at<s.st_size:
            b=os.pread(fd,min(65536,s.st_size-at),at);require(b,'Private DB shrank');blocks.append(b);at+=len(b)
        after=os.fstat(fd);require((s.st_dev,s.st_ino,s.st_mode,s.st_nlink,s.st_size,s.st_mtime_ns,s.st_ctime_ns)==(after.st_dev,after.st_ino,after.st_mode,after.st_nlink,after.st_size,after.st_mtime_ns,after.st_ctime_ns),'Private DB changed during same-FD snapshot');raw=b''.join(blocks)
        return raw,{'dev':s.st_dev,'ino':s.st_ino,'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid,'nlink':s.st_nlink,'mtime_ns':s.st_mtime_ns,'ctime_ns':s.st_ctime_ns,'bytes':len(raw),'sha256':digest(raw)}
    finally:os.close(fd)

def normalized(actual,files,v):
    require(type(actual.get('status'))is int and actual['status']in(0,1),'Do not turn signals/errors/127 into NoMatch')
    require(actual['stderr']=='','Unexpected diagnostic retained; no silent stripping')
    allowed=set(v['directories'])|{f['path']for f in v['files']}|{'/scan/active/new.txt'};paths=[]
    for line in actual['stdout'].splitlines(keepends=True):
        require(line.endswith('\n')and line[:-1].startswith(str(files.folder)+'/'),'Exact known absolute path line')
        p=line[len(str(files.folder)):-1];require(p in allowed and p not in paths,'No outside/unreviewed/duplicate result');paths.append(p)
    require((actual['status']==0)==bool(paths),'Actual supported missing must be1/empty; found0/nonempty')
    return {'status':'Found'if paths else'NoMatch','paths':paths,'stdout':''.join(p+'\n'for p in paths)}

def approve(args,c,raw,mode):
    require(args.approved_control==digest(raw),'Independent exact CONTROL approval required')
    for n in('plocate','updatedb'):require(getattr(args,'approved_'+n)==c['programs'][n]['sha256'],'Exact original ELF approval')
    if mode=='compare':require(args.approved_probe==c['probe']['sha256'],'Exact compiled probe approval')

def observe(args,base,c,raw,lock):
    approve(args,c,raw,'observe');require(os.geteuid()!=0,'Ordinary private-index observation only');v=fixture();run=fresh(base,'observed');retain_inputs(run,c);execs=fresh(run,'executables');budget=Budget();started=time.monotonic();deadline=started+90;rows=[];phase_rows=[];last_db=None
    with ExitStack()as stack:
        held={n:stack.enter_context(pinned_executable(Path(m['path']),m,16*1024*1024))for n,m in c['programs'].items()}
        for n,(_,fd)in held.items():retain_fd(execs/n,fd,c['programs'][n])
        _,probe_fd=stack.enter_context(pinned_executable(Path(c['probe']['path']),c['probe'],1024*1024*1024));retain_fd(execs/'probe',probe_fd,c['probe'])
        for n in('updatedb','plocate'):
            d=fresh(run,'version-'+n);env=closed_environment({},d);output(d/'launch.json',{'argv':[held[n][0],'--version'],'env':env});a=capture_restricted([held[n][0],'--version'],d,d,env,budget,(lock,held[n][1]),deadline=deadline);output(d/'actual.json',a);require(a['status']==0 and a['stderr']==''and a['stdout'],'Actual bounded version required')
        fixture_dir=fresh(run,'fixture');files=Files(fixture_dir,v,stack)
        for phase in v['phases']:
            require(time.monotonic()-started<90,'Whole reference deadline');d=fresh(run,phase['id'])
            for m in phase['mutations']:files.mutate(m)
            files.config(phase['prunes']);before=files.snapshot();output(d/'before.json',before)
            if phase['rebuild']:
                cap=fresh(d,'update');argv=[held['updatedb'][0],'-l','0','-o',str(files.path('/work/private.db')),'-U',str(files.path('/scan')),'--config-file',str(files.path('/work/index.conf'))];env=closed_environment({},fixture_dir)
                output(cap/'launch.json',{'argv':argv,'env':env});a=capture_restricted(argv,cap,files.path('/work'),env,budget,(lock,held['updatedb'][1]),deadline=deadline);output(cap/'actual.json',a);require(a['status']==0 and a['stdout']==''and a['stderr']=='','Fixed update failed/diagnosed; raw retained')
            after=files.snapshot()
            # The only permitted updater tree effect is private DB publication in /work.
            for p in before:
                left=dict(before[p]);right=dict(after[p])
                if p=='/work':
                    for key in('mtime_ns','ctime_ns'):left.pop(key);right.pop(key)
                require(left==right,'Updater changed a supplied source/config inode/mode/body')
            dbraw,db=db_snapshot(files)
            if not phase['rebuild']:require(db==last_db,'No-rebuild phase changed its private DB')
            last_db=db
            with(d/'database.raw').open('xb')as f:f.write(dbraw);f.flush();os.fsync(f.fileno())
            output(d/'database.json',db);output(d/'after.json',after)
            phase_rows.append({'id':phase['id'],'rebuild':phase['rebuild'],'database':db,'config':read_bytes(files.path('/work/index.conf'),4096).decode()})
            for q in phase['queries']:
                require(time.monotonic()-started<90,'Whole reference deadline');cap=fresh(d,q['id']);pre=files.snapshot();dbpre=db_snapshot(files)[1];argv=[held['plocate'][0]]+(['-e']if q['existing']else[])+['-d',str(files.path('/work/private.db')),q['literal']];env=closed_environment({},fixture_dir)
                output(cap/'launch.json',{'argv':argv,'env':env,'query':q});a=capture_restricted(argv,cap,files.path('/work'),env,budget,(lock,held['plocate'][1]),deadline=deadline);output(cap/'actual.json',a);post=files.snapshot();dbpost=db_snapshot(files)[1];require(pre==post and dbpre==dbpost,'Read-only query changed original/config/DB');output(cap/'before.json',pre);output(cap/'after.json',post)
                rows.append({'phase':phase['id'],**q,'actual':a,'normalized':normalized(a,files,v)})
    require(time.monotonic()<=deadline,'Whole observation deadline exceeded; raw retained');require(control()==c,'Source/executable/mode changed during observation');output(run/'OBSERVATIONS.json',{'control_sha256':digest(raw),'fixture_sha256':FIXTURE_SHA,'phases':phase_rows,'observations':rows,'limits':'Private-l0 ordinary-user restricted original ELF only; no privileged/default global index/unsupported permission/root/order acceptance by assumption'})

def compare(args,base,c,raw,lock):
    approve(args,c,raw,'compare');require(os.geteuid()!=0,'Ordinary restricted model comparison only');original_raw=read_bytes(base/'observed/OBSERVATIONS.json',CAP);require(args.approved_observations==digest(original_raw),'Independent actual raw observation approval required');original=strict(original_raw);require(original['control_sha256']==digest(raw)and original['fixture_sha256']==FIXTURE_SHA and len(original['observations'])==17,'Exact reference authority')
    run=fresh(base,'compared');retain_inputs(run,c);budget=Budget();meta=c['probe']
    with pinned_executable(Path(meta['path']),meta,1024*1024*1024)as(exe,fd):
        retain_fd(run/'retained-probe',fd,meta);a=capture_restricted([exe],run,run,closed_environment({},run),budget,(lock,fd));output(run/'PROBE.json',a)
    require(a['status']==0 and a['stderr']=='','Actual fixed model probe failed');model=strict(a['stdout'].encode());require(model['purpose']=='fixed-file-index-reference-v1'and model['compiled_manifest_dir']==str(ROOT)and model['fixture_sha256']==FIXTURE_SHA and model['compiled_inputs_count']==len(c['compiled_inputs']) and model['compiled_inputs_sha256']==binding_commitment(c['compiled_inputs']),'Probe source/root/fixture identity mismatch');require(len(model['observations'])==17 and len(model['phases'])==5,'Exact model observation roster')
    require([(p['id'],p['rebuild'])for p in model['phases']]==[(p['id'],p['rebuild'])for p in original['phases']],'Exact phase identity/rebuild roster')
    comparisons=[]
    for host,vm in zip(original['observations'],model['observations']):
        require(all(host[k]==vm[k]for k in('phase','id','literal','existing')),'Ordered host/model query identity');expected={k:vm[k]for k in('status','paths','stdout')};comparisons.append({'phase':host['phase'],'id':host['id'],'host':host['normalized'],'model':expected,'passed':host['normalized']==expected})
    require(control()==c,'Source/executable changed during comparison');output(run/'COMPARISON.json',{'control_sha256':digest(raw),'observations_sha256':digest(original_raw),'comparisons':comparisons,'phase_model':model['phases'],'limitations':'Actual exact order/root mismatch must remain failure; no privileged/global visibility or broader utility acceptance'});require(all(r['passed']for r in comparisons),'Actual fixed comparison differs; retain all raw evidence')

def main():
    p=argparse.ArgumentParser();p.add_argument('--mode',choices=('prepare','observe','compare'),required=True)
    for name in('control','plocate','updatedb','probe','observations'):p.add_argument('--approved-'+name)
    args=p.parse_args();lock=inherited_lock(ROOT);require((ROOT/'.git').is_dir() and not (ROOT/'.git').is_symlink() and Path(os.environ.get('CARGO_TARGET_DIR',''))==ROOT/'target','MAIN/default-target runner only');fixture();base=ROOT/BASE
    if args.mode=='prepare':
        require(not base.exists()and not base.is_symlink(),'Preserve prior attempts');require(base.parent.is_dir()and base.parent.resolve()==base.parent and not base.parent.is_symlink(),'Use existing canonical project-disk reference parent');c=control();base.mkdir(mode=0o700);output(base/'CONTROL.json',c);print('PROPOSED only; no child launched');return
    require(base.is_dir()and base.resolve()==base and not base.is_symlink(),'Canonical prepared reference root');raw=read_bytes(base/'CONTROL.json',1024*1024);c=strict(raw);require(c==control(),'Exact prepared source/executable control changed')
    if args.mode=='observe':observe(args,base,c,raw,lock)
    else:compare(args,base,c,raw,lock)
if __name__=='__main__':main()
