# Telnet transport diagnostics: bounded native client

2026-11-16/17. Integrated and verified in the two existing LPIC-2 service-exposure
investigations. Separate guided practice and full original state-based assignments
remain; no new title or utility-specific mastery requirement. Catalog507/204 is
preserved.45current scoped Rust cases, eight unchanged prior nc-model checks,
561focused Python/strict coverage and Cargo1all-targetClippy pass. Private upgrade,
installed271actions/54captures and normal two-process cold reopening pass. Ten
installed and all eight cold originals inspected, including the corrected single
local prompt and active local `qui` draft restored before quit/repair.212.4.T1 is
Partial only. [Exact candidate/launch/evidence](CHECKPOINT.md). No full curriculum,
real-client, otherOS, screen-reader, physical-device or independent novice claim.
Bundled pending annotations are historical; frozen artifacts remain immutable.

## Supported design before teaching authoring

This is a TCP transport diagnostic, not a complete TELNET protocol or login
emulator. `telnet -c SUPPLIED_IP PORT` accepts only the supplied numeric IPv4
server and an explicit ordinary decimal TCP port. Omitting `-c` is also accepted;
no personal configuration is read in either case. The option is taught to make
clear that startup configuration is not part of the exercise. Other options,
names, IPv6, implicit/service-name/list/range ports, shell operators, quoting,
trace files, local shell escape, authentication, payload and option negotiation
are unsupported and produce diagnostics. Port23 is explicitly excluded because
the normal TELNET option-negotiation path is not implemented. Other ports are
1through65535; successful transport still does not prove application health.

The existing World derives reachability from real virtual listener ownership and
numeric INPUT/OUTPUT decisions. Open transport leaves the simulated client
running: there is no exit status yet. Refusal/drop completes the attempted
connection with status1; a drop is a modeled timeout, with no actual wait. A
stopped process can retain an accepting socket; an ended owner cannot. These
are observations, not assignment grades.

Changing the virtual server invalidates its observer snapshot and clears this
simulated client phase; reconnect after the change. This is an explicit workspace
reset of stale observations, not a claim to model established TCP session
continuity, socket teardown, retransmission or connection tracking. Changing
observer position is disabled while the client is active.

The native integration explicitly enters a local client prompt using a named
Escape action (equivalent to the real client's default Ctrl+]). Only then does
`quit` close the transport and end the client with status0. The literal characters
`^]` are not an invented shell command. Remote application data and other local
commands fail honestly. The current model has no timer, socket, network, subprocess
or filesystem API; it reads only supplied typed state. Commands are single ASCII
lines within1KiB. The existing workspace's history/console/memory caps must stay.

## Compatibility contract

Never add Telnet semantics to old Scan or ProbeV2 events: previously rejected
commands and diagnostics must replay exactly. Use a new versioned event and an
optional source-bound client phase. Missing old phase data defaults to absence;
omit an absent phase on serialization so source initial values stay exact. Replay
must derive and compare client state alongside observations, not trust imported
claims. Retain legacy nc/nmap credit, separate drafts and example isolation.
Original Task/Goal values and independent requirements remain unchanged. Existing
completion cannot be relabeled as utility-specific Telnet mastery. An older app
cannot interpret new history; keep review profiles separate.

The optional phase is replay-bound; legacy absent fields remain absent and old
Scan/ProbeV2 commands and diagnostics stay exact. New ObserverV3 and
TelnetEscapeV3 records model the lifecycle. Both teaching overlays and all prior
requirements are preserved. Supported actions have model, workspace, authored and
native-widget tests. Installed reopening must still restore an active local prompt,
independent pending drafts and exact raw progress before repair. No coverage
promotion follows solely from the technical test count.

## Primary reference and current validation

[OpenBSD telnet manual](https://man.openbsd.org/telnet), checked2026-11-16,
documents explicit host/port connection, non-default-port negotiation behavior,
the local escape prompt and quit. Its `-n` means a trace file, unlike nc's numeric
lookup option. No trace or startup file is read by this simulator. The GNU
inetutils manual URL was unavailable during this check; no GNU-specific fidelity
claim is made. No upstream code was imported and no real Telnet client/network
comparison has run.

`tests/telnet_client.rs` currently verifies numeric syntax/bounds, unsupported and
escape expressions, ongoing open transport without fabricated exit0, refusal
versus drop, escape/local quit, explicit payload/local-shell rejection, supplied
target/viewpoint binding and stopped-versus-ended listener ownership. These eight
checks are technical model evidence, not beginner, screen-reader, physical-device,
otherOS or full TELNET conformance acceptance.

## Representative actual Fedora screens

- [Introduction/worked reasoning](../.build-artifacts/native-runs/run-nrtdl_n5/screenshots/workspaces/service-exposure-1-worked.png)
- [Separate practice and client exit](../.build-artifacts/native-runs/run-nrtdl_n5/screenshots/workspaces/service-exposure-1-telnet-exited.png)
- [Independent assignment](../.build-artifacts/native-runs/run-nrtdl_n5/screenshots/workspaces/service-exposure-2-assignment.png)
- [Restored local prompt and unfinished draft](../.build-artifacts/tmp/native-reopen-adace94f423f1dc50e459df8/read/reopen-service-exposure-reader-example-observer.png)

These are local ignored technical evidence, not mockups or an independent learner
study. Both normal cold phases exit0 at the unchanged120s limit; source history,
raw progress/preferences and distinct drafts compare before native repair.
