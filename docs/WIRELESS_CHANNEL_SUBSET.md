# Supplied-survey wireless channel planning

This original in-process model plans channels for supplied radios. It does not
scan, activate a radio, transmit packets or predict RF performance. Source421 adds
two authored CCNA lessons. Individual native review and separate421R1 packaged
admission pass; CHECKPOINT.md records exact identities and acceptance limits.

## Explicit site policy

The activity fixes the band at 2.4 GHz and the width at 20 MHz. Its managed radios
must follow this site's conventional **1 / 6 / 11** plan. Channels1–11 are accepted
as editable values so mistakes can be investigated. Other channel numbers can be
legal in real configurations; failing this task's policy does not declare them
universally illegal. Other bands, widths and regulatory plans are not modeled.

A supplied survey lists mutual-hearing pairs. Equal channels mean shared airtime,
not failed client association. Different channels fewer than five channel numbers
apart are flagged under this conservative planning rule. Valid site-plan choices
separate the pairs. No packet loss, Mbps, signal strength, coverage distance,
receiver sensitivity or spectral shape is calculated. Radios without a listed
relationship may reuse a channel only within this supplied model; that is not proof
that a real site is free of interference. Every supplied survey is checked against
the same configuration. Fixed neighbours are source facts outside editing authority.

## State, grading and bounds

Typed Network/Radio/Survey/Pair objects hold facts; independent native drafts hold
managed channel fields. Checks derive policy, preservation and each pair's outcome.
They do not compare the whole plan to a required reference list. The multi-survey
assignment accepts six equivalent configurations. Blank/malformed/out-of-band input
is diagnosed separately; survey evaluation waits for valid numbers rather than
using a stale previous configuration. A different legal number can still fail the
site policy or overlap a neighbour. Correct individual rows cannot hide failure.

Limits:2–8radios,1–3surveys, at most28distinct pairs per survey,8fields of8bytes,
64byte ASCII identifiers and120byte labels. Deserialization bounds collections and
text and refuses unknown fields. Duplicate/foreign/self relationships, missing
fields, edited protected radios and unreachable references refuse. No scripts,
external files, services, credentials, network targets or new dependencies.

Progress schema1 remains. Session version1 has separate assignment/example fields
and the last checked field snapshot. Edits invalidate current results; current
success is rederived from source facts. Stale imported snapshots cannot retain
success after values change. Source ID, scenario version, fresh initial state and
metadata are validated; bare saves require the actual catalog. Reset changes only
the selected attempt or example. No cached RF output, mastery or observed execution
is invented. Existing419 scenarios and161 units remain exact through retained APIs.

The @wireless-set and @wireless-check strings are fixed authoring reachability
helpers, not IOS/Linux commands or learner terminal practice. The native UI uses
channel fields and Check plan. Nothing passes a learner value to a host process.

## Authoritative references

Checked2026-11-11:

- [CCNA200-301 v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),
  specifically1.11a. SSID, encryption, broader RF, AP/controller modes and WLAN
  deployment/configuration are separate gaps. Futurev2.0 stays separately tracked.
- [Cisco Wireless RF Reference Guide](https://www.cisco.com/c/en/us/td/docs/wireless/controller/9800/technical-reference/wireless-rf-reference-guide.html),
  updated2024-04-18: channel widths, conventional channel planning and survey-based
  design. No upstream text, images or code are imported; diagrams/engine are original.

These are bounded educational constraints, not equipment conformance, a real survey,
vendor endorsement or full exam preparation. Independent novice, screen-reader,
physical-device and Windows/macOS runtime acceptance remain unobserved.
