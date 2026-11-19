# Current implementation status — 2026-11-02

The retained proposal below is now integrated into the actual native review
catalog217/58; its four Syslog activities form **Logging policy and useful
evidence**, after IPv4 foundations. Earlier unregistered/UNRUN statements below
record authoring history and are superseded only by the actual evidence here.

Current focused gates:19 core,22 workspace/persistence,11 authored-content,
2 current-registration/codec and20 Syslog widget/paint/capture checks pass across
recorded runs. All24 wrong/partial policy vectors and valid alternatives remain
active. No grading requirements or resource ceilings were relaxed. Native popup
clips now use the completed menu bounds; collector input widths reserve native
margins/buttons. Reverse-Tab tests process egui's documented next-pass focus;
scrolling tests target the real lesson viewport. The separate worked-command
clipping regression passes at100/150/200%. Packaged native verification is still
pending; see CONTINUATION.md for the subsequent candidate/evidence status.

User progress, existing209 sources and213 boundary objects remain protected.
No real packets, IOS/Linux command parser, logging service, host shell, clock or
filesystem operation is added. This is practical logging-policy coverage, not
full CCNA4.5/protocol conformance or certification preparation. Windows/macOS,
screen-reader, physical-device and novice acceptance remain unverified.

## Retained authored scope and integration history

# Virtual syslog policy foundation

Source-only first layer for current CCNA v1.1 objective4.5. The foundation supplies
explicit typed event, source policy, collector selector, delivery fact and history
models. It does not yet supply a learner workspace, CLI, authored scenario or
coverage credit. The objective remains Open until those layers and actual gates
are accepted. No cloud, network, clock, process or filesystem calls occur here.

## What is modeled

Severity0 is most urgent and7 least urgent: emergency, alert, critical, error,
warning, notice, informational, debug. A threshold includes every event whose
numeric severity is less than or equal to that threshold. The local buffer and
remote threshold are independent. `None` disables that output. Disabling the
modeled logging process disables these two modeled outputs; it does not claim
that real IOS stops console logging. Console/monitor behavior is not implemented.

A source may configure several collector addresses. Unknown configured unicast
addresses are retained and diagnosed as absent from the supplied inventory, rather
than pretending IOS would reject such a configuration. Per-source/collector
immutable supplied delivery facts say only whether this exercise permits delivery.
They do not prove UDP exchange, ARP, routing or real receiver availability. A
missing delivery fact is unavailable. No actual packets are sent.

The RFC facility is0–23 and the priority number is `facility * 8 + severity`.
Facilities16–23 are local0–local7. This wire facility is separate from the Cisco
message subsystem, for example `LINK` in `%LINK-4-CHANGED: ...`. Changing the
export facility does not rewrite that subsystem label. Collectors apply a simple
facility-to-severity selector map. This represents at-or-more-urgent filtering,
not an implementation of arbitrary rsyslog/syslog-ng configuration grammar.
The core does not encode RFC5424 headers, structured data or wire messages.

`State::emit` explicitly generates one supplied immutable event template. It is a
simulator API, not a Linux/IOS command. It computes delivery from current typed
policy, appends only accepted local/remote records, and advances an emission
index. Each supplied endpoint has a circular history measured in **records**;
this is deliberately different from IOS `logging buffered` byte-size arguments.
When full, the oldest record is removed first. Source policies and collector
selectors may change without rewriting already accepted history. Remote records
retain the facility used at emission time. The index is not a timestamp,
authenticated receipt, clock or protocol sequence number.

`current_probe`, `current_remote` and `checks` are non-mutating fresh decisions.
Checks never depend on old records or a cached success flag. A correct policy can
pass before generating traffic, and breaking that policy fails its next check
while historical accepted records remain visible. Requirements may check accepted
and rejected events, distinct sources, collectors and local behavior. The core
compares outcomes, not exact policy strings; stricter or looser equivalent
settings can pass when they meet all required acceptance/rejection cases.
Future authored tasks must include relevant negative event cases and protected
peer requirements rather than reward an overly broad debug policy everywhere.

Diagnostic outcomes distinguish: not configured, process disabled, remote output
disabled, source severity filtered, unknown supplied collector, unavailable
supplied delivery, collector facility/severity filtered, and accepted with a
specific facility. The first applicable stage is reported; fixing it may expose a
later independent problem. Local outcomes distinguish process/output disabled,
severity filtered and accepted. Friendly explanatory text is available alongside
explicit typed outcomes. Reports are labeled virtual and historical; they do not
pretend to be exact `show logging` output from a particular IOS platform.

## State, safety and resource contract

- At most8 sources,8 collectors,32 event templates,64 unique directed delivery
  facts,8 configured hosts per source and24 selectors per collector.
- Endpoint identities are distinct supplied unicast IPv4 values. Names are1–64
  printable UTF-8 bytes. Event IDs are1–32 lowercase ASCII/digits/underscore/dash;
  subsystem labels1–16 and mnemonics1–24 uppercase ASCII/digits/underscore.
  Event messages are1–256 printable UTF-8 bytes. Control characters are refused,
  so a message cannot inject a newline into a report. Shell-looking printable
  text is ordinary data and is never evaluated.
