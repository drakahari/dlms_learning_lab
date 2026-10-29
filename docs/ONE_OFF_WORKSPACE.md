# One-off scheduling: typed authoring and persistence boundary

The adapter now supports four bundled LPIC-1 tasks in `linux-one-off-jobs`.
Only narrow107.2 practice is claimed. Access-policy files, systemd timers, mail,
spool files, real daemon timing and parallel job execution remain outside the
model. See [ONE_OFF_LESSONS.md](ONE_OFF_LESSONS.md) for exact acceptance and
[ONE_OFF_SUBSET.md](ONE_OFF_SUBSET.md) for commands and real-system differences.

All work stays in the existing in-process simulator. The adapter delegates queue
commands and script parsing/evaluation to the unchanged one-off core. It never
passes source, commands, filenames, variables or times to a host shell, filesystem,
service, network or clock.

## Current outcomes and captured behavior

`State::OneOff` holds an `Assignment`, containing the current core simulation and
an optional bounded execution snapshot. `Goal::OneOff` holds its declarative
`Definition`. No persisted success Boolean is introduced.

Requirements can describe an exact queue snapshot, captured ordinary variables,
current virtual output files, a virtual clock, an exact execution window, or a
captured job dispatched through its due time. Queue selectors bind owner, script
filename and either absolute submission/due times or a delay after submission.
Matching is one-to-one: overlapping selectors cannot count one job twice and
hide an unwanted extra job. IDs themselves are not the intended answer.

Each assessed job needs at least two distinct due-run cases. A case selects the
actual captured script/environment, replaces only a private sample file map and
uses either an absolute window or offsets after that job's submission. Output,
error output, exit status, run identity/time and complete files are compared.
Before-due cases can require no execution. The original attempt never receives
these case outputs or file changes. Protected unchanged peer jobs are checked
separately from learner-authored behavior.

Submission-relative tasks should use relative selectors/windows and dispatch-time
expectations. A repaired script can be resubmitted at the current virtual time
and checked again without erasing drafts or rewinding the clock. Absolute
constraints remain useful for deadline assignments; a missed fixed deadline may
legitimately require resetting that selected exercise. Past-due jobs keep the
core's disclosed ASAP-at-next-simulated-Run behavior; timestamps are not invented
to make a late answer pass.

## Execution evidence is not an event-log certificate

Core records are bounded display history. They may be evicted, and are never
sufficient authority for completion. A completed window stores one before-state
and an end time. Check replays the same bounded core and compares the entire
result with the current simulation. Fresh captured-job cases then grade behavior
against other sample inputs. A copied result flag or matching last log line
cannot substitute for either check.

Consecutive **clock-only** continuations can retain the same before-state while
the combined span remains at most seven days. They can verify the original
captured owners, source, variables, dispatch times and final whole-state result
across that span. A harmless extra clock advance therefore need not erase a
valid relative-task result. Exact-window or exact-clock requirements still mean
what they say. Extending beyond the bounded span refuses the operation if it
would discard a required completed-job capture; existing work stays intact.

An actual saved-file or queue change starts a new verification window, with a
plain notice. Earlier output files and bounded history remain, but the old
capture is not silently treated as proof of the edited configuration. Merely
selecting a file/account, changing a variable draft, inserting a command or
saving identical text does not rewrite an already captured job. Unsaved file
edits remain separate and prevent completion for an inconsistent editor.

This is not a general event journal. The adapter cannot certify arbitrary earlier
windows across intervening edits, overwritten historical file versions, or an
unbounded history of job ownership/output. Author assignments within one proven
span, or assess the repaired current job and explicitly preserved current files.
Do not use evictable records to fill a missing historical requirement. For a
"prevent this job from running" task, bind the appropriate initial clock/window
and complete queue outcomes; absence from the current queue alone is insufficient.

## Drafts, practice and Stop

The persistent session owns selected file, per-file draft text, selected simulated
account, ordinary variable values, command draft, window draft and the last
actual queue-command response. `CommandDraft` and `WindowDraft` are inert. Run
parses the explicit UTC window and owns a bounded continuation. No example or
Insert action automatically executes an assignment command.

Teaching has skippable introduction data, separate supplied users/files, annotated
worked actions, an experiment and a repair with explicit outcome rows. Its
simulation/session is independent of the assignment. Preparing, changing views,
running practice and reopening cannot install example results as assignment
credit. A genuine saved assignment result remains valid only while its current
requirements, replay and saved drafts agree.

Stop drops an unfinished **simulated transaction**, including staged queue,
clock and file changes. This differs from real `at`, which does not roll back
already completed external processes. Check cancellation likewise never publishes
probe files. Async publication compares the complete source and original attempt
before moving only the selected assignment/example result. A late result cannot
overwrite an intervening edit, reset or context change.

## Bounds and authoring interfaces

Existing core limits are unchanged:16 pending jobs,64 retained records,32 flat
ASCII `/work/` files with96KiB aggregate text,8KiB captured scripts,16 captured
ordinary variables, bounded VM instructions/output and at most seven days per
window. The adapter admits at most16 requirements and16 cases. A full Scenario,
saved Attempt and separate teaching context stay within2MiB. Early decoders
bound names, values, sequences and map entries before excess/invalid payloads.

Native writers must use `one_off_workspace::budget`. Prepare/reset count the
complete fresh wire form through borrowed projections before copying drafts or
practice. Ordinary text/selection/variable edits copy only admitted fields.
Run and queue/file operations reserve a bounded possible result before candidate
allocation and publish atomically. Near-full attempts may be refused
conservatively even when one particular result would be small; no cap is raised.
The error retains the previous draft and explains that nothing ran.

`one-off {JSON}` is an internal declarative reachability codec, with a64KiB raw
bound. It is not a Linux command and must not appear in learner walkthroughs.
Use real supported simulated queue controls, editors and explicit Run/Stop in
the later native workspace. Raw JSON is never executable code.

No old scenario, draft, theme, progress producer or historical evidence is
rewritten by adding these optional fields. Old fields serialize as before when
the new metadata is absent. New completion always comes from current behavior;
it does not imply mastery, exam readiness or actions performed on Linux itself.

## Decoding boundaries

The six new tagged enums use explicit map visitors rather than Serde's internally
buffered tagged-enum decoder. Unknown and duplicate keys are refused before the
value is read. Environment keys and excess map/list entries are likewise refused
before their payloads. The `kind` tag can occur anywhere: before it is known,
recognized fields use the largest bounded type admitted by that enum's variants;
after it is known, incompatible fields are refused immediately and variant text
limits are applied. A tag arriving later still enforces its narrower final limit.
Canonical serialization is unchanged. Poisoned-tail tests cover standalone
Action, Requirement, timing/window/event-time and internal reference codecs,
and the same objects nested inside a Definition or teaching Lesson.

Definition::from_json and Lesson::from_json admit at most2MiB of raw input; the
internal action codecs admit64KiB. Existing outer engine State/Goal enum wire
paths still use Serde's inherited Content buffering, under pack/progress raw-size
admission. This adapter does not claim early member ownership through those
legacy outer buffers. No old engine/progress wire decoder is rewritten.

Source-aware APIs also enforce the valid walkthrough position and available hint
count. Own metadata enforces the same120-entry transcript count as normal save
and load. Direct budget operations therefore cannot save a new state that its
own ordinary loader rejects on those boundaries.
