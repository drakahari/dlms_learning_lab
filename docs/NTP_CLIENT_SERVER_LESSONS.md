# NTP client/server dependency lessons

Four distinct configuration and troubleshooting assignments are registered in
the normal native catalog (190 labs across50 units). They build on the bounded
single-upstream NTP model while retaining the approved Introduction → separate
ungraded example → full assignment workspace. Registration does not imply full
CCNA coverage or final native/installed acceptance. Current evidence is below;
CHECKPOINT.md records subsequent gates.

| Assignment | Required work | Supported alternative outcomes |
| --- | --- | --- |
| Build a client/server chain | Configure Harbor-relay to an allowed healthy reference and both branches to that relay; preserve an existing peer. | Three different allowed roots; any safe configuration order. Direct branch-to-root service is usable but fails the requested topology. |
| Recover a dependency cycle | Diagnose the Atlas/Birch cycle and unknown Floor-client address, then restore both relays and the client. | Break either relay's cycle edge, or configure both relays to allowed roots. A configured source without a usable complete chain cannot pass. |
| Diagnose asymmetric delivery | Read the supplied successful-request/missing-reply facts; move Primary-relay to a usable allowed root, configure Reserve-relay, and serve both clients through functioning relays. | Different allowed healthy roots and either client-to-relay choice. Reference-101 is healthy for other clients; the failed directed exchange is specific to Primary-relay. |
| Safeguarded source handover | Replace unhealthy Reference-104 at both relays and restore service to both downstream clients while retaining the exact supplied downstream and peer associations. | Different allowed healthy roots for each relay. Directly reconfiguring clients or changing a protected peer cannot satisfy preservation. |

The latter three require diagnosis and configuration from read-only evidence,
not a multiple-choice response or address substitution. Every configurable node
is either explicitly checked for current synchronization/source policy or has a
protected association. A protected-association row also requires current synchronization. Thus an
ungraded broken dependency cannot hide behind a configuration-only check.

## Teaching and native workspace

Plain-language Introduction teaches IPv4 identities, client/server chains,
strata, association removal, configured versus synchronized status, both delivery
directions/UDP123, source policy and preservation before assessment. Existing
`ccna-ipv4` and unit `ipv4-foundations` are advisory prerequisites. Task order is
advisory too; no walkthrough or quiz gates assignment access.

Each assignment has a separate persisted Aspen/Willow/Cedar/Maple practice world
with different addresses, relay names and preconfigured downstream/peer state.
Insert selects a device and prepares a draft without executing it. Run explicitly
applies one documented command. A worked example restores its relay, a deliberate
source removal then breaks both relay and downstream, and the repair restores the
same source. Willow starts in a different two-device cycle: inspect11→12→11,
remove one association, and configure an independent reference before checking
both nodes. The plain concept explains why mutual dependency cannot reach a
reference before the cycle assignment. Assignment state never receives example
work or credit.

The native page offers Introduction, Separate example, full Assignment and
Challenge. A bounded shared dropdown chooses a simulated device; a shared
centered command field runs the documented single-upstream subset. Run and Check
share an adjacent wrapping row, with current feedback immediately below. A compact
shared topology pair uses equal cards and center connectors for the selected
client and configured upstream; an unconfigured device has a single shared card
and no invented dependency edge. The full selected chain, failure reason,
permitted source addresses/health, selected exchange and any remote failing-hop
request/reply/UDP123 evidence remain visible by default. Thus a healthy first hop
cannot conceal a missing reply farther upstream. Full all-device chains and every
supplied directed fact remain available in optional inspection, rather than an
always-visible eight/nine-device status wall. No required goal or observation is
removed. Shared bounded popup/disclosure controls preserve wrapping and text
scale behavior. Failed checks say “Needs attention”; success says “Passed”.
Protected checks name the supplied versus current upstream and current sync state.
Successful retries clear obsolete command errors. Shared Next is available only
for current assignment completion with no pending text, command/Check or scope
action. The final source includes a real sibling-navigation/refusal regression;
normal registration is integrated; actual desktop and final acceptance remain
separate gates.

