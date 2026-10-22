# Implementation ledger

Completion authority: [COMPLETION.md](COMPLETION.md). Every milestone records changes, actual evidence, open failures and the next implementation work. No automatic closure based on lab count.

## 2026-10-22 — M1: wording and completion baseline

- Started from `69f695d`; accepted native workspace and teaching pattern retained.
- Replaced “Read the contract” with “Understand the task.” Sample-input wording replaces “fresh virtual workspace”; explains simulated file resets and retained drafts. Replaced learner-facing “fixtures” in optional walkthrough steps/hints.
- Rechecked official Cisco v1.1/v2.0 and LPI 5.0/4.5 sources; check date recorded in curriculum documentation. Current and upcoming blueprints remain separate.
- Added the completion standard with explicit incomplete coverage, simulator, UX, recovery, installer and platform acceptance gates.
- Validation: 65 existing Rust tests pass. This is a wording change, not additional curriculum or a claim of native/release acceptance.
- Independent baseline review started. A permissions privacy grading defect is queued for immediate repair; broader content and simulator work remains open.

Next: resolve review findings; implement safe progress backup/export/restore and course/unit navigation; extend bounded simulator capabilities and author reviewed curriculum batches. License and actual Windows/macOS acceptance remain owner/environment decisions, not reasons to stop independent implementation.

## 2026-10-22 — M1b: owner-approved license

The owner explicitly approved MIT for original code and authored curriculum, using “DLMS Learning Lab contributors.” Added root LICENSE/NOTICE and package metadata; retained palette attribution. This resolves the project-license decision, not the remaining dependency-notice or platform-acceptance gates. No public release.

## 2026-10-22 — M2: reviewed grading and persistence integrity

- Independent read-only review reproduced long-command diagnostics that made saves unreadable, exclusive-file privacy bypass, legacy host invention, concurrent-writer loss and ambiguous utility operands.
- Diagnostics now truncate on UTF-8 boundaries at creation; invalid in-memory state cannot replace a valid save. Regression covers assignment/example failures from long ASCII and Unicode sources through save/reload.
- Alex-only privacy now explicitly checks all modeled non-administrator users as well as other-class bits. Team traversal retains its distinct shared-access goal. Feedback identifies an unauthorized modeled user.
- New host requirements stay blank on migration. Historical analysis credit is displayed separately; values supplied by an older binary remain intact but need an explicit new check. Review/check persists, and broken answers invalidate current success.
- Ambiguous option-like grep patterns/file operands and cat operands fail honestly. Malformed numeric tests explicitly disclose the current unsupported boundary rather than claim faithful recoverable Bash behavior.
- Regression: 71 Rust tests, including old/new legacy UI flows, invalid-save retention, alternate privacy solutions and error-record roundtrips. Disposable 41-fixture Bash reference comparison is rerun. Full certification coverage remains open. Concurrent-writer protection follows in M3.

## 2026-10-22 — M3: progress recovery and writer protection

- Added native Settings → Your data, folder/file chooser, new-file-only backup export, cancellable bounded background inspection, explicit summary/confirmation and atomic restore of the inspected candidate.
- Exact original-file safety backup precedes replacement. Dirty writable sessions save first; edited read-only sessions receive a separate backup. Reserved application filenames, symbolic/special files, unknown/future/oversized data and forged script outcomes have explicit checks.
- OS writer lock is acquired before loading; a second copy cannot write progress or preferences. Lock files remain in place; OS ownership releases on exit/crash. Post-commit directory-sync failures report a durability warning instead of claiming the old file was retained.
- Independent read-only review exercised backend safety and found reserved-filename, unsaved-session and post-commit reporting edge cases, which were addressed. Oversized-current-file recovery deliberately requires a manual safe move, documented in PROGRESS_RECOVERY.md.
- 82 Rust tests pass, including seven backup tests, a controlled post-commit sync failure, two native UI data flows, background-shortcut isolation and prior regressions. Native Fedora capture pass exercises export, typed import filename, preview, confirmation, draft/favorite recovery, legacy disclosure and light 150% Settings. Captures remain ignored local artifacts; no personal profile is used.
- Full curriculum and cross-platform installer/runtime acceptance remain open; proceed to unit navigation and reviewed capability/content batches.

