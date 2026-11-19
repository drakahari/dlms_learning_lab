# Virtual LVM availability subset

This optional addition applies only to explicitly authored availability lessons.
Existing317 source scenarios/progress keep absent fields and original behavior.
All storage, application-reader and consumer facts stay in-process. No kernel,
real device, mount, process, service or LVM executable is accessed.

## Commands and behavior

| Supported input | Meaning in this subset |
|---|---|
| `vgchange -a y VG` / `--activate y VG` / `-ay VG` | Make every current LV in exactly this supplied group active. |
| `vgchange -a n VG` / `--activate n VG` / `-an VG` | Make its LVs inactive, retaining metadata, allocations, files and sizes. |
| `lvs` | Existing fixed simulator report plus an explicitly labeled `ModelActive` column in these lessons. |

Exactly one known group is required. Omitted VG/all-device scanning, automatic
activation `ay`, degraded/missing-PV activation modes, `lvchange`, force, tags,
selectors, boot policy and shell expressions are unsupported with diagnostics.
Individual LVs can initially have different active states; a group command applies
to all of them. This is distinct from LVM's unsupported partial activation mode.
Prior disclosed linear creation/growth/raw-retirement forms remain available.
Creating an LV adds its active entry; raw removal removes it only when not open.

Open-consumer facts are supplied immutable exercise data. Group deactivation is
refused before effects if any member is marked open. This deliberately narrow
refusal is not a claim of actual process discovery or real-system safety. Real
LVM group operations may have partial effects. No force, consumer-stop or real
mount/use check is modeled. Unsupported behavior is never reported as success.

An inactive LV's allocation and filesystem metadata still exist. The simulated
application reader and modeled resize2fs require availability. Activating does
not create/repair a filesystem, mount it or configure startup. The native retained-
file inspection is an application reader over supplied virtual filesystem data;
it is not `cat` on a raw LV device and is not a Linux mount implementation.

## State, grading and authoring

State.activation defaults absent and serializes only when present. Absence means
activation is unmodeled, not that a real LV is active. Public activation queries
refuse absent profiles; existing old reader/report behavior remains unchanged.
The extension requires the existing Lifecycle source profile, a bounded canonical
VG/LV-to-boolean map for every current LV, and unique active open-consumer facts.
Maximum16entries,64-byte IDs, existing2MiB raw/64KiB teaching model budgets remain.
Streaming count/key/text bounds precede excess bodies. No new dependency/schema
number or shell-evaluator change is required.

Definition/Lesson.availability is an exclusive goal form, separate from growth,
creation and retirement plans. It lists1..4existing nonempty groups and desired
states. Busy-group inactive goals are rejected as unreachable. Presence/open facts
remain bound to source; current active values may change. Grade each required LV
and actual reader against original contents, preserve all original allocations/
files/devices, and report each unrelated LV availability separately. A success
message/transcript, checked flag or walkthrough position cannot award completion.
Deactivation or a wrong unrelated pool invalidates current success immediately.

Separate examples require disjoint supplied devices/groups and actual in-process
worked/adverse/repair reachability. Insertion never executes. Assignment/example
drafts, favorites, safe writes and selected reset use existing progress handling.
Old goals do not acquire new requirements or fabricated mastery. New lesson IDs
are stable. No downgrade-to-an-older-candidate acceptance is implied.

## Sources and verification boundary

Primary sources read2026-11-06 local: [LPI LPIC-2 objectives v4.5/201-450,204.3](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
and upstream [vgchange(8)](https://man7.org/linux/man-pages/man8/vgchange.8.html).
Only the disclosed ordinary named-group y/n behavior is modeled. There is no
host-LVM/kernel differential reference, complete tool compatibility or full204.3
claim. Snapshot/COW, rename/reduction, filesystem-bearing removal, mount/mapper
integration and lvm.conf remain open.

Pure8/workspace6 tests pass; authored5 passes after specific unrelated-LV feedback
was added without changing the all-checks conjunction. Registration/native/UI,
current-source QA, Python/Clippy and a new candidate are pending; use verified317
T055552Z until superseded. See CHECKPOINT.md for current gates and limitations.
