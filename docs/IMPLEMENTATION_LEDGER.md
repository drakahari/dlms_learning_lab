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
