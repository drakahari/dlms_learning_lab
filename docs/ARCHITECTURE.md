# Architecture and content authoring

One native Rust package; no browser runtime, plugin platform, shell lab execution, server, VM, container, or required DLMS integration.

| Module | Responsibility |
| --- | --- |
| `engine.rs` | Typed simulation state, constrained command parser, live grading for current activity types |
| `filesystem.rs`, `terminal_teaching.rs` | Bounded ordinary-user virtual file tree and separate persisted ungraded terminal practice |
| `ipv4_routing.rs`, `ipv4_routing_workspace.rs`, `ipv4_routing_teaching.rs` | Bounded IPv4 static forwarding, endpoint-bound requirements, atomic native drafts and isolated saved practice; six authored route/recovery tasks |
| `switching.rs`, `switching_workspace.rs`, `switching_teaching.rs` | Bounded VLAN broadcast data plane, atomic native port drafts and isolated practice; no IOS interpreter or control protocols |
| `ipv6.rs` | Bounded prefix math, static forwarding/return paths and typed behavioral requirements |
| `service_teaching.rs`, `ui/service_learning.rs` | Single-service beginner teaching, separate bounded editor/terminal practice, current checks and namespace isolation |
| `address_teaching.rs`, `ui/address_learning.rs` | Bounded subnet/VLSM introductions and isolated native examples, current-state checks and immutable supplied problems |
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

`Subnet.second: Option<String>` and `Exercise.second` distinguish old four-answer state from the approved complete assignment. On launch, an incomplete old analysis gains a blank fifth field. A completed old analysis retains separate legacy credit and a blank new host field. Earlier binaries sometimes supplied a host; existing values are retained but require explicit review/check before current completion. No learner answer or new mastery is invented.

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

`src/script_teaching.rs` validates declarative introductions and independent saved example drafts; `src/ui/script_learning.rs` renders the skippable Learn introduction/example/assignment views. Example jobs use the same VM but write only optional `WorkspaceData.script_learning`, never assignment results. Existing attempts reopen the assignment; new Learn attempts start at Introduction. See [the teaching audit](SCRIPT_TEACHING.md).

`src/ui/scripting.rs` owns the transient multi-case job and editor/output layout, while typed `State::Script` stores resumable source and completed outcomes. In-flight execution is not serialized or automatically resumed. The original command simulators remain separate. `examples/shell_probe.rs` is a development-only JSON oracle executable; `tools/bash_reference.py` runs checked-in fixtures in temporary directories with real Bash for comparison. Neither is the app's learner engine.

Author unit content in `scenarios/shell-unit-v1.json`. Use a stable new ID/version, advisory prerequisite IDs, an unsolved starter, a meaningful complete assignment, progressive hints and 3–8 deterministic `Case` records (the delivered unit uses 3–5). Expected `after` maps include unchanged input/unrelated files, so accidental destructive writes fail grading. Include spaces, empty inputs, missing resources and meaningful status differences. Reference scripts run only through the bounded interpreter during pack validation. IDs, case bounds, reachability and acyclic prerequisites are checked. A new syntax feature requires grammar/expansion/runtime/error/oracle tests before content may depend on it. No generated numeric variants count toward authored totals.

`src/theme.rs` loads immutable palette data and derives native semantic roles with contrast targets. `assets/themes/` holds the complete pinned inventory and third-party notices; no executable theme files or desktop settings are used. `preferences-v1.json`, beside progress, stores the theme ID and text scale using a synced temporary file and atomic replacement. A malformed/future settings file is retained; appearance falls back and reports its own failure without blocking progress. Progress has a 16 MB read/write ceiling to accommodate bounded virtual run records; over-limit writes retain the original. Original progress schema/IDs/versions remain unchanged, and pre-workspace backup compatibility remains intact. New scripting attempts require the newer binary; downgrade is not promised.

The full top-level objective plan is in `CURRICULUM.csv`; it is not embedded as fake playable labs. The native library has four tracks and retains large-catalog navigation. `--ui-smoke` now captures all themes plus all scripting assignments and original workspaces using only synthetic progress.

## Course units and split content packs

`scenarios/units-v1.json` is a versioned, bounded unit index: stable IDs, track, title, short description, advisory prerequisite units and ordered lab IDs. Every bundled lab belongs to exactly one unit of the same track. Missing/duplicate labs, invalid IDs and prerequisite cycles fail validation. Unit order suggests a path; no learner is locked out. Completion offers the next unfinished assignment in that unit, including earlier skipped work, and never silently jumps certification tracks.

