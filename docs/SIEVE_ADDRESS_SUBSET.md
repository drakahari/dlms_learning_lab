# Structured address filtering subset

Initial address profile implemented,2026-11-14. Two registered lessons in472labs/
187units cover sender-domain classification and recipient-routing repair. Their
individual teaching reviews,13capability/6lesson tests,11widget/workflow tests,
development native108/32 and two-process partial-work reopening pass. All470prior
values remain exact. Current source gates pass53runtime/11UI/5catalog/1327Python and one-job
all-targetClippy warnings-denied; the installed472candidate also passes private upgrade, native108/32 and normal
two-process partial-work reopening. See CHECKPOINT.md for exact protected launcher. This is partial
LPIC-2 211.2 practice, not full Sieve or objective completion.

## Language profile and compatibility

Existing Sieve lessons retain the accepted header/size/filing profile. Their task
records deserialize an absent `address_tests` as false and omit it when serializing
false. New address lessons will explicitly enable the same address profile in
both their independent and separate example task. The learner does not configure
this authoring field. No existing lab ID, requirement, draft or result is replaced.

The address profile adds the real Sieve `address` test, not a new shell command:

```sieve
address :domain :is "From" "training.test"
address :localpart :comparator "i;octet" ["To", "Cc"] "help"
```

`:all` is the default address part; `:localpart` selects the portion before `@`,
and `:domain` selects the portion after it. Matching uses the existing `:is`,
`:contains`, `:matches`, `i;ascii-casemap` default and `i;octet` comparator rules.
The default comparison folds ASCII case in the selected part; it does not apply
separate automatic local/domain case rules. Plus suffixes are retained, rather
than silently using an unsupported subaddress extension. Duplicate options fail
clearly. No `require "address"` is taught: address is a base-language test.

## Represented mailbox syntax

The bounded parser accepts plain ASCII dot-atom mailboxes, optional display names
(including quoted display names), comma-separated lists and bounded comments.
Display names and comments never become addresses. Only address-bearing fields
are searched: From/To/Cc/Bcc/Sender/Reply-To and their represented Resent variants.
Subject, unknown fields and malformed field-name arguments cannot masquerade as
address fields. Missing fields and malformed dot-atom entries cannot match.

Quoted local parts, named groups, domain literals and obsolete routes produce an
explicit unsupported-feature diagnostic. MIME/encoded/non-ASCII/folded headers
remain outside the represented message subset; no partial MIME decoding is claimed.
An unsupported address field encountered during evaluation makes that sample fail
atomically with its explicit diagnostic and recovery INBOX copy, never success.
This is a disclosed subset, not a complete RFC5322 parser or Sieve implementation.

Limits:1024 bytes/header,32mailboxes/list,16comment nesting levels, plus all existing
source/message/instruction/elapsed bounds in SIEVE_SUBSET.md. Parsing never reads
files, resolves domains, authenticates senders, sends mail or starts processes.
A visible From field is not evidence of sender authenticity. Envelope from/to is
a separate, still-unimplemented feature; never substitute visible From/To for it.

## Development evidence

Parser/profile/legacy-goal serialization, source-bound run/stop/retry, separate
drafts and restored results pass focused tests. Both authored methods, legitimate
alternatives, common wrong rules and guided experiments pass. Native/cold evidence
is recorded in CHECKPOINT.md; final source gates pass; installed472package checks also pass, including native flows and normal
two-process partial-work reopening. No independent novice acceptance is inferred. No external Sieve reference executable is available; tests are derived from
specification requirements rather than an external implementation comparison.

Sources checked2026-11-14:

- RFC5228 sections2.7.4 and5.1: https://www.rfc-editor.org/rfc/rfc5228.html
- RFC5322 modern mailbox/address syntax: https://www.rfc-editor.org/rfc/rfc5322.html
- LPIC-2 v4.5 objective211.2: https://www.lpi.org/our-certifications/exam-201-202-objectives/
