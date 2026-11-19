# Apply a directory-backed LDAP policy safely

Source370/135: stable ID `lpic2-ldap-config-deployment`. LPIC-2 → **LDAP
configuration: inspect, apply and verify**, after **LDAP access: delegate contact
changes safely**. Prerequisite `lpic2-ldap-contact-delegation`. This distinct task
applies and verifies directory-backed configuration; it does not count a renamed
static policy or the three numeric/sample variants as new labs.

## Teaching and task

Introduction distinguishes settings from contacts, reads olcSuffix and the actual
indexed configuration name, demonstrates explicit LDIF Modify/replace and ordered
olcAccess values, then explains configuration plus data verification. The complete
worked workshop change is separate from the assignment. Never teach editing
slapd.d backing files as live configuration.

Try example starts with worked field rules sent to the wrong MDB entry. Inspect
Supplied configuration entries, Apply and try request, then fix dn and verify
protected mail/combined requests and nested people. An allowed own update alone
can coexist with a wrong deployment; the actual changed entry and preservation
result are shown. Check example tests every sample and never awards assignment
credit. Independent work uses a different backend index and branch, a broad
intentional starter, four scannable requirements and optional guidance.

Each scope has three directories and43actual contact requests, with actual
configuration changes/preservation checked alongside complete resulting data.
Equivalent supported ordered lists, dn.base and read fallbacks can pass the write
requirements. Wrong backend, extra neighbour damage, broad/deny-all policy, direct-
children-only scope, malformed/unsupported input and later request errors fail.
A later failure must not award completion for an earlier successful deployment.
Edits clear results; practice/assignment drafts and selections survive ordinary
progress and validated backup/restore. Previous369scenario/134unit values remain
exact. Existing static tasks omit the new optional field when serialized.

## Suitable workspace

Native inventory, no-wrap LDIF editor, actual applied entry/ordered readback and
Before/After contact data share one workspace. Wide columns stack when narrow;
shared font/padding controls, themes, focus/Tab/selection and larger text remain.
Full applied values and current supplied access values are optional inspection.
No assignment answer is exposed by default. Source row is still pending actual
native inspection, not complete merely through a shared renderer.

## Evidence and limits

Private evidence: next-ldap-config. R01core library51.67s, R02all15core, R03all66
retained codecs/query/change/utility, R05focusedClippy28.49s pass. R06workspace
compiles51.46s, R08all19new/retained checks pass; R07duplicate test-helper failure
was corrected without changing assertions. R10all27core/workspace/authored and
R13all26registered/retained tests pass. R14new4widget/capture and R15retained4
checks pass, covering five scales/two widths/light-dark, editor logical lines,
inventory bounds and keyboard behavior. R12capture helper collision was fixed
without changing prior capture code. Actual Fedora screens and370package are next.

This is a source-supplied configuration model with established identities, not
full OpenLDAP server setup, dynamic schema, real TLS/SASL/Bind/search authorization
or real slapd/ldapmodify interoperability. Every trial starts from original sample
configuration and directory. A bounded syntax preflight precedes sequential per-
request application; no whole-document transaction is claimed. No host file,
subprocess, network, module or system operation follows from learner input.
[Exact subset and primary sources](LDAP_CONFIGURATION_SUBSET.md).

No new full-inclusive Rust runtime or installed/cold lifecycle, all28theme sweep,
Windows/macOS/Omarchy runtime, physical-device/screen-reader or independent novice
acceptance. Developer/assisted review is not a beginner user study. Existing app
and original progress remain untouched; no release/publication or dependencies.

## Corrected native teaching review —2026-11-09

R17first actual76actions/17PNG passed; long horizontal worked code prompted a
small presentation refinement. R18all7authored checks now include the displayed
two-space-folded LDIF as a real runnable practice method. No assignment/grading
change. R19fourfinal widget/capture checks pass13.76s; R20build and R21corrected
actual76actions/17PNG pass. Root9originals opened in run-kr11ez5x: Introduction/
worked, practice, Assignment, partial failure, dark150editor and light200editor/
actual/result. The review row is updated; this is not independent novice acceptance.
Only210.4.K2gains new partial-practice credit; parent/K3scope expands narrowly.
All136parent objectives remain incomplete. Final lint/Python/inventory/coverage
and exact isolated370package verification remain pending.

Final source gates: R22all-targetClippy1m16s, R25all992Python12.888s, R26inventory
370(365updated/5already), R27strictcoverage pass.301Partial/7Supporting/932Open;
all136parents incomplete. Exact isolated370package remains pending; selected
inputs now freeze through its readbacks/native review/receipt. Ready369retained.

## Exact isolated370package verified

R28optimized build8m57s, R29/R32selected identity, R30content370/135, R31native
76actions/17PNG and R33receipt pass. Root9originals opened in run-woch6zet across
all3views, wrong/partial feedback, dark150and light200. Receipt precedes freeze
lift; do not replay old frozen MAINreaders after these notes. Selected identity
is not full compiler closure, independent seal or broader acceptance. No installed
app/original progress or system/publication changes. CHECKPOINT.md records exact
launch/hashes and limitations.

Actual packaged views (ignored local evidence, not mockups):
- [Introduction/worked code](../.build-artifacts/native-runs/run-woch6zet/screenshots/workspaces/config-introduction-worked.png)
- [Guided practice](../.build-artifacts/native-runs/run-woch6zet/screenshots/workspaces/config-practice-workspace.png)
- [Independent assignment](../.build-artifacts/native-runs/run-woch6zet/screenshots/workspaces/config-assignment-workspace.png)
- [Partial failure](../.build-artifacts/native-runs/run-woch6zet/screenshots/workspaces/config-partial-feedback.png)
- [Light200actual state](../.build-artifacts/native-runs/run-woch6zet/screenshots/workspaces/config-light200-actual.png)
