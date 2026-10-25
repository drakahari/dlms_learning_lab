# Bounded getopts extension — evaluator9 source milestone

Source implementation prepared2026-10-25 and integrated for validation. The
first library/integration run passed991 checks with one stale test sentinel
failure; the corrected18-test migration target passes. Thus992 distinct model/
content checks pass across the full run and scoped correction. The correction
tests both current/future history rejection instead of hardcoded8, which is now
legitimate history; production migration code is unchanged. All202 binary/UI checks, all-target Clippy and formatting pass. Installed
revision9 acceptance remains pending. This is not a new authored unit. Existing40 script definitions/246 cases,
40 separate examples, all scenario IDs/versions/Goals, dependencies and themes
are preserved. There is no host command, process, file, credential or network
API in the learner interpreter, and no new dependency.

## Supported scanner

`getopts OPTSTRING NAME [ARG...]` returns one option per invocation, using
explicit arguments when present and current positional parameters otherwise.
OPTSTRING is empty or unique ASCII letters/digits, each optionally followed
by a required-argument colon. One leading colon selects silent errors. Flags
may be clustered/repeated. Required values may be attached or separate; empty,
space-containing, option-looking and exact `--` values are consumed verbatim.
No stdin is consumed and no operands are permuted. NAME is an ordinary identifier;
an existing dense indexed value keeps its tail when element0 is updated.

The current argument index and position inside a cluster are separate typed
state. Completing a cluster advances OPTIND; a partial cluster leaves it at
the current argument. End gives NAME `?`, unsets OPTARG and returns1. Ordinary,
empty and lone-hyphen operands are not consumed. An exact `--` is consumed and
returns1 for that call, but a subsequent call can scan the following option:
there is no permanent end latch. A value required by a preceding option consumes
`--` as ordinary data instead.

Flags return0 and unset OPTARG. A required value returns0 with OPTARG set, even
when empty. Unknown options return0 with NAME `?`. Missing required values return0
with NAME `?` normally or `:` in leading-colon mode. Normal errors unset OPTARG;
silent errors set it to the offending option byte. OPTERR=0 suppresses normal
diagnostics; leading-colon mode always suppresses them. The retained Bash script
prefix/error wording is evidence, while simulator diagnostics are deliberately
short and not claimed byte-identical to Bash.

Each VM invocation starts with scalar OPTIND=1, scalar OPTERR=1 and OPTARG unset.
Fixture OPTERR is overwritten at startup, as actual Bash does in the controlled
observation. Fixture OPTIND/OPTARG remain excluded. Ordinary OPTERR scalar/text/
array assignments, reads, locals and capture behavior are retained. At reached
getopts only, require scalar OPTERR exactly0 or1; other values/types fail with
Unsupported. This is a disclosed simulator limit, not Bash invalidity.

Exact scalar value1 assigned to OPTIND resets the entire scanner, even if it
already equals1. Ordinary supported expansion may produce this value. Bare
integer expressions, arbitrary seeks,0/empty/noncanonical values, arrays and
read/for targets for OPTIND are excluded. OPTARG is readable (including scalar
array count to distinguish unset from set-empty), but user writes/localization
are excluded. No unsupported write path can bypass these restrictions.

Until reset, OPTSTRING and the entire effective argument vector must remain
byte-identical. Rebuilt equal strings/lists work; a pointer identity or digest
alone is never authority. Changing lists or declaration without reset fails
honestly instead of approximating wider Bash behavior. NAME can change separately.

## Function, capture and pipeline state

Ordinary function calls share the nearest current scanner. The first
`local OPTIND=1` in a genuinely entered function moves the caller's scanner
into that frame and starts fresh. Repeating it resets the current scanner,
retaining the original saved caller. Nested localization creates another scoped
scanner; an unlocalized helper uses its caller's local scanner. Return or ordinary
function completion restores the saved scanner by move. Ordinary output/local
variable restoration remains independent; printing restored `opt=a` immediately
after return is not a getopts result. Uninitialized local OPTIND remains excluded.

Captures copy the active scanner, including hidden cluster position; parent
state is restored on return. Every isolated pipeline stage starts from the
same saved parent scanner; its updates are discarded. Existing virtual-file,
finite-input and child isolation rules remain. There is no synchronous child
VM, fresh resource budget or host parser. Existing restrictions on direct
inherited local/return, function pipelines and redirection of whole function
calls are unchanged. Only explicit script Run or confirmed restore recheck can
create a new current result.

