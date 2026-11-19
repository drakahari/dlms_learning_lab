# NetworkManager static-Ethernet subset

Roadmap: LPIC-1v5.0/109.2, checked2026-11-02. Verified candidate247/68 now includes three lessons with scoped native gates
recorded in NM_PROFILE_LESSONS.md. Parser tests alone are not practical curriculum credit.
Do not import real profiles or contact a host NetworkManager service.

## Declared parser subset, before dependent authoring

Original in-process `nm_profiles` code uses no host file/process/network access.
It parses at most16KiB/256lines of ASCII keyfile data. Property names/values are
bounded; unknown sections/keys refuse honestly. # begins a whole-line comment.
Groups and property names are case-sensitive. Whitespace, repeated known groups
and last-assignment-wins scalar properties are supported. Escaped/multiline values,
Unicode and control characters are outside this initial subset. Empty Ethernet
and proxy groups are accepted; their nonempty unsupported properties refuse.

Required connection fields: id, supplied UUID, type=ethernet (or its documented
802-3-ethernet name), and one literal interface-name. Optional autoconnect defaults
to true and autoconnect-priority to0; priorities range-999..999. The separate policy layer below implements a narrow, settled selection model;
real activation, timestamps and interface operations remain outside this subset.

IPv4 method=manual and one explicit address1=ADDRESS/PREFIX use conventional
private/documentation /1–30 usable addresses. /31,/32 remain excluded special
cases. A numeric separate gateway overrides a legacy inline comma/semicolon
gateway. Empty gateway is an error;0.0.0.0 clears it. never-default suppresses
the effective gateway. The original text remains available to the future editor;
parsed fields describe effective settings. route-metric=-1/omitted retains an
unresolved automatic metric, never a guessed universal value. Zero is explicit.
At most8 routeN entries support network/prefix,gateway[,metric]; /0 and /32 route
destinations are distinct from excluded interface /31-/32 cases.

This isolated IPv4 subset requires the supplied IPv6 method=disabled setting.
That is an exercise boundary, not a recommendation for actual host policy.
Wi-Fi, DHCP, multiple addresses, DNS/resolver merging, authentication/secrets,
permissions, controllers, rules/tables, route attributes, broad defaults and
other methods remain unimplemented. Unsupported is a simulator limitation,
not necessarily an error in a valid real NetworkManager profile.

## Capability gates still required

Before lessons: test the parser, exact supported equivalents, malformed/unsupported
input and resource bounds. Compare fixed synthetic profiles with installed
nmcli's documented --offline mode, never a live daemon or learner code. Then
model saved/loaded versus active profiles, non-preemptive autoconnect and supported
selection rules. Do not invent filename-based profile preference or timestamp ties.

The existing transient `linux_host` core stays unchanged. Its implicit connected
route metric0 cannot silently stand in for NetworkManager's profile metrics:
route-metric also affects connected routes. A separate bounded policy projection
must represent that, or refuse any context it cannot faithfully evaluate. Reuse
its IPv4 mathematics and physical facts without altering old source/progress.
No dependency or major platform replacement is needed for the proposed subset.

## Primary references and observed investigation

Read2026-11-02:

- [LPI current5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/):109.2 calls for configuring NetworkManager Ethernet/Wi-Fi; systemd-networkd is awareness.
- [Official nmcli manual](https://networkmanager.dev/docs/api/latest/nmcli.html).
- [Keyfile format](https://networkmanager.dev/docs/api/latest/nm-settings-keyfile.html).
- [Connection properties](https://networkmanager.dev/docs/api/latest/settings-connection.html).
- [IPv4 properties](https://networkmanager.dev/docs/api/latest/settings-ipv4.html).

A fixed read-only version command observed nmcli1.56.1-2.fc44. Nine original
synthetic stdin profiles were investigated with --offline connection modify;
HOME/XDG/TMP were private project-disk directories, both D-Bus addresses pointed
to absent private sockets, and CPU/address-space/output/time limits were set.
No daemon, host profile or network was accessed/changed. Eight normalized; empty
gateway was rejected. The first investigation exited1 because it initially
expected all profiles to normalize; its genuine failure/source/receipts remain
retained. The parser was corrected to the observed empty/zero/never-default
semantics. tests/fixtures/nm-profiles-reference.json records those actual outputs
and identities. Nine parser groups pass, including a comparison with all eight genuine
normalized profiles and the actual rejection. This narrow property comparison
does not establish startup/activation behavior; those capabilities are next. It is not daemon/runtime conformance, new
practical coverage, other-platform or novice acceptance.

## Policy projection foundation (no lesson credit)

nm_policy is now exported and passes9focused groups alongside9parser and12
retained transient-host groups; scoped Clippy passes. It loads a bounded virtual
bundle, keeps selected settings as owned snapshots, applies unambiguous per-device
autoconnect priority without preemption, and distinguishes fresh startup from an
existing selection. Timestamp ties, duplicate UUID precedence, carrier-loss/retry
lifecycle and other unrepresentable contexts refuse explicitly. No NM daemon or
real activation/rollback behavior is claimed. This is a settled static IPv4
configuration projection. The source-bound workspace and three authored lessons
are integrated and pass the scoped source/native/package gates recorded below.

Connected, default and supported static routes use the actual profile metric or
an explicit supplied per-interface automatic metric. Connected metrics are not
silently zeroed. The adapter selects first, then reuses the unchanged host core
only to check physical facts along that selected path. A regression demonstrates
a metric50 manual route winning over a metric100 connected route, while keeping
the old transient core unchanged. Equal-prefix/metric multipath and never-default
combined with an explicit manual /0 route are declared unsupported. Overlapping
connected networks retain the existing model limit; no valid real Linux feature
is mislabeled as supported when it cannot be represented.

Policy load is an atomic validation operation in this simulator; it does not
claim real NetworkManager's partial per-file error/loading or activation failure
lifecycle. No reference daemon/runtime comparison was performed. New assignments
must stay within these declared boundaries and use multiple deterministic cases.

## Source-bound workspace gates —2026-11-02

`nm_workspace` stores separate raw virtual-file bundles, selected filenames and
current tested-draft marks for practice and assignment. Edited bundles are parsed
and tested afresh, never matched to an expected command string. The bounded phase
runner keeps supplied current selections as owned snapshots, re-reads edited saved
profiles without preemption, then derives a separate fresh-startup selection and
route outcomes. There is no claim of real restart, activation rollback or daemon
conformance. Stop, unfinished runs and edits cannot retain current credit.

Eight workspace groups pass along with the nine parser and nine policy groups.
Five authored-source/progress groups pass, including the exact244-lab/67-unit
predecessor comparison, all taught experiment/repair paths, legitimate alternatives,
wrong settings, separate example credit, safe save/export/restore and selected reset.
Definitions, collections, filenames, documents, UUIDs and imported sessions have
explicit byte/item limits; unknown source/metadata and unsupported syntax refuse.
Separate247/T214012Z is now verified by packaged content and all135native steps. See NM_PROFILE_LESSONS.md
for the evolving scoped evidence; the earlier244 candidate is retained.
