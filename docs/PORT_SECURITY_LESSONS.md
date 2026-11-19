# Secure access ports: identity, continuity and recovery

Development milestone, 2026-11-08. Three implemented lessons follow the unchanged
Ethernet-learning handover in CCNA. Current catalog:349 labs/123 units;
77CCNA,117LPIC-1,78LPIC-2,77Bash, with21/45/31/26units. The last ready Fedora
review is still346T234326Z/R51 until this batch's exact package is verified.

| Stable lab ID | Practical skill | Intentional unfinished work |
| --- | --- | --- |
| `ccna-security-identity` | Reserve the approved source before arrival order decides; keep authorized delivery working. | Maximum one with dynamic learning and no configured identity. |
| `ccna-security-observe` | Reject and count a third source while two approved sources stay online. | Protect drops but does not count the rejection. |
| `ccna-security-recover` | Replace a retired binding and explicitly recover an error-disabled desk without weakening neighbors. | Old configured identity, shutdown mode and an actual disabled latch. |

These are different tasks, not numeric variants. Each has a taught Introduction,
annotated worked configuration, separate runnable practice and full independent
assignment. The native workspace uses a topology, aligned port settings, explicit
recovery, one-frame metadata replay, actual secure table/counters and ordered case
feedback. Help is optional and scoped by source, practice/assignment and mode;
Challenge shares the meaningful assignment without inheriting an open solution.
Keyboard editing, Tab navigation, focus across column reflow and large-text layouts
have regressions. No pretend IOS terminal, real frames or host networking is used.

## What is taught and checked

A maximum limits the count; it does not name the intended first sender. Both the
Introduction and specific retry feedback explain how to clear an already occupied
learned slot: Shut down, apply the intended reservation, then Bring up. Source MAC
represents one replay frame; it does not replace the topology's receiving NIC or
prove human/device identity. This limitation is essential visible teaching.

Protect/restrict alternatives pass the identity and recovery tasks when they meet
the stated continuity requirement. The observable-rejection task requires actual
counted behavior and continued delivery; protect and shutdown fail different cases.
Address-list order is immaterial. Cases include unwanted-first, approved-first,
continued traffic, both directions, a neighboring secured identity arriving on the
wrong port and the second approved source. Protected settings are checked too.
Pretraining a dynamically learned approved source cannot replace an explicitly
required configured reservation. A down attachment is not a security rejection.

Checking clones current applied state for independent ordered samples; it does
not train the learner's current table or increment its displayed counter. Replay
history identifies attachment/source/destination. Blank, malformed, unsupported
and ordinary policy-rejection feedback remain distinct. Configuration edits and
new replay inputs invalidate the checked display; applying settings, actual manual
recovery and explicit checking are separate actions. Worked practice never awards
assignment credit. Imported checked flags cannot override actual failing state.

## Implementation and preservation

`port_security.rs` composes source admission with the existing VLAN/Ethernet engine
and its new bounded fixed-binding lookup. Configured and learned secure provenance
stay separate; no fabricated ordinary dynamic rows/timestamps are seeded. The
operational projection blocks both ingress and egress when disabled. The legacy
MAC state/schema/send path remain intact. `port_security_workspace.rs` owns bounded
source validation, drafts, practice, ordered trials and progress context; the
native page uses shared control metrics. No dependency or old-project code added.

All346previous scenario and122unit values/order remain exact. The74previous CCNA
presentation values match immutable346T234326Z resources: canonical238273bytes,
SHA25642f3c2e649e50370c18f47cb5f9779d488c7657485cbfdbc2909da8cfbf1fcf2.
Old aging preservation tests retain their assertions through the exact accepted346
catalog API. Private save/load/export/validated restore/safety-copy and selected
reset tests preserve drafts, favorites and an old aging attempt. Original owner
progress and installed application have not been read or replaced by these runs.

## Verification actually performed

- Static prerequisite:47focused tests, all-target Clippy and948Python checks.
- R53:45focused tests pass:17security-model,11workspace,4authored-content/
  preservation/persistence,10static forwarding and3retained aging tests.
