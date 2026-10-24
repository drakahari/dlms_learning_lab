# Native service definition workspace

2026-10-23. The four normal activities in UNIT_MANAGER_LESSONS.md pass actual
Fedora input/render acceptance. The existing single-service workspace, all28
themes and script evaluator7 remain unchanged. This is a bounded in-process
simulator, not a host systemd controller.

## Workspace and teaching

Introduction → Try example → Assignment follows the accepted app pattern.
Introductions teach prerequisites and can be skipped/reopened. Challenge keeps
the full task and tools; help is explicit. Worked examples use different supplied
units, configuration paths and values. One visible example-action selector shows
an explanation and a supported command; Insert only populates the field and
requests focus after pointer release. Internal AdvancePrograms actions are hidden.

The entire task precedes one compact status summary and an editor/terminal split.
Wide layouts split; narrow/large-text layouts stack and scroll. The multiline
code editor stays top-aligned and scrolls horizontally/vertically. Single-line
commands, labels, dropdowns and buttons use shared font-derived centered metrics.
No per-screen text offsets or font shrinking are used. Geometry tests measure
an editor's visible viewport rather than its wider scrollable content.

Only one editor is visible: Unit file or Application configuration. Each has a
fixed supplied-slot picker. Save editor drafts atomically updates raw simulated
files, not loaded definitions or current programs. Save, daemon-reload, stop/start,
enable/disable and default-target changes are separate actions. Unsupported and
malformed text remains editable. No arbitrary host/profile path is accepted.

The virtual terminal uses the closed real-spelling subset in
UNIT_MANAGER_TERMINAL.md. Start includes the bounded fictional program
initialization event; no simulator-only initialization command is taught.
Read-only queries preserve checked completion. Actual state changes and pending
editor changes require a new check. An optional collapsed inspector shows saved,
loaded and current execution inputs without reading Goal/solution values.

One Check work derives current plus next-session outcomes. Two initial failures
stay close to the action; full objective results/help are optional. Passing at
walkthrough position0 is complete. Next in unit uses the existing course
navigation. A previous successful command, journal entry or checked flag cannot
hold success after the state/draft is broken. Preparation left active with old
input must be explicitly stopped and started after repair; it is not silently
rerun. UNIT_MANAGER_LESSONS.md records the conservative raw-input snapshot limit.

## Progress and bounded sessions

Assignment/example state, raw drafts, active editor kind, selected slots,
example-action selection, command draft/history and learning view persist
independently. Reset example affects only practice. Confirmed selected-attempt
reset affects only that attempt. Leaving/reopening retains unfinished work.
Original IDs/schema versions and default-absent metadata stay compatible.

The optional version1 terminal Session is bounded to16entries/128KiB total text,
with2KiB always reserved for current raw command input. Each command<=2048UTF8
bytes, stdout<=32768 and stderr<=1024. Oldest display entries are evicted before
publication; history is never grading authority. Session is borrowed rather than
cloned on unchanged frames. Direct and serde inputs receive equivalent count,
string, aggregate, control/status and unknown-field checks. Failed execution
retains the input and leaves simulated state unchanged.

Assignment sessions are valid only for UnitManager. Practice sessions retain the
exact authored fixture pin. Draft.editing_configuration defaults false/omitted;
true is valid only with configuration slots. Existing serialized workspaces omit
all absent fields. Safe writes/backup/import and writer-lock boundaries remain in
PROGRESS_RECOVERY.md. No learner text reaches a host process or filesystem API.

## Focus and validation evidence

Contextual control names retain native AccessKit role/value. Editor/command IDs
include lab, assignment/example, file kind and selected slot, preserving separate
undo histories across views, files and reflow. Tab leaves the code editor.
Only the focused command field can consume Enter/Ctrl+Enter. Reset/settings/
appearance/backup modals block background execution/focus. Insert never executes.

Eight session checks plus focused native-egui/inspector/example-selector tests
precede the full current120-UI-check gate. The normal four-task native plan uses
actual menu/typing/button events and no installed reference State: reference
values populate inputs only, then the real controller/grader derives outcomes.
All four tasks pass partial/wrong/reference/alternate, fill-only practice/
experiment/repair, current success invalidation, read-only queries, exact separate
draft/Session/editor-kind save/load, Undo, reset/cancel and Challenge. Actual
configuration editing/Save/terminal retry occurs at both150% light and200% dark.

Seventy-two actual captures pass in `m70-normal-service-native-r2.log`, root
`.build-artifacts/native-runs/run-pejyy86y`; representative introduction, task,
completion, retry and scaled screens were visually reviewed. The window manager
provided1268×632 despite a1200×850 request; readable stacking/scrolling is verified,
not forced desktop configuration. An earlier private17-capture foundation flow
and failed normal r1 are separate dated evidence, not added lab counts.

Native inputs are synthetic automation, not a novice user study or physical
keyboard/assistive-device acceptance. Windows/macOS desktop, screen-reader and
novice acceptance remain pending. No installer/release publication or claim of
full systemd, boot or curriculum coverage follows from these checks.
