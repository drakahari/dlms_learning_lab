#!/usr/bin/env python3
"""Fixed owned-directory GNU chmod developer fixture; no learner input."""
import argparse
from contextlib import ExitStack
import hashlib
import os
from pathlib import Path
import re
import stat
import time
import file_index_reference as shared
from file_index_reference import capture_restricted,binding_commitment,retain_fd
from automation_before10_oracle import pinned_executable
from case_bash_oracle import closed_environment
from native_reopen import inherited_lock,pin,read_bytes
from startup_reference import Budget,fresh,output,strict
ROOT=Path(__file__).resolve().parents[1]
BASE='.build-artifacts/reference-runs/sticky-mode-r1'
FIXTURE='tests/fixtures/sticky-mode-reference-v1.json'
FIXTURE_SHA='7cc4742083f3948abbc010b16467300aa587795756804a2e936432d3b7fb0977'
PROBE='target/debug/examples/sticky_mode_reference'
BINDINGS='examples/support/sticky_mode_bindings.rs'
EXTRAS=('Cargo.toml','Cargo.lock','examples/sticky_mode_reference.rs',BINDINGS,'examples/support/sticky_mode_digest.rs',FIXTURE)
TOOLS=('tools/sticky_mode_reference.py','tools/generate_sticky_mode_bindings.py','tools/tests/test_sticky_mode_reference.py','tools/file_index_reference.py','tools/dev.py','tools/build_safety.py','tools/native_reopen.py','tools/automation_before10_oracle.py','tools/case_bash_oracle.py','tools/startup_reference.py')
SPECS=(('numeric-clear','0777'),('numeric-sticky','01777'),('add-sticky','+t'),('remove-sticky','-t'),('equals-all','a=rwx'),('equals-other','o=rx'),('equals-empty-other','o='),('equals-user','u=rw'),('equals-group','g=rx'),('equals-ugo','ugo=rw'),('mixed-user-group','u=rw,g=rx'),('mixed-other-middle','u=rw,o=rx,g=rx'),('mixed-other-first','o=,u=rwx,g=rw'),('mixed-all-adjust','a=rw,u+x,o-r'),('mixed-plus-minus','u+x,g-w,o+r'),('excluded-o-plus-t','o+t'),('excluded-setuid','4755'))
IDS=tuple(p[0]for p in SPECS)
CAP=4*1024*1024

def require(ok,message):
    if not ok:raise ValueError(message)
def digest(raw):return hashlib.sha256(raw).hexdigest()
def fixture():
    raw=read_bytes(ROOT/FIXTURE,65536);require(digest(raw)==FIXTURE_SHA,'Exact reviewed fixture changed');return validate_fixture(strict(raw))
def validate_fixture(v):
    require(isinstance(v,dict)and set(v)=={'schema_version','initial_mode','cases'}and type(v['schema_version'])is int and v['schema_version']==1 and type(v['initial_mode'])is int and v['initial_mode']==0o1770,'Closed version and initial mode')
    require(isinstance(v['cases'],list)and len(v['cases'])==17,'Exactly seventeen fixed cases')
    for i,(q,expected)in enumerate(zip(v['cases'],SPECS)):
        require(isinstance(q,dict)and set(q)=={'id','classification','spec'}and(q['id'],q['spec'])==expected and q['classification']==('supported'if i<15 else'excluded'),'Fixed identity/spec/category; no path/source/script selector')
    return v
def environment(folder):
    env=closed_environment({},folder);env.pop('PATH',None)
    require(set(env)=={'LC_ALL','LANG','TMPDIR','TMP','TEMP'}and env['LC_ALL']=='C','Closed C and private disk TEMP only')
    return env

def binding_names():
    text=read_bytes(ROOT/BINDINGS,256*1024).decode()
    pairs=re.findall(r'\(\s*"([^"\n]+)"\s*,\s*include_bytes!\(\s*"([^"\n]+)"\s*\)\s*,?\s*\)',text);names=[p for p,_ in pairs]
    require(all(not rel.startswith('/')and os.path.normpath('examples/support/'+rel)==p for p,rel in pairs),'Exact include path identity')
    expected=set(EXTRAS)
    for base in('src','scenarios','assets'):
        for p in(ROOT/base).rglob('*'):
            require(not p.is_symlink(),'No source aliases')
            if p.is_file():expected.add(p.relative_to(ROOT).as_posix())
            else:require(p.is_dir(),'Source kind')
    require(len(names)==len(set(names))and set(names)==expected and len(names)<=1024,'Complete source closure plus six extras')
    return sorted(names)
