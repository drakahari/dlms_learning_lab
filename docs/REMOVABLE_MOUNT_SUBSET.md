# Optional-media mount simulation subset

2026-11-02: source248/69 includes one authored removable-storage capstone.
The earlier verified247/T214012Z package predates it. See
[lesson evidence](REMOVABLE_MOUNT_LESSON.md) for actual source/native/package gates.
The original storage core remains unchanged; this is a narrow104.3 slice.

## Declared scope before authoring

`removable_mount` reuses original `storage` identity/parsing/mount state. It adds
only exact `nofail` option handling and an explicit supplied present-media set.
Limits:16KiB input,128lines,8entries/possible volumes and16options per entry.
Six plain fields, whole-line comments, ext4/xfs, exact supplied lowercase UUIDs or
literal virtual `/dev/` aliases, fixed non-nested mountpoints and the prior
ro/rw/auto/noauto/defaults behavior remain. Unknown options and control characters
refuse. A bounded normalized representation delegates the old parser; it never
truncates a field to fit. No file path is opened and no device is queried.

The separate systemd-style local-filesystem dependency projection distinguishes
Required, Wanted and Not enrolled. Automatic entries without nofail are Required
and ordered before the modeled target; nofail entries are Wanted and not ordered
before it. noauto is not enrolled by this table. Another unit pulling it in is
outside this model. Actual finite mount-attempt outcomes remain separate: an
optional failed mount stays visibly failed. Success of the Required attempts is
not actual boot completion, a timing guarantee or proof optional media is ready.

Explicit manual requests can mount a noauto entry; absence still fails despite
nofail. The model rejects hot removal of a mounted medium. A wrong UUID can look
like missing optional media: dependent grading must check intended mounted volume,
policy and read-only state in all-present cases, plus absent-optional and
absent-required contexts. Readiness alone is never adequate grading.

User/users/owner options remain unsupported pending faithful privilege, implicit
noexec/nosuid/nodev and unmount-owner behavior. Labels, escaping, automount,
network/FUSE/stacked/nested mounts, device timeouts, real generators, filesystem
checks and broad104.3 remain Open. dump/pass numbers are stored, not executed.
This is atomic draft syntax validation and finite supplied attempts, not a full
systemd boot or kernel. No real mount, even fake mode, daemon or service was run.

## Actual gates and references

Ten focused foundation groups and all17retained storage groups pass. These cover
missing optional versus required media, visible wrong-type/unknown-UUID failures,
manual/noauto behavior, identity under alias permutation, context and
path refusal, option ordering, unchanged old nofail rejection, limits and no field
truncation. Final scoped gates and actual native evidence are tracked in the lesson record.

Primary references read2026-11-02:

- [LPI5.0/104.3 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/).
- [Upstream util-linux fstab(5), via man7](https://www.man7.org/linux/man-pages/man5/fstab.5.html): mirror identifies2.43.devel-739-eee2e, obtained2026-08-04; not installed-version runtime evidence.
- [Versioned systemd256.16 mount documentation](https://www.freedesktop.org/software/systemd/man/256/systemd.mount.html), especially noauto and nofail. Latest262 was inspected separately; its existence is not Fedora acceptance.

No daemon/kernel/runtime conformance or independent novice study was performed.

## Source-session integration

`removable_workspace` adds bounded separate example/assignment drafts and checked-
current-draft marks, source/context bounds and cooperative Run/Stop. Seven groups
pass against typed synthetic fixtures: valid alternatives, a wrong-UUID draft with
one correctly passing absent-media case and two correctly failing present cases,
all-nofail/noauto/empty/wrong-type mistakes, independent practice credit, save/read,
retry/stale-result invalidation, cancellation and malformed/unsupported/imported
inputs. An actual expected required-storage failure can satisfy its task check;
a parser failure never counts as that expected outcome. No per-case or overall
success is trusted from serialized output; projections are derived again.
Source definitions require known media/targets and separate practice UUIDs. The new RemovableMounts scenario variant is registered with bounded source/context
validation, separate drafts, current-credit replay and versioned local persistence.
