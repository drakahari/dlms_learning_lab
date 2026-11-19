# Repository configuration, caches and reliable evidence

New LPIC-1 unit `linux-repository-evidence`, after `rpm-package-investigation`.
Source225/60:69CCNA,52LPIC-1,40LPIC-2,64Bash. Existing221 lab objects and59
unit objects remain exact; current-snapshot regression compares every object.
Native package acceptance is still pending; do not infer it from source tests.

| Stable ID | Practical work | What makes it distinct |
| --- | --- | --- |
| `lpic1-repo-visibility` | Enable a source, populate its index and inspect two package candidates | Disabled versus missing cached metadata, without installing software |
| `lpic1-repo-provenance` | Repair a configured source while inspecting cache provenance | Reuse matching current metadata or explicitly refresh; do not blindly refresh a mistaken address |
| `lpic1-repo-offline-evidence` | Inspect and retain useful older metadata while delivery is unavailable | A truthful older-cache report, not invented freshness or a forced unnecessary repair |
| `lpic1-repo-handover` | Repair live while preserving held, then inspect both packages | Positive current evidence plus exclusion of held candidates and protected historical state |

## Teaching and workspace

Each Introduction explains concepts before demonstrating different sample sources,
package names and versions. Try example uses separate state, editable native
settings and actual queries. Prepare worked action only fills a draft/name or
selects an alias: it does not apply settings, refresh, query or grade. Worked
examples include expected refusals and deliberate wrong-source/disabled cases.
The Assignment keeps all requirements visible without exposing reference actions.
Challenge presents the same independent task and optional requested help.

The native editor separates unapplied settings from applied configuration and
cached provenance. Alias editing controls remain outside the horizontally
scrollable read-only table. Form rows use shared font/padding metrics; large text
stacks the workspace. Actual query rows include alias, source, snapshot, package,
version/release and architecture. Source omissions explain why rows are absent.
Old query results retain the state that was actually inspected and are labeled
Previous configuration/cache state after an applied change. Expected requirements
are never rendered as though they were actual query results.

A supplied address is an offline label, even if it begins file:/// or https://.
Use address in draft fills the field; Apply settings makes a configuration change.
Refresh metadata uses supplied delivery facts and cached snapshots, not a real
path/URL. Query exact package reads the cache, without auto-refresh or installation.

## Behavior, isolation and progress

`src/rpm_repositories.rs` remains the reviewed pure bounded foundation. New
`repository_workspace` composes typed Settings/Refresh/Query events and actual
engine probes. It replays at most32 successful operations from immutable supplied
inputs, requiring the whole saved world to match. An unavailable refresh cannot
create new metadata by restoring a fabricated cache. Source, alias roster and
historical cache provenance remain bound to the authored task. No host I/O,
subprocess, real package operation or shell evaluation is introduced.

Grades probe two package names, desired configuration/cache policy and protected
configuration/cache records. Query order is independent; the provenance lesson
accepts both matching-current-cache reuse and explicit refresh. Positive live
rows do not excuse extra held rows. Earlier query evidence becomes stale after
applied edits/refreshes. Unapplied drafts invalidate current success; imports with
false draft markers and mismatching editor fields are rejected. Serialized
reference events are developer pack reachability data, never taught as commands.

Sessions retain both worlds, drafts, current alias, history, actual earlier query
views and learning screen. Reset affects the selected attempt only. Progress
save/decode use the authored catalog and retain prior bytes on failure. No old
IDs, versions, objectives, shell evaluator, saved answers or completion credit are
changed. All28 themes remain. Public/source snapshots do not include private
profiles or screenshot recordings.

## Explicit limitations and objective attribution

This is native metadata/configuration/evidence practice, **not a Zypper/Yum CLI**,
libzypp conformance, package installation, dependency solving, signature/trust
verification, transport, downloads, automatic refresh or actual source discovery.
Wrong-source candidate withholding is an explicit model policy. The core supports
one or two supplied repositories, up to12 RPM archives and two metadata snapshots,
with fixed delivery facts. History/state/output remain bounded. Failed operations
leave the world and successful-event history unchanged.

Official sources checked2026-11-02:

- [LPI LPIC-1 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/):
  current5.0, exams101-500/102-500, relevant102.5 package/repository concepts.
