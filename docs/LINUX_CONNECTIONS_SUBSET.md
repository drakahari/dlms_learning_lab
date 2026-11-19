## Later correction in MAIN: use live filter rule numbers

The retained487candidate can name a specialized row instead of the original
running-table row in some state-policy failures. Verdict grading is unchanged.
MAIN now independently rechecks the witness through the live state-aware matcher
and uses its real rule index. The new regression failed on the old behavior,
then all13workspace/5registered tests passed. Development56-action native run
and both actual originals verify the corrected4/5messages. The old artifact and
its earlier verification stay immutable; include the fix in a later candidate.

- [Correct live rule4](../.build-artifacts/native-runs/run-ur9gf2bb/screenshots/workspaces/connections-0-live-rule-index.png)
- [Correct live rule5](../.build-artifacts/native-runs/run-ur9gf2bb/screenshots/workspaces/connections-1-live-rule-index.png)

# Verified Fedora487 review —2026-11-14

Installed172actions/36captures, private485→487lifecycle and normal two-process
cold reopen pass.14native originals plus6cold originals inspected. Source,
archive, installed ELF and retained selected-input identities agree. See
[checkpoint](CHECKPOINT.md) for protected launcher/checksums and the precise limits.
This actual result postdates the frozen package's pending-verification notes.
OtherOS/device/screen-reader/novice and full curriculum remain incomplete.

Actual installed originals (local ignored evidence):
- [Introduction](../.build-artifacts/native-runs/run-i02a1dxw/screenshots/workspaces/connections-0-introduction.png)
- [Guided practice](../.build-artifacts/native-runs/run-i02a1dxw/screenshots/workspaces/connections-0-practice.png)
- [Independent publication](../.build-artifacts/native-runs/run-i02a1dxw/screenshots/workspaces/connections-1-assignment.png)
- [Actual DNAT reply](../.build-artifacts/native-runs/run-i02a1dxw/screenshots/workspaces/connections-1-assignment-reply.png)
- [Light150 populated command restored](../.build-artifacts/tmp/native-reopen-70bceaa855f7c489ed93d72a/read/reopen-flow-reader-assignment.png)

# Connection-aware IPv4 forwarding and NAT lessons

Status2026-11-14: two registered LPIC-2 lessons, with development native checks passing and installed/cold candidate acceptance
still pending. Current source has487labs/194units; latest verified installed
review remains485. Unit **Linux routers: tracked replies and NAT** follows
**Linux routers: policy, paths and saved configuration**.

- `lpic2-nat-shared-address-replies`: investigate two clients sharing one public
  source, recover tracked replies and adequate port capacity, and restrict new
  requests to the specified service across the entire stated LAN.
- `lpic2-nat-published-service-policy`: recover public-to-private address/port
  translation and first-packet permission, distinguish translated requests from
  directly addressed private requests, and retain established replies/outbound
  policy. NAT changes destination; it does not itself permit forwarding.

Both have essential concept sections, a different worked network, separate
practice commands and traffic trials, and an independent assignment. The terminal
runs typed simulated operations. A traffic trial starts with its own empty
connection table and sample inputs; it does not earn completion. Check work
compares all four state/translation contexts and required NAT properties across
whole stated ranges, then executes every supplied traffic sequence. Any supported
equivalent rule set can pass. Editing or executing a command clears stale checks;
a read-only traffic trial cannot invalidate an already correct configuration.
Practice/assignment input, rules, selected case and observed trial are persisted
independently. Saved trial results must exactly reproduce from saved rules before
loading; invented results are refused. Existing LinuxRouter labs retain their
models, objectives, IDs and progress without migration.

This is bounded partial212.1practice, not full firewall/kernel or exam coverage.

## Declared behavior

- Normal supplied IPv4 TCP/UDP tuples with distinct unicast source/destination
  addresses. No sockets, packet injection, kernel APIs, subprocess or host files.
- A confirmed mapping retains the original incoming tuple, the tuple after
  destination translation, and the outgoing tuple after source translation.
  Replies reverse the mapping at the corresponding points. Later NAT rule edits
  do not retrospectively rewrite confirmed mappings.
- NEW means this modeled connection has not been seen in both directions;
  ESTABLISHED means both directions have been observed. Neither label proves a
  TCP handshake or an application transaction. The first tracked reply is already
  classified ESTABLISHED. A later filter refusal does not undo its observation.
