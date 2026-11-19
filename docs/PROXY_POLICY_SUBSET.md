### Final source gates before430packaging

Python1173, core28, authored5, normal-catalog4, retained WLAN widgets17,
all-target Clippy with warnings denied, strict coverage and teaching inventory
pass. The inventory is425updated/five already-meets. Earlier eleven proxy widget/
capture checks and actual R3visual evidence remain valid for unchanged proxy
presentation. Final current-catalog helper/count fixes do not alter learner UI.
The first28Python failures and two native helper failures remain preserved;
CURRENT sizes/targets were corrected without changing historical source pins or
removing negative/assertion coverage. Clippy95seconds, observed minimum7.45GiB
available/fullPSIavg10 max0.54/OOMdelta0; shared lock/two jobs/two test threads retained.

Source/docs freeze begins before the separate430candidate build. Content, private
428→430install/upgrade/progress, installed proxy flows and NFS cold reopening still
need actual package receipts. No full-inclusive Rust runtime, proxy-specific GUI
cold reopening, independent novice, physical-device/AT or other-OS acceptance.

# Proxy access-policy foundation

Integrated source, 2026-11-12:430labs/166units, including two LPIC-2 access-policy
lessons. The separately verified428R1 package predates this source;430package
verification is pending. No dependency, progress schema or existing grader changed.
Only narrow208.3access-policy practice is credited; the whole objective is incomplete.

## Closed configuration language

`src/proxy_policy.rs` parses original Rust data structures, evaluates supplied
request facts and records rule decisions. It never opens a listener, reads a host
file, invokes a process, resolves a name or connects to a service.

Supported directives are `acl NAME src ADDRESS[/PREFIX] ...`, `acl NAME port
NUMBER[-NUMBER] ...`, `acl NAME method TOKEN ...`, and `http_access allow|deny
[!]NAME ...`. Source addresses are dotted IPv4 with an optional numeric0–32prefix;
non-network source bits are masked. Bare addresses mean/32. Port ranges are closed,
ascending0–65535; supplied requests require1–65535. Method comparisons are literal
and case-sensitive. The reserved `all` test always matches. Other names must be
explicitly declared before use. Repeated declarations of the same name/type combine
values; changing its type is an error.

Values within one named ACL are alternatives (OR). Tests in one rule must all match
(AND), with one leading `!` reversing a test. Rules are examined in written order;
the first matching rule decides. With no rules, access is denied. If rules exist
but none matches, the fallback is the opposite of the last rule's action. Lessons
must teach an explicit final `http_access deny all` instead of presenting an
unconditional default-deny shortcut. Readback distinguishes a matched source line
from implicit fallback and explains tested ACLs.

Configuration is ASCII, with blank lines and separate `#` comments. Quoted files,
escapes, continuation, inline comments, Includes, variables, regex, DNS/domain ACLs,
IPv6, external authentication, caching, resource controls and all other directives
are explicitly outside this closure. Method tokens containing quote/backslash/#
characters are also outside its configuration lexer. Unsupported input never
silently succeeds. This is a deliberately smaller language, not full Squid syntax.

## Saved and effective state

The pure virtual world retains saved and applied text separately. Saving may retain
a syntax error for diagnosis and does not activate it. Validation/application parses
the saved configuration; failed application preserves the previous effective policy.
Compiled parser internals never deserialize. Restored effective text must parse;
malformed state is rejected, not silently repaired. These are explicit simulator
transaction rules, not claims about every real service failure mode. The source-bound `proxy_workspace`, engine/progress binding and native editor now
retain separate examples and assignments. Actual teaching review and focused disk/
negative tests pass; see PROXY_POLICY_LESSONS.md for current package status.

## Bounded full-policy comparison

For this predicate language, address and port interval boundaries plus distinct
method tokens partition the request space into constant-behavior regions. Comparing
both actual and required policy boundaries can identify a concrete counterexample
or prove equivalence within this subset, accepting equivalent rules without exact
config-string grading. An extra method class represents all other method tokens.
This rejects narrow hardcoded example-client lists even when every displayed case passes. Required
policy intent must remain explicit in the Assignment; no hidden security objective.
The comparison is not a real HTTP transaction or network verification.

Limits:8192 configuration bytes,256 lines,384 bytes/line,32 named ACLs,256 total
values,64 rules,16 terms/rule;48-character names and20-byte methods. Virtual state
allows at most10,000 changed save/application generations per attempt. Comparison
has at most8192 regions and262,144 worst-case primitive predicate tests; excessive
complexity is an honest error before the comparison, never assumed success. No
learner-controlled recursion, background worker, process or file-sized allocation.
All deserialized configuration/method strings are bounded. Focused resource/complexity tests and actual native editing/checking pass within
these limits; this is not a large-scale performance or real-service conformance study.

## Sources checked 2026-11-12

- [LPI official LPIC-2 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/), currently4.5/201-450 and202-450. The broader208.3 also requires authentication and resource knowledge; no new objective credit yet.
- [Squid3.5.19 ACL reference](https://www.squid-cache.org/Versions/v3/3.5/cfgman/acl.html), the historical3.x family named by the current LPI objectives.
- [Squid3.5.19 http_access reference](https://www.squid-cache.org/Versions/v3/3.5/cfgman/http_access.html), including no-rules and opposite-last fallback.
- [Upstream ACL explanation](https://wiki.squid-cache.org/SquidFaq/SquidAcl), repeated values, ordered rules and AND/OR relationships. Its version notice and the versioned reference remain distinct from current Squid releases.

No upstream code, configuration bundle or message table was copied. This original
behavioral model does not redistribute Squid. No real Squid reference run or live
conformance is claimed. Current core/model/task/import/disk28, authored5 and native-widget/capture11 checks
pass. Actual development native R3 completed236actions/52PNG; nine rendered originals
were reviewed after R1 found layout/feedback issues. Preserved sources, alternate
rules/subnets, hardcoded clients, boundary values, limits, atomic failed application,
source binding, isolated examples and safe disk reset/resume are exercised. Current
final Python/all-targetClippy and separate packaged430 acceptance are still pending.
No real Squid reference run, novice, screen-reader or other-OS runtime acceptance.
