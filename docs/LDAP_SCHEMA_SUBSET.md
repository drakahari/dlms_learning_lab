# Bounded LDAP schema foundation

Implemented2026-11-09 and exercised by the registered native schema-extension
lesson. Source catalog371labs/136units; Ready371R2is the admitted isolated Fedora review package. This bounded subset does
not implement a complete OpenLDAP server or stock schema.

`ldap_schema` parses real descriptions, registers types/classes atomically and
validates actual in-memory entries. `ldap_ldif::schema` is an opt-in Add route;
ordinary contact LDIF, the fixed controlling schema, access-policy/configuration
workspaces and their saved formats retain their behavior. No learner text reaches
host files, subprocesses, services, modules, credentials or network connections.

## Declared subset

- Numeric OIDs, optional NAME aliases and DESC, attribute SUP/EQUALITY/SYNTAX/
  SINGLE-VALUE, and ABSTRACT/STRUCTURAL classes with SUP/MUST/MAY. Fields have
  defined order and separators. Whitespace between tokens may include tabs or
  line breaks in the native description parser; LDIF has its separate folding
  rules. Short descriptors allow32characters; descriptions2048bytes/four aliases.
- Directory String, IA5 String and Integer syntax. Printable ASCII only: Unicode
  and controls report Unsupported instead of receiving approximate preparation.
  Directory String cannot be empty. Integer is canonical signed decimal, with no
  plus, leading zeros or negative zero; magnitude is not limited to a machine
  integer. All actual values are bounded separately to256bytes.
- caseIgnoreMatch/caseExactMatch, caseIgnoreIA5Match/caseExactIA5Match and
  integerMatch, identified by supported names or numeric OIDs. Equality folds
  insignificant spaces, and letter case where specified. No substring/search
  claim follows from these equality rules. Undefined matching is not invented.
- SYNTAX{N} is a suggested minimum supported capacity, **not a maximum value
  length**. Values longer than Nremain valid within the independent256byte limit.
  A requested minimum above that resource limit is refused as Limit.
- A small authored projection of name, cn, sn, description, uid, top and person
  provides prerequisites. It is not a complete stock schema. Unsupported matching
  fields and optional person attributes are absent. Entry class membership is a
  dedicated typed list; it supplies the modeled objectClass requirement of top.
  No generic objectClass attribute syntax or operational attribute is invented.
- Definitions are immutable. Existing OIDs/names cannot be replaced; modeled
  syntax/matching identifiers and objectClass metadata OIDare reserved. Register
  dependencies before users; forward references are Unsupported. Syntax or
  equality refinement and subtyping a SINGLE-VALUEparent are not implemented.
- Classes inherit required/allowed attributes, must derive from top and follow
  one structural chain. Abstract classes cannot derive from structural classes.
  An entry cannot assert an unrelated abstract class or change its existing
  structural class during replacement. Allowed attribute subtypes can satisfy
  required supertypes. Missing, disallowed, repeated and equality-duplicate values
  fail with field-specific feedback. Accepted values are actually stored; failed
  Add/replacement leaves prior entries unchanged. Naming uses the existing bounded
  cn/ou/uid/dc DNparser, including escaped naming values. No DITplacement/parent-
  existence, schema-aware search or access/authentication claim is made here.

## Schema Add and limits

The opt-in LDIFroute requires version: 1and changetype: add for an unindexed
`cn=name,cn=schema,cn=config`, one matching cnand one olcSchemaConfig class value,
plus olcAttributeTypes/olcObjectClasses. Names allow32descriptor characters. Values
can be unfolded/base64-decoded through the existing bounded codec. All-unindexed
values or contiguous ordered {0}, {1}, ... values are accepted. Other branches,
Modify/Delete, URLvalues, controls and arbitrary configuration fields are refused.
Successful Adds get an actual simulated indexed inventory name.

LDIFenvelope syntax is preflighted before application. Each schema Add is atomic;
a later semantic/definition failure retains earlier successful Adds and reports
both applied records and the failure. The whole document is **not a transaction**.
The client preflight is a disclosed model choice, not general ldapadd streaming
conformance. Eight added schema entries,32definitions per Add and64total
attribute/class definitions (including base),32data entries,16classes/32fields
per entry,16values per field and16KiBcombined entry data bound work/memory. Parent
LDIF64KiB/1024physical-line/2048logical-line-byte limits also apply.

AUXILIARY, DITcontent/structure rules, name forms, operational/collective/obsolete
fields, arbitrary stock files, substring/ordering rules, new syntax implementations,
full Unicode preparation, server deployment and wire behavior remain unsupported.
No external server or reference tool was executed to establish interoperability.

## Sources and evidence

Primary references checked2026-11-09: [RFC4512](https://www.rfc-editor.org/rfc/rfc4512)
(description grammar, inheritance and structural rules),
[RFC4517](https://www.rfc-editor.org/rfc/rfc4517) (syntax/matching),
[RFC4518](https://www.rfc-editor.org/rfc/rfc4518) (preparation),
[RFC4519](https://www.rfc-editor.org/rfc/rfc4519) (minimal base definitions),
[OpenLDAP2.6schema guide](https://www.openldap.org/doc/admin26/schema.html) and
[RFC5612](https://www.rfc-editor.org/rfc/rfc5612) (documentation enterprise32473).
Example OIDs under32473are fictional documentation values, not production namespace
assignments. No OpenLDAP schema file/code or desktop/server configuration imported.

Focused Rust tests cover parser boundaries/diagnostics, alternate aliases/OIDs,
actual registry/data changes, invalid schemas/values, atomic failures, equality,
large integers, resource limits, escaping/folding and legacy-route separation.
This is not native lesson, platform, accessibility or novice acceptance. See the
checkpoint for actual current gates and the still-admitted review candidate.

Description syntax-error coordinates refer to the unfolded attribute/class value;
LDIF envelope coordinates refer to physical input lines. There is no complete
source-map back to folded editor positions in this bounded model.
