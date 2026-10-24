# Implementation continuation checkpoint

Resumed at the owner's request on2026-10-24. The application is NOT complete.
The M73 safe-stop snapshot is preserved in c90cec8 and ignored receipts.
M75 evaluator8 is now accepted for normal integration:931 distinct clean-target
model/UI checks, final Clippy/fmt/build, actual before/after comparisons, genuine
6/7 migration and15-capture native Fedora Run/Stop/history/restore/reset flow.
Rendered diagnostic and completion screens were reviewed at150%light/200%dark.
Native logm75-case-history-native-r2.log; screenshots run-_z5iwydb. In short tiled
windows, output/document scrolling is necessary; lower retry reminders may need
further output scrolling. No physical-device/novice/screen-reader claim.

**Current next actions supersede older pending sections below:** commit/integrate
reviewed8 source, build the identified normal executable, then test and integrate
the three distinct case-dispatch lessons from source-r3 (84c59d67…). They remain
unbundled/unrun and do not count yet. Root independently recomputed all31 expected
contract tables without running a shell/VM; real simulator checks still follow.
MAC-learning R4 remains uncompiled and has a remaining retained-String-capacity
fix; source review accepted the other guards. All work stays serial via MAIN
runner with exclusive targets. No public release is authorized.

## Verified normal app and source

- Main's application implementation is pushed `b73c2ef21cfa5738ab0091bcb8bc46c024235fd7`.
  The later pushed `ab4d990` changes only build-cache safeguards/docs, not the app.
- **106 authored labs /25 units:31 CCNA,21 LPIC-1,17 LPIC-2,37 Bash/Shell.**
  Evaluator8 is accepted for integration;28 themes and all existing scenarios/progress
  remain. Pending lesson files and numeric variants do not count.
- Launch the last verified development build from the main checkout:

  ```sh
  ./target/debug/dlms-learning-lab
  ```

  Embedded identity is b73c2ef. Executable375,840,112bytes;
  SHA256 `89c1dbf101fed80cb0bd34b01aa0a00762ed227eb592f8060aa00e0fd60b72a0`,
  independently rehashed unchanged at this stop. No current106 installer exists.
- M70 normal service unit is integrated: saved/loaded/running recovery, strong
  preparation/ordering, optional Wants and target handover. Current baseline
  has891 distinct Rust checks, Clippy/fmt/build and actual all-four Fedora native
 72-capture acceptance, including real typing at light150%/dark200%.
  Logs `m70-*`; screenshots in `.build-artifacts/native-runs/run-pejyy86y`.
  Earlier failed native preparation retry is retained and corrected in teaching,
  native actions and a14th content regression, without weakening the model.
- Latest actually installed candidate is still102 labs/24 units, source
  `29096be81982de2812b2e25c609703bf40688472`, under
  `dist/dlms-learning-lab-0.1.0-g29096be81982-x86_64-unknown-linux-gnu-fedora-44-candidate`.
  Installed lifecycle, all-four OSPF,72-view shared IPv4/VLAN alignment and10/10
  quiet Wayland readbacks passed. That candidate predates M69/M70 and is NOT
  completion. INSTALLATION.md records exact hashes and acceptance scope.

## Mandatory safe build workflow

Read AGENTS.md and BUILD_WORKFLOW.md. Use MAIN absolute `python3 tools/dev.py`.
One heavy invocation across all worktrees, Cargo2 jobs and2 runtime test threads;
never bypass the shared lock or a retained active/unfinished marker. Root verified
both markers absent and the lock free after the final running UI suite.
Both helpers confirm idle with no owned running processes; no further work until resumed.

**New, independently reviewed fix, pushed ab4d990:** targets must be exclusive
per canonical checkout even when builds are serial. A comparison caught Cargo
reusing a106-lab library from another worktree for pinned102 source in the old
shared `target/acl-foundation`. It failed before output; that target/log remain.
Main retains `target/`; worktrees now automatically use
`.build-artifacts/targets/NAME-PATHHASH`. Omit `--target-dir`; overrides to another
checkout and aliases of the target namespace are refused. Candidate builds require
main. All76 Python checks pass, including new cross-worktree/candidate regressions.

