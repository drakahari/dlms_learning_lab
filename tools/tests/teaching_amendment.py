"""Closed test-only teaching projection; never imported by a producer or application."""
import hashlib
import json
from functools import lru_cache

MANIFEST_SHA256 = '4f852c80674081bf83cfc15dbdb23458625f684cb34cab4ad355f55b27f04ef2'
ALLOWED = [
    ('scenarios/shell-text-v1.json', 'shell-text-capstone', 3),
    ('scenarios/shell-reusable-v1.json', 'shell-bundle', 3),
    ('scenarios/shell-file-batches-v1.json', 'shell-file-batch-capstone', 2),
    ('scenarios/shell-case-v1.json', 'shell-case-interface', 0),
    ('scenarios/shell-arrays-v1.json', 'shell-array-priority', 1),
]


def require(condition, message):
    if not condition:
        raise ValueError(message)


def pinned(raw, pin):
    require(len(raw) == pin['bytes'], 'Source byte length differs')
    require(hashlib.sha256(raw).hexdigest() == pin['sha256'], 'Source SHA256 differs')


def reverse_pack(entry, current):
    pinned(current, entry['after'])
    original = current
    for edit in reversed(entry['byte_edits']):
        at = edit['after_offset']
        after = edit['after_utf8'].encode()
        require(original[at:at + len(after)] == after, 'Amended byte range differs')
        original = original[:at] + edit['before_utf8'].encode() + original[at + len(after):]
    pinned(original, entry['before'])
    forward = original
    for edit in reversed(entry['byte_edits']):
        at = edit['before_offset']
        before = edit['before_utf8'].encode()
        require(forward[at:at + len(before)] == before, 'Original byte range differs')
        forward = forward[:at] + edit['after_utf8'].encode() + forward[at + len(before):]
    require(forward == current, 'Forward byte reconstruction differs')
    return original


def validate_manifest(raw):
    require(hashlib.sha256(raw).hexdigest() == MANIFEST_SHA256,
            'Closed teaching manifest SHA256 differs')
    data = json.loads(raw)
    require(data['schema_version'] == 1 and
            data['kind'] == 'closed-test-only-teaching-amendment', 'Unknown amendment schema')
    require([(f['path'], f['scenario_id'], f['scenario_index']) for f in data['files']] == ALLOWED,
            'Unexpected amendment identities/order')
    require(sum(len(f['leaf_delta']) for f in data['files']) == 13 and
            sum(len(f['byte_edits']) for f in data['files']) == 9, 'Closed counts differ')
    return data


@lru_cache(maxsize=1)
def manifest(root):
    raw = (root / 'tests/fixtures/shell-teaching-amendment-v1.json').read_bytes()
    data = validate_manifest(raw)
    pins = {f['path']: f for f in json.loads(
        (root / 'tests/fixtures/catalog133-original-index.json').read_bytes())['packs']}
    for entry in data['files']:
        old = pins[entry['path']]
        require(entry['before'] == {'bytes': old['bytes'], 'sha256': old['sha256']},
                'Original immutable pack pin differs')
        require(entry['original_teaching']['fixture'] == entry['amended_teaching']['fixture'],
                'Teaching fixture amendment forbidden')
        reverse_pack(entry, (root / entry['path']).read_bytes())
    return data


def historical_example(root, current):
    entry = next((f for f in manifest(root)['files'] if f['scenario_id'] == current['id']), None)
    if entry is None:
        return current['teaching']['example']
    require(current['teaching'] == entry['amended_teaching'], 'Whole amended Lesson differs')
    return entry['original_teaching']['example']
