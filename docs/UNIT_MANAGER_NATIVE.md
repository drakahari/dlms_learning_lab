# Native virtual-manager workspace proposal

2026-10-23. Native workspace foundation after tested terminal6e70c4d. The scoped
headless/serialization checks below pass; actual Fedora rendering is pending. The development fixture remains unbundled;
no lab, objective coverage, operating-system or novice acceptance is promoted.
The original single-service model and the shell evaluator are unchanged.

## One meaningful workspace

Introduction → Try example → Assignment follows the accepted app pattern. Learn
introduces the authored prerequisites and concepts, explains editing/Run, and
shows two diagnostic commands using the different example's supplied unit.
Experienced learners can skip to the entire assignment. Challenge uses the same
full task, editor, terminal and grader; it hides the introduction/example tabs and
offers guidance only on request. It never displays the authored action JSON.

The assignment stays visible above a compact status summary and an editor/terminal
split. The split uses readable font-relative available width; narrower windows
stack. The multiline editor remains top-aligned and scrolls in both directions.
Only one editor is visible: Unit file or Application configuration. Each has a
fixed-slot picker, with explicit supplied virtual names. Editor kind and both
selected slots persist. Unit text, configuration text and command drafts survive
leaving, reopening and application restart. No arbitrary profile or host paths
are taken from lab input.

Save editor drafts updates raw supplied files atomically and does not reload,
start or enable anything. Unsupported/malformed file text remains editable.
The virtual terminal accepts the reviewed real-spelling closed subset. Start
uses the same bounded post-job initialization policy as the terminal foundation;
there is no learner AdvancePrograms control or pretend Linux helper. Example
annotations map only supported typed actions to real command text; Insert only
populates the example command field. The internal initialization annotation is
omitted. Fill worked editor drafts edits only example raw text, and still requires
Save and learner-executed commands. Example state, selections, command history,
checks and reset are independent of the assignment.

The compact summary distinguishes unsaved editor text, current loaded unit text,
current runtime (including earlier consumed inputs), current install policy and
future-session target. It does not present enabled/static/active as proof of a
working future session. One Check work derives the complete current and fresh
outcomes, with two useful first failures and optional full objective results.
Passing does not depend on walkthrough position. Current completion is never
read from terminal messages, exit codes or an old checked flag.

The shared Back to library/Reset/navigation remain. Reset example clears only its
practice; selected-attempt reset clears that attempt's saved text, terminal and
practice. Read-only queries preserve the visible check state; actual mutations
and raw editor edits require a fresh check, while the existing state-based
completion predicates remain authoritative. Help has progressive hints, worked
reasoning and technical boundaries/sources behind explicit requests. It does not
use the generic guide's hidden command input or expose authoring JSON.

## Bounded optional session data

`terminal::Session` is additive, version1, default-absent in assignment workspace
and practice Learning. Existing workspace/schema/evaluator versions do not change.
It contains at most16 entries and128KiB total retained string budget, with2KiB
always reserved for the current raw command draft. This reservation prevents
later typing from making an otherwise valid transcript unsaveable. Oldest entries
are evicted before a new genuine observation is published. Each command<=2048
UTF-8 bytes, stdout<=32768, stderr<=1024. Status is the closed supported0/1/3 set;
syntax/unsupported/context/limit/model diagnostics have no invented Linux status.

Serde bounds every string and counts/aggregates entries while reading, then
validates the whole version/shape before any retained clone. Direct inputs get
the same validation before staging State. Raw malformed command drafts (including
incomplete quotes) are retained. Display output forbids controls other than LF/
tab; command history is safely escaped for display. Session is borrowed on an
unchanged UI frame; the renderer does not clone128KiB histories on every repaint.
Execution stages one bounded candidate and publishes state/history only after
validating the result and planning bounded retention. A rejected command leaves
State unchanged and retains its input for correction.