```sh
# Main
python3 tools/dev.py test --locked --offline --lib --test '*'
# Isolated worktree; no shared target override
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 \
python3 /path/to/dlms_learning_lab/tools/dev.py --cwd /path/to/worktree test --offline --test TARGET
```

All artifacts/cache/targets/dist remain on verified NVMe Btrfs project disk.
`/tmp` is RAM-backed; no new large artifacts there. Only the exact seven approved
old review executables were removed earlier (1,513,992,192bytes). Sources,
diagnostics, toolchain originals, old targets and uncertain evidence remain.
No broad cleanup, cargo clean, GPU/swap/OOM/service/desktop/model changes.
Swap is nearly full; low observed PSI/headroom is not a guarantee. The guard
refuses low headroom/pressure and cancels only its owned group if sustained.

## Earlier isolated evaluator8 gates (M75 acceptance above supersedes pending labels)

Worktree `.build-artifacts/worktrees/evaluator8-migration`, branch
`evaluator8-migration-acceptance`; HEAD `c06a03f5e33cad6311790a4caa76a3ebc0e9ca37`
merges main ab4d990. Reviewed uncommitted changes are intentionally retained.
Do not reset/clean them or confuse this worktree's binary with the normal app.

- Original case engine: author commits `27d0a0e` then corrected `65da217`;
  cherry-picked as `ede002a`/`2df5933` in migration. Own migration tests/fixtures
  commit `c228458`, main b73 merge `604ff4b`. No new dependency/public architecture.
  Explicit revision8 because case support/diagnostics change earlier outcomes.
- Corrected engine passes55 focused checks/Clippy/fmt; source and fixtures are
  frozen. See `docs/SHELL_CASE.md` in the worktree and ignored receipt
  `reviews/shell-case/CASE-CORRECTED-VALIDATION-R6.json` under `.build-artifacts`.
  Array-data delimiter, reached locale refusal and static diagnostic fixes are
  included. A later keyword-array concern was retracted after lexer inspection;
  its author test-only addition remains unrun/uncommitted, not a proved bug fix.
- Genuine6/7 migration acceptance:11 public+3 private checks pass, exact33 real6
  records and37 real7 records retained, target-driven exact before8 snapshots,
  Reset-first/practice-only, capacity, conflicts/write failures/cancel/recovery,
  portable restore and selected reset. Receipt
  `reviews/evaluator8-migration/passed-r5/RUN_RESULTS.json` (SHA48116208…).
  Old outcomes were NEVER generated with8 and relabeled6/7.
- **Fresh exclusive-target full regressions pass:**810 library/integration checks
  across76 targets (`m73-exclusive-full-lib-integration-r1.log`), plus121 UI tests
  (`m73-exclusive-full-bin-ui-r2.log`,434.36seconds). Combined931 distinct checks;
  no overlap counts added. Target
  `.build-artifacts/targets/evaluator8-migration-fe542e765a0347df`.
  Final stop receipt `reviews/evaluator8-migration/owner-stop-checkpoint/CHECKPOINT.json`
  SHA521d84a390693ec509dc8b48a37cfd0b1952cd01654185c654fab2d4b1801599
  records all20 dirty/untracked file hashes, patch, executable/log identity and
  remaining gates. UI memory438samples: minimum12,922,429,440available bytes,
  peakPSIavg10zero andOOMdelta0; ordinary observation, not a guarantee.
  Full dependency compile and source worktree are recorded. Old810 shared-target
  observation remains qualified; use the clean-target run as current evidence.
- The121 UI run includes new genuine7-to8 native-plan fixture/source checks but
  **the actual new native flow has NOT run**. First UI compile failed because a
  proposed observer used a test-only private Vm method; retained log
  `m73-exclusive-full-bin-ui-r1.log`. Accepted two-path correction observes a
  fixed virtual marker emitted by the selector child, with no VM/API/budget change.
  The successful121 run uses this correction (native8-marker-r2).
- Applied native proposal has six narrow UI paths, stage15000..15075 and
  `DLMS_SMOKE_STAGE=case-history`. Exact frozen writer7 private fixture, actual
  typed Run/Stop/retry, separate example, history6/7,150/200 syntax retry,
  export/old7 Cancel+Confirm/safety copy and selected Reset/Challenge are intended.
  No solved State is installed. Readback is not a cold-process restart claim.
