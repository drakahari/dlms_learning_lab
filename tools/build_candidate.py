#!/usr/bin/env python3
"""Build a local native milestone installation candidate. Never publishes anything.
Run on each target OS with its native Rust toolchain. Learners need none of these tools.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import re
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
import zipfile

from dependency_notices import collect
from build_safety import workflow, project_root, disk_directory, run as safe_run, checkpoint, capture

ROOT = Path(__file__).resolve().parents[1]


def command(args, **kwargs):
    checkpoint()
    # Metadata is small; the shared monitor still guards each additional phase.
    return capture(args, cwd=ROOT, **kwargs).strip()


def sha256(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            checkpoint()
            digest.update(chunk)
    return digest.hexdigest()


def copy_tracked_tree(relative, destination):
    # Never sweep ignored/untracked profile files or recordings into a candidate.
    paths = subprocess.check_output(['git', 'ls-files', '-z', '--', relative], cwd=ROOT).decode().split('\0')
    for tracked in filter(None, paths):
        checkpoint()
        source = ROOT / tracked
        component = ROOT
        for part in Path(tracked).parts:
            component /= part
            if component.is_symlink():
                raise ValueError('Candidate resources cannot follow source links')
        if not source.is_file() or not source.resolve().is_relative_to(ROOT):
            raise ValueError('Candidate resources must be ordinary contained files')
        target = destination / source.relative_to(ROOT / relative)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)


def compile_native(cargo, target, binary_name, env):
    # Explicit CLI options override Cargo environment and config target settings.
    common = project_root(ROOT)
    build_dir = (common / 'target/candidate-build').resolve()
    if not build_dir.is_relative_to(common):
        raise ValueError('Candidate build target escaped the project volume')
    disk_directory(build_dir)
    env = {k: v for k, v in env.items() if k not in ('CARGO_BUILD_TARGET', 'CARGO_TARGET_DIR')}
    env = {**env, 'CARGO_BUILD_JOBS': '2', 'RUST_TEST_THREADS': '2'}
    safe_run([cargo, 'build', '--jobs', '2', '--release', '--locked', '--offline', '--target', target,
                    '--target-dir', str(build_dir)], cwd=ROOT, env=env)
    artifact = build_dir / target / 'release' / binary_name
    if not artifact.is_file() or artifact.is_symlink():
        raise ValueError('Expected native compiler artifact was not created')
    return artifact


def embedded_content_hashes():
    names = set()
    for filename in ('src/pack.rs', 'src/course.rs'):
        names.update(re.findall(r'include_str!\(\s*"../scenarios/([A-Za-z0-9_.-]+\.json)"\s*\)', (ROOT / filename).read_text()))
    if not names:
        raise ValueError('Embedded content declarations unavailable')
    result = {}
    for name in sorted(names):
        path = ROOT / 'scenarios' / name
        if path.is_symlink() or not path.is_file() or not path.resolve().is_relative_to(ROOT):
            raise ValueError('Embedded content must be ordinary contained data')
        result[name] = sha256(path)
    return result


def build(output, allow_dirty=False):
    with workflow(ROOT, 'candidate'):
        return _build(output, allow_dirty)


def _build(output, allow_dirty=False):
    if platform.system() not in ('Linux', 'Windows', 'Darwin'):
        raise ValueError('Unsupported native build OS')
    revision = command(['git', 'rev-parse', 'HEAD'])
    dirty = bool(command(['git', 'status', '--porcelain']))
    if dirty and not allow_dirty:
        raise ValueError('Commit reviewed source first; use --allow-dirty only for development checks')
    if dirty:
        revision += '-dirty'
    cargo = os.environ.get('CARGO', 'cargo')
    rustc = os.environ.get('RUSTC', 'rustc')
    rust_version = command([rustc, '-Vv'])
    target = next(line.removeprefix('host: ') for line in rust_version.splitlines() if line.startswith('host: '))
    metadata = json.loads(command([cargo, 'metadata', '--locked', '--offline', '--format-version', '1']))
    graph = command([cargo, 'tree', '--locked', '--offline', '--target', target, '--edges', 'normal,build', '--prefix', 'none', '--format', '{p}'])
    dependencies = {tuple(m.groups()) for line in graph.splitlines() if (m := re.match(r'^(\S+) v(\S+)', line))}
    version = tomllib.loads((ROOT / 'Cargo.toml').read_text())['package']['version']
    build_os = platform.platform()
    flavor = 'fedora'
    if platform.system() == 'Linux':
        os_info = platform.freedesktop_os_release()
        flavor = os_info.get('ID', 'linux') + '-' + os_info.get('VERSION_ID', 'unknown')
    elif platform.system() == 'Darwin':
        flavor = 'macos-' + platform.mac_ver()[0]
    else:
        flavor = 'windows'
    flavor = re.sub(r'[^a-zA-Z0-9_.-]', '-', flavor)
    identity = f'dlms-learning-lab-{version}-g{revision[:12]}-{target}-{flavor}-candidate' + ('-dirty' if dirty else '')
    output = Path(output).resolve()
    if not output.is_relative_to(project_root(ROOT)):
        raise ValueError('Candidate output must stay on the project disk-backed volume')
    disk_directory(output)
    output.mkdir(parents=True, exist_ok=True)
    destination = output / identity
    if destination.exists() or (output / (identity + '.tar.gz')).exists() or (output / (identity + '.zip')).exists():
        raise ValueError('Candidate already exists; retain it and choose another output folder')
    env = {**os.environ, 'DLMS_BUILD_REVISION': revision}
    # No profile paths appear in the build identity. Remap source paths in diagnostics.
    remaps = [f'--remap-path-prefix={ROOT}=dlms-learning-lab']
    for var, default in [('CARGO_HOME', Path.home() / '.cargo'), ('RUSTUP_HOME', Path.home() / '.rustup')]:
        location = Path(os.environ.get(var, default)).resolve()
        remaps.append(f'--remap-path-prefix={location}={var.lower()}')
    for index in sorted({str(Path(p['manifest_path']).parent.parent) for p in metadata['packages'] if p['source']}):
        remaps.append(f'--remap-path-prefix={index}=cargo-registry')
    existing_flags = os.environ.get('CARGO_ENCODED_RUSTFLAGS', '').split('\x1f') if os.environ.get('CARGO_ENCODED_RUSTFLAGS') else os.environ.get('RUSTFLAGS', '').split()
    env.pop('RUSTFLAGS', None)
    env['CARGO_ENCODED_RUSTFLAGS'] = '\x1f'.join([*existing_flags, *remaps])
    binary_name = 'dlms-learning-lab.exe' if platform.system() == 'Windows' else 'dlms-learning-lab'
    checkpoint()
    binary = compile_native(cargo, target, binary_name, env)
    # Use fresh staging; a failed notice check cannot leave a candidate labeled complete.
    with tempfile.TemporaryDirectory(prefix='dlms-candidate-', dir=project_root(ROOT) / '.build-artifacts/tmp') as temporary:
        stage = Path(temporary) / identity
        stage.mkdir()
        checkpoint()
        contents = stage
        resources = stage
        if platform.system() == 'Darwin':
            contents = stage / 'DLMS Learning Lab.app/Contents/MacOS'
            resources = stage / 'DLMS Learning Lab.app/Contents/Resources'
            contents.mkdir(parents=True)
            resources.mkdir(parents=True)
            plist = {'CFBundleName': 'DLMS Learning Lab', 'CFBundleDisplayName': 'DLMS Learning Lab',
                     'CFBundleIdentifier': 'org.dlms.LearningLab', 'CFBundleExecutable': binary_name,
                     'CFBundlePackageType': 'APPL', 'CFBundleShortVersionString': version,
                     'CFBundleVersion': version, 'NSHighResolutionCapable': True}
            (contents.parent / 'Info.plist').write_bytes(plistlib.dumps(plist))
            shutil.copyfile(ROOT / 'packaging/macos/INSTALL.txt', stage / 'INSTALL.txt')
        shutil.copy2(binary, contents / binary_name)
        for name in ('README.md', 'LICENSE', 'NOTICE'):
            shutil.copyfile(ROOT / name, resources / name)
        copy_tracked_tree('docs', resources / 'docs')
        copy_tracked_tree('assets/themes', resources / 'theme-notices')
        shutil.copyfile(ROOT / 'packaging/icon.svg', resources / 'icon.svg')
        checkpoint()
        inventory = collect(metadata, resources / 'third-party', dependencies=dependencies, target=target)
        if inventory['unresolved']:
            raise ValueError('Unresolved notice/source obligations: ' + '; '.join(inventory['unresolved']))
        sysroot = Path(command([rustc, '--print', 'sysroot']))
        rust_notice = sysroot / 'share/doc/rust/COPYRIGHT-library.html'
        if not rust_notice.is_file():
            rust_notice = sysroot / 'share/doc/rust/COPYRIGHT.html'
        if not rust_notice.is_file():
            raise ValueError('Rust standard-library notices unavailable; do not label candidate complete')
        shutil.copyfile(rust_notice, resources / 'third-party/RUST-STANDARD-LIBRARY.html')
        if platform.system() == 'Linux':
            for name in ('install.sh', 'uninstall.sh'):
                shutil.copyfile(ROOT / 'packaging/linux' / name, stage / name)
        elif platform.system() == 'Windows':
            shutil.copyfile(ROOT / 'packaging/windows/Install.ps1', stage / 'Install.ps1')
        info = {'version': version, 'revision': revision, 'target': target, 'rustc': rust_version.splitlines()[0],
                'scope': 'development milestone candidate; completion checklist remains open', 'build_os': build_os,
                'signed': False, 'runtime_acceptance': 'not established by building',
                'binary_sha256': sha256(binary), 'dependency_packages': len(inventory['packages']),
                'bundled_content': embedded_content_hashes(),
                'rust_notice_sha256': sha256(rust_notice)}
        (resources / 'BUILD_INFO.json').write_text(json.dumps(info, indent=2) + '\n')
        files = [p for p in stage.rglob('*') if p.is_file()]
        if any(p.is_symlink() for p in stage.rglob('*')):
            raise ValueError('Candidate may contain ordinary resources only')
        manifest = stage / 'INSTALL_FILES.txt'
        manifest.write_text('\n'.join(sorted([p.relative_to(stage).as_posix() for p in files] + ['INSTALL_FILES.txt'])) + '\n')
        checkpoint()
        shutil.copytree(stage, destination)
    checkpoint()
    if platform.system() == 'Linux':
        archive = output / (identity + '.tar.gz')
        with tarfile.open(archive, 'w:gz') as tar:
            def checked(info):
                checkpoint()
                return info
            tar.add(destination, arcname=identity, filter=checked)
    else:
        archive = output / (identity + '.zip')
        with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED) as zipped:
            for file in sorted(destination.rglob('*')):
                checkpoint()
                if file.is_file():
                    zipped.write(file, Path(identity) / file.relative_to(destination))
    checkpoint()
    (output / (archive.name + '.sha256')).write_text(sha256(archive) + '  ' + archive.name + '\n')
    print(json.dumps({'candidate': str(archive), 'sha256': sha256(archive), 'build': info}, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=project_root(ROOT) / 'dist')
    parser.add_argument('--allow-dirty', action='store_true')
    args = parser.parse_args()
    build(args.output, args.allow_dirty)
