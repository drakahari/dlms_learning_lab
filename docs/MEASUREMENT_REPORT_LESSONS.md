# Measurement reports you can explain

Development unit: `shell-measurement-reports`, after log correlation. Three new
assignments use the unchanged bounded evaluator19. They are registered in the
399-lab/153-unit development catalog; development and isolated399R2 native acceptance pass. Original installed app and
progress remain untouched. See CHECKPOINT.md for the exact runnable build.

| Lab | Practical outcome | Prior skills |
| --- | --- | --- |
| `shell-counter-rates` | Validate paired sources and report interval byte rates, including sub-second intervals and counter resets | quoted fields, functions, integer arithmetic, status and file input |
| `shell-cpu-share` | Classify all eight interval CPU categories with a shared denominator and truthful rounding | rates, deltas and integer percentages |
| `shell-measurement-recovery` | Repair a streamed reporting project: validation, changing baselines, EOF handling, warning status and protected files | both reports, ERE validation, full-line input, status, indexed lists |

Each Introduction explains the new reasoning and supplies a different worked
example. Try example edits a separate source and simulated file set. Actual
results and the expandable **Expected preview · original example** are distinct;
comparison there is manual, not an automatic assignment grade. Deliberate practice
changes expose wrong units, category classification and warning status. The full
Assignment retains twelve deterministic cases and an unsolved starter. The
recovery starter contains three intentional faults. Canonical and meaningfully
different supported implementations pass; the grade never checks exact source.

## Contracts and limitations

Inputs are documented narrow CSV records, not arbitrary quoted CSV or raw Linux
proc files. Rates use elapsed milliseconds, multiply before division, and round
nonnegative results down. CPU work includes user/nice/system/irq/softirq; idle,
iowait and steal remain separate. This is aggregate normalized share, not process
CPU usage or proof of bottleneck cause. Decreasing iowait is rejected as an
unsupported interval rather than asserted to prove reboot. Streaming malformed
records break continuity; a later well-formed row can establish the next baseline.

The separate public `resource_measurements` pure model validates bounded synthetic
CPU/memory/disk/network observations with exact ratios. It has no host-reading API,
new learner command, State variant or progress format. Memory availability is not
a pressure diagnosis; sector conversions use512-byte accounting sectors. Fixed
inventory/boot identity and monotonic interval checks fail honestly. Guest ticks
are not added twice. No real monitoring utility, performance scheduler, arbitrary
system workload or Linux200.1/200.2 practical coverage is claimed by these scripts.

## Sources checked2026-11-11

- [Kernel proc accounting](https://www.kernel.org/doc/html/latest/filesystems/proc.html): CPU fields, guest inclusion, unreliable/decreasing iowait and memory accounting.
- [Kernel I/O statistics](https://www.kernel.org/doc/html/latest/admin-guide/iostats.html): cumulative counters and accounting units.
- [LPI201 objectives](https://www.lpi.org/our-certifications/exam-201-objectives/): LPIC-2v4.5/201-450 context; these Bash projects do not implement its monitoring commands.
- GNU online Bash manual arithmetic/loop requests timed out in this session. The
  installed original Bash Info was inspected instead (SHA256
  `c2ab10cda098ad2e6a43ced7a02ff1545fd9d85f3dc05183bb1d72cd3fcf60b2`).
  Fixed recipes were actually compared with GNU Bash5.3.9 on disposable development
  fixtures. The app never uses that Bash binary for learner execution.

## Evidence and authoring

`tests/resource_measurements.rs`:9 boundary/math/negative checks.
`tests/shell_measurement_authoring.rs`:6 checks, including independently derived
expected arithmetic, alternate recipes, hardcoding and plausible incorrect repairs.
`tests/shell_measurement_content.rs`:6 registration/progress/teaching checks.
The fixed `tools/measurement_scripts_reference.py` comparator ran75 canonical,
alternate and example observations against actual Bash; five guard tests check
its restricted development-only boundary. Private receipt:
`.build-artifacts/reference-runs/measurement-scripts-_90wz71m/report.json`.
It accepts no learner script or arbitrary fixture/path selector. Existing sealed
historical captures remain immutable. The160-action development-native run passed; actual images of every new lesson
screen and representative100/150dark/200Light editors were inspected under
`.build-artifacts/native-runs/run-rnhg2g2k/screenshots/workspaces`. The CPU experiment
observation was corrected after this run to match its actual45%work/0%wait output.
Four capture/reflow/actual-glyph checks pass. Isolated packaged evidence remains
pending and will be recorded only after actual runs. No independent novice, assistive
technology or other-OS acceptance follows from these technical checks.

Current catalog399/153 and exact retention of396preceding scenarios/152units pass.
Teaching inventory394updated/five already meets, no pending entries.1,079Python
checks, all-target Clippy and coverage validation pass. Focused existing evidence
is reused; no inclusive Rust runtime rerun or novice study is claimed.

Initial399R1 passed private396R2 upgrade, installed160new/85retained native actions,
canonical-installed NFS two-process reopening and final selected-input identity.
Its technical receipt is retained; a rendered worked caption still needed clearer
manual-comparison wording. A mistaken unused why property was removed and the real
explanation field corrected, with a new closed-key regression. Fresh399R2 is required
before admitting this milestone. The first NFS call safely refused a launcher
symlink; no check was weakened. Canonical installed ELF reopening then passed.

399R2 is now verified: private399R1 upgrade/uninstall/reinstall,160 installed
measurement actions,85 retained JSON actions, canonical installed NFS two-process
reopening and final selected-input identity pass. Actual corrected caption, CPU
experiment, mixed overall/case result,150dark and200Light editors were inspected.
Private receipt:next-measurement-scripts/candidate-r2/r2-verified.json. Minimum
15.16GiB available, fullPSIpeak0.00%, no OOM increase. Earlier R1 observations remain
historical; do not replay their identity after this documentation advance.
