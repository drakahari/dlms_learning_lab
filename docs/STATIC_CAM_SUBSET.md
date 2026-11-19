# Static forwarding-table foundation

Specified2026-11-08 before implementation. This is a capability prerequisite for
recorded CCNA5.7 port-security work, not an authored lab or new objective credit.
The current ready candidate is aging346T234326Z/R51.

A configured destination binding and a learned source entry are different. Add a
bounded typed overlay to the existing Ethernet-learning model rather than seeding
fake dynamic rows. The ordinary State, its serialized schema and send behavior
remain unchanged. A separate send_with_static entry point takes the supplied
configuration on each call; the future source-bound workspace will own/persist it.
No UI, existing lesson, host interface, socket, packet, process or clock changes.

The overlay contains at most64 complete unicast bindings, keyed by switch/VLAN/MAC,
with one target port per key. Validate names, uniqueness and the actual supplied
fabric, active VLAN and port membership before any mutation. A down port remains
a configured destination, but cannot deliver traffic. Reject malformed, multicast,
unknown, incompatible and oversized data honestly. No multicast/drop entries,
voice VLAN, learning restrictions, port-security enforcement, sticky behavior,
security aging, saving/rebooting or complete IOS command surface in this primitive.

Configured destinations take precedence over learned locations; source learning
must not move or refresh an overlaid address. Record such source observations
separately, without labeling them dynamic learning or inventing security drops.
Use the existing actual VLAN propagation, forwarding/filter/drop/NIC results.
Known destinations on the ingress port filter; known down destinations drop rather
than falling back to flooding. Broadcast behavior is unchanged. Any shadowed
dynamic rows are removed only in the successful transactional trial. Invalid input
or a resource failure leaves the original dynamic state unchanged.

Static entries have no learned timestamp and are not changed by advancing the
explicit virtual clock or clearing dynamic entries. Empty overlay behavior and all
old dynamic traces/state must remain exact. Bound static data to32KiB and existing
dynamic data to256entries; at most320effective entries. Propagation and traces keep
the existing bounded forest limits. Unsupported multicast/destination inputs stay
unsupported. The overlay itself is not an ingress security allowlist or an
identity/authentication mechanism.

Before any lesson: verify empty-overlay equivalence, first-packet known unicast,
static precedence, no dynamic movement/refresh, aging/clear independence, VLAN
separation, ingress filtering, down-port/cable behavior, malformed topology/data,
serialization/bounds and atomic failures. Retain the original MAC-learning tests
and authored-source values. No equipment/runtime conformance claim from tests.

Primary references checked2026-11-08, Cisco IOS XE Dublin17.12/Catalyst9300 guide:

- https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/port_security.html
- https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sys_mgmt/b_1712_sys_mgmt_9300_cg/administering_the_device.html
- https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf

Static secure entries are stored in the address table; this does not mean this
foundation already implements secure-port enforcement. The security chapter has
contradictory shutdown trap/log statements: leave that behavior outside any future
subset unless resolved. No upstream code copied or dependency added. Future
port-security work must also project error-disabled state into actual ingress and
egress forwarding; DHCP snooping/DAI remain separate gaps.

## Implemented prerequisite evidence

The new typed overlay and explicit metadata-source entry point preserve the old
MAC schema/default send traces.47focused tests and all-target Clippy passed before
secure-port composition. The latter supplies a private fixed-forwarding projection
while retaining configured/dynamic-secure origins in its own model; it never uses
Table::effective_rows to label learned secure rows as configured. No source-security
credit is awarded for this primitive alone. See PORT_SECURITY_LESSONS.md.
