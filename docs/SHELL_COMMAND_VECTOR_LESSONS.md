# Build and dispatch safe command requests

The `shell-command-construction` unit contains three distinct practical assignments:
configurable utility options, allowlisted job dispatch, and preview/publish planning.
Each assignment has12 graded cases, a separate runnable example, progressive help,
and two supported alternative solutions. The unit extends the existing46 scripting
assignments; numerical variants are not counted as authored labs.

The combined catalog contains154 activities in39 units, including49 Bash/Shell
assignments. Registration and content do not establish complete certification
coverage or general scripting mastery. See `docs/VERIFICATION.md`,
`docs/IMPLEMENTATION_LEDGER.md` and `docs/CHECKPOINT.md` for the actual tested build,
completed checks and outstanding platform/accessibility/novice acceptance.

| Assignment | Practical reasoning | Graded cases |
| --- | --- | ---: |
| Keep source names out of the option list | Two independent sort choices, preserving duplicates when requested and separating option-looking source names from flags |12 |
| Run approved transformations without losing jobs | Select sort/uniq/cut from a closed operation mapping, reset command state per pair, retain later work after rejection and report aggregate failure |12 |
| Review or publish a mixed transformation plan | Combine different utility results, preserve their internal text, reject a late bad job before publication and keep preview read-only |12 |

These are configurable utility investigations and publication tasks, not extra
numeric variants or another label-printing queue. Preflight, capture and preview
transfer prior skills. The unit does not claim a new crash-recovery mechanism or
that all possible malformed requests are represented by36 finite cases.

## Entry and teaching

The `shell-command-construction` unit follows the actual existing units
`shell-arrays`, `shell-text`, `shell-case-dispatch` and `shell-safe-automation`.
Its exact unit proposal is in `tests/fixtures/shell-command-vectors-review.json`;
the combined catalog appends that unit followed by the independent NFS unit.
The test-only joint recipe removes the exact NFS suffix, verifies the complete
Bash-only38-unit source, then removes the exact Bash insertion to reconstruct
every original37-unit byte; no source hash is waived. Lab
prerequisites name real earlier lessons;
the second and third also depend on the previous new assignment. Existing order,
unit values and optional direct entry remain intact.

Each introduction has five short plain-language concepts, then a separate runnable
museum/gallery example with different files and interface values. A suggested
change has independently declared output/status/files and a repair. An example
run cannot award assignment credit or replace an assignment draft. Learn keeps
Introduction → Try example → full Assignment, with optional hints and worked
reasoning; Challenge uses the same practical assignment without unsolicited help.
No new instructional UI panel is required.

Teaching explicitly distinguishes:

- Bash arrays from portable-style scalar command-name/quoted-argument use.
- The program name, flags and file operands as separate words.
- Quoting, which preserves arguments, from `--`, which ends these tools' options.
- The single `-` stdin operand, which remains special even after `--` and is
  rejected by these assignment interfaces.
- A closed operation allowlist from treating request text as a program or script.
- Neighbor deduplication from sorted/global deduplication.
- Captured output from exit status, and a partial helper result from a publishable
  complete plan.
- Lab-supplied arguments/files from variables created inside the script.

No `eval`, source reparsing, external/PATH lookup, executable script file,
subprocess or host command is taught or executed. Supplied quotes, stars, percent
signs and shell-looking strings remain data. Arrays do not make arbitrary program
names/options safe; the script still validates its interface and chooses known
operations. `printf` formats remain fixed.

## Authored evidence and negative controls

The new pack contains36 literal deterministic cases. Exact stdout, stderr, status
and complete resulting virtual-file maps are authored data; expected outcomes
were not captured from the candidate VM or reference Bash. Five additional guard
review cases exercise directory-part/empty-name and unpaired-tail checks without
raising the existing12-graded-case limit or earning separate credit.

