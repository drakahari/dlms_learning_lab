# Curriculum reconciliation and remaining implementation

Current Main182 source, reconciled2026-10-29; original154 baseline `4bc390a` is retained in historical evidence. This is an implementation queue, not
new curriculum or a completion claim. The accepted product requirements remain
in [COMPLETION.md](COMPLETION.md): substantive native simulation, prerequisite
teaching, separate runnable practice, meaningful assignments, negative/alternate
grading, persistence and available native acceptance. There is no lab-count target.

## Implemented inventory and unresolved references

| Track | Integrated authored labs | Units | Open references | Partial practice | Supporting only |
| --- | ---: | ---: | ---: | ---: | ---: |
| CCNA | 53 | 14 | 65 | 46 | 0 |
| LPIC-1 | 40 | 10 | 501 | 96 | 1 |
| LPIC-2 | 32 | 8 | 498 | 33 | 0 |
| Bash/Shell | 57 | 16 | Separate path below | — | — |

All136 current parent objectives remain incomplete (94 Open,42 Partial).
The full1240 overlapping parent/subobjective/knowledge/tool records in
[SKILL_COVERAGE.csv](SKILL_COVERAGE.csv) identify **every** unresolved reference,
its exact source position, implemented lab IDs, verified slice, remaining behavior
and proposed assignment. There is no inherited child/tool credit. Counts overlap
and are not a completion percentage. Documentation, pure libraries, private
worktrees and generated numerical practice are excluded from authored totals.