## Validation and limits

Exclude duplicate/punctuation/non-ASCII declarations, optional-argument `::`,
plus modes, long-option recognition, indexed/attribute/nameref output targets,
special-variable aliases and non-ASCII option bytes. Arbitrary UTF8 option values
and operands remain data. Existing unsupported shell grammar remains excluded.
Malformed declarations, changed-input reuse and unsupported contexts are error
Outcomes; they do not masquerade as normal end/status1 or Bash usage statuses.

Prepare one transition and validate its entire variable write set, including
names added/removed and hidden-parent names, before publishing scanner/NAME/
OPTIND/OPTARG. Invalid admission leaves those writes unapplied. This does not
undo preceding argument captures or already applied left-to-right redirections.

All retained argument identity, OPTARG values, local saved scanners, capture/
pipeline duplicates and transaction scratch count toward existing256KiB live
text. String slots count as well as bytes; comparison/copy/scan/reset work shares
the1,000,000 processed-byte budget. Reserve before cloning, compare borrowed lists
without recurring full clones, move local snapshots, and retain no extra argv
copy merely for function entry. Pending take-out and cached suspended ledgers
include the scanner. The existing2000 instruction,128 expanded-field,
32 aggregate-call,8 capture and32KiB output bounds remain. Preserve the original
256 distinct ordinary-name allowance, including unset/hidden local names, plus
at most three fixed names OPTIND/OPTERR/OPTARG (259 distinct live names maximum).
Every special name/value and clone still consumes the unchanged byte/work caps.
Unsetting OPTARG cannot free an ordinary slot. Scanner/special-variable byte
storage can still affect earlier near-byte-limit inputs; the ordinary-name
boundary itself is preserved under9. Stop clears every scanner/snapshot and cannot
resume, publish parent effects or retain current grading.

## Actual reference provenance and pending gates

Primary original GNU Bash Reference Manual Edition5.3,18May2025, installed HTML
SHA256 `b91026acffb159bd1e842e3116a95539bf858e2b57cbfa74df9e3a53379177a4`;
sections getopts, OPTIND, OPTERR, local and dynamic function scope. Official links:
[builtins](https://www.gnu.org/software/bash/manual/html_node/Bourne-Shell-Builtins.html),
[variables](https://www.gnu.org/software/bash/manual/html_node/Bash-Variables.html),
[functions](https://www.gnu.org/software/bash/manual/html_node/Shell-Functions.html).
Online fetches timed out in the latest recheck; the installed primary manual was
actually read. POSIX Issue8 retrieval failed403; no full POSIX/Bash conformance
claim or upstream implementation/code import is made.

Actual Bash5.3.9 results are retained in
`tests/fixtures/getopts-bash539-observations.json`:50 original observations from
manifest `d6b5777260f45c9551f1df013c531cbd0547b3f234edfa369e913c8b49cca827`,
plus5 distinct-offset/local/helper/error followups from
`f15f29582e1302622f444e2ae5c7df72fa2e6bc9dbcdaef6a55dd92276dea19a`.
Each row retains its original observation hash. One uninitialized-local case is
explicitly excluded; two normal-error cases allow disclosed diagnostic wording
variance. The other observations are exact stdout/status/files/stderr gates.
These are original Bash observations, not generated9 results.

The exact captured synthetic writer8 fixture is396309bytes, SHA256
`88c07f20276ba1555eaad886ec2a58eff3d3b3cb3cef2bf3108be64a63bacd26`.
It contains40 actual8 main records plus37 genuine7/33 genuine6 histories and116
original contexts. Three later PTR contexts absent from it remain ungraded;
they are not invented by migration. Startup OPTERR default/environment changes
require revision9 independently of new command support. Existing generic history/
writer code retains6/7/8 and creates an exact before-shell-v9 target copy on first
write, carrying only actual prior receipts (direct7→9 is[7,9]). Imported receipts
cannot authorize skipping a target. Existing before8 backups remain untouched.

The new VM/budget/migration checks passed in the M101 source gate, including all
55 retained Bash observations with explicit exclusions/diagnostic differences.
FullR1 and scoped migrationR2 logs are retained; there was no single all-green
full rerun after the test-only sentinel correction. Native acceptance is pending.
Original8 test source was archived before current-boundary assertions were
generalized; actual6/7/8 fixture bytes and old observations were not relabeled.
Future9 before/after comparisons must enumerate intentional differences rather
than regenerate old output. No new getopts lesson or coverage credit is authored.
