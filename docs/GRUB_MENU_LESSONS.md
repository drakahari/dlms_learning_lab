# GRUB menus: choice and recovery

LPIC-1 → **GRUB menus: choice and recovery**, immediately after **Boot entries:
from images to the system root**. Search for “GRUB” also finds both assignments.
Prerequisites remain advisory; previous IDs, progress and drafts are preserved.

| Stable ID | Independent assignment |
| --- | --- |
| `lpic1-grub-default-fallback` | Diagnose the failed supplied upgrade; select the working ordinary fallback while preserving three supplied entries. |
| `lpic1-grub-recovery-entry` | Repair and select the recovery entry's images, root and startup request while protecting the working normal entry. |

Introduction explains selection, command order and the distinction between
GRUB's file-reading root and Linux's `root=UUID=` before a complete workshop menu.
The recovery Introduction focuses on its additional repair/preservation task.
Try example uses different IDs, titles, ordering and filesystem values from the
assignment. Its actual optional **Fill worked example** only edits practice;
**Test menu · simulation** computes the selected handoff. Assignment retains its
intentional errors and full requirements. Challenge offers the same work without
unsolicited examples. Edits clear tested success immediately.

## Implemented subset

A bounded, in-process literal GRUB2 parser produces typed entries and delegates
image/root verification to the existing pure early-boot trace. No GRUB, Bash,
firmware, kernel, init, subprocess, host file or network operation is executed.
Paths are normalized literal keys in a supplied virtual `$BOOT` inventory.

- One explicit `set default=...`; selection of an existing entry by zero-based
  position, unique ID or quoted title. Numeric-only IDs/titles and ambiguous
  ID/title collisions are refused rather than assigned invented precedence.
- Up to four flat `menuentry` blocks with literal titles/IDs and braces on the
  documented lines. Each has one `linux` and an optional following `initrd`
  command, at most 16 kernel arguments and four ordered early images.
- Single/double literal quoting, ordinary word-boundary comments and
  backslash-newline joins. No variable/command expansion or compound commands.
- Kernel/initramfs existence and release agreement, supplied drivers/UUID lookup,
  system-root identity/init availability, initial `ro`/`rw` and ordinary/rescue
  requests. `quiet` is retained as configuration; the teaching trace stays visible
  and is not a kernel console.
- 4 KiB / 48 physical-line input bound. Unsupported syntax, extra commands, duplicated
  image/scalar commands and missing-default fallback are explicit diagnostics.
  Repeated `ro`/`rw` uses the supported kernel-option last-value behavior.

This is a hand-authored teaching menu. Editing a distribution-generated
`grub.cfg` is **not** presented as the ordinary persistent configuration workflow.
GRUB Legacy, `/etc/default/grub`/template generation, `grub-install`, MBR/ESP and
firmware installation, saved environment, interactive prompts, submenus, search/
device commands, Secure Boot and real boot failures remain unimplemented here.
The supplied readable boot partition/loader availability are preconditions.
The separate GRUB settings unit now adds bounded supplied-template generation;
this menu-editing activity does not perform it. No complete 102.2 or
exam-preparation claim follows from these tasks.

## Grading and persistence

Grade the actual selected handoff and source-held task requirements. Equivalent
supported selectors, quoting and option order pass. Protected entries are compared
as parsed configuration, not exact source strings; errors identify changed titles,
paths, root, initial mode, target or quiet setting. A bootable wrong target,
modified fallback, unsupported command and stale checked text fail distinctly.
Newly added entries cannot replace a required supplied choice.

`Task.menu` is opt-in metadata; absent legacy fields are skipped on serialization.
The original 376 scenarios /139 units and three Type #1 tasks remain exact. The extension
is `boot-entry-lessons-v2.json` plus `grub-menu-units-v1.json`; v1and explicit
historical getters remain available. Existing progress receives no answers or
new credit. Practice and assignment drafts survive save/restore separately,
including unfinished invalid text; imports cannot supply task facts or trusted
success booleans. Reset replaces only the selected attempt.

## Authoring

Provide genuinely separate initial/example facts, stable IDs, intentional starter
faults and reachable references. `menu.preserve` names 1–4 unique supplied entries;
`menu.selected_id` optionally requires a particular supplied entry. With an empty
selected ID, the selected entry must still come from the supplied menu and meet
the required kernel/root/mode/target. The path is a supplied
`/grub/learning-*.cfg`, never an instruction to edit a real boot configuration.
Use literal values within the disclosed grammar; unknown behavior requires a
model extension and tests before a lesson can claim it. New example IDs/order
must not expose the independent selector. Keep all three teaching views specific
to the actual controls and grade, following [the approved standard](TEACHING_PRESENTATION.md).

## Evidence and current limits

Final focused checks pass 30 integration groups across the new parser, authored
content/progress, retained Type #1 entries, current registration and IPv6 interface
IDs. Five GRUB widget/capture and seven retained boot-entry checks also pass.
Two exact normal-session UI checks, all-target Clippy, 1,013 Python checks and
strict coverage/inventory pass. Exact Ready378R1 package gates pass: actual content 378/140, 49 native actions /
14 captures, seven original screenshots opened, NFS separate-process reopening
and private installer lifecycle. Identity/paths are in CHECKPOINT.md. This is
not a full-inclusive Rust runtime or installed-path GUI claim.
Five widget/capture checks cover native editing, Tab, wrong/partial/correct/retry,
stale invalidation, separate practice/Challenge, selected reset/back and all five
supported scales in two widths/light-dark themes. Actual Fedora R1 and R2 each
complete 49 actions /14 PNGs. R1 opened 6 originals exposed doubled heading numbers and
repeated recovery material; corrected R2 opened 4 originals across both Introductions
and the recovery practice/assignment. Earlier R1 light 200% editor/protected-feedback
screens remain valid layout evidence; final packaged evidence is recorded separately.

The exact save/load/export/inspect/restore path and selected reset are tested
with synthetic project-disk profiles. This is not original personal progress,
a new GRUB cold-process reopening claim, actual GRUB runtime comparison,
independent novice comprehension, screen-reader or physical-device acceptance.
The native flow uses worked answers and authored references. Windows/macOS/Omarchy
runtime acceptance remains pending; additional OS delivery is deferred by owner.

## Primary references, checked 2026-11-10

- [LPI LPIC-1 objectives v5.0, 101-500/102-500](https://www.lpi.org/our-certifications/exam-101-102-objectives/): narrow 102.2 backup/menu-configuration practice; installation and other knowledge areas remain open.
- GNU GRUB Manual 2.14: [default selection](https://www.gnu.org/software/grub/manual/grub/html_node/default.html), [menuentry](https://www.gnu.org/software/grub/manual/grub/html_node/menuentry.html), [quoting and scripting syntax](https://www.gnu.org/software/grub/manual/grub/html_node/Shell_002dlike-scripting.html), [Linux handoff](https://www.gnu.org/software/grub/manual/grub/html_node/GNU_002fLinux.html). Official indexed manual text was available; direct GNU page fetches timed out. No real-GRUB comparison is inferred.
- [Linux kernel arguments](https://docs.kernel.org/admin-guide/kernel-parameters.html) and [init handoff](https://docs.kernel.org/admin-guide/init.html).

All descriptions/examples are original authored material; no external code or
manual passages are imported. Existing project/third-party notices remain intact.
