#!/usr/bin/env python3
"""Fixed genuine17 development evidence. No script, profile or engine selector.

Prepare launches nothing. Capture requires an independently reviewed exact control.
Verify reads retained evidence only; it never replays an old evaluator.
"""
import argparse
from collections import Counter
import copy
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess

from native_reopen import inherited_lock, read_bytes, ordinary, pin as file_pin
from automation_before10_oracle import pinned_executable

ROOT = Path(__file__).resolve().parents[1]
DESTINATION = '.build-artifacts/reviews/bash-file-lifecycle/captures/public17-269-573-r2'
REVIEW = '.build-artifacts/reviews/bash-file-lifecycle/before18-r2-control.json'
APPROVAL = '.build-artifacts/reviews/bash-file-lifecycle/before18-r2-approved.json'
ATTESTATION = '.build-artifacts/reviews/bash-file-lifecycle/before18-r2-attestation.json'
RETAINED = '.build-artifacts/reviews/bash-file-lifecycle/retained17-r2'
BINARY = 'target/debug/examples/scripting_before18_capture'
BINARY_CAP = 1024 * 1024 * 1024
INPUTS = ['Cargo.toml', 'Cargo.lock', '.cargo/config.toml', 'tools/dev.py',
          'examples/scripting_before18_capture.rs', 'examples/support/before18_inputs.rs',
          'tests/fixtures/bash-before18-source-index.json', 'tools/scripting_before18_verify.py',
          'tools/tests/test_scripting_before18_verify.py', 'tools/native_reopen.py',
          'tools/automation_before10_oracle.py', 'tools/case_bash_oracle.py']
SEALED_ATTESTATION = '3c51aea64b39fab23995b12b3197b62164c1e22e510cbc7ed91ea573086cbf19'  # Actual17 R2, independently read back; never replay after advance.


def require(condition, message):
    if not condition:
        raise ValueError(message)


def strict(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            require(key not in result, 'Duplicate JSON key')
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs)


def contained(root, name):
    require(isinstance(name, str) and name and not name.startswith('/')
            and all(p not in ['', '.', '..'] for p in name.split('/')), 'Closed relative path required')
    at = root
    for part in name.split('/'):
        at = at / part
        require(not at.is_symlink(), 'Symlink evidence path refused')
    return at


def pin(root, name, cap=16_000_000):
    path = contained(root, name)
    info = path.lstat()
    cargo_links = name == BINARY and info.st_nlink == 2
    if cargo_links:
        siblings = [
            p for p in path.parent.iterdir() if re.fullmatch(re.escape(path.name) + r'-[0-9a-f]{16}', p.name)
            and p.lstat().st_dev == info.st_dev and p.lstat().st_ino == info.st_ino]
        require(len(siblings) == 1 and not siblings[0].is_symlink(), 'Unexpected Cargo executable alias')
    require(stat.S_ISREG(info.st_mode) and (info.st_nlink == 1 or cargo_links) and info.st_size <= cap, 'Evidence file type/size refused')
    observed = file_pin(path, cap)
    return {'path': name, 'size': observed['bytes'], 'sha256': observed['sha256']}


def current17(root):
    raw = read_bytes(root / 'src/shell.rs', 512 * 1024)
    require(re.findall(rb'^pub const EVALUATOR_REVISION: u32 = ([0-9]+);$', raw, re.M) == [b'17'], 'Capture requires genuine current17')


def control(root):
    current17(root)
    inputs = list(INPUTS)
    text = (root / 'examples/support/before18_inputs.rs').read_text()
    names = re.findall(r'\(\s*"([^"\n]+)",\s*include_bytes!', text, re.M)
    index = strict(read_bytes(root / 'tests/fixtures/bash-before18-source-index.json', 16_000_000))
    require(names == index['compiled'] and len(names) == 535 and len(set(names)) == 535, 'Fixed compiled closure changed')
    inputs += names
    rows = [pin(root, name) for name in sorted(set(inputs))]
    return {'actual_evaluator_revision': 17, 'root_reviewed': False,
            'pins': rows, 'producer': pin(root, BINARY, BINARY_CAP),
            'destination': DESTINATION}


