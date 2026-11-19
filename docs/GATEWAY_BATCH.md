# Resilient first-hop gateways: current curriculum batch

2026-11-01 local /2026-11-02 UTC. Owner accepted the teaching rollout baseline.
This batch completes the next recorded HSRP unit in the actual native review
library. No new track, dependency, interpreter or grading rule. The existing
four authored task worlds, source policies and behavioral vectors remain exact.

## Learning progression

CCNA → Keep a virtual gateway usable through failure and return. Advisory unit
prerequisite: IPv4 addressing and diagnosis; each lesson also names its previous
lesson where appropriate. Experienced learners can open any assignment directly.

| Stable ID | Task | New decision |
| --- | --- | --- |
| ccna-hsrp-virtual-gateway | Replace fragile real-router gateways | Keep two candidates ready and give clients the shared virtual next hop |
| ccna-hsrp-incumbent-return | Recover the preferred router | Understand incumbency and put preemption on the router that must reclaim Active |
| ccna-hsrp-wan-tracking | Recover from upstream-link loss | Compare effective priorities during WAN loss, arrange takeover and return |
| ccna-hsrp-resilient-handover | Safeguarded handover capstone | Preserve protected router/client settings while restoring all clients through power and WAN events |

Introductions give concise prerequisite-specific concepts and a visible worked
example on a different network. Full original explanations stay in optional
detail. Guided practice uses native router/client fields and the real Insert,
Apply configuration and Check example actions. Assignment checklists preserve
every original condition, with no completed configuration exposed by default.
Feedback explains real-router gateway mistakes, missing candidates, incumbency,
strict priority comparisons and WAN/LAN distinctions from actual modeled state.
Practice and assignment drafts/histories remain separate.

The review library grows205→209 labs and55→56 units:65 CCNA,44 LPIC-1,36 LPIC-2,
64 Bash/Shell. This is the candidate's actual searchable/resumable library, not
unregistered fixtures. Historical201/54 normal-bundle comparison sources and raw
unit bytes remain unchanged; the accepted review entry point adds the magic and
gateway packs. Nothing replaces the installed197 candidate or writes its profile.
All accepted205 lab objects and55 unit objects compare exactly in the new
registration test. No old answers, grades or completion history are invented.

## Coverage and limitations

This unit supplies bounded practical support for current CCNA v1.1 objective3.5
(first-hop redundancy purpose/functions/concepts), not complete CCNA coverage.
Only one connected IPv4 LAN, one HSRPv1 group, two routers and up to eight clients
are modeled. Roles/probes are already settled; no IOS parser, packets, ARP, timers,
measured convergence, route/IP-SLA tracking, peer partition, VRRP or GLBP behavior.
Both routers' routes and link-health facts are supplied, protected data. Temporary
fault inspection runs on a clone and cannot change saved environmental facts.

Primary sources rechecked2026-11-01 local:

