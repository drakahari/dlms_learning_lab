# Architecture and content authoring

One native Rust package; no browser runtime, plugin platform, shell lab execution, server, VM, container, or required DLMS integration.

| Module | Responsibility |
| --- | --- |
| `engine.rs` | Typed simulation state, constrained command parser, live grading for nine activity types |
| `ipv4.rs` | Boundary math, host validation, numerical variants and retained legacy lesson/sandbox metadata |
| `workspace.rs` | Versioned draft/check metadata, compatibility preparation, pure editor operations and state-specific diagnostic feedback |
| `pack.rs` | Versioned declarative JSON validation and reference-solution reachability |
| `catalog.rs` | Search/filter predicates and a marked synthetic metadata fixture |
| `progress.rs` | Local-only loading, compatibility backup, atomic synced saves and invalid-file preservation |
| `ui.rs`, `ui/workspace.rs`, `ui/ipv4_workspace.rs` | Normal native library, tool-specific workspaces, approved IPv4 form, optional assistance and separate host-bit activity |
| `ui/smoke.rs`, `ui/tests.rs`, `tests/` | Native rendering/input captures, UI and model/storage regression tests |

## Simulation boundary and grading

The engine and pure workspace/editor functions use enum/struct state. They contain no filesystem, subprocess, shell or networking I/O. `std::net::Ipv4Addr` is only a parser/value type. Paths in terminal commands are simulated map keys. Unsupported input returns an explicit limitation. Host filesystem access belongs only to pack validation, progress storage and the opt-in native screenshot harness; learner input cannot choose those paths.

The GUI never substitutes a success string for a modeled repair. Grading inspects current state: VLSM capacity/alignment/containment/overlap; POSIX owner precedence, group membership and every parent’s search permission; current log-query text; saved versus loaded service/DNS configuration; boot enablement; and persistent/current mount state plus capacity. Valid alternate command sequences remain accepted. The IPv4 grader additionally accepts any free usable second host when the new optional field is present. /31 and /32 are excluded.

Completion is derived, never a permanent persisted success flag. Check actions expose feedback. Editing a checked IPv4 form invalidates its verification snapshot; the topology stops showing a previous green status for a changed host. The library still reflects mathematical current-state completion. A numerical variant has its own answers/check snapshot and never changes original credit or catalog size.

Configuration editors make explicit, pure modeled saves. Service saves do not restart the service; DNS saves do not reload the authoritative zone; fstab edits do not remount the volume. Draft text is stored separately from saved simulation state, so interrupted work can resume without pretending it is applied. A terminal mutation refreshes a matching untouched editor draft while preserving deliberately unsaved edits.

## Local progress and compatibility

The envelope and scenario versions remain 1. Existing optional service journal fields and IPv4 lesson metadata are retained. Old section/review positions are data for optional guidance, not gates on the assignment. Original/practice answers stay independent.

`Subnet.second: Option<String>` and `Exercise.second` distinguish old four-answer state from the approved complete assignment. On launch, an incomplete old analysis gains a blank fifth field. A completed old analysis gains the first free usable host (skipping the existing workstation) and a compatibility notice, retaining all original answers and completion. This is a documented migration, not a new authored scenario or evidence that a learner manually selected that address.

`Attempt.workspace` is optional; its own version is 1. It stores bounded command/editor drafts, a check visibility flag, original/practice IPv4 check snapshots, and migration notices. Loading never rewrites the input file. The first upgraded write backs up an existing pre-workspace save byte-for-byte using a same-directory temporary file and no-clobber rename. Later saves do not replace that backup. Main saves use synced temporary files and atomic replacement; Unix also syncs the directory. Invalid/corrupt/future data is preserved with a visible read-only explanation. Earlier binaries cannot read newly added fields; downgrade and concurrent writers are unsupported.

## Authoring an existing-state scenario

