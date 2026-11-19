# Offline filesystem consistency and retained-data recovery

Source/native development verified2026-11-04; packaged Fedora verification passes. Two distinct
LPIC-1 tasks are registered in `linux-filesystem-consistency`, after caller-aware
removable mounts and with hard-link identity as an explicit prerequisite:

- `lpic1-fs-offline-accounting`: inspect the supplied associations and named files,
  correct link/allocation/summary disagreements offline, preserve report bytes
  and blocks, restore its mount, and leave a separate volume exactly unchanged.
- `lpic1-fs-retained-recovery`: distinguish automatic corrections from the explicit
  decision to reconnect nonempty unnamed data; inspect actual lost+found names,
  preserve retained content/blocks, and restore access without changing a sibling.

Each has a separately authored workshop on different device/mount identities,
three deterministic assignment cases, short concepts before worked commands,
and guided actions beside a real in-process simulated terminal. The independent
assignment does not insert its recipe. A fixed command picker is available only
in practice; Insert fills the draft and does not execute anything. Check work
runs the current actions across each supplied sample, not a command-string match.
Supported alternative flag spellings and `fsck.ext4` aliases must pass.

Every case checks actual graph relationships, named/unnamed file preservation,
mount state and the protected sibling. Diagnosis alone, mounted repair, wrong
volume, incomplete recovery and a missing final mount must not pass. A successful
state-changing action clears current results immediately; harmless queries and
editing an unexecuted command draft do not undo actual state. Failed commands
are reported and cannot mutate the graph. The intentional starter faults remain.

The native console distinguishes unsupported syntax, invalid arguments, unknown
virtual devices, safety policy and execution status. e2fsck status1 means corrected;
4 means findings remain;5 means corrections plus remaining findings. This is
separate from the overall assignment result. A single completion panel reports
case/objective results and a worked explanation after success.

Progress records separate practice/assignment command drafts, bounded transcript
and successfully changed action history. Imported history is source-bound and
validated; runtime caches and grade proofs never deserialize. A saved checked
stamp only requests fresh cooperative verification, which may be stopped. It
cannot grant current credit alone. Reset replaces only the selected attempt.
Original lab IDs, objectives, data and old completion records remain untouched.

This is narrow supporting practice for LPIC-1 5.0 objective104.2 and LPIC-2 4.5
203.2. It is not full filesystem maintenance, actual recovery experience or a
claim of exam preparation. Capacity and partition preparation remain distinct.
See [the supported model and commands](FS_INTEGRITY_SUBSET.md) for exact bounds,
strict mounted-check policy, excluded damage and primary references. No real
filesystem utility has been run as a reference in this batch; conformance claims
are limited to the disclosed, deliberately smaller model. Third-party GPL
problem classifications were read as behavioral references only; no code/message
tables were copied or redistributed.

## Verification checkpoint

32unique focused core/adapter/runner/session/authored groups pass. All prior265
scenario values and78unit objects/order remain exact; the new unit follows BOTH
its file-identity and caller-mount prerequisites.30retained backup/progress/storage
groups also pass.5native widget/controller groups cover real Enter/Run, inert
insertion, separate practice, wrong/partial/correct states, Stop, reset, saved
drafts, actual re-verification and100/150/200% widths/light-dark layouts. The
controller regression additionally executes every pinned recipe expectation
against both complete source fixtures before capture.

Development Fedora run-f5hh17bs completes77native steps and28PNG. All six
Introduction/Try example/Assignment views, specific missing-mount feedback and
narrow light200controls were opened and visually inspected. That review shortened
repeated recovery prerequisites into one explicit recap and corrected the picker
instruction's direction; final packaged shortened recovery Introduction, corrected practice guidance, independent Assignment, partial feedback and light200controls were inspected. Source
admission first caught tagged-enum numeric-key decoding; bounded canonical keys
fixed it. Native r1/r2 caught controller expectations leaking between tasks and
assuming corrections before an orphan refusal. Static expectations were corrected
and fully regressed; the simulator correctly returned4. Neither case was repaired
by accepting a range of statuses or changing grading. Failed logs/captures remain.

Final default all-target Clippy passes26.94seconds and649Python checks pass
11.812seconds after the content/controller corrections. The separate267/79 package validates content and all77native steps/28PNG.
521source/data hashes and50offline guides matched before these post-build notes.
No new whole-app inclusive suite, installation lifecycle, full-process cold
reopening or actual filesystem-tool comparison is claimed in this narrow batch.
Unit/save/read tests do not substitute for a process-exit acceptance flow.

Original installed app/progress and the verified265/78 candidate are unchanged.
No installer replacement, dependency installation, commit/push/merge or release.
Windows/macOS/Omarchy runtime, screen reader, physical-device accessibility and
independent novice comprehension remain unverified. Worked-answer review is not
independent novice acceptance.


## Separate verified Fedora review build

`/path/to/dlms_learning_lab/dist/teaching-review-20261104T180204Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab`

BinarySHA256 `c1f79797048494549215eafac02aba688337fa372cf9697c6e462474d02a10f5`; archiveSHA256
`bd5b0d1c6b3a4e948551d375260cf2820006d0a1cf8cda19da449904ab1036a7`. Build380memory samples,min11.944GiB available,
PSIavg10max0,OOMdelta0,finalswapfree.911GiB. Guarded two-job workflow is not a
memory guarantee. Native evidence: run-q04etx8l,77steps/28PNG. This package embeds
protected review progress; no installed-app replacement. All prior265scenario
values/78unit objects remain exact. Detailed immutable private receipts are in
fs-integrity-stage. Documentation links below were recorded after packaging.

[Introduction](../.build-artifacts/native-runs/run-q04etx8l/screenshots/workspaces/teaching-lpic1-fs-retained-recovery-introduction-dark-100.png),
[Try example](../.build-artifacts/native-runs/run-q04etx8l/screenshots/workspaces/teaching-lpic1-fs-offline-accounting-example-dark-100.png),
[Assignment](../.build-artifacts/native-runs/run-q04etx8l/screenshots/workspaces/teaching-lpic1-fs-retained-recovery-assignment-dark-100.png).
