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
from build_safety import JOBS, TEST_THREADS, workflow, project_root, disk_directory, run as safe_run, checkpoint, capture

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


def compile_native(cargo, target, binary_name, env, *, jobs=JOBS):
    if type(jobs) is not int or not 1 <= jobs <= JOBS:
        raise ValueError('Candidate compilation stays within one or two jobs')
    # Explicit CLI options override Cargo environment and config target settings.
    common = project_root(ROOT)
    if ROOT.resolve() != common:
        raise ValueError('Build candidates from the main checkout; never share its packaging target with a worktree')
    build_dir = (common / 'target/candidate-build').resolve()
    if not build_dir.is_relative_to(common):
        raise ValueError('Candidate build target escaped the project volume')
    disk_directory(build_dir)
    env = {k: v for k, v in env.items() if k not in ('CARGO_BUILD_TARGET', 'CARGO_TARGET_DIR')}
    env = {**env, 'CARGO_BUILD_JOBS': str(jobs), 'RUST_TEST_THREADS': str(TEST_THREADS)}
    safe_run([cargo, 'build', '--jobs', str(jobs), '--release', '--locked', '--offline', '--target', target,
                    '--target-dir', str(build_dir)], cwd=ROOT, env=env)
    artifact = build_dir / target / 'release' / binary_name
    if not artifact.is_file() or artifact.is_symlink():
        raise ValueError('Expected native compiler artifact was not created')
    return artifact


def presentation_sources():
    declaration = ROOT / 'src/ui/lesson_presentation.rs'
    if not declaration.is_file():
        return []
    names = sorted(set(re.findall(r'include_str!\(\s*"../../scenarios/presentation/([A-Za-z0-9_.-]+\.json)"\s*\)', declaration.read_text())))
    sources = []
    for name in names:
        path = ROOT / 'scenarios/presentation' / name
        for component in (ROOT / 'scenarios', ROOT / 'scenarios/presentation', path):
            if component.is_symlink():
                raise ValueError('Presentation resources must not follow links')
        if not path.is_file() or not path.resolve().is_relative_to(ROOT.resolve()):
            raise ValueError('Presentation resources must be ordinary contained files')
        sources.append(path)
    return sources


def embedded_content_hashes():
    names = set()
    for filename, prefix in (('src/pack.rs', '../'), ('src/course.rs', '../'),
                             ('src/course/metadata.rs', '../../')):
        declaration = ROOT / filename
        if not declaration.exists() and filename == 'src/course/metadata.rs':
            continue
        names.update(re.findall(r'include_str!\(\s*"' + re.escape(prefix) +
                                r'scenarios/([A-Za-z0-9_.-]+\.json)"\s*\)', declaration.read_text()))
    if not names:
        raise ValueError('Embedded content declarations unavailable')
    result = {}
    for name in sorted(names):
        path = ROOT / 'scenarios' / name
        if path.is_symlink() or not path.is_file() or not path.resolve().is_relative_to(ROOT):
            raise ValueError('Embedded content must be ordinary contained data')
        result[name] = sha256(path)
    for path in presentation_sources():
        result[path.relative_to(ROOT / 'scenarios').as_posix()] = sha256(path)
    return result


def review_catalog_identity():
    # Use declarations, not a guessed lab-count target or directory sweep.
    from teaching_inventory import catalog
    labs = catalog()
    names = set()
    for filename, prefix in (('src/course.rs', '../'), ('src/course/metadata.rs', '../../')):
        declaration = ROOT / filename
        if not declaration.exists() and filename == 'src/course/metadata.rs':
            continue
        names.update(re.findall(r'include_str!\(\s*"' + re.escape(prefix) +
                                r'scenarios/([A-Za-z0-9_.-]+\.json)"\s*\)', declaration.read_text()))
    units = []
    for name in sorted(names):
        index = json.loads((ROOT / 'scenarios' / name).read_text())
        if index.get('version') != 1:
            raise ValueError('Unsupported bundled review unit version')
        units.extend(index['units'])
    if len({u['id'] for u in units}) != len(units):
        raise ValueError('Duplicate bundled review unit')
    if {u['id'] for u in units} != {lab['unit'] for lab, _ in labs}:
        raise ValueError('Review unit metadata differs from the registered lab catalog')
    return {'labs': len(labs), 'units': len(units), 'all_existing_review_labs': True}


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


