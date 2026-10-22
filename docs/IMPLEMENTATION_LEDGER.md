# Implementation ledger

Completion authority: [COMPLETION.md](COMPLETION.md). Every milestone records changes, actual evidence, open failures and the next implementation work. No automatic closure based on lab count.

## 2026-10-22 — M1: wording and completion baseline

- Started from `69f695d`; accepted native workspace and teaching pattern retained.
- Replaced “Read the contract” with “Understand the task.” Sample-input wording replaces “fresh virtual workspace”; explains simulated file resets and retained drafts. Replaced learner-facing “fixtures” in optional walkthrough steps/hints.
- Rechecked official Cisco v1.1/v2.0 and LPI 5.0/4.5 sources; check date recorded in curriculum documentation. Current and upcoming blueprints remain separate.
- Added the completion standard with explicit incomplete coverage, simulator, UX, recovery, installer and platform acceptance gates.
- Validation: 65 existing Rust tests pass. This is a wording change, not additional curriculum or a claim of native/release acceptance.
- Independent baseline review started. A permissions privacy grading defect is queued for immediate repair; broader content and simulator work remains open.

Next: resolve review findings; implement safe progress backup/export/restore and course/unit navigation; extend bounded simulator capabilities and author reviewed curriculum batches. License and actual Windows/macOS acceptance remain owner/environment decisions, not reasons to stop independent implementation.

## 2026-10-22 — M1b: owner-approved license

The owner explicitly approved MIT for original code and authored curriculum, using “DLMS Learning Lab contributors.” Added root LICENSE/NOTICE and package metadata; retained palette attribution. This resolves the project-license decision, not the remaining dependency-notice or platform-acceptance gates. No public release.

## 2026-10-22 — M2: reviewed grading and persistence integrity

- Independent read-only review reproduced long-command diagnostics that made saves unreadable, exclusive-file privacy bypass, legacy host invention, concurrent-writer loss and ambiguous utility operands.
- Diagnostics now truncate on UTF-8 boundaries at creation; invalid in-memory state cannot replace a valid save. Regression covers assignment/example failures from long ASCII and Unicode sources through save/reload.
- Alex-only privacy now explicitly checks all modeled non-administrator users as well as other-class bits. Team traversal retains its distinct shared-access goal. Feedback identifies an unauthorized modeled user.
- New host requirements stay blank on migration. Historical analysis credit is displayed separately; values supplied by an older binary remain intact but need an explicit new check. Review/check persists, and broken answers invalidate current success.
- Ambiguous option-like grep patterns/file operands and cat operands fail honestly. Malformed numeric tests explicitly disclose the current unsupported boundary rather than claim faithful recoverable Bash behavior.
- Regression: 71 Rust tests, including old/new legacy UI flows, invalid-save retention, alternate privacy solutions and error-record roundtrips. Disposable 41-fixture Bash reference comparison is rerun. Full certification coverage remains open. Concurrent-writer protection follows in M3.

## 2026-10-22 — M3: progress recovery and writer protection

- Added native Settings → Your data, folder/file chooser, new-file-only backup export, cancellable bounded background inspection, explicit summary/confirmation and atomic restore of the inspected candidate.
- Exact original-file safety backup precedes replacement. Dirty writable sessions save first; edited read-only sessions receive a separate backup. Reserved application filenames, symbolic/special files, unknown/future/oversized data and forged script outcomes have explicit checks.
- OS writer lock is acquired before loading; a second copy cannot write progress or preferences. Lock files remain in place; OS ownership releases on exit/crash. Post-commit directory-sync failures report a durability warning instead of claiming the old file was retained.
- Independent read-only review exercised backend safety and found reserved-filename, unsaved-session and post-commit reporting edge cases, which were addressed. Oversized-current-file recovery deliberately requires a manual safe move, documented in PROGRESS_RECOVERY.md.
- 82 Rust tests pass, including seven backup tests, a controlled post-commit sync failure, two native UI data flows, background-shortcut isolation and prior regressions. Native Fedora capture pass exercises export, typed import filename, preview, confirmation, draft/favorite recovery, legacy disclosure and light 150% Settings. Captures remain ignored local artifacts; no personal profile is used.
- Full curriculum and cross-platform installer/runtime acceptance remain open; proceed to unit navigation and reviewed capability/content batches.

