# Reverse DNS extension — capability contract

Specified2026-10-24 before dependent lessons. Extend the existing bounded
authoritative DNS model with PTR data and queries. This is capability work,
not an additional authored lab, coverage promotion or BIND implementation.

A PTR record maps its owner name to a domain name. Its target is returned as
data, not followed as a CNAME or automatically resolved to an address. Multiple
distinct PTR records at one owner form an RRset; do not invent a one-PTR-only
rule. Existing normalization, duplicate suppression, minimum RRset TTL, CNAME
coexistence checks, publication and secondary serial arithmetic still apply.
Reject a PTR target known within the same zone to be a CNAME; external target
canonicality cannot be established without another zone or recursion.

Provide a pure typed address-to-question helper: reverse four IPv4 octets and
append `in-addr.arpa.`; expand all32 IPv6 hexadecimal digits, reverse individual
digits and append `ip6.arpa.`. IPv6 compression and mixed IPv4 notation are
handled by Rust's typed address parser before conversion. This helper performs
no lookup and invents no record. The editor continues to accept absolute owner
and target names, not bare addresses or relative zone-file syntax.

Do not restrict PTR to special reverse suffixes: PTR is a general DNS type.
Queries still require the selected fixed authoritative origin. Inverse lookup
is not the obsolete DNS IQUERY operation. One configured zone and two loaded
copies remain the scope; no automatic forward/reverse consistency, delegation,
RFC2317 setup, recursion, DNSSEC, actual resolver or network activity is implied.
No new dependency or learner execution path is needed.

Preserve old record serialization and progress schema1; add only the new enum
variant. Bound records, names, editor/source size, aliases and output exactly as
before. Test IPv4 zero/max/documentation addresses, compressed and full IPv6,
digit ordering across groups, mapped addresses, PTR RRsets/case/TTL, absence
versus no-data, aliases, atomic invalid reload, secondary publication, equivalent
outcomes, broken-current-state grading and malformed draft resume. Existing DNS
assignments and full progress regression must still pass.

Primary references inspected2026-10-24:
[RFC1035 sections3.3.12 and3.5](https://datatracker.ietf.org/doc/html/rfc1035),
[RFC3596 section2.5](https://datatracker.ietf.org/doc/html/rfc3596#section-2.5),
[RFC2181 section10.2](https://datatracker.ietf.org/doc/html/rfc2181#section-10.2)
and current [LPIC-2 section207.2](https://www.lpi.org/our-certifications/exam-201-202-objectives/).
The current issuer objective still needs reverse-zone teaching, broader BIND
configuration and delegation; a library extension alone does not cover it.


## Scoped verification

Twelve new model/editor/progress checks and77 existing library/DNS checks pass;
all-target Clippy with warnings denied and formatting pass. Independent source
review rechecked the primary RFCs and found no material fidelity, bounds or
atomicity issue in this extension. Tests include all128 individual IPv6 bit
positions and exact malformed-draft roundtrip; these are development checks,
not authored reverse-zone curriculum or novice/native acceptance.

The existing six forward-DNS scenarios retain their identities, goals, initial
states and solutions. Their secondary limitations text now accurately mentions
editor PTR availability while distinguishing the unit's forward-only teaching.
Full combined regression and future native reverse-zone lesson acceptance are
tracked in CHECKPOINT.md; no objective is promoted for this capability alone.
