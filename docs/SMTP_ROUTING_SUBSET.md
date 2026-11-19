# SMTP destinations, relay boundaries and queued recovery

Three implemented LPIC-2 assignments, selected in the normal467-lab/185-unit
catalog. Development native screens and cold reopening pass as recorded below. Final source
gates and the new467candidate now pass (see installed evidence below). The protected467SMTP package is the latest
verified candidate; never re-attest it against later source.

## Teaching and authored practice

Library → LPIC-2 → **SMTP: destinations, relay boundaries and queued recovery**.
Prerequisites are local mail/aliases and host networking; navigation is advisory.

| Stable ID | Independent practical task |
| --- | --- |
| lpic2-smtp-destination-role | Repair mistaken local-domain classification without breaking operator delivery or relay refusals. |
| lpic2-smtp-relay-boundary | Replace all-client trust while preserving incoming/local and multiple trusted outgoing cases. |
| lpic2-smtp-queue-recovery | Repair a repeated next-hop path and deliver both original queued samples without replacing their identities. |

Each Introduction teaches the concept and a separately named guide configuration.
Try example places the exact actions beside the editor and command workspace;
expected observations are prose, actual command results and simulator observations
are labeled separately. Insert command never executes. Assignment retains its
fault and full requirements. Challenge does not show the guide by default.
Optional help contains hints, an explicitly revealed worked approach and scope.
No numeric variants count as separate authored labs. Independent novice acceptance
is unverified; the owner's earlier assisted review was not a novice study.

## Literal configuration subset

The in-process parser supports8 KiB/96 physical lines/16 list entries per represented
setting. Full-line comments, continuations and last-definition-wins are explicit;
duplicate definitions produce warnings. Document inspection retains explicit
logical values before semantic validation. Unknown settings, variables, maps,
quoted values, inline#characters, IPv6 and unsupported syntax produce honest
subset diagnostics rather than fabricated Postfix errors.

Seven settings must be explicit:myhostname,mydestination,mynetworks,relay_domains,
relayhost,smtpd_relay_restrictions,parent_domain_matches_subdomains. Real Postfix
has defaults; the exercise does not invent omitted defaults. The last setting is
supplied empty, making these literal domain matches exact, not a universal default.
Bracketed relay names/IPv4addresses may have numeric ports; no MX selection.
Trust lists use explicit canonicalIPv4networks/0–/32 or bare host addresses.
Nonzero CIDR host bits, dotted masks and named services are outside this subset.
This policy capability does not change subnet exercises' conventional/31–/32exclusion.

Restrictions:permit_mynetworks,reject_unauth_destination,
defer_unauth_destination,reject. Trusted permission terminates the sequence;
authorized-destination guards continue. A following reject can therefore still
deny an authorized destination. Require a represented guard/reject; SASL and other
rules are unsupported. Protected settings compare semantic domain/endpoint values,
network unions and policy outcomes, allowing reorderings, equivalent prefix splits,
case normalization and equivalent supported rule sequences.

## Finite routes, virtual files and sample mail

Up to6supplied endpoint nodes/16 literal destination routes/128 KiBsource facts;
up to16 hops per observation. Endpoints are mutually reachable unless marked
unavailable. Each connecting relay presents its own IP, not the original client.
Cycle detection tracks server and immediate peer: returning with a different peer
can reject instead of cycling. Literal names/IPs/ports match only supplied nodes.
Missing routes never query DNS. Unvisited broken settings do not poison a path.
Local recipient resolution reuses the unchanged bounded local_mail model; actual
mailboxes and recipient failures are retained separately.

Queue source supports1–16 immutable sample envelopes/subjects/bodies,256 KiBfacts.
Runtime has separate saved/running text and bounded queue/delivery records,384 KiB.
Saving can retain a malformed virtual file; failed reload leaves running settings
unchanged. One flush tries each pending sample owned by the selected server once.
Already accepted queued mail bypasses its owner's new SMTP-admission policy;
downstream servers still evaluate the connecting relay's actual address.
Delivered, partial, failed and deferred are distinct. Empty queue is not delivery.
A finite cycle finding is not a fabricated SMTP bounce/Received-limit/timer result.

