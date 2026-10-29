"""Fixed child-only Linux worker. No CLI, path/source selectors or shell execution."""
import hashlib
import fcntl
import importlib.machinery
import importlib.util
import json
import os
from pathlib import Path
import resource
import stat
import sys

ROOT=None  # Set only by the fixed child entry, never at import time.
BASE='.build-artifacts/reviews/links-reference/captures'
CAPTURE='links8-r2'
CAP=2*1024*1024

def require(ok,message):
    if not ok:raise ValueError(message)

def strict(raw):
    def pairs(items):
        out={}
        for key,value in items:
            require(key not in out,'Duplicate JSON key');out[key]=value
        return out
    require(len(raw)<=CAP,'JSON byte bound')
    return json.loads(raw,object_pairs_hook=pairs)

def read_fd(fd,cap):
    info=os.fstat(fd);require(stat.S_ISREG(info.st_mode) and info.st_size<=cap,'Ordinary bounded descriptor')
    raw=os.pread(fd,info.st_size+1,0);require(len(raw)==info.st_size,'Descriptor size changed')
    return raw

def hash_fd(fd,expected,cap):
    info=os.fstat(fd);require(stat.S_ISREG(info.st_mode) and info.st_size==expected['bytes'] and info.st_size<=cap,'Pinned descriptor size/type')
    h=hashlib.sha256();at=0
    while at<info.st_size:
        block=os.pread(fd,min(65536,info.st_size-at),at);require(bool(block),'Pinned descriptor shrank');h.update(block);at+=len(block)
    require(not os.pread(fd,1,at) and h.hexdigest()==expected['sha256'],'Pinned descriptor bytes')
    return info

def ordinary_directory(path):
    require(path.is_absolute() and '..'not in path.parts,'Canonical directory path')
    fd=os.open('/',os.O_RDONLY|os.O_DIRECTORY|os.O_CLOEXEC)
    try:
        for name in path.parts[1:]:
            child=os.open(name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=fd);os.close(fd);fd=child
        result=fd;fd=-1;return result
    finally:
        if fd>=0:os.close(fd)

def ordinary_read(path,cap):
    directory=ordinary_directory(path.parent)
    try:
        fd=os.open(path.name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK|os.O_CLOEXEC,dir_fd=directory)
        try:return read_fd(fd,cap)
        finally:os.close(fd)
    finally:os.close(directory)

def limits():
    for which,bound in [(resource.RLIMIT_AS,256*1024*1024),(resource.RLIMIT_CPU,2),(resource.RLIMIT_FSIZE,65536),(resource.RLIMIT_NOFILE,128)]:
        a,b=resource.getrlimit(which)
        a=bound if a==resource.RLIM_INFINITY else min(a,bound)
        b=bound if b==resource.RLIM_INFINITY else min(b,bound)
        resource.setrlimit(which,(min(a,b),b))

def packet(raw):
    require(len(raw)<=4096,'Fixed packet exceeds4KiB')
    p=strict(raw);keys={'schema','index','collector_pid','fixture_fd','adapter_fd','worker_fd','python_fd','control_fd','lock_fd'}
    require(type(p)is dict and set(p)==keys and type(p['schema'])is int and p['schema']==1,'Closed worker packet')
    require(type(p['index'])is int and 0<=p['index']<8,'Exactly eight fixed indices')
    require(type(p['collector_pid'])is int and p['collector_pid']>1,'Actual parent PID')
    descriptors=[p[k]for k in keys if k.endswith('_fd')]
    require(all(type(x)is int and 2<x<128 for x in descriptors) and len(set(descriptors))==len(descriptors),'Distinct bounded inherited descriptors')
    return p

def approval_shape(a):
    require(type(a)is dict and type(a.get('schema_version'))is int and a.get('schema_version')==1 and a.get('root_reviewed')is True and a.get('capture')==CAPTURE
        and a.get('root')==str(ROOT) and a.get('host_rows')==8 and a.get('vm_only')==6,'Exact approved worker scope')
    require(a.get('platform')=={'system':os.uname().sysname,'release':os.uname().release,'machine':os.uname().machine},'Approved Linux kernel/platform')
    require(a.get('python_version')==sys.version,'Approved Python version')
    return a

