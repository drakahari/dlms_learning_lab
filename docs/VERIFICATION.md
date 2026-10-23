# Current completion work

The sections below retain historical scoped verification. Current milestone evidence is maintained in [IMPLEMENTATION_LEDGER.md](IMPLEMENTATION_LEDGER.md). As of M37:321 Rust tests;64 authored labs/fifteen units; six authored IPv4 static-routing lessons and the original subnet/VLSM and single-service teaching flows have native Fedora keyboard/retry/resize review. Shared native control names have emitted AccessKit-tree checks; shared input geometry, actual click/selection and200% reflow have regressions. Worked-help reset/restore isolation has explicit checks. This is not screen-reader acceptance. Full curriculum, fine-grained objective mapping, assistive-technology review and actual Windows/macOS desktop acceptance remain open. Current notice/platform candidate gates are in [INSTALLATION.md](INSTALLATION.md). No release-readiness claim follows from these counts.

# Teaching improvement verification

Fedora 44 / COSMIC Wayland, 2026-10-21. This pass changes beginner teaching in the **existing seven scripting labs** and relocates theme samples. All sixteen assignments, scenario versions and behavior cases remain; no dependencies or interpreter features were added.

- **65 Rust tests passed**: 16 UI mechanics/layout tests, eight IPv4 tests, 22 prototype tests, four new teaching/content/compatibility tests, eight shell/theme tests and seven workspace tests. Offline locked build, formatting and Clippy (all targets, warnings denied) pass. Both declarative packs validate (9 + 7). All 28 palette inventory/hash/notice checks pass.
- All seven introductory examples and all seven suggested edits execute through the bounded VM. Tests cover separate example/assignment drafts and results, old optional-field omissions, completed work retention, saved selected view, reset, syntax/unsupported/limit diagnostics, output/newline differences, wrong/partial answers, equivalent supported scripts, edit invalidation, keyboard Run/Stop/Escape/Tab, theme persistence and large-text width/editor-space regressions.
- A **fresh synthetic-profile native walkthrough** opened every Introduction, ran every example, typed each suggested edit, opened each complete Assignment, ran an incorrect program, then retried successfully against all original cases. It asserted no example completion credit or assignment overwrite and reloaded progress successfully. Rendered captures were reviewed, including all seven introductions and the first lesson's practice/assignment/retry views. Review caught and corrected an undersized example editor that behavior tests had not caught.
- Native captures were 2560×1440 full-screen and 1268×632 tiled on this desktop. DLMS Light and Omarchy catppuccin were inspected at 150% text. A separate native scroll pass reached Introduction actions and example/assignment Run controls in the tiled window and ran the example there. Larger text/smaller windows use outer scrolling rather than shrinking text or clipping the controls. Changing teaching views returns to the top without changing either draft.
- Theme samples now live in **Settings → Appearance → Theme samples**, collapsed by default and explicitly labeled examples. Toolbar selection still applies immediately and saves independently. All 28 theme roles, selected text/focus/disabled semantics and source notices retain automated checks; every theme was not recaptured natively in this teaching-only pass.

Native input was injected through the application's event path. This is an agent walkthrough/render review, **not beginner usability acceptance from a novice user study**, physical-device testing or assistive-technology acceptance. Windows/macOS runtime status remains untested. The existing real-Bash reference suite was **not rerun in this pass**: no learner or teaching script was executed by a host shell; example execution and grading use only the in-process simulator. Interpreter changes only clarify diagnostic categories.

Local ignored screenshots under `screenshots/workspaces/`:

- `shell-handover-introduction.png`, `shell-handover-example.png`, `shell-handover-assignment.png`
- `shell-handover-retry-feedback.png`, `shell-handover-teaching-complete.png`
- `shell-<lab>-introduction.png`, `shell-<lab>-example.png`, `shell-<lab>-assignment.png` for all seven
- `shell-introduction-light-150-scrolled.png`, `shell-example-light-150-output.png`, `shell-assignment-dark-150-scrolled.png`
- `settings-appearance-theme-samples.png`

Reproduce the fresh teaching walkthrough with `DLMS_SMOKE_STAGE=teaching ./target/debug/dlms-learning-lab --ui-smoke`; use `DLMS_SMOKE_STAGE=teaching-layout` for the focused tiled/150% scroll check. Both use fresh isolated `/tmp` data, never the normal learner profile. The [teaching audit](SCRIPT_TEACHING.md) documents prerequisites, authoring and compatibility.

The preceding milestone's broader matrix/reference evidence is retained below as historical evidence; it was not all rerun for this scoped change.

# Prior milestone record

Fedora 44 / COSMIC Wayland, 2026-10-21; Rust 1.99.1, pinned egui/eframe 0.33.3, native Glow renderer. Baseline: accepted commit `89f2c0a`. No added dependencies, system packages, installers, releases, desktop-theme changes or other-project writes.

## Automated and reference checks

