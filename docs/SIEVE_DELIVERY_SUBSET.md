## Verified Fedora review build —2026-11-14

475source gates, private installation/upgrade, installed168/54native and normal
writer3846363/read3846507cold reopening pass. The corrected null-return-address
explanation was visually inspected in the installed app. All6cold originals were
inspected before this note. See CHECKPOINT.md for launcher/identities/evidence;
this post-verification note supersedes pending-package wording below and postdates
the retained bundle. No full Sieve, curriculum, novice or other-platform claim.

# Simulated Sieve envelope and delivery subset

Implemented source profile, checked 2026-11-14. Three authored delivery lessons
bring the source catalog to 475. Focused behavioral/UI and development native/cold
checks pass, as do62focused runtime,14UI,5catalog,1337inclusivePython and
one-job all-targetClippy checks. Fresh packaged verification remains pending. This deepens
partial211.2 only; it does not finish mail filtering or the certification track.
The verified472 review candidate predates this work and remains unchanged.

This additive profile retains the existing basic and address profiles. It adds
`require "envelope";` and envelope tests of `from` or `to`, with the existing
address parts, comparators and match types. The input is the simulated SMTP
delivery metadata, never a substitute read from visible From/To/Cc headers.
One recipient represents the delivery to this user. An empty reverse path is
the null sender and matches an empty string for every address part.

Envelope and redirect addresses use one bare ASCII dot-atom mailbox, at most
254 bytes. Display names, comments, brackets, quoted local parts, address groups,
domain literals and obsolete source routes are outside this disclosed subset;
unsupported forms fail rather than being treated as another mailbox.

`redirect "address";` records a virtual forward to one of at most 16 explicit
sample destinations. It cancels implicit keep but does not stop later rules;
an explicit keep also retains INBOX. Duplicate destinations coalesce. Destination
comparison preserves local-part case and ignores domain case under this site's
policy. At most four redirect actions may execute, including duplicates. A direct
forward to the current envelope recipient is a loop error. The lesson models a
single delivery hop, not a complete SMTP routing graph. The input message and
reverse path remain unchanged; no socket, DNS lookup, real delivery or bounce
is performed.

`require "reject"; reject "reason";` records a refusal and cancels implicit keep.
It is distinct from silent discard and does not automatically stop processing.
Executing reject twice is an error. This simulated site's strict policy also
refuses to combine reject with keep, fileinto or redirect in either order.
That combination policy follows the RFC's recommendation; it is not presented
as a universal mandatory rule. Discard can coexist with refusal. The represented
outcome is a refusal decision only: SMTP response framing, DSN/MDN generation,
backscatter policy and delivery transport are not implemented or assessed.

All effects are atomic per sample. A syntax/unsupported/runtime/resource error
awards no credit and retains the original in virtual INBOX with a diagnostic;
no proposed forwards or refusals survive. The existing 16 KiB source, bounded
parser, 32-action, comparison-work, message-size and elapsed-time limits still
apply. `:copy`, vacation, variables and other extensions remain unsupported here.
Stateful vacation handling and mail retrieval are separate remaining roadmap work.

Sources checked 2026-11-14: [RFC 5228](https://www.rfc-editor.org/rfc/rfc5228.html),
especially sections 2.10, 4.2 and 5.4; and
[RFC 5429](https://www.rfc-editor.org/rfc/rfc5429.html), especially section 2.4.
Tests are specification-derived, not comparison with an external Sieve engine.


## Teaching and authoring

LPIC-2 → Mail filters: delivery envelopes, forwarding and refusal follows the
structured-address unit. The three new tasks use stable identifiers
`lpic2-sieve-envelope-recipient`, `lpic2-sieve-redirect-recovery` and
`lpic2-sieve-rejection-priority`. Each teaches essential distinctions before a
separate four-sample worked repair and an independent8/8/11-case assignment.
The starters retain visible-recipient, unwanted-copy and silent-discard/priority
faults. No reference is inserted or run in the assignment automatically.

Delivery tasks opt in with `delivery_actions` and `address_tests`. Each case
supplies a closed typed envelope; `redirect_targets` restricts virtual forwarding.
Expected local copies, forwards and refusal decisions are graded together; any
nonblank refusal explanation is accepted. Source-bound replay verifies saved
results. Default-false/empty fields serialize as absent for old basic/address
lessons. All472prior scenarios,187unit objects and125LPIC-2teaching entries remain
exact. Reference actions remain internal authoring data, never shell commands.

Development native evidence is under ignored next-sieve-delivery receipts:
168actions/54captures,16original views inspected, then two normal processes
3816950/3817201 restored distinct partial drafts and10/11 plus3/4results before
fresh repairs. All6cold originals inspected. One earlier interrupted attempt143
is retained and not counted as a pass; sampled pressure was zero. A source wording
correction distinguishes null return addresses from delivery recipients; fresh
rendered confirmation and candidate verification follow. These checks are not
physical-device, screen-reader, novice-study or other-OS acceptance.
