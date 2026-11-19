# Linux router/filter profile and lessons

## Current status —2026-11-14

**485authored labs/193units:113CCNA/154LPIC-1/138LPIC-2/80Bash.** Three distinct
router lessons follow host routing and SSH policy. Every preceding482scenario
and192unit value is preserved. Existing135LPIC-2presentation values remain exact.
Separate485Fedora review candidate is now verified; original app/progress unchanged.

Focused runtime24and later17feedback/source/workspace checks pass; the latter
includes a new rule-versus-policy feedback regression. Seven widget/plan checks and an additional Tab/Shift-Tab traversal check
pass, including the actual short1268×632viewport at200%text. Normal-catalog5and
prior mailbox-capstone7checks pass. All-target ClippyR3passed before the observed
native refinements; final lint remains pending. Fresh post-integration source
admission189/198, reopen-tool45and live-coverage1passed before the final text edits;
final guards/coverage are rerun for delivery. No inclusive Rust runtime claim.

Development nativeR2completed238actual pointer/keyboard actions and55captures.
All three Introductions, guided practice and independent assignments were visually
inspected, along with errors, Light150and Navy200controls. Its actual ELF hash was
`2b1e783c79136356216a01a37ea17ecb172fe28beb9411eb82dade9ecc00add3`.
The first native run is retained as failed at action223: the10-row editor exceeded
the usable200%viewport. The correction derives full rows from actual parent clip
height and font metrics; no text shrink or per-screen offset. Policy feedback
now names the responsible rule/fallback, and long worked commands use the existing
wrapping component. Default practice offers inspection first. All grading
predicates and initial workspaces remain unchanged.

Normal cold writer4469and reader4857exited0(58.916s/45.189s), restoring distinct
unsaved drafts, saved filter repair, queued commands, raw progress/provenance and
preferences before actual native Save/Run/Check repairs. All six cold originals
were inspected. Development evidence, not packaged/other-platform, physical-device,
screen-reader or independent novice acceptance. Final text-only corrections use
plain direction wording and match the revised feedback; old snapshots are never
re-attested against later source. No other-project/system/owner-progress changes.

Partial212.1references:K1,K2,K4,K5,K6,T1,T4. Other address-scope, IPv6 and service
name references remainOpen. No complete router/exam preparation claim.

## Actual installed candidate evidence —2026-11-14

Protected launcher and artifact hashes are in CHECKPOINT.md. Private482→485
lifecycle, installed238actions/55captures and two normal-process cold reopen/
repair pass.14native and all6cold original captures inspected; source readback
and3154selected-file retention match the actual packaged ELF. Final all-target
ClippyR5and463focusedPython pass. Earlier development observations below retain
their own ELF identity and are not re-attested as this package.

Representative installed originals (local ignored evidence, no public screenshots):
[Introduction](../.build-artifacts/native-runs/run-c2c_vkzv/screenshots/workspaces/router-0-introduction.png),
[guided investigation](../.build-artifacts/native-runs/run-c2c_vkzv/screenshots/workspaces/router-1-guided-policy-trace.png),
[independent editing](../.build-artifacts/native-runs/run-c2c_vkzv/screenshots/workspaces/router-2-assignment.png),
[200%Save controls](../.build-artifacts/native-runs/run-c2c_vkzv/screenshots/workspaces/router-2-dark200-save.png),
[restored then repaired assignment](../.build-artifacts/tmp/native-reopen-0e6653526609bb65f54e196d/read/reopen-router-repaired-assignment.png).

Actual short geometry1268×632, not the larger requested Wayland size; content
scrolls without shrinking text. Keyboard focus traversal has an actual widget
regression; screenshots/automated native input are not physical-device,
screen-reader or independent novice acceptance. No Windows/macOS or kernel
oracle claim. Current manual notes before this paragraph were authored before
final package gates; this evidence is the later status, not a historical rewrite.

## Deliberately narrow first profile

