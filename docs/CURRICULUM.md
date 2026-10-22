# Curriculum plan and coverage ledger

Rechecked **2026-10-22**. This is a planning document, not a claim of complete exam preparation. Completion work is tracked in [COMPLETION.md](COMPLETION.md) and [IMPLEMENTATION_LEDGER.md](IMPLEMENTATION_LEDGER.md). Delivered content is the original nine authored assignments plus fifteen scripting assignments in three units. Numerical practice addresses and 900 synthetic catalog entries are never counted as authored labs. Planned assignments are not shown as playable catalog cards.

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

[CURRICULUM.csv](CURRICULUM.csv) has **165 rows**: each active top-level objective in CCNA v1.1, LPIC-1 5.0 and LPIC-2 4.5, plus every upcoming CCNA v2.0 top-level objective. LPI's removed 104.4 is deliberately absent. Cisco lettered subobjectives are scoped under their parent; they are not individually marked covered. This is a complete planning index at that granularity, not complete implemented coverage.

Each row records implemented scenario IDs (or None), a distinct proposed assignment, prerequisites, difficulty, the missing simulation model and its fidelity limit. **Partial** means only a narrow slice is implemented; even the existing labs do not cover the full named objective. Difficulty/prerequisite values are initial authoring estimates. Each proposed assignment may need several carefully authored scenarios after review, rather than one giant task per objective. No placeholder has been promoted into curriculum.

| Delivered assignment | Implemented practice | Important remaining gap |
| --- | --- | --- |
| `ccna-ipv4` | Conventional IPv4 boundaries and second host | No IPv6 or device CLI; /31 and /32 excluded |
| `ccna-vlsm` | Host-capacity allocation, alignment, containment, overlap | No routed topology configuration |
| `ccna-gateway` | Client mask/next-hop diagnosis | One topology; no IOS, ARP, multi-hop protocols or cross-OS command coverage |
| `lpic1-permissions` | Alex-only file access and privacy; alternate repairs | No ACL, SELinux or full Unix filesystem |
| `lpic1-traversal` | Group access and ancestor traversal | No account lifecycle, authentication or complete group database |
| `lpic1-logs` | Locate and filter literal ERROR lines | No regex, complete filter suite or logging service configuration |
| `lpic2-service` | Saved/runtime configuration and boot enablement | One service; no full systemd dependency graph |
| `lpic2-dns` | One A record, serial, reload and query | No recursive cache, other RR types, transfers or DNSSEC |
| `lpic2-storage` | Capacity, saved mount device and writable state | No real blocks, quotas, inodes, integrity repair, RAID or LVM |
| `shell-handover` … `shell-capstone` | Seven portable-shell foundations assignments below | No complete Bash/POSIX implementation; see supported subset |

| `shell-streams`, `shell-pipelines`, `shell-eof`, `shell-batch-audit` | Separate channels, pipeline intersection/status, EOF repair and resilient batches | No functions, shell option parser or broader utility suite |

| `shell-functions`, `shell-return`, `shell-shift`, `shell-bundle` | Reuse, return vs exit, consume arguments, preflight and preserve output | No Bash local/arrays/substitution or complete option parser |

## Learning order and capability gates

CCNA progresses through physical/device evidence → addressing and switching → routing → services → access/security → operational automation. Existing IPv4 work is an entry point, not completion of the fundamentals domain. Add IPv6 and packet/routing models before authoring dual-stack or protocol diagnosis. OSPF, STP, LACP, DHCP and FHRP need explicit state transitions and negative tests before content is accepted. Wireless RF, cable faults and virtualization can use offline evidence and conceptual models; those cannot establish physical or production acceptance. No VMs are required or launched.

LPIC-1 progresses through terminal/files → permissions/accounts → packages/processes → storage/boot → shell/scheduling → networking/services/security. Hardware probing, bootloaders, real package transactions, locale behavior, accessibility and cryptography require new evidence/state models; existing tests do not establish those capabilities. A visual configuration exercise must identify what it models and what it cannot execute.

