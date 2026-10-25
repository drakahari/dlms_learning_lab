# Curriculum plan and coverage ledger

Rechecked **2026-10-22**. This is a planning document, not a claim of complete exam preparation. Completion work is tracked in [COMPLETION.md](COMPLETION.md) and [IMPLEMENTATION_LEDGER.md](IMPLEMENTATION_LEDGER.md). Source inventory is **119 authored assignments in29 units:35CCNA,24LPIC-1,20LPIC-2 and40Bash/Shell**. The three new reverse-DNS tasks have passed model/content gates; actual native acceptance is pending. The installed116 Fedora candidate passes gateway/Find native flows,72 alignment captures,10 quiet starts and strict real two-process cold reopening. These are scoped milestone checks, not full-product or other-platform acceptance. The scripting path has ten units; the original nine scenarios retain their IDs and progress. Numerical practice addresses and 900 synthetic catalog entries are never counted as authored labs. Planned assignments are not shown as playable catalog cards.

## Primary sources and version boundaries

| Track | Blueprint checked | Official sources |
| --- | --- | --- |
| CCNA | 200-301 **v1.1**, current testing through 2027-02-02 | [Cisco exam overview](https://www.cisco.com/site/us/en/learn/training-certifications/exams/ccna.html), [v1.1 objectives PDF](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf) |
| CCNA, upcoming | 200-301 **v2.0**, testing starts 2027-02-03; separate planning rows | [v2.0 objectives PDF](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301_CCNA_v2.0_Exam_Topics_PDF.pdf), [Cisco transition notice](https://blogs.cisco.com/learning/stay-on-track-get-certified-before-the-ccna-refresh) |
| LPIC-1 | **5.0**, exams 101-500 / 102-500 | [Current objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/), [version overview](https://www.lpi.org/our-certifications/lpic-1-overview/) |
| LPIC-2 | **4.5**, exams 201-450 / 202-450 | [Current objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/), [version overview](https://www.lpi.org/our-certifications/lpic-2-overview/) |
| Bash/Shell | Learning progression, no certification-coverage assertion; subset v1 | [POSIX Issue 8 shell language](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html), [GNU Bash manual](https://www.gnu.org/s/bash/manual/bash.html), [expansion](https://www.gnu.org/s/bash/manual/html_node/Shell-Expansions.html), [redirection](https://www.gnu.org/s/bash/manual/html_node/Redirections.html) |

The Cisco PDF and LPI objective pages were inspected, including versions and complete top-level objective identifiers. Cisco's dynamic learning-network page failed to render; official PDFs and the dated Cisco transition article supplied the evidence. Some GNU `software/` URLs timed out; the official `/s/bash/` manual and POSIX reference supplied the language reference. Do not infer objectives from an old training course or exam dumps. Recheck sources before every reviewed content batch, and attach finer subobjective evidence during authoring.

Recheck evidence (2026-10-22): both official PDFs identify their blueprint versions; [Cisco’s certification page](https://www.cisco.com/web/go/ccna/) states the last v1.1 and first v2.0 testing dates. Both LPI overview/objective pages retain the versions above. No version changed in this recheck.

## Complete planning crosswalk

[CURRICULUM.csv](CURRICULUM.csv) has **165 rows**: each active top-level objective in CCNA v1.1, LPIC-1 5.0 and LPIC-2 4.5, plus every upcoming CCNA v2.0 top-level objective. LPI's removed 104.4 is deliberately absent. Cisco lettered subobjectives are scoped under their parent; they are not individually marked covered. This is a complete planning index at that granularity, not complete implemented coverage. The [granular evidence/gap ledger](OBJECTIVE_MAPPING.md) separately indexes all current lettered subobjectives, LPI knowledge areas and named tools/terms, with explicit evidence per item and source-change guards.

Each row records implemented scenario IDs (or None), a distinct proposed assignment, prerequisites, difficulty, the missing simulation model and its fidelity limit. **Partial** means only a narrow slice is implemented; even the existing labs do not cover the full named objective. Difficulty/prerequisite values are initial authoring estimates. Each proposed assignment may need several carefully authored scenarios after review, rather than one giant task per objective. No placeholder has been promoted into curriculum.

| Delivered assignment | Implemented practice | Important remaining gap |
| --- | --- | --- |
| `ccna-ipv4` | Conventional IPv4 boundaries and second host | No IPv6 or device CLI; /31 and /32 excluded |
| `ccna-vlsm` | Host-capacity allocation, alignment, containment, overlap | No routed topology configuration |
| `ccna-ipv6-neighbor` … `ccna-ipv6-handover` | Six native IPv6 addressing/static-route tasks with return paths and deterministic failover | No dynamic ND/RA/SLAAC, full address-type course, IOS or dynamic routing; objectives1.8/1.9/3.3 stay partial |
| `ccna-gateway` | Client mask/next-hop diagnosis | One topology; no IOS, ARP, multi-hop protocols or cross-OS command coverage |
| `lpic1-file-navigation` … `lpic1-file-handover` | Six file-management tasks: paths, quoted names, copy/move, cleanup, creation mask and capstone | No globs, recursive operations, links, ownership changes or full GNU option surface |
| `lpic1-permissions` | Alex-only file access and privacy; alternate repairs | No ACL, SELinux or full Unix filesystem |
| `lpic1-traversal` | Group access and ancestor traversal | No account lifecycle, authentication or complete group database |
| `lpic1-logs` | Locate and filter literal ERROR lines | No regex, complete filter suite or logging service configuration |
| `lpic2-service` | Saved/runtime configuration and boot enablement | One service; no full systemd dependency graph |
| `lpic2-dns` | One A record, serial, reload and query | No recursive cache, other RR types, transfers or DNSSEC |
| `lpic2-storage` | Capacity, saved mount device and writable state | No real blocks, quotas, inodes, integrity repair, RAID or LVM |
| `shell-handover` … `shell-capstone` | Seven portable-shell foundations assignments below | No complete Bash/POSIX implementation; see supported subset |

| `shell-streams`, `shell-pipelines`, `shell-eof`, `shell-batch-audit` | Separate channels, pipeline intersection/status, EOF repair and resilient batches | No complete shell option parser or broader utility suite; functions are taught in the next unit |

| `shell-functions`, `shell-return`, `shell-shift`, `shell-bundle` | Reuse, return vs exit, consume arguments, preflight and preserve output | This unit does not assess arrays, substitution or option parsing; later units teach arrays, capture and Bash scope, while getopts remains planned |

## Learning order and capability gates

CCNA progresses through physical/device evidence → addressing and switching → routing → services → access/security → operational automation. Existing IPv4 work is an entry point, not completion of the fundamentals domain. Bounded IPv4 and IPv6 static-forwarding models each support six reviewed assignments. Four bounded OSPFv2 configuration/route-reasoning assignments are implemented; neighbor FSM, LSA exchange, convergence and broadcast DR/BDR are not. Integrated dual-stack diagnosis, STP, LACP, DHCP and FHRP still require additional models and substantive content. Wireless RF, cable faults and virtualization can use offline evidence and conceptual models; those cannot establish physical or production acceptance. No VMs are required or launched.

LPIC-1 progresses through terminal/files → permissions/accounts → packages/processes → storage/boot → shell/scheduling → networking/services/security. Hardware probing, bootloaders, real package transactions, locale behavior, accessibility and cryptography require new evidence/state models; existing tests do not establish those capabilities. A visual configuration exercise must identify what it models and what it cannot execute.

LPIC-2 assumes LPIC-1 foundations. Sequence capacity and boot recovery → advanced storage/networking → DNS/web/file/identity/mail services → security and backup projects. The service/DNS/mount labs are small seeds. Kernel compilation, genuine recovery, TLS/VPN handshakes and hardware timing are not faithfully demonstrated now; proposed offline models must disclose this and may teach evidence interpretation rather than pretend to perform the operation. Never add a canned command-success response to fill a coverage cell.

## Bash/Shell progression

The first fifteen shell labs and four text-processing labs use **portable shell syntax**; the three scope lessons explicitly teach **Bash-specific function scope**. All are tested against Bash 5.3.9 on Fedora. Bash is the reference oracle, not the learner engine; this is not a claim of POSIX or Bash conformance.

| Stage | Status / distinct assignments | Prerequisites | Capability and portability boundary |
| --- | --- | --- | --- |
| 1. Foundations | **Implemented:** reliable handover; supplied-file labels; input without losing backslashes; missing-destination guard; incident report; multi-file handover; safe-report capstone | Listed preceding lab in the unit, advisory rather than a lock | Scalars, quoting, positional arguments, read, printf, status, test/if, redirects, for, literal grep/cat. Portable subset; 3–5 deterministic cases per assignment |
| 2. Composition and debugging | **Implemented:** route normal/error/file output; compose two-stage literal filtering; repair final unterminated input; audit a batch while retaining earlier failures | Unit 1 | Existing bounded pipeline/while evaluator; 3–6 deterministic cases, independent examples, alternate and plausible incorrect solutions. Bounded retry and tracing remain planned. `set -x` and pipefail remain unsupported; Bash local is taught in the following scope unit |
| 3. Reusable interfaces | **Implemented:** reusable label printer; per-item rejection without terminating the batch; consume leading prefix; validate-before-writing bundle capstone. **Implemented scope continuation:** caller-title isolation, nested filter visibility and input-preserving review repair. **Planned:** option parser and streamed row validator | Stages 1–2 | Portable brace-body functions, return and shift implemented with explicit restrictions. Bounded command-output capture now has its own taught unit (SHELL_CAPTURE_LESSONS.md). getopts, compound/function-call redirects and parameter defaults remain unsupported |
| 4. Structured Bash | **Implemented:** exact dispatch lists, whole-list mutation repair, grouped package inventory and preview/save handover. **Planned:** associative lookup, command construction and regex validation | Reusable functions, scope and filename selection | Bounded Bash-specific dense indexed arrays/counts/quoted @ implemented and taught. No associative/sparse arrays, computed indices/index writes, `[[ ]]`, `=~` or full Bash arrays. Portable arithmetic remains a separate counters unit. |
| 5. Text and data | **Implemented:** inventory field extraction, unique manifests, grouped location counts and a duplicate-ID audit. **Planned:** regex audit, snapshot comparison and broader malformed-row recovery | Streams, conditions and functions | Bounded C-locale sort/uniq/cut models and four reviewed assignments implemented; broader sed/awk/regex/locale behavior still needed. Distinguish shell language from external utility semantics |
| 6. Safe automation projects | **Planned:** dry-run cleanup planner; verified backup/restore; idempotent configuration rollout; resumable batch processor | Stages 1–5 | Virtual tree, safe path traversal, temporary-file/rename semantics, traps/cleanup, deterministic clocks and locking. No host actions. Teach validation before mutation, quoting, exit propagation and recovery |

Each planned stage includes a guided repair, independent construction and capstone with multiple deterministic cases, adverse inputs, alternate supported solutions and current-state invalidation. Do not simply vary constants to inflate counts. Before implementing unsupported syntax, extend the grammar, expansion order, evaluator and development-oracle fixtures; review fidelity and resource bounds before authoring the lesson.

## Next capability batch

The four composition assignments are implemented and independently reviewed; see [SHELL_COMPOSITION.md](SHELL_COMPOSITION.md). The reusable-interface capability and four reviewed assignments are implemented; see [SHELL_REUSABLE.md](SHELL_REUSABLE.md). Three [Bash scope/debugging assignments](SHELL_SCOPE.md) now follow the reusable unit, using the tested function-local subset. Bounded filename matching and three reviewed batch-selection/publication assignments now follow; see SHELL_FILE_BATCHES.md. Reviewed dense indexed lists and bounded cooperative command-output capture are now implemented; associative data, broader option parsing and robust recovery remain planned. IPv6 native addressing/static-route foundations are implemented; retain the separate current/upcoming Cisco blueprints. Full certification and advanced shell coverage remain open.

Linux file-management capability and six reviewed assignments: [LINUX_FILES.md](LINUX_FILES.md). This delivers narrow practical slices of LPIC-1 103.1, 103.3 and 104.5; the full objectives remain partial.

Four [text-record assignments](SHELL_TEXT.md) add narrow LPIC-1 103.2 filter practice, with separate cut/sort/uniq evidence. Full tool and option coverage remain open.

Four [counters/totals assignments](SHELL_COUNTS.md) teach bounded portable integer calculations and sequential read-only virtual-file loops before a safe counting/publication project. This does not complete structured Bash, numeric-text validation, advanced safe automation or any full issuer objective.

Four [access-policy assignments](LINUX_ACCESS.md) follow the six file foundations and precede the original access seeds. Narrow104.5 ordinary-file/directory/group practice is expanded; special files, special bits, privileged ownership/ACL/SELinux remain absent.


Three [filename-selection/batch assignments](SHELL_FILE_BATCHES.md) add simple * pattern practice, incident harvesting, and a captured-source handover that rejects collisions before writing. Flat virtual paths only; advanced/directory/bracket patterns and the broader safe-automation path remain open; later command-output capture is taught separately. This strengthens a narrow LPIC-1 103.3 slice; the separate Linux terminal does not gain expansion.

M40 LPIC-2 source recheck2026-10-23 confirms v4.5/201-450/202-450. Six distinct forward-DNS assignments add editable record syntax, typed alias/MX/address-set diagnosis, safe preservation and explicit primary/secondary revision practice; see DNS_LESSONS.md. This is a narrow207.1 reload /207.2 forward-record slice. Full BIND paths/configuration/tools, reverse zones, recursion, delegation/glue, transfer protocol and security remain missing. Catalog70/sixteen units; no full exam or future-version coverage claim. The complete objective source fingerprint inventory retains its last full2026-10-22 check date.

M44 adds four distinct Bash-specific indexed-list assignments in the eighth shell unit: exact dispatch queues, debugging whole-list replacement, grouped filename inventories and safe preview/save publication. See SHELL_ARRAY_LESSONS.md. Catalog74/seventeen units (19CCNA,13LPIC-1,9LPIC-2,33Bash). This does not promote any certification objective:97Partial/1143Open remain. Arrays do not establish portable sh, complete Bash, certification readiness or learner mastery; substitution/regex/associative data/status/debugging/recovery families remain planned.

## Process lifecycle and priority continuation — M47

Four authored LPIC-1 assignments extend Linux foundations through ownership-aware inspection, safe pause, pending termination, absolute niceness and a parent/child handover. Each has a separate editable practice table and full independent terminal task. Existing74 IDs/versions/initials/goals stay unchanged. Catalog78/eighteen units:19CCNA,17LPIC-1,9LPIC-2,33Bash. Narrow evidence is108Partial/1Supporting/1131Open of1240 overlapping references, not completion percentages. Default/inherited nice is explained, not assessed through process creation. See PROCESS_LESSONS.md and PROCESS_SUBSET.md.

## LLDP discovery continuation — M50

Four authored CCNA assignments teach receive direction, stale advertised inventory, selective transmit/reception and a multi-attachment recovery. Each retains a separate runnable example and a full native editing/verification task. Current CCNA v1.1 2.3 gains narrow LLDP Partial practice only; CDP/IOS/automatic scheduling/full TLVs and equipment conformance remain absent. Current official v1.1 PDF and exam overview rechecked2026-10-23. Existing78 definitions/progress unchanged. Catalog82/nineteen units:23CCNA,17LPIC-1,9LPIC-2,33Bash.109Partial/1Supporting/1130Open references remain overlapping gap/evidence bookkeeping, not percentages.

## M55 — persistent mounts and UUIDs

Four distinct LPIC-2 assignments now teach stable supplied identity, saved versus current access policy, manual-only mounting and a preserved multi-volume handover. Catalog86/twenty units:23CCNA,17LPIC-1,13LPIC-2,33Bash. Narrow203.1 UUID/blkid evidence brings the overlapping ledger to111Partial/1Supporting/1128Open; no filesystem-maintenance, swap, systemd-unit or real-boot coverage is claimed. Official LPIC-2 v4.5 objectives rechecked2026-10-23, full inventory fingerprints remain2026-10-22. See STORAGE_LESSONS.md.


## M57 — command output, decisions and report previews

Four distinct assignments teach intact captured lists, newline-preserving report evidence, helper output versus status, and a guarded preview/save batch capstone with shared ledger and isolated caller values. New examples use different data, stay ungraded and retain their own drafts/results. The approved full native editor task remains in Learn and Challenge; only guidance differs. Catalog90/twenty-one units:23CCNA/17LPIC-1/13LPIC-2/37Bash. See SHELL_CAPTURE_LESSONS.md for cases, alternatives, exact limits and validation.

LPIC-1 current v5.0 objectives and105.2 were rechecked2026-10-23. Only105.2.K2 gains narrow Partial practice for supported original-source dollar-parenthesis capture. Parent105.2 and return-status K3 evidence strengthens without full coverage; mail, executable/shebang/account policy and wider shell behavior stay Open.112Partial/1Supporting/1127Open overlapping references are not completion percentages. The full inventory fingerprints retain2026-10-22. This unit does not claim full Bash/POSIX fidelity, certification preparation or learner mastery.

## IPv4 access-policy unit — M62

Four distinct source/service-order/direction/two-boundary assignments now add narrowly Partial current CCNA5.6 practice. Their full native tasks have31 graded packets,22 separate example packets, alternatives/negative/invalidation tests and actual Fedora input/render acceptance. Current-source catalog94/22units:27CCNA/17LPIC-1/13LPIC-2/37Bash. The overlapping matrix now has113Partial/1Supporting/1126Open; it is not a completion percentage. Arbitrary wildcard, full IOS/hardware, IPv6 ACL and wider verification gaps remain explicit. Official v1.1/5.6 and technical sources rechecked2026-10-23; futurev2.0 remains separate. See ACL_LESSONS.md. This source milestone is not yet a new installed candidate.


## Local accounts and handover — M65

Four reviewed [local-record lifecycle assignments](ACCOUNT_LESSONS.md) follow the access-policy unit. They teach explicit account/home creation, exact membership repair and owned-home GID effects, precise retirement with orphan numeric evidence, and a stable-GID/expiry-metadata handover. Different runnable examples remain separate from full assignments; alternate supported commands and negative/current-state checks pass. Catalog98/23units:27CCNA/21LPIC-1/13LPIC-2/37Bash. Current LPI v5.0/107.1 was rechecked2026-10-23. Only explicit local-record/tool slices are Partial; passwords, chage, special-purpose policy, PAM/login, full NSS and complete options remain absent.

## M67 — configured OSPFv2 practice

Four original [P2P OSPF assignments](OSPF_LESSONS.md) add explicit identity/process diagnosis, parameter matching, directed equal-cost reasoning and a passive-LAN handover. They retain full native configuration tasks, different isolated worked networks, alternate solutions and current-state checks. Catalog102/24units:31CCNA/21LPIC-1/13LPIC-2/37Bash. Current CCNA v1.1/3.4 sources were rechecked2026-10-23; only four explicitly bounded current-version rows gain Partial practice. Most networking/Linux/scripting objectives remain unfinished. No neighbor FSM/Full observation, LSA exchange, convergence, broadcast DR/BDR or real-device/packet-delivery coverage is claimed; futurev2.0 remains separate. Fedora native evidence and final acceptance limits belong in the ledger.


M70 adds four [service definition/dependency/handover assignments](UNIT_MANAGER_LESSONS.md) in a coherent LPIC-2 unit, with separate worked systems, full editors/terminals, active-preparation retry, alternate solutions and current plus next-session outcomes. Normal catalog106/25units:31CCNA/21LPIC-1/17LPIC-2/37Bash. Official current LPIC-2v4.5/202.1 sources were rechecked2026-10-23. Only existing Partial202.1/K1/T4 evidence strengthens; SysV/LSB/real boot/readiness/full systemd and most curriculum stay unfinished. No new objective status counts or exam-completion claim.


M77 adds three case-dispatch assignments: an argument interface, prioritized full-record routing and a validate-before-publication capstone. Separate examples,31 deterministic assignment cases and alternate conditional solutions are verified in the bounded simulator and actual native Fedora flow. LPIC-1 105.2 remains narrow Partial reinforcement; mail, real interpreter selection/permissions and wider shell semantics remain absent. Native light150%/dark200% editing and save/readback/reset pass; cold-process reopen, novice, screen-reader and other-platform acceptance are not established by that flow. See SHELL_CASE_LESSONS.md.


## Ethernet learning and aging — 2026-10-24

Four distinct [MAC assignments](MAC_LESSONS.md) now follow VLAN foundations,
before static routing. They teach source learning, known/unknown/broadcast
forwarding, selective stale-location recovery, independent quiet-time reasoning
and a three-switch/two-group handover. Current CCNA v1.1 1.13/a–d receive narrow
Partial practice; source fingerprints remain unchanged, with scoped official
PDF and technical references rechecked2026-10-24. The overlapping ledger is now
133Partial/1Supporting/1106Open, not percentages or full objectives.

Native component flows and actual fullscreen/windowed100/150/200 Light/Dark
render review are accepted; installed Fedora73777e0 also passes the complete
all-four native flow,72-view shared alignment, quiet10/10 and cold reopening. Bounded find and reverse-DNS capabilities alone add no
new authored labs or certification credit. Most full-product criteria remain
Open, including broader networking/Linux/shell content and platform acceptance.


M96 integrates three distinct reverse-DNS assignments after forward DNS: IPv4
owner/type repair, full32-nibble IPv6 rollout, and multi-PTR/retirement/two-copy
handover. Eleven model/content checks include separate practice, two alternatives
per task, wrong/partial/current invalidation and original116 preservation. Actual
native flows are pending, so the coverage matrix remains134Partial/1Supporting/
1105Open and the latest scoped installed artifact remains116/28. Original
permissions/traversal/log tasks now have source-tested introductions and separate
terminal examples; new actual native review is still required. See
DNS_REVERSE_LESSONS.md and LEGACY_ACCESS_LOGS_TEACHING.md. No new getopts or full
certification/curriculum claim.
