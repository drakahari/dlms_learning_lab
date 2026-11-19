#!/usr/bin/env python3
"""Fixed vi development observations only; never accepts learner keys/scripts/paths.

This separate observer does not widen any historical Bash collector. It records
file/cursor effects of fixed private-PTY keystroke recipes, not interactive/physical-keyboard
or full vi conformance. The app never imports or launches this tool.
"""
import argparse
import hashlib
import json
import os
import errno
import fcntl
import pty
import selectors
import struct
import termios
from pathlib import Path
import subprocess
import tempfile
import time
from automation_before10_oracle import pinned_executable
from build_safety import owned_directory
from case_bash_oracle import capture_command, closed_environment
from native_reopen import inherited_lock, pin, read_bytes
from startup_reference import Budget, limits, output
ROOT = Path(__file__).resolve().parents[1]
PROGRAM = {'path': '/usr/bin/vi', 'bytes': 1880176, 'sha256': '4ee6607295ca87040f557ecf098888cb0a4c196c599eca2226a1140ff5125b74'}
CASES = (
    ('empty-write', '', '', 'wq'),
    ('empty-line-write', '\n', '', 'wq'),
    ('delete-last-line', 'abc\n', 'dd', 'wq'),
    ('delete-last-character', 'x\n', 'x', 'wq'),
    ('insert-text', 'abc\n', 'iX\x1b', 'wq'),
    ('insert-without-typing', 'abc\n', 'li\x1b', 'wq'),
    ('append-without-typing', 'abc\n', 'la\x1b', 'wq'),
    ('grouped-undo', 'abc\n', 'iHello\x1bu', 'wq'),
    ('open-empty-buffer', '', 'o\x1b', 'wq'),
    ('discard-edit', 'abc\n', 'iX\x1b', 'q!'),
)
# Additive closed profile: the original ten recipes and R10 capture stay intact.
NAV_CASES = (
    ('vertical-column', 'abcdef\nx\nuvwxyz\n', 'lllljj', 'wq'),
    ('vertical-end', 'abc\nx\nuvwxyz\n', '$jj', 'wq'),
    ('vertical-reset', 'abcdef\nx\nuvwxyz\n', 'llllj0j', 'wq'),
    ('vertical-left-reset', 'abcdef\nx\nuvwxyz\n', 'lllljhj', 'wq'),
    ('jump-first', '  abc\nx\n  last\n', 'Ggg', 'wq'),
    ('jump-last', 'abc\nx\n  last\n', 'G', 'wq'),
    ('copy-after', 'abc\ndef\n', 'yyp', 'wq'),
    ('copy-before', 'abc\ndef\n', 'jyyP', 'wq'),
    ('move-line', 'abc\ndef\n', 'ddp', 'wq'),
    ('restore-only-line', 'abc\n', 'ddp', 'wq'),
    ('copy-empty-line', '\nabc\n', 'yyp', 'wq'),
    ('put-character-after', 'abc\n', 'xp', 'wq'),
    ('put-character-before', 'abc\n', 'xP', 'wq'),
    ('undo-put', 'abc\ndef\n', 'yypu', 'wq'),
    ('put-survives-undo', 'abc\ndef\n', 'ddup', 'wq'),
    ('search-forward', 'red blue\nred green\n', '/red\r', 'wq'),
    ('search-repeat', 'red blue\nred green\n', '/red\rn', 'wq'),
    ('search-backward', 'red blue\nred green\n', 'G?red\r', 'wq'),
    ('search-wrap', 'red blue\nred green\n', '?red\r', 'wq'),
    ('search-overlap', 'aaa\n', '/aa\rn', 'wq'),
    ('search-repeat-empty', 'red blue\nred green\n', '/red\r/\r', 'wq'),
)
INSERT_CASES = (
    ('typed-backspace', 'abc\n', 'iXY\x08\x1b', 'wq'),
    ('backspace-insert-start', 'abc\n', 'li\x08\x1b', 'wq'),
    ('backspace-newline', 'abc\n', 'aX\rY\x08\x08\x1b', 'wq'),
    ('command-backspace', 'abc\n', ':wx\x08\r', 'wq'),
    ('put-into-blank', 'x\n', 'xpp', 'wq'),
    ('line-before-empty', 'abc\n', 'ddP', 'wq'),
    ('open-backspace', 'abc\n', 'oX\x08\x1b', 'wq'),
    ('undo-erased-insert', 'abc\n', 'iX\x08\x1bu', 'wq'),
)
INSERT_REFERENCE = '.build-artifacts/reference-runs/modal-editing/observe-fehloaa7'
INSERT_REFERENCE_SHA = 'b29b989d929e0f4db0a335b6ce1f0703338bf630028a4cceef902e143b3aac65'
NAV_REFERENCE = '.build-artifacts/reference-runs/modal-editing/observe-t_unzmpw'
NAV_REFERENCE_SHA = 'a081d96da3ca138e604f650a6a330913c6bf5bcaea939a78a5c66e7ad17792cc'
KEYS = frozenset(('', 'dd', 'x', 'iX\x1b', 'li\x1b', 'la\x1b', 'iHello\x1bu', 'o\x1b'))
REFERENCE = '.build-artifacts/reference-runs/modal-editing/observe-1l_i24is'
REFERENCE_SHA = 'f8815ade637b4f2688eb3bcfcdeb9d706cc0d55dd68faec2183b2bb34205a20e'
PROBE = 'target/debug/examples/modal_editing_reference'
MODEL_INPUTS = ('src/lib.rs','src/modal_editing.rs','src/modal_text.rs','src/cron.rs',
                'src/cron_workspace.rs','Cargo.toml','Cargo.lock')
