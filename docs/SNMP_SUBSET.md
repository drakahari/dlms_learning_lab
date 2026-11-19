# Virtual MIB monitoring — capability in development

Two lessons are registered in current428/165, individually native-reviewed with narrow4.4Partial practice. The verified428R1candidate
is separately identified in CHECKPOINT.md. Original scenarios, progress schema,
workspaces, Bash evaluator and saved learner data are unchanged.

## Supported read-only model

Original in-process Rust only. A supplied virtual agent has1–12interfaces, unique
positive Integer32 indexes, printable64-character descriptions, known positive
Gauge32 link capacity, Up/Down administrative/operational states and octet counters.
GET/GETNEXT use numeric lexicographic OIDs and return typed values or distinct
noSuchObject/noSuchInstance/endOfMibView outcomes. Each query accepts1–16OIDs, at
most32arcs/384characters. Generated tables have at most109objects. Unavailable
agents are explicit supplied conditions, not measured real network timeouts.

Objects: sysUpTime.0; ifDescr, ifSpeed, ifAdminStatus, ifOperStatus, ifInOctets,
ifOutOctets, ifHCInOctets, ifHCOutOctets and ifCounterDiscontinuityTime instances.
Counter32 values use the low32bits; Counter64 preserves the supplied total. This
virtual agent's narrow table does not claim every real agent implements that table.
Only numeric OIDs are parsed; no general MIB compiler or symbolic-name language.

Rate calculations use supplied exact1–3600second intervals and100HzTimeTicks, with
uptime wrap handled separately from inconsistent sample clocks. Counter-discontinuity
changes, invalid widths, capacity violations and possible unobservable whole wraps
return an explanation rather than a fabricated rate. Bytes/octets are converted to
bits with exact integer/rational comparisons. This assumes the supplied samples'
known capacities and timing; it is not a real polling/jitter/reset-detection guarantee.

## Monitoring layer being tested

The typed plan/observation layer discovers interface instances by GETNEXT over
ifDescr, then GETs the actual selected state/counters. It does not inspect expected
answers. Link rules distinguish operational-down from unplanned-down; Quiet does
not mean Up. Traffic plans select direction, counter width and percent threshold.
Missing/duplicate descriptions, unavailable agents, changed instances/capacities,
missing samples and unusable counters are distinct Unknown results. Malformed plans
or source metadata are errors, never successful Unknown cases. A run has at most
40object reads. Source/task/session and catalog-aware progress bindings are implemented and tested;
a native MIB browser/configuration workspace and two authored assignments are now
registered. See SNMP_MONITOR_LESSONS.md for actual evidence and pending acceptance.
Synthetic binding fixtures are never counted as authored lessons.

## Exclusions and verification

No sockets, packets, BER/PDU codec, host files/commands, live SNMP, credentials,
communities, SNMPv3security/crypto, writable SET, GETBULK, complete MIBs, link testing/
other operational states, unknown or above-Gauge32 link speeds, real sampling jitter,
notifications/traps/informs or vendor-agent interoperability. Those functions require
separate capabilities before dependent lessons. No dependencies were added.

Initial12model tests and library Clippy pass. They cover numeric ordering, actual
state-derived values, unknown/unsupported input, bounds, serialization, counter/time
wraps, rate units, ambiguous intervals and discontinuities. Later38model/monitor/task/binding checks pass, including actual disk retention of
all426prior attempts/favorites, separate drafts, malformed/prechecked/unreachable
packs and atomic failed saves. Readable actual rates do not change exact threshold
grading. Six authored/current-retention and ten widget/capture checks pass. Native R1/R2 and
actual installed428R1each completed210actions/53images. R2visual findings are fixed;
selected packaged originals and NFS cold reopening were inspected. Exact receipt and
limits are in CHECKPOINT.md; no independent novice acceptance follows.
All runs use MAIN's shared lock, jobs2/runtime2, disk artifacts and unchanged pressure
guards. Previous426candidate/evidence are immutable after source retention.

References opened2026-11-12; original explanatory prose/code only:

- [Current CCNA v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),4.4; future versions remain separate.
- [RFC2863](https://www.rfc-editor.org/rfc/rfc2863.html), interface types/counters/discontinuity semantics.
- [RFC3416](https://www.rfc-editor.org/rfc/rfc3416.html), read operations and response exceptions.
