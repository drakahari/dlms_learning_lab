#!/usr/bin/env python3
"""Fixed readonly GNU ls -i developer fixture. Never accepts learner input."""
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
from file_index_reference import capture_restricted,binding_commitment,retain_fd
from automation_before10_oracle import pinned_executable
from case_bash_oracle import closed_environment
from native_reopen import inherited_lock,pin,read_bytes
from startup_reference import Budget,fresh,output,strict
ROOT=Path(__file__).resolve().parents[1]
BASE='.build-artifacts/reference-runs/links-inode-r1'
FIXTURE='tests/fixtures/links-inode-reference-v1.json'
FIXTURE_SHA='b634eccdaec473f38b2b533a6092c80c4cbdb16ed18a5973f3d1eebe7f955f59'
PROBE='target/debug/examples/links_inode_reference'
BINDINGS='examples/support/links_inode_bindings.rs'
EXTRAS=('Cargo.toml','Cargo.lock','examples/links_inode_reference.rs',BINDINGS,'examples/support/links_inode_digest.rs',FIXTURE)
TOOLS=('tools/links_inode_reference.py','tools/generate_links_inode_bindings.py','tools/tests/test_links_inode_reference.py','tools/file_index_reference.py','tools/dev.py','tools/build_safety.py','tools/native_reopen.py','tools/automation_before10_oracle.py','tools/case_bash_oracle.py','tools/startup_reference.py')
QUERY_ARGS={
 'default-work':['-i'],'directory-symlink':['-i','/out/portal'],
 'hidden-only':['-i','/hidden-only'],'empty':['-i','/empty'],
 'literal-directory':['-i','--','-dir room'],'link-directory':['-i','/out'],
 'excluded-file':['-i','/work/original'],'excluded-all':['-i','-a','/work']}
IDS=tuple(QUERY_ARGS)
DIRECTORIES=('/','/work','/out','/empty','/hidden-only','/work/-dir room')
ROSTER=(('regular','/work/original'),('hard','/work/alias'),('copy','/work/copy'),('symbolic','/work/shortcut'),('symbolic','/work/dangling'),('regular','/work/.hidden'),('regular','/hidden-only/.only'),('symbolic','/out/portal'),('hard','/out/pin'),('regular','/work/-dir room/-name with space'))
CAP=4*1024*1024
def require(ok,message):
    if not ok:raise ValueError(message)
def digest(raw):return hashlib.sha256(raw).hexdigest()
def fixture():
    raw=read_bytes(ROOT/FIXTURE,65536);require(digest(raw)==FIXTURE_SHA,'Exact reviewed fixture changed');return validate_fixture(strict(raw))
def validate_fixture(v):
    require(isinstance(v,dict)and set(v)=={'schema_version','directories','entries','queries'}and type(v['schema_version'])is int and v['schema_version']==1,'Closed versioned fixture')
    require(tuple(v['directories'])==DIRECTORIES and tuple((e['kind'],e['path'])for e in v['entries'])==ROSTER,'Fixed world roster')
    known=set(DIRECTORIES);regular=set()
    for e in v['entries']:
        kind=e['kind'];p=e['path'];require(p not in known and str(Path(p).parent)in DIRECTORIES,'New fixed leaf under known real directory')
        if kind=='regular':
            require(set(e)=={'kind','path','text','mode'}and isinstance(e['text'],str)and len(e['text'].encode())<=8192 and '\0'not in e['text']and type(e['mode'])is int and e['mode']in(0,0o600),'Inert ordinary file')
            regular.add(p)
        elif kind in('hard','copy'):
            require(set(e)=={'kind','path','from'}and e['from']=='/work/original'and e['from']in regular,'Known regular source only');regular.add(p)
        else:
            targets={'/work/shortcut':'original','/work/dangling':'missing','/out/portal':'../work'}
            require(kind=='symbolic'and set(e)=={'kind','path','target'}and targets.get(p)==e['target'],'Exact internal symbolic target; no loops/absolute/escape')
        known.add(p)
    require(tuple(q['id']for q in v['queries'])==IDS,'Exact ordered eight queries')
    for i,q in enumerate(v['queries']):
        require(set(q)=={'id','classification','args'}and q['classification']==('supported'if i<6 else'excluded')and q['args']==QUERY_ARGS[q['id']],'Fixed literal argv, no scripts/options/paths selected by caller')
    return v