1. Add declarative data in `scenarios/starter-v1.json`: stable unique ID, positive version, supported certification/topic, title, short goal, difficulty and duration.
2. Use a typed initial state and compatible goal. Include `second: ""` for a new complete IPv4 assignment. Keep initial state unsolved.
3. Supply optional guidance (`steps` and matching `guide`), progressive hints, a worked explanation, official/technical source links and honest simplifications. Complete objectives must remain understandable without opening a walkthrough. Mark simulator helpers explicitly; use editors for configuration and terminals for investigation/verification.
4. Provide a `solution` list for reachability validation. It is parsed only by the in-memory simulator. `answer`, `allocate` and configuration helpers can serve as pack/test adapters; they are not the learner workflow for native forms/editors and are never executable scripts.
5. Rebuild and run `--validate-pack scenarios/starter-v1.json`, relevant success/failure/alternate-solution tests, and the actual native workspace. Unknown schema/fields, duplicate IDs, invalid state and unreachable goals are rejected. Arbitrary-pack loading in the learner GUI remains deferred.
6. An incompatible future schema/version requires an explicit migration; never silently clear learner work. This revision retains existing versions because old optional-field omissions are handled explicitly and tested.

## Native review

`./target/debug/dlms-learning-lab --ui-smoke` runs the **normal** app with fresh isolated progress under `/tmp/dlms-workspaces-smoke`, injects egui input and captures application framebuffers to ignored `screenshots/workspaces/`. It captures all nine Learn/Challenge screens, actual IPv4 field/check/host invalidation, service-file save and modeled verification, terminal Enter/query invalidation, real resizing, 150% text, optional filters and a synthetic large catalog. Screenshot acknowledgements keep captures synchronized. This is native renderer/event testing, not physical-device or screen-reader acceptance.

`--developer` exposes the collapsed 900-item synthetic metadata fixture. It never adds playable curriculum. Preview-only UI and the former `--ipv4-smoke` flow have been removed; normal launch needs no special flag.


## Scripting and theme milestone

`src/shell.rs` is a pure, unsafe-forbidden lexer, recursive-descent parser, bytecode evaluator and virtual text filesystem. It has no host-I/O/process/network imports. Each case has declared environment, arguments, stdin and initial files; expected stdout, stderr, status and complete resulting files are declarative data. A run records the exact source and case outcomes. Grading compares behavior, never solution text; editing the source invalidates the snapshot. Compile/runtime/limit errors fail the case. Parser/token/depth, command-argument, virtual-memory/file, output and instruction bounds are explicit. `tick(16)` advances each native frame; Stop drops the job. No unbounded loop runs on the UI thread.

`src/ui/scripting.rs` owns the transient multi-case job and editor/output layout, while typed `State::Script` stores resumable source and completed outcomes. In-flight execution is not serialized or automatically resumed. The original command simulators remain separate. `examples/shell_probe.rs` is a development-only JSON oracle executable; `tools/bash_reference.py` runs checked-in fixtures in temporary directories with real Bash for comparison. Neither is the app's learner engine.

Author unit content in `scenarios/shell-unit-v1.json`. Use a stable new ID/version, advisory prerequisite IDs, an unsolved starter, a meaningful complete assignment, progressive hints and 3–8 deterministic `Case` records (the delivered unit uses 3–5). Expected `after` maps include unchanged input/unrelated files, so accidental destructive writes fail grading. Include spaces, empty inputs, missing resources and meaningful status differences. Reference scripts run only through the bounded interpreter during pack validation. IDs, case bounds, reachability and acyclic prerequisites are checked. A new syntax feature requires grammar/expansion/runtime/error/oracle tests before content may depend on it. No generated numeric variants count toward authored totals.

`src/theme.rs` loads immutable palette data and derives native semantic roles with contrast targets. `assets/themes/` holds the complete pinned inventory and third-party notices; no executable theme files or desktop settings are used. `preferences-v1.json`, beside progress, stores the theme ID and text scale using a synced temporary file and atomic replacement. A malformed/future settings file is retained; appearance falls back and reports its own failure without blocking progress. Progress has a 16 MB read/write ceiling to accommodate bounded virtual run records; over-limit writes retain the original. Original progress schema/IDs/versions remain unchanged, and pre-workspace backup compatibility remains intact. New scripting attempts require the newer binary; downgrade is not promised.

The full top-level objective plan is in `CURRICULUM.csv`; it is not embedded as fake playable labs. The native library has four tracks and retains large-catalog navigation. `--ui-smoke` now captures all themes plus all scripting assignments and original workspaces using only synthetic progress.
