Current341T193140Z/R53native review is verified:67focused Rust, current widget/route,
all-targetClippy,931Python and exact packaged content;70native actions/20PNG/nine
opened originals. Serialization-only offline reference does not establish actual
activation/failover conformance. See BOND_VLAN_LESSON.md for exact identity/limits.

# Bonded VLAN control subset

Current2026-11-08: one bonded-VLAN capstone is registered in development341/119.
Corrected focused67Rust checks, current widget and route regressions pass; exact
packaged/native validation is pending. The dated pre-authoring finding below is
retained as history. No earlier incorrect port assumption is fidelity evidence.

## Fidelity correction before authoring —2026-11-08

No bond curriculum has been registered. The final primary-source check found a
wrong assumption in the first unreleased model/tests: autoconnect-ports=1 starts
matching bound port profiles regardless of their own autoconnect flag. R17manual-
derived regression is RED against that implementation; original source/logs stay
in the private bond-control/port-semantics-before directory. Corrected partial
practice uses a genuinely unbound backup candidate (both controller and port-type
absent), not autoconnect=false as a supposed exclusion from forced activation.

Modern controller/member behavior is checked against the official
[connection property reference](https://networkmanager.dev/docs/api/latest/settings-connection.html).
Independent port-driven activation/controller discovery remains explicitly outside
this settled subset when not already covered by forced controller activation;
supported authoring fixtures keep port autoconnect=false. Port bindings, physical
carrier and automatic activation are distinct. Do not teach false as preventing
an explicit controller start. No existing340lab, grading or candidate is changed.

The fixed development-only nmcli --offline tool validates minimal keyfile shape
without a daemon or host profiles/interfaces. First observed1.56.1-2.fc44 accepts
an unbound Ethernet profile with both IP methods disabled; a bound port serializes
without IP groups and with an empty bond-port group. R19retains the validation
failure that exposed this normalization difference; it is not a runtime activation
or failover comparison. The parser accepts omitted/non-applicable port IP methods,
explicit disabled methods, empty bond-port/proxy groups where applicable, and the
known default disabled-IPv6 addr-gen-mode; unknown settings still fail honestly.

---

# Bond control profiles: bounded contract before dependent lessons

Specified2026-11-08 after the verified340VLAN milestone. This is capability work,
not new curriculum credit. Existing205.2plans a bonded VLAN uplink; VLAN-only
practice did not complete it. Preserve all340existing labs, stored source values,
progress and strict Ethernet/VLAN entry points. No new dependency or host action.

## Planned deliberate slice

One provided active-backup bond controller and two provided physical member NICs,
with separate keyfiles. Supplied VLAN interfaces may sit above that controller.
A software VLAN name, controller name and member NIC name are different roles.
Model explicit primary choice with primary_reselect=always, explicit100ms MII
carrier monitoring, and settled source-held link/failure cases. Accept equivalent
supported numeric/text option forms. Do not infer healthy IP/tag delivery from
carrier; reuse actual two-way tagged propagation after selecting the active NIC.

MII monitoring observes carrier, not a host's end-to-end reachability or VLAN plan.
A carrier-up member with incorrect VLAN attachments can still be the active
member. A down primary can select the other eligible supplied member. These are
settled snapshots, not real timers, gratuitous ARP/MAC learning, kernel events,
packet loss timing, LACP negotiation, bandwidth, or all Linux bonding behavior.
No ARP monitoring, other bond modes, nested controllers, bridge, priority policies,
dynamic discovery, UUID-based parent/controller lookup, DHCP/IPv6 bonds, general
nmcli/ip bond commands or host operations until separately implemented.

Parse actual keyfile fields, not a command-to-success table. In [bond], mode,
miimon, primary and primary_reselect are individual dictionary keys; the nmcli
options string is not keyfile syntax. [connection] type=bond/interface-name name
the controller. Separate Ethernet port profiles use literal supplied interface-name,
controller and port-type=bond. Explicit autoconnect-ports=1 on the controller
starts matching bound member profiles even if their own autoconnect is false.
With port autoconnect=false,0 leaves them unselected; independent port-driven
startup is not modeled. An omitted
global default is not guessed. Source-held identities/filenames are retained.
Unsupported properties fail explicitly; bounded groups/repeated keys follow the
disclosed keyfile subset, with line-specific syntax/unsupported/invalid diagnostics.

Provide no IP on controller/member control profiles in this first VLAN-on-bond
slice; controller/unbound methods are explicitly disabled; bound port methods may be
omitted because NetworkManager treats those ports as layer2. Existing manual IPv4 VLAN profiles
carry the host addresses. Keep control eligibility, selected settings and physical
carrier separate. Re-reading saved files must not silently replace a current
control snapshot. Future startup resolves the edited controller/member dependencies
before evaluating VLAN/IP paths. Multiple deterministic cases, meaningful failures,
legitimate alternate settings, Stop, isolated practice and content-bound proof must
pass before any lesson is registered or credited. Do not claim a general daemon,
boot ordering or complete advanced-networking objective from this projection.

Bounds: one controller, two member profiles, two VLAN profiles, existing8file and
16KiB/file limits, finite switch/IP cases and existing64phase Run budget. No shell,
subprocess, host file, socket, namespace, service or real interface activity.

## Primary sources and limits

Inspected2026-11-08:
- [Linux bonding documentation](https://docs.kernel.org/networking/bonding.html)
- [NetworkManager bond settings](https://networkmanager.dev/docs/api/latest/settings-bond.html)
- [NetworkManager keyfile manual](https://networkmanager.dev/docs/api/latest/nm-settings-keyfile.html)
- [Upstream keyfile reader/writer](https://raw.githubusercontent.com/NetworkManager/NetworkManager/main/src/libnm-core-impl/nm-keyfile.c), specifically dictionary handling; rolling MAIN, not a pinned runtime comparison.

Original modeling only; no upstream source/manual implementation copied. No real
NetworkManager/kernel comparison or physical network operation performed. Broader
bonding, bridging/authentication, all-platform, accessibility and independent
novice acceptance remain open. Latest verified review candidate remains340T174340Z;
this document is not completion evidence.

## Reusing immutable source validation

The bonded source has one thread-local successful Definition entry, compared by
complete typed equality. It avoids rerunning the unchanged starter/worked-example
reachability checks on every paint. Any source field change requires full
validation again; failures are never retained. One bounded source entry is held
per thread, with no serialized cache or learner data. Ordinary Ethernet/VLAN-only
validation paths remain unchanged. Draft bounds, current proof, each deterministic
case and behavior/state grading are still evaluated against current data.
`tests/bond_workspace.rs` checks changed invalid/valid sources and a forged proof
after a successful run. This is source-validation reuse, never cached mastery.
