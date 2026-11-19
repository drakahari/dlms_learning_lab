#!/usr/bin/env python3
"""Fixed date comparison only; never runs learner input or changes a host clock."""
import argparse
from contextlib import ExitStack
import hashlib
import json
from pathlib import Path
import stat
import time
from automation_before10_oracle import pinned_executable
from case_bash_oracle import closed_environment, require_guard
from localization_reference import require, execute
from native_reopen import pin, read_bytes
from startup_reference import Budget, fresh, output

ROOT=Path(__file__).resolve().parents[1]
FIXTURE='tests/fixtures/timezone-reference-v1.json'
FIXTURE_SHA='5ed46017fa15552b98c13d251832215516f13114833ff51842e808bcf0cfe675'
SOURCES=('Cargo.toml','Cargo.lock','src/lib.rs','src/timezones.rs',
         'src/timezones/format.rs','src/timezones/command.rs','src/timezones/command/literal.rs',
         'examples/timezone_reference.rs','examples/support/whereis_digest.rs',FIXTURE)
ZONES={'UTC','Etc/UTC','Europe/Berlin','America/New_York','Asia/Kolkata'}
FORMATS={'%F %T %z %Z','%a %b %e %H:%M:%S %Z %Y','%Y/%m/%d %R %:z %s %%'}
IDS=('utc-winter', 'utc-summer', 'utc-year-start', 'utc-year-end', 'berlin-winter', 'berlin-summer', 'berlin-year-start', 'berlin-year-end', 'new-york-winter', 'new-york-summer', 'new-york-year-start', 'new-york-year-end', 'kolkata-winter', 'kolkata-summer', 'kolkata-year-start', 'kolkata-year-end', 'berlin-spring-before', 'berlin-spring-after', 'berlin-autumn-before', 'berlin-autumn-after', 'new-york-spring-before', 'new-york-spring-after', 'new-york-autumn-before', 'new-york-autumn-after', 'process-override-winter', 'process-override-summer', 'empty-tz-means-utc', 'utc-view-overrides-tz', 'utc-alias', 'half-hour-epoch-format')
TOOLS=('tools/timezone_reference.py','tools/localization_reference.py','tools/dev.py',
       'tools/build_safety.py','tools/automation_before10_oracle.py','tools/case_bash_oracle.py',
       'tools/native_reopen.py','tools/startup_reference.py')

def validate_fixture(value):
    require(isinstance(value,dict) and set(value)=={'version','cases'}
            and type(value['version'])is int and value['version']==1,'Closed fixed fixture schema')
    require(isinstance(value['cases'],list) and len(value['cases'])==30,'Exact30inert reference cases')
    for row,id in zip(value['cases'],IDS):
        require(isinstance(row,dict) and set(row)=={'id','configured','tz','epoch','utc','format'}
                and row['id']==id,'Exact case identity and fields; no scripts or paths')
        require(isinstance(row['configured'],str) and row['configured']in ZONES,'Supplied zone only')
        require(row['tz'] is None or isinstance(row['tz'],str) and row['tz']in ZONES|{''},'Closed TZ override')
        require(type(row['epoch'])is int and 1767225600<=row['epoch']<1798761600,'Fixed2026whole seconds')
        require(type(row['utc'])is bool,'Explicit UTC-view flag')
        require(isinstance(row['format'],str) and row['format']in FORMATS,'Closed literal date formats')
    require(len(IDS)==30,'Closed compiled observer roster')
    return value

def source_pins():
    return [{'path':name,**pin(ROOT/name,4*1024*1024)}for name in SOURCES]

def zone_pins():
    # Only these fixed nonpersonal system resources, never arbitrary TZ/TZDIR.
    root=Path('/usr/share/zoneinfo')
    result={}
    for name in sorted(ZONES):
        original=root/name
        actual=original.resolve(strict=True)
        require(actual.is_relative_to(root.resolve()) and actual.is_file(),
                'A supplied reference zone is unavailable; no dependency will be installed')
        result[name]={'resolved':str(actual),**pin(actual,1024*1024)}
    return result

