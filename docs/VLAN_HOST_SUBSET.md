# Linux VLAN profiles and tagged paths: capability contract

Specified2026-11-08 before implementation or new curriculum credit. The agreed
205.2 roadmap still has bonding, VLAN and bridging gaps. This increment implements
source-enabled VLAN profiles. Two authored tasks are registered in development;
exact new-package/native acceptance is still pending (see VLAN_LESSONS.md). Reuse the native
NetworkManager editor/case workspace and the existing bounded switching and IPv4
path engines. Keep old Ethernet parsers, outcomes and absent-option progress exact.
No dependency, host daemon, network interface, packet, subprocess or system change.

## Deliberate first slice

Local keyfile configuration for provided VLAN software-interface names and literal
parent NIC names. Model connection.type=vlan, VLAN ID1..4094 and 802.1Q with the
supplied header-reordering flag1. A VLAN ID is not derived from an IP octet. ID0 is
a real priority-tag case excluded from this teaching subset;4095 is reserved.
Parent UUIDs, nested tags/802.1ad, priority maps, registration protocols, alternate
flags, implicit device discovery, bridging/bonding, IPv6 and DHCP-on-VLAN remain
unsupported unless separately implemented. Unknown settings fail explicitly.
The old static/DHCP Ethernet APIs continue to reject VLAN profiles without an
explicit source-held VLAN context. No scenario-ID inference or canned success.

Separate physical parent facts, configured VLAN metadata and IP route selection.
A wrong tag can block traffic while carrier stays up. Use the selected profile's
actual parent/tag, not an expected answer, to propagate a bounded broadcast through
provided trunk/access ports. Verify both outward delivery and accepted return tags.
An untagged native-VLAN return is not delivery to a VLAN-only software interface.
Do not silently change existing unsupported tagged-input-on-Access behavior or
pretend to model every vendor's forwarding policy. Candidate parents attach to
supported trunks; peers use supplied untagged access attachments in the first slice.

At most two physical parents/software interfaces, two switches, eight endpoints,
twelve cables and one inter-switch cable in a source-enabled case. Reuse the
existing512-work/32-hop propagation limits. Physical carrier/cable/port state,
active/allowed VLANs and per-hop native tag handling affect results. No real frame
bytes, MAC learning, ARP, ICMP RTT, offload/capture behavior, weak-host/ARP-flux,
STP, kernel scheduling or general hardware conformance is claimed. IP reachability
remains the disclosed supplied-path projection plus actual modeled L2 delivery on
the selected subinterface in both directions, not a real ping.

Existing current-versus-future profile selections and multiple deterministic cases
remain separate. Edited files cannot preempt a current snapshot. Preserve bounded
Run/Stop, content-bound grading, isolated practice and saved drafts. Teach parent,
software name, VLAN plan, tags and return direction before assessing them. Display
source port/trunk facts only in the opt-in workspace if needed. No solved assignment
profile by default, no new track, no changed objectives on existing IDs. Register
new lessons only after capability, alternate/wrong/partial, persistence and actual
native checks. A VLAN-only increment does not complete205.2 or the curriculum.

## Checked sources and acceptance boundary

Checked2026-11-08: official NetworkManager
[VLAN settings](https://networkmanager.dev/docs/api/latest/settings-vlan.html) and
[keyfile manual](https://networkmanager.dev/docs/api/latest/nm-settings-keyfile.html),
plus upstream iproute2 [ip-link manual](https://man7.org/linux/man-pages/man8/ip-link.8.html).
These are rolling documents, not a newly pinned daemon or host-runtime comparison.
The attempted Linuxv6.18 raw vlan_core.c browser fetch timed out; no claim that it
was inspected. Original modeling only; no copied upstream implementation/manual.
No privileged reference interfaces/namespaces or physical equipment operation.
Windows/macOS/Omarchy, physical-device/screen-reader and independent novice
acceptance remain pending. Latest verified review build is338T163747Z; new network
work is separate from that immutable artifact and earns no coverage merely here.
