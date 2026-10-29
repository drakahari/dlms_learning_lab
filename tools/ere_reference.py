#!/usr/bin/env python3
"""Fixed development ERE/GNU comparison; MAIN runner only, never learner input.

prepare hashes only; capture needs independently approved source and ELF pins.
No pattern, source, worktree, path, binary, fixture or output selector exists.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import time
import case_bash_oracle as safe
from automation_before10_oracle import pinned_executable
from build_safety import owned_directory

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = 'tests/fixtures/ere-fixed-reference.json'
PROBE_SOURCE = 'examples/ere_reference.rs'
PROBE = 'target/debug/examples/ere_reference'
GREP = Path('/usr/bin/grep')
MANUAL = Path('/usr/share/man/man1/grep.1.gz')
BASE = '.build-artifacts/reviews/ere-reference/captures'
CAPTURE = 'ere34-r2'
CAP = 2 * 1024 * 1024
BINARY_CAP = 256 * 1024 * 1024
EVIDENCE_CAP = 512 * 1024 * 1024
VM_STREAM = 512 * 1024
STREAM = 16 * 1024
DEADLINE = 120.0
COMPILED = ('src/ere.rs', PROBE_SOURCE, 'examples/support/automation_digest.rs', FIXTURE, 'Cargo.toml', 'Cargo.lock')
HELPERS = ('tools/case_bash_oracle.py', 'tools/automation_before10_oracle.py', 'tools/build_safety.py', 'tools/dev.py')
SOURCES = COMPILED + ('tests/ere.rs', 'docs/ERE_SUBSET.md', 'tools/ere_reference.py', 'tools/tests/test_ere_reference.py', 'docs/ERE_REFERENCE.md', '.cargo/config.toml') + HELPERS
CORE_PINS = {'docs/ERE_SUBSET.md': '57e80fdf91228bd1e83bbb29a6e816cd90277a9ec6df81ce286da332967c1b6a', 'src/ere.rs': '42bc6a56292d333870ca952bf4d6f0a12683fa53a5e15e1710af79dc0c048d53', 'tests/ere.rs': '2e7c8d46249465f2538fdf3b19dca0d429400d5821026a1646d56e92391ba098', 'tests/fixtures/ere-fixed-reference.json': 'c5b0fd3e58698846ab174f6f6ed1d0309c4e164702465a3bfbc5efa9b5ad888b'}
ROW_PINS = {'literal-search': '86dddb4158edf6c6eb88033842805d735a12da635e87c408be70922b81fbdd3b', 'start-not-middle': 'be585a5f8893c6e858b5aa606df66fff09779ef8b268b6411dddc9e0bd6f7a5a', 'end-not-prefix': 'a0e39ca28d86c9e7762b1a635e42b8923cd2bb4a7f31980ac55ff1dc7cfd246f', 'severity-group': '1488f9f0a2577fdfedcfbda02bc17bd236b86442a42b20aed894b9c79e1e68df', 'group-precedence': '6f2dc77806b3a0807d558bf0540ac7e75d723c658d40a3bbb6d0141d6e47ed5d', 'escaped-dot': 'c3e21a22249bb1a0811c4de34cbf17b3c3595e7d2adbf91083f5d76fe3f199fe', 'range-positive': 'a141de61bcc0fef637ea82af6c7ae9998b31bfff70a7fc916df1c46bfbd8ef45', 'range-negative': '58b6b805e61184ca09d92c90dd4c956d7958432d7fef0bccfa003a67980fc7c2', 'negative-high-byte': 'dd5884d7520a4c5befff02df99fa28990796ccbd93a0034f21a7a8f3a20d7acd', 'digit-class': '9178af9d311103ea6fcf6995de511866c6834c11c1aac75632d5a78d304acae8', 'blank-class': '1d7ab0797a79f8758ec2d4563317b79bfb98c22d618343ce3b7ccb9f2b318df5', 'space-vertical-tab': 'e30a6f10acc4032ee17845ad12f77bdae1e9e3435c04a453a53756b134456e4c', 'c-alpha-not-unicode': '3ca562440d0271601586af38be5ea265af34d7be52850a4fae65029c4af0ab8f', 'c-byte-two': 'ee852e7ebed8a0d6675f10876487378064480061569cc82ff045eaa2ea2d726d', 'c-byte-one': '05e470619407e433628ee6f236a60a611e9e73ae721e112b079859f33d43f90d', 'empty-whole-pattern': '18e19bf2a00516688040cf1aa2d20acbe70215aa352cdc398e53a64b9594eb0b', 'empty-line': 'c8dfdba9dbce7b92481798a67e40b9dc069dbf987a8652171e6b69916092b9f5', 'nonempty-line': '8775bd40e2faed3e7597fbf5d72a237cc4a701b9d3c35583c19f1c86970af4b5', 'optional': 'c79d87f4488321ade80b3ab73295874dc11fa66cff947512fd9b83b24cbcb146', 'plus-nonempty': '746945be22550194bc7c22c97eae948ade055e5c6139a90985e333ed7d2dd1b5', 'star-empty': '901942b8e8fd1bfadfd194205d7e558dbc76be12fcfec7cadc67928cf7f25af3', 'exact-interval': 'ea971d259f8ed5be1f7a14bf7ba312877df09e3dba1d87fd6bdb9060a9e785ec', 'bounded-interval': '6f03811a3ea1b827e2b09e0fd1b7941fdf7693eb5c463c2eb25c2339122eb6c5', 'open-interval': '7153bb6ed2b666f375428f26ddf43fb09753701f240b880b6c5ee508c4b1339d', 'leading-zero-count': '8ce6bcaa16d89fbf78a68296029f8dd3c42882ee776137bef62c422409969e1f', 'nullable-nested-positive': 'e8e66b050f7312dac8fbef3b77f5a30da22d58411be93b1b20f1952078bca0c2', 'nullable-nested-negative': '44f847ff9d9a2a78bc3689e2bb14847bbd30d58535dc50bec611bb37f27d41ef', 'literal-dot-class': '2ced706070f717007f1872c6b803968b8501e92db108279427d7a0d3c7b1ba20', 'literal-first-close': '15e5978ebeefbbaa1fd8ddb5c11479e15647b629eb3e10f65e486e8c49852e36', 'literal-last-hyphen': '010dd2c92ea134728b6942cc07b57660a9a2ff6ed421409c8143076b79ca610e', 'literal-open-bracket': '0c46a894c0456114ffcddc3b1d5f59b2d7b2b5b6c57f40d881d29727682d7368', 'carriage-return-preserved': 'e0235613850668605bd07ea010188bcc6e204eba23c60a54edc0343a3a929313', 'unicode-line-separator-bytes': '5c4ec0b924531eb9aa02cd7541f704f6277639ff15f94cf7f821fa43364aa160', 'literal-command-looking-data': 'bee027d1f4f3a788eeb6f2c48f3bc0e83b17c41a57565cadf64f75736e95827c', 'backref': '7829aba84371ea3bebcf8cf009b5c8e40a09467b0d7e14bee4953643c656b865', 'lookaround': 'cb49aff68a940783e925205c3489e97bd5944657f3c4070969965700837b2d12', 'unicode-pattern': 'f0c09df56a6fae0fd84f5dbea87ade6fc64609cbc4289aed88947258351f2dfa', 'ordinary-escape': '84fd3d08481e335a0bdb34d3dfae1d42663599102f088a7c908203693f7c154b', 'collation': '5953596e0496941ddb7305826094fd89b24646241b1d8cdf9a4cba56cdb9a8f8', 'anchor-middle': '4bfda1393a7aa93416c23d6027c993c22b94fad05f5f0519317ff525daae2090', 'repeat-overbound': 'b140fd7deb957e10497e8bc16e15e9bff6433ff667b75cb3bcdfca1ea44625b7', 'reversed-range': 'f97d1d9dad4f03aebe434b1c31d1ab7f96f275b6b94d30d07deb0c6e3df553f3'}
CASE_IDS = ('literal-search', 'start-not-middle', 'end-not-prefix', 'severity-group', 'group-precedence', 'escaped-dot', 'range-positive', 'range-negative', 'negative-high-byte', 'digit-class', 'blank-class', 'space-vertical-tab', 'c-alpha-not-unicode', 'c-byte-two', 'c-byte-one', 'empty-whole-pattern', 'empty-line', 'nonempty-line', 'optional', 'plus-nonempty', 'star-empty', 'exact-interval', 'bounded-interval', 'open-interval', 'leading-zero-count', 'nullable-nested-positive', 'nullable-nested-negative', 'literal-dot-class', 'literal-first-close', 'literal-last-hyphen', 'literal-open-bracket', 'carriage-return-preserved', 'unicode-line-separator-bytes', 'literal-command-looking-data', 'backref', 'lookaround', 'unicode-pattern', 'ordinary-escape', 'collation', 'anchor-middle', 'repeat-overbound', 'reversed-range')
PROBE_SOURCE_SHA = 'e1f60fe8b9a78444da4cb10d9e5ffa97a9d90603d1941e4b50850bdd7e970626'


def digest(raw): return hashlib.sha256(raw).hexdigest()
def canonical(value): return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(',', ':')).encode()


def strict(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            safe.require(key not in result, 'Duplicate JSON key')
            result[key] = value
        return result
    return json.loads(raw, object_pairs_hook=pairs)


def check_row(row):
    safe.require(isinstance(row, dict) and row.get('id') in ROW_PINS, 'Fixed ERE row only')
    safe.require(digest(canonical(row)) == ROW_PINS[row['id']], 'Exact fixed pattern/input/prediction required')
    safe.require(row['vm_observation'] is None and row['gnu_observation'] is None, 'Predictions remain null')


def load_fixed(root):
    raw = safe.read_bytes(root / FIXTURE, CAP)
    safe.require(digest(raw) == CORE_PINS[FIXTURE], 'Fixed42 fixture changed')
    value = strict(raw)
    safe.require(set(value) == {'schema_version','scope','host_reference_policy','cases','actual_comparison'}
                 and value['schema_version'] == 1 and value['actual_comparison'] is None, 'Closed prediction fixture')
    rows = value['cases']
    safe.require(isinstance(rows, list) and tuple(r['id'] for r in rows) == CASE_IDS, 'Exact42 rows and order')
    for row in rows: check_row(row)
    safe.require(sum(r['partition'] == 'supported' for r in rows) == 34, 'Exactly34 GNU rows')
    return rows


def root_guard(root):
    safe.require((root / '.git').is_dir() and not (root / '.git').is_symlink(), 'MAIN checkout only')
    inherited = safe.require_guard(root)
    safe.require(os.environ.get('CARGO_TARGET_DIR') == str(root / 'target'), 'MAIN exclusive default target only')
    return inherited


def source_pins(root):
    for name, expected in CORE_PINS.items():
        safe.require(safe.pin(root / name, CAP)['sha256'] == expected, 'Reviewed pure foundation changed: ' + name)
    safe.require(safe.pin(root / PROBE_SOURCE, CAP)['sha256'] == PROBE_SOURCE_SHA, 'Reviewed fixed probe source changed')
    paths = [(root / name, CAP) for name in SOURCES]
    toolchain = root / '.build-artifacts/toolchains/rustup/toolchains/stable-x86_64-unknown-linux-gnu'
    paths += [(toolchain / 'bin/rustc', 64*CAP),
              (toolchain / 'lib/rustlib/multirust-channel-manifest.toml', CAP),
              (root / '.build-artifacts/toolchains/READY.json', CAP),
              (Path(sys.executable).resolve(strict=True), 64*CAP),
              (root / PROBE, BINARY_CAP), (GREP, 8*CAP), (MANUAL, CAP)]
    pins = [safe.pin(path, cap) for path, cap in paths]
    safe.require(len(pins) == len({p['path'] for p in pins}), 'No repeated source identities')
    return pins


def control(root, pins):
    return {'schema_version':1, 'capture':CAPTURE, 'root_reviewed':False,
            'main_head':safe.source_identity(root), 'pins':pins,
            'fixture_sha256':CORE_PINS[FIXTURE], 'host_rows':34, 'VM_rows':42,
            'compiled_sources':{name:next(p['sha256'] for p in pins if p['path']==str(root/name)) for name in COMPILED},
            'scope':'Fixed unexported ERE foundation only; no evaluator/utility/curriculum/native acceptance',
            'compiler_scope':'Cached rustc/channel-manifest/READY and Cargo.lock identity, not a claim of reproducible complete toolchain or audited standard-library bytes'}


def approve(raw, expected):
    value = strict(raw)
    safe.require(value.get('root_reviewed') is True and canonical(value) == canonical({**expected,'root_reviewed':True}), 'Exact independent source/ELF approval required')


def parse_vm(raw, root, rows, expected):
    value = strict(raw)
    safe.require(set(value) == {'identity','outcomes'}, 'Closed fixed probe result')
    identity = value['identity']
    safe.require(identity == {'kind':'fixed-ere-reference-r1','manifest_dir':str(root),
                 'fixture_sha256':CORE_PINS[FIXTURE],'cases':42,'host_cases':34,'VM_only':8,
                 'compiled_sources':expected['compiled_sources']}, 'Actual compiled module/probe/fixture/Cargo source identity')
    actual = value['outcomes']
    safe.require(isinstance(actual,list) and len(actual)==42, 'All42 actual VM observations')
    for row, item in zip(rows,actual):
        safe.require(set(item)=={'id','partition','pattern_sha256','line_sha256','matched','error_kind','diagnostic'}
            and item['id']==row['id'] and item['partition']==row['partition']
            and item['pattern_sha256']==digest(row['pattern'].encode())
            and item['line_sha256']==digest(row['line'].encode()), 'Exact row identity/order/input hashes')
        safe.require((type(item['matched']) is bool and item['error_kind'] is None and item['diagnostic'] is None)
                     or (item['matched'] is None and item['error_kind'] in ['Syntax','Unsupported','Limit']
                         and isinstance(item['diagnostic'],str) and 0<len(item['diagnostic'])<=1024), 'Honest boolean or error result')
    return actual


def host_argv(executable, row):
    check_row(row)
    safe.require(row['partition']=='supported', 'Excluded syntax must never reach host reference')
    return [executable,'-E','-q','--',row['pattern']]


def compare(row, vm, gnu):
    check_row(row)
    safe.require(row['partition']=='supported', 'Compare supported row only')
    good_gnu=gnu['status'] in (0,1) and not gnu['stdout'] and not gnu['stderr']
    actual=gnu['status']==0 if good_gnu else None
    passed=good_gnu and vm['error_kind'] is None and type(vm['matched']) is bool and vm['matched']==actual==row['expected_match']
    return {'expected':row['expected_match'],'VM_match':vm['matched'],'GNU_match':actual,'GNU_valid':good_gnu,'passed':passed}


def excluded(row, vm):
    check_row(row)
    safe.require(row['partition']=='excluded', 'VM-only exclusion row')
    return vm['matched'] is None and vm['error_kind']==row['expected_error_kind'] and bool(vm['diagnostic'])


def require_gnu_version(result, executable):
    # GNU derives its displayed program name from argv[0]. The reviewed helper
    # executes the exact held descriptor, so its basename is numeric, not grep.
    safe.require(isinstance(executable, str)
                 and re.fullmatch(r'/proc/self/fd/[0-9]+', executable),
                 'GNU version must identify the held executable descriptor')
    text = result.get('stdout')
    safe.require(type(result.get('status')) is int and result['status'] == 0
                 and result.get('stderr') == '' and isinstance(text, str)
                 and text.endswith('\n')
                 and not any((ord(c) < 32 and c != '\n') or ord(c) == 127 for c in text),
                 'Actual GNU grep version requires successful clean framed output')
    name = executable.rsplit('/', 1)[1]
    safe.require(re.fullmatch(re.escape(name) + r' \(GNU grep\) [0-9]+(?:\.[0-9]+)+',
                              text.split('\n', 1)[0]) is not None,
                 'Actual GNU grep version must match the exact held argv[0] basename')


def environment(work):
    return {'PATH':'/usr/bin:/bin','LC_ALL':'C','LANG':'C','HOME':str(work),
            'TMPDIR':str(work),'TMP':str(work),'TEMP':str(work)}


def run_child(out, name, command, data, budget, inherited, start, *, vm=False):
    safe.require(time.monotonic()-start<DEADLINE,'Whole fixed-reference deadline')
    folder=safe.private_dir(out,name); work=safe.private_dir(folder,'work')
    try:
        result=safe.capture_command(command,folder,work,environment(work),data,budget,
                VM_STREAM if vm else STREAM,timeout=15 if vm else 2,inherited=inherited)
    except Exception as error:
        # The bounded helper retains raw streams but does not return an exit
        # status on timeout/limit failures. Record unknown, never invent one.
        safe.json_new(folder/'STATUS.json',{'status':None,'complete':False,'error':str(error)[:1000]},budget,STREAM)
        raise
    safe.json_new(folder/'STATUS.json',{'status':result['status'],'complete':True,
        'stdout_bytes':len(result['stdout'].encode()),'stderr_bytes':len(result['stderr'].encode()),
        'stdout_sha256':digest(result['stdout'].encode()),'stderr_sha256':digest(result['stderr'].encode())},budget,STREAM)
    safe.require(not any(work.iterdir()),'Reference/probe created unexpected files in disposable cwd')
    return result


def retain_executable(fd, expected, out, name, budget):
    # Copy the exact held executable descriptor, without changing its file offset.
    count=0; observed=hashlib.sha256()
    with safe.new_file(out/name) as stream:
        while count<expected['bytes']:
            block=os.pread(fd,min(1024*1024,expected['bytes']-count),count)
            safe.require(bool(block),'Pinned executable shrank during retention')
            budget.charge(len(block)); stream.write(block); observed.update(block); count+=len(block)
        safe.require(not os.pread(fd,1,count) and observed.hexdigest()==expected['sha256'],'Held executable identity during retention')
        stream.flush(); os.fsync(stream.fileno())
    return safe.pin(out/name,BINARY_CAP)


def retain_sources(root,out,budget):
    directory=safe.private_dir(out,'source')
    for index,name in enumerate(SOURCES):
        # Flat ordinal evidence avoids interpreting even fixed paths as destinations.
        raw=safe.read_bytes(root/name,CAP)
        safe.write_new(directory/f'file{index}.raw',raw,budget,CAP)
    safe.json_new(directory/'INDEX.json',list(SOURCES),budget,CAP)


def run(root, mode):
    inherited=root_guard(root)
    rows=load_fixed(root); pins=source_pins(root); expected=control(root,pins)
    base=owned_directory(root,root/BASE)
    safe.require(not (base/CAPTURE).exists() and not (base/CAPTURE).is_symlink(), 'Fresh capture only; retain prior evidence')
    if mode=='prepare':
        safe.json_new(base/(CAPTURE+'-PROPOSED.json'),expected,safe.Budget(CAP),CAP)
        print(json.dumps({'status':'PROPOSED-NO-CHILD-EXECUTION','source_and_executable_pins':len(pins),'host_rows':34,'VM_rows':42}))
        return 0
    safe.require(mode=='capture','Closed capture mode')
    approve(safe.read_bytes(base/(CAPTURE+'-APPROVED.json'),CAP),expected)
    out=safe.private_dir(base,CAPTURE); budget=safe.Budget(EVIDENCE_CAP); start=time.monotonic()
    try:
        probe=root/PROBE
        ppin=next(p for p in pins if p['path']==str(probe)); gpin=next(p for p in pins if p['path']==str(GREP))
        with pinned_executable(probe,ppin,BINARY_CAP) as (vm,vfd), pinned_executable(GREP,gpin,8*CAP) as (gnu,gfd):
            held=(*inherited,vfd,gfd)
            identity=run_child(out,'probe-identity',[vm,'--identity'],b'',budget,held,start,vm=True)
            safe.require(identity['status']==0 and not identity['stderr'],'Fixed probe identity process failed')
            safe.require(strict(identity['stdout'])=={'kind':'fixed-ere-reference-r1','manifest_dir':str(root),
                'fixture_sha256':CORE_PINS[FIXTURE],'cases':42,'host_cases':34,'VM_only':8,
                'compiled_sources':expected['compiled_sources']},'Compiled identity differs before any GNU cases')
            observation=run_child(out,'probe-outcomes',[vm],b'',budget,held,start,vm=True)
            safe.require(observation['status']==0 and not observation['stderr'],'Fixed probe result process failed')
            actual=parse_vm(observation['stdout'],root,rows,expected)
            version=run_child(out,'grep-version',[gnu,'--version'],b'',budget,held,start)
            require_gnu_version(version,gnu)
            comparisons=[]
            for index,(row,item) in enumerate(zip(rows,actual)):
                if row['partition']=='excluded':
                    result={'id':row['id'],'VM':item,'GNU':None,'honest_exclusion':excluded(row,item)}
                else:
                    g=run_child(out,f'case{index}',host_argv(gnu,row),(row['line']+'\n').encode(),budget,held,start)
                    result={'id':row['id'],'VM':item,'GNU':g,'comparison':compare(row,item,g)}
                comparisons.append(result)
                safe.json_new(out/f'row{index}.json',result,budget,CAP)
            probe_artifact=retain_executable(vfd,ppin,out,'probe.elf',budget)
            grep_artifact=retain_executable(gfd,gpin,out,'grep.elf',budget)
        safe.require(time.monotonic()-start<DEADLINE,'Whole reference deadline')
        safe.require(source_pins(root)==pins and safe.source_identity(root)==expected['main_head'],'Sources/compiler/ELF/fixtures changed during capture')
        retain_sources(root,out,budget)
        safe.require(source_pins(root)==pins,'Source drift during evidence retention')
        good=sum(r['comparison']['passed'] for r in comparisons if 'comparison' in r)
        exclusions=sum(r['honest_exclusion'] for r in comparisons if 'honest_exclusion' in r)
        accepted=good==34 and exclusions==8
        manifest={**expected,'root_reviewed':True,'status':'MATCHED34-8-EXCLUSIONS' if accepted else 'COMPLETED-WITH-MISMATCHES',
            'accepted':accepted,'matches':good,'mismatches':34-good,'honest_VM_only':exclusions,
            'grep_version':version['stdout'].splitlines()[0],'observations':comparisons,
            'retained_probe':probe_artifact,'retained_grep':grep_artifact,
            'limits':{'GNU_seconds':2,'probe_seconds':15,'whole_seconds':DEADLINE,'GNU_stream':STREAM,'probe_stream':VM_STREAM,'evidence_bytes':EVIDENCE_CAP},
            'claims':'Only fixed boolean C-byte ERE comparisons; not full POSIX/GNU conformance, shell utility integration, curriculum or platform acceptance'}
        mpin,warning=safe.publish(out,manifest,budget)
    except Exception as error:
        # Failed raw streams/statuses remain. No completion marker is published.
        safe.json_new(out/'FAILED.json',{'accepted':False,'error':str(error)[:2000]},budget,CAP)
        raise
    # COMPLETE is already committed. A reporting/pipe error cannot revoke it.
    print(json.dumps({'status':manifest['status'],'manifest_sha256':mpin['sha256'],'durability_warning':warning}))
    return 0 if accepted else 1


def main(argv=None):
    parser=argparse.ArgumentParser(description=__doc__,allow_abbrev=False)
    parser.add_argument('--mode',required=True,choices=['prepare','capture'])
    args=parser.parse_args(argv)
    return run(ROOT.resolve(strict=True),args.mode)


if __name__=='__main__':
    raise SystemExit(main())
