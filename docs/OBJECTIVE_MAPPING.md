# Objective evidence and gap ledger

Checked2026-10-22 against the current official [Cisco CCNA v1.1 PDF](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf), [LPIC-1 v5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/) and [LPIC-2 v4.5 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/). Current/upcoming version dates remain in [CURRICULUM.md](CURRICULUM.md). Upcoming Cisco v2.0 stays in its separate planning rows; it receives no inherited current-version credit.

[SKILL_COVERAGE.csv](SKILL_COVERAGE.csv) indexes **1240 current references**:136 parent objectives,58 Cisco lettered subobjectives,375 LPI knowledge-area bullets and671 named-tool/term bullets. These references overlap in scope and are **not a completion percentage, curriculum count or additional labs**. There are144 narrowly evidenced Partial practice rows, one Supporting explanation row and1095 Open rows; no full-objective completion is asserted. The scoped verified native Fedora package ddb886b contains140 authored assignments/35 units. Three DHCP client/relay tasks add only the two explicitly scoped parent slices below; no child/tool or future-version credit is inherited.

Cisco references retain official IDs. LPI references `.K1`, `.K2`… mean the one-based knowledge-area bullet under that parent; `.T1`, `.T2`… mean the one-based named files/tools/terms bullet. These suffixes are local bookkeeping, not issuer identifiers. Each row links directly to its official parent section and records the source position. A bullet containing several tools remains one source item; partial evidence never implies all of them work.

Rows record exact authored IDs, a short verified scope, relevant test files, the remaining capability/fidelity gap and the proposed parent assignment. Evidence is **explicit for each row**: it is never propagated from parent to child, a tool name to its complete utility, or an implemented command to all related objectives. Open means required teaching/model/content is absent or not accepted. Partial practice means only the named behavior has reviewed instruction/practice or accepted alternatives. The existence of test files validates reference integrity, not the truth of a pedagogical claim; independent content review remains necessary.

The finer review corrected an earlier misleading association: `lpic2-storage` demonstrates capacity and one saved/applied mount, not ext-family/Btrfs/XFS/ZFS/SMART maintenance. LPIC-2 203.2 is now Planned/Open rather than Partial. Narrow actual IPv6 route components/selection, reusable functions, service-journal filtering and zone reload are recorded without promoting their wider objectives.

## Source change detection

[OBJECTIVE_SOURCES.json](OBJECTIVE_SOURCES.json) contains source URLs, versions/check dates, input hashes, reference/position/anchor metadata and normalized SHA-256 fingerprints. Official prose is not copied into this repository. LPI parent fingerprints include the title, full objective description and weight; each description also has its own fingerprint. Cisco wrapped lines are joined before hashing. The HTML input hash identifies the fetched copy; the Cisco input hash identifies extracted PDF text, not the binary PDF.

Every coverage row pins its source fingerprint. A changed description, weight, bullet or ordering requires review before evidence can be carried forward. Regenerating the inventory alone cannot silently validate old claims. Cosmetic surrounding HTML changes are ignored by the semantic comparison; an entirely new exam version requires a separate inventory rather than overwriting current references.

Development-only reproduction, after fetching the public documents to temporary files and extracting Cisco PDF text with `pdftotext -layout`:

```sh
python3 tools/objective_inventory.py --ccna-text .build-artifacts/sources/objectives-ccna.txt --lpic1-html .build-artifacts/sources/objectives-lpic1.html --lpic2-html .build-artifacts/sources/objectives-lpic2.html --checked 2026-10-22 --check docs/OBJECTIVE_SOURCES.json
python3 tools/dev.py tool verify_coverage.py
python3 tools/dev.py python-tests
```

The inventory tool reads supplied local source copies only. It performs no network requests or subprocess execution. Offline validation checks exhaustive current references, source fingerprints, parent/kind/position/URL, explicit gaps, existing authored IDs and contained test-evidence paths. It rejects missing/duplicate references, inherited evidence on Open rows and unsupported completion labels. CI checks the committed ledger without downloading objectives. Learners need neither these tools nor internet access.

## Authoring rule

Before adding content, identify the precise source items and implement the required model first. Introduce prerequisites, provide isolated runnable practice, retain a meaningful independent assignment, and test legitimate alternatives, plausible wrong/partial work and success invalidation. Record exact limits and reviewed evidence before updating a row. If a requirement needs unimplemented behavior, leave it Open even when neighboring tasks pass.

The completion checklist remains open: a complete index exposes missing work; it does not resolve those requirements or convert a simulation limitation into full certification preparation.

The ordinary-user access unit adds explicit narrow chgrp evidence for104.5.T4 and strengthens existing regular-file/directory/membership rows. It does not cover special-file/SUID/SGID/sticky-bit/privileged ownership behavior or the full utility syntax.