def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__,allow_abbrev=False)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args(argv)
    inherited=require_guard(ROOT)
    destination=args.output.absolute()
    require(destination.is_relative_to(ROOT/'.build-artifacts')
            and destination.parent.resolve()==destination.parent,'Fresh canonical project-disk output only')
    run=fresh(destination.parent,destination.name)
    raw=read_bytes(ROOT/FIXTURE,128*1024)
    require(hashlib.sha256(raw).hexdigest()==FIXTURE_SHA,'Fixed reference fixture changed')
    fixture=validate_fixture(json.loads(raw));before=source_pins();zones=zone_pins()
    tool_pins={name:pin(ROOT/name,4*1024*1024)for name in TOOLS}
    programs={}
    for name,path in [('probe',ROOT/'target/debug/examples/timezone_reference'),('date',Path('/usr/bin/date'))]:
        info=path.lstat()
        require(stat.S_ISREG(info.st_mode) and info.st_mode&0o111 and not info.st_mode&0o6000,
                'Fixed ordinary unprivileged executable required: '+name)
        programs[name]={'path':str(path),**pin(path,512*1024*1024)}
    output(run/'CONTROL.json',dict(scope='Selected capability inputs, not full compiler closure or independent seal',sources=before,tools=tool_pins,programs=programs,timezone_files=zones,fixture_sha256=FIXTURE_SHA))
    budget=Budget();deadline=time.monotonic()+60;env=closed_environment({},run);results=[]
    with ExitStack()as stack:
        executables={name:stack.enter_context(pinned_executable(Path(record['path']),record,512*1024*1024))for name,record in programs.items()}
        def call(name,args,directory,child_env,cap=65536):
            program,fd=executables[name]
            return execute([program,*args],directory,directory,child_env,budget,(*inherited,fd),deadline,cap)
        observed=call('probe',[],fresh(run,'probe'),env,512*1024)
        require(observed['status']==0 and not observed['stderr'],'Fixed in-process probe failed')
        model=json.loads(observed['stdout'])
        require(model['purpose']=='timezone-capability-reference-v1' and model['manifest_dir']==str(ROOT)
                and model['sources']==before,'Compiled selected inputs differ; rebuild via MAIN runner')
        require([r['id']for r in model['cases']]==list(IDS),'Actual compiled case roster differs')
        for n,(case,expected)in enumerate(zip(fixture['cases'],model['cases'])):
            # A model-unset TZ uses its supplied configured zone. Select that same
            # known zone explicitly here, so no reference reads host /etc/localtime.
            child_env=closed_environment({'TZ':case['configured']if case['tz']is None else case['tz']},run)
            args=['--date=@'+str(case['epoch'])]
            if case['utc']:args.append('-u')
            args.append('+'+case['format'])
            directory=fresh(run,'date-'+str(n));actual=call('date',args,directory,child_env)
            record=dict(id=case['id'],passed=actual['status']==expected['status'] and actual['stdout']==expected['stdout'] and actual['stderr']==expected['stderr'],expected=expected,actual=actual)
            output(directory/'comparison.json',record);results.append(record)
    require(source_pins()==before,'Selected source changed during comparison')
    require(zone_pins()==zones,'Reference zone files changed')
    require({name:pin(ROOT/name,4*1024*1024)for name in TOOLS}==tool_pins,'Reference harness changed')
    report=dict(status='PASS'if all(r['passed']for r in results)else'MISMATCHES',checks=len(results),passed=sum(r['passed']for r in results),skipped=[],results=results,limits='Fixed2026date display/format/override comparisons only. Unset-model TZ compared with its explicit supplied zone; no use/change of host clock or localtime as fixture inputs, actual systemd daemon comparison, full compiler closure, independent seal or learner execution.')
    output(run/'REPORT.json',report);print(json.dumps({k:v for k,v in report.items()if k!='results'},indent=2))
    require(report['status']=='PASS','Actual reference mismatches retained; fix before authoring')
    return 0

if __name__=='__main__':
    raise SystemExit(main())
