# Virtual archive and restore subset

This bounded model now supports four normally registered LPIC-2 backup/recovery
assignments. It defines the actual simulator boundary, not full GNU tar or exam
coverage. Source/course integration is distinct from final full-regression,
actual native and installed acceptance; current gates are in
[CATALOG168_MILESTONE.md](CATALOG168_MILESTONE.md). Earlier foundation results
below retain their historical source identities.

The model stores ordinary UTF-8 files, directories, ownership, permission modes
and logical modification/change times. A selected directory is represented by
relative names; the empty name denotes its root. An archive owns copied entries,
so later source edits cannot change a stored backup. Full archives include every
payload. Incremental archives include changed/new files and complete directory
inventories. A missing incremental snapshot produces the initial full level;
subsequent snapshots track the same named source and virtual timeline.

Incremental extraction uses directory inventories to remove names/subtrees no
longer present at backup time. Ordinary overlay extraction keeps extra names.
Restoring an incremental alone does not invent unchanged files; applying levels
out of order can restore stale contents. The model applies the requested archive
honestly instead of granting success based on an expected command or rejecting
every wrong chain before its consequences can be inspected. Snapshot branches
support both sequential increments and a larger increment since an older base.
Outcome grading restores into independent empty/stale sample targets and inspects files,
contents, modes and unrelated-data preservation, not merely archive filenames.

