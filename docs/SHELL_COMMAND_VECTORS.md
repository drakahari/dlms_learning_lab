# Command names and argument lists

The existing evaluator10 accepts a quoted scalar command name or a sole quoted
Bash array expansion as an ordinary command's words. This milestone tests those
existing semantics; it adds no parser feature, evaluator revision or authored lab.
The normal catalog remains147 assignments, including46 scripts. Three new
construction/dispatch tasks are being authored separately and are not yet counted.

A command is a program/function name followed by separate argument strings.
`command=(sort -r -- "$source")` followed by `"${command[@]}"` preserves each
argument, including spaces and empty strings. This array syntax is Bash-specific.
A quoted scalar name plus separately quoted arguments is a different supported
portable-style technique; quoting a whole command string does not split it into
an executable request. Expanded text is never reparsed as shell syntax.

Quotes do not validate a program name or disable a utility's option processing.
Choose supported operations through a closed case mapping. The supported sort,
uniq and cut accept `--` to end options before an operand. A sole `-` still means
standard input, including after `--`; the proposed file-processing tasks explicitly
reject it as a source name. Fixed printf formats keep format-like input as data.
These techniques are not a universal security guarantee for arbitrary utilities.

User functions can be dispatched in ordinary calls. Function calls in pipelines
or with call-level redirections remain unsupported; redirect individual commands
inside a function instead. Declarations such as local retain their literal-name
restriction. Unknown/external names return the simulator's Unsupported diagnostic,
not real Bash PATH lookup or ordinary command-not-found status127. No executable
script invocation, eval, source, host process, filesystem or network is introduced.
All existing instruction, output, expansion, memory, array and Stop limits apply.

## Verification

Root ran12 in-process groups: six basic command-head tests and six utility groups
with32 literal cases/63 vector-versus-direct or raw-control pairs. These cover
argument boundaries, literal hostile-looking data, function status, allowlists,
optional flags, spaced/option-looking source names, per-job vector replacement,
missing/invalid plans, destination preservation and equivalent direct branches.
The same scoped run includes the existing shell, arrays, text, case, substitution,
expansion-limit and safe-automation checks:93PASS/0FAIL across nine targets.

The existing fixed development reference helper now also reads a second immutable
fixture file. Original657 comparison fixtures remain byte-exact;63 explicitly
reviewed utility/source/case pairs bring the controlled Bash5.3.9 comparison to720,
all matching output, diagnostics, status and complete disposable file state.
Only checked-in fixed programs and sample inputs run through that helper, under
the MAIN memory-safe workflow and disk-backed temporary directories. Unsupported
external-head simulator tests are deliberately excluded from host comparisons.
Reference Bash never becomes the learner execution engine.

Primary sources: original installed GNU Bash Reference Manual5.3 (Arrays, Simple
Command Expansion, Command Search and Execution), and GNU Coreutils9.10 manual
(Common options, sort, uniq and cut), read2026-10-27. These explain the scoped
semantics; this evidence is not full Bash/Coreutils conformance or novice acceptance.
See SHELL_SUBSET.md for the current complete supported/excluded subset.
