# Shell subset, version 1

Defined before the introductory unit (checked 2026-10-21). This is an in-process educational interpreter, **not Bash**. No shell/process, host file or network API exists in the interpreter. Only virtual UTF-8 text files in a flat `/work` namespace exist; directories and nested paths are unsupported. Linux terminal labs retain their separate documented command models.

## Accepted language

- Scripts use LF line breaks. Commands separated by newline or `;`, comments at a word boundary, backslash escaping/line continuation, literal single quotes and expanding double quotes.
- Unquoted `*`/`?` filename matching after splitting over flat virtual files; quoted/escaped bytes stay literal, no-match words remain unchanged, hidden names need an explicit leading dot, matches are C-byte sorted. See SHELL_GLOBBING.md for bounds/exclusions.
- Integer arithmetic expansion `$(( expression ))` within the bounded contract below.
- Named scalar assignments (assignment-only commands); `$name`, `${name}`, `$1`…`$9`, `$#`, `$?`. Undefined parameters expand to empty. Unquoted expansion uses default space/tab/newline field splitting. Sole `"$@"` expands each positional argument separately, including empty arguments. Other uses of `$@` are rejected.
- `|`, `&&`, `||`; pipeline status is the last stage by default. Explicit `set -o pipefail` selects the rightmost nonzero stage status, or0 if all stages succeed; `set +o pipefail` restores the default. Pipeline variable/option changes are isolated; virtual file writes and first-stage stdin consumption persist. A first-stage redirected or explicit file input leaves the caller’s finite stdin untouched. No other `set` forms or `set -e`.
- `if …; then …; [else …;] fi`, `for name in words; do …; done`, `while …; do …; done`, nesting within the limits below.
- Text redirects `<`, `>`, `>>`, `2>`, `2>>`, `>&2`, `1>&2`, `2>&1`, evaluated left to right. Redirected paths must expand to exactly one word. Only a single trailing input redirect on while loops is supported, as specified below; other compound-command redirects remain unsupported.
- Commands (shell builtins and simulated text utilities): `printf` with literal `%s`, `%%`, `\n`, `\t`, `\\` formats (format repeats for excess arguments); `echo` without flags or backslashes; `cat PATH`; `grep -F LITERAL [PATH]` (literal lines, status 0/1/2); `read -r NAME` (one variable, default whitespace trimming, EOF status) and the narrowly scoped literal `IFS= read -r NAME` described below; `test`/`[` with `-n`, `-z`, `-f`, `=`, `!=`, `-eq`, `-ne`, `-lt`, `-le`, `-gt`, `-ge`, or one-string truth; `true`, `false`, `:`, `exit [0…255]`; function `return [0…255]`, `shift [N]`, and the Bash-specific local-variable extension as specified below.
- The first `#!/bin/bash` or `#!/bin/sh` line is a comment, not a process launcher. No startup files or inherited host environment. Each test starts with its declared environment, arguments, stdin and virtual files. Each Run starts fresh; editor work remains saved.

## Honest exclusions

Reject unsupported operators/syntax: backticks, bracket/recursive/directory globbing, tilde/brace expansion, parameter operators, unsupported array forms, subshells, groups, background jobs, here-documents/strings, `until`, `select`, `break`, `continue`, `export`, other `set` forms, `eval`, `exec`, `source`/`.`. No host commands, filesystem metadata/links/devices, executable files, non-C utility locale behavior or binary/NUL data. Filename matching is limited to the documented * and ? subset; unsupported patterns fail explicitly. `[[ ]]` and Bash-only syntax are not taught in unit 1. Foundation/composition/reusable/text units use portable shell syntax; the scope unit explicitly teaches Bash local. The reference oracle is Bash in its default mode with controlled inputs.

