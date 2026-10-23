# ACL configuration document and fresh grading adapter

Specified 2026-10-23 before parser/adapter implementation; bounded design reviewed.
Builds on classifier3e07e40 and transit wrapper5875f43. A standalone module, tests,
this document and a library export are implemented in the isolated worktree.
No application State/Goal, progress, native screen, authored scenario, dependency
or objective status changes. Final source review precedes integration.

## A configuration document, not a device terminal

The learner edits one router's complete modeled ACL configuration. Nothing runs
on Enter or while typing. A later native Check configuration action can parse
and apply it, then grade the resulting network. The editor must visibly explain:
**Applying replaces this router's modeled ACL lists and bindings. Omitted lists
or bindings are removed.** Other routers and the supplied topology stay intact.
This replacement behavior is a native-editor rule, not IOS command execution.

An empty document deliberately clears the selected router's lists/bindings. An
empty ACL section is unsupported, matching the reviewed classifier boundary;
it is not interpreted as implicit deny. A retained binding to an omitted list
fails with a specific message. There are no `no` commands in this first slice:
remove the binding/list section to clear it. No simulator-only setup commands,
JSON, shell, network device connection or simulated full IOS prompt is taught.

## Exact first grammar

Blank lines and whole-line `!` comments are ignored. Commands use ASCII spaces
or tabs; indentation is cosmetic. Keywords accept ASCII case differences; list
and interface names retain the core's case-sensitive1–32-byte identifier rules.
No abbreviations, inline comments, line continuation, quoted identifiers or
unrecognized trailing tokens are accepted. Each error identifies its source
line and the unsupported/malformed part without echoing an unbounded input.

```text
ip access-list standard NAME
 [SEQ] permit|deny SOURCE
exit

ip access-list extended NAME
 [SEQ] permit|deny ip|icmp SOURCE DESTINATION
 [SEQ] permit|deny tcp|udp SOURCE [PORTS] DESTINATION [PORTS]
exit

interface NAME
 ip access-group NAME in|out
exit
```

Each section must close with `exit` before another section starts; EOF can close
the final section. `exit` outside a section fails. Repeated list sections,
repeated interface sections, or repeated bindings for one interface/direction
fail rather than silently merging/replacing. Both directions may be specified
once in a single interface section. An interface section without bindings is
accepted and retains an empty attachment record for exact renderer round-trip;
that record activates no filter. An omitted interface section has no record and
also activates no filter. Changing between these representations may still need
an apply/check to synchronize a pending document with applied configuration.

SOURCE/DESTINATION is exactly `any`, `host IPV4`, or `IPV4 WILDCARD`. A bare IPv4
address without `host` or a wildcard is deliberately unsupported, with guidance
to use one of the explicit forms. All32 wildcard bits retain their independent
core meaning, including noncontiguous patterns and ignored nonzero base bits.
CIDR notation is not substituted for a wildcard mask. PORTS is absent (Any),
`eq INTEGER` or `range INTEGER INTEGER`. Port integers are decimal0–65535;
ranges are inclusive and ordered. Source and destination ports are independent.
IP/ICMP rows cannot carry ports. Named services, numeric protocol selectors,
ICMP type/code, flags/established, fragments, remarks/logging/counters, `no`,
resequence commands, object groups, NAT, IPv6 and all other IOS syntax fail
honestly. `!` is the documented comment alternative to unsupported `remark`.

Within a list either every rule supplies SEQ or none does. Mixed mode fails,
avoiding invented IOS automatic-insertion behavior. Explicit sequences are
distinct decimal1–65535 (a simulator limit), evaluated in ascending numeric order
regardless of physical line order. Unnumbered rules retain document order.
Duplicate sequence numbers fail rather than acting as a live edit operation.
This supports reordering without pretending to implement a stateful IOS session.

Compiler rule IDs are `seq-N` for explicit sequences and `row-N` for unnumbered
rules in final order. These internal IDs are not additional learner syntax.
Native authoring uses policies produced by this compiler. A canonical renderer
must round-trip these policies exactly, preserving numbered identities; it must
return Unsupported for arbitrary core rule IDs it cannot represent, rather than
silently rename them. The broader pure classifier remains unchanged. Grading
normally accepts either numbering style and equivalent rules; a prescribed ID
may only refer to an explicitly requested sequence, never an unexplained
generated unnumbered row identity.

## Limits and atomic per-router application

One raw router document is at most64KiB UTF-8,1024 lines,512 bytes per line and
16 tokens per non-comment line. Tokenization borrows bounded text. Existing
limits remain: at most8 lists/network,64 rules/list,512 rules/network and at
most8 interfaces/router. Bound sequence/number conversion before sorting or
retaining rows. Reject extra sections/rules/keys before building larger state.

The new adapter will expose parse, render and apply-router operations. Before
cloning anything it validates the current network, selected non-host router,
all input lengths/counts, and candidate network-wide totals. Parse the complete
document into a fresh DevicePolicy, resolve every interface/list reference and
validate its class/ports. Only then clone bounded current state, replace/remove
the selected router policy and validate the complete candidate. Publish one
successful result; any parse, resolution, bound or final-validation error keeps
all applied state byte/effect equivalent. Never fix another router implicitly.

A version1 raw-draft container preserves malformed work independently of applied
state: at most12 known non-host router keys,64KiB each and256KiB combined. No
silent truncation. Wrong version/keys/oversize reject without overwriting prior
data. Its JSON entry point caps input at2,000,000 bytes before parsing, enough
for maximum escaped draft text; raw UTF-8 byte totals are checked separately.
A draft matches applied state only if parsing succeeds and its compiled
selected-router configuration equals that applied configuration. Incomplete or
divergent drafts invalidate current success; harmless whitespace/comments need
not become different network behavior. Reset remains a future selected-attempt
operation, not a parser side effect. Native integration may compose multiple
validated router replacements atomically before one check; no control redesign
is part of this backend slice.

