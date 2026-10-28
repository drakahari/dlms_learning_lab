# Read-only fallback and alternate values

Implemented and gated on MAIN2026-10-28 as evaluator11, before dependent
curriculum registration. Full inclusive2045PASS/0FAIL/0ignored across159 harnesses,
including all19 retained-history checks; all-target Clippy/246Python pass. A real
Bash5.3.9 comparison accepts58 supported cases plus one confirmed malformed
proposal, with no unexpected mismatch. Genuine evaluator10 profiles and exact
raw backups remain protected. This adds no authored labs or certification credit
and does not claim full Bash conformance. See ledgerM173–M176.

## Supported contract

These four forms are portable shell syntax within the restrictions below:

| Form | Use literal WORD when | Otherwise use |
| --- | --- | --- |
| `${name-WORD}` | name is not set | its value, including empty text |
| `${name:-WORD}` | name is not set or is empty | its nonempty value |
| `${name+WORD}` | name is set, including empty text | empty text |
| `${name:+WORD}` | name is set and nonempty | empty text |

A variable supplied as an empty string is set. A local declaration with no
assignment is initially unset unless the existing array-type rule applies.
The same distinction applies to a missing positional argument versus an
explicitly empty one. The operators read values; none assigns a variable.
Selecting an alternate tests presence/emptiness without copying the existing
value. Selection is lazy; an unchosen literal is never evaluated as arithmetic,
a filename, a command, or a value to append. Parsing still checks the whole
script for unsupported syntax before execution.

Targets are ordinary supported scalar names or positional1 through9. Runtime
array values, array/index targets, special variables and the reserved simulator
namespaces remain excluded. Existing ordinary parameter reads, array forms and
other rejected operators keep their prior dispatch and diagnostics.

WORD may be empty or literal UTF-8 text, including spaces, line breaks and
ordinary punctuation. It cannot begin with a tilde or contain dollar expansion,
inner single/double quotes, backslashes, backticks, braces, brackets, CR or NUL.
These excluded forms fail explicitly at compilation, even if that branch would
not be chosen. Real shells also expand WORD; this simulator deliberately does
not implement that recursive grammar yet. Quote the complete expansion instead:
`"${label:-Morning shift}"`. Single quotes or an escaped dollar keep the complete
source literal, as before.

Outer quoting, concatenation, default field splitting and the existing flat
`*`/`?` filename matching retain ordinary word behavior. A quoted empty result is
one empty argument; an unquoted empty result disappears. Expanded text is data,
never reparsed as source. The same word preparation serves assignment values,
command arguments/heads, function/local arguments, for lists, redirection paths,
command capture and arithmetic. Arithmetic source still follows its normal parenthesis matching: each parameter
must close inside its own arithmetic span, even when its fallback is unselected.
Only a syntactically valid selected value is then evaluated as arithmetic.
No new commands, host lookup, process or filesystem access is introduced.

## Bounds and evidence requirements

Use the existing16KiB source, token/nesting,128 argument/field,256KiB virtual
text/memory,32KiB output,2000 instruction and1,000,000 expansion-byte work limits.
The selected borrowed string enters existing accounted appends; no retained
alternate AST or extra execution budget is created. Repeated selections, Stop,
redirect preparation failure and full-file preservation require regression
coverage. Unsupported/resource failures remain ungraded; exact Bash diagnostic
wording is not promised.

`tests/fixtures/bash-parameter-defaults-reference.json` contains35 fixed authored
reference proposals, not recorded Bash observations. Seven integration groups
exercise those cases at multiple cooperative tick slices, independent conditional
alternatives, wrong colon choices, first-case hardcoding, excluded WORD syntax,
array/special-target rejection, laziness, limits and Stop. New migration checks
use exact actually captured historical8 bytes and genuine current runs. Genuine actual10
observations were captured from the unchanged evaluator10 checkout before
integration; no current result is relabeled10. All original49 scenarios,
350 cases, examples, goals and versions are protected.

Capability acceptance requires the guarded MAIN runner, an independent source
review, actual controlled Bash comparisons in disposable fixed fixtures,
historical10 profile preservation, existing-shell regression and all-target
Clippy. The candidate is not validated merely because this contract exists.
Only then author a small coherent unit with prerequisites, skippable introduction,
separate runnable practice and meaningful multi-case assignments. A syntax probe
is not a delivered lesson.

## Primary references

