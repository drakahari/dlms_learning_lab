# Wired devices and routed paths

Two CCNA lessons extend the existing IPv4 foundations through actual simulated
cable editing. They reuse the established switching and static IPv4 engines.
The preceding 390 scenario objects and 149 unit values remain unchanged.

## Learning sequence

Open CCNA, then **Wired devices and routed paths**, after **IPv4 addressing and
diagnosis**. Introductions remain skippable and available again at any time.

- `ccna-device-path-patching` repairs exchanged workstation/server patch ends,
  preserves a monitoring endpoint and verifies local and routed replies.
- `ccna-device-path-broadcast-boundary` diagnoses an unintended Layer-2 bypass.
  IPv4 round trips already work. The learner must restore broadcast separation
  while preserving those paths.

Each Introduction explains endpoints, switch forwarding, separate router
interfaces and the difference between IP reachability and broadcast separation.
A worked table uses different IP networks from the assignment. Separate practice
provides explicit cable changes and observations. The Assignment retains its
faults, native topology, aligned cable-end selectors and one **Check paths** action.
Assignment answers stay out of the workspace until the learner solves the task.
Challenge presents the same task with unsolicited worked material hidden.

## Model and isolation

`device_path::Facts` binds every supplied IP interface to a distinct Ethernet
endpoint. It projects the current switching model's two-way visibility into IPv4
link domains. Interfaces drawn on the same router card remain separate Ethernet
endpoints. The existing forwarding engine then evaluates requests and replies
using the fixed addresses, gateways and routes.

All required paths are checked together. An Ethernet broadcast leak can fail even
when routed communication succeeds. Disconnecting an endpoint fails the required
local or routed path. Available equivalent switch-port layouts pass. Each cable
edit invalidates current feedback, and restored success is recomputed from state.

Eight attachments/cables, three switches and six IP devices bound the activity.
Persisted vectors and attachment strings have explicit decoding limits. Source
validation checks task identity, endpoint bindings, incomplete initial wiring and
reachable reference wiring. Work contains no host path, command or traffic target.
Invalid cable drafts remain editable. Checks explain duplicate jacks and invalid
layouts, rather than awarding partial fabricated packet results.

The scope is settled-state, acyclic, untagged VLAN1 Ethernet visibility combined
with ordinary /1–/30 static IPv4 forwarding. Device ports, addresses and routes
are supplied. No physical packets, ARP or MAC lifecycle, timers, media, live ICMP,
service application or Cisco CLI runs. Layer-3 switch configuration, firewalls,
IPS, wireless devices, controllers, PoE and broader architectures remain open.

## Verification checkpoint

The four projection tests and seven authored/content/progress tests pass. They
cover alternate ports, partial repairs, broadcast leaks with working IP, unplugged
networks, duplicate jacks, invalid source/progress, bounded decoding, independent
practice, selected reset and save/resume. Exact comparison preserves every
preceding scenario and unit. Two native-renderer tests pass for shared field
heights, horizontal bounds, reflow, Challenge and stale-success invalidation at
100/150% dark and narrow 200% light settings.

The development-native R7 flow passed 89 actions and 24 captures. Actual screen
review covered both Introductions, separate examples, assignments, alternate
ports, duplicate-jack diagnostics, a disconnected monitor, broadcast leakage
with working IP, retry, 100/150% dark and narrow 200% Light. It saved and reloaded
the synthetic progress in-process. Separate-process cable-draft reopening remains
unverified. Final checks pass 25 focused Rust integration tests, three UI tests,
all-target Clippy, 1,057 Python tests and strict coverage/inventory checks. The
first Python run found an out-of-date generated inventory; it was regenerated
and the entire Python rerun passed. A new Fedora package is next. The last
verified package remains the 390-lab candidate. No full inclusive Rust runtime
suite was rerun for this focused batch.

The early native driver clicked outside a scrolling popup. R1–4 retained the
failed evidence. Recording the actual popup-content clip and scrolling with the
pointer inside that popup fixed selection; the new widget regression reproduces
the real click. Strict visibility/20-attempt guards remain unchanged. R5 then
correctly refused a whole topology taller than the enlarged viewport. Captures
now target complete first/last cards and retain normal page scrolling. No text
was reduced and no clipping acceptance threshold was relaxed.

