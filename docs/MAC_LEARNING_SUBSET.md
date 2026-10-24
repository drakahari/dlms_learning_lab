# Ethernet source learning: simulator capability

Checked2026-10-24. The pure model and an unbundled native workspace fixture are
accepted foundations. **No MAC-learning course unit or certification coverage is
registered yet.** Authored lessons need separate content and native acceptance
before catalog registration.

The pure Rust module extends the existing bounded VLAN forest with dynamic
source-to-ingress entries per switch and local VLAN. A frame learns its source,
then uses a known individual destination or floods an unknown/broadcast address
within the modeled VLAN. Same-ingress individual forwarding is filtered.
Explicit simulated time ages entries; only a source frame refreshes its entry.
Selective clear and port/VLAN configuration changes invalidate relevant entries.
A supplied older table can be relearned; no live rewiring or MAC spoofing is
claimed. MAC learning is not a privacy/security boundary.

The observation distinguishes copies physically arriving at a running endpoint
from individual/broadcast frames accepted by the modeled untagged endpoint.
Tagged copies still arrive but are rejected by that endpoint model. This is not
hardware NIC, ARP, IP, application or packet-delivery conformance.

Requirements grade resulting tables/configuration and independent cloned
frame/time probes, never command text or old events. Every probe starts from the
same current state. Edits, clear and expiry can invalidate an earlier pass.
Negative probes cannot claim VLAN separation by disabling their source/known
destination attachment or physical route. Authors must still specify all required
healthy peer flows; one unrelated positive probe cannot define the whole policy.
Unknown/broadcast destinations do not identify one target attachment.

## Bounds and storage contract

Existing fabric limits:8 switches,16 ports each,32 fixed workstations,64 VLANs per
set and64 cables, with acyclic point-to-point attachments. Dynamic tables have
at most256 unique switch/VLAN/MAC entries; traces at most512 events. Matching and
propagation are iterative and finite. One action advances at most one simulated
day with checked clock arithmetic. Aging0 disables expiry; the0–3600second range
is a simulator policy, not the complete IOS command range.

The compact serialized state ceiling is256KiB. The separate raw JSON ceiling is
1MiB so legal pretty-printed maximum state can resume. Existing progress/pack
limits still apply around it. Collection readers reject excess/duplicate values;
fields and state are validated before acceptance. Retained UTF8 diagnostics and
grade feedback use fresh allocations bounded to512bytes, including the case of a
short message backed by an oversized prior allocation. These are bounded data
contracts, not a claim about exact allocator overhead or process RSS.

Unsupported/resource faults are atomic; ordinary modeled drops retain source
learning that occurred before the drop. Equipment/cable/host identities stay
fixed when port settings change. Configuring a changed port conservatively
flushes its entries. Seeded old entries may point at a down or previously used
port; that is authored historical evidence, not normal live-carrier automation.

No sockets, host network commands, real clocks, processes, packet bytes, STP,
redundant paths, arbitrary multicast, secure/sticky/static entries, hardware
capacity/timing, full IOS command grammar or vendor endorsement is included.

## Validation and references

Seventeen integration checks plus one private diagnostic-capacity check pass,
with all-target Clippy denying warnings and formatting checks. Coverage includes
learning/known/unknown/broadcast, comparison with existing VLAN broadcast results,
VLAN/trunk boundaries, expiry/source refresh, clear, valid alternative relearning,
partial/current invalidation, atomic unsupported inputs, exact resume, oversized
readers, physical separation shortcuts, tagged arrival and a maximum-shape pretty
JSON roundtrip. These pure-model checks do not establish learner acceptance; the separate
private-workspace native scope is recorded below.

Primary sources inspected2026-10-23 and rechecked2026-10-24:

