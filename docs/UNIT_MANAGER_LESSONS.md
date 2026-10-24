# Service definitions, dependencies and target handover

Checked2026-10-23. Four authored LPIC-2 assignments are bundled in unit
`linux-unit-manager-foundations`, following `linux-service-recovery`. The normal
catalog is106 assignments/25units (31CCNA/21LPIC-1/17LPIC-2/37Bash). These are
bounded systemd-style exercises, not real boot or complete exam preparation.

| Assignment | Meaningful task | Accepted alternatives |
| --- | --- | --- |
| lpic2-unit-loaded-recovery | Diagnose a wrong supplied ExecStart, distinguish saved/loaded/running definitions and recover the application while preserving the observer and policy | Different description/comments; stop before or after manager reload; start the application or its target |
| lpic2-unit-preparation-order | Repair preparation data, require preparation, guarantee its ordering and verify current plus next-session work | Consumer After or producer Before; starting the app or target |
| lpic2-unit-optional-wants | Keep a deliberately broken optional helper requested and failed while allowing the core to work | Keep or remove optional After; target or direct application start |
| lpic2-unit-target-handover | Prepare new data, retire the old service and link, enable/select the replacement target and preserve the observer and unrelated files/links | Different valid ordering relation and stop/enable/default sequence |

## Teaching and tools

The prerequisite teaches PORT configuration, not unit-file grammar. The first
introduction therefore explains [Unit]/[Service]/[Install], Type=simple,
ExecStart and fixed virtual program identities before testing them. Later
introductions teach one-shot completion/RemainAfterExit, requesting versus
ordering, Requires versus Wants and target/install/default policy. Experienced
learners can skip the introduction and reopen it later.

Each worked example uses different names, paths, ports and prepared values from
its assignment. A visible selector shows one annotated example action and real
command at a time. Insert fills input only; Save and Run are explicit. Fill
worked editor drafts changes only example text. Each example has a separate
experiment and repair. Challenge keeps the full practical task, editor, terminal
and grader, with guidance available only when requested.

The native workspace provides one selected unit/configuration editor, a bounded
virtual terminal and an optional read-only saved/loaded/current-input inspector.
Supported real command spellings are documented in UNIT_MANAGER_TERMINAL.md;
internal typed authoring actions are never presented as Linux commands.

An earlier wrong attempt can leave successful preparation active with old data.
Another start correctly does not rerun an active RemainAfterExit job. Teaching
and the native walkthrough now explain stopping the application and preparation,
then starting the target to use corrected input and order. This recovery was
found by the actual native flow, reproduced in a model regression, and fixed in
guidance/test actions without weakening the simulator.

## Grading and limits

Goals derive current program outcomes and a separate next-session workload from
State. Saved/loaded definitions, current input provenance, required preparation,
real transaction ordering and explicit preserved context are checked. No journal
entry, successful start message, command string or walkthrough position awards
success. Pending editor changes invalidate current completion. Queries preserve
it. Raw malformed drafts remain editable and resumable.

The optional helper must actually fail: removing its request or repairing its
protected input is not success. The handover requires the old service and old
target inactive in the next-session copy, not merely absent from a successful
start list. A healthy observer is required at the end; uninterrupted historical
uptime is not claimed. Protected files/links are explicit task constraints.

Current-input provenance compares actual saved/loaded/consumed text, including
comments, byte for byte. Comment-only edits therefore need a reload/new execution
to satisfy this conservative model. Alternate source text is accepted after it
is genuinely loaded and used; no one expected source blob is required. This
snapshot policy is narrower than reasoning about effective settings alone.

There is no host systemd, PID, process, listener, filesystem, real boot,
concurrency/readiness protocol, SysV/LSB administration or complete unit grammar.
The controlled fictional programs and deterministic scheduling limits are in
UNIT_MANAGER_SUBSET.md. Check work tests the next session in a separate copy;
it cannot repair the current state.

## Actual verification

Fourteen content regressions cover reference/two alternatives per assignment,
wrong and partial states, each individual current predicate, protected present
and absent links/default/files, wrong ports/stale inputs, failed helper shortcuts,
manual current-only ordering, invalidation and genuine retry after early start.
Original102 IDs, real37 revision7 runs/examples and33 exact revision6 histories
survive save/export/review/restore/reset at this milestone. No historical outcome
is regenerated and relabeled as an older evaluator.

The integrated source passes771 library/integration and120 UI checks
(891distinct combined), all-target Clippy and formatting. Actual normal all-four
Fedora input flow passes72 captures in `m70-normal-service-native-r2.log`, capture
root `.build-artifacts/native-runs/run-pejyy86y`. It exercises real picker/editor/
Save/terminal/Check controls, partial/wrong/reference/alternate/retry, fill-only
practice and experiment, exact separate draft/Session/editor-kind resume, Undo,
reset/cancel, query retention and independent Challenge. Every assignment types,
saves and retries at150% light and200% dark, with actual render review. The earlier
failed native r1 remains preserved; its active-preparation recovery cause is above.

These synthetic input flows are not novice comprehension, physical-device,
screen-reader, Windows/macOS or full-product acceptance. The latest installed
candidate is separately identified in INSTALLATION.md; development acceptance
does not silently update that artifact.

## Official sources and coverage

Rechecked2026-10-23: LPIC-2 objectives4.5, exams201-450/202-450. Only already
Partial202.1, its systemd knowledge bulletK1 and named systemctl termT4 gain
stronger evidence. SysV/LSB and other unimplemented requirements stay Open;
coverage counts remain128Partial/1Supporting/1111Open overlapping references.

- [Official LPIC-2 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [systemd262 unit dependencies and install links](https://www.freedesktop.org/software/systemd/man/262/systemd.unit.html)
- [systemd262 service start/one-shot behavior](https://www.freedesktop.org/software/systemd/man/262/systemd.service.html)
- [systemctl262](https://www.freedesktop.org/software/systemd/man/262/systemctl.html)
- [journalctl262](https://www.freedesktop.org/software/systemd/man/262/journalctl.html)

All five official pages were actually opened during content review. Original
scenario/teaching data and algorithms are project-authored MIT material; no
upstream implementation or manual passage is imported. No vendor endorsement,
certification guarantee or equipment equivalence is claimed.
