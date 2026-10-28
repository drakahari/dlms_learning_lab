#!/usr/bin/env python3
"""Check the offline coverage ledger's completeness and evidence references.
This validates bookkeeping, not educational quality or complete exam preparation.
"""
import csv
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]


def csv_rows(path):
    with path.open(newline="", encoding="utf-8") as stream:
        reader = csv.DictReader(stream)
        rows = list(reader)
        if (not reader.fieldnames or len(reader.fieldnames) != len(set(reader.fieldnames))
                or any(None in row or any(value is None for value in row.values()) for row in rows)):
            raise ValueError('CSV columns do not match the declared header: ' + path.name)
        return rows



def _source_file(root, name, cap):
    path = root / name
    if (root.is_symlink() or any((root / Path(*Path(name).parts[:n])).is_symlink()
                                for n in range(1, len(Path(name).parts) + 1))
            or not path.is_file() or not path.resolve().is_relative_to(root.resolve())
            or path.stat().st_size > cap):
        raise ValueError('Missing, unsafe or oversized catalog source: ' + name)
    return path.read_text(encoding='utf-8')


def _without_rust_comments(source):
    """Lex comments/quoted spans only; never interpret or execute Rust code."""
    out = []
    quoted = []
    at = 0
    raw_pattern = re.compile(r'(?:br|r)(#{0,255})"')
    char_pattern = re.compile("'(?:[^'\\\\\\r\\n]|\\\\(?:x[0-9a-fA-F]{2}|u\\{[0-9a-fA-F_]+\\}|[^\\r\\n]))'")
    while at < len(source):
        if source.startswith('//', at):
            end = source.find('\n', at)
            end = len(source) if end < 0 else end
            out.append(' ' * (end - at))
            at = end
        elif source.startswith('/*', at):
            begin = at
            depth = 1
            at += 2
            while at < len(source) and depth:
                if source.startswith('/*', at):
                    depth += 1
                    at += 2
                elif source.startswith('*/', at):
                    depth -= 1
                    at += 2
                else:
                    at += 1
            if depth:
                raise ValueError('Unclosed comment in catalog registry')
            out.append(''.join('\n' if c == '\n' else ' ' for c in source[begin:at]))
        else:
            raw = raw_pattern.match(source, at)
            char = char_pattern.match(source, at)
            begin = at
            if raw:
                end_token = '"' + raw[1]
                end = source.find(end_token, raw.end())
                if end < 0:
                    raise ValueError('Unclosed raw string in catalog registry')
                at = end + len(end_token)
                out.append(source[begin:at])
                quoted.append((begin, at))
            elif char:
                at = char.end()
                out.append(source[begin:at])
                quoted.append((begin, at))
            elif source[at] == '"':
                at += 1
                while at < len(source):
                    if source[at] == '\\':
                        at += 2
                    elif source[at] == '"':
                        at += 1
                        break
                    else:
                        at += 1
                else:
                    raise ValueError('Unclosed string in catalog registry')
                out.append(source[begin:at])
                quoted.append((begin, at))
            else:
                out.append(source[at])
                at += 1
    return ''.join(out), quoted


def registered_pack_paths(root):
    """Admit only the current literal load_catalog include list, fail closed."""
    raw = _source_file(root, 'src/pack.rs', 512_000)
    if '\0' in raw:
        raise ValueError('Control data in catalog registry')
    source, quoted = _without_rust_comments(raw)
    header = re.compile(r'^pub\s+fn\s+bundled\s*\(\s*\)\s*->\s*Result\s*<\s*Vec\s*<\s*Scenario\s*>\s*,\s*String\s*>\s*\{', re.M)
    headers = [m for m in header.finditer(source)
               if not any(begin <= m.start() < end for begin, end in quoted)]
    if len(headers) != 1:
        raise ValueError('Expected one static bundled catalog function')
    if source[:headers[0].start()].rstrip().endswith(']'):
        raise ValueError('Conditional or attributed catalog registry is unsupported')
    body = re.fullmatch(r'\s*load_catalog\s*\(\s*&\s*\[(.*?)\]\s*\)\s*\}\s*',
                        source[headers[0].end():], re.S)
    if body is None:
        raise ValueError('Unsupported dynamic or malformed bundled catalog registry')
    entries = body[1]
    entry = re.compile('\\s*include_str!\\s*\\(\\s*"([^"\\\\\\x00-\\x1f]+)"\\s*\\)\\s*,')
    at = 0; paths = []
    while entries[at:].strip():
        match = entry.match(entries, at)
        if match is None:
            raise ValueError('Unknown or malformed static catalog entry')
        name = match[1]
        if not re.fullmatch(r'\.\./scenarios/[a-z0-9][a-z0-9-]*\.json', name):
            raise ValueError('Unsafe or unknown registered pack path')
        name = name[3:]
        if name in paths or len(paths) >= 100:
            raise ValueError('Duplicate or excessive registered pack path')
        paths.append(name); at = match.end()
    if not paths:
        raise ValueError('The registered catalog cannot be empty')
    return paths