def build(output, allow_dirty=False, *, jobs=JOBS):
    with workflow(ROOT, 'candidate'):
        return _build(output, allow_dirty, jobs)


def _build(output, allow_dirty, jobs):
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
    binary = compile_native(cargo, target, binary_name, env, jobs=jobs)
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
        if env.get('DLMS_CCNA_PILOT') == '1':
            # Review builds retain a separate progress profile; all app builds use the current catalog.
            info['scope'] = ('Curriculum continuation on approved teaching baseline; review pending' if env.get('DLMS_PRESENTATION_REVIEW') == '1' else 'CCNA instructional presentation pilot; owner review pending')
            info['pilot_profile'] = 'Separate ccna-presentation-pilot subdirectory; originals never written'
            info['pilot_catalog'] = review_catalog_identity()
            source_paths = sorted([p for p in (ROOT / 'src').rglob('*.rs') if p.is_file()] +
                                  [ROOT / 'scenarios' / name for name in embedded_content_hashes()])
            info['pilot_source_sha256'] = {p.relative_to(ROOT).as_posix(): sha256(p) for p in source_paths}
            (resources / 'BUILD_INFO.json').write_text(json.dumps(info, indent=2) + '\n')
            if env.get('DLMS_PRESENTATION_REVIEW') == '1':
                info['pilot_catalog'] = review_catalog_identity()
                (resources / 'BUILD_INFO.json').write_text(json.dumps(info, indent=2) + '\n')
                for name in ('TEACHING_PRESENTATION.md', 'CAMPUS_ARCHITECTURE_SUBSET.md', "LEAF_SPINE_SUBSET.md", "SAMBA_SHARE_SUBSET.md", "VIRTUAL_ISOLATION_SUBSET.md", "GUEST_DRIVER_SUBSET.md", "SMTP_ROUTING_SUBSET.md", "SIEVE_SUBSET.md", "SIEVE_ADDRESS_SUBSET.md", "SIEVE_DELIVERY_SUBSET.md", "SIEVE_VACATION_SUBSET.md", "PROCMAIL_MIGRATION_LESSON.md", "MAILBOX_ACCESS_SUBSET.md", "MAILBOX_ADMIN_SUBSET.md", "LINUX_ROUTER_SUBSET.md", "LINUX_CONNECTIONS_SUBSET.md", "LINUX_IPV6_ROUTER_SUBSET.md", "LINUX_IPV6_LESSONS.md", "LINUX_SERVICE_RULES_SUBSET.md", "LINUX_SERVICES_ROUTER_LESSON.md", "LINUX_IPV6_ADMIN_SUBSET.md", "LINUX_IPV6_STARTUP_SUBSET.md", "FTP_IDENTITY_SUBSET.md", "SERVICE_EXPOSURE_SUBSET.md", "VPN_POINT_SUBSET.md", "RPM_CONFIG_UPDATES_SUBSET.md", "RPM_PATCH_LESSONS.md",
        "AUTH_FILTER_SUBSET.md",
        "IDS_RULE_SUBSET.md", "TCP_PROBE_SUBSET.md", "TELNET_TRANSPORT_SUBSET.md", "POE_SUBSET.md", "KERNEL_BUILD_SUBSET.md", "DKMS_SUBSET.md", "INITRAMFS_SUBSET.md", "PXE_SUBSET.md", "ISCSI_SUBSET.md", "ATA_SMART_SUBSET.md", "APACHE_HTTPS_SUBSET.md", "DESKTOP_PROFILE_SUBSET.md", "SECURITY_INVESTIGATION_SUBSET.md", "CREDENTIALS_VPN_SUBSET.md", "STAGED_NETWORK_SUBSET.md", "NETWORK_EVIDENCE_REVIEW_SUBSET.md", "SITE_CONNECTIVITY_SUBSET.md", "HOST_AND_DNS_SECURITY_SUBSET.md", "CCNA_VLAN_GATEWAY_SUBSET.md", "NFS_EXPORT_OPERATIONS_SUBSET.md", "ROUTE_READING_SUBSET.md", "OSPF_BROADCAST_SUBSET.md",
        "IDS_RULE_LESSONS.md",
        "AUTH_FILTER_LESSONS.md", 'TEACHING_INVENTORY.md', 'TEACHING_INVENTORY.csv', 'GATEWAY_BATCH.md', 'CONTINUATION.md', 'CONTINUATION_COVERAGE.csv', 'SYSLOG_SUBSET.md', 'SYSLOG_WORKSPACE.md', 'SYSLOG_POLICY_LESSONS.md', 'CAPACITY_LESSONS.md', 'FS_CAPACITY_SUBSET.md', 'RPM_REPOSITORIES_SUBSET.md', 'REPOSITORY_LESSONS.md', 'LINUX_HOST_SUBSET.md', 'HOST_NETWORK_LESSONS.md', 'LINUX_HOST_IPV6_SUBSET.md', 'HOST_IPV6_LESSONS.md', 'SWAP_SUBSET.md', 'SWAP_POLICY_LESSONS.md', 'LVM_SUBSET.md', 'LVM_PROVISIONING_SUBSET.md', 'LVM_PROVISIONING_LESSONS.md', 'LVM_RETIREMENT_SUBSET.md', 'LVM_RETIREMENT_LESSONS.md', 'LVM_ACTIVATION_SUBSET.md', 'LVM_ACTIVATION_LESSONS.md', 'LVM_RENAME_SUBSET.md', 'LVM_RENAME_LESSONS.md', 'LVM_PV_CAPACITY_SUBSET.md', 'LVM_PV_CAPACITY_LESSONS.md', 'MBR_LAYOUT_SUBSET.md', 'MBR_LAYOUT_LESSONS.md', 'LVM_COW_MODEL.md', 'LVM_COW_LESSONS.md', 'LVM_DEVICE_PATH_MODEL.md', 'LVM_REFERENCE_LESSONS.md', 'LVM_CONFIG_SCAN_SUBSET.md', 'LVM_CONFIG_LESSONS.md', 'LVM_MOUNT_MODEL.md', 'LVM_MOUNT_LESSONS.md', 'RAID_ASSEMBLY_SUBSET.md', 'RAID_ASSEMBLY_LESSONS.md', 'VLAN_HOST_SUBSET.md', 'VLAN_LESSONS.md', 'BOND_CONTROL_SUBSET.md', 'BOND_VLAN_LESSON.md', 'BRIDGE_FABRIC_SUBSET.md', 'BRIDGE_CONTROL_SUBSET.md', 'BRIDGE_LAN_LESSON.md', 'PAM_CONTROL_SUBSET.md', 'PAM_AUTH_LESSON.md', 'AGING_SUBSET.md', 'AGING_LESSONS.md', 'STATIC_CAM_SUBSET.md', 'PORT_SECURITY_SUBSET.md', 'PORT_SECURITY_LESSONS.md', 'ACCESS_OUTPUTS_SUBSET.md', 'DHCP_SNOOPING_SUBSET.md', 'DHCP_SNOOPING_LESSONS.md', 'DAI_SUBSET.md', 'ARP_INSPECTION_LESSONS.md', 'MANAGEMENT_POLICY_SUBSET.md', 'MANAGEMENT_POLICY_LESSONS.md', 'LOCAL_ACCESS_SUBSET.md', 'LOCAL_ACCESS_LESSONS.md', 'AAA_METHODS_SUBSET.md', 'AAA_DECISIONS_LESSONS.md', 'LDAP_QUERY_SUBSET.md', 'LDAP_CLIENT_LESSONS.md', 'LDAP_CHANGE_SUBSET.md', 'LDAP_CHANGE_LESSONS.md', 'LDAP_UTILITY_SUBSET.md', 'LDAP_UTILITY_LESSONS.md', 'LDAP_RETIREMENT_LESSON.md', 'LDAP_PROTECTED_AUTH_SUBSET.md', 'LDAP_PASSWORD_LESSON.md', 'LDAP_ACCESS_SUBSET.md', 'LDAP_ACCESS_LESSON.md', 'LDAP_CONFIGURATION_SUBSET.md', 'LDAP_CONFIGURATION_LESSON.md', 'LDAP_SCHEMA_SUBSET.md', 'LDAP_SCHEMA_LESSON.md', 'LDAP_SCHEMA_QUERY_SUBSET.md', 'LDAP_SCHEMA_QUERY_LESSON.md', 'IPV6_INTERFACE_IDS.md', 'BOOT_ENTRY_LESSONS.md', 'GRUB_MENU_LESSONS.md', 'GRUB_SETTINGS_LESSONS.md', 'BIOS_LOADER_LESSON.md', 'EFI_LOADER_LESSON.md', 'EFI_REMOVABLE_LESSON.md', 'GRUB_TEMPORARY_LESSON.md', 'RESCUE_STARTUP_LESSON.md', 'CHRONY_SOURCE_LESSON.md', 'COPIED_JOURNAL_LESSON.md', 'LOCAL_MAIL_SUBSET.md', 'LOCAL_MAIL_LESSONS.md', 'DEVICE_PATH_LESSONS.md', 'JSON_INVENTORY_LESSONS.md', 'DISK_LAYOUT_LESSONS.md', 'MEASUREMENT_REPORT_LESSONS.md', 'RESOURCE_OBSERVATION_LESSONS.md', 'CAPACITY_PLANNING_LESSONS.md', 'MAINTENANCE_NOTICES_SUBSET.md', 'MAINTENANCE_NOTICE_LESSONS.md', 'SOURCE_INSTALL_LESSONS.md', 'SOURCE_INSTALL_SUBSET.md', 'DNS_ACCESS_LESSONS.md', 'PRINT_QUEUE_SUBSET.md', 'PRINT_QUEUE_LESSONS.md', 'LOCALIZATION_SUBSET.md', 'LOCALIZATION_LESSONS.md', 'TIMEZONE_SUBSET.md', 'TIMEZONE_LESSONS.md', 'WIRELESS_CHANNEL_SUBSET.md', 'WIRELESS_CHANNEL_LESSONS.md', 'PROXY_POLICY_SUBSET.md', 'PROXY_POLICY_LESSONS.md', 'LINUX_EXISTING_COVERAGE_RECONCILIATION.md', 'HTTP_CACHE_SUBSET.md', 'HTTP_CACHE_LESSONS.md', 'REVERSE_PROXY_SUBSET.md', 'REVERSE_PROXY_LESSONS.md', 'INTERNET_SERVICES_SUBSET.md', 'SERVICE_NAME_LESSONS.md', 'MIXED_IP_LESSONS.md', 'PROTOCOL_EVIDENCE_SUBSET.md', 'HOST_ECHO_DIAGNOSIS.md', "REST_CONTROLLER_SUBSET.md", "CLIENT_DIAGNOSTIC_LESSON.md", "HOST_DIRECT_DNS.md", "HOST_QUERY_EXTENSION.md", "AUTHORITATIVE_QUERY_TOOLS.md", "SSH_ACCESS_SUBSET.md", "MODAL_EDITING_SUBSET.md", "EDITOR_HANDOVER_SUBSET.md", "NETWORK_TRANSFER_SUBSET.md", 'SNMP_SUBSET.md', 'SNMP_MONITOR_LESSONS.md', 'MANAGEMENT_ACCESS_RECONCILIATION.md', 'WLAN_PROFILE_SUBSET.md', 'WLAN_PROFILE_LESSONS.md', "WLAN_PATH_SUBSET.md", "WLAN_PATH_LESSONS.md", 'DNS_ACCESS_SUBSET.md', 'REMAINING_SCOPE_RECONCILIATION.md', 'SCOPE_RECONCILIATION_PASS1.md', 'AUTOFS_SUBSET.md', 'AUTOFS_LESSONS.md', 'DEBUG_RECOVERY_LESSONS.md', 'LINUX_LOGGING_SUBSET.md', 'LINUX_LOGGING_LESSONS.md', 'STARTUP_LESSONS.md', 'QOS_SUBSET.md', 'QOS_LESSONS.md', 'FILE_BACKUP_SUBSET.md', 'FILE_BACKUP_LESSONS.md', 'BACKUP_SELECTION_LESSON.md', 'DHCP_CONFIG_SUBSET.md', 'DHCP_SERVER_LESSONS.md', 'NM_PROFILES_SUBSET.md', 'NM_PROFILE_LESSONS.md', 'REMOVABLE_MOUNT_SUBSET.md', 'REMOVABLE_MOUNT_LESSON.md', 'CLIENT_NAMES_SUBSET.md', 'CLIENT_NAME_LESSONS.md', 'RESOLVER_SEARCH_SUBSET.md', 'RESOLVER_SEARCH_LESSONS.md', 'DEBIAN_CONTENTS_SUBSET.md', 'DEBIAN_CONTENTS_LESSON.md', 'DEBIAN_SOURCES_SUBSET.md', 'DEBIAN_SOURCES_LESSONS.md', 'SHELL_TRACING_SUBSET.md', 'SHELL_TRACING_LESSONS.md', 'SHELL_ENVIRONMENT_SUBSET.md', 'ENVIRONMENT_LESSONS.md', 'USER_MOUNT_SUBSET.md', 'USER_MOUNT_LESSONS.md', 'DEBIAN_CONFLICTS_SUBSET.md', 'DEBIAN_CONFLICT_LESSONS.md', 'NM_DHCP_SUBSET.md', 'NM_DHCP_LESSONS.md', 'FS_INTEGRITY_SUBSET.md', 'FS_INTEGRITY_LESSONS.md', 'SHELL_STARTUP_SUBSET.md', 'SHELL_STARTUP_LESSONS.md', 'SHELL_FILE_LIFECYCLE_SUBSET.md', 'SHELL_FILE_LIFECYCLE_LESSONS.md', 'DEBIAN_CONFFILES_SUBSET.md', 'DEBIAN_CONFFILE_LESSONS.md', 'RAID1_SUBSET.md', 'RAID1_LESSONS.md', 'SHELL_EVALUATOR18_BASELINE.md', 'SHELL_ASSOCIATIVE_SUBSET.md', 'SHELL_ASSOCIATIVE_REFERENCE.md', 'SHELL_ASSOCIATIVE_LESSONS.md', 'COURSE_METADATA_LOADING.md', 'SHARED_LIBRARIES_SUBSET.md', 'SHARED_LIBRARIES_LESSONS.md', 'FHS_LESSONS.md', 'SHELL_MANAGED_CONFIG_LESSON.md', 'SHELL_CONTENT_BACKUPS.md', 'TRANSPORT_SUBSET.md', 'TRANSPORT_LESSONS.md', 'RECURSIVE_TREE_SUBSET.md', 'RECURSIVE_TREE_LESSONS.md', 'FIND_SIZE_SUBSET.md', 'FIND_SIZE_LESSONS.md', 'KERNEL_MODULE_SUBSET.md', 'KERNEL_MODULE_LESSONS.md', 'KERNEL_MODULE_NAMES.md', 'SHELL_LOG_TRIAGE_LESSON.md', 'RPM_METADATA_SUBSET.md', 'RPM_METADATA_LESSONS.md', 'LISTENER_AUDIT_LESSON.md', 'ACCOUNT_SKELETON_SUBSET.md', 'ACCOUNT_SKELETON_LESSON.md', 'COMMAND_LOCATION_SUBSET.md', 'COMMAND_LOCATION_LESSON.md', 'WHEREIS_SUBSET.md', 'WHEREIS_LESSON.md', 'LINKS_INODE_TEACHING.md', 'DIRECTORY_POLICY_SUBSET.md', 'DIRECTORY_POLICY_LESSONS.md', 'FILE_INDEX_SUBSET.md', 'FILE_INDEX_LESSON.md', 'PATH_LOOKUP_SUBSET.md', 'PATH_LOOKUP_LESSON.md'):
                    shutil.copyfile(ROOT / 'docs' / name, resources / 'docs' / name)
                presentation = resources / 'scenarios/presentation'
                presentation.mkdir(parents=True, exist_ok=True)
                for source in presentation_sources():
                    shutil.copyfile(source, presentation / source.name)
                (stage / 'TEACHING-REVIEW-READ-ME.txt').write_text(
                    'LOCAL REVIEW ONLY. Run ./dlms-learning-lab directly. Do not run install.sh.\n'
                    'Current registered review labs use the approved teaching pattern; BUILD_INFO.json records this catalog identity.\n'
                    'Uses the existing separate ccna-presentation-pilot review profile for continuity; originals never written.\n'
                    'No independent novice, screen-reader or cross-platform acceptance. No public release.\nSee docs/TEACHING_PRESENTATION.md and docs/TEACHING_INVENTORY.md.\n')
            else:
                shutil.copyfile(ROOT / 'docs/CCNA_PILOT.md', resources / 'CCNA_PILOT.md')
                (stage / 'CCNA-PILOT-READ-ME.txt').write_text(
                'LOCAL REVIEW ONLY. Run ./dlms-learning-lab directly. Do not run install.sh.\n'
                'The binary uses a separate progress/preferences copy by default. Your installed app is unchanged.\n'
                'The approved foundations lessons appear first in CCNA; current source also includes the all-track presentation rollout.\n'
                'This uncommitted pilot has no release or cross-platform acceptance. See CCNA_PILOT.md.\n')

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
