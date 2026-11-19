## Verified493 IPv6 administration milestone

493 labs /197 units:113 CCNA,154 LPIC-1,146 LPIC-2,80 Bash. Three working scoped-
address/next-hop, router-role/RA and saved-settings lessons use the approved native
teaching pattern. All490prior scenario/196unit objects and143prior LPIC-2 teaching
values remain exact. Existing progress and owner installation stay untouched.

Protected review launch, from any directory:
```sh
/path/to/dlms_learning_lab/dist/teaching-review-20261115T073152Z/open-ipv6-administration-review.sh
```
Library → LPIC-2 → **IPv6 router configuration: scope, advertisements and startup
settings**. Private490→493install/upgrade/uninstall/reinstall, actual installed
150actions/29captures and normal cold writer/read processes pass. WriterPID368204
exit0/23.865s; readerPID368374exit0/22.293s. Thirteen installed originals (all nine
learning screens plus light150/dark200/error states) and all eight cold originals
actually inspected. Exact raw progress/preferences and unfinished separate drafts
restored before native repair. No novice, screen-reader, physical-device or otherOS
acceptance follows from injected input/technical review.

Final28focused Rust/535focused Python checks and all-target ClippyR3pass; prior
foundation/API evidence reused, reruns not summed. No new inclusive runtime suite.
Inventory493=488updated/five already; coverage533partial/24supporting/683open.
Only212.1K3/T2newlyPartial, no complete exam objective or curriculum claim. IPv6
firewall startup persistence and wider proc/template/kernel behavior remain gaps.
Selected source/data/docs/artifact readback matches;3241files/146926916bytes retained
before freeze ENDED. Selected inputs only, not full compiler closure/independent seal.
Bundled docs/per-row notes retain their build-time pending annotations; this is the
later verification record. Never re-attest this artifact against subsequent MAIN.
ELF `b6d1f46fe0bd48b31b16168880a205331e7e010cce44f0e86f2485dd9985e422`;
archive `04be31e569ef1db07cfee7145f46c7c2129081a0b0f089d61908b68f3cca42ff`;
BUILD_INFO `e194732cb945631c9a40ec1b28c5d9d20a60c68922e4cbe3894ce2e08192cb1b`.
19guarded logs/1340samples: min10.309GiBavailable, fullPSIavg10max2.33%, OOMdelta0.
Near-full swap persists; no stress/reservation guarantee. Initial sandbox
NoCompositor failure retained; authorized Wayland runs pass without desktop/security
changes. No Git/push/merge/release/dependencies/system/original-profile changes.
Private receipt: next-linux-ipv6-admin/candidate-r1/r1-verified.json.

# Linux IPv6 router configuration: supported model and lessons

Three authored lessons are now registered in the normal catalog: scoped primary
address/next-hop repair, router-role/advertisement recovery, and saved-settings
recovery. Catalog493/197:113CCNA,154LPIC-1,146LPIC-2,80Bash. All490prior scenario
and196unit objects compare exactly in the registered-catalog regression. Development native149actions/28captures passed; all three views of all three
lessons and light150/dark200controls were inspected. Development cold writer/read processes exit0 (57.275s/59.943s), with exact
raw progress/preferences restored before repair and all eight originals inspected.
Final installed493acceptance remains pending; the verified490candidate stays available.
These lessons do not establish full212.1 coverage. 212.1K3/T2 now have bounded partial practice in the coverage matrix;
remaining capabilities and full objective acceptance stay incomplete.

## Learning progression

Library → LPIC-2 → **IPv6 router configuration: scope, advertisements and startup
settings**. Prerequisite units: Linux host IPv6 and Linux router policies. Three
stable identifiers:

- `lpic2-router-ipv6-scope-recovery`: repair an incorrect LAN primary, distinguish
  repeated link-local next hops by interface, remove an overly broad default and
  restore the entire remote prefix. Different unused host IDs and equivalent
  split routes are accepted; sample-only host routes are insufficient.
