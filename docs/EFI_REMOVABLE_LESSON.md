# Independent removable EFI recovery

In LPIC-2, **Independent removable EFI recovery** builds on named EFI loader
recovery and service target handover. The assignment is **Recover from a
removable loader without the internal disk** (`lpic2-efi-removable-recovery`).

Learners repair a removable loader whose configured boot directory depends on
the internal disk. Guided practice first tests the installed state, then tries a
copied menu on the removable device. That second test reaches GRUB, the kernel
and early userspace before the original internal root UUID blocks recovery. A
working repair uses the supplied recovery menu and root. The assignment uses
different disks, UUIDs, mount paths and a different kernel release.

The **Test recovery paths** action checks two conditions against one current
installation. Normal startup follows the saved firmware entries and BootOrder.
The recovery test selects the supplied removable device and treats the designated
internal disk as unavailable. The learner must preserve normal startup, firmware
entries and all files outside the removable loader path.

## Supported model

The existing named-entry commands remain available. A removable install uses
the following supplied x64 form.

```text
grub-install --target=x86_64-efi --efi-directory=MOUNT --boot-directory=PATH --removable
```

The command writes the symbolic default application at
`\EFI\BOOT\BOOTX64.EFI` on the selected mounted removable ESP. It accepts
literal quoting, option order and a redundant `--no-nvram`. The installed
application names a separate boot directory. The model verifies the actual
device identity and configured directory before following the selected menu.
A root UUID on an unavailable disk stops the trace at the root stage.

The test reads existing state. It does not erase an unavailable disk or change
the normal firmware order. It validates the intended root, kernel release, initial
root mode and rescue target request. Applied changes invalidate the current test.
An unsupported command or pending input does not change the installation.
Practice and assignment retain separate drafts.

The scope is a supplied x64 FAT32 ESP and symbolic application, one unavailable
disk, and a bounded GRUB2 and Linux handoff. The app does not run GRUB or a real
kernel, change host mounts, alter firmware, create live media, format FAT or
inspect raw disk images. The lesson exercises the relevant portions of LPIC-2
objective 202.2. NVMe, hardware and daemon initialization, interactive loader use,
Legacy GRUB, actual rescue or emergency service control and the alternate
bootloader objective remain open.

## Fedora development evidence

The native workflow completed 41 actions and saved 12 screenshots. It exercised
keyboard command entry, inert worked-command insertion, separate practice, both
blocked stages, alternate repair, invalidation, and progress save/readback. The
dark console used a split view at 100% and 150%. The narrow Light console and
both results were visible at 200%. Long console lines scroll horizontally.
Screenshot originals are kept in the ignored directory
`.build-artifacts/native-runs/run-d9ddqt6v/screenshots/workspaces/`.

The 11 loader tests, four authored tests and seven UI tests passed. Catalog tests
compare all previous 382 scenario objects and 143 units with their retained
values. Automated and native checks used synthetic progress.

## Fedora candidate evidence

The separate review package passed its exact catalog check, a private upgrade
and reinstall, the installed 41-action recovery flow, the retained 44-action
named EFI flow, and separate-process NFS reopening. Installed recovery captures
are under `.build-artifacts/native-runs/run-ck0fi056/screenshots/workspaces/`. The launch path and checksums are in
[the checkpoint](CHECKPOINT.md).

## Sources

Source checks were made on 2026-11-10.

- [LPIC-2 objectives, system recovery 202.2 and alternate bootloaders 202.3](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [UEFI 2.11 Boot Manager and removable media behavior](https://uefi.org/specs/UEFI/2.11/03_Boot_Manager.html)
- [UEFI 2.11 media and FAT filesystem format](https://uefi.org/specs/UEFI/2.11/13_Protocols_Media_Access.html)
- [GNU GRUB 2.14 installation manual](https://www.gnu.org/software/grub/manual/grub/html_node/Installing-GRUB-using-grub_002dinstall.html)
