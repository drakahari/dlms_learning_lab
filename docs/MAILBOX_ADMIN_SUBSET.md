# Mailbox administration: deliberately bounded utility profile

Capability and one administration capstone registered in source482labs/192units,
checked against Dovecot CE2.4.1 references on2026-11-14. Focused runtime, authored,
persistence and widget tests pass; development native118actions/23captures and
normal two-process reopening pass. Named utility practice is partial211.3 coverage;
Courier remains supporting explanation. Installed482candidate acceptance remains
pending. The verified installed review build remains
481labs/191units; see CHECKPOINT.md.

## Execution and supported words

Commands run in-process against the existing typed virtual configuration/mail
store. No command reaches a host shell, process, file, daemon or socket. The
command field accepts one ASCII command with literal arguments, single/double
quotes and ordinary backslash quoting. This is a command-word subset, not Bash.
Expansion, pipelines, redirection, command lists, multiline commands, comments,
external files, socket selectors and environment/userdb lookup are not supported.
Unknown features diagnose the limitation rather than pretending success.

- `doveconf setting [setting ...]` reads the saved simulated file; `-h` hides names.
  Supported root settings: config/storage version, protocols, ssl,
  auth_allow_cleartext, auth_mechanisms, pop3_no_flag_updates,
  ssl_server_cert_file/key_file, mail_driver and mail_path. Values come from the
  actual parsed file. Unsaved editor text and running settings are different.
  Full/default dumps (`-n`, `-N`, `-a`), sections, filters, includes, alternate
  files, overrides and expansions are explicitly unmodeled. Do not teach those
  as supported commands or label the limited named queries a full configuration
  dump. Default-version data is not guessed.
- `doveadm mailbox list -u USER` lists that supplied account's flat mailboxes.
- `doveadm search -u USER QUERY` computes matches and prints GUID-shaped synthetic
  mailbox identities with actual message UIDs. These are fixture identifiers,
  not Dovecot's GUID allocation algorithm and not POP transaction positions.
- `doveadm fetch -u USER 'FIELD ...' QUERY` reads uid, flags, mailbox,
  hdr.subject and plain body. Pager-style records use form-feed separators.
  Repeated matching headers remain available. Fetch does not set Seen in this
  profile, unlike an IMAP read/write BODY[] fetch. ASCII headers/plain bodies only;
  MIME/encoded envelope processing is not claimed.
- `doveadm expunge -u USER QUERY` immediately removes matching virtual messages;
  it does not require a Deleted flag or POP QUIT. It must include one exact
  MAILBOX in this subset. Broad/all-user operations, quota, synchronization,
  userdb changes, subscriptions and new mailbox creation are unmodeled.

Queries AND together ALL, MAILBOX exact-name, UID positive numbers/commas/inclusive
ranges, SEEN/UNSEEN, DELETED/UNDELETED and SUBJECT nonempty literal text. Keywords
and ASCII subject matching ignore case; non-INBOX mailbox names retain case.
UID comparisons visit bounded actual messages, never expand huge ranges. UID `*`,
mailbox patterns, OR/NOT, dates and other keys are explicitly unsupported. Signed,
zero, malformed or overflowing UID numbers are syntax errors. Empty SUBJECT is
unmodeled rather than silently assigned an assumed upstream meaning.

The supplied administrator role is different from client LOGIN. When the virtual
service is running, administration uses its active configuration; otherwise it
parses the saved file. This represents config-socket availability with a simple
running flag, not a real Dovecot socket/process/userdb. Maildir resolution uses only
supplied account/path facts. No host path can be opened. The administration UI drops its
paused client connection on an administrative mail mutation rather than invent
concurrent-client/index semantics. Administrative removal must not fabricate the
POP-transaction observations required by the earlier protocol lesson.

## Bounds and authoring contract

Input2KiB/64words; at most16query predicates/16UID ranges/8fetch fields. Existing
facts limit users, mailboxes, messages and source/body sizes. Output ceiling32KiB;
parse/validation/unsupported/output failures retain the entire previous runtime.
Execution has bounded data loops and no learner-controlled loop construct. Store
UIDVALIDITY/UIDNEXT/message payloads retain their source identities. This is no
replacement for the bounded Bash evaluator or a route to execute learner scripts.

Before adding curriculum, test actual saved-versus-active behavior, equivalent
queries, incorrect scope, malformed/unsupported input, precise deletion, preserved
other mail, limits, persistence and stale-result invalidation. Teach named-query
syntax, quoted fetch fields, previewing matches before expunge and the actual
supported subset before independent work. Keep practice separate and ungraded.
No lesson count or issuer credit follows solely from this module compiling.

## Primary sources and verification limits

