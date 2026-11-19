> Ready368R1is now verified for exact package/content and scoped native Fedora behavior. Historical pending notes follow. No real TLS/OpenLDAP conformance is claimed.

> Current2026-11-09: source368/133now includes the protected password lesson,
> with core/authored/widget and actual Fedora development-native checks passed.
> Isolated368package acceptance is pending; Ready367R1 remains the admitted artifact.
> Earlier foundation-only notes below are historical, not current catalog status.
> See LDAP_PASSWORD_LESSON.md. Symbolic protection never claims actual TLS.

# Protected LDAP authentication foundation

2026-11-09: internal, opt-in simulator foundation only. **No new registered lesson,
UI, objective coverage or candidate acceptance.** Catalog remains367labs/132units;
Ready367R1 is the latest admitted Fedora artifact. Do not replay its frozen MAIN
readers after later source/doc changes. Existing plain query/identity/LDIF/utility
APIs, goals, IDs and saved work are unchanged. No new dependency or host execution.

## Symbolic protection, not TLS implementation

`ldap_protection` consumes authored endpoint/certificate facts and selected symbolic
CA keys. It models exact ASCII DNS-name matching, trusted issuer selection, an
inclusive sample validity interval and whether the modeled channel provides
confidentiality. The sample clock is a deterministic input, never the host clock.
It performs no DNS, network, handshake, cryptography, certificate parsing, chain
building, revocation, file reading, system trust or actual TLS operation.

Plain connections cannot issue protection proofs. Required StartTLS refuses an
unavailable upgrade. Optional StartTLS may yield a clearly plain outcome when the
upgrade is unavailable; certificate/trust/validity/confidentiality failures never
silently issue protected success. Only a verified, confidential symbolic outcome
can yield an opaque attempt-local proof. New endpoints with identical identifiers
are still distinct authorities; connections/proofs cannot cross those attempts.
There is no serialization, setter or learner input route for these capabilities.

Limits: canonical ldap:// identifiers256bytes, up to4exact DNS names253bytes each
with63byte labels,4symbolic CA keys64bytes each. URI and certificate-name budgets
are checked separately. IP/wildcard/IDNA/Unicode/trailing-dot names, ldaps/ldapi,
SASL, real certificates and full PKIX/protocol/cipher-suite behavior are unsupported.
Names and issuer keys are deliberately different data types/concepts: symbolic CA
keys are not X.509 issuer DNs, filenames or real trust assertions.

## Independent fictional credential state

`ldap_identity::protected::ProtectedRealm` composes the existing directory and a
new symbolic endpoint. It checks that attempt's protection before evaluating any
supplied credential. Its mutable current credential values are separate from
LDAP attributes and the immutable identity/grant authority. Existing plain Realm
APIs do not silently gain TLS flags or password-changing behavior.

Only authenticated self-service password changes are implemented. A supplied
server policy separately controls extension availability, self-service permission
and whether the current target password is required. An explicit target must be
the authenticated person's DN; a missing target means self. Administrative resets,
external/non-DN accounts, generated passwords and other policy controls are honestly
unsupported. Ordinary attribute-write permissions do not imply password-reset
rights. An identity with no ordinary write grant can still use a permitted
self-service operation. Policy is an authored source fact, never a saved answer.

All checks precede the sole credential mutation. A provided wrong old password,
missing/malformed replacement, policy refusal or foreign context changes nothing.
Successful replacement is verified with fresh authentication: replacement works,
old fails, unrelated identities and directory fields remain unchanged. A policy
without password history can set the same value, but that is not proof of rotation.
A future assignment must grade both login outcomes, not just operation status.
Already-authenticated contexts remain distinct from a fresh login; this model does
not pretend that changing a password necessarily disconnects bound sessions.

Source-supplied sample passwords allow1–128printable ASCII bytes. Debug/diagnostics
redact original, replacement and submitted values. No personal password is asked
for, hashed, stored, logged or changed. Current state and opaque contexts are not
progress inputs. Future workspaces must save only fictional profile keys/drafts
and re-evaluate against their own source cases rather than restoring proofs.

## Evidence and remaining integration

Private stage: `.build-artifacts/continuation/resume-20261109/next-ldap-password/`.
R01initial35tests pass. R02new253character certificate-name boundary fails; R03
retains the same failure because the guarded patch did not apply. R04corrected36
pass after separate DNS validation. R05all49pass: protection11, new credential13,
retained identity11and delete14. Groups overlap; do not sum milestone counts.
R06focused Clippy passes28.16s. Current-source guard checks are next. No actual OpenLDAP/TLS/server
reference execution or rendered new password workspace exists yet.

Next finite work: a real literal ldappasswd subset and distinct simulated bind,
old-target and replacement prompt inputs; then a substantive taught/practice/
independent task with behavior-based login/preservation checks, useful refusal
feedback and persisted draft isolation. Never count a helper module as a lab.

