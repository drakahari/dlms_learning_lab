# JSON network inventories

Development continuation, 2026-11-11. Two new source lessons extend the preserved
392-lab catalog to394 across151 units. Fedora392 R2 remains the verified package
until a new candidate is tested. No release or full-objective claim.

Library → CCNA → JSON network inventories follows VLAN data-plane foundations.

- `ccna-json-port-records`: repair JSON types and values, then restore two useful
  endpoint groups with broadcast separation. Three actual network cases.
- `ccna-json-inventory-handover`: recover nested records on two switches over a
  supplied trunk, retaining staff, guest and monitoring paths and the monitoring
  VLAN. Eight actual connectivity, separation and VLAN cases.

Introduction teaches objects, field/value pairs, quoted strings, arrays, nested
ownership, numbers, booleans and null with a separate worked practice document.
Guided practice repairs a field type, repairs group behavior, then disables and
restores a port. The independent assignment retains intentional errors. Different
supported VLAN allocations and equivalent JSON record/key ordering are accepted.
Worked fills affect practice only. Editing clears the current result; applying
is atomic; Check network requires the current document to be applied.

## Bounded schema and isolation

This is a lab-defined inventory schema, not a Cisco configuration format or a
complete JSON Schema engine. Existing serde_json parses real JSON. Strings,
booleans and numeric VLAN IDs are typed. VLAN IDs require plain integer notation
from1 to4094 and membership in the provided database. Other valid JSON numeric
forms receive a field diagnostic; they are not rounded or silently approximated.
Optional note accepts null or printable text up to160 bytes. Maximum8KiB document,
two switches, six editable access ports per switch and eight endpoints. Unknown
fields, duplicate/missing records and equipment outside the supplied map fail.

Only access-port up state and VLAN membership change in the existing in-process
Ethernet simulator. Cables, equipment, VLAN databases and trunk links stay supplied.
No host files, learner shell, network access, controller or device automation runs.
Disabled paths cannot satisfy isolation. Behavioral feedback comes from actual
simulated forwarding and port state, not document matching. No STP/MAC aging,
REST interface, SDN controller or Ansible/Terraform capability is added.

## Sources and scope

Checked2026-11-11 during authoring, original text/code with no copied third-party
implementation:

- [CCNA200-301v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),6.7 recognizing JSON-encoded data.
- [RFC8259 JSON](https://www.rfc-editor.org/rfc/rfc8259.html), objects, arrays and value types.
- [Cisco VLAN routing behavior](https://www.cisco.com/c/en/us/td/docs/routers/ios/config/17-x/lan-wan/b-lan-wan/m_lsw-conf-rout-vlan.html).

Only the implemented JSON inventory slice may receive partial coverage. No full
certification objective, automation domain or independent novice acceptance follows.

## Development evidence

Final33 focused Rust integration tests include eight JSON model and six authored
content/progress tests, plus retained boot/menu/IPv6 and normal catalog checks.
Six JSON UI/capture checks and two normal-catalog UI checks pass, alongside
all-target Clippy, all1,063 Python checks, supported coverage and inventory checks.
The inventory has389 updated and five already-meets entries. No full inclusive
Rust runtime suite was repeated. Earlier failures are retained, not counted as passes.

The final actual Fedora flow completed85 actions and21 captures. It exercised
both lessons, separate practice, field/syntax/behavior errors, alternate repairs,
keyboard editing/Tab/Enter, current invalidation, save/readback,100/150%dark and
narrow200%Light. An initially growing editor was corrected with existing bounded
scrolling components. The overall result shows failing cases first; complete
per-case results stay available in optional detail. No grading behavior changed.

Actual development screens, not mockups:

- [Typed records introduction](../.build-artifacts/native-runs/run-qvem5pzi/screenshots/workspaces/json-records-introduction.png)
- [Guided JSON practice](../.build-artifacts/native-runs/run-qvem5pzi/screenshots/workspaces/json-records-practice-initial.png)
- [Independent nested assignment](../.build-artifacts/native-runs/run-qvem5pzi/screenshots/workspaces/json-nested-assignment.png)
- [Specific failed-case feedback](../.build-artifacts/native-runs/run-qvem5pzi/screenshots/workspaces/json-nested-monitor-feedback.png)
- [Bounded editor at200%Light](../.build-artifacts/native-runs/run-qvem5pzi/screenshots/workspaces/json-records-light200-editor.png)

A new separate Fedora394 package still requires installed/lifecycle verification.
Original progress remains protected. JSON-specific separate-process reopening,
novice, screen-reader and physical-device acceptance are not established. Other
operating systems are deferred. Historical source/oracle observations are unchanged.

## Packaged R1 and capture follow-up

R1passed catalog, private392upgrade/reinstall, installed85JSON and89retained
wired-device actions, NFS two-process reopening and exact selected-input readback.
The installed200%editor capture showed only part of its viewport after scrolling;
preserve that evidence and do not claim a complete-pane image. R2is being built
with capture-only late visibility checks and parent-wheel routing through the
actual outer scrollbar gutter. Six UI/capture checks, all-target Clippy and1,063
Python checks pass. Corrected development85-action/21-capture screens above show
both entire editor viewports at200%Light. No grading or accepted workflow changed;
all original timing/visibility limits remain. R2package validation is pending.

## Verified packaged R2

R2passed installed85JSON and89retained device actions, private R1upgrade/reinstall,
NFS separate-process reopening and exact selected-input readback. Actual complete
200%Light editor viewports and clear failed cases were inspected. The immutable
receipt is candidate-r2/r2-verified.json under next-json-inventory. Installed
screens: `.build-artifacts/native-runs/run-n0e3x3u4/screenshots/workspaces`.
Launch: `/path/to/dlms_learning_lab/dist/teaching-review-20261111T060612Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab`. Original app/progress remain protected.
No full curriculum/platform/accessibility/novice acceptance follows.
