# DNS search paths, dotted names and absolute lookups

2026-11-02 source252labs/71units:72CCNA,71LPIC-1,43LPIC-2,66Bash. New unit
`linux-resolver-search` follows `linux-client-name-sources`; the second assignment
also names the first as its prerequisite. This is additional narrow109.4 practice,
not full client DNS, actual glibc conformance or complete exam preparation.

- `lpic1-dns-search-suffixes`: prefer the intended site when two search domains
  answer the same short name, retain second-site fallback and preserve absolute
  peer/missing lookups. Two supplied DNS snapshots change the address without
  changing the required policy.
- `lpic1-dns-absolute-precedence`: keep a short name under its branch suffix while
  protecting a dotted remote name from a suffixed shadow. Compare its final-dot
  spelling and a missing absolute name whose relative spelling can search after
  a definite miss. ndots1 and2 are supported legitimate alternatives; zero and
  four each break a different required lookup.

Introduction teaches each rule and displays a complete, separate workshop
configuration before an annotated table. Guided practice uses the same real
parser/evaluator as assignment work: run the faulty starter, compare candidates,
repair, deliberately remove a fallback suffix or use ndots0, then retry. Worked
insertion never executes or changes the assignment. The independent assignment
keeps its complete task, exact requirements, intentional faults, supplied DNS
facts, one named resolver excerpt, Run/Stop and actual candidate feedback. These
queries already reached DNS; local hosts entries and nameserver configuration
are outside this activity. No pointless file-selection control is shown.

The implementation adds a Sources(default)/Search mode to the existing bounded
configuration workspace. Default mode and absent hostnames serialize without new
fields; exact original local-name goals and initial states are checked. Source
key sets are enforced before indexing. Search only admits its one resolv.conf
excerpt and positive/definite-missing replies. A shared source marker does not
award credit: current outcomes are derived again. Editing invalidates results;
practice, assignment, selected view and drafts remain separate and resumable.

## Evidence so far

- Nine pure planner/parser groups pass, plus retained11local-name groups; initial
  scoped Clippy and126Python pass before full lesson integration.
- Six mode/scope/format guards and retained checks pass (37combined groups):
  actual candidate plans, alternatives, invalid/foreign file sets, cancellation,
  resume, absolute-query admission and exact old goal/initial serialization.
  Scoped workspace Clippy passes. A missed explicit default in one synthetic
  fixture was caught at compile and fixed without removing assertions.
- All five authored source/persistence groups pass, including exact250previous
  labs/70units, references, alternatives, common wrong settings, partial results,
  the exact guided experiment/repair, safe save/export/restore and selected reset.
- Eight new UI/controller/capture groups pass, plus one actual painted candidate-
  trace regression. They cover all three views/no unsolicited answers, inert
  insertion, real editing, stale/retry/save-read, Run/Stop/Escape, readable scaling,
 760/1600logical widths and Context-safe capture. Nine retained local-name UI
  groups also pass after the opt-in branch was added.
- Actual development native, individual content/render review, final Python/
  Clippy and separate packaged checks are pending and must be recorded as run.
  The latest VERIFIED candidate is still250/T225201Z; source252 is not that binary.

## Boundaries

See [the exact subset and reference limitations](RESOLVER_SEARCH_SUBSET.md).
These are finite supplied DNS A/missing-name observations, not an integrated OS
NSS/resolver or packet exchange. Other resolver options, network/retry sequencing,
cache, CNAME/IPv6/multiple answers, DNSSEC, HOSTALIASES/environment, systemd-resolved
and full109.4 remain Open. No host hostname, file, socket, service or subprocess
is used. No dependencies, old shell semantics or original profiles changed.
No new installation/upgrade/cold reopening, full-suite, other-platform,
screen-reader, physical-device or independent novice-comprehension claim.

## Development native and final source review

All78native steps pass,28PNG run-antv9syz. All six normal teaching screens,
actual failed-candidate traces and light200 controls inspected. Leftover generic
hosts-workspace captions caught in that render were corrected only for Search: the
editor explicitly names /etc/resolv.conf, the sample names DNS and the worked
caption explains candidate generation. Targeted actual painted-label assertions
pass afterward. Default Sources captions stay unchanged; nine retained UI groups
pass. Inventory252=247updated+5alreadymeets after individual content/render review;
47-row supplement still limits these to finite109.4practice. Final package/native
render and Python/Clippy gates remain separate; no installed/profile replacement.

## Final scoped Fedora package

Resolver-search milestone2026-11-02: separate252/T233226Z validates252labs/71units:
72CCNA,71LPIC-1,43LPIC-2,66Bash. Both search-suffix and ndots/absolute-name
lessons pass all78packaged native steps,28PNG run-uavcaoz2. All six normal
teaching screens, wrong-result traces and narrow light200 controls inspected;
corrected resolv.conf/DNS-only captions are present.473source pins and eight
explicit offline guides matched MAIN BEFORE further source work; immutable
resolver-search-candidate-build-identity/verified receipts retained. BinarySHA256
b5b9823a69856ca8152b7c26c33919aa19781f96e7f5b3556d961514b0a071b7;
archiveSHA256 7d25ac04151ce445ac9cc479c60cc42f05214808b52865b8ff5d6e220a7c6147.
Focused parser/model/session/content checks, exact250lab/70unit preservation,
9new and9retained UI groups,130Python and scoped Clippy pass. Inventory247updated
plus5alreadymeets;47-row supplement remains narrow109.4practice. Build364samples,
min16.663GiB,PSIavg10max0,OOMdelta0,finalswapfree2.762GiB; common lock/Cargo2/
runtime2/disk, not a memory guarantee. No real resolver or full objective claim;
no new lifecycle/full-exit cold reopening/full-suite, other-platform, physical,
screen-reader or independent novice acceptance. Installed/original profiles
unchanged. See RESOLVER_SEARCH_LESSONS.md. Continue existing roadmap.

- [Introduction](../.build-artifacts/native-runs/run-uavcaoz2/screenshots/workspaces/teaching-lpic1-dns-search-suffixes-introduction-dark-100.png)
- [Guided practice](../.build-artifacts/native-runs/run-uavcaoz2/screenshots/workspaces/teaching-lpic1-dns-absolute-precedence-example-dark-100.png)
- [Assignment](../.build-artifacts/native-runs/run-uavcaoz2/screenshots/workspaces/teaching-lpic1-dns-search-suffixes-assignment-dark-100.png)