PRELUDE = 'set nomodeline noexrc nobackup nowritebackup noswapfile'
INPUTS = ('tools/modal_editing_reference.py', 'tools/tests/test_modal_editing_reference.py',
          'tools/dev.py', 'tools/build_safety.py', 'tools/native_reopen.py',
          'tools/automation_before10_oracle.py', 'tools/case_bash_oracle.py', 'tools/startup_reference.py')
def require(ok, message):
    if not ok: raise ValueError(message)
def recipes(profile="foundation"):
    require(profile in ("foundation","navigation","insertion"), "Closed source-owned profile only")
    if profile == "insertion":
        require(len(INSERT_CASES)==8 and len({r[0] for r in INSERT_CASES})==8,"Exact insertion recipes")
        return INSERT_CASES
    if profile == "navigation":
        require(len(NAV_CASES)==21 and len({r[0] for r in NAV_CASES})==21,"Exact navigation recipes")
        return NAV_CASES
    require(len(CASES) == 10 and len({r[0] for r in CASES}) == 10, 'Exact ten fixed recipes')
    for name, text, keys, finish in CASES:
        require(name and name.replace('-', '').isalpha(), 'Fixed recipe label')
        require(len(text) <= 1024 and '~' not in text and all(c == '\n' or 32 <= ord(c) <= 126 for c in text), 'Inert ASCII fixture text without observer marker')
        require(keys in KEYS and finish in ('wq', 'q!'), 'Only closed normal-key recipes and fixed exit actions')
    return CASES

def script(recipe, marker=False):
    require(recipe in recipes()+recipes("navigation")+recipes("insertion") and type(marker) is bool, 'Only an exact source-owned recipe; no caller commands')
    _, _, keys, finish = recipe
    # Only fixed keyboard recipes enter a private pseudo-terminal. No Ex
    # expression evaluator, host shell, learner source or path selector exists.
    return (keys + ('i~\x1b:wq\r' if marker else ':' + finish + '\r')).encode('ascii')

def cursor_from_marker(raw):
    require(len(raw) <= 16384 and raw.count(b'~') == 1
            and all(b == 10 or 32 <= b <= 126 for b in raw), 'One bounded ASCII cursor marker')
    prefix=raw[:raw.index(b'~')]
    return [prefix.count(b'\n') + 1, len(prefix.rsplit(b'\n',1)[-1]) + 1]

def argv(executable, version=False):
    if version: return [executable, '--version']
    return [executable, '-N', '-u', 'NONE', '-U', 'NONE', '-i', 'NONE', '-n',
            '--noplugin', '--cmd', PRELUDE, 'work.txt']

