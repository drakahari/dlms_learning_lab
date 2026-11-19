# Linux host-networking batch — 2026-11-02

Four distinct assignments follow Linux file/terminal foundations, with separate
worked hosts and native practice. Existing225 lab/60unit objects remain exact;
current source review library229/61:69CCNA,56LPIC-1,40LPIC-2,64Bash.

| Stable ID | Practical task | Meaningful alternatives and failure checks |
| --- | --- | --- |
| lpic1-host-address | Repair a down interface and wrong LAN address without disturbing management | Any unused conventional address on the correct LAN; link-first/address-first, flush or exact delete; wrong prefix, network/broadcast, peer and gateway choices fail |
| lpic1-host-routing | Repair competing defaults and a more-specific stale host route | Remove/raise competing default, add better metric, replace or remove host route; high metric never defeats a longer prefix |
| lpic1-host-resolver | Isolate a name-resolution fault from a healthy IP path | Native virtual resolver editor, comments/spacing retained; two correct supplied answers required, healthy routes/interfaces protected |
| lpic1-host-handover | Repair application interface, default and resolver while preserving management | Safe address/metric/order alternatives; both application outcomes and retained management must pass together |

## Teaching and interaction

Introduction teaches the new concept before the assignment and shows a separate
worked sample. Try example places sample inspection/repair/deliberate-error
instructions near the terminal, with Insert worked action preparing text only.
The learner still chooses Run command or Save resolver. Assignment keeps its
complete requirements, terminal and applicable configuration editor. Hints and a
worked approach are optional. Challenge opens the full independent assignment
without unsolicited sample solutions. No tiny-question wizard or new theme/UI
redesign is introduced.

Actual terminal output is retained separately from supplied facts and unexecuted
input. The native Test address control explicitly says simulator; it is neither
ping nor ip route get and reports no RTT. Resolver edits are virtual: no file on
the host is opened. Text changes derive dirty state from raw saved text rather
than a forgeable flag. Raw comments/spacing, pending command, separate sample
work, terminal transcript and unfinished resolver draft are retained for resume.
Successful applied edits have bounded replay history tied to the exact held
scenario. False histories, altered facts or fabricated applied-text markers fail
restore without replacing the prior save. Editing saved state invalidates affected
outcomes; unapplied resolver text cannot keep current success.

## Fidelity and coverage

[Subset](LINUX_HOST_SUBSET.md) defines canonical supported commands and honest
exclusions. This is a narrow practical LPIC-1 v5.0/109.3 slice and client-resolver
109.4 slice, with supporting109.1/109.2 reasoning. It is **not persistent Linux
boot configuration**, full iproute2, NSS, real transport or complete parent
objective coverage. NetworkManager, policy routing, DHCP/IPv6, multiple addresses,
ss/ping/traceroute, host/dig and full DNS remain unimplemented here. No named tool
receives credit merely because a simulator control illustrates a related concept.
Current supplement: CONTINUATION_COVERAGE.csv. Historical201 issuer snapshots and
all sealed shell observations remain immutable. Evaluator15 and64 old scripts
are unchanged.

Official LPI objectives and ip-address(8), ip-route(8), resolv.conf(5) references
were read2026-11-02; URLs are in each scenario and LINUX_HOST_SUBSET.md. Broader
manual features are deliberately not approximated. No host-network command was
run for reference; pure-model tests and documented semantics are the evidence.

## Verification checkpoint

Passed:12 pure-host model groups,13 lesson/content/persistence/catalog groups,
13 retained repository groups (`host-model-r2.log`). The new tests cover valid
alternatives, wrong/partial outcomes, dirty-state invalidation, save/read/reset,
malformed/unsupported input, source/profile corruption, byte/history limits and
atomic refusals. Existing225 lab and60unit objects compare exactly.

Native widgets, light/dark layouts, focused Python guards, production Clippy,
separate candidate build and actual Fedora screenshots are still pending at this
source checkpoint. No current source-only lab is described as native-accepted.
No cold process reopening, installation/upgrade, Windows/macOS, screen-reader,
physical-device or independent novice-comprehension acceptance is inferred.


## Packaged Fedora verification

Separate229 candidate: `dist/teaching-review-20261102T152402Z/`, binarySHA256
`822a94f51f4eb854bb24e60046c062d19ac4b29315ec62ab2301b823a1e0d42f`.
Packaged229/61 content passes; actual host-batch native run passes all4 separate
examples and all4 initial/solution/damage/retry/save-read/reset flows.40 PNGs are
retained in `.build-artifacts/native-runs/run-pb55k0iu/screenshots/workspaces/`.
Actually inspected: address Introduction, route practice, resolver Assignment,
route damage feedback, solved handover, narrow light200% practice and Check/Save
controls. Read-only worked commands may use horizontal scroll; terminal output
and large-text pages scroll without shrinking the text.81 focused Python checks
and production Clippy pass. Prior source-checkpoint pending statements above are
historical; no full inclusive suite repeated for this focused batch.

- [Introduction](../.build-artifacts/native-runs/run-pb55k0iu/screenshots/workspaces/teaching-lpic1-host-address-introduction-dark-100.png)
- [Practice](../.build-artifacts/native-runs/run-pb55k0iu/screenshots/workspaces/teaching-lpic1-host-routing-example-dark-100.png)
- [Assignment](../.build-artifacts/native-runs/run-pb55k0iu/screenshots/workspaces/teaching-lpic1-host-resolver-assignment-dark-100.png)
- [Failure](../.build-artifacts/native-runs/run-pb55k0iu/screenshots/workspaces/host-lpic1-host-routing-damage-feedback.png)
- [Light200% controls](../.build-artifacts/native-runs/run-pb55k0iu/screenshots/workspaces/teaching-lpic1-host-handover-assignment-light-200-controls.png)

Native save/read is not cold process reopening. No new installation/upgrade,
Windows/macOS runtime, screen-reader/physical-device or independent novice study
acceptance. No installed app/progress replacement, push/merge/release, dependencies
or system settings changes. Continue Bash debugging/recovery rather than waiting
for routine approval.
