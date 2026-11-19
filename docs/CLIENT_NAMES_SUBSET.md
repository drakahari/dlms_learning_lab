# Client-name simulation subset — authored unit verification pending

Next existing roadmap slice: LPIC-1 v5.0 109.4, local name resolution and source
order. The bounded pure NSS/hosts model and separate source sessions pass9+7 focused
groups and scoped Clippy, with122Python before lesson integration. Two authored
lessons are now registered in source250/70, development native acceptance recorded; final candidate acceptance pending.
Existing host/network/DNS engines stay unchanged.

Scope: editable hosts-database excerpt of nsswitch.conf, services files and dns,
up to two distinct services, documented default SUCCESS=return and other statuses
=continue. Explicit NOTFOUND/UNAVAIL/TRYAGAIN return/continue and SUCCESS=return.
Bracket keywords case-insensitive, service names literal lowercase. SUCCESS=continue,
negation, merge, other databases/modules and malformed/unknown forms refuse rather
than silently approximate glibc. IPv4 hosts rows: canonical name plus bounded aliases,
case-insensitive matching, no duplicate names across rows (ambiguous real behavior
outside scope). Supplied finite DNS A-answer/status snapshots, no network or OS NSS.

Lookup produces a trace of backend status/action and final address/unresolved
outcome, separate from objective checks. No search/ndots, resolv.conf routing,
multiple A records, IPv6, CNAME/DNSSEC, reverse lookups, caching, daemon/modules,
real timeouts/retries or real getent/library calls. Those remain future model work.
No hosts-file line is treated as a command/path. Fixed resource budgets at parser,
context, lookup and serialized session admission; configuration drafts separate
from readonly supplied DNS inputs. Error syntax/unsupported/limit categories explicit.

Authoritative sources read2026-11-02:
- GNU libc latest manual reports version2.44:
  https://sourceware.org/glibc/manual/latest/html_node/index.html
  https://sourceware.org/glibc/manual/latest/html_node/Actions-in-the-NSS-configuration.html
  https://sourceware.org/glibc/manual/latest/html_node/Services-in-the-NSS-configuration.html
- Linux man-pages hosts(5), current upstream mirror:
  https://man7.org/linux/man-pages/man5/hosts.5.html
- LPI v5.0 109.4 (current101-500/102-500):
  https://www.lpi.org/our-certifications/exam-101-102-objectives/

Original teaching paraphrases these references; no manual redistribution/import.
Documentation is not implemented practical coverage or installed-libc conformance.

## Source/session gates

Source-session budgets:512KiB definition,300KiB serialized session; exactly two
16KiB files per scope, four samples/twelve lookups each and64cooperative ticks.
One lookup is evaluated per phase; Stop discards unfinished results and current
credit. The session budget includes worst-case JSON escaping for separate
practice/assignment and checked copies. Tested marks are current-draft markers,
never trusted stored success: lookup outcomes are derived again. The special
shared localhost preservation record does not copy assignment-specific practice
names; all other worked query names are distinct.

The exact248lab/69unit object preservation group passes. Four fast content/
persistence groups pass after including the actual host-resolver prerequisite
pack in the focused fixture. The earlier missing-prerequisite failure remains
retained. Alternative explicit default actions/comments work; wrong source order,
return-on-miss, static DNS copies, malformed/unsupported input and stale edits
fail honestly. Practice and assignment work remain separate, with safe writes,
export/restore and selected reset. Actual native acceptance remains pending.

Final model11/session7/content4 groups and scoped Clippy pass; an earlier separate
exact248/69 preservation group is reused.124Python pass before the guide-bundling
list refinement. Native122steps/28PNG run-mie0rioa pass; all six normal teaching
views, mixed feedback and light200 controls inspected. Missing prose arrow glyph
and redundant long table cells corrected afterward; the final package must show
those corrections. See CLIENT_NAME_LESSONS.md for acceptance boundaries.
