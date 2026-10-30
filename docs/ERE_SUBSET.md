# Bounded ERE foundation and proposed utility integration

The original pure Rust foundation prepares boolean regular-expression line matching.
The following opening sections describe that independently tested foundation;
the final section specifies the separate capability14 integration batch.
It is not connected to the shell, a terminal, a new lab, saved progress or the
normal catalog. No evaluator revision changes. Existing `grep -F` retains its
literal behavior. No complete POSIX, BRE, GNU grep or LPIC-1 objective coverage is
claimed from this foundation.

`Regex::compile(pattern)` parses syntax and constructs a bounded Thompson-style
NFA. `regex.matcher()` owns reusable scratch space. Call
`matcher.matches_line(line, &mut budget)` on one LF-stripped line; pass the **same
caller-owned Budget across every line in the whole operation**. The result says
whether any substring matches. Captures, match positions and longest-match
selection are not exposed. Those choices cannot affect this boolean result.

The accepted grammar includes ASCII literals, escaped special ERE punctuation,
`.`; `^` at an alternative's start and `$` at its end; concatenation, plain groups
and alternatives; `?`, `*`, `+`, `{n}`, `{n,m}`, `{n,}`. Counts are decimal0–32;
leading zero spellings are accepted. Quantifiers bind before concatenation,
then alternation. The empty whole pattern matches an empty substring (GNU
behavior); empty groups/alternatives and stacked quantifiers are rejected.
[GNU structure reference](https://www.gnu.org/software/grep/manual/html_node/Fundamental-Structure.html).

Brackets accept ASCII members/ranges, leading `^` negation, literal first `]`,
and literal first/last `-`. Twelve named classes use the C-locale ASCII sets:
`alnum alpha blank cntrl digit graph lower print punct space upper xdigit`.
Use two bracket pairs for a class, such as `[[:digit:]]`. Descending ranges fail.
Backslash within brackets, ambiguous extra hyphens, collating/equivalence
expressions and class-valued range endpoints are explicit exclusions.
[GNU bracket reference](https://www.gnu.org/software/grep/manual/html_node/Character-Classes-and-Bracket-Expressions.html).

Input remains valid UTF-8 text, but matching counts its encoded **bytes**, as in
GNU grep's C/POSIX locale. Thus `^.$` rejects `é` and `^..$` accepts it; ASCII
`alpha` does not include its two high bytes. A negated ASCII class can match those
bytes. This module only returns a boolean, never cuts a UTF-8 output string at a
byte match. LF belongs to the caller's line framing and is rejected inside one
line; CR, vertical tab and Unicode line-separator bytes stay ordinary input.
Patterns containing non-ASCII, LF or NUL and input containing NUL are unsupported.
No Unicode regex, case folding, binary-file detection or locale switching is
pretended. [GNU encoding reference](https://www.gnu.org/software/grep/manual/html_node/Character-Encoding.html).

Other exclusions include backreferences, lookaround, inline flags, special group
syntax, GNU word-boundary/shorthand escapes, ordinary backslash escapes, BRE and
unmatched closing parentheses. Literal braces outside intervals must be escaped.
Unsupported constructs report their limitation; malformed supported syntax and
resource limits have separate diagnostics. A valid GNU expression can therefore
be deliberately unsupported here, rather than silently interpreted differently.

## Resource and execution contract

- Pattern512 bytes;256 flat AST nodes;16 nested groups;1024 compiled states.
  Fixed-size byte masks avoid growing class lists.
- State and compiler-visit costs use checked arithmetic **before** counted
  repetition expansion or graph allocation. Compiler visits cap at16,384.
  Zero-state subtrees fold to epsilon, preventing nested empty counted groups
  from causing exponential compile traversal despite a tiny final graph.
- Compiler heap admission is128KiB, with checked actual vector capacities;
  parser recursion is16 and compiler recursion at most256 AST levels. Regex
  retained bytes and matcher scratch bytes are separately exposed for a future
  owning VM to account. No vector grows after its admitted capacity.
- One line is at most64KiB. Each operation's Budget allows at most one million
  units. Input validation, byte advances, active-state inspection, closure
  visits and duplicate-edge checks all spend from that same budget.
- Epsilon closure uses an iterative stack and per-position visited bitsets.
  Each state is admitted at most once per input position. Nullable cycles
  terminate. Thread/frontier memory is O(states), matching work is
  O((line bytes+1) × states), capped by the shared allowance; there is no
  backtracking path or recursion while matching. Scratch is reused across lines.

No host I/O, filesystem, process, network, environment or unsafe API appears in
the module. Data is borrowed; the module cannot modify simulated or real files.
A future grep wrapper must retain the VM's smaller aggregate memory/output and
instruction limits, use one budget for all selected input, preserve ordinary
redirection/status semantics, and keep Stop responsive. The synchronous pure
primitive is bounded; its actual latency and future VM scheduling still require
measurement and review. This foundation does not assert those integration gates
have happened.

## Curriculum and acceptance boundary

LPI's current LPIC-1 v5.0 objective103.7 includes constructing regex, BRE/ERE
distinctions, selection and text modification. This ERE boolean foundation is
only one prerequisite; a disclosed utility interface, authentic terminal/script
behavior, BRE/supporting teaching and substitution work remain. The curriculum
matrix must stay honest until meaningful assignments and behavior gates exist.
[LPI official objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/#rdp-we-103.7_Search_text_files_using_regular_expressions).

The capability is intended to enable later structured failed-event selection,
malformed-row diagnosis and log-analysis/recovery projects. No new lesson title
is registered here. Before changing live evaluator semantics, capture and seal
the actual current evaluator13 catalog after normal182 acceptance; do not reuse
the old55-script unregistered13 test run as a57-script historical producer.
Existing evaluator9–12 evidence and original authored content remain immutable.

References checked2026-10-29: GNU Grep3.12 primary manual pages and the current
LPI page were inspected through the web's primary-source extracts. GNU direct
page opens intermittently timed out. Official POSIX Issue8 chapter9 returned403;
its current contents were not independently fetched successfully in this pass:
https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/V1_chap09.html . The
installed GNU grep primary Info manual was also read locally and hash-pinned in
the private source receipt. No GNU implementation or manual code was copied;
this module and its fixtures are original project code, not redistributed GNU
regex source. Actual Rust, GNU-reference, native and curriculum acceptance are
pending; source formatting is not execution evidence.


## Foundation verification

Sixteen pure integration groups pass, including independent ASCII class tables,
exhaustive short-input alternatives, UTF-8 byte behavior, nested nullable/repeated
patterns, malformed/unsupported input and shared resource accounting. All-target
Clippy passes with warnings denied. The module is not yet exported into the
learner interpreter or a simulated utility. Its34 fixed GNU comparison predictions
and eight explicit exclusions have no actual observations yet; a guarded
development comparison is being prepared. No certification objective, lesson,
saved-result migration or responsive-Stop acceptance is inferred from this core.


## Capability14 integration batch (not yet accepted)

The proposed explicit `grep -E` wrapper is documented in SHELL_SUBSET.md. It
returns whole original lines, so it never slices output at a byte match. The
original synchronous Regex/Matcher implementations and old literal-F body remain
unchanged; an additional owned Cursor uses the same compiled graph, an immutable
UTF-8 snapshot and validated LF-free line ranges. It yields after at most64 work
units; the existing VM permits at most32 preparation advances per tick. Every
input byte, frontier/state/epsilon step and record-framing operation spends a
shared budget. Nullable cycles have per-position visited sets. Fixed-size bitmap
clearing is bounded independently. Stop drops the owned graph, scratch and input.

The VM has one1,000,000-unit ERE allowance across all calls, lines, functions,
pipelines and substitutions. Its existing2,000 semantic-instruction,256KiB memory
and32KiB output limits remain unchanged. Compilation is synchronous under the
existing512-byte/256-node/16,384-visit/128KiB bounds. It is conservatively precharged
using at most256 parser/mask units per pattern byte plus the admitted compiler
visits/nodes; `compile_work()` is a partial compiler metric, not a measured count
of every parser operation. Compiler allocation and retained graph/snapshot/scratch/
boxed-task/output capacity are admitted before allocation. No new dependency or
host filesystem/process/network API is introduced.

Twenty-four new fixed utility recipes have predictions only; GNU observations
remain null. Six explicit VM-only boundaries distinguish supported utility
outcomes from simulator resource/error/residual-input guarantees. They do not
replace the earlier34 supported/eight excluded pure-foundation recipes or their
evidence. A future independently reviewed closed collector must compare stdout,
stderr, status and virtual-file preservation using only fixed C-locale disposable
fixtures; this batch does not execute GNU or authorize learner-host execution.

Primary GNU Grep3.12 manual sources were checked2026-10-29: the full manual and
Exit Status page describe final-LF output and the `-q` error exception. Root
independently accessed those official web pages; direct fetches by this author
timed out. The installed official primary Info manual was read locally,
SHA25627d126caa28a6185038ad22af12f7f994dce957ae2fa6f349ac2aa06496af635.
Only paraphrased semantics and original fixtures/code are used. Full GNU/POSIX
conformance, complete LPIC103.7 coverage and new learner mastery are not claimed.


Capability14 acceptance2026-10-30: the integrated cooperative utility passes the
fullinclusive2,839-test run and24fixed full VM/GNU outcome comparisons, with
independent retained-stream/source/ELF verification. Resource/Stop/migration and
negative-path tests remain separate from GNU matching. Exact evidence/limits are
in SHELL_ERE_REFERENCE.md; no fullGNU/Bash, native193 or curriculum credit follows
from matcher or utility success alone.