def approved(raw, expected):
    actual = strict(raw)
    require(actual.get('root_reviewed') is True, 'Independent exact-pin review required')
    require(actual == {**expected, 'root_reviewed': True}, 'Review does not match actual source/producer')


def outcome(case, observed):
    return (isinstance(observed, dict) and set(observed) == {'stdout', 'stderr', 'status', 'files', 'error'}
            and type(observed['status']) is int and observed['error'] is None
            and observed['stdout'] == case['stdout'] and observed['stderr'] == case['stderr']
            and observed['status'] == case['status'] and observed['files'] == case['after'])


def records(profile):
    result = Counter()
    for a in profile['attempts'].values():
        state, workspace = a['state'], a.get('workspace') or {}
        items = ([state['run']] if state.get('kind') == 'Script' and state.get('run') else [])
        items += ([workspace['script_history']] if workspace.get('script_history') else [])
        items += workspace.get('script_history_more', [])
        result.update(json.dumps(x, sort_keys=True, separators=(',', ':')) for x in items)
    return result


def profile_checks(old, new, rows, drafts):
    require(new['favorites'] == old['favorites'], 'Favorites changed')
    expected_records = records(old)
    expected_ids = set(old['attempts'])
    for row in rows:
        name = row['id']; before = old['attempts'].get(name)
        require(before is not None, 'No new Script attempts belong in inherited profiles')
        if drafts and name in ['shell-command-plan', 'shell-handover']:
            require(before is not None and new['attempts'][name] == before, 'Unfinished draft changed')
            continue
        actual = new['attempts'][name]
        require(actual['state'] == dict(kind='Script', source=row['source'], run=row['record']), 'Current reference differs')
        learning = actual['workspace']['script_learning']
        require(learning['example_source'] == row['example_source'] and learning['example_revision'] == 17
                and learning['example_result'] == row['example_outcome'], 'Example context differs')
        expected_records[json.dumps(row['record'], sort_keys=True, separators=(',', ':'))] += 1
        if before is not None:
            def metadata(a):
                a = copy.deepcopy(a); a.pop('state')
                w = a.get('workspace') or {}
                for key in ['script_history', 'script_history_more', 'checked']:
                    w.pop(key, None)
                if w.get('script_learning'):
                    w['script_learning'].pop('example_revision', None)
                    w['script_learning'].pop('example_result', None)
                return a
            require(metadata(actual) == metadata(before), 'Learner metadata changed')
    require(set(new['attempts']) == expected_ids and len(expected_ids) == 158, 'Invented or missing attempt')
    for name, before in old['attempts'].items():
        if before['state']['kind'] != 'Script':
            require(new['attempts'][name] == before, 'Unrelated progress changed')
    require(records(new) == expected_records, 'Historical records dropped/relabeled/invented')


def inventory(root):
    folder = contained(root, DESTINATION)
    files = sorted(p for p in folder.rglob('*') if p.is_file())
    require(len(files) == 18, 'Incomplete/unexpected fixed capture inventory')
    total = 0
    pins = []
    for path in files:
        row = pin(root, path.relative_to(root).as_posix())
        total += row['size']; require(total <= 256 * 1024 * 1024, 'Capture total limit')
        pins.append(row)
    return pins


def canonical_hash(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode()).hexdigest()


def inherited(snapshot):
    require(isinstance(snapshot,dict) and set(snapshot)=={'parent','child'},'Typed variable snapshot shape')
    require(isinstance(snapshot['parent'],dict)and isinstance(snapshot['child'],dict),'Typed variable maps')
    for name,value in snapshot['parent'].items():
        require(isinstance(name,str)and isinstance(value,dict)and set(value)=={'value','exported'}
                and type(value['exported'])is bool and (value['value']is None or isinstance(value['value'],str)), 'Typed variable attributes')
    return {name:value['value']for name,value in snapshot['parent'].items()if value['exported']and value['value']is not None}


def parent_values(snapshot,expected):
    return all(snapshot['parent'].get(name,{'value':None,'exported':False})==value for name,value in expected.items())


