#!/usr/bin/env python3
"""Fixed export-state development oracle. Never accepts learner source or paths."""
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
FIXTURE = 'tests/fixtures/shell-environment-reference.json'
FIXTURE_SHA = '4e88810330980a1fc6a5fb25a2676eb46f0274a41011a8d940c7381f93560e25'
BASE = '.build-artifacts/reference-runs/shell-environment'
OBSERVATIONS_SHA = 'dfe163df8316e6867324fca56bbc7ae186b09ee6ff9a3084326152de98335454'  # Only after actual observation and independent readback.
PROBE = 'target/debug/examples/environment_reference'
# Filled with the fixed compiled probe inventory, never a caller path selector.
ENGINE_PATHS = ('Cargo.lock', 'Cargo.toml', 'examples/environment_reference.rs', 'examples/support/automation_digest.rs', 'src/ere.rs', 'src/ere/cooperative.rs', 'src/lib.rs', 'src/shell.rs', 'src/shell/arithmetic.rs', 'src/shell/capture.rs', 'src/shell/case.rs', 'src/shell/cron.rs', 'src/shell/environment.rs', 'src/shell/execution.rs', 'src/shell/getopts.rs', 'src/shell/globbing.rs', 'src/shell/grep.rs', 'src/shell/literal_trim.rs', 'src/shell/loop_control.rs', 'src/shell/one_off.rs', 'src/shell/parameters.rs', 'src/shell/preparation.rs', 'src/shell/substitution.rs', 'src/shell/text_tools.rs', 'src/shell/tracing.rs', 'src/shell/values.rs', 'tests/fixtures/shell-environment-reference.json')
TOOL_INPUTS = (FIXTURE, 'tools/environment_reference.py', 'tools/native_reopen.py',
               'tools/automation_before10_oracle.py', 'tools/case_bash_oracle.py',
               'tools/tests/test_environment_reference.py', 'tools/dev.py', 'tools/build_safety.py')
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
    raw = read_bytes(ROOT / FIXTURE, 256 * 1024)
    require(hashlib.sha256(raw).hexdigest() == FIXTURE_SHA, 'Fixed fixture changed; review first')
    value = strict(raw)
    rows = value['cases']
    require(value['schema_version'] == 1 and len(rows) == 28, 'Closed28 fixture roster')
    require(len({r['id'] for r in rows}) == len(rows), 'Duplicate fixture ID')
    for row in rows:
        require(set(row) == {'id', 'source', 'env', 'args', 'stdin', 'files', 'observe', 'host_observation'}
                and row['host_observation'] is None, 'Only unobserved fixed recipes')
        require(len(row['source']) < 2048 and len(row['args']) <= 4 and len(row['env']) <= 4
                and 1 <= len(row['observe']) <= 6 and len(set(row['observe'])) == len(row['observe']) and len(row['stdin']) <= 1024, 'Finite recipe')
        require(all(re.fullmatch(r'[A-Z][A-Z0-9_]*', n) for n in row['observe'] + list(row['env']))
                and not set(row['env']) & {'PATH', 'BASH_ENV', 'ENV', 'SHELLOPTS', 'BASHOPTS'}, 'Ordinary fixed names')
        require(len(row['files']) <= 4 and all(re.fullmatch(r'/work/[A-Za-z][A-Za-z0-9=.-]*', p)
                for p in row['files']), 'Only ordinary fixed virtual leaf files')
        require(all(isinstance(s, str) and '\0' not in s and len(s) <= 4096
                    for s in row['args'] + list(row['env'].values()) + list(row['files'].values())), 'Bounded UTF8 values')
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
    # The observer uses only fixed Bash builtins. Group redirection leaves the
    # tested declarations in the same shell; no source/function wrapper changes scope.
    body = '{\n' + row['source'] + '\n__dlms_status=$?\n} > business.stdout 2> business.stderr\n'
    return body + r'''printf '%s\0' "$__dlms_status"
for __dlms_name in ''' + ' '.join(row['observe']) + r'''; do
  __dlms_set=0
  __dlms_value=''
  if [[ -v "$__dlms_name" ]]; then
    __dlms_set=1
    __dlms_value=${!__dlms_name}
  fi
  __dlms_decl=$(declare -p "$__dlms_name" 2> declaration.stderr)
  __dlms_export=0
  if [[ "$__dlms_decl" == 'declare -x '* ]]; then __dlms_export=1; fi
  printf '%s\0%s\0%s\0%s\0' "$__dlms_name" "$__dlms_set" "$__dlms_export" "$__dlms_value"
done
'''


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
        require(set(row) == {'id','stdout','stderr','status','files','error','variables'}, 'Closed outcome shape')
        require(type(row['status']) is int and 0 <= row['status'] <= 255 and row['error'] is None, 'Complete successful observation')
        require(set(row['variables']) == set(recipe['observe']), 'Exact observed names')
        for v in row['variables'].values():
            require(set(v) == {'value','exported'} and type(v['exported']) is bool
                    and (v['value'] is None or isinstance(v['value'], str)), 'Typed actual binding')


