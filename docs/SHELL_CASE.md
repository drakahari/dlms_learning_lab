# Bounded case selection — evaluator 8

This page retains the original evaluator8 milestone and evidence. The separate
[evaluator12 ASCII-class contract](SHELL_CASE_CLASSES.md) describes the later
numeric-validation extension and its pending acceptance; it does not rewrite
the original observations or imply full bracket-pattern support.

Accepted capability milestone, checked 2026-10-24. The native application adds
bounded `case` selection without a new dependency or host execution. Current
content remains106 authored labs/25units; a supported language construct alone
is not new curriculum or certification coverage.

The corrected engine passed55 focused checks. The merged106 catalog passed810
model/integration and121 UI checks, with all-target Clippy and formatting.
Actual comparison preserved3,968 earlier outcomes;156 enumerated changes are
new support or static diagnostics. All37 authored scripts/215 results and37
examples retain their behavior. Genuine6/7 records keep their original revisions.
Backend migration checks and the actual Fedora7-to8 native flow pass; see
[EVALUATOR8_MIGRATION_ACCEPTANCE.md](EVALUATOR8_MIGRATION_ACCEPTANCE.md).

The reviewed internal extension keeps the existing pure cooperative VM and
public API. It adds no dependencies, host shell, process, file lookup or network.
The separate virtual terminal still accepts literal arguments only. A case
construct runs only in the Script editor evaluator.

## Exact supported syntax

`case WORD in PATTERN[|PATTERN]...) COMMANDS ;; ... esac`, with optional leading
`(` per arm and optional final `;;` after a normal final-list separator. Empty
cases and empty bodies are accepted. Only literal bare reserved words in proper
grammar positions introduce a construct; quoted/escaped/expanded data does not.
An `esac` pattern at the beginning of an arm needs quoting or the optional `(`.
Bodies use the existing command, function, conditional, loop, redirect and
cooperative capture subset. Function returns and exit retain their normal scope.

Selectors expand once to a single entire scalar, with no splitting or filename
expansion, including empty strings, whitespace and literal bracket data. Pattern
alternatives expand only when reached, left to right, stopping at the first
match. This follows the observed GNU order; portable instruction must not depend
on side effects within an alternative list, whose order POSIX leaves unspecified.
Quoted/escaped pattern bytes are literal. Bare `*` consumes zero or more bytes;
bare `?` consumes one byte. Quoted and bare fragments may be mixed. The entire
selector must match. Slash, leading dot and newline are ordinary bytes. C-byte
semantics make UTF8 literals exact but do not make `?` a Unicode-character match.
Repeated active stars are supported without recursive-directory semantics.

Matching supports C/POSIX requests only, following the existing filename
locale policy: nonempty LC_ALL overrides LC_COLLATE/LC_CTYPE/LANG; otherwise any
nonempty non-C/POSIX value in those categories is rejected. The guard runs after
the first reached pattern expands, before matching/body entry. Earlier selector
and pattern capture effects remain; no body runs on rejection. Skipped cases and
empty cases perform no pattern matching and do not invoke this guard. This does
not emulate Unicode/locale matching.

No match and selected empty bodies return0. Nonempty bodies return their last
command's status. Entry and successful matching do not reset `$?`: a previous
status remains visible, or a selector/reached-pattern capture's most recent
status becomes visible. Existing capture completion provides that status; no
new assignment-only special case is added.

Excluded: whole-case pipes/redirects, `;&`, `;;&`, bracket expressions/classes,
extended patterns, regex/options, unquoted dynamically expanded backslashes,
and list-valued `$@`/array-all in selectors or patterns. Quote dynamic bracket,
backslash or extended-looking text to match it literally. Scalar array-element
and count reads retain the existing Bash-specific contract, not portable-shell
claims. Unsupported static grammar fails before script effects; unsupported
reached dynamic patterns fail fatally and do not become no-match. An unselected
dynamic pattern is neither expanded nor checked. Child source still compiles
eagerly, so statically invalid source in an unselected capture is rejected.

## Implementation and limits

`Code::Case(Arc<Definition>)` shares immutable bounded descriptors. Compilation
relocates all arm/exit targets with the existing child arena. Execution clones
only the reached selector or pattern after preflight, never all alternatives.
`case::Scalar` uses the existing scalar parameter/arithmetic expansion and typed
capture requests, retains per-byte active flags for patterns, and never calls
ordinary argument splitting/sole-fields/filename matching. `case::Matcher` is a
separate original iterative matcher. It advances at most256 charged transitions
per preparation unit; existing VM preparation/instruction ceilings still apply.