## 2026-10-22 — M4: unit navigation and scalable bundled authoring

- Added a versioned unit index covering every existing assignment, advisory prerequisite guidance and course entry points. Unit/topic controls share consistent constrained widths and wrap cleanly; track/search/favorites/resume remain available.
- Completion continues within the same unit and offers earlier unfinished work; it cannot silently jump to another certification. New scripting learners enter the approved introduction; experienced learners can open any assignment.
- Added bounded multi-pack validation with global dependency checks, plus `--validate-content`. No executable packs, plugin platform, dependencies or extra curriculum titles were added in this milestone.
- Validation: 88 Rust tests, Clippy, bundled validation; native Fedora course start/return, unit/prerequisite screens, completion, 900-item synthetic filtering and light 150% rendering. Visual review confirmed baseline alignment and readable wrapping.
- Implemented navigation foundation, not full curriculum coverage. Next is the authored shell composition/debugging unit with independent content review and per-lab alternate/negative behavior tests.

## 2026-10-22 — M5: reviewed shell composition unit

- Authored four distinct practical assignments: route output channels, intersect log filters through a pipeline, repair EOF handling and audit batches without hiding failures. Eleven shell assignments now sit in two coherent units; the original nine certification seeds remain unchanged.
- Every new concept has prerequisite guidance, a skippable introduction, separate runnable example and suggested experiment. Assignment/example drafts and credit stay isolated. The documented shell subset did not expand and no dependency was added.
- Independent content review found missing quiet-log and literal-substring cases, ambiguous EOF trimming wording and guidance omitted for file/status-only failures. Added explicit cases/negative tests and corrected feedback. All eleven shell assignments now have tested alternate solutions, plausible wrong solutions and case-1 overfit rejection.
- Validation: 92 Rust tests, 83 controlled Bash 5.3.9 comparisons with no differences, Clippy, formatter, bundled validation (20 labs/5 units). Automated layouts cover both modes, 720/1200 widths and 100/150% text. Native Fedora synthetic-profile pass traversed all four introductions/examples/edits/assignments/incorrect retries/correct completion, plus light/dark larger-text scrolling; actual screens reviewed. No personal profile, host learner execution, Windows/macOS acceptance or novice-user study.
- Curriculum map remains explicitly incomplete. Next extend functions/return/argument handling with faithful bounded evaluation before authoring reusable-interface lessons; complete certification models/content and installation candidates remain open.

## 2026-10-22 — M6: bounded reusable shell capabilities

- Extended the existing lexer/bytecode VM with portable brace-body functions, execution-time definitions, restored positional arguments, return/implicit status and shift. Global scalar behavior is explicit. Calls use a cooperative VM stack rather than Rust recursion; depth, retained argument/loop memory and instructions are bounded.
- Documented the subset before dependent content. Whole-function redirects and pipeline function calls remain honest unsupported diagnostics; ordinary return redirection is supported, including quoted/expanded names. No dependency or host-execution route added.
- Independent review exercised 36 additional disposable Bash comparisons and found an incomplete reserved-name exclusion and a documented return-redirection restriction bypass. Centralized reserved-name rejection; deliberately supported/tested real return redirection and reconciled documentation.
- Validation: 99 Rust tests, 102 checked-in Bash 5.3.9 comparisons with zero differences, Clippy. Tests cover nested calls/loops/definitions, argument restoration, shared scalars/input, status, excessive shift, unsupported contexts, recursion/memory/instruction limits and Stop. Existing native workspace UI test runs a function-based handover, invalidates it, stops a looping function and retains its draft. No new native-screen or platform acceptance is claimed for this evaluator-only milestone.
- No new authored lab or full functions-course coverage is counted yet. Next author reusable-function/return/argument lessons using this validated subset, then expand remaining capabilities and certification practice.

## 2026-10-22 — M7: reusable-interface curriculum

