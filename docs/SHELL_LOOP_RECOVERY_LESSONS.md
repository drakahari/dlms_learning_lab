# Loop control and independent recovery

This unit adds two different practical assignments to the existing scripting
path. Numeric variants are not extra labs. Learn and Challenge retain the same
full script assignment; Learn supplies a skippable introduction, an isolated
runnable example, progressive hints and a requested worked explanation.

- **Stop intake without losing the tail:** read a finite stream, distinguish
  ignored entries from invalid records, stop at an exact marker and forward all
  remaining input unchanged. Warnings and a missing marker have distinct exit
  statuses. The source/audit file and unrelated files must remain untouched.
- **Recover each delivery job independently:** validate a request before deriving
  filenames; choose the first ready candidate within each job's small attempt
  budget; publish its complete contents only after selection; preserve failed
  destinations, including their absence; continue other jobs and return an honest
  combined status. Earlier successful jobs remain published when a later job fails.

The first introduction builds on the existing line-preservation, counter and case
lessons. It explains skipping an iteration versus leaving a loop, exact markers,
unread input and status. Its gallery example uses different values/files. The
second introduction builds on that task and the existing numeric-validation,
function-return and variable-scope work. It teaches per-job validation/order,
read's EOF status, selection before writing, independent helper variables and
aggregate errors. Bash `local` is identified as Bash-specific; one accepted
alternative uses portable functions with distinct working variable names.

Grading compares complete output, error output, status and virtual-file contents
across10 and12 authored cases. The capstone includes valid one-character,
digit/hyphen and eight-character IDs, and an unavailable job whose destination
must remain absent. Supplemental cases preserve the earlier budget/no-argument
checks, malformed paths, line endings and overflow boundaries. References and two
alternatives are checked independently; wrong, partial, first-case hardcoded and
file-damaging programs have explicit expected pass/fail rows. All original
capstone cases/masks remain replayed after the two strengthened graded cases.

All execution stays in the bounded in-process interpreter. `break` and `continue`
support only this function/substitution scope's `for`/`while` loops, with optional
canonical counts1–24. Caller-loop and pipeline escape are unsupported; this is a
subset of Bash, not general Bash conformance. Cron and one-off scheduled command
seams remain deliberately narrower and reject these control structures. No
network retry, wall-clock waiting, concurrent jobs, locks, atomic rename or crash
recovery is modeled.

The original55 authored script specifications,420 cases and55 separate examples
remain byte-pinned. The exact two-task addition is22 cases and two isolated
examples. Existing histories and genuine evaluator9–12 evidence are not rewritten.
The new objectives receive no invented learner answers or completion credit.
An edited successful source requires another actual Run before its result can be
current. Examples cannot award assignment credit; reset affects only that attempt.

Authoritative references (checked2026-10-29):

- [GNU Bash Bourne Shell Builtins](https://www.gnu.org/software/bash/manual/html_node/Bourne-Shell-Builtins.html)
- [GNU Bash Looping Constructs](https://www.gnu.org/software/bash/manual/html_node/Looping-Constructs.html)
- [POSIX break](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/break.html)
- [POSIX continue](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/continue.html)

The fixed developer comparison separately observed16 supported loop-control
programs against GNU Bash and retained eight explicit VM-only exclusions. That
capability evidence is not a claim that every authored solution, full application
flow, beginner understanding or other operating system has been accepted. The
normal182 registration now passes the full inclusive suite, static checks and
actual82-capture Fedora learning flows. See CATALOG182_MILESTONE.md for exact
source/ELF identities and remaining cold/installed/platform acceptance limits.
