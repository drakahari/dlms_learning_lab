# BIOS loader recovery: disk and boot files

LPIC-1 → **BIOS loader recovery: disk and boot files**, immediately after
**GRUB settings: generate and verify**. Advisory prerequisites also include
**MBR layouts**. Stable assignment ID: `lpic1-grub-bios-install`.

The Introduction teaches BIOS → MBR → embedded core → supplied menu → existing
kernel/initramfs/root handoff. Its worked commands use different disks, UUID and
mounted directory. Guided practice includes real inventory reads, inert command
insertion, installation, an empty-menu failure and repair. The independent
assignment requires choosing the intended whole disk and configured directory,
verifying the complete chain, and preserving the other loader. It is one distinct
scenario, not a numerical variant counted several times.

## Supported behavior and isolation

All commands below operate in process on supplied virtual geometry and typed boot
records. No actual tools, files, disks, firmware, services or network are accessed.

- `grub-install [--target=i386-pc] [--boot-directory=PATH] WHOLE_DISK`
- `lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS`
- `findmnt -T SUPPLIED_BOOT_DIRECTORY`
- `cat SUPPLIED_BOOT_DIRECTORY/grub/grub.cfg`

Options allow `=value` or separate literal words, either order, and literal quoting
or escapes. Unknown/duplicate options, shell expansion, pipes and compound commands
are rejected. Defaults are the supplied i386-pc package and `/boot`. Paths must name
supplied normalized directories; no arbitrary path lookup occurs. Input is bounded
at 1024 bytes/24 words and console history at 8192 bytes.

Installation checks writable disk/directory facts and the actual free sectors
between sector0 and the first MBR partition. The core's reservation is an explicit
supplied fact, not a universal size. It changes only a typed boot-code/module-location
record; partition ranges, UUIDs and file records remain exact. Mounted target
filesystems are valid: a rescue installation is not rejected simply because its
filesystem is mounted. A boot directory may legitimately reside on another disk.

A successful installation on the wrong disk does not repair the configured BIOS
startup disk. A directory without a menu can accept loader installation but cannot
complete the later test: installation does not generate `grub.cfg`. **Test boot
chain · simulation** is a native simulator action, not a Linux command. It traces
current records and separately grades the requested system/release/mode/target and
preserved alternate loaders. New installations invalidate previous observations;
pending typed input and read-only inventory commands do not change installed state.

The simulator refuses operations atomically; actual tools can overwrite valuable
boot code. This is not a promise of real-tool safety. Other-loader restoration is
outside the subset: Reset restores only this supplied attempt, retaining other labs.

## Boundaries and persistence

Only BIOS/MBR, 512-byte sectors, supplied ext4 mounted directories and the existing
literal GRUB2/image/root subset are modeled. No raw boot bytes, image parsing,
blocklists, `--force`, BIOS/GPT, EFI/FAT/ESP, GRUB Legacy, interactive boot prompt,
Secure Boot, mount/chroot, package installation or configuration generation is added.
Those skills remain open; this assignment does not complete objective102.2.

Saved sessions contain independent assignment/practice worlds, pending input,
bounded console output and current test observations. Source-owned geometry,
menus, expected outcomes and reference methods are never taken from saved progress.
Admission validates context/version and refuses replacement task authority. New
requirements receive no credit from another completed lab. Existing380 scenarios,
141 units and their data remain exact via retained baseline getters and comparisons.

## Sources and verification status

Authoritative sources checked2026-11-10:

- [LPI LPIC-1 objectives5.0, exams101-500/102-500](https://www.lpi.org/our-certifications/exam-101-102-objectives/), objective102.2.
- [GNU GRUB Manual2.14: BIOS installation](https://www.gnu.org/software/grub/manual/grub/html_node/BIOS-installation.html).
- [Installing GRUB using grub-install](https://www.gnu.org/software/grub/manual/grub/html_node/Installing-GRUB-using-grub_002dinstall.html).
- [Invoking grub-install](https://www.gnu.org/software/grub/manual/grub/html_node/Invoking-grub_002dinstall.html).

The GNU indexed primary-source excerpts were available; direct GNU page retrieval
timed out in this continuation. No GRUB code/manual text was imported. No actual
GRUB execution or machine boot was used as a reference oracle. Tests cover typed
geometry, alternate commands, failure/partial results, output bounds, isolation,
source admission, reset/resume and safe backup/restore.

The revised development build completed 48 native actions and saved 12 screenshots.
Inspection confirmed the split console at 100% and 150% in the dark theme and
stacked controls at narrow 200% in the light theme. The UI tests exercise real
keyboard input, visible-control scrolling, separate practice, selected Reset and
all supported text scales. The native fixture saved and read back an unfinished
command alongside the tested installation and separate practice. Separate-process
BIOS draft reopening, independent novice review, screen-reader use and physical
devices remain unverified.

- [Introduction](../.build-artifacts/native-runs/run-okrud0zx/screenshots/workspaces/bios-loader-introduction.png)
- [Practice and wrong-directory feedback](../.build-artifacts/native-runs/run-okrud0zx/screenshots/workspaces/bios-loader-empty-menu-feedback.png)
- [Independent assignment](../.build-artifacts/native-runs/run-okrud0zx/screenshots/workspaces/bios-loader-assignment.png)
- [Narrow console at 200% in the light theme](../.build-artifacts/native-runs/run-okrud0zx/screenshots/workspaces/bios-loader-light200-console.png)

The installed Fedora package also completed all 48 native actions and 12 captures.
The same executable hash passed catalog validation, private upgrade and reinstall,
and two-process NFS reopening. The installed practice screenshot shows the latest
command at the bottom of the console and the missing-menu result beside Test.
The package identity and exact launch path are recorded in CHECKPOINT.md.

- [Installed introduction](../.build-artifacts/native-runs/run-us5u770k/screenshots/workspaces/bios-loader-introduction.png)
- [Installed practice feedback](../.build-artifacts/native-runs/run-us5u770k/screenshots/workspaces/bios-loader-empty-menu-feedback.png)
- [Installed assignment](../.build-artifacts/native-runs/run-us5u770k/screenshots/workspaces/bios-loader-assignment.png)
- [Installed narrow console at 200%](../.build-artifacts/native-runs/run-us5u770k/screenshots/workspaces/bios-loader-light200-console.png)