- Authored four reviewed assignments: reusable labels, item-level return vs process exit, consuming leading arguments and a validate-before-writing bundle capstone. Total: 24 authored assignments, six units; numerical/generated variants remain excluded.
- Each lesson introduces prerequisites and new concepts before assessment, with separate editable examples and suggested changes. Behavior grading accepts equivalent supported inline programs as well as helper-based solutions; no mastery is inferred from source shape.
- Independent review found an append-before-preflight false positive when the destination did not already exist. Added an absent-output failure case and explicit regression; clarified filename restrictions and aligned the exact suggested example edit.
- Validation: 100 Rust tests, 148 controlled Bash 5.3.9 comparisons, Clippy, formatter and bundled validation. All fifteen script assignments now have alternatives/plausible wrong/overfit checks. Four new assignments receive partial-completion and stale-result coverage. Native Fedora synthetic flow traversed each introduction/example/edit/assignment/retry/completion, with light/dark 150% scrolling; actual rendered examples, assignments and resized states reviewed.
- No Windows/macOS runtime or independent beginner acceptance is claimed. Complete certification coverage, advanced shell scope/expansion/structured-data/utility models and installation candidates remain open. Continue independent work; no release publication.

## 2026-10-22 — M8: installation and notice foundations

- Added local native candidate tooling with source/compiler/OS identities, embedded-content/binary hashes, archive checksums and no release publication. Original MIT approval retained. The target graph bundles original notices, font licenses, Rust library notices and checksum-verified original dependency archives, including MPL covered source. Public upstream notice cache records exact revisions and hashes.
- Linux per-user installer/launcher/menu and uninstall protect separate progress. Independent review found intermediate-link deletion, dangling desktop entry, overlapping-root/field-code, Windows shortcut and collector target-label issues; all received fixes. Twelve Python fixture tests cover install/upgrade/uninstall, retained progress/user files, unrelated shortcuts, link boundaries, malformed manifests, ignored-resource exclusion notice tampering, forced native-target provenance and exclusion of uncompiled local JSON.
- Fedora development candidate was extracted/installed under an isolated /tmp root, validated with desktop-file-validate and opened outside the repository. Native course navigation and light 150% rendering pass; progress sentinel retained. 100 Rust tests continue to pass. This dirty-source development check is not the final reviewed-source candidate identity.
- Actual Linux target has 223 normal/build dependencies and Windows has 137, with notice checks clear. Complete-lock optional hexf-parse and linked macOS dispatch original notices are unavailable; explicitly reported, not guessed or silently dropped. macOS packaging stays gated; source compile/test CI is separate from desktop acceptance.
- Manual native CI workflow is prepared for Ubuntu/Windows/macOS; runtime, signing/notarization and owner installation acceptance remain open. Actual dispatch/results and clean candidate identity will be recorded after the reviewed source commit. Complete certification/advanced-shell curriculum remains open; continue independent capability/content work.

### M8b: clean Fedora identity and first native CI findings

Clean candidate source `c164580b5cc9b5d856279e14a4890f60e01686ad`: Fedora x86_64 archive SHA-256 `aab299d3e9a64052ee81194290c93167a322a1cc065af37c547372cc3a5db045`. Clean candidate upgraded the isolated installation, validated its desktop entry, retained the progress sentinel and passed native course flow outside the repository. This remains milestone installation evidence, not complete curriculum/desktop acceptance.

Manual CI run `39792196866`: Ubuntu build/tests/candidate succeeded; Windows and macOS Rust/Clippy checks passed, then candidate-resource tests failed because fixture roots retained native temporary-directory aliases while production ROOT is canonical. Canonicalized fixture roots without weakening containment/link guards. Rerun results follow separately; no Windows/macOS desktop acceptance inferred. No public release.

## 2026-10-22 — M9: bounded Unix file-management capability

