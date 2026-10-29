# Physical-uplink model foundation

Source-only first capability batch, checked2026-10-28. This pure module has no
registered curriculum, native editor or completion credit yet. It does not alter
saved progress, existing labs, existing networking models or the host. All
operations use bounded typed in-memory data. No dependency was added.

## Source boundaries

Current CCNA200-301 v1.1 objectives1.3 compare physical interface/cabling types
(single-mode fiber, multimode fiber, copper; shared media versus point-to-point).
Objective1.4 concerns interface/cable problems, including collisions, errors,
rate and duplex. The official issuer boundary is the
[CCNA v1.1 topic PDF](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf).
Future exam versions receive no inherited coverage. Existing normalized source
fingerprints and Open coverage rows are unchanged.

Technical sources, rechecked2026-10-28:

- Cisco [Ethernet negotiation guide](https://www.cisco.com/c/en/us/support/docs/lan-switching/ethernet/10561-3.html): supplied10/100 Auto/Fixed parallel detection and duplex resolution, full-duplex Gigabit device restrictions, and the meaning/ambiguity of interface counters.
- Cisco [Gigabit SFP data sheet](https://www.cisco.com/c/en/us/products/collateral/interfaces-modules/gigabit-ethernet-gbic-sfp-modules/datasheet-c78-366584.html): named copper/SX/LX-LH engineering profiles. Use its specified media, distances and platform caveats; a transceiver name alone does not establish arbitrary port compatibility.
- Cisco [port/interface troubleshooting](https://www.cisco.com/c/en/us/support/docs/switches/catalyst-6500-series-switches/12027-53.html): collision/CRC observations have multiple possible causes and must be compared with configuration and fresh evidence.

The subset is deliberately narrower than IEEE802.3, IOS, vendor hardware support
or a cabling qualification instrument. It has no vendor endorsement or physical
conformance claim. Sources are cited rather than redistributed as copied guides.

## Implemented pure semantics (runtime gates pending)

A topology supplies exact point-to-point port pairs, an immutable required cable
span for each route and a finite cable inventory. Selecting a spare shorter than
the required route is reported as an unsuitable assembly, even if its technology
would otherwise support that short cable. No learner Action edits the route.
Port capabilities explicitly list allowed interface profiles and rate/duplex
abilities. Configuration selects each port's administrative state/profile/
negotiation and an exclusive cable for each link. Reusing an endpoint as a shared
segment, using a cable twice or inventing equipment identifiers is rejected.

Three named profiles have engineering qualification rules:

| Profile | Supplied cable | Qualification span |
| --- | --- | --- |
| Copper RJ45 | Cat5e |1–100m |
|1000BASE-SX on OM2 | Duplex OM2 |2–550m |
|1000BASE-LX/LH on OS2 | Duplex OS2 |2–10000m |

Lengths are represented in whole metres; the1m copper minimum is a representation
limit, not a claim that sub-metre copper cables cannot work. These are disclosed
supplied-profile rules, not universal fiber limits. A route
outside a profile reports **OutsideModel**, rather than asserting that real
carrier must disappear immediately beyond a distance threshold. A declared
unplugged fixture and administrative shutdown are separate explicit states.
No optical power, attenuation, wavelengths, connector loss, DOM, BER, autoneg
pulses, master/slave resolution or real carrier measurement is synthesized.
LX/LH over multimode/mode-conditioning and other standards are outside the subset.

Two Auto copper ports select the highest common advertised ability, preferring
full duplex at the same rate. The supported1G copper path is Auto/full only;
forced1G copper and optical negotiation produce explicit unsupported diagnostics.
This is a subset restriction, not a claim that all hardware prohibits those
settings. Matching forced10/100 rates establish independently configured duplex;
mismatched forced rates have no common operating rate. Explicit legacy parallel
capability permits unrestricted Auto opposite forced10/100: the Auto end detects
the rate and uses half duplex. Restricted-advertisement mixed negotiation is not
modeled and is rejected, avoiding an unsupported inference about PHY behavior.

## Evidence, probes and stale observations

`SuppliedReading` labels its origin explicitly as an authored sample or a
reported measurement, with its own note and counters. No measurement is collected
by the app. It
is not a live diagnosis or a saved completion credential. `ObservedCounters`
retains reported input/output packet and error values without imposing our
separate derived-counter subset identities or inferring a cause. It stays independent
of current derived simulation and cannot identify the cause of a CRC reading.

Live probes use finite offered-frame counts (maximum1024 per direction). They
produce before/after counter snapshots. Counters named `offered` are simulation
offers, not fabricated IOS packets-output readings. Receive impairment is an
explicit deterministic fixture (everyN-th offered frame), not a predicted BER or
an implementation of Ethernet FCS bytes. Duplex mismatch does not automatically
create CRC errors. Repairing configuration/cabling leaves historical counters;
compare fresh deltas. Clearing counters does not repair the channel.

A supplied mismatch-contention exercise creates deterministic overlapping pairs
only to expose the half-duplex side's late-collision symptom and lost deliveries.
Full-duplex sides never gain collisions. This is an abstract adverse test pattern,
not Ethernet slot-time, throughput or timing simulation. Half/half contention,
shared-media scheduling/backoff/retry and collisions on healthy half-duplex shared
segments are explicitly unsupported; a serialized half-duplex exercise is allowed.
Do not promote1.3.b shared-media hands-on coverage from this point-to-point model.

## Bounds, persistence and next integration seam

Topology128KiB; saved pure state512KiB; action32KiB (reported readings8KiB);24 ports/24 cables/12 links;
five unique advertised abilities; counters/sequence limited to10^9. Custom map/
set decoders refuse duplicates and excess entries before reading their body.
Unknown fields/actions fail. Updates are atomic over an admitted bounded clone;
refusal retains the complete caller. No real clock, file, process or network API
is present. Resume round-trips typed state; reset reconstructs the supplied initial
state. No learner progress schema is changed by the pure module.

Before authoring/registration, a separate definition/adapter must pin the full
immutable topology/environment and grade resulting supported configuration plus
independent deterministic probe cases. It must not grade a historical counter
value, supplied reading, cleared counter or recorded probe as mastery. Then add
Intro → isolated practice → full assignment workspaces, diagnostics/configuration
tools, alternate/wrong/partial cases, reset/resume and actual native acceptance.
This foundation alone covers no new objective and adds no authored lab count.