Current versions were checked2026-10-28 against the official
[CCNA v1.1 PDF](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),
[LPIC-1 v5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
and [LPIC-2 v4.5 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/).
This version check does not redate the2026-10-22 normalized source fingerprints.
Cisco [separates current v1.1 from v2.0 starting2027-02-03](https://blogs.cisco.com/learning/stay-on-track-get-certified-before-the-ccna-refresh).
The29 future-version planning rows stay separate and receive no inherited credit.

## Every current objective and its remaining practical assignment

Each row is the recorded broader task still to implement, including when related
authored work already exists. Prerequisites, difficulty and required capabilities
remain in [CURRICULUM.csv](CURRICULUM.csv). The detailed remaining column of
SKILL_COVERAGE.csv is authoritative for each individual child and named tool.
An assignment may need several distinct activities; one row is not an arbitrary
one-lab quota. Physical/kernel/cryptographic limits require explicit explanation
and justified modeled evidence; they cannot silently substitute for missing
configuration or troubleshooting.

### CCNA v1.1

| Objective | Current practical credit | Remaining recorded assignment | Capability still required / fidelity gap |
| --- | --- | --- | --- |
| 1.1 | Open | Trace a campus request through its devices | device roles and packet path. Required model not implemented; cannot claim faithful command or state behavior. |
| 1.2 | Open | Choose a resilient branch topology | topology graph. Required model not implemented; cannot claim faithful command or state behavior. |
| 1.3 | Partial practice · ccna-physical-media-span;ccna-physical-duplex-diagnosis;ccna-physical-counter-baseline;ccna-physical-uplink-handover | Select a suitable physical uplink | Shared-medium/pairwise architecture, real transceivers/optical power/BER/installation and full cable specification/IOS/device conformance remain unimplemented; no full objective credit. |
| 1.4 | Partial practice · ccna-physical-duplex-diagnosis;ccna-physical-counter-baseline;ccna-physical-uplink-handover | Diagnose an error-prone uplink | Finite deterministic fault/counter model; no physical measurement, general BER/error taxonomy, real collision/backoff protocol or complete IOS troubleshooting commands. |
| 1.5 | Open | Explain a transport failure from a trace | TCP/UDP packet model. Required model not implemented; cannot claim faithful command or state behavior. |
| 1.6 | Partial practice · ccna-ipv4;ccna-vlsm;ccna-gateway;ccna-ipv4-branch-handover | Allocate and verify a branch address plan | Bounded native static practice only; no IOS protocol-code/metric display, dynamic protocol selection, recursive next hops, ECMP, real packets or full equipment-stack behavior. Conventional IPv4 interfaces exclude /31-/32. |
| 1.7 | Partial practice · ccna-private-office-plan | Repair a private-address overlap | Single supplied LAN and flat native configuration only; no public Internet, NAT, DHCP, ARP/kernel/device execution or complete private-address design. /31 and /32 remain excluded. |
| 1.8 | Partial practice · ccna-ipv6-neighbor;ccna-ipv6-workstation;ccna-ipv6-handover | Configure a dual-stack segment | No ND/RA/SLAAC, dual-stack or IOS behavior. |
| 1.9 | Partial practice · ccna-ipv6-neighbor;ccna-ipv6-scope | Choose appropriate IPv6 destinations | No complete global/ULA/link-local choice progression, multicast, general anycast or modified EUI-64 instruction. |
| 1.10 | Partial practice · ccna-gateway | Repair a client across OS-style views | No Windows/macOS client-command workspaces or complete client parameter checks. |
| 1.11 | Open | Resolve a channel interference plan | deterministic RF/channel model. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |
| 1.12 | Open | Map virtual and physical isolation | conceptual host/VRF topology. Required model not implemented; cannot claim faithful command or state behavior. |
| 1.13 | Partial practice · ccna-mac-learn-paths;ccna-mac-stale-location;ccna-mac-source-refresh;ccna-mac-handover | Predict a switch learning table | Finite metadata frames and manual time over fixed acyclic equipment. No real NIC/packet bytes, STP/redundant paths, arbitrary multicast, secure/static entries, full IOS/hardware or security guarantee. Native component and installed Fedora73777e0 all-four runs pass. See MAC_LESSONS.md. |
| 2.1 | Partial practice · ccna-vlan-access;ccna-vlan-trunk;ccna-vlan-native;ccna-vlan-handover | Reconnect a host in the correct VLAN | No voice/default-VLAN-specific progression, inter-VLAN routing, STP/PVST/PVID blocking, actual IOS or equipment conformance. Single educational tagged-native ingress policy; full objective remains incomplete. Source learning/aging is taught separately in MAC_LESSONS.md. |
| 2.2 | Partial practice · ccna-vlan-access;ccna-vlan-trunk;ccna-vlan-native;ccna-vlan-handover | Repair mismatched trunk settings | No voice/default-VLAN-specific progression, inter-VLAN routing, STP/PVST/PVID blocking, actual IOS or equipment conformance. Single educational tagged-native ingress policy; full objective remains incomplete. Source learning/aging is taught separately in MAC_LESSONS.md. |
| 2.3 | Partial practice · ccna-lldp-receive;ccna-lldp-inventory;ccna-lldp-transmit-policy;ccna-lldp-handover | Correct a misleading topology record | No CDP, IOS command syntax, real frames/NICs, automatic timers/reinitialization/fast-start, LLDP-MED or wider TLVs, multiple scopes, physical-device acceptance or full IEEE802.1AB conformance. Current diagnostics use simulator knowledge; manual actions differ from equipment scheduling. |
| 2.4 | Partial practice · ccna-lacp-initiation;ccna-lacp-member-service;ccna-lacp-resilient-handover | Repair an inconsistent uplink bundle | Layer3 bundles, actual IOS CLI/output, LACP packet exchange/negotiation/timers/convergence, hashing/throughput and physical-device verification remain absent. Finite L2 eligibility/service snapshots do not complete objective2.4 or the future blueprint. |
| 2.5 | Partial practice · ccna-stp-root;ccna-stp-local-cost;ccna-stp-parallel;ccna-stp-handover | Prevent a loop without isolating users | Rapid PVST+/per-VLAN instances, BPDUs, protocol transitions/timers/convergence, primary/secondary failover, Backup/shared-media roles, PortFast/guards, real IOS/output and physical traffic remain absent. |
| 2.6 | Open | Choose an AP deployment mode | wireless architecture model. Required model not implemented; cannot claim faithful command or state behavior. |
| 2.7 | Open | Wire a controller-based access point | physical/logical WLAN topology. Required model not implemented; cannot claim faithful command or state behavior. |
| 2.8 | Open | Restrict a management access path | device AAA/transport policy. Required model not implemented; cannot claim faithful command or state behavior. |
| 2.9 | Open | Repair a WLAN client policy | native wireless config simulator. Required model not implemented; cannot claim faithful command or state behavior. |
| 3.1 | Partial practice · ccna-ipv6-workstation;ccna-ipv6-specific-route;ccna-ipv6-backup;ccna-ipv4-network-route;ccna-ipv4-return-route;ccna-ipv4-specific-route;ccna-ipv4-host-exception;ccna-ipv4-floating-route;ccna-ipv4-branch-handover | Explain the selected forwarding entry | Bounded native static practice only; no IOS protocol-code/metric display, dynamic protocol selection, recursive next hops, ECMP, real packets or full equipment-stack behavior. Conventional IPv4 interfaces exclude /31-/32. |
| 3.2 | Partial practice · ccna-ipv6-specific-route;ccna-ipv6-backup;ccna-ipv4-specific-route;ccna-ipv4-host-exception;ccna-ipv4-floating-route | Resolve overlapping route preferences | Bounded native static practice only; no IOS protocol-code/metric display, dynamic protocol selection, recursive next hops, ECMP, real packets or full equipment-stack behavior. Conventional IPv4 interfaces exclude /31-/32. |
| 3.3 | Partial practice · ccna-ipv6-workstation;ccna-ipv6-scope;ccna-ipv6-specific-route;ccna-ipv6-backup;ccna-ipv6-handover;ccna-ipv4-network-route;ccna-ipv4-return-route;ccna-ipv4-specific-route;ccna-ipv4-host-exception;ccna-ipv4-floating-route;ccna-ipv4-branch-handover | Restore a primary and backup route | Bounded native static practice only; no IOS protocol-code/metric display, dynamic protocol selection, recursive next hops, ECMP, real packets or full equipment-stack behavior. Conventional IPv4 interfaces exclude /31-/32. |
| 3.4 | Partial practice · ccna-ospf-router-identity;ccna-ospf-p2p-parameters;ccna-ospf-directed-ecmp;ccna-ospf-passive-handover | Recover an OSPF neighbor | Neighbor FSM/Full state, Hello/LSA exchange, flooding, protocol timers/convergence, broadcast DR/BDR, multi-area/external routing and IOS CLI are not implemented yet. Snapshot calculations do not demonstrate packet delivery or real equipment experience. |
| 3.5 | Open | Trace gateway failover | FHRP state machine. Required model not implemented; cannot claim faithful command or state behavior. |
| 4.1 | Partial practice · ccna-nat-static;ccna-nat-pool;ccna-nat-handover | Configure a static address mapping; diagnose pool eligibility/capacity; preserve mappings during a policy handover | No IOS CLI/interface configuration or real equipment verification; no PAT, ports, sessions, routing, connectivity or real packets. Actual Fedora normal-lesson and installed00d69d3 flows passed; NAT-specific cold and other-platform acceptance remain pending. This is partial objective practice. |
| 4.2 | Open | Repair an inconsistent time source | Bounded single-upstream dependency model passes18 core tests; no accepted teaching/native assignments yet. Packet timing/filter/discipline, multiple-peer selection, ntp master, authentication and real clock behavior remain absent. |
| 4.3 | Partial practice · ccna-dhcp-held-options;ccna-dhcp-relay-scope;ccna-dhcp-lease-handover | Trace a name and address assignment | Broader DNS-role instruction, real resolver behavior, protocol packets, automatic client behavior and the full DHCP/DNS objective remain incomplete. Numeric network parameters do not prove routing or DNS service. |
| 4.4 | Open | Interpret a monitoring alarm | SNMP counter/event model. Required model not implemented; cannot claim faithful command or state behavior. |
| 4.5 | Open | Route an actionable log event | syslog severity/facility model. Required model not implemented; cannot claim faithful command or state behavior. |
| 4.6 | Partial practice · ccna-dhcp-held-options;ccna-dhcp-relay-scope;ccna-dhcp-lease-handover | Restore a relayed client lease | One logical server with supplied direct/relay transport facts; no IOS command syntax, real packets, automatic client timers/retries, multiple servers, routed/DNS connectivity or full RFC/device conformance. |
| 4.7 | Open | Classify congestion-sensitive traffic | QoS queue/policy model. Required model not implemented; cannot claim faithful command or state behavior. |
| 4.8 | Open | Restrict remote CLI administration | SSH management policy. Required model not implemented; cannot claim faithful command or state behavior. |
| 4.9 | Open | Choose a safe configuration transfer | FTP/TFTP metadata and transfer model. Required model not implemented; cannot claim faithful command or state behavior. |
| 5.1 | Open | Triage a suspicious network change | threat evidence model. Required model not implemented; cannot claim faithful command or state behavior. |
| 5.2 | Open | Choose a proportionate protection control | security policy decisions. Required model not implemented; cannot claim faithful command or state behavior. |
| 5.3 | Open | Restrict device login privileges | local AAA identities. Required model not implemented; cannot claim faithful command or state behavior. |
| 5.4 | Open | Repair a credential policy | credential policy model. Required model not implemented; cannot claim faithful command or state behavior. |
| 5.5 | Open | Choose a suitable VPN design | VPN topology metadata. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |
| 5.6 | Partial practice · ccna-acl-source-policy;ccna-acl-service-order;ccna-acl-direction-repair;ccna-acl-policy-handover | Repair ordered access-control entries | Finite IPv4 flows/native whole-document subset only; arbitrary wildcard practice, IPv6 ACLs, fragments, NAT, flags/established, logging/counters, full IOS/physical verification and broader policy coverage remain absent. |
| 5.7 | Open | Stop a spoofed edge attachment | DHCP snooping/DAI/port security. Required model not implemented; cannot claim faithful command or state behavior. |
| 5.8 | Open | Trace an AAA authorization denial | AAA policy state. Required model not implemented; cannot claim faithful command or state behavior. |
| 5.9 | Open | Choose a wireless security profile | WPA policy model. Required model not implemented; cannot claim faithful command or state behavior. |
| 5.10 | Open | Repair a secured WLAN assignment | wireless security config. Required model not implemented; cannot claim faithful command or state behavior. |
| 6.1 | Open | Choose an automation boundary | network change lifecycle. Required model not implemented; cannot claim faithful command or state behavior. |
| 6.2 | Open | Compare controller and device changes | management architecture. Required model not implemented; cannot claim faithful command or state behavior. |
| 6.3 | Open | Trace intent through control planes | SDN plane separation. Required model not implemented; cannot claim faithful command or state behavior. |
| 6.4 | Open | Review an AI troubleshooting claim | offline evidence/recommendation fixture. Required model not implemented; cannot claim faithful command or state behavior. |
| 6.5 | Open | Repair a REST change request | HTTP/REST metadata model. Required model not implemented; cannot claim faithful command or state behavior. |
| 6.6 | Open | Review a declarative device change | Ansible/Terraform model. Required model not implemented; cannot claim faithful command or state behavior. |
| 6.7 | Open | Repair a structured inventory document | JSON tree and schema model. Required model not implemented; cannot claim faithful command or state behavior. |

### LPIC-1 v5.0

| Objective | Current practical credit | Remaining recorded assignment | Capability still required / fidelity gap |
| --- | --- | --- | --- |
| 101.1 | Open | Identify a missing storage driver | virtual hardware inventory. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |
| 101.2 | Open | Trace a stalled boot | boot-event graph. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |
| 101.3 | Open | Recover the intended startup target | target dependency graph. Required model not implemented; cannot claim faithful command or state behavior. |
| 102.1 | Open | Plan disks for a growing log server | partition planner. Required model not implemented; cannot claim faithful command or state behavior. |
| 102.2 | Open | Repair an alternate boot entry | GRUB configuration model. Required model not implemented; cannot claim faithful command or state behavior. |
| 102.3 | Open | Diagnose a missing shared library | ELF dependency model. Required model not implemented; cannot claim faithful command or state behavior. |
| 102.4 | Partial practice · lpic1-package-install;lpic1-package-metadata;lpic1-package-payload;lpic1-package-handover | Resolve a Debian package conflict | No actual binary archives/downloads/maintainer scripts, unpack/configure intermediate states, general APT resolver/alternatives/providers/Conflicts, source-list editing, dpkg-reconfigure, uninstalled archive file search, real integrity/signature audit or conffile upgrade choices; no RPM/YUM/Zypper or full Debian conformance. |
| 102.5 | Partial practice · lpic1-rpm-provider;lpic1-rpm-epoch;lpic1-rpm-payload;lpic1-rpm-handover | Pin a working RPM update | Only the disclosed script-free noarch RPM subset. No rpm -V, binary extraction/execution, signature/integrity verification, dependency solver, configuration sidecars, YUM, Zypper or DNF. Actual normal Fedora development and installed87121151e82e flows passed; other-platform acceptance pending. |
| 102.6 | Open | Recognize a guest device mismatch | virtual guest metadata. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |
| 103.1 | Partial practice · shell-handover;shell-arguments;lpic1-file-navigation;lpic1-file-names | Diagnose a surprising shell value | Only disclosed command grammar; environment export, history, search path and broader command discovery remain absent. |
| 103.2 | Partial practice · lpic1-logs;shell-pipelines;shell-cut-inventory;shell-sort-manifest;shell-uniq-counts;shell-text-capstone | Normalize a noisy inventory stream | Broader tool suite (sed, wc, tr, formatters, digests/compressed inputs), complete option/locale behavior and Linux terminal integration remain absent; this is partial scripted filter practice. |
| 103.3 | Partial practice · lpic1-file-navigation;lpic1-file-names;lpic1-copy-move;lpic1-file-cleanup;lpic1-file-handover;lpic1-search-release-set;lpic1-search-incomplete-intake;lpic1-search-private-handover;lpic2-backup-selective-recovery;lpic2-backup-incremental-chain;lpic2-backup-snapshot-branch;lpic2-backup-recovery-handover | Reorganize a project without losing data | Cross-track tar-only recovery is explicit; cpio/dd, recursive directory copy/removal, terminal globs, real/compressed archives, size/time find, -exec/-delete and full GNU options remain absent. |
| 103.4 | Partial practice · shell-streams;shell-pipelines;shell-report;shell-loop | Separate report data from diagnostics | No xargs/tee/process substitution or full descriptor/process semantics. |
| 103.5 | Partial practice · lpic1-process-pause;lpic1-process-pending;lpic1-process-priority;lpic1-process-handover | Find a runaway process family | Fixed virtual metadata and supported positive-PID actions only. Jobs, fg/bg, nohup, spawning/nice, sorting, top/resource monitoring, groups, handlers, full scheduler/capabilities and complete Linux utility syntax remain absent. |
| 103.6 | Partial practice · lpic1-process-priority;lpic1-process-handover | Restore interactive workload priority | Fixed virtual metadata and supported positive-PID actions only. Jobs, fg/bg, nohup, spawning/nice, sorting, top/resource monitoring, groups, handlers, full scheduler/capabilities and complete Linux utility syntax remain absent. |
| 103.7 | Open | Select structured failures with regex | BRE/ERE implementation. Required model not implemented; cannot claim faithful command or state behavior. |
| 103.8 | Open | Repair a setting in a modal editor | vi input and buffer model. Required model not implemented; cannot claim faithful command or state behavior. |
| 104.1 | Open | Choose and create a suitable filesystem | partition/filesystem metadata. Required model not implemented; cannot claim faithful command or state behavior. |
| 104.2 | Open | Recover an inconsistent filesystem | filesystem integrity graph. Required model not implemented; cannot claim faithful command or state behavior. |
| 104.3 | Open | Repair a persistent removable mount | fstab/options/UUID model. Required model not implemented; cannot claim faithful command or state behavior. |
| 104.5 | Partial practice · lpic1-permissions;lpic1-traversal;lpic1-team-brief;lpic1-drop-directory;lpic1-update-existing;lpic1-retire-handover | Give a colleague least-privilege access | No ACL/SELinux or special-file/suid/sgid/sticky-bit practice; ordinary modeled users only. |
| 104.6 | Open | Repair a broken link deployment | inode and symbolic-link model. Required model not implemented; cannot claim faithful command or state behavior. |
| 104.7 | Open | Place and locate an application file | FHS tree and search indexes. Required model not implemented; cannot claim faithful command or state behavior. |
| 105.1 | Partial practice · shell-functions;shell-return;shell-bundle | Fix a startup environment override | No shell startup/PATH/aliases or export model; existing reusable-function and Bash-local scope lessons do not implement environment customization. |
| 105.2 | Partial practice · shell-validation;shell-loop;shell-eof;shell-batch-audit;shell-return;shell-bundle;shell-capture-preview;shell-capture-evidence;shell-capture-decision;shell-capture-handover;shell-case-interface;shell-case-routing;shell-case-publication | Build a guarded multi-file report | Mail, actual interpreter/shebang selection and script ownership/execution policy remain absent. No backticks, process substitution, arithmetic captures, custom IFS or complete POSIX/Bash behavior. |
| 106.1 | Open | Diagnose a display configuration mismatch | X configuration/log model. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |
| 106.2 | Open | Choose a remote desktop arrangement | session/desktop architecture model. Required model not implemented; cannot claim faithful command or state behavior. |
| 106.3 | Open | Choose accessible workstation settings | accessibility decision model. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |
| 107.1 | Partial practice · lpic1-account-onboarding;lpic1-account-membership;lpic1-account-retirement;lpic1-account-team-handover;lpic1-traversal | Onboard a user with correct group access | Bounded fixed local records only; no passwords, chage, authentication/PAM/login, live sessions, full NSS/defaults or complete utility option surface. Supplied templates only; no template maintenance/links/ACL/xattrs. Expiry is metadata only. |
| 107.2 |Partial practice · lpic1-cron-weekday-report;lpic1-cron-environment-output;lpic1-cron-access-retirement;lpic1-cron-calendar-handover;lpic1-at-report;lpic1-at-capture-repair;lpic1-at-owner-queue;lpic1-at-handover | Schedule a report without overlapping runs | Bounded UTC cron/one-off practice only; no anacron, at access files, systemd timers, overlap/locking, timezone/DST, mail/PAM/SELinux, spool administration or full Linux command/shell behavior. |
| 107.3 | Open | Repair a locale-sensitive report | locale/timezone/encoding model. Required model not implemented; cannot claim faithful command or state behavior. |
| 108.1 | Open | Diagnose clock drift and time sources | clock/NTP model. Required model not implemented; cannot claim faithful command or state behavior. |
| 108.2 | Partial practice · lpic2-service | Route and rotate useful logs | No date/priority filters, journal persistence/rotation/rescue, rsyslog or syslog-ng configuration. |
| 108.3 | Open | Trace a failed local notification | mail queue and aliases model. Required model not implemented; cannot claim faithful command or state behavior. |
| 108.4 | Open | Recover a paused print queue | CUPS queue model. Required model not implemented; cannot claim faithful command or state behavior. |
| 109.1 | Open | Choose a route for mixed IP traffic | IPv6 and transport model. Required model not implemented; cannot claim faithful command or state behavior. |
| 109.2 | Open | Make a client network repair persistent | NetworkManager/networkd configuration. Required model not implemented; cannot claim faithful command or state behavior. |
| 109.3 | Open | Localize a connectivity break | multi-hop packet/traceroute model. Required model not implemented; cannot claim faithful command or state behavior. |
| 109.4 | Open | Repair resolver search-order mistakes | NSS and resolver model. Required model not implemented; cannot claim faithful command or state behavior. |
| 110.1 | Open | Audit unwanted access and listeners | audit/process/socket metadata. Required model not implemented; cannot claim faithful command or state behavior. |
| 110.2 | Open | Restrict remote administration safely | sudo/SSH/access policy model. Required model not implemented; cannot claim faithful command or state behavior. |
| 110.3 | Open | Validate encrypted exchange and trust | key/certificate metadata model. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |

### LPIC-2 v4.5

| Objective | Current practical credit | Remaining recorded assignment | Capability still required / fidelity gap |
| --- | --- | --- | --- |
| 200.1 | Open | Find the actual resource bottleneck | CPU/memory/IO time-series model. Required model not implemented; cannot claim faithful command or state behavior. |
| 200.2 | Open | Forecast the next capacity upgrade | capacity trend model. Required model not implemented; cannot claim faithful command or state behavior. |
| 201.1 | Open | Choose a compatible kernel component | kernel/module metadata. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |
| 201.2 | Open | Plan a reproducible kernel build | build configuration dependency graph. Required model not implemented; cannot claim faithful command or state behavior. |
| 201.3 | Open | Recover a failed module load | module/udev/runtime parameter model. Required model not implemented; cannot claim faithful command or state behavior. |
| 202.1 | Partial practice · lpic2-service;lpic2-unit-loaded-recovery;lpic2-unit-preparation-order;lpic2-unit-optional-wants;lpic2-unit-target-handover | Restore service dependencies and boot behavior | No SysV/LSB administration, real boot/kernel/process/readiness or full systemd grammar, propagation and utility options. |
| 202.2 | Open | Recover a broken root boot path | recovery environment and initramfs model. Required model not implemented; cannot claim faithful command or state behavior. |
| 202.3 | Open | Repair a UEFI boot chain | bootloader/firmware state model. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |
| 203.1 | Partial practice · lpic2-storage;lpic2-mount-identity;lpic2-mount-policy;lpic2-mount-manual;lpic2-mount-handover | Recover a data volume with the right mount | No swap, systemd mount units, real boot/kernel, full mount/fstab/blkid syntax, filesystem maintenance or real device operations. Fixed non-nested mountpoints and supplied inventory; numeric dump/pass fields are stored only. |
| 203.2 | Open | Diagnose inode exhaustion | filesystem-specific maintenance/integrity models. The capacity/mount seed does not teach ext-family, Btrfs, XFS or ZFS maintenance; no coverage is claimed. |
| 203.3 | Open | Tune filesystem behavior for a workload | filesystem option/automount model. Required model not implemented; cannot claim faithful command or state behavior. |
| 204.1 | Open | Replace a failed mirror member | RAID degradation/rebuild model. Required model not implemented; cannot claim faithful command or state behavior. |
| 204.2 | Open | Diagnose a storage path mismatch | iSCSI/device mapping model. Required model not implemented; cannot claim faithful command or state behavior. |
| 204.3 | Open | Extend a volume while preserving data | LVM extent allocator. Required model not implemented; cannot claim faithful command or state behavior. |
| 205.1 | Open | Restore dual-stack server connectivity | persistent multi-interface IP model. Required model not implemented; cannot claim faithful command or state behavior. |
| 205.2 | Open | Repair a bonded VLAN uplink | bond/VLAN/bridge state machine. Required model not implemented; cannot claim faithful command or state behavior. |
| 205.3 | Open | Trace intermittent network failures | packet counters and route diagnostics. Required model not implemented; cannot claim faithful command or state behavior. |
| 206.1 | Open | Stage and verify a source installation | build/install manifest model. Required model not implemented; cannot claim faithful command or state behavior. |
| 206.2 | Partial practice · lpic2-backup-selective-recovery;lpic2-backup-incremental-chain;lpic2-backup-snapshot-branch;lpic2-backup-recovery-handover | Restore a consistent incremental backup | Registered manual/selective/chain/branch/handover recovery only. Directory selection, products/media, backup shell automation, dd, tape/mt and rsync remain Open. No real tar bytes, compression, links/devices, checksums or physical filesystem validation; final native/candidate receipts are separate. |
| 206.3 | Open | Communicate a planned maintenance window | notification audience/time model. Required model not implemented; cannot claim faithful command or state behavior. |
| 207.1 | Partial practice · lpic2-dns;lpic2-dns-dualstack;lpic2-dns-aliases;lpic2-dns-mail;lpic2-dns-rrsets;lpic2-dns-secondary;lpic2-dns-handover | Restore recursive versus authoritative behavior | No full BIND configuration/file locations, recursive/cache service, logging, real utilities or transfer protocol; this is zone-reload concept practice only. |
| 207.2 | Partial practice · lpic2-dns;lpic2-dns-dualstack;lpic2-dns-aliases;lpic2-dns-mail;lpic2-dns-rrsets;lpic2-dns-secondary;lpic2-dns-handover;lpic2-dns-reverse-investigation;lpic2-dns-reverse-ipv6;lpic2-dns-reverse-handover | Repair forward and reverse zone consistency | No full BIND configuration/utilities, zone-file locations, omitted/relative owners, directives or full SOA grammar. Reverse tasks use one fixed authoritative origin; external canonicality and paired forward/reverse consistency are not checked. Root hints, delegation/glue/RFC2317, wire transfers, caching/recursion and DNS security remain absent; TTL choices are accepted but not assessed. |
| 207.3 | Open | Restrict DNS transfer and recursion | DNS ACL/DNSSEC metadata model. Required model not implemented; cannot claim faithful command or state behavior. |
| 208.1 | Partial practice · lpic2-web-routing;lpic2-web-pages;lpic2-web-access;lpic2-web-handover | Diagnose an authenticated site using request and error logs | No Apache installation/processes, request/error logs, client identity/IP authentication, .htaccess, scripting modules, concurrency/resource tuning, dedicated-IP virtual hosts, configurable Redirect statements, HTTPS, proxying or complete Apache utility/configuration syntax. Native actions model only bounded static serving, not a real server. |
| 208.2 | Open | Repair a TLS identity chain | certificate/handshake metadata model. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |
| 208.3 | Open | Fix a proxy access-policy leak | HTTP cache and proxy ACL model. Required model not implemented; cannot claim faithful command or state behavior. |
| 208.4 | Open | Recover a reverse proxy backend | upstream routing model. Required model not implemented; cannot claim faithful command or state behavior. |
| 209.1 | Open | Repair a team Samba share | SMB identities/share permissions model. Required model not implemented; cannot claim faithful command or state behavior. |
| 209.2 | Partial practice · lpic2-nfs-reference;lpic2-nfs-publication;lpic2-nfs-handover;lpic2-nfs-capstone | Restrict an NFS export correctly | Full objective remains incomplete: no real NFS/RPC/services or Linux utility emulation, no TCP Wrappers/NFSv4/Kerberos/locking/cache/durability/security-stack configuration; supplied identities and immutable regular-file metadata only. |
| 210.1 | Open | Repair an exhausted DHCP scope | lease and relay state machine. Required model not implemented; cannot claim faithful command or state behavior. |
| 210.2 | Open | Diagnose authentication stack order | PAM control flow model. Required model not implemented; cannot claim faithful command or state behavior. |
| 210.3 | Open | Fix an LDAP client search base | LDAP query/directory model. Required model not implemented; cannot claim faithful command or state behavior. |
| 210.4 | Open | Delegate directory access safely | LDAP schema and ACL model. Required model not implemented; cannot claim faithful command or state behavior. |
| 211.1 | Open | Trace a mail routing loop | SMTP routing/queue model. Required model not implemented; cannot claim faithful command or state behavior. |
| 211.2 | Open | Filter and deliver local mail reliably | mail rules/delivery model. Required model not implemented; cannot claim faithful command or state behavior. |
| 211.3 | Open | Repair mailbox authentication and TLS | IMAP/POP state model. Required model not implemented; cannot claim faithful command or state behavior. |
| 212.1 | Open | Repair forwarding and firewall rules | packet filter/NAT routing model. Required model not implemented; cannot claim faithful command or state behavior. |
| 212.2 | Open | Constrain a file-transfer account | FTP identity/chroot model. Required model not implemented; cannot claim faithful command or state behavior. |
| 212.3 | Open | Recover key-based administration | SSH key/config/trust model. Required model not implemented; cannot claim faithful command or state behavior. |
| 212.4 | Open | Assess exposed services from evidence | audit/scanner evidence model. Required model not implemented; cannot claim faithful command or state behavior. |
| 212.5 | Open | Repair a VPN route and peer policy | VPN peer/tunnel metadata model. Evidence/configuration reasoning can be simulated; physical, kernel, cryptographic or assistive-device operation cannot be validated here. |

## Delivered unit inventory

These48 units, and only their182 registered authored scenarios, count as
implemented inventory. The presence of a unit does not mean its entire related
objective or wider course stage is complete. Exact per-lab prerequisites remain
in the versioned scenario packs; unit prerequisites are in units-v1.json.

| Track | Unit | Registered lab IDs |
| --- | --- | --- |
| CCNA | IPv4 addressing and diagnosis (`ipv4-foundations`) | `ccna-ipv4`, `ccna-vlsm`, `ccna-gateway` |
| CCNA | VLAN groups, trunks and handover (`vlan-data-foundations`) | `ccna-vlan-access`, `ccna-vlan-trunk`, `ccna-vlan-native`, `ccna-vlan-handover` |
| CCNA | Ethernet learning, aging and recovery (`ethernet-learning`) | `ccna-mac-learn-paths`, `ccna-mac-stale-location`, `ccna-mac-source-refresh`, `ccna-mac-handover` |
| CCNA | IPv4 routes, return paths and resilient handover (`ipv4-static-foundations`) | `ccna-ipv4-network-route`, `ccna-ipv4-return-route`, `ccna-ipv4-specific-route`, `ccna-ipv4-host-exception`, `ccna-ipv4-floating-route`, `ccna-ipv4-branch-handover` |
| CCNA | IPv6 links, routes and recovery (`ipv6-static-foundations`) | `ccna-ipv6-neighbor`, `ccna-ipv6-workstation`, `ccna-ipv6-scope`, `ccna-ipv6-specific-route`, `ccna-ipv6-backup`, `ccna-ipv6-handover` |
| LPIC-1 | Files, paths and safe handovers (`linux-file-foundations`) | `lpic1-file-navigation`, `lpic1-file-names`, `lpic1-copy-move`, `lpic1-file-cleanup`, `lpic1-creation-mask`, `lpic1-file-handover` |
| LPIC-1 | Access policy and safe file changes (`linux-access-policy`) | `lpic1-team-brief`, `lpic1-drop-directory`, `lpic1-update-existing`, `lpic1-retire-handover` |
| LPIC-1 | File access and investigation (`linux-file-access`) | `lpic1-permissions`, `lpic1-traversal`, `lpic1-logs` |
| LPIC-1 | Search, investigate and hand over files (`linux-file-search`) | `lpic1-search-release-set`, `lpic1-search-incomplete-intake`, `lpic1-search-private-handover` |
| LPIC-2 | Service, DNS and storage recovery (`linux-service-recovery`) | `lpic2-service`, `lpic2-dns`, `lpic2-storage` |
| Bash/Shell | Reliable shell foundations (`shell-foundations`) | `shell-handover`, `shell-arguments`, `shell-input`, `shell-validation`, `shell-report`, `shell-loop`, `shell-capstone` |
| Bash/Shell | Streams, pipelines and resilient batches (`shell-composition`) | `shell-streams`, `shell-pipelines`, `shell-eof`, `shell-batch-audit` |
| Bash/Shell | Reusable functions and safe composition (`shell-reusable`) | `shell-functions`, `shell-return`, `shell-shift`, `shell-bundle` |
| Bash/Shell | Bash scope and safe helper design (`shell-scope`) | `shell-local-heading`, `shell-scope-review`, `shell-scope-capstone` |
| Bash/Shell | Text records and reliable audits (`shell-text`) | `shell-cut-inventory`, `shell-sort-manifest`, `shell-uniq-counts`, `shell-text-capstone` |
| Bash/Shell | Counters, totals and safe publication (`shell-counts`) | `shell-number-records`, `shell-order-total`, `shell-pack-items`, `shell-count-capstone` |
| Bash/Shell | Filename selection and safe batch reports (`shell-file-batches`) | `shell-file-manifest`, `shell-collect-errors`, `shell-file-batch-capstone` |
| LPIC-2 | Forward DNS: records, repairs and publication (`dns-forward`) | `lpic2-dns-dualstack`, `lpic2-dns-aliases`, `lpic2-dns-mail`, `lpic2-dns-rrsets`, `lpic2-dns-secondary`, `lpic2-dns-handover` |
| LPIC-2 | Reverse names and DNS handovers (`dns-reverse`) | `lpic2-dns-reverse-investigation`, `lpic2-dns-reverse-ipv6`, `lpic2-dns-reverse-handover` |
| Bash/Shell | Bash lists, snapshots and safe handovers (`shell-arrays`) | `shell-array-intake`, `shell-array-priority`, `shell-array-package`, `shell-array-handover` |
| LPIC-1 | Process investigation and priority (`linux-processes`) | `lpic1-process-pause`, `lpic1-process-pending`, `lpic1-process-priority`, `lpic1-process-handover` |
| CCNA | LLDP neighbors, inventory and handover (`lldp-discovery`) | `ccna-lldp-receive`, `ccna-lldp-inventory`, `ccna-lldp-transmit-policy`, `ccna-lldp-handover` |
| LPIC-2 | Persistent mounts and stable volume identity (`persistent-mounts`) | `lpic2-mount-identity`, `lpic2-mount-policy`, `lpic2-mount-manual`, `lpic2-mount-handover` |
| Bash/Shell | Command output, reliable decisions and report previews (`shell-command-capture`) | `shell-capture-preview`, `shell-capture-evidence`, `shell-capture-decision`, `shell-capture-handover` |
| CCNA | IPv4 access policies without collateral blocking (`ipv4-access-policy`) | `ccna-acl-source-policy`, `ccna-acl-service-order`, `ccna-acl-direction-repair`, `ccna-acl-policy-handover` |
| LPIC-1 | Local accounts and careful team handovers (`local-account-lifecycle`) | `lpic1-account-onboarding`, `lpic1-account-membership`, `lpic1-account-retirement`, `lpic1-account-team-handover` |
| CCNA | Point-to-point OSPFv2 compatibility and route costs (`ospf-p2p-foundations`) | `ccna-ospf-router-identity`, `ccna-ospf-p2p-parameters`, `ccna-ospf-directed-ecmp`, `ccna-ospf-passive-handover` |
| LPIC-2 | Service definitions, dependencies and target handover (`linux-unit-manager-foundations`) | `lpic2-unit-loaded-recovery`, `lpic2-unit-preparation-order`, `lpic2-unit-optional-wants`, `lpic2-unit-target-handover` |
| Bash/Shell | Clear dispatch and safe publication (`shell-case-dispatch`) | `shell-case-interface`, `shell-case-routing`, `shell-case-publication` |
| Bash/Shell | Reliable short-option tools and independent jobs (`shell-option-interfaces`) | `shell-options-report`, `shell-options-jobs`, `shell-options-publication` |
| CCNA | DHCP client leases, relays and safe handover (`dhcpv4-client-foundations`) | `ccna-dhcp-held-options`, `ccna-dhcp-relay-scope`, `ccna-dhcp-lease-handover` |
| LPIC-1 | Debian packages: inspect, install, repair and retire (`debian-package-administration`) | `lpic1-package-install`, `lpic1-package-metadata`, `lpic1-package-payload`, `lpic1-package-handover` |
| CCNA | Spanning-tree roots and stable paths (`spanning-tree-steady-state`) | `ccna-stp-root`, `ccna-stp-local-cost`, `ccna-stp-parallel`, `ccna-stp-handover` |
| Bash/Shell | Reliable text input and careful publication (`shell-safe-automation`) | `shell-preserve-lines`, `shell-pipeline-status`, `shell-safe-handover` |
| LPIC-2 | Static sites: names, pages, access and handover (`static-web-services`) | `lpic2-web-routing`, `lpic2-web-pages`, `lpic2-web-access`, `lpic2-web-handover` |
| CCNA | Basic NAT address mapping (`basic-nat`) | `ccna-nat-static`, `ccna-nat-pool`, `ccna-nat-handover` |
| LPIC-1 | RPM records, providers, payloads and handover (`rpm-package-investigation`) | `lpic1-rpm-provider`, `lpic1-rpm-epoch`, `lpic1-rpm-payload`, `lpic1-rpm-handover` |
| Bash/Shell | Build and dispatch safe command requests (`shell-command-construction`) | `shell-command-options`, `shell-command-dispatch`, `shell-command-plan` |
| LPIC-2 | Shared files: export policy, clients and safe handover (`nfs-network-file-sharing`) | `lpic2-nfs-reference`, `lpic2-nfs-publication`, `lpic2-nfs-handover`, `lpic2-nfs-capstone` |

| CCNA | LACP: form, diagnose and safely migrate a bundle (`ccna-lacp`) | `ccna-lacp-initiation`, `ccna-lacp-member-service`, `ccna-lacp-resilient-handover` |
| Bash/Shell | Optional inputs and intentional empty values (`shell-optional-inputs`) | `shell-optional-policy`, `shell-optional-report`, `shell-optional-handover` |
| LPIC-1 | Recurring jobs: reports, access and safe handover (`linux-cron-foundations`) | `lpic1-cron-weekday-report`, `lpic1-cron-environment-output`, `lpic1-cron-access-retirement`, `lpic1-cron-calendar-handover` |

| LPIC-2 | Backups: inspect, recover and preserve history (`backup-and-recovery`) | `lpic2-backup-selective-recovery`, `lpic2-backup-incremental-chain`, `lpic2-backup-snapshot-branch`, `lpic2-backup-recovery-handover` |

| Bash/Shell | Numeric input and safe decisions (`shell-numeric-validation`) | `shell-numeric-retry`, `shell-numeric-intake`, `shell-numeric-publication` |

| CCNA | Physical uplinks: choose, investigate and hand over (`physical-uplink-diagnosis`) | `ccna-physical-media-span`, `ccna-physical-duplex-diagnosis`, `ccna-physical-counter-baseline`, `ccna-physical-uplink-handover` |
| CCNA | Private office planning (`private-office-planning`) | `ccna-private-office-plan` |

| LPIC-1 | One-off jobs: capture, inspect and hand over (`linux-one-off-jobs`) | `lpic1-at-report`, `lpic1-at-capture-repair`, `lpic1-at-owner-queue`, `lpic1-at-handover` |
| Bash/Shell | Loop control and local recovery (`shell-loop-recovery`) | `shell-loop-intake`, `shell-loop-fallback` |

## Bash/Shell remaining path

The57 source-integrated scripts span16 units. The new numeric unit has passed
private full/native and final Main full/static/coverage gates. Available171
candidate065032d has installed numeric101 and NFS cold/startup acceptance; it
does not contain the eleven newer assignments. The source-integrated scripting path covers bounded indexed arrays,
command-output capture, getopts, case dispatch, command-list construction,
read-only literal fallback/alternate expansion and optional-input projects,
C-locale cut/sort/uniq work, explicit pipefail and bounded case-class numeric
validation, malformed-record recovery, whole-plan publication and own-scope loop recovery with unread-input handoff and independent first-ready file jobs. Older Stage4 wording calling
command construction planned is stale; those three authored tasks are delivered.
The original agreed scope still requires the following distinct capabilities
and projects, with prerequisite teaching before assessment:

| Required area | Remaining implementation / authored work |
| --- | --- |
| Quoting and expansion | Read-only literal fallback/alternate forms and three optional-input policy/report/handover projects are implemented. Broader nested, assignment/error, substring/length/pattern operators remain unsupported; add only capabilities required by a reviewed substantive task with faithful expansion tests. |
| Arguments and interfaces | Robust long-option/input validation projects beyond the bounded getopts subset; streamed row validation and meaningful handling of malformed inputs. |
| Functions and scope | Applied state/return/error handling across the remaining projects; broader function-call redirection/pipeline forms remain explicit subset limits unless a task needs them. Existing scope lessons are implemented. |
| Conditions and loop recovery | Two implemented assignments teach own-scope break/continue, unread-input handoff, first-success selection and independent job failures with22 graded cases. Broader conditional/regex forms and project-specific retry/lock behavior remain required where used; caller-loop and pipeline control escape remain unsupported. |
| Structured Bash | Associative maps, supported conditional/regex constructs, array operations beyond whole dense-list replacement, with faithful parsing, bounds and new practical assignments. |
| Text processing | Regex, deliberate sed/awk or equivalent documented utility capabilities; snapshot comparison, malformed-row recovery, log-analysis projects. Current literal filters and byte-field utilities do not cover these. |
| Debugging and errors | Actual bounded tracing, useful error propagation, traps/cleanup and recovery; debugging and repair assignments, not only deliberately incorrect scripts. |
| Safe file operations | Virtual directory/path semantics, safe temporary-file/rename behavior, validated mutation, deterministic clock and lock/retry behavior where the project depends on it. |
| Substantial projects | Dry-run cleanup planning, verified backup/restore, idempotent configuration rollout, resumable batch processing; combine report generation/log analysis/input validation and preserve unrelated data. |

Portable shell behavior and Bash-specific features must stay explicit. Current
[SHELL_SUBSET.md](SHELL_SUBSET.md) and per-unit references define actual behavior;
Full Bash conformance is not an agreed requirement: an unsupported construct
is not automatically a new course objective. The practical skills and projects
above remain required, with justified limits. New capabilities require
parsing/evaluation, deterministic adverse cases,
controlled development comparisons and honest saved-result migration. A capability
or test-only fixture is not a delivered lesson.

## Current implementation order

1. The normal182/evaluator13 milestone passes inclusive2,683 Rust tests,
   all-target Clippy,348 Python checks and actual Fedora82-capture native flows.
   Preserve its original180 definitions,55 scripts/420 cases/55 examples and all
   genuine evaluator9–12 evidence. Installed171 candidate acceptance is separate.
2. Preserve the genuine evaluator13 baseline observed and independently sealed
   in SHELL_EVALUATOR13_BASELINE.md. Before any14 advance, add actual current-versus-
   genuine13 regression tests; a sealed baseline alone is not migration acceptance.
3. Integrate the reviewed four-task LVM unit after accepted normal182 and its
   sealed baseline, then the four-task NTP unit, with source-aware progress,
   runtime/alternative/negative checks and actual native flows. Their pure models
   are tested; source-only units currently earn no catalog or objective credit.
4. Validate controlled ERE reference comparisons and cooperative shell integration
   before authoring meaningful regex/text-processing projects. Keep unsupported
   syntax honest and all host reference work restricted to fixed disposable data.
5. Complete the fifth-backup directory-selection ArtifactSet/content/progress/
   normal-native gates against accepted shared controls. Test the independently
   reviewed HSRP and inode/link foundations before dependent content.
6. Continue every remaining network/Linux/scripting assignment and capability in
   the granular ledger. Current reference fingerprints and future CCNA separation
   remain as recorded; no arbitrary catalog quota or routine approval gate.

Full curriculum acceptance stays separate from Windows/macOS desktop,
physical-device/screen-reader and novice acceptance. No release publication.
