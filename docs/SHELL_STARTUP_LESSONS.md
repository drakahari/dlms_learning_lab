# Bash startup files: two applied LPIC-1 lessons

Source integration2026-11-04; focused and native verification in progress. These
lessons follow **Shell values and child environments** in LPIC-1, building on
quoted assignments and export before introducing startup selection.

| Stable ID | Practical task | Distinct failure to diagnose |
| --- | --- | --- |
| `lpic1-shell-startup-priority` | Repair exported application settings while preserving system defaults, a private note and fallback files. | A correct lower-priority file is irrelevant when `.bash_profile` wins; an empty winning file still blocks fallback. |
| `lpic1-shell-startup-quiet` | Keep interactive greetings while providing quiet startup settings to scripts. | A greeting corrupts script output; a setting without export does not reach the supplied new program. |

Each task has six login or five interactive/script sessions, including changed and spaced input
values. The grade uses actual selected-file execution, variable values/export
attributes, startup streams/status, unchanged sample files and an actual simulated
new-program report. Printing that report or exiting early is insufficient. No
exact-text solution matching is used. The first task preserves the local note
only in sessions whose selected personal file defines it; fallback sessions must
leave it unset. The second preserves inherited settings when no startup file is
selected. These are new objectives and never grant credit from an older lab.

## Learning flow

Introduction explains login versus interactive sessions, actual file priority or
quiet startup, and worked settings with **WORKSHOP / TOOL_LABEL**. Try example
runs separate virtual files using those names, then invites a deliberate break
and repair. Assignment uses **PROJECT / APP_NAME**, with its requirements visible
and no answer inserted by default. The worked files cannot directly solve it.
File selection, editor, Run startup checks, Stop, per-session feedback and output
inspection remain the workspace; no host profile preparation is needed.

Drafts in both activities, selected file/session and lesson view persist. Edits
clear current proof. Saved test stamps request real verification on resume; they
do not reconstruct mastery from a flag. Stop/unfinished checks grant no credit.
Reset affects only the selected attempt; backup/restore uses existing validation.

## Scope and acceptance

This is partial LPIC-1 v5.0 105.1 practice, not complete objective coverage.
See [supported model](SHELL_STARTUP_SUBSET.md). Alias/history/prompt/PATH command
lookup, source/dot, expanded BASH_ENV paths, unreadable files, invoked-as-sh and
remote/privileged startup remain unimplemented here. The ordinary evaluator17
and all older script lessons retain their supported behavior.

The upstream Bash manual informs selection. Eighteen fixed developer-only
comparisons with Bash check manually sourced settings and status, not automatic
host login startup. Their immutable raw observations and separate independent
review are private evidence; Bash is never the learner engine. This adds no
runtime dependency or redistribution of Bash. Actual Fedora screens, final test
results and candidate identity will be recorded when performed. No Windows/macOS,
screen-reader, physical-device or independent novice acceptance is inferred.

## Focused checks completed before native review

Ten core, seven workspace/storage and five authored groups pass. The exact old
267scenario values and79units/order are retained. Three native-widget groups
cover actual typing, Ctrl+Enter, Escape, correction, partial results, success
invalidation, loaded-proof re-verification and100/150/200percent light/dark layouts
at800/1440widths. A fourth checks every fixed native recipe expectation against
actual model execution plus the screenshot-request phase. Ten retained startup
and three filesystem behavior groups pass.660Python checks pass. The default all-target Clippy pass followed two style fixes; final refresh
after strengthened fixtures is pending. These are not actual rendered or novice acceptance;
native capture and isolated packaging are next. Independent read-only content/
code review found no substantive grading/persistence/isolation defect.

The final fixtures vary every editable startup context twice: six login sessions
and five interactive/script sessions (with a separate no-startup-file case).
Hardcoded application settings and hardcoded fallback system defaults have
targeted negative tests. This catches plausible shortcuts, not every possible
program specialized to a finite test set. Each task remains one authored lab.
Native attempt1 caught an empty-paste test-driver mismatch; explicit Backspace
and an actual editor-clearing regression fix the driver without weakening its
state assertion. Attempt2 is in progress; no new whole-app inclusive claim.

## Final source gate before packaging

Final all-target Clippy passes27.19seconds;660Python checks pass11.818seconds.
Development native run-_v1ymaos completes110steps/28PNG. Introductions, independent
assignment and enlarged light controls were opened and inspected. The final source
defaults to the file actually selected in the first session, rather than an
alphabetically first correct fallback; persisted file choices remain valid. One
worked-table value is shortened to avoid horizontal scrolling. Packaged native
verification must inspect those refinements and specific error feedback. The
original267scenarios and79units retain exact values/order. No current whole-app
inclusive, install lifecycle, full-process cold or novice acceptance is claimed.

## Packaged Fedora verification2026-11-04

Candidate T20261104T204143Z validates269labs/80units and passes110native steps,
28PNG in run-vu7tdiyy. All six normal Introduction/Example/Assignment views, the
specific legacy-value error and representative light200controls were opened and
inspected. Actual initial editor selection is the faulty file selected by sample1;
the shortened selection-order value no longer needs horizontal scrolling. The
previous empty-paste driver failure is retained; explicit Backspace and regression
coverage fix it without weakening the asserted paste result.

[Introduction](../.build-artifacts/native-runs/run-vu7tdiyy/screenshots/workspaces/teaching-lpic1-shell-startup-priority-introduction-dark-100.png),
[Practice](../.build-artifacts/native-runs/run-vu7tdiyy/screenshots/workspaces/teaching-lpic1-shell-startup-priority-example-dark-100.png),
[Assignment](../.build-artifacts/native-runs/run-vu7tdiyy/screenshots/workspaces/teaching-lpic1-shell-startup-quiet-assignment-dark-100.png),
[Specific feedback](../.build-artifacts/native-runs/run-vu7tdiyy/screenshots/workspaces/startup-lpic1-shell-startup-quiet-starter-feedback.png).
Screenshots/private receipts remain ignored; source links are local evidence.

528source/data hashes and242offline documents (52continuation guides) matched
MAIN before post-build notes. The initial private verifier assumed all documents
were the52continuation guides; it was corrected to verify all242and the exact
previous240-name set plus the two new guides. No artifact/notice was removed.
Independent read-only review and660Python/default all-target Clippy pass accompany
the scoped tests. No new full-process cold, install lifecycle, inclusive suite,
Windows/macOS/Omarchy, screen-reader/physical or novice acceptance is claimed.
