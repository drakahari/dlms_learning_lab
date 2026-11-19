# Command-location diagnosis: implementation gate

2026-11-05. Source catalog now registers **288 labs /91 units**; the latest verified review
candidate still contains287/90. The new lesson is not packaged yet. This is a bounded LPIC-1 5.0 objective104.7 slice, not complete
command discovery or exam coverage.

## Learning assignment

The new lesson follows ordered PATH and file traversal/permission lessons.
A supplied preferred command file exists but lacks owner execute. The learner
compares first-match `which` with all-match `type -a -p`, inspects `ls -l`, makes a
narrow permission repair, then verifies the preferred command and a neighboring
audit tool. The PATH, fallback and all unrelated state must remain intact.

Introduction teaches search order versus eligibility, permission letters, symbolic
owner-only changes and the simulation boundary. Six worked commands use separate
names and paths. Guided practice removes and restores execute to observe why the
selected result changes. Independent work supplies the required outcome, without
populating a repair command. Numeric and symbolic equivalent repairs are accepted;
a broad permission grant fails preservation even when the first path is right.

## Validation gates

- Core16 query/source/permission/bound groups pass.
- Tree/engine17 and source/practice/persistence6 groups pass.
- Retained filesystem, access, content and terminal-teaching gates pass.
- All-target ClippyR12 passes22.08s. PythonR16:766 tests pass12.148s.
- Fixed probe builds with651 compiled input bindings. Prepare launches nothing.
- Independent review caught a Btrfs directory-link-count assumption before capture.
  Fixed collector binds original held file/directory identities instead of assuming
  an empty directory always has two links. A new directory regression passes.
- R19 observes18 fixed queries across nine owner-permission/search states using
  retained Bash/which ELF descriptors, no startup files and generated contained
  PATH/HOME. Inert artifact contents never execute; before/after snapshots match.
  Independent exact raw readback passes; R20 actual model comparison matches all18
  query outcomes. Original source/ELFs/probe are retained under the reviewed control.
- R21 authored6/source-workspace6/prior-account11 and R22 capture/route2 groups pass.
  Three legitimate methods, wrong/excess permissions, retry, invalidation, separate
  practice, disk save/load/reset and preserved original catalog values are covered.
  R25 actual native54actions/8PNG pass: real command input/Enter, separate
  practice, wrong/excess permissions, alternate repair, invalidation, reset, disk
  save/read, Challenge and narrow150light/200dark typing. Introduction, guided
  practice, assignment, specific feedback and enlarged controls were opened for
  visual review. That exposed a nonexistent Check example instruction; final
  prose now asks learners to repeat searches. Candidate validation remains pending.
  Do not infer those from the pure comparison or from shared rendering.

Private original evidence: `.build-artifacts/reference-runs/command-location-r1`
contains the superseded prepare-only control; `command-location-r2` contains the
corrected control and actual host observations. Both are retained, never rewritten.
No Script evaluator revision or old reference collector is changed.

## Limits

Only supplied regular files, ordinary-user permissions, bounded literal queries
and distinct absolute PATH components. No aliases, builtins, functions, hash state,
whereis, relative/empty/default PATH search or execution of a found artifact.
`type -a -p` intentionally requests an uncached external-file search; `-P` alone
must not be described as always bypassing Bash’s command cache. Found/Missing
simulator results do not claim exact real diagnostic or exit126/127 behavior.

The source-bound optional Tree context keeps old None progress wire data and the
old terminal path. Catalog-aware save/restore binds both assignment and separate
practice to their original source; missing resumed metadata is rejected without
inventing answers. See [subset](COMMAND_LOCATION_SUBSET.md) for bounds and sources.
Windows/macOS/Omarchy, screen-reader, physical-device, full keyboard-only and
independent novice acceptance are not established by these gates.

## Actual development screenshots