- **Held, UNAPPLIED focus-band correction:**
  `reviews/evaluator8-migration/native8-focus-band-r3/SOURCE-READY.json`,
  SHA471c7486a5938e238d53e9da1802a28631def1e550330719a3cbeda84117b611.
  Root reviewed the patch; final application/test grant was not sent before stop.
  It reveals a font/control-height band inside the actual scrollable editor,
  then proves TextEdit focus. The whole180pt editor need not fit a short200%
  viewport. Changes capture-only helper and acceptance doc, no product geometry.
  On resume verify hashes, apply, run the affected plan check, Clippy/fmt/build,
  then inspect actual Fedora native flow before enabling8. Shared Tab tests are
  separate; new flow uses clicks/Ctrl+A/Ctrl+Enter, not native Tab acceptance.
- Root staged three docs in this worktree only: SHELL_SUBSET, SHELL_CASE_REFERENCE
  and PROGRESS_RECOVERY for8. Main docs still describe shipped7. Update stale
  staging headings/evidence after acceptance; do not publish pending as complete.

## Accepted before/after/reference evidence

All paths below are inside ignored `.build-artifacts/reviews/shell-case`.

- Actual public7 baseline `captures/before-case-public7-8eadf3d-r2`, COMPLETE
  `9a924e116ee7768053e361f6a651b956393d36c7df53c513924181de4fa486de`:
 4124 real observations,37 assignment records/215 results/37 examples,33 exact
  earlier6 records,102-context raw/writer profiles and4 actual Stop outcomes.
- Fixed37 builtin-only Bash5.3.9 reference is independently accepted, manifest
  `7af1b2e86a9cf9364c5ea170bcdfba4142a5c8cff0c5046efd0be57c55af02bd`.
  Root+peer verify all157 outputs and pins. This fixed developer oracle has no
  learner-input path and never becomes the app engine. Original NOT RUN/null
  input metadata remains immutable; actual observations are separate.
- Actual8 R1 completed but used inaccurate ticks:0 metadata on four genuine Stop
  outcomes. It and all original source/controls remain archived unchanged.
- **Corrected actual8 R2 accepted from a fresh exclusive author target**:
  `captures/after-case-public8-reviewed-r2`, manifest
  `7053d055a629fca5e07fc73efe074874682824f53293bd16630ee66a0a62fdb8`.
  Root independently verifies all16 output hashes,4124 outcomes,3968 equal to7,
 156 explicitly allowed changes/39 groups,37 actual8 records equal genuine7
  except revision, exact old raw profiles, Stop measured ticks3/1/1/1 by slice.
  `ROOT-ACTUAL-R2-VERIFICATION.json` records source/target/binary identity.
  Private control `CAPTURE-PINS-R2.json` SHA
  `f1a7fc196e297bbacbebf0b8a7cb07612962dc94ed0bc1d7f83f357ee05fa998`.
  Full binary SHA `c0c891333cb6a5ead07a2c2bd04e81f1ca354922966f8e0510f379cde55d8cb8`.
  Scope is isolated102 catalog/unchanged37 scripts, not106 native acceptance.
- First R2 attempt hit the wrong-library catalog guard before creating output.
  Preserve `m73-after8-actual-comparison-r2.log`; the same no-clobber captureID
  could be used by the later fresh-target invocation because no STARTED existed.
  No evidence was overwritten or a catalog assertion weakened.

## Source-only future curriculum/capability — not counted

**Three scripting tasks**, `.build-artifacts/reviews/shell-case-content`:
report-mode interface, prioritized batch routing and validate-all preview/apply
publication.11/8/12 independent cases, separate museum examples/experiments,
two alternatives each (including no-case methods), wrong/partial/hardcoded/file
mutation candidates and seven proposed tests. R1 manifestd811fb72… is preserved;
no VM/compiler/native/main registration or coverage claim.