def control(mode):
    inputs = TOOL_INPUTS + (ENGINE_PATHS if mode == 'compare' else ())
    program = ROOT / PROBE if mode == 'compare' else Path('/usr/bin/bash').resolve(strict=True)
    return dict(purpose='Fixed developer environment '+mode, root_reviewed=False,
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
    c = approved(base, 'observe')
    run = fresh(base, 'bash-r1'); budget = Budget(); started = time.monotonic()
    output(run/'control.json', c)
    for name in TOOL_INPUTS:
        p = run/'retained-inputs'/name; p.parent.mkdir(parents=True, exist_ok=True)
        with p.open('xb') as f: f.write(read_bytes(ROOT/name,CAP)); f.flush(); os.fsync(f.fileno())
    rows = []
    with pinned_executable(Path(c['executable']['path']), c['executable'], 16*1024*1024) as (executable, fd), (run/'observations.jsonl').open('x') as journal:
        for index, recipe in enumerate(recipes):
            require(time.monotonic()-started < 60, 'Whole fixed observation deadline')
            work = fresh(run,str(index)); capture = fresh(work,'capture')
            for path, value in recipe['files'].items():
                with (work/Path(path).name).open('x') as f: f.write(value)
            env = dict(PATH='', LC_ALL='C', LANG='C', HOME=str(work), TMPDIR=str(work), **recipe['env'])
            actual = capture_command([executable,'--noprofile','--norc','-c',script(recipe),'fixed-environment',*recipe['args']],
                capture,work,env,recipe['stdin'].encode(),budget,65536,timeout=3,inherited=(lock,fd),
                popen=lambda *a,**kw: subprocess.Popen(*a,preexec_fn=limits,**kw))
            require(actual['status']==0 and not actual['stderr'], 'Fixed observer failed; raw streams retained')
            status, variables = decode_protocol(actual['stdout'],recipe['observe'])
            names={Path(n).name for n in recipe['files']}|{'capture','business.stdout','business.stderr','declaration.stderr'}
            require({p.name for p in work.iterdir()}==names, 'Unexpected reference artifact')
            row=dict(id=recipe['id'],stdout=read_bytes(work/'business.stdout',32768).decode(),
                     stderr=read_bytes(work/'business.stderr',32768).decode(),status=status,
                     files={n:read_bytes(work/Path(n).name,32768).decode()for n in recipe['files']},error=None,variables=variables)
            journal.write(json.dumps(row,ensure_ascii=False)+'\n');journal.flush();os.fsync(journal.fileno());rows.append(row)
    observation_rows(rows,recipes)
    output(run/'observations.json',dict(purpose='Actual fixed Bash state observations; not VM acceptance',fixture_sha256=FIXTURE_SHA,control=c,observations=rows))
    print('Observed28 fixed Bash recipes; independent readback/VM comparison still pending')


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
    output(run/'report.json',dict(purpose='Actual in-process VM versus fixed Bash',vm=vm,reference=actual,failures=failures))
    require(not failures,'Environment mismatches retained: '+','.join(failures))
    print('PASS28 actual environment comparisons: '+str(run))


def main():
    parser=argparse.ArgumentParser(description=__doc__,allow_abbrev=False)
    parser.add_argument('--mode',choices=['prepare-observe','observe','prepare-compare','compare','self-test'],required=True)
    args=parser.parse_args();lock=inherited_lock()
    require((ROOT/'.git').is_dir() and Path(os.environ.get('CARGO_TARGET_DIR',''))==ROOT/'target','MAIN/default runner only')
    if args.mode=='self-test':
        import unittest
        result=unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.discover(str(ROOT/'tools/tests'),pattern='test_environment_reference.py'))
        require(result.wasSuccessful(),'Environment collector guard tests failed');return
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
