# Virtual GPT and blank-partition preparation

This bounded model supports native virtual-disk preparation for the still-Open
LPIC-1 v5.0 104.1 objective. It executes no process and opens no disk, filesystem
or host device. `/dev/vda` and similar names are keys in supplied virtual state.
Typed workspace, grading and progress integration now exist; all19 pure tests
pass. Four authored tasks pass their12 content groups but remain unregistered
pending normal-catalog and desktop gates. There is no objective-credit change.
The later source-review amendments below retain their historical proposal status;
current verification and continuation are recorded in CHECKPOINT.md.

A disk partition names a range of sectors. A partition type hint is separate
from the actual filesystem or swap signature inside that range. Creating an
`ext4` partition does not format it. The model reports these separately, so a
learner cannot complete a future formatting task by changing a partition code.

The supplied geometry has exactly512-byte logical sectors and a fixed GPT
layout with128entries of128bytes. LBA0 is reserved for a protective MBR, LBA1
for the primary header, and LBAs2–33 for entries. The final33sectors retain
space for backup entries and header. Usable inclusive sector numbers are34
through `sector_count - 34`. At most16slots are exposed in this bounded model.
This is a fixed valid layout, not every valid GPT layout or a raw GPT encoder.
It does not write or check CRCs, partition GUIDs or actual image bytes.

Partition start/end coordinates are exact inclusive sector numbers. Adjacent
ranges may meet at `previous_end + 1`; sharing even one sector is overlap.
`align-check optimal` checks the start against supplied modeled alignment,
not the end. Unaligned but otherwise valid partitions can be created and then
diagnosed. Optimal alignment is an explicit power-of-two fixture from1through
8192sectors, not a universal1MiB rule or discovered device characteristic.

Blank disks accept GPT initialization. GPT accepts named Linux-data or
Linux-swap partition hints. The lowest free modeled partition slot is used.
The ext4 operation creates a signature with4096-byte blocks and total modeled
capacity `floor(partition_bytes / 4096)`. The preparation profile requires1MiB
or more for ext4. No allocation bitmap, journal, inode table, reserved-block
policy or available-file-space result is simulated. This is not a byte-valid
ext4 implementation or assurance that real defaults fit a small filesystem.
Swap initialization requires at least10modeled4096-byte pages and reserves
one header page; activation, priorities and kernel memory are absent.

GPT type hints do not prevent initializing another supported content type:
real `mkswap` does not use the partition code as its source of truth. A future
Goal may independently require the intended partition type and signature.
Labels are restricted to at most16ASCII letters/digits/`._-`. UUIDs may be
explicit nonzero canonical lowercase strings; otherwise a persistent counter
allocates a unique deterministic simulator identity. These identifiers are
not random UUID generation or disk identities read from the host.

This preparation-only subset refuses any reinitialization of an existing table,
opaque occupied disk or partition signature, including empty existing ext4.
It also refuses mutations to supplied read-only/in-use disks. Real tools may
prompt, overwrite or accept force options; simulator refusal is not a real-system
protection guarantee. A failure preserves every disk, partition, retained file,
UUID counter and serialized state. Transactions validate borrowed source and
action before copying the bounded candidate; publication follows whole-candidate
validation. Real disk operations do not acquire this simulator rollback property.

The finite literal command adapter accepts:

```text
parted -s DEVICE mklabel gpt
parted -s DEVICE unit s mkpart NAME ext4|linux-swap STARTs ENDs
parted -s DEVICE unit s print
parted -s DEVICE align-check optimal NUMBER
mkfs.ext4 [-b 4096] [-L LABEL] [-U UUID] PARTITION
mkfs -t ext4 [-b 4096] [-L LABEL] [-U UUID] PARTITION
mke2fs -t ext4 [-b 4096] [-L LABEL] [-U UUID] PARTITION
mkswap [-L LABEL] [-U UUID] PARTITION
blkid [PARTITION]
```

Formatting options may be reordered, but not repeated. Commands use one supplied
`/dev/vd[a-z]` disk path or its canonical positive partition suffix1through16.
There are no aliases, shell strings, pipes, redirects, escaping, substitutions,
quotes, interactive prompts, percentages, negative sectors or approximate unit
rounding. The token `NAME` is an ordinary GPT name, not an MBR primary/logical
selector. Reports explicitly identify simulator output and derive their fields
from current state; they do not reproduce full utility output. Alignment returns
status1 when misaligned; signature lookup with no matching signature returns2.

