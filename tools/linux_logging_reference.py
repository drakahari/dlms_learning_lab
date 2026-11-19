#!/usr/bin/env python3
"""Fixed classic-selector development fixtures, never learner configuration.
MAIN runner only. No system configuration, host log input, socket listener or service change.
"""
import collections
import hashlib
import json
import os
from pathlib import Path
import resource
import stat
import subprocess
import sys
import tempfile
import time
from native_reopen import inherited_lock, read_bytes, ReopenError
ROOT=Path(__file__).resolve().parents[1]
CORE_PIN=('src/linux_logging.rs', 9698, 'a110f3ced69d567c0cbe0de086837131369d779f656613b0dcf4c7aa37f44b9f')
LEVELS=['emerg','alert','crit','err','warning','notice','info','debug']
EVENTS=[(f,n)for f in ['mail','daemon','authpriv','cron','ftp','local0']for n in [0,3,4,5,6,7]]
# Fixed routing fragments. No command-line or imported source selector exists.
FRAGMENTS={
 'threshold':['*.warning general'],
 'exact':['daemon.=info service','*.err general'],
 'private':['*.notice;authpriv.none general','authpriv.* private'],
 'duplicates':['daemon.err general','daemon.err general'],
 'grouped':['daemon,cron.=info service','*.crit general'],
}
def expected(kind,facility,level):
    if kind=='threshold':return ['general']if level<=4 else[]
    if kind=='exact':return (['service']if facility=='daemon'and level==6 else[])+(['general']if level<=3 else[])
    if kind=='private':return ['private']if facility=='authpriv'else(['general']if level<=5 else[])
    if kind=='duplicates':return ['general','general']if facility=='daemon'and level<=3 else[]
    if kind=='grouped':return (['service']if facility in ['daemon','cron']and level==6 else[])+(['general']if level<=2 else[])
    raise ValueError('Unknown fixed policy')
def limits():
    resource.setrlimit(resource.RLIMIT_CPU,(4,4));resource.setrlimit(resource.RLIMIT_AS,(512*1024*1024,512*1024*1024))
    resource.setrlimit(resource.RLIMIT_FSIZE,(1024*1024,1024*1024));resource.setrlimit(resource.RLIMIT_NOFILE,(128,128))
def main():
    if len(sys.argv)!=1:raise ValueError('No source/config/path/process selectors are accepted')
    lock=inherited_lock()
    name,size,sha=CORE_PIN;p=ROOT/name;info=p.lstat()
    if not stat.S_ISREG(info.st_mode)or info.st_nlink!=1 or info.st_size!=size:raise ValueError('Reference model type/size changed')
    try:raw=read_bytes(p,size)
    except (OSError,ReopenError)as e:raise ValueError('Aliased or changing reference model')from e
    if len(raw)!=size or hashlib.sha256(raw).hexdigest()!=sha:raise ValueError('Review the changed model before running its reference fixtures')
    binary=Path('/usr/bin/rsyslogd').resolve(strict=True)
    base=ROOT/'.build-artifacts/reference-runs';base.mkdir(exist_ok=True)
    root=Path(tempfile.mkdtemp(prefix='linux-logging-',dir=base))
    report={'purpose':'Fixed imfile-only classic-selector comparison, not a full rsyslog model','binary':str(binary),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'core_pin':CORE_PIN,'cases':[]}
    env={'PATH':'/usr/bin:/bin','LANG':'C','LC_ALL':'C','TMPDIR':str(root)}
    for kind,fragment in FRAGMENTS.items():
        work=root/kind;work.mkdir(mode=0o700);state=work/'state';state.mkdir()
        paths={k:work/(k+'.log')for k in ['general','private','service']};observed=work/'observed.log'
        # The ONLY input module is private text files. Internal messages stay in
        # this process and are discarded before the fixture rules, never /dev/log.
        lines=[f'global(workDirectory={json.dumps(str(state))} processInternalMessages="on")',
               'module(load="imfile" mode="polling" PollingInterval="1")',
               'template(name="FixtureOnly" type="string" string="%msg%\\n")',
               '$ActionFileDefaultTemplate FixtureOnly', '$RepeatedMsgReduction off',
               'if $inputname != "imfile" then stop']
        wants={k:[]for k in paths}
        for n,(facility,level)in enumerate(EVENTS):
            identity=f'case-{n:02}';file=work/(identity+'.txt');file.write_text(identity+'\n')
            lines.append(f'input(type="imfile" File={json.dumps(str(file))} Tag="fixture:" Facility="{facility}" Severity="{LEVELS[level]}" PersistStateInterval="0")')
            for dest in expected(kind,facility,level):wants[dest].append(identity)
        lines.append(f'*.* {observed}')
        for rule in fragment:
            selector,destination=rule.split();lines.append(selector+' '+str(paths[destination]))
        config=work/'fixture.conf';config.write_text('\n'.join(lines)+'\n')
        common=[str(binary),'-f',str(config),'-i',str(work/'fixture.pid')]
        syntax=subprocess.run([*common,'-N1'],cwd=work,env=env,capture_output=True,timeout=5,preexec_fn=limits,pass_fds=(lock,))
        (work/'syntax.stdout').write_bytes(syntax.stdout);(work/'syntax.stderr').write_bytes(syntax.stderr)
        if syntax.returncode:raise RuntimeError('Fixed syntax validation failed; retained '+str(work))
        with (work/'runtime.log').open('xb')as log:
            child=subprocess.Popen([*common,'-n'],cwd=work,env=env,stdout=log,stderr=subprocess.STDOUT,preexec_fn=limits,pass_fds=(lock,))
            seen=False
            try:
                deadline=time.monotonic()+5
                while time.monotonic()<deadline and child.poll()is None:
                    if observed.exists()and len(observed.read_bytes().splitlines())>=len(EVENTS):seen=True;break
                    time.sleep(.04)
                if child.poll()is None:child.terminate()
                try:child.wait(timeout=3)
                except subprocess.TimeoutExpired:child.kill();child.wait(timeout=2);raise RuntimeError('Owned reference process did not exit normally')
            finally:
                if child.poll()is None:child.kill();child.wait(timeout=2)
        actual={k:(v.read_bytes().decode().splitlines()if v.exists()else[])for k,v in paths.items()}
        passed=seen and child.returncode==0 and all(collections.Counter(actual[k])==collections.Counter(wants[k])for k in paths)
        report['cases'].append({'policy':kind,'input_events':len(EVENTS),'observed_all':seen,'exit_status':child.returncode,'actual':actual,'expected':wants,'passed':passed})
    report['passed']=all(c['passed']for c in report['cases'])
    (root/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({'output':str(root),'policies':len(report['cases']),'events_per_policy':len(EVENTS),'passed':report['passed']}))
    return 0 if report['passed']else 1
if __name__=='__main__':raise SystemExit(main())