## 2026-10-22 — M4: unit navigation and scalable bundled authoring

- Added a versioned unit index covering every existing assignment, advisory prerequisite guidance and course entry points. Unit/topic controls share consistent constrained widths and wrap cleanly; track/search/favorites/resume remain available.
- Completion continues within the same unit and offers earlier unfinished work; it cannot silently jump to another certification. New scripting learners enter the approved introduction; experienced learners can open any assignment.
- Added bounded multi-pack validation with global dependency checks, plus `--validate-content`. No executable packs, plugin platform, dependencies or extra curriculum titles were added in this milestone.
- Validation: 88 Rust tests, Clippy, bundled validation; native Fedora course start/return, unit/prerequisite screens, completion, 900-item synthetic filtering and light 150% rendering. Visual review confirmed baseline alignment and readable wrapping.
- Implemented navigation foundation, not full curriculum coverage. Next is the authored shell composition/debugging unit with independent content review and per-lab alternate/negative behavior tests.

## 2026-10-22 — M5: reviewed shell composition unit

- Authored four distinct practical assignments: route output channels, intersect log filters through a pipeline, repair EOF handling and audit batches without hiding failures. Eleven shell assignments now sit in two coherent units; the original nine certification seeds remain unchanged.
- Every new concept has prerequisite guidance, a skippable introduction, separate runnable example and suggested experiment. Assignment/example drafts and credit stay isolated. The documented shell subset did not expand and no dependency was added.
- Independent content review found missing quiet-log and literal-substring cases, ambiguous EOF trimming wording and guidance omitted for file/status-only failures. Added explicit cases/negative tests and corrected feedback. All eleven shell assignments now have tested alternate solutions, plausible wrong solutions and case-1 overfit rejection.
- Validation: 92 Rust tests, 83 controlled Bash 5.3.9 comparisons with no differences, Clippy, formatter, bundled validation (20 labs/5 units). Automated layouts cover both modes, 720/1200 widths and 100/150% text. Native Fedora synthetic-profile pass traversed all four introductions/examples/edits/assignments/incorrect retries/correct completion, plus light/dark larger-text scrolling; actual screens reviewed. No personal profile, host learner execution, Windows/macOS acceptance or novice-user study.
- Curriculum map remains explicitly incomplete. Next extend functions/return/argument handling with faithful bounded evaluation before authoring reusable-interface lessons; complete certification models/content and installation candidates remain open.

## 2026-10-22 — M6: bounded reusable shell capabilities

- Extended the existing lexer/bytecode VM with portable brace-body functions, execution-time definitions, restored positional arguments, return/implicit status and shift. Global scalar behavior is explicit. Calls use a cooperative VM stack rather than Rust recursion; depth, retained argument/loop memory and instructions are bounded.
- Documented the subset before dependent content. Whole-function redirects and pipeline function calls remain honest unsupported diagnostics; ordinary return redirection is supported, including quoted/expanded names. No dependency or host-execution route added.
- Independent review exercised 36 additional disposable Bash comparisons and found an incomplete reserved-name exclusion and a documented return-redirection restriction bypass. Centralized reserved-name rejection; deliberately supported/tested real return redirection and reconciled documentation.
- Validation: 99 Rust tests, 102 checked-in Bash 5.3.9 comparisons with zero differences, Clippy. Tests cover nested calls/loops/definitions, argument restoration, shared scalars/input, status, excessive shift, unsupported contexts, recursion/memory/instruction limits and Stop. Existing native workspace UI test runs a function-based handover, invalidates it, stops a looping function and retains its draft. No new native-screen or platform acceptance is claimed for this evaluator-only milestone.
- No new authored lab or full functions-course coverage is counted yet. Next author reusable-function/return/argument lessons using this validated subset, then expand remaining capabilities and certification practice.

