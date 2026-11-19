# Linux IPv4 host model — disclosed practical subset

2026-11-02: `src/linux_host.rs` is a pure exported library foundation.
`linux-host-tests-final.log` passes12 focused groups; `linux-host-clippy-r2.log`
passes library Clippy. Four native-workspace lessons are now registered in source229/61;
model/content gates pass, with actual native acceptance still pending. There is no claim of full iproute2, NSS, kernel or exam coverage.

The model uses supplied link/carrier, gateway-forwarding, destination-availability
and resolver-record facts. Nothing opens host files, sockets, services or commands.
It reuses mathematical IPv4 Prefix helpers, **not Cisco administrative distance**.
Route metric is u32; longer prefixes win before lower metric. Routes are confined
to one simplified main table, one primary IPv4 address per supplied interface,
up to2 interfaces,8 gateway routes,4 supplied gateways,8 nodes and2 resolvers.

## Disclosed terminal forms

- `ip link show`; `ip link set dev NAME up|down`.
- `ip -4 address show [dev NAME]`; add/del `ADDRESS/PREFIX dev NAME`;
  `ip -4 address flush dev NAME`.
- `ip -4 route show`; add/replace/del `PREFIX via GATEWAY dev NAME [metric N]`.
- `getent ahostsv4 NAME` reads supplied case-insensitive `.lab.test` A records.

Only these canonical forms, literal ASCII tokens and one command per entry.
No shell operators, expansion, pipes, host sudo or arbitrary utility dispatch.
Successful configuration commands are silent; inspections derive from typed
current state. Output omits unmodeled MTU/MAC/lifetime/uid fields rather than
inventing them. No real packets or RTTs are reported.

Unsupported forms explain the limitation. In particular `ip address replace`
must not pretend to remove another address: real Linux supports multiple addresses.
This activity instead requires deleting the modeled primary before adding another.
Gateway routes must be removed before deleting/flushing their interface’s address;
this deliberately avoids approximating broader kernel address/route cleanup.
Adding routes requires an up configured interface and an on-link next hop.
Overlapping connected prefixes, ambiguous equal route keys, connected-route
collisions, policy routing, multipath, namespaces, IPv6, DHCP, NetworkManager,
`ip route get` and Linux boot persistence are not implemented.

Interface addresses use conventional private/documentation unicast /1–/30.
This is a disclosed activity restriction, not a universal Linux address rule.
Route math correctly permits /0 defaults, /31 route ranges and /32 host routes;
it does not apply the subnetting exercise’s broadcast/host exclusion to routes.
Native path probes, when integrated, must be labeled simulator operations. They
use static configured selection and supplied facts, not complete kernel lookup,
ARP, sysctls, timers or measured physical reachability. A lower-metric unavailable
gateway does not magically fail over to another route.

## Virtual resolver

A separate editor accepts up to2 distinct private/documentation IPv4 nameserver
lines and # comments, bounded512 bytes. It never edits `/etc/resolv.conf` on the
host. Search/options/NSS/local-default/IPv6/cache/transport behavior is unsupported.
Configured servers are considered in order: a supplied unavailable/unreachable
server permits trying the next; a reachable supplied negative A answer stops,
rather than inventing a result from another server. Names are case-insensitive.
Unknown real domains/servers are never queried.

## Boundaries and gates

Raw decode64KiB, compact state32KiB, bounded serde sequences/text, checked u64
generation changes; all refusals preserve prior state. Profiles are bound in
whole on decode. Tests include route/metric maxima and longest-prefix override,
exact deletion/replacement, address ordering, wrong masks/LANs/conflicts, link/
carrier/gateway/destination failure distinctions, unsupported syntax/escape forms,
resolver ordering and negative answers, invalid config, codec/profile corruption
and generation limits. No actual host-network reference run was performed.
A failed test initially supplied a public address outside the declared interface
subset; its wrong-LAN fixture was corrected to private10.20.0.10 while retaining
the same diagnosis assertion. No existing tests were weakened.

Primary command/manual references read2026-11-02:

- [ip-address(8)](https://man7.org/linux/man-pages/man8/ip-address.8.html), upstream
  iproute2 address syntax and multiple-address semantics.
- [ip-route(8)](https://man7.org/linux/man-pages/man8/ip-route.8.html), upstream
  route syntax, prefixes, metrics and broader excluded features.
- [resolv.conf(5)](https://man7.org/linux/man-pages/man5/resolv.conf.5.html), Linux
  resolver ordering and the broader options/defaults deliberately not modeled.

The source-bound workspace now binds all edits and virtual resolver text to
the held scenario, preserves separate practice/drafts, and grades multiple supplied
targets and valid alternate repairs. See HOST_NETWORK_LESSONS.md for actual
evidence and still-pending native checks. Documentation alone is not coverage.
