# Current integration evidence — 2026-11-02

The reviewed pure filesystem capacity model is now exported by the application
library. All19 focused engine tests pass through MAIN's two-job/two-thread runner
(`.build-artifacts/continuation/resume-20261102/linux-foundations-first.log`). These are
capability tests, **not integrated lessons or practical objective coverage**.
No host filesystem, package-manager or network execution is introduced.
The retained proposal below describes its original review state; its UNRUN
labels are historical. Native lessons, progress admission and teaching remain
separate gates.

---

# Filesystem capacity accounting — unintegrated foundation

This pure library integration proposal has no registered lab, UI, terminal command, Goal/progress integration or objective credit. Its19prospective regression groups are UNRUN. It is a foundation for later104.2 investigation practice, not delivered203.2 filesystem maintenance.

One supplied virtual filesystem exposes typed block/inode measurements. The profile supplies fixed existing metadata/directory charges. Each modeled ordinary flat file consumes one inode and dense rounded-up data blocks; zero-length files still consume an inode. The model stores file logical sizes, not file contents or an ext4 image. Different fixed charges are explicit facts, not fabricated filesystem estimates. All totals are checked, valid states cannot overcommit, and unprivileged availability subtracts the separately supplied reserved blocks from free space, clamped at zero.

Results include total/used/free/available blocks and exact bytes, inode totals/usage/free, and upward-rounded percentages. A zero percentage denominator is None. A pressure result means a particular resource is exhausted; it does not certify integrity, health, device availability or safety. The numerical percentage equation follows coreutils9.10's nonnegative used+available branch. No `df`, `df -i`, human-readable output, platform report formatting or syscall result is emulated.

The only mutation is a virtual ordinary-file size update or explicitly previewed removal. Retained supplied names and exact sizes cannot change. Size replacement reuses an inode; growth cannot spend the reserved block pool. Previews validate every selected name before cloning. Applying a preview requires the entire current State to match its original snapshot, including unrelated files, all immutable profile facts and generation. A write followed by restoration still makes a preview stale. Failures publish nothing. These rules are simulator retention/transaction policies, not guarantees offered by Linux `rm` or a real filesystem.

Paths are bounded ASCII `/NAME` keys with no nested/traversal components. State decoding bounds string/map allocations, rejects duplicate keys and unknown fields, validates the entire state and limits raw JSON to32768bytes. Private fields prevent bypassing action validation. The source environment must later be compared to the actual held scenario; a structurally valid changed profile is not an authorized imported answer. Nothing here changes existing storage/LVM/partition models or progress.

Absent: directories as mutable objects, permissions/privileged allocation, links, sparse/inline/compressed data, open-unlinked files, quotas, allocation fragmentation, journal/block-group/inode-table layouts, real metadata repair, mounted checks, fsck/e2fsck exit statuses/options, tune2fs mutation, Btrfs/XFS/ZFS/SMART, host filesystem/network/process calls. Supplied fixed charges should never be labeled actual ext4 overhead. Future repair work needs a separately reviewed contract and primary evidence.

Primary references checked2026-10-31: [GNU coreutils9.10 df source](https://raw.githubusercontent.com/coreutils/coreutils/v9.10/src/df.c), installed coreutils9.10 Info df node (compressed SHA6e28a10ae4e4acad5c0147348ffe0a0a0cea4f93ed7cd794d434d953059e4716), [kernel ext4 superblock counts](https://www.kernel.org/doc/html/latest/filesystems/ext4/super.html), e2fsprogs1.47.3 [tune2fs](https://raw.githubusercontent.com/tytso/e2fsprogs/v1.47.3/misc/tune2fs.8.in) and [e2fsck](https://raw.githubusercontent.com/tytso/e2fsprogs/v1.47.3/e2fsck/e2fsck.8.in) manuals. GNU web manual retrieval timed out; the installed manual was read only. Latest kernel docs are mutable, not revision-pinned. Upstream implementations/manual prose are not copied into this original Rust code.

The separately frozen CONTRACT.md defines this scope before code; no utility/reference/model execution occurred during authoring. Future actual tests, primary technical review, independent source review and learner content/native/progress gates remain required.

Current201 integration is source-only and held pending Root full-suite completion, final parent reconciliation and actual core/QA gates. The core is byte-exact the independently reviewed foundation; the test header now imports the library export while all19 regression bodies stay exact. No usage command, learner workspace, filesystem maintenance or104.2 coverage is delivered by this export.
