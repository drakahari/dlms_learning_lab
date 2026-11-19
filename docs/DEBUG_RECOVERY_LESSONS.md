# Bash debugging and checked recovery projects — 2026-11-02

Two substantive repair assignments follow exact snapshot comparison and loop
recovery. They reuse evaluator15 and the approved editor/output/feedback workspace.
All229 preceding lab/61unit objects remain exact. Source review catalog231/62:
69CCNA,56LPIC-1,40LPIC-2,66Bash/Shell. Lab counts are not objective completion.

- `shell-debug-status`: repair a script that contaminates stdout, loses a filter’s
  status after reporting, and splits captured text. quiet/trace must retain the
  same business output and investigation status. Trace is explicitly manual
  diagnostic output on stderr, not Bash set-x. The different worked example saves
  false’s status before printf; practice deliberately moves the capture too late.
- `shell-recovery-ledger`: reconcile supplied completed work with exact outputs,
  reject inconsistent checkpoints/untracked outputs, preflight the whole request,
  then preview or finish pending jobs. The intentionally flawed starter writes too
  early and trusts substring matches. This capstone combines prior validation,
  line reads, Bash arrays, sentinel capture, statuses and virtual file operations.
  Runs start with controlled prior work; no interrupted OS execution is resumed.

Each project has12 graded input sets (the unchanged interpreter maximum).16
additional boundary fixtures preserve40 total checks, not extra authored labs.
All complete stdout, stderr, status and virtual file maps are checked. Grading is
behavior-based: alternate diagnostic guards and a recovery implementation that
uses a pending count instead of a pending array pass. Incorrect stream/status/
quote handling, hardcoded first outputs, unconditional apply and unvalidated
checkpoint output fail. Preview is read-only; all stated errors preserve every
file. A second valid apply has no pending work and preserves resulting file bytes.
No claim is made about counting identical writes or testing real crash atomicity.

Introductions teach the reasoning and show a runnable example before the full
assignment. Practice changes have an explicit action and observation. Example
source/output stays separate from graded drafts. Challenge retains the complete
assignment without unsolicited solutions. Optional hints help diagnose mistakes;
full reference scripts are hidden until requested. Existing presentation, themes,
keyboard behavior and scope limits are preserved.

## Technical limits

No new language semantics, dependency or evaluator revision. Arrays are explicitly
Bash-specific; status/quoting/streams are portable shell concepts within the
existing disclosed subset. No trap, set-x, signals, real processes, locks,
permissions, atomic rename, nested filesystem, concurrent writers, external
network or host filesystem execution. Stop discards the current grading run;
it does not create a real interrupted-system recovery case. Those broader
roadmap capabilities remain open, with no inherited mastery credit.

## Verification

`debug-recovery-integrated-r1.log`:8 focused engine/content/persistence/catalog
groups pass:40 full behaviors for references and legitimate alternatives,
small instruction slices, bounded Stop, negative repairs, separate examples,
stale-success invalidation, draft save/read/reset, exact229 prefix preservation
and repeated apply. Both source starters are intentionally failing. The first
content load refused16/24 cases above the unchanged12-case limit; supplementary
fixtures now retain every original boundary assertion rather than raising limits
or discarding tests. Original failed logs are preserved.

`debug-recovery-ui-r2.log`:3 focused actual-widget/paint/layout groups plus1
capture-phase regression pass. These include example change/retry, starter
failure, successful editor run, editing invalidation, Stop, retry and save/read,
100/115/125/150/200% at760/1600 widths in light/dark themes. The diagnostic
paint test was corrected to check **wrapped** prose; a one-line legacy heading’s
unused justification flag is not evidence of stretched word spacing. Existing
assertions/grades were not weakened. Real native screenshots remain pending.

### Controlled real Bash comparison

New fixed developer-only `tools/debug_recovery_reference.py` admits exactly two
SHA-pinned authored recipes and40 fixed synthetic cases, no caller-supplied
source/profile/path. MAIN runner/shared lock is required. Each reference child
has a fresh disk fixture directory, stripped environment, bounded time/CPU,
address space, file size and output, and inherits the runner lock. It never reads
personal files or executes learner scripts, and is absent from runtime execution.
The helper compares real Bash stdout/stderr/status/files with the declared
outcomes independently exercised by the in-process tests. No historical shell
collector, probe, baseline or sealed observation was rerun or overwritten.

