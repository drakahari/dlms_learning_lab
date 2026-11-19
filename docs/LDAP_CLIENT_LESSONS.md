# Directory clients: bases, scopes and useful queries

Implemented2026-11-09 in the existing LPIC-2 roadmap under210.3. The catalog adds
two distinct assignments in **LPIC-2 → LDAP clients: bases, scopes and useful
queries**, after Authentication: ordered PAM checks. Its prerequisite guidance
points to service/DNS client recovery; the second lab follows the first.

- `lpic2-ldap-client-base`: repair virtual client defaults after a directory move,
  distinguish a one-command override from repaired defaults, include nested people
  with subtree scope and keep an Archive branch out of the requested result.
- `lpic2-ldap-contact-query`: investigate role/mail fields, combine meaningful
  filters and project a useful contact list across changed rosters and no matches.

The core and native workspace are documented in [the subset](LDAP_QUERY_SUBSET.md).
These are read-only query lessons, not a simulated OpenLDAP server administration
course. Updates/import, ldapadd/delete/passwd, authentication/TLS, full schema,
ACLs and protocol interoperability remain unimplemented.210.3 is Partial;
210.4 server administration remains planned. Do not credit those missing actions
because a reference or a directory entry exists. No numeric variants count as labs.

## Teaching and behavior

Each Introduction teaches naming, defaults, scopes/filters or combined conditions
and demonstrates separate workshop values. The contact example first inspects
uid/description/mail and shows why a matching role without mail is insufficient.
Try example puts Apply configuration beside its editor and Run search beside the
literal command field/output. Worked fill is explicit and inert. Assignment uses
literal requirements with answers hidden. Challenge retains the full work and
optional help, without unsolicited worked material.

Each task runs the learner query against multiple supplied directories. Independent
probes verify default settings, so -b/-H cannot conceal broken defaults. Returned
DNs and attribute/value sets are graded behaviorally; supported flag/attribute/
condition order alternatives pass. Unknown filters retain three-valued semantics.
A successful empty result, a missing base, simulated unavailability and a command
that could not run have different feedback. Actual per-case results remain intact
when the overall task fails. Query errors appear beside their output; repeated
identical parse diagnostics are displayed once with all case statuses available.

Edits invalidate checks and transient output. Practice and assignment drafts,
selected inputs and navigation persist independently. Existing360scenario objects
and128unit values/order remain exact; the previous78LPIC-2presentation values are
pinned. No original profile or installed app is touched. Save/read tests are not
cold reopening. Progress backup/restore uses separate synthetic files under the
runner's disk temporary root.

## Actual development evidence

- R01:15query semantics/boundary tests; R02:16query+15client tests.
- R03: focused Clippy passes27.94s.
- R04:14task/session tests; R09:15session checks after escaped-import budget proof.
- R10:5actual authored/catalog/progress checks and15session checks pass.
- R12 retains a test-only context/font lock failure. Precompute the font outside
  the lock; R13all4widget/capture checks pass20.80s, preserving geometry assertions.
- R21 retains an actual Enter shortcut failure: single-line TextEdit releases
  focus on Enter. Include that event's lost-focus response while respecting
  disabled state. R22all5widget/capture checks pass22.27s, including actual paste,
  Enter/run/Tab, five scales, widths, light/dark and practice/Challenge isolation.
- R19 fixed development route:130actions/32PNG, exit0/reaped44669. Root opened13
  original images, both lessons/all three views, actual changed inputs, partial/
  broken feedback, dark150wide/light200narrow. Final candidate route additionally
  exercises an explicit successful empty-result display; its result is pending.
- R20 actual development executable validates362labs/129units with
  `--validate-content --presentation-review`. R16 validated the retained201/54
  base catalog; R18's wrong flag order was rejected and is not full validation.

Clippy also retained a test module-name warning and a too-many-arguments warning;
use a distinct fixture module name and an explicit presentation tuple, without
lint suppressions or weakened assertions. Failed diagnostics remain privately.
Final focused tests, all-target Clippy/Python and exact isolated package admission
are pending in the checkpoint, not implied by these earlier development gates.

Actual development originals (local ignored evidence):

- [Defaults Introduction](../.build-artifacts/native-runs/run-3fddjob4/screenshots/workspaces/lpic2-ldap-client-base-introduction.png)
- [Guided contact investigation](../.build-artifacts/native-runs/run-3fddjob4/screenshots/workspaces/lpic2-ldap-contact-query-practice-workspace.png)
- [Independent assignment](../.build-artifacts/native-runs/run-3fddjob4/screenshots/workspaces/lpic2-ldap-contact-query-assignment-workspace.png)
- [Changed-input actual output](../.build-artifacts/native-runs/run-3fddjob4/screenshots/workspaces/lpic2-ldap-contact-query-practice-changed-inputs.png)
- [Full result, light200](../.build-artifacts/native-runs/run-3fddjob4/screenshots/workspaces/lpic2-ldap-contact-query-light200-full-result.png)

