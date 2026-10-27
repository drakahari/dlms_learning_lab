# Bounded Layer2 LACP eligibility foundation

Drafted2026-10-27 from Root-reviewed contract163163d1 plus the amendments below.
The pure module passes22 isolated model tests (including20 literal vectors),
76 library checks and all-target Clippy. It is not app-integrated, playable,
registered or curriculum coverage evidence. Current147 scenarios/37units remain unchanged.
CCNA v1.1 objective2.4 includes L2 **and L3**; this proposed slice is L2 only.
No source/profile/record/evaluator/curriculum semantics outside the tiny library
export change. Existing VLAN/STP/routing modules are unchanged and not composed.

## Explicit model and API

`Topology` fixes fixture, device/system-MAC, local group slots, port capabilities
and physical cables. `Configuration` has exact device/port/group keysets, active
VLANs, group administrative state/template and per-port admin/membership/mode/
rate/duplex/profile. `Environment` holds current availability of exactly those
cables. `State::new/from_json/validate_context`, atomic `reconfigure/apply`,
readonly `snapshot` and batched `evaluate` are the public pure entry points.
No command parser/IOS alias, I/O, external clock or process is provided.

A group template is a disclosed educational representation of logical interface
and local member consistency: line rate plus access/trunk profile. Each local
member must match that template. It is **not** a hidden authored target, a
first/majority-member election or simulated IOS parameter inheritance. Group
edits do not auto-edit ports. A coherent change to both is valid. Duplex is full
for eligibility; half duplex remains representable and diagnosed. Port rates
must be among the supplied port's enum capabilities; a template with no compatible
member is a valid currently unusable configuration.

Local group names need not match across the wire. `None` membership keeps a
supplied port without adding/removing topology. Templates may be access or
explicit trunks. Native and allowed-VLAN differences **between remote groups**
do not by themselves block LACP eligibility. They affect separate logical VLAN
admission below. VLAN databases likewise do not determine negotiation readiness.

## Exact order: filtering, partner selection, service

1. For each supplied cable, collect finite exclusion reasons from **both** ends:
   actual cable availability/forced-down condition, port/group admin, present
   membership, each endpoint versus its own template, full duplex, equal line
   rates and an Active endpoint on that physical pair. No reasons means an
   eligible pair. Active/Active, Active/Passive and Passive/Active are valid;
   Passive/Passive does not initiate in this fresh static calculation.
2. Only eligible pairs enter local-group neighbor sets. Down/disabled/unassigned,
   half-duplex, noninitiating or locally mismatched noncandidates cannot poison
   an otherwise healthy group. A group with no eligible neighbors is Unavailable.
3. More than one eligible remote system/group yields
   `UnsupportedPartnerSelection`. A local group with one partner is Ready only
   if that partner uniquely points back; a reciprocal peer of an ambiguous group
   is unsupported too. No arbitrary winner, silent partial partner acceptance,
   platform flag or successful denial is invented. `eligible=true` on a link is
   not `participating=true` when its pair is ambiguous.
4. **One eligible reciprocal member is Ready.** Two is a later explicit learning
   outcome, never a protocol minimum. Named queries separately demand1..8 members.
5. `ServiceQuery` names both real group contexts and a data VLAN. Admission first
   requires reciprocal Ready, its requested actual member count, active VLAN on
   both devices, and compatible access or trunk admission. Different remote
   allowed sets can still admit a common tagged VLAN. Trunk native mismatch and
   access-versus-trunk semantics are Unsupported, not generic false/true service
   or guessed PVST behavior. Other known unavailable cases have typed reasons.

This is a logical positive VLAN-admission predicate, not observed frame delivery,
throughput, endpoint reachability, inter-VLAN routing or network isolation. A
future goal must not treat Unsupported as an expected denial. It must combine
positive member/service outcomes, explicit healthy-context preservation and any
requested minimum/current/controlled-outage conditions. No action history earns
credit. Future app context validation must also pin the intended supplied
availability and protect healthy contexts; this pure module adds no app goals.

## Limits and ownership before work

-2..8 devices,1..8 ports/device,0..4 group slots/device and16 total groups;
 <=32 cables, one cable/physical jack, no self/foreign/duplicate attachments or
 duplicate individual nonzero system MACs. Parallel cables are intentional.
- ID1..32ASCII bytes; no unbounded labels/documents. Rates100/1000/10000Mb/s,
 at most3 supported rates/port; VLAN1..4094, at most64 active/allowed values.
- Raw State256KiB, raw Action/Evaluation64KiB; compact State/Action admitted as
 well. Report/Snapshot compact JSON128KiB, fixed diagnostics<512bytes. No old
 global pack/profile/terminal cap increases.