LPIC-2 assumes LPIC-1 foundations. Sequence capacity and boot recovery → advanced storage/networking → DNS/web/file/identity/mail services → security and backup projects. The service/DNS/mount labs are small seeds. Kernel compilation, genuine recovery, TLS/VPN handshakes and hardware timing are not faithfully demonstrated now; proposed offline models must disclose this and may teach evidence interpretation rather than pretend to perform the operation. Never add a canned command-success response to fill a coverage cell.

## Bash/Shell progression

All fifteen delivered shell labs use **portable shell syntax**, tested against Bash 5.3.9 on Fedora. Bash is the reference oracle, not the learner engine; this is not a claim of POSIX or Bash conformance.

| Stage | Status / distinct assignments | Prerequisites | Capability and portability boundary |
| --- | --- | --- | --- |
| 1. Foundations | **Implemented:** reliable handover; supplied-file labels; input without losing backslashes; missing-destination guard; incident report; multi-file handover; safe-report capstone | Listed preceding lab in the unit, advisory rather than a lock | Scalars, quoting, positional arguments, read, printf, status, test/if, redirects, for, literal grep/cat. Portable subset; 3–5 deterministic cases per assignment |
| 2. Composition and debugging | **Implemented:** route normal/error/file output; compose two-stage literal filtering; repair final unterminated input; audit a batch while retaining earlier failures | Unit 1 | Existing bounded pipeline/while evaluator; 3–6 deterministic cases, independent examples, alternate and plausible incorrect solutions. Bounded retry and tracing remain planned. `set -x`, Bash `local`, and pipefail must be explicitly modeled before teaching them |
| 3. Reusable interfaces | **Implemented:** reusable label printer; per-item rejection without terminating the batch; consume leading prefix; validate-before-writing bundle capstone. **Planned:** option parser and streamed row validator | Stages 1–2 | Portable brace-body functions, return and shift implemented with explicit restrictions. getopts, compound/function-call redirects, command substitution and parameter defaults remain unsupported. Explain subshell scope/status before substitution |
| 4. Structured Bash | **Planned:** safe filename queue; associative lookup; array-based command construction; regex validation | Reusable functions and argument boundaries | Bash-specific indexed/associative arrays, `[[ ]]`, `=~`, arithmetic, `${array[@]}`. Explicit Bash track labels; do not suggest portability |
| 5. Text and data | **Planned:** delimiter-aware inventory transform; regex audit; compare snapshots; recover malformed rows | Streams, conditions and functions | Correct sort/uniq/cut/sed/awk models and regex/locale boundaries needed; none delivered. Distinguish shell language from external utility semantics |
| 6. Safe automation projects | **Planned:** dry-run cleanup planner; verified backup/restore; idempotent configuration rollout; resumable batch processor | Stages 1–5 | Virtual tree, safe path traversal, temporary-file/rename semantics, traps/cleanup, deterministic clocks and locking. No host actions. Teach validation before mutation, quoting, exit propagation and recovery |

Each planned stage includes a guided repair, independent construction and capstone with multiple deterministic cases, adverse inputs, alternate supported solutions and current-state invalidation. Do not simply vary constants to inflate counts. Before implementing unsupported syntax, extend the grammar, expansion order, evaluator and development-oracle fixtures; review fidelity and resource bounds before authoring the lesson.

## Next capability batch

The four composition assignments are implemented and independently reviewed; see [SHELL_COMPOSITION.md](SHELL_COMPOSITION.md). The reusable-interface capability and four reviewed assignments are implemented; see [SHELL_REUSABLE.md](SHELL_REUSABLE.md). Next extend Bash scope, structured data and expansion only after faithful grammar/evaluation and bounded execution tests. IPv6 addressing/neighbor state is the next networking capability; retain the separate current/upcoming Cisco blueprints. Full certification and advanced shell coverage remain open.
