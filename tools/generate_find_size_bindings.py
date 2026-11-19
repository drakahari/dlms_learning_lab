#!/usr/bin/env python3
"""Fixed compile-time source inventory; no executable or reference is launched."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
BINDING='examples/support/find_size_bindings.rs'
EXTRAS=('Cargo.toml','Cargo.lock','examples/find_size_reference.rs',BINDING,'examples/support/automation_digest.rs','tests/fixtures/find-size-reference-v1.json')
def names():
    paths=set(EXTRAS)
    for base in ('src','scenarios','assets'):
        for p in (ROOT/base).rglob('*'):
            if p.is_symlink():raise ValueError('No source aliases')
            if p.is_file():paths.add(p.relative_to(ROOT).as_posix())
            elif not p.is_dir():raise ValueError('Source kind')
    if len(paths)>1024:raise ValueError('Source inventory ceiling')
    return sorted(paths)
def main():
    rows=['// Fixed original developer inputs; no learner or host runtime data.','pub const INPUTS:&[(&str,&[u8])]=&[']
    for name in names():
        p=Path(name);rel='../../'+p.as_posix();rows.append(f'    ("{name}",include_bytes!("{rel}")),')
    rows.append('];');(ROOT/BINDING).write_text('\n'.join(rows)+'\n');print('Fixed size probe inputs:',len(names()))
if __name__=='__main__':main()
