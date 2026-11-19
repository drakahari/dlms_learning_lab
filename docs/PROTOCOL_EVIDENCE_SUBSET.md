## Packaged438 R1 verified —2026-11-12

The separate438candidate passed packaged83native actions/21captures, private
437→438lifecycle and actual NFS cold reopen/repair. Nine actual packaged images
inspected at100Navy/200Light; source gates and150Navy captures also pass. See
[checkpoint](CHECKPOINT.md) for exact build and acceptance limits. No full ICMP,
Linux firewall, physical narrow-window, independent novice or other-OS claim.
The source-stage notes below describe their earlier evidence boundary.

# Bounded ICMP Echo and protocol evidence

Source capability,2026-11-12. Latest verified package437predates it. New source
lab `lpic1-protocol-round-trips` is registered in `linux-protocol-evidence`, after
mixed-IP and service names, with the actual new three-screen/configuration flow verified in development R4
(83actions/21PNG). Separate438package acceptance remains pending. Catalog source438/171:
CCNA102, LPIC-1 144, LPIC-2 112, Bash80. No extra track or numerical-variant lab.

## Model and source boundary

`transport_core::echo` represents IPv4 unicast Echo request type8/code0 and reply
type0/code0. It reverses IP endpoints and reflects identifier,sequence and exact
binary payload. Identifier/sequence may be zero. An outward drop produces no
reply; a return drop does not count as a round trip. Empty payloads are valid.
There are no invented TCP/UDP ports, timers, latency or real ping output.

At most6supplied hosts,8queries,1024payload bytes/query,64byteASCII identifiers and
64KiB standalone configuration are supported. Decode caps raw bytes before Serde;
new row/payload/text deserializers enforce their own bounds. Each query takes at
most two transitions. Classifier errors/oversized diagnostics leave that transition
unchanged. Callbacks are trusted bounded in-process code, never serialized learner
functions. Reports are recomputed, Serialize-only and not progress authority.

`transport_workspace::echo` uses the existing IPv4 route/interface and stateless
ACL inspection in BOTH directions. It does not create another routing engine or
pass data to host processes, files, sockets, services or network targets. ICMP
policy matching is protocol1only: no ICMP type/code filtering is claimed. The
source-owned hosts act as Echo responders. No ICMP errors, ICMPv6, IPv6 discovery,
multicast, wire checksums, fragmentation, TTL, retry or rate-limit implementation.

The existing Task has an optional bounded `echoes` field, default empty and omitted
when empty. Its typed `echo_receive` requirement needs an actual matching round
trip. Protocol-comparison tasks retain at least one positive TCP/UDP receipt too.
Existing negative ACL requirements still need successful useful traffic crossing
the same protected boundary: deny-all and a missing listener cannot earn the
restriction credit. Fixed topology/listeners/queries are not learner-editable.
Existing progress Scope/Session structure, schema1and shell evaluator19 do not
change; old transport definitions reserialize without any added fields.

## Learning interaction

The new lesson introduces TCP/UDP/ICMP, matching replies, directions and only the
policy grammar needed for its supplied document. It uses a virtual IOS-style router
configuration to expose protocol evidence; it does not teach that grammar as Linux
commands or claim Linux firewall administration. Source references remain optional.
An explicit worked network/port differs from the assignment. Practice permits an
Echo-first partial repair, then TCP repair, deliberate breakage and retry. Fill
worked settings changes the example draft but does not run or grade it. The full
independent task keeps two TCP conversations, two Echo queries and one forbidden
UDP case visible through requirements, topology, supplied facts and actual traces.

The native addition opts in only when queries exist. Old transport screens keep
their controls/appearance. Echo feedback labels request/reply/type/code, directions,
identifier/sequence and actual reflected bytes without fake ports. Output preview
is bounded to32bytes and labeled; grading compares all bytes. Completion is based
on the existing current-state grade and actual unit-navigation controls, not a
capture-only flag. The first native run exposed that the copied automation checked
an unused completion flag; strengthened regressions verify actual completion/back
controls and their removal after a bad edit. That failure remains retained.

## Evidence and references

Focused core9, transport workspace14(eight retained/six new), extended existing
native widget tests3and authored content7pass. The latter includes all437prior
scenario values/170unit objects, distinct practice, Echo-only partial state,
alternate rules, hardcoded example values, broad allow/drop, malformed/protected
input, success invalidation and local disk resume/reset. Full Python1199, all-targetClippy (one job after observed two-job pressure),
strict coverage and inventory pass. Package acceptance is recorded separately in
CHECKPOINT.md. No independent
novice, physical-device, screen-reader or other-platform acceptance is implied.

Primary references checked2026-11-12; original implementation/explanations, no
upstream code/diagrams/text imported:

- [LPIC-1 objectives5.0,109.1](https://www.lpi.org/our-certifications/exam-101-102-objectives/).
- [RFC792 Echo request/reply](https://www.rfc-editor.org/rfc/rfc792.html).
- [RFC1122 section3.2.2.6](https://www.rfc-editor.org/rfc/rfc1122.html).
- [RFC9293 normal TCP opening](https://www.rfc-editor.org/rfc/rfc9293.html).
- [RFC768 UDP](https://www.rfc-editor.org/rfc/rfc768.html).
- [Cisco IOS XE17 policy grammar](https://www.cisco.com/c/en/us/td/docs/routers/ios/config/17-x/sec-vpn/b-security-vpn/m_sec-create-ip-apply-0.html).

This earns only the taught protocol slices after native admission. Broader ICMP,
full service/port coverage, IPv6 automatic behavior and the remaining curriculum
are not complete. Never substitute the older host Test address helper for ICMP.

Development R4actual images: `.build-artifacts/native-runs/run-s8oza7sf/screenshots/workspaces`.
Inspected final Echo-only partial feedback and separately filtered return feedback;
all five results now precede optional traces. Earlier R3three teaching views,
expanded Echo fields, real completion/back link and200Light editor were individually
inspected in run-65nfczj5. No independent novice acceptance. R1unused-metric and
R2offscreen-editor automation failures are retained; regressions exercise actual
unit controls and actual offscreen capture phases at1920×1080physical/150%.
No deadline, visibility, grade or input guard was weakened.