State is bounded to4disks,16partitions each,16flat retained files per ext4
partition,16KiB per file,128KiB aggregate file data, and256KiB escaped JSON.
Virtual disk size is capped at16TiB; no disk-capacity-sized allocation exists.
Raw JSON is size-checked before decoding. Custom visitors bound text, reject
duplicate map keys and refuse excess collection bodies. Programmatically built
states are validated before copies, reports and serialization. JSON encoding
uses a capped counting writer before allocating output. Stored file fixtures
provide exact preservation evidence, not a mounted writable filesystem.

Resume is a bounded full-state JSON roundtrip. Reset takes a separately supplied
validated initial state with matching device facts. This core function does not
decide whether that caller-supplied source is authoritative. A future workspace
must bind authored initial facts and Goal before accepting every attempt, save,
restore/export inspection or progress; a self-carried matching state/source is
not authority. No core test earns progress or proves that future boundary.

Unsupported scope remains explicit: MBR/DOS/extended/logical/hybrid tables,
4Kn or512e discovery, GPT repair/recovery/CRC/raw images, deletion/resizing,
force or destructive replacement, fdisk/gdisk interactive workflows, ext2/ext3,
XFS/VFAT/exFAT/Btrfs, filesystem features and maintenance, boot/ESP integration,
mounts/fstab, swap activation, LVM/RAID, performance and host disk operations.
There is no inherited102.1,104.2/104.3 or LPIC-2 credit.104.1/GPT-K1 now have a narrowly Partial native mapping after authored tasks,
normal integration and scoped installed197 gates; all other104.1 rows stay Open. No complete utility or objective claim is proposed.

Primary sources checked2026-10-29; paraphrases only, no upstream code/manual text:

- [Official LPI v5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/#rdp-we-104.1_Create_partitions_and_filesystems)
  establish that the complete objective includes MBR/GPT, several filesystem
  families, swap and Btrfs, beyond this subset.
- [GNU Parted manual](https://www.gnu.org/software/parted/manual/html_node/mkpart.html)
  web requests timed out. The installed GNU distribution `parted.info.gz`
  was read instead. Its mkpart/unit/align-check nodes distinguish partition
  creation, precise sector units and start alignment. Exact local hashes are
  retained in the private reference receipt, with no manual text copied.
- [Upstream util-linux GPT implementation](https://raw.githubusercontent.com/util-linux/util-linux/master/libfdisk/src/gpt.c)
  supports the header/entry/reserved-region reasoning. The model fixes128entries
  and512-byte sectors rather than implementing its general geometry choices.
  The UEFI2.10 web specification returned403; it is not claimed as a read source.
- [Upstream e2fsprogs mke2fs manual](https://raw.githubusercontent.com/tytso/e2fsprogs/master/misc/mke2fs.8.in)
  describes explicit ext4/block/label/UUID options and omitted-size device use.
  Real configuration defaults, metadata and features are omitted here.
- [Upstream util-linux mkswap manual](https://raw.githubusercontent.com/util-linux/util-linux/master/disk-utils/mkswap.8.adoc)
  distinguishes initialization from activation, minimum page count and labels/UUIDs.

Future integration must add only the `partition_fs` library export while retaining
all current LVM/NTP/HSRP/Links modules. No whole old lib.rs afterimage may replace
newer exports. Core design/source review precedes authored-unit work; compilation,
19 prospective pure groups and all-target Clippy remain Root-only gates.

## R2 source review amendment

The public `environment_matches` predicate now fails closed if either the current
or supplied source state is invalid, before comparing immutable device facts.
Its positive configuration-change semantics are unchanged. The original18test
bodies remain byteexact and one appended regression covers invalid version,
identity counter, reserved-sector range, overlap and filesystem capacity in either
argument, plus unchanged reset refusal. All19groups remain UNRUN. This validity
check still does not authenticate a self-carried source or Goal; future authored
workspace authority remains a separate mandatory boundary.
