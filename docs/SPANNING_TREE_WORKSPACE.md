# Bounded spanning-tree backend workspace

2026-10-25, SOURCE ONLY. Eighteen prospective adapter tests are UNRUN. No build,
VM, compiler, host command/reference, packet or native renderer was executed for
this adapter. The accepted pure core's separate68-library/20-model gate is not a
backend/native or curriculum result. No normal lesson, unit or objective credit
is added. The technical fixture remains outside the normal125 catalog.

The core contract and primary Cisco references are in SPANNING_TREE_SUBSET.md.
The adapter represents one fixed ideal point-to-point steady-state instance,
not a BPDU engine, RSTP/PVST implementation, convergence measurement, VLAN/frame
forwarding, PortFast or guards. The public core API, algorithm and dependencies are unchanged. The existing
borrowed Action::validate_for is now crate-visible for preflight reuse only.

## Fixed context and resulting current goals

Engine State::SpanningTree holds one validated simulation. Goal::SpanningTree
holds immutable initial State and at most32 typed requirements. Assignment
initial must equal the Goal initial. All operations/checks/decode compare the
simulation's exact fixed Topology (instance extension, bridge/MAC, port names and
numbers, link names/endpoints) to that initial. Configuration alone may evolve.
Snapshots derive afresh once per whole current predicate batch; they have no
Deserialize or saved-credit path. The initial is not duplicated into each State.

Requirement variants are Availability, Connected, Root, RootPort, PortRole,
RootCost, ForwardingPath, BridgePriority, PortSettings and Configuration. Each
Goal must contain exactly one complete declared Availability vector, at least
one2..12-bridge positive Connected group and a meaningful positive root/role/path
result. Behavior endpoints/root identities must be covered by the corresponding
positive connected groups. Offline/isolated roots cannot substitute for the
requested connected work. Availability includes every bridge.online, port.up and
link.up, including explicitly preserved absent/down settings. It excludes costs
and priorities, which can vary unless an exact predicate explicitly asks for them.

ForwardingPath requires distinct endpoints and a nonempty current path. Optional
expected links are ordered link IDs from source to destination, max11, checked
for adjacency and repeated vertices before evaluation. Parallel physical links
remain distinguishable. None accepts any current path. Root/Designated/Alternate
role checks require an operational attachment; Disabled is an absence check and
cannot be the sole positive work. Root cost uses actual local rootward port
costs. Exact numeric/configuration predicates are optional stated outcomes,
not a hidden universal reference configuration. Duplicate predicates reject;
pack reference reachability detects contradictory/unreachable requirements.

Direct collection/scalar/context checks (including each typed action through the
core crate-internal borrowed validator before wire counting) precede duplicate comparisons, derived
sets and clones. Goal compact admission (initial+requirements) is512KiB. Feedback
is current, contextual and freshly capped512 UTF8 bytes, without dumping a full
solved configuration. A saved checked flag, query output, event/action sequence
or prior success cannot authorize a result.

## Drafts and independent practice

Draft saves exact fixed Topology metadata, bounded bridge/port maps, raw priority
and cost strings (max64 UTF8 bytes each, no control characters), availability,
selection and notice. It admits malformed numeric text so unfinished work can
resume. Valid numeric input is decimal digits only; core priority increments,
positive cost ranges, shape and aggregate bounds remain authoritative. Draft
validity does not mean its configuration can apply.

Draft::new/valid/valid_for/same_shape/snapshot/matches/apply/fill are typed APIs.
Snapshot parses every field, validates borrowed shape and counts the proposed
wire before cloning context/configuration. Apply publishes one complete valid
candidate; any error retains the simulation and raw draft. Selection is not a
grading input. Pending text invalidates current assignment/practice completion.

Optional Lesson includes bounded actual prerequisite IDs, concepts, a distinct
fixture, annotated typed native configurations/actions, requirements, one
try-change outcome vector and repair. Its validator checks a substantive initial
repair, reachable worked outcome and exact experiment/repair results. Teaching
and scenario prerequisite IDs must match. No metadata or steps are authored as
normal curriculum in this stage.