Every task has a full reference and two different supported equivalents. The first
unit compares incrementally built option vectors with direct utility branches and
whole-vector selection. The second compares loop state with independently called
helpers and direct branches. The capstone compares staged vector capture with
direct utility capture and separately validated incremental publication. Grading
accepts behavior, not a required spelling or array syntax. Completion cannot prove
that a learner used arrays or establish general scripting mastery.

The review fixture includes exact12-row pass/fail vectors for plausible mistakes:
ignored flags, flattened command strings, missing option terminators, global
instead of adjacent deduplication, stale per-job vectors, early batch exit,
forgotten failure status, preview writes, ignored helper failure and premature
truncation. First-case hardcodes and extra file damage have independent controls.
A current source edit or broken run clears current success; a genuine retry can
restore it. Syntax, Unsupported, wrong behavior and a correct solution use the
existing distinct feedback paths and expected/actual comparisons.

The content tests validate the bundled assignments and exact unit composition, all references/alternates at several VM slices, declared negative
vectors, example/change/repair, full file comparisons, both draft sources,
favorites, actual older saved records, safe save/reopen, selected reset and
backup/export/validated restore/cancel. Main-file migration receipts are checked
as explicit local provenance, separate from portable learner data. A real original
older fixture remains byte-exact; no historical result or credit is synthesized
or relabeled. All36 prior pack/index source hashes and original147 IDs are pinned.
The verification documents record actual completed gates and any remaining failures;
source handoffs are not runtime acceptance.

## Fidelity and remaining acceptance

The existing in-process VM supports only its documented subset: bounded dense
arrays, scalar variables, case/conditions/loops/functions/capture and fixed C/POSIX
byte-order sort/uniq/cut flags on flat virtual text files. LC_ALL=C is the supported
locale selector here; this does not implement environment export. To send a new
LC_ALL variable to external programs in real Bash, use export or a command-prefix
assignment. Neither workflow is modeled by this simulator. No regex, general CSV,
locale-general ordering, directory traversal/alias detection, atomic rename,
power-failure recovery or arbitrary utility behavior is claimed. Function calls
in pipelines or with call-level redirects, generated `local` names and general
environment-prefix execution remain unsupported. Unknown programs are an honest
simulator Unsupported diagnostic, not a fabricated real Bash status127.

Acceptance evidence includes independent source/content review, behavior and
persistence regressions, fixed controlled Bash/Coreutils comparisons in disposable
development fixtures, and actual native introduction/example/assignment/error/
repair/reset/resume/Challenge flows at100/150/200 text with light/dark themes.
Installed Fedora candidate9a90f275e133 passes all three flows,72 captures, with
separate example/assignment drafts, real Run/Stop and warm full progress readback.
Seven dedicated feedback captures distinguish selected Case N/total Pass/Fail
from the whole Run status. Named numbered results support pointer or Tab/Enter
inspection. A passing selected case stays correct while other cases fail; the
overall result names the failure count and the individual rows explain why.
Wrong-file, nonzero-status and complete Expected/Actual comparisons remain intact.
The earlier00d presentation ambiguity is corrected without changing grading,
shell syntax, evaluator revision, cases or progress formats.
Read INSTALLATION.md for exact build identities and limits. Windows/macOS runtime, physical screen-reader
and novice comprehension remain separate pending acceptance. Do not count source
or automated tests as those observations.

Original authoritative manuals inspected2026-10-27: GNU Bash Reference Manual5.3
(updated18May2025), Arrays§6.7, Simple Command Expansion§3.7.1 and Command Search
and Execution§3.7.2; original installed manual SHA256
`b91026acffb159bd1e842e3116a95539bf858e2b57cbfa74df9e3a53379177a4`.
GNU Coreutils9.10 original installed info manual, Common options, sort, uniq and
cut invocation; compressed SHA256
`6e28a10ae4e4acad5c0147348ffe0a0a0cea4f93ed7cd794d434d953059e4716`.
Online primary URLs are bundled per lesson; this worker read installed originals
without executing reference commands and does not claim newly rendered online
pages. No certification objective promotion, vendor endorsement or full Bash/
Coreutils conformance follows from this unit.
