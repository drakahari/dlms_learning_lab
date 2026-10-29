# Inode and symbolic-link foundation

Checked2026-10-29. This is a separate, original, bounded teaching VFS. It does not
rewrite the existing `filesystem::Tree`, add an engine State/Goal, change progress
schemas, register labs or provide a shell command parser. LPIC-1v5.0 objective
104.6 remains **Open** until meaningful taught assignments and native acceptance
exist. Foundation tests alone do not provide practical curriculum coverage.

## State and supported operations

A supplied single virtual device has immutable physical directories, a fixed
physical working directory, a canonical path→inode table and a separate inode
table. Inodes are directories, UTF-8 regular-file data or symbolic-link text.
Paths under symlink names are resolved into physical containing directories;
they are not stored as phantom duplicate directory trees. Synthetic inode IDs
identify only this simulator's objects, never host devices or inode numbers.

- `hard_link` adds another name for the same **regular-file inode**. Saving through
  either name changes the same data. Link count is derived from the entry table.
- `copy_file` follows its source to a regular file and allocates an independent
  inode with initially equal bytes. Later edits do not propagate between copies.
- `symbolic_link` stores its target text verbatim without requiring a target to
  exist. Relative targets are interpreted from that link's **containing physical
  directory**, even when the learner's cwd is elsewhere. Absolute targets begin
  at the virtual root.
- `read_file` follows links to regular data. `read_link` returns the final link's
  raw text; it does not canonicalize ordinary filenames. `stat` inspects a final
  link itself by default; `stat_follow` explicitly follows it, like `stat -L`.
  Intermediate components always require physical directory resolution.
- `write_file` represents an editor save to an **existing** resolved regular
  inode. It follows symlinks and updates hard aliases. Missing/dangling targets
  fail; this API does not silently create them. `create_file` separately creates
  a new explicit path and refuses an occupied destination.
- `unlink` removes only the selected file/symlink name. Other hard aliases retain
  the inode; removing a symlink leaves its target intact. With no open-file model,
  the final removed name immediately collects its inode. Real open descriptors
  can preserve unlinked files; that behavior is not simulated.
- `move_entry` is an explicit **no-replace** file/symlink move, not full `mv` or
  POSIX `rename`. It retains inode identity and symlink text. Moving a relative
  link to a different directory can change what it resolves to. Existing
  destinations, including aliases of the same inode, are refused. Directory
  moves and replacement/backup/exchange modes are unsupported.

All destination arguments name an explicit new entry. Directory shorthand is
refused, never guessed. No terminal integration or claim to accept actual `ln`,
`rm`, `cat`, `readlink` or `stat` command lines is made yet. These typed operations
are the semantics a future clearly documented command subset can expose.
GNU option parsing, multiple operands, implicit basename creation, `--force`,
`--relative`, `-t`, `-T`, `readlink -f/-e/-m`, shell expansion and exit-status
formatting remain outside this foundation.

Hard links to directories **or symlinks** are explicitly refused. This is a
restriction, not a claim about Linux capabilities: GNU `ln -P` can link a symlink
inode, while `ln -L` follows it. The model will not silently approximate either
behavior. Multiple references to a raw directory/symlink inode are rejected on
restore. Regular-file aliases are first-class shared references, not repeated
path→text copies.

## Physical path rules and diagnostics

Resolution walks one component at a time, substituting link targets before any
subsequent `..`. For example, with `/desk/portal -> /srv/release/live`,
`/desk/portal/../note` uses `/srv/release/note`, not `/desk/note`. A leading slash
uses the virtual root; otherwise lookup starts at fixed physical cwd. Repeated
slashes are treated as one, `.` retains the current directory, and `..` at the
virtual root stays there. No resolved path can escape to the host.

A trailing slash requires a directory for read/stat resolution. An ordinary
file followed by `/`, `/.` or `/..` fails `NotDirectory`; it is not normalized
into a different path first. A trailing slash can require following even a final
symlink when default stat/readlink would otherwise inspect the link. Modification
APIs reject trailing slashes and final `.`/`..` explicitly rather than pretending
to implement all directory-command corner cases.

Missing targets fail `NotFound` but leave dangling links inspectable. More than
40 link traversals fail `Loop` with an ELOOP explanation. Growing expansion
queues/lookup work have separate `Limit` errors. Symlink cycles and dangling
links are valid stored states; an attempted traversal diagnoses them, while
stat/readlink can still inspect the final link without following it. Failure
never silently converts into empty content, a successful file operation or a
host lookup.

## Metadata, limits, atomicity and persistence

There are no permissions/users, mount boundaries, processes/open descriptors,
real device IDs, time metadata, arbitrary byte filenames or special files.
All supplied directories are immutable. Directory stat reports a conventional
virtual link count(2+immediate subdirectories), and deliberately has **no storage
size**. Regular sizes are UTF-8 byte counts; symlink sizes are target-text bytes.
These are partial typed observations, not fabricated full GNU stat/ls output.