`SIM select IPv4` in a reference is an app/test navigation helper, never an IOS
command. Native learners use the actual Selected device dropdown. The other
command strings are the existing modeled IOS-like subset. No mode prompt, daemon,
host shell, packet, socket or host clock is introduced. A synchronized modeled
dependency does not prove UTC accuracy, real convergence, measured reach, delay,
offset, jitter, authentication or full device conformance. An explicit cycle
diagnostic is a teaching invariant, not a claim that RFC5905 uses this exact
algorithm. The single-upstream restriction is a disclosed simulator limit;
real IOS permits more associations and optional syntax.

Assignment help reveals one persisted hint at a time. A separate, explicitly
labeled worked-approach disclosure reveals the complete plan; technical URLs and
full command/limit reference are independently requested. Challenge starts with
these disclosures closed. Guidance actions cannot dispatch a pending command or
Check/Next, even with mixed Enter/paste/held input. Errors appear beside Run;
source Unicode arrows are presented as supported ASCII without changing content
fingerprints or grading. Core/source values remain unchanged.

## Admission, persistence and bounded input

New typed `State::Ntp(Box<Session>)` and `Goal::Ntp(Box<Definition>)` keep practice,
assignment, selected device, draft, transcript and current checks separate. No
optional field is added to Scenario or WorkspaceData. Exhaustive borrowed guards
refuse old-family teaching/drafts on NTP sources and attempts. Older families
retain their existing behavior; legacy teaching preparation refuses NTP state.

Every current/practice world passes pure-core validation and a32KiB serialized
model cap. The source preserves the core's16-node/256-directed-fixture bounds.
Targets, allowed-reference vectors and protected identities are bounded at16;
duplicates and missing/foreign identities are rejected. Sessions cap at512KiB,
definitions128KiB, and a complete source/Attempt at1MiB. Both transcripts retain
at most32 rows of512-byte input/8192-byte output. Their bounded serde visitors
refuse an excess body before reading it. Outer pack2,000,000/progress16,000,000
raw-byte checks remain before parsing. Direct serde is not semantic admission
or a512KiB heap guarantee; publication/use requires full semantic checks.

All commands stage a candidate and validate before publishing. Root-reference
health/stratum, labels, roles, online state and the exact ordered delivery vector
must match the author's world. Core `environment_matches` and
`check_with_environment` enforce this; saved initial markers must also equal the
actual Goal's initial/practice fixtures. Reports, transcript prose and saved
checked flags cannot create synchronization or completion.

Normal NTP saves need the catalog. `progress::save_with_catalog` validates each
NTP Attempt against its actual Scenario/Goal before entering the unchanged main
writer. The native save path uses it. The older catalog-free `save` refuses NTP
profiles rather than trusting jointly forged current/initial markers. Existing
old-family saves retain their behavior. Restore already supplies its catalog;
decode performs the same NTP context checks. Opening validates source and saved
context before any visit clock, preparation, selection, header or draft mutation.
Portable export/inspection/restore and summaries use the app-owned catalog through
the existing versioned reader/current-completion path; imports carry no catalog
authority. All NTP attempts, including inactive, wrong-kind, missing-source and
foreign-source attempts, are checked before a normal save. New tests observe actual produced
main-save receipts separately from a fresh in-memory None, and preserve them on
reopen/reset/refusal; they invent no historical evidence or mastery.

Native edit admission scans the entire text packet before dispatch. A bounded
TextBuffer never truncates an ASCII/UTF-8 insertion into a valid command prefix.
Rejected, dropped, Enter-before-Paste and aggregate-overflow edits retain the old
draft and native selection/Undo, veto actions on that root frame and preserve a
held invalid gesture through release. Header, view, example and device selectors
wait while text is pending. Only an explicit later valid action can run.

## Evidence, preservation and acceptance limits

