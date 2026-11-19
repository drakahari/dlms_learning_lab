# Filesystem capacity: blocks, inodes and safe cleanup

Implementation in progress2026-11-02; four real source-registered lessons,
not yet a verified packaged milestone. Catalog/source counts and evidence belong
in CHECKPOINT.md and CONTINUATION.md. No full exam-objective credit follows.

## Progression

Advisory prerequisite: `persistent-mounts`. Track: LPIC-2. Unit:
`linux-capacity-handover`. Stable new IDs:

| Lab | Practical assignment | Legitimate choices |
| --- | --- | --- |
| `lpic2-capacity-inodes` | Diagnose inode exhaustion; create a result while retaining space for future files | Any three of five expendable queue names; more cleanup can also satisfy the stated task |
| `lpic2-capacity-blocks` | Budget a file that crosses a block boundary and leave ordinary headroom without spending the reserve | Resize an expendable cache, remove it, or combine safe cleanups |
| `lpic2-capacity-snapshot` | Grow an active file, create a manifest, and repair a stale cleanup projection | Safe operation orders pass without deliberately producing an error |
| `lpic2-capacity-handover` | Preserve source/audit, create a bundle and empty receipt, leave both resources available | Multiple expendable-file selections, provided both final budgets pass |

Each Introduction teaches the relevant concept and a worked example with different
files, lengths and block sizes. Separate Try example operates on its own virtual
filesystem. Prepare worked action fills only the editor or selection: it does not
apply, delete, check or earn assignment credit. Actual practice requirements are
checked after the learner applies the action. The Assignment keeps all independent
requirements visible and its worked approach collapsed. Challenge opens the same
meaningful Assignment without unsolicited guidance.

## Native interaction and outcomes

Current-capacity rows distinguish total, used, free, reserved and available blocks,
plus free inodes. The file table shows logical bytes and rounded data blocks. Edit
loads a file-size draft; Apply file length creates/resizes a virtual file. Empty
files consume an inode. Preview removal shows the actual selected names and
projection. Apply removal uses the entire preview snapshot, refusing even a write
followed by restoration of the old length. Changing a selection does not silently
change an already displayed preview; its names remain visible.

Grades check applied virtual state: exact required lengths, minimum ordinary
availability and free inodes. Protected audit sizes/presence are model invariants.
There is no exact-operation-string or forced-sequence grade. Pending size edits
clear checked results and current success. Incorrect syntax/paths, inode/block
exhaustion, protected files and stale previews have different retry messages.

The session stores its view, both worlds, drafts, selections and preview snapshots.
Source admission checks the complete immutable supplied profile for assignment,
practice and preview. Progress decode/save require the authored catalog; foreign
workspace metadata, altered capacity facts and invalid preview data fail without
replacing prior work. No existing IDs, scenario versions, shell semantics, theme
inventory or grading requirements are changed.

## Fidelity and coverage

This is bounded **logical capacity accounting**, not an ext4 implementation or a
real `df`, `du`, `truncate`, `rm`, `tune2fs` or `fsck` terminal. Flat virtual names,
logical sizes and dense per-file block rounding only. No contents, directories,
links, sparse allocation, quotas, open descriptors, allocation groups, journaling,
device I/O, real privileges or disk repair. Fixed overhead and reserve are supplied
facts. Maximum128 virtual files, bounded1TiB logical sum; no file-sized byte buffers.
All actions are in-process. Simulator-only serialized reference actions support
pack reachability checks; they are never taught as Linux commands.

Official sources checked2026-11-02:

- [LPI LPIC-2 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/):
  current4.5, exams201-450/202-450. This adds supporting capacity reasoning, useful
  before203.2 maintenance and200.2 planning. It does **not** implement their named
  filesystem/monitoring utilities, measurements, growth trends or full objectives.
