> Ready367R1 exact Fedora package/content/native verified. See current CHECKPOINT.md for launch and receipt. Historical development notes follow.

# Precise LDAP account retirement

Development2026-11-09: one new substantive assignment, `lpic2-ldap-account-retirement`.
Source catalog367labs/132units:88CCNA,117LPIC-1,85LPIC-2,77Bash; units26/45/35/26.
LPIC-2 → LDAP retirement: precise names and preservation, after LDAP tools.
Latest admitted Fedora candidate remains Ready366R1 until a new package is verified.

Introduction teaches complete-name ambiguity, DN-list format and request/preservation
verification using a separate training directory. Guided practice diagnoses an
out-of-branch name, then a successful but unrequested deletion. Independent work
retires only the two named archived people and keeps the active same-uid entry,
all unrelated people and complete fields across three meaningful source cases.
The full task stays visible; worked data is optional. Challenge retains the task.

The existing native utility profile/command/editor workspace is opt-in typed
`dn_list`; existing LDIF utilities and document editors retain their prior mode.
Default `ldif` serializes absent, so earlier goal values/progress remain unchanged.
All366earlier scenario values and131unit values are regression-compared exactly;
all84earlier LPIC-2 presentation values remain hash-pinned. A preserved historical
pack supports the old catalog while the active bounded pack gains this task.
The128pack guard is unchanged, not raised or bypassed. No IDs are reused.

## Supported deletion subset

`ldapdelete -x -W -H URI -D DN` with either quoted literal DN arguments, `-f`
for the supplied virtual DN list, or the supplied editor as standard input.
Options precede positional names; each occurs once. Literal lexer:4096bytes,
32words; up to16person requests, DN1024bytes; virtual input64KiB/1024lines.
Aggregate resource bounds fail before requests; a later per-name/request error
retains earlier successful deletions. Missing entries fail32, out-of-branch or
read-only writes50, credential mismatch49 and nonleaf people66. A container is
explicitly unsupported rather than recursively approximated. Deletion cannot
remove an unrequested entry without failing complete-state grading.

DN-list format is plain printable ASCII with LF line endings and one full name
per line. Blank/comment/CRLF lines, LDIF headers, recursive/continuous/dry-run
modes, controls, TLS/SASL, password management and container deletion are honestly
unsupported. These restrictions are simulator boundaries, not claims about the
full OpenLDAP implementation. Input-file identifiers resolve only supplied virtual
buffers; no host file, process, service, password, network or server is used.
Source-held fictional profiles supply a simulated -W prompt; saved work stores
only the profile key. No learner password is requested or saved.

## Validation and sources

R02delete core14+retained25passes; R03focusedClippy passes. R04all33authored/core/
workspace tests pass, including six retirement tests: taught/list-order/positional/
editor alternatives, wrong branch, over-deletion, partial/missing/duplicate/error
cases, format/escape rejection, separate practice, invalidation/reset, actual
private save/read and validated export/restore. R05all11LDAPwidget/capture tests
pass. Groups overlap and are not unique totals. Native/package verification is
still pending; same-process save/read is not cold reopening.

Primary OpenLDAP Project manual, packaged by Debian as OpenLDAP2.6.10+dfsg-1,
checked2026-11-09: https://manpages.debian.org/trixie/ldap-utils/ldapdelete.1.en.html
LPIC-2v4.5/210.3: https://www.lpi.org/our-certifications/exam-201-202-objectives/
LDAP Delete and DN models: https://www.rfc-editor.org/rfc/rfc4511.html and
https://www.rfc-editor.org/rfc/rfc4514.html. No source code copied, new dependency
or reference tool/server installed. No composed real-server comparison claimed.
This is narrow utility practice, not210.3/210.4or full exam-objective coverage.
Password management needs protected transport/authentication modeling first.

R07all-target Clippy passes1m07s after fixing item placement; R11all983Python,
R09inventory367and R13strictcoverage pass. R08missing source tag and R10stale
presentation total failures were corrected without changing historical pins.
R12strictparent-map mismatch was fixed by synchronizing CURRICULUM.csv with the
scoped matrix. R14development build passes28.45s. R15actual Fedora60actions/12PNG
passes, exit0/reaped61761; root8originals opened from run-na10o_6e: all3views,
specific over-deletion, dark150wide and light200command/editor/result. The editor
capture was scrolled below its initial text; new retirement capture now targets
the first two font-sized rows. This changes evidence framing only, not layout.
Final exact candidate must verify that corrected capture. No installed lifecycle,
cold reopen, full-inclusive runtime, all-theme or other-platform acceptance.

R16actual corrected capture-phase passes; R17finalall-targetClippy passes5.89s. Final inventory/Python and isolated exact-package verification follow.

## Exact candidate acceptance

R21optimized8m48s; R22/R25identity850/210/344/4/129, R23package367/132 and R24Fedora60actions/12PNG pass. Root9originals opened from run-97iedwau, including corrected two-line DN-list capture at light200. R26immutable receipt is written before freeze lift. No full-inclusive runtime, installed/cold lifecycle, all-theme, other-platform, physical-device/screen-reader or independent novice acceptance.

Launch: `/path/to/dlms_learning_lab/dist/teaching-review-20261109T165305Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab`

[Introduction](../.build-artifacts/native-runs/run-97iedwau/screenshots/workspaces/lpic2-ldap-account-retirement-introduction.png), [practice](../.build-artifacts/native-runs/run-97iedwau/screenshots/workspaces/lpic2-ldap-account-retirement-practice-workspace.png), [assignment](../.build-artifacts/native-runs/run-97iedwau/screenshots/workspaces/lpic2-ldap-account-retirement-assignment-workspace.png), [specific feedback](../.build-artifacts/native-runs/run-97iedwau/screenshots/workspaces/lpic2-ldap-account-retirement-partial-feedback.png), [light200editor](../.build-artifacts/native-runs/run-97iedwau/screenshots/workspaces/lpic2-ldap-account-retirement-light200-editor.png). Actual local ignored evidence, not mockups or public image commits.
