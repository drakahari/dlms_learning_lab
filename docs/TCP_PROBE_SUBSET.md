# Numeric TCP probes in the supplied-server workspace

Development slice2026-11-16; existing service-exposure lessons, not new authored
labs. The verified507candidate predates this extension. Native/candidate acceptance
of this slice is pending until recorded in CHECKPOINT.md.

## Supported behavior, before lesson authoring

The Observer terminal supports `nc -n -v -z SUPPLIED_IP PORT`. The three flags
may be grouped or reordered before the two operands. `-n` avoids name lookup,
`-v` requests visible diagnostics and `-z` tests a connection without sending
application data. Only one supplied numeric IPv4 server and one decimal TCP port
from1through65535 are accepted. Duplicate flags, service names, ranges, quoting,
shell operators, listen/UDP/payload/TLS/proxy/execute options and real destinations
produce explicit diagnostics. This is a deliberately disclosed subset, not a
general shell or a claim that all netcat implementations use identical options.

The evaluator reads the existing typed virtual listener owners and INPUT/OUTPUT
decisions from the selected observer. An open connection reports success/status0.
A permitted path without a listener reports connection refused/status1. A dropped
request or reply reports a modeled timeout/status1; no clock or real network wait
is involved. A stopped owner can retain a TCP listener; reaching it does not prove
application health. A rejected command has status1 but no connection observation.
The native status line distinguishes these cases. No color-only meaning is used.

Commands are one ASCII line of at most1024bytes. Existing world, port, history96,
console32KiB and serialized-workspace bounds remain. The probe has no socket,
subprocess or filesystem API, sends nothing, and does not alter virtual state.
State-based grading remains unchanged; a successful probe does not earn credit.
The entire required client range is still checked rather than a sampled address.

## Persistence and compatibility

Legacy `Scan` history permanently uses its original nmap-only interpreter, including
old unsupported `nc` diagnostics. New native runs use the distinct `ProbeV2` event
for nmap or nc. New nmap reports are byte-identical to legacy reports. Resume checks
replay the recorded event version, preserving old outputs without inventing an
observation or utility mastery. Keep ProbeV2 semantics fixed too: future command,
option or diagnostic changes need a new event version, so previously rejected
forms do not acquire invented observations. Schema1 and source task/goal values
are unchanged.
An older executable cannot interpret the new event; review candidates use separate
profiles. Do not open a newer progress file with an older executable. Example and
assignment drafts remain separate; reset affects the selected attempt scope only.

## References and limits

Checked2026-11-16:

- [OpenBSD nc manual](https://man.openbsd.org/nc): numeric, verbose and zero-I/O
  TCP probing. Output is labeled simulated and is not promised byte-identical to
  a particular installed netcat build.
- [Ncat usage](https://nmap.org/ncat/guide/ncat-usage.html): connect/listen and
  implementation differences; Ncat itself is not implemented or executed here.
- [LPI201/202 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/):
  version4.5,201-450/202-450,212.4 lists nc. Telnet remains a separate unimplemented
  objective reference. Numeric TCP probing does not cover application dialogue,
  arbitrary port scanning, packet capture or vulnerability assessment.

Eight focused production-model checks passed before adding UI/history integration.
No real nc reference comparison has been run, no utility installed, and no real
target contacted. Later scoped integration/native evidence must be recorded
separately; parser tests alone are not lesson or novice acceptance.

## Current development evidence

37distinct selected Rust checks pass: core8,workspace10,source5,authored6 and
UI/capture/cold-plan8. The final negative-status layout has a focused light/dark,
wide/narrow,100/150/200%rerun. No new inclusive runtime suite. R1native flow passes
193actions/38captures; nine originals actually inspected. It exposed a diagnostic
cause requiring horizontal scrolling; the existing status line now names refusal
or modeled timeout, with no new panel. Normal cold writer1391235exit0/105.999s and
reader1391941exit0/49.214s pass within unchanged120second limits. All eight cold
originals inspected; negative status1 evidence and separate pending commands
restore exactly before native repair. A valid negative probe can accompany a
completed assignment. Legacy unsupported-nc history remains unsupported.

The final source guards/Python/lint, expanded native negative-feedback captures
and installed candidate verification remain pending. The initial presentation gate
retained one stale generated-inventory-link failure; it was not a grading failure
and must not be recorded as a pass. Inventory regeneration and a fresh gate follow.
No actual nc binary/reference comparison, real target, independent novice study,
physical-device, screen-reader or Windows/macOS runtime acceptance is claimed.

Representative original development screenshots (private local evidence):

- [Guided practice](../.build-artifacts/native-runs/run-ib2bbc57/screenshots/workspaces/service-exposure-1-nc-open.png)
- [Independent task](../.build-artifacts/native-runs/run-ib2bbc57/screenshots/workspaces/service-exposure-2-assignment.png)
- [Current restored negative feedback](../.build-artifacts/tmp/native-reopen-e5040f4e6dff79911b4f57b1/read/reopen-service-exposure-reader-assignment-observer.png)

The two reviewed sidecar entries retain the original independent requirements.
All341old bundled data hashes match retained507. New capability is not a new lab,
and old state-based completion does not demonstrate nc usage or mastery.

## Final prepackage gates

Final558scoped Python checks pass (201literal,210ERE,46presentation,35coverage,
55reopen guards and11candidate guards), with strict current coverage validation
and staged one-job all-target Clippy denying warnings. The generated inventory
now links both existing labs to their reviewed two-row overlay. No assertions or
source limits were weakened; historical producers/collectors remain unchanged.
No inclusive Rust runtime suite or installed-candidate acceptance is implied.
Candidate construction and actual installed native/cold checks come next.

## R1native presentation finding and focused correction

R1private lifecycle,203native actions/40captures and installed cold reopening
pass. Ten native and three representative cold originals were actually viewed.
The missing worked-example heading/divider is a real rendered issue; R1is marked
NONFINAL rather than accepted. Its exact selected inputs/artifacts were retained
before the source hold ended. The existing helper now produces one Worked example
heading and a separate Expected observations and actions caption. An additional
widget regression passes (38distinct selected Rust checks overall). Targeted
guards/lint and a fresh corrected R2candidate follow. No grading/progress change.

The focused correction gate passes: the new heading regression, both current
source-isolation suites (411checks) and one-job app/all-target Clippy. Unchanged
simulation, authored-data and other Python evidence is reused, not relabeled as
new executions. Fresh R2package/installed rendering and cold checks remain pending.

## R2final scoped Fedora verification

The corrected R2candidate is verified: private lifecycle,203native actions/
40captures, six inspected current originals, normal cold writer/read exits0,
three inspected current cold originals and exact selected readback/retention.
Both introductions have one worked heading/divider and a separate expected
caption. Negative feedback wraps cleanly at200%.38distinct selected Rust and
558scoped Python checks, strict coverage/inventory and all-target Clippy pass;
unchanged valid evidence is reused, not invented as new runs. The protected
launcher and exact identities are in CHECKPOINT.md. R1remains NONFINAL evidence
of the caught heading defect. No original installation/progress was replaced.
This is technical Fedora verification, not full curriculum, conformance, otherOS,
accessibility or independent novice acceptance. The R2source hold ENDED only after
3394selected files/159816229bytes were retained and all final bytes matched.
Never re-attest that retained build against later main-source changes.
