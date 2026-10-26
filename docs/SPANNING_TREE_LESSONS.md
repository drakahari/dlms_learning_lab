# Ideal spanning-tree unit — authored source, normal-unit acceptance pending

Four distinct assignments in `scenarios/spanning-tree-v1.json` are now bundled.
The source catalog is133/33; all four normal lessons pass development-native
flows with48 actual captures. See SPANNING_TREE_LESSONS_NATIVE.md for exact
identity and retained failures; installed133 acceptance remains pending.
The accepted installed candidate remains125/31 and coverage is unchanged.

| Task | Full practical outcome | Distinct reasoning |
| --- | --- | --- |
| `ccna-stp-root` | Elect the intended root without disconnecting competitors | Composite bridge identity, priority before MAC |
| `ccna-stp-local-cost` | Prefer an indirect rootward path and retain the healthy peer | Receiving bridge's local cost plus advertised cost |
| `ccna-stp-parallel` | Prefer the second parallel uplink while keeping both available | Sending-port tie-break; a legitimate cost alternative also passes |
| `ccna-stp-handover` | Repair root and leaf-path faults in a four-bridge network | Root selection alone is incomplete; preserve the whole redundant topology |

Each introduction teaches its prerequisite concept before the full task, with a
separate editable graph and independently authored change/repair results. Native
bridge/port/link settings replace CLI imitation. Internal JSON actions only support
declarative authoring/reachability; optional guides contain no pretend IOS commands.
All physical availability and positive connectivity requirements are explicit.
Equivalent priorities, costs and repair order pass where the task permits.

Six content groups pass alongside20 pure-core and18 adapter checks: references,
alternate strategies, sender-versus-receiver priority misconception, partial
capstone repair, every-link isolation refusal, fresh invalidation/repair, isolated
Fill-only practice with exact experiment vectors, malformed pending-draft resume
and selected reset. Initial prerequisite metadata mismatch was corrected to the
scenario's IDs. One test wrongly included the additional applied-draft feedback
row in the authored vector comparison; it now checks that row separately. Expected
outcome vectors/model/Goals were unchanged. Independent content review accepts.

The native workbench has scoped **development technical-fixture** acceptance,
separate from these four normal lessons. Its15 headless groups cover actual
editing/Undo/reflow, modal blocking, repeated Apply persistence and the controller.
The final Fedora run captures16 actual946x929 views at100/150/200%, Light/Dark:
Intro, separate Fill/Apply/Check/change/repair, wrong and pending work, reference
and alternate current solutions, invalidation, typing, warm save/revisit and reset.
Representative actual final renders were inspected. It is not packaged acceptance,
a cold-process test, a physical-device/novice study or other-platform acceptance.

Native inspection found a real scroll lock missed by rapid headless frames:
practice feedback repeatedly requested reveal while already checked, undoing wheel
scrolling. Reveal now occurs only on Check; an idle-frame regression passes.
Earlier pointer-target corrections alone did not resolve it. The final controller
also marks its synthetic favorite setup dirty before save, matching native update's
autosave order; full decoded-attempt/favorite assertions remain. Failures/logs/images
are retained privately. Grading was never weakened to complete a capture.

No protocol packets, BPDUs, measured convergence, Learning transition, PVST/VLAN
instances, PortFast, guards, MAC forwarding or real switch administration are
modeled. This can support only a narrow future Partial slice of current CCNA
v1.1,2.5/a/b after normal-content acceptance, never full Rapid PVST+ coverage.
See [the exact model](SPANNING_TREE_SUBSET.md) and
[draft/progress boundary](SPANNING_TREE_WORKSPACE.md).

Primary sources checked2026-10-25:
[CCNA200-301 v1.1](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf)
and [Cisco document24062,2023-02-09](https://www.cisco.com/c/en/us/support/docs/lan-switching/spanning-tree-protocol/24062-146.html).
Future exam versions remain separate and uncredited.
