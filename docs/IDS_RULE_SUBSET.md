# Offline network-rule capability boundary

## Verified507 milestone —2026-11-16

507labs/204units (113CCNA,154LPIC-1,160LPIC-2,80Bash). Two bounded IDS rule
assignments, private505→507lifecycle, installed154-action native flow and normal
cold reopening pass;15original installed/cold images inspected. Earlier505data,
progress and themes are preserved. Selected inputs retained and hold ENDED.
[Current identity, command and evidence](CHECKPOINT.md). No full curriculum,
actualSnort, inclusive Rust runtime, novice/accessibility or other-OS acceptance.
The following pending notes retain their earlier build-time status. Next nc work
is still an uncompiled private draft; no new capability or count.

2026-11-16. Two registered rule-editing lessons now have scoped development
verification; final 507-candidate acceptance remains pending. The verified505
review candidate remains immutable. This module evaluates a small documented
Snort3-style header/literal subset over supplied decoded IPv4 records.
It is not a packet decoder, stream engine, Snort installation or live IDS. Current
development catalog is507/204. Core/task/workspace tests passed before dependent
lessons were authored; no complete exam-objective or actual Snort conformance
credit follows from those tests.

## Defined subset

One alert rule per2KiB ASCII file. Full-line # comments and whitespace outside
quoted strings are supported. Headers use tcp or udp; numeric IPv4 hosts/CIDRs or
any; a canonical decimal port0–65535 or any; and -> or <>. CIDR masks0–32 describe
header membership, including host bits normalized to their network. This does not
change the conventional IPv4 host-assignment lessons' /31-/32exclusion.

Supported options are optionalmsg/rev, requiredpositive sid for this activity,
and up to4nonempty ASCII content literals of at most128bytes, each optionally
modified by nocase. Options are semicolon-terminated; escaped backslash, double
quote and semicolon are recognized in quoted text. All content requirements must
match the supplied payload. These literals are not regular expressions. Packet
and content direction are kept separate; <> exchanges the complete endpoint pairs.

Snort itself permits sid omission; this activity requires an explicit ID so the
local rule can be identified. Upstream recommends local IDs starting at1000000.
Do not teach the activity's requirement as an upstream syntax restriction.

Variables, address/port lists or negation, ranges, IPv6, hex/mixed content, regex,
relative cursors/offsets/depth, flow/stream state, service/sticky buffers, Lua,
includes, inline actions, additional rules and unknown options refuse honestly.
No unsupported option may quietly become an always-true predicate.

Records contain only typed protocol/IPv4 endpoint/port facts and a bounded ASCII
payload. At most64records/4KiB per payload; only printable ASCII plusCR/LF/tab.
Validate the full input first. Matching shares250000counted work units across the
report, including content comparisons. Exhaustion returns a distinct limit error
and no partial report. No host path, filesystem, socket, process or service is used.
These records do not reproduce packet parsing, retransmission or reassembly.

## Verification and teaching gates

Test endpoint direction, both sides of prefix boundaries, /0,/20,/31,/32 header
membership, source/destination ports, literal/escaped/nocase content, combined
requirements, malformed/unsupported syntax and shared resource limits. A syntax
failure, unsupported feature and genuine nonmatch are different outcomes.

Reference comparisons with actualSnort remain unverified until a suitable local
utility is available and a reviewed fixed disposable fixture can run safely. Do
not install dependencies or feed learner rules to a host utility. The app engine
always stays in process. Source-derived pure tests are not full Snort conformance.

The two lessons now provide rule repair and false-positive investigation with
Introduction → separate guided practice → independent assignment. They retain
intentional starter errors, bounded editors and actual outcome feedback.
Greenbone/OpenVAS vulnerability-test findings and observed IDS matches must be
explained as different evidence; neither alone proves successful exploitation.
Do not count this document, a draft, static reference or prototype as a lab.

## Authoritative references checked2026-11-16

- Current LPI4.5/201-450/202-450,212.4:
  https://www.lpi.org/our-certifications/exam-201-202-objectives/
- Snort3rule structure: https://docs.snort.org/rules/
- https://docs.snort.org/rules/headers/ips
- https://docs.snort.org/rules/headers/ports
- https://docs.snort.org/rules/headers/directions
- https://docs.snort.org/rules/options/general/sid
- https://docs.snort.org/rules/options/payload/content
- Greenbone/OpenVAS roles: https://greenbone.github.io/docs/latest/background.html

Functional behavior is paraphrased; no upstream implementation/assets or new
dependency imported. Original code remains under the owner's approved MIT license.
Current official objectives are kept separate from their future-change suggestions.
