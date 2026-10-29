# Physical-uplink outcome and teaching backend

Unregistered capability work, based on the reviewed pure model. This is not
implemented course coverage or a claim of native acceptance. The gated R2 pure
backend originally added no engine/progress hooks; the current source integration
adds typed PhysicalLink State/Goal, optional Scenario teaching and separate saved
draft/practice fields against the pinned Main168 foundation. It adds no registered
Scenario, course unit, catalog count or coverage promotion. Native rendering remains
a later reviewed phase; unregistered technical fixtures are refused by the GUI.

`physical_link_assignment::Definition` is the typed payload intended for a later
Goal variant. It pins the complete initial topology/environment, including route
spans, cable conditions, port capabilities and traffic patterns. Requirements
name real links, minimum operating rates, optional full-duplex policy and at least
two distinct bidirectional deterministic sample cases. Protected neighbors retain
their full configuration/cable selection. Retired old interfaces must be stopped,
allowing a meaningful replacement-path handover with separate operations traffic.

Grading reads actual current configuration and runs independent sample probes.
Each sample constructs fresh counters/phase around that configuration; checking
does not reset the learner's own counters. Reported readings are optional source
material and never confer mastery. Sparse injected faults must be exercised by
every authored sample, so hardcoded zero counters or a favorable saved phase
cannot conceal them. A reference configuration must actually satisfy all outcomes;
an already-complete initial setup is rejected by reference validation. Correct
Auto and forced supported solutions can both pass, subject to the assignment's
minimum rate/duplex requirement. Wrong/protected-neighbor/partial/current-state
failures remain failures after a previous pass.

The core now has a bounded atomic `Configure` action over the complete supplied
configuration. It validates all port/link/cable domains and the16KiB configuration
allowance before cloning a large state. Valid two-link cable swaps need not pass
through an invalid duplicate-cable intermediate state. Counters, sample sequence
and all immutable topology data remain intact. Action input is bounded32KiB;
reported-reading input remains independently bounded8KiB. These changes live in
the later adapter worktree; the earlier R2 core source/proofs are preserved.

`Draft` holds native configuration values plus selection IDs. Partial unsupported
choices can remain in an editable draft; Apply validates them and refuses without
altering applied work. Pending edits invalidate current success. Selection alone
does not alter the configuration or grant credit.

`physical_link_teaching::Lesson` supplies short concepts, a different practice
Definition, a worked configuration, an adverse experiment and a valid repair.
All assignment/practice port, device, cable and link identities must be disjoint.
`Learning` pins the full practice lesson, persists its independent current state,
draft and Introduction/Example/Assignment view, and rejects changed lesson context.
Fill worked example populates only a draft; Apply is separate. Examples never own
or mutate the assignment State. Reset is scope-specific; source mismatch retains
work. No script, real shell, filesystem, network or clock is invoked.

Definitions256KiB, drafts64KiB, lessons384KiB and separate learning1MiB have
streaming encoded bounds and pre-body collection/text limits. Native rendering
still needs a full-source borrowed admission/memo budget, frame-wide edit veto,
whole-Attempt2MiB writes, persisted optional metadata/foreign guards, examples and
assignment reset/resume/backup tests. Ordinary saved progress must remain intact;
new requirements cannot invent answers or mastery. Raw/model evidence must remain
visibly distinct. No grading boolean or saved probe receipt may replace current
state checks.

Before any content registration: gate the pure core, gate this adapter/teaching
backend, review technical vectors and alternate/wrong/partial outcomes, compose
State/Goal and progress/loader guards against current Main, then implement the
accepted native topology/configuration/investigation workspace and inspect actual
screens. Introductions remain skippable, practice separate and the entire meaningful
assignment visible. Compact outcome summaries need optional complete diagnostics;
do not render a wall of all backend rows. Windows/macOS, hardware, novice and
screen-reader acceptance remain unverified.

## Embedded wire admission refinement (source R2)

Core identifiers (including map values naming selected cables) are admitted as
normalized identities up to48 decoded bytes before an owned field copy. Port and
cable labels and supplied-reading notes require nonblank control-free text up
to120 decoded bytes. These checks apply to direct embedded Deserialize as well
as the raw capped helper functions. The JSON parser can still use scratch space
for escape decoding; bounded enclosing raw inputs remain required. This is not
a claim of allocation-free parsing or a general parser memory sandbox.

The finite Action, Negotiation and Condition wire formats now use streaming
visitors instead of internally tagged derived deserialization, which buffers
arbitrary Content before inner field visitors run. Serialization and arbitrary
field ordering are preserved. Only exact variant fields are admitted. Unknown
and duplicate fields are rejected before their value; configuration maps and
advertised abilities retain their pre-body cardinality checks. Unknown action
field/tag diagnostics do not copy arbitrary untrusted names into the error.

Seven additional prospective technical groups exercise early scalar/map/action
refusal with poisoned tails, nested collection limits, direct unknown/duplicate
fields, every supported variant round trip with non-tag-first order, exact
Unicode-byte label boundaries, and wrong/mixed/missing field failures. Original
R1 and pure R2 source receipts and tests remain unchanged. This later source
refinement is uncompiled/unrun until the shared heavy-work lease is granted;
it adds no authored lab, curriculum credit, Scenario or progress metadata.

## Current complete-attempt integration (prospective until gated)

`physical_link_workspace` pins the supplied Scenario kind, full initial Definition,
immutable topology/environment and complete separate practice lesson. Per-family
foreign helpers and existing native memo source/workspace guards reject the new
optional fields before warm reuse or preparation. Nil fields preserve old wire
objects. Global progress decode/save validation and current completion use actual
state with pending-draft refusal, not a saved report or checked flag. Save without
a Scenario validates structural metadata; source-aware decode/import checks the
full currently supplied lesson. No personal profile is used as a test fixture.

All writers use exact borrowed whole2MiB Attempt projections, including absent
workspace overhead, both scopes, flags, view names, selected IDs and visit/section /
hint digit changes. Valid shrinking edits are permitted; rejected writes retain
all caller fields. Core ActionProjection borrows topology and uses only bounded
Configuration/counter/sequence scratch (under64KiB), sharing the original probe
arithmetic with actual execution. It serializes the exact projected State before
any full State/Attempt clone. Unsupported/domain/overflow refusals are preflighted.
Practice creation and scope resets are projected before owned lesson/state copies.

Core visitor guarantees do not remove buffering in the existing enclosing tagged
Engine State/Goal deserializer. Raw pack2MB/progress16MB caps remain necessary;
this is not allocation-free parsing or a new unrestricted content loader.

The actual gated R2 receipt remains immutable. New source/projection/whole-attempt /
reciprocal/compatibility tests are prospective until the shared runner is granted.
Existing168 definitions,43 units, historical captures and all before12 identities
remain unchanged. The authored four-task physical unit stays in a separate draft
review overlay until typed/native acceptance and normal registration are reviewed.
