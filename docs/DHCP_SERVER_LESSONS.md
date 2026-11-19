# Linux DHCP configuration lessons — scoped Fedora candidate verified

Continuation2026-11-02 of LPIC-2v4.5/210.1. The review source now registers244
labs/67units:72CCNA,63LPIC-1,43LPIC-2,66Bash. The separate244 candidate passes content/native checks below; previous242
is retained.
This is a narrow practical slice, not full210.1 or current deployment advice.
ISC DHCP is EOL. No ISC code or manual prose is redistributed.

## Two distinct assignments

Unit **DHCP configuration: capacity, scope and client settings** follows Linux
host networking. It uses a configuration editor and actual virtual client
exchanges, rather than new numeric versions of the CCNA pool-control lessons.

| Stable ID | Assignment | Important distinction |
| --- | --- | --- |
| lpic2-dhcp-dynamic-ranges | Repair dynamic capacity across empty and occupied-address cases while preserving a management lease. | A three-address range serves the first population but exhausts in the second; valid alternate ranges pass. |
| lpic2-dhcp-option-scope | Repair shared DNS/default timing and subnet-specific router/DNS options; inspect a renewing and an untouched client. | Current held options are separate from edited configuration; only the scheduled client renews. |

Introduction teaches DHCP-provided settings, network versus inclusive dynamic
range, capacity, scoped defaults, local overrides and held leases as appropriate.
Worked networks/clients differ from assignments. Try example retains unfinished
work, contextual editor/Run/observe actions and a deliberate failing experiment.
Assignment lists all requirements without filling the solution. Help and the
explicit solution disclosure remain optional; Challenge opens independent work.

## Simulation and feedback

`dhcp_config` parses the complete bounded text and compiles a real policy for
`dhcpv4`. `dhcp_server_workspace` runs two independent deterministic client
contexts per lesson. It preserves immutable supplied topology/occupancy/initial
bindings, replaces only a cloned policy and performs typed exchanges. New leases,
router/DNS/duration, range constraints, served subnets and protected held bindings
are graded from actual resulting states. No exact configuration string is needed.
Global versus explicit local equivalents, comments, whitespace and other valid
ranges are accepted where their outcomes meet the task.

Run advances one bounded phase/action per native frame; Stop discards unfinished
results and keeps the draft. At most3cases,16actions each and64ticks. The editor
is bounded to16KiB, a source definition to128KiB, a saved session to150000bytes;
collections reject excess items before decoding their bodies. Syntax and
unsupported-configuration diagnostics differ from parsed-but-incorrect client
outcomes. Overall failure retains correct individual case labels and names the
failing cases with specific reasons. Results clear after edits. Expected inputs,
actual exchanges, unchecked work and optional worked configuration are labeled.

The model supports numeric options, scoped default/maximum durations and one
inclusive dynamic range per conventional /1–30 subnet. It does not run dhcpd,
read host files, execute includes/hooks, use a network or change services. Unknown
syntax refuses. Renew only accepts a supplied renewal-window/eligible-address
context; the core's fallback ACK is not presented as ISC NAK/cache conformance.
Static host/BOOTP, DHCPv6, shared networks, failover, real daemon/lease files,
cache behavior, packets and full administration remain Open. See
[configuration subset](DHCP_CONFIG_SUBSET.md).

## Progress and preservation

The additive typed State/Goal leaves all existing IDs, sources, graders and
schema intact. Exact predecessor comparison preserves all242 labs/66units.
Sessions retain separate example/assignment drafts and view. Completion requires
a current checked draft and fresh behavior-based replay; imported booleans or old
marks cannot invent success. Catalog-aware save/restore validates source identity
and refuses foreign workspace fields without replacing the existing file.
Selected reset retains other attempts and favorites. Jobs/partial outcomes are
never serialized as credit. Original profiles and installed app remain untouched.

## Observed evidence so far

-17parser/compiler and25 retained DHCP core tests pass.
-9bounded workspace groups pass: actual cases/equivalents, mixed results, Stop,
  changed/unfinished runs, source isolation, bounds and meaningful fixtures.
-6authored/source/progress groups pass (the original catalog-preservation proof
  is reused after later new-only formatting/diagnostic changes), including valid alternatives, wrong/partial
  and malformed configurations, retry, save/load/export/restore/reset, failed-save
  byte preservation and exact242lab/66unit predecessors.
-The retained backup-selection snapshot test also passes with its original
  assertions against the explicit242 snapshot.
-Native editor compilation passes after correcting one String/reference comparison.
-8widget/capture/keyboard/Stop/layout groups pass, including all five supported
  text scales, light/dark and760/1600 widths, all three actual view paint paths,
  unchecked/hidden solutions, mixed correct/failed cases and a minimum editor
  viewport/new-visit scroll regression. A later grant/remaining label change is
  verified in native screens; final scoped Clippy and106Python checks pass.
