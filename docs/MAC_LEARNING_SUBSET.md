# Ethernet source learning: simulator capability

Checked2026-10-24. This is a tested library foundation, **not yet a playable unit
or claimed certification coverage**. Native workspace, teaching and authored
assignments must be accepted separately before catalog registration.

The pure Rust module extends the existing bounded VLAN forest with dynamic
source-to-ingress entries per switch and local VLAN. A frame learns its source,
then uses a known individual destination or floods an unknown/broadcast address
within the modeled VLAN. Same-ingress individual forwarding is filtered.
Explicit simulated time ages entries; only a source frame refreshes its entry.
Selective clear and port/VLAN configuration changes invalidate relevant entries.
A supplied older table can be relearned; no live rewiring or MAC spoofing is
claimed. MAC learning is not a privacy/security boundary.

The observation distinguishes copies physically arriving at a running endpoint
from individual/broadcast frames accepted by the modeled untagged endpoint.
Tagged copies still arrive but are rejected by that endpoint model. This is not
hardware NIC, ARP, IP, application or packet-delivery conformance.

Requirements grade resulting tables/configuration and independent cloned
frame/time probes, never command text or old events. Every probe starts from the
same current state. Edits, clear and expiry can invalidate an earlier pass.
Negative probes cannot claim VLAN separation by disabling their source/known
destination attachment or physical route. Authors must still specify all required
healthy peer flows; one unrelated positive probe cannot define the whole policy.
Unknown/broadcast destinations do not identify one target attachment.

## Bounds and storage contract

Existing fabric limits:8 switches,16 ports each,32 fixed workstations,64 VLANs per
set and64 cables, with acyclic point-to-point attachments. Dynamic tables have
at most256 unique switch/VLAN/MAC entries; traces at most512 events. Matching and
propagation are iterative and finite. One action advances at most one simulated
day with checked clock arithmetic. Aging0 disables expiry; the0–3600second range
is a simulator policy, not the complete IOS command range.

The compact serialized state ceiling is256KiB. The separate raw JSON ceiling is
1MiB so legal pretty-printed maximum state can resume. Existing progress/pack
limits still apply around it. Collection readers reject excess/duplicate values;
fields and state are validated before acceptance. Retained UTF8 diagnostics and
grade feedback use fresh allocations bounded to512bytes, including the case of a
short message backed by an oversized prior allocation. These are bounded data
contracts, not a claim about exact allocator overhead or process RSS.

Unsupported/resource faults are atomic; ordinary modeled drops retain source
learning that occurred before the drop. Equipment/cable/host identities stay
fixed when port settings change. Configuring a changed port conservatively
flushes its entries. Seeded old entries may point at a down or previously used
port; that is authored historical evidence, not normal live-carrier automation.

No sockets, host network commands, real clocks, processes, packet bytes, STP,
redundant paths, arbitrary multicast, secure/sticky/static entries, hardware
capacity/timing, full IOS command grammar or vendor endorsement is included.

## Validation and references

Seventeen integration checks plus one private diagnostic-capacity check pass,
with all-target Clippy denying warnings and formatting checks. Coverage includes
learning/known/unknown/broadcast, comparison with existing VLAN broadcast results,
VLAN/trunk boundaries, expiry/source refresh, clear, valid alternative relearning,
partial/current invalidation, atomic unsupported inputs, exact resume, oversized
readers, physical separation shortcuts, tagged arrival and a maximum-shape pretty
JSON roundtrip. These tests do not establish native or learner acceptance.

Primary sources inspected2026-10-23 and rechecked2026-10-24:

- [Current CCNA v1.1 objectives,1.13](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf).
- [Cisco IOS XE MAC management](https://www.cisco.com/c/en/us/td/docs/switches/lan/c9000/lyr2-fwd/cdp-lldp-mac-udld/cdp-lldp-mac-udld-configuration-guide/c-configure-mac.html), updated2026-09-28.
- [Cisco unicast flooding background](https://www.cisco.com/c/en/us/support/docs/switches/catalyst-6000-series-switches/23563-143.html).

Original project implementation; no vendor implementation code or manual
paragraphs are copied. Topic scope is partial and future blueprint versions stay
separate. The current catalog and objective evidence statuses do not change.
