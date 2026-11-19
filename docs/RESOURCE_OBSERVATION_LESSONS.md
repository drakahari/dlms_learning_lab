# Measure before diagnosing resource pressure

Development unit `linux-resource-observations`: three LPIC-2 investigations,
registered after content/progress checks. Catalog402labs/154units; the isolated402R1 package is now verified. All three entries now have individual source/method and actual Fedora screen
review. Registration and inherited rendering alone are not lesson acceptance.

| Stable lab ID | Practical work | New teaching |
| --- | --- | --- |
| `lpic2-resource-cpu-intervals` | Replay CPU windows, compare every eligible interval, attach peak/trough and runnable-process evidence | since-boot versus interval reporting; executing versus idle/wait/steal; instantaneous process counts |
| `lpic2-resource-swap-activity` | Compare paging rates with allocated swap and a final memory snapshot | historical allocation versus current activity; free versus available memory; correct KiB units |
| `lpic2-resource-restart-handover` | Recover a coherent post-restart report with CPU/block-output peaks and final memory evidence | source identity, incompatible boot boundaries, throughput without unsupported causal inference |

Prerequisite unit: `linux-processes`. Individual assignments follow the existing
process handover and each other. Experts can skip introductions; prerequisites are
advisory. Both Learn and Challenge retain the complete task and terminal/report
workspace. Introduction explains the new concepts and a different worked
recording. Try example gives specific commands, row choices, calculations and
intentional wrong-value/evidence experiments. Assignment fields begin blank,
without answers or pre-attached evidence. Worked assignment reasoning is available
only through an explicit disclosure in optional help.

## Simulator contract

This is a finite recording replay, not a live host or performance scheduler.
The Observation picker selects a supplied boot and uptime. Supported commands:

- `vmstat` or `vmstat -S K`: one since-boot activity report.
- `vmstat DELAY COUNT` or `vmstat -S K DELAY COUNT`: positive whole-second delay,
  at most eight rows, all requested observations must actually be present.
- `cat /proc/meminfo`: the selected **virtual** memory snapshot, limited to
  MemTotal, MemFree, MemAvailable, Buffers, Cached, SReclaimable, SwapTotal, SwapFree.

One command per Run. Other flags, paths, shell quoting/operators, continuous
sampling, pipelines and utilities are honestly unsupported here. Other application
shell/terminal workspaces retain their separately documented capabilities. Commands
never reach the host shell, filesystem, processes, network, services or real clock.
Replays complete in bounded in-process work; they do not sleep for the delay.

The first vmstat activity row is since boot; subsequent rows use counter deltas.
Process/memory columns describe the instant at each row’s end. The supplied page
size determines swap-page conversion. Block accounting is KiB, not disk sectors.
CPU guest ticks are included in user/nice counters and subtracted before the
separate guest display. Kernel/IRQ/softIRQ are combined as system work. Displayed
percentages round independently, so sums may differ slightly from100. This differs
from the preceding Bash report unit’s deliberately truncated integer percentages;
neither display is an exact unrounded utilization ratio.

Virtual MemAvailable is an independent supplied estimate, not total minus free.
Procps vmstat cache combines Cached and SReclaimable; virtual meminfo keeps them
separate. Its kB labels represent1024-byte KiB. Missing source intervals, mismatched
boot/inventory, decreasing counters or impossible guest accounting are rejected
before formatting a reading. These are simulator replay-validation diagnostics;
actual Linux tools may handle live changes/accounting anomalies differently. No
negative counter change is silently changed to zero. The paired counter model’s
broader fixed-inventory consistency guard is intentionally stricter than live
vmstat; it is not taught as the real program’s error behavior.

No real monitoring collection, per-process I/O attribution, scheduler repair,
per-device latency, network/client/firewall throughput, bottleneck proof or capacity
forecast is implemented by these lessons. The new capstone practices a genuine
boot boundary; it does not claim a counter-reset diagnosis. Defensive decreasing-
counter tests are input-safety/fidelity guards, not additional authored labs.
This is a partial measurement slice of LPIC-2 200.1;
200.2 and the remaining tool/diagnosis objectives stay open.

## Grading and persistence