def environment_case(case,row):
    require(isinstance(row,dict)and set(row)=={'parent','bindings','child','diagnostic'},'Typed environment result shape')
    snapshot=row['bindings']
    if snapshot is None:return False
    child=inherited(snapshot)
    return (outcome(case['input'],row['parent'])and snapshot['child']==child and parent_values(snapshot,case['parent'])
            and snapshot['child']==case['child']['env']and outcome(case['child'],row['child']))


def selection(case):
    present=set(case['present']);mode=case['invocation'];kind=mode['kind']
    if kind=='InteractiveLogin':
        if mode['noprofile']:return []
        return (['/etc/profile']if case.get('system_profile')is not None else[])+next(([name]for name in ['~/.bash_profile','~/.bash_login','~/.profile']if name in present),[])
    if kind=='InteractiveNonLogin':return ['~/.bashrc']if not mode['norc']and '~/.bashrc'in present else[]
    require(kind=='NonInteractive','Unknown frozen invocation')
    return [mode['bash_env']]if mode.get('bash_env')and mode['bash_env']in present else[]


def startup_case(case,row):
    require(isinstance(row,dict)and set(row)=={'id','label','checks','parent','child','error','passed'},'Typed startup observation shape')
    require(row['id']==case['id']and row['label']==case['label']and type(row['passed'])is bool,'Typed startup case binding')
    require(isinstance(row['checks'],list)and row['checks']and all(isinstance(c,list)and len(c)==2 and type(c[0])is bool and isinstance(c[1],str)for c in row['checks']),'Typed startup objective results')
    parent=row['parent'];passed=False
    if parent is not None:
        require(set(parent)=={'selection','output','variables','command_finished'}and type(parent['command_finished'])is bool,'Typed startup parent shape')
        exported=inherited(parent['variables'])
        passed=(row['error']is None and parent['command_finished']and parent['selection']==selection(case)
                and outcome(case['input'],parent['output'])and parent['variables']['child']==exported
                and parent_values(parent['variables'],case['parent'])and outcome(case['child'],row['child']))
    require(row['passed']==passed and all(c[0]for c in row['checks'])==passed,'Startup flags contradict actual values/outputs')
    return passed


def typed_results(row,expected):
    goal=row['goal'];require(goal==expected['goal']and row['goal_sha256']==canonical_hash(goal)and ('goal_sha256'not in expected or row['goal_sha256']==expected['goal_sha256']),'Typed goal/source binding differs')
    state=row['state'];kind=goal['kind'];require(state['kind']==kind,'Typed state kind differs')
    obs=row['observations'];phase=row['phase'];definition=goal['definition'];loaded='loaded-unverified'in phase
    if kind=='Environment':
        require(state['simulation']['definition']==definition,'Typed environment definition differs')
        require(set(obs)=={'assignment','example','assignment_verified_in_process','example_verified_in_process','assignment_checks','example_checks'},'Environment observation keys')
        results=[]
        for role,task in [('assignment',definition['assignment']),('example',definition['example'])]:
            checks=obs[role+'_checks']
            require(isinstance(checks,list)and checks and all(isinstance(c,list)and len(c)==2 and type(c[0])is bool and isinstance(c[1],str)for c in checks),'Typed environment objective results')
            record=obs[role];verified=obs[role+'_verified_in_process'];require(type(verified)is bool,'Runtime verification flag type')
            require(record==state['simulation'][role]['result'],'Environment record differs from serialized state')
            if record is None:
                require(not verified,'Unexecuted record claimed verification');results.append(False);continue
            require(record['revision']==17 and record['task']==task and record['source']==state['simulation'][role]['source'],'Environment task/source/revision differs')
            require(len(record['results'])==len(task['cases']),'Environment cases missing')
            actual=[environment_case(c,r)for c,r in zip(task['cases'],record['results'])]
            if verified:require([r[0]for r in obs[role+'_checks']]==actual,'Environment checks contradict actual parent/child results')
            results.append(verified and all(actual))
        if loaded:require(not obs['assignment_verified_in_process']and not obs['example_verified_in_process'],'Imported verification token invented')
    else:
        require(kind=='StartupFiles','Closed typed activity kind')
        require(state['simulation']['origin']==definition['initial']['id']and state['simulation']['version']==1,'Typed startup source/version differs')
        require(set(obs)=={'assignment','example','assignment_pending','example_pending','assignment_checks'},'Startup observation keys')
        results=[]
        for role,task in [('assignment',definition['initial']),('example',definition['example'])]:
            cases=obs[role]
            if cases is None:results.append(False);continue
            require(len(cases)==len(task['cases']),'Startup cases missing')
            results.append(all([startup_case(c,r)for c,r in zip(task['cases'],cases)]))
        if loaded:
            require(obs['assignment']is None and obs['example']is None,'Serialized startup proof/case rows invented')
            if phase.startswith('passed'):require(obs['assignment_pending']and obs['example_pending'],'Saved startup test must request real replay')
    require(type(row['current_solved'])is bool and row['current_solved']==results[0],'Typed overall result contradicts actual evidence')
    if phase in ['passed','passed-loaded-reverified']:require(all(results),'Typed reference or restored replay failed')
    elif phase=='worked-example-only':require(not results[0]and results[1],'Practice is not isolated from assignment')
    else:require(not results[0],'Starter/draft/import was credited without actual correct execution')


