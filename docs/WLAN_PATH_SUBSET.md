# AP/controller VLAN paths — integrated bounded simulation

Normal catalog426/164 includes two authored CCNA assignments and a native topology,
AP/WLAN configuration and switch-port-table workspace. See [lessons and actual
verification](WLAN_PATH_LESSONS.md). The separately verified424R2candidate remains
immutable; the separate426R1candidate is now verified (see CHECKPOINT.md). Existing scenarios and schema1 are retained.

## Deliberately bounded behavior

One supplied AP and one controller attach to the existing switching Fabric. Native
management and tagged client data can share the same controller attachment; this
model does not teach that separate physical management/data cables are required.
The same bounded Fabric derives VLAN ingress, egress, native tagging, active/allowed
VLANs, link/device state and forward/return paths. It is not a canned mode-to-success
lookup, a packet generator, an IP router or an actual CAPWAP controller-join test.

| Configuration | Derived behavior |
| --- | --- |
| AP Local mode | Client data uses the controller attachment, including when a WLAN permits local switching |
| AP FlexConnect mode, central WLAN | Client data uses the controller attachment and its WLAN VLAN mapping |
| AP FlexConnect mode, local WLAN | Client data uses the AP attachment and its local WLAN VLAN mapping |
| Native-management mismatch/unavailable path | Client cases are not evaluated; broken transport cannot earn isolation credit |

Only an already-provisioned, connected, same-LAN transport exercise is supported.
Management availability means the supplied management VLAN carries frames both ways;
it does not prove IP discovery, DTLS, CAPWAP joining, authentication or radio service.
The task layer separately requires the intended forwarding location/VLAN and actual
forward/return paths. Isolation requires both directions absent while the physical
path stays operational; powering off or disconnecting a target is not separation.
The operational-path guard and mismatched-native-VLAN trace checks pass focused
regressions. A required VLAN path must retain that VLAN through all recorded hops;
accidental native-VLAN translation is not an accepted solution.

Existing Fabric bounds remain8switches/16ports per switch/32endpoints/64cables,
64VLANs per switch,512propagation work and32trace hops. New bounds are1–4WLAN
mappings and1–8test cases. VLANs use1–4094. Strict typed modes, bounded text/lists,
unknown-field rejection and explicit task-authorized edits apply. Existing source
models and parser limits are not enlarged.

## Tasks and drafts

A task declares initial topology/settings, editable properties, supplied cases and
a reachable reference. Every WLAN needs a positive service case, including protected
traffic; negative-only or initially solved assignments refuse. Grade current paths
and case requirements, not action strings or reference equality. Multiple mode/
forwarding combinations may be valid when the task permits them.

Version1sessions keep separate example/assignment fields and check snapshots.
Examples must use different settings and task identities. Editing answers clears
current success; switch-table navigation alone is not an answer edit. Malformed
bounded drafts remain editable and serializable. Filling a worked example is inert
until Check, never completes independent work, and reset affects only one draft.
Scenario/engine/catalog-aware progress and native UI bindings are implemented.
Actual disk tests use separate synthetic profiles, not the owner's original profile.

## Exclusions

No real packets, radio/RF, controller, command execution, host files, services or
network access. No routed control transport, discovery/certificates, cryptography,
authentication, controller-outage/standalone behavior, DHCP/Internet, roaming,
controller redundancy/LAG, monitor/sniffer/mesh/bridge/rogue AP modes or complete
vendor CLI/GUI. These exclusions leave objectives2.6/2.7incomplete. The two implemented lessons provide only narrow Partial practice for these
objectives; full objectives remain incomplete.

## Authoritative references

Opened2026-11-12UTC, with future exam versions kept separately in the coverage map:

- [CCNA200-301 v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),
  existing-roadmap2.6/2.7; no entire-objective credit from this capability.
- [Cisco Wireless Controller8.10: Managing APs](https://www.cisco.com/c/en/us/td/docs/wireless/controller/8-10/config-guide/b_cg810/managing_aps.html),
  Local versus FlexConnect client-data forwarding.
- [Cisco Wireless Controller8.5: FlexConnect](https://www.cisco.com/c/en/us/td/docs/wireless/controller/8-5/config-guide/b_cg85/flexconnect.html),
  per-WLAN forwarding, local VLAN/native attachment and conditional standalone behavior.
  Standalone authentication is deliberately excluded rather than generalized.

Original explanatory prose and code; no vendor implementation/text/diagram imported.
Development checks are not independent novice, physical-device, AT or other-platform
acceptance. No new dependencies or system changes.

## Actual core verification

Nineteen focused Rust model/task/draft tests pass under MAIN runner/jobs2/runtime2.
Two regressions first reproduced false positives for disconnected isolation and
native-VLAN leakage, then passed with state/trace guards. The same checks cover
local versus central paths, unavailable control, alternate allowed mode/forwarding
choices, malformed/partial/foreign/read-only fields, raw numeric draft roundtrips,
example isolation, reset, success invalidation and navigation-only selection.
These core checks alone do not prove teaching or native acceptance; the later
per-lesson/native observations are recorded separately in WLAN_PATH_LESSONS.md.

## Integrated source/engine/progress and UI

Twenty-three focused core/binding checks pass, including actual disk retention of
all424prior attempts/favorites and separate new drafts. Read-only source settings
are protected while malformed editable values remain resumable. Invalid/mismatched/
prechecked/unreachable packs refuse. Internal author-reference actions are bounded,
atomic and never exposed as a host execution route or taught as vendor commands.
Seven content/current-registration and eight UI/capture-phase checks pass. Actual
native R1 completed194actions/53screenshots; six teaching screens and representative
errors/enlarged controls were inspected. Final current-source guards/Python1156, all-target Clippy, strict coverage and
inventory pass. The426package content/lifecycle/native/NFS checks pass; see the live checkpoint.
