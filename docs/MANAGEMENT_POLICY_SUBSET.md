## Current implementation status —2026-11-09

The two source-held management-policy lessons are integrated in the356-lab
development catalog, with focused model/storage/widget checks and actual
development-native verification. Separate candidate admission follows. Earlier
foundation notes below are historical gates, not current absence claims.
[Lesson scope and evidence](MANAGEMENT_POLICY_LESSONS.md). CCNA4.8is Partial
practice; authenticated SSH and5.3remain unimplemented here.

# Remote-management policy foundation

Designed2026-11-09 before dependent lessons. This is a small typed foundation for
the existing CCNA4.8roadmap gap, **not a new registered lesson or coverage credit**.
The admitted354Fedora review package predates this source. Existing351/354data,
progress and all historical evaluator evidence must remain unchanged.

## Meaning and boundaries

Evaluate explicit policy on one supplied virtual-terminal (VTY) line: allowed
incoming protocols, installed lab host-identity metadata, SSH version policy and
an optional nonempty standard IPv4 source list. Reuse the existing ACL classifier
for first-match/implicit-deny behavior; do not emulate a VTY access list by
attaching a transit-interface ACL. The activity must explicitly supply each line
to inspect. There is no guessed IOS line-allocation or capacity algorithm.

Report each applicable policy condition separately. Passing means only **allowed
to attempt authentication**. It is not an authenticated session, TCP delivery,
encryption or command authorization. A missing lab identity stops SSH admission;
it does not stop Telnet independently allowed on that line. Selecting SSH on a
line does not itself restrict SSH to version2. The compatibility policy represents
permitting either SSH version; it does not implement either wire protocol.

Host identity is a supplied name, never an actual key, credential, file or host
lookup. No key generation, encryption, real SSH/Telnet, socket, subprocess or user
authentication engine is present. Future assignments need a tested source-held
network/transport adapter and appropriate native editor/trial workspace before
they can count as implemented. TCP arrival at a host must not be relabeled as
management-to-router delivery without resolving that endpoint boundary.

Unspecified/multicast/broadcast source addresses, unknown lines, extended source
lists, empty lists, unsupported schema/fields and resource excess fail explicitly.
An error is never converted into a policy denial that can satisfy a negative
grading case. Multiple failed conditions are observations, not a claim about the
order or wording of a device's diagnostics. Configuration replacement is atomic.

Bounds:16explicit supplied lines (not a universal device maximum), line numbers
0–98,128total source-list rules with the existing64-per-list limit,64ASCII bytes
per lab identity,256KiB raw JSON before deserialization. Validate typed callers
before cloning. Serialized valid typed policies must fit the raw decode bound.

## Primary references inspected2026-11-09

