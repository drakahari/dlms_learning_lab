# Local mail routing subset

The model follows the cited Postfix local-recipient and forwarding contracts for
a finite set of supplied users. It never opens a host file, starts an MTA, sends
a network message or runs a program from a mail document.

## Configuration

The virtual aliases file accepts up to 16 entries of the form
`name: recipient, recipient`. Names begin with an ASCII letter and contain up
to 32 ASCII letters, digits, underscores or hyphens. Lookup folds names to
lowercase. A line has one alias and one to eight comma-separated local targets.
Blank lines, whole-line comments and CRLF are supported. Source files are
bounded to 4 KiB and 32 lines.

A supplied user's virtual .forward file contains its recipient list. Multiple
lines and whole-line comments are accepted, with at most eight recipients in
total. An empty file leaves ordinary local delivery. Including the owner's
name requests a local copy alongside the other forwarding destinations.

Alias keys are separate from the supplied usernames in this first subset.
User-name alias collisions, remote addresses, quoted names, address extensions,
file delivery, program delivery, includes, duplicate alias definitions and
indented continuations fail with explicit diagnostics. Their real behavior is
outside the modeled grammar.

## Saved policy and delivery

The editor, saved documents and aliases lookup have distinct state. Save file
validates the selected virtual document atomically. Rebuild aliases represents
newaliases and replaces the lookup from the saved alias text. A .forward file
is read from its saved text on each test. Lookup changes are immediate in this
exercise. Actual daemon cache timing is outside the model.

A test sends independent supplied sample messages through the local-recipient
graph. Aliases can point through other aliases. Loops and missing destinations
produce failures while independent destinations remain observable. Duplicate
final recipients are suppressed. Expansion is bounded to 128 steps per sample,
with at most eight users and eight sample messages per task.

Grading compares destination sets and failures for every sample. It checks
both the current lookup and the saved alias policy after a rebuild. A currently
working lookup cannot conceal a broken saved file. Equivalent supported alias
paths, ordering, comments and case can pass. Edits clear the current test, and
unsaved documents cannot retain completion credit. Example and assignment files
are independent. Restore validates source-bound metadata and reconstructs
deliveries from immutable task inputs and the saved state.

There is no SMTP, remote transport, queue, message-body/mailbox format or general
mail, mailq or sendmail implementation. The test action is labeled in the native
UI. It is an in-process simulation action.

## References

Checked on 2026-11-10. The postfix.org aliases and local manual URLs returned a
tool 402 error. The maintained Postfix source maintainer's manual mirror was
read successfully. No third-party manual text or executable was imported.

- [LPIC-1 v5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
- [Postfix aliases manual](https://vdukhovni.github.io/postfix/aliases.5.html)
- [Postfix local delivery manual](https://vdukhovni.github.io/postfix/local.8.html)
- [Postfix Sendmail compatibility manual](https://vdukhovni.github.io/postfix/sendmail.1.html)
