> Current: R05all47new/retained tests and R06focusedClippy19.44s pass for the opt-in protected mail/description Modify adapter. Earlier pending notes below are historical. Catalog and latest admitted candidate remain368/133; no dependent lesson or objective credit yet.

# Ordered LDAP access-policy foundation

2026-11-09: additive core under validation; **no new lesson, server integration or
objective credit**. Catalog and admitted candidate remain368labs/133units. Do not
replay the completed368freeze readers after later source/doc changes. This module
neither changes the existing directory grants nor runs a server or host utility.

## Exact initial subset

A bounded static `access to` list represents one backend's already combined
ordered policy, not a whole slapd.conf, database/frontend merge or cn=config LDIF.
Headers begin in column1; continuation lines are indented `by` clauses. Only
lowercase keywords, LF/CRLF, tabs, full-line comments and printable ASCII are in
scope. Configuration escaping, inline comments and other formats fail explicitly.
DN values can be quoted for spaces; names use the existing disclosed DN parser.

Targets are `*` alone, a dn.exact/base/one/subtree/children selector, an attrs list,
or a DN followed by attrs. Eleven known names include entry/children pseudo
attributes and userPassword; this does not expose or store actual passwords.
Subjects are *, anonymous, users, self and those DN selectors. Levels are none,
disclose, auth, compare, search, read, write, manage. Default or explicit stop is
supported; continue/break, masks, regex, groups, filters, SSF, sets and unlisted
features return Unsupported, never an invented allow result.

The first matching target selects the rule; the first matching subject selects
its level. Higher levels imply lower levels. A nonempty unmatched list denies;
an empty combined list defaults to read. Configured rootDN bypass requires a
supplied authenticated matching identity. Decision traces expose one-based rule,
clause and source-header line numbers and distinguish explicit/implicit defaults.
Subjects are caller-supplied established facts: this API performs no authentication.

Bounds:32KiB,256physical lines,4096bytes per line,16rules,16byclauses per rule,
1024tokens per rule and1200bytes per token. Unsupported fields and overlong inputs
fail with bounded diagnostics, without echoing submitted content.

## Important operation boundary

A positive per-attribute result is not authorization for a complete LDAP request.
Operation adapters must enforce entry/children checks as relevant plus
schema, identity and all other prerequisites. The opt-in contact Modify adapter
now checks every changed attribute and stages all fields before committing; other
operation types remain unsupported. See LDAP_ACCESS_LESSON.md for the registered lesson. Existing scoped write grants are not relabeled as slapd ACLs.
No server, crypto, network, host settings or personal data are touched.

## Authoritative references and current evidence

OpenLDAP2.6 Administrator's Guide, access control, checked2026-11-09:
https://www.openldap.org/doc/admin26/access-control.html
OpenLDAP2.6 slapd.access(5), Debian-maintained manual, checked2026-11-09:
https://manpages.debian.org/trixie/slapd/slapd.access.5.en.html
Only a selected subset is modeled; no real OpenLDAP/slapacl reference execution
has been performed. Tests/Clippy are pending at this initial checkpoint. No
curriculum, other-platform, novice or accessibility acceptance follows from it.

R01library build47.85s, R02all15policy tests and R03focusedClippy23.26s pass.
An opt-in protected Modify adapter is now under validation. It checks established
endpoint/session ownership, stages complete mail/description requests through
the existing schema, verifies every changed attribute against the ordered ACL,
and commits only after all pass. Add/delete/rename/password operations remain
unsupported in this adapter. The existing legacy grant and password paths are
unchanged. This is not a complete server, search or schema-administration model.
No additional lesson or coverage credit is counted.

Operation details: this opt-in route uses already established protected identities.
It does not implement ACL-aware fresh Bind, search, add/delete, schema administration
or password-modify authorization. The existing password path keeps its separate
source policy. Manage/root bypass changes ACL permission only; unsupported schema
mutations remain unsupported. Validated requests are staged before permission
checks, so model error ordering is not claimed as OpenLDAP wire-error behavior.
R05tests actual state/preservation, later-attribute atomic denial, scope/self,
foreign contexts, schema/unsupported operations, rotated credentials and equivalent
attribute order. Old protected credential13and utility12checks are retained.

R09task/session library44.89s and R10six synthetic workspace tests pass. This
source is not a registered lesson: no new catalog count, unit, Goal/State variant,
progress credit or native acceptance is claimed. Tasks grade actual authorized
contact changes and full-state preservation across varied source directories.
Malformed policy cannot pass a required-denial test; fixtures validate all requests
including expected denials as meaningful schema-valid changes. Practice/drafts,
stale success, reset and bounded serialized work are checked. Plain-language rule/
permission explanations are being verified next, before authored content/UI.

## Registered delegation lesson —2026-11-09

The earlier pending milestones above are historical. Current source369registers
`lpic2-ldap-contact-delegation` with a normal native policy editor and actual
protected contact Modify outcomes:43requests in each of three-case assignment and
separate practice. R19all27registered/authored/retained, R20fourUI/capture and
R23sevennew/retained widget checks pass. R25native76actions/17PNG exits0; root
opened corrected actual Introduction/practice/Assignment/partial/light200screens.
Final package/lint/Python/coverage gates remain pending. The first15policy and
7adapter checks remain retained. This does not broaden the exact subset above or
claim a full server, real reference-tool comparison or independent novice study.

Final source gates: R29all56selected integration checks, R31all-targetClippy,
R32all989Python, R26inventory369and R28strictcoverage pass. Exact isolated package
verification remains pending; no full-objective or new platform acceptance.

Ready369R1actual package now verifies369/134, native76actions/17PNG and selected
source/data/docs identity; root9originals opened. R38receipt precedes freeze lift.
See LDAP_ACCESS_LESSON.md/CHECKPOINT.md for exact identities and acceptance limits.
Never replay frozen old MAINreaders. No capability beyond the disclosed subset.
