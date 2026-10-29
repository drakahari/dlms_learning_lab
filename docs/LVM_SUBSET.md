# Virtual LVM growth foundation

Foundation implemented and checked2026-10-29. No authored scenario, native workspace or
curriculum credit is added by this foundation. The current LPIC-2 issuer page
still identifies objectives4.5, exams201-450/202-450. Objective204.3 includes
creation, removal, resizing, snapshots and activation; this first capability
models only the narrow recorded task “Extend a volume while preserving data.”
The objective and all its rows remain Open until reviewed instruction, separate
practice, assignment, interaction, grading and actual gates exist.

## Exact contract

State version1 stores supplied virtual devices, their signatures/VG ownership,
VG extent sizes, and ordered LV logical-to-physical extent maps. Each physical
extent belongs to at most one LV. Existing mappings remain a prefix during
growth; unrelated LVs, device inventory and all virtual file contents remain
unchanged. The allocator chooses unused physical extents deterministically in
device-name/index order. This is one valid linear allocation policy, not a
prediction of real LVM placement or performance.

Each supported device profile reserves1MiB before its data area. Grouped data
capacity is `floor((device bytes - 1MiB) / VG extent bytes)` whole extents.
The remainder cannot back an LV. Extent sizes are source-configured powers of
two from1MiB through64MiB. Actual LVM supports additional alignments and sizes;
neither a universal1MiB PE size nor universal metadata overhead is implied.
Virtual device byte sizes are sector-aligned and at most1TiB. Initialized PVs
need at least1MiB of data after the reservation in this teaching profile.

The filesystem on an LV is a separate optional, modeled ext4 container with
4096-byte blocks. Extending an LV does not extend this container. `resize2fs`
grows it within the current LV; omission of size grows to the LV boundary.
Its flat virtual files hold bounded UTF-8 text. A teaching use estimate charges
one metadata block plus each file's content rounded up to a block. This estimate
is explicitly not ext4 inode/journal/block-group/reserved-space accounting.
No ext4 disk structures, mount state, integrity check or kernel resize runs.
The typed read-file and capacity methods are simulator APIs, not Linux commands.

## Learner-facing command subset

Commands are parsed as one bounded command, with ordinary whitespace-separated
tokens and fixed validated identifiers. There is no shell interpreter, quoting,
expansion, redirection, command substitution, executable lookup or host access.
Identifiers that look like `/dev/NAME` or `/dev/VG/LV` are map keys only.

| Supported form | Modeled behavior |
| --- | --- |
| `pvs`, `vgs`, `lvs` | State-derived, read-only tables. A fixed simulator report names byte columns explicitly; this is not the complete real utilities' default formatting/options. |
| `pvcreate /dev/NAME...` | Initialize only supplied blank virtual devices. Existing PVs or foreign signatures are refused with an explicit limitation. No force, wipe, overwrite or reinitialization is modeled. |
| `vgextend VG /dev/NAME...` | Add supplied, unowned PVs to an existing VG. As documented by LVM, blank devices can also be initialized by this operation. An explicit earlier `pvcreate` is a valid alternative, not a mandatory real-LVM prerequisite. |
| `lvextend -l N VG/LV` | Grow to an absolute logical-extent count. `--extents` is equivalent. |
| `lvextend -l +N VG/LV` | Add N logical extents from that VG's actual free physical extents. |
| `lvextend -l +100%FREE VG/LV` | Add all the current VG free extents. Other percentages/policies are unsupported. |
| `lvextend -L N[M\|G] VG/LV` | Grow to the requested absolute byte size, rounded upward to a whole extent. `--size` is equivalent. Integer suffixless values mean MiB. |
| `lvextend -L +N[M\|G] VG/LV` | Add the requested byte size, rounded upward to a whole extent. |
| `resize2fs /dev/VG/LV [N\|NM\|NG]` | Grow the separate supplied modeled ext4 container. Suffixless integers mean4096-byte blocks, unlike LVM's suffixless MiB. Same-size requests are read-only no-ops. |

LVM utility forms accept `VG/LV` and `/dev/VG/LV`. `resize2fs` instead requires
the device pathname `/dev/VG/LV`; bare `VG/LV` is not a real resize2fs shorthand
and is refused. M/G input letters are case-insensitive,
binary powers of1024; `MiB`/`GiB` multi-letter suffixes, decimals and other size
units are honestly unsupported. Checked integer arithmetic precedes allocation.
Insufficient VG extents, oversized filesystem requests, wrong volume identifiers
or malformed/unsupported commands retain the full previous simulated state.
Each supported change stages one bounded candidate and validates before
publication. This simulator transaction is not a claim that a multi-device real
LVM command or an interrupted real resize rolls back all prior effects.

