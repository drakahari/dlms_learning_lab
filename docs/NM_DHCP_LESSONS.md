# Automatic Ethernet and protected default routes

Source2026-11-04:265labs/78units,72CCNA/82LPIC-1/43LPIC-2/68Bash. Two distinct
LPIC-1 tasks follow `linux-persistent-networking` in `linux-automatic-ethernet`.
Latest verified separate candidate is265/T20261104T162035Z.

- `lpic1-nm-dhcp-startup`: replace saved fixed addressing with acquisition from
  each supplied lease, keep the requested uplink path and current management.
- `lpic1-nm-dhcp-local-only`: retain a changing local DHCP address without adding
  a default on that adapter; preserve the management uplink and protected profile.
  Merely increasing the unwanted default's metric is an intentional wrong repair.

Introductions teach profile eligibility versus address acquisition, lease versus
saved configuration, and default-route policy with separate worked keyfiles.
Try example uses the existing editor, supplied-input picker and Test startup
profiles; inserting worked data is inert. Purposeful copied-address/metric-only
experiments fail usefully and can be repaired. The full independent task remains
visible with requirements and native editing; answers are optional. Each task
uses three supplied addresses/router cases. These are test inputs, not additional
authored labs or actual DHCP/NetworkManager activity.

[Declared subset and primary sources](NM_DHCP_SUBSET.md), checked2026-11-04. No
DNS merging, live daemon, packets, lease renewal/timeouts/retries, general fallback,
extra auto/static addresses, gateway overrides, real restart or full109.2 claim.

Source gates:7new model groups,26retained static parser/policy/workspace groups
and5authored content/progress/catalog groups pass. The exact263scenario values
and77unit objects/order remain; one new unit is inserted after its prerequisite.
The first content run had a test-only incomplete prerequisite catalog; the fixed
helper loads the original static unit alongside the new tasks before filtering.
The original snapshot proof passed and is reused. Earlier model checks caught an
incorrect duplicate-address fixture and ignored unknown fields on an unavailable
input; the input schema was fixed and an explicit collision regression retained.
No existing assertions were weakened.

New widget/keyboard/scaling groups pass in the first UI run. Two retained
capture-driver regressions caught a changed route key; old flow identity was
restored while the new flow uses its own key. All12new/retained native widget/controller groups now pass (93.37sec), including
keyboard Ctrl+Enter, Stop/Esc/navigation, three-case feedback, inert insertion,
separate drafts, save/read and wide/narrow light/dark100/150/200% controls.
All-target Clippy passes53.10sec. Development native run-46ht3mld completes104actual steps and28PNG. All six
normal teaching screens, specific metric-only failure, saved-work success and
narrow light200controls were opened and visually inspected. The Introduction
expected-result label was corrected afterward; packaged capture will verify it.
647Python checks pass (11.641sec). The final package validates265labs/78units and completes104native steps/28PNG
in run-khs0bxs2. The corrected expected-result label, representative packaged
teaching views, specific failure and narrow light200controls were visually
inspected.513source/data pins and48offline guides matched before post-build
notes; immutable receipts retain binary/archive/build identities. No new
installation lifecycle/full-process cold reopen or whole-app inclusive suite. No novice study,
physical-device, screen-reader or other-platform runtime acceptance is claimed.
Installed app/original progress and historical evaluator9–16 evidence stay intact;
shell semantics remain17. No dependency, system setting, commit/push/merge/release.

Separate Fedora launch:

```sh
/path/to/dlms_learning_lab/dist/teaching-review-20261104T162035Z/dlms-learning-lab-0.1.0-g6b54e34869e0-x86_64-unknown-linux-gnu-fedora-44-candidate-dirty/dlms-learning-lab
```

[Introduction](../.build-artifacts/native-runs/run-khs0bxs2/screenshots/workspaces/teaching-lpic1-nm-dhcp-startup-introduction-dark-100.png), [practice](../.build-artifacts/native-runs/run-khs0bxs2/screenshots/workspaces/teaching-lpic1-nm-dhcp-local-only-example-dark-100.png), [assignment](../.build-artifacts/native-runs/run-khs0bxs2/screenshots/workspaces/teaching-lpic1-nm-dhcp-startup-assignment-dark-100.png). Private technical evidence.
