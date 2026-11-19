#!/usr/bin/env python3
"""Check the offline coverage ledger's completeness and evidence references.
This validates bookkeeping, not educational quality or complete exam preparation.
"""
import csv
import hashlib
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


def _legacy_registered_pack_paths(root):
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


# Closed supported source grammar; never interpret or execute Rust.
# Existing literal parser is retained verbatim as _legacy_registered_pack_paths.

def _registry_source(root):
    raw = _source_file(root, 'src/pack.rs', 512_000)
    if '\0' in raw:
        raise ValueError('Control data in catalog registry')
    source, quoted = _without_rust_comments(raw)
    masked = list(source)
    for begin, end in quoted:
        masked[begin:end] = ['\n' if c == '\n' else ' ' for c in source[begin:end]]
    masked = ''.join(masked)
    depth = 0
    levels = []
    for char in masked:
        levels.append(depth)
        if char == '{':
            depth += 1
        elif char == '}':
            depth -= 1
            if depth < 0:
                raise ValueError('Unbalanced catalog source braces')
    if depth:
        raise ValueError('Unbalanced catalog source braces')
    return source, masked, levels


def _registry_function(source, masked, levels, name, helper):
    # Top-level named functions only; quoted fake headers, nested functions and
    # arbitrary macros never become a source of registered paths.
    pattern = re.compile(r'^[ \t]*(?:pub\s+)?fn\s+' + re.escape(name) + r'\s*\(', re.M)
    headers = [m for m in pattern.finditer(masked) if levels[m.start()] == 0]
    if len(headers) != 1:
        raise ValueError('Expected exactly one static registry function: ' + name)
    start = headers[0].start()
    if source[:start].rstrip().endswith(']'):
        raise ValueError('Conditional or attributed registry function is unsupported: ' + name)
    result = (r'Vec\s*<\s*&\s*\x27static\s+str\s*>' if helper else
              r'Result\s*<\s*Vec\s*<\s*Scenario\s*>\s*,\s*String\s*>')
    header = re.compile(r'[ \t]*(?:pub\s+)?fn\s+' + re.escape(name) +
                        r'\s*\(\s*\)\s*->\s*' + result + r'\s*\{')
    match = header.match(masked, start)
    if match is None:
        raise ValueError('Unsupported registry signature: ' + name)
    begin = match.end()
    level = 1
    for at in range(begin, len(masked)):
        if masked[at] == '{':
            level += 1
        elif masked[at] == '}':
            level -= 1
            if level == 0:
                return source[begin:at]
    raise ValueError('Unclosed registry function: ' + name)


_INCLUDE = r'include_str!\s*\(\s*"([^"\\\x00-\x1f]+)"\s*\)'
_HELPER = r'(?:bundled_texts|accepted(?:[0-9]{1,4})?_texts)'


def _registered_path(name):
    if not re.fullmatch(r'\.\./scenarios/[a-z0-9][a-z0-9-]*\.json', name):
        raise ValueError('Unsafe or unknown registered pack path')
    return name[3:]


def registered_pack_paths(root, catalog='bundled'):
    """Resolve only the explicitly selected finite static registry chain."""
    entries = {'bundled': 'bundled', 'teaching-review': 'ccna_pilot', 'current': 'ccna_pilot'}
    if catalog not in entries:
        raise ValueError('Unknown catalog profile')
    source, masked, levels = _registry_source(root)
    entry = entries[catalog]
    body = _registry_function(source, masked, levels, entry, False)
    # Keep the old literal-tail contract/tests exactly. A trailing unrelated
    # function cannot make a formerly rejected literal registry gain credit.
    if entry == 'bundled' and re.match(r'\s*load_catalog\s*\(\s*&\s*\[', body):
        return _legacy_registered_pack_paths(root)
    # Only the selected source-owned review entry may use the separately bounded
    # loader. Imported/default registries retain their original100-file contract.
    bundle_tail = re.search(r'load_bundled_catalog\s*\(\s*&\s*(?:texts|' + _HELPER + r'\s*\(\s*\))\s*\)\s*$', body)
    source_owned = catalog in ('current', 'teaching-review') and bundle_tail is not None
    loader = r'load_bundled_catalog' if source_owned else r'load_catalog'
    maximum_paths = 128 if source_owned else 100
    memo = {}
    visiting = set()

    def resolve(name, public=False):
        if not public and not re.fullmatch(_HELPER, name):
            raise ValueError('Unknown static registry helper: ' + name)
        if name in visiting or len(visiting) >= 64:
            raise ValueError('Cyclic or excessive registry chain')
        if name in memo:
            return memo[name]
        visiting.add(name)
        text = body if public else _registry_function(source, masked, levels, name, True)
        base = re.fullmatch(r'\s*vec!\s*\[(.*?)\]\s*', text, re.S) if not public else None
        forward = re.fullmatch(r'\s*' + loader + r'\s*\(\s*&\s*(' + _HELPER + r')\s*\(\s*\)\s*\)\s*', text)
        if base is not None:
            paths = []
            at = 0
            expression = re.compile(r'\s*' + _INCLUDE + r'\s*,')
            while base[1][at:].strip():
                match = expression.match(base[1], at)
                if match is None:
                    raise ValueError('Unknown or malformed static catalog entry')
                paths.append(_registered_path(match[1]))
                at = match.end()
        elif public and forward is not None:
            paths = list(resolve(forward[1]))
        else:
            tail = loader + r'\s*\(\s*&\s*texts\s*\)' if public else r'texts'
            append = re.fullmatch(r'\s*let\s+mut\s+texts\s*=\s*(' + _HELPER +
                                  r')\s*\(\s*\)\s*;\s*(.*?)\s*' + tail + r'\s*', text, re.S)
            if append is None:
                raise ValueError('Unsupported static registry body: ' + name)
            paths = list(resolve(append[1]))
            expression = re.compile(r'\s*texts\s*\.\s*push\s*\(\s*' + _INCLUDE + r'\s*\)\s*;')
            at = 0
            while append[2][at:].strip():
                match = expression.match(append[2], at)
                if match is None:
                    raise ValueError('Unknown or dynamic registry append: ' + name)
                paths.append(_registered_path(match[1]))
                at = match.end()
        if not paths or len(paths) > maximum_paths or len(paths) != len(set(paths)):
            raise ValueError('Empty, duplicate or excessive registered pack paths')
        visiting.remove(name)
        memo[name] = paths
        return paths

    return resolve(entry, True)


def _catalog_document(root, name, remaining=32_000_000):
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
    return json.loads(raw, object_pairs_hook=unique, parse_constant=constant), len(raw.encode('utf-8'))


def _scenario_labs(pack, name):
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
    return labs


def _pack_labs(root, name, remaining=32_000_000):
    document, size = _catalog_document(root, name, remaining)
    return _scenario_labs(document, name), size


def _unit_index(document, name):
    """Validate a bounded metadata schema for reporting only, never lab credit."""
    if (set(document) != {'version', 'units'} or type(document['version']) is not int
            or document['version'] != 1 or not isinstance(document['units'], list)
            or not document['units'] or len(document['units']) > 1000):
        raise ValueError('Malformed unit index: ' + name)
    seen = set()
    for unit in document['units']:
        if (not isinstance(unit, dict)
                or set(unit) != {'id', 'track', 'title', 'summary', 'prerequisites', 'labs'}
                or not isinstance(unit['id'], str) or not re.fullmatch(r'[a-z0-9-]{1,80}', unit['id'])
                or unit['id'] in seen
                or unit['track'] not in ('CCNA', 'LPIC-1', 'LPIC-2', 'Bash/Shell')
                or any(not isinstance(unit[key], str) or not unit[key].strip() or len(unit[key]) > cap
                       for key, cap in [('title', 160), ('summary', 1600)])):
            raise ValueError('Malformed or duplicate unit metadata: ' + name)
        seen.add(unit['id'])
        for key, maximum in [('prerequisites', 64), ('labs', 1000)]:
            items = unit[key]
            if (not isinstance(items, list) or len(items) > maximum or (key == 'labs' and not items)
                    or any(not isinstance(item, str) or not re.fullmatch(r'[a-z0-9-]{1,80}', item) for item in items)
                    or len(items) != len(set(items))):
                raise ValueError('Malformed unit reference list: ' + name)


# Exact source-owned consolidation origins, pinned from verified372R1 content.
# They grant no lab/unit credit. Unknown or changed duplicates remain errors.
_PRESERVED_ORIGINS = {'scenarios/ldap-change-lessons-v1.json': ('scenarios/ldap-administration-v2.json', 2, 'ec3bc07fa29f5574116c50128b6c555d2a6791f37c5695a14d819ba0f85d2c09'), 'scenarios/ldap-utility-lessons-v1.json': ('scenarios/ldap-administration-v2.json', 8, 'dbb3012edbbe67d6426c70ca24dbe68cd3353e8f4869b92cae43b8fe48c5d38a'), 'scenarios/kernel-module-lessons-v1.json': ('scenarios/kernel-module-curriculum-v2.json', 3, '19308826263ba3fafab52a613278febbf4e902bd02a3e794d5743df05f158bbd'), 'scenarios/kernel-module-names-lessons-v1.json': ('scenarios/kernel-module-curriculum-v2.json', 1, '7260bcc30fa84d6fda776117b29f6092f851bacbaafb757e5b7a4e83a500ecb3')}
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v1.json'] = ('scenarios/boot-entry-lessons-v36.json', 3, '5054a3cb15f5193b82a86d660cbbc8d31ebab8dfacc4dddbc4dab52b2f8d6de8')

