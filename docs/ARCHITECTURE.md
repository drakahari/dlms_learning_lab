# Architecture and content authoring

One small Rust package with explicit module boundaries; no plugin host or integration requirement.

| Module | Responsibility |
| --- | --- |
| `engine.rs` | Typed IPv4, subnet allocations, topology, POSIX permission trees, logs, service lifecycle, DNS and storage models; supported-command parsing; current-state grading |
| `pack.rs` | Strict versioned declarative JSON loading, metadata/state validation, identifier uniqueness, and reference-solution reachability |
| `catalog.rs` | Search/filter predicates and opt-in synthetic catalog fixture |
| `progress.rs` | Versioned attempts, local-only loading, safe atomic replacement and corrupt-save preservation |
| `ui.rs`, `ui/workspace.rs` | Native egui/eframe library, responsive workspace, native fields, simulated editor, completion and focus |
| `ui/smoke.rs`, `ui/tests.rs` | Native event/capture harness and headless UI regression checks |
| `main.rs` | Native startup and the explicit author-facing `--validate-pack PATH` utility |

`engine.rs` imports no filesystem, subprocess, shell or networking client API. Learner input is split into a constrained grammar and changes enum/struct fields or reads modeled values. Path-like command arguments are map keys, never host paths. The engine uses `std::net::Ipv4Addr` for parsing and arithmetic only. Unsupported commands and arguments return explicit limitations. The GUI has no external-link opener and no network client. Desktop integration dependencies may communicate with the local window manager, clipboard and accessibility bus.

Completion is derived, never a sticky persisted success flag. Every practice frame, Check work and library status evaluates current state. Completion never depends on walkthrough position. The worked explanation is shown only after current success. Challenge does not render guide examples before success. Native fields write the same typed answer/allocation state that old attempts already used. Unix access checks honor owner precedence, supplementary groups, and execute/search on every directory. Service config and loaded runtime config are distinct; boot enablement is independent. DNS edits and loaded answers are distinct. VLSM grades containment, alignment, capacity and pairwise overlap, allowing alternate layouts.

Progress stores state plus navigation metadata, not command recordings outside the local transcript. Writes serialize to a same-directory temporary file, flush and sync, then atomically replace the destination; Unix also syncs the containing directory. Version mismatches and invalid saves are surfaced without overwriting their original file. Version-1 compatibility is retained: all IDs, scenario versions and section counts stay unchanged. New service `failed` and typed `journal` fields have serde defaults, so old saves retain their exact existing fields without a read-time rewrite. Old attempts have no invented event history. New saves add these fields and require the newer binary; downgrade compatibility is not promised. The checked-in legacy fixture is synthetic and exercises all nine attempts. General cross-version migrations remain deferred. Concurrent writers are unsupported.

## Add an authored scenario

1. Add an object to `scenarios/starter-v1.json` with a unique lowercase ASCII `id` (digits/hyphens allowed), positive scenario `version`, supported `track`, `topic`, plain-language `title`, `summary`, `objective`, `difficulty`, and `minutes`.
2. Choose an existing tagged `initial` state and compatible tagged `goal`. See the Rust enum definitions for the schema; unknown fields are rejected. The pack envelope has `schema_version: 1`.
3. Keep `steps` for compatibility and supply a matching `guide` array. Each guide section has `title`, `why`, `action`, and a `commands` array of separate `command`, `explanation` and `simulator_only` examples. Native-form sections can have no commands. Include an `explanation` for the completion panel, progressively helpful `hints`, HTTPS `sources`, and honest `simplifications`. Do not copy exam questions or source prose. Include a reference `solution` as a list of supported simulated commands. These are data used to prove reachability, never executable scripts. Legacy `answer`/`allocate` actions remain solely as pack/test adapters; the learner UI for those labs contains native fields and no command terminal. Inserts in Learn mode set the input text only; execution requires a separate Run/Enter action.
4. Validate with `./target/debug/dlms-learning-lab --validate-pack scenarios/starter-v1.json`. Validation rejects unsupported schemas, duplicate/invalid IDs, incomplete/oversized metadata, malformed initial state, already-solved labs, incompatible objectives, unsupported solution commands and unsolved reference solutions.
5. Add meaningful failure, alternate-solution and post-success regression tests. Run tests and launch the native workspace. The production binary embeds the bundled pack, so rebuild after editing it. Loading arbitrary external packs into the learner UI is deliberately deferred.
6. Increment scenario version for incompatible state/content changes. Existing saved attempts with different versions are preserved and rejected, not silently reset. Implement migration before distributing such a change to learners.

To add a new simulation behavior, extend the typed state and objective enums, command interpreter, checks, validation and tests together. Avoid string-to-success shortcuts. Keep storage and host integration out of the engine. No external DLMS service is necessary.

## Local GUI verification

`./target/debug/dlms-learning-lab --ui-smoke` opens a real native window, injects egui text/key/pointer/scroll events, checks field input, completion/invalidation, terminal Enter/Tab/Shift+Tab and the service editor Save button, captures the application framebuffer under ignored `screenshots/after/`, and exits. It starts with fresh isolated progress under `/tmp/dlms-ui-v2-smoke`, never normal learner data. Capture acknowledgements prevent later phases from contaminating earlier screenshots. Window resizing is exercised through full-screen transitions because COSMIC constrains floating sizes. Layouts and filters also run at 150% text size in the tiled window. This is event injection at the native egui boundary, not physical-device or screen-reader acceptance.

`--developer` exposes the collapsed synthetic-catalog option. It is absent from normal launch. UI tests cover dynamic resize/zoom changes, aligned filter/control geometry, native-form bounds, output/input proximity, challenge visibility, insertion without execution, reset/resume and pagination. See VERIFICATION.md for the actual run results.