def _pack_labs(root, name, remaining=32_000_000):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError('Duplicate pack JSON key: ' + key)
            result[key] = value
        return result
    def constant(value):
        raise ValueError('Unsupported non-finite pack JSON constant: ' + value)
    raw = _source_file(root, name, min(2_000_000, remaining))
    pack = json.loads(raw, object_pairs_hook=unique, parse_constant=constant)
    if (not isinstance(pack, dict) or set(pack) != {'schema_version', 'scenarios'}
            or type(pack['schema_version']) is not int or pack['schema_version'] != 1
            or not isinstance(pack['scenarios'], list) or not pack['scenarios']
            or len(pack['scenarios']) > 1000):
        raise ValueError('Malformed scenario pack: ' + name)
    labs = {}
    for lab in pack['scenarios']:
        if (not isinstance(lab, dict) or not isinstance(lab.get('id'), str)
                or not re.fullmatch(r'[a-z0-9-]{1,80}', lab['id'])):
            raise ValueError('Malformed authored ID: ' + name)
        if lab['id'] in labs:
            raise ValueError('Duplicate authored ID')
        labs[lab['id']] = lab
    return labs, len(raw.encode('utf-8'))


def catalog_inventory(root):
    registered = registered_pack_paths(root)
    labs = {}; all_ids = set(); draft_ids = []
    total = 0
    for name in registered:
        # The safe reader bounds the remaining collection before JSON decoding.
        pack, size = _pack_labs(root, name, 32_000_000 - total)
        total += size
        if all_ids.intersection(pack):
            raise ValueError('Duplicate authored ID')
        if len(labs) + len(pack) > 1000:
            raise ValueError('Registered catalog capacity exceeded')
        all_ids.update(pack)
        labs.update(pack)
    # Directory enumeration is ONLY an unregistered-draft report, never credit.
    draft_files = 0
    draft_bytes = 0
    for path in sorted((root / 'scenarios').iterdir()):
        if path.suffix != '.json' or path.name == 'units-v1.json':
            continue
        name = 'scenarios/' + path.name
        if name in registered:
            continue
        if not re.fullmatch(r'[a-z0-9][a-z0-9-]*\.json', path.name):
            raise ValueError('Unsafe unregistered draft source path')
        draft_files += 1
        if draft_files > 100:
            raise ValueError('Unregistered draft inventory capacity exceeded')
        pack, size = _pack_labs(root, name, 32_000_000 - draft_bytes)
        draft_bytes += size
        if all_ids.intersection(pack):
            raise ValueError('Duplicate authored ID')
        if len(draft_ids) + len(pack) > 1000:
            raise ValueError('Unregistered draft inventory capacity exceeded')
        all_ids.update(pack)
        draft_ids.extend(pack)
    return labs, sorted(draft_ids), registered