- [Current Cisco CCNA v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf).
- [Cisco HSRP FAQ](https://www.cisco.com/c/en/us/support/docs/ip/hot-standby-router-protocol-hsrp/9281-3.html), updated2024-09-27.
- [Cisco HSRP preempt and tracking](https://www.cisco.com/c/en/us/support/docs/ip/hot-standby-router-protocol-hsrp/13780-6.html), updated2026-08-06.

These support shared gateway identity, takeover, incumbent preemption and tracked
priority reduction. Teaching is original paraphrase. Future CCNA objectives remain
separate. No third-party text, assets, licensing or redistribution changes.

## Verification checkpoint

Before integration, all48 existing HSRP core/workspace/content tests passed:
18 core,18 workspace and12 content. Existing tests cover two alternatives per
assignment, exact partial/wrong vectors, protected facts, stale success,
separate examples, codec/resource limits and source-aware persistence/refusal.
Log: `.build-artifacts/gateway-batch/hsrp-baseline.log`.

Additional scoped checks pass: three full-catalog registration/progress tests,
eight retained subnet/pilot tests, fifteen retained HSRP widget/admission tests,
and five new teaching/control/capture/feedback tests. This is79 distinct Rust tests with
no failures; repeated focused reruns are not added to the count. The five new UI
tests exercise actual controls for all four tasks, separate practice, stale
success, malformed drafts, retry, source-aware save/reload and all three views.
The retained geometry test covers20 width/scale/theme settings. Twelve focused
Python packaging/resource/inventory tests also pass. Production Clippy (library
and binary, warnings denied) passes, including a rerun after the feedback fix.
The final packaged native route passes. No full engine suite or broad platform
audit was repeated for this unit.

Logs: `registration-progress-tests.log`, `widget-tests.log`, `widget-final2.log`,
`python-tests.log`, `widget-feedback-final.log` and `clippy-final.log` under `.build-artifacts/gateway-batch/`.
The first packaged native route passed all four current/failed/retry/save-reload/
reset flows and the separate guided experiment, with38 actual screenshots. Native
review found Rust-style fault records duplicated below good coaching; those now
stay in optional check details. A new painted-text regression verifies the failing
return still has the correct single failed result and specific explanation, while
the default presentation contains no Rust fault record. The initial sandbox-only
launch had NoCompositor; actual display access succeeded without compositor or
desktop changes. This before-refinement package/evidence is retained, not relabeled.
Final evidence is below; the first package and38 screenshots remain as before
observations at `.build-artifacts/native-runs/run-q3lmp_s4/`.

## Remaining roadmap priorities

After this review batch: Links and Syslog capability/task gates; RPM repository
and filesystem-capacity foundations; Linux startup/environment and network
administration gaps; Bash bounded tracing, cleanup/recovery, safe file operations
and substantial dry-run/backup/resumable projects. Existing roadmap parents stay
incomplete. A review package is not full curriculum, OS or novice acceptance.


## Final Fedora review candidate — ready

Exact direct launch command (any working directory):

```sh
/path/to/dlms_learning_lab/dist/teaching-review-20261102T031512Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab
```

Navigation: **Back to library → CCNA → course-unit dropdown → Keep a virtual
gateway usable through failure and return → Learn** on the first task. Clear an
old search/Favorites/Recent filter if it hides results. Lessons appear in the unit’s
prerequisite order; experienced learners can choose Challenge directly.

Build identity: `6b54e34869e031e50d6cf0854cce3402f65c6fd4-dirty`, Rust1.99.1,
x86_64 GNU Linux, Fedora44. Uncommitted owner-approved baseline and batch remain
preserved; no commit/push/merge/install/release action. The candidate uses the
existing separate `ccna-presentation-pilot` review profile for continuity. Installed
progress/preferences are never its write destination. Smoke evidence uses another
private temporary profile on the project disk, not the owner’s review work.

Binary SHA256:
`4ecdbf42079c00926fb08483a5cd6b63b8072febb1a74d707d86a3874a2063dd`.
Sibling `.tar.gz` SHA256:
`d4f68f61876a9a15be7679d209958124df469adeb5893cce7c7533e7307160b8`.
All323 source pins and the binary match BUILD_INFO. Packaged content validation
reports209 authored labs/56 units. The final verification text was recorded after
packaging; the archive and build-time notes remain immutable. A verification
sidecar is beside the archive rather than silently changing its checksum.

`candidate-native-final.log` exits0 with both `GATEWAY_UNIT_NATIVE_FLOW_COMPLETE`
and `TEACHING_ROLLOUT_PREVIEWS_COMPLETE`. The actual packaged program receives
native widget events for the separate example’s picker/Insert/Apply/check,
experiment and repair; all four assignments receive initial incorrect checks,
reference edits, Apply/check, deliberate damage, invalidation, retry, actual
save/source-aware file reload and selected-attempt reset. No learner state or
passing badge is injected by this flow. It does not constitute a full-exit cold
reopen test; that distinction remains explicit.

Final screenshots:38 actual PNGs in
`.build-artifacts/native-runs/run-i1v6ws5g/screenshots/workspaces/`.
They include12 three-view lesson captures,14 actual practice/assignment result
captures and12 enlarged/scrolled views. Navy100% captures are2560x1440;
Light200% windowed captures are1268x632, logical634x316. All four lessons have
Introduction/Example/Assignment images; WAN tracking and the capstone additionally
have large-text practice/assignment views. Reviewed actual fields, populated and
empty values, focused/disabled controls, checklist wrapping, buttons, topology,
scrolling and specific mistake feedback. This is representative native sampling,
not an independent learner or physical-device study;150% is covered by retained
widget tests rather than a new native screenshot run.

Representative final images:

- [First Introduction and worked example](../.build-artifacts/native-runs/run-i1v6ws5g/screenshots/workspaces/teaching-ccna-hsrp-virtual-gateway-introduction-dark-100.png)
- [Guided practice workspace](../.build-artifacts/native-runs/run-i1v6ws5g/screenshots/workspaces/teaching-ccna-hsrp-virtual-gateway-example-dark-100.png)
- [Practice experiment: real gateway fails resilience](../.build-artifacts/native-runs/run-i1v6ws5g/screenshots/workspaces/gateway-guided-practice-experiment.png)
- [WAN tracking Introduction](../.build-artifacts/native-runs/run-i1v6ws5g/screenshots/workspaces/teaching-ccna-hsrp-wan-tracking-introduction-dark-100.png)
- [Independent protected handover](../.build-artifacts/native-runs/run-i1v6ws5g/screenshots/workspaces/teaching-ccna-hsrp-resilient-handover-assignment-dark-100.png)
- [Specific failed-return feedback](../.build-artifacts/native-runs/run-i1v6ws5g/screenshots/workspaces/gateway-ccna-hsrp-incumbent-return-damage-feedback.png)
- [Light200%: populated fields and focus](../.build-artifacts/native-runs/run-i1v6ws5g/screenshots/workspaces/teaching-ccna-hsrp-resilient-handover-assignment-light-200-scrolled.png)
- [Light200%: aligned action buttons](../.build-artifacts/native-runs/run-i1v6ws5g/screenshots/workspaces/teaching-ccna-hsrp-resilient-handover-assignment-light-200-controls.png)

## Resource and acceptance limits

Every heavy action used MAIN’s shared lock, Cargo jobs2, Rust runtime threads2
and disk-backed artifacts. The final170-sample build had minimum17.31GiB available,
PSI some/full avg10 maxima0, zero OOM-counter change,62 swap-in pages and no
swap-out pages. These are system-wide observations, not app-exclusive accounting.
The first318-sample build had minimum16.87GiB available, no PSI avg10 pressure or
OOM change, with existing system paging retained in its log. No stress test or
host/GPU/swap/service/model/desktop configuration change.

No implementation blocker remains for this batch. Full curriculum remains open;
CSV issuer ledgers retain their exact historical normal-bundle boundary, while
this document/reconciliation/inventory distinguish the implemented review unit.
No complete CCNA3.5, certification-preparation, convergence or equipment claim.
Windows/macOS runtime, screen-reader/physical-device and independent novice
acceptance remain pending. The owner’s assisted baseline review does not count
as novice comprehension. No new install/upgrade/uninstall acceptance is claimed;
this task explicitly preserves the installed application. Old-version reading
of new review attempts was not tested. Themes are preserved; native sampling
uses Navy and Light, not all28 themes individually. Safely stopped for owner review.
