# Pure bounded spanning-tree steady-state foundation

Contract fixed BEFORE implementation, checked2026-10-25. This is an original
in-process calculation, not STP/RSTP/PVST protocol execution. No code or lesson is
counted as practical coverage. All20 core tests and68 library tests pass the first guarded gate, including an
independent256-graph oracle. All-target Clippy passes after two predicate-preserving
nested-if style corrections; a final core-only retry also passes all20 checks after those style-only changes. No existing switching, VLAN, MAC, packet, engine, progress, UI,
scenario, unit, dependency or coverage model is changed.

## Supplied graph and supported identifiers

One supplied instance has a fixed12-bit system-ID extension0..4095. It is only an
identifier component; there is no VLAN membership, tagging, PVST interoperation
or multiple-instance model. Up to12 bridges each have0..8 supplied ports. Up to48
links each attach exactly two ports on different bridges. Each port attaches to
at most one link. Parallel links and physical cycles are supported. Self-links,
shared-media LANs, multiple attachment/fanout on a port, and backup/master roles
are explicitly unsupported. An unconnected port has no operational link; this
module supplies no endpoint/edge-port model.

Fixed topology contains bridge names, conventional individual nonzero MAC
identities, port names/numbers and link endpoints. The existing bounded
`mac_learning::Mac` representation is reused. Multicast/zero MAC bridge identities
are an explicit unsupported input boundary. Duplicate MAC identities, duplicate
port numbers on a bridge, unknown references and ambiguous attachments reject.
Names are1..32 simple ASCII bytes. Port numbers are1..4095. Bridge priority is
0..61440 in4096 increments; its OR with the common extension forms the16-bit
BridgeID prefix before the MAC. Port priority is0..240 in16 increments; PortID
orders priority before the12-bit supplied port number. Labels and insertion order
never break a protocol-identifier tie.

Mutable configuration supplies bridge priority/online status, each physical
port's priority/positive cost/up status, and each physical link's up status.
Online/up describe physical availability, not disabling STP while forwarding
without protection. Every configuration contains exactly the authored topology's
bridge/port/link identities. A link is operational only when both bridges and
both ports plus the link are available; asymmetric BPDU/data loss is excluded.
The whole topology is pinned through `validate_context` for any future caller.

Costs are explicit1..200000000 long-path values, not inferred bandwidth or
claimed Cisco defaults. The maximum simple path has11 edges and a cost of
2200000000, representable in32 bits; additions remain checked. Zero/negative,
out-of-bound and malformed costs/priorities reject rather than silently clamp.

## Current derived result

Each connected component of online operational links elects its lowest composite
BridgeID as root. An online isolated bridge is its own component root. An offline
bridge has no root/cost/root port in the result. Root cost is zero at the root.
For other bridges, each traversed link adds the receiving bridge's local port
cost as root information travels out from the root. Rootward path notation thus
adds the current bridge's departure-port cost; the remote root port's cost is
not charged instead. Different port costs at the two ends are meaningful.

The least root-port candidate orders:

1. RootBridgeID.
2. Sender's root cost plus this receiving port's cost.
3. Sender BridgeID.
4. Sender PortID.
5. Receiving PortID.

The final receiving-PortID comparison is represented for completeness; the first
four fields cannot all tie on distinct links in this restricted P2P graph because
sender port numbers and bridge identities are unique. Shared LAN cases where
that final tie can occur are not simulated. The designated end of each
operational P2P link is the lower advertised vector
`(RootBridgeID, sender_root_cost, sender_BridgeID, sender_PortID)`; it does not add
that end's local port cost. The other end is Root if that port is the bridge's
selected root port, otherwise Alternate. Lower numbers mean higher priority.

Only stable port outcomes are reported: Root/Designated are Forwarding,
Alternate is Discarding, and unavailable/unconnected ports are Disabled and
Discarding with a typed availability reason. This omits Learning and all
transient port states; it does not claim a real port has already converged.
Positive cost makes every selected root-port step strictly decrease root cost.
A link belongs to the abstract forwarding tree only if both endpoints forward.
An independent forest/connectivity check guards the derived result. Each
operational component must have one loop-free forwarding tree spanning exactly
that component. The path query follows those derived links; it proves no MAC
lookup, traffic delivery, VLAN path or real hardware behavior.

Snapshots and path queries derive afresh. No events, learned message cache,
timestamps, history, synchronization credential or success string can authorize a
result. Snapshot data has no Deserialize implementation. A later workspace must
check current state and its immutable context, not trust a saved snapshot. Down
links can produce valid separate components; future assignments must add positive
reachability and explicit preservation goals to reject isolation shortcuts.

## Typed API and bounds

`Topology` and `Configuration` have bounded typed decoders, duplicate-key refusal
and borrowed validation before derived sets or candidate clones. `State` owns
validated private values, supports a bounded raw decoder, immutable-context
validation, atomic typed changes and fresh snapshots. Actions only change known
priority/cost/physical-availability values or replace one full shape-preserving
configuration. There is no CLI, shell, plugin, packet/socket, host filesystem,
process or network access. Rejected actions retain the exact prior State.

