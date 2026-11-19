Implemented and scoped Fedora-review verified2026-11-08. See
[lesson evidence](RAID_ASSEMBLY_LESSONS.md); the original preimplementation
contract below remains the explicit subset, not full204.1 or kernel fidelity.

# Clean RAID1 assembly: bounded capability contract

Specified2026-11-08 before implementation or curriculum credit. This extends the
existing204.1 roadmap, not the old recovery engine or its saved-state format.

Supplied, clean, fully synchronized two-member mirrors only. Reuse the existing
RAID1 model for actual distinct replica/sample-file content and capacity. A separate
source-held metadata table supplies unique128-bit array UUIDs and member roles.
It is an abstract teaching representation, not a superblock decoder or disk image.
A separate active binding selects an existing identity at an admitted `/dev/mdN`;
saving configuration cannot create, copy, erase, assemble or rename disk metadata.

The initial intended lesson will inspect metadata, repair a saved ARRAY/DEVICE
policy, assemble existing members, inspect `/proc/mdstat` and preserve a sibling.
It must assess current identity/data and a new targeted assembly from saved policy
independently: a successful manual assembly alone does not prove saved policy.
No lab is counted until source, teaching, native workspace and grading are admitted.

## First supported subset

One4KiB/64line virtual `/etc/mdadm.conf`; up to2ARRAY rows and8literal DEVICE
paths. DEVICE defaults to all supplied components when absent, within this closed
inventory. Support case-insensitive full/three-letter DEVICE/ARRAY keywords,
whitespace and word-start inline comments. Require one explicit UUID identity per
ARRAY row. No quoted/continued words, wildcard discovery, arbitrary config paths,
additional tags, include directories, HOMEHOST/AUTO policy or network discovery.
Reject unsupported constructs honestly, rather than approximate their meaning.

Terminal commands will be bounded, explicitly parsed forms of mdadm examine,
detail, clean assembly with both literal members and UUID, targeted assembly scan,
stop, and cat of the virtual configuration or proc status. No unqualified scan:
real `mdadm --assemble --scan` can discover unused arrays when no ARRAY records
exist. This activity does not model that fallback or boot/initramfs behavior.
Stopping only affects the supplied unused simulated array; real mounted/busy
arrays need operational care beyond this closed exercise.

Render actual active clean membership in a small mdstat subset, including `[2/2]`
and `[UU]`. Member descriptor numbers are supplied by this model and happen to
match slots here; do not teach that as a universal rule. Omit invented times,
major/minor numbers, event counters, speed and progress. Actual sample data is
shown through a clearly labeled native simulated data view, not a fictitious
filesystem mounted on the host.

No degraded/dirty assembly, stale event arbitration, missing metadata, force,
create/grow/reshape, RAID0/5, recovery timing, real devices, host mdadm/kernel calls,
boot discovery or on-disk metadata changes. Existing recovery lessons remain
unchanged. No new dependency. All work in-process with bounded files/history and
source-bound replay for progress. No complete204.1 or exam-preparation claim.

## References and evidence

Checked2026-11-08: [mdadm(8)](https://man7.org/linux/man-pages/man8/mdadm.8.html),
[mdadm.conf(5)](https://man7.org/linux/man-pages/man5/mdadm.conf.5.html),
and [Linux MD administration](https://docs.kernel.org/admin-guide/md.html).
The manuals are upstream project documentation mirrored by man7. The pinned
mdadm4.6 raw GitHub manual sources referenced in RAID1_SUBSET.md were unavailable
through the browser cache today; do not claim that revision was newly inspected.
No upstream implementation or manual passages are copied into the MIT source.
This is reference-informed modeling, not a privileged device/reference comparison.
Kernel type0xFD boot autodetection is legacy-superblock-specific; it remains Open,
not a prerequisite or general modern metadata assembly fix.
