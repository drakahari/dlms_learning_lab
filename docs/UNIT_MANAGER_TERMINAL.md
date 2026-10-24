# Closed virtual service terminal and journal

2026-10-23. Backend implementation following backend63f08f4, independently
source-reviewed and verified by the focused checks below. No native UI, bundled
curriculum or objective coverage is claimed.
Preserve old State::Service dispatch/output/progress. New behavior belongs only
to State::UnitManager, its fixed Inventory and isolated example counterpart.

## Boundary and command inventory

Use the existing literal-argument lexer, not a host shell or the Bash interpreter.
Whole input at most2048 UTF-8 bytes/32 tokens. Full supplied `.service`/`.target`
names only; one unit per operation. Parse/validate every argument before mutation.
Quotes/backslashes may form literal arguments; no expansion, pipes, redirection,
command substitution, globs, chaining, sudo, host paths, remote/user instances,
implicit suffixes, templates, aliases, pagers or external tools. Unknown forms give
an honest supported-subset diagnostic and leave state unchanged. Never forward
input to a process, filesystem/network operation or systemd API.

Implemented unit-tested backend slice (no native UI adoption):

| Command spelling | Pure model operation or observation |
| --- | --- |
| `systemctl status NAME` | Current manager/runtime diagnosis; bounded familiar Loaded/Active lines and clearly marked simulator program detail |
| `systemctl cat NAME` | Exact saved backing unit text, not loaded source |
| `systemctl show NAME -p PROPERTY[,PROPERTY...]` | Explicit loaded-property subset below |
| `systemctl is-active NAME` | Current active/inactive/failed classification |
| `systemctl is-enabled NAME` | Enabled/disabled/static classification, separately from current runtime |
| `systemctl daemon-reload` | Typed DaemonReload; strict loader policy unchanged |
| `systemctl start NAME` | Typed Start plus the disclosed post-job event below |
| `systemctl stop NAME` | Typed Stop; current runtime only |
| `systemctl enable NAME` / `disable NAME` | Typed install policy actions, with existing implicit manager reload |
| `systemctl get-default` / `set-default NAME` | Read/change future-session target, never switch current runtime |
| `journalctl -u NAME -o cat [-n N]` | Retained typed virtual journal messages; requires event-store addition below |

Accept documented long synonyms `--unit=NAME`, `--output=cat`, `--lines=N`,
`--property=...`; support `--no-pager` harmlessly since no pager exists.
`--quiet` only for is-active/is-enabled. Accept options before/after operands;
reject repeated/conflicting/irrelevant options explicitly. N is decimal1..256;
bare `-n`, `all`, plus-prefixed counts and other outputs remain unsupported in
this first slice. No implicit conversion of a requested short/JSON log format.

Require explicit `-o cat` initially: it is a real message-only output mode and
avoids fabricating hostnames, PIDs, clocks or boot IDs. A missing output mode
explains how to request the supported mode. If ordinary short output is desired
later, specify its actual typed virtual timestamp model before implementing it.
No `restart` shorthand yet: Stop+Start alone does not establish faithful restart
propagation for requiring dependents. Add a separately reviewed typed Restart
algorithm before accepting that command. Likewise defer reload, mask, isolate,
--now, reset-failed, list-unit-files/list-units, multi-unit transactions and broad
show properties; do not quietly approximate them.

## Readable queries and honest status

Queries never apply editor text, reload definitions, initialize a program or
advance time. `cat` reads saved source. `show -p` reads loaded Description,
Requires, Wants, After, Before and ActiveState only; it rejects other properties.
Relations are effective DIRECT modeled relations: After/Before include inverse
declarations and targets' default ordering complements, and Wants includes current
modeled installed links. Saved WantedBy metadata alone is not an installed Wants
edge. The core inspector reuses the very same direct edge derivation as scheduling;
it does not project a transitive ordering path as a direct property. Its sources
are loaded definitions and current links. Known-but-unloaded Inventory targets
remain visible dependencies, and no host implicit units are invented.
DefaultDependencies=no remains explicitly Unsupported; an unapplied saved edit
cannot change effective loaded properties.
In particular do not invent MainPID/ExecMainStatus/Result/ExecStart structures,
or expose NeedDaemonReload as an exact real property: the model tracks byte
coherence, not filesystem timestamps. The native view can accurately explain
saved/loaded differences in its own words.

Status omits unmodeled process/cgroup/CPU/wall-clock fields. Known loaded units
show loaded; a known but not loaded definition gets an explicit simulator
reload-needed diagnosis, not a false real not-found claim or lazy-load side effect.
Unknown names outside fixed Inventory get a context diagnostic, not host lookup.
Malformed saved install information produces a query error, not a bogus is-enabled
`bad` line. The closed enabled classification applies to valid saved unit text:
modeled links mean enabled; without links, WantedBy means disabled, otherwise
static. It does not claim aliases/generators/runtime installations are modeled.

