# File-size searches and evidence collection

This opt-in extension reuses the bounded virtual Tree, traversal diagnostics and
native terminal. Only source-held **Collect** activities enable size queries.
Existing Tree terminals and Stage/Retire lessons retain their original behavior.
No new shell evaluator, subprocess, host file or network execution route exists.

## Meaning and supported syntax

`find START [-maxdepth N] -type f [-name QUOTED_PATTERN] -size [+|-]N[UNIT] [-print]`

Predicates combine with implicit AND. Existing name/type/depth grammar and bounds
remain; multiple size predicates are allowed within the16-predicate cap. Size
queries require an explicit `-type f`: directory metadata sizes are not modeled.

| Unit | Bytes per unit |
| --- | ---: |
| no suffix, or `b` | 512 |
| `c` | 1 |
| `w` | 2 |
| `k` | 1024 |
| `M` | 1048576 |
| `G` | 1073741824 |

Measure the virtual file’s UTF-8 **byte length**, round up to whole units, then
compare. No sign means equal; `+N` means strictly greater, `-N` strictly less.
Zero bytes rounds to zero units. Thus `-1k` matches only empty regular files,
whereas `-1024c` also includes nonempty files through1023bytes. This measures
content length, not allocated disk space, sparse-file blocks or character count.
Lowercase `m`, uppercase `K`, fractional quantities and extra signs fail explicitly.
Quantities use bounded unsigned64-bit parsing; no multiplication overflow is needed.

The virtual Tree retains256entries/256KiB content. Very large unit arithmetic has
boundary/reference tests, not large-file practical coverage beyond that data cap.
No timestamps/time predicates, mutation by find, `-exec`, general Boolean syntax,
links/devices or complete GNU option surface. Read/search permission, incomplete
search,32KiB output, effort, depth and pending-label limits retain the existing
behavior. A permission failure cannot become a silently complete empty result.

## State-based collection grading

The catalog holds initial source content and the typed criterion. Expected members
are derived mathematically from that immutable source, with only their required
parent directories. Grading verifies exact names/types/content, filtered source
owner permission bits, caller ownership/group, private defaults and full source/
unrelated preservation. It accepts equivalent supported operations rather than an
exact query/copy recipe. In particular an equivalent empty regular file can be
created with touch plus correct permissions; no action log is misrepresented as
proof of a particular method. Editing real state clears stale checked results;
every grading call still recomputes outcomes, including after imported tampering.

Assignment and practice remain separate source-bound roles. Both use the same task
family with distinct files/paths. Drafts, transcript bounds, safe save/read and
reset retain the existing recursive-workspace schema. No new completion is awarded
to old attempts and no existing lab ID or objective is changed.

## Primary references and actual verification

[LPI objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
were checked2026-11-06: LPIC-1v5.0,101-500/102-500, narrow103.3size-search practice.
The installed GNU Findutils4.10.0 Info Size node was read alongside the
[GNU size documentation](https://www.gnu.org/software/findutils/manual/html_node/find_html/Size.html);
live web retrieval failed, so the installed primary source was used. Its compressed
SHA256 is `2f5171b24949308d53144d6e9a96d19ce41b3c2513227926c73f94eaec3b1b02`.
The installed prose’s ambiguous plus-sign wording was resolved by actual boundary
comparisons rather than copied into teaching as an inclusive comparison.

R08observed31fixed queries against original GNU find4.10.0 and the actual in-process
probe. All31path memberships, status and empty stderr match, including equality,
strict signs, units, empty/UTF-8 files, compound predicates and depth. Raw output
ordering is retained; no readdir-order equivalence is claimed. Two model-only
refusals (directory-size query and find deletion) are exclusions, not host matches.
The control pins695compiled inputs, tool/fixture sources and held executable hashes.
Its SHA256 is`82372a7483cc557f921c93a9c14451f24c4e396b912709a101dd2caacd2c66da`.
Original find ELF SHA256`2d805195758c1352406b3b5e5a7b3df82aee496c808667f44824261ffaf3ff09`.

Disposable owned project-disk files, exact closed query grammar, original held
ELFs, stripped environment, empty stdin, child-only no-new-privileges/limits/umask,
bounded streams/time and MAIN’s shared lock guard this development comparison.
No learner content or arbitrary query/root/executable selector is accepted. ACL
inheritance would refuse rather than alter host settings. Source/ELF/raw fixtures
remain ignored under `.build-artifacts/reference-runs/file-size-r1/compared`.
This is root source-reviewed technical evidence, not an independent historical seal,
full Findutils conformance or new novice/platform acceptance.