def control(profile="foundation"):
    require(Path('/usr/bin/vi').resolve(strict=True) == Path('/usr/bin/vi'), 'Reviewed ordinary vi path')
    require(dict(path=PROGRAM['path'], **pin(Path(PROGRAM['path']), 16*1024*1024)) == PROGRAM, 'Reference editor changed; review it first')
    return {'purpose': 'fixed-vi-text-observer-v1', 'program': PROGRAM,
            'tools': {name: pin(ROOT/name, 4*1024*1024) for name in INPUTS},
            'recipes': [{'id': n, 'before': t, 'keys_hex': k.encode('ascii').hex(), 'finish': f} for n,t,k,f in recipes(profile)],
            'scope': 'Fixed private-PTY file/cursor effects only; no model comparison, physical keys, portability or full vi acceptance'}

def terminal_capture(executable, fd, lock, work, data, budget):
    require(len(data)<=2048, 'Fixed bounded keyboard input only')
    master,slave=pty.openpty();process=None;raw=bytearray();offset=0;ready=False
    deadline=time.monotonic()+5
    fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',24,80,0,0))
    try:
        process=subprocess.Popen(argv(executable),cwd=work,
            env=closed_environment({'TERM':'xterm-256color'},work),
            stdin=slave,stdout=slave,stderr=slave,pass_fds=(lock,fd),
            start_new_session=False,preexec_fn=limits)
        os.set_blocking(master,False)
        with selectors.DefaultSelector() as selector:
            selector.register(master,selectors.EVENT_READ)
            while True:
                require(time.monotonic()<deadline, 'Bounded private terminal deadline')
                # Observe raw mode on this freshly-created PTY only; no desktop
                # terminal settings or another process group is touched.
                if not ready:
                    attrs=termios.tcgetattr(master)
                    ready=not(attrs[3]&(termios.ICANON|termios.ECHO))
                    if ready:os.close(slave);slave=-1
                if ready and offset<len(data):
                    try:offset+=os.write(master,data[offset:])
                    except BlockingIOError:pass
                for _,_ in selector.select(.01):
                    try:block=os.read(master,4096)
                    except BlockingIOError:continue
                    except OSError as e:
                        if e.errno!=errno.EIO:raise
                        block=b''
                    require(len(raw)+len(block)<=65536, 'Bounded terminal output')
                    budget.charge(len(block));raw.extend(block)
                if process.poll()is not None:
                    require(ready and offset==len(data),'Actual editor initialization and complete fixed keys')
                    break
        return {'status':process.returncode,'terminal_bytes':len(raw),'fixed_input_bytes':offset}
    finally:
        with(work/'capture/tty.raw').open('xb')as f:f.write(raw)
        if process is not None and process.poll()is None:
            process.terminate()
            try:process.wait(timeout=.5)
            except subprocess.TimeoutExpired:process.kill();process.wait(timeout=.5)
        os.close(master)
        if slave>=0:os.close(slave)