The final small Run-error/placeholder wording refinement postdates R19; it requires
exact-package readback. Technical native checks are not independent novice,
physical-device, screen-reader, all28-theme or Windows/macOS/Omarchy acceptance.
No dependency, host LDAP/shell/network, system setting, unapproved cleanup,
commit/push/merge, installed replacement or release publication was performed.

## Final development gates before packaging

R29 all61 focused Rust checks pass:52 LDAP semantics/client/session/authored plus
9 retained AAA checks. The old AAA current-total assumption was replaced with
exact360 snapshot/prefix and retained128 actual-unit equality, without dropping
its358/127 comparisons. R30 all-target Clippy passes1m06s. R32 all973 Python pass
12.676s after correcting two old catalog-size assumptions; old61/77/78 presentation
hashes are retained. R25 strict coverage294 Partial practice/7 Supporting/939 Open,
all136 parents incomplete. R24 invalid new status/evidence paths were refused;
use existing Partial practice and actual tests/ evidence paths, not weaker guards.
R22 explicitly verifies Enter produces actual output and Tab moves focus; Run
syntax errors appear in the output workspace. Final package route135actions/33PNG
includes a visible successful empty search, pending actual package execution.
No full-inclusive Rust, installed/cold or other-platform acceptance follows.

## Ready362 Fedora directory clients; continuing the agreed roadmap —2026-11-09

Launch: `/path/to/dlms_learning_lab/dist/teaching-review-20261109T123732Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab`
No flags; separate protected review progress. **362labs/129units:**88CCNA,117LPIC-1,
80LPIC-2,77Bash (26/45/32/26units). LPIC-2 → **LDAP clients: bases, scopes and useful
queries**, after Authentication: ordered PAM checks. Two complete read-only client
assignments teach repaired defaults versus one-command overrides and useful filters/
attribute selection against changing rosters. No real directory connection or bind.
Original360scenarios/128units and78LPIC-2presentation values remain exact.

R29all61focused Rust, R22all5actual-widget/capture, R30alltargetClippy1m06s,
R32all973Python12.676s, R34final26teaching checks, R35strictcoverage pass. R37optimized
compile exits0/reaped72623. R38initial/R41final selected identity matches833source/
data,205resources,339docs,4presentations and117supplementalfixtures. This is not a
full compiler closure or independent seal. R39actual package validates362/129.
R40native135actions/33PNG exits0/reaped84121; root14originals opened, including both
lessons/all3views, changed inputs, successful empty search, broken/partial feedback,
dark150wide/light200narrow full results and clarified Run-output placeholder.
R42receipt precedes freeze lift. Never replay frozen MAINreaders after these notes
or subsequent source work. Archive and binary stay immutable.

ArchiveSHA256 `767e47cab709c92c5fc3dbb3a484bb24dbf26e48226e6324e9d0a9ad9750d99c`
BinarySHA256 `fce7588ff02bf3c725f91db2984b328ee025c9e9e478ee70b6b875c1caca7f8b`
R37memory528samples min11.944GiBavailable, some/fullPSImax0,OOMdelta0. R40native
8samples min17.534GiB,PSI0/OOM0. Swap nearly full; sharedlock/jobs2/threads2/diskTMP/
guards remain mandatory. Two jobs are not a guarantee. No original progress,
installed replacement, dependency/system change, cleanup, commit/push/merge/release.

294Partial practice/7Supporting/939Open; all136parent objectives incomplete.210.3
query practice only; updates/import/passwords,210.4server/schema/ACL, full curriculum,
new full-inclusive Rust runtime, installed/cold lifecycle, all28themes, Windows/
macOS/Omarchy runtime, physical-device/screen-reader and independent novice acceptance
remain unverified or unimplemented. Same-process save/read is not cold reopening.
Next finite work: bounded directory-change parsing/evaluation before dependent
update/import lessons; source/auth/schema limitations must stay explicit.
[Lesson evidence](LDAP_CLIENT_LESSONS.md); [model boundary](LDAP_QUERY_SUBSET.md).


Actual exact-package originals:

- [Introduction](../.build-artifacts/native-runs/run-cjnhnjlh/screenshots/workspaces/lpic2-ldap-client-base-introduction.png)
- [Guided practice](../.build-artifacts/native-runs/run-cjnhnjlh/screenshots/workspaces/lpic2-ldap-contact-query-practice-workspace.png)
- [Assignment](../.build-artifacts/native-runs/run-cjnhnjlh/screenshots/workspaces/lpic2-ldap-contact-query-assignment-workspace.png)
- [Successful empty search](../.build-artifacts/native-runs/run-cjnhnjlh/screenshots/workspaces/lpic2-ldap-contact-query-practice-empty-success.png)
- [Full readable result, light200](../.build-artifacts/native-runs/run-cjnhnjlh/screenshots/workspaces/lpic2-ldap-contact-query-light200-full-result.png)