An ungrouped PV report shows available data bytes before a VG's extent size has
been chosen. A grouped PV report excludes its unusable trailing partial extent.
VG free space is the count of actual unallocated physical extents; LV reports
derive size from the stored mapping. Reports and display history never grant
credit. Future assignments can use pure capacity/file-preservation checks; new
requirements must be evaluated against current state, never inherited success.

## Isolation, persistence and resource admission

`State::from_json` admits at most2MiB of raw versioned JSON and validates the
complete model. Streaming map/sequence/text visitors bound collections before
excess value bodies, reject duplicate/unknown keys, and bound strings before
copying. Direct serde decoding is not semantic acceptance; all public commands
and state-reading/grading methods validate the complete state. Future outer
pack/progress adapters must preserve the raw bound before derived State buffering.

At most8 supplied devices,4 VGs,16 total LVs,4096 extents per PV/LV,32 total files,
64KiB per stored file and128KiB total file text are modeled. All mappings are
checked for foreign ownership, overlap, valid index and capacity. The raw size
cap also applies to candidate serialization before publication. No learner input
is passed to `std::fs`, subprocesses, network APIs, real device access or host
services. Iteration is bounded by these fixed inventories; there is no learner
loop or executable script in this module. Native integration still needs its
own working-memory/frame/Stop gates and is not accepted by source review alone.

## Explicit exclusions and remaining204.3 scope

No VG/LV/filesystem creation commands, removal, shrink, rename, snapshots,
activation/deactivation, missing devices, recovery, PV resize/move, RAID,
striping, thin provisioning, cache pools, encryption, real LVM metadata/UUIDs,
device-mapper aliases, mounts, lvm.conf, filesystem corruption or real hardware
are implemented. Shrink refusal is a simulator limitation, not a statement that
real `resize2fs` cannot shrink an unmounted filesystem or that real LVM always
prevents destructive changes. No full204.3 or real-equipment equivalence claim.

## Sources and actual verification boundary

Primary issuer/source-manual references checked2026-10-29:

- [LPI current LPIC-2 objectives4.5](https://www.lpi.org/our-certifications/exam-201-202-objectives/), objective204.3 and current version. Existing project fingerprints remain untouched.
- [LVM upstream resource page](https://sourceware.org/lvm2/), including the official source-repository move.
- [LVM upstream lvextend manual source](https://github.com/lvmteam/lvm2/blob/main/man/lvextend.8_pregen), for extent/size growth, binary input units, relative values and the separate optional resizefs behavior that is not supported here.
- [LVM upstream pvcreate manual source](https://github.com/lvmteam/lvm2/blob/main/man/pvcreate.8_pregen), for initialization, default first-data alignment and the existence of real force/reinitialization behavior excluded here.
- [LVM upstream main manual](https://github.com/lvmteam/lvm2/blob/main/man/lvm.8_main), for physical/logical extent concepts and restricted naming/allocator scope.
- [Maintainer resize2fs manual source](https://github.com/tytso/e2fsprogs/blob/master/resize/resize2fs.8.in), for separate filesystem/device sizing, block-unit defaults, grow/shrink and online-resize qualifications.
- [Kernel ext4 design](https://www.kernel.org/doc/html/latest/filesystems/ext4/overview.html), documenting the richer block-group structure deliberately not reproduced by the teaching accounting.
- [Red Hat Enterprise Linux10 LVM documentation](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/10/html/configuring_and_managing_logical_volumes/limiting-lvm-device-visibility-and-usage), section10.1.1, for initializing new devices with `vgextend`. Its real devices-file behavior is excluded from this fixed supplied inventory.

The official distributor [vgextend manual](https://manpages.debian.org/unstable/lvm2/vgextend.8.en.html)
also confirms automatic initialization of blank PVs (retrieved package2.03.31).
Its directly requested upstream source mirror could not be fetched by the
browsing tool during this pass; the behavior is independently documented by
the primary Red Hat source above. The successful upstream manual pages were inspected at their moving
main/master URLs on the stated date, not at a falsely claimed immutable commit.
No upstream code or manual prose is copied into this implementation.

The proposal includes prospective tests for reference and alternate growth,
partial LV-only results, round-up and PV-tail arithmetic, physical allocation
holes/preserved mapping prefixes, state-derived reports, ownership/foreign
devices, wrong order/size/overflow/unsupported syntax, saved-state round trips,
data-loss/current-success detection and streaming malformed input boundaries.
Source formatting is not compilation or test execution. No host LVM/reference
device command was run, and no native/candidate/platform acceptance is claimed.

## Verification at this foundation milestone

Sixteen integration tests pass against the in-process model: normal and
alternate growth, real free holes and rounding, insufficient/foreign capacity,
atomic multi-device failure, preservation, round-trip validation, hostile
serialized data, real device-path refusal and resource boundaries. All original
fifteen outcome assertions remain. All-target Clippy passes with warnings
denied. These checks do not run real LVM and do not establish a lesson, native
workspace, full-suite milestone or real-device equivalence. Authored teaching
and learner interaction remain subsequent work.
