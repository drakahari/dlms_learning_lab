# Client name resolution: investigate and repair

One new capstone, `lpic1-client-name-investigation`, follows the existing local-name,
source-order and DNS-search lessons. Library → LPIC-1 → **Client name resolution:
investigate and repair** → **Find why an application gets the wrong address**.
Prerequisites are advisory. This is a distinct combined diagnosis, not another
numeric search exercise or complete LPIC-1 objective109.4 coverage.

## What learners do

Introduction explains application-facing IPv4 lookup, canonical names and aliases,
source actions, DNS search and checking changing inputs. A separate workshop
uses studio/preview/images and different addresses. Guided practice places the
investigation command beside its explanation and the file editor beside repair
guidance. The independent task keeps all requirements visible, intentional stale
aliases and a stopped-fallback policy. It does not insert a worked repair.

`getent ahostsv4 NAME` reads current virtual drafts for the selected sample.
Run command (or Enter) is read-only; Run lookups checks all requirements in both
samples. A successful lookup can still be the wrong address. Feedback compares
expected/actual addresses and sources and explains a stopped local miss. A hardcoded
DNS copy, removed protected row, retained retired alias or wrong source order fails.
Comments, different row order and equivalent supported explicit default actions pass.
Only checked current assignment files earn credit; practice and diagnostic queries do
not. Editing files clears current checks; editing a query or selecting another sample
invalidates its displayed diagnostic without changing configuration requirements.

## Deliberately finite model

The existing `client_names` and `resolver_search` parsers are composed by
`client_diagnostic`. **files sees the original name once; search runs inside dns.**
Running the complete source sequence again for every suffix would produce wrong
answers. Parsed hosts data now retains canonical/alias provenance without changing
old lookups or serialized configuration. DNS snapshots contain one A address per
exact lowercase name, with positive or definite-missing replies. Only a reached
DNS backend expands candidates; an irrelevant overlong search expansion or an
unavailable DNS service cannot prevent an earlier successful local result.
Unavailable/retrying DNS-search sequences refuse rather than approximating retries.

The new `sources_with_search` workspace mode edits the same two virtual files,
`hosts` and the hosts-only `nsswitch.conf` excerpt. Its source-owned resolver excerpt
(max253bytes) and sample hostname stay read-only. Existing Sources/Search modes,
Scenario values and saved bytes remain unchanged. At most four samples, twelve
probes each,64cooperative phases,16KiB per file,64DNS records,128local names,
three suffixes and four DNS candidates. No budget was raised.

The utility subset accepts one ordinary ASCII name (optional final dot) or canonical
four-octet decimal IPv4 literal, within512bytes. A single result produces finite
STREAM/DGRAM/RAW rows, with the canonical name on the first row and exit status0.
A missing key produces empty standard output and status2. Numeric IPv4 literals
need no name-source lookup. Unsupported databases/options, enumeration, multiple
keys, shell syntax, legacy abbreviated/octal/hex numeric forms, IPv6, multiple
answers, CNAME, caches and OS-specific resolver flags do not silently succeed.
Diagnostics are separate from actual standard output; the source/candidate trace
is a simulator observation, not extra real getent output. Output is bounded2KiB.

No OS resolver, NSS module, socket, network, process or host file is accessed.
This is a defined finite glibc-style subset, **not actual glibc conformance**.
No development host getent/DNS reference process was used. Authoritative descriptions
were consulted; implementation and examples are original, with no libc code copied.

## Teaching boundary and sources