- Specified and implemented an explicit typed virtual tree, component-by-component path resolution, ordinary-user permissions, working-directory/home/previous navigation, hidden listings, file creation/copy/move/removal, empty directory removal and octal umask. All learner paths remain virtual; mutations validate a bounded candidate before committing.
- Independent review caught ancestor rechecking from an already-open cwd, forced-removal resolution distinctions and special bare-dash operands. Fixed these with direct inode checks, typed path errors and honest unsupported diagnostics. Copy versus move preserves different permission/metadata semantics. Deleted previous directories remain remembered paths and fail honestly on return.
- Validation: 111 Rust tests, Clippy and formatting; eleven filesystem tests include malformed paths/packs, alternate resulting states, partial/failure rollback, permissions, resource/output bounds and old/new progress roundtrips. 79 controlled GNU command comparisons pass; long listings compare only documented modeled fields. All 148 Bash reference fixtures remain unchanged and pass. Reference tools operate only on checked-in fixtures in disposable development directories and are never learner engines.
- No new authored lesson, native UI change or exam coverage is claimed in this capability milestone. The 24 existing assignments and original progress remain intact. Next add reviewed file-management teaching and independent terminal practice using this foundation.


### M9b: reference-fixture guard follow-up

The final oracle review identified an overbroad option exemption in the development-only GNU harness: future checked-in operands beginning with `-` could bypass containment checks. Replaced it with a command-specific option whitelist and validated every post-`--` operand, cwd and previous-directory path before subprocess dispatch. Two harness tests prove traversal, path-bearing options, absolute paths and unapproved commands never reach subprocess execution. The current 79 fixtures remain confined and pass. Added the guard tests to native CI. The application evaluator was not affected.

### M8c: native CI rerun and isolated uninstall

Run `39793992548` at `db54a2c` passed on Ubuntu24.04, Windows2025 and macOS15. Ubuntu/Windows candidates and checksum sidecars were downloaded and verified; identities/hashes are in INSTALLATION.md. macOS compiled but stays explicitly gated by dispatch notice provenance. These are native build/tests, not Windows/macOS desktop acceptance. The isolated Fedora M8 installation was uninstalled after closing its native smoke; owned launcher/application files were removed and its exact synthetic progress sentinel remained. No personal profile was used.

## 2026-10-22 — M10: reviewed Linux file-management foundations

- Six distinct authored assignments progress through navigation, literal quoted names, copy vs move, hidden-file cleanup, private creation defaults and a combined handover. Total: 30 assignments, seven units. LPIC-1 starts with these foundations in unit order; original IDs/order and all existing attempts stay intact. Only narrow 103.1/103.3/104.5 practice is counted; full certification objectives remain partial/open.
- Added skippable/reopenable terminal introductions and separate persisted ungraded practice Tree/input/transcript. A compact example-command picker inserts without running; the learner executes and experiments in the terminal. Challenge retains the full assignment without unsolicited teaching. Clear feedback recomputes from current state; existing tool-specific workspaces are preserved.
- Independent review corrected a noncanonical goal path, missing protection of the emptied queue directory and premature flags in early walkthroughs. Declarative example outcomes validate authored behavior; tests execute each actual suggested change. Alternate command sequences, plausible wrong/partial outcomes and breaking every objective are covered. Normal selected reset does not touch other labs; old and new attempts roundtrip together.
- Validation: 121 Rust tests; Clippy/formatter; 14 Python packaging/oracle guard tests; 79 GNU command comparisons and 148 Bash comparisons with no differences. Automated native-layout tests exercise all six introductions/examples/assignments, widths720/1200, text100/150%, Challenge isolation, Enter, Tab, Insert focus and draft retention. The Insert test exposed same-frame mouse focus loss, fixed by deferring field focus past pointer release.
- Actual Fedora synthetic-profile flow traversed all six introductions, first example command by keyboard, declared example outcomes, assignment error/retry, state solutions, current-success invalidation and save. Reviewed full-size dark screens and resized946x929 light150 screens with clean scrolling; reduced competing Insert buttons to one picker and reviewed that layout again. Captures remain ignored artifacts under screenshots/workspaces. No independent novice study, assistive-technology acceptance or Windows/macOS desktop acceptance is claimed.
- Full remaining networking/Linux/advanced-shell curriculum and fine-grained objective crosswalk remain open. Continue the completion checklist; no release or installer publication.

