# DHCPv4 backend adapter

Backend stage1, 2026-10-25. The first guarded main-checkout gate passed68 library
checks and24 adapter integration tests; the current-result refinement adds a25th. The independent pure DHCP module's25
checks preceded this adapter. Existing catalog-wide regressions and the separate
native renderer require their own gates. No new activity, unit, coverage,
Linux/IOS command or host behavior is added by this adapter.

`engine::State::DhcpV4` wraps the existing bounded logical core.
`Goal::DhcpV4` stores the exact authored initial core State, typed requirements
and fresh-allocation probes. Initial State equality is required at pack load;
current state then pins the complete immutable supplied topology through
`State::validate_context`. Mutable policy, its pool names, current offer/lease
contents and the clock are not compared to the initial state wholesale. The
same goal-aware publication path checks context before and after bounded candidate
execution. Raw engine mutation requires that context. `show`/`help` are pure.

The pack-only authoring adapter is `dhcpv4 ` followed by bounded JSON for a typed
Action. It is simulator authoring syntax, never Linux/IOS syntax or a learner
terminal workflow. Actions are ApplyPolicy, Discover, Request of an actual token,
Renew, Rebind, Release and explicit Advance. No action executes a process, socket,
resolver or host file. The future native controls remain a separate stage.

Current ClientConfiguration requires a current client lease matching its server
binding, positive/requested remaining time, a host address and subnet on the
supplied physical segment, requested router and the requested DNS set. DNS order
is not a hidden answer; duplicates/additional servers fail this selected-set
predicate. Exact client address is optional and must be stated by an assignment,
for example a requested reservation. Router/DNS parameters do not prove a routed
path or responding DNS service. No gateway is inferred from giaddr or the DHCP
receiving interface.

Named unaffected Pool/Relay preservation and optional Binding preservation are
separate predicates. Binding preservation means those exact supplied grant and
client-held records, not evidence of uninterrupted uptime. NoClientLease means
only absence of a client-held lease: a server binding may remain after lost
Release. Excluded/Reserved/ServerEnabled predicates express explicit policy
requirements. Their protected pool names must exist in the authored initial
policy. Identical requirements and identical probe contexts reject. Absence-only
goals reject; at least one positive current client and positive fresh allocation
are required.

Each of at most4 probes starts an isolated core State at simulated time0 with
the same immutable topology and the learner's applied policy. At most8 distinct
clients per probe,32 total, undergo Discover and Request using the offer token
actually returned. Each client is checked against its declared configuration;
the learner's current state is untouched. Different allocation orders and optional
exact-address predicates admit alternate dynamic assignments while exposing
capacity/reservation/relay shortcuts. No arbitrary scripts, extra topology,
expected action history or cached result is accepted as authority. The maximal
probe test exercises32 allocations whose dynamic pool scans reach the end of a
256-address range. One ordinary guarded development-build observation measured
13.972ms for cloning the core state and checking this maximum probe arrangement
(58476-byte compact core state). This is not a latency guarantee or native UI
acceptance; repeated rendering and actual controls require separate verification.

Apply changes policy only. It does not create a client lease, update held options,
refresh expiry or rewrite a delivered offer. A Request may ACK previously offered
parameters. A CurrentLease ACK retains the old grant/options; Renewed explicitly
takes current policy and may even produce a shorter expiry. Advancing time only
changes the clock/expiry and never sends exchanges. Current requirements expose
stale/missing/expired configuration; fresh requirements expose broken policy even
when a usable historical lease still happens to satisfy current checks.

Draft fields are bounded raw text and remain saveable when malformed. Each pool
has individual network/prefix/range/router/duration fields, exclusions, reservation
rows and DNS rows. Duplicate reservation/exclusion text may be retained for
repair; snapshot rejects ambiguous typed policy. Duplicate structural JSON map
keys reject. All fields are at most64 UTF8 bytes; names/maps use core48-byte ASCII
IDs. Apply parses the complete policy and publishes atomically, without truncation.
Valid Apply normalizes policy text. Clock text is independent from policy equality;
typing an invalid future Advance does not pretend current policy changed. Queries
and editing never move time.

Draft stores a bounded immutable Topology copy as context metadata. It permits
mutable pool names and compares full supplied context rather than only counts.
Draft matches compares its parsed policy with applied policy; an invalid or pending
policy draft prevents current completion. Offers, bindings, clock and selection
are not success credentials. Context/readability safety is distinct from parseable
policy: a bounded raw invalid draft may be saved and resumed exactly.

