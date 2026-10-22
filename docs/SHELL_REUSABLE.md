# Reusable functions and safe composition

Authored 2026-10-22 after M6’s bounded evaluator extension. Four distinct assignments build on shell foundations and composition. Portable syntax within the explicit [subset](SHELL_SUBSET.md); global scalars are not silently treated as local variables.

| Assignment | Practical skill | Cases |
| --- | --- | --- |
| Reuse a label printer | Define/call a reusable helper, keep per-call arguments distinct, preserve spaces and empty values | 4: multiple labels, spaces/percent, empty label, empty queue |
| Reject one item without ending the batch | Repair exit vs return; continue useful work and remember earlier failures | 5: all valid, rejected first/middle, two rejected, empty batch |
| Separate a header from remaining items | Validate and consume a leading setting; preserve quoted remaining arguments | 5: several labels, spaces/empty values, header only, missing/empty header |
| Assemble a bundle only after preflight checks | Capstone: validate every source before replacing a destination; preserve exact text bytes and unrelated files | 7: valid multi-file, whitespace/unterminated input, missing first/later, no sources/arguments, invalid batch with a new destination |

Learn retains Introduction → Try example → Assignment, separate editable example and assignment drafts, progressive hints and optional worked explanation. Challenge keeps the complete task without unsolicited examples. The curriculum teaches reusable structure; grading compares behavior, so equivalent supported programs—including correct inline implementations—pass. It does not infer mastery from source shape.

Independent read-only review found a missing invalid-input/new-output case: opening a new destination with `>>` before validation still mutated state. The added case and explicit wrong-solution regression reject it. Review also clarified the sample experiment and the ordinary-filename boundary: supplied filenames do not begin with `-`; cat’s full option parsing is not modeled.

All fifteen script assignments have tested supported alternatives, plausible mistakes, hardcoded-first-case rejection and stale-result invalidation. The new four also exercise partial success. Curated reference/alternate/example cases are compared with real Bash only in disposable development fixtures. Native Fedora synthetic-profile flow reviews all four introductions, runnable examples/edits, assignments, incorrect feedback and completion, plus larger-text light/dark scrolling. This is not an independent beginner study or Windows/macOS runtime acceptance.
