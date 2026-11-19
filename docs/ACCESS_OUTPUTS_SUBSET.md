# Per-frame access-port outputs

This is an internal forwarding prerequisite for the recorded DHCP-snooping gap,
not a new lab or implemented exam objective. It extends the existing in-process
Ethernet engine; no packets, interfaces, sockets, host commands or dependencies.

An explicit bounded set selects the permitted outgoing ports of one supplied
switch for one frame. This first entry point accepts only a single switch with
at most16 access ports and directly attached endpoints. A port may be selected while down;
normal cable, administrative-state and VLAN rules still determine delivery. A
selection never makes an otherwise invalid path work. Unknown ports, multiple
switches, trunks or mismatched source attachment are diagnosed before mutation.

The policy is consulted before each Forward event and endpoint arrival. It does
not rewrite the fabric, fabricate destination bindings or hide arrivals after
forwarding. Source learning still uses the real ingress. Excluding every output
is valid and produces no delivery. Selection does not bypass a known-destination
lookup or turn known unicast into flooding. Ordinary send/static-table APIs and
serialized state remain unchanged; failed input/resource validation is atomic.

Implemented focused checks: broadcast/unknown/known unicast; excluded outputs have no
Forward or arrival; VLAN, physical-down and learned-destination behavior; empty
set; malformed/unsupported policy; exact old-path equivalence; bounded state and
roundtrip. This primitive does not implement DHCP admission, leases, bindings,
ARP inspection or any security guarantee. Dependent lessons require separately
tested composition and learner interactions before receiving coverage credit.

OUTPUT-R03/R04pass11new tests plus45retained MAC/static/port-security checks,
56distinct checks total. R05lib/R06app/R07all-targetClippy pass(all-target44.63s);
R08all953Python12.740s pass. Existing historical collectors/producers/seals remain
unchanged; CURRENT-only QA adds the new module and refreshes current parent pins.
No learner UI or lesson/grade changed, so prior349native evidence is retained for
the unchanged packaged application. The ready349candidate excludes this subsequent
foundation; its frozen MAIN readers must not be rerun against advanced source.
