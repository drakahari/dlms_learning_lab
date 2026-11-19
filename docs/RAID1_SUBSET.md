# Bounded RAID1 replacement and recovery — capability contract

Specified2026-11-04 before public code export. This implements a narrow slice of the existing LPIC-2
204.1 roadmap assignment, “Replace a failed mirror member,” without claiming
full-objective credit. The pure capability, source-bound workspace and two
registered lessons now pass focused source/development native checks; packaged
repetition remains pending. Existing storage models and goals remain
unchanged. No host device, mdadm, kernel, filesystem, process or network API.

## Exact first scope

Supplied active, writable, two-slot RAID1 arrays only; maximum2arrays/8devices.
Inventory supplies raw sectors and usable MD data sectors separately. Replacement
capacity compares usable sectors with the array component requirement; no guessed
universal metadata deduction. All paths are admitted literal virtual identities.
Use independently stored, bounded textual replication segments to demonstrate
copying and file preservation. These teaching units are not RAID1 chunk_size or
a representation of real superblocks, physical sectors or filesystem layout.

A member can be synchronized, faulty, an idle spare or recovering. A failed
member remains attached until removed; removal preserves its stale MD identity
and sample bytes. Missing slots have no device to remove. Fresh writable blank
media may be added as an idle spare to a healthy mirror, or become a recovery
target when a slot lacks a synchronized member. Only one spare/recovery target
is admitted, avoiding invented selection rules between multiple spares.

Adding a replacement alone does not restore redundancy. An explicit **Advance
simulated recovery** native action copies the next bounded sample segments from
the surviving synchronized member. Cursor and actual copied prefix must agree.
Only a complete matching replica can become synchronized. Derive readable health
from actual membership/content invariants: Healthy, Degraded or Rebuilding (still
degraded). Native advance is not a Linux command, mdadm --wait or elapsed real
kernel time. It can be saved and resumed; no automatic work occurs on load.

No concurrent learner writes in this first slice. Recovery preserves the readable
source and protected arrays/devices. Redundancy is not a backup or proof that
original application data was correct. Protection flags are exercise safeguards;
real mdadm does not promise to recognize every valuable disk.

## Terminal and refusal behavior

Supported forms after capability verification:

- `mdadm --detail ARRAY` (also `-D`) inspects the supplied active array.
- `mdadm ARRAY --fail DEVICE` marks a supported synchronized member faulty.
- `mdadm ARRAY --remove DEVICE` detaches a faulty or idle spare member.
- `mdadm ARRAY --add DEVICE` accepts fresh admitted blank media.
- `--manage` before ARRAY is an explicitly supported equivalent for mutation.

One operation and one literal device per mutation. An ordinary successful
--detail normally exits0 even for a degraded array: inspect the displayed state.
Do not infer healthy storage from command success. Exact --test edge behavior
was not independently established and remains unsupported. Native read-only
inventory/data views disclose their simulator role rather than fake utilities.

Refuse active-member removal, inadequate usable capacity, occupied/protected,
read-only/unavailable candidates and foreign identities without publishing a
partial modeled mutation. All-members-lost and losing the last readable source
are unsupported; refusal is not a real Linux guarantee against data loss.
Malformed/unsupported syntax is distinct from a supported operational failure.

Real --add may first try metadata-based re-add and avoid full copying. This
subset deliberately admits fresh media only. Stale metadata/re-add, --replace,
force, zero-superblock, missing/new metadata formats, read errors/bad blocks,
bitmaps, assembly/boot/configuration, create/grow/reshape, other RAID levels,
read-only/inactive arrays, concurrent I/O and real timing remain unsupported.
Simulator prevalidated publication is not a real mdadm rollback guarantee.

## Bounds, authoring and progress

Version1 model: up to128segments/device,256ASCII bytes/segment,16sample files,
16segments/file,512KiB encoded model and finite sector arithmetic. The authored
workspace may enforce smaller64KiB models,192KiB definitions,512KiB sessions and
1MiB source/attempt pairs. Bounded string/sequence/map decoders reject unknown
fields, duplicate keys and excessive sizes before accepting state. No dependency
or global resource limit is increased. Native recovery steps copy at most128
small segments, so no unbounded loop or background process is required.

The authored Definition is source authority. Bind current and example models to
full separate initial identities and inventory; do not trust a mutable saved
marker, dropdown, progress percentage or cached success. Grade final synchronized
copies, retained source bytes, required failed-member retirement and protected
siblings; accept alternative eligible fresh media. Wrong/partial work remains
possible and should receive specific feedback. Saved drafts and partial recovery
must round-trip; reset affects only its selected scope. New stable IDs receive
new objectives; existing learner answers/history are never fabricated.

Use the approved Introduction -> separate Try example -> Assignment structure,
normal investigation terminal and compact array/member views. Teach health versus
exit status, failed versus absent/spare, usable capacity and actual copying before
the independent assignment. Keep reference details optional and assignment
answers hidden by default. No idle reference execution or automatic imported
state repair. No lesson count until integration and substantive checks pass.

## Primary references and evidence boundary

Checked2026-11-04:

- Official mdadm4.6 release and commit
  [8b414a0d4753693d67b9a5822994e5f0c91b0b0a](https://github.com/md-raid-utilities/mdadm/commit/8b414a0d4753693d67b9a5822994e5f0c91b0b0a).
- [Official manual source](https://github.com/md-raid-utilities/mdadm/blob/8b414a0d4753693d67b9a5822994e5f0c91b0b0a/mdadm.8.in),
  [Manage.c](https://github.com/md-raid-utilities/mdadm/blob/8b414a0d4753693d67b9a5822994e5f0c91b0b0a/Manage.c).
- [Linux v6.18 MD administration](https://github.com/torvalds/linux/blob/v6.18/Documentation/admin-guide/md.rst)
  and [rolling rendered primary documentation](https://docs.kernel.org/admin-guide/md.html).
- [LPI LPIC-2 v4.5 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/).

Source-derived subset, not an actual mdadm/kernel comparison. Detail.c and the
complete RAID1 error-handler path were unavailable in the independent review;
--test and complete failure semantics are not guessed. No host loop/block device,
module, sysfs write or privileged reference operation is authorized. Upstream
GPL sources are references only; copy no implementation/manual text or binaries
into original MIT code. Full204.1, other OS/runtime, physical-device,
screen-reader and independent novice acceptance remain pending.
