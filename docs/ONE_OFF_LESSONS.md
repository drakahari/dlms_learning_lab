# One-off jobs: four authored assignments

Four distinct LPIC-1 tasks are bundled in `linux-one-off-jobs`. They preserve the
complete176-lab predecessor and add meaningful queue/source/environment/live-file
practice. This is narrow107.2 practice, not a completed objective or certification.
No host scheduler, shell, clock, service or filesystem receives learner input.

| Assignment | Different practical work | New concepts taught first |
|---|---|---|
| Queue and verify a one-time report | Save a script, queue one job and verify a live-input report after a real simulated interval | Script/comments, editor versus Save, cat and output replacement, relative/absolute UTC, atq, at -c and explicit Run/Stop |
| Repair a captured job without losing live updates | Inspect and remove the stale captured job, repair source and captured label, submit again, update live input after submission and verify | Captured text/environment versus live files, ordinary variable editor, quoted expansion, printf `%s` and `\n`, append versus replacement |
| Retire an obsolete job and preserve another owner's work | Inspect own/root-visible queues, cancel only obsolete work and deliver a pending tested replacement while retaining the peer | Owner-visible queue, IDs versus owners, at -l/at -r aliases, pending-work handover versus actual current execution |
| Hand over an ordered two-job report | Replace stale work with an earlier producer and later consumer; verify staged and final files while retaining future peer work | Dependency ordering, independent producer/consumer samples, two current artifacts, failure diagnosis and limits of real concurrent at jobs |

These are not renamed/numeric variants. Each changes the required workflow and
work product. Fifteen fresh probe cases (ten due-run, five before-due) supplement
four authored tasks; they are not fifteen more labs. Every assessed captured
job has two distinct due-run inputs and a before-due case. The capstone's consumer
gets different supplied stage contents in its own fresh cases; the actual current
run additionally verifies the full ordered producer/consumer behavior.

## Prerequisites and teaching

The unit recommends actual existing units `linux-file-foundations` and
`shell-foundations`. The first lab's scenario prerequisites are actual existing
IDs `lpic1-file-navigation` and `shell-handover`; the subsequent three form a
linear suggested progression using the new IDs. Nothing locks experienced
learners out. No account/password or filesystem-permission knowledge is assumed
from choosing a supplied simulated queue owner.

Each introduction is skippable/revisitable and teaches three short concepts before
the task. Its worked example uses different paths, owners, variables and input
values, with separate typed initial files and queue. Annotated native actions
select the simulated account/variables, edit and save files, invoke supported
queue commands, and explicitly advance virtual time where needed. Fill must only
insert an action for review; it must not execute or overwrite the assignment.
The authored Try change edits an unsaved practice comment, demonstrating the
difference between a draft and captured work. Repair restores the saved practice
text without replacing assignment work. The entire assignment remains visible;
Challenge receives the full task with optional guidance, not unsolicited solved
source. Progressive hints and worked solutions belong to optional help.

The provided reference actions are an internal JSON authoring codec, not Linux
commands to paste. `Actor` and `Environment` are native selectors/editors, not
`sudo`, `su`, `export` or privileged host operations. The only learner queue
commands used are the documented `at -f`, `at -t`, `atq`/`at -l`, `at -c` and
`atrm`/`at -r` subset. Equivalent file reads, redirection and either supported
flag order/time representation are exercised by two alternatives per task.

## Behavior, feedback and history

The review fixture records literal, independently reasoned row vectors before
execution: initial, plausible incomplete work, a constant first-sample script,
and permitted actions that damage a completed result. The constant passes the
visible first report but fails a second fresh input. The queue task's damage
removes the required pending replacement. For completed-window tasks, saving a
new live input clears the old proof; earlier visible output alone is insufficient.
Editing a script after submission does **not** rewrite captured source: that
correct distinction must remain visible in feedback and must not be treated as
a defect. Current and fresh checks compare all virtual file maps, captured owner,
time, variables, status, stdout/stderr and due/early behavior, not exact source.

Nine content groups cover combined bounded-pack loading (original171 and176
predecessors exact), taught references and two alternatives, full wrong/partial vectors,
actual damage/retry, separate runnable teaching, real four-attempt save/decode with
nonempty writer receipts and selected reset, permission/unsupported/time refusal,
and transactional Stop. The historical producers, progress receipts and original catalog definitions
remain unchanged. Catalog180 recipes pin the exact176-lab predecessor, raw/typed
unit inverse and canonical four-task append. Current behavior loops retain the
whole current catalog rather than hiding new kinds from existing assertions.


## Official scope and known limitations

Checked2026-10-29: [LPI released LPIC-1 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
still identify v5.0 (101-500/102-500);107.2 includes managing cron/at jobs and the
terms at, atq and atrm, with access policy and systemd timers separately required.
The unit strengthens107.2/K1 and adds narrow Partial practice for
107.2.T9/T10/T11 (at, atq, atrm). It does
not complete107.2 or the certification. `/etc/at.allow`, `/etc/at.deny`, systemd
services/timers, spool administration, umask, mail and real filesystem permissions
remain outside this capability. LPIC-2 206.1 is source installation, not scheduling.

[Debian's at3.2.5-2.2 manual](https://manpages.debian.org/testing/at/at.1.en.html)
was rechecked2026-10-29: it documents queue owner visibility, removal, script-file
submission, captured environment/cwd, supported time concepts and past absolute
ASAP behavior. The earlier core contract records POSIX at pages returning403;
that unavailable source is not treated as inspected or bypassed. No real at/atd
reference process is run. This text and curriculum are original; third-party
manual prose is not copied into the lesson.

Virtual UTC advances only through Run; it does not run jobs while the app is
closed. The flat `/work/NAME` VFS,32-simple-command portable script subset,
16-job bound and serial execution are deliberate. Stop discards an unfinished
simulated interval, not a Linux rollback guarantee. Real at commonly invokes
/bin/sh and can overlap jobs; the capstone explicitly warns that time separation
alone is not reliable production dependency management. No host shell, account,
clock, service, files, network or scheduler is accessed. Supported simulated owner
inspection/removal is not full OS security or a complete POSIX implementation.

## Verified behavior and remaining acceptance

The serialized runner passes21 core,36 workspace,9 authored-content and6
registration groups. References and two alternatives per task pass; declared
wrong/partial/hardcoded vectors fail, edits invalidate success, examples never
award assignment credit, and selected reset/save/decode retain other work.
Thirty-two UI groups include the full906-step native-event controller. Actual
Fedora runs finish all four tasks,53 captures and20 measured settings (five text
scales, two widths and light/dark), including owner/capture inspection and real
Run/Stop. Reset and actual reference reexecution restore proof; no state or
mastery is injected. The presentation pass leads with the actual failed indexed
result, preserves per-case truth and moves extended detail into optional Help.

Independent review covers37 before and16 polished original PNGs; Root viewed
three originals from each. These are scoped rendered checks, not a novice study,
screen-reader/physical-device result or all-theme/other-OS acceptance. Main's
unified inclusive gate and a new installed candidate/cold process verification
remain separately tracked in CHECKPOINT.md. Warm native navigation is not cold
reopening. No public release is authorized.
