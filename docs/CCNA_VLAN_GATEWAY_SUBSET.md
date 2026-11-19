# Verify default VLANs and switched gateways

These exercises follow tagged trunks and IPv4 return paths. Each has a separate
worked demonstration, guided practice and independent work order.

## Access defaults over a matched trunk

On the supplied access port, an unset VLAN override uses VLAN 1. Setting
VLAN 1 explicitly has the same frame behavior. The trunk has a different,
matched native VLAN for its untagged frames. Learners inspect effective
desk membership, move it to the intended workgroup and test two independent
Ethernet segments. A third configuration leaves the desk isolated from the
legacy pair while also unreachable from its workgroup, so grading both
positive and negative paths matters. The available VLANs and legacy settings
come from fixed inventory. No switchport mode negotiation or configuration
CLI is simulated.

## Connected IPv4 routes on two switched virtual interfaces

A switch virtual interface connects the switch's IPv4 routing function to
one VLAN. An enabled SVI needs an active local port carrying that VLAN to
be operational. The fixed trunk and known port state stand for ports already
forwarding. The far switch can reject a VLAN that is still available locally.

The model feeds one internal endpoint per SVI into the existing Ethernet frame
tracer, then uses the actual untagged broadcast components as the links for
the existing IPv4 engine. Each endpoint retains its supplied address. The
learner edits the SVI address and prefix, administrative enablement, global
IPv4 forwarding, far allowed entries and each host's default next hop. Both
studio sources must exchange traffic with the server, including a reply, while
local Ethernet broadcasts remain separate. The gateway need not use any
specific otherwise valid unused host number. Native fields preserve incomplete
address text and clear the previous grade on edit.

The scope is two supplied switches, one matched native trunk with no loops,
two existing VLANs, the local port rule and ordinary connected IPv4 forwarding.
The simulator has no IOS execution, STP, ARP exchange, VTP, dynamic routing,
SVI autostate overrides, voice tagging, QoS, routing protocols, ACLs,
router-on-a-stick or equipment interoperability. It does not expose any actual
network or host configuration.

## Primary sources checked November 17, 2026

[CCNA v1.1 §§2.1.b–c](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf) ·
[Catalyst 9300 IOS XE 17.1 Ethernet defaults](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-1/configuration_guide/int_hw/b_171_int_and_hw_9300_cg/configuring_interface_characteristics.html) ·
[Catalyst 9300 IOS XE 17.7 switched virtual interfaces](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-7/configuration_guide/int_hw/b_177_int_and_hw_9300_cg/configuring_interface_characteristics.html) ·
[Catalyst 9300 IOS XE 16.5.1a SVI line-state detail](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/16-5/configuration_guide/int_and_hw/b_165_int_and_hw_9300_cg/b_165_int_and_hw_9300_9500_cg_chapter_00.html)

Cisco lists the last date for v1.1 as February 2, 2027 and the first date
for v2.0 as February 3, 2027. The active objective rows above are v1.1.
[Cisco exam version schedule](https://www.cisco.com/web/go/ccna/) ·
[Announced v2.0 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301_CCNA_v2.0_Exam_Topics_PDF.pdf)
