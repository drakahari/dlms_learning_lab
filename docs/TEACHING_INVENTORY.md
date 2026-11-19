# Existing-lab teaching rollout inventory

Derived from the actual review library: preserved normal catalog plus accepted and continuation units.
Statuses record lesson-specific source/content and UI-path review of all three screens,
not merely shared-renderer inheritance. Compiled rendering and representative native
evidence are recorded in TEACHING_ROLLOUT_VERIFICATION.md, GATEWAY_BATCH.md and CONTINUATION.md. This is neither
curriculum-completion nor independent novice-comprehension acceptance.

**543 labs:** CCNA 129, LPIC-1 158, LPIC-2 176, Bash/Shell 80.

**Statuses:** 538 updated; 5 already meets; 0 pending; 0 blocked.

[Shared standard](TEACHING_PRESENTATION.md). Complete per-screen review notes are in
the linked UI-only JSON entries; CSV contains every full note and original source path.

| Track | Lab ID | Assignment | Status | Review details |
| --- | --- | --- | --- | --- |
| CCNA | `ccna-ipv4` | Read an IPv4 address | already meets | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-vlsm` | Give each team a subnet | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-gateway` | Reconnect the office client | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv6-neighbor` | Add an IPv6 workstation to a team link | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv6-workstation` | Repair an IPv6 workstation’s path | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv6-scope` | Put a link-local next hop on the right link | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv6-specific-route` | Find the route that overrides a working default | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv6-backup` | Make the backup route work before the outage | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv6-handover` | Recover an IPv6 branch handover | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-vlan-access` | Reconnect a workstation to its workgroup | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-vlan-trunk` | Carry two groups between offices | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-vlan-native` | Align the meaning of untagged traffic | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-vlan-handover` | Hand over three working, separate groups | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv4-network-route` | Connect the branch with a network route | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv4-return-route` | Repair the journey back | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv4-specific-route` | Repair the more-specific detour | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv4-host-exception` | Send one host through a different router | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv4-floating-route` | Restore the primary and floating backup | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv4-branch-handover` | Complete the resilient branch handover | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-lldp-receive` | Restore both discovery directions | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-lldp-inventory` | Refresh an advertised inventory change | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-lldp-transmit-policy` | Keep discovery inbound on the visitor link | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-lldp-handover` | Complete a discovery handover | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-acl-source-policy` | Let the right workstation reach the vault | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-acl-service-order` | Repair a service exception hidden by a broad permit | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-acl-direction-repair` | Find the filter facing the wrong way | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-acl-policy-handover` | Handover the archive and portal policies | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ospf-router-identity` | Recover a branch identity and transit process | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ospf-p2p-parameters` | Repair a P2P parameter handover | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ospf-directed-ecmp` | Restore two equal-cost first hops | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ospf-passive-handover` | Hand over a passive user LAN and both sites | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-mac-learn-paths` | Build directed paths from both sources | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-mac-stale-location` | Repair an older location without losing another group | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-mac-source-refresh` | Refresh the quiet source and prove expiry | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-mac-handover` | Hand over two learned groups across three switches | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-dhcp-held-options` | Replace held options without disturbing the monitor | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-dhcp-relay-scope` | Repair a relay and the selected client subnet | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-dhcp-lease-handover` | Hand over a small lease pool safely | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-stp-root` | Choose the intended root without cutting links | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-stp-local-cost` | Repair a rootward path using local costs | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-stp-parallel` | Prefer the second of two parallel uplinks | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-stp-handover` | Hand over a redundant four-bridge network | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-nat-static` | Give a server a stable translated address | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-nat-pool` | Repair translation eligibility and capacity | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-nat-handover` | Retire an old mapping and hand over the pool | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-lacp-initiation` | Bring two switch links into one working group | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-lacp-member-service` | Restore members and the missing data VLAN | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-lacp-resilient-handover` | Move a bundle without losing its spare path | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-physical-media-span` | Select a cable and matching interfaces for a long route | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-physical-duplex-diagnosis` | Repair a link that is up but disagrees about duplex | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-physical-counter-baseline` | Find a receive problem using fresh counter changes | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-physical-uplink-handover` | Commission a replacement uplink and retire the old path | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-private-office-plan` | Choose private space for a new office | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ntp-client-chain` | Build a time-service chain for two branches | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ntp-cycle-repair` | Recover a configured dependency cycle | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ntp-asymmetric-delivery` | Diagnose a request that never receives a reply | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ntp-policy-handover` | Hand over sources without changing downstream contracts | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-magic-blocks` | Find a subnet’s block size | already meets | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-magic-boundaries` | Find every address boundary | already meets | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-magic-cross-octet` | Follow a subnet across octets | already meets | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-magic-octet-aligned` | Handle an octet-aligned prefix | already meets | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-hsrp-virtual-gateway` | Replace fragile real-router gateways with a resilient virtual gateway | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-hsrp-incumbent-return` | Recover the preferred router after an incumbent takes over | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-hsrp-wan-tracking` | Make an upstream-link failure trigger a usable takeover | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-hsrp-resilient-handover` | Hand over a gateway while preserving protected client settings | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-syslog-local-remote` | Keep local detail without flooding Operations | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-syslog-export-path` | Recover the usable export path | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-syslog-facility-routing` | Route categories without confusing subsystem labels | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-syslog-policy-handover` | Hand over Operations without breaking Audit | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-qos-mark-boundary` | Mark traffic at a boundary without erasing other labels | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-qos-queue-bursts` | Protect interactive packets when a shared link is busy | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-qos-trusted-handover` | Hand over a trusted queue policy without starving other traffic | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-transport-port-pairing` | Pair clients with TCP and UDP listeners | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-transport-return-policy` | Repair a blocked TCP return path | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-security-identity` | Bind an approved source before the first arrival | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-security-observe` | Count rejected sources without disconnecting the desk | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-security-recover` | Replace a retired identity and recover the desk | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-dhcp-server-boundary` | Keep DHCP replies on the approved server path | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-dhcp-client-mac-check` | Compare Ethernet and DHCP client MAC fields | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-arp-sender-binding` | Check address claims before forwarding ARP | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-arp-static-policy` | Keep a static workstation working without trusting its port | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-arp-field-consistency` | Diagnose inconsistent Ethernet and ARP fields | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-management-protocols` | Keep remote administration on SSH version 2 | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-management-source-policy` | Restrict management sources without locking out operators | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-local-account-setup` | Set up named local accounts and their access levels | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-local-account-handover` | Hand over privileged access without leaving an old account | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-aaa-method-recovery` | Restore central authentication and a working local fallback | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-aaa-authorization-recovery` | Repair authorization after a successful central login | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ipv6-modified-eui64` | Connect a workstation using modified EUI-64 | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-device-path-patching` | Connect endpoints through switches and a router | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-device-path-broadcast-boundary` | Keep a router boundary from becoming a bridge | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-json-port-records` | Repair JSON values and endpoint groups | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-json-inventory-handover` | Recover a nested multi-switch inventory | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-wireless-neighbour-channels` | Give an access point its own channel space | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-wireless-multisurvey-plan` | Make one channel plan work across two surveys | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-wlan-ssid-repair` | Restore the right Wi-Fi network name | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-wlan-wpa2-personal` | Repair a WPA2-Personal staff WLAN | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-wlan-wpa3-handover` | Verify a WPA3-only client handover | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-wlc-campus-forwarding` | Restore a controller-forwarded campus WLAN | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-flexconnect-local-vlan` | Restore branch-local Wi-Fi forwarding | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-snmp-link-monitor` | Monitor unexpected uplink failures | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-snmp-counter-monitor` | Repair traffic-rate monitoring | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-rest-change-request` | Repair an authorized network change request | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-rest-profile-handover` | Hand over a controller profile without collateral changes | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-tftp-recovery-copy` | Stage a recovery file without replacing the current configuration | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-ftp-archive-channel` | Recover an FTP archive when login works but data does not | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-transfer-file-handover` | Hand over an original archive and a staged replacement | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-collapsed-campus-uplinks` | Restore independent uplinks in a two-tier campus | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-distribution-core-paths` | Repair building and core paths in a three-tier campus | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-leaf-spine-rack-expansion` | Commission a rack without losing fabric alternatives | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-virtual-host-resilience` | Keep virtual workloads ready after a host failure | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-virtual-switch-uplink` | Reconnect virtual NICs across two physical hosts | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-vrf-overlap-repair` | Keep overlapping tenants in separate routing tables | updated | [Three-screen notes](../scenarios/presentation/ccna.json) |
| CCNA | `ccna-poe-compatible-ports` | Power devices from compatible Ethernet ports | updated | [Three-screen notes](../scenarios/presentation/ccna_poe.json) |
| CCNA | `ccna-poe-budget-recovery` | Restore Wi-Fi power without losing the phone | updated | [Three-screen notes](../scenarios/presentation/ccna_poe.json) |
| CCNA | `ccna-management-change-evidence` | Triage a suspicious network change | updated | [Three-screen notes](../scenarios/presentation/ccna_security_investigation.json) |
| CCNA | `ccna-facility-access-plan` | Choose a proportionate protection control | updated | [Three-screen notes](../scenarios/presentation/ccna_security_investigation.json) |
| CCNA | `ccna-credential-policy-repair` | Repair a credential policy | updated | [Three-screen notes](../scenarios/presentation/ccna_credentials_vpn.json) |
| CCNA | `ccna-ipsec-design-review` | Choose a suitable VPN design | updated | [Three-screen notes](../scenarios/presentation/ccna_credentials_vpn.json) |
| CCNA | `ccna-controller-pilot-rollout` | Stage a controller rollout with a pilot port | updated | [Three-screen notes](../scenarios/presentation/ccna_staged_network.json) |
| CCNA | `ccna-controller-intent-path` | Trace intent through installed policy and transport | updated | [Three-screen notes](../scenarios/presentation/ccna_staged_network.json) |
| CCNA | `ccna-review-ai-operations-claim` | Review an AI troubleshooting claim | updated | [Three-screen notes](../scenarios/presentation/ccna_network_evidence_review.json) |
| CCNA | `ccna-review-declarative-change` | Review a declarative device change | updated | [Three-screen notes](../scenarios/presentation/ccna_network_evidence_review.json) |
| CCNA | `ccna-diverse-branch-paths` | Choose independent WAN paths for a branch | updated | [Three-screen notes](../scenarios/presentation/ccna_site_connectivity.json) |
| CCNA | `ccna-local-and-hosted-connectivity` | Keep local and hosted office services reachable | updated | [Three-screen notes](../scenarios/presentation/ccna_site_connectivity.json) |
| CCNA | `ccna-default-access-membership` | Move a new desk out of default VLAN 1 | updated | [Three-screen notes](../scenarios/presentation/ccna_vlan_gateway.json) |
| CCNA | `ccna-switched-gateway-repair` | Restore IPv4 routing between separate VLANs | updated | [Three-screen notes](../scenarios/presentation/ccna_vlan_gateway.json) |
| CCNA | `ccna-route-reading-backup` | Restore a learned route while keeping its backup | updated | [Three-screen notes](../scenarios/presentation/ccna_route_reading.json) |
| CCNA | `ccna-ospf-broadcast-handover` | Plan designated roles through a router outage | updated | [Three-screen notes](../scenarios/presentation/ccna_ospf_broadcast.json) |
| LPIC-1 | `lpic1-permissions` | Share a report safely | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-traversal` | Unlock the project directory | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-logs` | Find the failing requests | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-file-navigation` | Find a project and prepare its handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-file-names` | Package notes with spaces in their names | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-copy-move` | Keep a snapshot and process a receipt | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-file-cleanup` | Clean a staging folder without losing the release | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-creation-mask` | Create a private working area by default | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-file-handover` | Prepare a safe project handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-team-brief` | Share a brief with the review group | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-drop-directory` | Configure a drop folder without browsing | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-update-existing` | Update files inside a read-only directory | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-retire-handover` | Retire a read-only handover safely | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-process-pause` | Pause the right importer | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-process-pending` | End the paused archiver | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-process-priority` | Let the batch copy yield | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-process-handover` | Complete a safe process handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-account-onboarding` | Create an account with the right home template | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-account-membership` | Repair team membership without changing unrelated ownership | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-account-retirement` | Retire an account while preserving the right evidence | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-account-team-handover` | Handover a team without losing numeric ownership | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-search-release-set` | Build the right release evidence set | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-search-incomplete-intake` | Recover an incomplete intake investigation | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-search-private-handover` | Complete a private support handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-package-install` | Install a tool without disturbing its neighbours | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-package-metadata` | Refresh a stale list and meet a new dependency | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-package-payload` | Repair damaged files without losing local settings | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-package-handover` | Retire two tools with different retention policies | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-rpm-provider` | Find a provider beyond its package name | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-rpm-epoch` | Update a client when epoch changes the order | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-rpm-payload` | Repair recorded payload without upgrading it | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-rpm-handover` | Hand an API over before retiring its provider | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-cron-weekday-report` | Publish a weekday report on schedule | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-cron-environment-output` | Repair a noninteractive report and its output | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-cron-access-retirement` | Delegate scheduling and retire an installed job | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-cron-calendar-handover` | Hand over a system report without losing peer jobs | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-at-report` | Queue and verify a one-time report | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-at-capture-repair` | Repair a captured job without losing live updates | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-at-owner-queue` | Retire an obsolete job and preserve another owner’s work | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-at-handover` | Hand over an ordered two-job report | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-disk-safe-gap` | Prepare an export area beside retained evidence | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-disk-hints-versus-content` | Finish a correctly allocated handoff | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-disk-eligible-media` | Build a layout on eligible media | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-disk-delivery` | Deliver prepared storage without disturbing retained data | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-links-identity` | Choose a shared name and an independent snapshot | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-links-relative` | Repair a shortcut from the right directory | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-links-move` | Move a receipt without losing its identity | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-links-handover` | Publish a new release while retaining the old one | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-repo-visibility` | Make package metadata visible without installing anything | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-repo-provenance` | Repair a source address without trusting the wrong index | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-repo-offline-evidence` | Use an older cached index honestly while a source is offline | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-repo-handover` | Hand over current package evidence while retaining a held source | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-host-address` | Repair a Linux interface without disturbing management | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-host-routing` | Repair Linux default preference and a misleading host route | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-host-resolver` | Separate a working IP path from a wrong client resolver | updated | [Three-screen notes](../scenarios/presentation/lpic1_direct_dns.json) |
| LPIC-1 | `lpic1-host-handover` | Hand over application networking while preserving management | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-syslog-threshold` | Keep warnings and urgent events, without routine noise | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-syslog-exact` | Separate routine service information from urgent alerts | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-syslog-privacy` | Keep private authentication out of the general log | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-syslog-handover` | Repair a logging policy without leaks, noise or duplicates | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-startup-current-future` | Keep maintenance state separate from startup policy | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-startup-default-target` | Choose the next target without switching the current session | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-startup-install-links` | Move startup enrollment without leaving an old target link | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-nm-persistent-profile` | Prepare an Ethernet profile for the next startup | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-nm-autoconnect-choice` | Retire a stale startup preference without interrupting the current link | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-nm-uplink-handover` | Keep management access while changing the next default uplink | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-removable-startup-policy` | Make removable storage optional without hiding required failures | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-local-host-alias` | Repair a local host name and retire its old alias | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-name-source-fallback` | Keep local overrides while allowing DNS fallback | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-dns-search-suffixes` | Repair a DNS search path without choosing the wrong site | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-dns-absolute-precedence` | Separate short, dotted and absolute DNS lookups | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-package-file-provider` | Find and install the package behind a missing command | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-package-source-selection` | Choose Debian sources without losing updates | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-package-source-recovery` | Recover useful metadata when one source is offline | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-env-child-settings` | Pass settings to a child report | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-env-parent-note` | Keep a planning note in the parent | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-removable-personal-session` | Let an operator mount their own archive | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-removable-shared-handover` | Hand over shared media without widening personal access | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-package-conflict-retirement` | Replace incompatible software without losing its settings | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-package-conflict-upgrade` | Resolve a version conflict while keeping required software | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-nm-dhcp-startup` | Make a saved Ethernet profile follow supplied DHCP settings | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-nm-dhcp-local-only` | Keep a DHCP address without taking over the default route | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-fs-offline-accounting` | Repair offline filesystem accounting without losing files | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-fs-retained-recovery` | Recover retained unnamed data and restore access | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-shell-startup-priority` | Repair the login file Bash actually reads | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-shell-startup-quiet` | Keep interactive greetings out of script output | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-conffile-retain` | Keep site settings during an upgrade | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-conffile-adopt` | Adopt a new default without losing recovery data | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-conffile-handover` | Hand over automatic updates and a configuration conflict | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-library-search` | Repair library search for two applications | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-library-cache-handover` | Hand over library search without temporary overrides | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-fhs-application-layout` | Repair an application’s file layout | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-fhs-locate-active-files` | Find an active application’s support files | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-listener-audit` | Retire the obsolete preview listener | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-env-tool-path` | Repair a worker’s tool search | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-skeleton-handover` | Repair account defaults and hand over a new home | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-command-permission-search` | Diagnose a command skipped during search | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-file-index-handover` | Repair a stale file index and prepare a current handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-whereis-support-handover` | Repair an application’s manual and source handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-sticky-collaboration` | Keep collaboration while limiting deletion | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-sticky-collision-handover` | Publish without replacing a colleague’s file | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-tree-private-stage` | Create a private working tree | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-tree-verified-retirement` | Repair a handover before retiring its source | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-find-sized-incident` | Collect only oversized incident records | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-find-rounded-evidence` | Repair an empty-file evidence selection | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-module-device-driver` | Activate a supplied device driver without removing another provider | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-module-index-repair` | Repair a new driver’s missing lookup index | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-rpm-metadata-preserve` | Repair attributes without erasing site settings | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-rpm-mixed-recovery` | Recover attributes and damaged content with the right repairs | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-swap-prepared-activation` | Make prepared swap available now and next time | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-swap-replacement-handover` | Retire old swap and apply the replacement priority | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-mbr-primary-plan` | Prepare MBR primary payloads and their contents | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-mbr-logical-space` | Use the fourth slot to hold logical payloads | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-mbr-storage-handover` | Finish a protected mixed MBR handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-aging-rotation` | Set password rotation with useful advance warnings | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-aging-grace` | Repair a password-change grace without erasing rotation | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-aging-contract-reset` | Extend a contract account while retaining its password reset | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-boot-root-identity` | Recover the intended system root | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-boot-initramfs-match` | Match early boot support to a new kernel | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-boot-recovery-handoff` | Prepare a recovery entry after an upgrade | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-grub-default-fallback` | Choose a working fallback from a GRUB menu | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-grub-recovery-entry` | Repair a recovery menu entry | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-grub-persistent-default` | Make a GRUB choice survive regeneration | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-grub-argument-scope` | Restore recovery diagnostics without changing normal startup | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-grub-bios-install` | Reconnect the BIOS disk to its Linux boot files | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-grub-efi-registration` | Restore the EFI loader and its boot entry | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-chrony-source-diagnosis` | Repair a reachable but unselected time source | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-copied-journal-incident` | Recover incident evidence from a copied journal | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-mail-team-alias` | Send a team alias to the right people | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-mail-personal-forward` | Keep a local copy while forwarding mail | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-mail-index-recovery` | Repair the alias file and its stale lookup | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-layout-log-growth` | Reserve a contiguous log growth window | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-layout-media-policy` | Deliver a layout for independent disk growth | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-print-paused-queue` | Accept jobs and restart a paused printer | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-print-held-request` | Release an existing held print request | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-print-destination-handover` | Repair a print-destination handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-locale-report-handover` | Repair a locale-sensitive report handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-encoding-import-repair` | Convert imported text without losing its characters | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-timezone-system-display` | Save a regional default without moving the clock | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-timezone-process-override` | Make a report follow its machine’s time zone | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-service-client-targets` | Connect the intended service, not just an open port | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-services-file-repair` | Repair service names without moving the server | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-host-mixed-ip` | Repair both families of a Linux connection | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-protocol-round-trips` | Restore reachability and service delivery separately | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-echo-path-diagnosis` | Repair a path when nearby replies are misleading | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-client-name-investigation` | Find why an application gets the wrong address | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-ssh-server-trust` | Verify the SSH server before using an account | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-modal-setting-repair` | Repair a file using vi-style modes | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-modal-record-maintenance` | Find, copy and retire complete records | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-modal-recovery` | Recover an unsaved draft and finish a file handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-editor-handover` | Configure an inherited editor and complete a handover | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-guest-driver-migration` | Repair driver support after a guest image changes | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-guest-object-recovery` | Recover a missing driver file while the device is active | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-guest-image-handover` | Hand over an image with both device drivers ready | updated | [Three-screen notes](../scenarios/presentation/lpic1.json) |
| LPIC-1 | `lpic1-xorg-keyboard-scope` | Scope a keyboard layout to one Xorg device | updated | [Three-screen notes](../scenarios/presentation/lpic1_desktop_profile.json) |
| LPIC-1 | `lpic1-remote-desktop-arrangement` | Choose a remote desktop arrangement | updated | [Three-screen notes](../scenarios/presentation/lpic1_desktop_profile.json) |
| LPIC-1 | `lpic1-accessible-input-profile` | Configure an accessible input profile | updated | [Three-screen notes](../scenarios/presentation/lpic1_desktop_profile.json) |
| LPIC-1 | `lpic1-retire-socket-activation` | Retire an unused socket and its service | updated | [Three-screen notes](../scenarios/presentation/lpic_host_dns_security.json) |
| LPIC-2 | `lpic2-service` | Bring the service back | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dns` | Repair the internal DNS answer | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-storage` | Recover the data volume | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dns-dualstack` | Add a service with IPv4 and IPv6 | updated | [Three-screen notes](../scenarios/presentation/lpic2_dns_queries.json) |
| LPIC-2 | `lpic2-dns-aliases` | Repair a looping service alias | updated | [Three-screen notes](../scenarios/presentation/lpic2_dns_queries.json) |
| LPIC-2 | `lpic2-dns-mail` | Restore primary and backup mail discovery | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dns-rrsets` | Retire an obsolete address safely | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dns-secondary` | Bring a stale secondary up to date | updated | [Three-screen notes](../scenarios/presentation/lpic2_dns_queries.json) |
| LPIC-2 | `lpic2-dns-handover` | Complete a DNS service handover | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-mount-identity` | Identify and mount the assigned volume | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-mount-policy` | Make saved and live mount policies agree | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-mount-manual` | Keep maintenance storage manual but available | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-mount-handover` | Complete a stable storage handover | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-unit-loaded-recovery` | Recover the definition a running service actually uses | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-unit-preparation-order` | Make preparation a real prerequisite | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-unit-optional-wants` | Keep a failed optional helper from blocking the core | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-unit-target-handover` | Hand over a multi-unit service without damaging its observer | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dns-reverse-investigation` | Repair names returned for connection-log addresses | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dns-reverse-ipv6` | Publish reverse names for an IPv6 rollout | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dns-reverse-handover` | Retire an old address and hand over both copies | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-web-routing` | Route site names without changing the default site | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-web-pages` | Choose a landing page and publish a correction | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-web-access` | Keep public pages open and staff notes private | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-web-handover` | Hand over a release that survives its next start | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-nfs-reference` | Publish a restricted reference share | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-nfs-publication` | Diagnose and publish an existing result | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-nfs-handover` | Hand a share to its replacement client | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-nfs-capstone` | Complete the two-share publication handover | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-backup-selective-recovery` | Recover one file without losing newer work | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-backup-incremental-chain` | Recover the complete backup chain | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-backup-snapshot-branch` | Branch a backup without rewriting its history | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-backup-recovery-handover` | Deliver a recovery that survives independent verification | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-free-space` | Use existing group free space without touching a neighbor | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-add-pv` | Add a blank PV when the current group is too small | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-filesystem-gap` | Find the gap between LV and filesystem capacity | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-handover` | Plan a two-volume shift handover without starving the second LV | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-capacity-inodes` | Find why an empty file cannot be created | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-capacity-blocks` | Leave usable space without spending the reserve | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-capacity-snapshot` | Repair a cleanup plan after files change | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-capacity-handover` | Hand over a filesystem with room for two outputs | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-backup-directory-selection` | Choose and prove an application recovery set | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dhcp-dynamic-ranges` | Provide enough dynamic addresses without rewriting a held lease | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dhcp-option-scope` | Repair inherited DHCP options and verify a renewing client | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-raid1-replace` | Restore a degraded mirror after a member failure | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-raid1-handover` | Restore a missing mirror member without damaging a neighbor | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-module-options-reload` | Repair saved driver options and the running instance | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-module-name-handover` | Repair the names used to request a driver | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-host-ipv6-addition` | Add IPv6 without losing working IPv4 | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-host-ipv6-gateway-scope` | Find the interface behind a link-local next hop | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-host-dual-stack-handover` | Complete a protected dual-stack server handover | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-provision-staging` | Allocate a new staging volume without spending the reserve | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-provision-group` | Build a separate volume group from the intended blank devices | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-provision-handover` | Hand over two raw allocations while retaining future capacity | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-retire-raw` | Return unused raw allocations without disturbing live data | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-retire-pool` | Retire a raw storage pool and clear only its PV labels | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-activate-data` | Make intact data available without rebuilding storage | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-availability-handover` | Hand over access without taking live storage offline | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-rename-volume` | Rename a data volume without replacing its contents | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-rename-handover` | Hand over a renamed pool without changing its data | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-pv-capacity` | Recover device space before growing the application | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-pv-end` | Reduce a recognized PV end without excluding data | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-saved-view` | Keep an earlier report while the application changes | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-cow-budget` | Prevent a saved view from exhausting its COW space | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-record-recovery` | Repair a damaged import and hand over a verified saved view | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-reference-repair` | Repair applications that read the wrong storage | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-reference-handover` | Restore application access after a storage handover | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-filter-policy` | Repair a filter that hides application PVs | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-devices-file` | Restore a devices-file handover | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-mount-repair` | Reconnect reports and audit to their original filesystems | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-lvm-mounted-handover` | Keep mounted reports connected through a rename | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-autofs-indirect-repair` | Repair demand-mounted reports and audit data | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-autofs-direct-conversion` | Convert indirect directories to direct AutoFS paths | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-raid1-assembly-policy` | Recover a clean mirror and its saved assembly policy | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-vlan-profile-commission` | Commission a saved VLAN profile on a shared NIC | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-vlan-return-handover` | Diagnose a VLAN return-path failure before startup | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-bonded-vlan-handover` | Prepare a bonded VLAN uplink for a NIC failure | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-bridge-lan-commission` | Join two LAN segments with a Linux bridge | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-pam-auth-bypass` | Repair an authentication bypass without locking everyone out | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-client-base` | Repair LDAP client defaults after a directory move | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-contact-query` | Build an on-call contact query that survives roster changes | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-person-import` | Import new directory people without disturbing the roster | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-contact-maintenance` | Repair directory contacts and retire an obsolete entry | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-import-command` | Repair a directory import command | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-stopped-update` | Recover a stopped directory update | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-account-retirement` | Retire archived accounts without removing active people | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-protected-password` | Repair a protected password change | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-contact-delegation` | Delegate contact changes without opening the directory | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-config-deployment` | Apply an LDAP policy to the right backend | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-schema-extension` | Extend LDAP entries with a custom schema | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ldap-schema-consumer` | Repair directory matching for a consumer | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-efi-removable-recovery` | Recover from a removable loader without the internal disk | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-grub-one-boot-recovery` | Boot an inspection system once from an edited GRUB entry | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-rescue-startup-repair` | Repair an ordinary startup from a rescue session | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-resource-cpu-intervals` | Measure the busy interval, not the boot average | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-resource-swap-activity` | Separate allocated swap from current paging | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-resource-restart-handover` | Publish a coherent post-restart measurement handover | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-capacity-trend-plan` | Plan capacity for the current growth period | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-capacity-import-handover` | Repair a capacity plan before the import | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-maintenance-login-notices` | Notify working sessions and future logins | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-maintenance-restart-handover` | Repair a conflicting maintenance restart handover | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-source-project-build` | Build and check a supplied source package | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-source-stage-install` | Stage a package beside its live copy | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-source-stale-handover` | Repair a stale source-build handover | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dns-recursion-boundary` | Keep public DNS open without opening recursion | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dns-secondary-transfer-policy` | Limit zone copies to the approved secondary | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dns-access-handover` | Repair a DNS access handover | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-proxy-client-boundary` | Close a proxy client-access leak | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-proxy-connect-order` | Repair a tunnel restriction that runs too late | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-cache-shared-privacy` | Make public caching useful without storing personal pages | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-cache-freshness-handover` | Repair a shared cache that keeps old release notes | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-proxy-uri-mapping` | Repair a reverse proxy that forwards the wrong application path | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-proxy-private-paths` | Protect private paths without blocking similarly named public pages | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ssh-key-authorization` | Repair authorized keys and their access permissions | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ssh-policy-handover` | Recover key-based administration without losing rescue access | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-smb-identity-access` | Repair an SMB login and its Unix access | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-smb-team-policy` | Give writers and reviewers the right share access | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-smb-change-handover` | Handover the correct share without touching archived files | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-smtp-destination-role` | Deliver domain mail on the right server | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-smtp-relay-boundary` | Close an open relay without blocking legitimate mail | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-smtp-queue-recovery` | Recover a relay loop without losing queued mail | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-sieve-header-filing` | File a named list without moving unrelated mail | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-sieve-compound-filter` | Combine review conditions with a message-size boundary | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-sieve-rule-order` | Recover an ordered filter without losing unmatched mail | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-sieve-sender-domain` | File sender domains without matching display names | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-sieve-recipient-recovery` | Repair a filter that mistakes mentions for recipients | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-sieve-envelope-recipient` | Route the delivery recipient, not the visible header | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-sieve-redirect-recovery` | Repair selective forwarding without extra copies | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-sieve-rejection-priority` | Refuse a retired mailbox without silently discarding mail | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-sieve-vacation-safety` | Configure a considerate absence reply | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-sieve-vacation-routing-recovery` | Repair replies skipped by delivery rules | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-procmail-sieve-migration` | Migrate a legacy mail-copy policy | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-dovecot-secure-access` | Restore secure access to two mailboxes | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-imap-mailbox-recovery` | Inspect and repair an IMAP mailbox safely | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-pop-transaction-recovery` | Recover POP access and commit a precise cleanup | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-mailbox-administration-recovery` | Audit a pending change and clean up read notices | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-router-local-transit-policy` | Separate router administration from LAN transit | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-router-forwarding-recovery` | Recover forwarding through the correct next hop | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-router-saved-policy-recovery` | Make the saved router policy match the working policy | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-nat-shared-address-replies` | Share a public address and recover replies | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-nat-published-service-policy` | Publish one service with a precise destination policy | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ipv6-control-header-policy` | Keep selected IPv6 control traffic without a blanket permit | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ipv6-tracked-service-policy` | Permit tracked IPv6 replies without opening inbound access | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-router-service-name-recovery` | Repair a services file and replace stale router rules | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-router-ipv6-scope-recovery` | Repair IPv6 scope and the next-hop interface | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-router-ipv6-ra-recovery` | Keep router advertisements working on an IPv6 router | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-router-ipv6-saved-settings` | Save IPv6 router settings and test their restart behavior | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ipv6-startup-policy-recovery` | Repair an IPv6 firewall that must survive startup | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ftp-local-jail` | Give a file-transfer account its own root | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ftp-upload-rights` | Repair useful uploads without opening the jail root | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ftp-chroot-handover` | Hand over two FTP identities with different root policies | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-exposure-client-policy` | Repair client access using listener and scan evidence | updated | [Three-screen notes](../scenarios/presentation/lpic2_telnet_transport.json) |
| LPIC-2 | `lpic2-exposure-listener-retirement` | Retire an exposed listener without taking down the useful services | updated | [Three-screen notes](../scenarios/presentation/lpic2_telnet_transport.json) |
| LPIC-2 | `lpic2-vpn-peer-profile` | Repair a VPN peer's endpoint and identity references | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-vpn-lan-return` | Restore outward and return routes across a site-to-site VPN | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-vendor-backport` | Choose the vendor fix without losing site settings | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-vendor-patch-set` | Repair a coordinated security package update | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-auth-filter-detection` | Restore useful denied-login detection | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-auth-filter-exceptions` | Remove health-check false positives without hiding real failures | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ids-header-direction` | Aim an alert at the protected upload service | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-ids-content-specificity` | Reduce marker alerts without losing required detections | updated | [Three-screen notes](../scenarios/presentation/lpic2.json) |
| LPIC-2 | `lpic2-kernel-root-build` | Build a kernel with root-driver support | updated | [Three-screen notes](../scenarios/presentation/lpic2_kernel_build.json) |
| LPIC-2 | `lpic2-kernel-staged-handover` | Stage a new kernel beside a working fallback | updated | [Three-screen notes](../scenarios/presentation/lpic2_kernel_build.json) |
| LPIC-2 | `lpic2-dkms-kernel-recovery` | Prepare a module for a different kernel | updated | [Three-screen notes](../scenarios/presentation/lpic2_dkms.json) |
| LPIC-2 | `lpic2-dkms-version-handover` | Hand over a module version without changing the fallback | updated | [Three-screen notes](../scenarios/presentation/lpic2_dkms.json) |
| LPIC-2 | `lpic2-initramfs-root-drivers` | Repair the drivers available before the root filesystem | updated | [Three-screen notes](../scenarios/presentation/lpic2_initramfs.json) |
| LPIC-2 | `lpic2-initramfs-kernel-handover` | Match early userspace to the kernel without losing recovery | updated | [Three-screen notes](../scenarios/presentation/lpic2_initramfs.json) |
| LPIC-2 | `lpic2-pxe-firmware-selection` | Choose the right network boot program for each client | updated | [Three-screen notes](../scenarios/presentation/lpic2_pxe.json) |
| LPIC-2 | `lpic2-pxelinux-client-lookup` | Repair the boot files chosen for one PXELINUX client | updated | [Three-screen notes](../scenarios/presentation/lpic2_pxe.json) |
| LPIC-2 | `lpic2-iscsi-target-identity` | Connect to the iSCSI target that exposes the intended disk | updated | [Three-screen notes](../scenarios/presentation/lpic2_iscsi.json) |
| LPIC-2 | `lpic2-iscsi-startup-policy` | Set one iSCSI export to reconnect automatically | updated | [Three-screen notes](../scenarios/presentation/lpic2_iscsi.json) |
| LPIC-2 | `lpic2-smart-short-diagnosis` | Run a fresh self-test on the reported ATA disk | updated | [Three-screen notes](../scenarios/presentation/lpic2_ata_smart.json) |
| LPIC-2 | `lpic2-smart-monitor-identity` | Keep the right disks under SMART monitoring after a rename | updated | [Three-screen notes](../scenarios/presentation/lpic2_ata_smart.json) |
| LPIC-2 | `lpic2-apache-https-chain` | Repair the certificate chain for an HTTPS site | updated | [Three-screen notes](../scenarios/presentation/lpic2_apache_https.json) |
| LPIC-2 | `lpic2-apache-https-sni` | Correct certificate selection on a shared HTTPS listener | updated | [Three-screen notes](../scenarios/presentation/lpic2_apache_https.json) |
| LPIC-2 | `lpic2-authenticated-zone-handover` | Authorize a zone transfer by its verified key | updated | [Three-screen notes](../scenarios/presentation/lpic_host_dns_security.json) |
| LPIC-2 | `lpic2-withdraw-obsolete-nfs-export` | Withdraw an obsolete NFS export | updated | [Three-screen notes](../scenarios/presentation/lpic2_nfs_export_operations.json) |
| Bash/Shell | `shell-handover` | Print a reliable handover | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-arguments` | Label the supplied file | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-input` | Read a request without losing backslashes | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-validation` | Reject a missing destination | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-report` | Write an incident report | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-loop` | Assemble a file handover | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-capstone` | Publish a safe incident report | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-streams` | Route output without mixing messages | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-pipelines` | Select the intersection of two log filters | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-eof` | Keep the final record when input ends | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-batch-audit` | Audit a batch without hiding failures | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-functions` | Reuse a label printer | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-return` | Reject one item without ending the batch | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-shift` | Separate a header from the remaining items | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-bundle` | Assemble a bundle only after preflight checks | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-local-heading` | Protect the report heading | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-scope-review` | Keep two topic reviews independent | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-scope-capstone` | Repair a review that damages its inputs | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-cut-inventory` | Extract an inventory view | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-sort-manifest` | Make a stable unique manifest | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-uniq-counts` | Count locations without losing repeats | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-text-capstone` | Audit duplicate asset identities | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-number-records` | Number a handover queue | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-order-total` | Calculate an order total | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-pack-items` | Plan full cartons and leftovers | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-count-capstone` | Publish a trustworthy count report | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-file-manifest` | Build a changing file manifest | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-collect-errors` | Collect incidents across a changing log set | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-file-batch-capstone` | Publish a safe incident handover | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-array-intake` | Keep an exact dispatch queue | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-array-priority` | Repair a queue that loses its tail | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-array-package` | Inspect a package before handover | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-array-handover` | Preview and publish a safe handover | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-capture-preview` | Preview requested and available work | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-capture-evidence` | Keep the evidence intact in a report | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-capture-decision` | Investigate findings without losing their status | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-capture-handover` | Preview or publish a batch investigation safely | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-case-interface` | Build a reliable report-mode interface | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-case-routing` | Route a prioritized work list without losing records | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-case-publication` | Preview or publish a validated release plan | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-options-report` | Build a dependable short-option report | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-options-jobs` | Keep repeated export jobs independent | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-options-publication` | Publish only a fully validated export batch | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-preserve-lines` | Keep every label while reading text | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-pipeline-status` | Do not publish a pipeline that failed | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-safe-handover` | Publish a validated text handover | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-command-options` | Keep source names out of the option list | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-command-dispatch` | Run approved transformations without losing jobs | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-command-plan` | Review or publish a mixed transformation plan | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-optional-policy` | Honor a deliberately empty setting | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-optional-report` | Choose a report destination safely | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-optional-handover` | Repair a configuration handover | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-numeric-retry` | Validate a retry window | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-numeric-intake` | Audit malformed intake records | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-numeric-publication` | Preview a safe batch allocation | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-loop-intake` | Stop intake without losing the tail | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-loop-fallback` | Recover each delivery job independently | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-ere-event-view` | Select whole events without losing evidence | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-ere-inventory-review` | Review every inventory row | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-ere-incident-report` | Publish validated incident evidence | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-trim-options` | Keep long-option values intact | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-trim-manifest` | Publish a collision-free normalized manifest | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-snapshot-audit` | Explain exact snapshot drift | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-conditional-rollout` | Review and apply a protected rollout | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-debug-status` | Repair diagnostics that hide a failed investigation | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-recovery-ledger` | Resume a checked handover without trusting its checkpoint | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-trace-delivery` | Trace and repair a delivery decision | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-trace-fallback` | Diagnose a snapshot fallback that hides newer work | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-stage-report` | Publish an exact report without losing its source | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-cleanup-preview` | Preview a cleanup before applying it | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-recover-publication` | Repair deployment and recovery together | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-map-lookup` | Answer a changing catalog by name | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-map-reconcile` | Reconcile accepted updates into one current report | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-managed-config` | Reconcile settings without erasing operator notes | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-backup-content-set` | Create a small restorable content set | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-restore-validated-set` | Restore an entire validated recovery set | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-log-owner-triage` | Build an owner-aware incident report | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-counter-rates` | Turn counter differences into useful rates | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-cpu-share` | Report the CPU work that happened in the interval | updated | [Three-screen notes](../scenarios/presentation/bash.json) |
| Bash/Shell | `shell-measurement-recovery` | Repair a measurement handover without hiding bad intervals | updated | [Three-screen notes](../scenarios/presentation/bash.json) |

