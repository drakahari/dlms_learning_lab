# All-track presentation rollout: Fedora review evidence

Completed2026-11-01 local /2026-11-02 UTC. Stop for owner review. This is the
approved instructional-presentation rollout, not curriculum completion or an
installer release. Owner review used **Show worked answer**, so independent
novice-comprehension acceptance is explicitly **not established**.

## Scope and inventory

Actual review catalog:205 labs across55 units. No other tracks are registered.

| Track | Labs | Updated | Already meets | Pending/blocked |
| --- | ---: | ---: | ---: | ---: |
| CCNA | 61 | 56 | 5 | 0 |
| LPIC-1 | 44 | 44 | 0 | 0 |
| LPIC-2 | 36 | 36 | 0 | 0 |
| Bash/Shell | 64 | 64 | 0 | 0 |
| Total | 205 | 200 | 5 | 0 |

The five exceptions are the accepted existing IPv4 workspace and four accepted
magic-number pilot lessons; all receive the shared aligned-range correction where
applicable. Four magic lessons are inherited from the preceding pilot, not new
curriculum in this rollout. Historical normal registration remains201/54; this
review includes the accepted205/55 catalog. See the complete
[per-lab inventory](TEACHING_INVENTORY.md) and [CSV](TEACHING_INVENTORY.csv).
Every entry has concrete Introduction, Try example and Assignment review notes,
not simply a renderer-inheritance assertion. Shared components and immutable
per-lab presentation data implement the [approved standard](TEACHING_PRESENTATION.md).

Original scenario objectives, graders, intentional starter errors, supported
simulators, progress schemas and IDs were preserved in this rollout. Requirements
retain every original non-whitespace character; selective spacing changes make
quantities readable. No substantive grading/curriculum change is introduced.
Original longer explanations remain accessible as optional detail. Workspaces
remain subject-specific: terminals, scripts, configuration, topology and tables.

## Separate candidate and protection

Built from main with `python3 tools/dev.py candidate --presentation-review`.
HEAD remains `6b54e34869e031e50d6cf0854cce3402f65c6fd4` with preserved uncommitted
prior work plus this rollout. Nothing committed, pushed, merged or installed.

Launch directly, from any current directory:

```sh
/path/to/dlms_learning_lab/dist/teaching-review-20261102T022523Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab
```

Binary SHA256:
`bed5ca427dd0a8347c48c5a182766b7b9ae01139a80e57ff07a38f5861ddf8ee`.
Sibling `.tar.gz` SHA256:
`ff34433b46054450b0167196a1560a5b70f26da603b56e05c5889ff8fb6cdd31`.
Binary matches BUILD_INFO; all319 source pins match the current compiled sources.
The final evidence links were added to source documentation after packaging;
archive and build-time documentation remain immutable.

The review build defaults to its separate `ccna-presentation-pilot` progress and
preferences directory, reusing earlier review work rather than resetting it.
The installed profile is not a write destination. Do not run install.sh for this
review. Both original installed data and prior candidate artifacts remain intact.

## Focused verification

Private logs: `.build-artifacts/teaching-rollout/`.

| Check | Actual result / evidence |
| --- | --- |
| New rollout Rust checks | 11 passed; `focused-refined.log` |
| Retained CCNA pilot checks | 10 passed; `ccna-regressions.log` |
| Retained NFS Save/scrollbar/reflow regression | 1 passed; `nfs-layout-regression.log` |
| Python packaging/inventory/resource checks | 12 passed; `python-checks.log` |
| Production Clippy, library and binary, warnings denied | Passed; `clippy-final.log` |
| Inventory matches actual registered review catalog and original requirements | Passed; `inventory-final-check.log` |
| Packaged binary content validation |205 labs/55 units; `candidate-content.log` |
| Packaged native capture | Exit0; `TEACHING_ROLLOUT_PREVIEWS_COMPLETE`; `candidate-native.log` |
| Diff whitespace and binary/source identity | Passed |

The22 distinct focused Rust tests include actual navigation/rendering of all615
learning views. The saved painted-text receipt is `rendered-views.json`. Every
assignment State is unchanged except the known view selector stored within four
session models; answers, simulations and grades remain exact. Tests cover source
requirements, expected-versus-actual labeling, assignment answer hiding, dynamic
subnet guidance/stale feedback, focus/editing, separate practice drafts, equal
range geometry, font-measured worked-script height, and NFS Save-row stability.

Early test failures exposed an egui font-lock reentry, insufficient small-script
viewport height and generic experiment observations. All were fixed before the
final passing checks/build. Failed diagnostics and earlier captures are retained,
not relabeled as passes. Existing engine/progress evidence is reused: no full
engine-suite repetition or new manual grading walkthrough for every lab is claimed.

## Actual native screen review

Final **packaged** screenshots,48 PNGs:
`.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/`.
These are actual Fedora native captures, not mockups or development-binary images.

At Navy100%,1920x1080, all three screens captured for:
`ccna-magic-blocks`, `ccna-vlan-trunk`, `ccna-ospf-directed-ecmp`,
`lpic1-file-navigation`, `lpic1-account-onboarding`, `lpic2-service`,
`lpic2-nfs-reference`, `shell-functions`.

At Light200%,946x929, Try example and Assignment were additionally captured at
multiple scroll positions for magic blocks, file navigation, NFS and functions.
The narrow logical viewport is411.3x403.9; content scrolls rather than shrinking
text. Wide logical geometry is1669.6x939.1. This session's native screenshots do
not establish an additional150% native run; retained focused tests exercise
100/150/200% reflow and keyboard behavior.

Representative links:

- [Subnet Introduction: equal-width ranges](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-ccna-magic-blocks-introduction-dark-100.png)
- [VLAN guided practice and aligned configuration](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-ccna-vlan-trunk-example-dark-100.png)
- [Terminal guided practice](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-lpic1-file-navigation-example-dark-100.png)
- [Service diagnosis and configuration](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-lpic2-service-example-dark-100.png)
- [NFS editor and Save alignment](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-lpic2-nfs-reference-example-dark-100.png)
- [Script Introduction and full small worked example](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-shell-functions-introduction-dark-100.png)
- [Independent script Assignment](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-shell-functions-assignment-dark-100.png)
- [Light200% practice: actual result versus expected preview](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-shell-functions-example-light-200-scrolled.png)
- [Light200% NFS controls and scrolling](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-lpic2-nfs-reference-example-light-200-scrolled.png)

Native review confirmed aligned blocks, readable left-aligned teaching prose,
visible worked code, subject-specific guidance, preserved starters and scrolling
controls. These representative visual checks supplement per-lab content and
all-view rendering; they are not205 manual end-to-end learner sessions. Long
literal commands/configuration can scroll horizontally to preserve their actual
line boundaries. Longer introductions scroll intentionally.

## Resource safeguards and remaining acceptance

All heavy work used MAIN's shared lock, two Cargo jobs, two Rust runtime threads
and disk-backed project artifacts. Candidate build's170 memory samples show
minimum available17.01GiB, PSI some/full avg10 peaks0, no OOM-counter increase,
54 swap-in pages and zero swap-out pages. Sampler:
`.build-artifacts/logs/20261102T022523.958754Z-candidate-memory.jsonl`.
No host settings, other apps, services or model were changed/interrupted.

No implementation blocker remains for this presentation handoff. Still pending:
owner review of this candidate, independent novice comprehension, screen-reader
and physical-device accessibility acceptance, and actual Windows/macOS runtime.
Themes are preserved; this pass sampled Navy/Light, not every theme individually.
No release publication, installed-app replacement, cross-platform completion,
certification-completion or independent mastery claim is made.
