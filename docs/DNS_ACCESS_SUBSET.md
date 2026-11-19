## Verified412R1 package

Canonical-installed DNS301, retained source419/JSON85, private409→412 lifecycle
and NFS actual two-process cold reopen pass. Final selected-input identity matches;
actual100/150dark/200Light and NFS screens inspected. CHECKPOINT.md records exact
candidate/hashes/receipt and acceptance limits. Pending statements below describe
retained development history. This is technical Fedora review, not owner/novice,
accessibility or other-OS acceptance. Source freeze ended; keep package evidence
immutable. No publication or original app/profile changes.

# DNS access boundaries — source412, native review pending

Three authored lessons are registered in the normal catalog,412labs/158units.
The latest admitted package remains409R1. Pack/progress binding, authored methods,
separate drafts and focused UI tests pass; actual native interaction/screens and
new packaged acceptance remain separate gates. See CHECKPOINT.md.

## Supported model

The new wrapper reuses the existing authoritative Zone query and RFC1982 secondary
serial logic. Existing dns::Server fields/serialization are unchanged. One to four
supplied primary/secondary pairs have immutable source records; only explicit
allowed transfers replace secondary copies. The modeled source IP is a fictional
request attribute, not client authentication. All objects are in process; no DNS
socket, resolver, host file, service or subprocess exists.

A real tokenizer/parser reads the disclosed named.conf excerpt. One options block
must explicitly state recursion yes/no and allow-query, allow-recursion,
allow-query-cache and allow-transfer. Supplied primary zone declarations include
literal file names; optional zone allow-transfer overrides the global value.
Order, whitespace, three ordinary comment styles and equivalent positive IPv4
ACLs are supported. There is no guessed default, include loader, arbitrary file
access, named ACL, negation, view, forwarder, key, IPv6/on-address ACL or full
BIND configuration. Unsupported settings and malformed syntax report errors.
A file name is only the supplied virtual label, never a path opened on the host.

ACLs support any, none, empty lists, full IPv4 hosts and canonical-base CIDRs.
Exact normalized interval unions/intersections cover all32 address bits, including
/0 and /32. Equivalent split ranges can pass; checking only sampled clients cannot
hide a hole or accidentally broad scope. Noncanonical network bases and abbreviated
or unsupported address forms explain this subset's restriction.

Queries distinguish authoritative zone results, supplied upstream results,
existing simulated cache data, access refusal and no new recursion. The upstream
dataset is finite, not a network or iterative resolver. An unknown fixture reports
a limitation instead of invented DNS data. No TTL ageing, root referrals, packets,
DNS flags/transports or external aliases are promised. Disabling new recursion
alone does not remove access to existing cache entries. Query reports explain
semantic outcomes; they do not pretend to be a complete dig transcript.

Separate saved/live policies model save versus reload. Reload preserves cache.
Transfers evaluate the current effective zone policy and use actual modeled
records/serials; address lists do not supply TSIG, encryption or secure exchange.

## Bounds and task grading

Configuration8KiB/2048tokens; each positive address list32entries; at most four
served and four upstream zones; existing64-record zone limit; initial cache16keys,
runtime cache32keys; output32KiB. Filenames/domains and action histories are bounded.
No recursive parser constructs or unbounded script language is present. Invalid
operations publish no partial state. Traversal/escape filenames and unsupported
configuration cannot open host files.

Task grading separates effective public, cached and recursive access, per-zone
transfer scope and secondary preservation/synchronization. Multiple deterministic
cold/supplied-warm cases compare actual modeled answers. Each case clones current
policy/state and prepares its declared cache fixture: this is a test input, not
invented learner work. Case preparation never modifies the assignment. Foreign
worlds, unsupported test questions, falsely warm fixtures, solved starters and
unreachable methods are rejected. Expected/actual output is generated for feedback,
not used as an exact configuration-string matcher.

Separate version1 draft histories replay against authored facts. No serialized
world, cache, output or imported case result is trusted. Current editor, query/
transfer input, check position and action provenance are retained. A correct
example cannot award assignment credit. Explicit task IDs and source-bound replay
prevent drafts borrowing another exercise's result. Normal progress/pack admission validates authored source and replay. Bare saves
without that catalog refuse rather than guessing a source or trusting results.

## Evidence and limits

TransferR1 passes29 checks (ACL4/config3/world10/task5/session7). RegisteredR1
passes binding4/content6/session7/world11, including exact409/157 retention and
normal412/158 catalog loading. UIr5four checks and capture-r2two pass at recorded
source points; field-metric regression and actual native checks remain pending.
Earlier strict-decoding, parser, authoring and scrollbar failures remain retained;
tests/guards were corrected or strengthened, not weakened. No real named/BIND
comparison, independent novice, screen-reader, physical-device or other-OS
acceptance is inferred. Stage:
`.build-artifacts/continuation/resume-20261111/next-dns-access`.

Primary sources inspected2026-11-11:
[LPIC-2 current objectives v4.5.0](https://www.lpi.org/our-certifications/exam-201-202-objectives/),
[ISC BIND9.20.12 configuration reference](https://bind9.readthedocs.io/en/v9.20.12/reference.html#access-control).
The pinned ISC reference distinguishes client query/cache/recursive access and
zone transfer overrides; its recursion section warns that disabling new recursion
does not prevent cached answers. This finite model avoids unspecified defaults.
It does not cover full207.1/207.2/207.3, chroot/non-root operation, forwarders, TSIG,
DNSSEC/DANE or their named utilities. No coverage row is promoted for this core.
