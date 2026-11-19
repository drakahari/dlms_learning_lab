# Secure access-port subset (before lessons)

Specified 2026-11-08 after the static-CAM tests; this capability alone is not a lab
or objective credit. Ready review remains the separately preserved 346-lab
aging candidate. New lessons will follow existing VLAN/MAC investigation practice.

One supplied switch has at most eight managed, directly attached, static access
ports. Other switches can forward through the existing acyclic VLAN fabric, but
security on trunks, intermediate bridges, voice VLAN, EtherChannel, 802.1X,
sticky learning, secure aging, restart/save, automatic recovery and full IOS CLI
are outside this subset. Configuration uses native controls, not a pretend CLI.
No sockets, packet bytes, host interfaces, processes, clock or real equipment.

Each managed port has explicit admin state, security enablement, maximum 1–16,
protect/restrict/shutdown mode, configured identities, separately learned secure
identities, an error-disabled latch and a bounded violation counter. At most 64
secure entries across the supplied switch. Configured and dynamic secure entries
participate in actual forwarding; they never acquire fabricated ordinary-CAM ages.
Their internal nonaging forwarding projection is not their displayed provenance.
Ordinary unsecured learning uses the existing MAC engine unchanged.

A replay supplies a known physical workstation and a complete unicast source MAC
for one metadata frame. This permits testing an unexpected source on the same
attachment without changing authored workstation identities. It is not identity
authentication: a matching spoofed source is a disclosed limitation of MAC-only
security. The destination is unicast or broadcast within this virtual topology.
Validate every request before mutation; invalid/unsupported/resource errors are
atomic and must not masquerade as security drops or successful isolation.

Admission first checks actual source host/cable/port/VLAN state. A physically
unavailable source does not learn or increment violations. An address secured on
another managed port in the same VLAN violates even with free capacity. An
unrecognized source learns dynamically only when capacity remains; otherwise it
violates. Protect drops without a counter increment. Restrict drops and increments
its counter, with a typed notification fact (no SNMP/syslog transport). Shutdown
drops, increments the counter and latches error-disabled. Subsequent frames on a
latched port are operational drops, not new violations. Counter saturates at
1,000,000; no unbounded event history is retained by this model.

Error-disabled/admin-down state is projected into the actual fabric before both
ingress and egress forwarding. Admission is not delivery: use actual forwarding,
arrival and NIC acceptance results. Configured destination bindings stay present
when down and therefore drop rather than silently flood. No ingress-only filter.

Manual recovery requires admin Down then Up. Down clears dynamic secure addresses
and arms recovery by clearing the latch while keeping the interface down; Up
restores it. Configured entries and counters survive. Editing policy alone never
clears a latch. Removing security requires removing configured identities; it
clears dynamic secure entries but still requires manual recovery if latched.
Maximum cannot be below the retained configured plus learned count. Dynamic-to-
configured promotion on the same port is allowed; conflicting bindings and malformed
edits fail atomically. No arbitrary fabric, MAC identity or physical-cable edits.
An explicit virtual time advance ages ordinary CAM only; secure aging is disabled.

Persistence validates version, bounded collections, supplied topology and the
runtime projection; no saved success/proof is accepted by this model. A future
workspace must bind the saved world to its authored source and recompute grading
from actual trials, including unexpected-source-first order, continued authorized
traffic, protected ports and legitimate alternate configurations. No lessons may
be counted before separate examples, assignments, reachability and negative tests.

Primary sources checked 2026-11-08:

- Cisco CCNA 200-301 v1.1, 5.7 (port security is only one part; DHCP snooping and DAI remain separate gaps): https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf
- Catalyst 9300 IOS XE Dublin 17.12, port security, chapter updated 2023-07-28: https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/port_security.html

The same chapter contradicts itself about shutdown trap/syslog notifications.
This subset deliberately does not model those notifications. Restrict notification,
shutdown state/counter, manual recovery and default-disabled secure aging are
consistent in that source. Tests are source-based simulation checks, not actual
IOS/equipment conformance. No upstream implementation or dependency imported.
