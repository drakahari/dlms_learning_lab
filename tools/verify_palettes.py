#!/usr/bin/env python3
"""Verify the exact vendored palette inventory and immutable source hashes."""
import hashlib
import json
import pathlib
import tomllib

root = pathlib.Path(__file__).resolve().parents[1] / 'assets/themes'
manifest = json.loads((root / 'omarchy-manifest.json').read_text())
assert manifest['commit'] == 'c668141e9c42b13c80c9ca4ea108e11708c5e8a5'
expected = {f"omarchy-{t['upstream_id']}.toml" for t in manifest['themes']}
assert {p.name for p in root.glob('*.toml')} == expected
for theme in manifest['themes']:
    content = (root / f"omarchy-{theme['upstream_id']}.toml").read_bytes()
    assert hashlib.sha256(content).hexdigest() == theme['sha256'], theme['upstream_id']
    assert tomllib.loads(content.decode()) == theme['palette'], theme['upstream_id']
stock = json.loads((root / 'dlms-stock.json').read_text())
assert set(stock['palettes']) == {'dark', 'light', 'purple-gold', 'maroon-gold', 'ethereal'}
assert len(expected) == 22
for name in ['LICENSE.dlms', 'LICENSE.omarchy']:
    assert 'Permission is hereby granted' in (root / name).read_text()
print('Verified all five DLMS stock palettes and all 22 pinned official Omarchy source palettes/notices.')
