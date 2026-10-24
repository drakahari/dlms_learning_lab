# Ethernet learning unit — authored tasks and native verification

Four original assignments are registered in unit `ethernet-learning`, following
VLAN foundations before static routing. The final installed-candidate all-four
run is a separate pending delivery gate.
The accepted [model and native workspace](MAC_LEARNING_SUBSET.md) precede this
content. Current CCNA v1.1 section1.13 is the intended narrow reference;
future v2.0 coverage remains separate.

| Assignment | Practical outcome | New teaching |
| --- | --- | --- |
| Build directed paths from both sources | Learn each peer at its actual port; directed traffic excludes the third workstation while broadcast still reaches it | Frames, source/destination, MAC notation, source learning, known/unknown forwarding and copies versus acceptance |
| Repair an older location without losing another group | Relearn or selectively clear a wrong location while preserving another VLAN's known paths | Reading an older table, source refresh and scoped recovery |
| Refresh the quiet source and prove expiry | Choose a lifetime inside a range, refresh the necessary source and satisfy both survival and expiry checks | Explicit virtual time, source-only age and independent future checks |
| Hand over two learned groups across three switches | Repair active/access/trunk settings and rebuild both groups' distributed tables, preserving current and quiet-interval outcomes | Integrating prior VLAN and learning skills in a substantial handover |

Each introduction precedes assessment and can be skipped or reopened. Separate
examples use different workstations, addresses and VLANs; each has a
specific experiment and repair. Insert and Fill populate inputs only. Applying
settings or sending a frame is explicit and uses the same bounded model as the
assignment. Practice never changes assignment state or earns assignment credit.
Challenge retains the full assignment without unsolicited worked solutions.

The workspace emphasizes the assignment, topology, current table and source/
destination controls. Port configuration is prominent for the capstone's actual
repairs and optional when the task only preserves initial settings. The aging
task exposes its relevant lifetime setting. Time, selective clear, traces,
references and walkthrough help are available on demand. Unapplied settings and
malformed raw drafts remain visible on resume and cannot retain current credit.

Grading checks current configuration and table state, plus independent frame/
time checks on temporary copies. It does not reward a remembered trace or exact
sequence. Two alternative plans per assignment, plausible mistakes, partial
results and breaking each requirement are included in the content review.
Nine authored content checks pass, with reference and alternate outcomes,
plausible wrong/partial work, current invalidation and original109 progress
preservation. The normal combined model gate and native component evidence are
recorded below; pending delivery checks are not reported as passed.

Limits remain explicit: finite fixed equipment and source identities, an
acyclic physical fabric, manually advanced virtual time, educational aging
range, and no STP, ARP, real frame bytes, IOS terminal or hardware equivalence.
Copies and modeled endpoint acceptance are different observations. Neither a
learned entry nor these finite test flows establishes a security guarantee.

Sources checked2026-10-24: [current CCNA v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),
[Cisco MAC management](https://www.cisco.com/c/en/us/td/docs/switches/lan/c9000/lyr2-fwd/cdp-lldp-mac-udld/cdp-lldp-mac-udld-configuration-guide/c-configure-mac.html)
and [unicast flooding background](https://www.cisco.com/c/en/us/support/docs/switches/catalyst-6000-series-switches/23563-143.html).
Teaching and implementation are original; no vendor manual paragraphs or
implementation code are imported. See the subset document for precise limits.


## Verified scope and remaining delivery gate

The registered113/27 branch passes857 model/integration checks. The later main
combined gate with PTR passes869 checks; subsequent whole-main regression is
tracked in CHECKPOINT.md. Final focused UI gates pass30 distinct checks, including
real popup selection, actual capture-phase execution, shared label columns at
100/150/200, focus/Undo and affected IPv6/VLAN controls. Clippy/fmt/build pass.

Actual Fedora native first-three full flows pass in run-v518tg9y. Their capstone
continuation found a clipped-popup controller issue, retained as a failed full
run. The corrected exact capstone-plan suffix passes15 captures in run-f5mi3zoe,
including separate practice/experiment/repair, malformed/conceptual/partial work,
reference and alternate state,37 current objectives, reset/readback, Challenge
and200% editing. Another24 render-only views pass in run-_5ducto6: all four
assignments100% fullscreen/windowed and capstone150%Light/200%Dark in both.
Root reviewed real2560×1440 and1268×632 renders, centered values/cursors, equal
topology cards and reserved versus stacked port columns. Scrolling remains
necessary in short tiled windows and larger text; fonts are not reduced.

No single all-four successful run or final installed-candidate run is fabricated
from those components. The exact next candidate must run the full four-task
flow. Physical-device keyboard, screen-reader, novice and Windows/macOS desktop
acceptance remain pending. Earlier failed captures/source/ELFs are retained.

Actual review exposed a missing arrow glyph in the capstone prose; English
"archive to printer" replaces it without changing any state, goal or action.
The native popup helper now scrolls within actual content intersected with the
paint clip, confirms the selected value and rejects offscreen clicks. A separate
capture-only Context lock bug is corrected and covered by phase execution;
neither issue changes model grading or diagnoses the historical Wayland stall.
