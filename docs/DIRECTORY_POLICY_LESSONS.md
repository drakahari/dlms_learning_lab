# Shared folders and restricted deletion

Two distinct LPIC-1 assignments follow **Access policy and safe file changes** (`linux-access-policy`)
in **Shared folders and restricted deletion** (`linux-sticky-directory-policy`).
The exact library path is **LPIC-1 → Shared folders and restricted deletion**:

1. **Keep collaboration while limiting deletion** (`lpic1-sticky-collaboration`).
   Inspect existing access, add restricted deletion, and test several simulated
   participants. Preserve collaboration, ordinary permissions and every file.
2. **Publish without replacing a colleague’s file** (`lpic1-sticky-collision-handover`).
   Diagnose an existing destination owned by someone else, publish under a supplied
   unused name, retire only specified owned entries, and preserve the colleague’s data.

These are narrow practical slices of104.5, not full permissions/ownership coverage.
The existing290Scenarios/93Units remain exact; the new catalog is292/94:
72CCNA,101LPIC-1,45LPIC-2,74Bash/Shell. Numeric mode variants are alternative
solutions, not extra labs. Original IDs, saved progress and evaluator19 are preserved.

## Teaching and meaningful practice

Introduction explains directory names versus file data, normal write/search access,
the extra entry-owner/directory-owner removal rule and t/T indicators. A separate
annotated command example uses `/practice` paths and different participants.
Try example retains a native terminal, Insert command without execution, actual
output, contextual actions and Check example. The collaboration example intentionally
uses1775 while the assignment needs its supplied1770ordinary rights preserved.
The handover example demonstrates a blocked collision, a successful new publication,
and a writable-content mistake with recovery. Its data overwrite is not incorrectly
presented as blocked by sticky.

Assignment keeps the complete requirements and simulation terminal. Solutions are
not in starters or exposed by default. Learn adds optional hints and an explicitly
labeled worked approach; Challenge retains the full independent task. A collapsed
**Participant actions · simulated** activity runs the real bounded operation on a
discarded copy. It is not a Linux `su` command and earns no assignment credit.

## Grading and persistence

Collaboration compares the complete initial/final file state, retains ordinary
permissions and ownership, requires the one intended sticky flag, and exercises
seven fixed participant operations. Permission denial must be an actual typed
permission failure: a missing file or unsupported command cannot receive that credit.
Handover compares the complete expected observable Node/data map and sticky policies,
then exercises its supplied participant cases. Supported `mv` and equivalent
`cp` plus `rm` solutions pass; wrong file contents, partial retirement, missing
foreign files and weakened permissions fail. No inode/mtime equivalence is claimed.

State contains separate assignment/practice worlds, drafts and bounded mutation
histories. Import/save/launch/check hold the bundled definition, replay both scopes
from it and reject forged history, wrong source/family/version or unrelated opaque
metadata. Raw source-free saving refuses this family; catalog-aware safe saving
preserves existing files on failure. Reset example changes only practice. Ordinary
Reset changes the selected attempt; other attempts and favorites remain.
Checks are invalidated after a command changes the simulated state; current success
is always recomputed against the held source. The model and its limits are in
[DIRECTORY_POLICY_SUBSET.md](DIRECTORY_POLICY_SUBSET.md).

## Verification status

Actual focused tests so far: core20, retained filesystem12/access7, workspace11,
source/progress5, registered catalog/preservation/feedback3. All passed. Catalog comparison
verifies every prior Scenario and Unit, not just totals. Actual fixed GNUchmod9.10
comparison15supported cases plus2honest exclusions is independently sealed.
These are not privileged/cross-owner kernel tests.

Seven actual-widget/capture checks pass, including field focus and control widths
at100/150/200%,760/1500px and light/dark themes. Native R12runs82real actions and
captures11screens (run-uiiikls3); both Introductions/Assignments, failure feedback
and light150/dark200 controls were opened. After this review, source feedback was
made more specific about lost group access and overwritten colleague data; final
rendering of those wording refinements and a separate candidate remain pending.
Final new widgets7 and authored3 pass; Python854/12.332s and resumed all-target
Clippy49.23s pass after memory-safe root staging. Initial guarded cancellation
remains retained. Coverage227partial/8supporting/1005Open and inventory287updated/
5alreadymeets pass. Independent source review9babd6f4/finalrefresh35fdfa8c found
no new concrete blocker. See CHECKPOINT.md for the latest
verified executable and subsequent gates. No fresh full inclusive Rust-suite,
all28-theme, complete keyboard-only, independent novice, screen-reader, physical-device,
Windows/macOS/Omarchy acceptance is claimed. No owner installation/profile replacement,
dependency/system changes or public push/merge/release is authorized.

## Actual packaged Fedora evidence —2026-11-05

Separate T033211Z/bd2b8824 candidate validates292/94. Packaged R33completes82real
actions/11PNG (run-wxy0082g), exit0. Root opened the final Introductions, guided
collision feedback, both assignment paths, lost ordinary-rights feedback and the
specific overwritten-data explanation. Light150/dark200 command text, placeholder,
focus outline and Run alignment were inspected. No mockups substitute for these.

- [Introduction](../.build-artifacts/native-runs/run-wxy0082g/screenshots/workspaces/sticky-collaboration-introduction.png)
- [Guided collision practice](../.build-artifacts/native-runs/run-wxy0082g/screenshots/workspaces/sticky-handover-collision-feedback.png)
- [Independent assignment](../.build-artifacts/native-runs/run-wxy0082g/screenshots/workspaces/sticky-handover-assignment.png)
- [Specific data-write feedback at200%](../.build-artifacts/native-runs/run-wxy0082g/screenshots/workspaces/sticky-handover-data-feedback-dark200.png)
- [Light150 focused controls](../.build-artifacts/native-runs/run-wxy0082g/screenshots/workspaces/sticky-collaboration-light150.png)

All611source/data,272documents and137scenario hashes matched MAIN before these
post-build notes. Binarybd2b8824168ae32e331efea075a53a481f502bc401f09303c71a1eb168a42706;
archive00b80e19306b69f55f1b2eb8ac1b751ca1b475897c4f2bbb7ea83baa111924a4;
manifestc610da3e12c1dbd99669c08d0bd46e369622baf0c24aec793187e01519ab2240.
R35verified receipt and all earlier failed/partial observations remain private.
No new installed lifecycle or two-process cold check was added for this family;
actual source-bound disk save/load/reset and packaged save/read are separate proof.
All earlier platform/accessibility/novice limits remain. Continue the roadmap;
this package is not curriculum completion or authorization to publish.