- **57 Rust tests passed:** 12 native-UI mechanics/layout tests, eight IPv4 mathematics/compatibility tests, 22 engine/content/catalog/progress tests, eight shell/theme tests, seven workspace/migration/editor tests.
- Offline build, `cargo fmt --all -- --check`, and Clippy for all targets with warnings denied.
- Original nine scenarios plus seven authored scripting scenarios validate: initial states unsolved, references reach every objective, identifiers/metadata/state bounded, prerequisites valid and acyclic. Invalid script cases and unreachable reference programs are rejected.
- `tools/verify_palettes.py`: all five DLMS stock palettes, all 22 exact Omarchy files/fields/SHA-256 values and MIT notices verified. The official immutable source archive's complete color-file inventory was independently compared during import.
- `tools/bash_reference.py`: **41 controlled development fixtures**, zero stdout/stderr/status/virtual-file differences against **GNU Bash 5.3.9**. Seventeen language edge fixtures plus all 24 authored unit test cases. Cases cover quotes, empty/spaced arguments, concatenated expansions, read/EOF, exit status and conditional lists, loop status, pipeline variable isolation and status, truncation/append/input redirects, descriptor copies and file names with spaces. Real Bash ran only these checked-in fixtures in disposable temporary directories with a controlled environment and two-second timeouts. No learner script or personal file was passed to Bash.

The language oracle compares the declared subset; it does not establish Bash/POSIX conformance. Unsupported syntax and host paths/commands fail honestly. Tests cover finite instruction/output/memory bounds, Stop, resumable limit-failure records, alternatives, wrong/partial scripts, source-edit invalidation, reset isolation, old-nine saved progress and new script-draft round trips. Existing 500 generated IPv4 numerical cases remain mathematical tests, not new curriculum.

Every palette's derived text, input, panel, terminal, control, hover, primary/selected text, error/success and boundary roles have explicit contrast checks. Native UI tests render each theme's focus/selection/disabled samples. A separate native focus outline avoids egui's shared selection-foreground/focus-border role making light-theme editor focus invisible. Disabled controls are labeled unavailable/idle, and feedback uses words as well as color.

UI tests cover all 16 labs in Learn/Challenge at 1200×850 and 720×850, 100% and 150% text; original nine also retain 720×560 checks. They cover library search/topic/certification/favorites/resume/optional filters/pagination and 900 synthetic catalog entries, aligned fields/buttons, editor input, Ctrl+Enter Run, Escape/Tab focus, terminal Enter/Tab/Shift+Tab, Stop, completion invalidation and selected-attempt reset. These verify mechanics and geometry, not learner comprehension.

## Native Fedora rendering and input

The normal application's `--ui-smoke` harness used fresh synthetic data under `/tmp/dlms-workspaces-smoke`, never actual learner progress. It captured the original nine labs in Learn/Challenge, all 28 native theme state previews, each of seven scripting workspaces in Learn/Challenge and completed state, source editing through injected keyboard events, actual native Run/Stop button interactions, state invalidation, and light/dark 150% text after a real full-screen/tiled transition. Captures use 1920×1080 and the compositor's 946×929 tiled window on this desktop. Dimensions may differ on another desktop.

Actual captures were inspected for readable text, selected text, focus, disabled controls, errors/success, task prominence, editor space, button alignment and scrolling. Visual review caught the focus-outline and short-editor issues; the final capture pass includes those fixes. Screen captures are local ignored artifacts under `screenshots/workspaces/`, including:

| View | Capture |
| --- | --- |
| Themes, all 28 | `theme-<saved-id>.png` |
| Script assignments | `shell-<lab>-learn.png`, `shell-<lab>-challenge.png`, `shell-<lab>-complete.png` |
| Safe-report capstone | `shell-capstone-complete.png` |
| Stop and invalidation | `shell-stopped-and-invalidated.png` |
| Resizing / larger text | `shell-light-150-percent.png`, `shell-dark-150-percent.png`, `shell-library-light-150-percent.png` |
| Existing nine / library | Prior capture names retained and regenerated by the same normal-app harness |

After the full matrix pass, `DLMS_SMOKE_STAGE=scripts ./target/debug/dlms-learning-lab --ui-smoke` regenerated the unit captures with the final readable reference examples. This focused pass skips the original-lab/theme matrix.

Input was injected through egui's native event path; this was **not** a physical-device or beginner usability study. Most alternate solutions and error paths are exercised through pure/UI tests rather than manually typed on the desktop.

## Persistence and public-tree checks

The original nine IDs, scenario versions, state models, optional workspace drafts and progress envelope remain compatible. Scripting adds a typed state variant; source and completed run snapshots save locally, interrupted jobs do not resume executing on launch. A new Run starts fresh fixtures. Editing invalidates the previous result. Reset clears only the selected attempt; theme, favorites and other attempts remain. Earlier pre-workspace backup behavior remains intact. Progress retains its envelope/version with a 16 MB read/write ceiling for bounded run records; oversized writes retain the original. Appearance has its own versioned atomic file; invalid/future files are retained and reported without disabling learner progress.

Generated screenshots, local preferences, learner progress/backups, toolchains, native captures and recordings stay excluded. Public diff checked for credentials and private paths/data; third-party copyright attribution is intentionally retained. Root license selection remains the owner's decision.

## Unverified and remaining limits

- **Windows/macOS were not built or runtime tested.** The architecture remains native and portable Rust, but cross-platform acceptance is not implied.
- No physical keyboard/mouse acceptance, screen-reader/AccessKit study, IME, high-contrast OS theme, multiple-monitor DPI or beginner usability study.
- Shell v1 is a bounded text interpreter with a flat `/work` file namespace, explicit builtins and no external commands. No functions, arrays, substitution, globbing, jobs, traps, full filesystem or production Bash behavior. Exact supported syntax/limits are in [SHELL_SUBSET.md](SHELL_SUBSET.md).
- No genuine networking, hardware, kernel, processes, encryption or complete certification coverage. Planned curriculum is explicitly separate from implemented practice.
- Small windows/larger text and optional help can require vertical scrolling; long code/output lines can scroll horizontally. Concurrent application writers and downgrade to an older binary after saving new scripting attempts remain unsupported.