Scenario `unit` metadata does not change saved attempt IDs or versions. The library preserves track/search/favorites/resume while adding a scoped unit filter and topic options relevant to the selected track/unit. A course entry opens the first unfinished lab; new scripting attempts start at Introduction. All assignments remain directly accessible.

`pack::load_catalog` bounds each declarative pack to 2 MB, a bundle to 100 packs/32 MB and 1,000 scenarios, then validates global identifiers and prerequisite graphs without serializing a giant synthetic pack. This supports distinct versioned units without executable loaders or a plugin platform. Single-pack validation remains strict. `--validate-content` checks the actual complete bundled catalog and unit index; alternate/negative solution checks remain separate content tests.

File-management lessons use the same approved Introduction → Try example → Assignment pattern with a separately persisted virtual tree; see [LINUX_FILES.md](LINUX_FILES.md). IPv6 capability limitations and reference tests are documented in [IPV6_SUBSET.md](IPV6_SUBSET.md). Six authored IPv6 lessons use native device/interface/route editors with atomic drafts and isolated practice; see [IPV6_LESSONS.md](IPV6_LESSONS.md). This is partial static-network coverage, with dynamic protocols explicitly excluded.

### Shell evaluator revisions and retained work

`Record.engine_revision` separates current evaluation from older cached results; omitted values identify legacy records. Increment `shell::EVALUATOR_REVISION` when corrected semantics or limits can change previously saved outcomes. Current grading requires source/revision/result-count agreement. Scenario requirements still need version review when authored cases change; a nearby passing case is not evidence for new inputs.

Older source/output/file snapshots move, without duplication, to bounded optional `WorkspaceData.script_history` plus `script_history_more` for successive revisions when opened. Earlier credit is computed separately against the unchanged authored expectations; it never substitutes for current success. A current Run and behavior checks are required after an evaluator upgrade. Example output has its own evaluator revision and cannot award assignment credit. The first upgraded write retains exact original bytes in a non-overwriting evaluator backup; app-owned migration filenames are reserved from exports. See PROGRESS_RECOVERY.md.

Function-local storage uses the same visible variable map plus bounded per-call shadow records, matching Bash dynamic visibility. The256 ordinary-name limit includes unset declarations across active frames, and name/shadow/input bytes count toward virtual memory. Unsupported special variables and reserved namespaces fail explicitly. This is a deliberate subset, not a general Bash interpreter or host launcher.


Pure text utilities live in `src/shell/text_tools.rs`: typed option parsing and bounded execution on borrowed text only. Virtual input resolution and pipeline routing stay in the VM. Sort/uniq/cut use explicit C/POSIX byte behavior; supported locale overrides are checked. Cut ranges are merged and scanned in order, avoiding numeric-sized allocations and per-field scans of every range. Max8192 lines,128 input ranges and32KiB output bound each invocation. The native UI continues cooperative VM slices; no host lookup or utility subprocess exists.

Evaluator revision2 fixes pipeline input ownership: only the first stage consumes the caller’s original finite input; later stages consume their preceding pipe text. Command redirection restores the first-stage input cursor correctly, while pipeline variable isolation remains unchanged. Current records from revision1 require rerun; preserved historical credit does not imply present correctness.


`src/shell/arithmetic.rs` parses the disclosed integer-expression subset with bounded tokens/depth and explicit i64 wrapping operations. Source capture preserves arithmetic text until runtime; explicit parameter/nested expansion precedes expression parsing, and bare names resolve through the current simulated scope. No state mutation, host call or dependency exists in this module. Fatal expansion diagnostics use simulator status2.

Evaluator revision3 corrects assignment/redirection order: ordinary command and local-declaration arguments expand before redirects, while no-command assignments apply sequentially before redirects. Assignment recognition uses one first-bare-part predicate across command execution and function/pipeline head checks, preserving quoted fragments and avoiding premature field splitting. A new syntax feature does not by itself grant old credit; prior0/1/2 results remain distinct historical records, requiring current evaluation for current success.

Sequential while-file input uses typed input/cursor frames, restored at loop completion or function return. Saved text/names count toward the same memory limit. Active sources are read-only: writes are rejected before truncation/append rather than approximating changing-open-file behavior. Other compound redirection forms remain unsupported. See SHELL_COUNTS.md for dependent teaching and authoring cases.