- Ordered IPv4 filter INPUT/FORWARD/OUTPUT chains, explicit ACCEPT/DROP policies,
  numeric source/destination CIDRs, exact interfaces, TCP/UDP/ICMP and numeric
  TCP/UDP ports. CIDR host bits normalize on command entry; stored prefixes must
  be canonical. Named DNS lookup, custom chains, negation, match extensions,
  interface patterns and service/port ranges are unsupported.
- Command words support literal quotes/backslashes; expansion, shell operators,
  multiline/control bytes and execution outside the simulator are rejected.
  Operations: inspect `-S`, append `-A`, insert `-I`, delete by number `-D`,
  set policy `-P`, flush `-F`, with only the `filter` table. Other operations such
  as `-L` explain the limitation. Select TCP/UDP before port options here;
  explicit match-module loading is not implemented.
- The path model derives local INPUT, transit FORWARD or router-originated OUTPUT.
  It separates link state, route selection by longest prefix then metric,
  forwarding, filter verdict and supplied next-hop/node availability. Existing
  LinuxHost route/link behavior is reused without substituting the router's
  source address for a transit packet's source.
- Only supplied unicast paths are supported. Local-to-local, loopback, multicast,
  limited broadcast and link-local forwarding are refused. Directed-broadcast
  behavior is unmodeled; the authored traffic cases supply ordinary unicast
  endpoints only. No directed-broadcast objective coverage is claimed. No conntrack, reply-stream,
  NAT, IPv6, ARP, live application, real packet capture or kernel parity claim.
- `ip link show/set`, `ip -4 address show`, and the existing numeric `ip -4 route`
  subset operate on virtual state. Route deletion currently requires the complete
  prefix, gateway, device and metric match; shorter real iproute2 forms explain
  the unsupported format. This is not a complete Linux routing command parser.
- Named `sysctl` reads and `sysctl -w net.ipv4.ip_forward=0|1` change simulated
  running state. The only saved forwarding file is
  `/etc/sysctl.d/lab-forwarding.conf`; `sysctl -p` requires that explicit path.
  Other kernel settings and their real side effects are unmodeled.
- `iptables-save [-t filter] [-f /etc/lab-filter.rules]` exports virtual rules.
  `iptables-restore [--test] /etc/lab-filter.rules` parses one complete filter
  table; dry validation does not apply it. Fixed-file `cat` is supported.
  Counter fields are unmeasured zero placeholders, not traffic evidence.
  No host files, startup services or actual reboot are accessed.
- Saved files, unsaved editor drafts and live state are separate. Queries and
  exports retain dirty drafts; replacement requires explicit Reload editor.
  Worked practice never mutates the independent assignment or earns its credit.

## Bounds and grading foundation

Commands:2KiB/64literal words;96filter rules total;32KiBcommand output retained
in a128KiBconsole. General path facts:8links/32static routes; this administration
adapter retains LinuxHost's stricter2interfaces/8routes. Saved filter32KiB,
forwarding file4KiB, state128KiB, scope512KiB and session1MiB. Actions edit scratch
state and commit after validation; malformed commands cannot partially apply.

Exact typed-predicate comparison uses at most4096regions/1,000,000steps and
returns a concrete counterexample or an honest complexity diagnostic. It never
converts a budget refusal to success. Comparison is scoped to declared numeric
header/interface domains; it proves neither a network exchange nor kernel safety.
Existing checks cover exact subtraction, first-match/split alternatives,32tables
independently enumerated across144typed headers,128generic small-domain cases and
budget refusal. The dedicated integration harness preserves the actual private
math assertions and checks the linked production API without the large library-
test binary. No inclusive current Rust runtime acceptance follows.

## Three distinct lessons registered and natively checked

1. **Local versus transit policy:** replace broad/misdirected INPUT/FORWARD
   permits with precise LAN administration, remote web and DNS rules. Equivalent
   subnet splits are intended to pass; permitting only displayed clients must fail.
