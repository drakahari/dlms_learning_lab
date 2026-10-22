# Bash scope and safe helper design

Authored2026-10-22 after M15’s bounded scope implementation. Three distinct debugging assignments continue the reusable-functions unit. They use Bash-specific `local`, explicitly distinguish it from portable `sh`, and accept equivalent supported behavior such as explicit arguments or separate helper names.

| Assignment | New practical reasoning | Deterministic cases |
| --- | --- | --- |
| Protect the report heading | Find a scratch-variable collision; preserve the caller's title across calls and empty/spaced items |4: several items, empty list, spaces/percent/empty items, same-looking item |
| Keep two topic reviews independent | Trace the setting visible through a nested call; isolate filter values from the final summary; continue after no matches |5: two topics, one quiet topic, literal spacing/percent, repeated topics, no matches |
| Repair a review that damages its inputs | Capstone: a checking helper must not redirect subsequent writes to a source; verify files as well as messages |8: existing output, repeated quiet input, spaced names/absent output, later missing source, missing source/new output, output/input collision, no sources, empty pattern |

Each introduction states prerequisites, teaches the new idea before assessment and offers an editable example with different data and a specific change to try. Examples run in the existing VM with separate persisted drafts and cannot award assignment credit. Learn and Challenge retain the same complete task/editor; optional hints and worked reasoning do not appear automatically in Challenge.

The subset deliberately uses single-line nonempty literal patterns that do not start with `-`, and canonical single filenames without directory parts. These restrictions appear in the assignments. Raw filename equality is sufficient only for those supplied inputs. It is **not** general filesystem-alias protection: real links/alternate paths can name the same file. The capstone explicitly teaches that limitation. No real files or commands are touched by learners.

Reference programs, supported alternatives, plausible incorrect programs, partial behavior and hardcoded-first-case rejection are exercised. The capstone separately rejects opening a new output before validation and deduplicating repeated source arguments. Existing saved attempts remain under unchanged IDs/versions; newly added tasks do not grant earlier learners new mastery. Scoped variables remain bounded/cooperative and Stop uses the existing VM cancellation.

Sources: official GNU Bash function/builtin manual URLs are embedded in each scenario. Direct online retrieval timed out on2026-10-22; the installed GNU Bash5.3.9 manual and controlled disposable fixture comparisons verify the taught subset. See [SHELL_SUBSET.md](SHELL_SUBSET.md) for exact semantics and exclusions. This unit adds local-scope practice, not complete advanced scripting or certification coverage.
