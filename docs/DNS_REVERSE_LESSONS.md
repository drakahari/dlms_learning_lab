# Reverse-DNS teaching proposal

Original source authored and independently reviewed 2026-10-25. Three distinct
assignments are integrated into `scenarios/dns-reverse-v1.json` for validation.
The eleven model/content tests pass; actual native flows remain pending.
Source inventory is119/29; the last scoped installed artifact remains116/28.
The frozen source proposal/review tables and original116 snapshot remain under
`tests/fixtures/` for provenance and preservation checks. No coverage promotion
or capability change is claimed by source integration alone.

The unit is `dns-reverse`, after `dns-forward`, with that unit as its
advisory prerequisite. The first scenario requires the existing **Complete a DNS
service handover** (`lpic2-dns-handover`), followed by a chain through these three
new tasks. No original scenario ID/version/initial State/Goal/solution/unit is
changed; `dns-reverse-original116.json` preserves those exact original fields and
the complete original unit map. Historical Goals are decoded to the typed Goal
before comparison, so omitted legacy defaults are not mistaken for edits.

| Proposed task | Distinct work | Named preservation |
| --- | --- | --- |
| Repair names returned for connection-log addresses | Diagnose a misplaced owner and A-only reverse data; construct IPv4 questions and publish two direct PTR sets | Saved .53 PTR, its primary answer, saved apex NS |
| Publish reverse names for an IPv6 rollout | Expand 32 hex digits, reverse individual nibbles and pair two full questions with supplied host names | Saved ::53 PTR, its primary answer, saved apex NS |
| Retire an old address and hand over both copies | Move a service, repair a legitimate multi-PTR set, prove no old PTR data on each copy and synchronize publication | Saved .53 PTR, both served answers, saved apex NS |

Each full task states its exact saved/served requirements and preservation scope.
There is no hidden whole-zone whitelist: harmless unrelated owner/type data,
supported TTL choices, normalization and legitimate serial/action alternatives
remain acceptable. The second alternative for each task uses a staged repair;
the first exercises changed case/order/TTL/duplicates and a different serial.
The handover alternative deliberately leaves an A record at the retired owner:
the task requires no PTR data, not an unasked exact NXDOMAIN response.

## Teach and practice before the task

Every scenario has an Introduction, separate editable practice and full
assignment. Concepts explain the next operation before assessment. The first
introduces reverse owner versus returned host name, PTR versus A/AAAA, full
record syntax, authority and saved versus served data. The second explains
zero expansion, nibbles and the full ip6.arpa. suffix with a worked line. The
third explains valid multi-PTR data, retirement, selected-copy negative checks
and equal-serial secondary staleness.

Practice uses different address ranges, origins, canonical names and serials.
Existing **Fill worked example** fills only its draft; the learner explicitly
uses **Save zone**, **Reload primary**, **Update secondary** when needed,
**Look up** with type PTR and **Check example**. The assignment uses
**Check configuration**. Internal `zone` JSON actions exist only for author
reachability, never as a learner command or guide method. No new control or
address-conversion button is assumed.

Each example has one concrete experiment and repair. IPv4 saves a wrong target
without reloading, so an old lookup can look healthy while current saved checks
fail. IPv6 moves a record to a well-formed wrong-nibble owner and publishes it.
The handover restores the retired PTR on the primary only. The review fixture
contains explicit expected failed rows and runnable repair actions; the tests
check those individual rows instead of accepting any unrelated failure.

## Independent tables and grading evidence

`dns-reverse-content-review.json` holds independently authored address/question
and target tables, complete intended zones, two alternatives per task, common
wrong repairs, exact partial-row vectors, break-again mutations, and separate
example/experiment/repair tables. No expected result was taken from a VM or
reference execution. The ignored authoring source uses original literal data
and ordinary string reversal, without importing this project or calling a
resolver. Authoring metadata checks are not behavior tests.

