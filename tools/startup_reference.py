#!/usr/bin/env python3
"""Fixed startup-source development oracle. Never accepts learner source or paths.
Compares manually selected source files, NOT Bash automatic startup selection."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import resource
import tempfile
import subprocess
import time

from native_reopen import inherited_lock, read_bytes, pin
from automation_before10_oracle import pinned_executable
from case_bash_oracle import capture_command

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = 'tests/fixtures/shell-startup-reference.json'
FIXTURE_SHA = 'ef51a502ebb11cc921d67ff1c972705113e42cc18c71527e6ae78252d9becf9c'
BASE = '.build-artifacts/reference-runs/shell-startup'
OBSERVATIONS_SHA = 'f0bebf4d4b20cbe5efc9e90c0dddbd10edfe0ab485cbef4419ea0f2dac01788a'  # Independently sealed actual Bash observations.
PROBE = 'target/debug/examples/startup_reference'
# Filled with the fixed compiled probe inventory, never a caller path selector.
ENGINE_PATHS = ('Cargo.lock', 'Cargo.toml', 'examples/startup_reference.rs', 'examples/support/automation_digest.rs', 'src/cron.rs', 'src/ere.rs', 'src/ere/cooperative.rs', 'src/lib.rs', 'src/shell.rs', 'src/shell/arithmetic.rs', 'src/shell/capture.rs', 'src/shell/case.rs', 'src/shell/cron.rs', 'src/shell/environment.rs', 'src/shell/execution.rs', 'src/shell/getopts.rs', 'src/shell/globbing.rs', 'src/shell/grep.rs', 'src/shell/literal_trim.rs', 'src/shell/loop_control.rs', 'src/shell/one_off.rs', 'src/shell/parameters.rs', 'src/shell/preparation.rs', 'src/shell/startup.rs', 'src/shell/substitution.rs', 'src/shell/text_tools.rs', 'src/shell/tracing.rs', 'src/shell/values.rs', 'tests/fixtures/shell-startup-reference.json')
TOOL_INPUTS=(FIXTURE,'tools/startup_reference.py','tools/native_reopen.py','tools/automation_before10_oracle.py','tools/case_bash_oracle.py','tools/tests/test_startup_reference.py','tools/dev.py','tools/build_safety.py')
PATHS=('/etc/profile','~/.bash_profile','~/.bash_login','~/.profile','~/.bashrc','/work/startup.env')

CAP = 4 * 1024 * 1024


def require(ok, message):
    if not ok:
        raise ValueError(message)


def strict(raw):
    def unique(items):
        result = {}
        for k, v in items:
            require(k not in result, 'Duplicate JSON key')
            result[k] = v
        return result
    return json.loads(raw, object_pairs_hook=unique)


def digest(path, cap=CAP):
    return dict(path=str(path), **pin(path, cap))


def fixtures():
    raw=read_bytes(ROOT/FIXTURE,256*1024)
    require(hashlib.sha256(raw).hexdigest()==FIXTURE_SHA,'Fixed fixture changed; review first')
    value=strict(raw);rows=value['cases']
    require(set(value)=={'schema_version','cases'}and value['schema_version']==1 and len(rows)==18,'Closed18 fixture roster')
    require(len({r['id']for r in rows})==18,'Duplicate fixture ID')
    for row in rows:
        require(set(row)=={'id','invocation','files','expected_selection','env','command','observe'},'Closed fixed recipe')
        require(re.fullmatch(r'[a-z][a-z-]{0,63}',row['id'])and set(row['env'])<={'SOURCE','SUFFIX','OPTERR'},'Only ordinary reviewed inputs; no loader/path controls')
        require(len(row['files'])<=6 and set(row['files'])<=set(PATHS),'Known virtual startup labels only')
        require(all(set(f)=={'text'}and isinstance(f['text'],str)and len(f['text'])<=2048 and '\0'not in f['text']for f in row['files'].values()),'Bounded fixed source')
        require(isinstance(row['command'],str)and len(row['command'])<=2048 and '\0'not in row['command'],'Fixed command')
        require(1<=len(row['observe'])<=6 and len(set(row['observe']))==len(row['observe'])and all(re.fullmatch('[A-Z][A-Z0-9_]*',n)for n in row['observe']),'Fixed ordinary observation names')
        require(all(isinstance(v,str)and len(v)<=1024 and '\0'not in v for v in row['env'].values()),'Bounded synthetic input')
        require(len(row['expected_selection'])<=2 and len(set(row['expected_selection']))==len(row['expected_selection'])and all(n in row['files']for n in row['expected_selection']),'Reviewed selection roster')
        mode=row['invocation'];kind=mode.get('kind')
        require((kind=='InteractiveLogin'and set(mode)=={'kind','noprofile'}and type(mode['noprofile'])is bool)or(kind=='InteractiveNonLogin'and set(mode)=={'kind','norc'}and type(mode['norc'])is bool)or(kind=='NonInteractive'and set(mode)=={'kind','bash_env'}and mode['bash_env']in(None,'/work/startup.env')),'Fixed supported invocation')
    return rows


def output(path, value):
    with path.open('x') as f:
        json.dump(value, f, ensure_ascii=False, indent=2)
        f.write('\n'); f.flush(); os.fsync(f.fileno())


def fresh(parent, name):
    require(parent.resolve() == parent and not parent.is_symlink(), 'Canonical reference root')
    path = parent / name
    path.mkdir(mode=0o700)
    return path


def limits():
    resource.setrlimit(resource.RLIMIT_CPU, (2, 2))
    resource.setrlimit(resource.RLIMIT_AS, (256 * 1024 * 1024,) * 2)
    resource.setrlimit(resource.RLIMIT_FSIZE, (1024 * 1024,) * 2)
    resource.setrlimit(resource.RLIMIT_NOFILE, (32, 32))


class Budget:
    def __init__(self):
        self.remaining = CAP
    def charge(self, n):
        require(0 <= n <= self.remaining, 'Whole reference output limit')
        self.remaining -= n


def script(row):
    # Explicit dot-sourcing is a development observer only. It does not test
    # automatic Bash startup selection or become learner execution.
    observer=r'''__dlms_observe() {
  __dlms_status=$?
  {
    printf '%s\0' "$__dlms_status"
    for __dlms_name in '''+' '.join(row['observe'])+r'''; do
      __dlms_set=0; __dlms_value=''
      if [[ -v "$__dlms_name" ]]; then __dlms_set=1; __dlms_value=${!__dlms_name}; fi
      __dlms_decl=$(declare -p "$__dlms_name" 2> declaration.stderr)
      __dlms_export=0
      if [[ "$__dlms_decl" == 'declare -x '* ]]; then __dlms_export=1; fi
      printf '%s\0%s\0%s\0%s\0' "$__dlms_name" "$__dlms_set" "$__dlms_export" "$__dlms_value"
    done
  } > observer.protocol 2> observer.stderr
}
trap '__dlms_observe' EXIT
{
'''
    body=''.join('. ./startup-'+str(n)+'.sh\n'for n,_ in enumerate(row['expected_selection']))
    return observer+body+row['command']+'\n} > business.stdout 2> business.stderr\n'


def decode_protocol(text, names):
    fields = text.split('\0')
    require(fields[-1] == '' and len(fields) == 2 + 4 * len(names), 'Complete finite observer protocol')
    require(fields[0].isdecimal() and 0 <= int(fields[0]) <= 255, 'Observed exit status')
    values = {}
    for i, name in enumerate(names):
        key, present, exported, value = fields[1 + 4*i:5 + 4*i]
        require(key == name and present in ['0', '1'] and exported in ['0', '1'], 'Observer name/flags')
        require(present == '1' or not value, 'Unset value must be absent')
        values[name] = dict(value=value if present == '1' else None, exported=exported == '1')
    return int(fields[0]), values


def observation_rows(rows, recipes):
    require(len(rows) == len(recipes) and [r['id'] for r in rows] == [r['id'] for r in recipes], 'Full ordered observation roster')
    for row, recipe in zip(rows, recipes, strict=True):
        require(set(row) == {'id','selection','stdout','stderr','status','files','error','variables'}, 'Closed outcome shape')
        require(row['selection']==recipe['expected_selection'],'Selection must match independent primary-doc roster; not actual automatic-Bash acceptance')
        require(isinstance(row['stdout'],str)and isinstance(row['stderr'],str)and len(row['stdout'].encode())<=32768 and len(row['stderr'].encode())<=32768,'Bounded actual streams')
        require(type(row['files'])is dict and set(row['files'])=={'/work/keep.txt'}and all(isinstance(v,str)and len(v.encode())<=32768 for v in row['files'].values()),'Exact preserved virtual file roster')
        require(type(row['status']) is int and 0 <= row['status'] <= 255 and row['error'] is None, 'Complete successful observation')
        require(set(row['variables']) == set(recipe['observe']), 'Exact observed names')
        for v in row['variables'].values():
            require(set(v) == {'value','exported'} and type(v['exported']) is bool
                    and (v['value'] is None or isinstance(v['value'], str)), 'Typed actual binding')


def control(mode):
    inputs = TOOL_INPUTS + (ENGINE_PATHS if mode == 'compare' else ())
    program = ROOT / PROBE if mode == 'compare' else Path('/usr/bin/bash').resolve(strict=True)
    return dict(purpose='Fixed developer manually sourced startup '+mode, root_reviewed=False,
                fixture_sha256=FIXTURE_SHA, pins=[digest(ROOT/n) for n in sorted(set(inputs))],
                executable=digest(program, 1024*1024*1024 if mode == 'compare' else 16*1024*1024))


def control_key(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':')).encode()).hexdigest()[:16]


def approved(base, mode):
    expected = control(mode)
    actual = strict(read_bytes(base / (mode+'-approved-'+control_key(expected)+'.json'), CAP))
    require(actual.get('root_reviewed') is True and actual == {**expected, 'root_reviewed':True}, 'Independent exact source/executable approval required')
    return expected


def observe(base, recipes, lock):
    c=approved(base,'observe');run=fresh(base,'bash-r1');budget=Budget();started=time.monotonic()
    output(run/'control.json',c)
    for name in TOOL_INPUTS:
        p=run/'retained-inputs'/name;p.parent.mkdir(parents=True,exist_ok=True)
        with p.open('xb')as f:f.write(read_bytes(ROOT/name,CAP));f.flush();os.fsync(f.fileno())
    rows=[]
    with pinned_executable(Path(c['executable']['path']),c['executable'],16*1024*1024)as(executable,fd),(run/'observations.jsonl').open('x')as journal:
        for index,recipe in enumerate(recipes):
            require(time.monotonic()-started<60,'Whole fixed observation deadline')
            work=fresh(run,str(index));capture=fresh(work,'capture')
            for n,path in enumerate(recipe['expected_selection']):
                with(work/f'startup-{n}.sh').open('x')as f:f.write(recipe['files'][path]['text'])
            with(work/'keep.txt').open('x')as f:f.write('Retained fixture bytes.\n')
            source=script(recipe)
            with(work/'observer.sh').open('x')as f:f.write(source)
            env=dict(PATH='',LC_ALL='C',LANG='C',HOME='/home/learner',TMPDIR=str(work),**recipe['env'])
            actual=capture_command([executable,'--noprofile','--norc','-c',source,'fixed-startup'],capture,work,env,b'',budget,65536,timeout=3,inherited=(lock,fd),popen=lambda *a,**kw:subprocess.Popen(*a,preexec_fn=limits,**kw))
            require(not actual['stdout']and not actual['stderr'],'Fixed observer failed; actual streams retained')
            status,variables=decode_protocol(read_bytes(work/'observer.protocol',32768).decode(),recipe['observe'])
            require(actual['status']==status,'EXIT observer and real process status differ')
            require(not read_bytes(work/'observer.stderr',32768),'Observer diagnostics retained; refuse acceptance')
            names={f'startup-{n}.sh'for n in range(len(recipe['expected_selection']))}|{'capture','keep.txt','observer.sh','observer.protocol','observer.stderr','business.stdout','business.stderr','declaration.stderr'}
            require({p.name for p in work.iterdir()}==names,'Unexpected reference artifact')
            row=dict(id=recipe['id'],selection=recipe['expected_selection'],stdout=read_bytes(work/'business.stdout',32768).decode(),stderr=read_bytes(work/'business.stderr',32768).decode(),status=status,files={'/work/keep.txt':read_bytes(work/'keep.txt',32768).decode()},error=None,variables=variables)
            budget.charge(len(json.dumps(row,ensure_ascii=False).encode()))
            journal.write(json.dumps(row,ensure_ascii=False)+'\n');journal.flush();os.fsync(journal.fileno());rows.append(row)
    observation_rows(rows,recipes)
    output(run/'observations.json',dict(purpose='Actual fixed Bash explicit sources; NOT automatic startup selection',fixture_sha256=FIXTURE_SHA,control=c,observations=rows))
    print('OBSERVED18 fixed Bash sourced-settings recipes; unsealed: '+str(run))


def compare(base, recipes, lock):
    require(OBSERVATIONS_SHA is not None, 'Actual observation seal pending')
    raw=read_bytes(base/'bash-r1/observations.json',CAP)
    require(hashlib.sha256(raw).hexdigest()==OBSERVATIONS_SHA,'Sealed reference differs')
    actual=strict(raw);observation_rows(actual['observations'],recipes)
    require(actual['fixture_sha256']==FIXTURE_SHA,'Reference fixture differs')
    c=approved(base,'compare');run=Path(tempfile.mkdtemp(prefix='compare-'+control_key(c)+'-',dir=base));output(run/'control.json',c)
    for n in ENGINE_PATHS:
        p=run/'compiled-source'/n;p.parent.mkdir(parents=True,exist_ok=True)
        with p.open('xb')as f:f.write(read_bytes(ROOT/n,CAP));f.flush();os.fsync(f.fileno())
    with pinned_executable(Path(c['executable']['path']),c['executable'],1024*1024*1024)as(executable,fd):
        with (run/'probe-elf').open('xb')as f:
            at=0
            while at<c['executable']['bytes']:
                b=os.pread(fd,min(1024*1024,c['executable']['bytes']-at),at);require(bool(b),'Probe shrank');f.write(b);at+=len(b)
            f.flush();os.fsync(f.fileno())
        require(pin(run/'probe-elf',1024*1024*1024)=={k:c['executable'][k]for k in ['bytes','sha256']},'Retained executed probe differs')
        capture=fresh(run,'capture')
        result=capture_command([executable],capture,ROOT,os.environ.copy(),b'',Budget(),CAP,timeout=60,inherited=(lock,fd))
    output(run/'execution.json',result)
    require(result['status']==0 and not result['stderr'],'Probe failed; actual streams retained')
    vm=strict(result['stdout'])
    require(vm['evaluator_revision']==17 and vm['fixture_sha256']==FIXTURE_SHA,'Actual17 fixture context required')
    expected=[dict(path=n,**pin(ROOT/n,CAP))for n in ENGINE_PATHS]
    require(vm['compiled_inputs']==expected,'Stale compiled probe/source')
    observation_rows(vm['observations'],recipes)
    failures=[a['id']for a,b in zip(vm['observations'],actual['observations'],strict=True)if a!=b]
    output(run/'report.json',dict(purpose='Actual in-process startup VM versus fixed manually sourced Bash; not automatic startup acceptance',vm=vm,reference=actual,failures=failures))
    require(not failures,'Startup mismatches retained: '+','.join(failures))
    print('PASS18 actual sequential-startup comparisons: '+str(run))


def main():
    parser=argparse.ArgumentParser(description=__doc__,allow_abbrev=False)
    parser.add_argument('--mode',choices=['prepare-observe','observe','prepare-compare','compare','self-test'],required=True)
    args=parser.parse_args();lock=inherited_lock()
    require((ROOT/'.git').is_dir() and Path(os.environ.get('CARGO_TARGET_DIR',''))==ROOT/'target','MAIN/default runner only')
    if args.mode=='self-test':
        import unittest
        result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.discover(str(ROOT/'tools/tests'),pattern='test_startup_reference.py'))
        require(result.wasSuccessful(),'Startup collector guard tests failed');return
    rows=fixtures();base=ROOT
    for part in BASE.split('/'):
        base=base/part
        require(not base.is_symlink(),'No reference-root aliases')
        base.mkdir(exist_ok=True)
    require(base.resolve()==base,'Canonical project-disk reference root')
    if args.mode.startswith('prepare-'):
        mode=args.mode.removeprefix('prepare-');c=control(mode);destination=base/(mode+'-control-'+control_key(c)+'.json');output(destination,c);print('PROPOSED; no child executed: '+str(destination));return
    if args.mode=='observe':observe(base,rows,lock)
    else:compare(base,rows,lock)


if __name__=='__main__':
    main()
