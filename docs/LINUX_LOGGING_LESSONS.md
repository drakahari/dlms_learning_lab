# Linux logging: selectors and evidence — 2026-11-02

Four full editor assignments follow Linux file/log foundations. Source review
catalog235/63:69CCNA,60LPIC-1,40LPIC-2,66Bash. Existing231 labs/62units remain
exact; the new Linux parser never replaces the Cisco syslog model. This is narrow
LPIC-1 v5.0/108.2 practice, not full logging administration or exam preparation.

| Stable ID | Distinct practical assignment |
| --- | --- |
| lpic1-syslog-threshold | Select warning and more urgent messages, excluding routine noise |
| lpic1-syslog-exact | Separate one facility’s exact information level from urgent general alerts |
| lpic1-syslog-privacy | Exclude private authentication from the general rule while retaining all private levels |
| lpic1-syslog-handover | Combine private, general-threshold and two-facility exact streams without unwanted or duplicate copies |

Introduction teaches source/facility, urgency order and the new selector behavior
before a different worked configuration. Practice has its own faulty starter,
worked fill, meaningful edit/test/retry and extra positive/negative sample events.
Insert worked configuration fills only; Run routing test or Ctrl+Enter executes
the simulation. Tab indents, Esc releases focus. The editor has an accessible
name; actual assistive-device acceptance remains untested.

Assignment retains its whole policy and editor. Samples illustrate actual routing;
grading checks all192 facility/severity combinations, including excluded events,
wrong destinations and copy counts. Reordered rules, comments, case and supported
exact-level partitions can pass. A later private rule cannot undo a general-log
copy. Changed drafts retain labeled previous replay but invalidate current success.
Each failure names a facility, required destinations and actual destinations; the
common urgency/copy explanation appears once. Challenge hides worked help by default.

## Models, isolation and authoring

`linux_logging` parses the disclosed classic fragment into typed24×8 rule masks.
`linux_logging_workspace` holds distinct initial/example cases, independent
editor/tested-text/diagnostic state and source-aware restore. Paths are labels,
not openable host files. No subprocess, filesystem, socket or clock API exists in
the runtime model. No shell evaluator change or dependency was introduced.

[Subset](LINUX_LOGGING_SUBSET.md) lists all grammar and bounds. This is not a full
rsyslog daemon configuration. Later positive semicolon selectors, ! filters,
includes, modules, property filters, remote actions and templates fail explicitly.
Journal, logrotate, network reception/forwarding, queues, timestamps, encoding,
permissions and daemon lifecycle remain open. A routing-only private stream does
not establish real file access protection.

Authors supply initial/example task, starter, bounded sample events and expected
policies, plus a separate worked fragment. The initial task must equal the
Scenario objective. Expected policies define complete destination/copy outcomes;
the reference must be reachable, and wrong/partial policies must fail. Keep
prerequisites intact: tests load the real starter pack alongside this unit instead
of silently removing the required original log lesson. New identifiers are stable.
`SYSLOG CONFIG` is an internal reference prefix, never taught as a Linux command;
the optional worked view strips it. Save/restore uses the actual authored catalog.

## Source and focused checks

Primary LPI objectives, upstream rsyslog filters/actions/imfile documentation and
RFC5424 were read2026-11-02; URLs are in scenarios and the subset guide.

- 12 pure-core groups pass: all severity boundaries/facilities, exact/all/grouped
  selection, same-rule exclusions, duplicates, aliases/unsupported forms, malformed
  controls, path/input bounds and alternate-rule equivalence.
- 9 lesson/content/catalog/progress groups pass, including wrong and partial
  policies, stale invalidation, separate examples, exact231-prefix preservation,
  selected reset, resume, foreign metadata and oversized optional strings.
- 8 retained Bash project groups pass; evaluator15 and66 scripts are unchanged.
- 6 widget/paint/layout/keyboard/capture groups pass. The editor/Run are checked
  with populated actual output at760/1600 widths and100/115/125/150/200% in light
  and dark themes. Native package checks remain separate.

Logs under ignored `.build-artifacts/continuation/resume-20261102/`:
`linux-logging-model-final.log`, `linux-logging-ui-final.log`. A final Clippy
warning about an unnecessary unwrap was corrected with an explicit Some branch;
current Clippy/Python/package evidence follows separately. Failed attempts remain.

## Actual controlled rsyslog comparison

`tools/dev.py tool linux_logging_reference.py` accepts only five fixed developer
policies,36 synthetic messages each and a pinned model identity. Each owned
foreground child loads only imfile against fresh project-disk files, with its own
configuration, state, PID and output paths. Internal messages remain in that
process and are discarded; no host log input, socket/listener, system config,
service registration or running service is changed. Environment is stripped,
resources/time/output/files are bounded and the shared runner lock is inherited.
This tool never accepts learner configuration or participates in learner execution.

Both actual runs passed180 selected messages: threshold, exact, private exclusion,
independent duplicate copies and facility grouping. Original observations remain
in `reference-runs/linux-logging-6dbz54go/report.json`; current pinned source after
shortening repetitive diagnostic prose passes again in
`reference-runs/linux-logging-rkarbdqe/report.json`. Reference output compares
multisets, not asynchronous queue order. Pure-model grading covers192 combinations;
these180 selected real messages are not a full-protocol conformance claim.

Separate235 Fedora candidate/native screenshots, cold process reopening/new
installation, full inclusive suite, Windows/macOS runtime, physical-device,
screen-reader and independent novice acceptance are not established here.
The latest already verified separate231 package remains available in CHECKPOINT.md.

## Packaged Fedora milestone — 2026-11-02

Separate candidate `dist/teaching-review-20261102T165655Z/` validates235/63.
BinarySHA256 b47371ab351d30add3df90565fa03e4f1b9d7ba3f18ed64162bd8210a33ac64c;
archiveSHA256 d14a975f999a2da532d3e95a687bd3b9f07267338885335676050cbee1417b42.
Fixed native route passes all4 example/wrong/fill-only/correct/damage/retry flows
and all4 assignment starter/solution/stale/damage/retry/save-read/selected-reset
flows.40 PNGs are private under run-0oan7duz. Actual Introduction, privacy
practice, handover starter feedback and narrow light200 controls were inspected.
Dark output contains actual duplicate destinations; light200 unchecked controls
remain reachable by scrolling. Populated layout across scales is additionally
covered by widget tests, not assumed from those unchecked light screenshots.

Final focused gates:29 model/content/retained-Bash groups,6 UI/capture groups,
92 Python packaging/teaching/current-source/frozen-guard checks, production and
new-test Clippy pass. Current fixed rsyslog evidence is
`.build-artifacts/reference-runs/linux-logging-rkarbdqe/report.json`:180 selected
messages across5 policies, all exact. Prior reports/failures remain retained.
Candidate build minimum available17.21GiB, PSI avg10 maxima0, OOM counter delta0.
No full-inclusive suite, new install/upgrade/cold reopening, Windows/macOS runtime,
screen-reader, physical-device or independent novice acceptance is claimed.
Installed app and original progress were untouched; no push/merge/publication.
