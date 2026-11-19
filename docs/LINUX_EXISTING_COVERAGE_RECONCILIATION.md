# Existing Linux practice: narrow reporting reconciliation

Checked2026-11-12 against the actual428/165catalog and current official
[LPIC-1 v5.0](https://www.lpi.org/our-certifications/exam-101-102-objectives/) and
[LPIC-2 v4.5](https://www.lpi.org/our-certifications/exam-201-202-objectives/) pages.
No new labs, curriculum objectives, grading, source states or learner requirements.
The verified428R1package and its retained identity remain unchanged.

| Objective | Actual existing practice | Correct credit and remaining scope |
| --- | --- | --- |
| 201.1 | `lpic1-module-device-driver`, `lpic1-module-index-repair`, `lpic2-module-options-reload`: inspect supplied driver/dependencies, repair lookup, distinguish saved from effective options. | Parent Partial practice for this module-use slice only. Named kernel source/documentation/image/compression references remain Open. No kernel image builds, release/patch selection or real hardware acceptance. |
| 203.2,203.2.K1,203.2.T3 | `lpic1-fs-offline-accounting`, `lpic1-fs-retained-recovery`: investigate and repair typed offline graph consistency with bounded e2fsck/fsck.ext4 syntax, preserve content and sibling, explicitly recover retained unnamed data and restore mounting. | Narrow Partial practice. Full ext2/ext3/ext4 utilities, real ext4 image, Btrfs, XFS, ZFS and SMART remain absent. Neither capacity accounting nor generic service/configuration work earns filesystem-maintenance credit. |
| 109.1 | IPv6 host and transport models now exist elsewhere in the catalog. | The LPIC-1 fundamental-protocol bridge still needs authored progression and broader listed-port/protocol teaching; model availability alone earns no coverage. Replace the stale claim that those models do not exist. |

These corrections credit real prior hands-on interactions, not this documentation.
The per-lab teaching inventory is unchanged. Existing source/content/native evidence
is reused from KERNEL_MODULE_LESSONS.md and FS_INTEGRITY_LESSONS.md; filesystem4/driver5retention and behavior tests plus a separate current-source/outcome
comparison1now pass. Python1169, source guards162literal/171ERE, strict coverage and
teaching inventory pass. The first current comparison omitted prerequisite scenarios;
its fixture now validates the full accepted303catalog and retains every source/grade
assertion. Python first assumed legacy walkthrough arrays; it now verifies the actual
concept/worked/practice/assignment and three review records for all five lessons.
Both failed runs remain failed evidence. No content or grader changed. Full objective preparation, real hardware/recovery experience or
independent novice/physical-device/screen-reader acceptance is not claimed.
