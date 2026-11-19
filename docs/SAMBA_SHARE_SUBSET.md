# Standalone SMB sharing: bounded teaching and authoring scope

Source unit `samba-standalone-shares` follows NFS network sharing, with local
account lifecycle and Linux access policy as prerequisites. Experienced users
may launch it directly. Three distinct assignments use the accepted Introduction
→ separate guided practice → full independent assignment pattern:

- `lpic2-smb-identity-access`: distinguish an SMB account from its Unix identity;
  repair disabled/missing supplied SMB records and directory read/search without
  broadening already-correct regular-file modes.
- `lpic2-smb-team-policy`: configure separate writer/reviewer outcomes, reject
  outsiders at the share, and retain a default read-only public resource.
- `lpic2-smb-change-handover`: reconcile saved versus applied policy, direct
  clients to production, fix the remaining account/access problems, and preserve
  the readable archive instead of masking a wrong destination.

The guide server uses different supplied actors, groups and paths. Trying it or
explicitly filling its worked configuration never awards assignment credit.
Configuration fill does not save, apply or execute. Assignments start with their
intentional errors, not a populated solution. Their editor, Unix terminal,
account controls and client trial remain available independently in Learn and
Challenge; guidance is optional in Challenge.

## What is actually modeled

All operations use bounded in-process state and virtual regular files. The
configuration editor and `cat` refer to **one saved virtual smb.conf file**.
Editing, saving, syntax checking and applying are separate operations. Applying
an invalid file retains the prior policy. Each native trial creates a new
simulated connection and reports the actual resolved file and output, or the
specific failed layer. A syntax pass is not proof of access.

Named authentication uses supplied valid/invalid credential facts and enabled
SMB-account records mapped to supplied non-root Unix identities. These are
abstract test inputs, not real passwords, hashes or SMB cryptography. Invalid
named credentials never silently become guest access. Anonymous clients use the
supplied guest identity only when the share permits it.

Share permission and Unix permission both apply. `invalid users` wins over
`valid users`; `write list` wins over `read list` and the read-only default, but
never grants missing Unix file rights. Directory search, listing and file
read/write use the existing permission model, including owner-class precedence.
An existing-file replacement needs search plus file write; no creation rights
are inferred. Connection policy snapshots and fresh connections are tested;
Unix group membership changes require reconnecting in this disclosed model.

Supported configuration: `[global]` standalone/user-security/map-to-guest=Never,
named share `path`, `read only`, inverted `writable`/`writeable`, `guest ok`,
`valid users`, `invalid users`, `read list` and `write list`, with simple named
Unix identities or `@group` lists. Unknown directives or unsupported syntax
produce honest diagnostics. Duplicate parameters follow their last value;
duplicate sections, variable substitutions, includes, quoted principal syntax,
NIS/domain selectors and continuations are excluded rather than approximated.

Filesystem terminal: `ls`, `id`, `cat`, `pwd`, `cd`, `chmod`, `chgrp`, through the
existing bounded virtual filesystem only. Use `id -Gn USER` to inspect groups.
Native account controls and Apply saved settings are explicitly **simulator
administration**, not commands taught as Linux utilities. `smb-share` JSON in
source solutions is a developer fixture codec; generic learner terminals refuse
it without an authored goal and never pass it to a shell.

## Grading, limits and progress

Every task has twelve deterministic positive and negative client cases. Separate
case inputs replace only source-selected sample file text in cloned worlds;
the learner's trial files remain intact. Reads compare both resolved path and
output; writes compare resolved path, byte count and resulting contents. A
missing file or unsupported operation cannot earn intentional-denial credit.
Preservation requirements, account status, public read-only policy and agreement
of supported saved/applied settings are also checked. Equivalent policies,
permission repair forms and list order are accepted; exact configuration text
is not the answer rubric.

Configuration ≤16KiB/128lines/512bytes per line/8shares/16principals per list;
≤16account records/8clients/16cases, individual trial text ≤4096bytes, total
virtual file text ≤32KiB, serialized session ≤512KiB. Command input ≤2048bytes;
transient history ≤32entries/32KiB. Paths reject traversal, absolute client
paths and unsupported separators. No host files, subprocesses, network,
services, accounts or credentials are accessed through learner input.

Drafts, virtual files/modes, account changes and saved/applied settings persist
in the existing source-bound progress format; prior lab IDs/data are unchanged.
Transient output and trial observations do not claim a new run after reload.
Edits clear current checks; grading recomputes actual outcomes, so imported or
stale success flags cannot grant credit. Reset restores only that selected lab's
assignment and independent practice. Clearing/mistyping a trial filename is a
preservable draft; running it gives a diagnostic without filesystem mutation.

## References and remaining scope

