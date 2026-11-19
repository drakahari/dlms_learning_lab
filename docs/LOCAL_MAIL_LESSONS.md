# Local mail aliases and forwarding

Three LPIC-1 assignments follow virtual file and local-account foundations.

| Lab | Practical work |
| --- | --- |
| lpic1-mail-team-alias | Repair a shared alias and preserve administrative and direct delivery |
| lpic1-mail-personal-forward | Forward incoming mail while retaining the required local copy |
| lpic1-mail-index-recovery | Repair a loop and stale alias lookup while preserving forwarding |

Each Introduction teaches its prerequisite concepts and demonstrates a different
worked configuration with expected destinations. Practice uses separate users
and files. Assignment keeps the complete requirements, native editor and sample
results visible. Challenge retains the assignment and hides worked teaching.
The final handover combines aliases, the separate lookup and per-user forwarding.

The native editor exposes only supplied simulated files. Save file and Rebuild
aliases have distinct effects. The delivery test shows each sample's actual
mailboxes and failures, together with its requirement result. Partial results
remain distinct from overall completion. Optional lookup inspection exposes
current state. Worked configurations remain in the Introduction.

The [subset guide](LOCAL_MAIL_SUBSET.md) records supported behavior, exclusions,
bounds, authoring facts and sources. This gives partial LPIC-1 108.3 practice.
Program names and the MTA role have supporting explanation. Mail clients,
sendmail-compatible submission, queue inspection and wider MTA behavior remain
open. No full-objective or complete-exam-preparation claim is made.

## Development verification

Five model tests and seven authored, content and persistence tests pass. They
cover alternate alias paths, recipient order, own-name forwarding, partial
delivery, missing recipients, loops, saved/indexed divergence, atomic malformed
saves, limits, invalid packs, reset and resume. The exact comparison retains all
387 earlier scenario objects and 148 unit values. A focused UI-guidance test
keeps failed initial inspection on diagnosis guidance.

The first native run completed 95 actions and 24 captures. Review found a
missing body-font arrow glyph, imprecise practice actions and an overly tall
one-line forwarding editor. The revised view uses plain delivery sentences,
state-derived guidance, shorter forwarding editors and an inspectable current
lookup. The final native run completed 110 actions with 28 captures. All three
lessons exercised separate practice, independent assignment, Challenge,
keyboard text entry, partial failures, retry and save/readback. Wide dark100/150
and narrow Light200 views were inspected.

Final development screenshots are at
`.build-artifacts/native-runs/run-pq9kwqj9/screenshots/workspaces/`.
All nine learning screens were captured and reviewed, with representative
corrected screens checked in the final run. This is technical UI verification.
Independent novice comprehension, physical-device accessibility, screen readers
and runtime comparison against an actual MTA remain unverified. Final all-target Clippy, 1,053 Python tests, strict coverage validation and the
390-lab teaching inventory passed. The Fedora package passed private upgrade and reinstall, installed mail and
retained journal workflows, and separate-process NFS reopening. Cold reopening
of a mail draft remains unverified.

Installed screenshots are at
`.build-artifacts/native-runs/run-jcb8rnar/screenshots/workspaces`.