The goal stores no canned command-to-success strings. Typed observations generate
real rows. Report claims retain a supported measurement request and actual output
row. Values are compared with derived metrics from that evidence. Peak/trough
requirements compare every compatible eligible interval; ties and equivalent
supported command forms pass. Blank, malformed, incorrect, wrong-boot,
wrong-interval, boot-average and unavailable-metric evidence receive distinct
feedback. A report requires an explicit Check; changing its value/evidence clears
that check. Further read-only investigation preserves a still-correct report.
Command outcome and report outcome are separately labeled.

History is bounded. Attached evidence survives history rollover and is recomputed
from its authored recording on resume, rather than trusting saved output text.
Separate example and assignment drafts, selected result row, commands and report
values persist. Definition validation proves separate fixtures and reachable
checked references. Imported sessions are versioned, bounded and bound to their
actual source task. The existing progress version stays1. Unsupported-version
writes cannot replace the original file. Existing399labs/153units remain exact.
No old learner answers or mastery are invented.

New authors supply bounded `Recording` observations, typed `Rule` metrics/time
selections and declarative reference `Action`s ending in Check. `resource-action`
is an internal simulator-only authoring reference, never a Linux command to teach.
Keep display instructions in the LPIC-2 presentation shard, grade definitions in
the pack, and optional depth outside the main workspace. Tests must prove the
method, supported alternatives, plausible mistakes and saved-state boundaries
before registration/native review. Do not count a new numeric recording as an
additional authored lab.

## Sources checked2026-11-11

- [Official LPIC-2 objectives4.5.0](https://www.lpi.org/our-certifications/exam-201-202-objectives/):200.1 measurement/correlation and200.2 forecasting scope remain distinct.
- [Procps vmstat manual](https://gitlab.com/procps-ng/procps/-/raw/master/man/vmstat.8): first/interval reporting, instantaneous memory/processes and units.
- [Procps vmstat implementation](https://gitlab.com/procps-ng/procps/-/raw/master/src/vmstat.c): guest accounting, field grouping and display rounding were inspected as behavioral reference; source is not bundled or copied into the app.
- [Procps memory accounting](https://gitlab.com/procps-ng/procps/-/raw/master/library/meminfo.c): combined cache and available-memory meanings.
- [Kernel proc documentation](https://www.kernel.org/doc/html/latest/filesystems/proc.html): counter/memory semantics and iowait limitations.

The upstream master/latest links are mutable; this is a dated inspection, not a
pinned upstream source bundle or actual procps runtime comparison. Golden math
checks use fixed synthetic observations. No real host monitoring reference was run.

## Evidence so far

Core11, workspace10 and authored/registration8 checks pass. They cover calculation
boundaries, input isolation, meaningful alternatives, report error feedback,
success invalidation, separate practice, saved evidence, favorites, selected reset,
protected peer attempts and exact preceding catalog/unit retention. Final UI/event/capture7 checks pass, including real popup scrolling to the final
observation and strict attachment-column alignment at100/150/200% text. The actual
Fedora development app completed291 native actions for all three lessons: separate
practice, unsupported-command feedback, independent reports, per-row mistakes,
keyboard Run, success invalidation, selected reset and saved readback. Screens
inspected from `.build-artifacts/native-runs/run-58k0kw4i/screenshots/workspaces`
include each Introduction/worked example, practice and blank Assignment, checked
reports, partial feedback, incompatible boot refusal and populated light/dark
controls. Some captures deliberately show a scrolled region; they do not establish
whole-output simultaneous visibility.

Actual native inspection first exposed two defects: a new sizing closure read the
theme under the font lock, and selector metrics inherited the whole-window clip.
Both are fixed with regression coverage, without relaxing scroll, deadline or
visibility guards. A strict column test also caught short unit labels failing to
reserve their width. Original failed logs/screens remain privately preserved.
No inclusive Rust runtime rerun is implied. Python1086, all-target Clippy,
retained pure-model9/script-content6, normal-catalog2 and coverage/inventory pass.
The402R1 package passed private399R2 upgrade/reinstall, actual installed291 new
actions,85 retained JSON actions, NFS two-process cold reopening and final selected-
input identity. Actual packaged fields, Result row label, evidence and local
feedback were inspected at100dark and200Light; prior unchanged content/screens
were reused. Exact launch, hashes and receipt are in CHECKPOINT.md. Other-OS runtime, physical-device,
screen-reader and independent novice acceptance remain pending.
