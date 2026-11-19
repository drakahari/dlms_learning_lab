# Raw LVM retirement subset

Optional, source-bound `Lifecycle` profile. Existing absent/growth and `Linear`
profiles retain their previous commands, serialization and task requirements.
All state and commands stay in-process over bounded supplied virtual devices.
No actual LVM executable, host device, filesystem or subprocess is accessed.

## Supported operations

| Input | Modeled behavior |
|---|---|
| `lvremove -y VG/LV` / `--yes /dev/VG/LV` | Existing linear subset: remove one raw LV without a modeled filesystem; return its extents to its own VG. |
| `vgremove VG` / `-y VG` / `--yes VG` | Remove exactly one empty supplied VG; retain PV labels and device sizes, release their group ownership. |
| `pvremove /dev/NAME...` | Clear labels only on distinct supplied, unowned PVs. Device sizes stay unchanged. |
| `pvs`, `vgs`, `lvs` | Current fixed simulator tables, with byte columns; not full LVM report formatting. |

The existing disclosed linear creation and growth forms remain supported.
No force, tags/selectors, aliases, multiple VG names, combined flags, quoted paths
or shell expansion. Unsupported forms return an honest diagnostic. A command
publishes one fully validated candidate, so a refused multi-PV command leaves
all previous state intact. Real multi-device LVM does not promise that rollback.

`Blank` in serialized state means no modeled signature. For this profile the UI
says **No modeled PV label · not secure erasure**. This metadata model does not
zero media, securely erase data, format filesystems or establish that actual
storage is safe to retire. The scenario explicitly identifies its unused raw
allocations. Mount/process-use and kernel activation checks are not simulated.
Real removal commands can destroy data and have broader behavior; simulator
refusals are teaching boundaries, never real-system safety guarantees.

## Typed outcomes and progress

An optional retirement plan lists canonical existing raw LV IDs, emptyable group
names and PV labels to clear. The plan cannot target filesystem-bearing LVs,
retire a group containing unlisted LVs, clear a PV in a retained group, use aliases,
repeat identifiers or mix creation/growth/retirement goal forms. Separate example
worlds use disjoint device/group names. The authoring validator executes only
in-process worked actions, adverse change and repair to prove reachability.

Checks inspect current absence of target objects, returned free extents or final
label state, and exact unrelated groups/LVs/files/allocations/devices. Unexpected
new objects and damaged protected data fail. Reports, checked flags, walkthrough
position and transcript text never establish completion. Reintroducing a removed
allocation/label clears current success. Assignment/example drafts, selected
practice action, favorites and selected reset use the existing progress format.
New fields default absent; old goals and existing progress are not upgraded into
new credit. This does not change the shell evaluator or require a dependency.

Core budgets remain8devices/4VGs/16LVs/4096extents/2MiB serialized state; teaching
models64KiB/128mappedextents, session512KiB, command2KiB, bounded transcript32rows.
Plan lists are stream-bounded before excess bodies:8LVs/4groups/8PVs, IDs64bytes.
No snapshot/COW, general filesystem-bearing removal, activation, vgreduce, rename,
shrinking, thin/RAID LVM, mount/mapper integration or lvm.conf coverage is implied.

## Sources and evidence

Checked2026-11-06 local. Upstream LVM manuals via
[vgremove(8)](https://man7.org/linux/man-pages/man8/vgremove.8.html) and
[pvremove(8)](https://man7.org/linux/man-pages/man8/pvremove.8.html) describe the
broader real operations. The subset deliberately omits nonempty/force removal.
LPIC-2v4.5/201-450 [204.3 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
remain partial. No host-LVM differential reference was run.

Pure7 and workspace6 focused tests pass at initial admission. Authored task,
registration, native presentation and new candidate gates are pending; use the
verified315 candidate until explicitly superseded. See CHECKPOINT.md.