Author stopped with exact R1 intact; **R2 edits had NOT started**. Its precise
source-only follow-up is `reviews/shell-case-content/STOP_CHECKPOINT.md`,
SHA97320e43377754660b0bf392a0fd9ca11923b305cf87eced678cc2005128e174.

Root found a real teaching/reference mismatch before execution: default
`read -r` trims outer spaces/tabs, so a promised full-record trailing-space
preservation and one fixture could not work. Existing original shell-input,
shell-eof and shell-number-records already teach this correctly. Author was
asked to preserve R1 and scope a new R2 canonical-record format explicitly,
fix the wording/fixture, add input-format guards and defer whitespace-preserving
IFS behavior. Check the helper's final receipt/latest manifest before runtime.
Do not change the frozen8 engine or silently weaken expected outputs.

**CCNA MAC-learning draft**, `.build-artifacts/reviews/mac-learning-foundation`:
SOURCE-DRAFT-R2 SHA925e82f1d1c96634457e918a284626d393009573fa96421d0b15c2b940d6847a;
787-line pure module plus12 proposed tests, formatted/parsechecked only.
Uncompiled/unintegrated, no public lab/title/coverage. Source/VLAN learning,
known/unknown/broadcast, source-only aging, NIC arrival vs acceptance and independent
cloned current-state checks over existing VLAN forest. Primary current Cisco
sources checked10-23. Independent read-only review found three required fixes:

1. A negative frame can pass by disabling its own port while an unrelated
   positive stays healthy. Add per-negative physical/attachment health guards;
   one global positive is insufficient for VLAN-isolation teaching.
2. Cap retained UTF8 diagnostics (e.g512bytes); a huge unknown field currently
   expands serde error text to nearly the input ceiling.
3. Distinguish compact-state payload and raw JSON input ceilings. A legal
   maximum shape can be120934 compact but302001 pretty bytes, exceeding the
   current262144 raw decoder ceiling. Test legitimate maximum pretty restore.

Also review root's additional NIC-arrival concern: a tagged frame reaching an
up workstation is physically present even when the modeled untagged endpoint
rejects it; current draft omits that arrival. No fixes were applied after the
owner's stop. Down-link seeded stale entries are intentionally permissive and
need a documented older-table boundary before teaching. All actual compiler,
fidelity, limits, progress, state/goal/native/content acceptance remains pending.

## Resume order and remaining product scope

1. Confirm no live marker/owner; use exclusive targets and serialize all work.
2. Finish held focus-band source gate, affected check/Clippy/fmt/build and actual
   genuine7-to8 Fedora native flow. Preserve failures; correct real issues.
3. Commit/push reviewed8 activation only after those gates; normal app stays7
   until then. Do not relabel historical results or invent current mastery.
4. Independently review corrected three-task source/expected tables, test
   reference/alternates/negative/partial/drafts/resume in isolation, register only
   accepted real tasks, inspect actual native intro/example/assignment flows.
5. Correct/review/test MAC capability before authoring dependent CCNA tasks;
   continue substantive networking/Linux/shell batches against the gap map.
6. Produce an identified clean new Fedora candidate after coherent commits and
   test the INSTALLED app/lifecycle/reopen/alignment/startup, not only dev binary.

Coverage remains128 narrowly Partial/1 Supporting/1111 Open of1240 overlapping
references; not a percentage. Most full-product checklist items remain Open.
Current CCNAv1.1 is separate from futurev2.0 (2027-02-03), LPIC-1v5 andLPIC-2v4.5;
full source inventory checked10-22, scoped sources10-23. Preserve focused native
workspaces, separate examples/drafts, all28 themes, simulation-only isolation and
MIT original-code/curriculum approval. No old-project edits or new dependencies.

Historical intermittent Wayland frame-callback cause remains unresolved and
unreproduced; passing repeated launches are not a cure. Linux223 notice/source
packages resolve. Original macOS dispatch0.2.0 MIT notice is absent from its
published/pinned source; macOS packaging remains gated, no invented notice.
Actual Windows/macOS desktop installation/runtime/upgrades/uninstall/signing,
screen reader, physical-device keyboard/accessibility and novice comprehension
remain pending. Those limits do not excuse unfinished independent curriculum.
No installer/public release/deployment publication without separate approval.
