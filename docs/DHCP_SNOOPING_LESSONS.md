# DHCP inspection: server boundaries and sender consistency

Implemented review catalog351/124 (79CCNA,117LPIC-1,78LPIC-2,77Bash). Two lessons in
**CCNA → DHCP snooping foundations**, after Secure access ports and the earlier
DHCP client unit. Previous349 scenarios/123units and77CCNA presentation entries
are preserved exactly. This is partial CCNA5.7 practice, not full-objective or
actual-device acceptance. DAI remains a separate pending capability.

| Stable ID | Meaningful assignment | Distinct investigation |
| --- | --- | --- |
| `ccna-dhcp-server-boundary` | Keep approved replies working while rejecting server replies from both user ports. | Global/VLAN activation, ingress trust, actual directed delivery, Offer versus ACK and record expiry. |
| `ccna-dhcp-client-mac-check` | Reject inconsistent client identifiers without breaking valid exchanges or server protection. | Read Ethernet source and DHCP client MAC side by side; test mismatched and matching copied fields. Consistency is not authentication. |

## Teaching and native workspace

Both introductions review the necessary protocol concepts and provide annotated
worked settings. Separate guided practice uses different equipment/addresses;
its optional worked policy never populates the assignment or earns credit.
Assignments retain topology, aligned policy fields, incoming-server trust rows,
supplied traffic replay, actual recipients and observed binding records. Optional
message detail and help do not hide the second lesson's essential two MAC fields.
Challenge retains the assignment without unsolicited worked solutions.

Apply, Replay and Check work have different effects. Apply changes the policy;
Replay starts the selected supplied conversation with its own initial state while
preserving the draft. Check evaluates each requirement in separate deterministic
cases and leaves displayed observations untouched. Changed inputs clear current
success. Failed cases identify the first mismatch; additional observations remain
available on demand. This does not remove any grading condition or failure.

A recorded binding is a switch observation, not allocation or proof of client
application behavior. Offer alone records nothing. A delivered trusted ACK after
a Request records the supplied IP/MAC/VLAN/client port/expiry. Expiry and matching
Release remove it. The supplied direct-LAN setup explicitly disables Option82
insertion; no device-default claim is made. Unsupported exchanges fail honestly.
See [model contract](DHCP_SNOOPING_SUBSET.md) and
[forwarding composition](ACCESS_OUTPUTS_SUBSET.md) for exact limits/references.

## Preservation and grading

`dhcp_snooping_workspace` validates typed task data and bounded source-compatible
sessions. The fresh supplied state is used for each traffic case under the current
applied policy. Grade actual admission, recipients, rejection cause and every
binding field, not policy strings or the current replay history. Equivalent port
ordering and additional irrelevant inspected VLANs are accepted when all outcomes
match. Wrong/partial configurations, pre-existing observations and forged checked
flags cannot substitute for successful cases. Malformed unfinished drafts resume.

Focused integration tests cover separate practice, actual solutions/alternatives,
negative states, selected reset, private save/load/export/validated restore,
favorites and old drafts. No original owner progress is used. Schema/version1 is
additive; prior scenario IDs, values and completion rules stay intact.

## Verification evidence

- R53:42focused model/session/source/content/retained-port checks passed.
- R60:14new widget/capture/preview checks passed; R64 caught later output-layout
  clipping while actual blank editing/retry and12other checks passed.
- R71/R72 diagnosed actual form-width expansion. The cause was an unwrapped trust
  checkbox inside a horizontal value column. R75/R76 corrected only that label's
  wrapping and passed all five supported scales, narrow/wide, light/dark and
  populated/disabled bounds without changing assertions or font size.
- R77's development launch omitted the review-catalog flag and correctly refused
  its missing lesson IDs. R78 used the correct catalog, exited0/reaped, and completed
  88fixed native actions plus40original PNGs in `native-runs/run-sj__zj9h`.
  Root opened both introductions, first practice/feedback, second actual MAC
  diagnosis and light200narrow. A crowded worked value and dense failure text were
  found and refined before package admission. This is assisted technical review.

Final source gates: R84all43focused tests, R85all14widget/capture/preview tests
(106.90s), R92library/R93app/R94all-targetClippy(43.97s), R96all956Python
(12.710s) pass. The inventory is351entries:346updated and5already meets. Its
strict original-requirement guard caught paraphrased new bullets; presentation
bullets now retain the original assignment sentences exactly. No grading or
requirement guard was weakened. R97optimized build and R98initial/R101final selected identities pass:793source/data,
195resources,329documents,4presentation files and83supplemental inputs. R99actual
package validates351/124. R100exact-package native exits0/reaped,88actions/40PNG;
root opened9originals from run-pnew9gv6, including both lessons all three views,
wrong/passed feedback and light200narrow/dark150wide. Final R102receipt precedes
freeze lift. This is not an installed/cold or independent novice acceptance.

Representative actual packaged screenshots (private local evidence):
- [Introduction](../.build-artifacts/native-runs/run-pnew9gv6/screenshots/workspaces/ccna-dhcp-server-boundary-introduction.png)
- [Guided practice](../.build-artifacts/native-runs/run-pnew9gv6/screenshots/workspaces/teaching-ccna-dhcp-client-mac-check-example-dark-100.png)
- [Assignment and actual observations](../.build-artifacts/native-runs/run-pnew9gv6/screenshots/workspaces/ccna-dhcp-server-boundary-assignment-passed.png)
- [Light200narrow](../.build-artifacts/native-runs/run-pnew9gv6/screenshots/workspaces/dhcp-snooping-light200-narrow.png)

The isolated candidate is teaching-review-20261109T042029Z under dist; launch its
absolute dlms-learning-lab path without flags. Original user progress is protected.
A development flow is not installed/cold-reopen or final-package acceptance. No
full-inclusive Rust, hardware, all28theme, Windows/macOS/Omarchy runtime,
screen-reader/physical-device or independent novice study acceptance is claimed.
No host learner execution, dependency, install replacement or public publication.
