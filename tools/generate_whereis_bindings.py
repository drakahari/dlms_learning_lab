#!/usr/bin/env python3
"""Fixed MAIN-only whereis source-binding generator; launches no child."""
import argparse
import json
import os
from pathlib import Path
from native_reopen import inherited_lock
ROOT=Path(__file__).resolve().parents[1]
BINDINGS='examples/support/whereis_bindings.rs'
EXTRAS=('Cargo.toml','Cargo.lock','examples/whereis_reference.rs',BINDINGS,'examples/support/whereis_digest.rs','tests/fixtures/whereis-reference-v1.json')
def require(ok,message):
    if not ok:raise ValueError(message)
def render(root):
    names=set(EXTRAS)
    for base in('src','scenarios','assets'):
        for p in(root/base).rglob('*'):
            require(not p.is_symlink(),'No source symlink')
            if p.is_file():names.add(p.relative_to(root).as_posix())
            else:require(p.is_dir(),'Source kind')
    require(len(names)<=1024,'Bound source roster')
    lines=['// Generated fixed current-source bindings. No producer/oracle input.','pub const COMPILED: &[(&str, &[u8])] = &[']
    for p in sorted(names):
        require('\"'not in p and '\\'not in p and '\n'not in p,'Literal source path')
        rel=os.path.relpath(p,'examples/support')
        lines.append('    ('+json.dumps(p,ensure_ascii=False)+', include_bytes!('+json.dumps(rel,ensure_ascii=False)+')),')
    return ('\n'.join(lines)+'\n];\n').encode('utf-8')
def main():
    p=argparse.ArgumentParser();p.add_argument('--check',action='store_true');args=p.parse_args()
    inherited_lock(ROOT)
    require((ROOT/'.git').is_dir()and not(ROOT/'.git').is_symlink()and Path(os.environ.get('CARGO_TARGET_DIR',''))==ROOT/'target','MAIN/default-target only')
    data=render(ROOT);dest=ROOT/BINDINGS
    require(not dest.is_symlink(),'No destination link')
    if args.check:require(dest.read_bytes()==data,'Bindings must be regenerated before build');return
    # Fixed exclusive temporary sibling; never touches historical binding files.
    tmp=dest.with_name(dest.name+'.generated-new')
    with tmp.open('xb')as f:f.write(data);f.flush();os.fsync(f.fileno())
    os.replace(tmp,dest)
if __name__=='__main__':main()
