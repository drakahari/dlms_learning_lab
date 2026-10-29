# One-off scheduling subset — before implementation

Source-only, unregistered foundation. Reuse current `shell::Vm` (there is no
existing public ShellTask type) behind a narrow, additive shell::one_off seam.
No evaluator revision or ordinary shell behavior changes. No at/atd/spool/host
clock/filesystem/service command is ever invoked, including reference testing.

At-like commands: `at -f /work/NAME -t YYYYMMDDhhmm`, flags in either order;
`at -f /work/NAME now [+ N minutes|hours|days|weeks]`; `atq`/`at -l`;
`atrm ID...`/`at -r ID...`; `at -c ID`. Strict ordinary flat virtual paths,
queue a only, UTC minute precision, years2000–2099 and relative horizon<=7days.
An absolute past/equal time becomes eligible immediately when the next virtual
Run begins, rather than silently moving to tomorrow. Unsupported natural-language
forms, seconds, zones/DST, batch queues/load, mail, -u, umask/real privileges and
interactive stdin submission are honestly rejected. CLI options do not pass to
a shell. at -c shows captured source and labeled simulator metadata, not a fake
real spool wrapper. Queue listing includes pending jobs only.

Submission captures the file TEXT and a bounded ordinary scalar environment,
owner and fixed virtual cwd /work; later file/env edits do not rewrite the job.
Root lists/removes all; supplied users list/remove/read only their own jobs.
No filesystem ownership/ACL enforcement is claimed by the virtual text store.
Access-file policy, multiple queues, actual daemon availability and live
concurrent process ordering remain later capabilities; do not claim them covered.

Scripts: <=8KiB, up to32 portable simple printf/echo/cat commands, comments,
literal arguments, single/double quotes, quoted named scalar expansion,
redirection supported by the existing VM. No cron percent preprocessing: `%s`
is an ordinary printf format. Reject pipeline/control/function/array/substitution/
loop/glob/dynamic command-name syntax through parsed code before VM execution.
Only /work files and fixed cwd; no host paths, subprocesses or networking.
Real at normally invokes /bin/sh; this explicitly disclosed in-process subset
uses no host shell. Malformed/unsupported job scripts may be queued, then fail
honestly at due time with bounded diagnostics and no fabricated success.

Typed state: explicit Moment clock, users, virtual files, next ID, bounded pending
jobs and simulator-only completion/removal records with owner/time/status/output.
16 pending jobs,64 records,32 files/96KiB aggregate file text,16 captured variables,
64KiB aggregate retained job output and1MiB encoded state. Early bounded visitors
reject excess map/sequence bodies and long copied strings. Run owns an admitted
snapshot, orders due jobs by requested time then job ID and executes serially;
actual concurrent at behavior is outside scope. At most32 scheduler/VM work units
per tick, VM's existing instruction/output/memory limits also apply. A whole
advance is at most7days. Stop drops unfinished window changes/results and leaves
the caller's queue/files/clock untouched, matching the existing scheduling
simulator's explicitly transactional preview rather than pretending OS rollback.

Reference sources checked2026-10-28: official POSIX at page403(unavailable; no
bypass), Debian packaged at3.2.5-2.2 manual source dated2025-05-04 documents -f,
-t, relative forms, owner-visible queue/removal, captured environment/cwd and
past absolute ASAP behavior. No real at is run. Core, adapter, authored-content and native gates now have
actual evidence; see ONE_OFF_LESSONS.md for the exact four-task scope and
remaining candidate/platform acceptance. Full107.2 coverage is not claimed.

## Concrete foundation behavior

Queued script text, owner, environment and fixed cwd are captured; source file
edits cannot rewrite an already queued job. Other virtual file contents stay live:
each dispatch supplies the current virtual file map, including changes made after
submission and effects of earlier jobs in the same window. Environment values
are ordinary scalars only. HOME/SHELL/USER/LOGNAME are supplied consistently by
the model; PATH/locale/working-directory customization and special Bash names
are rejected rather than pretending to model them.

