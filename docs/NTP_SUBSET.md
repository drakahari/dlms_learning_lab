# Virtual NTP foundation v1

This pure model is a source foundation, not an integrated lesson, wire-protocol
implementation or completed objective. The current CCNA v1.1 4.2 row remains
Open until authored lessons, native interactions and verification are integrated.
No new catalog entries, UI, progress format, dependencies or host facilities are
added here. The existing upcoming CCNA v2.0 rows remain separate.

## Supported teaching problem

A virtual device has zero or one configured IPv4 upstream. An upstream can be
another device or a supplied external reference. Reference health and stratum are
scenario environment evidence; learner configuration cannot change them. Devices
can both consume a source and serve downstream clients. Status follows the
configured dependencies, not whichever reachable reference is closest. A healthy
source at stratum N gives its client stratum N+1; a device that would reach16 is
unsynchronized and cannot provide a usable source downstream. Reference identity
is the immediate configured upstream; a separately displayed chain identifies the
ultimate supplied source. A low stratum is neither authentication nor proof of UTC.

Directed delivery fixtures describe request delivery, reply delivery and UDP123
permission for an exchange. They are input facts, not computed routes, ARP, ACLs,
packet captures or actual UDP traffic. A missing fixture is unreachable. An unknown
but syntactically valid configured address is retained and diagnosed, not rejected
as though IOS checked network existence at configuration time. Offline devices,
blocked or asymmetric exchanges, missing configuration, unhealthy references,
loops and exhausted strata produce current-state failure reasons and complete
source chains. Removing or breaking an upstream immediately invalidates dependent
success. No old success flag is stored.

The dependency resolver deliberately diagnoses any explicit configured cycle.
This is a teaching graph invariant, not an implementation of RFC reference-ID
loop tests or evidence that actual NTP detects arbitrary loops in the same way.

## Commands and typed operations

Use `ntp server IPv4`, `no ntp server IPv4`, `show ntp status`,
`show ntp associations`, and `show running-config` on a selected virtual device.
Command context is selected by the caller; IOS privilege/configuration-mode
navigation is not modeled. Reports are explicitly labeled virtual projections,
not copied IOS transcripts. Unknown commands or optional flags fail honestly.
The second different `ntp server` is rejected as unsupported, explaining that real
IOS supports multiple associations. Remove the current association before adding
another. Repeating the same configuration is harmless; removing a different
association changes nothing. The typed `configure` operation replaces/removes one
upstream for future native configuration controls. Supplied reference nodes are
read-only through all configuration/command APIs.

Status, association and configuration reports derive from current typed state.
There are no fabricated delays, offsets, dispersion, jitter, polling timestamps or
377 reach registers. An association's source-stratum column is unavailable (`--`)
when this model cannot deliver the exchange or the source is offline. Otherwise
it is the dependency model's current source stratum, never a claim of a sampled
packet. `*` and `~` have accompanying readable selection text. Diagnostics explain
the actual failed dependency and support retry; grading accepts any upstream named
in the explicit allowed-source set, requires current synchronization and checks
all supplied requirements independently. Validating the whole graph also rejects
unrelated malformed nodes/delivery fixtures rather than ignoring them.

## Bounds, persistence and isolation

The version1 snapshot admits at most32KiB before JSON parsing,16 nodes, bounded
32-byte ASCII labels,256 delivery entries, no duplicate addresses/labels or pair
identities, and no unknown fields. Collection visitors bound admission; every
public state operation validates all members. Meaningful distinct directed
non-self pairs naturally number at most240. Each dependency walk visits at most16
nodes before a source or failure; a loop-closing chain can contain17 addresses.
The all-node report therefore performs at most256 recursive node visits, with
bounded scans of delivery fixtures. Commands have a512-byte bound and accept only
one command. Output is bounded by the16-node graph and fixed report text. There is
no unbounded learner program, host path, process, socket, clock or wall-time API.

`from_json` validates version, sizes and referential integrity; `to_json` also
rejects excess raw output. Generic serde deserialization is data decoding only;
callers must use validated admission and never bypass public state operations.
Scenario authors supply references and exchange facts; future integrations must
use `environment_matches` / `check_with_environment` at restored-attempt admission
to preserve node identities/roles/availability, supplied reference health/strata
and exact ordered delivery fixtures while allowing device upstream edits. Ordinary
`check` validates current state but cannot infer an absent author baseline.
The current foundation does not add a progress migration or invent credit.
Reset means restoring the scenario's supplied initial state; resume serializes
actual configuration and grades it again. Tests exercise these model operations,
not an app-level cold process, UI or migration claim.

## Deliberate omissions

No actual clock value, timestamp arithmetic, four-packet timing equations,
frequency correction, filter, discipline, offset convergence, holdover/aging,
poll cadence, persistence of NTP observations, multipeer selection/combining,
false-ticker voting, symmetric peers, broadcast/multicast, `ntp master`, local
clock fallback, orphan mode, DNS, IPv6, authentication, NTS, leap seconds,
source-interface selection, full IOS command modes, real routing or device-specific
hardware behavior. The supplied health flag is an authored fixture, not a trust
algorithm. The model's instantaneous recomputation is not real convergence time.
It must not teach that configuring one low-stratum server is a production best
practice or sufficient to establish correct/safe time. This is NTP hierarchy
practice, not SNTP, a substitute for actual equipment or complete4.2 coverage.

## Primary references and check date

Checked2026-10-29. The primary pages below were accessible without private access;
no source-access gap was encountered for this narrow subset. All code and examples
are newly authored; no RFC reference code, Cisco code or manual passage is copied.

- [Current CCNA v1.1 objective PDF](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf), page3,4.2: client/server configuration and verification scope.
- [Cisco transition notice](https://blogs.cisco.com/learning/stay-on-track-get-certified-before-the-ccna-refresh): current v1.1 through2027-02-02, v2.0 starts2027-02-03. Existing future-version mapping receives no inherited credit.
- [Cisco IOS XE17 System Management guide, NTP](https://www.cisco.com/c/en/us/td/docs/routers/ios/config/17-x/syst-mgmt/b-system-management/m_bsm-time-calendar-set.html): hierarchy, synchronized sources, client/server associations, redundancy and omitted selection/security capabilities.
- [Cisco Basic System Management command reference](https://www.cisco.com/c/en/us/td/docs/ios-xml/ios/bsm/command/bsm-cr-book/m_bsm-cr-n1.html): `ntp server`, its no form and status/association inspection. Exact platform options vary; only the disclosed command subset is implemented.
- [RFC5905, June2010](https://www.rfc-editor.org/rfc/rfc5905.html), sections7.3,9 and11: strata, reference identity, unsynchronized status and substantial protocol/selection behavior deliberately excluded here. Consult its current status/errata; this foundation claims neither full RFC conformance nor code equivalence.

The18 core tests pass through the serialized runner. They exercise derived
chains and reports, alternatives, wrong-source and broken-transport outcomes,
stratum limits/cycles, read-only fixture preservation, serialization and hostile
inputs. All-target Clippy also passes with warnings denied. The combined inclusive
application gate remains pending. There are no
accepted native interactions or authored lessons yet; objective4.2 remains Open.