# Complete five-object origin checked against the admitted378R1 manifest.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v2.json'] = ('scenarios/boot-entry-lessons-v36.json', 5, '59d89cea00a6f499afb64ec8f51620a7414b4e03dd00e14d8e42894dbd4fbdad')

# Whole seven-object origin checked against admitted380R1 BUILD_INFO before changes.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v3.json'] = ('scenarios/boot-entry-lessons-v36.json', 7, 'f43afb791578d4d74ce4e321ad80fd8749396ab2f468dc01bb9d4ef6d3dbc066')

# Eight original boot lessons checked against the verified 381-lab package.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v4.json'] = ('scenarios/boot-entry-lessons-v36.json', 8, '7c27eeb1704b2dff4437c43dc968acf870a97c438ec52a47ab363908a01db7d1')

# Nine original boot lessons checked against the verified 382-lab package.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v5.json'] = ('scenarios/boot-entry-lessons-v36.json', 9, 'be418f8ce8c2d330d408a14b6b694eaaa103f17451219d9de0df1e439bfa628c')
# Exact ten-scenario boot pack retained from the verified 383-lab candidate.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v6.json'] = ('scenarios/boot-entry-lessons-v36.json', 10, '8a870a7f930ae4ceeb600d617d49e81370d717733361ceabd9a0cd6b2fa1d8e6')
# Exact eleven-scenario boot pack retained from the verified 384-lab candidate.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v7.json'] = ('scenarios/boot-entry-lessons-v36.json', 11, '85d3b3284b24cc72ce4d06af59215c84d72117ba217330f239bbae230fb5b919')
# Exact twelve-scenario boot and startup pack retained from the 385-lab package.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v8.json'] = ('scenarios/boot-entry-lessons-v36.json', 12, 'af12d4075a8717e466591b3a108623b9b9053fcda024178e00a931df924f9dae')
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v9.json'] = ('scenarios/boot-entry-lessons-v36.json', 13, 'b1f42d2644ef1b94a6631b5b5c8901f113a6d2a154c4c7ad91c1a3feaa586fdd')
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v10.json'] = ('scenarios/boot-entry-lessons-v36.json', 14, 'db5e9fb874a77d5011932cd6e186f9ac162d2537970622e47bd67f6c22722d23')
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v11.json'] = ('scenarios/boot-entry-lessons-v36.json', 17, '21f5759f7dd17db7110adbab3cb4af2c87072a989a8a95cb55d0a8266968e585')
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v12.json'] = ('scenarios/boot-entry-lessons-v36.json', 19, '92054eb1d63e6af9244aa8d65f90f2b68fd90e0b07474e64b27d4b063fd1299b')
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v13.json'] = ('scenarios/boot-entry-lessons-v36.json', 21, 'bc541744b093822126dfb08923d533a035eda983601a617424ee51066c277414')
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v14.json'] = ('scenarios/boot-entry-lessons-v36.json', 23, '6f39054390e5bd24e72ff0e3c7022636fb3efff70d7646971f1dd1e9312e4bce')
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v15.json'] = ('scenarios/boot-entry-lessons-v36.json', 26, 'aaae4e60f3af48230ab4d807aebd2080e11c29a78b4bfb097cb9a4d74df322a4')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v16.json'] = ('scenarios/boot-entry-lessons-v36.json', 29, 'c29c2ed9068bd1a33a3f83cc98f96f0cc8bb74cd273bb288e88562b36d0b6f4b')
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v17.json'] = ('scenarios/boot-entry-lessons-v36.json', 31, '9da50487efaa06b54b2db93cd773028fdb42e3446155225095ba3ae4e6811acf')

# Exact33-object origin verified against admitted406R1 before the source unit.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v18.json'] = ('scenarios/boot-entry-lessons-v36.json', 33, 'fca07e6fa28af072651a3411c34e116f0b1fe50e042a024eec126ef3c425fa95')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v19.json'] = ('scenarios/boot-entry-lessons-v36.json', 36, '569ff0e5e6c4380181b13418b0a86fd203c9db9fd24ba6265832213c0cc16a52')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v20.json'] = ('scenarios/boot-entry-lessons-v36.json', 39, '68c69c5c4f886b7da8ae96d5f6093c6788f16a949f2b196029640179af26d943')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v21.json'] = ('scenarios/boot-entry-lessons-v36.json', 42, 'ec107c0c0fc77908ef416acb4e87b2ddcc165bac70a5da201b549fae70cb32f8')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v22.json'] = ('scenarios/boot-entry-lessons-v36.json', 44, '4e9ac56c294ac57169a9c320093e01be46afa3f620bc8294513c4a5ce839f9ee')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v23.json'] = ('scenarios/boot-entry-lessons-v36.json', 46, '4d6bb776f71777ed13c9ea60087ae2ca88f2a591ecee2197cd0f2f889414078f')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v24.json'] = ('scenarios/boot-entry-lessons-v36.json', 48, '4befe29f3e775fb884b228c0d31bde47e2774505ed41c8f2ed17d702a3df7b97')

# Exact51-object origin read from verified424R2 retained selected source.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v25.json'] = ('scenarios/boot-entry-lessons-v36.json', 51, '054f5284815c422480fe05042f7ff1d366da27425ce6ae75ea9ed47c5bd05faf')

# Actual canonical53-object values read from verified426R1 retained source.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v26.json'] = ('scenarios/boot-entry-lessons-v36.json', 53, 'de1536e742daecee03c7bc1030ee1cbf715f56d542670cfd7e5134ce28afaa83')

# Exact55-object origin from verified428R1 retained source, not guessed values.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v27.json'] = ('scenarios/boot-entry-lessons-v36.json',55,'2eb6ecf36f447707966ccea2b691d7feef4508e50d32b22e51b1452dfeb1991f')

# Exact57-object tail from verified430R1 retained inputs.
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v28.json'] = ('scenarios/boot-entry-lessons-v36.json',57,'9e182ecfed50464effc078e6d22e67b3eb34c8329e131a3db25da89692cc0c1c')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v29.json'] = ('scenarios/boot-entry-lessons-v36.json', 59, 'f6e947858f6961ec6b2dcc85772065d548a76174cadd41c2fc064301f41d1630')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v30.json'] = ('scenarios/boot-entry-lessons-v36.json', 61, '09478f2f388c963e49cbcf2693bd7c5e45ccd53a9c4e14cbcf9b6b59721ba36e')

_PRESERVED_ORIGINS['scenarios/service-name-lessons-v1.json'] = ('scenarios/boot-entry-lessons-v36.json', 2, 'fc47e14e2628c2e3566b85a2f07c70ca6123f1f39cad570e57197b3c42a5405d')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v31.json'] = ('scenarios/boot-entry-lessons-v36.json', 63, 'b10d4d2a2db1c1dd12d0cc47df627dd1fe0c1e3ddaf2a0b6dab43f04e8f1b6d5')
_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v32.json'] = ('scenarios/boot-entry-lessons-v36.json', 64, '595e364959ed8f52499a99bd1712ebda4c54a8b9529d8d7cb4fb657d4a59cc16')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v33.json'] = ('scenarios/boot-entry-lessons-v36.json',65,'ac9845239c2e790d17f553940dbb31a494f48a2ae886ff0814d3ce86fe8547a6')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v34.json'] = ('scenarios/boot-entry-lessons-v36.json',66,'723eb6b492341e07c8e58db8189733d9d2aa0317d05b46747ecd426b6c651729')

_PRESERVED_ORIGINS['scenarios/boot-entry-lessons-v35.json'] = ('scenarios/boot-entry-lessons-v36.json',68,'560a52d0a1586e4707a07f55b38edba1859b186bd90966001ede0e9d17c0f7f2')

