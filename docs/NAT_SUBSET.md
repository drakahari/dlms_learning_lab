# Basic inside-source NAT — pure foundation proposal

2026-10-26. The pure core and typed application adapter are implemented and
independently source-reviewed. All20 core groups and23 adapter groups (including
actual historical-progress replay) pass the guarded Fedora gate. The isolated
all-target regression passes1619 checks/zero failures, with11 private checks
skipped there and the NAT-specific private replay run explicitly. All-target
Clippy passes. MAIN integration regression is recorded separately in the ledger.
This is a simulator capability, not a playable lesson or CCNA objective credit.
No native renderer, normal registration, new dependencies or evaluator change.
See [NAT_WORKSPACE_SUBSET.md](NAT_WORKSPACE_SUBSET.md).

## Meaning and boundary

This original in-process model rewrites one IPv4 address in a supplied pair.
Basic NAT maps addresses; NAPT/PAT additionally involves transport ports, which
this model omits. Static mappings work both ways before any outbound probe;
current dynamic host mappings can be reused across destinations and reversed.
The other address stays exact. Those limited distinctions follow
[RFC3022](https://www.rfc-editor.org/rfc/rfc3022.html), an Informational document,
and Cisco's [configuration examples](https://www.cisco.com/c/en/us/support/docs/ip/network-address-translation-nat/13772-12.html).
Cisco [local/global terminology](https://www.cisco.com/c/en/us/support/docs/ip/network-address-translation-nat/4606-8.html)
helps name the addresses; inside/outside are supplied roles rather than a
private/public classifier. These primary pages were opened2026-10-25. No source
implementation or copied algorithm is used. Root's reviewed contractR2 is the
specific implementation authority. Its prospective CCNA v1.1 objective4.1 link
is context for future work, not demonstrated practical coverage.

No packets, sockets, protocols, transport ports, checksum/ALG, TCP sessions,
expiry, timers, routes, Internet/DNS connectivity, firewall/security authorization,
IOS commands, kernel or host resources are modeled. Translation is not proof of
reachability or permission. A known outside peer can reverse any extant dynamic
mapping; this is deliberately not session authorization. No elapsed time or
history is invented. Overlapping/aliased supplied endpoint addresses and
outside-address translation are excluded rather than declared invalid in real
NAT. There are no physical segment prefixes: disjointness here means exact supplied
identity/address sets, not a claim that every possible overlapping routed network
is detected.

## Configuration and observable results

An immutable `Inventory` supplies a fixture ID, up to64 inside hosts,64 outside
peers, and128 allowed translated globals. IDs are1..32 ASCII bytes matching
`[a-z][a-z0-9-]*`. Identity and actual IPv4 addresses are unique across both endpoint
sets; translated globals are distinct from actual endpoint addresses. The supplied
sets may be empty, yielding no fabricated translation. Numeric IPv4 addresses are
parameters, not assertions about their existence, routability or link-host validity.

Mutable `Configuration` has up to32 static inside-ID/global mappings and an
optional dynamic rule. The latter has one canonical IPv4 source prefix0..32 and
one inclusive pool of1..64 addresses, all drawn from allowed globals. Eligibility
is applied only to supplied inside hosts, not as a general traffic ACL. Pool
cardinality is computed in64-bit arithmetic before iteration, including ranges
crossing octet boundaries and full32-bit endpoints. There is no last-octet,
classful, network/broadcast-host or private/public heuristic.

`State` keeps private validated inventory/configuration/binding fields. Actions:

- `Configure { configuration }`: validate the complete candidate first. Compatible
  edits preserve every binding. Removing a bound host from eligibility, removing
  its global from the pool, disabling the rule, reserving its global statically,
  or making that local host static fails atomically until explicit clearing. Even
  converting the same dynamic local/global pair to static requires clearing first.
- `ClearDynamic {}`: remove only dynamic bindings. Static configuration persists.
- `Outbound { inside, peer }`: static match first; otherwise reuse a dynamic host
  binding, or allocate the lowest unused pool address if eligible. Static globals
  stay reserved even when inside the pool. The mapping is per inside host, not
  per destination or port.
- `Inbound { peer, global }`: reverse static/current dynamic mapping without
  allocating anything. An arbitrary unmatched IPv4 destination returns NoMapping;
  it need not already be an allowed/configured global. Peer identity must be supplied.

Typed results distinguish `Translated` (exact before/after addresses and static
or dynamic kind), `Untranslated` (no outbound policy match), `NoMapping` (no inbound
destination map) and `PoolExhausted` (eligible new host lacks a free global).
The last three preserve both addresses and all bindings. Unknown endpoint IDs are
context errors rather than network verdicts. Clear, lowest-free allocation and
incompatible-edit policy are simulator decisions, not promises about IOS address
selection, lifetime or operational CLI behavior.

An actual dynamic allocation is the only outbound probe mutation. Every Err path
leaves the entire State exact; Configure publishes only after all validation.
Static maps are editable configuration, not immutable inventory. There is no
arbitrary object insertion, reply history or action counter as success authority.

## Bounds and authority

`State::new` and actions check borrowed counts/scalar lengths before derived sets,
serialization counting, range expansion or retained clones. Live bindings<=64;
all maps/sets reject duplicate entries and use bounded serde visitors without
trusting/reserving an untrusted size hint. Excess keys/elements are rejected
before typing their next value. Direct constructors receive the same shape,
context/collision and pool checks. Valid State fields are private; configuration
and inventory getters are readonly. Outcomes and derived `Snapshot` implement
Serialize only; a saved snapshot cannot be imported as State/grade authority.
Snapshots list static/current dynamic mappings and free pool globals, not packets.

`State::from_json` admits raw bytes<=256KiB before deserialization;
`Action::from_json` / `State::apply_json` admit<=64KiB. Use these explicit entry
points for untrusted bytes. A generic serde deserializer still performs bounded
field/domain validation but cannot enforce a raw-byte ceiling on its caller's
arbitrary reader. The internally tagged Action may use serde's outer Content
buffer under the raw64KiB limit; this is not a raw-before-parse streaming claim.
Compact admission is counted through a bounded writer, with no serialized clone.
Retained Fault messages are fresh strings<=512 UTF8 bytes; raw serde failures
become a small fixed useful diagnostic instead of echoing unknown field bodies.

`State::from_json_for(raw, expected_inventory)` and `valid_against` pin exact
borrowed immutable inventory. Intrinsic decoding alone cannot establish an
external authored context; future adapters must enforce that comparison. Valid
hand-authored current bindings are state, not proof of earlier traffic. Future
policy grading must use multiple fresh host orders plus named current/preservation
requirements so manually populated current bindings cannot stand in for policy.
No goal, hidden success credential or generic plug-in is included here.

All computation is bounded by the fixed collection sizes: validation compares
at most128 endpoint identities/addresses; the largest pool has64 candidates and
checks at most32 static and64 live mapping values. No elapsed-latency guarantee
has been measured. The worst-name/count fixture prospectively checks64+64 endpoint
IDs of32 bytes,128 globals,32 statics and32 dynamic bindings, plus full JSON
roundtrip under existing caps. No global pack/profile limit or dependency changes.

## Verification and integration boundary

Twenty deterministic test groups cover independently authored prefix/pool/two-way vectors,
static-before-traffic/Clear behavior, per-host reuse, empty/unmatched/exhausted
non-mutation, reservations/collisions, compatible/incompatible policy changes,
unknown IDs, direct oversize fields/counts, strict duplicate/excess readers,
raw exact/+1 caps, bounded UTF8 diagnostics, context/roundtrip/snapshot rejection,
two fresh host orders and worst legal shape. Expected literals were authored before
execution in `tests/fixtures/nat-address-vectors.json`; they are not captured or
regenerated from this module. These are pure deterministic tests, not host NAT
experiments, performance evidence, complete CCNA coverage or playable lessons.

The private source and gate receipts retain the original proposal, independent
reviews and frozen previous evidence. Actual public9 capture/source closure was
completed before this integration; historical observations are not regenerated
or relabeled. The typed adapter is gated separately from the future renderer and
authored unit. Current140 labs/35 units and CCNA4.1 coverage status are unchanged.
Two jobs/two runtime threads and the MAIN shared runner remain mandatory.
