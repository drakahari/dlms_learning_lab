# Password-aging simulator contract (specified before implementation)

Specified 2026-11-08 before implementation for the existing LPIC-1 local-account/
security gap:107.1 and110.1. Three tasks are now registered in development346;
focused behavior/UI gates pass, final isolated package/native admission is pending.
The contract alone never earns curriculum credit. PAM343 passed its separate
packaged gate before this implementation began.

No credentials, password hashes, real accounts, PAM APIs, host files, commands,
subprocesses, clock, services, network, root paths or authentication. Only supplied
non-secret metadata and explicit sample UTC days. Preserve old accounts engine.

Use a new bounded typed aging model. Reuse accounts::EpochDay only; do not change
its positive-date domain. Last change has explicit Disabled, ForceChange and Day
variants. Account expiry has None or positive day. Ordinary 1970-01-01/day0 is
excluded rather than confused with the force-change sentinel. Optional age fields
are disabled or0..365000; at most16 unique lowercase virtual identities.

Outcome is a policy observation, not login access: AccountExpired first, then
forced password change, disabled aging/future recorded change, maximum+inactivity,
maximum age, warning window and minimum change age. At exact boundaries use the
pinned Linux-PAMv1.7.1 helper's >= behavior. Expose warning days and too-soon change
separately; a too-soon change does not itself deny a normal account check. Disabled
maximum also disables its inactivity/warning checks; independent account expiry
still applies. No actual password change is modeled. Reject unsupported dates.

The chage4.18.0 manual describes metadata and lists max+last less-than wording;
that differs at equality from this versioned policy helper. Teach this distinction
and distribution/context dependence, never universal login behavior.

Closed commands: chage -l [-i] LOGIN; edits -m/--mindays, -M/--maxdays,
-W/--warndays, -I/--inactive, -E/--expiredate, -d/--lastday. Support separate
literal values and long=value. Query and edit cannot mix. Duplicate fields fail.
Require explicit known options; no interactive fallback, shell evaluation, short
clusters, host-root/prefix, arbitrary utilities, paths or unknown identities.
Parse every field before committing one atomic update. Preserve the world on any
error. Command<=1024bytes/32tokens; bounded16KiB output and64KiB standalone input.
Output is ISO aging metadata with familiar field labels; show dated policy
observations in a separately labeled native table so chage is not taught as a
login simulator. Unsupported forms give honest diagnostics, never canned success.

Before authoring: test chronological/leap/year boundaries, all sentinels, future
history, deadline equality, warning==0/>max, disabled fields, grace==0, huge/bad
values, atomic compound edits, equivalent options, unknown/duplicate args, escape
strings, bounds and roundtrip. A state can be well formed but an assignment policy
still wrong; do not prohibit plausible wrong solutions artificially.

Author distinct rotation/warning, bounded grace, and finite account-extension with
forced-reset capstone only after primitives/workspace pass. Use separate supplied
practice and assignment records/dates, protected sibling snapshots, meaningful
unfinished starts, introductory worked examples, guided actions and independent
requirements. Grading checks records and multiple date observations, not exact
command strings or a single today's status. Changing history, removing all limits
or clearing an intended reset must fail. Practice cannot earn assignment credit.

Primary sources rechecked2026-11-08 (no source code copied):
https://raw.githubusercontent.com/shadow-maint/shadow/4.18.0/man/chage.1.xml
https://raw.githubusercontent.com/linux-pam/linux-pam/v1.7.1/modules/pam_unix/passverify.c
https://www.lpi.org/our-certifications/exam-101-102-objectives/

Implementation status: pure model and three separate source-held tasks are now
implemented in development346. R17focused27checks and R21threeUIchecks pass;
new exact Fedora package/native gate remains pending. See AGING_LESSONS.md.

Final scoped Fedora admission: R46–51 exactT234326Zpackage validates346/122,
135native actions/45PNG pass and11originals were opened. Initial/final selected
identity matches777/191/323/4/70. Private r51receipt records hashes and limitations.
This supersedes preceding pending-package notes. No broader acceptance claim.