Checked2026-11-13 against current [LPIC-2 v4.5 objective209.1](https://www.lpi.org/our-certifications/exam-201-202-objectives/),
[Samba smb.conf](https://www.samba.org/samba/docs/current/man-html/smb.conf.5.html),
[testparm](https://www.samba.org/samba/docs/current/man-html/testparm.1.html) and
[smbpasswd](https://www.samba.org/samba/docs/current/man-html/smbpasswd.8.html).
Current configuration/checker manuals identify documentation4.25.0; this is not
an installed Samba version claim. No local testparm binary was available; no
package installed or real-Samba reference comparison performed. Original code
and authored teaching retain the approved MIT license; no vendor code imported.
Announced future objective proposals remain separate from current requirements.

This unit is partial standalone policy/access practice, not full209.1 coverage.
AD/domain joins, Windows-name mapping, Kerberos/NTLM/wire negotiation, signing,
encryption, Windows ACLs, printers, CIFS mounts, service installation/lifecycle,
Samba logs, full utilities, filename case-folding, creation/deletion/rename,
locking and cached open handles remain unimplemented. New file operations
recheck Unix permissions; this is not a kernel open-descriptor cache model.
No vendor endorsement, exam-preparation completeness or real-equipment
experience claim. Windows/macOS runtime, physical-input, screen-reader and
independent novice comprehension remain unverified.

## Verification state

Focused source tests cover the parser,512-case independent access oracle,
source binding, taught methods, alternative policies, negative/partial results,
wrong-file false positives, invalid packs, limits, separate practice, stale
checks, reset and disk round-trips. Prior455scenario/181unit values/order are
compared exactly. Nine focused widget/plan/capture checks and development199-action/45-capture
render flow pass. Two actual normal-process cold launches restore and repair
both drafts exactly; four original captures were inspected. The separate458Fedora candidate and installed acceptance now pass; see
CHECKPOINT.md and the installed evidence below. These are technical checks, not
novice usability.

## Fedora development render evidence

Corrected development flow:199fixed actions/45actual captures across all three
lessons. Source checks and captures are synthetic technical observations, not
independent novice, physical-input or screen-reader acceptance. Mixed-status
account buttons now share a font-derived column; wide layout reserves actual
shared label/status/button/spacing/padding widths and stacks on compact windows.
Single-line fields retain native centered text/cursor geometry; the multiline
editor stays top-aligned. Existing tracks keep their prior rendering.

- [Introduction: SMB accounts and Unix access](../.build-artifacts/native-runs/run-t947dolm/screenshots/workspaces/samba-0-introduction.png)
- [Separate guided account/permission practice](../.build-artifacts/native-runs/run-t947dolm/screenshots/workspaces/samba-0-practice.png)
- [Independent policy assignment and editor](../.build-artifacts/native-runs/run-t947dolm/screenshots/workspaces/samba-1-assignment.png)
- [Handover worked example](../.build-artifacts/native-runs/run-t947dolm/screenshots/workspaces/samba-2-worked.png)
- [Light150 at a narrow window](../.build-artifacts/native-runs/run-t947dolm/screenshots/workspaces/samba-2-light150.png)
- [Navy200 at a narrow window](../.build-artifacts/native-runs/run-t947dolm/screenshots/workspaces/samba-1-navy200.png)
- [Actual reviewer write denial](../.build-artifacts/native-runs/run-t947dolm/screenshots/workspaces/samba-0-reviewer-denied.png)
- [Before: stacked wide layout and shifted account actions](../.build-artifacts/native-runs/run-eb1hx6co/screenshots/workspaces/samba-0-practice.png)

Development normal two-process cold reopening now passes, including exact
restoration and native repair of separate drafts. The separate458package now also passes its private installation/upgrade and
installed native/cold checks; source development evidence remains identified here.

- [Cold-loaded independent draft](../.build-artifacts/tmp/native-reopen-480dc0cac71c3dacdf421dbe/read/reopen-samba-reader-assignment.png)
- [Cold-loaded separate practice draft](../.build-artifacts/tmp/native-reopen-480dc0cac71c3dacdf421dbe/read/reopen-samba-reader-example.png)
- [Repaired practice and fresh state-based check](../.build-artifacts/tmp/native-reopen-480dc0cac71c3dacdf421dbe/read/reopen-samba-reader-repaired.png)

These local screenshots stay ignored; no recordings/progress/images are added
to a public commit or release. Checkpoint records live jobs and failed diagnostics.

## Installed458 candidate evidence —2026-11-13

The protected review launcher and exact checksums are in CHECKPOINT.md. Private
455→458 install/upgrade/uninstall/reinstall preserves synthetic progress exactly.
Installed native199actions/45captures and normal two-process cold reopen/four
captures pass. Reader observed exact raw/portable/provenance/preferences and
independent drafts before navigation, then repaired both through native controls.
These originals were inspected, not inferred from assertions:

- [Installed account Introduction](../.build-artifacts/native-runs/run-ibzlg68l/screenshots/workspaces/samba-0-introduction.png)
- [Installed practice and useful failure](../.build-artifacts/native-runs/run-ibzlg68l/screenshots/workspaces/samba-0-practice-failure.png)
- [Installed independent assignment](../.build-artifacts/native-runs/run-ibzlg68l/screenshots/workspaces/samba-0-assignment.png)
- [Installed policy editor, Light150](../.build-artifacts/native-runs/run-ibzlg68l/screenshots/workspaces/samba-1-light150.png)
- [Installed broken-again feedback](../.build-artifacts/native-runs/run-ibzlg68l/screenshots/workspaces/samba-1-broken-again.png)
- [Installed handover practice](../.build-artifacts/native-runs/run-ibzlg68l/screenshots/workspaces/samba-2-practice.png)
- [Cold-loaded independent draft](../.build-artifacts/tmp/native-reopen-1659e0a754a447aadca0b32c/read/reopen-samba-reader-assignment.png)
- [Cold-loaded separate practice draft](../.build-artifacts/tmp/native-reopen-1659e0a754a447aadca0b32c/read/reopen-samba-reader-example.png)
- [Fresh check after native repair](../.build-artifacts/tmp/native-reopen-1659e0a754a447aadca0b32c/read/reopen-samba-reader-repaired.png)

Original owner data/app stay separate. No real Samba/network comparison, broad
Rust runtime repeat, physical-input, screen-reader, independent novice or other-OS
acceptance. Long selections ellipsize at narrow200 and large pages scroll.
Selected2957inputs retained; not full compiler closure or independent seal.
Candidate remains immutable; these evidence notes were added after packaging.
