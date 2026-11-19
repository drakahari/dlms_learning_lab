# Leaf/spine rack commissioning

Source milestone2026-11-13; native and new packaged validation pending. Latest
verified candidate remains454labs. Do not treat source integration as release or
full CCNA objective coverage.

## Practical task and teaching

`ccna-leaf-spine-rack-expansion` is one distinct authored assignment, in
`ccna-leaf-spine-fabrics` after campus architecture. The introduction defines
rack, leaf, spine, full fan-out and a shared middle device before counting paths.
The worked example uses separate Guide equipment: three leaves/two spines.
Guided practice commissions that rack, then deliberately removes an alternative,
inspects a stated spine outage and repairs it. The independent assignment adds
a fourth rack to three existing racks/three spines, preserving nine fixed cables.
Every required old/new rack pair needs three internal spine alternatives normally
and two after each supplied single-spine loss. The assignment starts unplugged;
reference endpoints stay hidden unless the learner requests help. It is not a
renamed normal-connectivity test or a numerical variant counted as multiple labs.

Use cable-end selectors, Check answers and the stated-case/rack-pair selectors.
Observations derive from current endpoints; selected case/pair never changes
or limits grading. Failed pair reasons remain visible beside compact case totals;
passing pair details can be expanded. Learn/Challenge retain the same task, with
optional guidance. Separate practice, reset, saved drafts and stale-result
invalidation reuse the existing source-bound native workspace. Old scenario and
unit values/order, evaluator19, progress schema and28themes remain unchanged.

## Explicit fidelity boundary

The opt-in SpineLeaf model is one conventional two-tier Clos shape with already
configured routed adjacencies. Only leaf/spine links qualify. A leaf cannot be
transit between other leaves. The actual available spine neighbors shared by
both endpoints determine alternatives. Count each spine once, including when
parallel cables exist. Exclude unavailable devices/cables; reject occupied ports,
foreign equipment and malformed/beyond-bound source. A two-cable path is a
structural hop count, never measured latency or a performance guarantee. These
paths share endpoints and may share links; they are not fully disjoint end-to-end.

No packets, IP/routing protocol setup, ECMP hashing/forwarding, EVPN/VXLAN,
controller/ACI policy, LAG, throughput or convergence. Modern architectures can
use additional tiers and arrangements; this canonical exercise does not prohibit
those real designs. WAN/SOHO/on-premises/cloud remain separate open objectives.
No host process, network or filesystem route exists in this model/workspace/UI.

Existing limits stay16devices,32cables,8ports/device,96ports total,4unavailable
objects/experiment,8pairs/8cases and32KiB saved session. Optional case path minima
are valid only for this design,1–8; unreachable authored references fail validation.
Old Case JSON omits the new absent option, and old observations omit new counts.
Counts are recomputed from actual state, never accepted as imported grade authority.

## Authoring and sources

One versioned authoring/scenario pack, separate unit and presentation entry:
`scenarios/authoring/leaf-spine-v1.json`, `leaf-spine-lessons-v1.json`,
`leaf-spine-units-v1.json`, `presentation/ccna.json`. The selected v42 consolidation
preserves all81v41scenario objects exactly and appends this one task. The128pack
and2MBper-pack bounds are unchanged. All454previous scenarios/180units are checked
by direct value/order comparison; campus definition/session raw JSON round trips
without new fields. Source-bound protected links and alternate free port choices
remain enforced; optional criteria do not alter previous grading requirements.

Primary sources checked2026-11-13:

