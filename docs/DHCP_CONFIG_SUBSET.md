# Bounded DHCP configuration subset

Roadmap: LPIC-2v4.5/210.1, checked2026-11-02. This capability precedes dependent
server lessons. Current source244/67 registers two configuration-editor lessons,
with candidate gates pending; the verified separate242 candidate retains its own
source identity and does not contain this later foundation. Parser capability
alone is never counted as practical coverage. See DHCP_SERVER_LESSONS.md.

`dhcp_config::parse` is pure in-process parsing. It reads no files, starts no
process, resolves no hostnames, uses no network or clock, and executes no hooks.
The16KiB ASCII input,2048 tokens,64byte words,eight subnets and four DNS addresses
per option bound work/storage. Scope count reuses the existing DHCP core limit.
NUL/control text is refused, including in comments. Structured diagnostics carry
source positions and distinguish Syntax error, Unsupported configuration,
Invalid configuration and Simulator limit. Unknown directives never succeed by
being ignored. Seventeen parser/compiler groups and25 retained DHCP core groups pass; scoped
Clippy and104 focused Python checks also pass.

## Declared syntax

Whitespace, line comments beginning with #, case-insensitive keywords,
semicolon parameters and subnet NETWORK netmask MASK { ... } declarations.
Global parameters precede subnet declarations; local parameters precede range.
One range FIRST [LAST] per subnet uses inclusive usable endpoints and at most256
addresses. A range is optional: a subnet declaration alone does not invent a
dynamic pool. Canonical network addresses and contiguous /1..30 masks are used;
/31,/32 are excluded conventional special cases, /0 is outside this model.

Numeric `option routers` (one address), `option domain-name-servers` (1..4),
`option subnet-mask`, `default-lease-time` and `max-lease-time` are parsed at global
and subnet scope. Local values override only their inherited option. The parsed
advertised mask remains separate from the subnet selector. A wrong router/DNS
or different advertised mask must not be relabeled as a made-up syntax error.
Default-duration calculation assumes no client-requested lifetime and clamps the
declared/default duration to the declared/default maximum. It is not negotiation
or ISC cache/timer fidelity. Raw timing values are retained as bounded u32 data;
policy compilation honors actual core duration8..86400s limits.

Duplicate same-scope parameters, duplicate/overlapping subnets, multiple ranges
within one subnet,
router lists, hostname/IPv6/alternate addresses, quoted values, includes,
executable events, host/BOOTP declarations, classes/groups/shared networks,
DDNS/failover and wider grammar refuse explicitly. Some are valid real-dhcpd
features; refusal means outside this subset, not invalid Linux configuration.

## Workspace boundary

The compiler and bounded workspace pass model/source/progress tests. The native
editor passes focused widget/capture tests and an actual development flow.
Separate packaged verification remains pending.
No real lease-file grammar or server process/status is implemented. Conversion
never silently discards a mask/list or equates fixed-address declarations to
dynamic lease evidence. Existing dhcpv4 core/CCNA lessons stay unchanged.
The two new assignments teach range capacity and scoped options/held leases.
Development practical checks pass; separate packaged verification remains pending.
Broader per-client/static/BOOTP/relay/DHCPv6/real administration remains Open.

## Primary references and available evidence

Read2026-11-02:

- [Current LPIC-2 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/), version4.5,201-450/202-450,210.1.
- [ISC4.4 configuration manual](https://kb.isc.org/docs/isc-dhcp-44-manual-pages-dhcpdconf), updated2023-06-06.
- [ISC4.4 option manual](https://kb.isc.org/docs/isc-dhcp-44-manual-pages-dhcp-options).

The vendor identifies ISC DHCP as EOL. This is legacy-format simulation instruction,
not a deployment recommendation. No vendor code/manual prose is redistributed.
A read-only executable lookup found no dhcpd here; no package was installed and
no daemon/reference command ran. Real-daemon syntax/conformance comparison is
unverified. The manual, focused model tests and eventual native checks are separate
evidence; none implies full210.1, real equipment, novice or platform acceptance.

## Verified policy compiler foundation

compile() parses the complete text, resolves scoped defaults, normalizes pool
identifiers, retains rangeless declarations as no dynamic pool and validates a
Policy against supplied topology/relay facts. It does not inspect OS interfaces
or start a service. Different advertised masks and effective durations outside
core8..86400s explicitly refuse instead of silently altering values. Wrong
numeric router/DNS values remain wrong model-client parameters. Failed compilation
returns no replacement Policy. Core binding behavior is unchanged; future lesson
actions must not pretend ISC-specific NAK/cache/lease-file semantics are modeled.
In particular, do not expose ineligible renewal as a daemon-conformance action;
add an honest unsupported-context guard or model the required behavior first.
No new210.1 practice is credited before full source/progress/UI/content gates.

Held-binding rediscovery/cache contexts are refused explicitly. Fresh acquisition
requires a client without a held binding; only the supplied eligible T1 renewal
context is supported for an existing client. An execution row prevents an
unsupported exchange from passing merely because a preservation goal stayed true.
No new behavior is added to the original DHCP core or CCNA lessons.
