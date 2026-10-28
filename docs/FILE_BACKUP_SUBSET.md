# Virtual archive and restore subset

This foundation is not yet an integrated lab or objective credit. It supplies
typed, offline file/archive/snapshot behavior for the remaining LPIC-2 backup
work. Learner terminal syntax, teaching, assignments and native interaction still
need validation before catalog registration.

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
Future grading must restore into independent empty targets and inspect files,
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