M25 adds six narrowly partial evidence rows: CCNA v1.1 2.1/2.1.a (ordinary data access only) and2.2/2.2.a/b/c (explicit trunks, restricted802.1Q/native data classification). Default VLAN2.1.b, inter-VLAN2.1.c, switching/MAC learning1.13 and STP2.5 stay Open. A physical forest is a modeling boundary, not STP implementation; data-plane native translation does not predict Catalyst PVST/PVID protection. See VLAN_LESSONS.md and SWITCHING_SUBSET.md. Future v2.0 rows remain separately planned, not silently promoted from current practice.


M28 adds narrowly partial103.3.K4/T19 evidence for script-VM simple * selection, quoting/hidden/no-match handling and safe changing-file reports. Advanced/bracket/directory patterns and cp/mv/rm terminal expansion remain absent. A tested ? capability alone is not claimed as authored instruction. All full-objective and future-blueprint requirements stay open.

M40 strengthens the existing207.1 reload and207.2 forward-record Partial evidence and adds one narrowly Partial207.2.K4 row for direct forward host addition. Six distinct DNS assignments now exercise real bounded record grammar, A/AAAA, alias paths, MX targets, preserved RRsets and separate loaded copies. Native controls are not claimed as dig/host/nslookup command syntax, BIND configuration or transfer protocol. Reverse/PTR, full grammar/locations, root hints, delegation/glue, recursion/cache and security stay Open. LPIC-2 issuer version4.5 was rechecked2026-10-23; the complete normalized source inventory retains its full2026-10-22 check date. See DNS_LESSONS.md for exact scope and evidence.

M44 adds a separate Bash-specific indexed-list unit; no issuer row is promoted for arrays.97Partial/1143Open remain. The planning crosswalk still distinguishes native practical slices from missing Linux/network protocol behavior, and current source fingerprints/check dates are unchanged.

M47 adds four distinct LPIC-1 process assignments and explicit narrow103.5/103.6 evidence: snapshots/selection, supported positive-PID signals, pending termination, parent/child handling and existing-process niceness. Eleven rows gain Partial practice;103.6.K1 receives Supporting explanation only for default/inherited niceness, without spawning. No jobs, sorting, resource monitor, new-process nice or full scheduler credit. Authored pgrep evidence is only ordinary exact-name/owner lookup. The LPIC-1 official page/version was rechecked2026-10-23; complete inventory fingerprints retain2026-10-22. See PROCESS_LESSONS.md.

M50 adds four distinct LLDP assignments and only narrow current CCNA v1.1 2.3 Partial evidence: native direction/configuration, identity-versus-description, TTL/cache refresh, transmit-only policy and preserved handover. CDP, IOS, wider protocol behavior and physical acceptance remain absent; future v2.0 is not promoted. Official v1.1 PDF and Cisco exam overview rechecked2026-10-23; complete inventory fingerprints retain2026-10-22. Catalog82/nineteen units;109Partial/1Supporting/1130Open, not completion percentages. See DISCOVERY_LESSONS.md.

M55 adds four persistent-mount assignments with native file editing and simulated investigation. Current LPIC-2 v4.5 203.1.K3 (UUID identity) and203.1.T5 (bounded blkid inspection) gain narrow Partial practice; existing203.1/fstab/mount evidence is strengthened. Saved/live policies, skipped manual identities and deterministic alias changes are actually graded. Swap, systemd mount units, full utility syntax, real boot and filesystem maintenance remain Open;203.2 receives no inherited credit. The official LPIC-2 page/version was rechecked2026-10-23; the complete inventory retains its2026-10-22 fingerprints. See STORAGE_LESSONS.md.


M57 adds four meaningful scripting assignments after accepted evaluator7. Only LPIC-1 v5.0 105.2.K2 gains narrow Partial practice: quoted original-source dollar-parenthesis capture, trailing-LF behavior, inherited helper status and child-variable/shared-file differences within the documented subset.105.2 parent andK3 strengthen with fresh behavior evidence; mail, shebang execution, script ownership and unsupported language remain absent. The current official LPI105.2 page/version was rechecked2026-10-23; full inventory fingerprints retain2026-10-22. Evidence is SHELL_CAPTURE_LESSONS.md and the actual content/UI/native flows. Catalog90/21units and112Partial/1Supporting/1127Open do not imply full-objective completion.

M62 adds four distinct native IPv4 access-policy assignments and only narrow current CCNA5.6 Partial evidence: host/any source matching, ordered service/protocol/port conditions, ingress/egress diagnosis and two cooperating boundaries with required allowed-flow guards.31 assignment and22 separate example packets are actual modeled outcomes, not renamed labs. Arbitrary wildcard lessons, IPv6 ACLs, flags/fragments/NAT/logging/counters/full IOS and physical conformance remain unclaimed. Native Fedora task/partial/alternate/mode/reset/resume and light/dark large-text views were reviewed. Full source inventory fingerprints retain2026-10-22; this batch current Cisco technical/objective review is2026-10-23. See ACL_LESSONS.md.