Ordinary-user file access uses explicit typed operation requirements with declared simulated subjects. Access checks require an existing correct target kind, select owner/group/other bits in that order and require search through all parent components. The same permission predicate drives current-user file operations. Attribute changes validate ownership/group membership on a cloned candidate; unsupported symbolic modes never partially apply. Deterministic ID labels are derived from bundled account metadata, never discovered from the host.


### Switching capability authoring boundary

Use `State::Switching` with a validated fixed physical forest and `Goal::Switching` with typed requirements. Visibility goals check both directions; every isolated endpoint must also occur in a positive service case. Negative outcomes require up, active access attachments and a live physical path. Preserve intended access memberships through explicit AccessVlan outcomes; NativeVlan and ActiveVlan can enforce stated constraints. Internal `access`, `trunk`, `vlan` and `port` actions prove declarative reachability only; they are never taught as IOS commands. Native port tables are the learner tool.

Draft application is atomic, and current success rejects unapplied divergent settings. Optional `switching_draft`/`switching_learning` fields preserve malformed bounded text and separate examples without changing old attempt IDs/versions. Import validates fixed workstation/cable/port structure against the assignment or example, after the existing whole-document byte cap. Practice never gives assignment credit. See SWITCHING_SUBSET.md for platform-policy and STP/native-mismatch limitations. The development-only switching-preview test fixture is not bundled curriculum; no objective evidence is gained from a capability preview.


Evaluator4 bounds retained text/field count inside the shared expansion layer before append/allocation, including direct non-command consumers and quoted "$@". A per-VM Cell counter charges source/parameter/result/field work before scanning or copying, shared through nested arithmetic callbacks, functions, pipelines and later ticks. It is a finite resource counter, not a clock or host service; each case/new VM starts fresh. Exact evaluator3 history remains separately visible and non-overwriting v4 backup applies through the existing storage path. Large supplied-value inline UI previews truncate safely before escaping/formatting; existing full input details have bounded scrolling, so a legal large fixture cannot push the editor/Run/Stop behind a whitespace wall.

Filename matching lives in shell/globbing.rs: iterative C-byte matching over the existing flat virtual map, fed per-byte quote provenance after splitting. It has no host glob/filesystem API. Matching shares the interpreter work limit; unsupported bracket/directory/recursive/non-C features fail explicitly. Evaluator5 retains earlier actual credit with the existing history/backup mechanism.


### IPv4 routing integration boundary

`State::Ipv4Routing` is separate from the approved original subnet worksheet and from IPv6. Typed outcomes bind a named endpoint/interface; current mutable IP ownership never selects the intended grader target. Fixed-address, subnet and forwarding constraints are explicit. `StaticRouteFor` asks for an actually selected static prefix serving the named endpoint, rather than accepting a dummy unused route alongside a host-route workaround. Controlled operational failures clone topology; positive normal-state reachability is mandatory. No negative-policy credit is inferred from failed probes.

The normal native renderer uses device/address/route editors with one Check configuration action. It parses a clone, commits all valid settings together and reports current outcomes. Unfinished text remains saved and cannot borrow success from earlier applied state. Detailed results, forwarding paths, walkthrough and references are optional. One-interface router forwarding remains available through optional controls; native tools must be able to express every supported authored reference action.

`routing_v4_draft` and `routing_v4_learning` are optional omitted/defaulted fields in the existing version1 workspace. The latter owns a separate topology, raw draft, selected view and check result. Import pins each topology's device/interface/cable shape to its assignment or practice fixture and rejects foreign routing metadata. The clearly named `tests/fixtures/ipv4-routing-preview.json` is only a development fixture, never bundled authored curriculum or objective evidence. See IPV4_ROUTING_SUBSET.md for exact scope, tests and future content requirements.


Six reviewed IPv4 static-routing assignments now use this integration; see [IPV4_LESSONS.md](IPV4_LESSONS.md). Conditional StaticRouteFor outcomes require actual selected network prefixes under the same bounded cloned states as reachability. A broader backup default cannot masquerade as a requested /24. Native route forms use aligned table rows when wide, labeled stacked fields when narrow, selected-device adjacency as the main context and optional whole-topology inspection. The interface picker initializes focus only after enabled popup contents and pointer release; both keyboard-only and pointer opening permit Tab/Enter selection without stealing subsequent navigation. Current applied-only results are suppressed while a divergent raw draft is pending, including on resume.