def validate(p):
    require((ROOT/'.git').is_dir() and not(ROOT/'.git').is_symlink(),'MAIN worker only')
    require(sys.platform.startswith('linux') and len(sys.argv)==1 and sys.argv[0]==f"/proc/self/fd/{p['worker_fd']}",'Only pinned worker script descriptor')
    require(sys.flags.isolated and sys.flags.no_site and sys.dont_write_bytecode,'Python -I -S -B required')
    require(os.getuid()!=0 and os.geteuid()==os.getuid(),'Ordinary same-owner worker only')
    require(os.getppid()==p['collector_pid'] and os.getpgrp()==p['collector_pid'],'Actual collector-owned group/parent')
    require(os.environ.get('PATH')=='' and os.environ.get('LC_ALL')=='C' and os.environ.get('LANG')=='C' and 'HOME'not in os.environ,'Closed environment')
    require(not any(k.startswith(('PYTHON','LD_','DYLD_')) or k in ('ENV','BASH_ENV')for k in os.environ),'No startup/import overrides')
    marker=strict(ordinary_read(ROOT/'.build-artifacts/active-command.json',16384))
    require(marker.get('state')=='running' and marker.get('child_pid')==p['collector_pid'],'Actual MAIN runner marker')
    lock=ROOT/'.build-artifacts/heavy-work.lock';root_fd=ordinary_directory(lock.parent)
    try:
        expected=os.stat(lock.name,dir_fd=root_fd,follow_symlinks=False)
    finally:os.close(root_fd)
    actual=os.fstat(p['lock_fd']);require(stat.S_ISREG(expected.st_mode) and (expected.st_dev,expected.st_ino)==(actual.st_dev,actual.st_ino),'Actual shared-lock inode inherited')
    approved_path=ROOT/BASE/(CAPTURE+'-APPROVED.json');raw=read_fd(p['control_fd'],CAP)
    require(raw==ordinary_read(approved_path,CAP),'Same fixed approved control bytes')
    a=approval_shape(strict(raw));pins={x['path']:x for x in a['pins']};require(len(pins)==len(a['pins']),'Unique approved paths')
    for key,name in [('worker_fd','tools/links_reference_worker.py'),('adapter_fd','tools/links_fixed_host.py')]:
        required=fcntl.F_SEAL_WRITE|fcntl.F_SEAL_GROW|fcntl.F_SEAL_SHRINK|fcntl.F_SEAL_SEAL
        require(fcntl.fcntl(p[key],fcntl.F_GET_SEALS)&required==required,'Immutable sealed source descriptor')
        hash_fd(p[key],pins[str(ROOT/name)],65536)
    hash_fd(p['python_fd'],pins[a['python_path']],128*1024*1024)
    executable=os.stat('/proc/self/exe');python=os.fstat(p['python_fd']);require((executable.st_dev,executable.st_ino)==(python.st_dev,python.st_ino),'Actual pinned interpreter is running')
    expected_path=ROOT/BASE/CAPTURE/f"case{p['index']}"/'fixture'
    fd=ordinary_directory(expected_path)
    try:
        x,y=os.fstat(fd),os.fstat(p['fixture_fd'])
        require((x.st_dev,x.st_ino)==(y.st_dev,y.st_ino) and x.st_uid==os.getuid() and stat.S_IMODE(x.st_mode)==0o700,'Exact private case descriptor identity')
        with os.scandir(fd)as entries:require(next(entries,None)is None,'Fresh empty fixture only')
    finally:os.close(fd)
    folder=ROOT/BASE/CAPTURE/f"case{p['index']}"
    require(Path.cwd()==folder/'run','Fixed empty run cwd')
    require(all(os.environ.get(k)==str(folder/'tmp')for k in ['TMPDIR','TMP','TEMP']),'Private fixed disk temporary environment')
    return a

def bounded_names(fd):
    # dup/scandir can share the caller's directory-stream offset. Open the same
    # directory relative to the held descriptor with a new open description.
    scan=os.open('.',os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW|os.O_CLOEXEC,dir_fd=fd)
    try:
        names=[]
        with os.scandir(scan)as entries:
            for entry in entries:
                require(len(names)<16,'Unexpected final directory entry count');names.append(entry.name)
        return sorted(names)
    finally:os.close(scan)