- Each endpoint retains1–32 records. History records contain event IDs, source
  IDs and emission indices; remote records also contain captured facility.
  They never duplicate full message text. State validation rejects unknown
  identities, same-index conflicting events/facilities, out-of-order records,
  excess histories and remote records inconsistent with immutable delivery facts.
  It permits old records that differ from **current** mutable policy.
- All public state queries/actions validate state. Setters validate proposed
  policy before cloning bounded state. Emission checks index overflow before
  clone/publication. Failed setters/emissions leave the whole state unchanged.
  Buffer clearing only removes that selected history; policy/index remain.
- Raw JSON is limited to256KiB before decode. Maps reject duplicate/excess keys
  before deserializing their values; sequences reject an excess element before
  its body. Strings and all known collection counts have typed bounds. Unknown
  fields and invalid versions are refused. `decode` returns validated state;
  direct serde construction must still call `validate` before use.
- Maximum32 distinct source/event requirements; at most9 outcome rows per
  requirement. Reports/check explanations are capped at32KiB. Callers must treat
  a returned limit/error as a failed check, never as a partial pass.
- The finite core has no host filesystem, subprocess, sockets, timing, sleeping,
  generated code, plugin loading or new dependency. `Ipv4Addr` is a value type.
  No packet/retransmission loop or learner-supplied executable is involved.

### Conservative compact-JSON bound

Every valid typed state fits the256KiB raw limit, including worst permitted JSON
escaping and maximum history. Bounds below deliberately exceed exact encodings:

| Category | Conservative encoded bytes |
| --- | ---: |
| 512 retained records, each at most192 bytes | 98,304 |
| 32 event templates, each at most1,024 bytes | 32,768 |
| 16 supplied endpoints, each at most512 bytes | 8,192 |
| 64 directed delivery facts, each at most128 bytes | 8,192 |
| 8 source policies, each at most1,024 bytes | 8,192 |
| 8 collector policies, each at most2,048 bytes | 16,384 |
| All map/container punctuation and keys beyond these rows | 8,192 |
| Version/index scalar envelope | 256 |
| **Total** | **180,480** |

Printable UTF-8 excludes controls, so only quotation marks and backslashes expand
in JSON: at most2 encoded bytes per input byte, not6. A template has at most512
escaped message bytes plus unescaped bounded subsystem/mnemonic/identifier and
fixed field names;1,024 safely exceeds the total. Records contain only bounded
identifiers, IPv4 strings, twenty-digit `u64` indices, small facilities and fixed
field names, safely below192. Endpoint labels have at most128 escaped bytes.
A collector policy has at most24 two-digit keys and fixed severity names.
Container slack separately exceeds all16 history map keys, schema field names
and separators. `encode` still checks the exact resulting length defensively.
The proposed maximum-state regression constructs all32 templates with escaped
quotes/backslashes, all16 endpoint histories full, all24 selectors,64 delivery
facts and large twenty-digit indices, then requires exact resume roundtrip and
all reports within their independent output cap. This regression is **UNRUN**.

## Deliberate exclusions

No real IOS configuration parser/console/monitor output, timestamps, persistence
files, SNMP traps, arbitrary log files, TLS, encryption/authentication, source
interfaces, routing, IPv6, reliable delivery, replay protection, rate limiting,
rotation, wire serialization or real collector services. No evidence of authentic
network delivery is claimed. Supplied fixture facts and authored authority must
remain separate from learner edits when integrated; self-carried JSON is not an
authentication mechanism. No current full-objective or exam preparation claim.

## Primary sources and objective version

Checked2026-10-29. Original implementation and wording; no upstream code copied.

- Cisco, [CCNA200-301 v1.1 official exam topics](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),
  p4 objective4.5, “Describe the use of syslog features including facilities and
  levels.” Current1.1 only; announced future2.0 objectives are separate planning.
- Cisco, [IOS15S basic system management: troubleshooting and logging](https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/bsm/configuration/15-s/bsm-15-s-book/bsm-troubleshooting.html).
  Read logging process/console distinction, circular local buffer, multiple hosts,
  local/remote thresholds, numeric0–7 urgency and facility selection. This guide
  describes particular IOS releases, not a universal command/output guarantee.
- Cisco, [IOSXE2 system message logging guide](https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/esm/configuration/xe-2/esm-xe-2-book.pdf),
  message subsystem/severity/mnemonic syntax. The Cisco subsystem label and RFC
  facility are explicitly distinguished in this implementation.
- IETF, [RFC5424 section6.2.1](https://www.rfc-editor.org/rfc/rfc5424.html#section-6.2.1),
  facility/severity tables and priority calculation. The model uses these values;
  it does not implement the full protocol or assert its transport/security.

## Integration gates and future content

This proposal is pure source only:19 prospective core groups, uncompiled and
UNRUN. Root must add the narrow library export, compile, run all groups and
Clippy, obtain independent review and close any findings before integration.
No scenario, State/Goal variant, progress format, UI or catalog count changes
are included here. Objective4.5 remains Open.

After actual core gates, planned distinct investigations are local urgency vs
debug noise; remote export threshold/host repair; RFC facility/subsystem collector
mismatch; and multi-source protected-policy handover. Each needs separate values
for Introduction → runnable practice → full assignment and behavior-based positive
and negative current-policy cases. These are planned tasks, not authored labs.
