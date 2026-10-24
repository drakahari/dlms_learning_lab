# P2P OSPFv2 practice

Checked2026-10-23. Four original versioned native configuration assignments now
belong to the normal `ospf-p2p-foundations` unit. It follows
`ipv4-static-foundations`; the first task names the existing
`ccna-ipv4-branch-handover` scenario, followed by a three-task prerequisite chain.
The workbench models configured compatibility and directed SPF candidates,
not observed live adjacencies or the full current CCNA3.4 objective.

## Instruction and practice

Each activity begins with skippable concepts, then a separate worked network,
then the complete assignment. Every typed example action has an explanation of
the visible process/interface fields it changes. Fill changes only that
practice draft; apply/check is explicit. One concrete try-change has a declared
vector of failed/passed current rows and an explicit repair. Practice has
different router names, links, addresses and router IDs from its assignment.

Learner prose uses native fields and current result rows. The reference strings
in `solution` are only the existing pack's typed JSON reachability adapter;
they are not shown as IOS or Linux commands. No command-to-success lookup,
background protocol exchange, host process or real network is introduced.

## Four different assignments and preservation boundaries

| Activity | Useful work | Explicit preservation and allowed alternatives |
| --- | --- | --- |
| Router identity | Recover a 3-router records path with a duplicate Branch/Hub ID and a disabled Transit process. | All physical fields and all OSPF port configurations stay supplied. Transit retains its number, ID and area while enabled; Hub's whole process stays unchanged. Any unique nonzero Branch ID is accepted; its process number need not match peers. |
| P2P parameters | Put Office into requested area 0 and repair both interval comparisons while retaining the healthy Relay–Backup attachment. | All physical fields, process numbers/IDs/enabled state, both LAN ports and both healthy transport ports are fixed. Either endpoint may be changed to match Hello/Dead pairs; 5/20 and 10/40 are both valid. |
| Directed ECMP | Restore the complete East+West first-hop set for Storage's records prefix while retaining a separately computed reverse candidate. | Only Ingress east/west outgoing cost values may vary: compatible peer guards pin the other relevant source port fields against unchanged peers. All other ports/processes and physical fields remain fixed. The forward equality is required, not one forward metric. |
| Passive handover | Repair a disabled remote process and one transport mismatch, publish the user LAN passively to both ends, and retain both cross-site LAN candidates. | All physical fields; all process fields except Archive enable; both LAN ports and both healthy Hub–Archive ports are fixed. Hub/users explicitly needs enabled+passive with its supplied Broadcast/cost3/Hello10/Dead40 parameters. Either Site–Hub endpoint can be repaired; its positive costs may differ when all requested behavior remains. |

No assignment uses `WholeState` as a hidden exact answer. Each visible
preservation sentence has an actual Physical, Process or Port predicate. The
capstone's user-port exact guard is deliberate: the task explicitly asks to
retain its remaining supplied parameters. Presence of a route or absence of a
peer alone would be too weak: a singleton active P2P advertisement can contribute
a prefix in this model. Both that shortcut and disabling/shutting down the LAN
are negative cases.

## Independently stated arithmetic

The expected values below were worked out from the supplied fields; they were
not harvested from a reference execution.

- Identity records: Branch output6 + Transit output4 + Hub LAN2 =12.
- Parameter handover: Office output7 + Relay output3 + Backup LAN5 =15.
- ECMP reference: East9+6+4=19 and West10+5+4=19. Alternatives14/15 give24
  and7/8 give17. Reverse Storage viaEast3+7+2=12 versus viaWest8+11+2=21.
- Capstone: Site→users8+3=11; Archive→users9+3=12;
  Site→Archive records8+5+4=17; Archive→Site office9+12+2=23.

The worked ECMP network has distinct values: forward4+3+2=9 and3+4+2=9,
with reverse7+5+1=13 instead of11+8+1=20. Its try-change makes one forward
path10, leaving only one shortest hop. The forward route check must fail while
all preserved reverse/configuration checks continue to pass.

## Behavior and native verification

Eight content tests execute the four references and eight independently authored
alternates, reject21 plausible wrong/partial approaches, check independent route
cost/complete-hop equations, invalidate pending/malformed/applied breakage, and
verify isolated fill/experiment/repair, save/resume/backup/restore/selected reset.
No exact command-string solution is graded. The separate adapter regression
pins all original98 IDs, real current37 script runs and genuine frozen6 histories
(33records/182outcomes); synthetic fixtures never represent learner mastery.

The first normal all-four Fedora native flow passed84actual captures using
router/interface menus, text fields, toggles, Check and Reset/Confirm Reset.
It exercises the introductions, separate examples, experiments, meaningful full
tasks, wrong/partial/alternate outcomes, fresh invalidation, malformed atomicity,
exact both-draft resume and Challenge Light150/Dark200. Evidence:
`.build-artifacts/logs/m67-ospf-authored-native-r1.log` and
`native-runs/run-12xazcls/screenshots/workspaces`. Root inspected real rendered
screens. Final new-attempt focus and link-cost evidence polish passes the normal84-capture
run m67-ospf-authored-native-r2.log/run-35oa1lvn. Actual review found missing
arrow glyphs; plain ASCII/word labels then pass12final100/150/200 captures in
m67-plain-cost-labels-native-r1.log/run-nd6f773j, with actual root visual review.
Combined811Rust checks,53Python checks and final all-target Clippy pass.
These injected authored inputs are technical acceptance, not a novice study,
physical keyboard/screen-reader test, real network or other-platform acceptance.

Catalog102/24units:31CCNA/21LPIC-1/13LPIC-2/37Bash. Only current3.4/3.4.a/3.4.b/
3.4.d configured-model slices gain Partial practice. Broadcast DR/BDR stays Open;
futurev2.0 is separate. The1240 overlapping references are not a completion
percentage. Most required capabilities and full-product acceptance remain Open.

## Primary references and scope

- [Cisco CCNA200-301 v1.1 (2024),3.4](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf), rechecked2026-10-23. The issuer also expects live neighbor and broadcast DR/BDR work absent here. No full3.4 coverage is proposed.
- [RFC2328 (April1998), §§5/8.2/10.5/12.4.1.1/16.1/16.1.1/16.8](https://www.rfc-editor.org/rfc/rfc2328.txt), rechecked2026-10-23. It informs identity, configured P2P comparisons, directed SPF/stub costs and equal-cost choices. The actual protocol state machines and other listed updated-RFC features are outside this core.
- [Cisco IOS XE17.x Default Passive Interfaces](https://www.cisco.com/c/en/us/td/docs/routers/ios/config/17-x/ip-routing/b-ip-routing/m_iri-default-passive-interface.html), updated2022-11-02, rechecked2026-10-23. Basis for distinguishing passive network advertisement from disabling the interface's OSPF contribution.
- [Cisco document13689, interface field interpretation](https://www.cisco.com/c/en/us/support/docs/ip/open-shortest-path-first-ospf/13689-17.html), updated2005-08-10, version-independent, rechecked2026-10-23. Local process labels are separate from router identity, type, cost and intervals. No IOS command grammar is taught by this proposal.

The RFC HTML endpoint failed to fetch in this review; its official text endpoint
and the IETF datatracker copy were available. A guessed Cisco13689 URL ending
`-1.html` returned404; the actual `-17.html` document above was retrieved and
its title/date verified. No source code or manual passages were copied into the
implementation or curriculum.
