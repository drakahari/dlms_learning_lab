# Basic NAT application adapter — unbundled source proposal

2026-10-26. Implemented typed backend, independently source-reviewed. The final
isolated full all-target gate passes1619 checks/zero failures/11 private skips;
all20 pure and23 adapter checks also pass explicitly including the private actual9
replay. All-target Clippy passes. MAIN integration validation is tracked separately
in the ledger. No normal lab/unit, teaching or native renderer is added; current
140/35 and evaluator10 stay unchanged. No whole CCNA4.1 credit follows from this
backend. Earlier frozen source proposals and failed test-setup gates are retained.

The core boundary remains [NAT_SUBSET.md](NAT_SUBSET.md): static and dynamic IPv4
address mapping, no PAT/ports, packets, checksum/ALG, sessions/expiry/timers,
routing/connectivity, security, IOS or host effects. Inside/outside/global/local
are supplied roles, not private/public classifiers. Current pool clearing and
lowest-free allocation are declared simulator decisions. Root rechecked the
[CCNA v1.1 objective4.1](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),
[Cisco13772](https://www.cisco.com/c/en/us/support/docs/ip/network-address-translation-nat/13772-12.html)
and [RFC3022](https://www.rfc-editor.org/rfc/rfc3022.html) on2026-10-26. RFC3022 is
Informational; no real device/session conformance is claimed.

`State::BasicNat` wraps the unchanged validated core State. `Goal::BasicNat` has a
bounded Definition with exact authored initial State, current/policy requirements
and independent trial plans. The authored scenario.initial must equal that initial
State; ongoing State and Draft origin must retain its exact Inventory. Mutable
policy/bindings can evolve and legally coincide with another context's values.
No exact hidden solved State or history/action badge is required.

CurrentMapping reads an existing static entry or dynamic binding; it never probes
to create a missing one. NoCurrentMapping does not imply future ineligibility or
security. StaticPreserved requires an initial existing static and preserves its
exact address. DynamicPrefixMatches tests effective supplied-host membership;
DynamicCapacity counts pool slots after static reservations, not free-now slots.
DynamicRulePreserved is an explicit unchanged-initial-rule promise. Requirements
use Any or explicitly requested Exact globals and explicit mapping kind. Policy-only
tasks can omit held dynamic entries, but need positive policy plus copied-policy
translation/reversal, requesting Dynamic when static substitutions would be wrong.

These predicates do not require an arbitrary new exact prefix or pool spelling.
Future task prose must promise effective eligible hosts/capacity/supplied globals,
or first add/review a specific predicate. Preserve named mappings/configuration
explicitly; never imply global privacy, traffic reachability or session continuity.

One to four trials run sequentially on separate current-State copies, each cleared
of dynamic bindings. Each1..16 steps is Outbound, Reverse of the copy's actual map,
or explicit NoMapping. Reverse never allocates or guesses a global. Positive
static reverse can precede outbound; dynamic reverse requires an actual binding.
Different host orders permit different valid assignments. Every trial needs a
positive outbound/reverse pair. At most64 probes plus4 clears (68 core calls,
hard ceiling128), with no requirements×trials probe execution. Current State and
raw editors stay exact through checking. One summary per trial keeps feedback
bounded. `Report` counters are conservatively charged work (an early missing reverse
may be charged without invoking core), not elapsed-time measurements or imported
proof.

Draft is one optional `nat_draft`, defaultNone/skipNone under the existing workspace
schema. It holds an immutable Inventory copy, at most32 raw static rows, optional
dynamic raw fields, bounded request selectors and notice. Malformed bounded policy
and duplicate rows remain saveable until repaired. All active static rows parse.
Disabled dynamic fields remain exact but are not parsed for matches/Apply; enabled
invalid fields are pending and Apply fails atomically. Read-only request/selection/
notice edits do not block applied credit. Valid alternate text for the same parsed
policy is not a new policy. Pending policy blocks current success; typed mutations
are atomically admitted. Existing editors are never silently replaced by prepare.

Bounds: existing core256KiB State/64KiB Action;512KiB Definition,32 requirements,
4×16 steps;64KiB Draft,32 raw rows, inside32bytes, address fields64bytes,
prefix16bytes, notice512UTF8bytes. Checks have at most32current+4trial summaries
and context/pending rows, total below32KiB. Custom typed visitors bound strings and
collections without trusting size_hint. Borrowed direct shape/count admission
precedes serialization counting/clones. Empty tagged variants use `{}`. Generic
serde can already buffer tagged outer Content under the existing pack2MiB/profile
16MB admission; this is not a raw-before-parse streaming claim. Explicit raw helpers
check their family cap first. No global bound is raised and no input truncates.

The `basic-nat ` typed JSON bridge is a simulator-only pack/developer helper,
not learner IOS/Linux grammar. Goal-aware context is mandatory for mutation and
raw admission precedes trim; ordinary families retain their old limits. Transient
`perform` returns the actual Effect; neither Effect nor Snapshot is deserialized
as grade authority. There is no learner/native command interface in this stage.

Save/decode/current helpers reject NAT metadata on other State kinds and other
family metadata on NAT before success shortcuts. The new-kind-only guard also
rejects script/terminal learning, switching draft/learning and IPv6 draft/learning,
which lack equivalent old-family standalone wrong-kind validators. Shared command,
script history/provenance and legacy scalar editor fields remain retainable and
are not NAT grade authority. Existing family validator bodies are unchanged. Full current_solved recomputes
truthful checks, with no persisted grade cache. Before a future renderer/bundled
NAT card, explicit checks and ephemeral exact-state/goal/pending presentation
memoization must avoid replaying all trials every frame. No broad cache platform
or renderer is introduced here.

Executed tests cover missing current versus valid future policy, static first
inbound, reverse/reuse, two host orders and alternate prefixes/pools, capacity vs
no-match, pending/disabled/readonly editors, strict typed/raw bounds, maximum
Inventory/escaped draft/Definition, both-direction foreign metadata, atomic errors,
full old140 contexts, actual6/7/8 histories, shared backup/confirmed restore/reset
and an ignored hash-pinned private actual9 replay. No recordings are generated or
relabelled historical. Backup inspection's two documented current-practice fields
are independently projected from saved source+fixture; all other fields/history
remain fully compared. Disk-only receipt stamping and exact before8/current-target
raw backups retain the existing contract. The maximum trial fixture charges68 calls/4 clones; final observed elapsed time
is recorded in the gate log. A single technical sample is not a latency guarantee.
