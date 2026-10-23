# ACL backend/teaching integration contract

Specified2026-10-23 before integration. This milestone adds typed backend state,
goals, pack validation and optional local draft/example data, not native UI or
authored certification coverage. The existing90 scenarios and shell evaluator7
remain unchanged. The one development fixture is unbundled and uncounted.

`State::Ipv4Acl { network }` pairs only with `Goal::Ipv4Acl { definition }`.
The initial state must exactly equal definition.initial. Current policies and
bindings are editable; every supplied topology field and host-role declaration
remain pinned. Native-renderer compatibility is checked, including virtual port
labels and supported compiler IDs. Flow cases establish only the stated virtual
topology/service outcomes, not arbitrary unmodeled hosts, ports or full IOS CLI.

Global apply_documents validates current state and bounded raw documents, parses
every supplied router document against the ORIGINAL network, then totals the
final combined network policies. Only afterward does it clone once and publish
the complete candidate. No lexical-router-order limit rejection: removing a list
from one router and moving it to another can pass at the final8-list/512-rule
limit. Any malformed document or final error retains every applied router.

Saved version1 drafts contain every known non-host router's bounded document,
selected-router view and a short optional notice. Malformed work stays separate
from applied policies. Existing64KiB/router and256KiB aggregate limits apply.
Foreign/missing router keys and incompatible selections are rejected; unknown
workspace types are rejected before writes/import. Draft edits are never answers
until explicitly applied and freshly checked. A stored checked flag controls
feedback visibility only; current success always runs definition+state+draft
grading, including no-ACL delivery, required positives and denial boundaries.

The internal `acl-author {"kind":"ApplyDocuments",...}` reachability adapter is
declarative authoring data, not Cisco/Linux terminal syntax and not a learner
command subset. Native UI will call typed helpers directly. There are no host
files, processes, sockets, packets or infrastructure side effects.

An optional Scenario.acl_teaching field is required for this new activity type.
It contains short concepts/prerequisites, a different isolated example network
and fixed definition, worked documents, and a concrete try-change with expected
results for every example case. Validation executes only bounded simulation on
disposable state: worked configuration must pass; try change must change state
and match its declared outcomes. Fill writes example drafts only, does not apply
or check, and cannot access the assignment. Introduction/Example/Assignment view
and example malformed drafts resume separately. Challenge retains the full task
without unsolicited example solutions. No native renderer is added here.

Pack and import validation pin the complete assignment/example topology and host
roles to authored data, and reject foreign metadata. Save-side validation checks
state/draft shape and bounds without a catalog; full authored pinning happens in
the reader. A selected reset replaces only that attempt; existing progress,
shell7 transition receipts/history, all old drafts and favorites are preserved.

Required tests: multi-router move/swap at limits, late invalid-document rollback,
alternate explicit/implicit denial, incorrect/partial flows, current success
invalidation, fill/apply/check isolation, concrete try outcomes, malformed drafts
and exact resume, selected reset, foreign assignment/practice/profile rejection,
missing/invalid pack metadata, all90 original attempts and shell7 receipts/history
round-trip. Guarded testing/source review precede any native/content promotion.

## Native workspace acceptance —2026-10-23

The normal native dispatcher now supports this typed activity with one complete-document editor and one Check configuration action. The full task and default-open required-packet table are followed by supplied network state, router selection, editor and focused feedback. Small three-device paths use shared equal-height topology cards with centered connectors; other supplied shapes currently have a truthful interface/link/address list. A blank document removes that router's bindings, explicitly explained beside the editor. No author JSON is shown as learner command syntax.

Learn retains skippable Introduction → separate ungraded Try example → full Assignment. Fill worked example populates only the example documents; applying/checking is explicit. Example state/draft/undo is separate, with a concrete try-change. Challenge keeps the complete assignment and task table, with optional hints/reasoning/reference scoped by lesson and mode. Completion is fresh current policy plus matching drafts, not walkthrough position or a saved flag. Editing invalidates it; switching from a completed assignment through example to Challenge retains honest current completion.

Native code editors remain top-aligned and horizontally/vertically scrollable. Tab leaves the editor, Escape releases focus, Ctrl+Enter checks. Tab/router changes defer focus until pointer release, preventing the old editor consuming focus. Per-document64KiB and aggregate256KiB edit bounds are checked before publishing; a rejected edit retains the previous saveable draft. Errors appear once with Check; blocked required traffic, wrong enforcement boundary and correctly observed blocks awaiting restored allowed traffic have distinct feedback. The bundled font lacked a right-arrow glyph in table text, so plain “From / To” and “to” replace it; the same display-only wording fix covers IPv6 probe output.

Acceptance uses the unbundled development fixture, not new curriculum credit. Root186 combined backend/regression tests,10 ACL workspace checks and5 new native-egui tests pass;89 existing UI regressions and3 focused IPv6 display/workflow tests pass, plus final all-targetClippy/formatter. Earlier focused failures and fixes remain in private evidence. Actual Fedora native input/render flow covers fill/apply/change, partial and alternate configuration, keyboard Check, exact save/reopen, invalidation, selected Reset, Challenge, tiled1268×632 and wide2560×1440,100/150/200 and light/dark. Root inspected the actual screenshots; no physical-device, screen-reader or novice-user acceptance follows. Final flow screenshots: ignored `.build-artifacts/native-runs/run-8krbcc3b/screenshots/workspaces`; logs `m60-acl-native-final.log`. No existing bundled scenario, dependency or public shell revision changed. Catalog remains90; four future ACL lesson proposals are separate pending content tests/registration/native acceptance.