## Exceptions and existing approval

- `ccna-ipv4` — **already meets**: Read an IPv4 address. Reviewed Subnet lesson concepts (A computer belongs to an address group, A prefix determines the block size, Find the block containing this computer), supplied worked values and the actual introductory render path. Retains the approved custom introduction and its worked table; range alignment is corrected in the shared native component.
- `ccna-magic-blocks` — **already meets**: Find a subnet’s block size. Reviewed MagicSubnet lesson concepts (the approved four-step native subnet method), supplied worked values and the actual introductory render path. Retains the approved custom introduction and its worked table; range alignment is corrected in the shared native component.
- `ccna-magic-boundaries` — **already meets**: Find every address boundary. Reviewed MagicSubnet lesson concepts (the approved four-step native subnet method), supplied worked values and the actual introductory render path. Retains the approved custom introduction and its worked table; range alignment is corrected in the shared native component.
- `ccna-magic-cross-octet` — **already meets**: Follow a subnet across octets. Reviewed MagicSubnet lesson concepts (the approved four-step native subnet method), supplied worked values and the actual introductory render path. Retains the approved custom introduction and its worked table; range alignment is corrected in the shared native component.
- `ccna-magic-octet-aligned` — **already meets**: Handle an octet-aligned prefix. Reviewed MagicSubnet lesson concepts (the approved four-step native subnet method), supplied worked values and the actual introductory render path. Retains the approved custom introduction and its worked table; range alignment is corrected in the shared native component.

Status above distinguishes completed reviews from pending or blocked work. Registered labs have
Introduction, separate guided practice and independent Assignment paths. The five
already-meets entries retain the accepted CCNA teaching; the shared range-cell alignment
correction applies to their diagrams where relevant. The owner used Show worked answer
during review, so independent novice acceptance remains unverified.

Regenerate/check with `python3 tools/dev.py tool teaching_inventory.py --write` / `--check`.
