# Boot entries: images, early userspace and the system root

LPIC-1 → **Boot entries: from images to the system root**, after supplied device
and driver foundations and current/saved startup policy. Prerequisites are
advisory; existing progress is not reset.

| Stable lab ID | Independent work |
| --- | --- |
| `lpic1-boot-root-identity` | Diagnose a valid image chain pointing at a data archive; repair the system-root identifier. |
| `lpic1-boot-initramfs-match` | Repair an upgraded-kernel entry; distinguish a mismatched image from a matching image lacking a required storage driver. |
| `lpic1-boot-recovery-handoff` | Combine kernel selection, root identity, initial read mode and a recovery-target request into a complete handoff. |

Each Introduction recaps boot-entry keys, filesystem UUIDs, image paths, early
userspace, drivers and init before a separate worked example. Try example has a
faulty workshop editor, supplied image/root facts, specific edit/test actions and a
changed-setting experiment. Assignment preserves intentional errors and the full
task. Challenge offers the same work without unsolicited worked answers. Editing
clears tested status immediately. Practice never awards assignment credit.

## Actual model and limits

`boot_entry` is a pure, finite Type #1 Boot Loader Specification parser and
hypothetical early-root trace. It supports literal title/version/linux fields,
ordered repeated initrd/options lines, one filesystem-UUID root, ro/rw, quiet and
multi-user/rescue requests. Values are not evaluated as shell or GRUB code.
Unknown keys/options fail explicitly. Duplicate scalar fields/arguments, non-UUID
roots and quoted kernel arguments are outside the disclosed subset. Paths are
normalized keys in supplied virtual $BOOT, never host filesystem paths.

The trace computes image existence, kernel/initramfs release agreement, UUID
lookup, available storage/filesystem drivers, the actual selected root and init
availability. Correct configuration is distinct from a successful trace: an early
image failure does not label a correct root or target option wrong. A bootable but
unrequested system, mode, release or target still fails the assignment. Titles
and version descriptions do not change the loaded image. Missing init is modeled
from supplied facts; no executable is loaded or invoked.

Firmware, bootloader availability and a readable boot partition are preconditions.
This is not GRUB installation/configuration generation, firmware/UEFI/Secure Boot,
BLS discovery/menu sorting, raw image parsing, initramfs generation, a complete
kernel command line, actual systemd startup, a real reboot or hardware conformance.
Only finite in-memory data is inspected:4KiB/48lines,16arguments,4ordered early
images,8kernels/8images/4roots and bounded driver lists. There is no long-running
interpreter, so Test entry completes as a bounded synchronous check.

## Persistence and preservation

Original373scenario objects/order and138unit objects remain exact. The new three
IDs use a separate appended pack and one new unit: source376/139, comprising
89CCNA,120LPIC-1,90LPIC-2and77Bash labs. The two adjacent module packs consolidate
into `kernel-module-curriculum-v2.json` with all four whole scenario objects/order
unchanged; original files/getters remain. The registry stays at128files,32MBtotal,
2MBper file and1000scenarios. Reporting recognizes the explicit old origins only
when their complete pinned source and every bundled scenario match. Unrelated or
changed duplicate sources still fail.

New saved work stores bounded editor text, current view and a source-matching test
marker. All results are recomputed from the authored task; saved work cannot
supply inventories/goals or fabricate a successful result. Invalid syntax can be
saved as unfinished work. Save/load, export/validated restore, selected reset,
practice isolation, alternate supported repairs and stale-success invalidation
have focused tests. Existing profile schema/IDs/theme behavior remain unchanged.

## Primary sources

Checked2026-11-10; LPIC-1 objectives remain version5.0 (101-500/102-500).
Original teaching prose only; no external implementation/manual text is bundled.

