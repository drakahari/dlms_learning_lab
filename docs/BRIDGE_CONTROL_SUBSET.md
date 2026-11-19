# Bridge control keyfiles — contract before implementation,2026-11-08

Part of the existing Linux interface roadmap, not a lesson or coverage claim.
The341/119Fedora candidate stays unchanged. `BRIDGE_FABRIC_SUBSET.md` records the
actual three fixed offline keyfile normalization observations and pure forwarding
checks. Nine new controller-parser and53existing checks pass in R09; R10caught
two collapsible-if lints, corrected normally with no suppression (repeat pending).

One source-bound bridge controller and two named Ethernet candidates use distinct
flat filenames, UUIDs and interface names. A controller keeps the explicit typed
manual IPv4/STP/port-start properties. Each port keeps its actual identity, label,
autoconnect and optional literal controller binding. Both controller and port-type
keys must be present together to bind a port, with port-type=bridge. Bound ports
may omit IP groups; an unbound candidate requires disabled IP methods. Empty
Ethernet/bridge-port/proxy groups and disabled IPv6/default addr-gen-mode follow
the actual normalized shapes. Nondefault port cost/priority/hairpin, other kinds,
UUID controller lookup, nesting, partial bindings, unknown options and escapes are
unsupported. Last repeated supported key wins; malformed input is never executed.

Automatic future selection starts the controller if its autoconnect is true.
Explicit autoconnect-ports=1 starts all matching bound ports regardless of each
port's own autoconnect flag, as the official connection property specifies.
Independent port-driven activation/controller discovery remains an explicit
limitation; it must not be approximated as a missing path. An unbound candidate
does not become a bridge port merely by existing. No physical-primary selection
is imposed: all attached bridge ports can forward in the bounded acyclic fabric.

Supplied current membership is explicit and source-validated. An owned current
snapshot retains its original parsed plan and members when future files are
edited. No cached active path, invented membership or host activation. Physical
up/down is evaluated separately by the bridge fabric. A profile's typed signature
supports semantic preservation rather than exact text or absent-value equality.

Bounds: eight virtual files,16KiB/256lines per control file, three explicit source
bindings, two member names, closed source/current JSON. No subprocess, real file,
network, device, credentials, daemon, boot or timing behavior. Future policy/session
and native integration, grading, separate practice and progress preservation must
be verified before a bridge lesson is authored. This contract is not those gates.

Sources checked2026-11-08:
[connection](https://networkmanager.dev/docs/api/latest/settings-connection.html),
[bridge](https://networkmanager.dev/docs/api/latest/settings-bridge.html),
[bridge-port](https://networkmanager.dev/docs/api/latest/settings-bridge-port.html),
[keyfiles](https://networkmanager.dev/docs/api/latest/nm-settings-keyfile.html).
Original bounded modeling only; no upstream implementation copied.

## Current capability gates

R12library/R13app builds pass. R14all54focused checks pass:9control roles/snapshots,
9controller parser,10bridge forwarding and26existing Ethernet/bond checks. R15
library/R16app/R17three new test-targetClippy gates pass. Port tests cover forced
false/true flags, genuinely unbound candidates, incomplete binding, explicit
independent-startup limitations, owned current/future separation, equivalent
keyfile spelling, source identity, unsupported port/IP properties, bounds and
foreign/duplicate current members. This is not host activation or a lesson.

## Next composition contract, before implementation

Source facts must match the bridge/port identities to the untagged topology and
map every modeled bridge-LAN IPv4 next hop to a distinct provided peer endpoint.
The existing IP model still decides address/route selection first. Then a bridge
path must deliver the actual next hop in both directions; bridge reachability must
not rescue an incorrect IP/route. Separate peer-to-peer broadcast checks demonstrate
forwarding between NICs without claiming IP routing or MAC-table conformance.
A stopped bridge may leave external same-segment peers reachable. A missing/down
port isolates its segment; it is not another active-backup failover exercise.
Current owned IP settings, controller plan and explicit membership must remain
separate from newly parsed future files. Foreign snapshots, colliding host/peer
addresses, undeclared next hops and nested VLAN/bond/IP-on-member roles fail
honestly. No lesson is registered until policy/session/grade/persistence integration
and meaningful alternatives/partial/negative behavior have passed.