Primary references checked2026-11-09 (no code/text copied):
- https://www.rfc-editor.org/rfc/rfc3062.html — authenticated Password Modify,
  separate fields, unchanged state on failure and confidentiality requirement.
- https://www.rfc-editor.org/rfc/rfc4513.html — StartTLS/authentication/identity
  separation. This symbolic model is a restricted teaching subset.
- https://www.rfc-editor.org/rfc/rfc5280.html — inclusive validity interval;
  this model does not implement the rest of PKIX validation.
- https://manpages.debian.org/trixie/ldap-utils/ldappasswd.1.en.html — OpenLDAP
  Project manual from Debian2.6.10+dfsg-1; -W bind, -A old and -S new prompts.

## Literal password utility boundary (implementation under test)

A separate `parse_password`/`run_password` interface recognizes only `ldappasswd`.
It requires separate -x -W -S -H URI -D DN; optionally -A, either -Z or -ZZ,
and one final self DN. Options precede that operand; order is otherwise flexible,
and -- may delimit it. Password arguments/files (-w/-s/-a/-y/-t/-T), SASL,
controls, verbose/dry-run and other syntax are honestly unsupported. Existing
LDIF/delete parsers do not gain this command or TLS options. Shared literal
bounds4096bytes/32words apply; no shell expansion or process invocation occurs.

Bind, old-target and replacement prompts are separate supplied fictional values,
never host terminal reads. -A selects the old-target value; otherwise that value
is inert and the declared server policy decides whether omission is allowed.
-S requires a supplied replacement; generation remains unsupported. Missing or
malformed prompt values do not change state. Parsed invocation selects the virtual
endpoint and modeled protection, authenticates with its current credential, then
performs the typed self-service operation. A later invocation authenticates again;
stale bind data cannot succeed merely because an earlier call passed.

Report status0means the operation succeeded, not that a rotation requirement or
assignment passed. A fresh bind mismatch reports49; other local/subset/model
failures report1and a specific diagnostic. Other server-policy result codes and
full wire/utility behavior are not claimed. No output contains prompt contents.
R07literal129and R08ERE138current-source synthetic guards pass before this parser
addition; they are not genuine Bash comparisons. Authored lesson/workspace and
persistence integration are still pending, with no additional curriculum credit.

The initial utility target contains11tests.
R09all51pass (utility11, protected identity13, retained LDIFutility13/delete14).
The added twelfth utility regression makes explicit the protected subset boundary:
no TLS request or unavailable optional -Z fallback is an UnsupportedTransport
local diagnostic before authentication. It does not pretend that real ldappasswd
always refuses plaintext. -Z remains supported when modeled TLS actually succeeds;
-ZZ refuses unavailable protection. This boundary must be taught explicitly.
R10verifies the added diagnostic and all retained selected targets next.

R10all52password-utility/credential/retained tests pass, including the twelfth
utility case distinguishing unsupported plaintext fallback from real-client
behavior. R11focused Clippy passes20.95s. New virtual client settings are under
test: only TLS_CACERT and TLS_REQCERT, source-held /work/ CA identifiers, and
verified-peer demand/hard (or its omitted demand default). Unknown paths are
identifiers until separately resolved to supplied source facts; no host lookup.
Other real options/levels and duplicate settings are explicitly unsupported,
not silently implemented incorrectly. Full-line comments/blank lines, LF/CRLF,
case-insensitive option names and bounded whitespace are supported. Inline
comments, quoted/multiple CA paths, expansions and traversal are unsupported.
Bounds8KiB/64lines/1200bytes per line. No configuration/environment discovery.
Primary manual checked2026-11-09:
https://manpages.debian.org/trixie/libldap-common/ldap.conf.5.en.html
No new lab, renderer or credit; source catalog and Ready367candidate unchanged.

R12all21client/utility checks pass. R13new workspace initially failed raw-source vs canonical-directory equality; R14positive preservation regression reproduces it. R15all31pass after measuring the canonical modeled before/after state. R16workspaceClippy28.18s, R17engine/progress/retained24, R18pagecheck38.46s and R19authored/retained16pass. Source368/133 is now registered, but its new native interactions/rendered/package verification are pending. Ready367R1 remains admitted; no frozen MAINreaders replay. Source/history preserve all previous367scenario/132unit values, with128packguard unchanged. Source-backed progress/backup integration is tested with separate files; not cold reopening.

R23widgets3and R24actual capture-phase pass. R28native73actions/13PNG passes, but root screen review found a stretched table label. R30four widget/text checks pass after natural left-aligned galleys; R32actual75actions/13PNG passes and corrected originals are opened at dark100/light200. Source368/133 is integrated; isolated package acceptance is still pending. Matrix adds narrow210.3.K2/T2practice only. No actual TLS/server comparison or full-objective claim.