Assignment Session belongs only to UnitManager. Existing save/decode kind guards
reject it on another state before replacing prior progress. Practice Session is
validated with the exact authored fixture pin and has no assignment access.
Neither data is a grading credential. Default absence omits the new fields from
old serialized workspaces. `Draft.editing_configuration` defaults false and is
omitted when false, preserving old documents while retaining a learner's active
configuration editor when true. True is invalid if the fixed inventory has no
configuration slots. It has no effect on source matching or grading.

## Focus and modal boundaries

All selectors/fields have contextual AccessKit names using the existing shared
helper; the native role/value is retained. Single-line command fields and buttons
use shared centered metrics. Editor and command IDs explicitly include lab,
assignment/example role, file kind and fixed selected name; they remain distinct
across reflow and avoid cross-context undo state. Tab can leave the code editor.

Only the currently focused terminal command can consume Enter/Ctrl+Enter. It
cannot execute while reset confirmation, Settings, theme samples or backup/restore
modal is open. Editor Enter/Ctrl+Enter never silently saves or runs a command.
Switching lesson view returns before rendering the old input context for another
frame. Pending autofocus waits until pointer release and respects disabled UI.
The focused headless checks verify these input/metadata paths. Actual native
render/input checks remain pending; metadata is not screen-reader acceptance.

## Scoped headless validation — native acceptance pending

New `tests/unit_manager_sessions.rs` has8 passing tests: query/mutation/
pending-draft credit, malformed retry, count/byte retention with full command
reservation, direct/serde limits/control/status/unknown data, optional fields and
real save/reload, practice isolation, wrong-kind refusal retaining exact bytes,
forged history not earning credit, and active editor kind/new/legacy metadata.
The5 native-egui tests cover actual example Fill/Save/typed commands, assignment
isolation, query success retention and editor invalidation, context undo, modal
keyboard suppression, Challenge defaults, wrapping/scales and stable focus/Tab. A long-diagnostic regression verifies that
error explanations wrap to the finite terminal viewport at150/200% even when
preceding commands and tabular stdout need horizontal scrolling.
All13 new checks passed on their first guarded run, together with130 existing
manager/core/terminal/workspace/integration/backup/revision6+7-history checks.
All-target Clippy with warnings denied and full-crate formatting checks pass.
The first Clippy found3 test-style issues; correcting those changed no runtime
behavior, and the affected8 tests were rerun successfully. The earlier main-base
module-order formatting correction was also applied locally. Exact source hashes,
first diagnostics, final results and memory observations remain in ignored review
evidence. These are143 distinct checks; repeated tests are not added to that count.

Mandatory existing regressions include manager core/workspace/integration,
terminal/journal, backup and actual revision6+7 shell history. Final merge must
preserve the main checkout's independently developed OSPF renderer and rerun its
own tests; this branch's earlier main base is not a substitute. No public
screenshots or generated progress belong in the commit.

## Subsequent native smoke proposal

After the first guarded compile/headless review, append only the development
pack fixture inside the existing app-only smoke harness. Do not bundle it or
create a new curriculum entry. Stage a normal Fedora window, inspect Introduction,
enter practice, Fill (assert only draft changes), Save and type the real diagnostic/
reload/enable/default/start commands. Check the ungraded example, alter its config,
retry and repair. Open assignment and prove its initial state/drafts remain exact.

Type the assignment unit/config edits through the actual native editor, Save,
run status/journal, show a real wrong configuration/failed program, repair through
supported commands, Check all current/fresh outcomes at Step0, run a query and
retain success, then change text/state and invalidate. Save/reopen must retain
separate pending command and file drafts, journal, history and active editor kind.
Exercise actual Reset→Cancel/Confirm and example-only reset. Confirm Enter/Tab/
Ctrl+A/Undo behavior, disabled-modal keys and clean optional help/Challenge.

Capture actual editor/terminal/feedback screens in light/dark at100/150/200%,
wide split and narrow stacked, with the measured viewport used for scrolling and
clicks. Record scoped PASS only after all assertions; reject partial/timed-out
runs and stale binaries. Actual Windows/macOS, full accessibility and novice
acceptance remain separate. No actual native run has happened for this workspace yet.

