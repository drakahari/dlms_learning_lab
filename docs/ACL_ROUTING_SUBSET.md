# IPv4 transit ACL/routing foundation

2026-10-23; reviewed classifier commit 3e07e40, base application 974381f.
Specified before code; implementation remains isolated pending independent review.
No State/Goal, new scenario, native workspace, progress field or coverage change.

## First implementation slice

Add a pure `acl_routing` wrapper around the existing IPv4 Topology and the reviewed
ACL classifier. Keep original `Topology`, its serde representation, probe and
verify_endpoint behavior unchanged. A separate wrapper owns per-device policies
and per-interface inbound/outbound optional attachments. Names are local to a
device, allowing two routers to use the same ACL name without sharing edits.
Bound the wrapper to existing12 devices/eight ports, and globally at most eight
ACLs/512 total rules. Reuse each Policy's eight-list/64-rule validation, enforce
whole-wrapper byte limit before parsing, reject duplicate/foreign device/port
keys and references, and compute sum bounds before derived clones/collections.

The wrapper also declares at most12 fixed host device/interface pairs in a
bounded unique map. These identities are independent of the mutable forwarding
flag: stopping a router cannot turn it into a permitted host source/destination.
Declared hosts must remain non-forwarding and cannot acquire any ACL attachment.
Each Flow must use exactly its declared pair. `same_identity` pins these authored
roles together with the original physical device/interface/link shape; policy
rules and attachment edits are not identity. Future content/import adapters must
call it against the authored initial data. This pure capability does not load
or restore application progress by itself.

No attachment (`None`) means no filter at that transit boundary. An explicitly
named missing ACL or present empty ACL is an unsupported configuration error,
never implicit allow/deny. Do not attempt to emulate platform-specific creation
and attachment of undefined/empty IOS lists. Invalid drafts may be editable in a
later workspace, but cannot be applied or earn success. Inbound/outbound are
separate named fields, at most one list per interface per direction.

This slice supports ordinary one-way IPv4 transit packets only. Both endpoints
are named distinct non-forwarding devices/interfaces with up, usable addresses;
the source interface must be the first routed egress, preserving the existing
model's source-selection policy. At least one actual intermediate forwarding
router must be traversed. Router-originated, router-terminated, direct same-link,
fragments, TCP session state and control-plane ACLs report Unsupported. Changing
a host into a router cannot make it a successful alternate endpoint. Packet
addresses derive from the bound current endpoint interfaces, never arbitrary
learner-supplied spoofed source/destination strings. Protocol/ports are explicit
authored test inputs; all IPv4/ACL subset restrictions continue to apply.

The destination interface selects the host's address and declared role; it does
not pin the physical receiving port. The existing topology's weak-host local
delivery remains: an up host can receive traffic for its bound address through
another interface. Source binding is stricter here: the declared source interface
must be the actual first egress. Neither rule changes the old probe API.

## Forwarding/event order

Do not filter only an already-computed completed path and call that real ingress
ordering. Factor the existing private one-way path loop narrowly to share its
route/neighbor/loop decisions with a typed transit-boundary callback; the old
path uses a no-filter callback. No generic plugin platform or second routing
implementation. Existing probe regression fixtures must remain byte/effect
equivalent. The new endpoint-bound directed path helper binds source as well as
destination, and does not synthesize a reply.

For the real ACL-aware attempt, record ordered typed events:

1. Source host chooses its ordinary egress; do not run endpoint output ACLs.
2. At an intermediate router, use the previous hop's exact receiving interface.
   Evaluate inbound ACL BEFORE this router's route selection. Deny stops here;
   no selected-route/egress event may appear afterward for that attempt.
3. Check forwarding and select the route using existing LPM/distance rules.
4. Evaluate outbound ACL on the selected egress AFTER route selection, before
   forwarding to the resolved next neighbor. A denial stops this attempt.
5. Resolve/validate neighbor and continue. Endpoint ownership/up/usable checks,
   duplicates, loops, hop bounds and unsupported equal-cost behavior remain.

Separate no-ACL baseline evaluation is diagnostic/grading evidence. It may find
a later path fault even when the actual attempt stops at an earlier ACL, but
its events are never merged into the real-attempt trace or presented as though
the denied packet reached those locations. Inbound versus outbound differences
are explicit and testable with a missing-route fixture.

Result is a typed Delivered, Denied, RoutingFailure or Unsupported/Invalid value.
Denied carries device, exact interface, direction, list, explicit matched-rule
ID/current row or implicit-deny cause, and bounded events before that denial.
A missing-route/interface-down/unknown-list diagnostic is never a Denied value.
No render-time mutation or counters; repeated inspection is deterministic.

`Network::from_json` rejects more than2,000,000 bytes before parsing;
`Flow::from_json` rejects more than1024 bytes. Maps cap devices at12, interfaces
at8, host declarations at12, and network-wide lists/rules at8/512. The existing
topology's12-device/96-interface/384-route and16-iteration forwarding bounds
remain. At most48 typed events and two bounded traces are retained per inspection.
Direct serde callers must impose the outer byte limit. Direct constructed values
are validated before derived policy sets, trace copies or decision retention.