M65 adds four local-account lifecycle assignments after access foundations. Rechecked current LPIC-1 v5.0/107.1 on2026-10-23; only eleven previously Open local-record/tool slices gain narrow Partial evidence, while the parent/membership slices strengthen.124Partial/1Supporting/1115Open overlapping references. The shadow view is a fixed synthetic projection and expiry is stored metadata, not authentication/password practice. chage, passwd, special-purpose account policy and complete utility/NSS behavior stay Open. See ACCOUNT_LESSONS.md for commands, tests, native evidence and limitations.


M70 strengthens only the already Partial LPIC-2v4.5 202.1/K1/T4 rows with four actual service-definition, preparation/ordering, optional-failure and target-handover assignments. Current and separate next-session outcomes, preserved context and alternate solutions are tested. No new row is promoted; SysV/LSB/full boot and unimplemented exact terms remain Open. The original official fingerprints are unchanged; scoped sources checked2026-10-23. See UNIT_MANAGER_LESSONS.md.


At M76, the tested MAC-learning library foundation alone left current CCNA1.13
and its four lettered entries Open because native instruction and practice were
not yet accepted. Those gap descriptions distinguished the implemented source/
VLAN/table/aging model from the then-absent learner workspace and curriculum.
No status, authored-ID, objective-source fingerprint or count changes. Primary
Cisco v1.1/MAC references rechecked2026-10-24; full inventory remains2026-10-22.
See MAC_LEARNING_SUBSET.md.


M77 strengthens only the already Partial LPIC-1 105.2/K3 evidence with three distinct case-dispatch assignments. State-based tests cover31 deterministic cases, alternate supported scripts, partial/wrong/hardcoded attempts, independent examples and exact original106 progress preservation. Actual native Fedora flows cover all three tasks, Challenge and current-result invalidation. No objective/source fingerprint is promoted:128Partial/1Supporting/1111Open remain overlapping references, not percentages. See SHELL_CASE_LESSONS.md for limits and actual verification.


At M78, the verified native MAC workspace and isolated practice foundation used
a private unbundled developer fixture. CCNA1.13/a–d remained Open pending the
authored unit. Only the gap descriptions change; no status/fingerprint/ID
promotion or lab count. Actual native review caught and corrected inherited
Apply/Check wording before content authoring. See MAC_LEARNING_SUBSET.md.


The accepted authored Ethernet unit strengthens current CCNA v1.1 1.13 and its
four lettered rows to narrowly Partial practice, with explicit per-row source/
aging, frame, flooding and table evidence. Four distinct full tasks have separate
practice, alternate/negative/current-state tests and native component acceptance.
Actual fullscreen/windowed and large-text renders are reviewed; installed
Fedora73777e0 completes the full four-task native run with57 captures. No future-v2.0,
STP/ARP/real packet/IOS/equipment or security guarantee is inherited. Source
fingerprints remain unchanged; scoped sources checked2026-10-24. See MAC_LESSONS.md.
The overlapping totals133Partial/1Supporting/1106Open are bookkeeping, not a
completion percentage. Find/PTR library foundations gain no authored credit yet.


M92 verifies the installed116 Find unit and promotes only current LPIC-1 v5.0
103.3.K5 to narrow Partial practice, strengthening existing103.3/K1/T2 evidence.
The three actual tasks use bounded name/type/depth searches, incomplete-search
permission diagnosis and requested literal-path actions with preserved data/access.
Size/time predicates, -exec/-delete, Boolean/links/recursive operations and full
GNU syntax remain absent. Quoted find patterns do not imply shell-globbing or
104.7 FHS/command-discovery coverage. LPI5.0/103.3 rechecked2026-10-25; all
1240 source fingerprints/positions remain unchanged. Totals134Partial/1Supporting/
1105Open overlap and are not a completion percentage. See FIND_LESSONS.md.


M98 strengthens seven already Partial LPIC-2v4.5 207.2/K1–K4/T2/T3 rows after
installed72f three-task reverse-DNS acceptance (50nativecaptures, cleanexit).
IPv4 owner/type, full IPv6 nibble and multi-PTR/retirement/two-copy practice now
have independent examples, alternatives and negative/current-state evidence.
No new status or objective source fingerprint is promoted.134Partial/1Supporting/
1105Open remains overlapping bookkeeping, not percentage complete. Native
lookups do not add dig/host/nslookup commands, full BIND configuration, paired
forward/reverse consistency, delegation/RFC2317, transfers, recursion or security.

## M106: explicit DHCP client/relay slices