At final M481,38 scoped groups pass:14 content/persistence, seven catalog/
registration, fifteen real-egui workspace groups and two native-event/capture-
phase controller groups. The original18 core groups also passed. M460
exercises four taught references and two distinct valid alternatives each,
plausible wrong/partial/report-only outcomes, source-policy versus usable-service
requirements, practice isolation, current-success invalidation and repair,
read-only environment protection, joint-marker forgery, actual source-aware save
receipts, export/restore, reset/resume, invalid packs and bounded raw input.

The widget scopes exercise actual dropdown/edit/Run/Check/Challenge/reset and
Next gestures, focus and native Undo, whole-packet refusal for ASCII/UTF-8/late/
aggregate input, compact topology cards/connector centers, adjacent feedback,
remote failure facts and separate cycle practice. The transcript regression
checks40 combinations of assignment/example, five text scales, Light/Dark and
760/1600 widths, actual painted report glyphs/clipping and real old/latest history
wheels with the whole profile and genuine receipts unchanged. Its content width
accounts for the native allocated scrollbar; text is not shrunk to fit.

The updated controller retains all1369 previous actions/90 captures and adds
20 real help interactions/four requested-body captures. M477 passes its actual
event/paint controller, including an intentionally unpainted offscreen hint
followed by real-wheel reveal. M480 actually runs1389 events/94 Fedora captures
and exits0; independent16-after/9-before original review verifies selected
whole/error/help/cycle/100–200% light/dark wide/narrow views. This is development
native evidence, not a novice study, screen-reader/physical-device or installed
acceptance. M481fullinclusive passes2,788 tests with0fail/0ignored; all-target
Clippy and424 Python checks pass. Exact build/source and prior correction
identities are in CATALOG190_MILESTONE.md. Packaged lifecycle and cold reopening
remain separate required gates.

The full old186 scenarios and49 units remain independent preservation evidence.
A closed raw50→49/full-typed inverse validates the complete previous source/unit
prefix and exact new four-task append before unchanged LVM49→48, Loop, OneOff,
Physical and older historical proofs. Warm helpers compare complete borrowed
source fields, including every guide/command, without replaying grading on idle
library frames. Current-family behavior/foreign-state loops retain all190 sources.
The original57 scripts/442cases/57examples and genuine9–13 producers/seals remain
unchanged. No historical answer, current mastery or new completion credit is
invented when adding the unit.

Normal Save validates every LVM/NTP attempt independently against the app-owned
catalog, including inactive, missing-source, wrong-kind, foreign-metadata and
joint-marker cases, before the existing atomic writer. The catalog-free API
honestly refuses those new source-bound families. Load/export/inspection/restore
and completion use authored Goals, not a restored profile's self-carried world.
Older writer/provenance behavior remains unchanged. These scoped persistence
checks do not substitute for cold reopening the packaged application.

Coverage remains narrowly bounded: client/server hierarchy, configured-source
verification, supplied exchange diagnosis and policy-preserving source handover.
No packet timing, offset/jitter, filter/discipline, real convergence or UTC,
multiple-peer selection, authentication/NTS, ntp master, DNS/IPv6 or host clocks
are modeled. Supplied exchange facts are not a routing/ARP simulation or measured
traffic. LPIC time administration and future CCNAv2.0 objectives receive no
inherited credit. After actual native/full gates, current CCNAv1.1 4.2 is
marked narrow Partial practice in the coverage matrix; it is never complete
exam preparation or equipment equivalence.

No dependencies, host execution, installers or releases are introduced by this
unit. Windows/macOS runtime, screen-reader/physical-device and novice acceptance
remain pending until actually performed.

## Primary-source accuracy check

Read-only official sources were checked2026-10-29. The
[current Cisco v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf)
place NTP client/server configuration and verification in4.2. The
[Cisco command reference](https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/bsm/command/bsm-cr-book/m_bsm-cr-n1.html)
describes server association/removal and diagnostic commands; its actual syntax
has options and multiple associations that this model deliberately excludes.
[RFC5905](https://www.rfc-editor.org/rfc/rfc5905.html) supplies the hierarchy and
unsynchronized stratum interpretation, but includes wire, timing and selection
algorithms outside this dependency model. These are paraphrases, not copied
manual passages or a runtime reference comparison.
