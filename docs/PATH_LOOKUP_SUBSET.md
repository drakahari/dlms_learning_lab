# Readonly inherited-PATH lookup: capability boundary

2026-11-05. Pure core version1 and its opt-in Environment adapter are tested.
One authored source lesson brings the catalog to286/89; native/package gates
are recorded separately in PATH_LOOKUP_LESSON.md and CHECKPOINT.md. The latest
verified review package remains285 until those gates pass. Evaluator19, old
lessons and progress schema are unchanged.

## What the core actually does

`filesystem::path_lookup::resolve` examines a supplied ordinary-user Tree in
memory. For each fixed external basename it tries PATH directories in order,
checks searchable ancestors and an ordinary file's selected execute bit, and
returns typed candidates/selection. Nothing is loaded or run. Empty artifact
contents are metadata placeholders, not valid executable-format claims.

Lookup and policy are separate: finding a candidate does not prove the assignment's
complete PATH uses only declared trusted directories. Policy inspects every
component, including unused suffixes. Declared trusted directories must be
readonly for the modeled caller under owner-first/group/other permission selection.
This does not prove global trust against other users, writable ancestors, ACLs,
mounts, root/capabilities or future changes. The source inventory itself is readonly.

File execute permission and directory search permission are different checks;
neither requires file or directory read permission for this lookup. The ordinary
Tree access API adds only `ExecuteFile`, a permission query, with old serialized
operation names retained exactly. It adds no terminal execution command.

## Limits and honest exclusions

- Supplied Tree: at most64nodes and empty artifact values; no symlink or actual
  filesystem traversal. Canonical source identity includes all metadata/users.
- One to four distinct source-fixed ASCII external basenames, at most48bytes,
  beginning with a letter. A small activity-reserved query-name list avoids
  ambiguity with VM interfaces; cat/grep are real external tools, not classified
  as Bash builtins by this restriction.
- Defined PATH: at most16nonempty canonical absolute directory components,
  at most1024bytes each and16KiB total. Duplicate components remain observable;
  Unicode directory data is supported. Unset/empty/current-directory/relative/
  noncanonical forms report Unsupported rather than inventing default searches.
  Real shells support forms outside this initial activity subset.
- At most16candidate rows per query and4160bounded path-entry checks. These are
  work units, not measured time or real system calls. No generic shell126/127
  status is fabricated from typed NotFound/Denied decisions.
- No alias/function/hash lookup, command-not-found hook, ELF/shebang loading,
  subprocess, host PATH/environment/files, network activity or utility output.
  Existing Bash simulator dispatch remains separate.

New explicit Profile::decode/Observation::decode entry points apply raw-byte
caps (128/256KiB), duplicate/depth/work guards, bounded typed fields and semantic
validation; observation decode recomputes its result. Serde may allocate bounded
scratch for escapes. Generic nested Tree serde retains its existing reader:
callers still need their own bounded outer input gate and validation. A valid
alternative source tree is not the authored profile. Observation.matches proves
only consistency with its declared PATH/profile; a future adapter must also bind
that PATH to the worker's actual inherited snapshot. No stored pass flag grants
execution or progress authority.

## Opt-in Environment integration

Task.path_lookup and CaseRun.path_lookup default to absent and stay omitted for
old tasks. The setup produces real parent/export attributes; the typed worker
lookup uses its ACTUAL inherited CHILD PATH, never the canonical expected value.
A parent shell can use its own private PATH; this lesson does not teach that
export is necessary for the parent's own lookup. Resolved utilities never run.
The supplied child_source VM runs separately as an environment-report script,
not as the artifact found by PATH.

For new tasks only, valid PATH strings are graded by source-bound provider and
whole-path policy outcomes. The export attribute, every other environment key,
fixed report/status and protected file remain exact. Each expected case name
binds to one actual input; allowed paths must be canonical direct source files,
searchable/executable and inside a listed readonly directory. Actual observation
must equal a recomputation of the stored inherited snapshot. Saved runtime proof
is not serialized, and source erasure/mutation fails catalog-aware save/load.

At most8cases,4queries,4approved provider choices per query and the core bounds
apply. A bounded serialized Task is checked before Job clones it; lookup reserves
and charges its conservative budget before the child stage. Stop clears pending
state and cannot publish an incomplete record. Native opt-in rows distinguish
actual selection/policy from expected practice providers and fixed child output.
Independent assignment does not expose a solved PATH. The source lesson, focused
cases and native/package acceptance are documented separately, not inferred from
this capability definition. All old None activity behavior stays exact.

## Evidence and primary references

R1sixteen pure-core groups pass0.01s after actual compilation. R2retains all12
filesystem,7access and12FHS groups. Tests cover ordered/alternative/duplicate
lookup, permission precedence, denied skips, Unicode, unsafe unused suffixes,
source forgery, malformed/bounded decode and recomputed observations. No real
host lookup or utility execution was used. Adapter12/exact-phaseStop1, old Environment/Startup and authored7 groups also
pass. The old verified candidate stays unchanged pending the new package; this
is neither full-shell conformance nor novice/platform acceptance.

[Linux path_resolution(7)](https://man7.org/linux/man-pages/man7/path_resolution.7.html),
Linux man-pages6.19, dated2026-02-08, read2026-11-05, informs search permissions and
ordinary permission classes. [GNU Bash command search](https://www.gnu.org/software/bash/manual/html_node/Command-Search-and-Execution.html)
was read from the installed original Bash Reference Manual Edition5.3
(18May2025), checked2026-11-05; online fetch timed out. Installed manual identity
is retained privately. OpenGroup2024 pages returned403 and are not claimed read.
Original teaching/code paraphrases facts; no manual is bundled or copied as code.
