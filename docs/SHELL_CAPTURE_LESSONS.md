# Command output, decisions and report previews

Authored and primary-source rechecked2026-10-23. Four distinct assignments follow the accepted bounded evaluator7 capability; this document describes their scope, not complete Bash/POSIX or certification preparation. The normal offline library bundles unit `shell-command-capture`, with advisory unit prerequisites `shell-text` and `shell-scope`. Existing IDs, goals and prior progress are unchanged. Numerical variants are not new authored labs.

| Assignment | Practical task | Independent cases |
| --- | --- | --- |
| `shell-capture-preview` | Combine independently sorted unique requested/available lists while retaining each multiline value and source files |6: ordering, independent duplicates, one/both empty, spaced/literal/tab/UTF-8 text with wildcard decoys, unfinished records and changed reviewer |
| `shell-capture-evidence` | Replace a report with intact captured evidence, validating before a destructive redirect |9: ending/leading/interior newline differences, spaces/CR/literal text, empty notes, missing source, collision and empty destination |
| `shell-capture-decision` | Repair a helper investigation, keeping findings, diagnostics and meaningful status separate |6: hold, ready, empty existing source, multiple/unfinished/literal findings, substring match and explicitly guarded missing source |
| `shell-capture-handover` | Preflight a whole batch, preserve caller title, append an inspection ledger and preview or save a complete report |12: four normal cases and eight independent guard failures |

The capstone separately tests missing arguments, late empty/missing/colliding sources, ledger-as-source, unknown mode, empty destination and ledger-as-destination. Combining unrelated invalid inputs could hide an omitted guard. Repeated sources and a clear final source after an earlier hold are included in normal cases. This is distinct reasoning, not title/number inflation.

## Teach, try, then complete the whole task

Introductions teach each new idea before asking for it. Different runnable examples explain printed output as a value, quotes, removal of all ending LF characters, helper status versus fatal simulator diagnostics, and child variables versus shared files. Each has a concrete change and a repair. Visible `\n`, `\t` and `\r` representations are explained before evidence comparisons use them. Supplied values and learner-created variables are distinguished.

Introduction, Try example and Assignment remain skippable/revisitable views of the approved workspace. Running/changing an example affects only its own draft/results and isolated sample files; it cannot grant assignment credit. Learn and Challenge keep the same full assignments and native script editor. Optional help does not replace essential instructions; Challenge does not expose worked solutions automatically.

A meaningful hold uses status1 and can be correct. Status2 from the missing-source helper is an explicit supported guard/return, not recovery from a fatal interpreter error. Feedback compares expected and actual output/status/files and directs retry. The status message now explains that a nonzero status may be the intended result. Unsupported syntax, malformed syntax and resource/Stop failures stay distinct from ordinary script results.

## Fidelity and grading boundaries

- Only the supported original-source dollar-parenthesis form is taught. Existing printf/cat/sort-u/grep-F, quoted arguments, functions/return, conditions, loops, shift and command redirects use the bounded in-process VM. No host executable, dependency or terminal expansion is added.
- Capture removes every final LF, preserving leading/interior LF, spaces, tabs, CR, backslashes and literal shell-looking output. Quotes preserve the remaining value; output is never re-executed as source.
- Child variable changes stay local to the capture, but virtual files remain shared. Preview deliberately appends inspection ledger entries while preserving the destination. Validation must finish before effects; no transaction/rollback promise is made for Stop.
- Supplied inputs are flat canonical virtual filenames, without path aliases or leading hyphens. Essential tasks say “Use the supplied filenames exactly”; detailed filename/shell limitations are secondary. Backticks, process substitution, arithmetic captures, custom IFS and the other exclusions in SHELL_SUBSTITUTION.md remain unsupported.
- Each case grades stdout, stderr, status and the entire final virtual file map. It never requires a capture token, exact variable names or one expected program. Two substantive alternatives per task pass, including a supported non-capture capstone reconstruction. That review-only alternative temporarily uses the ledger as a sink before restoring the required final map; it demonstrates final-state equivalence, not transactional safety, and is not recommended learner guidance.
- An edit invalidates current success until its own Run; old source/results/credit remain distinct history where appropriate. Reset affects only the selected attempt. Drafts, examples, favorites and other labs survive save/reload.

## Evidence and source identities

Root independently calculated all33 case expectations from separate line/set/string and explicit guard specifications, without evaluating reference scripts or using VM observations. They agree exactly, including whole-file maps. Seven normal-Cargo content regressions cover reference/two-alternative behavior,16 plausible wrong programs and partial results, first-case hardcoding, unrelated-file damage, changed-source invalidation, all default/changed/repaired examples, explicit missing-source status and all86 prior attempts plus new draft/results/reset roundtrips. Two native-egui regressions check the normal bundled lessons at100/150/200% and status1-versus0 feedback.

The initial actual Fedora native all-four flow passes introduction/example/change, separate assignment state, keyboard source edits/Run, partial feedback, Step0 completion without walkthrough clicking, exact resume/example retention, stale-success invalidation, actual selected Reset confirmation, library filtering and Challenge light150/dark200 reachable controls. Reference and negative source text is supplied through synthetic native keyboard events. Root reviewed actual introductions, complete capstone assignment, partial feedback and large-text controls. This technical evidence is not a beginner study, physical-device test, screen-reader acceptance or Windows/macOS desktop acceptance. Final bundled regression and build identities belong in IMPLEMENTATION_LEDGER.md/INSTALLATION.md; an older86-lab candidate does not include this unit.

The original author also recorded315 fixed VM observations and7 standalone checks before the memory-safety workflow. Those are historical evidence; future checks use the main `tools/dev.py` guard and project disk paths, never the old raw compiler or RAM/tmp recipes. No new host-Bash comparison is claimed for these lessons. The accepted runtime already has controlled fixed Bash5.3.9 comparisons; those tools never become the learner engine.

Primary grounding:

- GNU Bash5.3.9(1)-release, manual Edition5.3: [command substitution](https://www.gnu.org/software/bash/manual/html_node/Command-Substitution.html), [word splitting](https://www.gnu.org/software/bash/manual/html_node/Word-Splitting.html), [simple-command expansion](https://www.gnu.org/software/bash/manual/html_node/Simple-Command-Expansion.html), [execution environment](https://www.gnu.org/software/bash/manual/html_node/Command-Execution-Environment.html), [functions](https://www.gnu.org/software/bash/manual/html_node/Shell-Functions.html). Root re-read installed original info sections3.5.4/3.7.1/3.7.3; decompressed manual SHA256 `b039902cb1d0a959818d7db87f9627d1aab4f623bdb359d2edd4d1fb89682b92`. Direct GNU page retrieval timed out; no successful rendered-online retrieval is claimed.
- [GNU sort](https://www.gnu.org/software/coreutils/manual/html_node/sort-invocation.html) and [GNU grep status](https://www.gnu.org/software/grep/manual/html_node/Exit-Status.html) inform the existing finite C-byte line models. They do not imply full flags, locale or missing-file diagnostic fidelity.
- Current [LPIC-1 v5.0 objectives105.2](https://www.lpi.org/our-certifications/exam-101-102-objectives/#105.2), checked2026-10-23, explicitly includes command substitution and interpreting return values. Only105.2.K2 gains narrow Partial practice; parent/K3 evidence strengthens. Mail, executable/shebang/ownership policy and wider shell language remain absent. Full source inventory fingerprints retain2026-10-22; K2 wording fingerprint matches the current page.

All authored code/data/prose is original synthetic MIT content. No GNU manual prose or implementation was copied into the curriculum; all existing third-party licenses/notices remain intact.
