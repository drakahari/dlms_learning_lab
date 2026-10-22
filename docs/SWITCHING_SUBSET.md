# VLAN data-plane capability contract

Specified2026-10-22 before implementation/dependent teaching. This capability will be a pure typed in-process model, with no learner host/network/process operations or new dependency. It does not yet add a playable lesson or objective evidence. The current51 authored assignments remain unchanged.

## Initial model

A bounded physical forest contains switches, named ports, plain untagged workstations and point-to-point cables. Each port and cable has explicit up/down state. Each switch has an active VLAN database; a port is access (one VLAN) or an explicitly configured802.1Q trunk (native VLAN plus an allowed-data-VLAN set). Valid IDs1–4094; missing configured VLANs are representable operational faults, not malformed JSON. Duplicate IDs, unknown cable endpoints, reused physical jacks, self-links, parallel/redundant switch cycles and out-of-bound structures must be rejected.

At ingress, an access port assigns an untagged frame to its access VLAN. A trunk assigns an untagged frame to its native VLAN, or uses a single existing tag. The switch must have the VLAN active; trunk data must be allowed. This educational fabric explicitly accepts tagged ingress for every allowed VLAN, including a tag equal to the native VLAN. That policy follows the pinned Nexus reference below; platform behavior differs, so this is not a Catalyst emulator. Forward a broadcast to other operational ports in that local VLAN. Access egress is untagged; trunk egress is untagged for its native VLAN and tagged for other allowed VLANs. The far switch independently classifies the arriving frame. A plain workstation receives only untagged frames. No global VLAN-ID equality shortcut may bypass per-link tagging/native classification.

The probe is **broadcast visibility**, not ping or a real command: compute recipients and deterministic tagged/untagged path/drop evidence from typed state. Check both directions where a task promises mutual visibility; isolation means neither direction delivers, not merely failure of the mutual check. One-way leakage must not pass an isolation outcome. A known operational drop differs from unsupported behavior; unsupported tagged ingress on access ports must fail honestly and cannot earn an isolation outcome. This port behavior varies by platform/voice-VLAN features, so it is excluded rather than claimed as universal Cisco behavior. Learner-facing editors/trace controls will be native; any content/test adapters must be clearly internal, not invented IOS commands.

Initial bounds:8 switches,16 ports per switch,32 workstations,64 VLANs per switch/allowed set,64 cables,64-byte ASCII identifiers and80-byte readable labels. One cable per physical jack and at most one per workstation. A physical forest avoids pretending STP is implemented. Frame propagation and trace/output must have explicit work/memory limits independent of numeric VLAN values, validate before probing, terminate on malformed inputs and return typed diagnostics. No sockets, host NICs, clocks or external commands are involved.

## Integration gates

Isolation grading must separately require a live physical path (both workstations, cables and port ends operational), preserve the authored physical attachments, and include positive service cases for every intended group. Disabling, unplugging or deleting an endpoint cannot earn policy credit. A live path alone does not assert VLAN delivery. Active host VLANs and task-specific configuration retention must be checked explicitly. One-direction failure cannot stand for isolation.

All pack/import entry points must bound the whole document **before** deserialization (existing pack2MB/profile16MB limits); bounded collection decoders do not by themselves cap a malicious string allocation. Invalid drafts must remain editable without changing applied state or retaining current success.

## Important boundaries

This is the explicit **data plane without control protocols**. No STP/PVST election/blocking, native-mismatch BPDU protection, CDP/LLDP/DTP/VTP, EtherChannel, MAC learning/aging, ARP/IP/ICMP/routing, VLAN interfaces, voice/private VLANs, stacked/priority tags, tag-native mode, security attacks or physical performance/timing is initially promised. VLAN1 is ordinary data here; real management/control-frame exceptions are not modeled. The physical forest and explicit configuration do not replace the missing STP objectives.

A native-VLAN mismatch can translate untagged data between different local VLAN IDs in this restricted data plane. **Do not teach that real Catalyst always forwards such traffic:** PVST/PVID inconsistency handling can block affected ports, including on a physical tree. Any native-mismatch example must explain the omitted control behavior and cannot claim equipment troubleshooting acceptance. Unsupported states/behaviors stay failures or explicit gaps, never complete issuer coverage. Native previews, curriculum and progress integration follow only after model tests/independent review.

## Primary references checked2026-10-22

- Cisco *VLAN Configuration Guide, IOS XE17.18.x, Catalyst9300*, “Configuring VLAN Trunks,” updated2025-07-31: https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-18/configuration_guide/vlan/b_1718_vlan_9300_cg/configuring_vlan_trunks.html . Native untagged classification, allowed VLAN data and explicit trunk configuration inform the model; CLI text is not copied into an invented learner terminal.
- Cisco *Troubleshoot Spanning Tree PVID- and Type-Inconsistencies*, document24063, updated2025-08-18: https://www.cisco.com/c/en/us/support/docs/lan-switching/spanning-tree-protocol/24063-pvid-inconsistency-24063.html . Establishes why data-plane native translation alone is not a claim about default Catalyst behavior.
- Current CCNA200-301 v1.1 objectives and upcoming v2.0 remain separate in CURRICULUM.md/OBJECTIVE_MAPPING.md. This contract does not promote either blueprint's coverage.

No issuer endorsement, hardware/packet-capture comparison, real network experience or full802.1Q conformance is claimed. Model tests must include VLAN database absence, allowed-list asymmetry, native tagging, alternate valid isolation, both directions, down ports/cables, malformed topology, bounded propagation and success invalidation before dependent content.

Additional pinned ingress reference: Cisco Nexus5000 NX-OS Layer2 Switching Guide5.0(3)N1(1), Configuring Access and Trunk Interfaces: https://www.cisco.com/c/en/us/td/docs/switches/datacenter/nexus5000/sw/layer2/503_n1_1/Cisco_n5k_layer2_config_gd_rel_503_N1_1_chapter6.html . The non-tag-native profile accepts tagged ingress within its allowed set. Tag-native/global/platform-specific native-ingress policies are not selectable in this first educational fabric and are not inferred from its behavior.
