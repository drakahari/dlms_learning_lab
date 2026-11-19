## Installed Fedora481 candidate acceptance —2026-11-14

The isolated candidate now passes private478→481install/upgrade/uninstall/reinstall,
exact synthetic original-nine progress retention, installed250actions/53captures
and actual mailbox-specific cold reopening.12native and all six cold originals
were reviewed. Writer4061319/reader4061660exit0restore separate drafts, selected
clients, unsent QUIT and pending transaction marks before native commit/check.
See CHECKPOINT.md for exact protected launcher, hashes, resource observations and
remaining curriculum/platform limits. Earlier development sections below are
historical; they are not re-attested against later executables.

Representative **installed**, actual originals:
- [Introduction](../.build-artifacts/native-runs/run-5v51zp2c/screenshots/workspaces/mailbox-0-introduction.png)
- [Guided IMAP practice](../.build-artifacts/native-runs/run-5v51zp2c/screenshots/workspaces/mailbox-1-practice.png)
- [Independent POP assignment](../.build-artifacts/native-runs/run-5v51zp2c/screenshots/workspaces/mailbox-2-assignment.png)
- [Specific pending-deletion feedback](../.build-artifacts/native-runs/run-5v51zp2c/screenshots/workspaces/mailbox-2-pending-pop.png)
- [200% command control](../.build-artifacts/native-runs/run-5v51zp2c/screenshots/workspaces/mailbox-1-navy200-input.png)
- [Cold restored transaction](../.build-artifacts/tmp/native-reopen-d0962b9c77e468d363f1f315/read/reopen-mailbox-reader-assignment.png)

# Mailbox access: implemented subset and acceptance limits

Source catalog481/191 contains three new LPIC-2tasks. Development native250actions/
53captures passed; final source/cold/package gates are still pending. Latest verified installed review candidate remains478. These
are partial211.3practice, not a full Dovecot installation, protocol implementation
or exam-preparation claim. Do not mark Courier, doveconf or doveadm practical
coverage from this unit; their remaining work stays explicit in the matrix.

## Learner workspaces and state

The unit follows SMTP/host networking: restore protected two-account access;
inspect and repair IMAP mail without losing unrelated messages; restore POP and
commit a precise cleanup. Each has essential Introduction concepts, an annotated
worked example, separate guided practice and an independent assignment. Worked
accounts, certificate paths, endpoint names and Maildir locations differ from
assignment values. Copying the worked server file is not a passing repair.

Native controls are editor, Save configuration, Restart service, Enable at boot,
client selection, Connect, Disconnect, protocol input/Send and Check work. They
are explicitly simulator actions, not invented Linux utilities. The IMAP-only
activity keeps server configuration read-only and omits unrelated editing controls.
Insert command fills input without sending. Challenge omits unsolicited examples.
Failures distinguish configuration, service/listener, TLS, authentication and mail
state. Fresh probes use cloned current state; grading never mutates stored mail.

Saving does not apply configuration. Restart applies the saved file, invalidates
older connections and preserves mail even on startup failure. Boot enablement is
separate. Configuration edits and protocol operations clear checked results;
checking recomputes current outcomes. Equivalent supported settings/operations can
pass. A closed listener is not proof of rejecting unprotected authentication.
Other supplied messages must remain, regardless of whether target removals pass.
The POP capstone also requires the simulator to observe an authenticated POP QUIT
committing those removals: deleting through IMAP cannot substitute for that skill.
Missing observations in older development snapshots default to no earned POP
transaction credit; all other work remains, without inventing learner operations.

Practice/assignment own different drafts, servers, stores, logical connections and
output. Resume validates account/mailbox names, original UID identities, UIDNEXT,
UIDVALIDITY and original message content; only supported flags/deletions can differ.
New lesson IDs receive no inherited credit. The478preceding scenario values and
190preceding unit objects remain exact. Existing progress schema is unchanged.
Reopening pauses/resumes a **simulated** session; it does not reconnect real TCP.
Use Disconnect to model an actual dropped connection and cancel pending POP marks.

## Dovecot configuration profile

