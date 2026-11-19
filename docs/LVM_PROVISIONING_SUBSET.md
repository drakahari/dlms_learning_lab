# Opt-in linear LVM provisioning subset

Implemented source315/106,2026-11-06 local. Core13, binding/workspace5 and
authored/registration6 pass. Final native/package verification remains pending;
no full objective or host-tool conformance claim.

Reuse the existing virtual PV/VG/LV extent model and its separate filesystem
containers. A typed optional `provisioning: Linear` profile must be present in
both the supplied assignment and its distinct workshop. Old absent-profile JSON,
commands, growth goals and saved progress stay unchanged. New saved state cannot
supply that capability to an old authored definition.

New literal command forms (no host LVM, shell, device or service runs):

| Form | Meaning and disclosed boundary |
|---|---|
| `vgcreate [-s SIZE] VG DEVICE...` | Create a group on distinct supplied unowned PVs or blank devices; blank inputs are initialized, as in vgcreate. Default extent4MiB; this teaching subset admits power-of-two extents1–64MiB. Optional flag precedes the group; `--physicalextentsize` is equivalent. |
| `lvcreate -L SIZE -n NAME VG` | Create a new raw linear allocation, rounding to a whole extent. Integer suffixless sizes mean MiB; M/G suffixes use powers of1024. Name and size pairs may be in either order; long `--size`/`--name` equivalents work. |
| `lvcreate -l COUNT -n NAME VG` | Allocate an absolute positive extent count; `100%FREE` means the current free pool. Other percentages/relative/combined options are refused; `--extents` works. |
| `lvremove -y VG/LV` | Deliberately remove one raw/no-filesystem LV and return its actual extents. `--yes` and `/dev/VG/LV` work. Filesystem-bearing removal is unsupported; an exercise also preserves originally supplied allocation context. Real removal can destroy filesystems and data; no real-system safety guarantee follows. |

Names use the existing bounded ASCII component grammar and refuse LVM's reserved
internal LV names. New VG names cannot collide with supplied virtual /dev entries;
there is no inspection of the host /dev namespace. Allocation uses the actual
free PE set; original mappings and files are not silently relocated. Insufficient
space, unknown/occupied/foreign devices, collisions and malformed commands fail
before one validated simulator candidate is published. Real multi-device command
rollback, metadata/zeroing, prompts, udev, autoactivation and tool configuration
are not simulated by that in-process transaction.

Existing pvcreate/vgextend/lvextend/resize2fs forms remain available. Raw LVs have
no filesystem and no mount; this unit must never claim otherwise. The inherited
one-MiB data start,16 LV/four VG/eight-device/4096-extent limits,2MiB raw-state
bound and much smaller64KiB/128-mapped-extent teaching model bound remain. New
plan lists/strings have streaming bounds; transcripts stay32rows/8192bytes each.

Grading compares exact requested raw capacities, actual group member sets,
extent sizes and minimum remaining reserves, plus every protected original LV,
file and unrelated device. Reject unintended extra groups/LVs. No command string,
report line, checked flag or historical completion grants credit. Equivalent size
units, creation orders and explicit/implicit PV initialization pass focused tests.
Guided examples are independently owned, run actual supported commands and
allow removal/recreation of only their NEW raw LV to demonstrate invalidation.

Still absent: snapshots, thin/striped/RAID LVM, reduction, rename, general VG/PV
removal, activation, mapper aliases, lvm.conf, filesystem creation/mount integration,
actual LVM metadata/device/kernel conformance. Full204.3 remains partial.

Primary readback2026-11-06: official LPIC-2 objective version4.5/201-450,
https://www.lpi.org/our-certifications/exam-201-202-objectives/ (204.3);
LVM2-maintained manuals at
https://man7.org/linux/man-pages/man8/vgcreate.8.html ,
https://man7.org/linux/man-pages/man8/lvcreate.8.html ,
https://man7.org/linux/man-pages/man8/lvremove.8.html , and
https://www.man7.org/linux/man-pages/man8/lvm.8.html (valid names).
No executable LVM reference test is authorized or needed on host devices.

## Declarative authoring and progress

State profiles opt in with `"provisioning":"Linear"`; old absent profiles
serialize unchanged. Assignment Definition and its distinct Lesson each carry
a `plan`, leaving the older `targets` list empty. Plans have at most two groups
(name, extent_bytes, devices, reserve_extents) and four new raw volumes (id,
bytes). Every new capacity must be a positive whole extent, under the existing
teaching bounds. Names/devices and the two example worlds must stay distinct.
Known group membership and existing allocations are source-held context, not
learner-editable inventory. Imported progress cannot enable the profile for an
old lesson. No progress-schema bump or invented mastery is needed for new IDs.

`Definition::validate` runs the actual worked sequence, an adverse removal and
a repair in the example, requiring a genuine fail/retry. Scenario loading also
runs its assignment solution against the real typed state. Author additional
negative/alternate/persistence tests; a reference solution alone is not coverage.
Use a fresh existing ID only for an actual existing task; new tasks need stable
new IDs, prerequisite/unit registration and reviewed teaching presentation.

The new device-facts table stacks at narrow widths. Raw allocation and free-pool
rows derive from current state; existing filesystem capacity is still separate.
Existing absent-profile growth screens retain their presentation and behavior.
