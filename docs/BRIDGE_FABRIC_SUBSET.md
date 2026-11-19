# Linux bridge forwarding foundation — specified2026-11-08

This is a capability gate for the existing LPIC-2 advanced-interface roadmap,
not an authored lesson or complete objective coverage. The341/119review candidate
stays immutable. No bridge lab, editor, NetworkManager parser or saved-progress
field is claimed yet.

A bridge forwards Layer2 traffic between attached ports; an active-backup bond
selects one physical member. The first bounded model joins two supplied untagged
segments using an ordinary virtual bridge and validates both directions. It
preserves existing external switch behavior, source physical facts and peer state.
The local bridge endpoint is distinct from its physical port endpoints. A stopped
bridge cannot forward through its ports; external same-segment traffic can still
work. A detached or physically down NIC cannot carry bridge traffic. No artificial
fallback from a VLAN/IP failure to another port is invented.

## Supported contract before implementation

- Version1closed source facts: one named bridge, two distinct named physical
  attachments, at most two external switches/eight hosts/twelve cables,64KiB.
  All supplied switch ports are Access/untagged. Source NIC endpoints are leaves;
  every other supplied host is a peer. Known identities only; no host I/O.
- Explicit bounded current snapshot: bridge up/down plus attached member names.
  Membership and physical availability are separate. Both ports can forward at
  the same time. This is a settled acyclic forwarding projection, without timers.
- Project an ordinary additional virtual switch/local endpoint into the existing
  bounded broadcast engine. Keep real external ports, cables, VLAN classification
  and up/down facts. Never mutate the supplied fabric. Untagged broadcast delivery
  and its reverse are reported separately; this is not ping or MAC-table evidence.
- Reject any topology that would form a loop after joining the physical ports,
  even if a supplied cable is down. Reject tagged frames and trunks, rather than
  stripping tags or pretending to model unfiltered tagged bridging. Reserve
  internal identifiers and validate before indexing. Unknown keys/duplicates,
  excessive lists/text, foreign member names and invalid endpoints fail clearly.

No IP routing/forwarding, NetworkManager activation/keyfile editing, FDB learning,
STP election/convergence, VLAN filtering, hairpin/multicast policy, real packet,
kernel/daemon/device execution or hardware conformance. An acyclic projection is
not evidence that general loops are safe. Future NM defaults must follow NM's
manual, rather than the kernel's different STP default. Existing source/progress,
VLAN and bond semantics stay unchanged; new lessons wait for composition checks.

## Authoritative references

Checked2026-11-08; original modeling, no upstream implementation copied:

- [Linux Ethernet bridging](https://docs.kernel.org/networking/bridge.html):
  Layer2 forwarding, separate STP/port state and VLAN filtering behavior.
- [NetworkManager bridge settings](https://networkmanager.dev/docs/api/latest/settings-bridge.html):
  STP enabled by default and VLAN filtering disabled by default at the NM layer.
- [NetworkManager bridge-port settings](https://networkmanager.dev/docs/api/latest/settings-bridge-port.html):
  separate hairpin, cost and priority controls; none is bond-primary selection.
- [LPI201/202objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/):
  v4.5/201-450/202-450stillcurrent;205.2interface practice is a narrow slice,
  without full authentication/routing/status/traffic-analysis coverage.

## First actual foundation gates

R01library/R02app builds, R03all50forwarding checks (10new/40existing), and
R04scoped library/testClippy pass. New checks cover both attached ports/local
endpoint, detach/down, external same-segment independence, each physical down
source, external VLAN mismatch, stopped peers, actual hop paths, unsupported tags,
foreign/duplicate members, potential cycles even with down links, closed/bounded
JSON, malformed source and reserved identities. No lesson/curriculum credit yet.

The new fixed `nm_bridge_keyfile_reference.py` accepts no CLI input and uses the
same MAIN lock/held-binary/finite stream-time/project-disk guard as the separate
bond tool. R05five guard/normalization tests and R06all3actual fixtures pass on
nmcli1.56.1-2.fc44, SHA256
5fd7577a716dadfd50b34bcd492ff14fd4ea08f09cdab63fd3be5cba666039f9. Output retained
in `.build-artifacts/references/nm-bridge-offline-sk54l9m3`. Controller output has
manual IPv4, an empty bridge group/default STP and empty Ethernet/proxy sections;
bound ports omit IP groups and have an empty bridge-port section. Unbound ports
retain disabled IP methods. These are serialization observations only, with no
NetworkManager activation, host profile/device change or forwarding comparison.

## Next parser contract, before implementation

Add an explicit bridge-controller API to the bounded existing keyfile parser;
ordinary Ethernet and VLAN APIs remain strict and return unchanged typed data.
Return both typed IP settings and bridge properties, rather than discarding the
controller role. Only type=bridge, one manual IPv4 address with existing bounded
route semantics, disabled IPv6/default addr-gen-mode, empty Ethernet/proxy groups,
STP true/false (omitted NM default true), and absent/false VLAN filtering. Require
explicit autoconnect-ports=0or1; inherited-1/global policy is not guessed. Defaults
and repeated-key behavior follow the defined subset. Unknown keys, nesting,
bridge-port options, DHCP/IPv6, VLAN filtering and unsupported timers fail clearly.
No activation/member parser, policy integration, IP grading or lesson follows
merely from accepting a controller file. Composition/current snapshot and old
regression checks remain required before authoring.