Every public state operation/view validates the world before use. Fixed limits:

| Resource | Bound |
| --- | --- |
| Entries and inodes |128 each, including directories |
| Fixed directories |32 |
| File bytes / aggregate regular-inode data |8192 /65536, counting shared data once |
| Input/expanded physical path or link target |1024 bytes;64 input components;255 bytes/component |
| Symlink traversals / total resolution steps |40 /4096 |
| Expanded pending components |256 |
| Individual output/listing |32768 bytes; regular reads are at most8192 |
| Raw version1 JSON |1MiB before parsing |
| Inode/event counters |checked u32/u64, no wrap/reuse |

The raw ceiling deliberately accommodates all valid escaped state:128 escaped
1024-byte path keys(<264KiB),128 escaped1024-byte link texts(≤256KiB),64KiB
regular data escaped at worst6×(≤384KiB), and bounded IDs/cwd/record overhead remain
below1MiB. Actual states cannot reach all independent maxima simultaneously.
This avoids admitting data that cannot later be saved merely because it contains
JSON escapes. The parser retains its normal depth guard. Map visitors reject
oversized collections and duplicate keys; bounded string visitors reject
oversized typed strings. State validation rejects foreign reference IDs,
noncanonical entries, missing/non-directory parents, orphan inodes, prohibited
aliases and invalid counters. No executable pack or host path is involved.

Resolution is iterative, with admission before growing its pending queue. A
mutation validates, clones a bounded state, stages all changes, validates the
whole result and only then publishes it. A failure preserves every prior path,
inode, byte, ID counter and revision. A move at a full entry table remains valid
because it does not add an entry. Codec/read-only views never mutate the world.
No new dependency, unsafe block, subprocess, socket, host filesystem or clock API
is used. File contents, including UTF-8/NUL bytes, remain inert data.

`environment_matches` compares fixed physical directory identities and cwd only.
It is not grading or permission to alter task-protected files. Future content
must bind initial/current worlds to the authoritative lesson and specify all
protected files/inodes/configuration facts. Tasks requiring hard aliases must
inspect shared identity and write-through behavior, not merely equal bytes.
Behavioral alternatives, wrong link targets, stale copies, partial repairs and
post-success damage must all be graded from current state. Resume retains inode
sharing; reset clones the source initial world. No learner mastery or completion
credit is added by this foundation.

## References and source availability

All references checked2026-10-29; original code and paraphrases only. No upstream
code, examples, full documentation passages or assets are redistributed.

- [LPI official LPIC-1 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/#104.6_Create_and_change_hard_and_symbolic_links)
  reports current version5.0/101-500+102-500 and104.6: creating/identifying links,
  copying versus linking, and administration uses. The source was accessible.
- GNU Coreutils manual9.10, local official-distribution copy
  `/usr/share/info/coreutils.info.gz`, SHA256
  `6e28a10ae4e4acad5c0147348ffe0a0a0cea4f93ed7cd794d434d953059e4716`.
  Read `ln invocation`, `readlink invocation`, `stat invocation`, and
  `realpath invocation` for shared inode/relative target semantics, `-P/-L`,
  raw readlink/default stat and physical resolution. Official
  [ln](https://www.gnu.org/software/coreutils/manual/html_node/ln-invocation.html)
  and [readlink](https://www.gnu.org/software/coreutils/manual/html_node/readlink-invocation.html)
  web endpoints timed out; this gap is recorded rather than claiming web access.
- Linux man-pages upstream project,
  [path_resolution(7)](https://man7.org/linux/man-pages/man7/path_resolution.7.html),
  [unlink(2)](https://man7.org/linux/man-pages/man2/unlink.2.html) and
  [rename(2)](https://man7.org/linux/man-pages/man2/rename.2.html), accessible.
  They support component traversal/root/trailing-slash/40-hop behavior and
  shared-identity unlink/move distinctions; open descriptors and replacement
  rename semantics are explicitly outside this subset.

All16 standalone integration groups pass in monitored M444:
identity/copy behavior, last-name unlink, relative/absolute/dangling links,
physical parent traversal, stat/readlink/write differences, move anchor changes,
loops, honest unsupported forms, atomic failures, alternate repairs/current
invalidation, reset/resume, raw corruption and resource/isolation boundaries.
M448 all-target Clippy passes after removing one unused import, with every
semantic byte unchanged. M455 additionally matches eight fixed Linux sequences
and six actual simulator-only refusals; M456 verifies retained evidence without
replay. See LINKS_REFERENCE.md. No native or curriculum acceptance follows from
these pure-model/reference gates.
