# Campus architecture: a bounded structural model

Source milestone2026-11-13: **454 labs/180 units** (CCNA109, LPIC-1 151,
LPIC-2 114, Bash80). Two new assignments follow `ccna-wired-device-paths` in
**Campus architecture: access, distribution and core**:

- `ccna-collapsed-campus-uplinks`: replace repeated-parent/missing upper endpoints;
  retain protected links and keep paths after either combined device is unavailable.
- `ccna-distribution-core-paths`: repair cross-building access links and missing
  core links, preserve eight fixed cables, and keep paths across the stated core
  and distribution losses. A physically connected wrong-building shortcut fails.

Both teach the device roles visibly, demonstrate a different guide topology, give
specific guided cable changes/observations, then present independent requirements.
No worked endpoints populate the assignment. Compatible unused ports and equivalent
wiring order pass; one surviving normal path alone cannot pass the entire task.
The cable table edits only supplied editable links; fixed endpoints remain available
on demand. Current physical paths and required hierarchy paths are named separately.
Case selection is an observation, not a cable edit or a separate assignment.

## What is actually modeled

`network_architecture` is an opt-in source-held port/cable graph, separate from the
older `device_path` model whose inventory/untagged-switching limits remain intact.
Links assume already-configured routed adjacencies. Two-tier upper devices combine
distribution/core roles; three-tier access/distribution pairs stay in their stated
building, and separate cores join distribution blocks. In these exercises an access
device is not a transit shortcut between other access devices. These are disclosed
exercise constraints, not universal laws for every campus topology.

BFS derives physical and permitted-role paths independently from current endpoints.
Every supplied cable must be complete and role-compatible; a jack cannot host two
cables. Distinct parent devices, rather than cable count, determine uplink diversity.
A failed device/cable is excluded from the chosen experiment without deleting the
learner's draft. Drafts may retain known unplugged/conflicting endpoints for recovery;
unknown equipment, altered fixed cables and extra inventory are rejected. Observations
are derived results, never persisted grade authority. Changes clear current checks.

Limits:16devices,32cables,8ports/device and96ports total,4failures per experiment,
8cases/8access pairs per task. Source validation requires a nonpassing starter, a
reachable reference, exactly one normal case and at least one stated failure case.
Saved assignment/practice drafts are separate, bounded and source/version checked.
The header Reset explicitly clears this lab including practice; Reset example only
leaves assignment work alone. Other labs/favorites are unchanged.

No IP addressing, packets/ping, IOS configuration, STP, policy, convergence, capacity,
physical links or deployment validation is claimed. Passing stated single-device
cases does not prove every possible failure combination or real-world resilience.
Spine-leaf, WAN, SOHO and on-premises/cloud remain separate open1.2subtopics. This
unit is partial practical evidence for current CCNA v1.1 1.2a/b, not exam completion.

## Sources and provenance

Checked2026-11-13. Current versus future objective boundaries remain as documented
in OBJECTIVE_SOURCES; no future objective receives credit from these exercises.
No third-party source code, diagram or desktop configuration is imported.

