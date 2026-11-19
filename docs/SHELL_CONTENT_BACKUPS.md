# Content copies with a recovery index

Two distinct projects follow managed configuration in the existing Bash/Shell path:

| Stable ID | Independent assignment | Main samples |
| --- | --- | --- |
| `shell-backup-content-set` | Validate a complete request, preserve exact source text in new copies, and write an ordered recovery index | 12 |
| `shell-restore-validated-set` | Validate the whole index and every recovery copy before restoring any target | 12 |

These extend the recorded backup/restore and debugging/recovery project gap.
They are not numeric variants of the earlier one-file publication exercise.
An actual created multi-file set can be passed into restore after its targets
are damaged or removed. The index and copies remain available afterward.
Source catalog294/95 adds these two after the verified292/94 snapshot, with all
prior Scenario/Unit values and progress retained. Source registration is not
native or packaged acceptance. The last verified Fedora package is292T033211Z.

## Teaching and interactions

Introduction teaches the difference between a content copy and an archive or
filesystem snapshot, complete preflight before opening outputs, quoted names,
exact final-newline preservation and an ordered index. Restore teaches literal
read/EOF handling and preserving recovery evidence. Worked examples use different
names, `demo.index` and `saved-` files, plus an untouched practice note.
Quoted `"$@"` is portable shell syntax. Indexed-array assignment and
`"${names[@]}"` in the worked strategy are Bash-specific. A supported portable
argument-loop strategy also passes the copy task; a two-pass restore is accepted.

Guided practice uses the existing editor and Run example/Stop controls. A copy
through command capture demonstrates lost final newlines. Removing the EOF
condition skips the final record; removing empty IFS loses a padded name.
Expected preview and actual output/files remain distinct. Examples cannot
complete assignments or overwrite assignment drafts. Each test starts with its
own sample inputs; its simulated files reset for the run, while editor work persists.
Independent starters contain real early-write/read/newline mistakes and retain
full tasks. Solutions remain behind explicit optional help; Challenge keeps the
independent work. No new renderer or simulation engine is introduced.

## Bounds and honest differences

This project handles at most eight virtual UTF-8 text files in the disclosed flat
`/work` namespace. The supplied filename domain is at most80UTF-8 bytes, no LF/NUL;
this is an input-domain guarantee, not a claim that the script validates arbitrary
name lengths. The five-byte `copy-` prefix remains under the250-byte VM path bound.
The deliberately line-delimited index cannot represent newline-containing names.
Spaces, literal wildcards, backslashes and dash names within the domain matter.

`cat -- "-"` still means standard input. The taught `cat < "$name" > "copy-$name"`
uses redirection to select a literal file even when its name is exactly `-`.
Normal redirection may create/truncate a file before the utility runs. Defined
validation failures must leave every file unchanged; that does not promise
rollback for Stop, resource exhaustion or other unexpected failures.

No binary content, directories, metadata/ownership, tar/archive formats, real
secure storage, concurrency, locks, traps, transaction atomicity or crash safety
is modeled. Those broader roadmap capabilities remain open. Existing VM
instruction/memory/output/file limits and cooperative Stop are unchanged;
revision19 and all earlier actual reference/baseline proofs remain immutable.
Learner input never executes a host shell, filesystem operation or network request.

## Focused verification so far

R08six recipe groups pass: two genuinely different supported implementations per
project, twelve main samples each, thirteen supplemental boundaries and two
separate examples:76 predicted outcomes checked with the actual in-process VM.
Additional tests run an actual save result through restore, reproduce a complete
first result to defeat hardcoding, exercise partial wrong approaches, reject
escape/unsupported syntax, invalidate edited success and cooperatively Stop.
Reserved recovery targets and valid eight-file boundaries are in the actual
grading roster, not only supplemental tests. Literal backslashes and a bare dash
filename (with different stdin) are exercised.

R07three admission/persistence groups pass9.05s: actual pack/unit validation,
worked experiments, separate drafts, favorites, safe save/load and selected reset.
The original private-Index and oversized single-pack test setup errors remain
recorded; the corrected test uses the real collection loader without increasing
its2MB pack/32MB collection bounds. Registered-prefix preservation and actual
native editor workflows are being checked next. No full-suite/native/candidate
acceptance is inferred from these authoring checks.

Independent content/algorithm review `eb9cc278bd13e495fed10df357c392e9f160fece17b20f41ee7ddc808cabcf89`
closed the literal-dash, filename-domain/path-budget, reserved recovery target,
backslash and portable-$@ findings. R02actually failed the old bare-dash recipe;
its unchanged log is retained. No grading test was weakened or engine limit raised.

## Primary references

