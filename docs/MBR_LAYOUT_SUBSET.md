# MBR partition layouts in virtual storage

The optional Dos512 profile adds bounded primary, extended and logical partitions
to the existing native disk editor. Old GPT sources and drafts omit these fields
and retain their original behavior. Learner operations never access a host device,
filesystem, kernel, subprocess or network. No dependency or shell change is needed.

| Supported literal form | Modeled behavior |
|---|---|
| `parted -s /dev/vdX mklabel msdos` | Initialize only a supplied blank writable disk. |
| `parted -s /dev/vdX unit s mkpart primary\|logical ext4\|linux-swap STARTs ENDs` | Add an inclusive payload range with a type hint; does not format it. |
| `parted -s /dev/vdX unit s mkpart extended STARTs ENDs` | Add one blank container, not a filesystem. |
| `parted -s /dev/vdX unit s print`, `parted -s /dev/vdX align-check optimal NUMBER` | Inspect the actual virtual table and modeled alignment. |
| `blkid [PARTITION]` | Inspect actual initialized virtual signatures. |

The existing blank-only ext4/swap initialization forms remain disclosed by the
workspace's command reference. The lesson uses native role, range, selection and
format controls; its worked operations describe the corresponding literal subset.
It does not execute those examples on Linux. Unsupported flags, host paths, units,
shell expressions, delete, force, resize, boot code and raw EBR manipulation refuse.

This profile has512-byte sectors and disks up to2TiB. Primary and extended entries
share slots1–4. One extended container can hold consecutive logical payloads
numbered5–16. Each logical payload has an explicit earlier, disjoint one-sector
modeled EBR reservation within its container. The bounded allocator selects a free
reservation without scanning every sector. This is a teaching abstraction, not raw
EBR-chain emulation or a claim about actual GNU Parted placement policy. CHS,
bootability, rescue/recovery, kernel rescanning and actual equipment are excluded.

Typed validation distinguishes intentional container nesting from illegal payload
or metadata overlap. Containers cannot be formatted. Existing signatures, readonly
or in-use objects cannot be overwritten. Operations validate a candidate before
publishing; simulator refusal is not protection promised by real disk utilities.
The existing disk/state/session/input budgets remain; there are at most16entries
per disk and8outcome targets. Target matching is bounded and assigns distinct
payloads, accepting supported alternate ranges, sizes and creation orders.

Goals check current role, capacity, alignment, type hint, filesystem/swap content
and requested labels, while preserving supplied geometry, identity, files and all
unrelated media. A supplied-slot goal requires the original specified blank slot;
formatting a substitute elsewhere cannot pass. Partial work gets layer-specific
feedback. Extra unrequested allocations invalidate success. Check flags, transcripts
and walkthrough position never constitute a solution. Old named-GPT goals cannot
admit the new profile through imported state.

## Sources and acceptance limits

Authoritative sources checked2026-11-07:
[LPI101/102objectives, version5.0](https://www.lpi.org/our-certifications/exam-101-102-objectives/),
[GNU Parted mkpart](https://www.gnu.org/s/parted/manual/html_node/mkpart.html),
and [util-linux fdisk(8)](https://man7.org/linux/man-pages/man8/fdisk.8.html).
The last reference informs limitations; fdisk itself is not simulated. No actual
partition-tool or device conformance comparison has been performed.104.1remains
partial and102.1design-hard-disk-layout remains open; native bounded exercises do
not establish full Linux storage, boot, hardware or exam coverage.
