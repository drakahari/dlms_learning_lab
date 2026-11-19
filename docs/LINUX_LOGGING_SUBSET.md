# Linux classic-selector subset — capability foundation

2026-11-02: new pure `linux_logging` module and focused tests are being integrated.
No authored Linux logging lesson is registered yet; current catalog231/62.
This is independent of the working Cisco `syslog` model. No existing engine,
scenario, progress record, shell evaluator or dependency is replaced.

## Deliberately bounded grammar

The editor will contain a **selector fragment**, not a full daemon configuration.
Each nonempty line has one selector and one supplied virtual log destination:
`facility.level /var/log/name`. Blank lines and whole-line # comments are allowed.
Names are case-insensitive; paths are case-sensitive. Supported canonical names
cover Linux facilities0–11 and local0–local7 (16–23). Other RFC facility codes are
observable through wildcard matching only; numerical names, mark and deprecated
aliases are not accepted as if they were implemented. No invented names are taught.

- Named priority includes that severity and every more urgent lower number.
- `=priority` selects exactly one severity; `*` selects all levels/facilities.
- A comma shares the same level selection among the named facilities.
- Following `;facility.none` clauses remove those facilities from that rule only.
- Independent matching lines each contribute a copy; a later private rule does
  not undo an earlier general-log copy. Rule order does not mean first-match wins.

Other positive semicolon selectors and `!` filters are explicitly unsupported:
rsyslog has additional compatibility behavior that must not be approximated.
No remote actions, includes, modules, RainerScript, property filters, templates,
stop/discard, journald, clocks, file permissions, queues, daemon lifecycle or
logrotate. Optional depth must explain those limits without implying they work.

## Isolation and bounds

No socket, process, clock or filesystem API in this module. Paths are labels in
supplied state; no learner text reaches host operations. Parse at most8192 bytes
and32 lines against1–4 unique safe virtual /var/log paths. ASCII whitespace only;
unknown/extra action tokens and unsafe/unprovided paths fail with a line-specific
diagnostic. Events use24 facility codes,8 severities and at most256 UTF-8 message
bytes, with controls refused. Shell-looking printable messages remain data.
Rules use explicit24×8 Boolean selection masks. Behavioral comparison checks the
complete192-combination delivery multiset; extra copies or unwanted destinations
fail. Displayed sample events will illustrate replay rather than define all tests.
The upcoming workspace must retain separate drafts/results and invalidate stale
success whenever edited configuration no longer matches its tested text.

## References and verification limits

Primary references read2026-11-02:

- https://docs.rsyslog.com/doc/configuration/filters.html — classic selectors,
  threshold/exact/none/group behavior and excluded compatibility cases.
- https://docs.rsyslog.com/doc/configuration/actions.html — independent actions.
- https://www.rfc-editor.org/rfc/rfc5424.html — facility/severity codes; not a
  storage-file format or complete rsyslog configuration specification.
- https://www.lpi.org/our-certifications/exam-101-102-objectives/ — LPIC-1 v5.0,
  108.2. No parent objective is made complete by this foundation.

The upstream rsyslog redirect initially failed; canonical docs.rsyslog.com pages
were then actually read. Installed rsyslogd is available but no live reference
process has run yet. Future fixed reference comparisons must use only private
fixture/config/input/output/pid paths and no host services, sockets or logs.
No model export, documentation or unregistered draft is authored practical credit.