- [Introduction](../.build-artifacts/native-runs/run-awsyyn_5/screenshots/workspaces/command-location-introduction.png)
- [Guided search results](../.build-artifacts/native-runs/run-awsyyn_5/screenshots/workspaces/command-location-guided-results.png)
- [Independent assignment](../.build-artifacts/native-runs/run-awsyyn_5/screenshots/workspaces/command-location-assignment.png)
- [Excess permission feedback](../.build-artifacts/native-runs/run-awsyyn_5/screenshots/workspaces/command-location-excess-permissions-feedback.png)
- [Light150 terminal](../.build-artifacts/native-runs/run-awsyyn_5/screenshots/workspaces/command-location-light150-terminal.png)
- [Dark200 terminal](../.build-artifacts/native-runs/run-awsyyn_5/screenshots/workspaces/command-location-dark200-terminal.png)

These development screenshots precede the final control-name/plain-language prose
correction. They do not substitute for a final packaged screen check. New-lesson
cold-process reopening and independent novice comprehension remain unverified.

## Final source review and gates

Independent source/content review finds no remaining narrow blocker. It also
clarified that the audit query is typed into the terminal, since it is not in the
six worked picker commands. Original comparison manifests retain the exact old
source identity; the later source-free-draft guard and foreign-State guard fixes
are reviewed separately. Reference-review manifest SHA256
`00c9df3f85e54440a46e6a114717df6fcf905b778962422c7947a4e1fbdb4507`;
content-review manifest `591e7e71e055050c7dcc7c83216e218fc53c01294dae5d30cbfe759d11ac7ef6`.
Both are private ignored receipts, not owner/novice acceptance.

R28 final authored6 and retained map7 checks pass. R30 Python768 passes12.152s,
current coverage288/91/0drafts and inventory283updated+5alreadymeets pass. Source
ledger adds only two partial term slices (which/type) and narrows104.7 knowledge/
parent wording; no full objective closes. Total1240overlapping issuer rows remain
216partial/7supporting/1017Open. Native/candidate and platform boundaries above
remain explicit. Whole-inclusive Rust/UI was not rerun for this focused batch.

## Packaged Fedora verification (post-build note)

Candidate `teaching-review-20261105T203001Z` is verified288/91. Binary
`b94b9796564e4d88e813dcf3aa4bec7acd50cd775a117d84a1d3e167e0f8450d`, archive
`1dd8131c01be60768c0a9cbbbc02b464d0883a49d293105f83aed5015280ad83`, manifest
`975f983547eaddc68d31ba97285f200f88100c422f4cbd923e1b6878fcd1afe9`.
All589source/data and265bundled documents matched before these later notes;
archive/bundled earlier documentation remain unchanged. R34content passes;
R35actual packaged54actions/8PNG passes. Private287→288installation/upgrade,
desktop metadata and clean uninstall/reinstall preserve exact synthetic progress.
Installed NFS two-process save/full-exit/load/repair also passes (73.551s writer,
31.448s reader, bothexit0); it does not prove this lesson’s own cold-process flow.

Actual final packaged captures inspected:

- [Corrected Introduction](../.build-artifacts/native-runs/run-dsm7etr6/screenshots/workspaces/command-location-introduction.png)
- [Guided practice and precise controls](../.build-artifacts/native-runs/run-dsm7etr6/screenshots/workspaces/command-location-guided-results.png)
- [Independent assignment](../.build-artifacts/native-runs/run-dsm7etr6/screenshots/workspaces/command-location-assignment.png)
- [Specific excessive-permission feedback](../.build-artifacts/native-runs/run-dsm7etr6/screenshots/workspaces/command-location-excess-permissions-feedback.png)
- [Light150 terminal](../.build-artifacts/native-runs/run-dsm7etr6/screenshots/workspaces/command-location-light150-terminal.png)
- [Dark200 terminal](../.build-artifacts/native-runs/run-dsm7etr6/screenshots/workspaces/command-location-dark200-terminal.png)

Navigation: LPIC-1 → Diagnose command discovery and permissions → Diagnose a
command skipped during search. Search for **command skipped** also finds it.
Launch the protected executable listed in CHECKPOINT.md without flags. The
installed owner application and original progress are unchanged; no public release.
