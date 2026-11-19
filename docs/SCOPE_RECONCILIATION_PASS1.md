# Scope reconciliation: first six stale parent entries

Checked against the registered source catalog, continuation evidence and verified371R2 on2026-11-10. This corrects reporting; it adds no lab, grading requirement or completion credit. All child reference rows retain their previous status. None of these parents is complete.

| Blueprint / objective | Existing practical labs | Remaining behavior / limitation |
| --- | --- | --- |
| CCNA v1.1 / 3.5 | `ccna-hsrp-virtual-gateway`; `ccna-hsrp-incumbent-return`; `ccna-hsrp-wan-tracking`; `ccna-hsrp-resilient-handover` | Finite two-router HSRP-style model; no packets, timers, authentication, arbitrary groups, IPv6 FHRP or device convergence. |
| CCNA v1.1 / 4.5 | `ccna-syslog-local-remote`; `ccna-syslog-export-path`; `ccna-syslog-facility-routing`; `ccna-syslog-policy-handover` | No real daemon, UDP delivery, IOS commands, rate control, authentication or reliable transport; no full protocol conformance. |
| CCNA v1.1 / 4.7 | `ccna-qos-mark-boundary`; `ccna-qos-queue-bursts`; `ccna-qos-trusted-handover` | Equal-size virtual packets only; no policing, shaping, token buckets, weighted/fair scheduling, congestion avoidance, real timing or end-to-end guarantees. |
| LPIC-1 v5.0 / 101.3 | `lpic1-startup-current-future`; `lpic1-startup-default-target`; `lpic1-startup-install-links` | Supplied unit programs only; no actual boot/shutdown/reboot, SysV runlevels, wall/broadcast notification or general systemd command surface. |
| LPIC-1 v5.0 / 109.2 | `lpic1-nm-persistent-profile`; `lpic1-nm-autoconnect-choice`; `lpic1-nm-uplink-handover`; `lpic1-nm-dhcp-startup`; `lpic1-nm-dhcp-local-only` | No real NetworkManager daemon, ifupdown/networkd/netplan, wireless profiles, full retry lifecycle, D-Bus or full distro-specific persistent configuration. |
| LPIC-2 v4.5 / 210.1 | `lpic2-dhcp-dynamic-ranges`; `lpic2-dhcp-option-scope` | Legacy ISC-format simulation only; no daemon/lease-file administration, BOOTP/fixed hosts, shared networks, DDNS/failover, DHCPv6 or general server conformance. |

These21 existing labs must not be counted again as21 new tasks. Each parent now says Partial rather than Planned/Open and names its actual IDs and supported behavior. The evidence tests already exist; this pass checks their references and preserves the lesson specifications. It is not a new execution of every historical lab.

## Current counts after correction

- Current certification planning:60Planned,75Partial,1Supporting explanation (136total).
- CCNA:23Planned,30Partial; LPIC-1:14Planned,27Partial,1Supporting; LPIC-2:23Planned,18Partial.
- Granular map:309Partial practice,7Supporting explanation,924Open (1240overlapping references). Only six parent rows changed.
- The29future CCNA rows remain separate and unchanged.
- The372registered labs and137units are unchanged. Latest admitted package is now372R1 after exact package/content/native verification.

## Still to reconcile

The original66-proposal snapshot remains in [Remaining scope reconciliation](REMAINING_SCOPE_RECONCILIATION.md). This is a first pass, not a complete missing-lab inventory. Other proposed labs, expansions, supporting-only areas and nine Bash categories still need an outcome-level breakdown. Neither60Planned proposals nor924Open references is an additional-lab total. An exact remaining total is still unestablished.

No full exam preparation, issuer-objective completion, actual-device conformance or independent novice acceptance is claimed. Original IDs, progress, drafts and objectives remain intact.