- Borrowed scalar/count shape validation precedes derived identity/attachment
 sets, encoded counting and direct Action configuration clones. Bounded map/
 sequence visitors reject duplicates and typed N+1 before parsing that next value;
 no untrusted size-hint reserve. Internally tagged serde Content can buffer under
 the outer raw cap; this is not a raw streaming/no-buffer assertion.
- At most8 service queries and4 fault conditions, each<=4 known down-cable IDs.
 Preflight exact counts/context/compact request before derived work. Compute
 current plus at most4 snapshots **once**, at most160 physical eligibility cable
 visits. Reuse each snapshot for all queries; no query rebuilds it or traverses
 physical cables. `work=physical visits+query evaluations` <=200 for maxima,
 hard ceiling256. Internal group/neighbor maps have<=16 keys/<=8 neighbors;
 group/status lookups are bounded, not an unbounded Cartesian expansion.
- Snapshots/reports borrow admitted identities/profiles/VLAN sets from immutable
 State. They do not clone State, identity strings, configurations or history.
 Small result vectors/maps are bounded by those numeric counts before creation.
 Output counting rejects an oversized report before publication. Snapshot/group
 accessors are readonly; caller mutation cannot leave a borrowed result stale.

`configure` is the only Action and retains immutable topology/environment. All
proposed direct fields/context/compact Action are checked before its owned clone;
failed admission leaves the complete original State exact. Saved State JSON is
strict schema1 with unknown/duplicate fields rejected. Empty tagged input helpers
are struct shapes. Readonly fault evaluation adds no saved runtime, mount,
allocation, packet, event or history.

## Prospective verification, not observations

`tests/fixtures/etherchannel-vectors-v1.json` has20 manually reasoned literal
cases; it is not evaluator output or a device oracle. Positive mode/rate changes,
one-versus-two members, unavailable/half/rate/member mismatch, VLAN admission
separate from readiness and a three-to-two cable failure are explicit. Native
flags or fabricated recordings are absent.

The scoped tests cover these vectors plus eligible-only mixed partner and
reciprocal ambiguity, six noncandidate counterexamples, whole-group disabled,
split remote groups, unchanged whole State during current+four fault queries,
preservation/break-again/repair, exact immutable contexts, attachments/MAC/
keysets, disconnected groups, unknown-query/fault versus known unavailability,
direct giant names/maps, schema/unknown/duplicate/numeric/raw/escaped bounds,
N+1 typed frontier, exact byte caps, maximal8device/16group/32cable/8query/4fault
shape, reverse attachment order and coherent group/member changes. These22 groups PASS in the isolated Root gate; Main integration still needs
its own checks. No performance figure or
conformance result is claimed from source review.

## Deferred work and distinct later content

Only after pure gates: separately reviewed bounded State/Goal/draft/progress
adapter; then native aligned group/port configuration and member/service table,
real events/modal/focus/Undo/save/resume/reset tests and captures; then substantive
negotiation, member-versus-service diagnosis and resilient handover lessons.
Each needs disjoint persisted practice, full assignment, two legitimate solutions,
wrong/partial/broken vectors and positive preservation. No app or normal scenario
registry exists in this batch; no existing count or objective promotion follows.

No protocol PDU/state machine, timers, convergence, exact Cisco output/flags,
standalone forwarding, suspension/errdisable implementation, static on, PAgP,
LACP priority/winner/max-bundle/hot-standby/min-links, MLAG/stacking, routed L3,
packet hashing/traffic/performance or composition with existing STP/VLAN cores.
The supported snapshot follows a disclosed simplified fixed topology, not full
IEEE802.1AX conformance or a substitute for real equipment experience.

## Primary sources checked2026-10-27

- [Cisco current CCNA200-301 v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),2024,p2/2.4 (both L2/L3).
- [IEEE802.1AX-2020 project scope](https://1.ieee802.org/tsn/802-1ax-rev/), approved2020-01-30: full-duplex P2P aggregation. Public project page only; not every normative clause/state-machine source.
- [Cisco Catalyst9300 IOS XE17.18.x, Configuring EtherChannels](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-18/configuration_guide/lyr2/b_1718_lyr2_9300_cg/configuring_etherchannels.html), updated2025-08-08: modes, consistency and standalone distinctions; no copied implementation.
- [Cisco19642](https://www.cisco.com/c/en/us/support/docs/lan-switching/etherchannel/19642-126.html), updated2005-12-14, Step2: locally meaningful keys; consulted only for that property, not current defaults.

Root contract amendments: one member Ready; partner ambiguity only after complete
live eligibility; reciprocal ambiguity blocks admission; compute/reuse5snapshots
and8queries within declared budget. These supersede any implication that two
members are required by LACP itself. Source-only public subset, no new credit.
