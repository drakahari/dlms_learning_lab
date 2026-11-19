# IPv6 router lessons

Two normal-catalog LPIC-2 lessons extend the existing routing/filtering work.
They do not establish full objective 212.1, production firewall readiness, kernel
parity or independent beginner acceptance. The current source catalog has 489
labs in 195 units; the previous 487 lab objects and 194 unit objects remain exact.

## Learning path

In Library, select **LPIC-2**, then **IPv6 routers: control headers and tracked
replies**. Search for `IPv6` or `control headers` also finds the lessons. The unit
follows **Linux host IPv6** and **Linux routers: tracked replies and NAT**.
Prerequisites guide navigation without forcing experienced learners through them.

| Stable identifier | Practical assignment |
| --- | --- |
| `lpic2-ipv6-control-header-policy` | Replace blanket/incorrect permits with the stated LAN, interface-direction and ICMPv6 type/code policy. Keep other modeled traffic blocked. |
| `lpic2-ipv6-tracked-service-policy` | Restore an outgoing web request, its actual reverse and later original traffic; reject unsolicited arrivals and preserve the selected ICMPv6 control header. |

Introduction explains the new concepts and demonstrates different example
prefixes. Try example uses a separate terminal, command draft and traffic report.
Insert command fills the field; Run command applies it inside the simulator.
Assignment retains its complete independent requirements and intentional errors.
Challenge opens that assignment without unsolicited worked guidance.

Traffic tests distinguish expected requirements from actual header/path results.
The control lesson has stateless ICMPv6 observations. The service lesson derives
an actual reverse tuple from a permitted earlier request, then verifies the
observed direction and state. A blocked request does not fabricate a reply.
Changing input or rules clears stale checks/reports. A read-only traffic trial
can preserve an already checked configuration, but cannot initially award credit.

## Grading and authoring

The source is `scenarios/linux-v6-lessons-v1.json`, with unit metadata in
`linux-v6-units-v1.json` and the reviewed presentation shard `presentation/lpic2.json`.
The consolidated v56 pack contains the exact v55 objects followed by these two
objects. Its compact representation stays within the unchanged pack-size limit;
human-readable authored sources remain separate. Closed origin checks verify
both parts. Historical factories remain available for preservation comparisons.

`linux_v6_workspace::Definition` binds independent tasks, initial sessions and a
reachable worked example. Each task declares fixed interface/route facts, three
protocol/state policy contexts and bounded traffic cases. Rules are parsed and
evaluated by the typed simulator; grading never matches learner command strings
against an expected answer. Exact whole-range policy comparison supplements the
deterministic traffic cases. Equivalent supported split-prefix rules and numeric
protocol aliases pass. Hardcoding all displayed clients can pass the sample
traffic while correctly failing the required `/64` policy.

The scenario objective retains the exact full requirement text. Presentation may
reformat that text into a checklist; its short goal is a separate heading. The
inventory guard compares the original requirement characters rather than allowing
new conditions to be introduced by a renderer.

Internal `v6-workspace` JSON actions are authoring/native-control records, not
Linux commands or executable scenario scripts. Guides contain only supported
literal simulator commands. Source validation rejects mismatched initial sessions,
bad versions, malformed cases, unsupported worked commands and unreachable worked
outcomes. Progress validation binds the saved session to that source and replays
stored traffic reports; forged reports are rejected. Existing progress formats,
lab identifiers, themes and other-track workspaces are preserved.

## Verification status

Current evidence: 47 focused numeric/filter/state/path checks, 23 retained
flow/router checks and 15 final new source/workspace checks pass. Eight native
widget/plan and five current-catalog UI checks pass. Overlapping reruns are not
added to these counts; this is not an inclusive Rust suite. Serialized/disk resume, supported alternatives,
wrong codes/prefixes/state lists, partial completion, stale success, reset, draft
isolation and source preservation are exercised. Widget checks cover Enter/Tab,
100/150/200% text, light/dark themes and wide/narrow logical layouts.