2. **Forwarding and path recovery:** preserve a correct filter, diagnose disabled
   forwarding, then repair an unavailable more-specific next hop. Correctly deleting
   that route to use the available default is a legitimate supported alternative.
3. **Saved policy recovery:** compare correct live state with unsafe saved files,
   export/edit, validate, apply and verify both live and reapplied outcomes.

Prerequisites follow existing host routing and SSH policy administration. Private
preparation is under `.build-artifacts/continuation/resume-20261113/next-linux-router/`.
`task-preparation.json` and `presentation-preparation.json` contain separate
practice/assignment values and actual typed actions, essential concepts, worked
commands, action/observation guidance and independent requirements. The matching source-owned scenario records are registered; private drafts are not
native screenshots or completed teaching-inventory rows.

`tests/fixtures/linux-router-authored-preparation-v1.json` and
`tests/linux_router_authored.rs` prepare six runtime regression checks. The six authored checks now pass, along with four source/persistence/catalog
checks and the existing administration/workspace checks (24total). Goal/State and
pack/progress binding are integrated. Native terminal/editor/trace controls are
in source; rendered inspection, final lint, fresh content tooling and candidate
verification remain. Three source labs are registered; no new212.1objective
credit or delivered-candidate acceptance is claimed.

## References checked2026-11-14

- [Current LPI201/202 objectives,212.1](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [Upstream-authored iptables1.8.13 manual via man7](https://man7.org/linux/man-pages/man8/iptables.8.html)
- [Extension semantics](https://man7.org/linux/man-pages/man8/iptables-extensions.8.html)
- [Restore semantics](https://man7.org/linux/man-pages/man8/iptables-restore.8.html)
- [Linux IP sysctls](https://docs.kernel.org/networking/ip-sysctl.html)
- [Historical Netfilter traversal explanation](https://www.netfilter.org/documentation/HOWTO/packet-filtering-HOWTO-6.html)
- [Historical NAT explanation](https://www.netfilter.org/documentation/HOWTO/NAT-HOWTO-6.html)

The HOWTOs describe Linux2.4history; they are explanatory context, not current
CLI proof. The direct upstreamv1.8.11Git manual URL was unavailable through the
web tool; man7 identifies its1.8.13manual as obtained from upstream. No manual
implementation code is copied; no dependency/license change. No privileged
iptables/kernel oracle has run or is proposed. Primary-source review and
deterministic model tests are not real-kernel, IPv6 or complete212.1acceptance.

## Actual rendered evidence (local ignored files, not mockups)

- [Local/transit Introduction](../.build-artifacts/native-runs/run-zl8byw1c/screenshots/workspaces/router-0-introduction.png)
- [Guided inspection and trace](../.build-artifacts/native-runs/run-zl8byw1c/screenshots/workspaces/router-0-guided-policy-trace.png)
- [Route recovery Assignment](../.build-artifacts/native-runs/run-zl8byw1c/screenshots/workspaces/router-1-assignment.png)
- [Saved-policy practice](../.build-artifacts/native-runs/run-zl8byw1c/screenshots/workspaces/router-2-guided-policy-trace.png)
- [200%editor](../.build-artifacts/native-runs/run-zl8byw1c/screenshots/workspaces/router-2-dark200-editor.png)
- [200%Save controls](../.build-artifacts/native-runs/run-zl8byw1c/screenshots/workspaces/router-2-dark200-save.png)
- [Actual cold restored assignment](../.build-artifacts/tmp/native-reopen-4706b363d98540893f799c59/read/reopen-router-reader-assignment.png)
- [Actual cold repaired assignment](../.build-artifacts/tmp/native-reopen-4706b363d98540893f799c59/read/reopen-router-repaired-assignment.png)

Screenshots precede final direction-glyph/wording-only corrections. Installed
candidate evidence is recorded separately after that actual build runs. Expected
results, actual command output and read-only policy analysis remain distinct.