M10 evidence wording follow-up: the shorter `files-layout` native rerun now prints a layout-only PASS message instead of repeating the full six-lab traversal description. The original full `files` run evidence remains separate. No learning behavior changed.

## 2026-10-22 — M11: IPv6 addressing and static forwarding capability

- Specified and implemented a bounded typed device/interface/link/static-route model with /0–/128 prefix arithmetic, explicit manual on-link declarations, interface scope, longest-prefix-before-distance selection, floating routes, neighbor ownership, transit forwarding and forward/reply-path verification. No sockets, processes, dependency or learner command grammar was added.
- Invalid graphs fail validation; down links, duplicate same-link addresses, loops, absent routes and reply failures stay explicit. Equal-cost forwarding, recursive next-hop resolution and ND/DAD/RA/SLAAC/source-policy behavior remain disclosed exclusions. Typed route-status inspection explains inactive routes. IANA/RFC9637 checks include both current documentation ranges; no IPv4 broadcast rule is applied to IPv6.
- Independent code review found no material defect within the declared model and requested clearer inactive next-hop diagnostics, which were added. Nine targeted Rust tests cover every prefix, octet boundaries, equivalent text, malformed input, address categories, link-local sending/receiving scope, alternate valid gateways, primary/backup behavior, invalid graphs, duplicates, loops, return paths and current-state invalidation. Full regression: 130 Rust tests, Clippy/formatter;774 math comparisons against Python ipaddress have no differences.
- Capability only: no new IPv6 lesson, new exam coverage or native UI acceptance is claimed. Existing30 assignments, saved progress and accepted workspaces remain unchanged. Next author native IPv6 addressing/configuration/troubleshooting practice with isolated examples using this tested model.

### M11 installation candidate identity

Clean Fedora candidate source `71734c65df33aa2dda88596258a90f2d7c4bce0b`: `dlms-learning-lab-0.1.0-g71734c65df33-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz`, SHA-256 `8067a2191cfa5d1b6a64e44a7686e8dc0c32f1acf5bdcaa2413ac68ace99318e`. Installed under the isolated /tmp acceptance root, validated30 labs/7 units outside the checkout and passed desktop-file-validate plus actual native light150/focus layout. This is a milestone candidate while curriculum completion stays open. Native CI run `39800447563` dispatched for this source; results pending. No release published.

## 2026-10-22 — M12: resumable IPv6 configuration and teaching integration

- Added bounded atomic native configuration drafts, typed address/preservation/forwarding/connectivity outcomes and deterministic failover conditions. Pack adapters remain explicitly internal rather than invented router commands. Old scenarios/progress fields remain unchanged; incomplete new drafts are retained and cannot preserve stale success.
- Added isolated persisted example topology/drafts and declared example outcomes. Load-example fills only practice drafts; applying/checking examples cannot touch assignments or credit. Native rendering and six authored activities follow separately; current catalog remains30 assignments/seven units.
- Independent review requested immutable import shape/link validation and an unconditional positive base-state probe alongside failover conditions. Both implemented and tested; flexible-address tasks must keep probe targets consistent with advertised alternatives.
- Validation:138 Rust tests, Clippy and formatter. Eight new integration tests exercise atomic malformed/unsupported edits, alternate addresses and global/link-local gateways, each conditional probe without mutation, partial/current-state invalidation, old/new progress roundtrips, retained incomplete drafts, forged graph/draft import rejection, wrong-shape save retention and isolated example fill/apply/retry. No new dependency, host learner execution or native IPv6 usability acceptance is claimed.

M11 CI follow-up: run39800447563 passed on all three native runners. Ubuntu/Windows candidate hashes were verified after download and recorded in INSTALLATION.md; macOS remains packaging-gated. This evidence describes source71734c6 (130 tests), separate from later M12 integration tests.

## 2026-10-22 — M13: reviewed IPv6 static-network unit

