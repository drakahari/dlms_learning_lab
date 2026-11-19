#!/usr/bin/env python3
"""Fixed whereis development comparison; no learner input or executable fixture."""
import argparse
from contextlib import ExitStack
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import time
import file_index_reference as shared
from file_index_reference import capture_restricted, binding_commitment, retain_fd
from automation_before10_oracle import pinned_executable
from case_bash_oracle import closed_environment
from native_reopen import inherited_lock, pin, read_bytes
from startup_reference import Budget, fresh, output, strict

ROOT=Path(__file__).resolve().parents[1]
BASE='.build-artifacts/reference-runs/whereis-r1'
FIXTURE='tests/fixtures/whereis-reference-v1.json'
FIXTURE_SHA='955905fe9eb14e61ec3b3967fe4b49002b1e70e90068b90bbc71985690a97c9a'
PROBE='target/debug/examples/whereis_reference'
BINDINGS='examples/support/whereis_bindings.rs'
EXTRAS=('Cargo.toml','Cargo.lock','examples/whereis_reference.rs',BINDINGS,'examples/support/whereis_digest.rs',FIXTURE)
TOOLS=('tools/whereis_reference.py','tools/tests/test_whereis_reference.py','tools/generate_whereis_bindings.py','tools/file_index_reference.py','tools/dev.py','tools/build_safety.py','tools/native_reopen.py','tools/automation_before10_oracle.py','tools/case_bash_oracle.py','tools/startup_reference.py')
IDS=('binary-zero-mode','manual-zero-mode','source-zero-mode','missing-binary','missing-manual','missing-source','directory-binary','directory-manual','read-without-own-search','owner-class-denied','unreadable-directory','ancestor-without-search','non-directory-search','missing-directory','case-sensitive','not-recursive','removed-live-entry','added-live-manual','unsupported-manual-section','unsupported-compressed-manual','unsupported-source-prefix','unsupported-source-extension','unsupported-multiple-manuals','unsupported-plain-manual')
CAP=4*1024*1024
OPTIONS={'binary':('-b','-B'),'manual':('-m','-M'),'source':('-s','-S')}
def require(ok,message):
    if not ok:raise ValueError(message)
def digest(raw):return hashlib.sha256(raw).hexdigest()
def safe_path(p):
    return isinstance(p,str)and re.fullmatch(r'/scan(?:/[A-Za-z0-9_.-]{1,48}){0,3}',p)is not None and all(x not in('.','..')for x in p.split('/')[1:])
def valid_node(n):
    require(isinstance(n,dict)and type(n.get('directory'))is bool,'Typed inert node')
    require(set(n)==({'path','directory','mode'}if n['directory']else{'path','directory','mode','text'}),'Closed node fields')
    require(safe_path(n['path'])and type(n['mode'])is int and 0<=n['mode']<=0o777,'Fixed canonical node/mode')
    if not n['directory']:require(isinstance(n['text'],str)and len(n['text'].encode())<=8192 and '\0'not in n['text'],'Bound inert text')
def validate_fixture(v):
    require(isinstance(v,dict)and set(v)=={'schema_version','cases'}and type(v['schema_version'])is int and v['schema_version']==1,'Closed fixture/version')
    require(isinstance(v['cases'],list)and tuple(c['id']for c in v['cases'])==IDS,'Exact24 reviewed recipes')
    for i,c in enumerate(v['cases']):
        require(set(c)=={'id','classification','category','directory','name','nodes','mutations'},'Closed case fields')
        require(c['classification']==('supported'if i<18 else'unsupported')and c['category']in OPTIONS and safe_path(c['directory'])and c['name']=='atlas','Exact class/category/name')
        require(isinstance(c['nodes'],list)and 4<=len(c['nodes'])<=8,'Small fixed inert node roster')
        known={'/':True,'/work':True}
        for n in c['nodes']:
            valid_node(n);require(n['path']not in known and known.get(str(Path(n['path']).parent))is True,'Unique parent-first node')
            known[n['path']]=n['directory']
        require(isinstance(c['mutations'],list)and len(c['mutations'])<=1,'At most one fixed mutation')
        for m in c['mutations']:
            require(isinstance(m,dict),'Typed mutation')
            if m.get('op')=='add':require(set(m)=={'op','node'}and c['id']=='added-live-manual','Only fixed add');valid_node(m['node']);require(m['node']['path']=='/scan/man/atlas.1'and m['node']['path']not in known,'Known new manual leaf')
            elif m.get('op')=='remove':require(m=={'op':'remove','path':'/scan/bin/atlas'}and c['id']=='removed-live-entry','Only fixed remove')
            elif m.get('op')=='chmod':
                require(set(m)=={'op','path','mode'}and type(m['mode'])is int,'Typed chmod')
                expected={'read-without-own-search':('/scan/bin',0o400),'owner-class-denied':('/scan/bin',0o055),'unreadable-directory':('/scan/bin',0o100),'ancestor-without-search':('/scan',0o400)}
                require(c['id']in expected and(m['path'],m['mode'])==expected[c['id']],'Only fixed private directory mode transitions')
            else:raise ValueError('Unknown mutation')
    return v
