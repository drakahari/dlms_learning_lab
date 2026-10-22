# Shell composition unit

Authored and checked 2026-10-22. Portable syntax within [SHELL_SUBSET.md](SHELL_SUBSET.md); this is not a full shell or certification coverage claim. Requires the seven foundations assignments, but prerequisites remain advisory.

| Assignment | New learning and practical task | Cases |
| --- | --- | --- |
| Route output without mixing messages | stdout/stderr/file channels; replace a dispatch slip while retaining normal and problem messages separately | 3: spaces, percent signs, empty label, previous report and unrelated file |
| Select the intersection of two log filters | Pipe data between literal filters; retain ordering and last-stage status | 5: mixed severity/component, spaces, no matches, unterminated input, `ERROR:` substring |
| Keep the final record when input ends | `while`, `read` status and `||`; repair lost final input without losing blank complete lines | 6: complete/unterminated/blank/empty input, backslashes, outer whitespace |
| Audit a batch without hiding failures | Capstone: retain failures, continue useful work, distinguish normal no-match from missing file | 5: matches, missing first, quiet+missing, no files, readable quiet file after a match |

Each has a skippable introduction teaching the new concept, runnable example with different values, suggested edit, independent assignment draft, progressive hints and optional worked reasoning. Example runs never count as assignment credit. `read -r` uses the simulator’s disclosed normal trimming: a final unterminated whitespace-only value becomes empty; a complete blank line is still processed.

Review fixtures in `tests/fixtures/shell-content-review.json` cover all eleven shell assignments with a supported alternate and a plausible wrong solution. Tests additionally construct a case-1 hardcoded program that must fail other inputs. Explicit negatives reject treating grep no-match as a batch failure and narrowing literal ERROR to ERROR-plus-space. This establishes tested examples, not proof against all possible incorrect programs.

Independent read-only content review found the two missing cases above and a feedback omission. All were addressed; guidance now appears for any behavior mismatch, including file-only/status-only errors. Full curriculum completion and novice-user acceptance remain separate gates.

Native Fedora synthetic-profile review traverses all four introductions, example runs and suggested edits, assignments, wrong-answer retry, typed correct solutions and completion; also light/dark 150% rendering. Screenshots are local ignored artifacts. Automated UI checks cover 720/1200 widths, 100/150% text, both modes, all introductions and practice editors. This is not physical keyboard/device, assistive-technology, Windows or macOS acceptance.
