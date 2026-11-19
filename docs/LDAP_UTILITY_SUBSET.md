> Ready367R1 now admits the retained utility workflows plus DN-list retirement. Exact package/content/native verification is recorded in CHECKPOINT.md and LDAP_RETIREMENT_LESSON.md. Historical statuses below remain as recorded.

> Current status2026-11-09: Ready366R1 utility candidate is verified and retained.
> Source367adds one DN-list retirement task; its new native/package verification
> remains pending. See LDAP_RETIREMENT_LESSON.md. Historical gate notes follow.

# Directory utility subset and boundaries

2026-11-09. Development catalog366labs/131units includes two utility assignments:
repair an import command and recover a stopped update. Native candidate acceptance
is still pending. The latest admitted Ready364R2 package remains immutable and
runnable. These are narrow simulated workflows, not full objective/server coverage.
Existing Apply LDIF lessons keep their whole-document parsing behavior.

## Literal invocation

Supported names are `ldapadd` and `ldapmodify`, with explicit separate options
`-x -W -H URI -D DN`, optional `-a` and `-f VIRTUAL_INPUT`. Each option occurs once;
order is flexible. A final `--` is permitted without operands. Naming an identity
with `-D` does not authenticate it. `-W` consumes only the separately provided
fictional sample-password value; it does not read the host terminal. Password
arguments/files, anonymous or unauthenticated writes, SASL and TLS are unsupported.

`ldapadd` and `ldapmodify -a` accept content records as Add requests. Explicit
`changetype: add`, `modify` or `delete` selects the stated operation regardless of
that default. Without `-a`, this subset requires an explicit changetype; implicit
attribute replacement is honestly unsupported. This is a subset restriction,
not a claim that real ldapmodify requires changetype in every accepted input.

The shared existing literal lexer supports quoted/escaped literal words; shell
expansion, pipelines, redirection, command substitution, combined/unknown options,
positional operands and multiline commands fail. Bounds:4096command bytes,
32words,1024DN bytes and128virtual-identifier bytes; components allow64ASCII
letters/digits/dot/underscore/hyphen bytes. Only supplied `/work/` identifiers
resolve. Dot/dot-dot components, empty components, URLs and host paths fail.

## Virtual inputs and request order

Omitting `-f` selects the supplied editor buffer. Specifying it selects exactly
one supplied virtual file; no OS file, standard input, subprocess or network
lookup occurs. Up to8files,64KiB per input and128KiB combined input are borrowed
only after bounds and unique identifiers are checked. Unselected drafts are inert.
The URI selects only the supplied virtual directory; unknown targets fail without
trying to connect.

`UtilityRecords` requires explicit `version: 1` and parses records in sequence.
A later syntax/unsupported-record error stops processing and preserves earlier
successful requests. A failed directory request is atomic within itself, retains
earlier successful requests, and stops subsequent records. Comments, folded lines,
CRLF, safe base64 and detailed physical line diagnostics reuse the bounded LDIF
grammar. Content and explicit change records may follow each other in utility
add-default mode. The older document editor intentionally rejects mixed forms
and parses its complete document before applying anything.

Aggregate64KiB/1024physical-line limits fail before the first request. Up to16
records are processed; a seventeenth produces a limit diagnostic without undoing
earlier requests. No continuous mode, controls, transactions, rename, file URLs,
password operations, arbitrary schema or real LDAP server/protocol is implemented.
The restricted writable schema and request codes remain documented in
[LDAP_CHANGE_SUBSET.md](LDAP_CHANGE_SUBSET.md). A report is successful only when
at least one request succeeds and no failure occurs. LDAP request failures retain
their modeled result code; local diagnostics return1, success0.

## Fictional identity and delegated rights

`Realm` validates1–8author-supplied fictional identities against existing person
entries and delegated subtrees. Only nonempty printable ASCII sample passwords
of at most128bytes are supported. Matching uses the supported DN equality rules
and exact password comparison. Unknown identities and wrong nonempty passwords
produce InvalidCredentials49; empty/unsupported forms produce a subset diagnostic,
never anonymous write access. Use only the fictional values supplied by lessons.

Each invocation authenticates anew. Its opaque session is bound to that exact
attempt's immutable authority; it cannot be replayed into another attempt even
with the same URI and facts. There is no rebind API, connection pooling, actual
Bind exchange, encryption, password hashing/storage/management or OpenLDAP ACL
parser. Source-held grants still determine subtree, Add/Delete and mail/description
permissions for each request. A valid password cannot grant missing rights; a
retired identity cannot authenticate or keep writing through an old grant.
Identity/realm/session Debug output redacts source sample passwords; reports never
retain submitted password copies. None of these types are learner-deserializable.

## Sources and verification

Primary sources opened2026-11-09; no upstream code copied:

- [OpenLDAP2.6.10 ldapmodify manual](https://raw.githubusercontent.com/openldap/openldap/OPENLDAP_REL_ENG_2_6_10/doc/man/man1/ldapmodify.1)
- [OpenLDAP2.6.10 utility implementation](https://raw.githubusercontent.com/openldap/openldap/OPENLDAP_REL_ENG_2_6_10/clients/tools/ldapmodify.c)
- [RFC4513 §§4–5.1.3](https://www.rfc-editor.org/rfc/rfc4513.html)
- [RFC2849](https://www.rfc-editor.org/rfc/rfc2849.html) and
  [RFC4511](https://www.rfc-editor.org/rfc/rfc4511.html)

The new parser, identity, streaming and execution tests cover actual state changes,
alternatives, wrong/missing credentials, least privilege, cross-attempt replay,
partial failure/atomicity, limits, input isolation and rejected host escapes.
Retained LDAP parser/request/search checks also run. Exact gates live in
CHECKPOINT.md. No installed OpenLDAP tools/server comparison was available; no
dependency installation was attempted. This is source/reference review and
in-process regression evidence, not real-server conformance or delivered coverage.
Password-change/server/schema/ACL teaching and utility UI remain separate work.