def validate_typed(root,folder,index,manifest):
    data=strict(read_bytes(folder/'TYPED-OBSERVATIONS.json',16_000_000));require(data['actual_evaluator_revision']==17,'Typed evaluator differs')
    expected=index['typed'];ids=[r['id']for r in expected];require(len(ids)==4 and len(set(ids))==4,'Typed fixed roster differs')
    reports=[dict(role=role,contexts=4,draft_bytes_exact=True,reload_current_credit_before_execution=False,reverified=role=='passed',repeat_save_stable=True)for role in ['passed','drafts']]
    require(data['reports']==reports and manifest['typed_profile']==dict(contexts_per_profile=4,reports=reports,observations=32),'Typed preservation summary differs')
    order=[]
    for drafts in [False,True]:
        for id in ids:
            order.append((id,'starter'))
            if not drafts:order.append((id,'worked-example-only'))
            order.append((id,'drafts'if drafts else'passed'))
        for id in ids:
            order.append((id,('drafts'if drafts else'passed')+'-loaded-unverified'))
            if not drafts:order.append((id,'passed-loaded-reverified'))
    rows=data['observations'];require([(r['id'],r['phase'])for r in rows]==order,'Typed observation order/coverage differs')
    for row in rows:
        e=next(e for e in expected if e['id']==row['id']);require(row['scenario_version']==e['version'],'Typed source version differs');typed_results(row,e)
    for role in ['passed','drafts']:
        raw=read_bytes(folder/('writer17-typed-'+role+'.json'),16_000_000)
        require(raw==read_bytes(folder/('typed-'+role)/'progress-v1.json',16_000_000),'Typed repeat-save bytes differ')
        p=strict(raw);require(set(p['attempts'])==set(ids)and p['favorites']==[ids[0]],'Typed profile roster/favorites differ')
        for id in ids:
            actual=p['attempts'][id]['state']
            phases=[role,role+'-loaded-unverified']+(['passed-loaded-reverified']if role=='passed'else[])
            require(all(next(r for r in rows if r['id']==id and r['phase']==phase)['state']==actual for phase in phases),'Typed drafts/observations changed on reload')
            if role=='drafts':
                sim=actual['simulation']
                if actual['kind']=='Environment':
                    require(sim['assignment']['source']=="# unfinished assignment\nSITE='"and sim['example']['source']=="# different practice\nLOCATION='",'Unfinished environment drafts changed')
                else:
                    require(sim['assignment']['files'][sim['assignment']['selected']]['text']=="# unfinished assignment\nAPP_NAME='"and sim['practice']['files'][sim['practice']['selected']]['text']=="# different practice\nTOOL_LABEL='",'Unfinished startup drafts changed')