- [Cisco CCNA v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf), current1.2a/b.
- [Cisco exam landing page](https://www.cisco.com/site/us/en/learn/training-certifications/exams/ccna.html).
- [Cisco Campus LAN/WLAN design guide](https://www.cisco.com/c/en/us/td/docs/solutions/CVD/Campus/cisco-campus-lan-wlan-design-guide.html), updated2020-05-04; role and collapsed/separate-core concepts, including routed access alternatives. No current hardware recommendation is implied.
- [Historical routed-access design](https://www.cisco.com/c/en/us/td/docs/solutions/Enterprise/Campus/routed-ex.html), updated2008-08-29, background only; not current IOS/platform authority.

## Verification status

Final source gates:11graph,8workspace,4source/progress and3authored/catalog checks
pass (26), plus6retained transfer checks and4normal-catalog UI/capture openers.
Nine widget/capture checks cover actual native selectors, independent practice,
wrong/partial/correct work, reset, Challenge guidance isolation, keyboard focus,
responsive columns, enlarged text and complete centered endpoint galleys. All452
old scenario values/179unit objects and ordering, and all107prior CCNA teaching
entries, remain exact. No old objective, evaluator or progress schema changed.

Actual development R2:106fixed actions/26screenshots pass on Fedora Wayland. It
uses real application widgets with synthetic private data, including case changes,
failed repairs, valid wiring, save/readback, stale-success invalidation and reset.
Representative originals were inspected at1920×1080/100% Navy and946×929/narrow
Light150%/Navy200%. R1before images exposed missing arrow glyphs and truncated
long values; the corrected R2images show font-safe paths, full-width narrow
selectors and compact worked rows without unnecessary horizontal scrolling.

- [Two-tier Introduction](../.build-artifacts/native-runs/run-9pxi3fdx/screenshots/workspaces/campus-0-introduction.png)
- [Guided investigation and feedback](../.build-artifacts/native-runs/run-9pxi3fdx/screenshots/workspaces/campus-1-practice-mistake.png)
- [Independent three-tier assignment](../.build-artifacts/native-runs/run-9pxi3fdx/screenshots/workspaces/campus-1-assignment.png)
- [Light150% endpoint controls](../.build-artifacts/native-runs/run-9pxi3fdx/screenshots/workspaces/campus-1-light150-controls.png)
- [Navy200% complete device/port value](../.build-artifacts/native-runs/run-9pxi3fdx/screenshots/workspaces/campus-1-dark200-controls.png)

Final inclusive Python1277, one-job all-target Clippy with warnings denied,
strict coverage and the454-row teaching inventory all pass after the last
capture-count/help-namespace synchronization. Packaged454content,
lifecycle, installed native/cold reopening and selected-input retention are still
pending. Latest verified package remains452in CHECKPOINT. No new inclusive Rust
runtime suite, full curriculum, physical-input, screen-reader, independent novice
or Windows/macOS runtime acceptance is claimed. Original failures remain private.

## Authoring and reuse

Keep `Facts` in the definition, never in learner-owned drafts. Name roles,
ports/buildings and the fixed wire inventory explicitly. A `Task` supplies its
unfinished `initial`, a reachability-only `reference`, editable cable IDs, distinct
parent goals, access pairs and named normal/failure cases. The reference is not an
exact-answer comparison: equivalent supported ports and wiring are graded from
resulting structure. A hierarchy path rises through the tiers and then descends;
a distribution block cannot become a transit backbone between disconnected cores.

Use stable scenario/definition IDs and matching versions. `new_session()` produces
the two separate drafts; `source()`/`context()` bind runtime and imported progress
to that immutable source. Internal Edit/Check JSON in pack solutions is declarative
validation data, never a device command taught to learners. The native workspace
uses the same typed edit/check methods. Source validation rejects solved starters,
unreachable references, altered fixed cables, foreign inventories and invalid
case/role identifiers. Known unfinished conflicts can be saved, diagnosed and fixed.

Write reviewed concepts, worked rows, exact practice-control actions/observations
and independent requirements in the CCNA presentation shard. Do not populate graded
fields from a worked answer. Keep technical detail here and in optional help.
Native selectors share normal control metrics, expand their row layout at narrow
sizes, and keep internal keyboard focus across reflow without taking OS focus.
Supporting paths use plain font-safe separators. Other workspaces retain their
existing appearance and behavior. Source-bound persistence has no schema change
for previous labs; no learner answers or mastery are fabricated.

Use the focused architecture graph/workspace/source/content tests and the campus
widget/capture tests through MAIN's guarded runner. Matrix evidence paths remain
restricted to `tests/`; the actual widget module and native screenshots are reported
separately here. The first inclusive Python run found that data-policy mismatch;
only the inadmissible CSV UI-path reference was removed. All four real integration
paths remain, and the unchanged validator plus all1277Python checks subsequently
pass. No source guard or historical baseline was loosened.

The retained earlier one-job all-target Clippy passed. Actual development
R1completed106actions/26captures; representative original images exposed missing
proportional-font arrow glyphs and long endpoint truncation at200%. Those before
images and selected source/ELF identity remain in private `native-r1`. Subsequent
actual-galley and keyboard/reflow regressions plus corrected development R2pass;
new package acceptance is still pending at this document checkpoint. These are technical
checks with synthetic inputs, not independent novice or assistive-device acceptance.


## Final packaged454 R1 acceptance —2026-11-13

The separate candidate in CHECKPOINT now passes initial/final selected-input
readback, private452→454 lifecycle, installed106-action/26PNG campus checks and
NFS/editor two-process cold restoration/repair. The protected launcher validates
454/180. All2929selected inputs/117120457bytes are retained; the source freeze ended.
This is source/technical Fedora acceptance, not full curriculum/platform/novice
acceptance. Earlier pending paragraphs are build-time history and remain bundled
at their original bytes; this post-build receipt does not modify that archive.

Installed raw screenshots:

- [Introduction](../.build-artifacts/native-runs/run-o3ghoxpr/screenshots/workspaces/campus-0-introduction.png)
- [Assignment](../.build-artifacts/native-runs/run-o3ghoxpr/screenshots/workspaces/campus-1-assignment.png)
- [Navy200% full selected endpoint](../.build-artifacts/native-runs/run-o3ghoxpr/screenshots/workspaces/campus-1-dark200-controls.png)

Actual normal-process IDs/exits: NFS3352582→3353119, both0; editor3353587→3353830,
both0. NFS unfinished exports, separate practice and repaired grading were inspected.
Editor unfinished script/Insert buffer and repaired example grading were inspected.
Five packaged GUI launches succeeded under these fixtures; historical intermittent
Wayland cause, physical input, assistive technology and novice acceptance remain open.
