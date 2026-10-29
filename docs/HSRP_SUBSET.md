# HSRP foundation: a settled two-router teaching model

Source proposal checked2026-10-29. This foundation adds no scenario, Goal, native
workspace or catalog registration. CCNA200-301 v1.1 objective3.5 remains **Open**
until meaningful content, interactions and acceptance evidence exist. It is not
complete FHRP coverage or an HSRP wire implementation.

## Supported state and actions

One supplied connected IPv4 LAN has one fixed HSRPv1 group(0–255), a fixed virtual
IP, exactly two routers(A/B) with distinct real addresses, and1–8 clients. Fixed
identities include labels and client IDs, addresses, LAN prefix and virtual IP.
Client gateways, HSRP participation, priority, preemption and one optional WAN-link
tracking decrement can change through typed simulator actions. Fault events
explicitly change modeled power, LAN link, WAN link, supplied upstream-route
availability or client-link facts. These never affect actual devices.

The model stores the active and standby identity. At simultaneous startup with
no incumbent, the higher effective priority wins; a tie uses the higher real IP.
An eligible active router stays active unless the other router has preemption
and a strictly higher numeric priority. A higher IP alone does not preempt an
incumbent. An active-router power/LAN/participation failure lets the eligible
standby take over without requiring preemption. A returning former active router
stays standby unless its own preemption and strictly greater priority justify
another transition. Saving/reopening preserves this history, rather than sorting
priorities again.

The optional track monitors only the supplied WAN link's up/down fact. On a down
link it subtracts the configured positive decrement. This subset restricts base
priority to1–255 and decrement to1..base_priority, excluding the upper end, so
effective priority stays positive. Priority zero, underflow/saturation, multiple
track objects, route tracking and IP SLA are explicitly unsupported. A failed
untracked WAN does not cause magic failover. A lost route with an up WAN does not
change a link-tracked priority. A competing router still needs preemption and
strictly higher priority to replace an otherwise eligible incumbent.

Each client's observation distinguishes the virtual address from a particular
router's real address. A virtual gateway follows the active identity; a real
gateway remains tied to that router, even when another router is active. Disabling
HSRP participation does not disable a router's ordinary real-address forwarding.
State-derived outcomes distinguish bad/off-LAN/unknown gateways, client-link loss,
no active router, router power/LAN loss, WAN-link loss and unavailable upstream
route. `ForwardingAvailable` means only that these supplied virtual conditions
permit off-LAN forwarding. It is **not** a ping, packet receipt, end-to-end
reachability measurement or proof of real IPv4 routing/ARP.

## Timing and network boundary

Every accepted action produces the already-settled result on a **connected LAN**.
There are no wall clocks, HSRP packets, hello/hold timers, transient states,
measured loss/convergence, startup/preemption delays, MAC/ARP caches, authentication,
IPv6, more than two routers, multiple groups, VRRP or GLBP. Real HSRP failure
recovery takes time and depends on packet delivery and configuration. This model
must never be used to teach instantaneous real-device recovery.

`PeerCommunicationLost` refuses the action atomically. Real partitions can permit
multiple active routers; this foundation does not choose one arbitrarily or claim
to repair split brain. Snapshots with duplicated/ineligible roles or an unresolved
strictly-higher-priority preempting peer are rejected. Valid lower-priority
incumbents without a qualifying preempting peer are accepted on resume.

Fixed endpoint validation reuses the application's full32-bit IPv4 mask function,
including prefixes that cross octet boundaries. Conventional host-address rules
exclude each subnet's network/broadcast addresses and /31-/32; /0 is not a shared
LAN here. /1–/30 arithmetic is supported, but every world still needs four distinct
usable addresses(VIP,two routers,one client), so /30 cannot fit a valid world.
Examples must not imply only the last octet can change.

## Persistence, safety and future authoring

Version1 JSON has a16KiB raw bound before parsing, strict unknown/duplicate-field
rejection, exactly two routers, at most eight clients,32-byte ASCII endpoint
labels, distinct bounded IDs/addresses, checked u64 event counters and bounded
serialization. Collections, scans and elections are limited to these fixed
bounds; there is no learner execution loop or unbounded history/output buffer.
Labels cannot be paths. All configuration/fault actions validate, clone a bounded
state, apply, settle and validate before publishing; any failure preserves the
full prior state and revision. Read-only observations never change state.
There are no host filesystem, process, network, clock or unsafe APIs and no new
dependency. The JSON codec only consumes/returns caller-owned bytes.

`environment_matches` checks fixed physical identities against a source world;
it intentionally allows documented configuration and typed-fault changes. It is
**not** task grading or authorization to alter protected fault/config facts.
Future content must define which facts learners may change and bind initial and
current sessions to its authoritative authored source, rather than trusting a
forged snapshot marker or active role. Grade current client behavior across
normal/failure/return cases; do not award credit for one preferred priority string
or role alone. A real-gateway workaround can appear healthy before failure and
must fail the resilience assignment. Do not persist a successful grade across
new damage without recomputing the outcomes.

Reset should clone the source initial state; resume should validate and retain
incumbency. Future assignments need separate example worlds and drafts, clear
native controls, failure diagnostics, legitimate alternatives, full save/load
source-context guards and rendered acceptance. This foundation supplies none of
those catalog or UI claims.

## Sources and licensing

Primary sources checked2026-10-29; paraphrases only, no Cisco code, text, logos,
protocol tables or assets are imported. New model/tests are original project code;
existing project and third-party notices remain unchanged.

- Cisco, [CCNA200-301 v1.1 official objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),
  objective3.5: purpose/functions/concepts of first-hop redundancy. This entry is
  separate from any announced future exam version; no future mapping is inferred.
- Cisco IOS XE17.x, [Configuring HSRP](https://www.cisco.com/c/en/us/td/docs/routers/ios/config/17-x/ntw-servs/b-network-services/m_fhp-hsrp-0.html),
  updated2024-03-27: operation, priority/preemption and addressing sections. These
  distinguish initial IP tie-breaking from strict-priority preemption after an
  incumbent exists, and explain that hosts use the virtual gateway. Timer and
  packet behavior are deliberately omitted here, not claimed as implemented.
- Cisco doc9281, [HSRP FAQ](https://www.cisco.com/c/en/us/support/docs/ip/hot-standby-router-protocol-hsrp/9281-3.html),
  updated2024-09-27: LAN-failure takeover, initial equal-priority election,
  default priority100, and tracked-link decrement requiring peer preemption.
- Cisco doc13780, [HSRP Preempt and Track](https://www.cisco.com/c/en/us/support/docs/ip/hot-standby-router-protocol-hsrp/13780-6.html),
  updated2026-08-06: interface-line-state tracking and the distinction between
  HSRP and separately supplied routing. Its general priority description is read
  with the IOS guide's explicit exception for equal-priority incumbent preemption.

All four official sources were accessible at this check. No physical devices or
host network references were used. The monitored M444 run passed all18 standalone
test groups. They cover elections/history, higher-IP false
preemption, tracked/untracked failures, real versus virtual gateways, partial
repair, damage invalidation, alternatives, codec/resource/isolation boundaries,
resume and reset. M448 all-target Clippy also passes. These are pure-model gates; native
interactions, authored lessons and practical coverage still require separate
implementation and acceptance.