Manager-active includes Running, active/exited oneshots, active targets and a
Dispatched simple start. Inactive/failed are nonactive. Do not equate active or a
zero query status with readiness, correct consumed inputs, future enablement or
assignment completion. Static is-enabled output may return zero without an
install link. Existing typed current/fresh outcomes remain the only grader.

Use common query status conventions for supported states: active0, inactive or
failed3; successful mutations0, failed requested jobs nonzero. Failure of a weak Wants
child does not turn a successful requested target job into a failed command. Unsupported/syntax/context
results have a distinct simulator diagnostic category; do not pretend full Linux
error-code coverage outside the declared domain. Nonzero diagnostic queries are
normal investigation, not malformed command syntax. All text has word labels in
addition to color. No raw true/false cards or Rust Debug enum dumps.

## Deterministic post-job program event

Start first obtains its real model Job report. Its exit status derives only
from the requested job, not an unrelated failed weak dependency; then one bounded AdvancePrograms event evaluates pending simple programs
before returning the terminal prompt. This disclosed deterministic event boundary
removes UI races without pretending start waits for application readiness. A
simple start can return0 while the subsequent fictional configuration check
produces Failed; status/journal then explain that failure. One-shot failure occurs
inside its start job and returns nonzero. No automatic retry or self-repair.

Typed native Start controls must use this same controller policy, not a separate
hidden progression path. Queries never advance it. Explicit low-level developer
core actions can still expose Dispatched; status then shows manager-active with
`Simulator: initialization pending`, not an invented Linux substate. Goal checking
must never initialize the main State: its existing fresh clone behavior stays
separate and cannot repair current work or replace current execution evidence.

Pending editor changes are retained. Terminal actions use applied State only;
a previously clean draft may synchronize without changing its selection, while
divergent unsaved text remains and blocks current completion. Example terminal,
selection, draft, event journal and history stay fully separate from assignment.
Challenge changes guidance, not grammar or task/grade requirements.

## Optional typed event journal

The new core has an optional bounded journal. It is emitted from committed typed
job results, not manufactured from command text or repeatedly inferred from the
current failure. Low-level typed buttons and terminal actions share these events.
Saving, reloading or changing install/default policy creates no program-output
event. The original single-service model remains unchanged.

At most256 retained events: checked monotonically increasing sequence,
supplied unit ID, closed event kind, bounded typed cause and at most512 UTF-8 bytes
of generated diagnostic detail. Events come from actual start jobs, simple
initialization and stops; saved text/reload/install messages are not fictional
program stdout. No PID, real date, credential, network result or full source copy.
Prune oldest on the documented count boundary with a dropped count visible in
inspection; never claim the retained window is a full journal. Sequence overflow
fails before mutation rather than wrapping. Validate all names/counts/schema and
UTF-8 detail bounds before clones/publication, including direct callers.

The event store is historical display, not success authority. Imports can retain
bounded history, but current runtime/snapshots and fresh jobs are re-derived for
grading. Current failure with an old Started event must still fail. Empty default
serialization preserves old new-kind wire shapes; original Service remains
untouched. A fresh-session clone starts a new virtual event window while leaving
the caller's window unchanged; no real boot/journal persistence claim.

Journal filtering uses exact supplied unit identity, ascending retained sequence;
-n takes the newest matching N but preserves their chronological order. `-o cat`
prints only stored message bodies. No-match is a successful empty result. Output
is capped32KiB: preflight and diagnose a too-large request with advice to use -n,
rather than silently printing a successful partial result. Unsupported formats,
live follow, boot/time/priority filters, arbitrary fields, journal files and host
paths fail honestly. Control bytes in raw `cat` source must be safely represented
or explicitly refused for terminal display; stored editor bytes are unchanged.

## API, isolation and review gates

The standalone unit_manager_terminal::parse and execute APIs returning bounded typed
stdout/stderr/status plus change classification. Reuse the goal-aware workspace
mutation boundary and core event controller; no command-to-success recognizer.
Execution preflights all arguments, acts only in this assignment/fixture and
returns partial job failures honestly. No journal record, output or exit status
awards credit. General Bash scripts cannot invoke these utilities until that
separate interpreter integration is designed; do not claim shell automation.

Tests before content: accepted/unsupported argument matrix; parsing is atomic;
saved cat versus loaded show; enable/active/default independence; static output
without link; simple0-then-Failed versus failed oneshot; matched Requires ordering
and Wants continuation; current old inputs versus new saved config; missing/invalid
loaded definition; pending draft and example isolation; Step0 success broken again;
journal real-event creation/filter/tail/retention/overflow/control/output bounds;
old Started entry never stale success; fresh event clone unchanged caller;
malformed journal/profile/foreign origin; exact existing98/current7/legacy6
save/export/restore/reset. Original Service parity remains required. Native
focus/scroll/status readability and curriculum review follow implementation, not
this proposal. No coverage or platform/runtime acceptance is promoted.

## Primary references actually opened