## Optional live inspector — headless-verified enhancement

Following the accepted143-check workspace checkpoint9711c8a, the collapsed
Loaded definition and current inputs inspector exposes only the selected unit's
actual saved/loaded source, loaded virtual program descriptor and its current
saved configuration, runtime source/program, consumed configuration/prerequisite,
and derived result. It does not read the grader's expected values or worked
reference. The compact summary includes the current derived port/value and marks
earlier execution inputs. It introduces no new Linux command or show property.
The introduction teaches where to open this read-only view. The sixth UI test
checks distinct saved-new/loaded-old/runtime-old program identities, reloaded-
program/current-input mismatch, default-closed/actual header opening and no
mutation by inspection. It passes; actual Fedora rendering remains pending.

## App-only native smoke plan — compiled, not executed

The reviewed proposal now has a source implementation in
`src/ui/smoke/unit_manager.rs`, reached only with `--ui-smoke` and the invocation-
local `DLMS_SMOKE_STAGE=unit-manager-workspace`. Stage12000 appends the private
fixture only to that smoke app; the normal catalog remains unchanged. Its bounded
plan uses real measured picker options, editor clicks and typed input, Save,
terminal Run/Enter, Check, optional inspector, Reset/Cancel/Confirm and Challenge.
It never installs a reference State. Authored reference/alternative values target
only bounded editor input events, and all claimed outcomes are asserted from the
actual current simulation, draft, Session or persisted profile reader.

The source plan covers initial failed initialization/journal diagnosis, partial
unit/policy repair while config remains wrong, full repair, valid Before ordering,
read-only status preserving success, a loaded-versus-running wrong-program
inspection, real state/edit invalidation, isolated context Undo, unfinished
assignment/example file and command drafts with active editor kind, exact real
save/reload, example-only reset, selected Reset with cancellation and another lab
preserved, and light150/dark200 stacked Challenge feedback. Screenshots are
requested only from actual frames. The PASS branch is after every assertion;
partial/timeout runs are not acceptance. The plan has an additional bounds/private-fixture test. All7 focused tests
(the6 manager UI checks plus that plan test) passed, and all-target Clippy with
warnings denied and full formatting checks pass. The first Clippy reported2
nested-if style warnings in the inspector; equivalent let chains fixed them and
the focused tests were rerun. No behavioral/model change was needed. The5 prior
UI checks are repeated regressions, not5 more new tests. Actual native execution,
final merged-catalog regression and screenshot inspection remain mandatory before
claiming Fedora acceptance. No native run was performed on this earlier98-lab
worktree; the root agent will run the final merged application.


## Merged Fedora native verification — M69

Merged main102-catalog development app passes the private fixture's actual-input
plan in m69-manager-native-r1.log (17captures, native-runs/run-erni39j9). Root
reviewed the real assignment, loaded/current-program mismatch and large dark
layout. Actual menus, whole-file text editing/Save, command Run/Enter, example
isolation, partial/failure/repair, Before alternative, current/fresh completion,
query retention, state/edit invalidation, exact separate command/file/history/
editor-kind resume and Reset/Cancel pass. Light150/Dark200 Check/scroll/layout
views pass; native field typing at those larger scales is reserved for the
upcoming authored-unit flow. Headless scaled editing/focus tests already pass,
which is a separate kind of evidence. No authored unit-manager task is bundled
or counted yet. No novice, physical-device, screen-reader, full-systemd or
Windows/macOS runtime acceptance is inferred.

Root main750library/integration and117distinct UIchecks pass combined after the
separately recorded narrow OSPF heading correction; all-target Clippy/fmt pass.
The earlier failed wrapper is preserved, not relabeled as green. A final two-string
feedback refinement removes the hidden initialization instruction and developer
jargon;65core/workspace/integration/terminal/session checks pass afterward, and
the actual native flow above uses that final text.