Reset sample mail only is an explicitly labeled simulator helper: it restores
only the selected example/assignment's original samples, preserving both file
sets, drafts and history. It is not a Linux/Postfix command. An ordinary retry does
not duplicate delivered mail or silently restore permanently failed samples.

## Commands on the selected simulated server

| Input | Represented effect |
| --- | --- |
| postconf -n | Show saved explicit literal parameters and duplicate-definition warnings; not running settings or proof of correct delivery. |
| postfix reload | Parse supported saved settings and apply atomically on success; never changes another server or retries mail. |
| postqueue -p | List only pending samples owned here, IDs, represented subject/body bytes, sender, recipient and last deferred reason. No invented real arrival time or full RFC message size. |
| postqueue -f | Request retry; model advances one finite attempt immediately and labels observations separately. Exit0alone never proves delivery. Real Postfix requests asynchronous delivery. |

No shell, sudo, paths, pipes, expansions, extra flags, real daemon or host command.
Real postfix check examines ownership/permissions and missing directories; those
checks are unsupported, not replaced with a pretend routing validator. Native
Check configuration verifies the task's delivery/refusal requirements instead.
Console input4 KiB; retainedUTF-8history32 KiB. Unsupported input cannot mutate state.

## Grading, preservation and implementation

Each task uses9 fresh positive/negative probes plus2 queued delivery requirements,
not command matching. Only an actual policy refusal passes a required-refusal
case; unavailable routes/cycles do not. Both saved and running routes must satisfy
the probes. Queued outcomes need actual retained delivery records and still-valid
saved/running delivery behavior. A correct negative case cannot hide failed
positive cases. Editing invalidates checked results; printed success earns nothing.
Other supplied settings retain their semantic behavior.

Source-bound version1 sessions keep separate practice/assignment drafts, server
selections, command histories and queues. No old scenario IDs, state shapes,
evaluator19 or progress schema changes. A session must match its source/version;
unknown/malformed/oversized work is refused with originals retained. Task384 KiB,
scope512 KiB,session1MiB,action32 KiB. Closed authoring JSON is never Linux syntax.

Focused evidence so far:44configuration/routing/queue/workspace/console/source
checks,5authored-method/alternate/incorrect/partial/reset/resume checks,4 normal
catalog/preservation checks and7 widget/plan/capture/cold-plan checks pass. The final
wording improvement to recipient diagnostics awaits a fresh check. One earlier
new test fixture failed to compile due to mutable map indexing; corrected without
loosening assertions. Cold-planR2wrong include path corrected;R3 all7pass.
Native plans are not observed native acceptance. No full inclusive Rust repeat,
actual Postfix oracle, screen-reader, physical-input, novice or other-OS acceptance.
No Postfix executable is available; none installed or invoked. No new dependencies.

## Objective scope and references

LPIC-2currentv4.5,201-450/202-450reconfirmed2026-11-13:
https://www.lpi.org/our-certifications/exam-201-202-objectives/
This is partial211.1configuration/relay/queue practice, not full objective coverage.
TLS/SASL, quotas, virtual mail domains, daemon installation/service/ownership,
master.cf, actual SMTP sessions/MX queries, monitoring logs, Sendmail/Exim and
real queue storage/timing remain open.211.2Sieve and211.3mailbox access are separate
planned units. Existing local-alias lessons are preserved, not counted again.

Postfix primary references checked2026-11-13:
https://www.postfix.org/BASIC_CONFIGURATION_README.html
https://www.postfix.org/SMTPD_ACCESS_README.html
https://www.postfix.org/postconf.5.html
https://www.postfix.org/postconf.1.html
https://www.postfix.org/postqueue.1.html
https://www.postfix.org/postfix.1.html
Canonical direct fetches returned402; indexed text and maintainer source copies
were readable, including README_FILES/BASIC_CONFIGURATION_README,
README_FILES/SMTPD_ACCESS_README and man/man1/{postconf,postqueue,postfix}.1under:
https://github.com/vdukhovni/postfix/tree/master/postfix
Mutable checked-date references; no upstream code imported, distribution pinned,
actual Postfix comparison or independent fidelity seal claimed.