Actual40-case run passed; retained evidence:
`.build-artifacts/reference-runs/debug-recovery-ntf2y3b8/report.json`.
`debug-recovery-bash-reference.log` names that exact run. This is authored-project
comparison, not a new evaluator seal or general Bash-fidelity guarantee.
GNU Exit Status, Redirections and Arrays online pages timed out on2026-11-02;
the installed upstream `/usr/share/doc/bash/bashref.html` sections were read,
and actual fixed Bash comparisons supplied the scoped execution evidence.
Primary online URLs remain in each lesson for reference.

Separate Fedora candidate/native checks are pending at this source checkpoint.
No full suite, cold reopening, installation/upgrade, Windows/macOS, screen-reader,
physical-device or independent novice-comprehension acceptance is inferred.


First231 package: content231/62 validates,416 source pins, binarySHA256
`a5185fd3f924f2b5314a8ce3cc560981b5e7149912fcc932f6b22b7cd510bbc5`.
Its first native run stopped at a capture-focus assertion after scrolling to
starter feedback (two retained screenshots,run-6njth3wp). The failed assertion formed a full-width band from the editor viewport after
scrolling; its exact rectangle was not logged. Empty/thin and fractional bounds
were unhandled. This is an incomplete native run, not acceptance. A project-local
focus-band/parent-scroll fix and empty/thin/fractional regression are in progress;
old generic focus helpers remain unchanged. See CHECKPOINT.md for exact next gates.


The scoped focus-band fix passes5 UI/capture regression groups and the actual
development native flow: both examples/change/retry, starter failure, full
solutions, edit invalidation, Stop, actual save/read, retry and reset.24 actual
PNGs run-dv04k6b6; Introduction, full recovery Assignment and specific failing
cases/overall failure inspected. Production/new-test Clippy-r2 and86 focused
Python-r2 pass. A fresh optimized candidate is building; its packaged native
acceptance remains pending. Development observations are not relabeled as that
new binary’s acceptance.


## Corrected packaged Fedora verification

Separate review `dist/teaching-review-20261102T155736Z/`, binarySHA256
`ba4dcdb4a47cfa04e8e27b25dfb0e1230c42517b1256508e0b7352ec2b557437`;
archiveSHA256 `bb20ca3c6862a8ab33dc60aee56307673c29e196caa0570f3fd311f25627c1e4`.
416 source pins. Packaged231/62 content validates. Actual corrected native run
passes both examples/change/retry, starter failure, full solutions, edit
invalidation, Stop, actual save/read, retry and selected reset.24 PNGs retained
in `native-runs/run-2vroneo3/screenshots/workspaces/`. Inspected the recovery
Introduction, diagnostic guided practice, full recovery Assignment, specific
starter-failure feedback and narrow light200% Run/Stop/output controls.
This supersedes pending package status above, not the retained failed first run.

- [Introduction](../.build-artifacts/native-runs/run-2vroneo3/screenshots/workspaces/teaching-shell-recovery-ledger-introduction-dark-100.png)
- [Practice](../.build-artifacts/native-runs/run-2vroneo3/screenshots/workspaces/teaching-shell-debug-status-example-dark-100.png)
- [Assignment](../.build-artifacts/native-runs/run-2vroneo3/screenshots/workspaces/teaching-shell-recovery-ledger-assignment-dark-100.png)
- [Failure feedback](../.build-artifacts/native-runs/run-2vroneo3/screenshots/workspaces/debug-shell-recovery-ledger-starter-feedback.png)
- [Light200% controls](../.build-artifacts/native-runs/run-2vroneo3/screenshots/workspaces/teaching-shell-recovery-ledger-assignment-light-200-controls.png)

The5 scoped UI/capture tests,86 focused Python checks and production/new-test
Clippy-r2 pass after the fix. No installed app or original progress was changed.
No inclusive-suite, new lifecycle/cold reopening, Windows/macOS runtime,
screen-reader/physical-device or independent novice acceptance is inferred.
Continue the established Linux logging gap; no routine approval stop.
