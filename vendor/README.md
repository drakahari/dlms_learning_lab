# Narrow macOS Foundation backport

This is the unmodified crates.io `objc2-foundation 0.2.2` source except for
`Cargo.toml` and `src/thread.rs`, with its exact upstream MIT notice and a
machine-verifiable provenance receipt added. The original registry archive and
the complete two-file change are retained alongside the source.

## Why this change exists

The native macOS graph required `dispatch 0.2.0`, whose published archive and
recorded upstream source do not provide the original permission/copyright notice.
Its closed upstream issue does not supply one. We do not infer or invent that
notice. `dispatch2 0.3.1` is already pinned elsewhere in the dependency graph and
has verified applicable upstream license material.

Foundation exposes `run_on_main` to multiple native consumers, including winit.
Changing winit's two calls alone would require another replacement and reviewing
all other consumers. Removing the feature would remove used API. A broad
Foundation/objc2/winit upgrade would introduce substantially more compatibility
work. This backport instead preserves Foundation 0.2's public API and marker type,
and changes the single synchronous queue operation.

The synchronous dispatch2 API returns unit. A local `Option` preserves the
closure's return value after the synchronous call. The existing main-thread fast
path and `Send` bounds are unchanged. As before, the non-main caller requires an
active main runloop and may deadlock if the caller blocks that runloop. No new
queue, host execution, runtime permission, or accessibility removal is introduced.

## Provenance and verification

- Original crate SHA-256: `0ee638a5da3799329310ad4cfa62fbf045d5f56e3ef5ba4149e7452dcf89d5a8`
- Original revision: `e282618be4c3a3b9542957e0c8540e9588472ce8`
- Original notice: <https://github.com/madsmtm/objc2/blob/e282618be4c3a3b9542957e0c8540e9588472ce8/LICENSE.txt>
- dispatch2 registry SHA-256: `1e0e367e4e7da84520dedcac1901e4da967309406d1e51017ae1abfb97adbd38`
- dispatch2 original notice: <https://github.com/madsmtm/objc2/blob/8852b424193ca41602281b3d7540d7c8ed51e49a/LICENSE.md>

The notice collector accepts only this exact source, fixed original archive,
notice, patch and changed-file hashes. It compares every other source file with
the archive, rejects symlinks/unexpected files and rejects other local dependency
paths. Packages retain the original archive, patch and full resulting source.
It does not silently skip nonregistry dependencies or claim the old dispatch
notice was found. This is a downstream patch, not an upstream release.

This changes the release source/lock identity. All platform candidates must name
the new common source revision; old frozen candidates must not be relabeled.
Signing, notarization and native owner acceptance remain separate evidence.