- [Official openSUSE Leap16 Zypper manual](https://manpages.opensuse.org/Leap-16.0/zypper/zypper.8.en.html):
  distinguishes configuration, repository metadata and refresh/query operations.
  The subset document retains the original upstream source review and its dates;
  no upstream implementation code or manual prose is copied.

This strengthens supporting concepts within102.5; named Zypper/Yum command/tool
rows remain Open where unsupported. No parent objective or exam preparation is
made complete. See CONTINUATION_COVERAGE.csv; historical issuer CSV fingerprints
and counts remain unchanged. Broader package administration is still roadmap work.

## Actual focused evidence

Private `continuation/resume-20261102/repository-model-final.log`:12 passing
model/content/persistence groups, including four real solution paths and separate
examples, current-cache alternatives, disabled/missing/wrong-source failures,
offline retention, stale evidence, extra held candidates, partial queries, forged
cache/history refusal, false editor markers, invalid packs, path labels that do
not access files, unsupported syntax, operation limits and failed-save retention.
The earlier regression also preserves every221 lab and59 unit object.

`repository-ui-r1.log`:four actual widget/paint groups pass across all four lessons;
`repository-columns.log`:four aligned columns/two rows and fields at100/115/125/
150/200%,760/1600 widths, light/dark. `repository-capture.log`:actual capture-phase
Context safety passes. Production Clippy passes. New final feedback gives actual
enabled/source/cache states and unexpected candidate versions without demanding
an unnecessary refresh. Final Python/candidate/native gates are pending. These
checks are not a beginner study or physical/screen-reader/cross-platform acceptance.

Final pre-package gates: repository-model-final2.log passes13 groups, including
reverse query order and specific wrong-version feedback. repository-ui-final.log
passes five real widget/paint/layout/capture groups. repository-python.log passes
77 focused packaging/inventory/frozen-reference guard checks; explicit CURRENT
layer adds six source inputs and proves each frozen-prepare refusal before child
launch. Frozen collectors, compiled input tables and historical observations are
unchanged. repository-clippy-final.log production Clippy passes. Candidate/native
checks remain separate and pending; no whole suite was repeated for this batch.

## Packaged Fedora technical verification — 2026-11-02

Separate candidate `dist/teaching-review-20261102T143143Z/`; binarySHA256
`725ade83d31509653cc1c0069ea24118b10fa80d2ed7763913478aecbeeb356e`.
405 recorded source inputs. `packaged225-content.log` validates225labs/60units.
`packaged225-native.log` passes all4 separate examples and all4 partial, solved,
draft-damage, applied-damage, retry, actual save/file-load and reset flows.
40 actual captures in `native-runs/run-31y_vzn7/screenshots/workspaces/`.
Inspected three Introductions, guided practice, independent handover, failed
handover feedback and narrow light200% Check controls. Read-only metadata tables
may scroll horizontally; editing controls remain outside their scroll regions.
A long worked reference value also uses the existing horizontal reference scroll;
it is not vertically clipped or rendered as an executed result.

- [Introduction](../.build-artifacts/native-runs/run-31y_vzn7/screenshots/workspaces/teaching-lpic1-repo-visibility-introduction-dark-100.png)
- [Guided practice](../.build-artifacts/native-runs/run-31y_vzn7/screenshots/workspaces/teaching-lpic1-repo-provenance-example-dark-100.png)
- [Assignment](../.build-artifacts/native-runs/run-31y_vzn7/screenshots/workspaces/teaching-lpic1-repo-handover-assignment-dark-100.png)
- [Failure feedback](../.build-artifacts/native-runs/run-31y_vzn7/screenshots/workspaces/repository-lpic1-repo-handover-damage-feedback.png)
- [Light200% controls](../.build-artifacts/native-runs/run-31y_vzn7/screenshots/workspaces/teaching-lpic1-repo-handover-assignment-light-200-controls.png)

Original installed app/progress and all prior work remain preserved. No commit,
push, merge, dependency/system change or release. This is not cold-process
reopening, installation/upgrade, Windows/macOS runtime, screen-reader/physical
accessibility or independent novice acceptance; no full-inclusive suite rerun
is claimed for this focused batch.