def observe(profile="foundation"):
    lock = inherited_lock()
    require(os.geteuid() != 0, 'Do not run the reference editor as root')
    require(Path.cwd() == ROOT and os.environ.get('CARGO_TARGET_DIR') == str(ROOT/'target'), 'MAIN runner only')
    c = control(profile)
    base = owned_directory(ROOT, ROOT/'.build-artifacts/reference-runs/modal-editing')
    run = Path(tempfile.mkdtemp(prefix='observe-', dir=base))
    output(run/'control.json', c)
    for name in INPUTS:
        target=run/'retained-inputs'/name;target.parent.mkdir(parents=True, exist_ok=True)
        with target.open('xb') as f: f.write(read_bytes(ROOT/name,4*1024*1024))
    rows=[];budget=Budget();began=time.monotonic()
    def launch(executable, fd, work, data, version=False):
        capture=work/'capture';capture.mkdir(mode=0o700)
        if version:
            return capture_command(argv(executable, True), capture, work,
                closed_environment({},work), data, budget, 16*1024, timeout=3,
                inherited=(lock,fd), popen=lambda *a,**kw: subprocess.Popen(*a,preexec_fn=limits,**kw))
        return terminal_capture(executable, fd, lock, work, data, budget)
    with pinned_executable(Path(PROGRAM['path']),PROGRAM,16*1024*1024) as (executable,fd):
        folder=run/'version';folder.mkdir(mode=0o700)
        version=launch(executable,fd,folder,b'',True);output(run/'version.json',version)
        require(version['status']==0 and version['stdout'].startswith('VIM - Vi IMproved'), 'Need actual pinned Vim identity; no result inferred')
        for recipe in recipes(profile):
            require(time.monotonic()-began < 45, 'Whole reference deadline')
            name,text,_,_=recipe;case=run/name;case.mkdir(mode=0o700)
            observed={}
            for phase in ('business','cursor-replay'):
                work=case/phase;work.mkdir(mode=0o700)
                with (work/'work.txt').open('xb') as f:f.write(text.encode('ascii'))
                data=script(recipe, phase=='cursor-replay')
                with (work/'recipe.ex').open('xb') as f:f.write(data)
                result=launch(executable,fd,work,data)
                output(work/'process.json',result)
                require(result['status']==0, 'Reference failed; retain actual terminal diagnostics, do not infer success')
                observed[phase]=read_bytes(work/'work.txt',16384)
                require({p.name for p in work.iterdir()}=={'work.txt','recipe.ex','capture','process.json'}, 'Unexpected editor artifact; retain for inspection')
            rows.append({'id':name,'written':observed['business'].decode('ascii'),
                         'cursor_one_based':cursor_from_marker(observed['cursor-replay']),
                         'cursor_probe':'Separate identical-key replay, then insert one literal tilde at the Normal cursor; not expression evaluation or interactive-mode acceptance',
                         'status':0})
            output(case/'observation.json',rows[-1])
    require(control(profile)==c, 'Source/editor identity changed during observation')
    output(run/'observations.json', {'control':c,'version':version,'observations':rows})
    output(run/'COMPLETE.json', {'count':len(rows),'scope':c['scope']})
    print('OBSERVED'+str(len(rows))+' fixed vi recipes (no model/interactive acceptance): '+str(run))

def reference(profile="foundation"):
    require(profile in ("foundation","navigation","insertion"), "Closed source-owned profile only")
    location,digest=(REFERENCE,REFERENCE_SHA) if profile=="foundation" else ((NAV_REFERENCE,NAV_REFERENCE_SHA) if profile=="navigation" else (INSERT_REFERENCE,INSERT_REFERENCE_SHA))
    require(location is not None and digest is not None,"No actual reference capture recorded for this profile")
    base=ROOT/location
    raw=read_bytes(base/'observations.json',256*1024)
    require(hashlib.sha256(raw).hexdigest()==digest,'Retained actual observation changed or is unavailable; no result inferred')
    document=json.loads(raw)
    require(document['control']['program']==PROGRAM and document['version']['status']==0,'Actual held vi identity/version')
    expected=[{'id':n,'before':t,'keys_hex':k.encode('ascii').hex(),'finish':f}for n,t,k,f in recipes(profile)]
    require(document['control']['recipes']==expected,'Actual observer and current fixed recipes differ')
    for name,row in document['control']['tools'].items():
        require(name in INPUTS,'No unexpected retained observer source')
        require(pin(base/'retained-inputs'/name,4*1024*1024)==row,'Retained observer input changed')
    rows=document['observations'];require(len(rows)==len(recipes(profile)),'Exact ordered actual observations')
    for recipe,row in zip(recipes(profile),rows,strict=True):
        require(row['id']==recipe[0] and row['status']==0,'Actual ordered successful observation')
        folder=base/recipe[0]
        for phase in ('business','cursor-replay'):
            work=folder/phase
            require(read_bytes(work/'recipe.ex',2048)==script(recipe,phase=='cursor-replay'),'Actual fixed key recipe changed')
            process=json.loads(read_bytes(work/'process.json',2048))
            require(process['status']==0 and process['fixed_input_bytes']==len(script(recipe,phase=='cursor-replay')),'Actual complete private-PTY keys/exit')
        require(read_bytes(folder/'business/work.txt',16384).decode('ascii')==row['written'],'Actual written reference bytes changed')
        require(cursor_from_marker(read_bytes(folder/'cursor-replay/work.txt',16384))==row['cursor_one_based'],'Actual separate marker position changed')
    return document

