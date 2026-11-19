# Virtual workloads and routing isolation

Current source:461 authored labs/183 units; this unit adds three distinct CCNA
assignments. This is scoped current CCNA v1.1 objective1.12 practice, not full
exam preparation, actual virtualization experience or a vendor endorsement.
The announced v2.0 blueprint remains separate in the curriculum map.

Library → CCNA → **Virtual workloads and routing isolation**, following leaf/spine.
Prerequisites: VLAN data foundations, IPv4 static foundations and campus roles.
Experienced learners can enter directly; each Introduction is revisitable.

- `ccna-virtual-host-resilience`: distinguish a VM guest kernel from a container's
  shared host kernel; repair placement within supplied planning budgets and retain
  a ready replica of each service during every stated single-host outage.
- `ccna-virtual-switch-uplink`: repair a virtual adapter's group and its host's
  uplink while preserving both cross-host group paths and local/remote separation.
- `ccna-vrf-overlap-repair`: repair interface membership in separate connected-route
  tables on one physical router; identify the endpoint that actually replied when
  two tenants reuse the same IPv4 numbers.

## Teaching and workspaces

Introductions define the essential terms, explain the relevant model and show an
annotated example using separate names. Practice preserves its own configuration
and observations. The assignment starts with intentional errors and tests the
whole task; no assignment references are filled automatically. Challenge retains
the meaningful assignment and hides unsolicited teaching. Hints/worked reasoning
remain explicitly optional. New fields and cards reuse shared native controls.

Host practice uses placement selectors, visible kernel/runtime/budget facts and
an explicit host-outage experiment. The vNIC lesson reuses the existing native
VLAN table, **Apply and test example** and broadcast inspector. VRF practice shows
fixed router/endpoint addresses beside context selectors and names both requested
and actual destinations. A reply from a different same-numbered endpoint is not
intended-destination credit. Trials, unchecked work and full-task checks differ.

Changing configuration clears current check success and actual trial output.
Changing a trial selection clears only its observation, because it does not alter
the configuration being graded. Reset example affects only practice; the normal
header Reset affects only this lesson's attempt. Both drafts and selections save
for resume. Transient observation text is not invented after restoration.

## Bounded simulation and fidelity

All activity remains inside ordinary Rust data structures and existing bounded
forwarders. No VM/container, runtime, host shell, file, network or actual system
configuration is executed. Host/guest/kernel/runtime facts, physical links and
endpoint addresses are source-owned; learners edit only the disclosed draft.

Host scope:2–4 represented hosts,2–8 workloads and1–4 service groups. Listed VM
operating systems are declared compatible with supplied hypervisors. Linux
containers require the supplied Linux kernel/runtime; non-Linux container runtimes
are not modeled. Memory and CPU units are exercise planning reservations, not
universal admission/overcommit rules or measured performance. A represented host
outage makes its workloads unavailable; the model does not boot OSs, start actual
applications, migrate workloads, emulate CPU scheduling, prove security against
escapes or test application protocols. “Ready in this plan” describes this model.

vNIC scope:fixed represented placement/cabling and availability; generic virtual
Layer2 switches with the existing finite broadcast/VLAN engine. No ESXi product,
port-group manager, MAC lifecycle, STP, teaming, actual frames or performance is
emulated. The native VLAN is unused by guests. Existing trunk-profile limitations
remain; simulator reference actions are not taught as real vendor commands.

VRF scope:one physical router,2–3 tables and2–6 fixed links/endpoints. Each
interface uses one table. Connected routes derive from membership; projection
reuses the existing static IPv4 request/reply engine. Derived routing-instance
IDs map back to the physical router/context and are never selectable host
endpoints. Duplicate addresses inside one context fail with a diagnostic; overlap
across separate contexts is supported. Router unavailability affects every table.
No inter-VRF leaking, MPLS, BGP, dynamic protocols, IOS interpreter or kernel VRFs.
Addresses remain fixed on native context reassignment; some actual IOS commands
remove the interface address and require reconfiguration. Conventional interface
scope retains the existing /31 and /32 exclusion. This is not wire or equipment
acceptance.

## Content and progress authoring

Canonical standalone source: `scenarios/virtual-isolation-lessons-v1.json`, its
unit fragment and the three UI-only entries appended to presentation/ccna.json.
The selected v44 consolidation retains85 earlier scenario values then adds these
three exactly. Tests verify that relationship and all458 earlier catalog/182unit
values and relative order. Earlier110CCNA teaching values have a fixed canonical
SHA guard. Private preparation scripts are not a supported executable pack format.

`virtual_workloads` supplies admission/outage observations; `virtual_routing`
projects connected tables into existing endpoint-bound forwarding;
`virtual_isolation_workspace` owns separate typed drafts, selections and checks;
`virtual_isolation_source` binds attempts/actions to their authored source;
`ui/virtual_isolation_page` renders the native tools. The vNIC lesson reuses the
Switching state/teaching path. No dependency or shell-evaluator change is needed.

Use schema1/stable new IDs and a reachable source reference with a genuinely
incomplete initial state. Named services must cover every workload; routing cases
must cover every endpoint with positive paths and include separation cases.
Equivalent placements and swapped table labels pass when their outcomes pass.
No exact answer string or canonical configuration match is the grading criterion.

All vectors, identifiers, serialized contexts, output and forwarding paths have
finite existing/new limits. Source/action/import validation rejects foreign or
oversized data. Developer `virtual-net` JSON is inert fixture data for bounded
native-model actions; ordinary terminal execution rejects it without its authored
goal. It never invokes a host tool. Existing progress schema and evaluator19,
old IDs, favorites, drafts and28themes remain unchanged. Bare progress saving
refuses this source-bound state without its catalog, retaining the original file.