- [Linux ext4 superblock](https://www.kernel.org/doc/html/latest/filesystems/ext4/super.html):
  distinguishes total/free blocks, reserved blocks and total/free inodes. Its real
  allocation/metadata behavior is substantially broader than this teaching model.
- [GNU df reference](https://www.gnu.org/software/coreutils/manual/html_node/df-invocation.html):
  retained foundation review used coreutils source/installed Info; the online page
  was unavailable during the earlier check. No new host/reference command was run
  for these lessons, and no full GNU-output equivalence is claimed.

## Focused gates

See private ignored `continuation/resume-20261102/capacity-gates-r2.log`:10 lesson
checks,2 prior continuation-registration/codec checks,19 capacity engine and19
repository engine checks pass. Lesson checks include ten different correct inode
selections, shrink alternatives, partial work, exact length failures, protected
file/path refusal, source forgery, stale-preview atomicity, separate examples,
success invalidation, reset and save/reload of an invalid draft plus stale preview.
Native widget/layout/capture checks and candidate evidence are still pending;
automated checks do not establish beginner comprehension or device accessibility.

Final source gates before packaging: capacity-final-model.log passes11 lesson
and2 registration/codec checks (including invalid packs and atomic failed-save
retention). capacity-final-ui.log passes five actual-widget/paint/layout/keyboard
lesson groups, one real capture-phase safety regression and three retained
related resource-budget regressions. Layout spans100/115/125/150/200% and
760/1600 widths in light/dark themes. capacity-clippy-r3.log production Clippy
passes; capacity-python-r1.log75 focused Python checks pass. New CURRENT-only
reference-test inventory closes six capacity inputs and individually proves
frozen reference refusal before any child; collectors/baselines remain untouched.
These are automated source gates; packaged native images are still pending.

Packaged221 first native run passes all four full interaction flows and all four
separate examples: `packaged221-native.log`;40 PNGs in native-runs/run-t5e2ttrb.
Inspected Introduction, block practice, full handover Assignment and200% light
Check controls. Images exposed duplicated new Introduction numbering (renderer
and data both numbered it). Only the four new concept-title prefixes were
removed; a painted-view regression now rejects duplicate numbering. It passes,
as do the75 focused Python checks after the pin refresh. A corrected separate
package is being built; earlier candidate/images remain immutable. No model,
assignment, objective or grade changed for this correction.

Corrected packaged221 technical verification now passes:

- Candidate `dist/teaching-review-20261102T133216Z/` (exact executable in
  CHECKPOINT.md), binarySHA256
  `7f30a4abae296ecc58362013a337c6804194efb7bf46e47fd27ba6435037c62c`.
- `packaged221-r2-content.log`:221labs/59units validated.
- `packaged221-r2-native.log`:all4 separate examples and all4 native assignment
  flows pass, with real file saves/reloads and resets.40 actual PNGs in
  `native-runs/run-jq6eeq1_/screenshots/workspaces/`. Corrected Introduction and
  light200% Check controls were visually inspected, in addition to the prior
  practice/full-assignment inspection. Earlier flawed-numbering evidence retained.
- [Introduction](../.build-artifacts/native-runs/run-jq6eeq1_/screenshots/workspaces/teaching-lpic2-capacity-inodes-introduction-dark-100.png)
- [Guided practice](../.build-artifacts/native-runs/run-jq6eeq1_/screenshots/workspaces/teaching-lpic2-capacity-blocks-example-dark-100.png)
- [Assignment](../.build-artifacts/native-runs/run-jq6eeq1_/screenshots/workspaces/teaching-lpic2-capacity-handover-assignment-dark-100.png)
- [Light200% controls](../.build-artifacts/native-runs/run-jq6eeq1_/screenshots/workspaces/teaching-lpic2-capacity-handover-assignment-light-200-controls.png)

These are technical source/native checks, not owner content acceptance, an
independent beginner study, screen-reader/device accessibility, cold application
reopening, install/upgrade or Windows/macOS runtime acceptance. No full suite was
repeated for this focused batch. No original progress/installed app was changed;
no commit, push, merge, release, dependency or system change was performed.
