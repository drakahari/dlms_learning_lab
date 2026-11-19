# Inspect and re-export a virtual NFS policy

The utility-focused lesson follows the existing NFS handover unit. It uses the
same bounded NFSv3-style state for export policy, mount decisions, numeric
identity and supplied files. The new activity adds a small native terminal
beside the export editor and client tests. Existing NFS inputs and grading do
not need a new workspace or progress migration.

| Supported command | Observed state or transition |
| --- | --- |
| `exportfs -v` | Reads the currently applied exports on the supplied server, including the options represented by the simulator. |
| `exportfs -r`, `exportfs -ra` | Applies the saved virtual `/etc/exports` file. The equivalent `-ar` and separate `-r -a` option orderings are accepted. |
| `showmount -e ADDRESS` | Lists the applied exports advertised by the known virtual NFSv3 server. `--exports` is equivalent. |

A virtual server and two test clients are supplied. The administrator saves one
policy edit, finds an old mount that conflicts with the planned withdrawal and
removes only that mount. The remaining export clauses and unrelated archive
mount must stay intact. Normalized matches ignore the order of independent
lines, interchangeable supported syntax and harmless comments. The access
checks use actual virtual mount and read permissions. A fresh approved client
must still read, and a retired client must fail before receiving a mount.

The translated re-export action retains the existing simulator rule that
protects incompatible live mounts by refusing the change. It demonstrates the
difference between saved and applied policy. Real Linux does not provide this
simulator's whole-document rollback or identical active-filehandle behavior.
The displayed `exportfs -v` options reflect the modeled fields, and omit options
outside this exercise. `showmount` can describe available exports without
establishing actual mount admission or file access. NFSv4-only servers may not
advertise a traditional MNT service.

Terminal commands are parsed in process, never passed to a shell, daemon, host
filesystem or real network. Only the single supplied export file participates.
There is no `/etc/exports.d` expansion, privileged system command, inode or
kernel operation, arbitrary hostname lookup, real RPC MNT, NFSv4 negotiation,
mountd administration, TCP wrapper handling or nfsstat/portmapper coverage.
The replacement launcher and preceding progress remain local to review.

## Primary sources checked November 17, 2026

[LPI LPIC-2 v4.5, objective 209.2](https://www.lpi.org/our-certifications/exam-201-202-objectives/#rdp-we-209.2_NFS_Server_Configuration_(weight:_3)) ·
[Upstream exportfs manual source, master](https://github.com/linux-nfs/nfs-utils/blob/master/utils/exportfs/exportfs.man) ·
[Upstream showmount manual source, master](https://github.com/linux-nfs/nfs-utils/blob/master/utils/showmount/showmount.man) ·
[RFC 1813, NFS version 3](https://www.rfc-editor.org/info/rfc1813)
