# Logical Debian packages — backend stage1 source proposal

Source-only,2026-10-25. The accepted pure core is commit2b777f6. This app adapter,
teaching metadata, technical fixture and20 prospective tests have not compiled
or run. It adds no native feature, registered lesson, unit, objective promotion,
package-manager alias or host operation. Existing normal125 data is a pinned
prerequisite source snapshot; its acceptance belongs to separate milestones.

## Native architecture and explicit boundaries

`State::DebianPackages { simulation }` contains the accepted pure state;
`Goal::DebianPackages { definition }` contains a fixed initial state and explicit
current requirements. The complete immutable Catalog pins architecture,
repository availability, published and known snapshot identities, packages,
artifacts and the virtual path universe. Pack validation requires the Goal initial
to equal the Scenario initial exactly. Current grading and catalog-aware progress
decode repeat that supplied-initial equality and current Catalog pin.

This is a logical restricted **script-free** operation model, not real dpkg/APT
transactions or rollback. No `dpkg -i`, apt, rpm, yum or dnf command alias is added.
Unpacked/configure/half-installed states, maintainer scripts, full solver/provider/
alternative policy and real archives remain unsupported as described in
DEBIAN_PACKAGES_SUBSET.md. The adapter does not bridge this model to another
package manager or claim package authentication, system administration or practical
102.4/102.5 coverage.

The only string mutation prefix is `packages {typed Action JSON}`, exclusively an
authoring reachability representation. It is rejected by context-free engine
execution. A valid fixed Goal and same Catalog are required for every mutation.
It is not learner syntax or help text to type. Logical typed helpers expose
Refresh index, Install, Reinstall, Remove and Purge. An admitted candidate is
published only after the pure action and app size checks succeed. Errors preserve
the exact original current state. No source/publication/history APIs call the OS.

## Current outcomes and explicit preservation

Definition has1–32 unique requirements and at least one positive Installed outcome.
Checks derive from the same actual current state on every call:

- Installed with optional semantic version constraint; RetainedConfig with
  optional version; and NoRecord in the package database.
- PayloadIntact compares ordinary payload text for the installed artifact.
  Configuration is deliberately separate; explicit FileMatches, FileAbsent and
  FilePreserved predicates express its required content/presence/absence.
- DependenciesSatisfied checks all actual installed Depends.
- PackagePreserved pins the named original database record, not every file or
  cache entry. FilePreserved separately compares named current text or absence.
- Cached names an exact supplied metadata snapshot or no snapshot. ArchiveAvailable
  checks local artifact availability; these do not imply installed state.

There is no hidden whole-Runtime equality or action-history credential. Each task
must explicitly name its preservation requirements. NoRecord does not imply that
arbitrary unmanaged files disappear; the task can add explicit FileAbsent rows.
Queries and proposed operations are not completion evidence. Partial feedback
comes from individually evaluated predicates; no cached Check/inspection result
is accepted. A previously passing state that loses a required record, file or
integrity outcome immediately loses current credit; a legitimate repair can
regain it without requiring a particular action sequence.

## Read-only typed queries and saved drafts

`query` returns typed Inventory, CachedCandidate, LocalArchive, RecordedOwner,
Payload findings or Dependencies findings. No fake real-utility output/status is
invented. Owner information comes from database manifests even when the file is
missing. Cached candidate metadata is distinct from locally available artifact
metadata/payload. A missing local artifact produces a query error rather than
pretending it has been downloaded. All queries are read-only and produce at most
32KiB of compact result data; oversized results are rejected, not truncated. Local
artifact result size is checked on a borrowed representation before cloning it.

Defaulted/omitted workspace1 fields are `debian_packages_draft` and
`debian_packages_learning`; Scenario's optional field is
`debian_packages_teaching`. Draft version1 records the immutable Catalog plus a
logical operation, single target, at most8 multi-install requests, query kind and
raw query argument. Names/version/path fields allow incomplete/malformed bounded
text for exact save/resume. Performing an action parses its relevant fields and
repeats core syntax validation. Fill inserts operation fields only, never executes.

These fields propose an operation or query; they are not saved package
configuration. Editing them therefore does **not** invalidate an otherwise
truthful current package outcome. Missing old metadata likewise adds no pending
state. Operations change the simulation explicitly; successful or rejected
queries cannot award credit. There is no transcript/last-response grade authority.

## Separate teaching and fixture pinning

Optional Lesson contains a skippable Introduction, independent practice definition,
annotated logical action list, an explicit changed experiment with boolean outcomes,
and a repair plan. Pack validation actually replays those bounded pure action lists
and requires worked/repair success and the declared failing experiment rows.
This is model reachability, not native/runtime/novice acceptance.

Practice uses a disjoint package/artifact/index/path Catalog. Its Learning stores
one immutable original State marker, current simulation and separate draft. The
marker must equal the exact authored initial State, even if another lesson shares
a Catalog but starts from different current files. That marker is never used for
grading. Legitimate evolving values may coincide with assignment values; immutable
identities, not coincidental current text, establish origin. A practice's changed
current files can be wrong without making its save invalid.