- [Current CCNA v1.1 objectives,1.13](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf).
- [Cisco IOS XE MAC management](https://www.cisco.com/c/en/us/td/docs/switches/lan/c9000/lyr2-fwd/cdp-lldp-mac-udld/cdp-lldp-mac-udld-configuration-guide/c-configure-mac.html), updated2026-09-28.
- [Cisco unicast flooding background](https://www.cisco.com/c/en/us/support/docs/switches/catalyst-6000-series-switches/23563-143.html).

Original project implementation; no vendor implementation code or manual
paragraphs are copied. Topic scope is partial and future blueprint versions stay
separate. The current catalog and objective evidence statuses do not change.

## Native workspace and progress contract

The additional State/Goal kind is `MacLearning`. Its goal stores the exact
initial model and typed outcomes. Assignment identity checks pin equipment,
cables (including supplied operational state), host roles and MAC assignments.
Learners can change existing port/VLAN settings, the explicit entry lifetime,
and learned/time state through the finite native actions. No host operation or
new terminal grammar is exposed. Existing `Switching` and all bundled scenarios
remain unchanged. The private `mac-learning-preview.json` is not bundled or
counted; no objective status is promoted.

Native configuration is a version1 draft, reusing the existing switch/port table
without changing its original widget IDs. Its raw VLAN, lifetime, destination
and time text remains saved even when malformed. Apply settings parses every
configuration field into a candidate before publishing; failures preserve the
applied state. Send frame, Advance time and selective Clear act only on the
applied simulator. Check work recomputes from current state; pending settings
cannot earn current success. Source/destination selections are input drafts,
not grading credentials. The last-frame trace is temporary UI observation,
never saved history or completion evidence.

The goal remains prominent. Port settings are visible where explicit port/VLAN
outcomes require them, otherwise available on demand. The selected switch's
learned table and finite source/destination Send controls are the main workspace.
Time/clear tools and the full trace remain on demand except where a time outcome
makes those tools relevant. Topology uses the existing equal-card components or
a truthful connection list, with actual virtual MAC identities.

Practice has its own version1 immutable authored fixture marker, mutable model,
configuration/input draft, selected example action and checked flag. Its fixture
must exactly match this lesson on restore; mutable coincidence is allowed but
foreign origins are rejected. Assignment and practice use different workstation
IDs and MAC values. Fill/Insert only populate example inputs; they never send,
clear, advance time, publish a solved state or award assignment credit. An
explicit practice action uses the same bounded simulator. The introduction is
skippable/reopenable, and Challenge opens independent assignment work without
showing the example. Lesson solution/experiment action counts are checked before
replay or clone; the actual suggested change must mutate the practice and reach
its declared outcomes.

Optional metadata defaults to absent under workspace schema1. Save/decode reject
wrong-kind drafts and malformed shapes; decode additionally pins assignment
identities and the exact authored practice origin. Existing original106 progress,
actual6/7 records, current8 scripts, histories, favorites and other drafts retain
their own semantics. The separate109-lab scripting milestone is not part of
this fixture. The isolated workspace gate used its original106 catalog; merged
main regression is separate. There is no evaluator revision change.

Limits reuse the pure model bounds above, existing16MB profile/2MB pack limits,
and the native switch draft's8×16 shape/field bounds. MAC destination text is17
UTF8 bytes; lifetime/VLAN fields4 bytes; time20 bytes. At most64 typed objectives
are decoded before accepting a further body. Internal pack-only `@mac` actions
have a4096-byte JSON envelope and a closed typed action enum. They are authoring
reachability metadata, never taught IOS/Linux commands. No general plugin,
loader, arbitrary script or external integration is added.

Ten adapter/persistence checks and seven native-egui/plan checks pass, alongside
all-target Clippy with warnings denied, formatting and an identified binary build.
The later instruction/compact-practice gate also repeats three existing VLAN UI
checks; those repetitions do not add new checks. The17 workspace checks are
separate from the18 pure-model checks. Compilation initially caught probe API
mistakes; a modal test initially read post-pass context metadata instead of actual
widget-response flags. Those failures are retained. Corrected tests keep strict
keyboard/no-mutation and direct current-widget disabled assertions. The model,
grading and saved-data semantics did not need a workaround.

Native MAC walkthroughs use prose
and the practice action selector, never a hidden generic-terminal Insert path.
Pack validation rejects terminal-command guide entries for this state kind.
Settings, reset confirmation and data dialogs disable MAC editing/actions while
open. A focused-event regression verifies that a previously focused port field
cannot edit or send behind reset/settings; other workspaces are unchanged.

Current credit remains fresh state-based, consistent with other model labs:
actual Apply/Send actions may genuinely establish every outcome before Check.
Check reveals feedback; it is not a click-history credential. No result is cached
or forced by an action. Every current check still reads the actual model and
rejects divergent pending settings. Practice never contributes assignment credit.

The app smoke flow at `DLMS_SMOKE_STAGE=mac-learning`, stage16000,
appends only the unbundled developer fixture to a private smoke session. It uses
actual menus, Insert, typed fields, Send/Apply/Advance/Check and Reset controls;
reference values become input events only. It covers the independent example,
expiry/repair, full assignment malformed/partial/current/alternate paths, actual
port edits at light150% and dark200%, saved-state readback, Cancel/Confirm Reset
and Challenge. Temporary traces clear on selected Reset. The finite plan has a
400-step ceiling, no host commands or supplied solved State. Its bounds test and
actual Fedora flow pass. The final R2 run retained17 native PNGs and passed all
assertions; root render review inspected the compact practice controls, the
correct Apply-before-Check instruction and actual100%/150%/200% fields, errors
and completion. The window manager retained a1268×632pixel window, so those
captures do not assert the requested window dimensions were honored.

The first native run is preserved separately; it exposed the shared editor's
incorrect MAC pending instruction. The editor now receives an explicit contextual
instruction, keeping VLAN text and IDs unchanged. Actual emitted-text checks
verify both contexts and the single practice action caption/adjacent Insert.
The final R2 source, binary,17 PNG hashes, native log and memory observations are
recorded in ignored `reviews/mac-learning-workspace/native-r2/RECEIPT.json` under
`.build-artifacts`. No captures, progress or private evidence enter the public
commit. Readback is not a cold-process restart, novice, screen-reader,
Windows/macOS, hardware-NIC or full-certification acceptance claim.