def fixture():
    raw=read_bytes(ROOT/FIXTURE,65536);require(digest(raw)==FIXTURE_SHA,'Exact reviewed fixture bytes changed')
    return validate_fixture(strict(raw))
def environment(folder):
    env=closed_environment({},folder);env.pop('PATH',None)
    require(set(env)=={'LC_ALL','LANG','TMPDIR','TMP','TEMP'}and env['LC_ALL']=='C','Closed C-locale disk-temp environment only')
    return env
def binding_names():
    text=read_bytes(ROOT/BINDINGS,256*1024).decode()
    pairs=re.findall(r'\(\s*"([^"\n]+)"\s*,\s*include_bytes!\(\s*"([^"\n]+)"\s*\)\s*,?\s*\)',text);names=[p for p,_ in pairs]
    require(all(not rel.startswith('/')and os.path.normpath('examples/support/'+rel)==p for p,rel in pairs),'Exact included-source identity')
    expected=set(EXTRAS)
    for base in('src','scenarios','assets'):
        for p in(ROOT/base).rglob('*'):
            require(not p.is_symlink(),'No source symlinks')
            if p.is_file():expected.add(p.relative_to(ROOT).as_posix())
            else:require(p.is_dir(),'Source kind')
    require(len(names)==len(set(names))and set(names)==expected and len(names)<=1024,'Complete current three-root closure plus six extras')
    return sorted(names)
