# Workload-aware disk layout

Development batch2026-11-11: two registered LPIC-1 assignments in **Plan disk roles and future growth**, after **Disk preparation**. Source396/152; packaged acceptance is
pending. Existing394 scenario/151 unit values and optional-absent Plan wire data
remain exact. No legacy answers or new completion credit are fabricated.

- **Leave room for growing logs** (`lpic1-layout-log-growth`): retain an archive,
  choose an aligned log extent, initialize actual ext4, and reserve at least16MiB
  of unallocated space immediately after it. Total free space elsewhere fails.
- **Plan root, logs and swap across supplied media** (`lpic1-layout-media-policy`):
  preserve a startup reference and file; allocate differently sized roles with
  actual filesystem/swap signatures; place root/logs on different disks and reserve
  adjacent log growth space. Either eligible media assignment and equivalent
  valid extents/order/UUIDs pass.

Introductions teach role selection, sector/MiB arithmetic, inclusive boundaries,
existing allocations, adjacent versus total free space and separate hints/content.
Worked tables use different practice media/values. Practice fills drafts only;
Add/Initialize/Check perform actual bounded model actions. A scratch allocation
consumes growth room; Reset practice recovers without touching assignment work.
The independent checklist gives constraints, never a solved allocation.

The optional version1 layout plan adds at most four typed constraints to the
existing virtual-GPT model. It validates names, unknown/duplicate rules, bounds,
actual unique targets and the supplied state. A growth window stops at the next
partition or reserved GPT tail. Separate-disks compares actual media identities.
No subprocess, disk/network access, resize, delete, mount, swap activation or boot
installation is introduced. Small synthetic capacities teach geometry, not real
production sizing. Separate fixed filesystems already isolate ordinary file
allocations on a shared disk; the additional media policy concerns independently
managed replacement/growth pools, not measured performance or failure domains.

## References and coverage

Checked2026-11-11: [LPI current101/102 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
still identify version5.0 and101-500/102-500. The102.1 slice is allocation and
workload-aware planning only. Hardware-specific /boot/ESP, a complete mounted
hierarchy and LVM design decisions remain outside this unit; basic LVM has its
own existing practice and is not reassigned full102.1 credit here.

[Debian's Parted3.6-5 manual](https://manpages.debian.org/trixie/parted/parted.8.en.html)
(source updated2025-02-23; checked2026-11-11) distinguishes partition-boundary
changes from resizing their contained filesystem. GNU manual endpoints timed out;
no successful access or real-Parted comparison is claimed. The existing modeled
512-byte-sector/GPT reserved geometry is disclosed; this is a configuration
workspace, not a claim to implement Parted/fdisk command syntax.

## Development evidence

Passed39 focused/retained integration checks, seven UI/capture/current-catalog
checks,1,069 Python checks, all-target Clippy and supported coverage/inventory.
No inclusive Rust runtime suite was repeated. Native221actions passed; both
complete assignments, separate practice, growth failure/reset, saved readback,
100/150dark and946×929/200Light screens were inspected. Final package flow also
captures Introduction worked tables. One fixture originally clicked outside the
popup clip; its correction and regression preserve bounded visibility/timing.

A multi-target compile was stopped by the memory guard atfullPSIavg10=6.59%, with
no OOM increase. Recovered passive observations and subsequent single-target
checks are retained. All work remains under MAIN's lock/two-job/two-thread/disk
workflow; no global settings or other applications were changed.

Separate396 package verification remains pending. Latest verified package is394R2;
original installation/progress are untouched. Layout-specific two-process reopening,
novice, screen-reader, physical-device and other-OS acceptance remain unverified.
Private evidence: `.build-artifacts/continuation/resume-20261111/next-layout/`.

Representative development images (actual native screenshots, not mockups):

- [Introduction](../.build-artifacts/native-runs/run-nrpydtq9/screenshots/workspaces/layout-1-introduction.png)
- [Separate practice](../.build-artifacts/native-runs/run-nrpydtq9/screenshots/workspaces/layout-1-practice-initial.png)
- [Independent assignment](../.build-artifacts/native-runs/run-nrpydtq9/screenshots/workspaces/layout-0-assignment.png)
- [Wrong adjacent space](../.build-artifacts/native-runs/run-nrpydtq9/screenshots/workspaces/layout-0-wrong-adjacency.png)
- [Light200% fields](../.build-artifacts/native-runs/run-nrpydtq9/screenshots/workspaces/layout-1-light200-fields.png)

## PackagedR1 and visual follow-up

R1passed223installed layout and85retained JSON actions, private394upgrade/reinstall,
NFS two-process reopening and exact selected-input readback. Its actual images
revealed long worked-value rows and a partially shown failure message. The original
technical receipt/images are retained. Only new presentation values and capture
targets changed. A real glyph/clip regression now passes every value at normal
and enlarged widths; all six layout UI/capture tests pass, alongside prior two
normal-catalog tests,1,069 Python checks and all-target Clippy. No grade or source
geometry changed; the prior39integration checks are reused. R2package/visual
admission remains pending; R1is not relabeled as complete visual acceptance.

## Verified packagedR2

R2passed installed223layout/85retainedJSON actions, private R1upgrade/reinstall,
NFS two-process reopening and selected-input readback. Both complete worked tables
and the actual wrong-adjacency/scratch failure rows were inspected; arithmetic no
longer needs horizontal scrolling. Narrow200Light fields/feedback stay readable.
R1technical evidence is preserved. Layout-specific cold reopening and independent
novice/screen-reader/physical-device/other-OS acceptance remain unverified.

Launch:`/path/to/dlms_learning_lab/dist/teaching-review-20261111T075134Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab`.
Receipt:`.build-artifacts/continuation/resume-20261111/next-layout/candidate-r2/r2-verified.json`.
Actual installed screenshots: `.build-artifacts/native-runs/run-hkbi5ddp/screenshots/workspaces`.
Original app/progress remain protected. No curriculum-completion or release claim.