def validate(root):
    folder = contained(root, DESTINATION)
    manifest_raw = read_bytes(folder / 'MANIFEST.json', 16_000_000)
    manifest = strict(manifest_raw)
    require(read_bytes(folder / 'COMPLETE', 128).decode() == hashlib.sha256(manifest_raw).hexdigest() + '\n', 'Incomplete capture')
    require({k: manifest[k] for k in ['actual_evaluator_revision', 'catalog_contexts', 'script_records', 'authored_cases', 'examples', 'observations']}
            == dict(actual_evaluator_revision=17, catalog_contexts=269, script_records=68, authored_cases=573, examples=68, observations=641), 'Capture roster changed')
    rows = [strict(x) for x in read_bytes(folder / 'actual17-authored.jsonl', 16_000_000).splitlines()]
    obs = [strict(x) for x in read_bytes(folder / 'VM-OBSERVATIONS.jsonl', 16_000_000).splitlines()]
    require(len(rows) == 68 and len(obs) == 641, 'Missing actual VM observations')
    index = strict(read_bytes(contained(root, RETAINED + '/tests/fixtures/bash-before18-source-index.json'), 16_000_000))
    source = strict(read_bytes(folder / 'SOURCE.json', 16_000_000))
    require(source['actual_evaluator_revision'] == 17, 'Compiled revision differs')
    index_pin = pin(root, RETAINED + '/tests/fixtures/bash-before18-source-index.json')
    require(source['index_sha256'] == index_pin['sha256'], 'Compiled index differs')
    compiled_names = re.findall(r'\(\s*"([^"\n]+)",\s*include_bytes!',
                               contained(root, RETAINED + '/examples/support/before18_inputs.rs').read_text(), re.M)
    compiled_names += ['examples/scripting_before18_capture.rs', 'examples/support/before18_inputs.rs', 'tests/fixtures/bash-before18-source-index.json']
    require(compiled_names == index['compiled'] + ['examples/scripting_before18_capture.rs', 'examples/support/before18_inputs.rs', 'tests/fixtures/bash-before18-source-index.json'] and len(set(compiled_names)) == 538, 'Compiled binding inventory differs')
    expected_pins = []
    for name in compiled_names:
        row = pin(root, RETAINED + '/' + name)
        expected_pins.append({**row, 'path': name})
    require(source['inputs'] == expected_pins, 'Actual compiled source differs from retained reviewed source')
    require([r['id'] for r in rows] == [r['id'] for r in index['scripts']], 'Authored roster differs')
    cursor = 0
    for row in rows:
        expected = next(x for x in index['scripts'] if x['id'] == row['id'])
        require(all(row[k] == expected[k] for k in ['source', 'cases', 'example_source', 'example_case']), 'Declared fixture differs from compiled index')
        record = row['record']
        require(record['engine_revision'] == 17 and record['source'] == row['source'] and len(record['results']) == len(row['cases']), 'Record context differs')
        for n, (case, result) in enumerate(zip(row['cases'], record['results'])):
            require(outcome(case, result), 'Actual VM assignment failed')
            require(obs[cursor] == dict(id=row['id'], role='assignment', case=n, outcome=result), 'Observation order/content differs'); cursor += 1
        require(outcome(row['example_case'], row['example_outcome']), 'Actual VM example failed')
        require(obs[cursor] == dict(id=row['id'], role='example', case=0, outcome=row['example_outcome']), 'Example observation differs'); cursor += 1
    require(cursor == len(obs), 'Extra VM observations')
    for role, count in [('original', 68), ('draft', 66)]:
        p = manifest[role + '_profile']
        require(p['contexts'] == 158 and p['current_records'] == count and p['non_script_contexts_preserved'] == 90
                and p['all_prior_records_retained'] is True and p['added_script_ids'] == [], 'Profile preservation summary differs')
        stem = 'drafts' if role == 'draft' else 'original'
        old = strict(read_bytes(folder / ('input16-' + stem + '.json'), 16_000_000))
        new = strict(read_bytes(folder / ('writer17-' + stem + '.json'), 16_000_000))
        require(hashlib.sha256(read_bytes(folder / ('input16-' + stem + '.json'), 16_000_000)).hexdigest()
                == index['prior']['writer16-' + stem + '.json'], 'Genuine prior profile differs')
        profile_checks(old, new, rows, role == 'draft')
        profile_folder = folder / ('draft-profile' if role == 'draft' else 'original-profile')
        require(read_bytes(profile_folder / 'progress-v1.json', 16_000_000) == read_bytes(folder / ('writer17-' + stem + '.json'), 16_000_000), 'Inherited repeat-save bytes differ')
        require(read_bytes(profile_folder / 'progress-v1.before-shell-v17.json', 16_000_000) == read_bytes(folder / ('input16-' + stem + '.json'), 16_000_000), 'First genuine16 backup differs')
        require(new['migration_receipts']['shell'] == list(range(7, 18)), 'Migration receipts differ')
    validate_typed(root, folder, index, manifest)
    return inventory(root)