## Literal-pattern correction —2026-11-14

Final primary-source review found that configured destination patterns must retain
a final dot rather than being normalized like a recipient domain. Case folding
remains supported. Recipient-domain rewriting strips one final dot and rejects
multiple final dots; a configured mydestination/relay_domains entry ending in a
dot therefore does not silently become a matching undotted entry. The regression
covers both roles and keeps them distinct from envelope normalization. The
alternate solution test still uses a genuinely equivalent reordered uppercase
list. No existing lab/progress format changes.

Primary references checked2026-11-14:
https://www.postfix.org/ADDRESS_REWRITING_README.html
https://raw.githubusercontent.com/vdukhovni/postfix/master/postfix/src/global/resolve_local.c
https://raw.githubusercontent.com/vdukhovni/postfix/master/postfix/src/util/match_ops.c
https://raw.githubusercontent.com/vdukhovni/postfix/master/postfix/src/util/match_list.c
This is source comparison, not execution of a real MTA or an independent seal.


## Actual Fedora development evidence —2026-11-14

Normal integration is467labs/185units; all464older scenarios/184units and the first
117LPIC-2presentation values are exact. Runtime59plus updated6-console checks
cover60unique scoped tests;7widget/plan/capture/cold-plan checks and5normal-catalog
UI checks pass. FinalR3passes1311inclusivePython checks and one-job all-target
Clippy with warnings denied. Final9lesson/catalog and7UI checks pass; strict
inventory467/462updated/5already and coverage508Partial/22Supporting/710Open pass.
No full inclusive Rust runtime repeat. Final installed467candidate is not yet built.

Actual development nativeR2passes234actions/39images. All9R1teaching originals were
inspected; R2all3corrected introductions, clarified assignment/failure and full
150%Light/200%Navy editors were inspected.100%1920×1080and enlarged946×929views
keep shared fields/buttons readable, permit normal page/editor/output scrolling
and retain focused command entry. Prose labels use native-supported characters;
worked fields/values fit and large editor evidence anchors the actual viewport.
Normal two-process R2cold reopen passes:3670430/3670721exit0,46.990s/46.504s.
Both unsaved drafts, deferred original queues, selected servers, unfinished commands,
history/raw/provenance/preferences restore exactly before actual save/reload/retry
and fresh grading; all6cold original images inspected. These are injected native
input and rendered observations, not a physical-keyboard/screen-reader/novice study.

The first cold run received externalSIGTERM(exit143); its interrupted writer and
logs remain.33samples:min15.833GiBavailable,fullPSIavg10=0,OOMdelta0; cleanup left
both markers absent. Signal origin is unknown. A new private retry passed without
raising limits or changing the machine. Two successful smoke launches plus two
cold launches are finite observations, not a startup-reliability guarantee.
All evidence/profiles/retained native ELFs stay ignored under project-disk artifacts;
no personal progress, installed owner app, services, GPU/swap/theme or model changed.

## Installed Fedora candidate evidence —2026-11-14

The distinct467candidate at dist/teaching-review-20261114T065801Z is verified.
Use open-smtp-review.sh there for a separate0700profile/all4XDG roots. Private
464→467install/upgrade/uninstall/reinstall and synthetic original-nine retention
pass; packaged native234actions/39screens and normal two-process cold reopen pass.
7installed originals and all6cold originals inspected, including enlarged Light/Navy.
Exact unsaved separate drafts, deferred sample queues, selections/commands/history/
raw/provenance/preferences survive before native repair/fresh grading. Processes
3690096/3690336exit0(32.566s/30.899s). Current/final selected identities agree and
3025inputs are retained privately. Never attest this package against later main.
Final1311PythonR3/one-job all-targetClippyR3 and strict inventory/coverage pass.
See CHECKPOINT.md for identities, launcher, memory evidence and acceptance limits.
Screens stay private: .build-artifacts/native-runs/run-c8t6x34v/screenshots/workspaces/
and .build-artifacts/tmp/native-reopen-bcb6f5dc80335d2f2fcb6cc0/.