- `lpic2-router-ipv6-ra-recovery`: separate route choice from forwarding, configure
  explicit upstream/client-link advertisement policy, observe a supplied message,
  and recover after a zero-lifetime withdrawal. Flags alone invent no message.
- `lpic2-router-ipv6-saved-settings`: edit and Save one virtual settings file, Load
  it separately, observe the upstream message, and test fresh settings against the
  saved file. Live-only repairs, unsaved drafts and a wrong final duplicate fail.

Every lesson has essential teaching and a worked example using different networks
and interface names, separate guided practice, and a complete independent task.
The first two retain terminals/topology; the last adds a configuration editor.
Observations, transit trials and restart tests are explicitly simulator controls,
not Linux commands. Actual command/test results stay separate from worked values.

## Scope and fidelity

`linux_v6_admin` composes the retained literal Linux-host IPv6 parser with a fixed
Linux6.17 settings profile and supplied physical neighbor/onward/host facts. One
primary /48–/126 per interface; supported `ip -6 address show/add/del`, route
show/add/replace/del, `ip link show`, literal `sysctl KEY`, `sysctl -w KEY=VALUE`
and cat of a supported virtual proc setting. No address replace/flush, shell
operators or learner-host process/file/network access. Unsupported forms fail.

Global forwarding writes set interface host/router flags; global-off clears force
flags even when already off. Transit uses global OR ingress force_forwarding.
accept_ra=1 accepts while that interface is a host;2 explicitly permits acceptance
as a router;0 ignores. Validated forwarding writes can purge learned defaults
unless the receiving interface has accept_ra=2, including an idempotent write.

Native observation uses only a supplied available link-local router on the named
interface. Positive lifetime learns a default; zero withdraws. Metric1024/medium
preference is fixed. Flags do not create advertisements. No elapsed clock,
expiry, SLAAC, NDP, DAD, multicast memberships, actual packets or full kernel
behavior. Equal-choice multipath, learned-route editing and same-metric manual/
learned collisions fail honestly rather than choosing an arbitrary gateway.

Whole-route comparison checks every prefix-induced address region, not only the
displayed servers. Selected interface/next hop must match. Equivalent split
prefixes and irrelevant metrics pass. Actual supplied path checks independently
verify neighbor availability, onward reachability and host availability. Transit
headers are supplied independent tests, not application-generated replies or
connection tracking. Link-local traffic remains scope-bound.

The single editor path is `/etc/sysctl.d/70-learning-lab-ipv6.conf`, a simulated
file only. Save validates the supported subset but does not load it. `sysctl -p`
for that exact path loads the saved text. Supports decimal assignments, ASCII
surrounding whitespace, whole-line #/; comments and last-duplicate-wins order.
No arbitrary paths/includes/globs/inline comments/quoted values/ignore-error
syntax or full procps partial-error behavior. The native Save prevalidation is a
conservative editing constraint, not a claim that real procps rolls back errors.

The saved-settings test resets modeled flags/learned routes, applies the saved
file and observes a new supplied message. Supplied addresses/network are already
persistent. This is not a complete boot, systemd/sysctl.d precedence, interface
activation or firewall-persistence model; those remain separate coverage gaps.

## Isolation, grading and progress

8interfaces,8combined gateway routes,4learned routers,96state changes,512byte
commands,4KiB/24line file,128KiBconsole. No interpreter, subprocess or network
escape. State/source identity, successful edits, saved text and cached reports
are replayed/validated. Separate assignment/example drafts, explicit Check,
current-state regrading and stale invalidation; no credit for running examples
or trials. Existing progress formats/IDs remain intact. Reset touches only the
selected attempt; app progress persistence is not Linux startup persistence.

## Current evidence

