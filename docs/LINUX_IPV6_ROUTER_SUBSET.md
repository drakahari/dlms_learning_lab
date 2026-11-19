# IPv6 router capability work

Status2026-11-14: capability plus two integrated LPIC-2 assignments. Current
source489labs/195units preserves all prior487scenario/194unit values. Actual R2
Fedora development160actions/36captures and two normal cold processes pass;
11native plus6cold originals inspected. A missing direction glyph found in R1 was
fixed with plain text and a regression. The latest packaged review remains487;
final489candidate verification is pending. No full212.1or exam-completion claim.

## Declared first filter subset (before new lesson authoring)

Numeric IPv6 filter/header predicates, with built-in INPUT/FORWARD/OUTPUT chains,
ACCEPT/DROP and ordered first-match rules. `ip6tables [-t filter]` supports -S,
-A, -I(optional positive rule number), -Dby positive rule number, -P and -F,
including the corresponding long operation names. Match options: -s/-d numeric
address or prefix0–128; -i/-o exact bounded device names; -p tcp/6,udp/17,
ipv6-icmp/icmpv6/58orall; exact numeric --sport/--dport afterTCP/UDP selection;
numeric --icmpv6-type type[/code] afterIPv6ICMPselection. A type without code
matches every code of that type. Prefixes normalize host bits, like an address
match rather than a request to configure an interface. Listing is a canonical
representation of this subset, without implicit extension-module formatting.

One2KiB/64literal-word command;96rules total;32KiBlisting. Parsing works on
literal arguments, including quotes, and never expands variables/substitutions,
executes a shell or touches real interfaces. Failed operations retain the table.
Serialized rule sequences are bounded; malformed, unknown or incompatible fields
are rejected on validation. Raw128-bit interval values are internal and are not
JSON numbers in learner progress.

The filter alone describes a header decision, not packet delivery, a service,
neighbor discovery, route selection, address configuration or kernel acceptance.
Typed path and direction-state extensions below are separate capabilities. No
NAT, RELATED/error association, extensions/fragments, counters,
named ICMP types, named ports, explicit match modules, arbitrary targets/chains,
negation or nftables support is claimed by this first parser. Unsupported syntax
must report its limitation; no canned command-to-success rules or host fallback.

## Bounded direction state and path model

The additive TCP/UDP tracker keeps up to 64 confirmed conversations, with full
IPv6 source/destination addresses, ports and transport. Original and reverse
keys must be unambiguous. A rejected first event creates no entry. NEW means no
reply direction has been observed; the first observed reverse tuple is
ESTABLISHED, even if a subsequent route or filter blocks it. A preview changes
nothing. This is direction observation, not TCP handshake/application success.
The saved table is limited to 64 KiB; capacity/identifier refusal is atomic and
has an explicit unsuccessful outcome. No timers or eviction are invented.

The state-aware filter adds `-m conntrack --ctstate NEW`, `ESTABLISHED` or their
comma-list union. This profile requires explicit `-p tcp` or `-p udp` for such
rules. It refuses ICMPv6 state matching, RELATED, INVALID, UNTRACKED and NAT state
names. ICMPv6 remains a numeric, stateless header exercise. Literal quoted option
values remain values, even if an interface name resembles a flag. Unsupported
operations retain the previous configuration. Comparisons recheck counterexample
headers against the actual running table so feedback names the original row.

Fixed path facts allow 1–8 interfaces and at most 32 static routes. Each interface
has an ordinary unicast address/prefix and an explicitly scoped link-local
address. Connected routes and canonical static prefixes use all 128 address bits;
longest prefix wins before metric. Equal-preference multipath is refused rather
than guessed. An ordinary next hop must be on its named link. A link-local next
hop takes its scope from that route's explicit interface, so the same link-local
number on another interface is a different neighbor assumption.

