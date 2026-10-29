# Existing-volume growth and careful handovers

Four authored LPIC-2 assignments are integrated in the normal native catalog.
They add narrow204.3 practice, not a complete LVM course or real-device training.
See [verification](CATALOG186_MILESTONE.md) for actual build and test identities.

| Assignment | Meaningful decision | Supported alternatives |
| --- | --- | --- |
| Existing records pool | Use existing free extents; preserve the sibling and files. Grow both backing and filesystem to at least16MiB. | Absolute/relative sizes or extent requests; using all free space is valid if preservation holds. |
| Insufficient archive pool | Add the supplied blank PV to the existing VG, then grow both layers to at least40MiB. | Explicit pvcreate or supported vgextend initialization;40/44MiB outcomes. |
| Filesystem gap | The LV is already24MiB; only its8MiB filesystem is too small. Reach at least20MiB without changing the LV mapping. | Explicit20MiB,5120 filesystem blocks or filling24MiB. |
| Two-volume handover | Add capacity and budget for reports≥16MiB and queue≥12MiB while preserving the neighbor. | Balanced allocation retaining free space or a larger20/16MiB allocation. |

The prerequisite unit is `persistent-mounts`; later tasks also recommend the
previous LVM tasks. Introductions teach PV/VG/LV, the supplied1MiB data offset,
PE rounding, report interpretation, binary M/G sizes and separate filesystem
capacity before assessment. Separate examples use different names,2MiB extents,
capacities and file contents. Insert never executes; learners Run each command.
The example experiment grows only the LV, producing a real modeled filesystem
gap, then repairs it. Neither example runs nor checks award assignment credit.
Challenge keeps the complete assignment and optional help, hiding unsolicited
walkthroughs. Assignment Reset confirms its selected-attempt effect; example
Reset affects only the example. Both drafts survive resume.

## Workspace and command scope

Use the bounded simulated storage terminal to investigate and change state.
`pvs`, `vgs`, `lvs`, selected `pvcreate`, `vgextend`, `lvextend` and `resize2fs`
forms are documented in LVM_SUBSET.md. `resize2fs` requires `/dev/VG/LV`; bare
`VG/LV` is not taught as its device shorthand. Unsupported syntax is refused.
The capacity table and optional retained-file inspector are simulator views.
No learner input reaches a host shell, device, filesystem or network operation.

Current feedback sits beside Run/Check and describes the first failing outcome;
individual results retain their own truth values. Capacities use readable units,
with exact bytes/files on request. Populated output and file viewers occupy six
font-derived rows and the full row width, with normal scrolling. This avoids
both nested zero-height collapse and a narrower hidden mouse-wheel target.
Single-line fields use shared controls; multiline file text remains top-aligned.
Hints are progressive; the worked answer/reference is separately requested.

## State, persistence and authoring

`State::Lvm` owns a versioned Session with separate assignment/example models,
commands, transcripts and view selection. `Goal::Lvm` owns the authoritative
Definition/Lesson. Runtime context binds both initial markers to that source,
including device sizes/signatures, group/LV identities, extent sizes and original
allocation prefixes. Current capacities, file contents and sibling preservation
are recomputed outcomes; a report, saved success flag or forged marker is not
proof. A breaking state or pending invalid work cannot retain current success.

Model admission is bounded to128 mapped extents/64KiB serialized bytes, Session
512KiB, Definition192KiB and complete source/Attempt pair1MiB. Transcripts have32
rows with2048-byte input/8192-byte output limits. The existing raw pack2,000,000
and progress16,000,000-byte limits remain; these are not exact heap guarantees.
Unknown fields, excessive collections and invalid typed relationships fail
before publication. Commands stage a candidate and validate before committing;
this simulator transaction is not a promise about interrupted real LVM tools.

Normal saves use `save_with_catalog`, checking every LVM attempt against its
app-owned source before the existing safe writer. Catalog-free saves refuse
profiles containing this family. Load/export/inspection/restore/current grading
also bind the source; old history and receipts are retained. No reference script
runs on Save or idle rendering. Raw packet checks preserve native selection/Undo
and refuse ignored/oversized or post-Enter text before dispatch.

The pack is `scenarios/lvm-growth-v1.json`; its unit is in `units-v1.json`.
Registration tests pin all182 predecessor scenarios/48 raw units, reconstruct
old source independently and validate complete new definitions. Keep strict
predecessor adapters when appending later units; do not assume a prior unit stays
at the catalog tail. Content/widget/native tests exercise references, two valid
alternatives, wrong/partial work, example isolation, invalidation and reset/resume.

## Limits and references

Only supplied linear growth is modeled. VG/LV creation/removal/reduction,
PV removal, snapshots, activation, mount integration, mapper aliases and
`lvm.conf` are absent. The flat retained-file/capacity model is not ext4 metadata,
filesystem repair or203.2 maintenance coverage. No host LVM execution is used.

Primary references, checked2026-10-29: [LPI204.3](https://www.lpi.org/our-certifications/exam-201-202-objectives/#rdp-we-204.3_Logical_Volume_Manager_(weight:_3)),
[LVM lvextend](https://github.com/lvmteam/lvm2/blob/main/man/lvextend.8_pregen),
[pvcreate](https://github.com/lvmteam/lvm2/blob/main/man/pvcreate.8_pregen),
[e2fsprogs resize2fs](https://github.com/tytso/e2fsprogs/blob/master/resize/resize2fs.8.in).
Manual text is not copied. Installed/cold, Windows/macOS, physical-device,
screen-reader and novice acceptance remain separate gates.