The original technical reference is GNU tar1.35, *Incremental Dumps*, local
upstream manual dated7September2026, inspected2026-10-28. The online primary
reference is [GNU tar incremental dumps](https://www.gnu.org/software/tar/manual/html_node/Incremental-Dumps.html).
The Fedora original `/usr/share/info/tar.info-1.gz` has SHA-256
`e2a5577834435166482dee180f55f8192ed21081a7ac5f7f25aeab532ee56365`.
This independent Rust implementation does not copy GNU source, manual prose or
archive bytes. Controlled real-tar comparisons pass eight fixed restore chains on Fedora: full,
sequential, overlay, delta alone, reversed, second sequential level, differential
branch and missing-middle level. Compared names/text/types/modes only; no claim
of binary-format, physical ctime or host-ownership equivalence. See M172.

The virtual archive is explicitly **not** the tar binary format. There is no
compression, inode/device discovery, actual filesystem traversal, hard/symbolic
link, special file, sparse-file, ACL, xattr, device/tape, network copy, encryption
or host identity operation. A monotonic logical clock replaces wall-clock
mtime/ctime and avoids teaching physical-filesystem timestamp precision rules.
No inode-based rename detection or snapshot byte-format compatibility is claimed.
Extraction preserves archived logical modification times but advances the virtual
change time, so restoring old data is visible to a subsequent incremental backup.
File/directory type-changing extraction is explicitly unsupported; it needs
additional unlink policy, so use a clean designated restore folder. All restored
metadata is virtual; no host permission or ownership change occurs. Archive root
names are metadata; the eventual workspace adapter must select the correct
virtual destination explicitly, never turn them into host paths.

Inputs are bounded to2MiB before decoding; maps reject duplicate/path-invalid
keys and the257th entry before reading its value. Text is capped at256KiB per
tree/archive, each path at1024bytes/64components and each component255bytes;
directories have unique bounded child lists. Absolute, dot, parent and control
character paths are refused. This is a virtual containment rule, with no host
file I/O in the module. Mutations validate capacity first; extraction builds a
private bounded result and publishes only after complete validation. Failures
leave the target unchanged. General file-copy permissions and GNU tar's
partial-write/error-continuation behavior are not yet modeled by this foundation.

## Terminal adapter in the four registered assignments

The bounded adapter uses one literal command at a time. Its typed
surface is `tar -c/-x/-t -f ARCHIVE [-C DIRECTORY] [-g SNAPSHOT | -G]`
(with optional -v for creation/extraction only),
including clustered short options and the corresponding long forms. Creation
selects one complete supplied tree by `-C PARENT NAME`; extraction selects a
supplied destination parent and restores the archive's named tree. Literal stored file members can be restored/listed independently; their parent
folders must already exist. Partial restore neither selects directory dumps nor
changes unrelated contents or parent metadata. Selective directory recursion,
legacy option syntax without dashes, repeated context options,
compression and old `-G` creation are explicitly unsupported. Extraction with
`-g /dev/null` is a virtual flag convention; it never opens that host device or
reads a snapshot during extraction. Listing shows actually stored member names.

`pwd`, `cd`, `ls [-a]`, `cat`, `mkdir`, `touch`, numeric `chmod` and regular-file
`rm` inspect/change only the supplied virtual trees. The `cp` subset copies opaque
archive/snapshot artifacts inside the supplied backup folder, including snapshot
branches; it is not a general file-copy lesson. Directory removal, touching
directories, shell expansion, operators, pipelines and subprocesses are excluded.
Literal quoting is parsed by the existing in-process terminal lexer. A snapshot
copy retains its source-tree identity; using it against a different source is
explicitly unsupported because inode-based relocation is not modeled. The supplied operator is explicitly the simulated backup administrator, root,
with no host privileges. Extraction preserves virtual owners/modes as privileged
GNU tar does by default; -p/--same-permissions is also accepted. Ordinary-user
umask/ownership restrictions are not silently approximated. Partial-error
continuation, inode relocation and actual privilege changes remain excluded.

The adapter has at most6 disjoint trees,64 entries/32KiB text per tree/archive,
8 archives,8 snapshots,32 command words/4096 input bytes,32KiB output and2MiB
encoded workspace data. Artifact/root maps reject excess or duplicate keys before
decoding values. Creation uses fixed0644/0755 virtual defaults for newly created
files/directories owned by simulated root; SGID parents supply their group and
propagate SGID to new directories. No host creation mask or identity is read. Whole commands publish a bounded
candidate on success. Focused runtime16 core/11 terminal groups and final all-target Clippy pass. The
full2045-test inclusive run passes on the source before the last numeric-chmod/
listing correction; the corrected source passes all27 focused groups. Ten controlled GNU tar comparisons include
partial restores that preserve unrelated changes. These are historical foundation observations; normal registration and its
separate final acceptance gates are recorded below. A foundation test count
alone never earns another authored assignment or objective credit.


The interactive backup terminal has no stdin/archive byte stream: `cat -` and
`cat -- -`, and a tar archive operand `-`, fail honestly instead of reading a
same-named virtual file. Bare `HOST:FILE` tar archive syntax is refused as remote.
An explicit absolute `/backups/-` or `/backups/host:archive` can name an ordinary
local virtual archive. These are parser distinctions, never host operations.
File additions/removals update the immediate virtual parent directory's logical
mtime/ctime; writing an existing file does not change its parent's entry clock.
The GNU comparison remains a projection of names/text/types/modes, not a claim
of physical timestamp or archive byte-format equivalence.

Numeric directory chmod follows GNU's set-ID preservation rule: one to four
octal digits preserve unmentioned directory setuid/setgid bits; a five-digit
form such as00755 clears them explicitly. Regular files take the exact requested
mode. Symbolic/operator modes and longer numeric forms are outside this adapter.
The corrected set-ID rules and verbose/duplicate-listing refusal pass the final
27-group focused gate and Clippy. Eight fixed numeric-mode comparisons against
GNU chmod match; the full2045 run precedes this last peer-review correction.
Verbose tar metadata listings and repeated literal selectors fail explicitly;
name-only listing is not silently presented as GNU tar -tv output.

## Outcome grading and separate teaching foundation

The workspace definition can require restored names/text/metadata,
independent recovery from declared archive levels, exact stored payload/type/root,
a current source-bound snapshot, and preservation of named supplied archives and
snapshots. An archive filename or a clean-looking existing restore folder is not
recovery proof. Each recovery test starts with its own sample files, including a
destination with obsolete data where declared. Only its private test destination
changes; learner files are retained. Wrong roots, missing levels, stale contents,
wrong metadata and overwritten history fail with specific counts or causes.
Logical clocks are not falsely graded as physical restoration timestamps.

Definitions have bounded early label/path/sequence admission and a2MiB aggregate
limit; test trees have at most64 entries/32KiB text. CheckRun validates borrowed
inputs before owning fixed definition/state snapshots. Each tick handles at most
one archive application or one result row; dropping it cancels without publishing
success. At most54 result rows exist. The native integration reserves
combined snapshots, recovery transients and results, keys results by the COMPLETE
input and publishes only finished, still-current outcomes. Its four-entry cache
is bounded to8MiB per entry/32MiB aggregate with a separately reserved16MiB
active checker. Borrowed edits preserve native Undo/cursor on rejected input;
whole-attempt2MiB admission occurs before publication. These source invariants
need actual native/candidate verification; passing a backend alone is insufficient.

The separate lesson supplies concise concepts, runnable explained example
commands, an adverse change and a repair. Loading verifies all three against the
same bounded simulator. Practice paths and the archive folder must be disjoint
from the assignment. Persisted practice has independent command text, transcript
and view; its commands never receive an Attempt or award assignment credit.
Transcript retention is16 entries/64KiB total, with32KiB command output; draft
4096bytes and raw lesson2MiB. Outer progress byte bounds and valid_for validation
must precede display/use of restored practice. The current registry counts four authored tasks; source registration and
actual native/candidate acceptance remain separate gates in the current milestone.

`stat -c FORMAT [--] FILE` now reads actual virtual tree metadata. Supported
format substitutions are `%a`, `%A`, `%U`, `%G`, `%n` and `%%`, without modifiers;
other formats fail explicitly. Symbolic permission text includes set-ID/sticky
execute distinctions. No ancestor/archive metadata, UID lookup, real date, file
size or link count is invented. Literal name/quote/delimiter behavior and output
bounds are tested. The original GNU coreutils9.10 manual, sections14.3 and27.5,
was inspected2026-10-28; its pinned local SHA256 is
`6e28a10ae4e4acad5c0147348ffe0a0a0cea4f93ed7cd794d434d953059e4716`.
The fixed disposable collector matches eight `%a %A` outputs as well as all eight
numeric modes and ten restore projections. No reference utility is part of the
learner execution engine. See M178 for source-bound evidence and remaining gates.


## Historical M179 authored integration boundary

Four unregistered assignments now have typed engine/pack validation and tested
complete-attempt transactions. Native scenario admission is256KiB; each complete
attempt, including both scopes and history, is2MiB. Before adding missing practice,
borrowed size admission accounts for the stored initial Definition, example State
and workspace overhead; refusal leaves every caller field unchanged. Actions
validate context before cloning, reserve8MiB transient space in the native caller,
and publish only after the proposed whole attempt remains within its bound.
Native memo/presentation wiring remains pending. Assignment CLI history retains
up to120 entries/256KiB; separate practice remains16 entries/64KiB.

Practice now stores its complete immutable Definition origin. Matching path
names alone cannot re-label a saved example after its source data changes.
Existing original154 profile fields remain compatible; new attempts are never
invented. Fresh file-load tests do not imply native process-cold acceptance.
See FILE_BACKUP_LESSONS.md and ledgerM179; all13 objective206.2 references remain
Open until appropriate native integration and broader required content exist.

## Current normal168 integration boundary

The normal registry now contains these four distinct lessons; the M179
unregistered observations above remain historical. Complete attempts still
include both scopes/history within2MiB; no cap is raised by registration.
Cheap reciprocal kind/learning guards protect other lab families before their
preparation can mutate a backup Attempt. Complete source and state keys prevent
warm results from attaching to a changed/foreign activity.

The current map explicitly credits only206.2 parent/K4/K5/K6/T3 and cross-track
103.3.K6 tar-only/T9 within this disclosed subset. Structural Payloads compares
member names/root/incremental type; actual text/metadata validation belongs to
Tree and independent Recovery outcomes. No checksum or binary integrity is
implied. Directory selection/products/media, shell backup automation, dd, tape/mt
and rsync remain Open. Final composed inclusive suite passes2279 checks with no
failures or ignored tests. Final81 development-native screenshots and scoped render review pass. Exact installed
candidate/NFS cold reopening remain separately pending.
