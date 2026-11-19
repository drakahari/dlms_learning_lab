# Finite TCP/UDP teaching model

Implementation in progress, 2026-11-05. The 294-lab review candidate predates
this source. Two transport labs are now registered in source, with native/final checks pending;
source/tests alone earn no finished-lesson or full CCNA objective credit. Current CCNA v1.1 objective 1.5 now has narrow Partial practice after source/native
checks; the broader protocol limits below remain. Future v2.0 numbering is separate.

## Meaning of an observation

TCP models a normal active/passive open: SYN, reverse SYN-ACK, then final ACK.
Each peer has its own state. Receiving the SYN-ACK establishes the client;
receiving the final ACK establishes the server. One supplied text fragment can
then be sent. Receipt requires delivery through the supplied network and a
matching listener. TCP is a byte stream; this finite activity does not imply
that real application writes correspond one-to-one to packets or messages.

UDP emits one datagram without a TCP handshake or invented acknowledgment.
Sending, network delivery to a host, and application receipt are distinct.
A missing listener is reported separately from filtering. No timeout, RST,
ICMP message or operating-system command result is fabricated.

A conversation uses protocol and full source/destination address/port pairs.
TCP and UDP can use the same numeric port independently. Activity ports are
1–65535; this is an exercise restriction, not a claim that UDP source port zero
is universally invalid. Listeners bind exact supplied host addresses.

## Routing and policy

Every generated packet is independently inspected by the existing IPv4
route/interface and stateless ACL engines. SYN-ACK reverses both address and
port pairs. Host IDs and interfaces are bound explicitly by authored topology,
never inferred interface names or real network discovery. The network must be
deliverable without ACLs before policy restriction can count as evidence.
Five-tuple ACLs cannot distinguish a forward SYN from a final ACK with the same
tuple. The core's final-ACK-loss test uses a fixed typed development fixture;
the application does not teach that the old ACL syntax matches TCP flags.

## Bounds and isolation

The reducer has no sockets, subprocesses, host I/O, clock or executable scripts.
Trusted Rust callbacks classify packets; learner-supplied callbacks are not
accepted or claimed sandboxed. Configuration raw decoding caps64KiB before
Serde, then checks six hosts/eight listeners/eight exchanges, unique identities,
1024UTF-8-byte payloads and16KiB aggregate text. A direct typed caller must
validate before cloning. Bounded raw decoding is not allocation-free Serde.
Each exchange generates at most four packets; each batch at most32 under a
64-event guard. Retained classifier detail is capped, including error text.
Invalid actions and classifier failures leave reducer state unchanged.

Settings drafts and source IDs are persisted separately from authored goals.
No packet report is deserialized as completion authority. Checks recompute
outcomes from caller-held source and current settings; edits clear prior checks.
Practice uses distinct hosts/settings and cannot award assignment credit.
Protected endpoint rows and topology cannot be edited through policy exercises.

Not modeled: sequence/acknowledgment numbers, retransmission, timeouts, data
acknowledgments, close/reset, windows/congestion, segmentation/reassembly,
checksums, simultaneous open, handshake data, IPv6, wildcard binding, NAT/PAT,
real application protocols or a complete TCP implementation. Broader roadmap
coverage and physical-equipment experience remain separate.

## References and current evidence

Primary references inspected2026-11-05; original implementation and explanations,
no extracted standards code or copied diagrams:

- [RFC9293 / STD7, August2022](https://www.rfc-editor.org/rfc/rfc9293.html),
  especially key concepts, states and normal-open Figure6.
- [RFC768, August1980](https://www.rfc-editor.org/rfc/rfc768.html), UDP fields
  and absence of guaranteed delivery/duplicate protection.
- [RFC6335 / BCP165, August2011](https://www.rfc-editor.org/rfc/rfc6335.html),
  protocol-specific port namespaces.

MAIN's guarded staged library39.25s/app21.47s and15 core tests pass (R01–R03).
Tests exercise each handshake-leg loss, separate peer states, missing listeners,
same-port namespaces, different clients, byte bounds, malformed configuration,
callback failures, action legality and actual ACL tuple projection. The
source-held workspace and real routing adapter are being tested next. This
does not establish lesson, native, platform or beginner-comprehension acceptance.
