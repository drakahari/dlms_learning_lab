# Saved Ethernet profiles and supplied DHCP outcomes

Capability contract written2026-11-04 before lesson authoring; now implemented
in development source265/78 with two authored tasks. This extends the existing
bounded static-profile engine. Runtime acceptance does not follow merely from
this document. The263candidate remains the latest verified build.

The saved method is typed: manual addressing or automatic addressing. The
original static `nm_profiles::parse` entry point retains its narrow contract;
a separate settings parser admits `method=auto`. An automatic profile in this
slice has no extra static address or gateway override. Unsupported combinations
are reported, rather than silently discarding a valid real NetworkManager feature.

Each supplied test has at most two interface-bound outcomes: a conventional
address/prefix and optional router, or explicitly unavailable DHCP. These are
settled sample inputs, not packets, a lease server or a prediction of actual
activation timing. No DNS data, lease renewal, classless routes, retries, alternate
profile fallback, IPv4LL, shared networking or dual-stack lifecycle is modeled.
Automatic activation without a supplied successful outcome gets no address;
it cannot invent connectivity or claim success. Current owned snapshots remain
unchanged when saved files are reread. Fresh selection uses that test's outcome.

`route-metric` applies to connected and received default routes. `never-default`
suppresses the received default while retaining the address and local route.
Supported explicit static routes remain separate. Saved method and actual
resolved settings remain distinct; merely typing one sample address is not DHCP.
No daemon, host profile, real network, subprocess or dependency is accessed.

All existing keyfile/file/count/line bounds, per-case incremental Run/Stop,
separate practice/assignment drafts and state-based grading remain. New explicit
automatic-method objectives do not redefine existing goals or fabricate credit.

Primary sources checked2026-11-04:

- [NetworkManager IPv4 properties](https://networkmanager.dev/docs/api/latest/settings-ipv4.html): method, never-default and route-metric.
- [Keyfile format](https://networkmanager.dev/docs/api/latest/nm-settings-keyfile.html): grouped properties and automatic Ethernet example.
- [Connection properties](https://networkmanager.dev/docs/api/latest/settings-connection.html): autoconnect and preference.
- [LPIC-1 current5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/):109.2. Full objective coverage remains incomplete.

This is an original in-process projection informed by documentation, not a
NetworkManager implementation or runtime conformance claim. Prior fixed offline
manual-keyfile comparisons remain intact; no new live-daemon comparison is claimed.

New tasks may name protected saved profiles. Their parsed supported settings must
remain equivalent to the supplied starter; comments, spacing and documented
spelling aliases may differ. Old tasks omit this optional field, retain their
original outcomes and serialize without it. New explicit goals check automatic
acquisition and the absence of an interface default; they do not alter old goals.
