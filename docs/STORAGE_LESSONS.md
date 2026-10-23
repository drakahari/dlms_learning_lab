# Persistent mounts and UUIDs

Four authored LPIC-2 lessons, accepted in M55 on 2026-10-23. They extend the original capacity/mount seed; they are not filesystem-maintenance or full boot administration.

| Assignment | Practical work | Builds on |
| --- | --- | --- |
| Identify and mount the assigned volume | Inspect supplied UUID/type inventory, repair the work source, mount it and preserve the healthy reports volume | Mountpoints, saved rows, stable identity |
| Make saved and live mount policies agree | Repair two opposing access policies in both saved configuration and current mounts | Identity lesson; saved versus live state |
| Keep maintenance manual without losing access | Attach a read-only XFS volume now while retaining `noauto` for independent future sessions | Policy lesson; manual versus automatic |
| Complete a stable storage handover | Repair unstable identity, a filesystem-type mismatch, wrong access policy and a skipped manual source while preserving a healthy peer | All three lessons |

Each introduction teaches prerequisites before the task and supplies separately explained commands. A different example has its own inventory, configuration, terminal history and checks. **Fill worked example** changes that editor only; learners still save and execute commands. The suggested experiment breaks a relevant outcome, and its tested repair restores it. Assignment drafts and credit remain separate. Challenge retains the same complete task without unsolicited examples.

The assignment ticket supplies each volume's purpose and UUID. `blkid` reports identity/type, not business purpose. Native Save changes the simulated file; `mount TARGET` changes live state separately. The full task remains above an editor and investigation terminal, with optional hints/references. A fresh-session test uses independent copies with deterministic device-name permutations; it is not a real boot.

## Review and verification

The implementing content author proposed the four lessons; root independently reviewed instruction prerequisites, distinct faults, actual state transitions and rendered screens. `tests/storage_content.rs` exercises every reference, four alternative solutions, 33 plausible incorrect configurations/actions, intermediate partial states, nonmutating inspection, independent example experiments and repairs, current-success invalidation, selected reset, all prior82 attempts, separate drafts and composed catalog filters. Alternative comments, supported option orders, action orders and stable aliases are accepted when they meet all outcomes.

Every saved identity is rechecked under each authored alias permutation, including skipped `noauto` entries. Saving an apparently valid but unstable device path cannot earn success merely because it works in the current session. Healthy peers, numeric metadata, current mounts and manual-only absence remain explicit requirements. See the underlying [model](STORAGE_SUBSET.md) and [native workspace](STORAGE_WORKSPACE.md) tests for malformed packs, duplicate maps, bounds and pinned restoration.

Actual Fedora native walkthroughs cover all four introductions, separate examples, initial feedback, Save/terminal/Check events, completion at the unchanged walkthrough position, exact resume, current invalidation, selected reset, Challenge and narrow light150/dark200 views. The harness supplies reference editor text programmatically; this is technical workflow/render verification, not a novice study or physical-input accessibility acceptance. Screenshot artifacts stay outside public version control.

## Sources and exact coverage

Current [LPIC-2 v4.5 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/) were rechecked2026-10-23. Narrow203.1 UUID/blkid evidence and stronger fstab/mount evidence are recorded explicitly in [SKILL_COVERAGE.csv](SKILL_COVERAGE.csv). Existing complete-inventory fingerprints retain their2026-10-22 check date. Technical references and fixed util-linux comparisons are listed in [STORAGE_SUBSET.md](STORAGE_SUBSET.md); each bundled scenario also records sources and simplifications.

Only supplied ext4/XFS metadata, bounded six-field rows, fixed non-nested mountpoints and the disclosed command subset are modeled. No real devices, file content, kernel, boot sequence, swap, systemd mount units, filesystem repair, dump or fsck execution exists. These lessons do not promote203.2 maintenance or full utility syntax to covered status.