- Ordered numeric filter rules reuse the existing typed parser, matcher and exact
  seven-dimensional region comparer. The new finite state layer supports
  `-m conntrack --ctstate NEW`, `ESTABLISHED`, `DNAT` or their comma-separated
  union. DNAT is an independent destination-translation flag; comma lists match
  either condition. It derives from the retained original mapping, so a source-
  NAT reply does not become DNAT merely because its destination was inverted.
  This enables refusal of direct private-address requests. It retains global rule
  numbers after specialization. Unsupported states/options fail explicitly;
  malformed commands do not partially mutate the policy. Literal arguments are
  re-encoded when delegated to the numeric parser, preventing quoted bad values
  from becoming extra options. Old historical reference collectors remain closed.
- Ordered typed DNAT precedes route and FORWARD-filter evaluation. Ordered typed
  SNAT/MASQUERADE follows that filter. NAT is not a permission to forward.
  The flow result distinguishes a router-local destination, unavailable route,
  disabled interface/forwarding, filter refusal and mapping refusal.
- A rejected first flow creates no confirmed mapping. Malformed/unsupported events
  are atomic. An interface-down event never reaches the modeled tracking point.
  Accepted events mean the fixed route/filter permits an outgoing tuple, not
  actual delivery or application success. Supplied route facts have no ARP,
  neighbor discovery, real peer probe or packet timing model.
- A translated address is one numeric address. DNAT may also select one exact
  destination port. SNAT/MASQUERADE may select one explicit inclusive port range.
  Explicit port changes require a TCP/UDP predicate. Source-port preservation is
  preferred; otherwise a disclosed deterministic search selects an unused tuple.
  Implicit mappings stay within0–511,512–1023or1024–65535. Even a null source
  binding may require a port remap after a prior translation occupies its tuple.
  This is not the kernel's allocation/randomization algorithm.
- MASQUERADE selects the supplied outgoing-interface address. Explicit simulated
  address-loss handling forgets only mappings marked for that interface; there
  is no host event subscription or automatic real-device change.

## Resource and validation bounds

64confirmed mappings;32total NAT rules;96total filter rules;8supplied interfaces/
32routes. Each connection stores fixed numeric tuples and at most15interface-name
bytes. Mapping and rule sequences deserialize through bounded visitors, reject
unknown fields and validate relationships before use. Combined flow state has a
256KiBserialized ceiling. NAT allocation examines at most129ordered candidates
(two possible lookup keys per mapping plus one), or the smaller selected range.
Exhaustion refuses without consuming a connection ID or accepting a colliding
mapping. Normal-flow tuples exclude same-address/local/hairpin translation.

Filter commands retain2KiB/64literal-word input and32KiBlisting bounds. Exact
policy comparison specializes each of four(class,DNATflag)contexts and reuses the
existing4096region/1,000,000step proof. Those bounds are not relaxed; a failed
comparison budget is never success. Authored grading binds declared state/header
and translation domains, use multiple actual sequences, and reject solutions
that handle only a displayed sample client. Library checks alone do not establish native lesson acceptance.

## Verification so far

Final scoped runtime86across10targets, UI/plan7, normal-catalog5 and repeated
post-lint preservation1pass. All-targetClippyR2 warnings-denied and503focused
Python pass, including fresh190/199source guards after the unchanged factory
relocation. Strict inventory487/coverage529Partial/24Supporting/687Open pass.
Selected inputs now frozen for a separate candidate; installed/cold/final
identity acceptance remains pending. No inclusive Rust runtime suite claim.

Registered5 checks and latest12 workspace checks pass: taught methods, supported
alternatives, hardcoded client/broad policy/wrong pool/NEW,DNAT mistakes, partial
work, stale success, malformed records, bounded authoring, trial replay, reset
and disk resume. Seven native-widget/plan gates pass, including Enter, inert
insertion, Challenge guidance, Tab/Shift-Tab and supported widths/scales/themes.
Development native168actions/36captures passed;17originals inspected across all
three views of both lessons, actual bad and repaired outcomes, focused controls,
wide2560×1440 and short1268×632 Light150/Navy200. The second lesson's capture
named guided-reply showed a second request; it is retained only as request
evidence. The corrected final capture plan derives the actual reply index and
asserts ESTABLISHED/reply direction. Installed/cold verification is still pending.

The first sandboxed native attempt refused Wayland connection(NoCompositor),
with0actions/0captures. The same ELF connected and completed with authorized
desktop access. This distinguishes sandbox access from a product startup fault;
it does not establish indefinite launch reliability. No desktop settings changed.