Private `next-linux-ipv6-admin` logs retain failed attempts as well as passes.
Foundation/composition/source API checks passed. Authored-r2 passes6new tests;
catalog-r1 refused an oversized pretty-printed consolidation. Compact encoding
preserves all parsed objects within the unchanged2MBbound; catalog-r2 passes7
checks including every prior scenario/unit. Shared128pack/512KBregistry bounds
are unchanged. Six widget/capture/cold-plan checks and149development native actions pass.
All nine actual learning screens were inspected. Screen review corrected generic
worked captions and a missing taught file-read picker entry. The first sandboxed
launch failed with NoCompositor despite a visible Wayland socket; the authorized
display-access launch rendered successfully. This is not a passed sandboxed
launch or proof of general startup reliability. Final development native150actions/29captures passes; the changed worked rows
and full200%editor were re-inspected. Development cold reopening passes in two
normal processes, preserving distinct unfinished drafts and current/saved state.
Installed493candidate checks remain pending. Do not infer novice, screen-reader or otherOS
acceptance from these tests. Latest available verified490launcher is in CHECKPOINT.

Primary sources checked2026-11-15UTC:
- [Kernel6.17 IP sysctl documentation](https://www.kernel.org/doc/html/v6.17/networking/ip-sysctl.html)
- [IPv6 forwarding implementation](https://raw.githubusercontent.com/torvalds/linux/v6.17/net/ipv6/ip6_output.c)
- [Address configuration handlers](https://raw.githubusercontent.com/torvalds/linux/v6.17/net/ipv6/addrconf.c)
- [Learned-router route handling](https://raw.githubusercontent.com/torvalds/linux/v6.17/net/ipv6/route.c)

Behavior review informed original project code; no upstream source was copied.
No host sysctl/ip commands, packets, network, privileges or kernel oracle ran.
Tests are model/source evidence, not native UI, novice or platform acceptance.
Next compose existing bounded host configuration, supplied RA observations and
real typed traffic, then source-bound progress and meaningful authored lessons.

Representative development originals (private ignored evidence):

- [Scoped-address Introduction](../.build-artifacts/native-runs/run-wipp6j5k/screenshots/workspaces/v6-admin-scope-recovery-introduction.png)
- [RA Introduction](../.build-artifacts/native-runs/run-wipp6j5k/screenshots/workspaces/v6-admin-ra-recovery-introduction.png)
- [Saved-settings guided practice](../.build-artifacts/native-runs/run-wipp6j5k/screenshots/workspaces/v6-admin-saved-settings-practice.png)
- [Independent assignment](../.build-artifacts/native-runs/run-wipp6j5k/screenshots/workspaces/v6-admin-saved-settings-assignment.png)
- [Dark200full editor](../.build-artifacts/native-runs/run-wipp6j5k/screenshots/workspaces/v6-admin-dark200-editor.png)
- [Cold-restored assignment draft](../.build-artifacts/tmp/native-reopen-76b63a5c7e15d54b53ff2d80/read/reopen-v6-admin-reader-assignment-editor.png)

The final build includes this model/teaching document explicitly. Bundled status
will predate the later installed verification receipt; never re-attest an older
artifact against subsequently edited MAIN. No independent novice, screen-reader,
physical-device or Windows/macOS runtime acceptance.

Installed representative originals:

- [Introduction](../.build-artifacts/native-runs/run-ywsuqxs_/screenshots/workspaces/v6-admin-scope-recovery-introduction.png)
- [RA guided practice](../.build-artifacts/native-runs/run-ywsuqxs_/screenshots/workspaces/v6-admin-ra-recovery-practice.png)
- [Independent saved-settings assignment](../.build-artifacts/native-runs/run-ywsuqxs_/screenshots/workspaces/v6-admin-saved-settings-assignment.png)
- [Full200%editor](../.build-artifacts/native-runs/run-ywsuqxs_/screenshots/workspaces/v6-admin-dark200-editor.png)
- [Restored unfinished draft](../.build-artifacts/tmp/native-reopen-1d89ea0a3f85f477c331f17f/read/reopen-v6-admin-reader-assignment-editor.png)
