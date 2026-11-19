#!/usr/bin/env python3
"""Fixed authored-project Bash comparisons; development only, MAIN runner only.
No CLI source/profile/path selector and no learner input. Evaluator15 is unchanged.
"""
import hashlib
import json
import os
from pathlib import Path
import resource
import stat
import subprocess
import sys
import tempfile
from native_reopen import inherited_lock, read_bytes, ReopenError
ROOT=Path(__file__).resolve().parents[1]
PINS=(('scenarios/shell-debug-recovery-v1.json', 37785, 'bc5647171d95d54816dacd1777603c87a6a6d0174bfee6509a6c694a96e9adae'), ('tests/fixtures/shell-debug-recovery-cases.json', 24827, 'd8bfb1da7bea19608b723b17f4e6f737633e6f1001f6334e7438ce54d3d6714b'))
IDS=('shell-debug-status','shell-recovery-ledger')
def read_pinned(name,size,sha):
    path=ROOT/name;info=path.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_nlink!=1 or info.st_size!=size:raise ValueError('Reference input type/size changed')
    try:raw=read_bytes(path,size)
    except (OSError,ReopenError)as e:raise ValueError('Aliased or changing reference input')from e
    if hashlib.sha256(raw).hexdigest()!=sha:raise ValueError('Reference input changed; review the exact source before refreshing its pin')
    return json.loads(raw)
def limits():
    resource.setrlimit(resource.RLIMIT_CPU,(2,2))
    resource.setrlimit(resource.RLIMIT_AS,(256*1024*1024,256*1024*1024))
    resource.setrlimit(resource.RLIMIT_FSIZE,(1024*1024,1024*1024))
    resource.setrlimit(resource.RLIMIT_NOFILE,(32,32))
def main():
    if len(sys.argv)!=1:raise ValueError('This fixed comparison has no source/path selectors')
    lock=inherited_lock()
    pack,fixtures=(read_pinned(*p)for p in PINS)
    rows=pack['scenarios']
    if tuple(r['id']for r in rows)!=IDS or set(fixtures)!=set(IDS):raise ValueError('Fixed recipe roster changed')
    bash=Path('/usr/bin/bash').resolve(strict=True)
    if not bash.is_file():raise ValueError('Reference Bash unavailable; comparison NOT RUN')
    base=ROOT/'.build-artifacts/reference-runs';base.mkdir(exist_ok=True)
    output=Path(tempfile.mkdtemp(prefix='debug-recovery-',dir=base))
    report={'purpose':'Authored project comparison, not a new evaluator baseline','bash':str(bash),'bash_sha256':hashlib.sha256(bash.read_bytes()).hexdigest(),'input_pins':PINS,'observations':[]}
    for row in rows:
        source=row['solution'][0]
        for n,c in enumerate(fixtures[row['id']]):
            if c['env'] or c['stdin'] or len(c['files'])>16:raise ValueError('Unexpected fixed fixture interface')
            work=output/(row['id']+'-'+str(n));work.mkdir(mode=0o700)
            for name,text in c['files'].items():
                if not name.startswith('/work/') or '/'in name[6:]or name[6:]in['','.','..']:raise ValueError('Flat fixed fixture paths only')
                (work/name[6:]).write_bytes(text.encode())
            p=subprocess.run([str(bash),'--noprofile','--norc','-c',source,'fixture',*c['args']],cwd=work,env={'PATH':'/usr/bin:/bin','LC_ALL':'C'},input=b'',capture_output=True,timeout=3,preexec_fn=limits,pass_fds=(lock,))
            if len(p.stdout)+len(p.stderr)>65536:raise ValueError('Reference output ceiling')
            files={}
            for path in work.iterdir():
                if not stat.S_ISREG(path.lstat().st_mode)or path.stat().st_size>65536:raise ValueError('Unexpected reference artifact')
                files['/work/'+path.name]=path.read_bytes().decode()
            actual={'stdout':p.stdout.decode(),'stderr':p.stderr.decode(),'status':p.returncode,'files':files}
            expected={'stdout':c['stdout'],'stderr':c['stderr'],'status':c['status'],'files':c['after']}
            report['observations'].append({'lab':row['id'],'name':c['name'],'actual':actual,'expected':expected,'passed':actual==expected})
    report['passed']=all(x['passed']for x in report['observations'])
    (output/'report.json').write_text(json.dumps(report,indent=2,ensure_ascii=False)+'\n')
    print(json.dumps({'output':str(output),'cases':len(report['observations']),'passed':report['passed']}))
    return 0 if report['passed']else 1
if __name__=='__main__':raise SystemExit(main())