def environment(folder):
    env=closed_environment({},folder);env.pop('PATH',None)
    require(set(env)=={'LC_ALL','LANG','TMPDIR','TMP','TEMP'}and env['LC_ALL']=='C','Closed C/disk TEMP only')
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
    require(len(names)==len(set(names))and set(names)==expected and len(names)<=1024,'Complete three-root source closure plus six extras')
    return sorted(names)
def control():
    require(shared.ROOT==ROOT,'Unchanged capture helper bound to MAIN')
    program=Path('/usr/bin/ls');s=program.stat()
    require(stat.S_ISREG(s.st_mode)and stat.S_IMODE(s.st_mode)==0o755,'Original0755 ls only')
    return {'purpose':'fixed-links-inode-reference-v1','root':str(ROOT),'fixture_sha256':FIXTURE_SHA,
      'compiled_inputs':[{'path':p,**pin(ROOT/p,4*1024*1024)}for p in binding_names()],
      'tools':{p:pin(ROOT/p,4*1024*1024)for p in TOOLS},
      'program':{'path':str(program),**pin(program,16*1024*1024)},
      'program_mode':{'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid},
      'probe':{'path':str(ROOT/PROBE),**pin(ROOT/PROBE,1024*1024*1024)},
      'scope':'Fixed readonly directory queries; child NNP proof/limits; not a filesystem jail or full GNU ls'}
def retain_inputs(run,c):
    require(shared.ROOT==ROOT,'Exact retained-source helper root');shared.retain_inputs(run,c)

class Files:
    """One fixed owned world; fixture FDs are parent evidence, never child pass_fds."""
    def __init__(self,folder,v,stack):
        self.folder=folder;self.rows={};self.stack=stack;self.allowed=set(v['directories'])|{e['path']for e in v['entries']}
        fd=os.open(folder,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW);stack.callback(os.close,fd)
        self.rootstat=os.fstat(fd);require(folder.resolve()==folder and not folder.is_symlink()and stat.S_IMODE(self.rootstat.st_mode)==0o700 and self.rootstat.st_uid==os.geteuid(),'Canonical owned0700 fixture root')
        self.rows['/']={'fd':fd,'kind':'directory','identity':self.rootstat,'mode':0o700}
        for p in v['directories'][1:]:
            parent=self.parentfd(p);os.mkdir(Path(p).name,0o700,dir_fd=parent)
            f=os.open(Path(p).name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=parent);stack.callback(os.close,f)
            self.rows[p]={'fd':f,'kind':'directory','identity':os.fstat(f),'mode':0o700}
        for e in v['entries']:
            p=e['path'];parent=self.parentfd(p);kind=e['kind']
            if kind=='regular':self.regular(p,e['text'].encode(),e['mode'])
            elif kind=='hard':
                source=self.rows[e['from']];require(source['kind']=='regular','No hard symbolic/directory links')
                os.link(Path(e['from']).name,Path(p).name,src_dir_fd=self.parentfd(e['from']),dst_dir_fd=parent,follow_symlinks=False)
                f=os.open(Path(p).name,os.O_RDWR|os.O_NOFOLLOW,dir_fd=parent);stack.callback(os.close,f)
                s=os.fstat(f);o=os.fstat(source['fd']);require((s.st_dev,s.st_ino)==(o.st_dev,o.st_ino),'Actual shared identity')
                self.rows[p]={'fd':f,'kind':'regular','identity':s,'mode':source['mode'],'bytes':source['bytes']}
            elif kind=='copy':
                source=self.rows[e['from']];raw=os.pread(source['fd'],8193,0);require(raw==source['bytes'],'Known held source bytes')
                self.regular(p,raw,source['mode'])
                a=os.fstat(source['fd']);b=os.fstat(self.rows[p]['fd']);require((a.st_dev,a.st_ino)!=(b.st_dev,b.st_ino),'Copy is independent')
            else:
                os.symlink(e['target'],Path(p).name,dir_fd=parent)
                f=os.open(Path(p).name,os.O_PATH|os.O_NOFOLLOW,dir_fd=parent);stack.callback(os.close,f)
                require(stat.S_ISLNK(os.fstat(f).st_mode),'Held final symbolic inode')
                self.rows[p]={'fd':f,'kind':'symbolic','identity':os.fstat(f),'target':e['target'],'mode':0o777}
        # Open every known alias/copy FD before applying0mode, never restore it.
        for r in self.rows.values():
            if r['kind']=='regular':os.fchmod(r['fd'],r['mode'])
        self.baseline={p:self.state(r['fd'])for p,r in self.rows.items()}
    @staticmethod
    def state(fd):
        s=os.fstat(fd)
        return {'dev':s.st_dev,'ino':s.st_ino,'mode':stat.S_IMODE(s.st_mode),'uid':s.st_uid,'gid':s.st_gid,'nlink':s.st_nlink,'size':s.st_size,'mtime_ns':s.st_mtime_ns,'ctime_ns':s.st_ctime_ns}
    def verify_root(self):
        s=self.folder.lstat();r=os.fstat(self.rows['/']['fd']);o=self.rootstat
        require(self.folder.resolve()==self.folder and stat.S_ISDIR(s.st_mode)and(s.st_dev,s.st_ino,s.st_mode,s.st_uid,s.st_gid)==(r.st_dev,r.st_ino,r.st_mode,r.st_uid,r.st_gid)==(o.st_dev,o.st_ino,o.st_mode,o.st_uid,o.st_gid),'Original fixture root identity')
    def parentfd(self,p):
        r=self.rows[str(Path(p).parent)];s=os.fstat(r['fd']);o=r['identity']
        require(r['kind']=='directory'and stat.S_ISDIR(s.st_mode)and(s.st_dev,s.st_ino)==(o.st_dev,o.st_ino)and s.st_uid==os.geteuid(),'Held original real parent')
        return r['fd']
    def path(self,p):
        self.verify_root();require(p in self.allowed,'Fixed contained fixture path only')
        return self.folder if p=='/'else self.folder/p[1:]
    def regular(self,p,raw,mode):
        require(p not in self.rows and len(raw)<=8192,'Exclusive bounded inert file')
        f=os.open(Path(p).name,os.O_RDWR|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=self.parentfd(p));self.stack.callback(os.close,f)
        require(os.write(f,raw)==len(raw),'Complete fixed write');os.fsync(f)
        self.rows[p]={'fd':f,'kind':'regular','identity':os.fstat(f),'mode':mode,'bytes':raw}
    def snapshot(self):
        self.verify_root();result={};counts={}
        for r in self.rows.values():
            s=os.fstat(r['fd'])
            if r['kind']=='regular':counts[(s.st_dev,s.st_ino)]=counts.get((s.st_dev,s.st_ino),0)+1
        for p,r in self.rows.items():
            s=os.fstat(r['fd']);o=r['identity'];now=self.state(r['fd'])
            require((s.st_dev,s.st_ino)==(o.st_dev,o.st_ino)and s.st_uid==os.geteuid()and s.st_dev==self.rootstat.st_dev and now==self.baseline[p],'Held one-device original inode unchanged')
            if r['kind']=='directory':
                require(stat.S_ISDIR(s.st_mode)and s.st_nlink>=1,'Directory type/link count')
                os.lseek(r['fd'],0,os.SEEK_SET)
                actual=set(os.listdir(r['fd']));expected={Path(k).name for k in self.rows if k!='/'and str(Path(k).parent)==p}
                require(actual==expected,'Exact known directory roster')
                body=b''
            elif r['kind']=='regular':
                require(stat.S_ISREG(s.st_mode)and s.st_nlink==counts[(s.st_dev,s.st_ino)]and stat.S_IMODE(s.st_mode)==r['mode'],'Regular identity/link/mode')
                body=os.pread(r['fd'],8193,0);require(body==r['bytes'],'Held original body unchanged')
            else:
                require(stat.S_ISLNK(s.st_mode)and s.st_nlink==1,'Only one symbolic name')
                target=os.readlink(Path(p).name,dir_fd=self.parentfd(p));require(target==r['target'],'Original exact internal symbolic text')
                body=target.encode()
            row={**now,'kind':r['kind'],'bytes':len(body),'sha256':digest(body)}
            if p!='/':
                parent=self.parentfd(p);q=os.stat(Path(p).name,dir_fd=parent,follow_symlinks=False);ps=os.fstat(parent)
                require((q.st_dev,q.st_ino,q.st_mode,q.st_nlink)==(s.st_dev,s.st_ino,s.st_mode,s.st_nlink),'Path/heldFD nofollow identity')
                row.update(parent_dev=ps.st_dev,parent_ino=ps.st_ino)
            if r['kind']=='symbolic':row['target']=r['target']
            result[p]=row
        return result

def argv(exe,files,q):
    return [exe]+[str(files.path(a))if a.startswith('/')else a for a in q['args']]
def listing(actual,labels):
    """Keep raw inode tokens/padding. Normalize relations, not actual numeric IDs."""
    if type(actual.get('status'))is not int or actual['status']!=0 or actual.get('stderr')!='':
        return {'normalized':{'error':'Nonzero/malformed status or diagnostic, never an empty successful listing'},'parsed':[]}
    raw=actual.get('stdout');parsed=[]
    if not isinstance(raw,str)or(raw and not raw.endswith('\n')):
        return {'normalized':{'error':'Incomplete listing line'},'parsed':[]}
    for line in raw.splitlines(keepends=True):
        m=re.fullmatch(r'( *)([0-9]+) ([^\r\n]*)\n',line)
        if not m:return {'normalized':{'error':'Unexpected prefix/separator/line format'},'parsed':[]}
        if m[2]=='0'or(len(m[2])>1 and m[2].startswith('0')):return {'normalized':{'error':'Noncanonical positive inode token'},'parsed':[]}
        name=m[3]
        if not name or name.startswith('.'):
            return {'normalized':{'error':'Hidden/empty name in default inode view'},'parsed':[]}
        parsed.append({'name':name,'inode_raw':m[2],'padding':len(m[1]),'digits':len(m[2])})
    if len(parsed)>128 or len({r['name']for r in parsed})!=len(parsed):
        return {'normalized':{'error':'Duplicate/oversized names'},'parsed':parsed}
    width=max((r['digits']for r in parsed),default=0);local=dict(labels);entries=[];columns=[]
    for r in parsed:
        if r['inode_raw']not in local:local[r['inode_raw']]='inode-'+str(len(local)+1)
        entries.append({'name':r['name'],'identity':local[r['inode_raw']]})
        columns.append(r['padding']+r['digits']-width)
    labels.clear();labels.update(local)
    return {'normalized':{'status':0,'entries':entries,'relative_columns':columns},'parsed':parsed}
QUERY_DIRECTORIES={'default-work':'/work','directory-symlink':'/work','hidden-only':'/hidden-only','empty':'/empty','literal-directory':'/work/-dir room','link-directory':'/out'}
def bind_listing_to_inodes(view,metadata,q):
    require('error'not in view['normalized'],'Unexpected raw directory format retained; no comparison claim')
    directory=QUERY_DIRECTORIES[q['id']]
    expected={Path(p).name:p for p in metadata if p!='/'and str(Path(p).parent)==directory and not Path(p).name.startswith('.')}
    require({r['name']for r in view['parsed']}==set(expected),'Actual visible name roster differs from supplied directory')
    for r in view['parsed']:
        inode=metadata[expected[r['name']]]['ino']
        require(type(inode)is int and inode>0 and str(inode)==r['inode_raw'],'Raw emitted inode must equal its independently captured own entry inode')
def approve(args,c,raw,mode):
    require(args.approved_control==digest(raw)and args.approved_ls==c['program']['sha256'],'Exact independently reviewed control/source/original ELF')
    if mode=='compare':require(args.approved_probe==c['probe']['sha256'],'Exact probe identity approval')
def observe(args,base,c,raw,lock):
    approve(args,c,raw,'observe');require(os.geteuid()!=0,'Ordinary synthetic-owner fixtures only')
    v=fixture();run=fresh(base,'observed');retain_inputs(run,c);execs=fresh(run,'executables');budget=Budget();deadline=time.monotonic()+90;observations=[];labels={}
    with ExitStack()as stack:
        exe,fd=stack.enter_context(pinned_executable(Path(c['program']['path']),c['program'],16*1024*1024));retain_fd(execs/'ls',fd,c['program'])
        _,probe=stack.enter_context(pinned_executable(Path(c['probe']['path']),c['probe'],1024*1024*1024));retain_fd(execs/'probe',probe,c['probe'])
        version=fresh(run,'version');env=environment(version);output(version/'launch.json',{'argv':[exe,'--version'],'env':env})
        a=capture_restricted([exe,'--version'],version,version,env,budget,(lock,fd),deadline=deadline);output(version/'actual.json',a)
        require(a['status']==0 and a['stderr']==''and a['stdout'],'Actual bounded version required')
        files=Files(fresh(run,'fixture'),v,stack);output(run/'initial.json',files.snapshot())
        for q in v['queries']:
            folder=fresh(run,q['id']);before=files.snapshot();output(folder/'before.json',before)
            command=argv(exe,files,q);env=environment(files.folder);output(folder/'launch.json',{'argv':command,'env':env,'query':q})
            a=capture_restricted(command,folder,files.path('/work'),env,budget,(lock,fd),deadline=deadline);output(folder/'actual.json',a)
            after=files.snapshot();output(folder/'after.json',after);require(before==after,'Readonly query changed known names/inodes/link/text/body')
            parsed=listing(a,labels)if q['classification']=='supported'else None
            if parsed is not None:bind_listing_to_inodes(parsed,before,q)
            observations.append({'id':q['id'],'classification':q['classification'],'actual':a,'listing':parsed})
    require(control()==c,'Source/ELF changed during observation');require(time.monotonic()<=deadline,'Whole90s deadline')
    output(run/'OBSERVATIONS.json',{'control_sha256':digest(raw),'fixture_sha256':FIXTURE_SHA,'observations':observations,'identity_labels':labels,'limits':'Six directory comparisons/two exclusions; raw real inode IDs preserved. No exact numeric identity/full GNU/permission/platform/curriculum claim'})
def comparison_rows(host,model,model_metadata=None):
    require(tuple(r['id']for r in host)==IDS and tuple(r['id']for r in model)==IDS,'Exact ordered identities')
    result=[];model_labels={}
    for i,(h,m)in enumerate(zip(host,model)):
        classification='supported'if i<6 else'excluded';require(h['classification']==m['classification']==classification,'Exact comparison classification')
        if i<6:
            model_listing=listing(m,model_labels)if 'status'in m else {'normalized':{'error':m.get('error','Missing model outcome')},'parsed':[]}
            if model_metadata is not None and 'error'not in model_listing['normalized']:bind_listing_to_inodes(model_listing,model_metadata,{'id':m['id']})
            n=h['listing']['normalized'];v=model_listing['normalized']
            result.append({'id':h['id'],'classification':classification,'host':h['listing'],'model':model_listing,'exact_relationship_and_relative_columns':('error'not in n and 'error'not in v and n==v)})
        else:result.append({'id':h['id'],'classification':classification,'host_actual':h['actual'],'model':m,'exact_relationship_and_relative_columns':None,'honest_exclusion':m.get('honest_exclusion')is True and isinstance(m.get('error'),str)and('unsupported'in m['error'].lower()or'outside this subset'in m['error'].lower())})
    return result
def compare(args,base,c,raw,lock):
    approve(args,c,raw,'compare');require(os.geteuid()!=0,'Ordinary fixed model probe only');deadline=time.monotonic()+90
    observed=read_bytes(base/'observed/OBSERVATIONS.json',CAP);require(args.approved_observations==digest(observed),'Independent actual raw observation readback required')
    host=strict(observed);require(host['control_sha256']==digest(raw)and host['fixture_sha256']==FIXTURE_SHA,'Exact observed authority')
    run=fresh(base,'compared');retain_inputs(run,c);budget=Budget()
    with pinned_executable(Path(c['probe']['path']),c['probe'],1024*1024*1024)as(exe,fd):
        retain_fd(run/'retained-probe',fd,c['probe']);a=capture_restricted([exe],run,run,environment(run),budget,(lock,fd),deadline=deadline);output(run/'PROBE.json',a)
    require(a['status']==0 and a['stderr']=='','Actual fixed probe failed');model=strict(a['stdout'].encode())
    require(model['purpose']=='fixed-links-inode-reference-v1'and model['compiled_manifest_dir']==str(ROOT)and model['fixture_sha256']==FIXTURE_SHA and model['compiled_inputs_count']==len(c['compiled_inputs'])and model['compiled_inputs_sha256']==binding_commitment(c['compiled_inputs']),'Actual compiled source/root/fixture commitment')
    expected_paths=set(fixture()['directories'])|{e['path']for e in fixture()['entries']}
    require(set(model['source_inodes'])==expected_paths,'Exact independently inspected model entry roster')
    rows=comparison_rows(host['observations'],model['observations'],{p:{'ino':v}for p,v in model['source_inodes'].items()})
    require(control()==c,'Source/ELF changed during compare');require(time.monotonic()<=deadline,'Whole90s comparison deadline')
    output(run/'COMPARISON.json',{'control_sha256':digest(raw),'observations_sha256':digest(observed),'checks':rows,'supported_matches':sum(r['exact_relationship_and_relative_columns']is True for r in rows),'honest_exclusions':sum(r.get('honest_exclusion')is True for r in rows),'limits':'Raw IDs/padding retained; only identity relations/name ordering/relative columns compared, not fake numeric equality. Exclusions NOT matches; old8+6proof immutable'})
    require(all(r['exact_relationship_and_relative_columns']is True if r['classification']=='supported'else r['honest_exclusion']is True for r in rows),'Actual finite mismatch retained')
def main():
    p=argparse.ArgumentParser();p.add_argument('--mode',choices=('prepare','observe','compare'),required=True)
    for name in('control','ls','probe','observations'):p.add_argument('--approved-'+name)
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
