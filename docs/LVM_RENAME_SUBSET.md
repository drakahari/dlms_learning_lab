# Virtual LVM names and retained objects

This source-enabled addition changes only new naming exercises. All319 previous
source scenarios, serialized absent fields and their original guards retain their
behavior. No learner input invokes LVM, a shell, a device, a real filesystem or a
kernel operation. Existing growth, raw creation/retirement and availability limits
remain as documented in the separate subset references.

| Supported form | Modeled effect |
|---|---|
| `lvrename VG OLD NEW` | Rename that original LV within its current supplied VG. |
| `lvrename VG/OLD NEW` or `/dev/VG/OLD NEW` | Equivalent current-path forms; NEW is a simple LV name. |
| `vgrename OLD NEW` | Rename that original group and every member path; keep the same supplied PV membership. |

Names must be valid, destinations unused, sources present. Collisions and malformed
commands explain the refusal without publishing a candidate state. Both valid
orders of a group-and-member rename are supported. Reports and retained-file
inspection use current names. Old paths no longer resolve. File contents do not
change to match a new name. Extents, sizes, filesystem capacity, individual active
states and supplied open-consumer facts stay with the original objects.

No flags, force, UUID-source lookup, duplicate-VG discovery, historical LVs,
device-mapper name escaping, udev, mount, boot or application-reference rewriting
is supported. The simulated reader is an application over the virtual file model,
not a Linux mount or raw-device command. Real reference changes and kernel/device
effects require broader practice. The simulator's candidate publication/refusal
does not assert real LVM transaction rollback or real-system safety guarantees.

## State and grading boundary

State.identity is optional and omitted when absent. Only explicit new source
worlds with the Lifecycle and availability profiles may contain it. Complete
bounded maps bind current group/LV names to internal simulation object identities;
these integers are **not real LVM UUIDs** and are not learner-facing syntax. Four
groups,16LVs,64-byte canonical keys, unique nonzero IDs and a monotonic bounded
next-ID counter are validated. Streaming map/count/text limits precede oversized
values. The existing2MiB raw/64KiB teaching budgets remain. No dependency, schema
version or shell-evaluator change is needed.

Renames move keys without changing object IDs or consuming the creation budget.
Existing creation/removal commands maintain the maps centrally inside the checked
candidate: recreating a name gets a fresh ID. Exhausted creation budgets refuse
atomically; rename itself remains available. Group renames also update PV group
names, member availability keys and supplied open references. Open facts are
compared by original object only in this explicit extension. Old name/open/prefix
guards remain unchanged when identity is absent; source-old sessions cannot inject
the extension.

Definition/Lesson.renaming is an exclusive plan, separate from growth, creation,
retirement and availability goals. It requests up to4group and8LV name changes.
Sources use original canonical names; LV destinations are simple names. Final
names must be unique across the entire inventory. Source binding preserves each
original filesystem object, original group identity, allocation prefix and file
keys at intermediate names. A missing raw allocation may remain an honest wrong
state, but its replacement cannot satisfy the original-object goal.

Current grading checks every original group and LV's requested or protected name,
exact allocations/files/capacities/PV membership, no extra/recreated storage,
individual availability and supplied consumers. Active filesystem readers must
return original contents. Transcript text, checked flags and walkthrough position
never establish success. Wrong temporary names and partial handovers remain
repairable; later invalid changes clear current success.

## Authoritative references

Checked2026-11-07: [upstream lvrename(8)](https://man7.org/linux/man-pages/man8/lvrename.8.html)
and [vgrename(8)](https://man7.org/linux/man-pages/man8/vgrename.8.html), plus the
existing [LPI201/202objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/).
The selected command forms are a deliberately bounded portion of the manuals;
there was no real LVM/device reference execution.204.3.K2remains partial because
general reduction/removal and other utility capabilities are still absent.
