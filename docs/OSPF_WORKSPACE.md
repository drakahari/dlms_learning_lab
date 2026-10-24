# Native OSPF settings and isolated practice

Checked2026-10-23 against OSPF_SUBSET.md. The normal catalog now includes four
configured-snapshot activities, documented in OSPF_LESSONS.md. The private
ospf-workspace-preview fixture remains excluded from curriculum counts. This
native Rust workspace introduces no host session, packets, shell or dependency.

The engine variant holds only the current `ospf::State`. The immutable authored
initial State lives in Goal::Ospf, alongside at most32 typed requirements.
Identical repeated requirements are rejected after bounded item validation;
distinct overlapping predicates remain valid when the reference reaches them. Every
mutation/check/import uses fixed device/interface/link identities and forwarding
roles. Non-forwarding host interface data is readonly and pinned exactly. Router
addresses/up flags and OSPF settings can change, with physical preservation or
requested physical repair stated separately in the goal. Snapshot observations
are derived afresh, never imported as success credentials or Full adjacency.

`ospf_workspace::Draft` provides new/valid/same_shape/snapshot/matches/apply. It
contains per-router enable/process-ID/router-ID/area text and per-interface
address/prefix/up/enable/passive/network-type/cost/Hello/Dead settings. Device and
port keys and the selected Endpoint stay fixed. Blank or malformed numeric text
is valid unfinished editing state, retained without truncation within64-byte
field bounds. Controls cannot change graph identity, roles, static routes or
host data. `snapshot(current)` parses every field and checks address uniqueness
before the topology clone; `apply(initial,current)` publishes only a valid
fixed-context candidate. An error leaves applied simulation and unfinished draft
text untouched. `matches` compares semantic settings; pending different settings
or invalid text cannot earn completion from an older applied configuration.

Goals use Compatibility/AbsentCompatibility, RemoteRoute/AbsentRemoteRoute,
Physical, optional exact Process/Port/RouterId and optional WholeState predicates.
Compatibility is a current derived P2P edge, not observed neighbor history or
DBD/LSA synchronization. RemoteRoute requires an actual current remote SPF
candidate, optionally with an explicit cost and complete equal-cost next-hop
set; a connected prefix cannot stand in for that. Every goal needs meaningful
positive Compatibility or RemoteRoute behavior. An absence predicate also needs
an explicit Physical requirement to prevent interface-down/address shortcuts.
Positive unaffected behavior and exact per-port/process guards preserve the
context stated by the task. WholeState is optional and is appropriate only when
the task asks for that exact final configuration. No hidden reference-state
comparison disallows otherwise valid alternate costs, matching timers, areas or
action orders. Pack validation proves initial work is incomplete and the typed
reference reaches every outcome. Checks return bounded local feedback, without
Debug enum dumps or automatic credit based on UI checked/Challenge flags.

The pack-only `ospf ` + JSON Action adapter uses the same typed core and immutable
goal context. Raw engine execution offers readonly help/show and refuses
mutations. It is not learner IOS syntax. The existing normal terminal assignment
callback already uses execute_for_goal; other state kinds retain their original
execution path. No new protocol clock, advertisement, Run/Stop or convergence
event is invented: Apply changes the existing derived snapshot immediately.

`ospf_teaching::Lesson` has prerequisites, concepts, separate fixture and
individually explained typed example actions, positive outcomes, a concrete
try-change with expected check results, and a repair. Default/try/repair behavior
is validated against the same core. Example graph names, link identities,
addresses and router IDs differ from assignment values. `Learning` contains its
own simulation, Draft, view and checked display state, plus one immutable fixture
copy as bounded context metadata. That copy must exactly match Lesson.fixture on
decode and every practice operation; it is not a history credential. Mutable
practice addresses/router IDs can legitimately coincide with current assignment
values without creating one shared namespace. `fill_example` changes only the
draft; `apply_check` is explicit and never touches assignment state or credit.

Saved optional ospf_draft/ospf_learning fields preserve all old kinds and Bash
evaluator7 history. Save-side validation rejects wrong-kind or malformed editing
metadata; catalog-aware decode additionally pins the assignment and exact
practice origin. A changed simulation must still satisfy actual current goal
requirements. Selected reset affects only that activity. Existing backup/export,
inspection and confirmed restore retain bounded drafts, current state and
provenance without a new migration scheme or evaluator revision.

Ingress and resource limits are explicit. State custom Deserialize uses bounded
typed device/port/config maps,32-byte names and bounded dotted numeric fields;
it rejects unknown fields/duplicate keys/nonempty static routes and runs intrinsic
State validation. It does not derive unrestricted authority from imported data.
State::from_json retains its raw256KiB input cap; Draft::from_json is256KiB and
Learning::from_json is1MiB. Generic nested serde cannot measure the raw source
span of a subobject. It enforces structural/count/text bounds before retaining
data; padding/escapes remain governed by the existing outer2MB pack/16MB profile
reader caps. The existing writer refuses encoded profiles over16MB before any
replacement. No draft or profile is silently shortened. Snapshot output has no
decoder or saved completion authority. NextHop decoding is only bounded expected
goal data, validated against fixed endpoints; it is not proof of a current route.

The maximum-shape regression uses12 routers ×8 ports with32-byte names, maximum
numeric parameters and64-byte quote/backslash draft fields (maximal JSON escaping
for permitted draft text), plus512-byte notices. Both immutable/mutable practice
copies, compact/pretty Learning and a real profile save/load must roundtrip under
their declared ceilings. The core's directed positive-cost arithmetic and
equal-cost-set bounds remain unchanged. The source also tests the important
same-graph/different-authored-initial practice copy case and legitimate evolving
identity coincidences. These are developer fixtures, not counted lessons.

The adapter preservation regression pins the original98 catalog IDs explicitly,
including the ACL/account lessons. It constructs real current evaluator7 runs
for the37 existing scripts and retains the33 actual frozen evaluator6 records
(182 outcomes) as history without relabeling them. Other reference commands use
the goal-aware engine wrapper. Save/load/export/inspection/confirmed restore
and selected reset assertions retain exact attempt data, unfinished assignment
and example drafts, favorites, and the observed current-completion results.
This source-authored regression is not a new curriculum or native UI claim.

## Native interaction and authoring

Introduction, a different ungraded practice network and the full assignment use
separate persistent editing state. One Check configuration applies all numeric
and toggle fields atomically and grades fresh state. Pending edits clear current
success. A router/interface selection is navigation and does not break success.
A new/reset assignment opens the router/endpoint of its first positive task
requirement; it never fills required values or infers a desired next hop. A new
practice example opens its first explained action's router/interface; Fill keeps
the learner's current selection and changes only example field values. Resume
keeps saved selection and drafts. Put meaningful behavior before preservation
guards in authored requirements; cross-unit prerequisites resolve through the
combined catalog, not standalone single-pack loading.

Shared font-derived native fields retain padding, cursor/selection geometry and
stable IDs across responsive column/stack layouts. A compact chain uses equal
cards with centered connectors; more complex graphs use truthful link readouts,
with directional raw link/LAN costs rather than solved path totals. Current
parameter mismatches are visible before checking. Failed objective feedback
names the local field or configuration condition. Passing preservation details,
worked reference values, advanced scope and sources are optional. Modal backup
operations disable background shortcuts/focus. Challenge keeps the complete task
without unsolicited worked solutions.

Actual Fedora normal-unit native input/render acceptance is recorded in the
implementation ledger, separate from headless UI checks and synthetic progress
roundtrips. Neither establishes novice comprehension, physical-device or screen
reader acceptance, real routing-equipment experience or Windows/macOS runtime.
