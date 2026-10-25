# DHCP client, relay and lease-handover lessons

Original authorship, checked2026-10-25. Three assignments are registered in the
normal catalog, bringing source content to125 labs/31 units. All13 content tests
pass, including the unchanged original122-specification/unit prefix. Native
interaction tests pass separately; actual rendered and packaged acceptance must
be recorded before expanding the coverage claim. No progress schema is changed.

The bundled pack is `scenarios/dhcpv4-client-v1.json`. The other
owned paths are `tests/dhcpv4_content.rs`, the review-action fixture,
`dhcpv4-original122.json` and this document. The122 source specifications and
unit map are preserved as a typed-default-aware historical baseline; they are
not fabricated learner attempts or records. The separate genuine writer8 fixture
retains its actual6/7/8 records unchanged.

## Progression and distinct assignments

Advisory unit: `dhcpv4-client-foundations`, after the existing
`ipv4-foundations` unit. The first scenario depends on the actual
`ccna-gateway` lab, “Reconnect the office client”; the next two depend on the
preceding new scenario. Existing unit order/content stays intact. Coverage updates require reviewed
model/native acceptance.

| Assignment | Investigation and result | Explicit alternatives |
| --- | --- | --- |
| Replace held options without disturbing the monitor | Separate applied policy, offers, an acknowledged old grant and a renewed/current grant. Correct a direct client's router/DNS parameters while retaining a named monitor. Prove two-client policy in both request orders. | Renew manually while eligible, including before T1; the reference waits until T1; Release/reacquire or expiry/reacquire are also valid. |
| Repair a relay and the selected client subnet | Diagnose delivery to the logical server and a broad answering pool whose mask differs from the physical /24. Correct both clients, retain an exclusion, and preserve the healthy direct segment. | Repair in either order; reverse client exchanges; allow extra eligible capacity and longer valid leases. |
| Hand over a small lease pool safely | Retire a client, exclude its address, reserve a kiosk address, and create sufficient eligible addresses for two other current clients and independent new requests. | Release or expiry for retirement; reverse client allocation; larger eligible range, additional valid reservations and longer grants. |

These are complete configuration/investigation assignments. They do not grade
command text or event history. Assigned addresses are unconstrained except the
expressly requested kiosk reservation. There is no hidden exact whole-policy
answer: router/DNS parameters, minimum positive remaining time, the named
exclusion/reservation, explicit healthy-pool/relay/grant preservation and new-client
results are the actual predicates. Additional valid per-client reservations are accepted, even if the dynamic range has no eligible addresses; the reference teaches planning a dynamic range around the requested reservation and exclusion. The final handover does not require a
server-side release receipt or prove notification occurred. It requires no
client-held lease for the retired client; its address stays excluded from new
allocations. The named office monitor must finish healthy with its original grant
and client-held state; uninterrupted uptime is not claimed.

## Teaching before the full task

Each lesson introduces DHCP, policy versus client-held state and the relevant
concepts before testing. Introduction is skippable. The separate gallery example
has another server, different physical subnets and disjoint client/segment IDs.
Its complete task and individually annotated actions precede the assignment.
Insert changes only policy editing data; Apply and every exchange/time action are
explicit controls. Typed JSON solution strings are pack-only reachability helpers,
never learner command instructions or IOS/Linux aliases.

Three different experiments have independently authored boolean row tables:

- Direct options: change applied DNS after success. Current held parameters stay
  correct while all new-client tests fail; restoring policy repairs those tests.
- Relay: Release one configured client. Its current row fails while policy and
  new-client tests remain correct; Discover/Request repairs it.
- Handover: additionally exclude the kiosk reservation. Current held grants stay
  correct, but the reserved client fails in both new-client test orders. Removing
  only the extra exclusion repairs them.

No new instruction panels are requested. The approved native workspace supplies
Intro, separate practice and the full assignment, with optional hints and
Challenge. The content tests check that filling a policy does not execute it,
then execute actual typed actions to exercise example/experiment/repair isolation.

## Sources and exact limits

The implemented contract and source receipts in `DHCPV4_SUBSET.md` and
`DHCPV4_WORKSPACE.md` are the capability boundary. Primary references checked by
the foundation/root reviewers on2026-10-25:

- [RFC2131](https://www.rfc-editor.org/rfc/rfc2131.html), §§3.1,4.3.1–4.3.4,
  4.4.5–4.4.6: offers, selected requests, held grants, renewal/rebinding, release
  and expiry. Time progression does not send requests.
- [RFC2132](https://www.rfc-editor.org/rfc/rfc2132.html), §§3.3,3.5,3.8,9.2,
  9.7,9.11–9.12: subnet/router/DNS and timing options, represented as typed values.
- [RFC1542](https://www.rfc-editor.org/rfc/rfc1542.html), §§3.4,4.1.1–4.1.2:
  relay-address selection; relay interface and client router option differ.
- [CCNA v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),
  4.3/4.6: prospective bounded DHCP role/client/relay practice only. DNS service,
  actual IOS configuration and full objective coverage are not established.

The model has one logical server, supplied transport facts, conventional fixed
physical segments, explicit manual time and no packets, host services or
resolvers. There are no automatic retries, renewal, routing/ARP validation,
multiple-server failover, authentication, DHCP snooping, reboot state or full
RFC/client implementation. An ACK proves delivered data, not routed or DNS
connectivity. The30-second offer hold and exclusion/reservation collision policy
are disclosed local simulator choices. Existing LPIC-2 210.1 remains unrelated
uncredited work; these native CCNA tasks teach no real Linux DHCP configuration
syntax. Forthcoming CCNA blueprints receive no mapped credit.


Manual renewal is allowed before T1 when the held address remains eligible. T1
is the usual scheduled renewal time, not a mandatory gate for a manual action.
When current policy prevents extending the held address, a server may acknowledge
the existing grant without changing its older options or expiry. The corrected
procedural test excludes the initial .10 address while retaining .11/.12 for both
new-client requests; it distinguishes that result from successful early renewal.
This follows RFC2131§4.3.2 and§4.4.5, re-opened by the root reviewer on2026-10-25.

## Acceptance and remaining gates

Thirteen tests cover real prerequisite/pack reachability, reference plus two
alternates per task and the capstone’s additional-reservation strategy, read-only/partial/offer-only/wrong-option negatives,
Apply-only staleness, atomic held-client Discover rejection, eligible early renewal and declined-extension acknowledgements, relay-only/wrong-mask behavior, each current predicate's
own failing row, independently failing new-client rows with every current row
true, exact remaining-time/expiry boundaries, separate examples/fill/experiments,
pending malformed drafts and shared explicit backup/reset/resume, original122
specifications/units and actual6/7/8 records, plus malformed/foreign-context
rejection with exact saved-byte preservation. Expected outcomes are authored from
contract predicates and explicit tables; no reference run generated them.

The retained first R4 guarded content log `m106-dhcp-content-r1.log` reports11
passing and2 failing tests. The failures were an invalid Discover-while-held
setup and an incorrect assumption that eligible early Renew keeps old parameters.
The core and Goals were unchanged. Corrected content passes all13 checks, both
before and after normal registration (`m106-dhcp-content-r3` and
`m106-registered-dhcp-content-r1`). The current19 native-egui interaction checks
also pass; actual rendered, cold-reopen and packaged acceptance are separate
gates. No host reference command or real DHCP service was used. Model and
interaction passes do not prove novice comprehension, physical-device keyboard
or screen-reader acceptance, or Windows/macOS runtime behavior.
