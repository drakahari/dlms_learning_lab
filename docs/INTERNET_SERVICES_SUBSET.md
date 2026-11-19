# Virtual service names and client transport subset

Prepared2026-11-12 for existing109.1, current LPIC-1 objectives5.0/101-500 and
102-500. Preserve the separate434candidate's frozen selected inputs while preparing
the next source batch. The two registered tasks and their separate examples are documented in
SERVICE_NAME_LESSONS.md; a model library alone is not curriculum credit.

The pure ASCII services-file parser and lookup evaluator use no host data. Use only
supplied virtual data: canonical name, decimal port/protocol and optional aliases,
comments and blank lines. Deliberately limit the subset to tcp/udp, simple bounded
case-sensitive names, unambiguous per-protocol names/ports and64records. Other valid
Linux file/provider behavior must produce an honest limitation, not a false claim
that Linux universally rejects it. Do not silently skip malformed/unsupported
records and pretend this models every NSS implementation.

Read-only terminal subset: cat /etc/services; getent services with no key or one
literal name/number, optionally /tcp or /udp; help. No shell syntax/options, pipes,
other databases, arbitrary virtual paths, host NSS providers, host files, sockets,
processes or subprocesses. Lookups derive canonical output and0/2exit status from
the parsed data; unsupported commands are simulator diagnostics, not fake Linux
success. Name lookup and actual supplied listener facts remain distinct. Never
teach that a service-file entry starts a process or guarantees protocol support.

Reuse transport_core for any dependent client-request behavior. Do not invent a
second transport engine. Interpret virtual client settings into bounded existing
transport Config/Exchange data, then run its finite reducer with source-owned
classifiers. A delivered transport alone must not satisfy a different required
service; grade endpoint/protocol requirements as well. No real application/TLS/DNS
messages or interoperability are implied by opaque teaching payloads.

The two authored tasks under validation are: diagnose client protocol/
service selections using the reference and supplied active listeners; then repair
a services-file mapping while preserving unrelated meanings and demonstrating that
the live listener does not move when the name database changes. Foundation model/negative/alternate/isolation tests pass; native admission and
package acceptance are recorded separately. Keep separate examples and
independent editor/terminal drafts under the approved presentation standard.

Only the actually taught/tested109.1slices may gain partial credit. IPv4 masks,
private/public distinctions, ICMP, IPv4/IPv6 differences/features and the complete
listed port/application set remain unfinished until distinct substantive practice
exists. Retain the planned mixed-IP route task; this service unit does not replace it.
No numerical variants or static reference rows count as independent authored labs.

Sources checked2026-11-12:
https://www.lpi.org/our-certifications/exam-101-102-objectives/
https://www.sourceware.org/glibc/manual/latest/html_node/Services-Database.html
https://man7.org/linux/man-pages/man5/services.5.html
https://man7.org/linux/man-pages/man1/getent.1.html
The www Sourceware manual was inspected successfully. The original glibc2.42
getent.c formatting/dispatch behavior was inspected through its archived read-only
mirror: https://github.com/bminor/glibc/blob/glibc-2.42/nss/getent.c .
No upstream code was copied. No real getent/NSS runtime comparison was performed;
host database/provider configuration is not part of development fixtures. Reference file grammar/lookup facts only; do not copy upstream code.
Do not repeat old generalizations about privileged ports or infer that an assigned
TCP/UDP registry entry means a particular application implements both protocols.
Original implementation remains MIT; no dependency or system change is proposed.

Implementation: service_names parses the supplied database; service_clients resolves
a name or numeric destination then calls the existing transport_core reducer.
service_workspace binds independent source-owned tasks, examples, editable settings,
saved file and deterministic online/outage cases. Reports are Serialize-only and
recomputed; progress stamps cannot authorize stale or wrong state. Database grading
preserves every source-required name/alias mapping while accepting comments, record
order and whitespace differences. Client tasks accept supported names, aliases or
numeric ports; numeric ports deliberately bypass a broken name database.

Limits:32KiB file,512byte line,64records,8aliases per record,64byte names/settings,
4clients,8requests/listeners,1024byte opaque payload,256byte command,20KB visible
transcript tail. Names/ports must be unambiguous within each protocol. Source roles
and listeners cannot be edited or moved by a learner. Unknown JSON fields, unsafe
paths, shell operators, other databases and unsupported syntax fail explicitly.
The help command and data-only @service-clients pack actions are simulator helpers;
pack actions are not taught as Linux commands and cannot run on the host.