def control():
    require(shared.ROOT==ROOT,'Unchanged helper must refer to this MAIN checkout')
    compiled=[{'path':p,**pin(ROOT/p,4*1024*1024)}for p in binding_names()]
    program=Path('/usr/bin/whereis');s=program.stat()
    require(stat.S_ISREG(s.st_mode)and stat.S_IMODE(s.st_mode)==0o755,'Original0755 whereis only, no setuid/setgid/copy')
    probe=ROOT/PROBE
    return {'purpose':'fixed-whereis-reference-v1','root':str(ROOT),'fixture_sha256':FIXTURE_SHA,'compiled_inputs':compiled,'tools':{p:pin(ROOT/p,4*1024*1024)for p in TOOLS},'program':{'path':str(program),**pin(program,16*1024*1024)},'program_mode':{'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid},'probe':{'path':str(probe),**pin(probe,1024*1024*1024)},'restriction':'Inherited child-only NNP/limits/proof; effective query dirs explicit, startup system metadata access acknowledged'}
def retain_inputs(run,c):
    require(shared.ROOT==ROOT,'Held helper source root')
    shared.retain_inputs(run,c)

class Files:
    """Fixed owned inode roster. Retain descriptors before restrictive modes.
    Descriptor-only inspection is parent evidence, never passed to the tool child."""
    def __init__(self,folder,case,stack):
        self.folder=folder;self.rows={};self.stack=stack
        self.rootfd=os.open(folder,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);stack.callback(os.close,self.rootfd)
        self.rootstat=os.fstat(self.rootfd)
        require(folder.resolve()==folder and not folder.is_symlink()and stat.S_ISDIR(self.rootstat.st_mode)and stat.S_IMODE(self.rootstat.st_mode)==0o700 and self.rootstat.st_uid==os.geteuid(),'Canonical owned private0700 root')
        self.add({'path':'/work','directory':True,'mode':0o700})
        for n in case['nodes']:self.add(n)
        # Apply all final supplied directory modes only after children are opened.
        for r in self.rows.values():os.fchmod(r['fd'],r['mode'])
        for m in case['mutations']:
            if m['op']=='chmod':
                r=self.rows[m['path']];os.fchmod(r['fd'],m['mode']);r['mode']=m['mode']
            elif m['op']=='remove':
                r=self.rows[m['path']];require(not r['directory']and r['active'],'Fixed regular leaf removal');os.unlink(Path(m['path']).name,dir_fd=self.parentfd(m['path']));r['active']=False
            else:self.add(m['node']);r=self.rows[m['node']['path']];os.fchmod(r['fd'],r['mode'])
        self.baseline={p:self.state(r['fd'])for p,r in self.rows.items()}
        self.rootbaseline=self.state(self.rootfd)
    def verify_root(self):
        s=self.folder.lstat();q=os.fstat(self.rootfd)
        require(self.folder.resolve()==self.folder and stat.S_ISDIR(s.st_mode)and(s.st_dev,s.st_ino,s.st_mode,s.st_uid,s.st_gid)==(q.st_dev,q.st_ino,q.st_mode,q.st_uid,q.st_gid)==(self.rootstat.st_dev,self.rootstat.st_ino,self.rootstat.st_mode,self.rootstat.st_uid,self.rootstat.st_gid),'Original private root')
    def parentfd(self,p):
        parent=str(Path(p).parent)
        return self.rootfd if parent=='/'else self.rows[parent]['fd']
    def path(self,p):
        self.verify_root();require(p=='/work'or safe_path(p),'Only literal contained fixture namespace')
        return self.folder/p[1:]
    def add(self,n):
        p=n['path'];require(p not in self.rows and(p=='/work'or safe_path(p)),'Exclusive known contained node')
        parent=self.parentfd(p)
        if n['directory']:
            os.mkdir(Path(p).name,0o700,dir_fd=parent);fd=os.open(Path(p).name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=parent);raw=b''
        else:
            raw=n['text'].encode();require(len(raw)<=8192,'Inert leaf bound')
            fd=os.open(Path(p).name,os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=parent)
            require(os.write(fd,raw)==len(raw),'Complete inert write');os.fsync(fd)
        self.stack.callback(os.close,fd)
        self.rows[p]={'fd':fd,'directory':n['directory'],'mode':n['mode'],'bytes':raw,'active':True,'identity':os.fstat(fd)}
    @staticmethod
    def state(fd):
        s=os.fstat(fd)
        return {'dev':s.st_dev,'ino':s.st_ino,'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid,'nlink':s.st_nlink,'mtime_ns':s.st_mtime_ns,'ctime_ns':s.st_ctime_ns}
    def snapshot(self):
        self.verify_root();require(self.state(self.rootfd)==self.rootbaseline,'Private root metadata changed')
        allrows={'/':{'fd':self.rootfd,'directory':True,'active':True}}
        allrows.update(self.rows);result={}
        # Verify every parent's exact live name roster through pre-opened FDs,
        # including read-only/no-search parents, without restoring permissions.
        for p,r in allrows.items():
            if r['directory']:
                expected={Path(k).name for k,v in self.rows.items()if str(Path(k).parent)==p and v['active']}
                os.lseek(r['fd'],0,os.SEEK_SET)
                actual=set(os.listdir(r['fd']));require(actual==expected,f'Unexpected/missing fixed directory entry {p}: expected {sorted(expected)}, actual {sorted(actual)}')
        for p,r in self.rows.items():
            s=os.fstat(r['fd']);o=r['identity'];current=self.state(r['fd']);parentfd=self.parentfd(p);parent=self.state(parentfd)
            require((s.st_dev,s.st_ino)==(o.st_dev,o.st_ino)and s.st_uid==os.geteuid()and stat.S_IMODE(s.st_mode)==r['mode'],'Held inode/owner/mode')
            require((stat.S_ISDIR(s.st_mode)and s.st_nlink>=1)if r['directory']else(stat.S_ISREG(s.st_mode)and s.st_nlink==(1 if r['active']else 0)),'Original type/link count')
            require(current==self.baseline[p],'Known inode changed after fixed preparation')
            try:q=os.stat(Path(p).name,dir_fd=parentfd,follow_symlinks=False)
            except FileNotFoundError:require(not r['active'],'Active fixed leaf disappeared')
            except PermissionError:
                require(not(parent['mode']&0o100),'Unexpected lookup denial on searchable held parent')
                # No lstat access is invented: unchanged parent timestamps/held
                # listing + held inode/link count bind the inaccessible name.
                key=str(Path(p).parent);expected=self.rootbaseline if key=='/'else self.baseline[key]
                require(parent==expected,'Nonsearchable parent changed')
            else:
                require(r['active']and(q.st_dev,q.st_ino,q.st_mode,q.st_nlink)==(s.st_dev,s.st_ino,s.st_mode,s.st_nlink),'Path/heldFD identity')
            raw=b''if r['directory']else os.pread(r['fd'],8193,0)
            require(raw==r['bytes'],'Original inert contents unchanged')
            result[p]={**current,'active':r['active'],'directory':r['directory'],'parent_dev':parent['dev'],'parent_ino':parent['ino'],'parent_mode':parent['mode'],'bytes':len(raw),'sha256':digest(raw)}
        return result

def argv(exe,files,c):
    small,big=OPTIONS[c['category']]
    return [exe,small,big,str(files.path(c['directory'])),'-f',c['name']]
def normalized(actual,files,c):
    require(type(actual.get('status'))is int and actual['status']==0,'Supported normal whereis query status must be observed0, never turn127/signal into missing')
    require(actual['stderr']=='','Unexpected stderr retained, not stripped')
    raw=actual['stdout'];prefix=c['name']+':'
    require(raw.startswith(prefix)and raw.endswith('\n')and raw.count('\n')==1,'Exact one-line name-colon format')
    tail=raw[len(prefix):-1]
    require(tail==''or tail.startswith(' '),'Exact path separator')
    hostpaths=[]if tail==''else tail[1:].split(' ')
    require(all(p.startswith(str(files.folder)+'/')for p in hostpaths),'Only explicit fixture results')
    paths=[p[len(str(files.folder)):]for p in hostpaths]
    require(len(paths)<=8 and len(paths)==len(set(paths))and all(p in files.rows for p in paths),'Only known distinct fixture entries')
    # Keep observed ordering, including unsupported multi-match rows.
    return {'status':actual['status'],'paths':paths,'stdout':prefix+''.join(' '+p for p in paths)+'\n'}
def approve(args,c,raw,mode):
    require(args.approved_control==digest(raw)and args.approved_whereis==c['program']['sha256'],'Independent exact source/control/original ELF approval')
    if mode=='compare':require(args.approved_probe==c['probe']['sha256'],'Exact probe approval')
def observe(args,base,c,raw,lock):
    approve(args,c,raw,'observe');require(os.geteuid()!=0,'Ordinary owner permission observations only')
    v=fixture();run=fresh(base,'observed');retain_inputs(run,c);execs=fresh(run,'executables');budget=Budget();deadline=time.monotonic()+90;rows=[]
    with ExitStack()as stack:
        exe,fd=stack.enter_context(pinned_executable(Path(c['program']['path']),c['program'],16*1024*1024));retain_fd(execs/'whereis',fd,c['program'])
        _,probe_fd=stack.enter_context(pinned_executable(Path(c['probe']['path']),c['probe'],1024*1024*1024));retain_fd(execs/'probe',probe_fd,c['probe'])
        version=fresh(run,'version');env=environment(version);output(version/'launch.json',{'argv':[exe,'--version'],'env':env})
        a=capture_restricted([exe,'--version'],version,version,env,budget,(lock,fd),deadline=deadline);output(version/'actual.json',a)
        require(a['status']==0 and a['stderr']==''and a['stdout'],'Actual bounded version, no inferred installed version')
        for case in v['cases']:
            require(time.monotonic()<deadline,'Whole90s deadline');case_run=fresh(run,case['id'])
            with ExitStack()as local:
                folder=fresh(case_run,'fixture');files=Files(folder,case,local)
                before=files.snapshot();output(case_run/'before.json',before)
                capture=fresh(case_run,'query');command=argv(exe,files,case);env=environment(folder)
                output(capture/'launch.json',{'argv':command,'env':env,'case':case['id'],'classification':case['classification']})
                a=capture_restricted(command,capture,files.path('/work'),env,budget,(lock,fd),deadline=deadline);output(capture/'actual.json',a)
                after=files.snapshot();output(case_run/'after.json',after);require(before==after,'Read-only query changed a known inode/name/mode/body')
                rows.append({'id':case['id'],'classification':case['classification'],'actual':a,'normalized':normalized(a,files,case)})
    require(control()==c,'Source/ELF changed; preserve raw attempt');require(time.monotonic()<=deadline,'Whole90s deadline exceeded; preserve raw attempt')
    output(run/'OBSERVATIONS.json',{'control_sha256':digest(raw),'fixture_sha256':FIXTURE_SHA,'observations':rows,'limits':'24 fixed queries plus one version; 18 supported candidates and6unsupported probes; original whereis startup SYSTEM metadata I/O acknowledged, not a filesystem jail; no learner/utility fixture execution'})
def comparison_rows(host_rows,model_rows):
    require(tuple(r['id']for r in host_rows)==IDS and tuple(r['id']for r in model_rows)==IDS,'Exact ordered comparison rosters')
    checks=[]
    for i,(h,m)in enumerate(zip(host_rows,model_rows)):
        classification='supported'if i<18 else'unsupported'
        require(h['classification']==m['classification']==classification,'Exact expected case classification')
        if classification=='supported':
            expected={k:m[k]for k in('status','paths','stdout')}if all(k in m for k in('status','paths','stdout'))else m
            checks.append({'id':h['id'],'classification':classification,'host':h['normalized'],'model':expected,'exact_match':h['normalized']==expected})
        else:
            checks.append({'id':h['id'],'classification':classification,'host':h['normalized'],'model':m,'exact_match':None,'unsupported_honest':m.get('error_kind')=='Unsupported'})
    return checks
def compare(args,base,c,raw,lock):
    approve(args,c,raw,'compare');require(os.geteuid()!=0,'Ordinary fixed model probe');deadline=time.monotonic()+90
    original_raw=read_bytes(base/'observed/OBSERVATIONS.json',CAP);require(args.approved_observations==digest(original_raw),'Independent exact actual-observation readback approval')
    host=strict(original_raw);require(host['control_sha256']==digest(raw)and host['fixture_sha256']==FIXTURE_SHA and tuple(r['id']for r in host['observations'])==IDS,'Exact actual authority')
    run=fresh(base,'compared');retain_inputs(run,c);budget=Budget()
    with pinned_executable(Path(c['probe']['path']),c['probe'],1024*1024*1024)as(exe,fd):
        retain_fd(run/'retained-probe',fd,c['probe']);a=capture_restricted([exe],run,run,environment(run),budget,(lock,fd),deadline=deadline);output(run/'PROBE.json',a)
    require(a['status']==0 and a['stderr']=='','Actual fixed model probe failed')
    model=strict(a['stdout'].encode())
    require(model['purpose']=='fixed-whereis-reference-v1'and model['compiled_manifest_dir']==str(ROOT)and model['fixture_sha256']==FIXTURE_SHA and model['compiled_inputs_count']==len(c['compiled_inputs'])and model['compiled_inputs_sha256']==binding_commitment(c['compiled_inputs']),'Actual compiled root/source/fixture commitment')
    checks=comparison_rows(host['observations'],model['observations'])
    require(control()==c,'Source/ELF changed during comparison');require(time.monotonic()<=deadline,'Whole90s comparison deadline exceeded')
    output(run/'COMPARISON.json',{'control_sha256':digest(raw),'observations_sha256':digest(original_raw),'checks':checks,'supported_exact_matches':sum(r['exact_match']is True for r in checks),'unsupported_honest':sum(r.get('unsupported_honest')is True for r in checks),'limits':'Unsupported rows do NOT count as utility matches; no sorted ordering, default/personal searches, general suffix/ELF/permission or curriculum acceptance'})
    require(all(r['exact_match']is True if r['classification']=='supported'else r['unsupported_honest']is True for r in checks),'Honest observed mismatch retained')
def main():
    p=argparse.ArgumentParser();p.add_argument('--mode',choices=('prepare','observe','compare'),required=True)
    for name in('control','whereis','probe','observations'):p.add_argument('--approved-'+name)
    args=p.parse_args();lock=inherited_lock(ROOT)
    require((ROOT/'.git').is_dir()and not(ROOT/'.git').is_symlink()and Path(os.environ.get('CARGO_TARGET_DIR',''))==ROOT/'target','MAIN/default-target runner only')
    fixture();base=ROOT/BASE
    if args.mode=='prepare':
        require(not base.exists()and not base.is_symlink()and base.parent.is_dir()and base.parent.resolve()==base.parent,'Fresh contained prepared attempt')
        c=control();base.mkdir(mode=0o700);output(base/'CONTROL.json',c);print('PROPOSED only; no child launched');return
    require(base.is_dir()and base.resolve()==base and not base.is_symlink(),'Canonical prepared attempt')
    raw=read_bytes(base/'CONTROL.json',1024*1024);c=strict(raw);require(c==control(),'Prepared exact source/ELF changed')
    if args.mode=='observe':observe(args,base,c,raw,lock)
    else:compare(args,base,c,raw,lock)
if __name__=='__main__':main()