Current CCNAv1.1 parents4.3 and4.6 gain narrow Partial practice after three
state-graded native configuration assignments and complete installed Fedora
flows. Policy/offers/held leases, relay delivery/physical mask, exclusions and
reservations, current/fresh-client orders and preservation are actually modeled.
DNS option delivery is data, not a working resolver; full DHCP/DNS role coverage,
IOS syntax, real packets/timers/clients/multiple servers and routed connectivity
are absent. No LPIC-2 daemon/configuration or exact OS-client tool credit follows.
All1240 source fingerprints/positions and upcomingCCNAv2 rows remain unchanged.
Counts136Partial/1Supporting/1103Open overlap; they are not completion percentages.
Sources and tests: DHCPV4_LESSONS.md; actual installed evidence: INSTALLATION.md.


M116/M117 installed follow-through promotes only current CCNAv1.1 2.5/a/b and
LPIC-1v5.0 102.4 to narrowly Partial practice after same-ELF133/33 Fedora acceptance.
Four ideal stable-P2P STP assignments teach root/cost/tie/path reasoning; four
script-free package tasks teach installed-versus-cached records, dependency upgrade,
payload repair and distinct remove/purge retention. The exact source fingerprints,
positions and all1240 references remain unchanged. No per-VLAN RapidPVST+ protocol,
BPDUs/convergence/guards, full Debian archive/transaction/resolver/RPM, LPI child
term/knowledge or future-v2 credit is inherited. Overlapping140Partial/1Supporting/
1099Open are not a completion percentage. Same installed38a ELF passed lifecycle,
11normal starts, script/DHCP cold reopen,72alignment and all8normal lesson flows
(69package+48STP captures); representative actual renders reviewed. See
INSTALLATION.md, DEBIAN_PACKAGE_LESSONS.md and SPANNING_TREE_LESSONS.md.


M120 adds only four narrowly Partial LPIC-2 v4.5 rows:208.1 parent and K1/K3/K7,
after model/content/alternative/negative checks and all four real development-native
lesson flows (68 screenshots; representative renders independently reviewed).
The native flow used normal140/35 source, not appended developer labs. It covers
bounded static configuration, directory-path access and wildcard name/alias/default
routing. Dedicated-IP vhosts, logs, authentication, .htaccess, modules, resource
controls and exact named Linux tools remain Open. K8 remains Open: an implicit
directory-slash redirect does not teach configurable Redirect statements.
Current LPI objectives/version and Apache references were rechecked2026-10-26;
all1240 source fingerprints/positions and future Cisco rows remain unchanged.
Source inventory140/35=42CCNA/28LPIC-1/24LPIC-2/46Bash; latest accepted installed
Fedora catalog is now140/35 at ddb886b after the final same-ELF candidate gates. Overlapping
144Partial/1Supporting/1095Open is not a completion percentage. The three reliable
text scripts add real practice but no new issuer-row or universal POSIX claim.


### Basic NAT backend foundation — 2026-10-26

The independently reviewed [pure model](NAT_SUBSET.md) and
[typed workspace backend](NAT_WORKSPACE_SUBSET.md) pass their compiler, outcome,
alternate-policy, bound, invalidation and progress-retention gates. They model
inside-source static and pool address mappings. The [native workbench](NAT_NATIVE.md)
and isolated teaching/practice pass technical gates. Three authored tasks are
being integrated; no accepted normal NAT unit is bundled yet; current CCNA4.1 stays **Open**, and140/35 and all1240
issuer fingerprints/status totals remain unchanged. The planned unit separates
static mapping, dynamic eligibility/capacity diagnosis and preservation handover.
It does not promise PAT, transport sessions, IOS or actual packet delivery. The
old plan's generic NAT/PAT-session label was not the issuer objective wording;
current4.1 explicitly concerns inside-source static mappings and pools. This
clarifies the implementation boundary without granting practice credit or altering
the separate future-blueprint plan.


### RPM foundation — no practical credit yet

The [pure RPM foundation](RPM_PACKAGE_SUBSET.md) passes23 independent behavior/
negative/bound groups and library integration checks. It distinguishes RPM EVR,
capability providers and installed/local/repository sources, with atomic logical
script-free sets. The [command and separate-practice backend](RPM_PACKAGES_WORKSPACE.md)
now passes guarded query, update/removal, alternate-outcome, isolation and saved-
progress gates. Its native workspace and four authored tasks are under
implementation; no RPM practice is bundled. All LPIC-1 v5.0 102.5 references
remain **Open**. Configuration sidecars, general RPM/YUM/Zypper syntax, dependency
solving, signatures, binary archives and real transactions remain unmodeled.
Current140/35 and issuer fingerprints/status totals are unchanged. Backend code
is not implemented learner coverage or complete package administration.
