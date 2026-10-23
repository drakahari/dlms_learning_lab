# IPv4 ACL classifier foundation

Specified 2026-10-23 before implementation. This is a pure bounded rule evaluator,
not a router, firewall, Cisco CLI, new lesson or coverage claim. It is separate from
the existing IPv4 forwarding engine. No dependency, progress schema, authored
scenario, native workspace or host execution is added.

## Model

A version-1 policy contains zero to eight named lists, each with one to 64 ordered
rules. Names and per-list rule IDs are unique, case-sensitive ASCII identifiers
of one to 32 bytes (initial alphanumeric, then alphanumeric, dash, underscore or
dot). Standard lists accept only source-address rules. Extended lists also match
destination and IP protocol: all IP, ICMP, TCP or UDP. TCP/UDP rules support any
port, one exact port or an inclusive ordered range, independently for source and
destination. Port values are unsigned 16-bit numbers including 0 and 65535.

An address condition has an IPv4 base and wildcard. A zero wildcard bit must
match the base; a one bit is ignored. All 32 wildcard bits are independent,
including noncontiguous masks. Bits set in the wildcard do not require zero base
bits. Host is wildcard zero; any is wildcard all ones. This is address-bit
matching, not prefix arithmetic, address allocation or an ownership assertion.

A typed packet describes source, destination and one of ICMP, TCP with two ports,
UDP with two ports, or another numeric IP protocol. Other excludes 1/6/17, which
must use their structured variants; malformed variants are rejected. ICMP here
has no modeled type/code and no ports; TCP has no flags or connection state.
Every IPv4 bit pattern is accepted as classifier input. This does not mean it is
a usable unicast endpoint or a packet that the existing router could deliver.

Rules are evaluated in their stored order, never sorted by ID or specificity.
The first complete match returns its permit/deny action, list name, stable rule
ID and zero-based row index. If none matches a **nonempty** list, a typed implicit
deny is returned. Standard matching ignores destination, protocol and ports.
All-IP matching includes the structured protocols and every Other protocol.
Evaluation is read-only and deterministic; there are no hidden counters.

An empty policy is representable, but requesting a missing list fails. Empty
lists fail validation as unsupported. They are deliberately not modeled as an
implicit deny: the inspected IOS XE guide documents permissive behavior for an
applied empty list. Undefined list attachment behavior is also outside this
classifier. Neither diagnostic may be treated as a successful denial outcome.

## Isolation, validation and bounds

Public JSON entry points reject documents over 512 KiB for a policy and 512 bytes
for a packet before parsing. Unique bounded list maps and rule vectors reject
extra entries; bounded identifier visitors check strings before retaining them.
Unknown fields/variants, duplicate map/struct keys, wrong schema versions,
duplicate rule IDs, mixed list/rule classes, reversed port ranges and malformed
packet protocol shapes fail. Deserialized or manually constructed values are
validated again by evaluation. Direct serde callers must impose their own outer
byte limit; the provided JSON entry points are the content-loading boundary.
There is no partially applied mutation on error. A valid evaluation visits at
most eight lists for validation and 64 rules for the chosen list, with at most
512 fixed-size rule records in the whole policy. Decision strings are bounded
by the same 32-byte identifiers.

This is a metadata classifier intended for later **transit-only** use. There are
no host files, sockets, subprocesses, live packets, timers or I/O APIs. It does
not implement routing/attachments, interface direction, local/control-plane
traffic, fragmentation, IPv6, NAT, stateful sessions, reflexive/time-based ACLs,
ICMP types, TCP flags/established, named services, remarks/logging/counters,
object groups, IOS sequence-number editing or device-specific hardware rules.
Unsupported fields fail instead of being silently ignored.

Later routing integration must bind the intended endpoint and actual interface
direction, preserve fixed topology, require a live otherwise-deliverable path
for negative tests and retain positive service tests. Shutdown, removed routes,
spoofed endpoint ownership or an unsupported diagnostic cannot count as policy
success. Separate request/reply packet cases are necessary; a permit is not a
stateful-session guarantee. No objective status changes until integration,
native teaching and substantive assignments receive their own review.

## Sources and verification plan

Checked 2026-10-23 against authoritative Cisco documentation:

- [IOS XE 17.x: Creating an IP Access List and Applying It to an Interface](https://www.cisco.com/c/en/us/td/docs/routers/ios/config/17-x/sec-vpn/b-security-vpn/m_sec-create-ip-apply-0.html),
  chapter dated January 11, 2021: standard versus extended fields, ordered first
  match, implicit deny for populated lists, empty applied list caveat, and
  different local-origin versus transit handling. Hardware restrictions in this
  guide are not generalized to all devices or copied into this abstract model.
- [IOS 15SY: IP Access List Overview](https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/sec_data_acl/configuration/15-sy/sec-data-acl-15-sy-book/sec-access-list-ov.html):
  independent wildcard-bit meaning and noncontiguous patterns.
- [IOS 15M&T: Creating an IP Access List to Filter IP Options, TCP Flags, Noncontiguous Ports](https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/sec_data_acl/configuration/15-mt/sec-data-acl-15-mt-book/sec-create-ip-al-filter.html):
  independent source/destination TCP/UDP operators and inclusive range.
- Current CCNA v1.1 objective 5.6 remains Open in the existing checked-in coverage
  ledger. This foundation alone does not satisfy configure-and-verify practice.

Tests must cover first-match/reordering/shadowing, explicit versus implicit deny,
source-only independence, all protocols/port endpoints, invalid protocol shapes,
noncanonical wildcard bases, each bit and an independent bit-by-bit matching
oracle, equivalent supported rules, wrong/partial policies, maximum shapes,
oversized/duplicate/malformed JSON and pure repeated evaluation. These are model
tests against documented equations, not physical IOS or network acceptance.

## Implementation checkpoint

The pure module and eleven targeted tests are implemented in an isolated
same-repository worktree. Source/independent review is still pending before
integration. Tests compare 65,536 low-byte wildcard cases and 4,096 full-width
cases against an independent per-bit equation, check each of 32 ignored/fixed
bits and all 65,536 destination-port values, and exercise maximum eight-list /
512-rule compact and pretty JSON. Tests distinguish equivalent rule encodings,
overbroad/deny-all/partial matches, missing and empty lists, malformed structured
protocols, duplicate keys and direct-construction validation. No live router,
packet generator, host network command or native workspace acceptance is claimed.

An initial negative test exposed that serde internally tagged *unit* variants
can ignore extra fields despite the enum setting. Empty protocol/port variants
are therefore explicit empty structs, with regression tests rejecting ignored
TCP ports on ICMP/all-IP/any-port shapes. The corrected targeted suite and scoped
Clippy pass; the original failed test is not counted as acceptance evidence.
