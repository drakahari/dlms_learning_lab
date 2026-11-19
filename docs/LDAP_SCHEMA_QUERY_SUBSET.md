# Custom-schema filter evaluation

Implemented in the opt-in `ldap_schema::Filter` and `Directory::evaluate_filter`.
The fixed legacy query/client evaluator retains its declared behavior. No learner
input reaches host files, commands, subprocesses, services or network targets.

## Actual behavior

The existing bounded RFC4515 parser supplies equality, presence, AND, OR and NOT.
Attribute descriptors resolve through the actual registry, including aliases and
compatible inherited subtypes. Stored fields use canonical OIDs. Equality follows
the registered syntax and matching rule: case-ignore/exact Directory String,
case-ignore/exact IA5 String and canonical Integer within the existing ASCII model.
Text rules ignore insignificant ASCII spaces. Integers are not converted to a
machine-sized integer. No equality rule or invalid assertions produce Undefined.
A known absent field is False; an unknown field is Undefined. Presence needs no
equality rule, but the entry model still refuses stored values without one.

AND lets False dominate Undefined; OR lets True dominate it; NOT keeps Undefined.
Only True selects an entry. objectClass uses actual inherited class OIDs: a
recognized descriptor resolves to its OID, an unresolved descriptor is Undefined,
and a valid unknown numeric class assertion compares False. This is search-filter
truth, not the result code of an LDAP Compare operation.

Public evaluation remains on an owned Directory, keeping its registry and entries
together. Parsing allows4096bytes,16levels,128nodes and256-byte assertions. The
registry/entry limits still apply. Numeric attribute aliases, attribute options,
empty equality assertions, substring/ordering/approximate/extensible filters,
non-ASCII/controls and arbitrary matching-rule implementations fail explicitly.

## Consumer activity and bounded grading

The consumer workspace loads one supplied-name schema beneath its documentation
OID branch, then actually loads the supplied person entries and evaluates the
learner's filter. It has at most4rosters of12entries and8rule probes; schema and
filter drafts allow8192/4096bytes. All typed bounds are checked before cloning.
Accepted data has the real person inheritance chain, not merely copied cn/sn rules.
Three authored rosters test different people, case/spacing, missing fields and
multiple team values. Comparison probes check declared behavior, not rule strings.

An additional named check evaluates actual entries after cn/sn/uid and DN changes,
while preserving team/state/class data. Deterministic names avoid all decoded
filter equality literals under the broadest supported text preparation. At most
512candidate names per request are considered; the bounded grammar and data make
this finite. Enumerating the original people or guessing verifier names cannot
substitute for selecting by the requested facts. Original roster outcomes remain
independently correct. Harmless identity-presence checks and equivalent supported
Boolean/schema-rule forms still pass. Generated verification inputs are not labs.

A failed Add/preflight never becomes a successful exclusion. Reports distinguish
not run, actual zero matches, missing/extra results, Undefined and rule failures.
Practice cannot earn assignment credit; changes clear current output/checks. Saved
checked drafts are regraded, and transient observations are not persisted.

## Deliberate limits and sources

This activity evaluates all supplied entries. It does not implement a complete
LDAP Search request: no base existence/scope, projection, referrals, access policy,
Bind, protocol traffic, server lifecycle, stock schema or full Unicode preparation.
No actual OpenLDAP server/tool comparison was performed. The in-memory schema
subset and its limits remain in [LDAP_SCHEMA_SUBSET.md](LDAP_SCHEMA_SUBSET.md).

Primary references checked2026-11-09: [RFC4511§4.5.1.7](https://www.rfc-editor.org/rfc/rfc4511#section-4.5.1.7),
[RFC4512](https://www.rfc-editor.org/rfc/rfc4512),
[RFC4515](https://www.rfc-editor.org/rfc/rfc4515),
[RFC4517](https://www.rfc-editor.org/rfc/rfc4517),
[RFC4518](https://www.rfc-editor.org/rfc/rfc4518) and
[RFC5612](https://www.rfc-editor.org/rfc/rfc5612). No external code/schema files
were imported. This is a bounded learning model, not LDAP interoperability or
complete issuer-objective acceptance.
