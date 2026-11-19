# Ordinary directory transfers: supported simulation

The opt-in recursive-tree workspace extends the existing bounded `filesystem::Tree`.
The original ordinary-file terminal is unchanged and still rejects recursive flags.
All learner commands operate on virtual data in-process. There are no subprocess,
host-file, service or network calls from this workspace.

## Commands and behavior

- `cp -R SOURCE_DIR DEST`, with `-r` and `--recursive` aliases. Optional `--`
  separates flags from literal names. Exactly one ordinary source directory.
- A new destination receives the complete source subtree: nested files, hidden
  names and empty directories. An existing destination directory adds the source
  basename. An existing target subtree is refused: no merge or overwrite.
- New entries belong to the simulated caller and their primary group. Source
  permission bits are filtered by the current virtual creation mask. Source
  ownership/timestamps are not preserved. Directories temporarily allow population,
  then receive their final filtered mode. A read-only file stays read-only.
- `rm -r DIR`, with `-R` and `--recursive` aliases, removes the complete directory
  subtree. It requires modeled directory read/search and parent write/search;
  file write permission is not the permission to remove its name. Removal is
  noninteractive, as disclosed, with no undo. Root/current/home trees are refused.
- Existing literal `pwd`, `cd`, `ls`, `cat`, bounded `find`, ordinary file changes,
  `mkdir`, `rmdir`, `chmod`, `chgrp`, `umask`, `id` and `show` keep their existing
  documented subsets. `help` describes the combined subset.

No shell expansion, pipelines, scripts, multiple operands, combined flags, `-a`,
`-p`, links/devices, final dot/dotdot or trailing-slash operands, directory merging,
ACL/SELinux, privileged overrides or full GNU utility semantics. Intermediate
dot/dotdot path components resolve only inside the virtual tree.

## Limits and saved work

Before cloning/mutation: valid ordinary tree, at most256 entries,256KiB aggregate
content,1024-byte paths,255-byte components,64 levels and bounded virtual clock.
Each recursive operation preflights permissions/limits and commits transactionally.
**Real cp/rm can leave partial changes on error; the simulation's atomic refusal is
not a real-system safety guarantee.** Commands cap at4096bytes. Transcript retention
is40 entries of8192bytes; whole saved sessions cap at4MiB.

The catalog holds the trusted task, initial source content and separate practice
world. Saved sessions contain only bound state, drafts, navigation and check flags.
Flags are never sufficient grading evidence: outcome checks recompute exact names,
types, content, modes, ownership and source/unrelated preservation. Mutations clear
stale success; read-only inspection and navigation preserve it. Source, role, caller,
accounts, outside-workspace entries and incompatible contexts are checked on
launch/load/save/grade. No migration invents new answers or completion.

## Technical references and actual comparison

[LPI's current objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
were rechecked2026-11-06: LPIC-1v5.0,101-500/102-500; this is narrow103.3 practice.
Installed GNU Coreutils9.10 Info (compressed SHA256
`6e28a10ae4e4acad5c0147348ffe0a0a0cea4f93ed7cd794d434d953059e4716`)
was read alongside [cp documentation](https://www.gnu.org/software/coreutils/manual/html_node/cp-invocation.html)
and [rm documentation](https://www.gnu.org/software/coreutils/manual/html_node/rm-invocation.html).
Live GNU page retrieval was unavailable during initial research; installed primary
documentation and the actual fixed comparison are the local evidence.

R11compares eight SHA-pinned, disposable, ordinary same-UID/GID GNU operations with
actual in-process probe states: new/existing destinations,022/077masks, a read-only
directory, spaced/dash names, nested removal and a read-only file. All8match exact
relative names/types/modes/contents and caller ownership/group. Three additional
model-only refusal checks are **exclusions**, not GNU matches: merging, final-dot
naming and atomic refusal versus possible real partial access-failure effects.

The MAIN runner held its common lock, two-job/thread limits and project-disk
artifacts. Held original cp/rm/probe executable descriptors, empty stdin, stripped
environment, child-only no-new-privileges/limits/proof and child-only umask bounded
execution/output. Fixture ACL inheritance would refuse, not change host settings.
The control binds688compiled inputs and tool/source/ELF identities. Raw launches,
streams, before/after fixtures and retained source/ELFs are ignored local evidence:
`.build-artifacts/reference-runs/recursive-tree-r1/compared/COMPARISON.json`.
Control SHA256`08cd903eb0665f2a516f16752103461c92323dd0cc31a0356de88092203e4dc7`.
This was root source-reviewed, **not an independently reviewed historical seal**.
No reference program is part of learner execution or required for offline use.