def create(path, data):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open('x') as f:
        json.dump(data, f, indent=2); f.write('\n'); f.flush(); os.fsync(f.fileno())


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--mode', choices=['prepare', 'capture', 'verify'], required=True)
    args = parser.parse_args()
    lock = inherited_lock()
    require((ROOT / '.git').is_dir() and Path(os.environ.get('CARGO_TARGET_DIR', '')) == ROOT / 'target', 'MAIN/default runner only')
    if args.mode == 'verify':
        require(SEALED_ATTESTATION is not None, 'Actual capture has not been independently sealed')
        raw = read_bytes(contained(ROOT, ATTESTATION), 16_000_000)
        require(hashlib.sha256(raw).hexdigest() == SEALED_ATTESTATION, 'Attestation changed')
        sealed = strict(raw)
        require(sealed['outputs'] == validate(ROOT), 'Retained capture bytes changed')
        require(sealed['producer'] == pin(ROOT, sealed['producer']['path'], BINARY_CAP), 'Retained producer changed')
        for row in sealed['retained_inputs']:
            require(row == pin(ROOT, row['path']), 'Retained compiled source/control changed')
        print('PASS retained genuine17 evidence; no replay'); return
    c = control(ROOT)
    if args.mode == 'prepare':
        create(contained(ROOT, REVIEW), c)
        print('Prepared exact genuine17 proposal; no VM executed; independent review required'); return
    approved(read_bytes(contained(ROOT, APPROVAL), 16_000_000), c)
    require(not contained(ROOT, DESTINATION).exists(), 'Preserve existing capture; no overwrite')
    retained = contained(ROOT, RETAINED)
    require(not retained.exists(), 'Preserve retained source and producer')
    retained.mkdir(mode=0o700)
    for row in [*c['pins'], c['producer']]:
        source = contained(ROOT, row['path'])
        dest = contained(retained, row['path']); dest.parent.mkdir(parents=True, exist_ok=True)
        with ordinary(source, BINARY_CAP) as (stream, info), dest.open('xb') as output:
            copied = 0
            while chunk := stream.read(1024 * 1024):
                copied += len(chunk); require(copied <= row['size'], 'Input grew during retention'); output.write(chunk)
            output.flush(); os.fsync(output.fileno())
        require(pin(retained, row['path'], BINARY_CAP) == row, 'Retained source/producer differs')
    env = dict(os.environ, DLMS_FIXED_CAPTURE17='before18-r2')
    expected = dict(path=str(contained(ROOT, BINARY)), bytes=c['producer']['size'], sha256=c['producer']['sha256'])
    with pinned_executable(contained(ROOT, BINARY), expected, BINARY_CAP) as (executable, descriptor):
        subprocess.run([executable], cwd=ROOT, env=env, check=True, timeout=240, pass_fds=(lock, descriptor))
    outputs = validate(ROOT)
    retained_producer = pin(ROOT, RETAINED + '/' + BINARY, BINARY_CAP)
    retained_inputs = [pin(ROOT, RETAINED + '/' + row['path']) for row in c['pins']]
    create(contained(ROOT, ATTESTATION), {'purpose': 'Actual genuine17 execution; independent sealing still required', 'control': c, 'outputs': outputs, 'producer': retained_producer, 'retained_inputs': retained_inputs})
    print('Actual capture validates; retain producer/source and obtain independent seal before semantics advance')


if __name__ == '__main__':
    main()
