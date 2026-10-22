# Introductory scripting teaching review

Seven existing assignments, same scenario IDs/versions and behavior cases. No additional curriculum or interpreter features. Reviewed 2026-10-21.

Learn starts a **new** attempt at its skippable Introduction. The Introduction, Try example and Assignment tabs are always available; no quiz or click-through gate is required. Challenge opens the same complete assignment without introductions or examples. Hints, references and explicitly requested reasoning remain optional.

## Prerequisite audit

| Existing assignment | Assumed foundation | Teaching added before the assignment |
| --- | --- | --- |
| Reliable handover | None | Script/editor/Run, comments, printing, supplied versus created variables, assignment syntax, single/double quotes, `%s` and backslash-`n` |
| Supplied file label | Lab 1 | Positional arguments, copying them into named variables, spaces and empty arguments |
| Read a request | Labs 1–2 | Standard input versus output and arguments, `read -r`, default trimming, data backslashes versus printed line breaks |
| Missing destination | Labs 1–3 | Status 0/nonzero, `exit`, `-z`/`-n`, test spacing, `if`/`then`/`else`/`fi`, semicolons and separate error output |
| Incident report | Validation | Virtual paths, literal `grep -F`, match/no-match/read-error status, truncate versus append, file output |
| File handover | Report | `for`/`do`/`done`, `"$@"`, loop variables and `cat` |
| Safe incident capstone | Validation, report and loop; earlier foundations transitively | `-f` file existence versus nonempty text, checking before redirection, accepting no matches and choosing final status |

Previously these foundations were mostly hints rather than teaching. The capstone's graph now names all three relevant prerequisites explicitly. Its newly introduced file test is explained before the assignment. Technical subset documentation remains optional reference, not the beginner introduction.

Every introduction has a short example with different values/messages from its assignment, an explicit description of supplied inputs, and one suggested edit. The first example creates `visitor='Mina'`, reads lab-supplied `ROOM`, and prints a greeting; the assignment still requires a two-line SITE/TASK handover. Later examples teach the necessary syntax without displaying the capstone's complete solution.

## Practice isolation and persistence

Declarative `Scenario.teaching` supplies concepts, example source, input description, suggested change and a complete deterministic example fixture. Pack validation runs the example in the **existing bounded VM** and rejects malformed/unreachable examples. No host commands are invoked by practice or grading.

`WorkspaceData.script_learning` is optional version-1 metadata: selected Learn view, independent example draft and optional example outcome. The assignment stays in `State::Script` with its own source/run snapshot. Example Run uses one fresh virtual fixture and never writes assignment state, check status or completion. Edited examples are exploration, not tests against an expected answer. Moving views cancels an unfinished job but retains both drafts and completed results. No saved job resumes executing on launch.

Old saves omit this optional field and remain readable. Existing attempts reopen their Assignment so an upgrade does not interrupt current work; Introduction stays one click away. New attempts start at Introduction. Reset confirmation clears only the selected attempt, including its example draft; other attempts, preferences and favorites remain. No scenario version, goal, case or reference solution is changed. Downgrading to an older binary after saving new metadata is not supported.

## Feedback

Parser diagnostics identify malformed scripts separately from unsupported syntax/commands and resource limits. Runtime command failures remain visible. Assignment Run selects the first failing behavior case, displays actual output and status, and compares expected versus actual text when useful (`\n` is shown explicitly in comparisons). Missing final newlines and spacing differences have targeted explanations. File/status/error-channel mismatches explain what to inspect. A correct solution passes all original deterministic cases; equivalent supported programs remain accepted. Editing assignment source clears the previous run and success. Running or editing an example never clears an assignment's earned result.

## Appearance

The toolbar retains immediate persistent theme selection and a Settings button. **Settings → Appearance → Theme samples** contains explicitly labeled sample controls; these do not perform lab actions. All 28 palettes and original attribution/notices remain intact.

Verification and native review evidence are recorded in [VERIFICATION.md](VERIFICATION.md). No novice user study is claimed.
