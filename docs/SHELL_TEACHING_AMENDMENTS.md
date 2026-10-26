# Five scripting prerequisite corrections

2026-10-26. This improves five existing assignments in the46-lab scripting path;
it adds no title, capability, prerequisite gate or exam-coverage claim.

| Existing assignment | Teaching added before the assignment |
| --- | --- |
| Text-processing capstone | Argument count, numeric inequality and exact filename comparison |
| File-bundle capstone | Minimum argument count before checking source files |
| File-batch capstone | Conditional AND and comparison versus assignment |
| Case interface | The equivalent `test` and `[ ... ]` spellings |
| Array priority report | printf format reuse and the empty-array caveat |

The text and bundle practice defaults now demonstrate their guards using the
same sample inputs and declared outcomes as before. Their suggested experiments
show a rejected count or filename collision, then repair. Practice remains
ungraded and separate from assignment drafts. Introductions remain skippable
and can be reopened. Challenge assignments and optional guidance are unchanged.

## Saved work and historical evidence

Assignment IDs, versions, order, prerequisites, starters, goals, test cases and
reference solutions are unchanged. Existing example sources/results and assignment
drafts are never replaced on resume. Resetting the selected attempt receives the
new default without affecting other attempts or favorites.

Original pack bytes and evaluator captures remain authoritative historical evidence.
The test-only `shell-teaching-amendment-v1.json` manifest permits exactly13 teaching
changes across five named packs. It pins before/after bytes and reconstructs each
original pack to its original hash. Historical comparisons replace only those
reviewed teaching values after verifying the complete current Lesson; changes to
goals, inputs or other scenario fields remain visible and fail the original checks.
The application has no amendment loader or historical projection layer. Frozen
source-handoff metadata in the manifest records that handoff, not current test status.

## Verification

The focused six tests run all46 current practice examples, both retained old
defaults, suggested guard experiments and repair, exact writer8 history, draft
retention, export/restore and selected reset. The eight affected integration suites
pass88 checks including the explicit historical public9/current comparison.
MAIN's complete regression passes1653 checks with11 private-evidence checks
excluded from its ordinary run. All11 private-evidence checks also passed in an explicit guarded MAIN run.
MAIN all-target Clippy and211 Python workflow checks pass. Native review remains
pending; this evidence is not a novice user study or assistive-technology acceptance.
