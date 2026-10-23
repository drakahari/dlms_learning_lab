# Bounded OSPFv2 compatibility and SPF foundation

Contract fixed before implementation, checked2026-10-23. Original MIT Rust code;
no copied protocol implementation, new dependency, host commands or packets.
This is foundation-only: no engine, terminal, persistence, UI, lessons, credit or
coverage promotion. Existing IPv4 routing and subnet modules remain unchanged.

`ospf::State` reuses `ipv4_routing::Topology`, `Assignment` and `Prefix` and holds
complete typed configuration for every forwarding device and its interfaces.
Device/interface/link identities and forwarding roles are fixed by a future
caller's immutable initial state through `valid_against`. Addresses, up flags
and OSPF configuration may change. Static routes are rejected, not ignored.
Unnumbered operation and /31-/32 interfaces are excluded; the existing /1-/30
private/documentation address rules remain. Missing or non-host interface
addresses are diagnosable operational conditions, not invented neighbors.

Each router has one area, an explicit optional router ID, a local process ID
(1..65535), and an enable flag. Each interface has enable/passive flags, explicit
positive16-bit output cost, positive16-bit Hello and32-bit Dead parameters, and
PointToPoint/Broadcast type. Costs are not inferred from bandwidth. Timers are
parameters only: no clock, missed Hellos, convergence delay or retained history.
Router IDs are opaque32-bit identities, never subnet host addresses; every
nonzero value is supported, including multicast/all-ones numeric values. Missing
ID and explicit zero are distinct non-running configuration conditions in this
explicit-ID model, without a claim that RFC2328 declares zero an invalid address.
Duplicate enabled IDs block affected routers' graph use. IDs need not be assigned
to any interface.
No automatic ID election or process restart is claimed.

The snapshot reports each router/interface's direct potential peer, all modeled
blocking conditions and nonblocking address/mask warnings. Only a segment with
exactly two forwarding-router endpoints, with both configured active P2P,
usable addresses, enabled processes/ports, up flags, distinct usable IDs,
matching areas and matching Hello/Dead parameters gets reciprocal graph edges.
Different process IDs are permitted. P2P masks and source subnets are not Hello
compatibility tests: differences are reported without blocking the edge.
Broadcast-active and multiaccess-P2P operation are explicitly unsupported.
Passive interfaces advertise their usable attached prefix and form no peers;
passive Broadcast LANs need no DR/BDR operation. No asymmetric packet loss is
modeled. A numbered active P2P port can advertise its subnet while its peer is
unavailable, provided the local port remains operational.

Fresh directed SPF uses the sender interface cost on every router edge, then
adds the destination advertising interface's stub cost. Every minimum-cost
first-hop interface/peer/address is retained, including parallel equal-cost
links and converging equal-cost paths. Positive costs make the predecessor
ordering strict; no path is chosen arbitrarily. Only reachable advertisers in
the same compatible component contribute. Alternative origins for the same
prefix merge only at equal total cost. Up usable locally connected prefixes are
reported separately and excluded from exact-prefix OSPF candidates. Lookup uses
longest prefix first, connected preference on an identical prefix, then the
complete OSPF candidate. This is not a complete RIB/FIB or packet forwarder:
administrative distance, redistribution, static competition and per-packet ECMP
selection are absent. A P2P next hop may be outside the local IP subnet; the
legacy static forwarder is not changed or called to invent forwarding results.

Bounds:12 total devices,8 ports/device, at most96 endpoints/segments and96
advertised interface prefixes. Typed maps reject duplicate keys/oversized names
while decoding; public name fields are32 ASCII bytes. JSON state input is
256KiB, action input4KiB, diagnostics at most512 UTF-8 bytes retained in fresh
small allocations. Direct configurations and actions are validated by borrowing
before a candidate clone; unknown context leaves current state untouched.
The largest simple router path has11 edges plus one stub cost, at most786420,
well below24-bit OSPF infinity; arithmetic still uses checked addition. Per-source
first-hop sets are bounded by8 directly attached P2P interfaces; output is bounded
by12 router views and at most96 distinct candidate prefixes/view. Snapshot data
is derived afresh and has no decoder/authority for saved completion. State
decoding uses the bounded `State::from_json` entry point; generic serde decoding
of a State is deliberately absent. Typed actions use `Action::from_json` then
`State::apply`; this is data interchange, not a command interpreter.

API: `State::new/from_json/valid/valid_against`, borrowed `topology` and
`configuration`, atomic `reconfigure/apply`, and fresh `snapshot`. A Snapshot has
router views with router issues, endpoint compatibility conditions, directed
distances, local connected networks and remote candidate routes. `RouterView`
provides exact-prefix `route` and longest-prefix `lookup`. No typed lesson grader
or goal/record integration is introduced by this foundation.

Not implemented: actual Hello/DBD/LSR/LSU/LSAck exchange, neighbor or interface
FSMs (including Down/Init/2-Way/ExStart/Exchange/Loading/Full), LSAs/LSDB/flooding,
sequence/age/checksums, authentication/MTU/options mismatch, broadcast DR/BDR,
NBMA/P2MP/virtual links, ABRs/multi-area/stub areas/NSSA, external/default routes,
network statement CLI, loopbacks, automatic router ID and bandwidth costs.
Compatibility never means observed Full adjacency or proven synchronization.

Primary references rechecked2026-10-23:

- [Cisco CCNA200-301 v1.1,2024,3.4/a–d](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf): single-area OSPFv2 scope includes adjacency, P2P, broadcast DR/BDR and router IDs. This foundation completes none of those objectives; broadcast remains absent.
- [RFC2328, April1998, §§8.2,10.5,12.4.1.1,16.1/16.1.1/16.8](https://www.rfc-editor.org/rfc/rfc2328.html): P2P ignores source-subnet/mask checks; output costs weight directed links; numbered subnets can be stub links; SPF adds stub cost and merges equal-cost next hops. This code models a bounded derived graph, not the RFC state machines.
- [Cisco IOS XE17.x, Default Passive Interfaces](https://www.cisco.com/c/en/us/td/docs/routers/ios/config/17-x/ip-routing/b-ip-routing/m_iri-default-passive-interface.html), updated2022-11-02: passive OSPF interfaces suppress protocol operation while their attached network is represented as a stub.
- [Cisco document13689, What Does the show ip ospf interface Command Reveal](https://www.cisco.com/c/en/us/support/docs/ip/open-shortest-path-first-ospf/13689-17.html), updated2005-08-10, version-independent: process IDs are locally significant; explicit interface cost, router ID, network type and timer fields are separate configuration.

Known simplifications above are deliberate model limits, not vendor behavior
claims. No IOS, device-equivalence or full RFC/CCNA conformance is claimed.
