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
        return list(csv.DictReader(stream))


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
    current = {(r['blueprint'], r['objective_id']) for r in parents if 'upcoming' not in r['blueprint']}
    indexed = {key for key, item in expected.items() if item['kind'] == 'objective'}
    if current != indexed:
        raise ValueError('Current top-level map and source inventory disagree')
    rows = csv_rows(root / 'docs/SKILL_COVERAGE.csv')
    keys = [(r['blueprint'], r['reference']) for r in rows]
    if len(keys) != len(set(keys)) or set(keys) != set(expected):
        raise ValueError('Coverage has missing, duplicate or unexpected references')
    labs = {}
    for path in (root / 'scenarios').glob('*.json'):
        for lab in json.loads(path.read_text(encoding="utf-8")).get('scenarios', []):
            if lab['id'] in labs:
                raise ValueError('Duplicate authored ID')
            labs[lab['id']] = lab
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
            raise ValueError('Evidence references missing authored lab: ' + str(key))
        for name in tests:
            path = Path(name)
            if (path.is_absolute() or '..' in path.parts or not name.startswith('tests/')
                    or not (root / path).is_file() or not (root / path).resolve().is_relative_to(root.resolve())
                    or any((root / Path(*path.parts[:n])).is_symlink() for n in range(1, len(path.parts) + 1))):
                raise ValueError('Missing or unsafe test evidence: ' + name)
    return dict(references=len(rows), current_objectives=len(current),
                partial=sum(r['status'] == 'Partial practice' for r in rows),
                supporting=sum(r['status'] == 'Supporting explanation' for r in rows),
                open=sum(r['status'] == 'Open' for r in rows), authored_labs=len(labs))


if __name__ == '__main__':
    print(json.dumps(validate(ROOT), indent=2))