## Later grading adapter (separate from pure integration)

Each positive or negative case binds source and destination device/interface and
an explicit protocol/port tuple. Keep authored address/prefix/shape constraints
separately, as existing routing goals do. Do not infer intended endpoints from
current address ownership. No new requirement mutates the saved topology.

Every negative case must first demonstrate endpoint-bound delivery on the SAME
current topology with only ACL filtering disabled. This is a guard, not a lab
repair. If interfaces/routes/forwarding/addresses are broken or ownership is
spoofed, the baseline failure means the negative case fails. A typed actual
Denial must then occur at the authored device/interface/direction. Expected cause
can require ExplicitDeny or ImplicitDeny. A named rule ID is required only where
the assignment explicitly asks to preserve that rule; otherwise equivalent
supported deny rules remain valid. Row position is evidence, not exact-answer
text. A denial at a different boundary cannot substitute for the requested fix.

Goal validation requires meaningful positive flows: every negative boundary must
also be crossed successfully by at least one required positive packet on that
same device/interface/direction. An unrelated bypassing positive flow cannot
justify a deny-all ACL at the required boundary. Fixtures must preserve required
endpoint configuration and, where the task demands it, route/attachment policy.
Introduce distinct negatives for source, destination and service cases; do not
allow deleting all lists, broad deny-all, broad permit-all or changing routes to
stand in for the requested access policy.

The small pure `guarded_denial` helper implements the baseline, expected-location /
cause and same-boundary positive-crossing checks for up to16 supplied positive
inspections. All supplied required positives must deliver. It is not a complete
authored Goal validator, does not assert case coverage or fixed addresses, and
does not run/regrade caller-supplied snapshots. A future Goal must build every
inspection from the same current state and pin its authored Flow inputs.

Request and reply are explicit independent packet cases, with their own port
and direction tuples. No automatic established-connection inference or silent
reverse-port guessing. A permitted request alone never claims a working session.
Cases may require both directions when that is the actual assignment. The
existing IPv4 probe's reply behavior stays unchanged for all old activities.

## Persistence/native boundaries (future reviewed slice)

Use a separate State/Goal/workspace instead of adding ACL fields to the existing
Topology or retroactively expanding old routing goals. Preserve old scenario
IDs/versions/credit and all saved attempts. Version new optional drafts, pin
physical device/interface/link shape and endpoint inputs on import, validate
foreign metadata, and suppress current completion for pending divergent drafts.
Examples retain separate wrapper/state/draft and never grade main work. Accepted
native route context plus an ordered ACL table is appropriate; no invented IOS
terminal and no new workspaces in the pure integration commit.

## Required tests before accepting even the pure wrapper

- Ingress deny precedes failed route lookup; outbound check follows selected
  route and precedes neighbor forwarding. Baseline remains separately failed.
- Endpoint output/input binding is explicitly unsupported, not silently applied
  with generic Cisco transit semantics. Same-link and moved/duplicate endpoints
  cannot masquerade as supported routed policy tests.
- None, missing reference, empty ACL, wrong router/port/direction, duplicate keys,
  excessive lists/rules/bytes, unknown packet fields and direct invalid values.
- Several routers and asymmetric directed flows; source-only versus service
  selectors; explicit/implicit deny; rule reordering; alternate equivalent ACLs.
- Baseline-down/route removal/forwarding-off/endpoint relocation, deny-all and
  unrelated-positive-flow shortcut probes fail proposed grading validation.
- No mutation after invalid operation, repeated evaluation unchanged, maximum
  16-hop traces and bounded decision/event storage, all original routing tests.
- No native/persistence/content/coverage acceptance inferred from these tests.

Primary sources are those inspected in ACL_SUBSET.md (Cisco IOS XE17 ingress /
egress / local-origin and empty-list distinctions, plus wildcard/port manuals).
The existing current CCNA v1.1 5.6 remains Open until reviewed substantive practice.

## Isolated implementation checkpoint

Twelve new integration tests pass alongside11 classifier,10 existing pure IPv4
routing, six authored routing-content and eight routing-workspace tests (47
total). They cover real boundary event order, independent failed baselines,
explicit/implicit denial and equivalent rules, directional request/reply cases,
required positive crossings, role/owner/interface spoofing, strict bounded JSON,
and a12-device/96-interface/384-route/512-rule topology with11 forwarding hops
and30 actual events. Original topology serialization/probe behavior and old
assignment outcomes retain regression coverage. This is not native, curriculum,
platform or complete-goal acceptance. Independent source review is pending.

Tests ran through the main shared safe-workflow runner with two Cargo jobs,
two test workers, explicit debug0/no-incremental settings and disk-backed
target/temp/toolchain. The final47-test run, scoped Clippy with warnings denied,
formatting and diff checks pass. No heavy invocation bypasses the shared lock.
All prior evidence remains intact. This observed run does not guarantee memory
headroom under other workloads or replace the required later integration review.