The [getent manual](https://man7.org/linux/man-pages/man1/getent.1.html) describes
NSS lookup, the IPv4 ahostsv4 database and exit statuses. Source order and actions
follow the supported portion of [nsswitch.conf](https://man7.org/linux/man-pages/man5/nsswitch.conf.5.html);
canonical/alias rows follow [hosts](https://man7.org/linux/man-pages/man5/hosts.5.html).
The traditional suffix/ndots model is based on
[resolv.conf](https://man7.org/linux/man-pages/man5/resolv.conf.5.html), not every
resolver implementation. Consulted2026-11-13UTC; original objective source hashes
and their older check dates remain unchanged.

Introduction also gives explicit awareness of systemd-resolved: a local resolution
service with an NSS integration and DNS stub/cache. Its behavior differs from the
traditional dns backend, so identifying the active path matters on a real system.
This is supporting explanation only, based on the
[official systemd documentation](https://www.freedesktop.org/software/systemd/man/latest/systemd-resolved.service.html).
No resolve module, daemon, cache, DNSSEC, LLMNR/mDNS, per-link routing or service
configuration is simulated here. LPIC-1 v5.0/102-500 remains the current mapped
[issuer objective](https://www.lpi.org/our-certifications/exam-101-102-objectives/).
Remote-server query tooling (`host`, `dig`) remains open; this capstone cannot stand
in for those utilities or complete real Linux networking experience.

## Persistence and authoring

Progress schema1/evaluator19 is unchanged. New mode/resolver/diagnostic fields are
opt-in; absent fields serialize exactly as before. The two scopes retain separate
configuration drafts and bounded command/sample selection. Derived output is not
saved or accepted as grading authority. Restore validates source identity, mode,
file names, input bounds and sample selection, then regrades current files. Reset
replaces only the selected attempt; prior existing IDs, favorites and work remain.

Authored data is appended to `boot-entry-lessons-v36.json`; all68v35Scenario objects
and all441earlier catalog objects remain exact. The new unit is inserted after
`linux-resolver-search`; all173earlier Unit objects are preserved. Retained441helpers
support direct compatibility tests. Source admission still proves a separate,
reachable worked example and meaningful unfinished starters.

## Verification checkpoint

Focused core6plus previous name/search20pass. Workspace/authored6and current
registry preservation1pass; earlier Sources/Search workspace13and management5pass.
Five new UI/event/capture regressions pass, including five supported scales, two
themes,760/1600logical widths, actual nonwrapping configuration row counts, Enter,
current-state grading, practice separation and context-safe screenshot requests.
These are technical checks, not a novice study or physical-device acceptance.

Development-native R20completed60actions/13PNG, with changing DNS samples, absolute
misses, honest unsupported commands, wrong/correct/broken/repaired grading and
Save/readback. Actual R16Introduction/worked/practice/assignment/refusal screens and
R20full200%Light command input inspected. R12/R16premature enlarged captures are
retained; screenshot requests now wait for smooth scrolling and recheck geometry.
Final practice-checklist/trace sampling and **packaged verification remain pending**.
No claim of narrow native resizing, physical keyboard, screen-reader, independent
novice or Windows/macOS runtime acceptance. Full inclusive Rust runtime suite has
not been repeated for this focused batch. Candidate441R2remains the verified package
until a new frozen build passes its actual gates. No installation replacement,
publication, Git mutation, new dependency or desktop/system change.

Final source gates: inclusive Python1217R28passes, one-job all-targetClippy
warnings-denied R30passes, strict coverage R31and inventory R32pass. Shared legacy
name-source filter11and DNS-search filter9pass; some capture tests overlap the new
five, so these counts must not be added as unique tests. R26/R27live-cardinality/
current-pin failures and R29identical-helper ordering failure are retained; frozen
reference tables, old-prefix fingerprints and grading expectations were not relaxed.
The selected source will be frozen before building442R1. No post-build acceptance
is implied by these pre-build notes. Older executables may reject newly added work;
this preserves forward loading of existing progress, not binary downgrade compatibility.

###442R1packaged failure and R2correction

R1identity/content/private441R2upgrade pass; installed workflow failed at step46
while revealing the200percent capture target, exhausting the unchanged20scroll
attempt bound. NFS was not reached. Final input identity matched and2814selected
files/106206736bytes were retained; freeze ended. This is failed evidence, not a
ready candidate. R2uses the shared15second/1200eligible-poll guard with100mssettling,
focus release and the same20scroll limit. Passive-paint/focus regressions pass.

Source review of actual R1practice/trace/assignment originals additionally caught
a trailing-dot name split by punctuation. Explicit authored newline clauses now
preserve every word and requirement, including the absolute missing name plus
unresolved outcome. A test inspects actual painted labels rather than parser code.
The optional trace also avoids a missing arrow glyph. New8UI/event/capture and
current13backend/authored/retention tests pass; old441lab/173unit/145teaching values
stay exact. No scoring change. Fresh R2source/package gates remain pending.

###442R2pre-build source gates

Corrected8UI/event/capture, current13backend, shared name-source13/search9and
common poll/deadline2pass. Inclusive Python1217R40, one-job all-targetClippy
warnings-denied R39and strict coverage/inventory R41/R42pass. New example
instructions have explicit newline-delimited clauses with identical normalized
words/outcomes; no prose punctuation is parsed as requirement structure. Only
the new example text and three UI/capture files differ from R1selected source.
All model, grade, progress, prior441labs/173units/145teaching data remain exact.
Fresh R2package/runtime and final visual/readback gates are still required.

###442R2actual Fedora verification

The replacement candidate passes selected identity/content, private441R2upgrade,
installed62actions/14PNG, and actual two-process NFS restore/repair (both exits0).
Original Introduction/practice/assignment and complete200%Light field/action PNGs
were individually inspected, alongside restored/repaired NFS originals. Final
identity matches and2814selected files/106218042bytes are retained; freeze ENDED.
The outer orchestration exited143 after NFS helper PASS, cause unknown. Its log,
manifest and interruption readback stay retained; this is not relabeled overall
exit0.100NFSmemory samples: min15.389GiB, fullPSI0, OOMdelta0; markers absent and
shared lock free. See CHECKPOINT.md for exact review launch and acceptance limits.