def preserved_authored_pack(name, pack, known_packs):
    if 'scenarios/curriculum-continuation-v19.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v19.json']
        current = list(target.values())
        def ospf_broadcast_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v18.json': (39, 'e3589b5e5a00717f4d3c5bb16eedc6de9f0c1ea229c702d1e13fc269fed535e3'),
                   'scenarios/ccna-ospf-broadcast-lessons-v1.json': (1, 'e54dc36d06ffa9f405475cded47bce860595959e9ac72bf929e743a8874cbb96')}
        if (len(current) != 40 or ospf_broadcast_hash(current[:39]) != origins['scenarios/curriculum-continuation-v18.json'][1]
                or ospf_broadcast_hash(current[39:]) != origins['scenarios/ccna-ospf-broadcast-lessons-v1.json'][1]):
            raise ValueError('broadcast designated roles continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or ospf_broadcast_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('broadcast designated roles continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:39]}
        prior = known_packs.get('scenarios/curriculum-continuation-v18.json')
        if prior is not None and prior != retained:
            raise ValueError('broadcast designated roles continuation conflicts with retained NFS export records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v18.json': retained})
    if 'scenarios/curriculum-continuation-v18.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v18.json']
        current = list(target.values())
        def route_reading_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v17.json': (38, '95da170bf4e8554b0114ea8b1e3e52a7ffa5073ab55c98c9e2dc028097dafd8a'),
                   'scenarios/ccna-route-reading-lessons-v1.json': (1, '0f902a386193a61a3138fcc8a202e3b2912e6e7fd7c72d95839bab324b465777')}
        if (len(current) != 39 or route_reading_hash(current[:38]) != origins['scenarios/curriculum-continuation-v17.json'][1]
                or route_reading_hash(current[38:]) != origins['scenarios/ccna-route-reading-lessons-v1.json'][1]):
            raise ValueError('installed route reading continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or route_reading_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('installed route reading continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:38]}
        prior = known_packs.get('scenarios/curriculum-continuation-v17.json')
        if prior is not None and prior != retained:
            raise ValueError('installed route reading continuation conflicts with retained NFS export records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v17.json': retained})
    if 'scenarios/curriculum-continuation-v17.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v17.json']
        current = list(target.values())
        def nfs_export_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v16.json': (37, '8ac5e49758a89715286fa19dc055e180a781ef40a49df7747183c7534a7bb8e7'),
                   'scenarios/nfs-export-operations-v1.json': (1, '9de3beb0c93c229c23a9398470a524a6856408f8c177c5c3ccb6e4805b4e35d1')}
        if (len(current) != 38 or nfs_export_hash(current[:37]) != origins['scenarios/curriculum-continuation-v16.json'][1]
                or nfs_export_hash(current[37:]) != origins['scenarios/nfs-export-operations-v1.json'][1]):
            raise ValueError('NFS export operations continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or nfs_export_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('NFS export operations continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:37]}
        prior = known_packs.get('scenarios/curriculum-continuation-v16.json')
        if prior is not None and prior != retained:
            raise ValueError('NFS export operations continuation conflicts with retained VLAN and gateway records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v16.json': retained})
    if 'scenarios/curriculum-continuation-v16.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v16.json']
        current = list(target.values())
        def vlan_gateway_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v15.json': (35, '26305daab87f6d013493ff30996d871c9dd22d26aa09f1e23eeea6a028eada24'),
                   'scenarios/ccna-vlan-gateway-lessons-v1.json': (2, 'd55fedb6239054acb0a8397551ce72eb1c4c09abf0851d4cd82f3671ad98d334')}
        if (len(current) != 37 or vlan_gateway_hash(current[:35]) != origins['scenarios/curriculum-continuation-v15.json'][1]
                or vlan_gateway_hash(current[35:]) != origins['scenarios/ccna-vlan-gateway-lessons-v1.json'][1]):
            raise ValueError('VLAN gateway repair continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or vlan_gateway_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('VLAN gateway repair continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:35]}
        prior = known_packs.get('scenarios/curriculum-continuation-v15.json')
        if prior is not None and prior != retained:
            raise ValueError('VLAN gateway repair continuation conflicts with retained service and DNS security records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v15.json': retained})
    if 'scenarios/curriculum-continuation-v15.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v15.json']
        current = list(target.values())
        def host_dns_security_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v14.json': (33, 'ec7377e927e5b74d9839143e6219e82e5a16fa1e6ca7b006860e7cb1e6277a26'),
                   'scenarios/host-and-dns-security-lessons-v1.json': (2, '73a36ce044ae33fe123c7d4bb4363a1670e4df86dce263553f318bf143bec241')}
        if (len(current) != 35 or host_dns_security_hash(current[:33]) != origins['scenarios/curriculum-continuation-v14.json'][1]
                or host_dns_security_hash(current[33:]) != origins['scenarios/host-and-dns-security-lessons-v1.json'][1]):
            raise ValueError('Host and DNS security continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or host_dns_security_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Host and DNS security continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:33]}
        prior = known_packs.get('scenarios/curriculum-continuation-v14.json')
        if prior is not None and prior != retained:
            raise ValueError('Host and DNS security continuation conflicts with retained site connectivity records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v14.json': retained})
    if 'scenarios/curriculum-continuation-v14.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v14.json']
        current = list(target.values())
        def site_connectivity_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v13.json': (31, '3098949892e52f9ca9d87bd3bf35504c3991cfc9c165f94c1bc6029313d92de3'),
                   'scenarios/site-connectivity-lessons-v1.json': (2, 'e67b615823e71f8cccafcb2df3d26f9e2ce65ff66e3b1be53b341e24496ebf2d')}
        if (len(current) != 33 or site_connectivity_hash(current[:31]) != origins['scenarios/curriculum-continuation-v13.json'][1]
                or site_connectivity_hash(current[31:]) != origins['scenarios/site-connectivity-lessons-v1.json'][1]):
            raise ValueError('Site connectivity continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or site_connectivity_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Site connectivity continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:31]}
        prior = known_packs.get('scenarios/curriculum-continuation-v13.json')
        if prior is not None and prior != retained:
            raise ValueError('Site connectivity continuation conflicts with retained network evidence review records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v13.json': retained})
    if 'scenarios/curriculum-continuation-v13.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v13.json']
        current = list(target.values())
        def network_evidence_review_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v12.json': (29, '7ef447fbeac9f908b26eda98db93b409cf1a676a928402f6d1e80e90741c73e6'),
                   'scenarios/network-evidence-review-lessons-v1.json': (2, 'c9edda6689dab806ae10a78934128ca173a6426c848313a468199e50d56e2fb3')}
        if (len(current) != 31 or network_evidence_review_hash(current[:29]) != origins['scenarios/curriculum-continuation-v12.json'][1]
                or network_evidence_review_hash(current[29:]) != origins['scenarios/network-evidence-review-lessons-v1.json'][1]):
            raise ValueError('Network evidence review continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or network_evidence_review_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Network evidence review continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:29]}
        prior = known_packs.get('scenarios/curriculum-continuation-v12.json')
        if prior is not None and prior != retained:
            raise ValueError('Network evidence review continuation conflicts with retained staged-network records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v12.json': retained})
    if 'scenarios/curriculum-continuation-v12.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v12.json']
        current = list(target.values())
        def staged_network_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v11.json': (27, '2836a1ec6502109ea93c615c73efb8c9d129f6252fa9df77c04fbe3ff321fbaa'),
                   'scenarios/staged-network-lessons-v1.json': (2, '4adba2a362f30ffcd5c2a22b4ef0fba3463c2fa11e5d8ceb3a01cc6b1aef0bee')}
        if (len(current) != 29 or staged_network_hash(current[:27]) != origins['scenarios/curriculum-continuation-v11.json'][1]
                or staged_network_hash(current[27:]) != origins['scenarios/staged-network-lessons-v1.json'][1]):
            raise ValueError('Staged network changes continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or staged_network_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Staged network changes continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:27]}
        prior = known_packs.get('scenarios/curriculum-continuation-v11.json')
        if prior is not None and prior != retained:
            raise ValueError('Staged network changes continuation conflicts with retained credential and VPN records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v11.json': retained})
    if 'scenarios/curriculum-continuation-v11.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v11.json']
        current = list(target.values())
        def credentials_vpn_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v10.json': (25, '06db552f603b3ba13b102d8fd435e061bf9af2ddeefbf0378f9938eed6d70275'),
                   'scenarios/credentials-vpn-lessons-v1.json': (2, '74164f3780ee440135ef7b8a376905c5a0ebdb5d05629fd50f222e17211c906a')}
        if (len(current) != 27 or credentials_vpn_hash(current[:25]) != origins['scenarios/curriculum-continuation-v10.json'][1]
                or credentials_vpn_hash(current[25:]) != origins['scenarios/credentials-vpn-lessons-v1.json'][1]):
            raise ValueError('Credentials and VPN continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or credentials_vpn_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Credentials and VPN continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:25]}
        prior = known_packs.get('scenarios/curriculum-continuation-v10.json')
        if prior is not None and prior != retained:
            raise ValueError('Credentials and VPN continuation conflicts with retained security investigation records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v10.json': retained})
    if 'scenarios/curriculum-continuation-v10.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v10.json']
        current = list(target.values())
        def investigation_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v9.json': (23, 'a7e353a13e6a4771ce53734b5eac96eec5594fc07f1170a8b1a1373c3bfbdd12'),
                   'scenarios/security-investigation-lessons-v1.json': (2, '7a1adbd4a3708e2f88602a5e1f78bb2df982fc8264adee2e08bf12a9eb4ddaf5')}
        if (len(current) != 25 or investigation_hash(current[:23]) != origins['scenarios/curriculum-continuation-v9.json'][1]
                or investigation_hash(current[23:]) != origins['scenarios/security-investigation-lessons-v1.json'][1]):
            raise ValueError('Security investigation continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or investigation_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Security investigation continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:23]}
        prior = known_packs.get('scenarios/curriculum-continuation-v9.json')
        if prior is not None and prior != retained:
            raise ValueError('Security investigation continuation conflicts with retained desktop workstation records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v9.json': retained})
    if 'scenarios/curriculum-continuation-v9.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v9.json']
        current = list(target.values())
        def desktop_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v8.json': (20, 'e6c1534f234bfb709664f276bf7066e67a8d935b6503dd8ec1d466da1684f6a0'),
                   'scenarios/desktop-profile-lessons-v1.json': (3, '44265527e089ef63b73949c5669caa81fe8155bc6874671a9c6969871ae539fe')}
        if (len(current) != 23 or desktop_hash(current[:20]) != origins['scenarios/curriculum-continuation-v8.json'][1]
                or desktop_hash(current[20:]) != origins['scenarios/desktop-profile-lessons-v1.json'][1]):
            raise ValueError('Desktop profile continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or desktop_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Desktop profile continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:20]}
        prior = known_packs.get('scenarios/curriculum-continuation-v8.json')
        if prior is not None and prior != retained:
            raise ValueError('Desktop profile continuation conflicts with retained Apache HTTPS records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v8.json': retained})
    if 'scenarios/curriculum-continuation-v8.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v8.json']
        current = list(target.values())
        def apache_https_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v7.json': (18, 'bc1b60652a504328bd8df473ceae5cb5145e1347a26e4b25d1755d97975c9a25'),
                   'scenarios/apache-https-lessons-v1.json': (2, 'c7a055b8926bd5f0e77bfecb5f4e497c65a8338a471c82a17c239239dfd14ca8')}
        if (len(current) != 20 or apache_https_hash(current[:18]) != origins['scenarios/curriculum-continuation-v7.json'][1]
                or apache_https_hash(current[18:]) != origins['scenarios/apache-https-lessons-v1.json'][1]):
            raise ValueError('Apache HTTPS continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or apache_https_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Apache HTTPS continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:18]}
        prior = known_packs.get('scenarios/curriculum-continuation-v7.json')
        if prior is not None and prior != retained:
            raise ValueError('Apache HTTPS continuation conflicts with retained ATA SMART records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v7.json': retained})
    if 'scenarios/curriculum-continuation-v7.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v7.json']
        current = list(target.values())
        def ata_smart_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v6.json': (16, '201b15a052caa1c5e0eed77c50c76b25233a1404a0553e2f83b191a5455029e3'),
                   'scenarios/ata-smart-lessons-v1.json': (2, 'b771833197372df3adca41cc2bd767c395876b3688e4f6c6517608457bdc4bda')}
        if (len(current) != 18 or ata_smart_hash(current[:16]) != origins['scenarios/curriculum-continuation-v6.json'][1]
                or ata_smart_hash(current[16:]) != origins['scenarios/ata-smart-lessons-v1.json'][1]):
            raise ValueError('ATA SMART continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or ata_smart_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('ATA SMART continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:16]}
        prior = known_packs.get('scenarios/curriculum-continuation-v6.json')
        if prior is not None and prior != retained:
            raise ValueError('ATA SMART continuation conflicts with retained iSCSI records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v6.json': retained})
    if 'scenarios/curriculum-continuation-v6.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v6.json']
        current = list(target.values())
        def iscsi_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v5.json': (14, '77a9c0b58568d3f39ce6478435d8e5c1c00170130f6cbd7f22eadd4cedceb076'),
                   'scenarios/iscsi-lessons-v1.json': (2, 'f18b7db90a5ec1d89bbe5f5f87163f0d939fa84f84cc26348363f3d22c2b327c')}
        if (len(current) != 16 or iscsi_hash(current[:14]) != origins['scenarios/curriculum-continuation-v5.json'][1]
                or iscsi_hash(current[14:]) != origins['scenarios/iscsi-lessons-v1.json'][1]):
            raise ValueError('iSCSI continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or iscsi_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('iSCSI continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:14]}
        prior = known_packs.get('scenarios/curriculum-continuation-v5.json')
        if prior is not None and prior != retained:
            raise ValueError('iSCSI continuation conflicts with retained network boot records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v5.json': retained})
    if 'scenarios/curriculum-continuation-v5.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v5.json']
        current = list(target.values())
        def pxe_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v4.json': (12, '4799423fff1a18207d63cdd0aac2320cf7e833c1145a1d1068f461a3641631fd'),
                   'scenarios/pxe-lessons-v1.json': (2, '1f629976e9914b308e58b719fa141e1db03b823de84a366eb25682ff8ee88b07')}
        if (len(current) != 14 or pxe_hash(current[:12]) != origins['scenarios/curriculum-continuation-v4.json'][1]
                or pxe_hash(current[12:]) != origins['scenarios/pxe-lessons-v1.json'][1]):
            raise ValueError('Network boot continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or pxe_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Network boot continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:12]}
        prior = known_packs.get('scenarios/curriculum-continuation-v4.json')
        if prior is not None and prior != retained:
            raise ValueError('Network boot continuation conflicts with retained early-userspace records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v4.json': retained})
    if 'scenarios/curriculum-continuation-v4.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v4.json']
        current = list(target.values())
        def initramfs_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v3.json': (10, 'eb8be0efe2f5cec3b12ec7b81093dd903f7921db41807ec1856e3f99761d4cb3'),
                   'scenarios/initramfs-lessons-v1.json': (2, 'ef0eccf64ae5cff8b4e821afb3667f596b31de78d6677b90f7ddf96532f100ed')}
        if (len(current) != 12 or initramfs_hash(current[:10]) != origins['scenarios/curriculum-continuation-v3.json'][1]
                or initramfs_hash(current[10:]) != origins['scenarios/initramfs-lessons-v1.json'][1]):
            raise ValueError('Early-userspace continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or initramfs_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Early-userspace continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:10]}
        prior = known_packs.get('scenarios/curriculum-continuation-v3.json')
        if prior is not None and prior != retained:
            raise ValueError('Early-userspace continuation conflicts with retained DKMS records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v3.json': retained})
    if 'scenarios/curriculum-continuation-v3.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v3.json']
        current = list(target.values())
        def dkms_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v2.json': (8, '8e6777bdabec1ac82e5603c5bc6694a4678787bc2c1bde459437a24100fc41de'),
                   'scenarios/dkms-lessons-v1.json': (2, '9551b2f031e5440af002ee376b87c4d4d7bcbeb1d4bd8f219c5e83dba893e33b')}
        if (len(current) != 10 or dkms_hash(current[:8]) != origins['scenarios/curriculum-continuation-v2.json'][1]
                or dkms_hash(current[8:]) != origins['scenarios/dkms-lessons-v1.json'][1]):
            raise ValueError('DKMS continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or dkms_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('DKMS continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:8]}
        prior = known_packs.get('scenarios/curriculum-continuation-v2.json')
        if prior is not None and prior != retained:
            raise ValueError('DKMS continuation conflicts with retained kernel records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v2.json': retained})
    if 'scenarios/curriculum-continuation-v2.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v2.json']
        current = list(target.values())
        def kernel_hash(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        origins = {'scenarios/curriculum-continuation-v1.json': (6, 'bc1d8085ca4f7416eacfeb9298602fb6a09ae03dda753b1aa22fcf6e0bb9cb35'),
                   'scenarios/kernel-build-lessons-v1.json': (2, 'bd111bc64ffdd8524224d9e1e89e3a93e2dd843160cd6d305b647c322390d870')}
        if (len(current) != 8 or kernel_hash(current[:6]) != origins['scenarios/curriculum-continuation-v1.json'][1]
                or kernel_hash(current[6:]) != origins['scenarios/kernel-build-lessons-v1.json'][1]):
            raise ValueError('Kernel continuation consolidation changed exact predecessor or additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or kernel_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('Kernel continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:6]}
        prior = known_packs.get('scenarios/curriculum-continuation-v1.json')
        if prior is not None and prior != retained:
            raise ValueError('Kernel continuation conflicts with retained power/detection records')
        known_packs = dict(known_packs, **{'scenarios/curriculum-continuation-v1.json': retained})
    # A closed six-record continuation retains the previous four detection
    # records exactly. Bridge only those verified values into the existing
    # predecessor validation; arbitrary duplicate drafts remain rejected.
    if 'scenarios/curriculum-continuation-v1.json' in known_packs:
        target = known_packs['scenarios/curriculum-continuation-v1.json']
        current = list(target.values())
        def continuation_hash(rows):
            return hashlib.sha256(json.dumps(rows,sort_keys=True,separators=(',', ':'),ensure_ascii=False).encode()).hexdigest()
        origins = {
            'scenarios/security-detection-lessons-v2.json': (4, '73f4125ce0c79bd45d40c4843ceb3d922de9b495aaab504732977186b4e729c7'),
            'scenarios/poe-lessons-v1.json': (2, 'f7a071a289dc96f4915dafbc01f3127c93f088b4bbc3c48f11f7265385cfbf22'),
        }
        if (len(current) != 6
                or continuation_hash(current[:4]) != origins['scenarios/security-detection-lessons-v2.json'][1]
                or continuation_hash(current[4:]) != origins['scenarios/poe-lessons-v1.json'][1]):
            raise ValueError('PoE continuation consolidation changed exact predecessor or authored additions')
        if name in origins:
            count, expected = origins[name]
            if (len(pack) != count or continuation_hash(list(pack.values())) != expected
                    or any(target.get(identity) != lab for identity, lab in pack.items())):
                raise ValueError('PoE continuation origin differs from authored lessons')
            return True
        retained = {lab['id']: lab for lab in current[:4]}
        prior = known_packs.get('scenarios/security-detection-lessons-v2.json')
        if prior is not None and prior != retained:
            raise ValueError('PoE continuation conflicts with retained detection records')
        known_packs = dict(known_packs, **{'scenarios/security-detection-lessons-v2.json': retained})
    if 'scenarios/security-detection-lessons-v2.json' in known_packs:
        target = known_packs['scenarios/security-detection-lessons-v2.json']
        fingerprints = {'scenarios/auth-filter-lessons-v1.json': (2, 'dc890e4a0163eba6f90e494d4bc492db56049e286606cff352c1c0c0e0d8a248'), 'scenarios/ids-rule-lessons-v1.json': (2, '3148ccbf8f81a2d970efe32ca264694afc7edbf13d895dd9db26fb6b32b27982')}
        def fingerprint(rows):
            return hashlib.sha256(json.dumps(rows,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
        current = list(target.values())
        if len(current)!=4 or fingerprint(current[:2])!=fingerprints['scenarios/auth-filter-lessons-v1.json'][1] or fingerprint(current[2:])!=fingerprints['scenarios/ids-rule-lessons-v1.json'][1]:
            raise ValueError('Security detection consolidation changed exact originals or authored additions')
        if name in fingerprints:
            count,digest=fingerprints[name]
            if len(pack)!=count or fingerprint(list(pack.values()))!=digest or any(target.get(identity)!=lab for identity,lab in pack.items()):
                raise ValueError('Security detection origin differs from authored lessons')
            return True
    # Closed exact-value LDAP consolidation frees one registry slot; no budget
    # increase and no semantic change to the prior twelve activities.
    if 'scenarios/ldap-administration-v3.json' in known_packs:
        target = 'scenarios/ldap-administration-v3.json'
        current = list(known_packs[target].values())
        fingerprints = {
            'scenarios/ldap-client-lessons-v1.json': (2, '64be6486d9902caa1142599d7baccde22d2adb4e84b684fcf98bf48c021a8273'),
            'scenarios/ldap-administration-v2.json': (10, 'b6f05e475f0db160bec0c5cd1f07ecbb9357a68bd0df801259137d4e26900821'),
        }
        def canonical(rows):
            return hashlib.sha256(json.dumps(rows, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()
        if (len(current) != 12 or canonical(current[:2]) != fingerprints['scenarios/ldap-client-lessons-v1.json'][1]
                or canonical(current[2:]) != fingerprints['scenarios/ldap-administration-v2.json'][1]):
            raise ValueError('LDAP client/admin consolidation changed exact predecessors')
        if name in fingerprints:
            count, expected = fingerprints[name]
            if len(pack) != count or canonical(list(pack.values())) != expected:
                raise ValueError('Changed preserved LDAP predecessor')
            if any(known_packs[target].get(identity) != lab for identity, lab in pack.items()):
                raise ValueError('LDAP predecessor differs from consolidated activity')
            return True
    if name == 'scenarios/dns-forward-v1.json' and 'scenarios/dns-forward-v2.json' in known_packs:
        canonical = json.dumps(list(pack.values()), sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        if len(pack) != 6 or hashlib.sha256(canonical).hexdigest() != '494747dd4454c8b06d1de2f8b13154ea2e3e6bc605c0ab787ccb88269f8fefe5':
            raise ValueError('Changed original six authoritative DNS lessons')
        current = known_packs['scenarios/dns-forward-v2.json']
        if set(current) != set(pack):
            raise ValueError('DNS lookup expansion changed catalog IDs')
        for identity, old in pack.items():
            expanded = json.loads(json.dumps(current[identity]))
            if identity in {'lpic2-dns-dualstack','lpic2-dns-aliases','lpic2-dns-secondary'}:
                endpoints = [{'address':'192.0.2.53','served':'Primary'}]
                if identity == 'lpic2-dns-secondary': endpoints.append({'address':'192.0.2.54','served':'Secondary'})
                capability = expanded['dns_teaching'].pop('lookup',None)
                if capability != {'assignment':{'endpoints':endpoints},'example':{'endpoints':endpoints}}:
                    raise ValueError('Unreviewed authoritative lookup endpoints')
                if expanded['sources'].pop() != 'https://bind9.readthedocs.io/en/stable/manpages.html':
                    raise ValueError('Unreviewed DNS utility reference')
            if expanded != old:
                raise ValueError('DNS lookup expansion changed original state, goals or teaching fixture')
        return True
    # One explicit read-only capability expansion; every old scenario field and
    # all independent grading/initial work remain exact. Historical v1 stays pinned.
    if name == 'scenarios/linux-host-lessons-v1.json' and 'scenarios/linux-host-lessons-v2.json' in known_packs:
        canonical = json.dumps(list(pack.values()), sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        if len(pack) != 4 or hashlib.sha256(canonical).hexdigest() != '72a6db11847b81f60eaefb3c0e37ec25e7324101e3e67dc8d9544a3db65c25eb':
            raise ValueError('Changed original four host lessons')
        current = known_packs['scenarios/linux-host-lessons-v2.json']
        if set(current) != set(pack):
            raise ValueError('Expanded host lessons changed catalog IDs')
        for identity, old in pack.items():
            expanded = json.loads(json.dumps(current[identity]))
            if identity == 'lpic1-host-resolver':
                definition = expanded['goal']['definition']
                capability = definition.pop('direct_dns', None)
                expected = {scope: {'servers': [r['address'] for r in definition[world]['profile']['resolvers']]}
                            for scope, world in [('initial','initial'), ('example','example')]}
                if capability != expected:
                    raise ValueError('Unreviewed direct-query capability')
            if expanded != old:
                raise ValueError('Expanded host lesson changed original outcomes or state')
        return True
    # Exact continuation consolidation; no collection limit is increased.
    current_ssh_origins = {'scenarios/boot-entry-lessons-v36.json': ('scenarios/boot-entry-lessons-v37.json', 69, 'd0c41a0cba7e8103743606b9011a6460254cbadf9d355e51197ea5940e99abde'), 'scenarios/ssh-access-lessons-v1.json': ('scenarios/boot-entry-lessons-v37.json', 3, 'bc17fa954392c73067900bd1e17fd2c49c923d3026017b9d4067365daa670bd4')}
    current_modal_origins = {'scenarios/boot-entry-lessons-v37.json': ('scenarios/boot-entry-lessons-v38.json', 72, 'f4c1efe6e6cae87099c8b48810ace8ef625f601f99a9f1094e507eefdc4d7614'), 'scenarios/modal-editing-lessons-v1.json': ('scenarios/boot-entry-lessons-v38.json', 3, '117374d09e59f34dcdb34f98657aa062dad032fb9faeeb25faa9988be71e0b45')}
    current_editor_handover_origins = {'scenarios/boot-entry-lessons-v38.json': ('scenarios/boot-entry-lessons-v39.json', 75, '6e553a1b1feb6599b96e48f128ae882c8d5b9e63deb8e452dd2ebad45e0ccf10'), 'scenarios/editor-handover-lessons-v1.json': ('scenarios/boot-entry-lessons-v39.json', 1, 'aa4549beee73ba869d4135e0322c5c2906593b8941dee816aefdaa99bde4c4f5')}
    current_network_transfer_origins = {'scenarios/boot-entry-lessons-v39.json': ('scenarios/boot-entry-lessons-v40.json', 76, 'd24951f2918730b8f2fa501a667dac9faf5e028b6c4b242bfc273564e4b0bee1'), 'scenarios/network-transfers-lessons-v1.json': ('scenarios/boot-entry-lessons-v40.json', 3, 'ae61099acf04f8a1f0da990359ac95e30253bccff513f74bf1398689d224ecf6')}
    current_campus_origins = {'scenarios/boot-entry-lessons-v40.json': ('scenarios/boot-entry-lessons-v41.json', 79, '0187ba740b9cc65b203b702623ae1c3bbf9e4083c6f5c394e026cca598fdd078'), 'scenarios/campus-architecture-lessons-v1.json': ('scenarios/boot-entry-lessons-v41.json', 2, 'fd7a987719d50ae9f61524ae2da87f0c1907e0fc500c7a59a549fd4b0a60d416')}
    current_leaf_spine_origins = {'scenarios/boot-entry-lessons-v41.json': ('scenarios/boot-entry-lessons-v42.json', 81, '7d8ed83308db9dfe5f75cb7d861ea6a21f600fdf7278c2aa681c7901de8c76c7'), 'scenarios/leaf-spine-lessons-v1.json': ('scenarios/boot-entry-lessons-v42.json', 1, '739cae380e188a1ee275fd32280f613ae75177e818543cf8035acff418020e07')}
    current_samba_origins = {'scenarios/boot-entry-lessons-v42.json': ('scenarios/boot-entry-lessons-v43.json', 82, '4350e6e087f174d06d954ac82deaf1d14fa914bf40eff32f8bc1aa1bfbc987e3'), 'scenarios/samba-standalone-lessons-v1.json': ('scenarios/boot-entry-lessons-v43.json', 3, '3a80fd3451ef1220380e69728f96d36c8250ac8ed9da346e56b0081de86fd0ef')}
    current_virtual_isolation_origins = {'scenarios/boot-entry-lessons-v43.json': ('scenarios/boot-entry-lessons-v44.json', 85, '139602bb19769f50fff5a3455f95ea5d771f51090a904ae800b5a22087d1eb47'), 'scenarios/virtual-isolation-lessons-v1.json': ('scenarios/boot-entry-lessons-v44.json', 3, '51dded0975922040bbd20e6e535888c68e54420fedda5b6d5b1c7c2f10ff1966')}
    specification = current_leaf_spine_origins.get(name,current_campus_origins.get(name,current_network_transfer_origins.get(name,current_editor_handover_origins.get(name,current_modal_origins.get(name,current_ssh_origins.get(name, _PRESERVED_ORIGINS.get(name)))))))
    specification = current_samba_origins.get(name,specification)
    current_guest_image_origins = {'scenarios/boot-entry-lessons-v44.json': ('scenarios/boot-entry-lessons-v45.json', 88, '50ff811e236fdcbb8db9fdce61210df93d7a592e432b3d3b1c94fc75c6f56fc0'), 'scenarios/guest-driver-lessons-v1.json': ('scenarios/boot-entry-lessons-v45.json', 3, 'c6da8aff38d6e92e9c155a4fa3b092bcc697cb3b25746a7d1551b9dc3b6ee290')}
    specification = current_virtual_isolation_origins.get(name,specification)
    current_smtp_origins = {'scenarios/boot-entry-lessons-v45.json': ('scenarios/boot-entry-lessons-v46.json', 91, '07c0e62addea159495d39e4143bcad8b69b981cfeb818249c135b14e67b42806'), 'scenarios/smtp-routing-lessons-v1.json': ('scenarios/boot-entry-lessons-v46.json', 3, 'a36293c4035140687f29f67e79c671198c5ca4e97169db0899602323e3e9650e')}
    specification = current_guest_image_origins.get(name,specification)
    specification = current_smtp_origins.get(name,specification)
    current_sieve_origins = {'scenarios/boot-entry-lessons-v46.json': ('scenarios/boot-entry-lessons-v47.json', 94, '775bdb173c0f9a899109e9f48e1ec7a1c48f2eda85f9edc27bdf25d505cba512'), 'scenarios/sieve-filter-lessons-v1.json': ('scenarios/boot-entry-lessons-v47.json', 3, '46d5115f57688e2f96cbd7bedc676e81cf96ac774066ebe3f0d301593c2041c7')}
    specification = current_sieve_origins.get(name,specification)
    current_address_origins = {'scenarios/boot-entry-lessons-v47.json': ('scenarios/boot-entry-lessons-v48.json', 97, 'ccff23d9bb769ac8863a2e7cce118b4724f982b21fc3f7f96db0c41290205abf'), 'scenarios/sieve-address-lessons-v1.json': ('scenarios/boot-entry-lessons-v48.json', 2, 'c78c549d013cafad452a20159aff63eb2add1e06c6adc39a14b49b71018a8721')}
    specification = current_address_origins.get(name,specification)
    current_delivery_origins = {'scenarios/boot-entry-lessons-v48.json': ('scenarios/boot-entry-lessons-v49.json', 99, 'd7b183a8a2f29ee58c8fd034e124a5ec28b92bc8ee214a2375c0293a1ee995e5'), 'scenarios/sieve-delivery-lessons-v1.json': ('scenarios/boot-entry-lessons-v49.json', 3, 'd72b70d8c27b0f9095022b378fb30c499a8a86b1aa5574c875621ed2f4a6cad4')}
    specification = current_delivery_origins.get(name,specification)
    current_migration_origins = {'scenarios/boot-entry-lessons-v50.json': ('scenarios/boot-entry-lessons-v51.json', 104, 'f682201152ff347a804cc5a68fad7848b667be0a83b4d88456e8b16f02212266'), 'scenarios/procmail-migration-lessons-v1.json': ('scenarios/boot-entry-lessons-v51.json', 1, '798f7623c72e3f2e950fdd490cbc19ca2decb12c1d03b29f90e996f39174cb14')}
    specification = current_migration_origins.get(name,specification)
    current_mailbox_origins = {'scenarios/boot-entry-lessons-v51.json': ('scenarios/boot-entry-lessons-v52.json', 105, 'c404cb86e05f7ea07418f5b72a9d1d9db0d8e81085717fe5731a37f7161abc4b'), 'scenarios/mailbox-access-lessons-v1.json': ('scenarios/boot-entry-lessons-v52.json', 3, '0994bdb4a41acdf576497bd4ec3a89946f70b8e0ce82ddb40e79e83e08f5baab')}
    specification = current_mailbox_origins.get(name,specification)
    current_mail_admin_origins = {'scenarios/boot-entry-lessons-v52.json': ('scenarios/boot-entry-lessons-v53.json', 108, '66ed33a8b1c086ad3698f90782f99b8d4a925769071701ad3b1d0863f4e84921'), 'scenarios/mailbox-administration-lessons-v1.json': ('scenarios/boot-entry-lessons-v53.json', 1, '4b321604b303bc86c55161be2f1ddd88c4efed3751928fccfbce103aa0918562')}
    specification = current_mail_admin_origins.get(name,specification)
    current_router_origins = {'scenarios/boot-entry-lessons-v53.json': ('scenarios/boot-entry-lessons-v54.json', 109, '29fd7440f675304975fb935b277305d28defdc102896086195c0a7e414b1eeee'), 'scenarios/linux-router-lessons-v1.json': ('scenarios/boot-entry-lessons-v54.json', 3, '5c6a721867873c13f0637cfe2c042219d469770eb4e19aeaa6614cd036dc29bb')}
    specification = current_router_origins.get(name,specification)
    current_flow_origins = {'scenarios/boot-entry-lessons-v54.json': ('scenarios/boot-entry-lessons-v55.json', 112, 'c91f1b3fd8bd6edebe00f6667640ebbdd7ef987f96d90495f6b1f7749da3e6dd'), 'scenarios/linux-flow-lessons-v1.json': ('scenarios/boot-entry-lessons-v55.json', 2, '50b2d7ea6ae8a6f6565629dc9da9b6b0a6e2fc235873556a596c871adf36fd40')}
    specification = current_flow_origins.get(name,specification)
    current_ipv6_router_origins = {'scenarios/boot-entry-lessons-v55.json': ('scenarios/boot-entry-lessons-v56.json', 114, '8ecf5cc0f61fbd57f80fdc1a569045bed80df05fd9533595bea14e02bf70e1af'), 'scenarios/linux-v6-lessons-v1.json': ('scenarios/boot-entry-lessons-v56.json', 2, 'effe39caa705ddb363cecc7093cabc26fc695ffbaa81dfd5e94980d0b30364bc')}
    specification = current_ipv6_router_origins.get(name,specification)
    current_service_router_origins = {'scenarios/boot-entry-lessons-v56.json': ('scenarios/boot-entry-lessons-v57.json', 116, '08ff743fe8d9f41ef0499cce0de5ff8596abe1326918594787f249f19da8ac30'), 'scenarios/linux-services-router-lessons-v1.json': ('scenarios/boot-entry-lessons-v57.json', 1, 'ad771b282ebd26c160ffe32b98bce32165325beebff72daae78367a91c2f967e')}
    specification = current_service_router_origins.get(name,specification)
    current_v6_admin_origins = {'scenarios/boot-entry-lessons-v57.json': ('scenarios/boot-entry-lessons-v58.json', 117, '1836c3dbd3a2de318125f54c56bd17c935e1a82daed64e6cf71b82d579edc9de'), 'scenarios/linux-v6-admin-lessons-v1.json': ('scenarios/boot-entry-lessons-v58.json', 3, '81905fae2fcdee496e928fa3fcafe757d447019517161f8cb4fea91056654520')}
    specification = current_v6_admin_origins.get(name,specification)
    current_v6_startup_origins = {'scenarios/boot-entry-lessons-v58.json': ('scenarios/boot-entry-lessons-v59.json', 120, '82436d753f445f4394e76d136c5b21c1fb926ad3c442d9ce6ab100847bf6e105'), 'scenarios/linux-v6-startup-lessons-v1.json': ('scenarios/boot-entry-lessons-v59.json', 1, 'c3f5a71763adea1f41b144a87fe22450b88ad4fac568588bd412462ebdd3fb31')}
    specification = current_v6_startup_origins.get(name,specification)
    current_ftp_identity_origins = {'scenarios/boot-entry-lessons-v59.json': ('scenarios/boot-entry-lessons-v60.json', 121, '6dfada3edc4014108b49bb6276aa34ca237d0930488fc84670e066ef2eff18a9'), 'scenarios/ftp-identity-lessons-v1.json': ('scenarios/boot-entry-lessons-v60.json', 3, '911262bca0a63e695bc61ae008b92e44b3db8cf2d481bfe69d25766df849fa29')}
    specification = current_ftp_identity_origins.get(name,specification)
    current_service_exposure_origins = {'scenarios/boot-entry-lessons-v60.json': ('scenarios/boot-entry-lessons-v61.json', 124, '933e0ba153ae6e7cf24f052dbcd869c908c2d05c8537f6838f8159de9a89b7e3'), 'scenarios/service-exposure-lessons-v1.json': ('scenarios/boot-entry-lessons-v61.json', 2, 'f930237686320674e4de57c391ca0997d87386e6f14448c11000a75bb2c48be9')}
    specification = current_service_exposure_origins.get(name,specification)
    current_vpn_point_origins = {'scenarios/boot-entry-lessons-v61.json': ('scenarios/boot-entry-lessons-v62.json', 126, '35327669895234d1e68866f131d39c0011fab7c5910266f5a8e0712b1aba35e6'), 'scenarios/vpn-point-lessons-v1.json': ('scenarios/boot-entry-lessons-v62.json', 2, 'd0e0d3b05505d7a939f677e36b3a86cc6002a51d6ee9b0698d28e0e4da34faf0')}
    specification = current_vpn_point_origins.get(name,specification)
    current_advisory_patch_origins = {'scenarios/boot-entry-lessons-v62.json': ('scenarios/boot-entry-lessons-v63.json', 128, '16ec4a2d74c4e342420e84a4ae70f0178c495dbc241f98a90491a6282c06c504'), 'scenarios/advisory-patch-lessons-v1.json': ('scenarios/boot-entry-lessons-v63.json', 2, '1c4a55471a58f1e9fc0314d697d7efc86a6be8be4a16322b9bede8d48b453eb4')}
    specification = current_advisory_patch_origins.get(name,specification)
    current_vacation_origins = {'scenarios/boot-entry-lessons-v49.json': ('scenarios/boot-entry-lessons-v50.json', 102, 'f275776ebb784281980d23b6d2d4d95655469dc47fb54c160bf24ecb4dad3758'), 'scenarios/sieve-vacation-lessons-v1.json': ('scenarios/boot-entry-lessons-v50.json', 2, '0647b292dad5d2a870b4cb00a5649fcfab96371ba7802b122a051f5f154d70d6')}
    specification = current_vacation_origins.get(name,specification)
    if specification and specification[0] == "scenarios/boot-entry-lessons-v36.json" and "scenarios/boot-entry-lessons-v37.json" in known_packs:
        specification = ("scenarios/boot-entry-lessons-v37.json", *specification[1:])
    if specification and specification[0] in ("scenarios/boot-entry-lessons-v36.json","scenarios/boot-entry-lessons-v37.json") and "scenarios/boot-entry-lessons-v38.json" in known_packs:
        specification=("scenarios/boot-entry-lessons-v38.json",*specification[1:])
    if specification and specification[0] in ("scenarios/boot-entry-lessons-v36.json","scenarios/boot-entry-lessons-v37.json","scenarios/boot-entry-lessons-v38.json") and "scenarios/boot-entry-lessons-v39.json" in known_packs:
        specification=("scenarios/boot-entry-lessons-v39.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v40.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v40.json"].values())
        prefix=json.dumps(current[:76],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[76:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=79 or hashlib.sha256(prefix).hexdigest()!=current_network_transfer_origins["scenarios/boot-entry-lessons-v39.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_network_transfer_origins["scenarios/network-transfers-lessons-v1.json"][2]):
            raise ValueError("Network-transfer consolidation changed exact predecessor or additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v36.json","scenarios/boot-entry-lessons-v37.json","scenarios/boot-entry-lessons-v38.json","scenarios/boot-entry-lessons-v39.json"}:
            specification=("scenarios/boot-entry-lessons-v40.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v41.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v41.json"].values())
        prefix=json.dumps(current[:79],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[79:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=81 or hashlib.sha256(prefix).hexdigest()!=current_campus_origins["scenarios/boot-entry-lessons-v40.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_campus_origins["scenarios/campus-architecture-lessons-v1.json"][2]):
            raise ValueError("Campus architecture consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v36.json","scenarios/boot-entry-lessons-v37.json","scenarios/boot-entry-lessons-v38.json","scenarios/boot-entry-lessons-v39.json","scenarios/boot-entry-lessons-v40.json"}:
            specification=("scenarios/boot-entry-lessons-v41.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v42.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v42.json"].values())
        prefix=json.dumps(current[:81],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[81:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=82 or hashlib.sha256(prefix).hexdigest()!=current_leaf_spine_origins["scenarios/boot-entry-lessons-v41.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_leaf_spine_origins["scenarios/leaf-spine-lessons-v1.json"][2]):
            raise ValueError("Leaf/spine consolidation changed exact predecessor or authored addition")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v36.json","scenarios/boot-entry-lessons-v37.json","scenarios/boot-entry-lessons-v38.json","scenarios/boot-entry-lessons-v39.json","scenarios/boot-entry-lessons-v40.json","scenarios/boot-entry-lessons-v41.json"}:
            specification=("scenarios/boot-entry-lessons-v42.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v43.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v43.json"].values())
        prefix=json.dumps(current[:82],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[82:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=85 or hashlib.sha256(prefix).hexdigest()!=current_samba_origins["scenarios/boot-entry-lessons-v42.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_samba_origins["scenarios/samba-standalone-lessons-v1.json"][2]):
            raise ValueError("Standalone-sharing consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v%d.json"%v for v in range(36,43)}:
            specification=("scenarios/boot-entry-lessons-v43.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v44.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v44.json"].values())
        prefix=json.dumps(current[:85],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[85:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=88 or hashlib.sha256(prefix).hexdigest()!=current_virtual_isolation_origins["scenarios/boot-entry-lessons-v43.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_virtual_isolation_origins["scenarios/virtual-isolation-lessons-v1.json"][2]):
            raise ValueError("Virtual-isolation consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v%d.json"%v for v in range(36,44)}:
            specification=("scenarios/boot-entry-lessons-v44.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v45.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v45.json"].values())
        prefix=json.dumps(current[:88],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[88:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=91 or hashlib.sha256(prefix).hexdigest()!=current_guest_image_origins["scenarios/boot-entry-lessons-v44.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_guest_image_origins["scenarios/guest-driver-lessons-v1.json"][2]):
            raise ValueError("Guest-image consolidation changed exact predecessor or additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json" for n in range(36,45)}:
            specification=("scenarios/boot-entry-lessons-v45.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v46.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v46.json"].values())
        prefix=json.dumps(current[:91],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[91:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=94 or hashlib.sha256(prefix).hexdigest()!=current_smtp_origins["scenarios/boot-entry-lessons-v45.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_smtp_origins["scenarios/smtp-routing-lessons-v1.json"][2]):
            raise ValueError("SMTP consolidation changed exact predecessor or additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json" for n in range(36,46)}:
            specification=("scenarios/boot-entry-lessons-v46.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v47.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v47.json"].values())
        prefix=json.dumps(current[:94],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[94:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=97 or hashlib.sha256(prefix).hexdigest()!=current_sieve_origins["scenarios/boot-entry-lessons-v46.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_sieve_origins["scenarios/sieve-filter-lessons-v1.json"][2]):
            raise ValueError("Sieve consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,47)}:
            specification=("scenarios/boot-entry-lessons-v47.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v48.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v48.json"].values())
        prefix=json.dumps(current[:97],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[97:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=99 or hashlib.sha256(prefix).hexdigest()!=current_address_origins["scenarios/boot-entry-lessons-v47.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_address_origins["scenarios/sieve-address-lessons-v1.json"][2]):
            raise ValueError("Address-filter consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,48)}:
            specification=("scenarios/boot-entry-lessons-v48.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v49.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v49.json"].values())
        prefix=json.dumps(current[:99],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[99:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=102 or hashlib.sha256(prefix).hexdigest()!=current_delivery_origins["scenarios/boot-entry-lessons-v48.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_delivery_origins["scenarios/sieve-delivery-lessons-v1.json"][2]):
            raise ValueError("Delivery-filter consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,49)}:
            specification=("scenarios/boot-entry-lessons-v49.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v50.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v50.json"].values())
        prefix=json.dumps(current[:102],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[102:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=104 or hashlib.sha256(prefix).hexdigest()!=current_vacation_origins["scenarios/boot-entry-lessons-v49.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_vacation_origins["scenarios/sieve-vacation-lessons-v1.json"][2]):
            raise ValueError("Vacation-filter consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,50)}:
            specification=("scenarios/boot-entry-lessons-v50.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v51.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v51.json"].values())
        prefix=json.dumps(current[:104],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[104:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=105 or hashlib.sha256(prefix).hexdigest()!=current_migration_origins["scenarios/boot-entry-lessons-v50.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_migration_origins["scenarios/procmail-migration-lessons-v1.json"][2]):
            raise ValueError("Legacy-migration consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,51)}:
            specification=("scenarios/boot-entry-lessons-v51.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v52.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v52.json"].values())
        prefix=json.dumps(current[:105],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[105:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=108 or hashlib.sha256(prefix).hexdigest()!=current_mailbox_origins["scenarios/boot-entry-lessons-v51.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_mailbox_origins["scenarios/mailbox-access-lessons-v1.json"][2]):
            raise ValueError("Mailbox-access consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,52)}:
            specification=("scenarios/boot-entry-lessons-v52.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v53.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v53.json"].values())
        prefix=json.dumps(current[:108],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[108:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=109 or hashlib.sha256(prefix).hexdigest()!=current_mail_admin_origins["scenarios/boot-entry-lessons-v52.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_mail_admin_origins["scenarios/mailbox-administration-lessons-v1.json"][2]):
            raise ValueError("Mail-administration consolidation changed exact predecessor or authored addition")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,53)}:
            specification=("scenarios/boot-entry-lessons-v53.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v54.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v54.json"].values())
        prefix=json.dumps(current[:109],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[109:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=112 or hashlib.sha256(prefix).hexdigest()!=current_router_origins["scenarios/boot-entry-lessons-v53.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_router_origins["scenarios/linux-router-lessons-v1.json"][2]):
            raise ValueError("Linux-router consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,54)}:
            specification=("scenarios/boot-entry-lessons-v54.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v55.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v55.json"].values())
        prefix=json.dumps(current[:112],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[112:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=114 or hashlib.sha256(prefix).hexdigest()!=current_flow_origins["scenarios/boot-entry-lessons-v54.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_flow_origins["scenarios/linux-flow-lessons-v1.json"][2]):
            raise ValueError("Linux-flow consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,55)}:
            specification=("scenarios/boot-entry-lessons-v55.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v56.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v56.json"].values())
        prefix=json.dumps(current[:114],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[114:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=116 or hashlib.sha256(prefix).hexdigest()!=current_ipv6_router_origins["scenarios/boot-entry-lessons-v55.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_ipv6_router_origins["scenarios/linux-v6-lessons-v1.json"][2]):
            raise ValueError("IPv6-router consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,56)}:
            specification=("scenarios/boot-entry-lessons-v56.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v57.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v57.json"].values())
        prefix=json.dumps(current[:116],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[116:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=117 or hashlib.sha256(prefix).hexdigest()!=current_service_router_origins["scenarios/boot-entry-lessons-v56.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_service_router_origins["scenarios/linux-services-router-lessons-v1.json"][2]):
            raise ValueError("Services/router consolidation changed exact predecessor or addition")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,57)}:
            specification=("scenarios/boot-entry-lessons-v57.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v58.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v58.json"].values())
        prefix=json.dumps(current[:117],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[117:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=120 or hashlib.sha256(prefix).hexdigest()!=current_v6_admin_origins["scenarios/boot-entry-lessons-v57.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_v6_admin_origins["scenarios/linux-v6-admin-lessons-v1.json"][2]):
            raise ValueError("IPv6 administration consolidation changed exact predecessor or additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,58)}:
            specification=("scenarios/boot-entry-lessons-v58.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v59.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v59.json"].values())
        prefix=json.dumps(current[:120],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[120:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=121 or hashlib.sha256(prefix).hexdigest()!=current_v6_startup_origins["scenarios/boot-entry-lessons-v58.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_v6_startup_origins["scenarios/linux-v6-startup-lessons-v1.json"][2]):
            raise ValueError("IPv6 startup consolidation changed exact predecessor or authored addition")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,59)}:
            specification=("scenarios/boot-entry-lessons-v59.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v60.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v60.json"].values())
        prefix=json.dumps(current[:121],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[121:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=124 or hashlib.sha256(prefix).hexdigest()!=current_ftp_identity_origins["scenarios/boot-entry-lessons-v59.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_ftp_identity_origins["scenarios/ftp-identity-lessons-v1.json"][2]):
            raise ValueError("FTP identity consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,60)}:
            specification=("scenarios/boot-entry-lessons-v60.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v61.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v61.json"].values())
        prefix=json.dumps(current[:124],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[124:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=126 or hashlib.sha256(prefix).hexdigest()!=current_service_exposure_origins["scenarios/boot-entry-lessons-v60.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_service_exposure_origins["scenarios/service-exposure-lessons-v1.json"][2]):
            raise ValueError("Exposure consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,61)}:
            specification=("scenarios/boot-entry-lessons-v61.json",*specification[1:])
    if "scenarios/boot-entry-lessons-v62.json" in known_packs:
        current=list(known_packs["scenarios/boot-entry-lessons-v62.json"].values())
        prefix=json.dumps(current[:126],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[126:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=128 or hashlib.sha256(prefix).hexdigest()!=current_vpn_point_origins["scenarios/boot-entry-lessons-v61.json"][2]
                or hashlib.sha256(tail).hexdigest()!=current_vpn_point_origins["scenarios/vpn-point-lessons-v1.json"][2]):
            raise ValueError("VPN consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json"for n in range(36,62)}:
            specification=("scenarios/boot-entry-lessons-v62.json",*specification[1:])
    if 'scenarios/boot-entry-lessons-v63.json' in known_packs:
        current=list(known_packs['scenarios/boot-entry-lessons-v63.json'].values())
        prefix=json.dumps(current[:128],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        tail=json.dumps(current[128:],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        if (len(current)!=130 or hashlib.sha256(prefix).hexdigest()!=current_advisory_patch_origins['scenarios/boot-entry-lessons-v62.json'][2]
                or hashlib.sha256(tail).hexdigest()!=current_advisory_patch_origins['scenarios/advisory-patch-lessons-v1.json'][2]):
            raise ValueError("Advisory patch consolidation changed exact predecessor or authored additions")
        if specification and specification[0] in {"scenarios/boot-entry-lessons-v"+str(n)+".json" for n in range(36,63)}:
            specification=('scenarios/boot-entry-lessons-v63.json',*specification[1:])
    if specification and specification[0] == 'scenarios/ldap-administration-v2.json' and 'scenarios/ldap-administration-v3.json' in known_packs:
        specification = ('scenarios/ldap-administration-v3.json', *specification[1:])
    if specification is None or specification[0] not in known_packs:
        return False
    target, count, digest = specification
    ordered = list(pack.values())
    canonical = json.dumps(ordered, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
    if len(ordered) != count or hashlib.sha256(canonical).hexdigest() != digest:
        raise ValueError('Changed preserved authored origin; review the source consolidation')
    if any(known_packs[target].get(identity) != lab for identity, lab in pack.items()):
        raise ValueError('Preserved origin differs from its bundled scenario')
    return True

def catalog_inventory(root, catalog='bundled'):
    registered = registered_pack_paths(root, catalog)
    labs = {}; all_ids = set(); draft_ids = []; known_packs = {}
    total = 0
    for name in registered:
        pack, size = _pack_labs(root, name, 32_000_000 - total)
        total += size
        if all_ids.intersection(pack):
            raise ValueError('Duplicate authored ID')
        if len(labs) + len(pack) > 1000:
            raise ValueError('Registered catalog capacity exceeded')
        all_ids.update(pack)
        labs.update(pack)
        known_packs[name] = pack
    # Enumeration reports unregistered data only. It never grants lab/unit credit.
    draft_files = 0
    draft_bytes = 0
    unit_count = 0
    unit_paths = []
    for path in sorted((root / 'scenarios').iterdir()):
        if path.suffix != '.json':
            continue
        name = 'scenarios/' + path.name
        if name in registered:
            continue
        if not re.fullmatch(r'[a-z0-9][a-z0-9-]*\.json', path.name):
            raise ValueError('Unsafe unregistered draft source path')
        # Read at most one bounded file before classification. Exact preserved
        # origins are compared to the selected catalog and a closed fingerprint
        # roster, then discarded; they are not additional unintegrated content.
        # Charging those repeated historical copies against the draft budget
        # eventually rejects a valid selected catalog as its history grows.
        document, size = _catalog_document(root, name, 2_000_000)
        if isinstance(document, dict) and ('version' in document or 'units' in document):
            draft_bytes += size
            if draft_bytes > 32_000_000:
                raise ValueError('Unregistered data byte capacity exceeded')
            _unit_index(document, name)
            # Match the native course's unit budget. Unit metadata and exact
            # preserved origins do not consume the unrelated draft-pack limit.
            unit_count += len(document['units'])
            if unit_count > 256:
                raise ValueError('Unit metadata inventory capacity exceeded')
            unit_paths.append(name)
            continue
        pack = _scenario_labs(document, name)
        if preserved_authored_pack(name, pack, known_packs):
            continue
        draft_bytes += size
        if draft_bytes > 32_000_000:
            raise ValueError('Unregistered data byte capacity exceeded')
        draft_files += 1
        if draft_files > 100:
            raise ValueError('Unregistered data inventory capacity exceeded')
        known_packs[name] = pack
        if all_ids.intersection(pack):
            raise ValueError('Duplicate authored ID')
        if len(draft_ids) + len(pack) > 1000:
            raise ValueError('Unregistered draft inventory capacity exceeded')
        all_ids.update(pack)
        draft_ids.extend(pack)
    return labs, sorted(draft_ids), registered, unit_paths


def validate(root, catalog='bundled'):
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
    labs, draft_ids, registered_paths, unit_paths = catalog_inventory(root, catalog)
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
    return dict(catalog_profile=catalog, registry_entrypoint=("ccna_pilot" if catalog in ("teaching-review", "current") else "bundled"),
                unit_index_paths=unit_paths, references=len(rows), current_objectives=len(current),
                partial=sum(r['status'] == 'Partial practice' for r in rows),
                supporting=sum(r['status'] == 'Supporting explanation' for r in rows),
                open=sum(r['status'] == 'Open' for r in rows), authored_labs=len(labs),
                registered_labs=len(labs), registered_pack_paths=registered_paths,
                unregistered_draft_ids=draft_ids)


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--catalog', choices=('bundled', 'teaching-review', 'current'), default='current')
    args = parser.parse_args()
    print(json.dumps(validate(ROOT, catalog=args.catalog), indent=2))

# Exact59-object origin from verified432R1 retained selected inputs; hashes remain fixed.
