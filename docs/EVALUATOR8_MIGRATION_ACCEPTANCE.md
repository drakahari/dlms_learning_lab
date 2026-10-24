# Evaluator8 migration acceptance proposal

Scoped acceptance checkpoint2026-10-23. This isolated worktree combines the exact
seven reviewed/tested engine candidate paths from65da217 with migration checks.
All11 public and3 private migration checks passed through the guarded runner,
with all-target Clippy and formatting checks passing. A whitespace-conflict case
was strengthened to use exact original bytes plus LF (parsed JSON equality is
asserted); its affected rerun passed and is not an extra distinct test. No
production engine/writer/history fix was needed by the migration checks. Main
activation, full regression, actual8 baseline comparison/capture and native
acceptance remain separate gates. Existing application/runtime architecture,
writer and recovery policies are unchanged.

## Controlled historical evidence

Three exact synthetic developer fixtures are copied, without reserialization,
from accepted capture manifest
`9a924e116ee7768053e361f6a651b956393d36c7df53c513924181de4fa486de`.
`tests/fixtures/shell-v7-fixture-provenance.json` records source commit, captured
binary identity and each file's byte count/hash. Independent read-only review
verified COMPLETE and all14 capture outputs, including the original7 binary.
The producer constructs every attempt from bundled authored fixtures plus
explicit unfinished synthetic edits; it never reads an existing personal
profile. Absolute host/capture paths, binaries, recordings, credentials and
personal progress are not included in these test fixtures.

The102 original contexts contain37 genuine revision7 assignment records/215
outcomes,33 genuine revision6 historical records/182 outcomes,37 genuine7
example results, favorites and separate pending assignment/example contexts.
The writer7 file is an actual successful writer output with actual receipt7;
its attempt data exactly equals the original synthetic profile. These observations
must never be regenerated under8 and relabeled7 or6. Synthetic observed completion
is evidence of the fixture's behavior, not a person's mastery.

## Proposed gates

- Ordinary decode/prepare preserves all102 contexts, immutable actual6/7
  snapshots and edited assignment/example sources. Current credit requires8;
  historical credit remains separately visible. Preparation moves actual7 to
  additional history without replacing the original6 slot or executing drafts.
- Saving an actual writer7 target preserves exact original raw bytes in the
  non-overwriting before8 slot, derives receipts7/8 from the actual target only,
  and leaves the caller's provenance untouched. Repeat saves retain history and
  the exact first snapshot. Reset-first and a practice-only actual7 result cannot
  bypass the boundary even when the replacement has no attempts.
- Valid different/whitespace-only occupied slots fail. An existing exact slot or
  matching no-clobber race succeeds. Failed callback/persist leaves target/caller
  unchanged; postreplacement sync warning is committed. These private tests add
  genuine7 input to the already reviewed writer mechanisms, not new fault APIs.
- Portable export omits receipts. Explicit import preview rechecks the actual
  editable sources and can produce real8 results, while retaining all6/7 data.
  The two deliberately unfinished assignments cannot earn current credit.
  Confirmed restore protects the old7 target; cancel/conflicting slot cannot
  replace it. Explicit corrupt/future raw recovery remains distinct from an
  ordinary old7 migration.
- A synthetic capacity pool constructed from genuine frozen observations across
  activities exercises31→32, identical reuse and rejection of a distinct33rd
  record. It is not a captured learner’s chronological history. Malformed/future
  records or provenance are refused without changing files.
  An explicit assignment Run can earn8 credit; edits invalidate it. Replacing
  one selected attempt preserves every other saved context. This final test
  models the selected reset operation; it is not native-button acceptance.
- Cancellation inside an active child capture uses a private observer/barrier
  and checks that no candidate, target replacement or artifact is published.
  Its input retains actual7 observations alongside all102 contexts. No shell
  script is ever passed to a host command.

The11 public tests plus3 private writer/cancellation tests pass using existing
bounded in-process APIs. They verify historical data and current behavior without
regenerating or relabeling any earlier6/7 snapshot. The separately authored engine
passed its55 focused checks; those are separate evidence, not14 extra migration
checks. No new curriculum, native or platform acceptance follows from this gate.

## Current versus frozen revision assumptions

Do not perform a blanket7→8 replacement. Frozen fixture versions, source commits,
real7 records, recorded7 observations, actual6 snapshots and their hashes remain
unchanged. The following current-writer expectations need narrow review when
actual8 is merged (line numbers are from61888a3):

- `src/progress.rs:1188,1220,1224`: private tests create a current write. Their
  expected backup filename and newly emitted receipt use the actual current
  evaluator. The controlled raw7 writer tests added here remain specifically7→8.
- `tests/shell_substitution_migration.rs:66,107,337,387`: helpers/expectations for
  a newly written current target should use EVALUATOR_REVISION. Actual captured
  revision6 fields remain6. The `[8]` future-provenance negatives at351/372/572
  must mean current+1;8 is no longer future under actual8. Candidate forged7
  receipts and fixed historical output data need semantic review, not relabeling.
- `tests/progress_sequences.rs:196..250`: rename the *current empty-record shape*
  test and use EVALUATOR_REVISION for that explicitly synthetic empty shape;
  retain genuine6 data and valid old `[1,6,7]` provenance. The `[8]` future negative
  at282 must become current+1. Do not claim that synthetic shape is an observation.
- `tests/shell_substitution.rs:20` and `tests/shell_capture_content.rs:76`: explicit
  current-capability gates should accept only the reviewed current8 merge;
  former evaluator7 fixtures/expectations must not change silently.
- `tests/unit_manager_integration.rs:88,454`, `tests/accounts_workspace.rs:61,367`,
  `tests/acl_workspaces.rs:432,471`, `tests/ospf_workspace.rs:242`, and
  `tests/unit_manager_content.rs:776,834,971`: separate generated *current* test
  records from frozen historical records. Current runs should be labeled only
  by the evaluator that genuinely produced them; migrated real7 should stay7.
  Fixed before7 filename assumptions belong to historical7 evidence only.
  Real6/current7 profile preservation test names should describe the actual data
  and newly required current run rather than silently inventing current credit.

The engine author's revision bump and supported syntax implementation remain
separate. The root integrator owns selecting/applying each narrow correction,
merged-regression gates and native current/earlier-output presentation checks.
