# DHCP snooping: scoped implementation contract

Status: bounded core, saved native workspace and two authored lessons are implemented
in development351/124. Final package admission is pending; see
[lesson scope and evidence](DHCP_SNOOPING_LESSONS.md). The stage notes below record
what was true at each earlier foundation gate, not current catalog exclusions.
This follows the existing CCNA5.7 gap and DHCP/Ethernet prerequisites. It is a
composition of the current bounded Ethernet engine, not a host DHCP service or
an alternative execution engine. No new dependency, packet socket or host action.

## Supplied setup and supported metadata

One supplied access-only switch, at most16 ports, with directly attached virtual
endpoints. VLANs and physical connections come from the existing validated Fabric.
Learners change global enablement, selected VLANs, per-port DHCP trust and source-
MAC verification. These are different from port security and ARP-inspection trust.
Ordinary VLAN/port/cable/NIC restrictions still determine delivery.

The source explicitly supplies Option82 insertion disabled; this is not a claim
about device defaults. Relay and Option82 transformation are excluded. Typed
metadata represents initial Discover/Offer/Request/ACK and matching Release;
it includes transaction ID, client hardware address, Ethernet source/destination,
assigned address and lease duration where relevant. No packet-byte parser or
automatic real-time protocol scheduler is implied. Other messages/flows receive
an honest unsupported diagnostic unless a documented ingress-rejection rule can
be modeled without inventing the rest of that protocol.

## Admission and actual forwarding

With snooping inactive globally or on the ingress VLAN, supported ordinary frames
use the unchanged Ethernet path and create no observed lease records. When active,
server messages on untrusted ingress are rejected. Enabled client source checking
compares Ethernet source with client hardware address. Nonzero relay metadata or
received Option82 on untrusted ingress is rejected; transformations on trusted
ingress are unsupported. Client messages from untrusted ingress can leave only
through trusted ports, using the tested pre-forward output selection. A trusted
interface is a configured allowance, not cryptographic server authentication.

Known initial-exchange replies target the observed client attachment. Unknown or
ambiguous transactions are outside this first conversation subset and receive an
unsupported diagnostic, not an invented assertion that equipment drops them.
Never invent a client arrival, lease or completed exchange because a configuration
matches a reference string. Report actual recipients separately from admission.

## Observed leases and limits

Record an observed client transaction only after supported admission/forwarding.
Offer alone does not create a lease. A trusted ACK following an observed Request
and a valid physical client delivery creates or updates the VLAN/client record:
IP, MAC, original untrusted client port, simulated expiry and dynamic origin.
Clients observed behind trusted ingress do not create untrusted-client bindings.
Duplicate/ambiguous client attachments are unsupported in this initial subset.
Matching Release and explicit virtual-time expiry remove records; a release from
the wrong recorded client port is rejected. No address allocation is fabricated:
the existing logical DHCP server/client model remains the authority for its own
pools/exchanges, while this observer consumes explicit supplied message metadata.

Enforced limits:64 live transactions,64 bindings, one simulated day per time
advance, leases1–86400seconds, bounded identifiers and input envelopes, atomic
resource/validation failure. Preserve baseline equipment and current learning in
versioned state. Validate source/session context before grading/restoring work.
No arbitrary command strings, paths, files, subprocesses or network activity.

## Acceptance gates before authoring

Test real output restrictions, global/VLAN gating, correct/incorrect trust,
MAC/chaddr mismatch, unsupported messages, physical-down paths, Offer versus ACK,
matching and wrong-port Release, expiry, bounded resources, malformed restore,
atomic failure and retained old forwarding. Dependent lessons need separate
Introduction, guided example, independent assignment, alternate and negative
behavioral grading, draft/progress tests and native verification. A tested pure
library alone is not implemented practical objective coverage.

Full relay/Option82, retransmission/reboot/renew/rebind state machines, server
authentication, rate/queue limits, stacks/trunks, database-agent persistence,
hardware scheduling, IOS CLI and ARP inspection remain outside this contract.
Do not imply basic ARP inspection implements IP Source Guard's interface rules;
the subsequent DAI contract must independently specify its lookup and trust.

## Primary references

Checked2026-11-08, Fedora local date. Paraphrased behavior; no Cisco endorsement
or runtime/equipment conformance claim.

- [Catalyst9300 IOS XE Dublin17.12 DHCP chapter](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/ip/b_1712_ip_9300_cg/configuring_dhcp.html),
  updated2023-07-28: enablement, trust, output restriction, metadata validation and
  binding fields. Normative version pin for the proposed subset.
- [Cisco support217055](https://www.cisco.com/c/en/us/support/docs/ip/dynamic-host-configuration-protocol-dhcp-dhcpv6/217055-operate-and-troubleshoot-dhcp-snooping.html),
  updated2024-06-12, explicitly IOS XE16.12.X/Catalyst9000: supplemental worked
  initial-exchange trace, binding on ACK, expiry/Release, Option82-disabled example.
  Do not relabel this trace as17.12 runtime testing.
- [Catalyst9300 IOS XE Dublin17.12 DAI chapter](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/configuring_dynamic_arp_inspection.html),
  updated2023-07-28: later feature's distinct ingress trust and binding dependency.

## Earlier core-stage evidence and deliberate diagnostics

SNOOP-R03passes15new+56retained checks; R06passes20new checks after edge additions,
76distinct current/retained checks across these two runs. R07lib/R08app/R09all-target
Clippy pass(all-target52.42s);R10all955Python pass12.726s. Fixed metadata fixtures
only, no host DHCP/packet tools or real equipment comparison. Source is
`src/dhcp_snooping.rs`; tests are `tests/dhcp_snooping.rs` plus a synthetic fixture.

The60second pending initial-exchange cache is a simulator bound, not a protocol
retransmission timer. Applying a materially changed policy clears that sample
routing cache and retains unexpired recorded leases; reordered equivalent settings
retain the exchange. This is explicit simulator behavior, not a device lifecycle
claim. Unsupported replies/competing attachments/conflicting leases/failed-client
ACKs fail atomically with a diagnostic, not an assertion about hardware dropping
those packets. Observed bindings are all dynamic DHCP observations; they do not
allocate a client address or prove that a client application accepted the payload.
No DHCP/DAI practical credit exists until complete taught/native/graded lessons.

## Earlier source/session stage

SNOOP-R35passes the explicit five-field lease-grade regression;R36passes20model,
10session,4synthetic source/persistence adapter and4retained port-security lesson
checks. Private export/restore retains old aging work and favorites. Separate
practice never earns assignment credit; malformed drafts resume and stale checked
flags cannot override actual policy outcomes. Checks use separate deterministic
traffic from the supplied setup, leaving displayed observations untouched. Replay
starts its own supplied conversation and preserves the policy draft. Policy apply
clears old replay text; unexpired recorded leases remain historical observations.

R37library/R38app/R41app/R42all-targetClippy pass;R43all955Python pass12.849s.
R20CURRENT-inventory length, R31test-module ordering andR39native-variant fixture
partition failures were retained and corrected without weaker assertions. No
historical collector/seal/COMPILED snapshot was changed. New engine/source/progress
adapters exist, but no new registered lesson/UI/candidate or objective credit.
