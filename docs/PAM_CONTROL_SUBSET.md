# PAM authentication-stack foundation

Specified2026-11-08 before implementation; now implemented in development343.
The existing roadmap names LPIC-2v4.5/210.2 authentication-stack diagnosis. This
foundation teaches control flow; it does not implement a real authentication service.

## Contract

One virtual `pam.d`-style file, `auth` rules only. Supported simple controls:
`required`, `requisite`, `sufficient`, `optional`. Known module names are
`pam_unix.so`, `pam_permit.so`, `pam_deny.so`, without arguments. The supplied
case gives the modeled Unix module's return code; the simulator does not check
passwords, prompt, read accounts, load modules or authenticate anyone. Permit
returns success under the supplied valid-conversation assumption; deny returns
authentication error. These are explicit teaching facts, not real module execution.

Parse real rule order, whitespace, blank lines and comments. Bound input to8KiB,
128physical lines,256bytes/line and32rules. Refuse unsupported controls, arguments,
other management groups, includes/substacks, bracket actions, continuations,
module paths and unknown modules. Diagnose syntax, unsupported behavior and
limits separately. No filesystem, process, clock, network, PAM or DSO API.

Required failure is remembered while later rules run; requisite failure stops.
The first remembered failure survives. Sufficient success stops only without a
remembered failure; its failures are ignored. Optional success contributes while
optional failure is ignored. `PAM_IGNORE` on required/requisite contributes no
verdict. With no contributing success/failure, return permission denied, never
success. Keep every visited rule's module result, control action and remembered
failure in a bounded trace, separate from the final result. Parse errors are
errors, not successful denial cases. Modules after short circuit are not visited.

## Primary references and limits

Checked2026-11-08 against Linux-PAM **v1.7.1**, a documented source revision:

- [Configuration syntax and simple-control mappings](https://github.com/linux-pam/linux-pam/blob/v1.7.1/doc/man/pam.conf-syntax.xml).
- [Dispatcher, including ignored results and first-failure retention](https://github.com/linux-pam/linux-pam/blob/v1.7.1/libpam/pam_dispatch.c).
- [Unix authentication module](https://github.com/linux-pam/linux-pam/blob/v1.7.1/modules/pam_unix/pam_unix_auth.c) and [permit module](https://github.com/linux-pam/linux-pam/blob/v1.7.1/modules/pam_permit/pam_permit.c) distinguish actual credential/conversation work from supplied case outcomes.
- [LPI's versioned LPIC-2 objectives](https://wiki.lpi.org/wiki/LPIC-2_Objectives_V4.5),210.2.

No upstream implementation copied and no dependency added. Source-based semantic
checks are not an actual libpam comparison. Other PAM groups, credential/session
operations, password policy, LDAP/NSS, module arguments, distribution defaults and
complete authentication administration remain unimplemented. Do not change host
PAM files or call host authentication to validate this model.

The v1.7.1deny source URL was unavailable through the research tool. Its auth
return was separately checked against the [upstream master deny implementation](https://github.com/linux-pam/linux-pam/blob/master/modules/pam_deny/pam_deny.c)
and [upstream manual mirrored by man7](https://man7.org/linux/man-pages/man8/pam_deny.8.html)
on2026-11-08. This is an explicitly dated fallback, not a claimed v1.7.1download.

R01library/R02app and R03ten primitive tests passed; R04–06scopedClippy passed.
R09six workspace and ten primitive checks passed (draft separation, equivalents,
all-allow/all-deny, wrong/unsupported, source changes, bounds, Stop/retry/resume).
R12preflight compile error used the wrong internal test API; fixed without changing
assertions. R13two standalone authored checks plus the preceding16passed. The task was unregistered at that checkpoint. It is now registered as Partial
practice for210.2parent/K1; the exact packaged native gate remains pending.
Current ready342 remains unchanged, with no novice-comprehension credit.

Final source gates: R53all29behavior/preservation checks and R54five widgets/route checks pass after separate evaluate/publish phases; R57all-targetClippy, R60Python943and R61strictcoverage pass. Exact343package/native acceptance is still pending; latest ready342R54remains immutable.

The new standalone progress envelope is72KiB to hold four bounded8KiB raw
draft/tested strings after JSON escaping. R63red/R66green reproduce and fix the
worst-case roundtrip; raw field limits remain unchanged. R67widgets/route5,
R70all-targetClippy/R71Python943/R72strictcoverage pass. Replacement package
and actual native screenshots remain pending; r1is withheld, ready342unchanged.

R82eight widget/route checks and R83development-native59actions/22PNGpass after a single parser diagnostic/Actual results heading and native geometry/scroll pacing fix. Grading is unchanged; requested geometry must actually settle, and15-second/20-attempt limits remain. Final isolatedr3package/native admission is pending; r1/r2withheld, ready342unchanged.

Optimized r3 content passed but native capture was withheld after callbacks used
the work budget while waiting. R94/R97 reproduce timer and asynchronous waits;
R100 all11 regressions pass after preserving actual geometry observations and
separating waits from attempts. The independent real15-second action deadline,
1200 work-attempt and20 scroll-attempt limits remain. Candidate-r4 is pending.

Final scoped Fedora admission: R105–110 exactT224545Zpackage validates343/121;
59native actions/22PNG pass, root opened11originals, selected initial/final identity
matches. Receipt is private candidate-r4/r110-verified.json. Narrow200light and
wide150dark are actually observed. Previous failures remain withheld; no broader
platform, installed/cold, full-inclusiveRust, libpam or novice acceptance claim.