- Six distinct native assignments progress from a unique workstation address through prefix/gateway repair, interface-scoped next hops, longest-prefix diagnosis, multi-case primary/backup preference and a branch handover capstone. Catalog36 assignments/eight units. Only narrow current CCNA1.8/1.9/3.3 slices move to Partial; dynamic protocols, full address-type coverage and multihop IPv4 stay open.
- Normal Learn workspaces retain Introduction → Try example → Assignment with independent saved example topology/drafts and different example values. Challenge retains full configuration work. Wide native screens split compact topology and selected-device editor; narrow/large-text views stack and scroll. One Check applies a validated candidate atomically; malformed edits retain old state and invalidate current success. Optional help/path inspection remains collapsed.
- Independent review found hidden manual on-link prerequisites and example selection loss; default new manual assignments to on-link and preserve selected device/view on example fill. Native checks exposed inherited horizontal card flow and an off-screen Check action; explicitly top-down cards and responsive split fix the normal desktop regression. Route tasks initially open the relevant editor. No new dependency or learner host execution.
- Validation:147 Rust tests, Clippy/formatter, bundled36/eight validation, all28 palette inventory/notice checks,14 Python packaging/oracle guard tests and774 IPv6 prefix comparisons. Every authored reference, distinct alternate, plausible wrong/partial result and actual suggested example change is tested. Old/new attempts and incomplete drafts roundtrip; first-hop checks observe actual route preference without forcing it.
- Actual Fedora synthetic-profile flow rendered all six introductions/examples/assignments, incorrect retry, reference-draft Check, completion, invalidation and resume. First address was entered via native keyboard events; later reference settings were staged in drafts, then applied through Check. Reviewed dark1920×1080 and resized light150%946×929 screens. A focused follow-up scrolled Check fully into view and clicked it to verify readable retry feedback. Independent reviewer inspected both themes and reran content/UI tests; no remaining material finding.
- These are agent input/render checks, not independent novice, screen-reader, physical-network or Windows/macOS desktop acceptance. Screenshots remain ignored local artifacts. See IPV6_LESSONS.md for reproduction and fidelity limits. Full completion checklist/curriculum/crosswalk/platform acceptance remain open; continue independent work. No release publication.

### M13 CI select-all fixture correction

First native CI run39807948400: Ubuntu passed; macOS correctly used Control-A as beginning-of-line rather than Select all, exposing a synthetic input fixture that had omitted egui’s logical Command modifier. The exact retained-draft assertion caught the unintended concatenation. Corrected all three Control-A-only select-all fixtures to include logical Command (existing cross-platform fixtures already do this); Control-Enter Run remains unchanged. This changes test input, not app keyboard semantics. Native rerun and candidate identities follow separately.

## 2026-10-22 — M14: granular objective evidence and honest gaps

- Reinspected current official source documents and indexed136 parent objectives,58 Cisco lettered children,375 LPI knowledge areas and671 named-tool/term bullets.1240 references,84 explicitly scoped partial-practice rows,1156 Open; no full-objective completion or additional curriculum is claimed. Upcoming Cisco2.0 remains separate. Exact source URLs/versions/check dates and semantic fingerprints are bundled as documentation, without copying official prose.
- Evidence is pinned per source item rather than inherited. Corrected the seed-era203.2 mapping: capacity/mount repair does not teach filesystem-specific/SMART maintenance. Added actual narrow route components/selection, function, service-journal and zone-reload slices. Independent review checked the partial claims and reproduced the inventory; no false full-coverage finding.
- Review found LPI descriptions absent from source-change detection. Included full description/weight in parent fingerprints and added description-only/weight regressions. Coverage rows pin fingerprints so regenerating the inventory cannot silently reuse evidence against changed scope.
- Seven new Python tests verify extraction, removed markers, wrapped PDF lines, missing/duplicate references, description changes, explicit gaps and safe evidence paths. Offline validation covers all1240 references and36 existing authored IDs. These checks enter native CI; they validate bookkeeping, not beginner comprehension or complete course acceptance. No native behavior, dependency, progress format or scenario changed. Full completion checklist remains open; continue capabilities/content.