`Pending::Case` uses the same capture continuation, stderr binding, root memory,
work, instruction and call limits as other commands. Pending/source/subject/
pattern/mask bytes remain charged while a task is temporarily taken out. Actual
append copies and returned capture sources receive peak preflight. Stop/fatal
failure drops pending and suspended tasks; later ticks cannot enter a body.
The delimiter scanner tracks bounded lexical case frames and command positions
without treating pattern `)` as capture end. Quotes, comments, continuations,
parameter/arithmetic spans, arrays, function parentheses, nested captures and
redirect-target words keep separate roles. The actual Parser still validates
grammar before any effects; the scanner is not an execution parser.

Per case:32 arms,128 alternatives total,8 alternatives per arm,256 expanded bytes
per pattern (even literal-only). Existing source16KiB, aggregate4096tokens,
syntax24, capture128sites/depth8, functions64/calls32 across the frame tree,
variables256, files64, live text256KiB, output32KiB, instructions2000 and expansion
work1000000 remain. Selectors are not truncated; root live memory bounds them.

The old global bare bracket-glob check moves to ordinary-word/pattern parsing so
selectors can contain bracket data. The terminal retains the old bracket guard.
Stray bare `esac` and new `;;`/fallthrough token recognition can change static
error precedence; these are intentional8 deltas, not claimed7 compatibility.

## Actual reference provenance and acceptance

The retained fixed37 GNU Bash5.3.9 oracle manifest is
`7af1b2e86a9cf9364c5ea170bcdfba4142a5c8cff0c5046efd0be57c55af02bd`.
The original manual is Edition5.3, updated2025-05-18, SHA256
`b91026acffb159bd1e842e3116a95539bf858e2b57cbfa74df9e3a53379177a4`.
Primary references checked2026-10-23: GNU Bash manual section Conditional
Constructs (`https://www.gnu.org/software/bash/manual/html_node/Conditional-Constructs.html`),
Pattern Matching (`https://www.gnu.org/software/bash/manual/html_node/Pattern-Matching.html`),
and POSIX Issue8 Shell Command Language
(`https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html`).
No GNU implementation code is copied. These are original MIT project additions.

`tests/fixtures/shell-case-bash539.json` preserves exact observed output/status/
files and each original observation hash. The four excluded Bash observations
are retained as exclusions; their Bash acceptance does not become subset support.
The focused test suite compares the33 supported observations at slices1/16/32/64,
then adds independent grammar, limits, laziness, provenance, input, memory/work,
function/frame/PC and Stop cases. An independent prefix dynamic-programming
oracle also checks40,404 exhaustive short pattern/subject pairs, including
quoted wildcard masks. The focused tests passed at this checkpoint.

Actual before/after comparison is retained with manifest
`7053d055a629fca5e07fc73efe074874682824f53293bd16630ee66a0a62fdb8`.
The first corrected-metadata attempt caught a cross-worktree Cargo cache mismatch
before publishing any output. Fresh exclusive checkout targets now prevent that
reuse; the accepted comparison rebuilt in its own target. Earlier failures and
all genuine historical evidence remain unchanged. No false old revision label,
new authored scenario or platform acceptance follows from these observations.

## Reviewed follow-up after the first focused gate

Commit27d0a0e passed the50 focused tests above. Subsequent independent source
review found three boundaries needing correction; the guarded follow-up passed
18 integration tests and37 shell unit tests (55 total), all-target Clippy with
warnings denied and scoped formatting checks. The capture delimiter now recognizes the
existing command-position literal `name=(...)` grammar as array data. Bare array
elements named case/in/esac cannot open/close case frames; nested `$(...)` retains
its own command domain. Real case after array assignment and function-body cases
retain their command roles. New tests cover empty, quoted, multiline/commented,
nested-capture and function-contained arrays. Pattern-specific static bracket
validation removes ordinary `[`/`]` command-word exceptions only in patterns,
preserving ordinary test commands and quoted literals. The locale guard above
makes requested unsupported matching explicit. No original observed goldens,
parser/VM budgets, profile bytes, historical revisions or main source changed.

The combined source follow-up also rejects a simple command whose first word
is the bare literal case/esac, including later pipeline stages or after leading
redirects. That closes a compile-first diagnostic bypass without reinterpreting
quoted/escaped/expanded command heads, later arguments or assignment prefixes.
This additional correction and its timing regressions passed the same55-check
focused gate. The later actual7/8 comparison, migration and native acceptance passed as
recorded above; focused checks alone were not used to establish those gates.