- [Current CCNA v1.1,1.2c](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf).
- [Cisco Nexus3400 fundamentals9.3(3), overview](https://www.cisco.com/c/en/us/td/docs/switches/datacenter/nexus3400s/sw/93x/fundamentals/configuration/guide/cisco-n3400s-nx-os-fundamentals-configuration-guide-933/cisco-n3400-nx-os-fundamentals-configuration-guide-9z_chapter_010.pdf).
  Historical conceptual text, not current equipment advice. Its example has
  multiple physical uplinks per pair; this model represents single adjacencies.
  The figure screenshot could not be retrieved; no image inspection/import claimed.
- [Cisco ACI design guide](https://www.cisco.com/c/en/us/td/docs/dcn/whitepapers/cisco-application-centric-infrastructure-design-guide.html),
  updated2024-06-06, Physical Topology/Leaf and Spine Functions. Multi-tier support
  is why the lesson explicitly limits the canonical case, not every modern fabric.

No vendor code, diagrams, firmware, dependencies or configurations imported.
No endorsement, full-objective, physical-input, novice, screen-reader or
Windows/macOS runtime acceptance. Native/package evidence will be added only
when actually observed; automated teaching checks are not a novice study.

## Development validation —2026-11-13

Passed43focused integration tests across six targets:15graph,11workspace,
4source-binding,4new-content/preservation,3retained campus and6retained transfer.
Independent six-link Boolean oracle checks64layouts under normal/three-spine-loss
conditions; parallel links, peer shortcuts, wrong roles, malformed/bounded data,
partial work, protected old links, equivalent ports/orientation/order, separate
practice, resume and stale success are exercised. Direct raw old campus definition
and session JSON round trips add no optional fields.

Final UI:23architecture/catalog/transfer tests passed, then12architecture tests
passed after the native-driven clarity change. They cover real widgets, chosen
pairs/cases, explicit guidance isolation, reset/resume, keyboard focus, reflow,
complete centered values and equal topology cards. The new concise-feedback
regression checks actual0/1/2spines against required2/3without parsing prose.
All1284Python tests passed atR2; final UI-only source changes require fresh focused
source-identity/content guards before candidate freeze. All-target Clippy and
new package/lifecycle remain pending at this writing. No broad Rust runtime rerun.

Actual Fedora development R2:55fixed actions/14PNGs; source-held choices delivered
to real native controls, not direct assignment-state mutation. Normal1920×1080,
Navy100; narrow946×929, Light150/Navy200. Representative raw originals inspected:

- [Introduction](../.build-artifacts/native-runs/run-s8y4ryub/screenshots/workspaces/leaf-spine-0-introduction.png)
- [Separate practice](../.build-artifacts/native-runs/run-s8y4ryub/screenshots/workspaces/leaf-spine-0-practice.png)
- [Independent assignment](../.build-artifacts/native-runs/run-s8y4ryub/screenshots/workspaces/leaf-spine-0-assignment.png)
- [Useful failed counts after a repair is broken](../.build-artifacts/native-runs/run-s8y4ryub/screenshots/workspaces/leaf-spine-0-broken-again.png)
- [Light150 centered endpoints](../.build-artifacts/native-runs/run-s8y4ryub/screenshots/workspaces/leaf-spine-0-light150-controls.png)
- [Navy200 centered endpoints](../.build-artifacts/native-runs/run-s8y4ryub/screenshots/workspaces/leaf-spine-0-dark200-controls.png)
- [Navy200 actual paths after a spine loss](../.build-artifacts/native-runs/run-s8y4ryub/screenshots/workspaces/leaf-spine-0-dark200-topology.png)

R1behavior passed, but actual images exposed repetitive full failed-path prose.
R2keeps actual-versus-required failed counts visible and full paths expandable;
grading is unchanged. R1images/before ELF and partial selected source remain
private in next-leaf-spine/native-r1; that is not a complete compiler closure.
Long pages scroll; component captures do not claim every page fits one viewport.
The pre-existing narrow200%theme-name ellipsis remains; endpoint values are full.
No physical-device, screen-reader or independent novice study was performed.

Coverage-tool correction: exact historically preserved copies are fingerprinted
and compared against selected data before being discarded, rather than charged
as new unintegrated drafts. Real drafts/unit metadata retain the32MBbyte budget;
all reads retain the2MBper-file guard. A negative accounting regression rejects
excess real drafts and changed preserved origins. No history is removed and no
selected-pack, lab, unit or draft count/byte limit is raised.

Final source gates: all-target Clippy with warnings denied, strict coverage455/
496Partial/22Supporting/722Open and inventory455(450updated/5already-meets) pass.
After final UI-only changes, fresh literal177/ERE186 closed-source guards and
leaf/spine content4pass. Eleven completed final/source/native MAIN logs retain
501samples/min13.038GiB/fullPSIavg10max0.54%/OOMdelta0. This is not a memory
reservation; near-full swap remains a risk. New package verification is next.

## Packaged Fedora milestone verified —2026-11-13

The separate455/181candidate validates bundled content and frozen identity. Private
454→455 install/upgrade/uninstall/reinstall plus outside-checkout desktop/hash and
exact synthetic original-nine progress retention pass. The actual privately
installed ELF completed55native actions/14PNGs; representative originals inspected:

- [Packaged Introduction](../.build-artifacts/native-runs/run-poauij0y/screenshots/workspaces/leaf-spine-0-introduction.png)
- [Packaged separate practice](../.build-artifacts/native-runs/run-poauij0y/screenshots/workspaces/leaf-spine-0-practice.png)
- [Packaged Assignment](../.build-artifacts/native-runs/run-poauij0y/screenshots/workspaces/leaf-spine-0-assignment.png)
- [Packaged failure after breaking success](../.build-artifacts/native-runs/run-poauij0y/screenshots/workspaces/leaf-spine-0-broken-again.png)
- [Light150endpoints](../.build-artifacts/native-runs/run-poauij0y/screenshots/workspaces/leaf-spine-0-light150-controls.png)
- [Navy200endpoints](../.build-artifacts/native-runs/run-poauij0y/screenshots/workspaces/leaf-spine-0-dark200-controls.png)

Launcher/profile/checksums and retained-input receipt are in CHECKPOINT.md and
private next-leaf-spine/candidate-r1/r1-verified.json. Freeze ENDED; never re-attest
this immutable candidate against later main. Post-build notes differ intentionally
from bundled pending notes. No new normal-process leaf/spine cold reopening;
serialization/disk resume and installed save/readback pass. Prior454NFS/editor
cold evidence is retained, not relabeled455. One packaged GUI process is finite
runtime evidence, not a startup reliability guarantee. No physical/novice/screen-
reader or Windows/macOS acceptance. Final16MAIN logs1221samples/min8.717GiB/
fullPSIavg10max0.54%/OOMdelta0; near-full swap remains a risk. No Git, release,
installed-app replacement, original progress, dependency or system changes.