Official versioned manuals identified themselves as systemd262 and were opened
2026-10-23. No host systemctl/journalctl or real service was run for this review.

- https://www.freedesktop.org/software/systemd/man/262/systemctl.html — cat,
  query/install/default distinctions and status conventions. Its is-enabled table
  explicitly distinguishes static from an enabled link; invalid files report an
  error rather than a literal is-enabled bad state.
- https://www.freedesktop.org/software/systemd/man/262/systemd.service.html — the
  simple start-job boundary can precede program initialization; it is not readiness.
- https://www.freedesktop.org/software/systemd/man/262/journalctl.html — exact-unit
  filtering, recent-line selection and cat output without timestamp metadata.

These sources constrain spelling/meaning, not a claim that this closed virtual
model reproduces full systemd, journald, Linux boot or process supervision.

## Source/API and persistence details

`State.journal` is optional/default-absent. Existing states with no events encode
without this field; State version1 and the existing workspace version stay1.
The journal's independent version is1. Its private fields are `last`, `dropped`
and at most256 events. Every event has a contiguous checked sequence and known
unit; its closed typed event may retain a known dependency or <=512-byte UTF-8
error detail without controls. Both count and scalar bounds apply while decoding,
then State validates fixed Inventory context. Unknown fields/kinds, duplicates,
future journal versions and invalid counters reject. A forged valid historical
event is still merely display data; it cannot prove current execution.

Typed core `apply` constructs one candidate, records its actual report and fully
validates before replacing State. Journal overflow therefore fails atomically.
The terminal `perform` also stages Start and its single AdvancePrograms together;
a validation error during that later step discards both candidate actions and
leaves the exact original State/journal. Actual job failures remain committed
model outcomes. The original start-job status is preserved even when subsequent
program initialization fails. No query calls AdvancePrograms.

`parse(input, inventory)` returns a closed Command. `execute(definition, state,
input)` pins the full validated immutable assignment; `perform` is the same
controller for future typed native buttons. `read_only(state, input)` provides
inspection without any mutation fallback. Native/engine/teaching command routing
has not been wired in this source slice. Unsaved draft synchronization and
example/assignment terminal focus must be reviewed at that later integration.
These commands are unavailable inside script labs: the existing shell evaluator
and evaluator revision7 are unchanged. No current success path grades any
terminal outcome, status or journal message.

A status query exposes how many older events the retained window discarded.
`journalctl -o cat` remains message-only, without an invented retention message
inside the event stream. Safe display escapes control bytes in raw saved source;
this does not rewrite the raw editor document. Output is bounded32KiB and an
overlarge query returns a limit diagnostic rather than a successful truncation.

The17 standalone tests in `tests/unit_manager_terminal.rs` pass: closed syntax/options and honest unsupported forms; saved/loaded/
active/install/default distinctions; original simple job status vs initialization;
ordered failed dependency events; exact rollback at late sequence overflow;
retention/tail/fresh-clone isolation; malformed/foreign/oversized decode; full
512-byte error details and output limits; current-input failure despite old
successful history; foreign context and direct oversized actions. Existing
core/workspace/progress/current7+historical6 tests remain mandatory regression
gates. Nothing here constitutes native Fedora, novice, screen-reader or platform
acceptance.

## First source review corrections

The original seven source-ready paths/manifest were preserved and hash-verified
before these changes. Independent review found that command status initially
examined every transaction job and that show initially printed only literal
parsed ordering/Wants lists. Those drafts were not compiled or accepted.
The corrected source derives status from the requested Start/Stop job alone and
uses `State::effective_properties` for direct modeled relations. Source-ready
regressions cover weak Wants failure with target success, blocked ordered Requires,
explicit/inverse edges, target ordering, live installed links versus WantedBy,
nontransitive output, known missing references, saved-only changes and the honest
unsupported default-dependency override. Existing example/repair tests have been
source-audited: whole-State equality concerns exact rollback/reset/roundtrip or
assignment isolation. Repair acceptance uses fresh/current goals; historical
journal entries are retained rather than erased to simulate equality.

## Actual backend verification

The shared guarded runner executed130 distinct Rust checks on this isolated
baseline:48 library,16 pure manager,17 terminal/journal,14 workspace,10 manager
integration,7 backup and18 shell-substitution migration tests. Every target passed
its first run; all-target Clippy with warnings denied also passed. First frozen
and corrected source manifests/logs are retained in ignored review evidence.
No historical journal entries were removed to satisfy a State equality test.
The existing progress tests exercise the original98 scenario contexts, actual
revision6 histories and current revision7 script runs on this branch; integration
onto a later catalog still needs that main checkout's own regression gate.

This verifies pure actions, query output, atomic error handling, decoding and
persistence behavior within the declared test scope. It does not establish native
terminal input/focus/scrolling, example command routing, beginner usability,
actual systemd equivalence or Windows/macOS runtime acceptance. The new standalone
terminal API remains unwired to the engine/native workspace in this milestone.
The shell source/evaluator revision and original single-service path are unchanged.
