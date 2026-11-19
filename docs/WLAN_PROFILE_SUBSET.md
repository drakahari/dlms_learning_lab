# Supplied WLAN-profile compatibility

This original in-process model compares proposed WLAN settings with supplied saved
client profiles. It does not perform encryption, authentication handshakes, radio
association, scanning, packet transmission or host configuration. Three WLAN
lessons are now registered in source424labs/163units, with individual native teaching
review recorded. The separately identified421R1 review candidate predates this unit. See CHECKPOINT.md.

## Supported configuration

- One to four supplied WLANs; one to twelve supplied client profiles. Managed and
  protected WLANs are explicit. Stable identifiers, labels and collections are bounded.
- Exact, case-sensitive printable-ASCII SSIDs of1–32bytes. Spaces are meaningful;
  neither names nor passphrases are trimmed into a successful match.
- Open, WPA2-Personal with PSK/AES-CCMP, or WPA3-Personal with SAE/AES-CCMP. These
  are distinct typed configuration choices. No cryptographic implementation is
  claimed: only supplied synthetic passphrase compatibility is compared.
- Disabled, optional or required protected management frames (PMF). WPA3-only
  requires Required. A WPA2 client profile without PMF cannot satisfy Required;
  Optional permits that profile. Supplied PMF capability means the profile offers
  that capability, not a live hardware measurement.
- This activity subset accepts8–63 printable-ASCII characters for a training
  passphrase. Other SAE ranges and64hexadecimal PSKs are excluded explicitly;
  this is not a universal rule for every device or security implementation.

Use only the supplied fictional training values. Do not enter real credentials.
Failure diagnostics never echo either passphrase. Example and assignment values
will remain separate; matching a configuration is not proof of equipment operation.

## Decisions and task foundation

Every client is evaluated from current settings: missing exact name, disabled WLAN,
incompatible authentication, unavailable required PMF, mismatched training passphrase,
or configuration-compatible with the actual method and modeled protection state.
Malformed fields and protected-setting changes defer client checks. Deferred results
cannot earn negative-case credit. Duplicate-name profile selection is outside this
small model and produces a configuration diagnostic rather than an invented choice.

The task layer declares editing scope, site policy, all supplied client expectations
and a reachable reference. It refuses missing/duplicate/foreign cases, unsatisfiable
references and initially solved independent tasks. Grading evaluates policy and
every client, rather than requiring equality with one reference configuration.
For example, Disabled and Optional PMF can both satisfy a WPA2 legacy-client task
when its site policy permits either. Requiring PMF can preserve a modern client's
compatibility while correctly failing an older profile.

Session version1 stores separate practice and assignment drafts and checked-field
snapshots. Edits clear current results; outcomes are recomputed rather than storing
a success boolean. Filling a worked example does not check or complete it. Reset
affects only the selected draft. Source/engine/pack/catalog-aware progress binding and synthetic421-attempt retention
checks pass. Three complete source lessons and the task-specific native workspace
are registered; individual native review is recorded and final package admission
remains pending. Coverage is limited to the explicitly documented Partial practice
slices; no full-objective or real-wireless conformance claim is made.

## Exclusions

No WPA2/WPA3 transition mode, enterprise/RADIUS/802.1X, legacy WPA/TKIP/WEP, OWE,
arbitrary Unicode SSIDs, vendor-controller emulation, roaming, discovery, captive
portals, DHCP, routing, measured RF, cryptographic negotiation or Internet access.
Unknown serialized modes refuse. A compatible configuration is not evidence that a
real client connected, that traffic was encrypted, or that a site is secure.

## Authoritative references

Opened2026-11-11 local /2026-11-12UTC:

- [Current CCNA200-301 v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf):
  roadmap1.11b/d,2.9,5.9 and5.10. Future versions remain separately tracked.
- [Cisco SSID configuration, document116586](https://www.cisco.com/c/en/us/support/docs/interfaces-modules/security-modules-routers-switches/116586-config-ap-00.html),
  updated2022-12-20. The document's device-specific character restrictions are
  not presented as universal SSID restrictions.
- [Cisco Wireless Controller8.10 WLAN security](https://www.cisco.com/c/en/us/td/docs/wireless/controller/8-10/config-guide/b_cg810/wlan_security.html):
  personal-security configuration, PSK formats, AES-CCMP, separate SAE and PMF.

No upstream code, diagrams or documentation text is imported. Focused model tests
are development evidence, not vendor conformance, independent novice acceptance,
physical-device, screen-reader or Windows/macOS runtime acceptance.
