#!/usr/bin/env python3
"""Copy the existing isolated DLMS toolchain to disk; retain every source artifact."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

from build_safety import workflow, owned_directory


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--cargo-home', type=Path, required=True)
    p.add_argument('--rustup-home', type=Path, required=True)
    args = p.parse_args()
    root = Path(__file__).resolve().parents[1]
    with workflow(root, 'copy-toolchain') as session:
        cache = owned_directory(session.root, session.artifacts / 'toolchains')
        if any((cache / name).exists() for name in ['cargo', 'rustup', 'READY.json']):
            raise RuntimeError('Cache destination already exists; inspect it rather than replace or delete it')
        for source, name in [(args.cargo_home, 'cargo'), (args.rustup_home, 'rustup')]:
            if not source.is_dir() or source.is_symlink():
                raise RuntimeError('Expected an existing ordinary isolated toolchain directory')
            session.checkpoint()
            shutil.copytree(source, cache / name, symlinks=True)
        binary = cache / 'cargo/bin/rustup'
        source_binary = args.cargo_home / 'bin/rustup'
        def digest(path):
            with path.open('rb') as stream:
                return hashlib.file_digest(stream, 'sha256').hexdigest()
        if digest(binary) != digest(source_binary):
            raise RuntimeError('Copied rustup executable differs; sources retained')
        import os
        env = {**os.environ, 'CARGO_HOME': str(cache / 'cargo'), 'RUSTUP_HOME': str(cache / 'rustup')}
        version = subprocess.check_output([str(cache / 'cargo/bin/cargo'), '--version'], env=env, text=True).strip()
        rustc = subprocess.check_output([str(cache / 'cargo/bin/rustc'), '--version'], env=env, text=True).strip()
        session.checkpoint()
        (cache / 'READY.json').write_text(json.dumps({'cargo': version, 'rustc': rustc, 'rustup_sha256': digest(binary), 'source_directories_preserved': True}, indent=2) + '\n')
        print(json.dumps({'cache': str(cache), 'cargo': version, 'rustc': rustc, 'source_directories_preserved': True}, indent=2))


if __name__ == '__main__':
    main()
