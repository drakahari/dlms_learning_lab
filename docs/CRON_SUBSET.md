# Scheduling subset — foundation and authored unit contract

Four authored recurring-job assignments are integrated after the exact previous160
Main definitions, with one unit after the exact previous41. Main source has164
assignments/42units. The earlier standalone158/40 evaluator10 source epoch and
its failed/scoped gates remain retained separately; they are not Main11 acceptance.
The normal full-task editors, Run/Stop, separate practice and continuation memo
are implemented. Actual Main native77, final candidate and cold-reopen receipts
are pending in CATALOG164_MILESTONE.md, without any novice/platform claim.

This is a deliberately bounded Cronie-style model, not a cron daemon or complete
shell. Only the named recurring-job/access/configuration slices of LPIC-1 107.2
are Partial; one-off jobs, timers, concurrency and full CLI remain absent.

## References checked2026-10-28

- [LPI LPIC-1 objectives5.0,107.2](https://www.lpi.org/our-certifications/exam-101-102-objectives/): recurring jobs, one-off jobs, access and timers remain distinct requirements.
- [LPI LPIC-2 objectives4.5,206.2](https://www.lpi.org/our-certifications/exam-201-202-objectives/): backup is a separate unimplemented practical frontier.
- Cronie source revision **cronie-1.7.2**: [crontab5](https://github.com/cronie-crond/cronie/blob/cronie-1.7.2/man/crontab.5), [crontab1](https://github.com/cronie-crond/cronie/blob/cronie-1.7.2/man/crontab.1), [calendar decision](https://github.com/cronie-crond/cronie/blob/cronie-1.7.2/src/cron.c), [percent handling](https://github.com/cronie-crond/cronie/blob/cronie-1.7.2/src/do_command.c), [field provenance](https://github.com/cronie-crond/cronie/blob/cronie-1.7.2/src/entry.c).

These are behavior references; no upstream implementation or manual prose is
imported. Existing project/third-party notices remain unchanged. The official
LPI objective pages still identify5.0/101-500+102-500 and4.5/201-450+202-450;
the overview pages timed out. Existing granular source fingerprints retain
their2026-10-22 date; this check does not silently redate them.

## Calendar and table language

The clock is explicit virtual UTC, at minute resolution, Gregorian2000–2099.
No host clock is consulted. Invalid dates, leap-day mistakes, invalid fields,
reversed ranges and bounds fail before execution. Five fields precede a command
in user tables; system tables additionally require a supplied account name.
Spaces/tabs, blank lines and leading-# comments are accepted. Nonempty tables
must end in LF. CR/NUL and multiline commands are rejected.

Fields accept decimal values, inclusive ranges, comma lists, `*`, `*/n` and
`a-b/n`. Month/day-of-week names use case-insensitive three-letter names.
Sunday0 and7 are equivalent. A step restarts within its field, not after an
elapsed duration. `*/35` minutes means0 and35 each hour. Wildcards inside a
comma list and bare `a/n` are explicitly outside this subset.

The original leading wildcard is retained separately from the selected set.
Minute/hour/month must match. If either day field begins with a wildcard, both
day fields must match; otherwise either may match. Thus `1-31` and `*` need not
be interchangeable when a weekday is restricted. Tests must include `*/2`,
Sunday aliases, month/year/leap boundaries and impossible month/day pairings.
Nicknames, random ranges/delay, timezone/DST jumps, seconds and reboot are not
implemented. A syntactically valid impossible date simply never matches.

Environment declarations affect following entries only and are literal (no
shell expansion). Bounded ordinary scalar variables are supported. The model
supplies LOGNAME/USER from the actual selected job owner, HOME=/work and
SHELL=/bin/sh. The artificial flat `/work` exercise filesystem is explicit;
cwd changes, home-directory administration and PATH lookup are not taught.
Unsupported Cronie control variables and changes to HOME/SHELL/LOGNAME/USER/PATH
fail honestly. No mail, PAM, SELinux, spool file ownership or daemon reload is
simulated. User crontabs are installed by a modeled crontab action, never by
writing host spool paths.

## Closed portable command seam

Cron command text is first checked for Cronie's percent rule. An escaped `\%`
becomes literal `%` before shell parsing, even inside shell quotes. Unescaped
percent/stdin splitting is explicitly rejected in this first subset; it must not
silently execute as ordinary printf syntax. Odd/even preceding backslashes must
be tested.

The existing shell parser and bounded VM remain the only execution engine.
An additive AST preflight admits one simple portable `printf`, `echo` or `cat`
command, supported redirections, literal quoting and double-quoted
`"$NAME"`/`"${NAME}"` scalar expansion. Unquoted expansions are rejected to keep
field splitting and generated globs out of this first scheduling subset.
Command names must be literal; pipelines, command substitution,
arithmetic, arrays, functions, conditions, loops, assignments and newer parameter
operators are rejected in this scheduling subset. Existing shell labs retain
all their existing capabilities/evaluator semantics. Unsupported paths and
utility options still produce the existing honest diagnostics.

## State, access and resources

All accounts, tables, policies and files are supplied virtual data. Up to8
accounts,8 user tables,16 total entries,8KiB/table,2KiB/command,16 environment
variables,32 files (32KiB each) and96KiB total virtual file text are accepted. Strict JSON
loading rejects unknown/duplicate keys, excessive collections and oversized data.
No host filesystem, subprocess, network or service API is part of this module.

Root may manage supplied user tables and the system table. A regular account
may manage only its own table if cron.allow/cron.deny policy permits. An existing
allow file takes precedence; otherwise a deny file excludes listed users; if
neither exists only root can use crontab. Access policy affects management,
not already installed execution. Removing an installed table is a separate
explicit action. Readability/PAM rules remain outside this bounded policy.

Simulation runs through an explicit future window of at most7 days, excluding
its starting minute and including its final minute. At most64 job dispatches,
64KiB aggregate output and the VM's existing instruction/memory/output limits
apply. A run owns its working copy, yields after a bounded tick budget, and
publishes a result only when complete. Stop discards unfinished simulated changes;
it never changes the caller's saved state. Jobs run in deterministic serial
order; concurrent duration/overlap, locking, at and systemd timers remain absent.
No objective credit may be claimed for those missing behaviors.

## Four distinct registered assignments and prerequisites

1. A recurring report: configure a user table and prove wanted/unwanted dates.
   Requires lpic1-file-navigation and shell-handover; teaching introduces each
   calendar field before the full assignment.
2. A noninteractive report: repair explicit input variables, escaped printf
   formatting and append/output retention. Requires previous task and shell-report.
3. A delegated account: repair table-management policy, then explicitly remove
   an unwanted installed job while preserving another account's table. Requires
   previous task and lpic1-account-onboarding.
4. A coordinated handover: user/system format, correct run identity, replacement
   publication and preserved peer jobs/files under several fresh sample inputs.
   Requires previous tasks; Learn and Challenge retain the same complete task.

The authored pack contains introductions, separate ungraded examples, full
assignments, two supported alternatives and independently reasoned wrong/partial
vectors. Main content, behavioral and event gates cover library, edit/install/
remove/Run/Check, practice isolation, reset/resume, saved output and receipt
readback. The normal controller has77 capture phases including40 alignment
observations at all five supported text sizes in light/dark wide/narrow modes.
Headless event tests execute capture code but do not produce real native PNGs.
The Main native77 receipt is explicitly pending in CATALOG164_MILESTONE.md; no
packaged, novice, screen-reader or other-platform acceptance follows from tests.

## Native execution and saved work

Run and fresh-sample Check own separate continuations. A frame shares at most64
calendar/VM steps across them; Stop discards private unfinished work. The native
cache compares the complete borrowed Scenario and Attempt before a warm hit,
with no JSON metering, saved-state clone or VM execution on an unchanged hit.
Four entries,8MiB per entry and32MiB aggregate include working/report/editor/run
reserves, including a separate3MiB per-entry allowance for the fresh-case Run,
its bounded VM/entries/output and transient state copies alongside the retained
Check context. The single active learner Run owns a separately admitted copy. Complete
Attempt2MiB admission precedes edits or publication; rejected edit/action packets
retain all prior fields, cursor/Undo state and other work.

Completed Run history records actual virtual job owners, times, output and exit
status separately for practice and assignment. It is resumable output, never a
grading receipt. Checking uses installed behavior and fresh supplied dates/files.
The optional simulator-only retry restores only the selected activity's supplied
files and initial virtual clock, clears its run output, and leaves installed
tables, policies and drafts intact. It never auto-runs or awards completion.