def model_control(profile="foundation"):
    names=MODEL_INPUTS+('examples/modal_editing_reference.rs','examples/support/automation_digest.rs')+INPUTS
    return {'purpose':'fixed-current-modal-pair-v1','inputs':{name:pin(ROOT/name,4*1024*1024)for name in sorted(set(names))},
            'probe':dict(path=str(ROOT/PROBE),**pin(ROOT/PROBE,1024*1024*1024)),
            'reference_sha256':REFERENCE_SHA if profile=='foundation' else (NAV_REFERENCE_SHA if profile=='navigation' else INSERT_REFERENCE_SHA),'scope':f'Selected linked model sources plus fixed probe/recipes versus retained{len(recipes(profile))} actual vi outcomes; not full compiler closure, complete vi, UI or curriculum acceptance'}

def compare(profile="foundation"):
    lock=inherited_lock()
    require(os.geteuid()!=0 and Path.cwd()==ROOT and os.environ.get('CARGO_TARGET_DIR')==str(ROOT/'target'),'Nonroot MAIN runner only')
    actual=reference(profile);control_value=model_control(profile)
    base=owned_directory(ROOT,ROOT/'.build-artifacts/reference-runs/modal-editing')
    run=Path(tempfile.mkdtemp(prefix='compare-',dir=base));output(run/'control.json',control_value)
    capture=run/'capture';capture.mkdir(mode=0o700)
    with pinned_executable(ROOT/PROBE,control_value['probe'],1024*1024*1024)as(executable,fd):
        result=capture_command([executable] if profile=='foundation' else [executable,profile],capture,run,closed_environment({},run),b'',Budget(),65536,timeout=5,inherited=(lock,fd),popen=lambda *a,**kw:subprocess.Popen(*a,preexec_fn=limits,**kw))
    output(run/'process.json',result)
    require(result['status']==0 and not result['stderr'],'Actual model probe failed; diagnostics retained')
    model=json.loads(result['stdout'])
    require(set(model)=={'purpose','selected_linked_inputs','probe_sha256','observations','scope'} and model['purpose']=='fixed-modal-model-probe-v1','Closed actual model protocol')
    linked={r['path']:{'bytes':r['bytes'],'sha256':r['sha256']}for r in model['selected_linked_inputs']}
    require(len(model['selected_linked_inputs'])==len(MODEL_INPUTS) and linked=={name:control_value['inputs'][name]for name in MODEL_INPUTS},'Linked model inputs differ from exact current selected sources')
    require(model['probe_sha256']==control_value['inputs']['examples/modal_editing_reference.rs']['sha256'],'Compiled probe source differs')
    rows=model['observations'];require(len(rows)==len(recipes(profile)),'Exact current cases')
    failures=[]
    for recipe,expected,observed in zip(recipes(profile),actual['observations'],rows,strict=True):
        name,text,keys,finish=recipe
        require(set(observed)=={'id','before','keys_hex','finish','written','cursor_one_based'},'Closed current case fields')
        require((observed['id'],observed['before'],observed['keys_hex'],observed['finish'])==(name,text,keys.encode('ascii').hex(),finish),'Current compiled case differs from reference recipe')
        if (observed['written'],observed['cursor_one_based'])!=(expected['written'],expected['cursor_one_based']):failures.append(name)
    require(model_control(profile)==control_value,'Model/source identity changed during comparison')
    output(run/'report.json',{'scope':control_value['scope'],'reference_sha256':control_value['reference_sha256'],'current':model,'failures':failures})
    require(not failures,'Model mismatches retained: '+','.join(failures))
    print('MATCH'+str(len(rows))+' actual current in-process modal cases: '+str(run))

def main():
    parser=argparse.ArgumentParser(description=__doc__,allow_abbrev=False)
    parser.add_argument('mode',choices=('inspect','observe','compare'))
    parser.add_argument('--profile',choices=('foundation','navigation','insertion'),default='foundation')
    args=parser.parse_args()
    if args.mode=='inspect': print(json.dumps(control(args.profile),indent=2))
    elif args.mode=='observe': observe(args.profile)
    else: compare(args.profile)
if __name__=='__main__': main()
