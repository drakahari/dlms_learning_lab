# LVM-backed virtual filesystem mounting

The bounded model and two authored tasks are integrated2026-11-07. MAINhas335labs/
115units; focused and development-native checks pass. Final package validation and
coverage reconciliation remain pending. This is a narrow204.3mounting slice,
not full objective, kernel or real-device fidelity. See LVM_MOUNT_LESSONS.md.

## What is modeled

`lvm::mounts` joins the existing pure LVM and `storage` models. Supplied synthetic
filesystem UUIDs bind explicitly to existing original LV objects. An internal
simulation ID is not a filesystem UUID, PV UUID or real hardware observation.
The adapter derives each current canonical `/dev/VG/LV` name from actual LVM
identity and uses the existing bounded fstab parser and virtual mount evaluator.
These supplied ordinary filesystems are modeled as ext4, consistent with the
existing resize2fs exercises; no real format inspection is implied.

Saved policy and live mounts are distinct. Saving text does not replace a live
mount. A mount uses its supplied saved entry and requires its LV to be active.
A reader through a mounted path returns actual data from the original simulated
LVM filesystem; it does not copy a canned answer or trust a pathname as identity.
Renaming its VG or LV preserves the live filesystem association. A saved old
name may become invalid; its supplied filesystem UUID remains unchanged.

The adapter couples supported LVM name/availability actions to live mounts in
one transaction. An attempt to deactivate a mounted filesystem is refused
without partial effects. Unmount first where appropriate. Existing immutable
source-held open-block-device facts retain their old meaning and restrictions;
they are not invented open-file or process facts. This is not a real-system
safety guarantee.

## Disclosed command and configuration subset

The opt-in `lvm_workspace::mount_flow` interpreter supports only:

- `pvs`, `vgs`, `lvs`, and the existing supplied `vgchange`, `vgrename`,
  `lvrename` forms.
- `blkid`: the supplied virtual filesystem UUID/type/current-name inventory.
- `findmnt [supplied-mountpoint]`: actual current virtual mount associations.
- `mount supplied-mountpoint`: resolve its saved fstab entry and attach its
  active original filesystem.
- `umount supplied-mountpoint`: remove that current virtual association.
- `cat supplied-mountpoint/file`: read the actual mounted virtual file.

The six-field fstab subset is inherited from `storage`: canonical device or
`UUID=` source; fixed supplied mountpoint; `ext4`; `defaults`, `rw`/`ro`,
`auto`/`noauto` options; supported numeric dump/check fields. Comments are
whole-line only. Unknown flags, escaping/quoting, arbitrary sources, shell
syntax, pipelines, redirects and unsupported LVM mutations fail honestly.
The editor's Save action is a native simulator action, not a Linux command.
`auto` is saved policy here, not proof of a real system boot or systemd reload.

Limits:1–4 supplied filesystems/points,4096configuration bytes/64lines,
2048command bytes, bounded serialized context/live data and existing bounded
LVM allocations/files. Supplied mountpoints must be distinct and non-nested.
One live mount per filesystem and unmount-before-replacement are simulator
restrictions: real Linux supports multiple/overlaid mounts and remounts.
No busy-filesystem processes, namespaces, mount propagation, bind/network/loop
mounts, force/lazy unmount, permissions/privilege emulation, udev discovery,
mapper aliases, automatic activation or kernel/device operations are modeled.
Underlying unmounted directory contents are explicitly outside this exercise.

## Teaching, grading and progress boundary

The typed flow keeps editor drafts, saved policy and current mounts separately.
Assignment and example runtimes have different supplied storage and remain
isolated. Task admission executes actual typed reference actions; practice must
start incomplete, reach its requirements, demonstrate a real broken outcome and
repair it. These authoring checks never run in place of a learner action.

Checks recompute saved source identity, access/automatic policy, live identity,
actual original file contents, unsaved edits and preservation requirements.
Optional renaming requirements refer to original objects. Identical text from
the wrong filesystem cannot satisfy identity requirements. Current success is
lost after an invalidating edit, save, unmount or name change; no cached report
or command string awards credit. Canonical and UUID alternatives and supported
whitespace/option alternatives are accepted by behavior.

Optional default/omitted source, lesson and session fields keep old serialized
values exact. Both unfinished drafts survive serialization; reset affects only
its selected scope. A new profile does not turn old progress into completed
mounting work. Normal LVM sources still reject mount/umount/findmnt commands.

## References and evidence limits

Primary references checked2026-11-07:
[fstab(5)](https://man7.org/linux/man-pages/man5/fstab.5.html),
[mount(8)](https://man7.org/linux/man-pages/man8/mount.8.html),
[umount(8)](https://man7.org/linux/man-pages/man8/umount.8.html).
These support saved policy, mountpoint/file-tree behavior, filesystem UUIDs,
multiple real mounts and busy unmount differences. No host mount/umount command,
personal fstab, kernel mount table, host device or external helper is used as a
reference engine. Pure tests use only disposable in-memory fixtures.

Current evidence:12adapter tests,12flow tests,6source/session tests and retained
configuration/reference/snapshot sessions pass. R17seven authored tests pass;
R21Fedora native164actions/50captures exits0. All three views of each task and
three additional syntax/scaled images were opened. Native feedback polish is now
covered by R23seven actual egui tests; final candidate/screens remain pending.
Initial include-path, new-data admission and synthetic-keyboard failures remain
retained locally and are corrected without weakening grading or source guards.
This is root-reviewed development evidence, not independent full Linux fidelity,
novice, screen-reader or cross-platform runtime acceptance.
