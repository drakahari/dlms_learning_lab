# LDAP directory-search foundation

Designed2026-11-09 for the recorded LPIC-2 210.3 client-search gap. This is a
capability before dependent lessons: no new catalog lab or objective coverage
credit follows from a parser test.210.4 schema/ACL administration remains planned.

## Deliberate first subset

An in-process, read-only directory contains supplied entries and attributes.
A distinguished name (DN) names an entry; the search base chooses where to begin,
scope chooses the base itself, immediate children, or the entire subtree, and a
filter chooses matching entries. Names are parsed components, never host paths.

DNs support at most16 single-valued relative names using cn,ou,uid,dc. Values use
printable ASCII, with required special-character escapes and hexadecimal escapes.
Equality uses these attributes' declared case-ignore matching rules, including
ASCII space preparation; it is not universal DN canonicalization. Multivalued
relative names, OID aliases, BER forms, Unicode preparation and legacy quoted DN
syntax are explicitly unsupported. Empty DN/root DSE is outside this first subset.

Filters support equality, presence, AND, OR and NOT. Unrecognized attribute types
and unknown objectClass names evaluate Undefined; NOT does not turn Undefined into
True. Only True selects an entry. Empty equality assertions, substring, ordering, approximate and extensible
filters are diagnosed as unsupported in this first increment. No silently guessed
matching rule. Known text attributes are cn,ou,uid,dc,mail,description,sn; objectClass
uses a small named schema registry. All text is printable ASCII. No passwords,
operational attributes, aliases, referrals, ACLs, schema inheritance inference,
updates, authentication, TLS or protocol transport are implemented here.

The immutable model validates bounded fixtures and RDN/value consistency, duplicate
names/attributes/values and missing parents within the supplied naming context.
It does not claim to replace full LDAP object-class schema validation. Search has
explicit noSuchObject for a missing base; zero matches is successful empty output.
Result order is deterministic fixture order, not a claimed LDAP ordering guarantee.

Limits:512KiB fixture JSON,128entries,16attributes/entry,16values/attribute,
256bytes/value,1024bytes/DN,4096bytes/filter,16filter nesting levels and128nodes.
The pure model returns typed selected entries with a bounded result size. There
are no sockets, files, host processes, user loops or external library dependencies.
CLI/configuration adapters require their own documented supported syntax and tests
before a lesson can teach them. No real ldapsearch/server reference run is claimed.

## Primary references checked2026-11-09

- [LPI current LPIC-2 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/): version4.5,201-450/202-450;210.3 queries/updates and210.4 directory/schema/ACL administration.
- [RFC4511 §4.5.1](https://www.rfc-editor.org/rfc/rfc4511.html): search scopes and three-valued filter decisions. Unknown types are not Boolean False.
- [RFC4514](https://www.rfc-editor.org/rfc/rfc4514.html): DN string structure and escaping; string representation is distinct from equality.
- [RFC4515](https://www.rfc-editor.org/rfc/rfc4515.html): parenthesized filter syntax and escaped assertion octets.
- [RFC4519](https://www.rfc-editor.org/rfc/rfc4519.html) and [RFC4524](https://www.rfc-editor.org/rfc/rfc4524.html): declared attribute matching rules, including uid and case-insensitive mail equality. No full schema enforcement is claimed.
- [RFC4517](https://www.rfc-editor.org/rfc/rfc4517.html) and [RFC4518](https://www.rfc-editor.org/rfc/rfc4518.html): matching and string preparation; full Unicode behavior remains excluded.
- [OpenLDAP2.6.10 ldapsearch(1) source](https://github.com/openldap/openldap/blob/OPENLDAP_REL_ENG_2_6_10/doc/man/man1/ldapsearch.1) and [ldap.conf(5)](https://github.com/openldap/openldap/blob/OPENLDAP_REL_ENG_2_6_10/doc/man/man5/ldap.conf.5): inspected upstream manuals for a later bounded command/configuration adapter. No upstream implementation or manual prose imported. Earlier official man-CGI access failed; versioned upstream sources resolved the reference gap.

[RFC2798](https://www.rfc-editor.org/rfc/rfc2798.html), checked2026-11-09, identifies inetOrgPerson; fixtures explicitly supply its parent classes. The model does not infer class inheritance.

Existing360labs, progress, themes and accepted workspaces remain unchanged by this
foundation. The separately verified360Fedora candidate is immutable; subsequent
source work is not included in that package.

## Bounded client adapter, before dependent teaching

The virtual configuration document supports one URI and one BASE, comments and
blank lines; option names are case-insensitive. Unknown options and duplicate
settings are rejected as outside this subset. No home/environment overrides or
host files are read. A URI identifies one of at most four source-supplied virtual
directories; an unknown identifier reports simulated unavailability. Only the
explicit ldap://host[:port] shape is supported, not TLS/IPC or URL query components.

The command workspace supports `ldapsearch -x -LLL [-H URI] [-b DN]
[-s base|one|sub] 'FILTER' [attributes]`. This first interface requires -x,-LLL and
an explicit filter. -x selects simple rather than SASL authentication in the real
utility; these lessons use an anonymous read-only model without a bind exchange.
-LLL selects LDIF without comments/version. Named attributes, quoted '*' and 1.1
have distinct projection behavior; unknown requested attributes produce no values.
An omitted -s uses subtree. Command options override virtual config defaults.
Literal single/double quoting and escapes are tokenized in process; expansion,
operators, globbing, scripts, credentials and unlisted options fail honestly.
Configuration is8KiB/64lines; command4KiB/64arguments; output32KiB. Exceeding output
limits fails explicitly rather than displaying a truncated successful result.
[LDIF RFC2849](https://www.rfc-editor.org/rfc/rfc2849.html), checked2026-11-09,
informs plain/base64 output for supported values; no LDIF import, file URLs or
change records are implemented. No OpenLDAP code/manpage text was imported.
