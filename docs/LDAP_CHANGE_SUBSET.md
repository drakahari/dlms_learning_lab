# Directory changes: bounded LDIF and in-memory requests

Designed2026-11-09 for remaining LPIC-2 210.3 update/import skills. The verified
362-lab read-only client candidate remains immutable. The additive development catalog now includes two directory-data exercises; native
and package validation are pending. No utility, bind or server-administration
coverage follows from parsing or simulator editor actions.

The parser, typed request evaluator, source-held grants and three-case tasks are
implemented. Parsing is never execution. No host path,
process, network, actual directory service or personal credential is accessed.
No new dependency or server architecture is required for these pure capabilities.

## Parser contract

Require explicit version1. A document contains either entry-content records or
add/delete/modify change records, not a mixture. Names use the existing parsed
ASCII DN subset. Support LF/CRLF, comments, single-space folded lines, plain safe
values and standard base64 encoding of supported ASCII values. Unfold before
ignoring comments; preserve actual line numbers in diagnostics. Repeated attribute
lines collect values; the parser does not silently deduplicate them.

A Modify record supports ordered add/delete/replace of mail or description.
Empty delete/replace lists are meaningful; add requires values. Other attributes,
renaming, controls, file/other URL references and unimplemented operations produce
explicit diagnostics. In particular `:< file:///...` never opens a resource.
Binary/non-ASCII/control/empty attribute values are outside this first subset.
A base64 value representing printable text remains text; it is not an instruction.

Limits:64KiB raw document,1024physical lines,2048bytes/unfolded line,16records,
16attributes/entry,16values/attribute,16edits/request,256bytes/attribute value,
1024bytes/DN. Input length is checked before unfolding/decoding; decoded data is
bounded before allocation growth. Typed request consumers require their own shape
validation; these parser limits do not validate arbitrary hand-built enum values.

## Implemented request and authorization boundary

The separate `ldap_changes::Directory` constructor checks its controlling subset:
`domain` requires one valid dc domain label; organizationalUnit requires ou; person
requires cn and sn. organizationalPerson/inetOrgPerson inherit the person chain;
uid/mail belong to inetOrgPerson in this subset. Only objectClass, those class
fields and description are allowed. Superclasses are made explicit in this new
write context; old read-only fixtures/queries retain their existing behavior.
This is not a general schema parser or a server-schema administration lesson.

Add combines the parsed first naming value with the supplied attributes, preserving
original decoded spelling and declared equality, then validates the resulting
entry. Duplicate names, missing immediate parents and resource exhaustion fail.
Only person creation/deletion is modeled; container/root management, naming/class/
password modifications and rename/move are explicit exclusions. Delete removes
only a leaf. Modify supports ordered Add/Delete/Replace of mail and description.
Add preserves existing values and rejects an equivalent existing value. Delete
needs an existing attribute/value. Empty Replace removes an optional attribute
when present and is ignored when absent. A failed edit rolls back the entire
Modify; the completed request must satisfy the schema and all state limits.

A document stops at its first failed request. Earlier successful requests remain;
later requests are unattempted. No whole-file transaction, continuation mode or
protocol/session semantics are invented. Supported result categories are explained
with their modeled meanings (16/20/32/50/65/66/68); unsupported syntax uses a
simulator diagnostic, rather than a guessed protocol result.

A private grant supplies an existing person identity, parsed delegated subtree and
explicit create/delete/mail/description rights. Every request rechecks that fact;
LDIF cannot create or change grants. This is an already-established scenario
identity, not a real bind, password check, TLS exchange or OpenLDAP ACL. The native
Apply LDIF control is labeled as a simulator action, never a Linux command. No
ldapadd/ldapmodify/ldapdelete/ldappasswd utility or210.4 server coverage is claimed.

The editor limits drafts to16KiB. Every Apply and every grading case starts from
that sample's original virtual directory. Record results and actual final state
are displayed separately from worked expected previews. All requests must succeed
and the complete final directory must match the source-held expected state, using
declared value equality, not source-string/field-order matching. Varied samples
exercise absent fields, several old values and changed unrelated rosters.

Separate practice/assignment drafts, checked-input snapshots, selected sample and
view are versioned locally. Edits clear stale success and transient results; Run
never awards credit. Same-process save/read and backup/restore are tested separately
from any future cold reopening. Session144KiB permits JSON escaping of four16KiB
drafts/snapshots. Output32KiB, at most128entries,16records/edits/fields/values and
existing aggregate source bounds keep all execution finite. There is no loop,
script, file-resource resolver, host subprocess or transport entry point.

Development evidence and exact tested limits are in [lesson notes](LDAP_CHANGE_LESSONS.md).

Primary sources checked2026-11-09:

- [LPI current LPIC-2 v4.5 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/),210.3/210.4; query practice does not cover remaining data management/server administration.
- [RFC2849](https://www.rfc-editor.org/rfc/rfc2849.html): LDIF record structure, unfolding, encoded values and change records.
- [RFC4511](https://www.rfc-editor.org/rfc/rfc4511.html),§4.6–4.8: request semantics; modifications within one request differ from a batch of requests.
- [OpenLDAP2.6.10 ldapmodify/ldapadd manual](https://github.com/openldap/openldap/blob/OPENLDAP_REL_ENG_2_6_10/doc/man/man1/ldapmodify.1): default stop-on-error and separate continuous mode. No upstream code or prose imported.
- [OpenLDAP2.6 access control](https://www.openldap.org/doc/admin26/access-control.html) and [RFC4513](https://www.rfc-editor.org/rfc/rfc4513.html): authorization and authentication remain separate future capabilities.

This contract is deliberately narrower than LDIF/OpenLDAP. Full Unicode, operational
attributes, controls, rename/move, broader schema, passwords, protocol, TLS, SASL,
server configuration and ACL administration remain open. No platform, novice or
accessibility acceptance follows from a pure parser test.
