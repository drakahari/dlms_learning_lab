# Per-hop marking and queue practice

Current source:241 labs/65 units;72 CCNA,63 LPIC-1,40 LPIC-2,66 Bash/Shell.
Three new labs in **CCNA → Per-hop traffic treatment**, after IPv4 forwarding:

| ID | Meaningful assignment | Distinct investigation |
| --- | --- | --- |
| ccna-qos-mark-boundary | Rewrite trusted/guest labels while preserving varied report labels and delivery. | Keep versus constant rewrite; an incoming guest mark is not trusted. |
| ccna-qos-queue-bursts | Repair interactive waiting without losing reports or changing labels. | FIFO versus explicit priority; buffer capacity does not increase link rate. |
| ccna-qos-trusted-handover | Repair trust, priority mapping and shared capacity together. | Spoofed labels, alternative non-high guest codes, all-stream delivery. |

Each has three deterministic arrival patterns, a separate worked system,
Introduction → Try example → Assignment, intentional starter errors, progressive
help and an independent Challenge. No assignment answer is filled by default.
All238 prior lab objects and64 unit objects compare exactly; evaluator15 and all
66 scripts remain unchanged. No dependency or project-license change.

## Authoring and simulation

[Subset](QOS_SUBSET.md) defines arrivals-before-service, finite queues, explicit
marking/mapping and bounds. `qos` is a pure model; `qos_workspace` validates typed
cases/goals and source-bound drafts; `ui/qos_page` provides actual native edits,
case inspection and per-packet replay. Pack JSON is data, never an IOS interpreter.
Every editable stream needs a goal and actual evidence in every pattern. Maximum
waiting also requires delivery, so dropping a delayed packet cannot fake success.
One passing pattern does not hide another failing pattern. Nonpriority labels and
valid alternate allocations pass according to resulting behavior.

Practice and assignment retain separate drafts/tested snapshots. Edits invalidate
stale success. Load/save refuses unknown/foreign context, malformed/bounded-invalid
sessions and fabricated stored grades; outcome checks replay actual policy. Reset
only clears the selected attempt. A bounded semantically incomplete policy can
resume and produce useful diagnostics on Run. No host traffic, files, commands,
clock or service is used by learner policy evaluation.

## Focused technical evidence — 2026-11-02

Private logs: `.build-artifacts/continuation/resume-20261102/qos-*`.
Foundation11, workspace8, authored-source4 and actual catalog/progress5 groups
passed before final source gates. UI6 covers practice isolation, actual numeric
editing/stale invalidation, keyboard focus, mixed-case feedback, capture-phase
Context safety and layout/paint at100/115/125/150/200%,760/1600 widths,light/dark.
Initial failed gates and diagnostics remain; no assertion was weakened.
Final28 model/content/progress groups pass (qos-model-final.log); final6 UI groups
pass (qos-ui-final.log). Scoped production/new-test Clippy passes after collapsing
two nested conditions, with original warning log retained.100 focused Python
checks pass (qos-python-final.log), including exact inventory, resource bundling,
closed disjoint CURRENT QoS source layers and frozen-reference early-refusal.
No frozen source/collector/probe/seal was rewritten. Package verification follows
below when actually observed.

Development native runs `run-dsqv6rhm` then corrected `run-x_mg6scg` each passed
all3 examples/experiments/repair and assignments/wrong/reference-by-real-widget-
events/edit-invalidation/damage/retry/save-read/reset. Each retained33 screenshots.
The second run fixes the arrow glyph and reduces redundant control rows. All9
normal dark100 teaching screens were visually inspected individually, plus the
handover failure and light200 Check controls. Two action descriptions were then
corrected to name the actual pattern selector and paired DSCP replay column.
This is technical screen inspection, not an independent novice study.

Representative corrected development captures:

- [Introduction](../.build-artifacts/native-runs/run-x_mg6scg/screenshots/workspaces/teaching-ccna-qos-mark-boundary-introduction-dark-100.png)
- [Practice](../.build-artifacts/native-runs/run-x_mg6scg/screenshots/workspaces/teaching-ccna-qos-queue-bursts-example-dark-100.png)
- [Assignment](../.build-artifacts/native-runs/run-x_mg6scg/screenshots/workspaces/teaching-ccna-qos-trusted-handover-assignment-dark-100.png)
- [Useful failure](../.build-artifacts/native-runs/run-x_mg6scg/screenshots/workspaces/qos-ccna-qos-trusted-handover-starter-feedback.png)
- [Light200 controls](../.build-artifacts/native-runs/run-x_mg6scg/screenshots/workspaces/teaching-ccna-qos-trusted-handover-assignment-light-200-controls.png)

## Objective accounting and limits

Cisco200-301 v1.1/4.7 and RFC2474 checked2026-11-02. Sources:
https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf
and https://www.rfc-editor.org/rfc/rfc2474.html . RFC4594 retrieval returned429;
it is not inspected evidence. The36-row current continuation supplement records
these three scoped lessons; historical issuer snapshots remain unchanged.

Supplied stream identity is not arbitrary IP-header classification. Policing,
shaping,AQM/ECN,weighted fairness,variable serialization,real/end-to-end timing,
fullIOS and the parent4.7 objective remain Open. No full suite, new installation/
upgrade/cold reopen, Windows/macOS runtime, physical accessibility, screen-reader
or independent beginner acceptance is inferred from these focused checks. No
installed app/original progress,system settings,push,merge or release was changed.

## Verified separate package

Separate Fedora review candidate241/65, built2026-11-02T184007Z:
`/path/to/dlms_learning_lab/dist/teaching-review-20261102T184007Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab`
BinarySHA256 d6596ab6f9cbda0d4bd670ebf58495757539e42a203c7fd0390340f1bae65194;
archiveSHA256 7347044557032e6e7ac3a13fa096a68fb8eb8f7543c6eb411481ea6aa1a150cd.

434 source pins. The actual packaged binary validates241 authored labs/65 units. Its fixed native
QoS route passes all3 separate examples/change/retry and assignments/starter/
reference-via-real-controls/edit-invalidation/damage/retry/save-read/reset.
33 captures are in run-z2pj_xyb; corrected practice wording, handover passed
case results and narrow light200 Check controls were visually inspected. The
package embeds review mode on normal launch and protects original progress;
its separately stored review progress can resume. No installed replacement.
355 build memory samples: minimum17.23GiB available, PSIavg10max0,OOMdelta0.
No stress test, full suite, new installation/upgrade/cold reopening or external
platform/accessibility/independent novice acceptance is claimed.

[Packaged practice](../.build-artifacts/native-runs/run-z2pj_xyb/screenshots/workspaces/teaching-ccna-qos-mark-boundary-example-dark-100.png) · [Packaged passed assignment](../.build-artifacts/native-runs/run-z2pj_xyb/screenshots/workspaces/qos-ccna-qos-trusted-handover-passed.png) · [Packaged light200 controls](../.build-artifacts/native-runs/run-z2pj_xyb/screenshots/workspaces/teaching-ccna-qos-trusted-handover-assignment-light-200-controls.png).
