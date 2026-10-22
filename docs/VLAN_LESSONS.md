# VLAN groups, trunks and handover

Four distinct authored assignments in `vlan-data-foundations`, bundled offline in `scenarios/vlan-data-v1.json`. This extends the catalog to55 assignments/thirteen units; numerical values in the isolated examples do not count as extra labs. Unit prerequisites are advisory. The first lesson requires no subnetting, binary or terminal knowledge; later assignments remain directly accessible.

| Assignment | Skill and practical outcome | Prior lesson |
| --- | --- | --- |
| `ccna-vlan-access` | Diagnose a wrong access membership on one switch while maintaining two usable, separate groups | None |
| `ccna-vlan-trunk` | Establish a trunk and distinguish an active VLAN database from each port’s allowed list; three independent causes | Access membership |
| `ccna-vlan-native` | Trace tagged/untagged classification, align the requested native policy and distinguish a one-way path from service/isolation | Trunks |
| `ccna-vlan-handover` | Repair five interacting access/database/list/native/port-state faults in a three-group handover and verify the complete policy | Native classification |

## Teaching and interaction

Each Learn introduction teaches its prerequisites in plain language, gives a different office to try, and names a concrete experiment. It can be skipped or reopened. The example has its own saved fabric/draft and explicit Apply and test example action; optional Load example settings fills only that practice draft. Example results cannot earn assignment credit. Challenge keeps the full assignment and native editor without opening guidance or solutions.

A selected-switch table keeps connected equipment, up state, port mode and relevant VLAN fields together. Active VLANs and Allowed VLANs use comma-separated individual IDs; IOS ranges are deliberately unsupported. Desktop rows align; narrow/larger-text controls stack with ordinary scrolling. Check configuration is the single apply/grade action. Bounded malformed entries stay editable and cannot partly apply. Default retry feedback shows the first two failures; all objective results, broadcast delivery/path evidence, hints, worked explanations and source/limitation references are optional.

The handover checks all three positive workgroup pairs, every cross-group west/east separation pair, all six access memberships and both native settings. Local separation follows the explicit access/native constraints and is independently tested across all endpoint pairs. Group service remains required: deleting/stopping/uncabling endpoints or disabling uplinks cannot stand in for isolation. A one-way leak cannot pass a negative case. Equivalent allowed-list order, leading-zero input and extra unused groups are accepted when the stated policy remains correct; no exact-string grading is used.

## Scope and sources

Checked2026-10-22 against current CCNA200-301 v1.1; the announced v2.0 remains separately planned. This unit provides **partial** data-access2.1/2.1.a and trunk/802.1Q/native2.2/2.2.a/b/c practice. Voice/default-VLAN-specific progression, inter-VLAN routing, MAC learning, STP/PVST, real IOS command practice and equipment acceptance remain absent. No vendor endorsement, full exam preparation or hardware equivalence is claimed.

- Current issuer objectives: https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf
- Cisco Catalyst9300 IOS XE17.18 VLAN trunk guide, updated2025-07-31: https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-18/configuration_guide/vlan/b_1718_vlan_9300_cg/configuring_vlan_trunks.html
- Pinned Cisco Nexus5000 NX-OS5.0(3)N1(1) access/trunk guide (tagged-native ingress policy): https://www.cisco.com/c/en/us/td/docs/switches/datacenter/nexus5000/sw/layer2/503_n1_1/Cisco_n5k_layer2_config_gd_rel_503_N1_1_chapter6.html
- Cisco PVID/type inconsistency protection, doc24063, updated2025-08-18: https://www.cisco.com/c/en/us/support/docs/lan-switching/spanning-tree-protocol/24063-pvid-inconsistency-24063.html

The model traces broadcasts only. **Actual Catalyst PVST/PVID protection may block native mismatches even on a physical tree.** Its absence here is disclosed before the native exercise and in references. The model’s untagged classification cannot be presented as a prediction of all real equipment. See SWITCHING_SUBSET.md for precise supported behavior, bounds and exclusions; ARCHITECTURE.md for declarative authoring and progress integration.

## Verification

`tests/switching_content.rs` exercises references, equivalent editor repairs, plausible wrong and partial repairs, each individually broken policy, the exact suggested example changes and separate draft/credit/reset/resume behavior. Every omitted handover repair stays incomplete. Model tests cover every VLAN ID1–4094, a maximum physical forest and9216 native/allowed/database transition cases; these are analytical model comparisons, not hardware or packet-capture acceptance. No learner input reaches real processes, files or networks.

Native Fedora synthetic flows and screenshots are recorded in IMPLEMENTATION_LEDGER.md. They verify renderer/input behavior, not independent novice usability, screen-reader acceptance or Windows/macOS desktop runtime. Existing saved scenario IDs/versions are unchanged, and no new answers/mastery are invented for old attempts.