Actual development originals (local ignored evidence):
- [Source-NAT Introduction](../.build-artifacts/native-runs/run-4b7l_qok/screenshots/workspaces/connections-0-introduction.png)
- [Source-NAT guided practice](../.build-artifacts/native-runs/run-4b7l_qok/screenshots/workspaces/connections-0-practice.png)
- [Published service Assignment](../.build-artifacts/native-runs/run-4b7l_qok/screenshots/workspaces/connections-1-assignment.png)
- [Specific failed traffic](../.build-artifacts/native-runs/run-4b7l_qok/screenshots/workspaces/connections-1-guided-first-trial.png)
- [Light150 aligned input](../.build-artifacts/native-runs/run-4b7l_qok/screenshots/workspaces/connections-1-light150-input.png)
- [Navy200 stacked traffic controls](../.build-artifacts/native-runs/run-4b7l_qok/screenshots/workspaces/connections-1-dark200-traffic.png)


Private `next-linux-connections/policy-r1.log`:57focused runtime checks pass
(12tuple,9state-filter,10translation,10composed-flow,7literal NAT parser,9exact NAT
policy). Final `policy-witness-r1.log`:9PASS after actual NAT rule-index mapping and
first-match property recheck. Focused library/six-test Clippy warnings-denied and
fresh190/199current Python source guards pass. Earlier all-target Clippy passed
before later DNAT/proof additions; no new inclusive Rust runtime/kernel oracle.

NAT commands support explicit `iptables -t nat` with -S/-A/-I/-Dby number/-Fand
ACCEPT policies on PREROUTING/POSTROUTING. Numeric DNAT address/optional single port,
SNAT address/optional port range and MASQUERADE/optional --to-ports parse into
actual typed rules. Prefixes/targets/modules outside the declared profile refuse.
Literal word re-encoding preserves quoted values; match arguments cannot become
jump/translation options. Listing round-trips and failed operations are atomic.

The exact NAT proof specializes each TCP/UDP protocol and derives at most96
numeric first-match rules. It checks required public source addresses, actual
changed destination endpoints, and destination results restricted to identity or
the stated endpoint. The latter prevents an unwanted redirect to a different
server from hiding behind a false required-endpoint property. Split source
prefixes are equivalent valid solutions; sample-only clients and broad/shadowed
rules fail. Mismatch witnesses recheck the actual original NAT predicate and
property; reported indices identify NAT rules, not internal derived rows. Boolean
proof ACCEPT/DROP means the property holds/does not hold, **not** a network filter
verdict. Grading/UI must preserve that distinction. Actual flow cases separately
exercise port capacity, replies and state/path behavior.

The485candidate predates these modules and remains bound to its own retained
source; never re-attest it against later main. No new native acceptance or learner progress migration follows from library checks alone.

## Exclusions and authoring obligations

No raw TCP flags/sequence state machine, pickup of pre-existing TCP flows,
fragments/reassembly, ICMP/RELATED/INVALID/UNTRACKED, helpers, timer/expiry,
checksum, namespaces/zones, OUTPUT/local/hairpin NAT, policy routing, IPv6 or
nftables/kernel parity. Numeric predicates may express the older filter's ICMP
rules, but this new connection/flow engine refuses ICMP events; that is no ICMP
connection-tracking coverage. Do not teach every possible tracked connection as
starting with this simulator's own event: real TCP loose pickup differs.

The NAT CLI parses the declared real options; it does not recognize a
solution string to invent success. Separate virtual table restore transactions
must reflect their actual supported scope, rather than promise global atomicity.
Examples and assignments need separate drafts and source-bound progress. Introduce
five-tuples, observed directions and translation stages before assessing them;
keep meaningful terminal investigation, concise contextual practice and full
independent tasks. No new scenario count before content, grading, interaction,
persistence and native checks actually pass.

## Primary references and provenance

Checked2026-11-14:

- [Upstream iptables extensions1.8.13 manual](https://man7.org/linux/man-pages/man8/iptables-extensions.8.html): conntrack state meanings, DNAT/SNAT/MASQUERADE target options.
- [Kernel conntrack sysctls](https://docs.kernel.org/networking/nf_conntrack-sysctl.html): real TCP loose pickup and behaviors excluded here.
- [Netfilter NAT HOWTO section6](https://www.netfilter.org/documentation/HOWTO/NAT-HOWTO-6.html): **historical Linux2.4 explanation**, used only for the enduring hook-order, collision and implicit port-class concepts; its old OUTPUT limitations and obsolete command abbreviations are not current support claims.
- [Current LPI201/202 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/):212.1roadmap context; existing versioned coverage map remains authoritative about actual authored coverage.

No upstream implementation code was imported; original code remains MIT under
the owner's recorded decision. This document describes the declared subset; it is not a certification guarantee
or a real-device experience claim.