The portable seam compiles through the existing parser, inspects the complete
code shape, and starts the same compiled Vm. Accepted scripts are not rewritten
into canned actions. Loops, pipelines, arrays, functions, substitutions, dynamic
command names and assignment forms are rejected before any accepted prefix
executes. Runtime utility failures use the existing VM's diagnostics and exit
status; like a shell without automatic error-exit, a later successful command
can set the script's final status to zero while earlier stderr remains visible.
printf percent formats are ordinary here, with no cron percent transformation.
Current utility restrictions (printf formats, echo without option/backslash
forms, literal virtual cat files and disclosed redirection) remain in force.

Jobs are removed from the pending queue once executed, including failed jobs.
Later jobs still run. atq exposes pending jobs only. The bounded result/removal
ledger is labeled simulator output: it is not real at mail/spool output. Retain
the latest64 events within64KiB total output/diagnostics, evicting oldest events;
this history cannot become future grading authority. Owner-visible queries and
root-wide listing/removal are modeled, but filesystem ACLs and at.allow/deny are
not. Multi-ID removal validates all owners/IDs first and is atomic in this
simulator; do not teach that as a portable real-atrm rollback guarantee.

The state has a1MiB encoded bound. from_json checks raw size before decode.
Individual strings, maps and sequences also have pre-copy/pre-body limits for
embedded Deserialize; this does not imply allocation-free JSON or that a caller
can omit validation of a directly decoded State. Submission/edit/removal validate
first, construct bounded candidate state and publish only after complete state
validation. An advance owns one admitted snapshot plus one current VM; no learner
input reaches host I/O. Tick budgets cap scheduler and VM progression independently
of outer render frequency. The native adapter separately enforces once-per-frame advancement, total
presentation/Attempt budgets and responsive Stop; see ONE_OFF_WORKSPACE.md.

## Evidence and integration gates

Twenty-one core groups cover capture versus live input, calendar boundaries,
absolute/relative alternatives, past-due execution, due/ID ordering, ownership,
atomic removal, preflighted syntax/unsupported failures, status, quotes and
redirection, Stop, budgets, escape boundaries and state roundtrip. Additional
workspace/content/UI/native acceptance is recorded in ONE_OFF_LESSONS.md. No
real at process is executed. The normal State/Goal, pack and UI adapters use this
core without treating stored history or script spelling as completion authority.
The four new tasks add no shell evaluator revision or claim of full107.2 coverage.

Sources checked2026-10-28: Debian's at3.2.5-2.2 manual, source last updated
2025-05-04, https://manpages.debian.org/testing/at/at.1.en.html. It describes
-f/-t, pending ownership/list/removal, relative specifications and captured
environment/cwd. Official POSIX at pages at both9799919799 and9699919799 returned
403; their contents were not independently fetched and no access bypass was
attempted. No manual passages are copied into curriculum; this original subset
contract and implementation retain the project's approved MIT license.

## R2 admission and reference clarification

Captured names require 1..32 ASCII characters: a letter or underscore first, then
letters, digits or underscores. Reserved/special variables remain excluded. The
portable script seam also rejects OPTIND, OPTARG and OPTERR in both $NAME and
${NAME} reads before any script command executes; its ordinary-name validation
does not expose implicit getopts values from the underlying VM. Single-quoted
literal text such as '$OPTIND' remains ordinary data. No general shell evaluator
semantics or revision changes.

One-off virtual filenames have a deliberately stricter rule than the underlying
shell lesson engine: /work/ followed by 1..100 ASCII letters, digits, dots,
underscores or dashes, excluding `.` and `..`. Unicode, spaces, slashes and nested
folders are not in this foundation. A dynamically produced unsupported output
name fails with an unsupported-name diagnostic, separately from exceeding file
count/text limits; no staged changes publish. This is a disclosed simulator
restriction, not a restriction of real Linux filenames or real at.

Authoritative LPI v5.0 objectives were rechecked2026-10-28 at
https://www.lpi.org/our-certifications/exam-101-102-objectives/:107.2 includes
one-off at/atq/atrm and separately requires access policy and systemd timers.
Those latter capabilities remain open here. LPIC-2 206.1 concerns installing
software from source; it is not a scheduling objective and is not associated
with this foundation. Existing curriculum statuses and CSVs remain unchanged.
