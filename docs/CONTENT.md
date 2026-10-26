# Content scope and references

Authored and checked 2026-10-21. This page records the original nine short illustrative exercises. The growing authored course inventory and exact remaining gaps are tracked in CURRICULUM.md, OBJECTIVE_MAPPING.md and the unit guides; none claims complete exam preparation. Original fictional systems and reserved documentation addresses are used. All paths, users, devices and logs inside labs are simulated.

Objective alignment was checked against [Cisco CCNA 200-301 v1.1](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf), [LPI LPIC-1 v5.0](https://wiki.lpi.org/wiki/LPIC-1_Objectives_V5.0), and [LPI LPIC-2 v4.5](https://wiki.lpi.org/wiki/LPIC-2_Objectives_V4.5). LPIC-2 v5.0 was labeled draft on the LPI wiki when checked. Cisco's v2.0 page gives a first-test date in 2027; this prototype uses the v1.1 IPv4 objectives. Alignment means a small example of a topic, not complete preparation for that objective.

| Authored lab | Alignment | Modeled learning outcome |
| --- | --- | --- |
| Read an IPv4 address | CCNA 1.6 | Calculate a conventional /27 network, broadcast and usable range, then assign an unused host address |
| Give each team a subnet | CCNA 1.6 | Allocate VLSM networks with adequate usable capacity and no overlap |
| Reconnect the office client | CCNA 1.6, 1.10 | Correct a LAN prefix and reachable default gateway |
| Share a report safely | LPIC-1 104.5 | Repair effective file access while restricting other-class permissions |
| Unlock the project directory | LPIC-1 104.5, 107.1 | Diagnose membership and parent-directory search permissions |
| Find the failing requests | LPIC-1 103.2, 103.3 | Locate a simulated log and filter literal error lines |
| Bring the service back | LPIC-2 202.1 | Repair a configuration and distinguish running from enabled |
| Repair the internal DNS answer | LPIC-2 207.1, 207.2 | Edit an A record, update serial, reload and verify the loaded answer |
| Recover the data volume | LPIC-2 203.1; introductory capacity practice | Repair fstab, remount, free disposable capacity and exercise mounting |

Technical references consulted include [RFC 1878 subnet tables](https://www.rfc-editor.org/rfc/rfc1878), [RFC 3021’s /31 point-to-point exception](https://www.rfc-editor.org/rfc/rfc3021), [Linux path resolution](https://man7.org/linux/man-pages/man7/path_resolution.7.html), [systemctl's upstream manual](https://www.freedesktop.org/software/systemd/man/latest/systemctl.html), [journalctl's upstream manual](https://www.freedesktop.org/software/systemd/man/latest/journalctl.html), [BIND's configuration and zone guide](https://bind9.readthedocs.io/en/latest/chapter3.html), [the upstream mount manual](https://github.com/util-linux/util-linux/blob/master/sys-utils/mount.8.adoc), and [ext4's kernel documentation](https://www.kernel.org/doc/html/latest/filesystems/ext4/index.html). GNU coreutils/grep manuals are also linked in relevant pack entries, but the web fetch timed out during authoring; they are supplementary references. Scenario entries record sources and known simplifications alongside the content.

## Intentional simplifications

- The complete IPv4 assignment uses native fields and one check action. Any different usable second-host address passes; the network, broadcast, occupied address, malformed addresses and outside-subnet addresses fail. The optional binary explanation distinguishes address bits from mask selectors and labels network/host bits. Zero/one host patterns reserve network/broadcast for this scope; /31 and /32 remain excluded. Numerical variants and a separate host-bit activity are optional and are not additional authored labs. Original answers/credit stay separate from numerical practice. VLSM permits larger-than-minimal valid subnets when all allocations fit without overlap. Malformed drafts are retained but cannot pass.
- Connectivity models one client, one LAN router and one remote server. No real packets, ARP, switches, routing protocol, latency or firewall. `ip addr set` is a teaching shortcut for replacing the single client interface, not a complete iproute2 command grammar.
- Unix mutations run with simulated administrator capability; `test-access` evaluates the named ordinary user's access. Owner precedence and directory execute/search are enforced. No ACLs, sticky/setgid/setuid bits, SELinux, capabilities or root bypass. Group changes apply immediately without a new login. `ls -l` lists immediate directory children; `ls -ld` inspects the directory itself. Symbolic permissions are real modeled values; sizes, dates, link counts and numeric IDs are deterministic teaching metadata, not a stat database.
- Log commands support one-word, case-sensitive literals and a small pipeline subset. No regex, shell quoting, expansion, redirection or executable commands. `find` examines only the in-memory log map. Success grades the last successful text query, not terminal history.
- Service configuration is one key and one fictional service. Start/restart requires port 1–65535. Configuration edits do not change an existing runtime; enabling is separate from starting. The native config editor changes the saved file, not the runtime. The existing service assignment now has skippable prerequisite teaching and a separate editor/terminal example (SERVICE_TEACHING.md). Unit status uses Loaded/Active fields and readable states. A bounded typed journal records rejected starts, starts and stops; editing the config never rewrites that history. Journal dates, timestamps and the displayed PID are synthetic. Older saved attempts start with no history rather than fabricated events. No dependency graph, processes or actual listening sockets.
- The original DNS seed has one authoritative A-record map and staged edits. Its strictly increased serial requirement is an exercise constraint, not a universal primary-loader rule. No cache, TTL expiry, recursion, secondary transfer, DNSSEC or serial wrap. Native editors stage the A record/serial and mount-device field without invoking a command. `dns set`, `dns serial` and `config set` remain supported compatibility/test helpers, not real host commands. `named-checkzone lab.test` checks this model directly rather than opening a real zone file.
- Storage uses integer MiB and one modeled mount. Only `rm -r /data/cache` can delete content; it is explicitly disposable. No inode pressure, sparse files, deleted-but-open files, ext4 reserved space or real block devices. The fstab editor changes one device field; no UUID resolution or real boot.

The supported subset is shown per scenario through `help` and the workspace disclosure. Unknown commands/arguments report the limitation. The simulation does not pretend to offer a full Linux or Cisco shell.

The integrated workspace revision keeps the nine authored IDs and curriculum count. Learn and Challenge retain complete assignments, with differing default assistance. Walkthroughs, references and worked reasoning are optional; completion never depends on visiting guidance sections. The IPv4 task includes its approved second-workstation requirement. Completed legacy analyses retain credit through an explicit compatibility migration; other graders and valid alternate repairs/layouts are preserved. The permissions summary and optional explanation describe Alex’s access objective rather than promising an editing-team task.


## Introductory scripting unit (milestone)

Seven newly authored assignments in `scenarios/shell-unit-v1.json`, each with advisory prerequisites and 3–5 deterministic behavior cases: handover output; quoted positional arguments; finite line input; missing-destination validation; filtered report creation; multi-file iteration; safe incident-report capstone. Twenty-four distinct input fixtures exercise these seven assignments; fixtures are not separate labs. The syntax is a portable shell subset, not Bash-specific arrays/functions. References are the official GNU Bash manual and POSIX Issue 8 shell-language specification; checked 2026-10-21. Forty-one controlled oracle fixtures, including every unit case, were compared with development Bash. Supported syntax, limitations and resource bounds are in `SHELL_SUBSET.md`; complete curriculum planning and objective versions are in `CURRICULUM.md` and `CURRICULUM.csv`. None of the planned entries are counted as implemented coverage.


IPv4 static-routing capability and native integration are specified in [IPV4_ROUTING_SUBSET.md](IPV4_ROUTING_SUBSET.md). The development fixture is intentionally excluded from the 64 authored assignments and coverage ledger. Authored lessons teach supported route/address decisions before independent diagnosis, preserve required endpoint identities/addresses and test both forward and reply paths. Native fields are configuration tools, not purported IOS/Linux command syntax. Existing original IPv4/VLSM assignments remain unchanged; their skippable foundational teaching and isolated native examples are documented in ADDRESSING_TEACHING.md.


The six substantive IPv4 static-routing assignments in IPV4_LESSONS.md cover network construction, reverse-path diagnosis, longest-prefix detours, host exceptions, equal-prefix floating behavior and a two-client recovery capstone. They have independent introductions/examples/experiments and state-based positive/negative/alternate tests; number variants remain only ungraded examples. Their narrow objective evidence extends existing Partial rows, without claiming complete IOS/dynamic routing or certification preparation.

The six new forward-DNS lessons are separate from this original-nine scope: see DNS_LESSONS.md, DNS_WORKSPACE.md and DNS_SUBSET.md for typed records, primary/secondary publication, preserved-data goals, current limitations and checked references. Existing seed IDs/goals/progress are unchanged.


Current source inventory and exact implemented-versus-planned coverage live in
CURRICULUM.md and OBJECTIVE_MAPPING.md. M116 registers133 authored tasks, including
four distinct package policies/repairs and four ideal STP graph tasks; its
installed acceptance is still pending. Different example values and generated
practice never add authored lab counts. DEBIAN_PACKAGE_LESSONS.md and
SPANNING_TREE_LESSONS.md record checked2026-10-25 official technical references
and the explicit transaction/protocol limitations. Historical counts above
belong to their original scopes.