-Actual final development native74-step flow passes: both separate examples,
  inert Insert, real case-picker selections, wrong options, independent starter,
  reference, stale/damaged configuration, retry, safe save/read and selected reset.
 26captures are retained in .build-artifacts/native-runs/run-rth5sny_/screenshots/workspaces/.
  All six normal teaching screens, mixed-case failure, renewed/untouched timing
  and narrow light200 controls were visually inspected.
-First functional native run26captures run-pbr1_5u0 exposed long IDs and condensed
  configuration. A second run-l_gc8csp exposed a collapsed/stale inner editor;
  minimum font-derived viewport height plus visit-specific scroll identity fixes
  it. The final screenshots show the full first configuration and a usable
  scrollable second editor. Failed diagnostics and before evidence are retained.
-Separate244/T203336Z passes actual content244/67 and all74 packaged native
 steps;26captures run-efa0euss. Representative final failure, renewal timing and
 narrow light200 controls were inspected. Previous242 stays unchanged.

The read-only executable lookup found no dhcpd; no package was installed or daemon
run. Real-daemon syntax/conformance comparison remains unverified. Primary LPI4.5
and ISC4.4 configuration/options references were inspected2026-11-02; their URLs
and EOL context are recorded in the subset and each lesson. This is technical
verification, not an independent novice study. Full-suite, new installation/
upgrade/cold reopening, Windows/macOS runtime, screen-reader and physical-device
acceptance are not inferred. The owner previously used Show worked answer.

Representative final development screenshots (actual native app, local ignored evidence):

- [Range Introduction](../.build-artifacts/native-runs/run-rth5sny_/screenshots/workspaces/teaching-lpic2-dhcp-dynamic-ranges-introduction-dark-100.png)
- [Range guided practice](../.build-artifacts/native-runs/run-rth5sny_/screenshots/workspaces/teaching-lpic2-dhcp-dynamic-ranges-example-dark-100.png)
- [Independent options assignment](../.build-artifacts/native-runs/run-rth5sny_/screenshots/workspaces/teaching-lpic2-dhcp-option-scope-assignment-dark-100.png)
- [Mixed case feedback](../.build-artifacts/native-runs/run-rth5sny_/screenshots/workspaces/dhcp-config-lpic2-dhcp-dynamic-ranges-starter-feedback.png)
- [200% light controls](../.build-artifacts/native-runs/run-rth5sny_/screenshots/workspaces/teaching-lpic2-dhcp-option-scope-assignment-light-200-controls.png)

The final fast model run passes31groups (17parser/compiler,9workspace,5content/
progress); the expensive additive catalog proof passed earlier and is reused.
All242 preceding presentation entries also compare exactly with the verified
242candidate. Both original frozen reference collectors/producers remain
unchanged; six exact new current-only source paths and their negative
frozen-prepare guards pass in both current QA modules. No original capture,
compiled source, seal or observation is rebased.106focusedPython checks and
production/new-test scoped Clippy pass. No dependency or license change.

## Verified separate Fedora candidate (post-build observations)

Open directly; do not run install.sh or replace the installed app:

```sh
/path/to/dlms_learning_lab/dist/teaching-review-20261102T203336Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab
```

The protected existing ccna-presentation-pilot review profile is reused; the
original profile is never written. --ui-smoke used a separate private synthetic
profile. This is candidate verification, not new installation/upgrade/uninstall
or cold reopening. Build-time bundled docs retain their then-pending gate status;
these later observations do not rewrite the immutable binary/archive.

Binary SHA256: b788344e27ae1ec9600eab75d00c738380b453f3a467ac0bc038b4ef7815cffb.
Archive SHA256: 5139ad3924c8e3b4014fe170a1186795a6d3c26c22e481cf4b40a7a16ad99e01.
446exact source pins match.359 ordinary build memory samples: minimum available
16.802GiB, maximum some/full PSI avg10=0, OOM counter delta0; final free swap
2.726GiB. This is not a stress test or a guarantee under other workloads.

Package screenshots: .build-artifacts/native-runs/run-efa0euss/screenshots/workspaces/.
Actual packaged --validate-content confirms244/67. Full74-step native route and
all26captures complete; mixed correct/failed cases, clear actual timing and
light200 Run/Stop were inspected. Original/installed app, themes and original
progress remain untouched. Full curriculum, Windows/macOS runtime, screen-reader,
physical-device and independent novice acceptance remain incomplete. No release,
commit, push, merge, installation or dependency changes were made.