def control():
    require(shared.ROOT==ROOT,'Unchanged child capture helper bound to MAIN')
    program=Path('/usr/bin/chmod');s=program.stat()
    require(not program.is_symlink()and stat.S_ISREG(s.st_mode)and stat.S_IMODE(s.st_mode)==0o755,'Original ordinary0755 chmod only')
    return {'purpose':'fixed-sticky-mode-reference-v1','root':str(ROOT),'fixture_sha256':FIXTURE_SHA,
      'compiled_inputs':[{'path':p,**pin(ROOT/p,4*1024*1024)}for p in binding_names()],
      'tools':{p:pin(ROOT/p,4*1024*1024)for p in TOOLS},
      'program':{'path':str(program),**pin(program,16*1024*1024)},'program_mode':{'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid},
      'probe':{'path':str(ROOT/PROBE),**pin(ROOT/PROBE,1024*1024*1024)},
      'scope':'Owned empty directory mode operators only; ordinary actor, child NNP proof and limits. No cross-owner deletion, general chmod, ls-format, curriculum or platform acceptance'}
def retain_inputs(run,c):
    require(shared.ROOT==ROOT,'Exact source retention root');shared.retain_inputs(run,c)

class Subject:
    """Fresh owned empty directory per case. Only mode/ctime may change."""
    @staticmethod
    def state(fd):
        s=os.fstat(fd)
        return {'dev':s.st_dev,'ino':s.st_ino,'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid,'nlink':s.st_nlink,'size':s.st_size,'mtime_ns':s.st_mtime_ns,'ctime_ns':s.st_ctime_ns}
    def __init__(self,folder,stack):
        self.folder=folder;self.path=folder/'subject'
        self.parent=os.open(folder,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);stack.callback(os.close,self.parent)
        s=os.fstat(self.parent)
        require(folder.resolve()==folder and not folder.is_symlink()and stat.S_ISDIR(s.st_mode)and s.st_uid==os.geteuid()and stat.S_IMODE(s.st_mode)==0o700,'Canonical owned private0700 fixture root')
        os.mkdir('subject',0o700,dir_fd=self.parent)
        self.fd=os.open('subject',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=self.parent);stack.callback(os.close,self.fd)
        os.fchmod(self.fd,0o1770)
        self.initial_parent=self.state(self.parent);self.initial_subject=self.state(self.fd)
        require(self.initial_subject['mode']==0o1770,'Exact supplied initial1770 mode')
    def snapshot(self):
        rootstat=self.folder.lstat();s=os.fstat(self.parent)
        require(self.folder.resolve()==self.folder and stat.S_ISDIR(rootstat.st_mode)and(rootstat.st_dev,rootstat.st_ino,rootstat.st_mode,rootstat.st_uid,rootstat.st_gid)==(s.st_dev,s.st_ino,s.st_mode,s.st_uid,s.st_gid),'Held fixture parent/path identity')
        parent=self.state(self.parent);subject=self.state(self.fd)
        require(parent==self.initial_parent,'Fixture parent changed')
        unchanged=lambda r:{k:v for k,v in r.items()if k not in('mode','ctime_ns')}
        require(unchanged(subject)==unchanged(self.initial_subject),'Subject ownership/identity/link/size/mtime changed')
        a=os.fstat(self.fd);b=os.stat('subject',dir_fd=self.parent,follow_symlinks=False)
        require(stat.S_ISDIR(a.st_mode)and stat.S_ISDIR(b.st_mode)and(a.st_dev,a.st_ino,a.st_mode,a.st_uid,a.st_gid,a.st_nlink)==(b.st_dev,b.st_ino,b.st_mode,b.st_uid,b.st_gid,b.st_nlink)and a.st_uid==os.geteuid()and a.st_nlink>=1,'Exact owned held directory; no directory link-count assumption')
        require(type(subject['mode'])is int and 0<=subject['mode']<=0o7777,'Bounded actual mode')
        os.lseek(self.parent,0,os.SEEK_SET);require(set(os.listdir(self.parent))=={'subject'},'Only known subject entry')
        os.lseek(self.fd,0,os.SEEK_SET);require(os.listdir(self.fd)==[],'Subject stays empty')
        return {'parent':parent,'subject':subject}

def argv(exe,subject,q):
    require(q['id']in IDS and q['spec']==dict(SPECS)[q['id']],'Exact reviewed operator only')
    return [exe,'--',q['spec'],str(subject.path)]
def approve(args,c,raw,mode):
    require(args.approved_control==digest(raw)and args.approved_chmod==c['program']['sha256'],'Exact independent source/control/original chmod approval')
    if mode=='compare':require(args.approved_probe==c['probe']['sha256'],'Exact compiled probe approval')
def observe(args,base,c,raw,lock):
    approve(args,c,raw,'observe');require(os.geteuid()!=0,'Ordinary owned disposable fixtures only')
    v=fixture();run=fresh(base,'observed');retain_inputs(run,c);execs=fresh(run,'executables');budget=Budget();deadline=time.monotonic()+90;rows=[]
    with ExitStack()as stack:
        exe,fd=stack.enter_context(pinned_executable(Path(c['program']['path']),c['program'],16*1024*1024));retain_fd(execs/'chmod',fd,c['program'])
        _,probe=stack.enter_context(pinned_executable(Path(c['probe']['path']),c['probe'],1024*1024*1024));retain_fd(execs/'probe',probe,c['probe'])
        version=fresh(run,'version');env=environment(version);output(version/'launch.json',{'argv':[exe,'--version'],'env':env,'cwd':str(version)})
        a=capture_restricted([exe,'--version'],version,version,env,budget,(lock,fd),deadline=deadline);output(version/'actual.json',a)
        require(a['status']==0 and a['stderr']==''and a['stdout'],'Actual version required')
        for q in v['cases']:
            folder=fresh(run,q['id'])
            with ExitStack()as local:
                subject=Subject(fresh(folder,'fixture'),local);before=subject.snapshot();output(folder/'before.json',before)
                command=argv(exe,subject,q);env=environment(folder);output(folder/'launch.json',{'argv':command,'env':env,'cwd':str(folder),'case':q})
                a=capture_restricted(command,folder,folder,env,budget,(lock,fd),deadline=deadline);output(folder/'actual.json',a)
                after=subject.snapshot();output(folder/'after.json',after)
                rows.append({'id':q['id'],'classification':q['classification'],'actual':a,'initial_mode':before['subject']['mode'],'mode':after['subject']['mode']})
        require(control()==c,'Source or original ELF changed during observation')
    require(time.monotonic()<=deadline,'Whole90s deadline')
    output(run/'OBSERVATIONS.json',{'control_sha256':digest(raw),'fixture_sha256':FIXTURE_SHA,'observations':rows,'limits':'Seventeen fresh owned directories/eighteen fixed host launches. Only mode operators; no cross-owner removal/rename or full GNU chmod acceptance. Exclusions are retained, not matches.'})
def comparison_rows(host,model):
    require(isinstance(host,list)and isinstance(model,list)and tuple(r['id']for r in host)==IDS and tuple(r['id']for r in model)==IDS,'Exact ordered seventeen rows')
    result=[]
    for i,(h,m)in enumerate(zip(host,model)):
        category='supported'if i<15 else'excluded';require(h['classification']==m['classification']==category,'Exact supported/excluded classification')
        a=h['actual'];require(type(a['status'])is int and isinstance(a['stdout'],str)and isinstance(a['stderr'],str),'Retained actual status and streams')
        require(type(h['initial_mode'])is int and type(h['mode'])is int and 0<=h['mode']<=0o7777 and h['initial_mode']==0o1770,'Actual bounded initial/after mode')
        if i<15:
            equal=(a['status']==0 and a['stdout']==''and a['stderr']==''and m.get('ok')is True and m.get('stdout')==''and type(m.get('mode'))is int and m['mode']==h['mode']and type(m.get('initial_mode'))is int and m['initial_mode']==h['initial_mode'])
            result.append({'id':h['id'],'classification':category,'host_actual':a,'host_initial_mode':h['initial_mode'],'host_mode':h['mode'],'model':m,'exact_mode_and_success':equal})
        else:result.append({'id':h['id'],'classification':category,'host_actual':a,'host_mode':h['mode'],'model':m,'exact_mode_and_success':None,'honest_exclusion':m.get('ok')is False and m.get('honest_exclusion')is True and isinstance(m.get('error'),str)and bool(m['error'])})
    return result

def compare(args,base,c,raw,lock):
    approve(args,c,raw,'compare');require(os.geteuid()!=0,'Ordinary fixed in-process model probe only');deadline=time.monotonic()+90
    observed=read_bytes(base/'observed/OBSERVATIONS.json',CAP);require(args.approved_observations==digest(observed),'Independent actual observation/raw-evidence readback required')
    host=strict(observed);require(host['control_sha256']==digest(raw)and host['fixture_sha256']==FIXTURE_SHA,'Exact observed authority')
    run=fresh(base,'compared');retain_inputs(run,c);budget=Budget()
    with pinned_executable(Path(c['probe']['path']),c['probe'],1024*1024*1024)as(exe,fd):
        retain_fd(run/'retained-probe',fd,c['probe']);a=capture_restricted([exe],run,run,environment(run),budget,(lock,fd),deadline=deadline);output(run/'PROBE.json',a)
    require(a['status']==0 and a['stderr']=='','Actual fixed probe failed');model=strict(a['stdout'].encode())
    require(model['purpose']=='fixed-sticky-mode-reference-v1'and model['compiled_manifest_dir']==str(ROOT)and model['fixture_sha256']==FIXTURE_SHA and model['compiled_inputs_count']==len(c['compiled_inputs'])and model['compiled_inputs_sha256']==binding_commitment(c['compiled_inputs']),'Actual complete source/root/fixture commitment')
    rows=comparison_rows(host['observations'],model['observations']);require(control()==c,'Source/ELF changed during compare');require(time.monotonic()<=deadline,'Whole90s deadline')
    output(run/'COMPARISON.json',{'control_sha256':digest(raw),'observations_sha256':digest(observed),'checks':rows,'supported_matches':sum(r['exact_mode_and_success']is True for r in rows),'honest_exclusions':sum(r.get('honest_exclusion')is True for r in rows),'limits':'Mode and successful quiet result only; no cross-user/directory deletion, content, ls-format, generalchmod or curriculum/platform acceptance. Exclusions are not matches.'})
    require(all(r['exact_mode_and_success']is True if r['classification']=='supported'else r['honest_exclusion']is True for r in rows),'Actual finite mismatch retained')
def main():
    p=argparse.ArgumentParser();p.add_argument('--mode',choices=('prepare','observe','compare'),required=True)
    for name in('control','chmod','probe','observations'):p.add_argument('--approved-'+name)
    args=p.parse_args();lock=inherited_lock(ROOT);require((ROOT/'.git').is_dir()and not(ROOT/'.git').is_symlink()and Path(os.environ.get('CARGO_TARGET_DIR',''))==ROOT/'target','MAIN/default-target only')
    fixture();base=ROOT/BASE
    if args.mode=='prepare':
        require(not base.exists()and not base.is_symlink()and base.parent.is_dir()and base.parent.resolve()==base.parent,'Fresh contained preparation')
        c=control();base.mkdir(mode=0o700);output(base/'CONTROL.json',c);print('PROPOSED only; no child launched');return
    require(base.is_dir()and base.resolve()==base and not base.is_symlink(),'Canonical prepared attempt')
    raw=read_bytes(base/'CONTROL.json',1024*1024);c=strict(raw);require(c==control(),'Exact prepared source/ELF changed')
    if args.mode=='observe':observe(args,base,c,raw,lock)
    else:compare(args,base,c,raw,lock)
if __name__=='__main__':main()
