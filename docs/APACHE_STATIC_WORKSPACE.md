# Static Apache app foundation — source-only handoff

2026-10-25. This integrates the accepted pure static-serving model with typed
app context, bounded native-editor data and separate optional teaching. Its
20 prospective integration groups are **UNRUN**. No compiler, VM, HTTP reference,
real server, native renderer, authored lesson, registry or coverage gate was run
for this patch. Existing125 scenarios/31 units and their identities/versions,
initial states, Goals, solutions, raw packs, historical records and ordering stay
unchanged. The technical fixture is unbundled; source capability is not new credit.

## Current state and truthful goals

`State::ApacheStatic` holds the pure simulation. `Goal::ApacheStatic` owns a boxed
`Definition` with exact authored initial state and1–32 distinct typed requirements.
Initial state is not duplicated into every mutable engine State. Inventory pins
fixture/server/config slot and supplied directory/file identities/kinds. Saved
config, optional active source and live contents remain mutable. No response or
history record is accepted as Goal authority.

`CurrentHttp` evaluates a typed request against current active config and current
files. `NextStartHttp` evaluates the same kinds of predicates on one bounded copy:
Stop it, then Start from current saved text. The copy is shared across the batch;
failed Start fails the probes, never becomes a successful denial. The learner's
active source, files and clock are untouched. Each Definition needs positive200
current and next-start serving. Optional400/403/404/301 checks cannot be satisfied
by a stopped server, NoListener or Unsupported. The task still must explicitly
include every site/path promise; one positive is not universal host coverage.

Expected typed responses specify status, logical selected site, mapped resource,
body, representation byte count, redirect target and optional cause. Site identity
comparison ignores ASCII case as the core does; resource paths/text remain exact.
GET200 needs an explicit body and its independent UTF8 byte length. HEAD200 needs
empty body and a declared byte length. Redirect301 needs an explicit target path.
Error responses have this model's empty body; no real Apache error HTML/headers
are invented. Causes can distinguish authorization denial from no-index403.
Unknown/excluded Host/path forms stay core diagnostics rather than canned400.

`FilePreserved` compares only a named supplied file with authored initial text;
`FileMatches` compares explicitly requested contents. `ActiveMatchesSaved` is
optional exact coherence, not a Reload history badge. `SavedTextPreserved` is only
for an explicitly stated exact-text task. Ordinary serving tasks allow alternate
supported directive case, comments, layout and policies meeting every outcome.
Feedback names actual status/site/resource or a bounded diagnostic, never dumps
golden file/config contents. All retained app notices/check messages use fresh
<=512UTF8-byte strings.

## Draft, queries and explicit mutation

Draft contains immutable Inventory context, raw config editor text, one selected
supplied file+raw text and virtual request fields. It stores no Outcome. Request
port/Host/path/method edits and clean selection are read-only and do **not**
invalidate applied credit; unsupported query text can remain saved for correction.
Only pending config/file text prevents completion. Query parses decimal port
before calling the pure bounded request API and cannot change serving/credit.

Config/file Save actions are atomic and explicit. Invalid syntax can be saved
within the pure config bounds without erasing a healthy active server. An overlong
line may remain in the bounded Draft while Save refuses the pure line limit.
Start while active is unchanged; typed Reload is running-only and parses first.
Static-file Save affects later responses immediately without Reload. Check only
computes current, next-start, explicit preservation and pending outcomes.

Selecting a different file refuses to discard unsaved file text. Selecting the
same file retains its pending text. Current configuration/file editors are kept
separate from practice editors. Failed action/context/admission preserves current
simulation; no silent truncation, automatic repair, activation or file creation.

The engine's `apache-static ` JSON prefix is a pack-authoring bridge for typed
SaveConfig/WriteFile/Start/Stop/Reload only. Mutation requires immutable Goal
context. Raw engine execution offers honest help/show and refuses mutations.
No apachectl/systemctl/curl, shell, service alias, host path or network bridge is
added. Native UI is a later stage using explicit existing typed controls, not an
invented Linux command terminal. Current125 command limits are unchanged.

## Independent teaching and progress

Optional Lesson has an admitted Definition, bounded prerequisites/concepts,
individually annotated typed steps, a concrete experiment's **objective rows**,
and repair. Pack validation proves reference/example/experiment/repair reachability.
Its separate fixture server/config slot/file identities differ from assignment;
common structural parents such as `/` are allowed. Evolving config text is not
required to have disjoint namespaces, so legitimate wrong edits stay saveable.

Learning keeps one immutable initial fixture copy plus independent simulation and
Draft. Every operation/import compares that copy with authored Lesson initial;
mutable simulation uses only its own fixed Inventory. The selected SaveConfig or
WriteFile step can be inserted into its practice Draft without execution. Other
steps say to use their virtual control, not pretend they insert runnable text.
`perform` applies typed actions after all bounds/context checks and aggregate
projection, preserving unrelated pending editor text. Practice checks prepend one
pending-editor row; authored experiment vectors compare objective rows after it.
No practice completion grants assignment credit. New Learn starts Introduction;
metadata is ready for future focused Intro/example/assignment native views.

