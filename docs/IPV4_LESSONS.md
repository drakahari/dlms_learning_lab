# IPv4 static routes and branch recovery

Six distinct authored assignments in `scenarios/ipv4-static-v1.json`, unit `ipv4-static-foundations`. They extend the original subnet/VLSM/client-repair practice without renaming those tasks or counting different numbers as additional labs. Current catalog:64 authored assignments/fifteen units. Unit prerequisites are advisory; experienced learners may open any assignment directly.

| Assignment | Practical skill | Required evidence |
| --- | --- | --- |
| Connect the branch with a network route | Construct an explicit network entry with an immediate neighbor | Selected /24 plus actual Client→Server delivery and reply |
| Repair the journey back | Diagnose forward success and failed reverse routing | Selected office /24 on R2 plus full round trip |
| Repair the more-specific detour | Repair longest-prefix selection while preserving a default-served service | Actual selected /24 and default entries serve different endpoints |
| Send one host through a different router | Make a deliberate /32 exception without changing its peer’s network direction | Selected host/network entries, distinct observed router egresses and Client’s reply |
| Restore the primary and floating backup | Compare equally specific eligible entries, then prove withdrawal behavior | Normal transit and controlled two-ended primary-down backup, selected /24 in both states, full replies |
| Complete the resilient branch handover | Add a free second workstation and diagnose forwarding, neighbors, preference and office-wide returns | Both clients in normal/failure conditions, selected branch /24, normal R2 return egress and office /24 |

## Teach, try, apply

Learn introductions teach local versus remote directions, route fields, immediate neighbors, both-direction verification, prefix specificity before preference, host exceptions, source addresses, eligible floating routes and the difference between absent neighbors and withdrawn interfaces. The earlier subnet/VLSM assignments now have foundational introductions and isolated editable examples (ADDRESSING_TEACHING.md). First-time novice usability acceptance remains separate and open.

Every introduction has a separate private-address example and a concrete experiment. Load example settings fills only its practice draft; Apply and test example applies only that topology. Assignment drafts, state and credit stay separate across views and restarts. Suggested experiments include alternate distances, a single-host patch that fails an office-network requirement, a deliberately broken specific direction, an oversized exception, an ineligible backup and disabled forwarding. Tests execute each described change, not merely the reference example.

The full assignment stays visible with its native tools in both Learn and Challenge. Guidance differs; the work is not reduced to trivia. Device selection, address/forwarding controls and route editors act only on typed simulated state. Wide route rows align destination, next hop, interface and distance; narrow windows stack labeled fields and scroll. The selected device’s connected neighbors supply relevant next-hop evidence; the complete topology, applied eligibility/paths, full outcome details, walkthrough and references remain optional. Check configuration is the only primary grading action. Invalid text stays editable, all valid edits apply together, and changed drafts cannot borrow earlier success.

## Grading and review rules

Never grade only a route’s existence or infer the intended endpoint from mutable address ownership. Use fixed assignment/range outcomes plus named endpoint verification. `StaticRouteFor` requires that exact static prefix to be selected for its endpoint and that delivery/reply actually work. Default-empty `when` conditions can test selected prefixes under bounded cloned operational failures. A broader default must not stand in for an explicitly requested backup /24. Capstone normal office return egress must not be masked by a supplied working reserve.

Different free client addresses, unequal eligible distance pairs, route ordering and supported next-hop choices remain legitimate. No exact action string or fixed numeric distance earns credit. A /32 route can validly reach one host, and multiple host routes can reach multiple hosts; a task requesting an office network entry still requires that /24. Static candidates with absent neighbors do not automatically withdraw; no tracking or dynamic health monitor is implied.

The goal sets preserve every fixed interface assignment, including isolated examples. Positive tests and cloned failure cases must fail when forwarding, endpoints or return paths are broken. Check tests for moved/deleted endpoint ownership, incomplete/raw drafts, rejected shapes, partial repairs, alternate configurations, missing backups and broader-prefix workarounds before accepting content. No generated placeholder or development fixture enters the authored count.

## Sources and scope

Official CCNA200-301 v1.1, checked2026-10-22, current through2027-02-02; upcoming v2.0 remains separate. The source inventory and fingerprints are in CURRICULUM.md/OBJECTIVE_SOURCES.json. Primary technical references are [Cisco route selection](https://www.cisco.com/c/en/us/support/docs/ip/enhanced-interior-gateway-routing-protocol-eigrp/8651-21.html) and [RFC1812](https://datatracker.ietf.org/doc/html/rfc1812). Content paraphrases narrow modeled behavior; it is not a vendor CLI or full RFC conformance course.

Exact boundaries are in IPV4_ROUTING_SUBSET.md: private/documentation endpoints, /1–/30 interfaces, /0–/32 route prefixes, immediate next hops, supplied operational interface states and bounded path/reply checks. No real packets, ARP cache, dynamic protocols/metrics, ECMP, recursion, ACL/NAT, transport or complete source-address policy. Original subnet exercises still exclude /31 and /32; host-route prefixes do not change that. Router-origin checks use the selected outgoing interface address as source, so supplied reverse directions must serve that source too. Actual Linux/Cisco equipment experience and Windows/macOS desktop acceptance are separate.


## Accepted development evidence

289 Rust tests, formatter/Clippy,21 Python tooling/boundary checks,64/fifteen bundled validation, all28 palette/notices checks and unchanged official-source fingerprints pass. Six new content tests execute every taught reference and exact experiment, alternate address/distance/order methods, plausible partial/mistaken repairs, stale success, broader backup-prefix workarounds, masked primary returns, moved endpoints, cloned-state immutability and separate example/draft/reset/resume behavior alongside all prior58 attempts. Two additional native-egui tests cover real manual field/Tab/Enter route entry, both keyboard-only and pointer-opened interface menus, and all six view/mode layouts across desktop/narrow sizes,150% and the table cutoff. No shell interpreter behavior changed; the earlier601 disposable Bash comparisons remain separate evidence, not a new run in this content batch.

The full Fedora44 XWayland flow passed all six introductions, real example-load/apply actions, isolated practice, complete assignments, wrong retries, solved configuration drafts through native Check, completion, saved invalid drafts/reload/reset and a manually typed route. It also reached Check in long150% stacked editors, inspected stable light/dark Challenge and reviewed unit-library wrapping. Actual captures were1920×1080 and946×929; screenshots stay ignored under screenshots/workspaces/ccna-ipv4-* and ipv4-unit-*. Most alternate configurations are backend-tested rather than individually typed on the desktop. Native event injection and agent visual review do not establish novice, physical keyboard/device, screen-reader or Windows/macOS desktop acceptance.

Independent review found and resolved broader-fallback prefix and hidden primary-return grading gaps, added router-source teaching, corrected a host-route misconception in a hint, and checked the responsive/keyboard/current-draft behavior. Screenshot capture now waits through theme transitions; scrolling targets the actual Check rectangle instead of assuming one wheel step is enough. Full curriculum, default-Wayland startup investigation, assistive-technology and other platform acceptance remain open. No installer or public release publication.
