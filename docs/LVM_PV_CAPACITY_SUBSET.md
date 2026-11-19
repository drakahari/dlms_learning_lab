# Recognized PV capacity in virtual storage

This optional profile separates an immutable supplied physical device from the
PV size recognized inside it. Existing321source scenarios keep their absent fields,
original behavior and progress. All execution is bounded and in-process; no device,
partition, filesystem, kernel, mount, process or LVM executable is accessed.

| Supported form | Modeled effect |
|---|---|
| `pvresize /dev/NAME...` | Recognize the full size of each named supplied labeled PV's device. |
| `pvresize --setphysicalvolumesize SIZE[M\|G] /dev/NAME...` | Set its recognized end within the unchanged physical device. |
| `pvs`, `vgs`, `lvs` | Fixed simulator reports derived from current recognized capacity and actual virtual allocations. |

Use1–8distinct supplied flat device paths. Explicit sizes are positive integer
binary M/G, case-insensitive; suffixless size means MiB. Fractions, other units,
flags, force, partition editing/rescanning, relocation/pvmove, real metadata/device
discovery, snapshots, kernel/mount operations and host shell expressions are
unsupported. Existing disclosed pvcreate/vgextend/lvextend/resize2fs growth forms
remain. This first profile is disjoint from the optional provisioning, retirement,
availability and naming profiles; it does not claim all combinations work.

The exercise reserves1MiB before data. Recognized whole PE count is
floor((recognized bytes−reserved start)/group extent bytes). Real LVM metadata
arrangements can differ. Reports use whole-extent data bytes for grouped PVs;
the native supplied facts distinguish total device and recognized PV sizes.
Highest allocated PE is a derived simulator fact, not a new real pvs column.
Physical indexes begin at zero. Every original allocated position must remain
below the proposed PE count; a large free-space total does not prove placement.

A PV end change does not resize the physical device, LV or filesystem. Growth
requires the appropriate later layer changes; a reduced end cannot discard an
allocated extent. Each named set is checked before mutation and a validated
candidate is published. Real multi-PV commands may have partial effects; this is
neither real LVM rollback assurance nor proof that reducing actual storage is safe.

## State, limits and goals

State.pv_capacity and Device.pv_bytes default absent and serialize only when present.
The source-only MetadataSize profile requires a bounded recognized size on every
labeled PV and none on blank/foreign devices. Ends must be sector-aligned, past
the exercise's reserved start and within immutable device geometry. New metadata
cannot be injected into old source/session goals. Initialization/joining maintains
recognized sizes; joining an already labeled PV does not silently refresh it.

Existing8device/4group/16LV/4096PE raw bounds,2MiB raw state,64KiB teaching model,
128mapped teaching extents and512KiB session budgets remain. Oversized commands,
malformed imports, out-of-device ends and unrepresentable extents refuse without
publishing a partial state. No dependency, shell-evaluator or schema-version change.

Optional bounded Target.pv_sizes specifies actual recognized-byte outcomes alongside
existing LV/filesystem requirements. Required ends must be reachable using supported
integer-M/G or supplied-size inputs, or already be the source size. Duplicate,
foreign/wrong-group, out-of-bounds and allocation-excluding goals are invalid.
Target.preserve_volume can additionally require the exact original allocation,
filesystem capacity and files; it is restricted to the new explicit PV goals.
Old targets omit both fields unchanged. Unrelated device signatures, recognized
sizes, geometry and memberships remain protected. Checked flags, walkthrough
position and transcript strings never grant credit; current state is regraded.

## References and validation boundary

Checked2026-11-07: [upstream pvresize(8)](https://man7.org/linux/man-pages/man8/pvresize.8.html)
and the current [LPI201/202objectives, version4.5](https://www.lpi.org/our-certifications/exam-201-202-objectives/).
The existing growth subset references define selected lvextend/resize2fs forms.
No actual LVM/device conformance was run.204.3remains partial: snapshots/COW,
general LV/filesystem/device reduction/removal, real mapper/mount/configuration
and broader utility behavior are still unsupported.