Screen review found long worked values in horizontal scrollers and overlapping
stacked graph lines. The new opt-in page now wraps complete worked values and
uses explicit current-neighbor text on narrow cards. Wide diagrams retain actual
center-attached lines. Path feedback names the endpoints and includes the final
destination in each request/reply trace. These presentation corrections leave
all outcome tests intact.

Representative actual development screenshots are under
`.build-artifacts/native-runs/run-r5y5wprt/screenshots/workspaces`.

- [Introduction](../.build-artifacts/native-runs/run-r5y5wprt/screenshots/workspaces/device-patch-introduction.png)
- [Guided practice](../.build-artifacts/native-runs/run-r5y5wprt/screenshots/workspaces/device-patch-practice-pass.png)
- [Broadcast fault with working IP](../.build-artifacts/native-runs/run-r5y5wprt/screenshots/workspaces/device-boundary-assignment-leak.png)
- [Narrow 200% Light addressing](../.build-artifacts/native-runs/run-r5y5wprt/screenshots/workspaces/device-patch-light200-topology.png)
- [Narrow 200% Light results](../.build-artifacts/native-runs/run-r5y5wprt/screenshots/workspaces/device-boundary-light200-result.png)

These are technical development checks. Independent novice comprehension,
screen-reader and physical-device acceptance remain unverified. Other operating
systems remain deferred.

The first authored fixtures used a benchmark address outside the existing IPv4
engine's disclosed private/documentation range and a nonexistent unit anchor.
They were corrected to supported separate example networks and the existing
`ipv4-foundations` unit. Tests and source-validation rules were retained.

## Sources

Checked 2026-11-11. Current CCNA v1.1 objective 1.1 covers device roles. These labs
support a subset of router, Layer-2 switch and endpoint practice. An addressed
server endpoint does not establish server-application coverage. No full objective
or complete exam preparation is claimed.

- [Cisco CCNA v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf)
- [RFC 1812 IPv4 router requirements](https://www.rfc-editor.org/rfc/rfc1812.html)
- [Cisco VLAN broadcast domains and routing](https://www.cisco.com/c/en/us/td/docs/routers/ios/config/17-x/lan-wan/b-lan-wan/m_lsw-conf-rout-vlan.html)

Source references informed original teaching text and model composition. No
third-party implementation or diagrams were imported.

## Monitor-dependent capture correction

The first installed392 candidate timed out while requiring1268×1308 on a display
reporting1920×1080. A monitor-aware request then showed the compositor retaining
946×929 at native display density1.15. Isolated capture fixtures now wait for
zoom/fullscreen settlement, convert between UI and native pixels, and validate
the stable actual window inside a monitor-safe narrow envelope. Requests and
actual sizes are recorded. Normal app controls and grading were untouched.

The revised development runs pass89 CCNA and110 retained mail actions, with
actual200% Light screenshots inspected at946×929. Strict click visibility,
bounded scrolling and the15second action deadline remain enforced. Two geometry
regressions cover this retained-window case and refuse full-size/oversized or
undersized results. The new packagedR2 workflow remains pending.

The original coverage command used an unsupported --strict flag. The actual
supported verifier then found UI/Markdown paths in the tests-only evidence field.
The five new coverage rows now cite tests/device_path.rs and
tests/device_path_content.rs. The verifier and inventory checks pass. A live-map
Python regression now guards this reporting path. Prior failed logs are retained.

## Installed Fedora392 verification

R2 passed exact selected-input readback, content validation, private upgrade
from390, uninstall/reinstall,89 installed CCNA and110 retained mail actions,
and separate-process NFS save/reopening. Actual200% Light screens at946×929
were inspected after packaging. The15second action deadline,20reveal attempts
and one-physical-pixel control visibility remain enforced. R1 is retained as
failed evidence. Original user progress and installation were protected.

Receipt and build identity are in CHECKPOINT.md. This is focused technical
verification. Cable-specific cold reopening, independent novice comprehension,
screen-reader/physical-device acceptance and other operating systems remain
unverified. No full inclusive Rust runtime suite was rerun in this batch.
