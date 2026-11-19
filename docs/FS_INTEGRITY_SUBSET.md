# Filesystem consistency foundation

Implemented source subset2026-11-04. This original bounded graph and literal
console support two authored filesystem-consistency tasks currently under native
verification. The verified265candidate and all preceding labs remain unchanged;
new source work is not yet a verified review candidate. Capacity and blank
partition preparation do not establish filesystem consistency coverage.

The proposed exercise distinguishes file inodes/content/block references,
directory names, stored link counts, allocation bitmaps and free counters. Root,
lost+found and reserved metadata are fixed abstract regions, not an ext4 disk
image. Findings must be derived from inconsistent relationships, never a broken
flag or command-to-success mapping. Structural limits admit supported logical
damage while rejecting unsupported formats and oversized data.

Read-only checking changes nothing. Supported preen repairs correct accounting
and link counts; a nonempty unattached inode requires an explicit recovery
decision. Recovery may reconnect retained bytes to modeled lost+found; it cannot
reconstruct overwritten data. Multiply claimed blocks and dangling entries cannot
be guessed away. Stop/refusal and corrected-versus-uncorrected exit bits must be
honest. Mounted checks are refused by this exercise: real read-only mounted
checking can be safe to run but its findings are unreliable, and repair is unsafe.

No raw ext4 parser, journal replay, superblock recovery, extents/sparse files,
arbitrary directories, block I/O, real fsck/mkfs/mount, host device, kernel or
recovery guarantee. No Btrfs/XFS/ZFS or full104.2/203.2 claim. The native console accepts only the literal commands below and never forwards
input to a host shell. Typed graph and adapter gates passed before registration;
native verification is recorded separately in FS_INTEGRITY_LESSONS.md. No dependency or shell evaluator change is needed for this slice.

Primary references checked2026-11-04:

- [LPIC-1 current5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/),104.2.
- [Upstream e2fsck manual](https://man7.org/linux/man-pages/man8/e2fsck.8.html): read-only checking, preen, explicit yes and exit bits; mounted-results caveat.
- [Maintainer e2fsprogs v1.47.3 classification](https://raw.githubusercontent.com/tytso/e2fsprogs/v1.47.3/e2fsck/problem.c): reference-count/bitmap fixes versus a nonempty unattached inode's intervention requirement.

The last source is GPL code, consulted only for behavior. Its implementation and
message tables are not copied or redistributed here. The model uses original
Rust algorithms and reports. No actual-tool comparison or native acceptance is
claimed merely from these references or this contract.

## Literal in-process console

`fs_integrity_terminal` adds only supplied virtual devices and fixed /mnt/ leaf
mountpoints. Its syntax is `findmnt [DEVICE|TARGET]`, `dumpe2fs -h DEVICE` (stored
summary only), `ls -li MOUNTPOINT[/lost+found]`, `cat MOUNTPOINT/FILE`, `umount
MOUNTPOINT`, `mount DEVICE MOUNTPOINT`, and `e2fsck`/`fsck.ext4` with explicit `-f`
and one of `-n`, `-p`, `-y`. Combined and separate flag spellings are admitted.
Implicit clean-state/journal skipping, other flags, generic fsck dispatch, shell
quoting/expansion/globs/operators, busy processes and host fallback are refused.
The strict mounted-check refusal is exercise policy, not a claim that real `-n`
cannot run. Mounting is only a virtual association: no journal or kernel behavior
is inferred. Mounting alone cannot correct the graph's errors.

Limited ls output derives inode/link/byte data; owner/mode/time and directory
regions are fixed exercise metadata. Reports explicitly identify the abstract
model rather than pretend to decode ext4 bytes. `dumpe2fs` shows stored counters,
which may be wrong. Real e2fsck's corrected/error-left exit bits are represented:
0 clean,1 corrected,4 uncorrected,5 corrected plus uncorrected. A successful
repair returning1 is displayed as corrected, separate from assignment completion.
Read-only queries cannot repair anything or award completion.

Commands are at most2KiB, output32KiB, profiles two volumes, and admitted system
JSON2MiB. Each volume has at most32 regular inodes,64 names,256 blocks,128 inode
slots and16KiB per file. Counts are data, not allocations sized by an untrusted
number. Duplicate and excess collections refuse during deserialization. No
reference utility is executed. Core/terminal groups pass, including actual
virtual unmount/check/repair/remount/read workflows, alternate flags, partial
preen, correct exit1 handling, separate protected volume and escape refusals.
Standalone model tests do not establish native UI or actual-tool conformance.

## Source-bound work and progress

Each authored task has three deterministic samples. Healthy, mounted, preserved
named data, retained orphan recovery and unchanged sibling requirements inspect
actual resulting graphs. The checker is cooperative, at most198phases (one
command per phase), and Stop discards unfinished results. Histories hold at
most64 successful state-changing commands/32KiB, console40messages/64KiB, each
saved message32KiB and complete session512KiB. One command is bounded by the
small fixed graph; no command can spawn a process, access a real path or loop.
Only typed literal parsing enters the virtual state model.

Practice and assignment drafts, transcripts and histories are separate. Read-only
queries and failed commands never change history; actual mutations invalidate
current grade immediately. Runtime caches and proofs are not serialized. Loaded
history is revalidated against its immutable task and device associations before
replay. A stored checked-history stamp requests actual cooperative rechecking,
not credit. An unexecuted input draft is preserved literally and never executed
on resume. Reopening/reference evidence must be reported separately from unit
fixtures. No new shell syntax, utility engine or dependency is introduced.
