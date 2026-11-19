# Remote management policy foundations

Development batch,2026-11-09. Two authored CCNA assignments extend the recorded
4.8 remote-administration gap. This is **bounded practical coverage**, not a full
SSH server, authenticated CLI session, cryptographic implementation or complete
exam preparation. CCNA5.3 local identities/privileges remains open.

## Learning sequence

Prerequisites: the existing IPv4 access-policy and TCP/UDP-conversation units.
The new unit is **Remote management: protocols and approved sources**.

| Stable lab ID | Independent assignment | Distinct investigation |
| --- | --- | --- |
| `ccna-management-protocols` | Retain both administrators on SSH version2 across every supplied VTY line; reject Telnet and the legacy-version trial. | Incoming-protocol policy and global SSH-version policy are separate; a single correct selected line does not prove the entire range. |
| `ccna-management-source-policy` | Keep both approved administrators while rejecting two other supplied sources through an incoming standard list. Preserve the protocol/version restrictions. | First-match ordering, exact permits, implicit deny, terminal attachment and avoiding an administrator lockout. A nearby workstation detects an overbroad subnet permit. |

Introduction teaches the router-local management path, terminology, settings and
a worked sample. Try example uses separate addresses and a smaller supplied line
range, with a real editable configuration document, Apply policy, selectable
client/line trials and actual virtual-path/policy feedback. The worked example
is optional and filling it neither executes nor grades the assignment.
Assignment retains its own unfinished document, explicit requirements and
Check configuration across every deterministic client/protocol case and supplied
line. Challenge opens the same full assignment with guidance available on request.

The learner does not prepare a VM, enter a profile path, supply credentials or
run host commands. No SSH/Telnet client or server is started. See the
[defined subset](MANAGEMENT_POLICY_SUBSET.md) for the parser and simulation limits.
A successful trial means the IP path and policy permit an authentication attempt;
it does **not** mean login succeeded. Host identity is supplied read-only metadata.

## Preservation and grading

All354 previous scenarios and125units retain their exact serialized values.
The existing82 CCNA presentation rows are retained: canonical274848bytes,
SHA256 `e487b77e20127db0239e99b1350422d3fe41faf8295b78aa15aac43daa567f08`.
The new catalog is356labs/126units:84CCNA,117LPIC-1,78LPIC-2,77Bash.
These are authored registrations, not numerical practice variants.

Accepted alternatives include equivalent line grouping and standard source-rule
decomposition. Broken paths, missing line coverage, accepting everyone, denying
everyone, retaining only one administrator and allowing an extra nearby source
cannot pass. Errors identify the document line/category or first failing VTY and
expected versus actual policy outcome. Editing clears current success and trial
observations. Saved practice/assignment drafts remain separate; checks are
recomputed against source-held requirements, not trusted saved report flags.
Transient observed output is not imported as executed evidence.

## Evidence in progress

R36:53focused model/path/document/task/session/adapter/authored tests pass.
R37:3/4native-widget tests pass; the failing diagnostic expectation exposed an
incomplete supported setting classified as unsupported. R40:11document tests pass
after that diagnostic correction, including original line-number checks.
R41:5widget/capture tests pass, including real Ctrl+A/Paste, invalidation, separate
practice/Challenge, all five supported scales, wide/narrow and light/dark states.

R42 development-native used the base catalog without the explicit review flag,
so the fixed route correctly had no new task to open; no product launch acceptance
was inferred. R43 selected the review catalog and exercised the first assignment,
actual trials, alternatives, save-read and success invalidation. Rendered inspection
showed a wide-layout threshold and a large-text reveal problem. The stable shared
editor viewport and a font-derived split threshold address these; R44/R45 validate
the resulting build/widgets before a new native run. This paragraph is a work
ledger, not candidate admission or a claim all native actions passed.

No new full-inclusive Rust runtime, installed-app upgrade/cold lifecycle,
Windows/macOS/Omarchy runtime, all28-theme sweep, screen-reader, physical-device or
independent novice-comprehension acceptance is claimed. Owner's previous assisted
review is not an independent novice study. Original progress/installation and the
ready354 candidate remain untouched; no push, merge or release.

## Development-native admission of the two lessons

R49all5 widget/capture tests pass27.11s after using a stable global editor ID
across reflow and a shared fixed editor viewport. R45/R47retain the preceding
focus-ID failure; assertions were retained and strengthened to measure the full
native editor frame and require wide splitting/narrow stacking.

