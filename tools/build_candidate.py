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
    if ROOT.resolve() != common:
        raise ValueError('Build candidates from the main checkout; never share its packaging target with a worktree')
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


def write_candidate_readme(stage, system, version, revision, target):
    """Describe this staged candidate, never a repository-specific launch path."""
    for value, pattern in (
        (version, r'[0-9A-Za-z][0-9A-Za-z.+-]{0,79}'),
        (revision, r'[0-9a-f]{40}(?:-dirty)?'),
        (target, r'[A-Za-z0-9][A-Za-z0-9_.-]{0,127}'),
    ):
        if not isinstance(value, str) or re.fullmatch(pattern, value) is None:
            raise ValueError('Candidate README requires ordinary build identity fields')
    target_component = {'Linux': 'linux', 'Windows': 'windows', 'Darwin': 'darwin'}.get(system)
    if target_component is None or target_component not in target.split('-'):
        raise ValueError('Candidate README platform does not match its native target')
    if not stage.is_dir() or stage.is_symlink():
        raise ValueError('Candidate README requires an ordinary staging directory')
    if system == 'Linux':
        entries = ('dlms-learning-lab', 'install.sh', 'uninstall.sh')
        prefix = ''
        instructions = (
            'Open `dlms-learning-lab` from this extracted folder, or open a terminal '
            'in this folder and run:\n\n```sh\n./dlms-learning-lab\n```\n\n'
            'For optional per-user installation, run `sh ./install.sh` from this '
            'folder, then open **DLMS Learning Lab** from the application menu. '
            'The installer reports its launcher location. No administrator access '
            'or development tools are needed.\n'
        )
    elif system == 'Windows':
        entries = ('dlms-learning-lab.exe', 'Install.ps1')
        prefix = ''
        instructions = (
            'Open `dlms-learning-lab.exe` from this extracted folder. '
            'In PowerShell opened in this folder, the equivalent command is '
            '`.\\dlms-learning-lab.exe`.\n\n'
            'For optional per-user installation, if permitted by local policy, '
            'run `.\\Install.ps1` in PowerShell from this folder. Then use the '
            '**DLMS Learning Lab** Start-menu shortcut. No developer tools or '
            'administrator access are needed. Windows runtime and installation '
            'acceptance must be recorded separately.\n'
        )
    elif system == 'Darwin':
        entries = ('DLMS Learning Lab.app/Contents/MacOS/dlms-learning-lab',
                   'DLMS Learning Lab.app/Contents/Info.plist', 'INSTALL.txt')
        prefix = 'DLMS Learning Lab.app/Contents/Resources/'
        instructions = (
            'Open `DLMS Learning Lab.app` from this extracted folder in Finder. '
            'To install it, drag the app into Applications or your user Applications '
            'folder. Read `INSTALL.txt` for upgrade and removal instructions.\n\n'
            'This candidate is unsigned and not notarized. Follow normal macOS '
            'security checks; signing/notarization and macOS runtime acceptance '
            'remain pending. No development tools are needed.\n'
        )
    else:
        raise ValueError('Unsupported native README platform')
    # Fixed relative entries must really exist in this stage. A README for a
    # wrong-platform binary or partial package must not point outside the folder.
    required = (*entries, *(prefix + name for name in (
        'BUILD_INFO.json', 'docs/LEARNER_GUIDE.md', 'docs/INSTALLATION.md',
        'LICENSE', 'NOTICE',
    )))
    for relative in required:
        path = stage
        for part in Path(relative).parts:
            path = path / part
            if path.is_symlink():
                raise ValueError('Candidate README cannot point through resource links')
        if not path.is_file():
            raise ValueError('Candidate README launch/resource entry missing: ' + relative)
    text = (
        '# DLMS Learning Lab — local installation candidate\n\n'
        f'Version: **{version}**  \nRevision: `{revision}`  \nTarget: `{target}`\n\n'
        '## Open this candidate\n\n' + instructions + '\n'
        'The application is native and offline, with bundled lessons and fonts. '
        'No repository checkout, account, profile path, VM or browser is needed.\n\n'
        '## Progress and guidance\n\n'
        'Learner progress stays in a separate application-data folder. Use '
        '**Settings → Your data** to back it up before upgrading. Close the app '
        'before replacing application files. Keep application data when removing '
        'the app if you want to retain progress.\n\n'
        f'- [Learner guide](<{prefix}docs/LEARNER_GUIDE.md>)\n'
        f'- [Installation and upgrade details](<{prefix}docs/INSTALLATION.md>)\n'
        f'- [Exact build information](<{prefix}BUILD_INFO.json>)\n'
        f'- [Project license](<{prefix}LICENSE>) and [third-party notices](<{prefix}NOTICE>)\n\n'
        'This is a development milestone candidate. Building a package does not '
        'establish runtime, complete-curriculum, accessibility or beginner '
        'acceptance. No public release is published by the packaging tool.\n'
    )
    # Staging is fresh; never replace an existing candidate README or link.
    with (stage / 'README.md').open('x', encoding='utf-8') as stream:
        stream.write(text)


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
        for name in ('LICENSE', 'NOTICE'):
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
        write_candidate_readme(stage, platform.system(), version, revision, target)
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
