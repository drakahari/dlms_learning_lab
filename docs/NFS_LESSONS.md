# NFS exports, clients and safe handover

Four authored LPIC-2 assignments use native document editors, supplied-client
requests and current-mount observations to teach export policy, publication
permissions and safe handover. They are part of the154-assignment,39-unit source
catalog, following the persistent-mount and Linux file-access units.

Each task retains a full assignment in Learn and Challenge. Learn adds a
skippable introduction, independent editable practice and optional progressive
help; examples cannot complete the assignment. Save, Apply, mount, Read and
Write are separate actions. Typing replacement text never writes a file.
The writing editor starts open when the task requires publication; read-only
exploration remains optional. Selected server/client addresses are supplied beside
the selectors. The optional inventory lists all addresses and numeric profile IDs.

This is an offline model of supplied NFSv3-style clients, existing virtual files
and numeric identities. It performs no host mount, RPC, network access or actual
service change. Supported layers and fidelity limits are documented in
[NFS_WORKSPACE.md](NFS_WORKSPACE.md); this unit is partial practice, not full
LPIC-2 coverage or real-equipment experience.

The four assignments differ in inventory, initial fault, required operation and
preservation obligation. Each has three full assignment stages, three optional
native-action walkthrough entries, progressive hints and a worked explanation.
Introduction teaches essential concepts before testing; the walkthrough is
optional. There are no learner JSON commands. `solution` is only the bounded
internal authoring codec used by the loader and goal-aware engine.

| Task | Initial fault and meaningful outcome | Two supported alternatives |
| --- | --- | --- |
| Restricted reference export | The wrong address is admitted; the reader has a correct saved mount but no live map. Narrow the export, Save/Apply and mount/read the reference. Preserve archive mount, bytes and client layers. Fresh checks deny every other supplied client and prove server ro using an actual rw mount plus Write. | Explicit mount after policy repair; equivalent single-host `/32` selectors and reordered export lines with saved mounting. |
| Publication diagnosis | Server export and held client mount are both ro; file bytes are unpublished. Diagnose both gates and numeric search/file permissions, remount rw and explicitly Write the exact result. | Supplementary-group helper with explicit mount and reordered saves; configured anonymous UID/GID maps the supplied root profile to the owner/group, proving root_squash is not universal anonymous denial. |
| Retired/replacement handover | Old client holds rw work and a saved entry; replacement has neither. Unmount old before incompatible Apply, narrow policy, save replacement and empty retired documents, mount/read new. | Save client documents first then explicit replacement mount; equivalent `/32` policy plus explicit mount in a different supported order. Every other supplied client must be denied admission; a fresh rw replacement mount must also receive export_read_only on Write. |
| Two-share capstone | Work has a held ro mount; reports admits the wrong address; publication bytes are unfinished. Coordinate rw work and ro reports on the publication server while preserving a separate vault server/client/mount/file and original reports bytes. | Client-first saves with explicit mounts and group helper; reordered equivalent selectors and group helper. A fresh rw reports mount must still be denied writing by the export. |

First prerequisite IDs resolve to actual existing lessons: `lpic2-mount-policy`
(Make saved and live mount policies agree) and `lpic1-traversal` (Unlock the
project directory). Later tasks depend on the preceding NFS task. The first
Introduction explains server/client paths, IPv4 literal/CIDR export selectors,
closed export/fstab syntax, default ro and root_squash, separate draft/saved/
applied/live layers, numeric UID/primary/supplementary GID classes, directory
search, client versus export write gates, immutable metadata, and independent
current/fresh Check trials. Later Introductions teach their added diagnosis,
retirement and two-server preservation requirements. Practice fixtures use
studio namespaces, disjoint IDs/addresses/paths and different publication text.

The model remains a supplied NFSv3 AUTH_SYS-style ACCESS/server decision profile,
not a host mount/RPC/service or complete Linux open/credential model. Regular
files are owner-RW and nonexecuting; archive preservation is an outcome, not an
invented immutable-file mode. Numeric inode metadata cannot change. No task asks
for chmod/chown/create. Mapped owner access can succeed; unsupported unsquashed
UID0 is not a denial. Incompatible policy Apply requires affected live mounts
removed as an explicit conservative planner restriction, not a real Linux
file-handle revocation claim. This is a bounded part of LPIC-2 network file
sharing, not full209.2 coverage, novice acceptance, other-OS or packaged acceptance.

`tests/fixtures/nfs-review.json` contains independently authored complete row
vectors and plans: reference, two alternatives, plausible wrong repair, partial,
no-op, read-only, break after success and repair. It also records the independent
practice initial/worked/adverse vectors and explicit preservation requirements.
The expected values were reasoned from declared requirements and policy layers;
they are not recorded evaluator output. Row0 is context; requirement i is row
 i+1; remaining rows are independent fresh trials. Future runtime mismatches
