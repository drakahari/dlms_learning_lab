# Temporary GRUB recovery

The LPIC-2 **Temporary GRUB recovery** unit follows independent removable EFI
recovery. The assignment, `lpic2-grub-one-boot-recovery`, uses a supplied saved
menu with working ordinary startup and an inspection entry whose root request
is outdated.

The Introduction explains which kernel and initramfs belong to the entry, how
`root=UUID=` selects the system filesystem and how a one-time menu edit differs
from saved configuration. Guided practice tests the outdated root, fixes the
UUID and observes a successful ordinary target request, then adds
`systemd.unit=rescue.target` to the edited line. The independent assignment uses
different images and root identities.

Choose **Boot once** to test the selected entry's edited `linux` line. The test
traces both the edit and a subsequent ordinary boot from the unmodified menu.
Changing either the selected entry or its line clears the displayed result.
**Discard edit** restores the selected entry's saved line. Practice remains
separate from assignment progress.

## Scope

One literal `linux` command may be edited. The entry's saved `initrd` paths and
default selection are supplied. Equivalent supported quoting and argument
ordering are accepted. Additional commands, shell evaluation, expansion and
device or host file access are outside the model. The result describes the
kernel and init handoff and the requested systemd target. The simulation
does not start rescue or emergency services.

This exercise targets a narrow slice of LPIC-2 objective 202.2. Source checks
were made on 2026-11-10.

- [GNU GRUB 2.14 menu entry editor](https://www.gnu.org/software/grub/manual/grub/grub.html#Menu-entry-editor)
- [Linux Professional Institute LPIC-2 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [systemd special targets](https://www.freedesktop.org/software/systemd/man/262/systemd.special.html)

## Fedora development checks

Six temporary boot tests and six retained menu parser tests passed. They verify
both outcomes, wrong roots, mismatched images, unsupported or multiline input,
alternate arguments, stale result invalidation, separate drafts and a real
save/reload. Catalog admission retains all 383 previous scenarios and 144 units.

Both Fedora development walkthroughs completed 34 actions and captured 10
screens. The first exposed a clipped worked UUID and temporary command and
repeated pending checks after a blocked root. The second shows wrapped values
and a focused diagnostic. The console remains usable at normal and 150% dark
text and in the narrow 200% Light view. Originals remain private under
`.build-artifacts/native-runs/run-a_aed7dp/screenshots/workspaces/` (before)
and `.build-artifacts/native-runs/run-07_nggve/screenshots/workspaces/` (after).

## Fedora candidate evidence

The separate package passed private upgrade and reinstall, the installed
34-action GRUB workflow, the retained 41-action removable EFI workflow and
NFS cold reopening. The installed GRUB captures are in
`.build-artifacts/native-runs/run-b8pluzqs/screenshots/workspaces/`. The launch path and checksums are in
[the checkpoint](CHECKPOINT.md).