def validate(root):
    inventory = json.loads((root / 'docs/OBJECTIVE_SOURCES.json').read_text(encoding="utf-8"))
    if inventory['schema_version'] != 1:
        raise ValueError('Unsupported objective inventory')
    expected = {}
    source_urls = {}
    for source in inventory['sources']:
        for item in source['items']:
            key = (source['blueprint'], item['reference'])
            if key in expected:
                raise ValueError('Duplicate source reference')
            if not re.fullmatch(r'[0-9a-f]{64}', item['normalized_sha256']):
                raise ValueError('Invalid source fingerprint')
            expected[key] = item
            source_urls[key] = source['url'] + ('#' + item['anchor'] if item['anchor'] else '')
    parents = csv_rows(root / 'docs/CURRICULUM.csv')
    parent_keys = [(row['blueprint'], row['objective_id']) for row in parents]
    if len(parent_keys) != len(set(parent_keys)):
        raise ValueError('Duplicate parent planning objective')
    current = {(r['blueprint'], r['objective_id']) for r in parents if 'upcoming' not in r['blueprint']}
    indexed = {key for key, item in expected.items() if item['kind'] == 'objective'}
    if current != indexed:
        raise ValueError('Current top-level map and source inventory disagree')
    rows = csv_rows(root / 'docs/SKILL_COVERAGE.csv')
    keys = [(r['blueprint'], r['reference']) for r in rows]
    if len(keys) != len(set(keys)) or set(keys) != set(expected):
        raise ValueError('Coverage has missing, duplicate or unexpected references')
    labs, draft_ids, registered_paths = catalog_inventory(root)
    for key, row in zip(keys, rows):
        source = expected[key]
        if row['source_sha256'] != source['normalized_sha256']:
            raise ValueError('Source scope changed; re-review this coverage claim')
        if row['source_url'] != source_urls[key]:
            raise ValueError('Coverage source URL mismatch')
        if row['parent'] != source['parent'] or row['kind'] != source['kind'] or str(source['position']) != row['source_position']:
            raise ValueError('Coverage source position/parent mismatch: ' + str(key))
        if row['status'] not in ('Open', 'Partial practice', 'Supporting explanation'):
            raise ValueError('Unknown/unreviewed completion claim: ' + str(key))
        if not row['remaining'].strip() or not row['planned_assignment'].strip():
            raise ValueError('Every incomplete item needs an explicit gap and planned assignment')
        evidence = row['implemented_scenarios'].split(';') if row['implemented_scenarios'] else []
        tests = row['evidence_tests'].split(';') if row['evidence_tests'] else []
        if row['status'] == 'Open':
            if evidence or tests or row['verified_scope']:
                raise ValueError('Open items cannot imply verified evidence')
        elif not evidence or not tests or not row['verified_scope'].strip():
            raise ValueError('Partial/supporting claims require scoped evidence')
        if any(lab not in labs for lab in evidence):
            raise ValueError('Evidence references missing or unregistered authored lab: ' + str(key))
        for name in tests:
            path = Path(name)
            if (path.is_absolute() or '..' in path.parts or not name.startswith('tests/')
                    or not (root / path).is_file() or not (root / path).resolve().is_relative_to(root.resolve())
                    or any((root / Path(*path.parts[:n])).is_symlink() for n in range(1, len(path.parts) + 1))):
                raise ValueError('Missing or unsafe test evidence: ' + name)
    coverage = {(row['blueprint'], row['reference']): row for row in rows}
    status = {'Open': 'Planned', 'Partial practice': 'Partial',
              'Supporting explanation': 'Supporting explanation'}
    for parent in parents:
        key = (parent['blueprint'], parent['objective_id'])
        if key not in current:
            continue
        scoped = coverage[key]
        if parent.get('status') != status[scoped['status']]:
            raise ValueError('Parent planning status disagrees with scoped evidence: ' + str(key))
        credited = parent.get('implemented_scenarios', '')
        if credited == 'None':
            credited = ''
        if credited != scoped['implemented_scenarios']:
            raise ValueError('Parent planning scenarios disagree with scoped evidence: ' + str(key))
    return dict(references=len(rows), current_objectives=len(current),
                partial=sum(r['status'] == 'Partial practice' for r in rows),
                supporting=sum(r['status'] == 'Supporting explanation' for r in rows),
                open=sum(r['status'] == 'Open' for r in rows), authored_labs=len(labs),
                registered_labs=len(labs), registered_pack_paths=registered_paths,
                unregistered_draft_ids=draft_ids)


if __name__ == '__main__':
    print(json.dumps(validate(ROOT), indent=2))
