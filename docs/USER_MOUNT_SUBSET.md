# Ordinary-caller removable mounts

Implemented opt-in extension to the existing virtual storage engine. Existing
storage/removable lessons, IDs and serialized mount fields retain their behavior.
No host mount, subprocess, UID lookup, device access or kernel changes occur.

Three supplied actors represent an administrator and two ordinary operators.
`user` permits either ordinary operator to mount; among ordinary callers, only
the actual successful mounter may unmount. `users` permits ordinary handover.
Administrator permission is distinct. Mount permission never changes ownership
or access permissions inside an ext4/XFS filesystem.

The current saved source/target entry governs ordinary unmount authorization.
Saving does not replace the actual mounter, change live read-only state or
reapply flags. This fixed simulator treats another request for the same mounted
volume/options as Already mounted, retaining its mounter; it does not model
Linux's ability to stack mounts. Unmount/remount is the supported application
workflow. Source identities remain the supplied UUIDs, independent of aliases.

Supported six-field options: `defaults`, `ro/rw`, `auto/noauto`, `nofail`, one
bare `user`, `users` or `nouser`, and `exec/noexec`, `suid/nosuid`, `dev/nodev`.
`user/users` set noexec, nosuid and nodev; later opposite options override them.
In the explicitly supplied base those restrictions start clear; defaults is a
no-op, not a universal kernel-default claim. Flags are inspected policy facts:
execution, set-user-ID and device effects are not simulated.

Mixed/repeated permission tokens, `owner/group`, `user=NAME`, `nousers`, helpers,
CLI option merging, busy/lazy/forced detach, hot removal, nested/network mounts,
FUSE, namespaces, polkit, discretionary file access and ACLs are unsupported.
Unsupported syntax is diagnosed rather than approximated.

All state is reconstructed from bounded successful actions against the authored
initial state. Refusals and no-op requests do not transfer ownership or rewrite
flags. Saved histories must replay successfully; no imported ledger asserts
provenance. Histories cap at48actions/64KiB, configuration16KiB/eight entries,
activity2MiB, five authored cases per task (model maximum8). Cooperative checks
advance one finite case per frame; Stop drops the unfinished result. Draft and
successful session actions remain saved. Serialized result caches restore no
current credit until actual bounded replay; original history remains distinguishable.

Sources checked2026-11-04:

- [LPI LPIC-1 v5.0,104.3](https://www.lpi.org/our-certifications/exam-101-102-objectives/).
- [mount(8)](https://man7.org/linux/man-pages/man8/mount.8.html),
  [fstab(5)](https://man7.org/linux/man-pages/man5/fstab.5.html),
  [umount(8)](https://man7.org/linux/man-pages/man8/umount.8.html).
- [util-linux v2.41 unmount authorization](https://github.com/util-linux/util-linux/blob/v2.41/libmount/src/context_umount.c).

This is partial104.3 practice, not full exam/equipment equivalence. Upstream text
or implementation is referenced, not copied or linked into the simulator.