must be investigated and retained, not relabeled as successful observations.

Eleven content-test groups in `tests/nfs_content.rs` use the real loader with actual
prerequisite packs and goal-aware typed outcomes. They check distinct task
shapes, initial/negative/partial vectors, reference/eight alternatives, exact
unrelated data, current invalidation/repair, isolated practice experiments,
fresh trials with no current map/file installation, fault versus denial, extra supplied-reader and server-rw negative controls,
foreign practice/other-task hardcodes, malformed saved-policy atomic Apply, and
whole raw drafts/Session histories/genuine old records through load/backup/
inspection/restore/selected reset. Historical shell inspection rechecks are
compared with an independent old-only inspection control. The independent content loader is checked against the four newly registered
normal scenarios; the exact original147 prefix remains protected. No test uses
exact-string-only grading.

Native routes use the reviewed typed editors and explicit Save exports,
Apply saved exports, Save mount document, Mount saved, optional explicit mount,
Unmount, Read existing and Write existing controls. Fill example fields only
fills; it cannot execute a plan. Default mount controls stay compact; explicit
source/root/mode and detailed inventory can be optional expansions. Full task
remains prominent in Learn and Challenge. Current verification evidence is recorded separately from the authored content;
compiler and event checks alone do not establish native or learner acceptance.

R2 closes two grading gaps in the first and handover tasks. Their four fresh
trials now contain applied positive read, one bounded group of MountExplicit
denials for every other supplied client at a declared target, saved positive
read, and an authorized rw MountExplicit plus Write expecting the exact
ExportReadOnly stage. This finite supplied-client scope is explicit in the
assignment and Introduction; it is not a universal IPv4-security claim. Added
extra-vault-reader and rw-server plans fail independently. The reference task's
rw-server counterexample reaches a later FilePermission denial, which cannot
substitute for ExportReadOnly; the handover counterexample permits writing.
References, supported alternatives, initial States, current requirements and
all preservation clauses remain unchanged. Expected row counts are12/12/14/17;
all revised vectors remain authored expectations, rather than recorded evaluator
outputs. Earlier content gates remain separate from full catalog, native and packaged
acceptance.


## Catalog and verification boundaries

The catalog preserves all original147 assignments and the exact original37 unit
bytes. Three Bash tasks follow that prefix, then these four NFS tasks; the Bash
unit precedes the NFS unit. Closed reconstruction checks both reviewed appends
and their order without changing historical fixtures or learner records.
Existing progress stays compatible, and new activities receive no invented
answers or completion credit.

Content and event checks exercise reference methods, two alternatives per task,
wrong and partial states, invalidation, separate practice, current versus fresh
mount behavior, reset/resume, complete progress and migration receipts. Positive
read captures must show the exact text required by the completed task, including
the replacement client after handover. Native acceptance and artifact identities
are recorded in [IMPLEMENTATION_LEDGER.md](IMPLEMENTATION_LEDGER.md) and
[INSTALLATION.md](INSTALLATION.md); test descriptions alone are not acceptance.

**Installed Fedora candidate9a90f275e133 passes all four NFS flows**,106 captures,
and the40-frame editor/Save sweep at all five text sizes100/115/125/150/200%,
wide/narrow, light/dark. The shared viewport reserves font/style-derived space
before Save rows; horizontal/vertical scrollbar changes no longer shift them.
All40 sweep PNGs and representative full-flow screens were visually reviewed.
Visible supplied IPv4 rows, required Write editors, editing/focus/Undo and existing
Save/Apply behavior are preserved. Lifecycle and exact synthetic legacy-byte
retention pass; see INSTALLATION.md for identities. New-task resume remains full
warm readback, not four cold-process tests. Optional expanded inventory is not
newly visually reviewed. Windows/macOS runtime, screen-reader/physical-device and
novice acceptance remain pending. Previous00d scrollbar evidence is historical.
No public release is published.

### Source continuation: actual cold-process validation — 2026-10-28

The guarded development tool now supports the closed `--case nfs` fixture. Two
normal GUI processes share a newly created private profile: save actual applied
policy/mounts/files plus independent unfinished assignment/example documents,
fully exit, relaunch, compare all raw/portable/provenance/appearance fields, then
repair both drafts through native editors and check their real goals. FinalR3
passes with12 retained captures. It does not access personal progress or host NFS.
The original installed9a90f275e133 candidate predates this fixture; final installed
acceptance must repeat it on the rebuilt curriculum candidate. Exact scopes and
retained failures are in M168 of IMPLEMENTATION_LEDGER.md. This changes validation,
not NFS capability, authored lab count or certification credit.