Raw State JSON is limited to256KiB; typed Action input to64KiB; each retained
error is a fresh UTF8-safe allocation of at most512 bytes. Bounded map visitors do
not reserve untrusted size hints and refuse an excess key before its typed value.
Generic serde nested input cannot measure an outer raw source span: a future pack
or progress reader still needs its existing raw byte ceilings. Shape/scalar
validation caps12 bridge records,96 port records and48 links before comparisons,
sets, serialization or cloning. Snapshot output is capped by those same physical
counts. The bounded root-cost calculation uses at most11 relaxation passes over48
links; no per-cost/per-time loop exists. The independent path query visits at
most12 vertices/48 links.

The prospective test suite must cover root priority/MAC order, sender-versus-local
port tie breaks, receiving-port cost/asymmetric links, parallel paths/cycles,
disconnected/offline components, current role invalidation, exact forest paths,
checked maximum arithmetic, malformed/ambiguous graphs, atomic edits, context
pins, bounded readers/diagnostics, maximum shape compact/pretty roundtrips and an
independent finite simple-path oracle. Passing compilation/model tests would
remain distinct from native/curriculum acceptance.

## Primary references and missing protocol behavior

Read from their primary publishers2026-10-25, without executing host reference
commands or importing implementation code:

- [Cisco CCNA200-301 v1.1,2024,2.5/a–d](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf):
  Rapid PVST+ interpretation includes roots/port names, states/roles, PortFast and
  guards. This pure slice promotes none of those rows.
- [Cisco document24062, Understand Rapid Spanning Tree Protocol, updated2023-02-09](https://www.cisco.com/c/en/us/support/docs/lan-switching/spanning-tree-protocol/24062-146.html):
  stable root/designated/alternate roles, role/state distinction and
  forwarding/discarding; backup is distinct and excluded here.
- [Cisco STP Configuration Guide, chapter updated2026-07-17](https://www.cisco.com/c/en/us/td/docs/switches/lan/c9000/lyr2-fwd/stp/stp-configuration-guide/m-stp.html):
  composite bridge identity, bridge/port priority increments and explicit
  long-path cost range. The published chapter title is IOS XE17; its current
  feature table also mentions26.2.1ea, so no single tested device release is
  inferred from that evolving page.
- [Cisco IOS XR7.10.x NCS540 Multiple Spanning Tree Protocol chapter](https://www.cisco.com/c/en/us/td/docs/iosxr/ncs5xx/l2vpn/710x/b-l2vpn-cg-710x-ncs540/multiple-spanning-tree-protocol.pdf),
  pp1–2 and examples: root cost, sender BridgeID and neighboring PortID order.
  Only those common election principles are used; MST regions/gateway behavior is
  not implemented or implied.

Not implemented: BPDUs/wire fields/transmission, hello/age/forward-delay timers,
proposal/agreement or sync state machines, protocol migration, actual
convergence/failure recovery latency, stale/lost/unidirectional messages,
learning/MAC flushes, topology-change notifications, PortFast/edge behavior,
BPDU filter/guard, root/loop guard, backup/master ports/shared LAN, root
primary/secondary CLI automation, EtherChannel, VLANs/PVST+/Rapid PVST+/MST
interoperation, authentication or equipment/kernel administration. An immediate
new calculation after an edit is a new ideal steady-state result, never measured
rapid convergence. All unsupported exam content stays Open.

## Concrete source schema

The standalone raw format is exactly a `topology` and `configuration` object.
For example (authored data, not a captured device configuration):

```json
{
  "topology": {
    "system_id_extension": 1,
    "bridges": {
      "a": {"mac":"02:00:00:00:00:01","ports":{"uplink":1}},
      "b": {"mac":"02:00:00:00:00:02","ports":{"uplink":1}}
    },
    "links": {
      "ab": {"a":{"bridge":"a","port":"uplink"},"b":{"bridge":"b","port":"uplink"}}
    }
  },
  "configuration": {
    "bridges": {
      "a": {"priority":32768,"online":true,"ports":{"uplink":{"priority":128,"cost":4,"up":true}}},
      "b": {"priority":32768,"online":true,"ports":{"uplink":{"priority":128,"cost":4,"up":true}}}
    },
    "links": {"ab":true}
  }
}
```

API names are `State::new/from_json/validate/validate_context`, borrowed
`topology/configuration`, `reconfigure/apply/snapshot` and
`Action::from_json`. Action decoding checks bounded syntax and intrinsic scalar
shape; applying additionally checks the actual immutable context. Snapshot offers
borrowed bridge/link/component views and `forwarding_path`. Its stable role and
state enums are typed results, never a stored success credential.

Twenty prospective tests are source-authored and UNRUN. They include an
independent enumeration of every simple path in all64 labeled four-bridge
physical graph shapes under4 cost/root-priority configurations, plus independent
disjoint-set cycle checks; separate hand-derived parallel/asymmetric/identifier
cases; full12-bridge96-port48-link compact/pretty roundtrips and maximum11-hop
cost arithmetic. These are not host/vendor observations or claimed test passes.
