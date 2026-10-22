# IPv6 static-network foundations

Six original authored assignments, bundled in `scenarios/ipv6-static-v1.json`, unit `ipv6-static-foundations`. Prerequisite advice: IPv4 foundations. This is narrow partial practice for current CCNA v1.1 objectives1.8/1.9/3.3, not complete coverage or a device CLI course. Numeric alternatives are answers within a lab, not additional authored scenarios.

| Assignment | New skill and full task | Legitimate alternatives verified |
| --- | --- | --- |
| Add an IPv6 workstation to a team link | Read grouped hexadecimal/`::` and a conventional /64; choose a unique ordinary workstation address and verify replies | Many unique interface identifiers, compressed/expanded equivalent text |
| Repair an IPv6 workstation’s path | Correct the prefix and neighboring default next hop without changing supplied peers | Link-local or on-link global gateway |
| Put a link-local next hop on the right link | Repair outgoing-interface scope on a multihomed router; preserve branch and return communication | Network route or two specific host routes |
| Repair the route that beats the default | Diagnose longest-prefix selection before distance; retain another working destination | Remove the broken route or repair its next hop |
| Make the backup route work before the outage | Preserve preferred primary routing and prove both failure cases with unchanged base interfaces | Different valid distance pairs/global gateways; observed first hop graded |
| Hand over a working IPv6 branch | Combine prefix, transit forwarding, next-hop and return-route repair | Specific destination/return routes instead of reference network/default routes |

Learn uses Introduction → Try example → Assignment. Introductions teach new concepts before assessment; examples use different addresses and independent saved topologies/drafts. Load example settings fills a practice draft, never the assignment; Apply and test example is explicit. Suggested experiments are tested, including deliberately broken next hops/interfaces/preference and repaired outcomes. Challenge retains the complete task/editor without unsolicited teaching or solutions.

The normal native workspace has a small topology, selected-device address/interface or static-route editing, one Check configuration action, plain retry causes and optional path inspection/help. Wide screens split topology/editor; narrow and larger-text screens stack and scroll. Routing tasks initially select routes; the last device/view and incomplete fields survive resume. All application execution remains in-process/offline; internal pack adapters are not presented as IOS/Linux commands.

`tests/ipv6_content.rs` verifies every reference, distinct alternative, plausible wrong/partial solution, each actual suggested experiment, reserved/duplicate/out-of-subnet feedback, first-hop failover preference and old/new attempt roundtrips. `tests/ipv6_workspaces.rs` covers bounded atomic edits and import/save boundaries. Native egui tests cover typed address entry without optional settings, Check/retry, focus/Tab, isolation, reset/resume, Learn/Challenge, widths720/1200 and text100/150%.

Fedora44/COSMIC native synthetic-profile run traversed all six introductions/examples/assignments, wrong-state feedback, reference-draft application, completion, invalidation and save/resume. First assignment was typed through native keyboard events; later reference configurations were staged as typed drafts, then applied through the native Check button. Full dark1920×1080 and light150%946×929 views were rendered and reviewed. This is agent input/render evidence, not an independent novice study, screen-reader acceptance, physical networking test or Windows/macOS desktop acceptance.

Reproduce: `DLMS_SMOKE_STAGE=ipv6 ./target/debug/dlms-learning-lab --ui-smoke`; focused resize/scroll input check: `DLMS_SMOKE_STAGE=ipv6-layout ./target/debug/dlms-learning-lab --ui-smoke`. These use isolated synthetic /tmp progress. Local captures stay ignored under `screenshots/workspaces/ccna-ipv6-*.png` and `ipv6-*-light-150.png`.

Boundaries: manual on-link declarations, deterministic neighbor ownership and static forward/reply paths only. No ND/DAD/RA packets, SLAAC/DHCPv6, ECMP, recursive next hops, full source-selection policy or actual router commands. Ordinary-workstation /64 choices exclude the known subnet-router/reserved-anycast identifiers; IPv6 has no broadcast exclusion. Details and primary references: [IPV6_SUBSET.md](IPV6_SUBSET.md). Address-type progression, dynamic protocols and multihop IPv4 remain unfinished curriculum work.
