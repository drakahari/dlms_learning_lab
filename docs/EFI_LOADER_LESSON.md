# UEFI loader recovery

In LPIC-1, **UEFI loader recovery: files and boot entries** follows the BIOS
recovery unit and builds on GPT disk preparation. The assignment is
**Restore the EFI loader and its boot entry** (`lpic1-grub-efi-registration`).

Learners identify the intended EFI System Partition and a separate boot directory,
install the modeled GRUB file, register its firmware path, and test the result.
The existing archive entry and every file on its EFI partition must remain
available after the intended working entry. Practice uses different disks,
mounts, partition identifiers and root UUID. It shows why installing the file
alone leaves the previous firmware order in effect. Learners can change that
order, test again, and recover.

## Model and supported commands

The supplied GPT role, partition GUID, FAT volume serial and ext4 root UUID have
different purposes. The typed FAT32 profile validates sector and cluster capacity
against the actual partition extent. It uses 512-byte sectors, 32 reserved sectors,
two FATs and a fixed directory allocation reserve. The file inventory tracks
cluster-rounded payload sizes and ASCII paths under `/EFI`. Path lookup folds case,
normalizes separators and refuses traversal. Each modeled EFI file has a bounded
size and role. The application and GRUB module records are symbolic.

| Operation | Supported forms |
| --- | --- |
| Inspect supplied devices and mounts | `lsblk -o NAME,TYPE,FSTYPE,MOUNTPOINTS`, `blkid [PARTITION]`, `findmnt -T SUPPLIED_DIRECTORY` |
| Read the supplied menu | `cat SUPPLIED_BOOT_DIRECTORY/grub/grub.cfg` |
| Install the supplied GRUB application | `grub-install --target=x86_64-efi --efi-directory=MOUNT --boot-directory=PATH --bootloader-id=ID --no-nvram` |
| Inspect firmware entries | `efibootmgr`, `efibootmgr -v`, `efibootmgr --verbose` |
| Register an application path | `efibootmgr --create --disk DISK --part NUMBER --label LABEL --loader PATH` and the documented short flags |
| Change the current order | `efibootmgr --bootorder XXXX,YYYY` or `efibootmgr -o XXXX,YYYY` |
| Verify the resulting chain | Native **Test boot chain · simulation** action |

The installer writes the modeled file on a supplied writable ESP. The configured
boot directory can reside on a different disk. With `--no-nvram`, installation
leaves firmware entries and order in their current state. Creating an entry adds
it to the front of BootOrder. A registration can reference a missing file. The
boot test looks up the actual path on the identified partition and can move on
when an entry is inactive or cannot start its application. Once GRUB starts, a
subsequent menu or Linux handoff problem stops that modeled path.

Learners enter one literal command at a time. Supported flags accept separate
values or `=value`. Firmware paths use backslashes and can be single quoted.
The parser rejects expansion, compound commands, unknown options and paths outside
the supplied records. Commands use only in-process state. The model bounds input
at 1,024 bytes and 24 words, console history at 8,192 bytes, EFI files at 16 and
firmware entries at eight.

## Grading, persistence and limits

The assignment grades the started partition, Linux root, kernel release, initial
root mode, startup target, and preserved archive entry and files. It accepts
equivalent valid loader IDs, labels, flags and path case. Changing applied files
or firmware order clears the saved test observation. Read-only inspection and
pending input leave the machine unchanged. Practice has independent state and
unfinished input. Reset restores the selected attempt.

The supplied-machine subset excludes EFI formatting, disk discovery, raw GPT/FAT
bytes, Secure Boot, PE execution, BootNext, removable fallback, GRUB Legacy and
interactive loader commands. The model does not use a real disk, firmware variable,
subprocess or host command. Simulator refusal and preservation rules describe this
activity's behavior. The coverage map credits partial LPIC-1 boot-manager practice.
Partition creation and the LPIC-2 alternate bootloader unit remain open.

## Sources

Source checks were made on 2026-11-10.

- [LPIC-1 exam objectives version 5.0, objective 102.2](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
- [UEFI Specification 2.11](https://uefi.org/sites/default/files/resources/UEFI_Spec_Final_2.11.pdf)
- [UEFI media access and FAT filesystem format](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html)
- [GNU GRUB 2.14 installation manual](https://www.gnu.org/software/grub/manual/grub/html_node/Installing-GRUB-using-grub_002dinstall.html)
- [Debian efibootmgr manual](https://manpages.debian.org/bookworm/efibootmgr/efibootmgr.8.en.html)

## Development verification

Nine EFI core, command and workspace tests, two FAT32 layout and path tests,
and four authored catalog and progress tests passed. Retained BIOS and partition
tests passed after introducing the supplied FAT32 layout. Six native widget and
capture tests cover both bootloader consoles. The revised development executable
completed 44 native EFI actions with 12 captures, including installation without
registration, wrong order, recovery, alternate commands and saved unfinished
input. The native captures include 100% and 150% dark layouts and a narrow 200%
light layout.

- [Introduction and complete worked command](../.build-artifacts/native-runs/run-patm2ds9/screenshots/workspaces/efi-loader-introduction.png)
- [Practice after installing the file without registration](../.build-artifacts/native-runs/run-patm2ds9/screenshots/workspaces/efi-loader-file-without-entry.png)
- [Independent assignment](../.build-artifacts/native-runs/run-patm2ds9/screenshots/workspaces/efi-loader-assignment.png)
- [Completed console at narrow 200%](../.build-artifacts/native-runs/run-patm2ds9/screenshots/workspaces/efi-loader-light200-console.png)

## Installed Fedora verification

The 382-lab package passed private installation and upgrade, exact catalog and
executable checks, the 44-action EFI flow and the retained 48-action BIOS flow.
The direct package also completed the separate-process NFS reopening check.
See CHECKPOINT.md for the launch path and checksums.

- [Installed introduction](../.build-artifacts/native-runs/run-05zgmjg0/screenshots/workspaces/efi-loader-introduction.png)
- [Installed file-only feedback](../.build-artifacts/native-runs/run-05zgmjg0/screenshots/workspaces/efi-loader-file-without-entry.png)
- [Installed wrong-order feedback](../.build-artifacts/native-runs/run-05zgmjg0/screenshots/workspaces/efi-loader-wrong-order.png)
- [Installed narrow light-theme console at 200%](../.build-artifacts/native-runs/run-05zgmjg0/screenshots/workspaces/efi-loader-light200-console.png)
