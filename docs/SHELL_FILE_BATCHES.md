# Filename selection and safe batch reports (M28)

Three distinct authored assignments,29 scripting assignments/seven scripting units,58 total assignments/fourteen units. This follows the accepted native Introduction → Try example → Assignment pattern. No syntax/dependency or host execution is added by this content batch; M27 implemented and reviewed the required bounded filename matcher first. Existing scenario IDs/versions and progress fields stay unchanged.

| Assignment | Practical skill | Adverse cases |
| --- | --- | --- |
| Build a changing file manifest | Discover visible .log inputs, keep selected names whole and replace/append a stable manifest | Spaces/backslashes/percent/UTF-8 names, hidden/case-sensitive names, empty and absent selection, new/old report |
| Collect incidents across a changing log set | Distinguish filename selection from literal record filtering; combine ordered matches without losing duplicates | Normal grep no-match status, empty/no files, final unterminated match, incidental substring matches and unchanged sources |
| Publish a safe incident handover | Capture source names before output creation, validate the full request before mutation, publish headings/records/counts | Missing/extra/empty destination, late existing-source collision, new .log destination, hidden/no sources, spaces, empty sources and unterminated matching records |

The first prerequisite is the foundational file-test capstone; its introduction still explains file testing and no-match behavior. Later tasks point to the actual pipeline, function, return and counter lessons. Prerequisites are advisory; experienced learners can enter any full assignment directly. Challenge retains the same meaningful task and editor, with no unsolicited example/solution.

Each skippable introduction teaches the new idea in plain language. Its different .note/TODO or protected note-list example runs in the same bounded VM, with a separate saved draft and no assignment credit. Suggested changes have explicit testable outcomes: an empty selection/list, normal empty filter results/status0, or a collision rejection preserving every example file. Help/hints and worked reasoning remain optional. The capstone uses a captured function argument list: quoted "$@" does not re-expand names after creating a new .log report.

## Scope and evidence

Only simple * selection is authored in this unit; ? has capability/reference tests but is not counted as authored instruction. C/POSIX byte order, flat virtual /work files, no directories/bracket/recursive patterns, no host lookup. Supplied filenames have no newlines or leading hyphens; capstone destination inputs are canonical relative names without wildcard characters. The full map/state is graded, including preserved sources and unwanted file creation. See SHELL_GLOBBING.md and SHELL_SUBSET.md for exact language/resource limits.

Twenty-two bundled deterministic input cases verify behavior, with alternate supported solutions rather than exact source matching. Negative tests cover quoted patterns, unquoted selected filenames, truncation inside the loop, treating normal no-match status as batch failure, creation before selection, missing empty-name guard, damaged files/results, overfit first-case scripts and current success after edits. A24-source supplemental test checks a larger captured batch; numerical expansion is not another authored lab. Reset/resume checks preserve separate drafts and original55-attempt data.

All authored/alternate methods and examples are fixed development fixtures for disposable Bash comparisons. Reference Bash never receives learner/imported source and is absent from the application's execution path. The controlled reference corpus has601 fixtures; runtime/UI evidence and independent findings are recorded in IMPLEMENTATION_LEDGER.md after final checks. Synthetic native captures remain ignored local artifacts, not public learner recordings.

## References and coverage

Checked2026-10-22 against the current official LPIC-1 v5.0 source copy and unchanged objective fingerprints: https://www.lpi.org/our-certifications/exam-101-102-objectives/ . Narrow103.3.K4/T19 simple wildcard practice is partial; advanced/directory/bracket patterns, wildcard cp/mv/rm in the separate literal terminal and full file-management coverage remain absent. No exam endorsement or preparation guarantee is claimed.

Technical references: GNU Bash [Filename Expansion](https://www.gnu.org/software/bash/manual/html_node/Filename-Expansion.html), [Pattern Matching](https://www.gnu.org/software/bash/manual/html_node/Pattern-Matching.html), [Word Splitting](https://www.gnu.org/software/bash/manual/html_node/Word-Splitting.html), [GNU grep](https://www.gnu.org/software/grep/manual/grep.html), and [POSIX shell language](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html). GNU web nodes timed out during the capability check; the official installed Bash5.3.9 manual nodes and controlled reference comparisons supplied local technical verification. Runtime budgets and virtual namespace are simulator policy.


## Final verification

260 Rust tests pass, including five content/persistence regressions and two native-egui layout/keyboard checks. Ctrl+Enter runs each new assignment; Esc then Tab releases editor focus to navigation. Formatter/Clippy,21 Python tooling/boundary checks, all28 palette checks and official objective fingerprints pass. All601 fixed disposable Bash comparisons agree. Independent14 targeted teaching/content tests and601 reference comparisons pass.

Independent review found a missing empty-destination graded case. It is now one of the capstone’s eight cases, and removing its guard fails; ordinary success includes an empty source instead of spending a separate case on it. Review also corrected a self-referential prerequisite note and linked the exact return lesson. No remaining material issue was found.

Actual Fedora XWayland native flow passes all three introductions, example runs/declared experiments with separate drafts, incorrect assignments and keyboard repair/Run, current completion, save/reload, keyboard edit invalidation, selected-only reset, light/dark150% scrolling/Challenge, and desktop/narrow unit library. Fresh rendered screens were reviewed. These are agent renderer/input checks, not an independent novice study, screen-reader or Windows/macOS desktop acceptance. Default-Wayland startup investigation remains open. Local captures: shell-file-manifest-*, shell-collect-errors-*, shell-file-batch-capstone-* and file-batches-* under ignored screenshots/workspaces. No release published.
