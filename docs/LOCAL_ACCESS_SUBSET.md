# Local access capability contract

2026-11-09, source-resolved follow-up to remote-management policy foundations.
This model contract alone grants no curriculum coverage. Two authored lessons
now use it in development; see [lesson evidence](LOCAL_ACCESS_LESSONS.md). The356
review candidate remains immutable until separate exact-package admission.

## Meaning

The first subset uses an **explicitly supplied local-case authentication method**
and **explicit local EXEC authorization**. A named local record contains a training
secret token and either user access (level1) or privileged access (level15).
Names and training tokens compare exactly and case-sensitively. An unknown record and wrong token are different observed rejections. A missing
configured token is an unsupported/incomplete configuration diagnostic; do not
infer a device’s no-password default. Correct matching
returns the record's identity and level in the simulation; it does not establish
an SSH session or execute a CLI.

Training tokens are ordinary lesson data, stored as text. They are not Cisco
password hashes, real credentials or a cryptographic implementation. Lessons must
use clearly supplied training values and explain that drafts are saved locally.
Do not ask for a personal password, encode fake hashes, teach deprecated storage
formats as preferred, or claim hash/KDF interoperability. All real password types,
KDFs, transport crypto and key exchange stay excluded. No new dependency is needed
for this explicitly bounded metadata model.

Composition with the existing management path must first observe the actual
virtual IPv4 route/reply and per-line source/protocol/version policy. A broken IP
path or earlier policy rejection is not a failed local-password comparison. Only
a permitted attempt reaches the local record check. Malformed, oversized or
unsupported input produces a diagnostic; it must not satisfy a negative test.

## Bounds and exclusions

Version1; at most16local records; names1–32ASCII letters/digits/underscore/dot/dash;
training tokens1–128printable ASCII characters, with spaces preserved exactly;
32KiBserialized policy. These are simulator limits, not a claim of every device's
accepted password lengths or characters. Missing or empty configured tokens are rejected as incomplete/unsupported
configuration. Submitted empty tokens are wrong tokens, not a bypass.
Atomic replacement validates the whole candidate before changing applied state.
Unknown serialized fields, methods, authorization modes and privilege variants
fail honestly. There is no automatic default authentication/authorization method.

This first subset excludes non-AAA login local, shared line passwords, AAA local
case folding, remote RADIUS/TACACS+, method-list fallback, none/nopassword bypass,
accounting, arbitrary command privilege remapping, enable/disable sessions,
intermediate privilege levels and platform password-policy/default inference.
Only levels1/15 are claimed. No service, host filesystem, process or network access
is available to the learner through this capability.

## Primary references checked2026-11-09

Catalyst9300 IOS XE Dublin17.12 chapters, updated2023-07-28:

- [Local authentication and authorization](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/configuring_local_authentication_and_authorization.html): local authentication and EXEC authorization are distinct explicit settings.
- [Passwords and privilege levels](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/controlling_switch_access_with_passwords_and_privilege_levels.html): levels1/15 and current secret-storage guidance; old plaintext/reversible examples are not recommendations for a deployed device.
- [IOS AAA command reference](https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/security/a1/sec-a1-cr-book/sec-cr-a1.html): local-case is a separately named case-sensitive method. Do not assign its behavior to another method by assumption.

The local-AAA and privilege chapters differ in their wording for level0; this
subset does not guess level0's full command set. General IOS references inform
the explicitly named method, not an inferred Catalyst hardware default. No source
prose or vendor implementation is imported as simulator code.

## Configuration-document adapter

The next adapter reads a complete management document with explicit `aaa new-model`,
`aaa authentication login default local-case`, `aaa authorization exec default local`
and `username NAME privilege 1-or-15 secret TRAINING_WORD` records. Global default
means the explicit default login method on the supplied terminal lines; named or
non-AAA overrides are unsupported. Optional `login authentication default` may
make that selection explicit inside a line group. It does not model other ports
or secure HTTP automatically.

For this document subset, training words contain only ASCII letters/digits/dot/
underscore/dash, with no quoting, expansion, hashes or algorithm-type setting.
The core's wider printable-token model does not imply a richer CLI lexer.
Existing management-document parsing still owns the IP/protocol/source settings;
filtered AAA/account rows retain original line numbers for its diagnostics.
Duplicate declarations are rejected rather than treated as session overrides.
No defaults are inferred if a required AAA setting is absent. This is logical
training-token matching, not Cisco secret hashing or a running IOS configuration.

Core gates: R05all11focused tests pass;R06–08staged/all-targetClippy pass
(53.63salltarget);R09all965Python pass12.710s. R04retains a test-only E0507match-guard
ownership diagnostic; borrowing the decision preserves the exact denial assertion.
No lesson, UI, progress schema or5.3credit has been added by this capability gate.

Document gates: R13all21core/document tests pass. R14–16staged/all-targetClippy
pass49.41salltarget;R17all965Python pass12.780s. Complete-document syntax supports
explicit supplied default methods, not inferred device behavior. Source-held
behavioral task/session grading must precede any authored lesson or5.3credit.
Before the next candidate, add this new guide to the explicit review package list
and verify its actual inclusion; the ready356package is intentionally unchanged.

## Source-held task and session gates

Source grading checks the complete permitted-account list plus every required
incoming-policy and local-account case on every supplied terminal line. A rotated
or renamed surplus account cannot satisfy retirement. Wrong upstream policy or
path failure cannot substitute for rejected local credentials. Equivalent record
order, line grouping and supported source-list rules remain valid alternatives.

Practice and assignment retain separate bounded raw/applied documents and trial
selectors. Edits clear checks/output; malformed application is atomic; check flags
are recomputed against source requirements. Transient observations are not restored
as execution. Oversized edits retain the last bounded text with an explicit error.

R24all44focused checks pass;R29all15session/task checks andR30all-targetClippy/
R31all965Python pass. R36all20source-adapter/session/task checks pass after additive
Goal/State/progress/backup integration. R33/R34compile the native page; actual
widget/compositor/content review and authored lessons remain pending. No5.3credit
is granted by these capability/adapter tests. The existing356review stays intact.

## Authored development admission

R48 actual authored4/adapter5 tests pass, retaining the exact356scenario/126unit
baseline and old drafts. R47 all5 actual-widget/capture checks pass. R50 actual
native131actions/30captures exits0/reaped, including both new lessons/all three
views, actual trials, partial/wrong feedback, alternatives, invalidation/retry and
save-read/reset. Root opened12originals including light200narrow full result and
dark150wide. No cold reopening or novice-study claim. Catalog358/127;5.3 is Partial
practice, not complete. Real device/password/crypto/remote AAA limitations above
remain in force. Exact-package admission is a separate gate, still pending.