Ingress, route selection, outgoing link, forwarding and filter are distinct gates.
The incoming interface's own link-local destination can use INPUT. A link-local
endpoint cannot become a forwarded destination on another link by broadening a
filter. This first profile refuses routed link-local events and local-origin
link-local output requiring a socket scope; it does not approximate those hooks.
Allowed means this fixed path/filter permits a header, not successful neighbor
discovery, PMTU discovery, a live peer, a TCP connection or an application. Route
withdrawal, RA reception, per-interface sysctls and actual NDP are not implemented.
No actual kernel changes, packets, subprocesses, sockets or new dependency.

## Full-width comparison foundation

Nine inclusive interval coordinates retain both128-bit addresses plus supplied
interface codes, protocol, source/destination ports and ICMPv6type/code. Exact
first-match comparison retains4096region/1,000,000step limits; exhaustion is an
explicit refusal, never success. Tests cover0/u128::MAX, high bits, equivalent
split prefixes, disjoint subtraction, independent small enumeration and checked
counterexample witnesses. This is not a kernel oracle or new authored coverage.

## Evidence so far

Forty-seven focused numeric/filter/state/path checks pass. Registered/source,
workspace and UI checks additionally cover supported alternatives, hardcoded
clients, partial/incorrect solutions, current-success invalidation, independent
practice, reset and disk resume. The new INPUT-versus-FORWARD refusal regression
first fails, then passes with the actual-chain check; feedback names that chain.
No authored requirement changed. Exact before-source/ELF and screenshots remain
retained; later code is not attested against those artifacts.

Actual R2 development native160actions/36captures and two normal cold processes
pass. Writer156474exit0(58.752s), reader156854exit0(43.531s) restore independent
unfinished rules, queued commands, traffic reports and exact raw/provenance/theme/
scale before native repairs.11native and all6cold originals inspected, including
all lesson views, errors/success, wideNavy100 and actual1268x632Light150/Navy200.
Final protocol labels/state-context wording and strict-chain refinement follow
that source hold; final installed candidate must exercise the resulting code.
Source holds and immutable selected-input/ELF retention are in ignored
next-linux-ipv6/. These are technical observations, not a kernel comparison,
full compiler closure, independent seal or novice/device/accessibility acceptance.

## Authoring gates and remaining work

Validate the parser/matcher against its disclosed subset before exposing it in
new lessons. Configuration/address-scope repair and additional sysctl/file behavior still need
implementation before dependent assignments. Reuse existing IPv6Prefix/category and host models
where appropriate. Link-local addresses cannot cross routers to other links;
ULAfc00::/7and locally assignedfd00::/8have different purpose from link-local.
Documentation addresses stand in for a global role; they are not real reachable
targets. Keep link-local next-hop device scope explicit. Do not teach a blanket
ICMPv6block as a safe default or equate an allowed header with workingNDP/PMTU.
No real advertisements, kernel sysctls, packets, root privileges or new dependency.
All new lessons must retain the approved three-screen teaching standard, separate
practice, substantive independent work and behavior/state-based alternate grading.

Primary reading checked2026-11-14; original code is MIT, no upstream implementation
or appendix scripts copied or executed:

- [iptables/ip6tables1.8.13 manual](https://man7.org/linux/man-pages/man8/iptables.8.html): command/chain/rule semantics.
- [IPv6ICMPmatch manual](https://man7.org/linux/man-pages/man8/iptables-extensions.8.html): numeric type/code predicate.
- [RFC4291§2.5.6](https://www.rfc-editor.org/rfc/rfc4291.html#section-2.5.6): link-local scope.
- [RFC4193§3.1](https://www.rfc-editor.org/rfc/rfc4193.html#section-3.1): ULA format/local-assignment bit.
- [RFC4890](https://www.rfc-editor.org/rfc/rfc4890.html): informational2007ICMPv6filtering guidance, not a full current production policy.
- [Kernel IPsysctl documentation](https://docs.kernel.org/networking/ip-sysctl.html): IPv6forwarding changes host/router behavior; accept_ra2can override the forwarding-related advertisement default. No such behavior is implemented here yet.
