# Modal text editing: bounded native practice

Current source adds one LPIC-1 unit after **Files, paths and safe handovers**:
**Text editing: repair, maintain and recover**. Its three stable new IDs are
`lpic1-modal-setting-repair`, `lpic1-modal-record-maintenance` and
`lpic1-modal-recovery`. Registration is448 labs/177 units; rendered lesson review
is still in progress. The445 candidate remains the last verified package until
this batch's candidate gates finish. No full vi, exam or novice acceptance follows.

The first lesson teaches the buffer/file distinction, Normal and Insert modes,
movement, correcting text and writing. The second introduces plain search and
linewise/characterwise copy/delete/put. The recovery assignment starts with an
intentional unsaved mistake and requires resolving it before repairing another
file. Introductions and worked examples precede separate guided practice; the
independent assignments keep their own state. File names are supplied virtual
labels. No personal setup, terminal or host editor is required.

## Implemented subset and boundaries

- Normal: `h j k l`, `0 $`, `gg G`, `i a o`, `x dd yy p P u`, `/ ? n`.
  Native arrows, Home/End and Delete provide Normal-mode movement/delete aliases.
- Insert: printable ASCII, newline, Backspace and Escape. Backspace may erase
  before the insertion start and join lines, matching the observed Vim-style
  permissive behavior. Real vi/Vim options can change that behavior. Insert-mode
  arrows are explicitly unsupported; Escape then move in Normal. Tab navigates
  application controls rather than inserting a tab.
- Search: case-sensitive plain letters, digits, spaces and `_-=:,@%`. It begins after/before the cursor and
  wraps. `n` repeats the last direction; an empty search reuses its prior text.
  Regular expressions, delimiters and offsets are rejected, never approximated.
- Colon: `:w`, `:wq`, `:q`, `:q!`, completed by Enter. No filename argument,
  shell escape, editor script, plugin, external process or arbitrary path exists.
- One unnamed internal register distinguishes complete lines from characters.
  It is separate from the desktop clipboard and is not undone with the buffer.
  Undo retains32 grouped changes. An Insert session is one change. There is no
  redo, branching history, named register, count, visual mode or general Ex.
- The text model bounds each buffer to16 KiB and32 snapshots. These activities
  further restrict current/saved files to4 KiB and four supplied files. Imports
  bound keys, file collections, histories and text; context validation ties
  mutable files, IDs and versions to their source. No learner data is passed to
  a filesystem, subprocess, network or actual editor.

Logical nonempty editor lines end in LF; the actual saved bytes remain distinct
until a write. Empty files and blank lines have different outcomes. A read-only
file may have an unsaved draft but cannot be written. Switching files refuses
unresolved changes. Reopening a closed file uses its saved virtual bytes.

## Outcome grading and persistence

Checks read saved files and require all buffers to be saved or clean. Settings
checks accept whitespace and key reordering while preserving unrequested keys.
Record checks accept legitimate line order changes but verify exact complete-line
counts and all other records. Duplicate/malformed settings, wrong copies,
missing records and unrelated changes fail. Feedback names expected/saved values,
record counts and dirty file paths. No prescribed keystroke string earns credit.

Editing a successful buffer invalidates its current check; movement/search alone
does not erase valid completion. The grade is always recomputed from current
state. Worked examples never complete assignments. Resetting the assignment
preserves separate practice, and resetting practice preserves the assignment.
Drafts, mode, cursor, undo, saved files and checks serialize in the new version1
session; existing scenario IDs and older progress fields are unchanged. Existing
445 scenarios and176 units compare exactly in the preservation regression.

Authoring data: `scenarios/authoring/modal-editing-v1.json`; the no-input development
example `modal_lesson_authoring` validates both examples and assignments and emits
`modal-editing-lessons-v1.json`. Native scenario initial/goal data is declarative;
`modal-lab` JSON in authored solutions is a simulator event format, not a Linux
command taught to learners. Bundled `boot-entry-lessons-v38.json` contains the
exact v37 prefix plus these three scenarios, retaining the128 bundled-file limit.
All imported packs retain existing external/bounded catalog restrictions.

## References and controlled comparison

Official LPIC-1 remains version5.0, objective103.8; checked2026-11-13:
https://www.lpi.org/our-certifications/exam-101-102-objectives/#rdp-we-103.8_Basic_file_editing

Primary Vim9.2 documentation checked2026-11-13, without copying implementation:

- https://vimhelp.org/usr_02.txt.html
- https://vimhelp.org/motion.txt.html
- https://vimhelp.org/change.txt.html
- https://vimhelp.org/editing.txt.html
- https://vimhelp.org/undo.txt.html
- https://vimhelp.org/pattern.txt.html
- https://vimhelp.org/insert.txt.html
- https://vimhelp.org/options.txt.html

The Open Group2024 vi page returned403, so no POSIX conformance claim is made:
https://pubs.opengroup.org/onlinepubs/9799919799/utilities/vi.html

`tools/dev.py tool modal_editing_reference.py` has three closed fixed profiles,
not an arbitrary source/key/path selector. Ten foundational,21 navigation/search/
register and8 insertion recipes were actually observed with pinned Fedora
`/usr/bin/vi` (Vim9.2tiny) in disposable private PTYs. Startup/configuration,
plugins, modelines, history and swap files are disabled. Time, CPU, address space
and output are bounded under MAIN's shared runner lock. No personal files or
other process are used. This tool never becomes learner execution.

