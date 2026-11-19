# Bounded command tracing — evaluator16

Current capability gate:36 fixed real-Bash comparisons pass against the source-bound
in-process VM, after preserving/sealing genuine15. Native lesson/candidate review
is still separate. This is a deliberately narrow tracing subset, not full Bash
conformance and not complete debugging/error-handling curriculum coverage.

## Supported behavior

- `set -x` / `set +x`, and `set -o xtrace` / `set +o xtrace`.
- Programs containing scalar assignments, supported simple commands, nested
  `if`/`else`, `&&` and `||`. Existing unsupported `elif` remains unsupported.
- Commands are traced after their arguments expand and before the builtin runs.
  Each standalone scalar assignment shows its evaluated value. Only reached
  commands/assignments appear; trace output does not change `$?`.
- Default `+ ` prefix, fixed C-locale diagnostic quoting. Empty command arguments
  appear as `''`; empty scalar assignments appear as `name=`. Quotes group the
  displayed arguments. Control/UTF-8 bytes use explicit ANSI-C/octal notation.
- Traces use the original shell standard-error destination. A command's `2>` or
  `2>&1` does not redirect the shell's own trace. Normal program output and file
  redirection continue through the existing simulated descriptors.

Enabling tracing validates the entire immutable parsed program shape. A loop,
pipeline, case statement, function, array assignment, command substitution or
prefixed command anywhere in that program is refused explicitly, including after
a later `set +x` or inside an unselected branch. Ordinary nontracing support for
those existing constructs is preserved. An unexecuted `set -x` does not activate
tracing. No dynamic custom PS4, BASH_XTRACEFD, `set -v`, combined option flags,
external processes, host files, signals or real OS debugging is implemented.
These limits must be taught before an assignment invites tracing.

## Isolation, resources and progress

This module hooks the existing parsed command/assignment continuations. It does
not recognize authored answer strings or route learner input to Bash. It adds
no separate log or saved execution schema. Formatted trace bytes share the32KiB
output limit and256KiB memory ceiling. Formatting uses measured quoted lengths,
bounded temporary ownership and the existing cumulative expansion-work ledger.
Cooperative ticks and Stop remain in effect; an interrupted or resource-failed
run cannot earn completion.

The evaluator revision advances to16 because previously unsupported commands
now have defined behavior. Existing Records/fixtures remain unchanged. Actual
current-versus-genuine15 tests preserve all549 existing assignment outcomes and
66 examples, draft/history/favorites and exact before-v16 backups. Old completion
is visibly historical until an explicit current run; no example or migration
fabricates independent credit. A new lab must grade output/status/files against
multiple deterministic cases and accept supported alternate implementations;
matching the text or number of trace lines is not its independent assignment.

## Actual development evidence

All reference inputs are fixed authored development fixtures, never learner work.
The MAIN memory-safe runner executes the pinned Bash descriptor in fresh0700
project-disk folders, with an empty PATH, no startup files, fixed C locale, stripped
environment and bounded CPU/address space/output/files/time. Recipes use only
Bash builtins and a few fixed relative output files. Each actual row is synced
before continuing; the exact observer and fixture are retained. No personal files
or installed-app profile are accessed.

- 30 command/state/branch/redirection observations, report SHA
 `74b165cd820820bd06cbe653cbca7da3042380a595225efb689f37b95e745fa8`.
- 4 punctuation/control/selected-word-format observations, report SHA
 `dae6452bbae9b5bab6cd26c3df4ceeac91ba6f85659b0f6fea0d2f0cfa49ce0f`.
- 2 empty-assignment/apostrophe-edge observations, report SHA
 `330ecc46fc4d4f2d8c32aab12145870d300eee7211f6d1c736e7dab1d57f6e9c`.

Each observed batch was independently read back before sealing. This is not an
exhaustive enumeration of all Bash programs, ASCII strings or locale behavior.
The actual in-process comparison in `compare-iyvdisrn` passes all36 full outcomes,
including stdout/stderr/status and complete file maps. Report SHA
`a5c49a42666cf77060e6ad8d21c20214142ea3b1ec1463bfbdf514bdb0cabb72`.
All25 compiled-source/data pins match; the same held-descriptor ELF and all inputs
are retained before future edits. Probe SHA
`184c1ba3da87f3ee70262cc0c0cf215868063a17fb49d84f15e1045edbabef15`.

Earlier `compare-wim90gax` remains failed/held. Its runtime freshness gate caught
a probe built before the small unsupported-elif wording correction. No comparison
was accepted from that run; its ELF, mismatched source snapshot and raw execution
remain intact. The corrected probe was rebuilt before the successful comparison.
No old collector/producer/closure or sealed baseline was edited to make it pass.

## References and remaining work

GNU Bash Reference Manual, [The Set Builtin](https://www.gnu.org/s/bash/manual/html_node/The-Set-Builtin.html)
and [Quoting](https://www.gnu.org/s/bash/manual/html_node/Quoting.html), checked
2026-11-02 local; the official indexed set section was retrieved after a direct
fetch timeout. Local reference Bash identity is retained in each actual report.
No GNU source implementation was copied into the MIT project.

Broader compound tracing, traps/cleanup/error propagation, custom trace settings,
real scheduling/processes and the remaining safe-automation roadmap stay open.
There is no new Windows/macOS runtime, screen-reader, physical-device or independent
novice acceptance from these controlled developer comparisons.