Versioned after Dovecot CE2.4.1documentation, explicit config/storage version2.4.0.
Supported root settings: versions, protocols, ssl, auth_allow_cleartext,
auth_mechanisms(plain/login), ssl_server_cert_file, ssl_server_key_file,
mail_driver(maildir), mail_path and pop3_no_flag_updates. Protocol Boolean-list
assignment replaces, whereas its block merges/removes named protocols. Supported
Boolean spellings are case-insensitive yes/y/1/no; false/0are not synonyms for no.
Comments respect double quotes. Named filters/braces use separate lines.

Only service imap-login/pop3-login and their reserved imap/imaps/pop3/pop3s
inet_listener blocks, with port/ssl values, are modeled. Port0disables. Duplicate
active ports fail startup, rather than masquerading as a syntax error. These
reserved names are a profile restriction; real Dovecot permits broader naming.
Unknown settings/filters/includes/versions/expansions diagnose Unsupported. Older
2.3ssl_cert/ssl_key/mail_location syntax is not silently interpreted as2.4.

Mail paths resolve only against supplied per-user metadata using absolute paths,
%{user}or ~/Maildir. No host path is opened. Fixed authentication/storage facts
replace actual passdb/userdb/namespace providers. This profile explicitly requires
supplied certificate/key paths even with SSL disabled; it does not invent missing
host-file behavior. Logical certificate matching checks key identity, client trust,
exact DNS name and exercise day. Key owner/mode are displayed metadata, not a model
of real key-file access. No cryptography, real keys, clock lookup, DNS or sockets.
All initial clients are remote: localhost/proxy secured exceptions are explained
as exclusions. ssl=required and ssl=yes/auth_allow_cleartext=no are equivalent for
the represented remote PLAIN login policy; no other SASL exchange is executed.

## IMAP command subset

Commands are tagged, ASCII and line oriented. Quoted strings allow escaped quote
and backslash; malformed quoting/lists are BAD. Input literals are Unsupported.

