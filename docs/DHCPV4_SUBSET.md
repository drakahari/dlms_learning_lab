# Pure DHCPv4 foundation

Source contract, checked2026-10-25. The M102 guarded pure-module gate passes
25 DHCP tests plus68 library regressions, all-target Clippy and formatting. No State/Goal, pack, progress, UI, lesson, coverage, real
packet or host-network behavior is added. Existing models remain unchanged.

## Source basis

[RFC2131](https://www.rfc-editor.org/rfc/rfc2131.html), §§3.1,4.3.1–4.3.4,
4.4.5–4.4.6 supports the offer/request/acknowledgement, lease, release and renewal
boundaries. Offers are not committed leases. A selected server unable to honor
its offer can send NAK. Renewal is unicast; rebinding is broadcast. A client
must stop using an expired lease. T1/T2 default to one-half/seven-eighths of the
lease duration. Reachable early renewal may acknowledge without extending it.
ACK parameters should not conflict with the selected offer.

[RFC2132](https://www.rfc-editor.org/rfc/rfc2132.html), §§3.3,3.5,3.8,9.2,9.7,
9.11–9.12 identifies subnet-mask, router, DNS, lease-time, server-identifier and
renewal/rebinding options. The model represents these as typed values, not TLVs.

[RFC1542](https://www.rfc-editor.org/rfc/rfc1542.html), §§3.4,4.1.1–4.1.2 defines
relay-address behavior: a first relay supplies its client-facing address, and
that address is not the client's default-router option.

The current [Cisco IOS-XE17.x DHCP server guide](https://www.cisco.com/c/en/us/td/docs/routers/ios/config/17-x/ip-addressing/b-ip-addressing/m_config-dhcp-server-xe.html)
confirms pool selection using the relay address or receiving interface subnet.
It also documents many capabilities deliberately absent here. No IOS CLI or
vendor exclusion/manual-binding equivalence is claimed. References were opened
from their primary publishers on2026-10-25; no host reference execution occurred.
The current [CCNA v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf)
4.3/4.6 were checked by the root reviewer; the current [LPIC-2 v4.5 objective
210.1](https://www.lpi.org/our-certifications/exam-201-202-objectives/#rdp-we-210.1_DHCP_configuration_(weight:_2))
was opened during this source work. These remain unmet curriculum work, not
status credit for this foundation.

## Fixed supplied context and mutable policy

One logical server has a fixed IPv4 identifier. Up to8 nonoverlapping physical
segments have canonical conventional prefixes1–30, optional supplied server and
relay interface addresses, fixed known occupied addresses, and explicit supplied
relay-to-server/client-unicast reachability. Up to32 fixed client identifiers each
belong to one segment. These are model input facts, not inferred routing or ARP.
Future assignment integration must pin this whole topology separately from policy.
Current offers and server/client bindings also retain a necessary relational pin:
their copied subnet must contain that client's supplied server-interface selector,
or its supplied relay-interface selector when indirect. This is independent of
current policy and allows bad broad masks, old options, renamed pools and relay
failures; it rejects moving a narrow-subnet grant to an unrelated segment. It is
not an action-history or authentication credential.

Mutable policy has0–8 nonoverlapping pools, relay settings and a server-enabled
flag. Each pool has a canonical subnet, inclusive range of at most256 ordinary
host addresses, at most256 exclusions, at most32 known-client reservations, an
optional router, at most4 DNS addresses and a lease length8–86400 seconds.
Ranges use full32-bit IPv4 arithmetic, including octet crossings. Reservations
may be outside the dynamic range but must be ordinary hosts of the pool subnet.
The deterministic policy chooses a client's eligible existing server binding,
then its reservation, then the lowest eligible dynamic address. Reservations
are unavailable to other clients. This foundation's exclusions block all new
allocations, including reserved addresses; an excluded reservation produces a
specific no-offer diagnosis rather than dynamic fallback. This is declared local
policy, not an RFC/vendor-wide rule.

Pool/collection ambiguity, unknown references, malformed bounds and unsupported
shapes reject atomically. A well-formed wrong router/DNS value or relay target
remains editable and observable. Options are delivered data; an ACK alone proves
neither working routing nor DNS service. A broad/wrong pool can deliver a subnet
which differs from the supplied physical segment; current lease inspection must
retain that actual configuration rather than replace it with an expected one.

## Typed actions and current state

`discover(client)` starts a new bounded exchange and supersedes its earlier
pending offer. A direct broadcast uses the supplied server interface. Otherwise
an enabled relay must target this server and use its supplied client-facing
interface address; incorrect settings produce a transport/configuration diagnosis,
not a pretend server NAK. Valid nonzero giaddr selects the pool by containment.
There are no secondary interface/subnet selection fallbacks or option82 rules.

The server holds a candidate address for30 virtual seconds as a disclosed local
policy. It does not configure the client. One pending offer per client includes
a monotonically bounded token and copied offered parameters. Offer.lease_seconds
is the actual relative offered duration. Binding.granted_seconds records the
original grant; remaining(now) exposes its current usable duration. Existing
server bindings can be offered with their original remaining lease (even one
second), without inventing new timers or extending that binding.
`request(client, token)` uses only a delivered current offer. Missing, expired or
stale offers and misdelivery produce silent typed diagnostics. Availability is
rechecked against current policy and ownership. Newly unavailable selected
parameters produce NAK and discard the offer; accepted parameters configure the
client and server binding together. An accepted still-valid offer retains its
actual offered options/duration even if policy options were edited meanwhile.

Server bindings and client-held leases are separate bounded maps. Acknowledged
current leases preserve their acquired/renew/rebind/expiry timestamps and options;
a renewed acknowledgement reports that it took current policy and created new
timestamps. A renewed grant need not have a later expiry if policy shortened its duration.
`renew` uses supplied unicast reachability and ignores relay settings.
`rebind`, allowed in the rebinding window, uses broadcast/relay delivery. Reachable
policy refusal acknowledges the existing lease without extending or applying new
options. Unreachable/disabled service returns no reply and retains the held lease.
No unsolicited NAK is generated for transport loss or time passage.

`release` stops client use immediately. If its unicast notification reaches the
server, the binding is freed; otherwise the server retains it until expiry. A
release has no ACK. `advance` only moves the explicit clock and expires offers,
server bindings and client leases. It never sends requests, retries or auto-renews.
Bound/renewal/rebinding are current time-window labels, not evidence that a packet
was automatically emitted. A successful acknowledgement is never cached credit.

## Resource and isolation boundary

The explicit clock is at most1,000,000,000 seconds; one advance is at most86400.
Every addition and the offer counter are checked before mutation. Minimum lease8
makes the integer defaults strictly ordered. No per-second loops exist. A pool
scan examines at most256 candidates against the bounded current tables. No
allocation scans an entire /16 or larger subnet.

All names are bounded48-byte simple ASCII identifiers. Duplicate map/set keys,
collection overflow and unknown JSON fields reject. Whole snapshot input is
capped at1MiB; typed visitors cap nested text and collections before retaining
extra values. Direct values validate counts/scalars before sets/clones. Every
operation validates context and leaves exact state on a returned validation Error;
a modeled no-reply, NAK or release is a normal typed outcome with its declared
state effects. Error text has a fresh UTF8-safe allocation capped512 bytes.
No engine/host command strings, sockets, OS clock, files, subprocesses or external
resolver calls exist in this module.

No DHCP wire format, UDP/IP forwarding, broadcast flags, retransmission/jitter,
multiple servers/failover, persistent server database, client reboot/cache,
DECLINE/ARP conflict detection, DHCPINFORM, dynamic DNS, classes, option82,
raw vendor/client-ID formats, DHCPv6, Cisco/Linux server syntax, authentication,
or full RFC conformance is claimed. Explicit typed unknown/invalid operations
are diagnosed; omitted behavior is not silently represented as success.

## Prospective verification

Twenty-five model tests are source-only and unrun. They cover independent offers
and bindings, exhaustion and offer expiry, reservations/exclusions/static addresses,
held options versus pending offers and current policy, ACK-current versus renewed
grants, direct/relayed scopes, wrong relay and independent unicast paths, lost
release notification, stale selection/NAK/retry, no automatic clock actions,
/16 ranges across octets, malformed shape/unknown references, exact rejected-action
state, clock/token ceilings, strict duplicate/count/text readers, maximum legal
compact/pretty snapshots, foreign current copies and supplied-context pins.
No result count is accepted until the explicit first guarded gate runs.
