# Dynamic ARP inspection: bounded implementation contract

Designed before implementation2026-11-08. Core, separate draft/trial sessions and three registered development lessons are
implemented; native editing/geometry checks and actual Fedora lesson flows pass.
Bounded practical5.7credit and exact354Fedora package/native admission are recorded.
This does not establish full-objective or hardware conformance.
Compose the existing single-access-switch Ethernet/DHCP observer; preserve its
state/schema/APIs and all existing scenario/progress values. No dependency or host
networking is needed. Typed metadata is not a packet-byte parser or host IP stack.

## Documented rules

The pinned Catalyst9300 IOS XE Dublin17.12 chapter describes ingress-only,
per-VLAN inspection, independent trusted-port bypass, DHCP IP/MAC bindings and
ARP ACLs. An explicit ACL deny overrides a binding. Unmatched ACL rules fall back
to DHCP unless the attached static option makes them deny. Optional checks compare
Ethernet/ARP source MACs, reply destination/target MACs, and invalid IP values;
they are disabled by default. Source IP is checked for both kinds; target IP only
for replies. Zero, all-ones broadcast and multicast IP values are rejected by that
optional check. Basic trust does not authenticate a device. ARP rate handling is
separate; untrusted defaults are15pps, trusted unlimited.

Source checked2026-11-08: [Cisco chapter, updated2023-07-28](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/configuring_dynamic_arp_inspection.html).
[Cisco support222274, updated2024-08-05](https://www.cisco.com/c/en/us/support/docs/switches/lan-switch-software/222274-troubleshoot-dynamic-arp-inspection-dai.html)
separately demonstrates a mismatched sender pair being rejected and distinguishes
IP Source Guard's interface rules. Neither source establishes this simulator's
runtime conformance. Do not import an IOS XR/Nexus or IPSG rule into this pin.

## Deliberate model boundary

One validated access-only switch, at most16ports; direct supplied endpoints and
actual existing VLAN/cable/NIC/CAM forwarding. ARP Request/Reply metadata contains
Ethernet source/destination, sender IP/MAC and target IP/MAC. Ordinary unicast
source metadata and individual/broadcast Ethernet destinations are supported.
Physical unavailability is separate from policy rejection. A rejected message must
not learn a source or reach a virtual recipient. Accepted metadata uses the SAME
DHCP observer's active Ethernet state, not a parallel fabricated table.

DAI VLAN selection/trust and optional validations are separate typed configuration
from DHCP snooping. Filters have ordered exact IP/MAC permit/deny rules (at most16
per filter and16attached filters), no unsupported wildcard approximation. A permit
may replace a missing binding for a static sender. Optional field validations still
apply. Configuration, event and clock errors are atomic; no partial policy/state.

Binding checks use currently observed DHCP data. Initial fixtures obtain a real
Request/ACK in the simulator, never an invented history. A matching pair arriving
on a different port is an unresolved Catalyst pin edge: absent an applicable ARP
ACL or trusted bypass, diagnose it as unsupported without asserting pass/drop.
This is an explicit fidelity limit. ARP cannot create/remove a DHCP record; actual
DHCP Release/expiry continue to do that. No endpoint ARP-cache poisoning, ping,
routing or application-connectivity claim follows from virtual frame delivery.

Resource bounds: at most8ARP arrivals per virtual second; a further arrival returns
a simulator-limit diagnostic atomically. Advancing the explicit clock permits a
new bounded sample. This is a software guard BELOW the documented rate, not an
emulation of hardware rate drops/error-disable/recovery. JSON world1MiB, message
4KiB; bounded collections/identifiers. No wall timers, real sockets or commands.

Exclude trunks/stacks, port channels, packet-byte malformations, probes beyond
these disclosed fields, full rate/log/counter scheduling, IPSG, CLI, device saving
and platform/hardware conformance. Preserve these limits in reference and help;
never count this library alone as a completed lab or full CCNA5.7 objective.

## Required focused tests before lessons

Actual request/reply recipients, rejection before CAM/delivery, per-VLAN gating,
independent trust, live/expired/released bindings, static ACL permit, deny precedence,
static/fallback, first-match order, optional request/reply field checks, malformed
input/imports, atomic failure, burst limits and round-trip saved state. Retain DHCP
and Ethernet behavior and explicitly test the unresolved-port diagnostic.

## Foundation evidence

DAI-R04passes16new core plus31retained DHCP/access-output checks (47 total).
R05library/R06app/R07all-targetClippy pass(52.76s);R08all958Python pass12.778s.
R03's physical fixture tried to alter protected structural data through Configure;
the guard correctly refused. R04 constructs the supplied initial fixture instead,
retaining all endpoint/cable/port-down assertions. No protection was weakened.

R09library/R10app/R11pass9new draft/session/behavioral tests plus16core and20DHCP
checks. Three synthetic task types exercise source/configuration compatibility,
actual per-case Request/ACK setup and ARP delivery, wrong/partial/broken work,
legitimate alternative server ACL, canonical equivalent policy order, isolated
practice, preserved invalid drafts, selected reset, unsupported reference rejection,
read-only supplied DHCP policy and malformed/oversized restore. These fixtures
are NOT authored registered labs and add zero catalog/objective credit. The current
DAI core is not in immutable351T042029Z. Session-stage Clippy/Python remain pending.

## Explicit zero-sender boundary

The current baseline does not decide special ARP-probe handling on inspected,
untrusted ingress with the optional IP check off: zero sender IP returns an
atomic unsupported diagnostic. With that check enabled, the documented zero-IP
rejection is modeled. Trusted/VLAN bypass still means actual ordinary Ethernet
delivery, not a host ARP-cache claim. No authored assignment uses zero sender IP;
ordinary requests with an unknown target MAC are a separate supported case.
This clarification changes no authored grading objective. Do not infer a default
probe-permit/drop rule or borrow another product's allow-zeros option.

## Registered development stage

Three distinct lessons now load in354/125: sender-binding boundary, exact static
sender/fallback, and Ethernet/ARP field consistency. R30all37focused checks pass;
all351previous scenarios/124units are exact. R32all6actual-widget checks pass at
five scales, two widths, light/dark and disabled states, with partial results,
editing/retry, stale-success invalidation, isolated practice and Challenge/focus.
R35passes17core,10session and4authored checks after the explicit probe boundary.
These are available implementation checks, not native-screen/package or novice
acceptance. The last ready review remains351T042029Z and excludes DAI.


R49actual Fedora native158actions/62PNG exits0/reaped. Root opened13originals,
including every lesson's three views, partial failure, worked practice and enlarged
light-theme reflow. Inventory records lesson-specific review, not renderer-only
inheritance. R50inventory354 and R51all20presentation checks pass. Exact354package
and final gates remain pending. Save-read is not cold-reopening acceptance.