New practice starts in Introduction; preparing an older attempt with no optional
metadata chooses Assignment and preserves applied state. Preparation rejects a
foreign State kind, overlapping assignment/practice catalogs, or an existing
practice from another immutable lesson before publishing metadata. Reset can replace only
the selected Attempt through the existing API. No mastery migration, new profile
schema or shell evaluator bump is introduced. Existing genuine6/7/8 observations,
receipts, favorites and unrelated attempts are preserved; none is regenerated or
relabeled. No UI launch preparation or native controls are included in this stage.

## Admission and resource limits

The core's declared maximum is intentionally larger than this app's admission:

| Item | Adapter bound |
| --- | --- |
| Admitted simulation / immutable draft Catalog | 256KiB compact JSON each |
| Definition / Draft | 512KiB /384KiB compact JSON |
| Lesson / Learning | 1MiB each |
| Requirements / example actions / experiment / repair | 32 each; concepts8 |
| Raw package / version / query argument | 64 /128 /160 UTF8 bytes |
| FileMatches content / error | 2048 /512 UTF8 bytes |
| Internal action / query result | 16KiB /32KiB |
| Outer pack / progress | existing2,000,000 /16,000,000 bytes, unchanged |

Direct objects are checked for counts/scalars before draft cloning/replay;
bounded serde visitors reject excessive sequence entries before constructing an
extra body and reject oversized text before owned copies. Aggregate compact size
uses a counting writer. Input state and catalog bounds precede adapter clones.
The accepted pure action constructs a bounded candidate under its own limits;
the adapter's smaller candidate size check can reject that result without
publishing it. It does not claim to eliminate all transient allocation inside the
pure model or serde_json raw-input scratch. The outer existing file caps remain
in force, so the Cartesian product of all per-field maxima need not be admitted.
A legal maximum pure State may be refused by app State/Draft/Definition admission.
No global cap is raised to make that state fit.

## Prospective tests and handoff

The original source proposal contained twenty tests covering reference and lifecycle-equivalent repairs, partial/
query-only noncredit, typed query distinctions, real-command rejection, each
explicit predicate's own negative row, retained configs/no record/absent files,
fresh success-break-repair, malformed action/query draft retention without a fake
pending gate, isolated fill/experiment/repair, same-Catalog foreign initial markers,
foreign Catalog/Goal/practice and wrong-kind metadata, optional old omission,
original125 specification/31-unit preservation with actual old histories, real
save/export/inspect/restore (full selected Attempt, distinctive current practice
and malformed separate draft compared before reset) and selected reset isolation,
direct10,000-action
rejection, strict JSON/positive goal bounds, escaped State/Learning/real-profile
roundtrips, pure-legal/app-oversize refusal, pre-trim input admission, direct-field
checks before aggregate serialization, and atomic wrong-kind/foreign-practice
preparation. See the MAIN checkpoint below for subsequent actual execution.

First gate requires parent/peer source review and an explicit sole-heavy-slot
grant. Proposed guarded targets: `--lib --test debian_packages
--test debian_packages_workspace --test backup --test shell_substitution_migration
--test shell_case_migration --test shell_getopts_migration`, followed sequentially by
all-target Clippy with warnings denied and format checks. Existing runner supplies
exclusive worktree target, disk TMP and two jobs/two runtime threads. Do not run
host package commands or install dpkg to fabricate reference observations.

The technical fixture is `development-debian-packages`, unbundled and uncounted.
Pinned original125 data/prerequisite registration is kept separate in the ignored
base receipt. Apply only package-specific source hunks to later main; never copy
old pack/unit/UI files over the evolving application.

## MAIN application integration checkpoint

Reviewed backendR3 and terminalR2 were integrated after full source/base hashes
and three-way preservation of newer STP/DHCP code. The source-only original125
metadata baseline predates exactly three M106 completion-explanation paragraphs;
ONLY those three strings were explicitly reconciled to committed125 metadata.
The frozen original is retained. All other initial/Goal/reference/case/unit and
genuine learner-history data remain unchanged; no expected outcomes regenerated.

First guarded gate:68 library+19 pure core+15 terminal PASS;19/20 backend passed.
The remaining read-only test incorrectly expected an initial Installed record to
already be ConfigFiles. Its independent world now performs actual Remove first,
then pins those bytes before read-only queries; initial/Goal unchanged. All20
backend checks PASS on retry. Native workspace/content gates remain separate,
unbundled and uncredited. No host package utility or external oracle was run.

Broader MAIN gate: eight existing progress/backup/workspace/content/history targets
pass after integration (m110-package-progress-regressions-r3); all-target ClippyR3
and formatter pass. One context-guard style correction and an array-only test
allocation correction preserve predicates/expected outcomes. First ordinary gate
observed min19.48GiB available, max memory PSIavg10 some/full0.3/0.3, no OOM
increase; no sustained pressure or extra heavy invocation. No stress test.
