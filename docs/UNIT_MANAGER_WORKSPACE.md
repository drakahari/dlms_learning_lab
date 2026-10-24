# Multi-unit workspace — backend adapter draft

Source-only2026-10-23. New pure workspace/teaching modules, not registered in
State/Goal/pack/progress/native UI, not bundled curriculum or objective coverage.
Guarded first compilation passes47 library,16 core and14 adapter tests plus
all-target Clippy. These standalone tests do not establish shared-hook progress
compatibility or native acceptance. Existing single-service labs, progress,
shell evaluator and migration receipts are untouched.

## Editing and fixed context

Definition pins its exact initial Inventory; editable saved unit/config text,
loaded definitions, runtime, boot links and default remain separate core layers.
Draft has exact-key bounded raw maps and selected fixed IDs. It preserves bad
text. Applying a complete draft validates shape/text limits before one candidate
save; it does not parse/reload/start/enable anything. Structural failures publish
nothing. Core's strict document loader is a separate operation.

Typed EditAction values are internal native-control/authoring data, never Linux
commands or scripts. There is no learner command parser or host engine. All raw
config/program names stay within the fixed core Inventory. An optional future
pack adapter must label its data prefix authoring-only.

## Current and fresh checks

Definition has bounded current requirements, one fixed fresh-session group and
explicit preservation metadata. Both current and fresh groups require positive
workload outcomes. Successful runtime outcomes require a paired ExecutionCurrent
check. A no-remain oneshot can prove its work through a re-evaluated artifact,
not inactivity or an old report. Fresh artifacts must have the expected producer
and current unit/config/prerequisite input; a currently failed producer cannot
hide behind a retained old artifact.

LoadedCurrent means saved and manager-loaded source are coherent. It does not
prove that any action was performed. ExecutionCurrent additionally requires a
successful execution snapshot matching saved and loaded source, current consumed
application configuration and prerequisite value. Edits that return to identical
consumed values can regain coherence without inventing action-history rules.
Dispatched simple services are not running/readiness; their initialization is an
explicit in-process event.

Checking derives all results from the supplied current State and raw draft.
Unsaved divergent text always prevents success. A checked flag, emitted event,
report, transcript or walkthrough position cannot grant completion. Messages are
bounded512 UTF-8 bytes. Whole-state equality to one model answer is not grading.

The fixed fresh test clones current state, clears runtime/artifacts, loads saved
units, starts the selected saved default and explicitly initializes pending
simple programs. The caller stays unchanged. Its newly produced start report
and derived runtime/results are checked separately. Opaque core StartOrdering
comes from the same actual start transaction; it contains only its job-induced
edges. The workspace never reconstructs target/link semantics, accepts cached
reports, treats a full graph path through an absent unit as ordering, or treats
an earlier layer index as a guarantee. There are no goal-supplied scripts or
arbitrary replay plans.

Requirements can explicitly ask for a saved Requires/Wants/After/Before property;
only then is that exact property required. Behavioral fresh ordering accepts
equivalent Before/After paths. Comments and other supported valid spelling may
vary if the saved/loaded/executed layers are current and preservation allows it.

## Explicit preservation

The definition names unchanged raw unit documents and application configs. It can
preserve default-target selection and all boot-link pairs except named editable
pairs. Requirements contradicting those preservation rules reject. Unaffected
runtime is checked through named semantic outcomes, never opaque snapshot
identity. Authoring must show these preservation boundaries in the assignment;
there is no hidden full-configuration or action-history requirement.

## Teaching and persistence boundary

Lesson owns a distinct definition/fixture, short concepts, a runnable annotated
typed-action sequence, solved-source actions, a concrete try change with checked
outcomes and repair. Validation runs these only inside pure bounded State clones;
no host reference utilities or learner shell execution exist. Learning owns
an exact immutable authored fixture copy as origin metadata, separate mutable
State, raw Draft, selected action, view, bounded transcript and check visibility.
The fixture must equal Lesson.definition.initial in valid_for; mutable current
values can legitimately coincide with other workspaces. The origin marker never
grants success; grading still derives current outcomes from the mutable State. Fill worked source changes its draft only. Saving, reload, start,
initialization and Check remain separate explicit actions. Clean draft source can
synchronize after typed actions without losing selected document; unsaved edits
remain untouched. Example reset replaces only Learning.

The eventual engine/progress adapter must validate State/Goal kind pairing,
original assignment equality, Inventory pinning and Learning's exact authored
fixture on import/save. Evolving examples need not retain disjoint namespaces.
Optional fields belong to existing workspace version1; no old Service migration
or shell evaluator bump. Current modules cannot access an assignment Attempt or
modify progress. Existing progress/history/export/reset compatibility still needs
the later shared-hook tests; it is not claimed by these standalone modules.

Collections/maps use bounded duplicate-rejecting visitors. Borrowed
Lesson::validate_bounds checks action counts, fixed context, fields and Draft
shapes before every teaching clone or replay, including save_draft. Direct callers
cannot bypass the32-action bound by constructing a larger Vec themselves. The
private replay helper repeats the borrowed count/action preflight; no serialized
copy is used to measure validity. Failed bounds checks retain State and drafts.

Lesson::from_json and Learning::from_json each impose an8MiB raw input ceiling;
the latter validates the supplied lesson before parsing and pins exact practice
origin afterward. Generic serde callers must supply their own outer byte bound.
The normal whole-pack2MB and progress16MB limits remain independent and may bind
first; individually bounded fields do not promise every possible combination can
fit an outer container. No oversized data is silently truncated. The executed maximum escaped2State+Draft/transcript test measures compact/pretty
Learning at5,027,909/5,037,042 bytes, below its8MiB cap. The bounded Lesson is
2,632,173/2,637,813 bytes: valid for the standalone8MiB reader but larger than the
normal whole-pack2MB limit, which would independently refuse such a giant pack.
Compact/pretty roundtrip passes and a clearly labeled synthetic size-only16MB
envelope also round-trips. That envelope is not the app's
Progress type and proves no actual migration or profile acceptance. The private development fixture
`tests/fixtures/unit-manager-workspace-preview.json` is not a Scenario pack and
is not counted as authored content. Its reference and Before-alternative exercise
multi-unit configuration, state freshness and enablement independently.

See [UNIT_MANAGER_SUBSET.md](UNIT_MANAGER_SUBSET.md) for the versioned official
systemd262/LPI sources and exact loader/job/concurrency/readiness limitations.
No complete systemd, real boot, process, network, LPIC or runtime claim is made.