R50owned97239 exits0/reaped:116actions/28PNG in `native-runs/run-dpkq3od7`.
Root opened10originals: both Introductions/practice/Assignments, partial failures,
actual trial output, light200narrow editor/feedback and dark150wide. This technical
screen review is not independent novice or physical-device acceptance. Native
save-read is same-process, not cold reopening. Assignment source/grade/behavior
was retained; only the supported-incomplete-setting diagnostic was corrected.

R51 inventory records356actual registrations,351updated+5alreadymeets. R52strict
coverage validation passes; CCNA4.8becomes Partial practice, not complete, and5.3
remains Open.288Partial/7Supporting/945Open; all136parents remain incomplete.
R53–55staged/all-targetClippy pass (55.98salltarget). R56Python reports960/961pass:
the old ARP presentation test incorrectly treated every later appended lesson as
an ARP row. Its exact three-row boundary is retained, plus a new old82canonical
pin and explicit two-management-row checks. The corrected full Python run follows.

Separate review candidate preparation follows final gates; ready354R65 remains
immutable until actual exact-package validation and screenshots are recorded.

Final pre-candidate gates: R57all21presentation checks pass; R58all962Python
checks pass12.757s. R59all54focused Rust checks pass. R49all5native widget/capture
checks and R55all-targetClippy remain valid; no further UI/engine behavior changed.
R50memory observations: see the private runner log for measured values; no guard
termination or OOM increase was observed. The runner kept one heavy invocation,
two Cargo jobs/two test threads and disk-backed temporary data throughout.

Representative actual development screenshots (ignored local evidence):

- [Protocol Introduction](../.build-artifacts/native-runs/run-dpkq3od7/screenshots/workspaces/ccna-management-protocols-introduction.png)
- [Guided policy editing](../.build-artifacts/native-runs/run-dpkq3od7/screenshots/workspaces/ccna-management-protocols-example-guidance.png)
- [Source-policy partial feedback](../.build-artifacts/native-runs/run-dpkq3od7/screenshots/workspaces/ccna-management-source-policy-assignment-partial.png)
- [Light200: readable check/results](../.build-artifacts/native-runs/run-dpkq3od7/screenshots/workspaces/ccna-management-source-policy-light200-feedback.png)

PackagingR1not admitted: R61optimized build exits0/reaped8m33s; R62exact document
readback detects the two new guides absent from the packager's explicit review
copy list. The artifact and failure remain intact. Source/data metadata and actual
binary SHA match their frozen identities; this does not waive the missing docs.
The copy list is corrected and a focused regression checks the actual copy loop.
R2will refreeze and rebuild a separate candidate; ready354remains current meanwhile.

R63all22presentation/package-guide tests and R64all963Python checks pass12.693s.
R61compile520memory samples: minimum12.379GiBavailable, some/fullPSIavg10max0,
OOMdelta0. Swap remained nearly full; no system settings changed. R2packaging
reuses the verified compiler inputs and must pass its own exact document readback.

## Exact Fedora package verification

R2corrected bundle: R66cached compiler0.06s; R67/R70selected initial/final identity
match809source/data,199resources,333documents,4copied presentation shards and97
supplemental test/tool fixtures. This is not a full compiler closure or independent
seal. R68actual packaged catalog356/126passes without a review-selection flag.
R69owned54073exits0/reaped:116actions/28PNG;10originals opened from run-97c1oim7.
All three lesson views, partial failures and actual trial feedback are readable.
Dark150wide and light200narrow editor/actions were inspected. The screenshot named
light200-feedback stops at the action row: do not infer visible case results from
its filename; developmentR50contains the enlarged actual-result evidence.
Same-process save-read is not installed or cold-reopening acceptance.
R71records admission before lifting the selected-input freeze; later public notes
do not rewrite the immutable candidate. R1and its rejected packaging diagnostic
remain intact. Exact launch and hashes are in [CHECKPOINT](CHECKPOINT.md).

- [Packaged Introduction](../.build-artifacts/native-runs/run-97c1oim7/screenshots/workspaces/ccna-management-protocols-introduction.png)
- [Packaged practice editor](../.build-artifacts/native-runs/run-97c1oim7/screenshots/workspaces/ccna-management-source-policy-example-guidance.png)
- [Packaged independent feedback](../.build-artifacts/native-runs/run-97c1oim7/screenshots/workspaces/ccna-management-source-policy-assignment-partial.png)

Next source-resolved local-access work remains research-only at this receipt.
No dependency, system/security setting, cleanup, original-profile write or release.