def inventory(fd,allowed,targets):
    """Never follow final symlinks while recording the worker-owned final tree."""
    result=[];total=0;groups={}
    def walk(current,prefix,depth):
        nonlocal total
        require(depth<=2,'Fixed directory-depth bound')
        for name in bounded_names(current):
            path=prefix+name;require(path in allowed and '/'not in name and name not in ('.','..'),'Unexpected final entry')
            info=os.stat(name,dir_fd=current,follow_symlinks=False)
            require(info.st_uid==os.getuid(),'Unexpected fixture owner')
            if stat.S_ISDIR(info.st_mode):
                require(stat.S_IMODE(info.st_mode)==0o700,'Private fixture directory')
                result.append({'path':path,'kind':'Directory'})
                child=os.open(name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=current)
                try:walk(child,path+'/',depth+1)
                finally:os.close(child)
            elif stat.S_ISLNK(info.st_mode):
                target=os.readlink(name,dir_fd=current);require(targets.get(path)==target,'Unexpected symbolic target')
                result.append({'path':path,'kind':'SymbolicLink','target':target})
            else:
                require(stat.S_ISREG(info.st_mode) and 0<=info.st_size<=8192 and 1<=info.st_nlink<=2,'Bounded ordinary fixed file')
                child=os.open(name,os.O_RDONLY|os.O_NOFOLLOW|os.O_NONBLOCK,dir_fd=current)
                try:
                    opened=os.fstat(child);require((opened.st_dev,opened.st_ino)==(info.st_dev,info.st_ino),'Final regular identity')
                    raw=os.read(child,8193);require(len(raw)==info.st_size,'Final regular byte count')
                finally:os.close(child)
                total+=len(raw);require(total<=65536,'Final aggregate data bound')
                groups.setdefault((info.st_dev,info.st_ino),[]).append((len(result),info.st_nlink))
                result.append({'path':path,'kind':'Regular','bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()})
            require(len(result)<=16,'Final entry count bound')
    walk(fd,'',0)
    for group in groups.values():
        require(all(links==len(group)for _,links in group),'Every hardlink stays inside the fixture')
        label=min(result[i]['path']for i,_ in group)
        for i,links in group:result[i].update(links=links,alias_group=label)
    return result

def fixed_root(cwd,index):
    require(len(cwd.parents)>=7 and cwd.name=='run' and cwd.parent.name==f'case{index}' and cwd.parent.parent.name==CAPTURE,'Exact fixed child cwd suffix')
    root=cwd.parents[6]
    require(cwd==root/BASE/CAPTURE/f'case{index}'/'run','Exact project-disk cwd hierarchy')
    return root

def main():
    global ROOT
    p=packet(sys.stdin.buffer.read(4097));ROOT=fixed_root(Path.cwd(),p['index']);limits();validate(p)
    loader=importlib.machinery.SourceFileLoader('links_fixed_host',f"/proc/self/fd/{p['adapter_fd']}")
    module=importlib.util.module_from_spec(importlib.util.spec_from_loader(loader.name,loader));loader.exec_module(module)
    observation=module.observe_fixed_in_owned_fd(p['index'],p['fixture_fd'])
    require(observation['id']==module.IDS[p['index']],'Fixed case order')
    files=inventory(p['fixture_fd'],module.NAMES[p['index']],module.SYMBOLIC[p['index']])
    # Recheck current source/control/descriptor identities after the fixed operations.
    # validate()'s empty assertion is intentionally a pre-operation-only condition.
    raw=read_fd(p['control_fd'],CAP);require(raw==ordinary_read(ROOT/BASE/(CAPTURE+'-APPROVED.json'),CAP),'Approval changed during worker')
    a=approval_shape(strict(raw));pins={x['path']:x for x in a['pins']}
    hash_fd(p['adapter_fd'],pins[str(ROOT/'tools/links_fixed_host.py')],CAP)
    out={'schema_version':1,'index':p['index'],'result':observation,'inventory':files}
    raw=(json.dumps(out,ensure_ascii=False,separators=(',',':'))+'\n').encode();require(len(raw)<=8192,'Worker output bound');sys.stdout.buffer.write(raw)
    return 0

if __name__=='__main__':
    try:raise SystemExit(main())
    except Exception as error:
        sys.stderr.write(('Fixed links worker refused: '+str(error))[:2000]+'\n');raise SystemExit(2)