- [LPI official objectives,101.2](https://www.lpi.org/our-certifications/exam-101-102-objectives/).
- [UAPI Boot Loader Specification1.0, Type #1](https://uapi-group.org/specifications/specs/boot_loader_specification/).
- [Linux kernel: init-handoff failures](https://docs.kernel.org/admin-guide/init.html).
- [Linux kernel command-line parameters](https://docs.kernel.org/admin-guide/kernel-parameters.html).

Only narrow101.2partial practice is claimed. Full boot sequencing, log inspection,
SysV/Upstart, GRUB/MBR installation and broader102.2administration remain open.
This does not establish complete exam preparation or real-equipment experience.

## Current verification checkpoint

Core5 and authored6 tests pass, including actual private backup/restore, wrong and
partial repairs, malformed/unsupported input, an early-failure feedback regression
and complete373-scenario/138-unit preservation. Current-catalog and retained EUI64
checks also pass in the combined18-test invocation. Final5widget/capture tests pass
at five supported text scales, two widths and light/dark themes, with real keyboard
editing/Tab, retry, independent practice, shared Back/Mode and confirmed Reset.
A regression requires a normal-width split, readable complete boot lines, and
feedback immediately below Test entry. Long arguments use supported repeated
options lines, taught before practice, rather than reduced font size.

Actual final development-native R5 `run-uryp9rt6` completes54actions/15captures,
exit0: three editors, independent/practice testing, specific wrong/partial results,
stale-success invalidation, save/read, native mode changes, confirmed selected
reset and Back to library. Root opened4final originals including the header,
independent task, correct configured settings with an earlier blocked stage and
light200results. Earlier R1/R2/R3/R4 are retained; they are not additional labs.
R1exposed stacked wide layout and distant feedback. R3corrected dependent-result
wording, R4made complete option lines readable, and R5restored the shared header.
All final state checks remain substantive; none were waived to accept a screen.

All1007Python, strict current-catalog coverage,376-lab teaching inventory, final
all-target Clippy,31retained registration checks, two normal-catalog widgets and
actual unflagged376/139content validation pass. The exact376candidate is next. Latest
admitted Fedora package remains372R1; no376package is yet accepted.

This is technical verification, not an independent novice study, screen-reader,
physical-device, installed upgrade/uninstall or other-operating-system acceptance.
All new evidence stays private/ignored. No original profile, installed app, system,
dependency, Git push/merge or release changes.

## Narrow-window package follow-up —2026-11-10

The first376release artifact is retained, not admitted: boot capture targeted nine
editor rows while1.7logical points of empty space were below the narrow200window;
EUI64capture33independently failed its bounded settlement guard. Packaged content
and private two-process NFS reopening passed. Selected final identity agrees with
initial; candidate-r1/r1-failed-retained.json records all outcomes before unfreeze.

The boot editor now derives its two-column breakpoint from the native font width
of supplied starter/reference lines as well as the existing body-height minimum.
A new regression caught a real150%split hiding a UUID; it now stacks at that width.
Screenshot targeting uses the actual laid-out text rectangle, so every authored
line must be visible, without treating empty editor rows as a click target. Real
clicks still require the unchanged full editor/control visibility test. The six
boot widget/capture checks pass at normal/150/200scales and retained reflow themes.
The initial new measurement accidentally re-entered a context lock; it was fixed
by reading theme color before the font-lock closure. The owned interrupted test
and exact marker recovery remain private; no other process was signaled.

EUI64timeout diagnostics now include phase, frames, elapsed time, pending screenshot
and actual metrics. The instrumented development flow completes37actions/11PNGs;
this does not establish a fix or acceptance for the optimized artifact. A fresh
isolated release identity and native gates remain required. No teaching/grading,
lab IDs, saved-progress format or existing success policy changes in these fixes.

### OptimizedR2 diagnosis and paced capture fix

The second exact release reproduces EUI64failure with decisive timing evidence:
action33,phase1,1200redraws in217.998232milliseconds, no pending screenshot, and a
present/visible address input. It was still waiting for a timed settlement pause.
The frame guard counted passive redraws as action retries. Boot likewise exhausted
20unpaced scroll-reveal attempts before its text target settled. The empty-row
explanation above was an initial hypothesis, not the complete optimized cause.
R2is retained/rejected; NFSwas not rerun for an already-failed artifact. Initial and
final selected identities agree and its failed receipt precedes freeze lift.

Both fixed actors now count only eligible action polls, pause unfinished phases
for25ms and preserve any longer settlement pause. Every frame still checks the
unchanged15second wall deadline;1200eligible polls and strict20scroll attempts/
one-physical-pixel visibility remain unchanged. No grading or learner behavior
changes. Pure regressions distinguish a million fast passive redraws from actual
retries and require timeout/active-bound/nonfinite-clock refusal. Fresh optimized
R3native evidence is required; passing development checks cannot admit the package.

### Disappearing confirmation regression

R3actual optimized EUI64flow passes37actions/11captures, both boot200captures are
delivered and private packaged NFScoldreopening passes. Boot then stalls at52,
phase1,572polls/15second deadline: the actual confirmation disappears before the
driver acknowledges its delivered click. R3is retained/rejected after final identity.

A real-widget regression clicks Confirm reset, allows the resulting layout to
settle, and then dispatches the actor. Before the fix, actual state resets but
actor remains52instead of53; the retained failing log demonstrates that mismatch.
The driver now uses the existing visible-click helper that drains a delivered click
before requiring a still-present rectangle. Back uses the same handling. The final
regression exercises both disappearing controls, exact selected reset and unrelated
attempt preservation; all seven bootwidget/capture checks pass. Final all-target
Clippy and the134/143focused Python guard groups pass. No action, grading, timeout,
visibility or persistence requirement is weakened. FreshR4package gates remain.