## Technical evidence and acceptance limits

Source integration34tests pass:31new capability/workspace/source/content checks
plus3preserved SMB tests. These include independent81-placement and16-membership
oracles, legitimate alternatives, partial/wrong states, exact old catalog/unit
preservation, practice isolation, disk restore and stale success. Six actual-widget,
keyboard, plan and clipped-capture checks pass. Inclusive Python1298pass in130.242s;
one-job all-target Clippy passes with warnings denied. Final focused runtime and
current-catalog readbacks pass (31new runtime,6UI,5catalog). Three old native
capture helpers now assert the expanded current461/183 catalog; original checks
and fixed task-ID restrictions remain. Their retained R1failure and new REST
opener regression precede fresh inclusivePythonR3 and all-targetClippyR2 passes.

Actual Fedora development R2passed140fixed native actions/39PNGs at1920×1080
Navy100 and946×929 Light150/Navy200. All three learning views for each new lesson
and representative large light/dark originals were inspected. Retained R1before
images exposed doubled authored section numbers, an imprecise example button and
a missing arrow glyph. New-only refinements correct these and give a relevant
unavailable-router retry. A real-widget Escape regression exposed lost dropdown
focus: the opt-in native helper now retains the trigger and Tab navigation; old
screens do not call it. Original assertions and capture guards remain intact.

Separate normal GUI cold reopening passes in two actual processes3510975/3511614,
with eight inspected captures. Assignment and practice are deliberately left
unfinished for BOTH host and routing models. The reader verifies exact raw and
portable progress/preferences/provenance, separate partial drafts, unchecked state
and no invented observations before actual native repairs/checks. Light150 is
selected through the normal toolbar. No grade or style/state injection substitutes
for these controls. This is a finite technical check, not a reliability guarantee.

Development originals (local ignored evidence, not mockups):

- [Workload Introduction](../.build-artifacts/native-runs/run-m3u67yas/screenshots/workspaces/virtual-0-introduction.png)
- [Virtual-switch guided practice](../.build-artifacts/native-runs/run-m3u67yas/screenshots/workspaces/virtual-1-practice.png)
- [Routing assignment](../.build-artifacts/native-runs/run-m3u67yas/screenshots/workspaces/virtual-2-assignment.png)
- [Restored then repaired host assignment](../.build-artifacts/tmp/native-reopen-4935d1e9e32c75cc9001d27d/read/reopen-virtual-reader-host-repaired.png)
- [Restored then repaired routing assignment](../.build-artifacts/tmp/native-reopen-4935d1e9e32c75cc9001d27d/read/reopen-virtual-reader-routing-repaired.png)

A separate461installed candidate and its upgrade/native/cold verification are
pending. Latest verified458candidate remains immutable. No new full inclusive
Rust runtime repeat, physical-input, screen-reader, independent novice or
Windows/macOS acceptance is claimed. Partial1.12 does not cover every virtualization
skill. Current certification parents:110Partial/2Supporting/24Planned; overlapping
references:501Partial/22Supporting/717Open. These are not lab-count targets.

## Primary references checked2026-11-13

- [Current CCNA v1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf)
- [Cisco VM concepts](https://www.cisco.com/site/us/en/learn/topics/computing/what-is-a-virtual-machine.html)
- [Docker container concepts](https://docs.docker.com/get-started/docker-concepts/the-basics/what-is-a-container/)
- [Cisco virtualization/networking architecture](https://www.cisco.com/c/en/us/td/docs/solutions/Enterprise/Data_Center/vmware/vmware/VMware.html)
  (historical VMware Infrastructure3 reference, terminology only, not current
  product commands or a claim of product support)
- [Cisco VLAN trunks, IOS XE17.18](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-18/configuration_guide/vlan/b_1718_vlan_9300_cg/configuring_vlan_trunks.html)
- [Cisco VRF-Lite, IOS XE17.17](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9600/software/release/17-17/configuration_guide/rtng/b_1717_rtng_9600_cg/configuring_vrf_lite.html)

Original authored text/models only; no third-party code or vendor illustrations
imported. Existing MIT approval and third-party notices remain. No installer
replacement, dependency/system changes, Git/push/merge or release publication.

## Packaged Fedora readback —2026-11-13

The separate461candidate is now VERIFIED: private458->461lifecycle/retention,
installed140native actions/39screens and two normal cold processes/eight images
pass. All nine teaching views and representative failure/Light150/Navy200
originals inspected. Exact launcher, hashes and acceptance limits are in
[CHECKPOINT.md](CHECKPOINT.md). This post-build note was added after final identity
and selected-input retention; the immutable bundled copy retains build-time
pending wording. Never re-attest the package against these later docs.

Installed originals (local ignored evidence):

- [Introduction](../.build-artifacts/native-runs/run-smve1kpg/screenshots/workspaces/virtual-0-introduction.png)
- [vNIC practice](../.build-artifacts/native-runs/run-smve1kpg/screenshots/workspaces/virtual-1-practice.png)
- [VRF assignment](../.build-artifacts/native-runs/run-smve1kpg/screenshots/workspaces/virtual-2-assignment.png)
- [VLAN fields at200%](../.build-artifacts/native-runs/run-smve1kpg/screenshots/workspaces/virtual-1-navy200-allowed.png)
- [Specific broken-state feedback](../.build-artifacts/native-runs/run-smve1kpg/screenshots/workspaces/virtual-2-broken-again.png)
- [Cold-restored and repaired routing assignment](../.build-artifacts/tmp/native-reopen-d6d5159626ea3f9c6e9e9470/read/reopen-virtual-reader-routing-repaired.png)

No actual virtualization, wire behavior, complete objective, physical-input,
screen-reader, independent novice or other-platform acceptance follows.