| State | Operations |
| --- | --- |
| Any open connection | CAPABILITY, NOOP, LOGOUT |
| Before authentication | STARTTLS, LOGIN user password |
| Authenticated | LIST with empty reference and flat name/*/%, SELECT, EXAMINE |
| Selected mailbox | FETCH, STORE, UID FETCH/STORE, EXPUNGE, CLOSE |

FETCH supports UID, FLAGS, RFC822.SIZE, BODY[]and BODY.PEEK[], singly or in lists.
BODY[]sets Seen in read/write mode; PEEK and read-only EXAMINE do not. STORE
supports FLAGS/+FLAGS/-FLAGS, optional .SILENT, and only Seen/Deleted system flags.
Sequence/UID sets accept positive numbers, *, comma lists and reversible ranges;
evaluation iterates supplied messages, never an integer-sized range. Absent UIDs
yield no matching FETCH data. Out-of-range sequence numbers are diagnosed.

SELECT/EXAMINE deselect the old mailbox before attempting another valid selection;
a failed selection leaves authentication but no selected mailbox. EXAMINE is
read-only. EXPUNGE removes marked messages and renumbers positions without reusing
UIDs or resetting UIDNEXT. CLOSE expunges only read/write selections, without
individual EXPUNGE replies. LOGOUT, selecting again and disconnecting do not
implicitly expunge. Output literals use canonical CRLF byte lengths, without POP
dot-stuffing. CAPABILITY says X-SIMULATOR rather than claiming full IMAP4rev1.

No SEARCH, APPEND, COPY/MOVE, MIME sections, partial fetches, arbitrary flags,
international names, nested lists, concurrent mail arrival or IMAP4rev2 claim.

## POP command subset

Authorization: CAPA, STLS, USER/PASS and QUIT. Transaction: STAT, LIST, UIDL, RETR,
TOP, DELE, RSET, NOOP, CAPA and QUIT. Required remote TLS policy rejects USER
before TLS, not just PASS. CAPA reflects USER/STLS availability. A rejected STLS
does not consume an earlier valid USER. The modeled successful USER can survive
an allowed upgrade; PASS consumes the candidate, including failed attempts, as
in the inspected Dovecot2.4.1login source. No APOP or SASL AUTH implementation.

DELE marks only. RSET clears marks; Disconnect, server generation loss and limits
never commit them. QUIT commits only an authenticated transaction. A POP number
is stable within that transaction; pending deleted entries are excluded from
LIST/STAT and cannot be retrieved. Subsequent sessions renumber positions while
opaque synthetic UIDL identities remain stable. Their text is not Dovecot's
default UIDL format and must not be treated as a position/portable UID encoding.

RETR normally sets the shared Seen flag; explicit pop3_no_flag_updates=yes
suppresses that update, as documented upstream. TOP does not set Seen. Cancelling
deletion marks is **not** rollback of earlier Seen updates. Canonical CRLF sizes
exclude dot-stuffing and the final terminator; multiline POP output is dot-stuffed.
No concurrent sessions, storage locking, actual Maildir layout or real downloads.

## Bounds and execution boundary

In-process typed Rust only; no learner subprocess, shell, network or host-file
execution route. Config8KiB/160lines; protocol line2KiB/96tokens; session128commands
and64KiBoutput, with atomic effect cancellation on exceeded output. At most4users,
4flat mailboxes each,16messages per mailbox/32total; bounded ASCII plain-message
fixtures and256KiBfacts. Scope512KiB, separate-session1MiB, console128KiB and
action16KiB. Console truncation clears older display text with an explicit notice,
preserving the complete latest reply. It does not erase stored message outcomes.
Each Send is finite; Disconnect is immediately available between bounded actions.

## Primary references and evidence

Checked2026-11-14. No external Dovecot/Courier daemon comparison was run.

- [LPI4.5official211.3objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [Dovecot CE2.4.1quick configuration](https://doc.dovecot.org/2.4.1/core/config/quick.html), [syntax](https://doc.dovecot.org/2.4.1/core/settings/syntax.html), [types](https://doc.dovecot.org/2.4.1/core/settings/types.html)
- [TLS policy and secured-connection exceptions](https://doc.dovecot.org/2.4.1/core/config/ssl.html), [testing IMAP](https://doc.dovecot.org/2.4.1/core/admin/testing.html), [POP Seen/UIDL behavior](https://doc.dovecot.org/2.4.1/core/config/pop3.html)
- [Dovecot2.4.1POP login/authentication source](https://github.com/dovecot/core/blob/2.4.1/src/pop3-login/client-authenticate.c), [TLS dispatch](https://github.com/dovecot/core/blob/2.4.1/src/login-common/client-common.c); inspected for behavior, not copied/redistributed code.
- [IMAP4rev1 RFC3501](https://www.rfc-editor.org/rfc/rfc3501.html), [POP3 RFC1939](https://www.rfc-editor.org/rfc/rfc1939.html), [STARTTLS RFC2595](https://www.rfc-editor.org/rfc/rfc2595.html). Historical cipher guidance is not a modern recommendation.

Focused model/teaching/widget evidence and failed attempts are recorded in
CHECKPOINT.md and the private next-mail-access receipts. Native/package, other-OS,
physical-device, screen-reader and independent novice acceptance remain separately
pending until actually observed. Prior approved teaching review was assisted.

## Actual development evidence —2026-11-14

Fresh74runtime/lesson/preservation and nine widget/cold-plan checks pass; complete
478scenario/190unit/131LPIC-2teaching prefixes remain exact. InclusivePython1360,
current source187/196and all-target Clippy pass. Development native250actions/
53captures passed;12originals were reviewed (all nine lesson views and selected
Light150/Navy200controls/failure). Normal debug writer4044585/reader4045159exit0
restore exact separate drafts, client choices, pending POP marks, unsent QUIT and
raw/provenance/preferences before native commit/check. All six Light150cold
originals were reviewed. Buffered logs do not diagnose startup stalls.

Representative actual originals, not mockups:
- [Introduction](../.build-artifacts/native-runs/run-d3917nk8/screenshots/workspaces/mailbox-0-introduction.png)
- [Guided practice](../.build-artifacts/native-runs/run-d3917nk8/screenshots/workspaces/mailbox-1-practice.png)
- [Independent task](../.build-artifacts/native-runs/run-d3917nk8/screenshots/workspaces/mailbox-2-assignment.png)
- [Cold restored POP transaction](../.build-artifacts/tmp/native-reopen-48b428aac5302bb1e73f11f7/read/reopen-mailbox-reader-assignment.png)

These observations predate only a readable protocol/port label correction; its
widgets/source/lint gates pass. Installed481candidate gates are pending. Original
app/progress remain untouched. Existing200%theme-name ellipsis persists. No new
inclusive Rust runtime suite, full daemon/oracle, indefinite startup guarantee,
other-OSruntime, screen-reader, physical-device or independent novice acceptance.
