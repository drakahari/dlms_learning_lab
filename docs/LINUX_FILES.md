# File-management foundations

Six distinct LPIC-1 assignments, version1, in unit `linux-file-foundations`. Checked 2026-10-22 against current LPI 5.0 [103.1/103.3/104.5 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/). They practice narrow parts of those objectives; no full objective/exam coverage claim.

| Assignment | Taught before assessment | Independent outcome |
| --- | --- | --- |
| Find a project and prepare its handover | Terminal input/Run, directory/path, pwd/ls/cd/cat/cp | Locate Aurora, retain source, make handover copy and finish in its directory |
| Package notes with spaces | Literal quoting, mkdir, explicit copy destination | Create named folder and rename copied notes without losing original |
| Keep a snapshot and process a receipt | Copy vs move, overwrite caution, inspect both sides | Retained ledger snapshot and relocated receipt; equivalent copy-then-remove accepted |
| Clean a staging folder | Hidden names, ls -a, precise rm, empty-only rmdir | Delete named disposable entries while retaining visible and hidden release evidence |
| Create a private working area | Owner/group/other, r/w/x, octal numbers, creation mask and touch | Owner-only new folder/file; original announcement and home permissions retained |
| Prepare a safe project handover | Review only; combines preceding lessons | Private delivery, retained report, processed queue, precise cleanup, retained supplied directories and final working location |

Prerequisites are advisory, not access locks. LPIC-1 Start course uses unit order, beginning here, while old scenario IDs/order and existing attempts remain unchanged. Challenge opens the same complete assignments without introductions or unsolicited solutions.

Each lesson has a skippable/reopenable introduction, runnable commands using different names/values and a specific suggested experiment. Try example uses a separately saved Tree, input and transcript, including its own working directory/mask. Insert only populates input. It cannot change assignment state, transcript, command draft or credit. Reset example only resets practice; normal Reset confirms resetting just the selected attempt.

Content validation checks bounded fields, valid fixture trees, runnable commands and declared resulting example outcomes. Tests execute each actual suggested experiment, all reference solutions, distinct alternate sequences, plausible mistakes, partial work and breaking every objective after completion. Current-state goals include retained text/directories and mask; command histories are not graded. Existing progress schema1 is extended with optional `terminal_learning`; old saves remain readable and no answers/mastery are invented.

The [filesystem subset](FILESYSTEM_SUBSET.md) defines real command semantics and explicit exclusions. No shell operators/expansion/globs, recursive operations, links, devices or host access. Ordinary user permissions are modeled; older administrator permission/service labs retain their own subsets. Creating with the wrong mask can be retried by removing/recreating the affected empty entries (or resetting this selected attempt); changing umask alone never edits existing permissions.

Author in `scenarios/linux-files-v1.json`: initial typed Tree, typed requirements, reference sequence, guidance and `terminal_teaching` with isolated fixture/commands/outcomes. Add an alternate, a plausible incorrect/partial sequence, retention negatives and suggested-change assertions in `tests/terminal_teaching.rs`. Do not add an unimplemented command to make a lesson appear complete.