Actual R2 Fedora development windows pass160fixed actions/36captures.11originals
were inspected across all three views of both lessons, actual stateless control
headers and tracked replies, failure/completion, Navy100 and short Light150/
Navy200. A direction-glyph defect was caught in the originals, corrected with
plain text and tested; the older source/ELF/captures remain retained. Final native
protocol labels use TCP/UDP/ICMPv6, and state groups say ESTABLISHED rather than
implying only the reply direction. Final source gates pass: 542 focused Python checks, all-target Clippy with
warnings denied, the 489-row inventory and strict coverage. Final installed
verification remains pending.

Two normal cold processes pass with independent unfinished rules, queued control
commands, exact traffic reports, raw/provenance/preferences preserved before real
native repairs; all six originals inspected. These are development observations.
The latest verified package remains487in CHECKPOINT.md, with its disclosed older
NAT feedback-number limitation. The separate489package will include that fix.

The INPUT-versus-FORWARD negative control first reproduced a false-positive
refusal. Grading now requires an actual FORWARD drop for that stated outcome;
feedback identifies the actual chain. Both authored assignments retain their
requirements and legitimate solutions. Whole-source/progress tests pass after
the correction. No inclusive Rust suite or independent novice acceptance follows.

Actual originals (local ignored evidence):

- [Introduction](../.build-artifacts/native-runs/run-lmob8otx/screenshots/workspaces/ipv6-router-0-introduction.png)
- [Guided practice](../.build-artifacts/native-runs/run-lmob8otx/screenshots/workspaces/ipv6-router-1-practice.png)
- [Independent assignment and actual reply](../.build-artifacts/native-runs/run-lmob8otx/screenshots/workspaces/ipv6-router-1-assignment-reply.png)
- [Light150 restored populated draft](../.build-artifacts/tmp/native-reopen-e9420e408a4865cefbc1377a/read/reopen-v6-reader-assignment.png)

Cached traffic reports are source-bound, including their explanations. Future
changes to those recorded explanations require a controlled refresh/migration;
do not silently reinterpret old stored results. The current text refinements
precede the first owner-facing489candidate and affect no original learner profile.

See [the declared simulator subset](LINUX_IPV6_ROUTER_SUBSET.md) for exact limits,
references and exclusions. No real packets, root privileges, kernel changes,
neighbor discovery, actual PMTU discovery, TCP stack, ICMP connection tracking,
RELATED association, timers or NAT are claimed. Windows/macOS runtime, physical
device, screen-reader and independent novice acceptance remain pending.


## Actual489 installed acceptance after frozen build

Private487to489 lifecycle, installed160actions/36captures, prior NAT feedback
56/2 and two normal cold processes pass.15native+2feedback+6cold originals were
viewed: actual final protocol labels, strict chain feedback, actual reversed TCP,
complete/stale states,1920x1080Navy100 and946x929Light150/Navy200. All distinct
unfinished drafts, reports, raw/provenance/preferences restore before native repair.
Current CHECKPOINT.md has exact launcher/hashes and immutable selected retention.
This post-build record intentionally differs from bundled pending-verification docs.
No full curriculum, indefinite startup, otherOS, physical-device, screen-reader or
independent novice acceptance. Existing T3services and K3/T2scope/settings stayOpen.

Actual installed originals:
- [Introduction](../.build-artifacts/native-runs/run-fc27y708/screenshots/workspaces/ipv6-router-0-introduction.png)
- [Separate practice](../.build-artifacts/native-runs/run-fc27y708/screenshots/workspaces/ipv6-router-1-practice.png)
- [Actual reversed reply](../.build-artifacts/native-runs/run-fc27y708/screenshots/workspaces/ipv6-router-1-assignment-reply.png)
- [Restored populated command](../.build-artifacts/tmp/native-reopen-5b0cc43418387bd5651cd05c/read/reopen-v6-reader-assignment.png)