Checked2026-11-05: installed GNU Bash5.3 upstream manual nodes Arrays, Redirections
and builtin read/printf; installed GNUcoreutils9.10 cat invocation node. Compressed
manual pins: Bashc2ab10cda098ad2e6a43ced7a02ff1545fd9d85f3dc05183bb1d72cd3fcf60b2;
coreutils6e28a10ae4e4acad5c0147348ffe0a0a0cea4f93ed7cd794d434d953059e4716.
Canonical split web pages timed out; no successful live-page retrieval is claimed.

[Redirections](https://www.gnu.org/software/bash/manual/html_node/Redirections.html),
[Arrays](https://www.gnu.org/software/bash/manual/html_node/Arrays.html),
[builtins](https://www.gnu.org/software/bash/manual/html_node/Bash-Builtins.html),
[cat](https://www.gnu.org/software/coreutils/manual/html_node/cat-invocation.html).
No GNU text, code or executable is copied into these original lessons. These
compositions reuse the previously verified evaluator19 subset; no new real-Bash
composition comparison is claimed. Reference tools never execute learner drafts.

Native workflows, enlarged themes, a separate new Fedora candidate and final QA
remain pending. No fresh inclusive Rust suite, physical-device, screen-reader,
novice, Windows/macOS/Omarchy or full course acceptance follows. No owner app/
profile replacement, dependency/system change, push/merge/release is authorized.

## Native development check —2026-11-05

R11registered preservation/teaching/progress4 passes18.48s: all292prior Scenarios
and94Unit values remain exact, with course ordering and72/101/45/76track totals.
R12actual widget/route3 passes6.35s: both example experiments, independent runs,
Stop, retry, drafts and save/read; all three views hide assignment solutions and
preserve plain wrapping at100/115/125/150/200%,760/1600px and light/dark.
R13actual development native62actions/30PNG exits0 (run-p33oeu0n). Root opened
Introduction, guided editor, independent task, specific starter failure and
light200focused editor/Run/Stop controls. No mockups were used.

Review found a redundant optional panel repeating the exact essential concepts.
The two new presentation entries now use their original five source concepts
directly, removing only that duplicate control; all teaching stays visible.
A practice sentence and secondary input-bound spacing were clarified. Grading,
cases and simulator behavior stay exact. Final current-render tests/QA and a
new separate packaged check follow; latest verified executable is still292.

## Final focused gates before packaging

R16current-render3 passes6.26s after removing the redundant explanation; R17full
registered source/unit preservation and progress4 passes17.98s. R20all-target
Clippy passes24.88s after staging lib/app; observed min9.525GiBavailable, fullPSI0,
OOMdelta0. R23Python856passes12.374s. The initial stale-inventory failure remains
retained; inventory was regenerated, not its test weakened. R22inventory294:
289updated/5alreadymeets. R24coverage294/0drafts passes; issuer ledger remains
227partial/8supporting/1005Open. R25preserves every original Bash requirement.
No fresh whole-inclusive Rust rerun, new host-Bash composition proof or new
installed/cold-process acceptance. Next separate candidate and actual packaged
flow are pending. All earlier platform/accessibility/novice limits remain.

## Packaged milestone verified

72 CCNA /101 LPIC-1 /45 LPIC-2 /76 Bash. Both content-backup and validated-restore
projects are registered, taught and graded with separate practice drafts. All prior
292 scenarios/94 units are preserved exactly. Evaluator19 and progress format stay.

Separate protected Fedora review executable (installed app/profile untouched):
`/path/to/dlms_learning_lab/dist/teaching-review-20261106T043606Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab`

Binary SHA256 `14e2c6f660ac66281b3e4078ebaaca7c39557ab08d049194d06ad9530b256257`;
archive SHA256 `6051ee8058e55df9c924f00bcaff10065f8892e823e162887f4a7f858ce432ef`.
R27 matched614 source/data,273 bundled-document and139 scenario hashes before
post-build notes and the later transport-core source admission. R28 packaged
content validation passes294/95. R29 packaged native exits0 with62 actions/30PNG
(run-gw5f_n9q): Run/Stop, errors/retry, independent drafts, state invalidation,
save/read and reset, normal and light200 views. Introduction, guided editor,
starter feedback and enlarged controls were visually opened. R30 retains exact
identity/verification receipts. Candidate build428 memory samples: minimum7.016GiB
available, fullPSIavg10 maximum0, OOM delta0. One MAIN lock, jobs2/runtime2/disk temp.

Focused recipe6/76 predictions, registered preservation/progress4, UI3, Python856,
all-target Clippy and coverage gates pass as recorded below. No fresh whole-app
inclusive Rust rerun, new host-Bash composition comparison, installer lifecycle,
new two-process cold acceptance, all-theme/keyboard-only, physical-device,
screen-reader, novice or Windows/macOS/Omarchy runtime acceptance is inferred.
Issuer ledger remains227partial/8supporting/1005Open, not full exam coverage.