Optional Lesson/Learning uses the existing Concept/View model. It has different
fixed server/client/segment identities, annotated typed actions, a reachable worked
example, a declared non-no-op incomplete experiment and an explicit repair.
Learning's single immutable full fixture copy is context metadata; decode and
practice operations compare it exactly with Lesson.fixture. A same-graph fixture
with a different authored policy cannot be substituted. Evolving policy names may
legitimately coincide between independent assignment/example worlds; no evolving
namespace-disjointness rule is imposed. Fill changes only policy draft fields;
exchange/time actions need explicit execution. Assignment state is inaccessible
from the example. checked/notices never establish completion.

Bounds remain8 pools/segments,32 supplied clients,256 range addresses/exclusions,
32 reservation rows per pool and4 DNS entries. Existing global pool reservation
uniqueness is enforced when parsing typed policy. Lists do not trust size_hint,
and refuse an excess typed item before its body. Derived internally tagged outer
serde Content may already be buffered under the existing2MiB pack/16,000,000-byte
progress bounds; this is not a pack-wide streaming claim. Core standalone decode
keeps its1MiB input limit. No global pack/progress/backup limit is raised.

Draft compact serialization is admitted only through262144 bytes; Learning and
Lesson use1048576-byte compact aggregate ceilings. A counting writer enforces the
ceiling without allocating a second serialized String. Shape/text/count validation
precedes counting/cloning. The Cartesian combination of every collection maximum
and every64-byte heavily escaped raw field can exceed262144 bytes: it rejects
honestly, retaining prior UI state rather than truncating. Maximum8-pool raw drafts
with256 escaped exclusion entries and32 raw reservation rows per pool have an
explicit admitted compact/pretty regression; full64-byte escape variants prove
rejection. Core State/practice context/runtime/draft copies and a mixed actual
legacy-history profile have passing compact/pretty save/load closure checks.
Pretty input may exceed the compact draft ceiling while representing an admitted
compact value; it remains within the enclosing file bounds and typed limits.

All retained adapter feedback uses a fresh UTF8-safe allocation through512 bytes.
Unknown author-action errors are generic, not a dump of arbitrary input. Current
messages distinguish missing lease, wrong subnet/address, held options and remaining
time; fresh messages name the probe/client and do not print a full golden policy.

Saved optional metadata is omitted when absent, preserving old wire shapes.
Import pins current state to Goal.initial.topology, draft to current topology and
practice to its authored fixture. Wrong-kind metadata rejects. Existing save has
no catalog parameter, so its local guard checks intrinsic state, draft/practice
shape and immutable example/assignment separation; exact authored Lesson/Goal
checks occur in decode/export/import. App actions and practice operations must
publish only through the context-aware APIs. Direct callers must not invent an
unvalidated state/lesson and expect save alone to authenticate its authorship.
Backup inspection does not auto-run DHCP exchanges or discard pending drafts.
The existing receipt/atomic file paths and shell evaluator9 histories are unchanged.

The unbundled fixture in tests/fixtures/dhcpv4-workspace-preview.json is a technical
backend fixture, not curriculum. The original119 ID/version/unit/initial/typedGoal/
solution snapshot is retained for equality; historical Goals are deserialized as
typed data so old omitted defaults do not create false differences. Actual frozen
6/7/8 records remain exact read-only data and are never relabeled as9. Current
catalog helpers may include later additions without inventing saved contexts.

Protocol and scope sources remain [DHCPV4_SUBSET.md](DHCPV4_SUBSET.md): RFC2131,
RFC2132, RFC1542 and Cisco IOS-XE17.x primary references, checked2026-10-25. CCNA
v1.1 4.3/4.6 and LPIC-2 v4.5 210.1 are only future authored-slice directions.
DHCPv6/RA, BOOTP, Linux DHCP configuration-file/daemon/lease-file/log operation,
real forwarding/DNS, multiple servers, packets and full protocol conformance remain
excluded. Pure library/adapter tests do not confer playable or objective credit.

## Current-result publication

`progress::current_dhcpv4_checks` supplies one fresh, nonempty result vector for
feedback and completion. It checks the authored initial state, immutable topology,
practice context and pending policy draft before running allocation probes. A
malformed or unapplied draft returns an explicit Apply failure rather than scoring
stale held state. `current_solved` uses the same function. No result is persisted
or reused across a mutation. Native callers should derive rows and completion from
this one vector in a frame, rather than repeat the full probes for each display.
The regression includes omitted optional drafts, malformed and pending values,
release after success and foreign practice/assignment context.