Workspace metadata is optional/omitted when None. Wrong-kind/foreign metadata
rejects before publication; existing125 imports do not acquire these constraints
or new fields. Saving/decode validates context and pending drafts independently
of checked/completion flags. No old6/7/8 record is relabeled or silently rerun by
this adapter. Prospective preservation tests import the actual frozen writer8
fixture, retain full6/7/8 history equality and generate *current* records only by
actual in-process VM calls when those tests are eventually granted execution.

## Explicit bounds and decoder limits

- Requirements32 distinct; Goal compact768KiB; Lesson compact768KiB.
- App State compact512KiB; pure State/raw limit2MiB stays unchanged.
- Draft compact192KiB; raw config16KiB, selected file text4KiB, port8bytes,
  Host2×260bytes, path256bytes; notice512bytes.
- Learning compact1.5MiB, with full fixture copy, independent simulation and Draft.
- Typed action raw prefix+128KiB, checked **before** trim/parse; other families retain
  their own limits. Core request raw8KiB and scalar limits are unchanged.
- Pack2,000,000/profile16,000,000 ceilings remain unchanged. Not every combination
  of independent pure maxima fits an app document; app admission rejects it.

Borrowed scalar/count checks precede duplicate comparisons, serialization counting
and bounded clones. Collections use bounded visitors/seeds and no untrusted
size_hint reservation. Existing internally tagged engine/requirement readers can
buffer outer Content under the existing whole pack/profile limits; this is not a
new raw-streaming or pre-JSON-allocation claim. Bounded app wrappers replace serde
errors with fixed diagnostics, including unknown large field names. The sole pure
source change is `Request::shape` private→pub(crate), **same body**; no grammar,
request ordering or HTTP algorithm changes.

## Prospective verification and remaining gates

20 groups cover reference/semantic alternate case+format; exact current-vs-fresh
shortcuts; stopped/invalid saved failures; live-file/current-preservation effects;
atomic bad reload/unknown file/raw mutation bounds; query-only credit; malformed
pending text/file selection; direct scalar/collection/context caps; separate
insert-only/experiment/repair; strict context/wrong-kind profiles; pure-legal data
over app admission; HEAD bytes/400-vs-Unsupported; exact301 targets; near-max escaped
world/Learning and32-probe timing; original125 typed metadata+units/None wire; actual
current Script records plus genuine6/7/8 history with both editor drafts/favorites,
exact before9/before8 bytes, export/inspect/confirmed restore and selected reset.
They are source proposals, not pass counts. Compiler/type/lint failures may remain.

Next: root+peer source review, sole guarded first compilation/tests, measured max
clone/check cost, affected regression/Clippy/format, then separately reviewed native
editor and event-only controls/render/cold checks. No native data publication,
novice, Windows/macOS, full Apache208.1, HTTPS208.2, Squid208.3 or Nginx208.4 claim.
Existing primary Apache2.4/RFC9110/9112/LPI4.5 references and exact exclusions are in
[APACHE_STATIC_SUBSET.md](APACHE_STATIC_SUBSET.md); this source patch makes no fresh
external source/version observation.

## Source review R2 — still unrun

The two fieldless Requirement variants now use empty struct syntax internally,
keeping the same `{"kind":"ActiveMatchesSaved"}`/SavedTextPreserved wire. This
closes serde internally tagged unit-variant unknown-field admission; both forms
have explicit positive/unknown-key/duplicate-kind tests, including whole Definition.
Teaching prose checks byte length before trim, with identical accepted empty/NUL
behavior and direct oversized-whitespace negatives. Original R1 snapshot remains
intact; no Goal predicate, fixture, limit, pure algorithm or runtime result changes.


## MAIN backend gate — 2026-10-26

Reviewed R4 integrated over current133/33 without changing bundled definitions.
Twenty pure checks and21 app integration groups pass, together with69 library
checks. Actual currentVM9 records and genuine6/7/8 histories survive raw snapshots,
separate pending editors, favorites, load/export/inspect/restore/selected reset.
One first-run fixture wrongly used14 bytes in an8-byte port draft; save correctly
refused it. The corrected bounded malformed value roundtrips; the overbound value
has its own rejection/target-preservation regression. No admission was widened.

An immutable nonserializable internal RequestView validates and parses one borrowed
State once per request batch. Public single-request shape-before-state precedence
and all20 pure vectors remain intact; active/stopped batch equivalence and malformed
request/invalid-State precedence have a new library regression. The same maximum
32-probe case measured91.05ms before and30.77ms after in unoptimized tests on this
Fedora host. This is a bounded observation, not a universal frame-time guarantee.
UI must still avoid repeating complete grading while idle. No response cache or
persisted result grants credit. The four normally bundled tasks now have model,
content and development-native editor checks; see STATIC_WEB_LESSONS.md and
APACHE_STATIC_NATIVE_ACCEPTANCE.md. Final polished installed acceptance remains
pending. A backend alone does not add curriculum or objective coverage.