- R56:15actual widget/capture-route/preview tests pass60.96s, including five text
  scales, narrow/wide, representative light/dark, disabled controls, actual editing,
  partial/success/broken/retry, practice isolation, disclosure isolation, explicit
  recovery and keyboard focus/Tab after reflow.
- R57:development native98109exit0, normally polled/reaped.173fixed actions/57PNG
  in `.build-artifacts/native-runs/run-0y0cncbp/screenshots/workspaces`.
  Every task's incorrect/partial/equivalent/broken/retry, occupied-slot repair,
  private save/readback/reset, normal screens, dark150wide and light200narrow ran.
  Root opened12originals across all lesson-specific screen types. This is technical
  rendered review, not a novice study. A few worked-table values received explicit
  line breaks afterward; exact-package captures must confirm that final polish.
- R59library/R60app/R61all-target Clippy pass (all-target51.99s). Python/inventory/
  strict coverage and exact optimized candidate admission are the remaining gates.

Failures were retained and fixed without weakening assertions: R22used the wrong
development catalog flag; R23missed a disclosure capture target; R28used differing
measurement/click heights (R29fixed the fixture). R30caught real field identity
changes during reflow, fixed with stable per-source/scope IDs. R33caught inherited
cross-lesson disclosure state. R36reproduced a saturated counter falsely classifying
Protect as counted; outcome classification now checks actual mode behavior too.
R39caught a renamed diagnostic category and R49caught an incomplete attribution
edit; corrected source passesR53. R44passed all166functional actions but its later
capture selector chose unrelated enlarged defaults; R45/R46fixed selection andR57
completed the entire route. These failed runs are not acceptance evidence.

## Coverage and remaining limits

Only a bounded port-security slice of CCNA200-301v1.1/5.7 is implemented. DHCP
snooping and dynamic ARP inspection remain gaps, as do sticky learning, voice/trunk
security, secure aging, automatic recovery, save/restart, full IOS interaction and
actual equipment conformance. The pinned Cisco chapter contradicts itself about
shutdown notifications; those notifications are excluded. See PORT_SECURITY_SUBSET.md
and STATIC_CAM_SUBSET.md for bounds and primary sources checked2026-11-08.

No full-inclusive Rust runtime, installed/cold lifecycle, all28-theme sweep,
Windows/macOS/Omarchy runtime, physical-device/screen-reader or independent novice
acceptance is claimed. Native save/readback is same-process; private backup/restore
is separately tested. No public release, push/merge, dependencies, cleanup or system
settings changes. The main runner's single lock, two jobs/two test threads and
verified disk-backed artifacts remain mandatory; low swap headroom is still a risk.

## Optimized capture retry

R72exact-package native failed at step10: practice remained initial after a click
into an expanding worked-example section. Grading correctly failed. Candidate-r1
T013847Z remains withheld; R73final selected readback/R74withheld receipt precede
lifting its freeze. R75app/R76all17widget-route-preview tests pass60.65seconds.
The new capture-only guard uses the recorded child clip and waits for animation;
the original15second/1200work/20scroll limits remain. Two new regressions cover
partial hit areas and asynchronous waits. Learner code/grades remain unchanged.
R77app/R78all-targetClippy and R79all951Python pass. Replacement exact-package
admission remains pending. Do not interpret the earlier development run as package
or installed/cold acceptance.

## Exact replacement Fedora package admitted

R80release4m04s; R81initial/R84final selected identity passes785source/data,
193resources,326docs,4copiedpresentations and76supplemental inputs. R82actual
binary validates349/123. R83owned57320exit0normallyreaped:173actions/57PNG;
root opened12originals across all3screen types for each task, partial feedback,
light200narrow/dark150wide. Final worked-table line breaks confirmed. R85private
receipt records exact hashes/images and precedes freeze lift. Current CHECKPOINT
contains the runnable T020018Z isolated path. Original install/progress untouched.
The earlier candidate remains withheld; no failed observation was relabeled.
No full-inclusive Rust/installed-cold/otherplatform/device/screenreader/novice claim.