## Fixed assignment context and fresh behavior grading

ACL-focused tasks freeze the **entire supplied Topology** (addresses, prefixes,
routes/order, operational/forwarding flags and physical shape) and declared host
map. Policies/bindings remain editable. This is stronger than the existing
shape-only same_identity helper and must not change old routing tasks. Validate
the fixed context before applying/grading/importing; the learner cannot repair
an ACL assignment by changing routing or redefining host roles.

A typed definition contains2–16 uniquely identified cases, at least one required
positive and one negative. Each fixed case binds source/destination device and
interface, protocol/port tuple and a bounded plain-language label. A negative
also states the expected device/interface/direction. Its cause is optional:
omitted/None accepts either explicit or implicit denial at that exact boundary.
Only a task that explicitly requires a cause supplies Explicit/Implicit; exact
sequence IDs are optional only when requested by the task. Ordinary denial tasks
must accept equivalent explicit-deny and permits-only/default-deny policies.
Definition validation checks all references, non-host denial locations and
no-ACL endpoint delivery for every case; both the negative baseline and at least
one positive baseline must cross each expected boundary. Author solutions and
alternate solutions still need separate reachability/content tests.
The definition is version1, rejects unknown/duplicate fields and case IDs, bounds
labels to80 characters/320 UTF-8 bytes without control characters, and caps its
JSON entry point at2,000,000 bytes before parsing. Fixed-data and collection
validation precede any baseline clone. Native definitions require their initial
policies to be renderer-compatible; arbitrary opaque core rule IDs are rejected
explicitly at this adapter boundary. No existing scenario has this new state.

The grade API takes the fixed definition, current Network and raw-draft container
(the container may be empty when no edits are pending);
it does **not** accept cached Inspection objects or caller-supplied pass flags.
On every check, validate pins and drafts, then inspect every case freshly against
the SAME immutable current network. Require all positive deliveries. Each
negative requires fresh no-ACL delivery, actual denial at its expected location
(and cause only if specified), and a successful required positive crossing there.
Unrelated bypasses cannot justify deny-all. Request/reply packets are separate
explicit cases; no session inference. Results report observed baseline, action
and location so partial work can explain a correct denial alongside a broken
allowed flow. Overall success requires every condition. No edits may reuse a
saved successful report; regrading a broken state must fail. Reports are bounded
to16 inspections with the existing48-event/two-trace limits per case.

## Required tests and later acceptance gates

- Grammar: all supported standard/extended/port/address forms; case/spacing,
  numbered sorting versus physical order, equivalent encodings, and an exact
  canonical round-trip. Reject mixed/duplicate sequences, duplicate sections,
  missing close, extra tokens, unknown names/services and unsupported syntax.
- Limits: exact/+1 byte, line, token, rule, list, draft-total and case-count
  boundaries; malformed UTF-8 at JSON boundary; duplicate draft keys; direct
  constructed invalid values before clone. No partial application or mutation.
- Grade: alternate numbering/Any versus full port range; wrong protocol,
  source/destination port, wildcard and direction; shadowing, permit-all,
  deny-all, wrong explicit/implicit cause, unrelated positives and partial work.
  Reuse a previously successful report after editing: the API cannot accept it.
- Pins/isolation: changed routes, address, up/forward flags, host map or shape;
  selected-router changes leave others untouched; malformed drafts survive an
  exact serialization round-trip; examples remain separate in later teaching.
- Source review and guarded tests precede State/Goal/pack/progress/native wiring.
  Later native keyboard/editor/resizing/scaling/reset/resume and learner-facing
  clarity need actual rendered acceptance. No IOS equipment/runtime acceptance,
  new lab title or CCNA objective promotion follows from this parser contract.

Sources checked2026-10-23: the authoritative Cisco sources linked in
[ACL_SUBSET.md](ACL_SUBSET.md), plus [IOS XE17 sequence numbering](https://www.cisco.com/c/en/us/td/docs/routers/ios/config/17-x/sec-vpn/b-security-vpn/m_sec-acl-seq-num.html).
Only the supported command shapes and ordering inform this bounded native editor;
whole-document replacement, mixed-mode rejection, narrower sequence bounds and
compiler IDs are explicitly simulator design choices, not claims about full IOS.

## Isolated verification checkpoint

Thirteen adapter tests and47 existing classifier/transit/routing/content/workspace
tests pass (60 total). Tests exercise exact renderer round-trip; explicit numeric
ordering; numbered/unnumbered and port-range equivalents; ordinary explicit or
implicit denial acceptance; cause-specific tasks; compiler-identity rejection;
per-router atomic replacement/clearing; saved malformed drafts; byte/line/token /
case/list/rule and maximum escaped-draft boundaries; fixed-context tampering;
fresh success invalidation and partial/broad/wrong traffic outcomes. Named ports
produce Unsupported, while malformed ordered ranges produce Syntax diagnostics.
An explicit empty interface record survives round-trip and activates no filter.

The final test run and scoped Clippy with warnings denied use the main guarded
workflow: two build jobs, two test workers, debug0/no incremental state and
disk-backed target/toolchain/temporary files. Formatting and diff checks pass.
This is backend evidence, not native editor, beginner, equipment, complete
curriculum, Windows/macOS or progress-migration acceptance. No scenario count or
coverage status is promoted. Source remains isolated pending final root review.