Business writes and cursor measurement use separate identical-key runs; a
single-marker replay measures position without expression evaluation. Original
R10, R20 and R24 captures and their retained observer inputs remain immutable.
The current model matched all39 retained file/cursor cases in R27–29 before later
source integration. Those selected-source comparisons are not a full compiler
closure or independent seal; rebuild/readback is required for a new identity.
Failed observers and the undo-put regression are retained, not relabeled passes.

## Verification status and remaining work

R42:33 model/content tests pass, including alternatives, negatives, partial work,
actual progress save/load and exact predecessor preservation. R49:6 actual-widget/
lesson tests pass at five text sizes, two widths and registered light/dark themes.
R48:3 native-plan/capture-phase tests pass. R43 also passed12 matching pre-existing
modal guards. These checks are distinct from actual native screenshots.

R47 actual native completed the first lesson's typed practice/assignment,
feedback/retry, unsaved invalidation, saved-profile readback and reset, then failed
at the second lesson's worked-table capture scrolling. Partial evidence is kept
under `.build-artifacts/native-runs/run-7965e6gd`. Four originals were inspected;
duplicate numbering, an overlong worked value and excess empty editor height were
corrected. The capture-only parent-gutter, per-action attempt counter and late-
clipping checks are now regression-tested; the full actual route must be retried.

No new package, full native batch, physical-keyboard, screen-reader, independent
novice, Windows/macOS or complete103.8 acceptance is claimed here. Other editors,
EDITOR integration, full vi syntax and other remaining issuer items still need
an honest coverage decision. No installation replacement or public release.


### Follow-up gates and rendered evidence

R51 actual native completed111actions/30captures; full three-screen originals for
all3lessons and representative worked/light150/dark200 views inspected under
`native-runs/run-a_hds1l_`. R47 remains a failed partial run. R59passes35 model/
content tests; R74passes10 actual-widget tests, now checking drawn focus outlines
across all28themes and valid long-line/oversized-paste behavior. Input events move
rather than clone arbitrarily large clipboard text. The editor frame stays on
its viewport; glyph-advance metrics and immediate local caret scrolling keep long
pastes aligned/visible. These later visual corrections await packaged native
recheck. R75inclusive Python1252 and strict inventory/coverage pass. Library Clippy
passes; remaining final gates/package still pending. Other-editor awareness and
standard EDITOR configuration retain Open status; no full103.8 claim.


Final source gates: R82one-job all-target Clippy, R83normal UI4, R85–87current39
fixed comparisons, R88exact authored-pack reproduction and R89inclusive Python1252
pass. R74UI10 includes actual drawn focus/cursor contrast in all28themes; large
valid/invalid pastes, stable viewport frame, glyph advances and immediate caret
scrolling are checked. A new native insert-focus view joins the final packaged
route (114actions). That package/installed route is still pending at this source
freeze; no result from the older445 artifact is relabeled448 acceptance.


## Final packaged milestone — 2026-11-13

The separate448/177 candidate is verified. Protected review launch:
```sh
/path/to/dlms_learning_lab/dist/teaching-review-20261113T125002Z/open-modal-review.sh
```
This wrapper sets only its child XDG_DATA_HOME to a fresh ignored review location;
it does not overwrite the installed app, original progress or earlier review
profile. Its sibling candidate and archive remain immutable.

Packaged content validation and private445→448 lifecycle pass with exact synthetic
progress retention. The actual installed ELF completed114fixed native actions and
31captures in `native-runs/run-0r6dcb9g`, including later focus/viewport corrections.
Inspected final originals include:

- [Introduction](../.build-artifacts/native-runs/run-0r6dcb9g/screenshots/workspaces/modal-0-introduction.png)
- [Focused practice editor](../.build-artifacts/native-runs/run-0r6dcb9g/screenshots/workspaces/modal-0-insert-focus.png)
- [Independent assignment](../.build-artifacts/native-runs/run-0r6dcb9g/screenshots/workspaces/modal-1-assignment.png)
- [Recovery practice](../.build-artifacts/native-runs/run-0r6dcb9g/screenshots/workspaces/modal-2-practice.png)
- [Light150](../.build-artifacts/native-runs/run-0r6dcb9g/screenshots/workspaces/modal-2-light150.png)
- [Dark200](../.build-artifacts/native-runs/run-0r6dcb9g/screenshots/workspaces/modal-2-dark200.png)

ELF SHA256 `ce4fe62b343610b448a0242b7511f79e2196a1a7ba8bce472bd784ee1b99cbb9`;
archive `7486bc8cceec03cf8fe25b77d797499c3fc3d67b05b2f5a36f442dc389d3d97d`.
Final source/data/docs/ELF/archive readback and selected2878-file retention pass.
These are selected inputs, not full compiler closure or independent sealing.
No new inclusive Rust runtime, two-process modal GUI reopen, real narrow-native,
physical keyboard, screen-reader, independent novice or Windows/macOS acceptance.
The fresh launcher has the same verified ELF; ordinary manual learner launch is
separate from the fixed installed native route. Standard EDITOR configuration,
other-editor awareness and broader vi remain explicit roadmap work.