Diagnostic clarification (2026-10-25): `elif` clauses remain unsupported. Use
nested `if`/`else` blocks. A bare unquoted `elif` command head, including a pipeline
stage or after leading redirects, receives an explicit Unsupported diagnostic
while compiling the whole script, before any commands or virtual writes run.
Comments, quoted/escaped/expanded words, ordinary arguments, redirection
filenames, loop/case data and keyword-named variables keep their existing roles.
This changes only the excluded-syntax diagnostic; no `elif` semantics, evaluator
revision, history migration or reference observations are added. GNU Bash itself
supports `elif`; see its primary [conditional constructs](https://www.gnu.org/software/bash/manual/html_node/Conditional-Constructs.html)
and [reserved-word recognition](https://www.gnu.org/software/bash/manual/html_node/Reserved-Words.html)
references (checked2026-10-25). All four diagnostic regression tests pass in the guarded full model run. They
cover static rejection before writes and preservation of quoted/data roles.

## Bounded execution

Maximum source 16 KiB; 4096 tokens; 24 levels of syntax nesting; 128 words/arguments and 64 redirects per command; 16 pipeline stages; 256 ordinary variable names across active scopes (including unset declarations); 64 files; 256 KiB aggregate virtual text/variables, including names, finite input and saved local shadows; 32 KiB captured output per case; 2,000 VM instructions per case. Shared word expansion retains at most128 fields/256KiB before concatenation; the complete command has the same aggregate bounds. Each VM additionally has a cumulative1,000,000-byte expansion-work budget across source scans, parameter copies, text appends and field processing, including functions/pipelines/loops/ticks. UI advances a bounded instruction slice per frame and offers Stop; interruption discards the current grading run. There is no wall-clock wait, sleep or blocking input. Input comes only from a finite fixture. Resource/unsupported diagnostics invalidate success. A stopped or edited script cannot retain current success. These are educational limits, not OS resource accounting.

## Fidelity evidence

`tools/bash_reference.py` runs **only checked-in controlled fixtures**, from a fresh temporary directory with an empty controlled environment and a timeout. It does not accept learner scripts or personal paths. It compares stdout, stderr, exit status and resulting text files with the Rust interpreter probe. Real Bash is a development oracle only; it is absent from the runtime interpreter. See the milestone verification report for the fixtures actually run and remaining exclusions.

Integrity clarification (2026-10-22): `grep -F` rejects patterns beginning with `-` until real `-e`/`--` option parsing is implemented. Integer comparisons accept signed 64-bit decimal operands; malformed or out-of-range operands return an explicit unsupported diagnostic instead of pretending to implement Bash’s recoverable test error. Stored diagnostics are bounded to 1,000 UTF-8 bytes so any supported-length script remains resumable after failure.

## Reusable-interface extension

Specified before authoring dependent lessons, 2026-10-22. Portable `name() { commands; }` definitions become available when execution reaches them. A call replaces positional parameters until it returns; named variables remain global unless the Bash-specific extension below explicitly declares a local. The function’s last command supplies its status; `return [0…255]` leaves only the current function. `shift [N]` removes positional arguments (nonnegative decimal N, default 1); excessive counts return 1 without changing them. Top-level arguments are restored after a call. This does not teach Bash `local` or pretend global assignments are local. Redirection on return follows the ordinary command rules, including when its name is quoted or expanded.

Initial explicit restrictions: function calls must be separate commands, not pipeline stages or redirected as a whole; redirect individual commands within the body. Builtin/keyword replacement, non-brace bodies and return outside a function are unsupported. Function definitions inside supported bodies follow normal execution order. No host command lookup occurs. Maximum 64 functions, 32 active calls, shared 2,000-instruction budget; calls execute cooperatively and Stop can interrupt recursion. Functions do not create an unbounded Rust recursion path.

Primary references: [GNU Bash functions](https://www.gnu.org/software/bash/manual/html_node/Shell-Functions.html), [Bourne shell builtins](https://www.gnu.org/software/bash/manual/html_node/Bourne-Shell-Builtins.html), [POSIX shell function definitions](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html). Direct page retrieval failed in this recheck (GNU timeout, POSIX 403); the official GNU search index, local Bash 5.3.9 help and disposable reference comparisons verify supported cases. Functions, restored positional parameters, return status and shift semantics are covered by those primary references. No dependent lab is authored in this capability milestone.

## Function-local variables: extension contract

Specified2026-10-22 before dependent lessons. Bash `local` is a Bash-specific feature in this course; it is not promised by portable POSIX `sh`. Within a function, literal `local NAME` creates an initially unset function-local variable (default Bash options, without `localvar_inherit`), while `local NAME=value` sets it. Repeating a declaration in the same call preserves its current value unless a new value is assigned. Multiple declaration arguments expand before any of those declarations take effect. Assignment arguments preserve spaces and literal wildcard values without field splitting. Ordinary dynamic name arguments follow the supported word-expansion rules.

A local variable is visible to functions called from that function. A called function that assigns the same name without its own local declaration changes the caller's local value. This is dynamic visibility, not lexical isolation. Return or reaching the function end restores the shadowed values, including whether the earlier variable existed. Local loop/read/assignment updates use the same visible variable map. Saved shadow values/names count toward the common memory bound; active frames remain bounded/cooperative.

Initial scope: at least one valid variable argument, no `local` options/attributes, compound array declarations, namerefs or listing mode. The command name must be literal unquoted `local`; quoted/expanded declaration-command names remain explicitly unsupported. `local` outside a function and within a pipeline are unsupported contexts with honest diagnostics. Attribute inheritance, `localvar_unset`, unset/export/declare and shell-option restoration are not implemented by this extension. Existing portable function lessons retain global-variable behavior unless they explicitly declare `local`.

Primary reference: GNU Bash5.3.9 installed manual (`/usr/share/info/bash.info.gz`, sections Shell Functions, Bash Builtins and The Shopt Builtin), plus controlled disposable development fixtures. GNU online function/builtin pages timed out during this check; source URLs remain [functions](https://www.gnu.org/software/bash/manual/html_node/Shell-Functions.html), [builtins](https://www.gnu.org/software/bash/manual/html_node/Bash-Builtins.html) and [shopt](https://www.gnu.org/software/bash/manual/html_node/The-Shopt-Builtin.html). Three reviewed local-scope/debugging assignments follow this capability; see [SHELL_SCOPE.md](SHELL_SCOPE.md). Full advanced Bash and certification coverage remain open.

Default `IFS` reads as space/tab/newline. Except for the literal-empty read prefix below, custom IFS assignment/declaration/read/loop targets and fixture overrides are rejected until custom splitting is implemented; it must never silently behave as ordinary text. Variable names, saved shadow values, file names and finite input count toward the aggregate memory limit.

Unmodeled special/readonly/dynamic variables (including UID/EUID, `_`, RANDOM, positional-metadata arrays and shell-option variables) are rejected in expansion, assignment, local/read/for targets and fixture environment. The subset also reserves BASH*, COMP_* and HIST* names conservatively; these prefixes are a simulator namespace restriction, not a claim that every possible such name is special in Bash. Ordinary application names remain available.

## Text-utility extension contract

Specified2026-10-22. Simulated utilities are ordinary commands in the existing parser/pipeline/redirection VM; no host program lookup occurs. Supported forms:

- `sort [-r] [-u] [--] [FILE]`: full-line byte ordering in the fixed C locale, reverse and/or unique, including combined `-ru`. No numeric/key/locale/random/merge/check/in-place/temp-file options. sort/uniq/cut are utilities, so existing user-defined functions with those names still take precedence.
- `uniq [-c] [-d|-u] [--] [FILE]`: adjacent whole-line groups, optional count (GNU seven-column count plus a space), repeated-only or singleton-only groups. No ignoring fields/case/characters or output-file operand. Combined flags are supported; simultaneous `-d` and `-u` is outside this subset.
- `cut [-d CHAR] -f LIST [-s] [--] [FILE]`: one ASCII delimiter byte (tab by default), positive field numbers/ranges separated by commas; `N`, `N-M`, `-M`, `N-`. Output remains in input-field order once per selected field, irrespective of list order/overlap. Delimiter-free lines pass unchanged unless `-s`; an out-of-range selection on a delimited line still emits its line break. No byte/character mode, complement or custom output delimiter. Only separate `-d CHAR` / `-f LIST` argument forms are initially supported.

Options precede the sole optional input filename; `--` permits option-like filenames. Missing FILE or FILE `-` consumes the remaining finite simulated standard input. A file operand reads the virtual file without consuming stdin. Line endings are LF; CR remains ordinary data. Unterminated nonempty final records receive a newline in output; empty input has no records. There is no CSV quoting/escaping interpretation. All valid virtual UTF-8 content uses byte equality/order in the C locale; changing LC_ALL/LC_COLLATE/LC_CTYPE/LANG to a different locale is explicitly unsupported for these tools.

Resource contract: at most8192 input lines per utility,128 cut-list ranges and field indices up to1,000,000, with existing256KiB aggregate virtual text and32KiB captured/intermediate-output limits. No numeric-range-sized allocation. Unsupported flags/forms and malformed lists produce honest diagnostics. Missing virtual files return a command error/status (sort2, uniq/cut1). Learner paths use the existing flat /work resolver; no host filesystem is consulted.

References: official GNU coreutils manual, sort/uniq/cut invocation sections ([sort](https://www.gnu.org/software/coreutils/manual/html_node/sort-invocation.html), [uniq](https://www.gnu.org/software/coreutils/manual/html_node/uniq-invocation.html), [cut](https://www.gnu.org/software/coreutils/manual/html_node/cut-invocation.html)). Online fetches timed out on2026-10-22; the installed original `/usr/share/info/coreutils.info.gz` was inspected. Controlled disposable comparisons verify the accepted forms before dependent content. This extension does not establish full filter-suite or LPIC objective coverage.

## Integer arithmetic extension contract

Specified2026-10-22. Supported `$(( expression ))` arithmetic expansion supports fixed signed64-bit integers, addition/subtraction/multiplication, integer division/remainder, unary plus/minus and grouping parentheses with normal precedence. Arithmetic expansion produces decimal text, usable in an assignment, a quoted argument or an unquoted word under the existing expansion rules. Empty expressions and unset/empty named variables evaluate to0. Named variables read the currently visible scope; positional/status/count parameters can be expanded explicitly. Nested arithmetic expansion is bounded.

Initial integer literals: decimal, C-style leading-zero octal and0x/0X hexadecimal, magnitudes at most2^64−1. Arithmetic operations wrap at64 bits as Bash does on the Fedora reference platform. Division truncates toward zero; remainder follows the dividend sign. Zero divisors fail before executing the associated command. Signed minimum divided/remainder by−1 is explicitly unsupported until its platform-specific reference behavior is established. Bare named variables must contain a supported integer literal with optional attached sign/outer ASCII whitespace; recursive expressions/names stored in their values are unsupported rather than silently evaluated incorrectly. Explicit parameter expansion occurs before parsing the expression.

No arithmetic assignment/increment, exponentiation, comparisons, bitwise/logical/ternary/comma operators, base# literals, array references, embedded quote removal or command substitution is initially promised. Unsupported syntax fails clearly; it is never sent to a host. This is arithmetic expansion only, not `(( commands ))`, `let`, arithmetic for loops or floating-point calculations. No new command name/dependency is introduced. Limits:4096 expanded expression bytes,256 tokens and24 parser recursion levels, in addition to the existing VM/source/text/instruction limits. No exponent-sized or numeric-range allocation.

Reference: official GNU Bash manual [Arithmetic expansion](https://www.gnu.org/software/bash/manual/html_node/Arithmetic-Expansion.html) and [Shell arithmetic](https://www.gnu.org/software/bash/manual/html_node/Shell-Arithmetic.html), installed original Bash5.3.9 info manual inspected2026-10-22; direct online fetches timed out. Supported behavior must pass controlled disposable comparisons before content is authored. Four dependent counter/totals lessons are described in SHELL_COUNTS.md; this contract alone does not establish full arithmetic or advanced-course coverage.


Expansion errors use explicit simulator diagnostics/status2; Bash's exact fatal-error text/exit code is not reproduced. Command/declaration arguments expand before redirection. Assignment-only commands apply values sequentially before their redirections; an input-redirection failure does not undo those assignments. Pipeline context checks inspect the command head without evaluating later assignment values. Evaluator revision3 corrects this preexisting ordering behavior; earlier credit is retained separately as documented in PROGRESS_RECOVERY.md.

## Sequential file input for while loops

Specified2026-10-22 before finalizing dependent counter lessons: `while CONDITION; do BODY; done < WORD` accepts one virtual input redirection. Expand/open the path once on loop entry, consume sequentially across condition/body/function calls, then restore the caller's prior input and exact cursor on normal exit or an early function return. Variables modified in the loop remain visible; this is not a pipeline/subshell. Missing input sets status1, reports the missing virtual file and skips the loop; `||` can handle that result. Diagnostic text is simulator-specific.

Only this trailing single `<` on while loops is added; output/multiple redirects, redirected for/if/groups/function calls and descriptor management remain unsupported. Open-loop inputs are finite read-only snapshots: an attempted truncate/append/write to any active source is explicitly rejected before mutation, until changing-open-file semantics are modeled. Nested inputs and saved cursors/names count toward the aggregate memory bound; syntax/call/instruction limits and cooperative Stop remain in effect. No host file handle/API is opened. Controlled Bash comparisons must verify the accepted forms before the dependent teaching ships.


### Expansion resource correction (evaluator4)

Individual bounded values cannot be joined or split into arbitrarily large temporary fields before the final argument guard. Shared early checks cover ordinary commands, assignments, local declarations, input/output redirection, while-file input and sole quoted "$@". UTF-8 is counted in bytes while retaining valid text boundaries. The cumulative work budget bounds repeated text replacement even if whitespace later disappears. Limit diagnostics invalidate current grading and explain the likely cause; no new shell syntax or utility is added. Earlier evaluator credit stays historical with exact original-byte backup. See SHELL_EXPANSION_LIMITS.md for the pre-implementation contract, controlled prior/current evidence and known scope.


## Filename matching (evaluator5)

The contract in SHELL_GLOBBING.md is implemented before dependent content. Per-byte quote provenance survives splitting, so quoting a wildcard keeps it literal even next to an active wildcard. Only the current virtual file map is searched; no host glob API exists. Matching work shares the VM-wide expansion budget. A test of a file name remains distinct from a text-pattern test. Bracket expressions arriving through unquoted variables now fail honestly instead of silently remaining literal; actual evaluator4 results stay historical with exact-byte backup. This capability alone adds no authored assignments or certification coverage.

## Bash indexed-array extension (evaluator6)

Bash-specific dense lists support standalone `name=(words)`, empty `name=()`, constant canonical decimal `${name[0]}` reads (index0–4294967295), sole quoted `"${name[@]}"` preserving separate/empty elements, and `${#name[@]}` counts. At most128 elements. Ordinary assignments/read/for targets update element0 and preserve the tail; `$name` reads element0. Scalar @ has one element when set. Empty arrays have none. Local declarations retain an outer array’s type with an empty local and restore the whole previous list on return. Arguments, loops, local shadows and pipeline copies count toward the shared virtual-memory limit, including array slot overhead.

No explicit index writes, sparse/associative arrays, +=, computed/negative/leading-zero indices, slices, star/indirect expansion, mixed or unquoted @, unset, read -a, attributes, local compound declarations, or compound assignments combined with commands/pipelines/redirections. Unsupported forms fail explicitly. See [the precise array contract](SHELL_ARRAYS.md), including original GNU Bash reference provenance and limitations. These features are not portable POSIX sh.

M44 authored indexed-list unit: four practical assignments with isolated examples and5/5/5/10 behavior cases. Loaded packs and saved records share a12-case maximum; each case retains its independent execution/resource limits. See SHELL_ARRAY_LESSONS.md for exact coverage, oracle evidence and native acceptance limits.


Evaluator7 adds bounded original-source `$(LIST)` command substitution in the Script editor, with cooperative isolated child contexts, trailing-LF removal, quote-aware splitting/globbing, ordered redirects, shared resource/Stop limits and explicit exclusions. The separate terminal remains literal-only. Captures inside arithmetic, backticks, process substitution, Bash input-only shorthand and inherited-direct local/return are excluded. See SHELL_SUBSTITUTION.md for the precise contract and Fedora acceptance scope. Earlier records remain history; only an explicit current Run earns current credit.


## Case selection — evaluator 8 extension

The Script editor supports `case WORD in PATTERN) COMMANDS ;; ... esac` within
its bounded shell subset. The selector is one complete value, including spaces;
patterns match the whole value. The first matching arm runs. An unquoted `*`
matches any number of bytes and `?` one byte; quote wildcard characters to match
them literally. Matching uses the C/POSIX locale. `case` is portable shell syntax;
Bash-specific indexed lists inside it retain their separate portability limits.

The subset deliberately excludes bracket expressions, extended patterns,
fallthrough (`;&` / `;;&`), whole-case pipelines/redirections and list-valued
selectors/patterns. Unsupported syntax produces a diagnostic. Limits are 32 arms,
128 alternatives total, 8 per arm and 256 expanded bytes per pattern, sharing
the existing instruction, memory, output, expansion-work and responsive Stop
budgets. Matching is iterative and cooperative. See [SHELL_CASE.md](SHELL_CASE.md)
for expansion/status semantics, exact exclusions and acceptance evidence.

The case extension introduced evaluator 8. Genuine earlier results keep their original
revision, source and output; opening a lesson does not invent a new solution.
Only a current explicit Run or confirmed restore recheck can establish current
success. Examples remain independent and ungraded. See
[PROGRESS_RECOVERY.md](PROGRESS_RECOVERY.md) before downgrading an existing profile.

## Getopts — evaluator9 bounded extension

The isolated source milestone adds bounded `getopts OPTSTRING NAME [ARG...]`,
cluster/required-value parsing, explicit OPTIND=1 reset and separate hidden cursor
ownership through locals, captures and pipelines. Each run initializes OPTIND=1,
OPTERR=1 and OPTARG unset; ordinary OPTERR operations remain available. Parsing
requires scalar OPTERR0/1 and an exact unchanged declaration/argument list until
reset. Uninitialized local OPTIND, arbitrary seeks, optional/long/non-ASCII option
declarations and special-variable output aliases are excluded. No host command
is invoked. See [SHELL_GETOPTS.md](SHELL_GETOPTS.md) for the exact boundaries,
actual55 reference observations, resource accounting and completed model/migration
checks. Installed revision9 acceptance passed on the122-lab Fedora candidate atcd31af6;
its three practical assignments are documented in SHELL_GETOPTS_LESSONS.md.
The evaluator extension alone does not establish curriculum coverage. Only genuine current9 execution may
produce9 results; actual6/7/8 records and old backup bytes remain historical.
The original256 ordinary-name allowance is preserved across current and hidden
scopes, with at most three additional fixed getopts names (OPTIND/OPTERR/OPTARG).
Their bytes and all snapshots remain charged to the unchanged memory/work caps;
unsetting a special name never frees an ordinary-name slot.


## Scoped empty-IFS read and pipefail: private evaluator10 source

Source implementation2026-10-26; compiler, model, migration and native checks are
**unrun**. This is not a new lesson, coverage claim or activation on MAIN.
The exact accepted public9 capture remains immutable and its producer refuses
other evaluator revisions. Only an actual current10 Run can produce10 results.

Accepted temporary forms are `IFS= read -r NAME`, `IFS='' read -r NAME` and
`IFS="" read -r NAME` (literal empty quoted fragments may combine). Literal
quoted/escaped `read` heads keep their ordinary text role. The prefix is admitted
from original literal syntax, not a variable/capture that happens to yield empty.
Only the builtin's line assignment uses empty IFS: preserve all bytes before the
first LF, including edge spaces/tabs, internal spaces, CR, UTF-8 and backslashes.
LF is consumed but not stored. A nonempty final line without LF is assigned with
status1; empty EOF assigns empty/status1; an LF-terminated empty line has status0.
The finite input, target validation, array element0 update, local scope and
ordinary redirected-input cursor rules are unchanged. NUL remains excluded.

Arguments and then ordered redirects expand before the read policy takes effect.
A redirect capture that expands `$IFS` still sees the default space/tab/newline.
There is no temporary environment-map assignment and no change to later field
splitting or ordinary `read -r NAME` trimming. Nonempty/dynamic/persistent/local
IFS, general assignment prefixes, multiple targets and other read flags remain
unsupported; no host environment or process is involved. Rejected reached
commands keep the existing argument/redirect timing, including redirects already
performed before a builtin-argument diagnostic.

Each fresh run starts with pipefail off. Only `set -o pipefail` and
`set +o pipefail` are supported. A pipeline pins its option at entry. Ordinary
functions share the caller's option; captures inherit it and restore the parent
on return; each isolated pipeline stage starts with the parent option and cannot
change it for another stage or the caller. No `errexit`, option listing,
`set --`, shell-option variables, pipeline negation or new pipeline function-call
support is implied. Unsupported/resource/Stop errors remain fatal ungraded
outcomes, not recoverable pipeline status values. Existing file effects and
first-stage input consumption are unchanged; failure status is not a rollback.

The cooperative continuation retains one four-byte rightmost-nonzero accumulator
for an enabled multi-stage pipeline, charged to the existing memory/work ledger.
No output copies, new VM/child budget, external process or timing dependency is
introduced. Source/token/argument/redirect/stage/input/output/instruction/memory/
work/capture-depth limits and cooperative Stop remain unchanged. Default-off
pipelines keep their existing dynamic-memory accounting.

Reference scope is GNU Bash5.3 manual Edition5.3 and **actual Bash5.3.9** in the
accepted fixed28 controlled C-locale cases. The source/input/output fixture
`automation-bash539-observations.json` contains those actual Bash observations
and provenance hashes only, never saved learner Records or claimed evaluator10
observations. Two observed general-prefix/dynamic-IFS cases remain explicitly
excluded. POSIX Issue8 portability has not been verified for pipefail here; do
not call it universally nonportable or claim complete POSIX/Bash behavior.

The frozen public9 producer is a nondefault Cargo feature target
`historical-evaluator9-capture`; ordinary all-target gates skip this historical
producer rather than execute its revision9 assertions under10. Explicit requests
must pass the MAIN runner's source-revision preflight and its existing runtime9
identity guard; no feature selects older evaluator semantics. See the
[Cargo target contract](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#required-features)
(checked2026-10-26). Retained before10 pins/binaries remain the evidence of9.


### Current implementation gate — 2026-10-26

The scoped extension above is now implemented on MAIN as evaluator10; earlier
source-only statements describe its handoff. Fifteen new VM groups pass, including
all26 supported fixed actual Bash5.3.9 cases and both honest exclusions. Controlled
comparisons do not execute learner input on the host. A separate opt-in comparison
classifies all1570 frozen9 rows and reruns380 vectors at four tick slices:357 retain
exact complete outcomes and ticks;23 intended changes match actual Bash. Four Stop
paths retain exact observed outcomes/ticks. Historical profile/identity metadata
remains labeled9; it is never presented as new10 observations.

Actual9 migration replay passes raw-target protection, full6/7/8/9 histories,
separate drafts, target-derived receipts, reset-first/practice-only, occupied slots,
failed writes/durability warnings and explicit backup recheck/cancel/restore. No
new requirements or learner answers are invented. The full application regression
passes before a test-only shared-helper cleanup, followed by affected library/
migration checks and all-target Clippy. New practical assignments and native
acceptance are separate gates; the installed133 candidate38a still uses evaluator9.
