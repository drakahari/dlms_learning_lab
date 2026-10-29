# Numeric text checks with bounded case classes

Evaluator12 implementation, source integrated2026-10-29. The bounded capability
passes scoped129 and fixed48actualBash comparisons (stdout/stderr/status/fullfile
maps,0mismatches), with genuine evaluator11 history retained. Private integrated
171 full2337/all24histories and numeric101 native acceptance pass; final Main
full2337/static/coverage gates pass, with a new candidate still pending. No dependency, host execution or network feature.
The accepted workspace and prior52/384/52curriculum stay unchanged; the separate
three-task numeric unit supplies new instruction/practice, not this capability
alone. See [the lesson evidence](SHELL_NUMERIC_VALIDATION_LESSONS.md).

The next practical skill is checking input **before** treating it as a number.
Shell arithmetic accepts leading 0 as octal. The simulator's signed 64-bit integer
tests reject malformed/out-of-range operands with an explicit limitation, rather
than implementing Bash's recoverable test-error surface. A digits-only check is
also not overflow protection. A robust program needs text, length and practical
bounds before arithmetic or file publication.

## Supported pattern boundary

In a `case` pattern only, ASCII lists (`[abc]`), ascending ranges (`[0-9]`,
`[0-9A-F]`) and portable first-position `!` negation (`[!0-9]`) become available.
A class consumes exactly one C-locale byte. Existing `*` consumes any number of
bytes and `?` one byte. Quoted/escaped pieces remain literal, including a quoted
hyphen or opening bracket inside a class. A first member `]`, or a first/last
hyphen, can be literal. Quote the entire pattern to match its brackets literally.

`[0-9]*` means a digit followed by **anything**, so it is not a digits-only guard.
`*[!0-9]*` finds a non-digit anywhere, while empty input needs a separate pattern.
For canonical decimal text, a program can reject empty input, non-digits and
multi-digit leading 0. Use an explicit length guard before integer comparisons;
for example 19 question marks followed by `*` reject 19-or-more bytes. Then enforce
the task's small numerical maximum before multiplying. These are supporting
explanations, not an assignment solution or new assessment unit.

Matching remains byte-oriented in fixed C/POSIX locales. A UTF-8 `é` occupies two
bytes: `?` and one negated class do not match that complete two-byte string;
two such byte matches can. Exact UTF-8 literals outside classes remain supported.
Non-ASCII class members are excluded; negated ASCII classes can still identify
non-digit bytes in non-ASCII input. No Unicode-character or locale-aware claim.

Still excluded with an honest Unsupported diagnostic: named classes such as
`[[:digit:]]`, collating/equivalence forms, Bash-only `^` class negation,
descending ranges, adjacent active hyphens with ambiguous range roles,
non-ASCII class members, active dynamic backslashes, extended patterns, regex,
and whole-case pipes/redirects. Malformed/empty/unclosed classes and standalone
active closing brackets are outside this subset; some are literal or otherwise
valid patterns in Bash, so the diagnostic describes a simulator limit rather
than falsely calling all of them Bash syntax errors. Quote literal bracket text.

This extension does not add bracket filename globbing, bracket text in the
read-only parameter-operator WORD subset, computed array indexes, or new terminal
expansion. These separate namespaces keep their existing documented limitations.

## Evaluation, responsiveness and saved work

Completely literal patterns receive compile-first subset validation, including
quoted fragments. Patterns containing parameter/arithmetic/capture expansion
are checked after their reached scalar expands; an unselected dynamic pattern
is not evaluated. A selector/pattern capture's prior virtual effects and status
remain visible; matching adds no status reset or hidden command execution.

Only reached class patterns allocate a token vector. Admission reserves its
bounded capacity before allocation and keeps that capacity in the same pending
memory ledger while matching or capture continuation is suspended. Ordinary
literal/`*`/`?` patterns keep the prior zero-token-storage matcher path. Class
membership uses two fixed 64-bit ASCII masks and at most two word operations per
range, never a locale/regex library or recursive matcher. Matching still yields
after at most256 charged transitions per preparation step; shared instruction,
expansion-work, memory and output limits and Stop remain unchanged. An exhausted
limit is failure, never a successful no-match/fallback.

The 256-byte expanded-pattern limit, 32 arms, 8 alternatives per arm and 128 per
case remain. Existing source 16 KiB, live text 256 KiB, output 32 KiB, 2,000 VM
instructions and 1,000,000-byte expansion-work limits remain.

Genuine evaluator 11 data was captured **before** this change: 52 scripts/384
cases / 52 separate examples and two 142-context synthetic developer profiles.
Both preserve all 90 original non-script Attempts and prior 6–10 records; the
draft variant retains 2 complete unfinished assignment/example Attempts. Original
9/10/11 producers, raw profiles, observed outputs and receipts stay immutable.
Current 12 consumers preserve prior11 results as history and earn new current
credit only through actual supported execution. Direct9→12 or10→12 saves do
not invent intermediate receipts. Numeric variants do not become authored labs.

## Primary behavior and pending verification

Primary references are GNU Bash's
[pattern matching](https://www.gnu.org/software/bash/manual/html_node/Pattern-Matching.html)
and the POSIX shell's
[pattern matching notation](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html#tag_19_14).
The local original GNU Bash 5.3.9 manual is pinned at SHA
`b91026acffb159bd1e842e3116a95539bf858e2b57cbfa74df9e3a53379177a4`.
The 2026-10-28 direct fetch attempts timed out (GNU) or returned 403 (POSIX); this is
not a successful web recheck. Controlled fixed-source comparisons use real Bash
only as a development reference in owned disposable fixtures, never the learner
engine. The original 37-case Bash fixture remains exact; one source/case/hash
keyed feature amendment admits the previously excluded `[ab]` observation.

Before acceptance: actual reference comparisons for declared edge cases,
negative/alternate/resource checks, all 52/384/52 current replays, all historical
groups, native Stop/resume/editor behavior for a dependent unit, and ordinary
full tests/Clippy are required. A passing source parser or test count does not
establish beginner, screen-reader, other-platform or complete-course acceptance.