## 2026-10-22 — M7: reusable-interface curriculum

- Authored four reviewed assignments: reusable labels, item-level return vs process exit, consuming leading arguments and a validate-before-writing bundle capstone. Total: 24 authored assignments, six units; numerical/generated variants remain excluded.
- Each lesson introduces prerequisites and new concepts before assessment, with separate editable examples and suggested changes. Behavior grading accepts equivalent supported inline programs as well as helper-based solutions; no mastery is inferred from source shape.
- Independent review found an append-before-preflight false positive when the destination did not already exist. Added an absent-output failure case and explicit regression; clarified filename restrictions and aligned the exact suggested example edit.
- Validation: 100 Rust tests, 148 controlled Bash 5.3.9 comparisons, Clippy, formatter and bundled validation. All fifteen script assignments now have alternatives/plausible wrong/overfit checks. Four new assignments receive partial-completion and stale-result coverage. Native Fedora synthetic flow traversed each introduction/example/edit/assignment/retry/completion, with light/dark 150% scrolling; actual rendered examples, assignments and resized states reviewed.
- No Windows/macOS runtime or independent beginner acceptance is claimed. Complete certification coverage, advanced shell scope/expansion/structured-data/utility models and installation candidates remain open. Continue independent work; no release publication.

## 2026-10-22 — M8: installation and notice foundations

- Added local native candidate tooling with source/compiler/OS identities, embedded-content/binary hashes, archive checksums and no release publication. Original MIT approval retained. The target graph bundles original notices, font licenses, Rust library notices and checksum-verified original dependency archives, including MPL covered source. Public upstream notice cache records exact revisions and hashes.
- Linux per-user installer/launcher/menu and uninstall protect separate progress. Independent review found intermediate-link deletion, dangling desktop entry, overlapping-root/field-code, Windows shortcut and collector target-label issues; all received fixes. Twelve Python fixture tests cover install/upgrade/uninstall, retained progress/user files, unrelated shortcuts, link boundaries, malformed manifests, ignored-resource exclusion notice tampering, forced native-target provenance and exclusion of uncompiled local JSON.
- Fedora development candidate was extracted/installed under an isolated /tmp root, validated with desktop-file-validate and opened outside the repository. Native course navigation and light 150% rendering pass; progress sentinel retained. 100 Rust tests continue to pass. This dirty-source development check is not the final reviewed-source candidate identity.
- Actual Linux target has 223 normal/build dependencies and Windows has 137, with notice checks clear. Complete-lock optional hexf-parse and linked macOS dispatch original notices are unavailable; explicitly reported, not guessed or silently dropped. macOS packaging stays gated; source compile/test CI is separate from desktop acceptance.
- Manual native CI workflow is prepared for Ubuntu/Windows/macOS; runtime, signing/notarization and owner installation acceptance remain open. Actual dispatch/results and clean candidate identity will be recorded after the reviewed source commit. Complete certification/advanced-shell curriculum remains open; continue independent capability/content work.

### M8b: clean Fedora identity and first native CI findings

Clean candidate source `c164580b5cc9b5d856279e14a4890f60e01686ad`: Fedora x86_64 archive SHA-256 `aab299d3e9a64052ee81194290c93167a322a1cc065af37c547372cc3a5db045`. Clean candidate upgraded the isolated installation, validated its desktop entry, retained the progress sentinel and passed native course flow outside the repository. This remains milestone installation evidence, not complete curriculum/desktop acceptance.

Manual CI run `39792196866`: Ubuntu build/tests/candidate succeeded; Windows and macOS Rust/Clippy checks passed, then candidate-resource tests failed because fixture roots retained native temporary-directory aliases while production ROOT is canonical. Canonicalized fixture roots without weakening containment/link guards. Rerun results follow separately; no Windows/macOS desktop acceptance inferred. No public release.