Dovecot CE2.4.1 man pages (revision0a16fcf when read):
[doveconf](https://doc.dovecot.org/2.4.1/core/man/doveconf.1.html),
[mailbox](https://doc.dovecot.org/2.4.1/core/man/doveadm-mailbox.1.html),
[search](https://doc.dovecot.org/2.4.1/core/man/doveadm-search.1.html),
[query language](https://doc.dovecot.org/2.4.1/core/man/doveadm-search-query.7.html),
[fetch](https://doc.dovecot.org/2.4.1/core/man/doveadm-fetch.1.html),
[expunge](https://doc.dovecot.org/2.4.1/core/man/doveadm-expunge.1.html).
Source-reviewed fetch header/flag/stream behavior:
[upstream2.4.1 fetch implementation](https://github.com/dovecot/core/blob/2.4.1/src/doveadm/doveadm-mail-fetch.c).
No upstream implementation code is copied or linked. No external Dovecot runtime
comparison has been performed; these primary-source checks are not daemon parity.

Courier awareness must be supporting explanation, not simulated Courier coverage:
[official IMAP overview](https://www.courier-mta.org/imap/) and
[Authlib overview](https://www.courier-mta.org/authlib/). The official IMAP README
URL returned a verification interstitial; it was not bypassed. Courier is a
separate Maildir access server with its own configuration/authentication tools;
Dovecot files/commands must not be presented as portable Courier administration.

## Authored capstone and actual Fedora checks —2026-11-14

Library → LPIC-2 → **Mailbox administration: inspect and recover** →
**Audit a pending change and clean up read notices**. Source482labs/192units;
all481preceding scenario objects/191units/134LPIC-2teaching values remain exact.
This is one compound diagnosis/recovery assignment, not several numeric variants.

Worked elin/gabe and [DRILL] data remain separate from rowan/tess and [EXPIRED].
Taught and alternate home-directory/explicit-UID solutions pass; missing SEEN,
wrong user, giant UID removal, copied example paths, partial and malformed work
fail meaningfully. Edits clear success; examples cannot award assignment credit.
The source-bound activity rejects swapped goal/state kinds and protocol-only
references reject administrative actions. Existing progress wire shapes remain.

Development native118actions/23captures passed;10originals inspected across all
three views, specific failures, Light150Save alignment and Navy200input.
Normal writer4096559/reader4097203exit0(80.933/45.729s) restored exact raw/portable/
provenance/preferences plus independent drafts, partial INBOX cleanup and unsent
Archive commands before native completion. All six originals inspected.
Selected1427inputs/88350091bytes plus actual840547800byte debugELF retained;
not a complete compiler closure or independent seal. Hold ended.

Representative actual images (private ignored evidence):
- [Introduction](../.build-artifacts/native-runs/run-2v592mwf/screenshots/workspaces/mail-admin-introduction.png)
- [Guided practice](../.build-artifacts/native-runs/run-2v592mwf/screenshots/workspaces/mail-admin-practice.png)
- [Independent assignment](../.build-artifacts/native-runs/run-2v592mwf/screenshots/workspaces/mail-admin-assignment.png)
- [Specific protected-mail failure](../.build-artifacts/native-runs/run-2v592mwf/screenshots/workspaces/mail-admin-protected-mail-loss.png)
- [Light150 aligned controls](../.build-artifacts/native-runs/run-2v592mwf/screenshots/workspaces/mail-admin-light150-save.png)
- [Restored unsent assignment command](../.build-artifacts/tmp/native-reopen-23a708c369c4620981005330/read/reopen-mail-admin-reader-assignment.png)

Finite injected native input/rendering evidence, not independent novice, physical-
device, screen-reader or Windows/macOS acceptance. Long console lines deliberately
scroll horizontally; the long command placeholder and existing theme name truncate
at narrow200%text. Text is not shrunk and controls remain reachable. No external
Dovecot/Courier oracle or complete exam-preparation claim. Verified installed481
review launcher remains in CHECKPOINT.md until separate482packaging is verified.

Final source gates:33focused runtime,7widget/capture/cold-plan,5normal-catalog,
1368inclusivePython(409.819s), one-job all-targetClippyR3warnings-denied, strict
inventory477updated/5already-meets and coverage521Partial/24Supporting/695Open.
No new inclusive Rust runtime run. Debug native/cold predates only equivalent
button binding/helper placement and final review bookkeeping; installed candidate
will verify its own ELF. Final source ready, candidate482verification pending.

## Installed482candidate acceptance —2026-11-14

Verified private481→482lifecycle, installed118actions/23captures (seven originals
viewed), actual normal writer4124244/reader4124623exit0(51.922/16.304s) and exact
raw/provenance/preferences restore before native repair. All six cold originals
viewed. Finite Fedora technical checks; no other-platform/novice/accessibility
acceptance follows. Original app/progress untouched. Correct --validate-content
and protected launcher report482labs/192units outside checkout. Source freeze
ended after final identity/readback and3129selectedfiles/136447327bytes retention.
This post-build record does not alter the package/archive or its bundled docs.

- [Installed Introduction](../.build-artifacts/native-runs/run-f7zsrwdf/screenshots/workspaces/mail-admin-introduction.png)
- [Installed guided practice](../.build-artifacts/native-runs/run-f7zsrwdf/screenshots/workspaces/mail-admin-practice.png)
- [Installed Assignment](../.build-artifacts/native-runs/run-f7zsrwdf/screenshots/workspaces/mail-admin-assignment.png)
- [Installed Light150Save](../.build-artifacts/native-runs/run-f7zsrwdf/screenshots/workspaces/mail-admin-light150-save.png)
- [Installed exact pending command restored](../.build-artifacts/tmp/native-reopen-fab7b8550981512a6b1e1f34/read/reopen-mail-admin-reader-assignment.png)

Protected launcher: `dist/teaching-review-20261114T185000Z/open-mailbox-administration-review.sh`.
Binary SHA25632ad4df83aea570b3a8ccb07ff325b0f7b2a8ead80a91e518f5d81c1ce4d8725.
See CHECKPOINT.md for archive/build-info identities and known limits.