GNU Bash5.3 manual, section3.5.3
[Shell Parameter Expansion](https://www.gnu.org/software/bash/manual/html_node/Shell-Parameter-Expansion.html),
and POSIX Issue8
[parameter expansion](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html).
The original installed GNU manual was read2026-10-28 at
`/usr/share/doc/bash/bashref.html` (SHA256
`b91026acffb159bd1e842e3116a95539bf858e2b57cbfa74df9e3a53379177a4`).
Fresh direct GNU retrieval timed out; POSIX returned403. Portability here names
the four standard forms, not unverified full shell or WORD behavior. Reference
Bash is development evidence only and never the learner execution engine.

## Colon fidelity correction in evaluator11

The prior subset listed `:` but accepted it only without arguments. Evaluator11
corrects that mismatch: colon discards any successfully prepared arguments and
returns0 with no output. Arguments still expand, captures still have their normal
virtual effects and redirections still run in order. Failed expansion/resource
checks do not become success; missing input is the usual recoverable command
status and a later failed redirect does not roll back an earlier truncate.
No command-prefix environment assignment or new special-builtin environment
semantics is implied. `true` and `false` retain their separately disclosed
no-argument subset. Six additional fixed colon proposals are separate from the
unchanged35 parameter inputs; three new groups and the original colon-based
instruction/Stop test verify preparation, effects and bounds. Exact fatal-error
wording/status remains simulator-specific as documented above.

## Required safe-file operand support

For the dependent report assignment, evaluator11 also introduces the bounded
cat grammar above. Rejecting every option-looking source would artificially
narrow useful practice; `cat -- "$source"` now preserves a single quoted file
operand without interpreting a leading dash as a flag. A single `-` still means
finite stdin after the delimiter. Bare cat and cat with only the delimiter also
read finite stdin. Multiple files and all flags remain excluded. This changes
only ordinary builtin operand selection; word preparation, redirects, virtual
paths, file/status behavior, memory/work/output limits and Stop remain shared.

Reference checked2026-10-28: GNU Coreutils9.10 original installed info manual,
cat invocation and Common options, SHA256 of coreutils.info.gz
`6e28a10ae4e4acad5c0147348ffe0a0a0cea4f93ed7cd794d434d953059e4716`.
Fresh [GNU cat page](https://www.gnu.org/software/coreutils/manual/html_node/cat-invocation.html)
retrieval timed out. No manual text/code or host tool is bundled by this model.
Thirteen additional fixed cat proposals stay separate from the immutable35
parameter and six colon proposals. Their expected values are authored inputs,
not actual Bash observations. Their runtime and controlled-reference gates now
pass; captured observations remain separate and private.

## Confirmed arithmetic-boundary amendment — 2026-10-28

The first controlled GNU Bash5.3.9 capture produced54 observations:53 matched
the original proposed expectations. Proposal32, “arithmetic ignores parentheses
in unselected literal,” was wrong: unmatched `))))` inside arithmetic source
closes that arithmetic span before the parameter’s brace. Real Bash reports a
bad substitution/status1 with no output. Lazy value selection does not exempt
source from lexical parsing. The wrong expected8, all35 original inputs, genuine10
results and the complete failed-reference evidence remain unchanged.

`tests/fixtures/bash-parameter-arithmetic-amendment.json` explicitly identifies
that one source/input and retained observation hashes. It is exercised as a real
compile-time Syntax failure, never skipped as if it were a successful case. Of
the original35,34 are supported (33 newly supported forms plus one literal
control); one is confirmed malformed. Balanced selected grouping and unselected
`1/0` remain supported. Ordinary non-arithmetic fallback text may still contain
literal parentheses. Five additional balanced controls are separate fixed
proposals now passing controlled comparison, not rewritten original observations.

The candidate restores ordinary arithmetic lexical scanning and validates each
parameter and nested arithmetic expression within its own borrowed span. An
invalid nested close cannot borrow a later brace from an outer expression. No
extra payload copies or new language constructs are introduced. Existing source
size and24-level parenthesis limits bound this recursion. The new missing-parameter-brace failures are Syntax. Original plain arithmetic
lexical diagnostic text is retained; there is no blanket reclassification. Explicit
exclusions and nesting limits retain their own categories.

The simulator deliberately preflights the whole script before any command or
virtual write; Bash’s diagnostic can occur during execution after earlier
commands. Consequently the simulator’s Syntax/status2 and absence of earlier
virtual effects differ intentionally from Bash’s status1. Exact Bash diagnostics
are not promised. A valid selected division-by-zero value is still a runtime
arithmetic failure: syntax preflight does not evaluate an unchosen value or roll
back earlier effects of a valid script. Full current reference/regression gates pass after this structural correction.
The original failed54-case reference remains retained with its false expectation;
new59-case validation records58 positives and one exact confirmed invalid case.
Neither capture is relabeled or overwritten.