Learning saves immutable fixture State, independent current State/Draft,
Introduction/Example/Assignment view, selected action, checked and notice. Every
operation/decode compares its fixture exactly with Lesson.fixture, including
initial configuration, then validates evolving simulation against that fixed
Topology. Authored bridge/link IDs and MACs are disjoint from assignment context;
port labels/numbers and instance extensions may coincide. Mutable values may
legitimately coincide and are never compared as a namespace-protection rule.
Fill inserts only the worked Draft; Apply is explicit. Practice cannot complete
an assignment. prepare rejects wrong kinds/foreign existing context before
publishing any new optional fields.

Draft compact admission is256KiB; Lesson512KiB; Learning1MiB. Learning holds at
most two core States plus one Draft (each<=256KiB), with small bounded metadata.
Constructors and successful mutations preflight the aggregate candidate before
publication/copies. Core raw State256KiB and Action64KiB ceilings remain; app-only
raw typed input is at most64KiB plus the fixed `spanning-tree ` prefix, checked
before trim/context work. This bridge is pack authoring data, not IOS or learner
shell syntax. Context-free engine mutations reject; help/show are simulation
helpers. Other command families retain their original caps and grammar.

Bounded field/map/sequence readers reject oversized typed values and duplicates,
never reserve an untrusted size_hint or silently truncate. Existing internally
 tagged Goal/Requirement readers may first buffer serde Content under the outer
pack2,000,000-byte ceiling; this is not raw-before-parse streaming. Nested serde
cannot measure a source byte span. Existing16,000,000-byte profile admission
remains. Oversized legal combinations can be rejected; global ceilings do not
increase. The worst-shape/escaping compact/pretty/profile test remains UNRUN.

## Persistence and evidence boundary

New optional fields omit None, leaving every old wire unchanged. Shared hooks are
narrow State/Goal dispatch, pack validation/reachability, workspace draft
initialization, current completion and save/decode kind/context guards. The sole
UI edit is an exhaustive test-match arm; there is no renderer or new bundled lab.
Parallel package hooks must be merged, not overwritten wholesale.

Prospective tests cover alternate priorities/costs, physical isolation shortcuts,
current root/role/cost/path invalidation, exact requested preservation, atomic
malformed input, raw pre-trim caps, duplicate/path/oversized direct requirements,
collection excess/duplicate fields, independent example Fill/experiment/repair,
wrong kind/foreign fixture, maximum12×8/48 graph and escaped64-byte drafts,
current125 complete source metadata/unit order and omitted fields. Historical
Goal defaults are compared as typed Scenarios. The baseline includes the current
scenario-specific DHCP explanations, not the earlier pending text.

The progress regression starts from frozen genuine writer8 bytes with original
6/7 histories, preserves legacy records before explicit current VM runs in the
test, creates current9 results only from those runs, retains distinct unfinished
script assignment/example and both new configuration drafts, then checks exact
all125 Attempt/favorite retention through save/load/export/inspect/confirmed
restore and selected reset. Existing before8 bytes and exact before9 target copy
are asserted. Current source fixture files are never regenerated or relabeled.
These are future test actions, not measurements already executed or fabricated
mastery. All18 groups, Clippy, full integration and native stages remain pending.

Later native UI should use compact responsive configuration/result columns and
shared font-derived controls, immutable per-lab/mode/bridge/port focus/Undo IDs,
modal guards, actual Apply/Check and clearly labeled applied results. Optional
reasoning can show actual election vectors; no question wizard, fake CLI,
protocol clock or convergence animation. Native geometry/input/cold persistence
acceptance and meaningful authored lessons require separate reviewed gates.

## MAIN first guarded execution — 2026-10-25

The independently reviewed frozen source was integrated incrementally over the
current package/DHCP application. No existing family hook was removed. The first
guarded command passes68 library,20 pure STP,18 STP adapter and20 pure Apache
checks. All-target Clippy with warnings denied also passes on the first run.
Historical source-only/UNRUN statements above describe the original handoff, not
the current execution status. Broader progress regression and native/content
acceptance are separate gates. No real protocol/server/package process was run.
The catalog and objective coverage are unchanged.