For IPv6 the source records full expanded 32-digit strings before constructing
question names. For example, assignment address
`2001:db8:4400:1200::3a4:50b6` expands to
`20010db8440012000000000003a450b6`; practice address
`2001:db8:5500:2400::be:29` expands to
`20010db8550024000000000000be0029`. Expected owners are the reverse sequence of
those literal digits, dotted and suffixed with `ip6.arpa.`. The content test
independently checks the written address against the expanded integer, the
digit count and the textual reversal. It never calls `dns::reverse_name` to
generate expectations.

All task outcomes are current typed records, selected-copy answers and
publication. Positive direct saved PTR requirements distinguish an alias that
merely yields the same answer. Handover independently requires both NoAnswer
rows, positive new/shared/healthy answers and `Published { secondary: true }`.
An internal alias reaching PTR and an external unresolved alias both fail old
address retirement. Absence alone gives no synchronization credit. A successful
former lookup, a performed transfer event or a particular reload order is not
a credential.

Prospective tests cover references, alternatives, specific wrong/partial rows,
each required outcome broken independently after success, preserved records,
equal-serial stale secondary repair, no-PTR/NODATA alternatives, separate
fill/save/publish/experiment/repair, exact pending drafts, current invalidation,
save/load, selected reset and foreign-practice rejection. They retain all 116
original contexts and genuine frozen revision6/7 script history, without
fabricating new current results. A separate new-DNS-only backup check avoids
misrepresenting the shared inspector's deliberate rechecks of old script runs.
All eleven checks pass in the combined279-check/23-target model gate.
Actual native acceptance remains pending.

## Technical and curriculum limits

One fixed authoritative origin is available per assignment. The supplied
inventory identifies canonical target names; this reverse-only model cannot
independently prove their external canonicality or forward A/AAAA consistency.
PTR target names are data, not automatic recursive queries. Full names and the
native record editor are a bounded subset of DNS administration, not complete
BIND master-file syntax or utility use. No packets, live services, cache or
negative-cache timers, delegation/glue/RFC2317, transfer protocol or security
configuration is simulated. TTL choices are retained but not prescribed by
these tasks. No new dependency or learner-host execution is introduced.

The official objective page was rechecked 2026-10-25: LPIC-2 objectives4.5.0,
exams201-450/202-450, section207.2. Only actual accepted reverse-record and
selected-copy investigation teaching could later strengthen relevant Partial
evidence. Full BIND configuration, zone-file locations, broader master-file/SOA
syntax, root hints, delegation, utility command syntax and security remain
gaps. This proposal changes no coverage status.

Primary references rechecked 2026-10-25, paraphrased with no copied code or
manual passages:

- [LPI objectives4.5, section207.2](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [RFC1035, PTR and IN-ADDR.ARPA](https://www.rfc-editor.org/rfc/rfc1035.html)
- [RFC3596 section2.5, IPv6 reverse nibbles](https://www.rfc-editor.org/rfc/rfc3596.html)
- [RFC2181 section10.2, multiple PTR and alias distinctions](https://www.rfc-editor.org/rfc/rfc2181.html)
- [RFC1982, secondary serial arithmetic](https://www.rfc-editor.org/rfc/rfc1982.html)
- [RFC2308, missing-name versus missing-type responses](https://www.rfc-editor.org/rfc/rfc2308.html)

Independent source/data review accepted all20 reverse-question entries and
the task/grade scopes. Before delivery, normal bundled text removes proposal
bookkeeping, doubled trailing DNS dots and misleading IPv6 .53 shorthand.
Next gates are the sole guarded test invocation and actual native acceptance. No
novice, accessibility, other-platform or full-objective completion is claimed.


M96 guarded source gates:279 model checks/23 targets PASS, including all11 new
content checks, references/alternatives/row-specific wrong and partial outcomes,
current breakage, isolated examples and exact original116 specifications/unit
relative order/history preservation. First integration compile failed from a
removed test-fixture constant; it now reads the actual bundled pack. The next
run stopped at an old live-catalog104-kind count, now107 because three actual
DNS tasks were added. Failed logs are retained. Five real-egui controller tests
pass after correcting its Confirm metric field name; actual native run pending.
