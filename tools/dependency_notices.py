#!/usr/bin/env python3
"""Development packaging tool, never called by learner execution.
Read locked Cargo metadata; collect original license files and pinned upstream notices.
Network use requires explicit --fetch-upstream. No third-party code is executed.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import tomllib
import urllib.error
import urllib.parse
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
CACHE = ROOT / 'assets/licenses/upstream'
PREFIXES = ('license', 'licence', 'copying', 'notice', 'copyright', 'authors', 'ofl', 'ufl')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def download(url):
    req = urllib.request.Request(url, headers={'User-Agent': 'DLMS-Learning-Lab-license-audit'})
    with urllib.request.urlopen(req, timeout=20) as response:
        if not response.url.startswith('https://'):
            raise ValueError('Insecure notice source')
        data = response.read(1_000_001)
    if len(data) > 1_000_000:
        raise ValueError('Notice response exceeds 1 MB')
    return data


def license_files(root):
    return sorted(p for p in root.rglob('*') if p.is_file() and not p.is_symlink()
                  and p.name.lower().startswith(PREFIXES))


def upstream(package, root, fetch, manifest):
    vcs = root / '.cargo_vcs_info.json'
    if not vcs.is_file():
        return []
    sha = json.loads(vcs.read_text()).get('git', {}).get('sha1', '')
    repo = urllib.parse.urlparse(package.get('repository') or '')
    parts = repo.path.strip('/').split('/')
    if repo.hostname != 'github.com' or len(parts) < 2 or not re.fullmatch('[a-f0-9]{40}', sha):
        return []
    owner, project = parts[:2]
    project = project.removesuffix('.git')
    if not all(re.fullmatch(r'[A-Za-z0-9_.-]+', x) and x not in ('.', '..') for x in [owner, project]):
        return []
    key = f'{owner}/{project}/{sha}'
    cached = CACHE / key
    if key not in manifest:
        if not fetch:
            return []
        api = f'https://api.github.com/repos/{owner}/{project}/git/trees/{sha}?recursive=1'
        tree = json.loads(download(api))
        entries = [e for e in tree.get('tree', []) if e.get('type') == 'blob'
                   and Path(e['path']).name.lower().startswith(PREFIXES + ('copyright',))]
        notices = []
        for entry in entries:
            name = entry['path']
            if not all(re.fullmatch(r'[A-Za-z0-9_.-]+', part) and part not in ('.', '..') for part in name.split('/')):
                raise ValueError('Unsafe upstream notice filename')
            url = f'https://raw.githubusercontent.com/{owner}/{project}/{sha}/{name}'
            data = download(url)
            cached.mkdir(parents=True, exist_ok=True)
            dest = cached / name
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(data)
            notices.append({'file': name, 'url': url, 'sha256': digest(dest)})
        manifest[key] = {'revision': sha, 'tree_source': api, 'notices': notices}
    result = []
    for entry in manifest[key]['notices']:
        if not all(re.fullmatch(r'[A-Za-z0-9_.-]+', part) and part not in ('.', '..') for part in entry['file'].split('/')):
            raise ValueError('Unsafe cached notice path')
        path = cached / entry['file']
        if not path.is_file() or path.is_symlink() or digest(path) != entry['sha256']:
            raise ValueError(f'Changed/missing cached notice: {key}/{entry["file"]}')
        result.append(path)
    return result


def collect(metadata, output, fetch=False, dependencies=None, target=None):
    output = Path(output)
    if output.exists() and any(output.iterdir()):
        raise ValueError('Choose an empty notice output directory')
    output.mkdir(parents=True, exist_ok=True)
    manifest_path = CACHE / 'manifest-v1.json'
    manifest = json.loads(manifest_path.read_text()) if manifest_path.exists() else {}
    packages, problems = [], []
    locked = {(p['name'], p['version']): p.get('checksum') for p in tomllib.loads((ROOT / 'Cargo.lock').read_text())['package']}
    for p in sorted(metadata['packages'], key=lambda p: (p['name'], p['version'])):
        if not p['source'] or (dependencies is not None and (p['name'], p['version']) not in dependencies):
            continue
        if not p['source'].startswith('registry+'):
            raise ValueError('Non-registry dependency requires a separate source/notice review')
        root = Path(p['manifest_path']).parent
        files = license_files(root)
        extra = []
        # Workspace crate archives often omit their root license. Font data's
        # OFL/UFL notices do not replace its Rust code's MIT/Apache license.
        if not files or p['name'] == 'epaint_default_fonts':
            try:
                extra = upstream(p, root, fetch, manifest)
            except (urllib.error.URLError, ValueError, KeyError) as error:
                problems.append(f'{p["name"]} {p["version"]}: {error}')
        if not files and not extra:
            problems.append(f'{p["name"]} {p["version"]}: original license text unavailable')
        key = f'{p["name"]}-{p["version"]}'
        if not re.fullmatch(r'[A-Za-z0-9_.+\-]+', key):
            raise ValueError('Unsafe package identifier')
        dest = output / key
        notices = []
        for original, relative in [(f, f.relative_to(root)) for f in files] + [(f, Path('upstream') / f.relative_to(CACHE / '/'.join(f.relative_to(CACHE).parts[:3]))) for f in extra]:
            notice_dest = dest / relative
            notice_dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(original, notice_dest)
            notices.append({'file': notice_dest.relative_to(output).as_posix(), 'sha256': digest(notice_dest)})
        checksum = locked.get((p['name'], p['version']))
        if not checksum:
            problems.append(f'{key}: pinned registry checksum unavailable')
        source_archive = None
        archive = root.parents[2] / 'cache' / root.parent.name / f'{key}.crate'
        if not archive.is_file() or archive.is_symlink() or digest(archive) != checksum:
            problems.append(f'{key}: exact locked source archive unavailable')
        else:
            archive_dest = output / 'source-archives' / archive.name
            archive_dest.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(archive, archive_dest)
            source_archive = {'file': archive_dest.relative_to(output).as_posix(), 'sha256': digest(archive_dest)}
        packages.append({'name': p['name'], 'version': p['version'], 'license_expression': p.get('license'),
                         'repository': p.get('repository'), 'registry': p['source'], 'crate_sha256': checksum,
                         'notices': notices, 'source_archive': source_archive,
                         'covered_source': source_archive if 'MPL-2.0' in (p.get('license') or '') else None})
    if fetch:
        CACHE.mkdir(parents=True, exist_ok=True)
        manifest_path.write_text(json.dumps(manifest, indent=2, sort_keys=True) + '\n')
    inventory = {'version': 1, 'scope': f'normal/build dependency graph for {target}' if dependencies is not None else 'conservative complete locked graph; platform/build/optional dependencies included',
                 'packages': packages, 'unresolved': problems}
    (output / 'inventory-v1.json').write_text(json.dumps(inventory, indent=2) + '\n')
    (output / 'README.txt').write_text('Original dependency license/notice texts; preserve them with application copies.\n'
        'License alternatives retain their upstream SPDX expression. This bundle includes all available notices rather than relicensing dependencies.\n'
        'Bundled font data retains OFL and Ubuntu Font Licence notices. Every dependency’s original source archive is included and verified against Cargo.lock. This preserves embedded headers/notices and supplies MPL covered source.\n'
        'No third-party file has been modified by this collector. Platform runtime libraries are not bundled.\n'
        'See inventory-v1.json for versions, checksums, source URLs and any unresolved obligations.\n')
    return inventory


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('metadata', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--fetch-upstream', action='store_true')
    parser.add_argument('--dependency-tree', type=Path, help='Cargo tree --edges normal,build --prefix none --format {p}')
    parser.add_argument('--target-label')
    args = parser.parse_args()
    dependencies = None
    if args.dependency_tree:
        dependencies = {tuple(m.groups()) for line in args.dependency_tree.read_text().splitlines() if (m := re.match(r'^(\S+) v(\S+)', line))}
        if not dependencies or not args.target_label:
            raise SystemExit('Expected nonempty dependency graph and target label')
    result = collect(json.loads(args.metadata.read_text()), args.output, args.fetch_upstream, dependencies, args.target_label)
    print(json.dumps({'packages': len(result['packages']), 'unresolved': result['unresolved']}, indent=2))
    raise SystemExit(bool(result['unresolved']))