- [Catalyst9300 IOS XE Dublin17.12 SSHv2 chapter, updated2023-07-28](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/secure_shell_version_2_support.html): explicit version policy and supplied server identity are distinct prerequisites. Its labelled-key configuration does not universally require hostname/domain settings.
- [Same platform, general SSH chapter](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/configuring_secure_shell__ssh_.html): deleting the server key disables SSH. Older v1-era/key-size examples are not current security recommendations.
- [Cisco IOS terminal-services reference, updated2016-07-29](https://www.cisco.com/c/en/us/td/docs/ios/termserv/command/reference/tsv_book/tsv_s1.html): incoming protocol selection is per line and distinct from source access lists. Its default/tty text is not used to infer Catalyst VTY defaults.

The same pinned [IPv4 ACL chapter](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/configuring_ipv4_acls.html)
was subsequently inspected: its terminal-line section uses numbered ACLs and
requires consistent restrictions across supplied VTY lines. Its line-range text
and the same book's privilege chapter differ; this model's bounded supplied line
labels are not a claimed device maximum. A future CLI adapter must reject named
VTY access classes for this pin. The classifier's internal derived list name is
not a learner-configurable IOS name. No line allocation or VRF behavior is inferred.

No vendor implementation copied and no dependency added. This original metadata
model is not IOS/crypto conformance. Local AAA, credentials, privileges, outbound
access classes, VRFs, session scheduling/timeouts, SSH algorithms and key rotation
remain outside this first foundation. Only the documented incoming standard source-list slice is selected; outbound
classes, extended rules and VRFs require further source review. The LAT command
entry is not the IPv4 VTY source.

## Source-held IP path composition

The next adapter binds one management interface and1–6supplied clients to the
existing IPv4 topology. Its unchanged bidirectional endpoint verifier can target
an actual forwarding device's local address. Do not weaken the existing transit-
only ACL/transport endpoint guards or disguise that target as a host. The source
topology, endpoint identities and addresses remain outside learner policy edits.

Verify a usable unique bound source, the intended management endpoint and its
reply path before reporting policy conditions. A down interface or failed route
is **path unavailable**, not a successful policy rejection. Unknown/duplicate
bindings and alternate source-interface selection are explicit errors. No TCP
handshake, interface ACL, SSH exchange or authenticated session is inferred from
an IPv4 path. A future assignment must keep those limitations visible and grade
its declared policy behavior rather than claim an actual secure connection.

Decode at most1.5MiB before Serde; preserve the existing12device/eight-interface/
32route bounds and add six unique client bindings,32ASCII-character identifiers
and schema version1. Reports are transient observations, never progress authority.

Primitive source gates: R01library55.76s/R02app25.22s/R03eleven focused tests pass;
R04–06staged/all-target Clippy pass (53.44salltarget); R07all961Python pass12.861s.
No new lab, normal UI path, progress schema or objective credit has been added.

## Configuration-document adapter contract

Next: a complete-document parser, not an IOS session. Preserve the supplied host
identity, topology and exact policy-line roster. Require an explicit SSH version
policy and one explicit incoming-protocol setting per supplied line. Support
`ip ssh version 2`, `no ip ssh version`, `line vty FIRST [LAST]`, `transport input`
with `ssh`, `telnet`, either ordering of both, or `none`; optional numbered
`access-class NUMBER in`; and numbered `ip access-list standard NUMBER` sections.
Reuse the existing native ACL document parser for standard rule syntax, ordering,
wildcards and line-numbered errors. Numeric standard ranges are1–99/1300–1999.
`exit` closes a section; full-line `!` comments do not close sections.

Reject named VTY lists, `all` (other protocols are excluded), version1-only,
overlapping/repeated line sections, repeated settings, unknown source lines,
missing list references, outbound/VRF rules, session commands and other syntax
honestly. Do not approximate an unsupported command or infer unmodeled device
defaults. Removing a setting from a complete document is not an IOS `no` command.
Preserve malformed drafts, but apply only a fully validated replacement. A future
lesson must teach this document boundary and the supplied identity/authentication
context. No credentials or generated keys are accepted as infrastructure setup.
Source-list names used inside the shared parser are numeric here, not named VTY
attachments. Bound raw input64KiB,1024lines,512bytes/line and16tokens/line.

## Source-held grading contract

Before authoring, require3–8distinct behavioral cases, at least two approved
clients and at least one prohibited behavior. Every supplied client must appear.
Each case checks every supplied line; a document cannot remove a group to avoid
checking it. Distinguish ready-for-authentication, protocol denial, legacy-version
denial and source-list denial. The latter two must preserve the other applicable
conditions, so disabling SSH or removing the identity is not a source-policy fix.
An unavailable path or unsupported query must never satisfy a denial requirement.

Keep initial/reference documents and the endpoint context in the authored source,
outside saved learner drafts. Validate that the reference passes every case and
the starter is unfinished; reject invalid labels, duplicate cases, unknown clients
and unreached reference goals. Grade parsed behavior, not reference text equality.
Reports name the first failing supplied line and show partial matched-line counts.
No new lab or persisted mastery is created by adding this task model.

Session contract: separate authored initial/example tasks, raw drafts and applied
documents. Filling a worked example edits practice only and does not execute or
award assignment credit. Edits invalidate checks and observations immediately;
parse failures retain the previous applied policy while keeping current success
unchecked. Restore preserves malformed drafts, validates the source binding and
recomputes grading from source/current policy rather than trusting saved reports.
Transient observation output is excluded from saved JSON. Bound serialized
sessions to2MiB, each draft/applied document to64KiB, and selectors to supplied
cases/lines. No duplicated source definition, key data or reference answer is
stored as a learner response. Initial applied syntax errors are permitted only
when they are the exact intentional authored starter.

R08library42.66s/R09app21.16s/R10all8path+11policy tests pass. The existing IPv4
reducer and transit-only guards are unchanged. Router-local delivery, actual reply
route, interface failures, duplicate bindings, alternative return route and
unsupported alternate source-interface selection are tested. No new lab credit.


Post-R65 management foundation —2026-11-09: four new source modules implement
explicit per-line metadata policy, actual router-local IPv4 paths, bounded complete
configuration documents, and source-held grading/separate sessions. R25all44
focused checks pass (11policy/8path/10document/8task/7session). R26–28staged/all-target
Clippy pass (57.16salltarget);R29all961Python pass12.855s. R20fixture type-inference
compile diagnostic is retained; R21all37pre-session checks pass after only an
explicit String annotation. No test assertions were weakened. Current-only source
inventory adds exactly four files; historical collectors/seals/evaluator19remain
unchanged. Transient trial output is not imported as actual execution.

The ready354R65candidate is immutable and excludes this new foundation. No new
scenario, unit, normal UI path, Goal/State variant or4.8/5.3coverage credit yet.
Next source/engine/progress adapters with synthetic invalid-source/restore/reset
checks, then suitable native editor/trials and two distinct authored policy tasks.
The latter is a bounded4.8frontier, not full SSH/AAA, crypto or5.3implementation.
Do not rename a host to pretend router-local delivery or weaken transit guards.
No active owned heavy job/freeze or owner blocker. Preserve all354registered labs,
progress, inherited work and ready receipts. Do not rerun frozen MAINreaders after
these post-build notes. MAINrunner/sharedlock/jobs2/threads2/projectdisk/guards.

