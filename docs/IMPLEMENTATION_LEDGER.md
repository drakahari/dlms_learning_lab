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


### M13–M14 clean candidates and native CI follow-up

Native CI run39810533865 at c6d8afe61585 passed all three native runners (147 Rust tests plus bookkeeping/tooling checks). Ubuntu and Windows candidate checksums were downloaded and verified; identities are in INSTALLATION.md. macOS compile/test remains separate from its unresolved original dispatch notice and actual desktop acceptance. The clean Fedora82fbf868f5b3 candidate installed/upgraded the isolated /tmp test installation, validated36 labs/eight units outside the checkout, passed desktop-file-validate and native IPv6/light150 layout; the exact synthetic progress sentinel stayed intact. No personal profile or public release.

## 2026-10-22 — M15: bounded Bash scope and evaluator history

- Specified/implemented Bash-specific function-local variables using the existing cooperative VM. Dynamic caller visibility, shadow restoration, early return and initially unset declarations match the disclosed default-Bash subset. All active names, saved values and inputs count toward bounds; no dependency or host execution added. Unsupported declaration contexts/options and unmodeled special variables fail honestly. Default IFS reads are modeled; changing splitting rules is explicitly unsupported.
- Added evaluator revisions so older saved runs cannot masquerade as current success after semantics change. Original source/output/credit remain separately labeled history; a new Run verifies current behavior. Draft edits, unsupported reruns and Stop invalidate current success without deleting earlier work. Exact original progress bytes are retained before the first upgraded write; occupied backup slots and Reset as the first action are checked, not overwritten. App backup names are reserved from exports.
- Independent review found an export-slot alias and a Reset-first backup bypass; both fixed with regressions. Final targeted20 tests independently pass with no remaining material finding. Full validation:161 Rust tests, Clippy/formatter,159 controlled Bash5.3.9 comparisons and79 GNU comparisons with zero differences,21 Python checks and offline1240-reference coverage verification. No new assignment or coverage is counted: catalog remains36/eight units.
- Actual Fedora synthetic-profile flow reviewed old-credit library, retained output, unsupported retry, scoped-loop Stop, native keyboard repair/current Run, saved reload and resized light150 scrolling. Final screenshots reflect neutral simulator labeling and one diagnostic for repeated identical failures. Earlier screenshots were independently inspected. This is rendered agent input testing, not novice/assistive-technology or Windows/macOS desktop acceptance.
- Next author distinct scope/debugging/safe-automation practice using this tested subset. Full certification/advanced-shell curriculum and platform acceptance remain open. No installer or release publication.


## 2026-10-22 — M16: reviewed Bash scope/debugging unit

- Three distinct assignments retain meaningful scripts: repair caller-title corruption, trace nested literal-filter settings without leaking them into a summary, and protect input files from a preflight helper that changes the caller's destination. Catalog39 assignments/nine units, including18 scripting tasks. No numeric variants, new dependency or learner host execution.
- Each lesson teaches prerequisites/new behavior before independent assessment, with separate editable examples and tested suggested changes. Bash-specific local is explicit; legitimate argument-passing/renaming repairs pass without source-pattern grading. Challenge keeps the full task and optional assistance.
- Independent review found untested repeated-source behavior and overbroad input/alias wording. Added repeated quiet sections and a dedup negative; clearly restricted supplied patterns/filenames and explained that textual comparison does not provide general real-file alias protection. Final13 targeted tests and five actual native captures independently reviewed with no material finding.
- Validation:165 Rust tests, Clippy/formatter,196 controlled Bash5.3.9 comparisons with zero differences,21 Python checks, all28 palette provenance checks and offline1240-reference bookkeeping. Every new reference/alternate/example, plausible wrong/partial/overfit outcome, early file mutation, saved draft isolation/selected reset and current-success invalidation is covered. Full certification coverage remains unchanged/partial; no evidence is inherited from new lab counts.
- Actual native Fedora synthetic flow traversed all three introductions, example Runs and exact suggested edits, assignments, wrong feedback, keyboard reference repair/Run, completion, saved reload and invalidation. Dark full-size and resized light/dark150% screens reviewed, including scrolling to actions. Final rerun passed after case/copy fixes; the subsequent filename-hyphen clarification changes copy only. No novice-user study, assistive-technology or Windows/macOS desktop acceptance claimed.
- Continue expansion/curriculum capability work. Full completion/platform/license-notice gates remain open; no release publication.


## 2026-10-22 — M17: bounded text tools and pipeline input ownership

- Specified/implemented pure typed sort/uniq/cut utilities inside the existing VM: C/POSIX byte ordering, reverse/unique sorting, adjacent-group/count filtering, ordered overlapping field ranges, finite stdin or one virtual source. Limits8192 lines/128 ranges/1M field indices/32KiB output; ranges are merged/scanned without numeric-sized allocation. Unsupported options/locales and malformed lists fail honestly. No dependency, host lookup or learner subprocess added; catalog remains39/nine.
- Independent review found existing pipeline input replay: restoring its original cursor let later commands reread consumed stdin. Corrected first-stage consumption while preserving input redirection, explicit-file behavior, caller cursor offsets and variable/argument isolation. Six disposable Bash comparisons cover this ownership. Actual prior evaluator f16704a was rebuilt from its public source in /tmp solely as a development probe: all6 EOF cases accepted a replay-dependent draft; current semantics reject the5 nonempty cases. No learner data or script was passed to a host shell.
- Advanced evaluator revision2 and preserved successive bounded historical runs, including a newer earned result alongside an older incomplete history. Current success requires a current Run. Original source/output/files/credit stay available in optional history; exact pre-upgrade bytes go to non-overwriting revision-specific backups. Full history rejects unsafe replacement, with original retained. Tests cover prior false credit, multi-revision credit, exact backup/export/recheck and full-capacity rejection.
- Validation:176 Rust tests, Clippy/formatter,359 controlled Bash5.3.9/GNU utility comparisons with zero differences,21 Python checks,28 palette provenance checks and1240-reference bookkeeping. Independent24 targeted utility/migration/backup tests pass with no remaining material finding. Existing lesson reference methods and alternatives remain intact; no new authored coverage is counted yet.
- Actual Fedora synthetic native upgrade flow passed earlier credit/output display, current unsupported retry, Stop, keyboard repair/current Run, saved reload and resized light/dark150 scrolling. Expanded two-history source panel inspected. This verifies rendered/input behavior, not novice-user, assistive-technology or Windows/macOS desktop acceptance. Full curriculum and platform gates remain open; continue dependent content. No release publication.


## 2026-10-22 — M18: reviewed text-record/audit unit

- Four distinct meaningful assignments: asset/owner field extraction, stable unique manifest, grouped location counts and a constructed duplicate-identity audit. Catalog43 authored assignments/ten units, including22 scripting tasks. Native accepted workspaces and Introduction → Try example → Assignment stay intact; no new dependency or learner host execution.
- Introductions teach fields/delimiters versus CSV, fixed-C text order versus numeric/human-language order, adjacent groups/counts and repeated-only selection before assessment. Examples have different data and exact tested experiments. The audit starts with a small input/output scaffold so learners construct validation/report publication, rather than repairing one flag.
- Independent review caught hardcoded filenames passing and an extra-argument test failing for the wrong reason. Vary supplied names (including spaces), add an extra-argument case with a valid input and retained output, and reject omitted count guards/hardcoded paths. Clarified output-name wording, strengthened empty/edge-spaced fields and named GNU count formatting.14 targeted content/teaching tests plus7 coverage checks independently pass; all6 native views reviewed without material findings.
- Validation:181 Rust tests, Clippy/formatter,415 controlled Bash5.3.9/GNU comparisons with zero differences,21 Python checks,28 palette provenance checks and43-lab/ten-unit bundled validation. Per-task reference/alternate/plausible wrong/partial/overfit behavior, exact example experiments, early file mutation, output aliases, count validation, current-success invalidation and isolated draft/reset/resume paths are exercised.
- Actual Fedora synthetic flow traversed all4 introductions, example Runs/edits, complete assignments, wrong feedback, native keyboard repair/Run, completion, reload and invalidation. Reviewed dark full-size and resized light/dark150% scrolling, including final constructed-audit starter. This is agent rendered/input testing, not independent novice/assistive-technology or Windows/macOS desktop acceptance.
- Narrow LPIC-1 103.2 cut/sort/uniq rows gain explicit script-VM evidence:87 partial references,1153 Open across1240 overlapping references. No complete objective/exam preparation claim. Full advanced shell/certification curriculum and platform gates remain open; continue bounded capabilities and reviewed content. No release publication.


### M18 clean installation / native CI follow-up

Source90039eefdddb passed all three native CI runners (run39821972644). Downloaded Ubuntu/Windows archives match their sidecar hashes; identities are in INSTALLATION.md. The clean Fedora candidate installed/upgraded an isolated /tmp root, validated43 assignments/ten units outside the checkout, passed desktop-file-validate and retained the exact synthetic progress sentinel. The installed native binary passed all four text-lesson flows, including larger text/light/dark rendering. Retaining a sentinel is not old-profile acceptance. Windows/macOS desktop acceptance and the macOS notice/signing gates remain open.

## 2026-10-22 — M19: bounded integer arithmetic and expansion ordering

- Specified/implemented portable arithmetic expansion within the disclosed fixed-width subset: integer literals/variables, explicit parameter and nested arithmetic expansion, unary signs, parentheses, precedence and + - * / %. Arithmetic stays inside the cooperative VM; no dependency, host lookup or process added. Limits4096 expression bytes/256 tokens/24 nesting levels; unsupported operators, recursive variable expressions and quotes in arithmetic fail honestly. Division-by-zero has focused feedback. Catalog remains43 assignments/ten units; this is capability, not additional curriculum.
- Arithmetic probes exposed preexisting assignment/redirection ordering defects. Corrected sequential assignment-only expansion before redirects and ordinary/local argument expansion before redirects; pure local parsing does not bind until redirection succeeds. Independent review caught premature pipeline preflight expansion and multipart quoted-assignment classification; both fixed with reference regressions.
- Advanced evaluator revision3, retaining earlier sources/output/files/credit as history and exact original bytes in non-overwriting revision-specific backups. An actual prior revision2 probe from public source1dbe345 accepted a redirect-order-dependent report in all3 cases; current3 rejects all3. Tests retain that earlier credit separately and reject stale current success, including successive history revisions. No learner script was executed by host Bash.
- Validation:191 Rust tests, Clippy/formatter,449 checked-in controlled Bash5.3.9/GNU comparisons with no differences,28 palette provenance checks and1240-reference coverage validation. Independent27 targeted tests and extra arithmetic edge comparisons passed, with no remaining material finding. A local debug stress probe completed16 maximal-size repeated-value calculations in about94ms; this is one bounded-workload measurement, not a universal timing guarantee.
- Actual synthetic-profile native Fedora checks passed current calculation/Run, zero-divisor feedback and success invalidation, Stop on an arithmetic loop, repair/current Run, save/reload and resized light150% scrolling. Four arithmetic captures independently reviewed. This capability smoke uses an existing assignment and does not claim new arithmetic teaching, novice-user/assistive-technology or Windows/macOS desktop acceptance. Full curriculum/checklist remains open; continue dependent teaching and projects. No release publication.


## 2026-10-22 — M20: sequential file loops and reviewed counters/totals unit

- Four distinct authored assignments: number handover records, accumulate whole-cent order totals, guard integer division/remainder, and publish a validated multi-file count report. Catalog47 assignments/eleven units, including26 scripting tasks in six units. No numerical-variant inflation, dependency or host learner execution. Numeric inputs are deliberately supplied small valid integers; general lexical validation, floats and full advanced-shell coverage remain open.
- Initial runnable checks caught an unimplemented trailing while-input form before content shipped. Specified/implemented only one < redirect on while: open once, sequential finite input, preserved variables and restored parent cursor on normal exit/early return. Typed input frames count toward memory/call/syntax limits. Other compound/multiple/output redirects remain unsupported. Active-source writes fail before truncation/append until live changing-file behavior is modeled. This additive form does not change old supported loop instruction counts or evaluator revision3.
- Review strengthened bundled inputs with spaced source filenames,64 quantities999/rate99, and an eight-source case containing empty/16-record/unterminated inputs. Independent review found return-with-own-input restoration failing without a popped loop scope; restoration now checks actual input-stack depth, with no-loop/unredirected-loop/caller-loop regressions and four Bash comparisons. Independent20 targeted tests and additional disposable edge comparisons pass; six representative native captures were reviewed with no remaining material finding.
- Validation:203 Rust tests, Clippy/formatter,524 checked-in controlled Bash5.3.9/GNU comparisons with no differences,21 Python tooling/boundary checks, all28 palette provenance checks,47/eleven bundled validation and unchanged1240-reference objective-source fingerprints. Existing87 scoped partial evidence rows remain unchanged; no complete exam/objective claim. References/alternates/misconceptions/partial/overfit paths, exact teaching experiments, preflight/no-new-file behavior, maximum batch, zero-divisor feedback, bounded loops/memory, state invalidation and isolated draft/reset/resume paths are tested.
- Actual final native Fedora synthetic flow passed all4 introductions, independent example Runs/edits, complete assignments, wrong feedback, keyboard repair/current Run, saved reload and invalidation. Reviewed dark full-size and resized light/dark150% scrolling; shortened first introduction retains its primary actions on the full desktop. Separate native infinite-file-loop Stop, keyboard repair/all-eight-case Run and saved reload passed. Captures remain ignored screenshots/workspaces/shell-number-records-*, shell-order-total-*, shell-pack-items-*, shell-count-capstone-* and shell-counts-*. These are agent render/input checks, not independent novice/assistive-technology or Windows/macOS desktop acceptance. Full checklist remains open; continue capabilities and curriculum. No release publication.


### M20 candidate / CI follow-up

Clean Fedora94b0dbe candidate (47/eleven) installed/upgraded the isolated /tmp root, validated bundled content outside the checkout, passed desktop-file-validate and retained the exact synthetic sentinel. Installed native all4 counter lessons passed including larger text/light/dark; checksums are in INSTALLATION.md. No personal progress was used.

Initial CI39828321927 passed Ubuntu/macOS; Windows failed the corrupt-profile restore UI assertion without a cause message. Added a test-only diagnostic and settled measured async-window geometry before synthetic clicks, preserving restoration/safety-copy assertions. Fedora targeted test passes. This addresses a suspected stale first-frame pointer target; Windows causality/status remains pending rerun. No production recovery behavior or curriculum change is made by this fixture follow-up.


## 2026-10-22 — M21: ordinary-user access capability

- Specified/implemented numeric and explicit-class symbolic chmod, owner/member-restricted chgrp and informational id with shared deterministic synthetic IDs. New typed goals distinguish file reading/writing, listing directory names, directory search and creating entries. Owner precedence and every parent search permission apply; missing users/targets or wrong target kinds cannot satisfy denied-access goals. No root override, special bits/ACL/SELinux/links, account modification, dependency or host learner execution.
- Commands remain bounded cloned-state transactions; errors cannot partially change permissions/group/data. Existing Tree/progress fields, IDs, versions and evaluator revision3 remain unchanged. No new lesson/coverage is counted:47 assignments/eleven units. Excluded chmod forms are diagnosed rather than approximated, including omitted classes/copy/X/special bits/recursion.
- Validation:209 Rust tests, Clippy/formatter,152 controlled GNU filesystem comparisons with no differences and strengthened development-oracle guard tests. Exhaustive512 ordinary starting modes with eight symbolic transformations, numeric modes and three user-class access matrices are checked. Chgrp/id synthetic identities/membership are model/source reviewed; the GNU oracle comparisons cover chmod and the existing filesystem commands, not host account equivalence. Existing script/teaching/progress regressions pass.
- Independent17 targeted tests,152 GNU comparisons and two guard tests passed. Review requested a wrong-target-kind explanation; fixed and tested alongside missing-target/user messages. No remaining material implementation/isolation finding. This is backend capability, not new curriculum or native usability/platform acceptance; dependent meaningful terminal teaching follows. Full completion checklist remains open. No release publication.


### M20 restore fixture / native CI follow-up

Run39829793024 at bd14db0 passed all three native runners, including the Windows corrupt-profile restore/safety-copy assertion. Measured-window geometry is settled before synthetic clicks; failure diagnostics remain available for future investigation. This one rerun supports fixture stability, not proof of every possible timing condition or desktop acceptance. Ubuntu/Windows archives were downloaded and hashes verified; identities are in INSTALLATION.md. These results precede M21/M22. No release publication.

## 2026-10-22 — M22: reviewed Linux access-policy unit

- Four distinct terminal assignments: group-specific read/write sharing; creating entries without name browsing; updating another owner's existing writable files inside a550 directory; and a metadata-preserving read-only handover with temporary parent permission/revocation. Catalog51 assignments/twelve units. New unit follows file foundations and precedes the original access seeds; old IDs/states/attempts remain intact and prerequisites remain advisory.
- Added known-owner/group retention outcomes before dependent content. Copy-delete cannot fake an in-place update or another owner's moved file; missing/unknown metadata fails explicitly. No chown/root/account-modification command, dependency or host learner execution added. Every lesson teaches before assessment and retains separate persisted runnable examples. Intentional experiment failures preserve practice state; Challenge retains independent full assignments.
- Validation:216 Rust tests, Clippy/formatter,21 Python tooling/guard checks, all28 palettes and51/twelve bundled validation. Reference/alternate/policy mistakes, group-only/mode-only, missing-search/overbroad listing, wrong/one source update, recreation that fails only ownership, final-writable staging, retained text/mode/identity damage, every individual valid-state objective invalidation, exact experiments and selected reset/resume/draft isolation pass. Existing152 GNU filesystem comparisons support the underlying permission capability; this is not an actual multiuser Linux acceptance test.
- Independent18 targeted tests and granular coverage validation pass; all7 representative native captures reviewed with no remaining material finding. Actual Fedora synthetic flow traversed all4 introductions, keyboard first-example commands, declared isolated example outcomes, partial-policy retry feedback, normal command repairs/Check, current success invalidation, saved reload and resized light/dark150% scrolling. The incomplete rename is repaired by restoring its final parent permission, without resetting/inventing its state. A separate native library pass reviewed all four new cards plus certification/unit/topic filters at full dark desktop and narrow light150%; aligned controls wrap and scroll cleanly. The new long unit label also passes headless width/scale checks. No novice-user study, screen-reader, real multiuser or Windows/macOS desktop acceptance claimed.
- Added one narrowly scoped chgrp tool row and strengthened existing104.5 ordinary-file/directory/group evidence:88 Partial practice/1152 Open across1240 overlapping current references. Special files/bits/privileged ownership/ACL/SELinux and full certification coverage stay open. Full application completion checklist remains open; continue capability/content work. Captures stay ignored local artifacts; no release publication.


### M22 candidate / CI follow-up

Source19aad147796b passed all three native CI runners in run39833026248. Downloaded Ubuntu/Windows sidecars match locally computed hashes. Clean Fedora installation/upgrade, bundled51/twelve validation outside checkout, desktop entry validation and all-four installed native access flows passed; exact synthetic sentinel was retained. INSTALLATION.md records identities/checksums and distinguishes retained-file checks from legacy-profile acceptance. macOS archive remains notice-gated; actual Windows/macOS desktop acceptance is open.

## 2026-10-22 — M23: bounded VLAN broadcast data plane

- Specified then implemented pure typed switches/ports, active VLAN databases, access/native/allowed trunk configuration, workstations and physical cables. Broadcast propagation performs per-link tagging and independent ingress classification; no global VLAN equality shortcut. Both directions matter: one-way visibility is neither mutual service nor isolation. Known operational drops differ from explicit unsupported tagged-access behavior. No dependency, host network/process/I/O, invented IOS CLI or authored coverage added; catalog remains51/twelve.
- Topology decoding/validation rejects duplicates, invalid IDs/VLANs, unknown/reused jacks, loops/redundant physical paths and oversized bounded collections. Work/trace/drop limits are explicit. Pure live-path guard independently checks operational endpoints/cables/port ends before future isolation grading. Integration must retain authored attachments, bound document bytes before decoding and include positive service cases; shutdown/unplugging cannot earn policy credit.
- Validation:227 Rust tests, Clippy/formatter. Eleven new tests cover every VLAN1–4094, a maximum8-switch/32-host forest, complete bounded traces, wrong access/active/allowed/native cases, current-state invalidation, down/unplugged links, malformed/duplicate/oversized declarations and9216 independent one-link forward/reverse transition cases. Independent capability review finds no material forwarding/forest/classification/bound defect; integration safeguards are explicit in SWITCHING_SUBSET.md. Final additions reviewed separately before commit.
- Cisco references pin the educational tagged-native policy and explain omitted Catalyst PVST/PVID blocking. This is data-plane broadcast visibility, not ping, MAC learning, STP, real equipment acceptance or full802.1Q conformance. No UI/content acceptance is claimed by this backend milestone. Full checklist remains open; native integration and substantive switching teaching follow. No release publication.


## 2026-10-22 — M24: native switching drafts and isolated teaching workspace

- Integrated typed switching state/goals, atomic native port/VLAN drafts, optional applied-state broadcast inspection and the accepted Introduction → Try example → Assignment pattern. Desktop port rows align connection/mode/VLAN controls; narrow/large-text layouts stack and scroll. Challenge retains the full independent assignment. No invented IOS terminal, dependency, learner host activity or added authored coverage; catalog remains51/twelve. The development fixture is explicitly separate from bundled content.
- Isolation outcomes require live paths, active access attachments and both-direction non-delivery; each negative endpoint must also participate in an authored positive service case. Up access/native and active-VLAN constraints express stated policy. Fixed equipment/attachments are checked against authored state on import. Existing2MB pack/16MB profile caps remain before deserialization. Unsupported tagged-access behavior cannot earn policy credit.
- Optional version1 draft/practice fields retain old attempts and malformed bounded edits without inventing credit. Different unapplied settings clear current success; example settings/results remain separate, reset is selected-only, and loading never rewrites the original. Assignment/example actions are internal reachability adapters, not shell or Cisco commands.
- Validation:232 Rust tests, Clippy/formatter; four new workspace tests cover atomic malformed edits, invalid packs/tampered shapes, partial/alternate valid configurations, shutdown/unplug/missing-VLAN and one-way negatives, preserved drafts/example credit isolation, reset/resume and byte caps. Headless UI checks include desktop, transition widths,720px and150% text with focus/navigation. Independent15 model/workspace tests plus the native-egui check passed; no material persistence/grading issue remained. Final rendered review found no material layout defect and requested matching practice action wording; captions now say Apply and test example in practice.
- Actual Fedora synthetic native flow passed introduction, isolated loaded example/changed example, incorrect assignment, keyboard field repair/Check/completion, save/reload, edit invalidation and selected reset. Reviewed compact desktop dark, optional broadcast inspection and resized light/dark150% captures. A scroll fixture initially left its pointer outside the resized window; corrected the pointer target before wheel input and recaptured. These are rendered/input checks, not novice-user, screen-reader, equipment or Windows/macOS desktop acceptance. Screens stay ignored as switching-fixture-*. Full completion stays open; reviewed substantive switching content follows. No release publication.


## 2026-10-22 — M25: reviewed VLAN workgroups/trunks/handover unit

- Four distinct assignments: local access membership; establishing an inter-switch trunk plus separate active/allowed faults; explicitly caveated native classification; and a three-group handover combining five access/database/list/native/up-state repairs. Catalog55 authored assignments/thirteen units. The first lesson requires no subnet/binary/terminal knowledge; later prerequisites are advisory. Existing IDs/versions/progress,28 themes and native simulation-only boundaries stay intact; no dependency or learner host execution added.
- Each introduction teaches before assessment and offers a different saved practice office plus a specific experiment. Challenge retains the independent assignment. All workgroup service and cross-group separation cases are graded, with explicit access/native policy. Equivalent field/list ordering, leading-zero editor input and extra unused groups are accepted when policy holds. Every omitted repair remains incomplete; stopping/removing endpoints cannot earn separation.
- Actual rendered feedback exposed repeated multi-objective walls. Default now shows at most the first two failures, with full objective results optional; practice gets a concise summary. Diagnostics name the actual down port/missing active VLAN and explain unsupported Access/Trunk tagged input plainly. Independent review found a resumed-draft false-headline edge after this refactor; completion now respects current_solved and pending drafts. A serialize/decode/rendered-Shape-text regression proves accepted saved solved-state/divergent-draft data cannot display Objective complete. No applied-state checks were weakened.
- Validation:239 Rust tests, Clippy/formatter,21 Python tooling/boundary checks, all28 theme provenance checks,55/thirteen bundled validation and unchanged1240-reference issuer source fingerprints. Five new content tests exercise taught methods/alternatives/misconceptions/partial work, all endpoint-pair service/separation, each individual broken outcome, exact example experiments and separate draft/reset/resume credit. New headless UI layouts cover all4 intros/examples/assignments,1080/1200 widths, narrow150% and independent Challenge tools. Corrected legacy test assumptions about single-page CCNA and multi-step lessons while retaining the original-nine progress fixture’s exact assertions.
- Independent20 model/workspace/content tests, coverage validation, final UI/rendered-source review and the resumed-headline regression pass with no unresolved material finding. Review corrected capstone wording to configure the stated memberships rather than preserve an initial fault. Testing corrected a native-example experiment expectation: changing only the repaired east native causes one-way visitor service failure, not cross-group leakage in that fixture.
- Actual Fedora synthetic native flows passed all4 introductions, isolated example fills/experiments, wrong and partial checks, keyboard final-field repairs, Check/completion, save/reload, changed-draft invalidation, specific malformed-VLAN feedback and selected reset. Reviewed dark desktop, light/dark150% resized/scrolled views and unit-library navigation. A separate actual saved-profile resumed-draft flow through existing XWayland confirms neutral unverified feedback and keyboard repair/current completion. Extra default-Wayland resume attempts stalled before the harness’s first timed action; the app-specific protocol trace ends with a committed frame awaiting a callback after keyboard focus leaves. Session was active/unlocked. Occlusion/frame-callback starvation is a hypothesis, not an established application root cause. Those attempts are not counted as passed; investigate the startup smoke harness next. No session setting or security control was changed. Practice setup/partial fixtures use pure internal adapters; final repairs and Checks use injected native keyboard/pointer events. These are agent renderer/input checks, not independent novice, screen-reader, real-switch or Windows/macOS desktop acceptance. Captures stay ignored as ccna-vlan-*, vlan-capstone-*, vlan-unit-* and vlan-resumed-*.
- Added six narrowly partial CCNA v1.1 rows:2.1/2.1.a data access and2.2/2.2.a/b/c explicit trunks/tag/native. Granular ledger94 Partial practice/1146 Open across1240 overlapping references. Voice/default-VLAN-specific teaching, inter-VLAN routing, MAC learning, STP/PVST/PVID protection and real IOS/hardware behavior remain open. Future v2.0 stays separate. Full completion checklist remains open; continue required capabilities/content. No release publication.


### M25 candidates / CI follow-up

Run39841022031 at85d0274 passed all three native runners (239 Rust tests,55/thirteen). Downloaded Ubuntu/Windows sidecars match local hashes; INSTALLATION.md records identities and their pre-M26 evaluator3 scope. Clean Fedora archive installed/upgraded the isolated /tmp root; outside-checkout bundled validation, desktop entry validation and exact synthetic sentinel retention pass. Installed all4 VLAN native flows passed through existing XWayland with no personal profile. macOS packaging/actual Windows/macOS acceptance remain gated. A pre-map fullscreen smoke-only experiment did not fix the extra Wayland startup stall and was removed; it is not a product change or an accepted fix. Continue independent capabilities/curriculum; no release publication.


## 2026-10-22 — M26: bounded shared shell expansion

- Audit found multipart/whitespace expansion could allocate beyond final argument limits before rejecting input. Added early field/count/UTF-8 byte guards shared by ordinary arguments, assignments, local declarations, redirects and while inputs, plus a cumulative1,000,000-byte work budget across all VM ticks, functions, pipelines and arithmetic lookups. No new syntax, dependency, host access, assignment or coverage:55/thirteen and94 Partial/1146 Open remain unchanged.
- Evaluator4 preserves actual earlier outcomes/credit as history, with exact non-overwriting revision-specific backups. Three controlled resource-heavy fixtures were genuinely accepted by retained revision3 and Bash; current failure does not erase them or invent current mastery. Export/review/reset retain history correctly. Large supplied-value previews are truncated before formatting; full details remain optional and scrollable.
- Validation:244 Rust tests, formatter/Clippy,21 Python tooling/boundary checks, all28 palettes and bundled55/thirteen. Independent37 focused tests,524 disposable Bash comparisons/zero differences, prior/current synthetic allocation measurement and final9-test feedback/expansion rerun pass with no unresolved material finding. Five new tests cover direct consumers, multipart/empty/UTF-8/quoted arguments, aggregate bounds, shared work/tick determinism, Stop, exact prior credit/backup, export, repair and reset. See SHELL_EXPANSION_LIMITS.md for the preimplementation contract and observed memory measurements rather than a universal RSS claim.
- Actual native Fedora XWayland synthetic flow passed current resource feedback, prior history, exact backup, keyboard loop/Stop, repair/all3cases, save/reload, edit invalidation and retained credit. Desktop dark and resized light/dark150% reviewed. Initial Stop test failed because Run enables it after action rendering; the harness now settles the enabled previous-frame target before clicking. No production Stop behavior change. Default-Wayland startup investigation, novice/screen-reader and Windows/macOS desktop acceptance remain open. No new curriculum or release publication. Continue independent required capability/content work.


### M26 candidates / CI follow-up

Native CI39845121080 at44c1f40 passes all3runners with244Rust tests. Ubuntu/Windows candidates downloaded and hashes verified; installation identities are in INSTALLATION.md. Clean Fedora candidate installed/upgraded and validated outside checkout; exact synthetic sentinel retained and desktop entry valid. Installed expansion native flow passes Stop/current repair/history/reload/invalidation/light-dark150%. PredatesM27 evaluator5; not a final completed-curriculum candidate. Default-Wayland and actual Windows/macOS desktop acceptance plus macOS notices remain open.


## 2026-10-22 — M27: bounded virtual filename matching

- Defined then implemented unquoted * and ? over flat virtual filenames, after splitting with retained per-byte quote provenance. Sorted C-byte matches preserve relative/explicit /work form; hidden names require an explicit leading dot; unmatched words remain unchanged. Assignments/local assignment forms suppress expansion. Redirect ambiguity fails before mutation; for/pipeline/function argument contexts use shared behavior. No dependency, host glob/I/O/process, authored assignment or coverage added:55/thirteen,94 Partial/1146 Open.
- Iterative matching and copy bounds share the VM cumulative work budget. Bracket/recursive/directory/non-C and unquoted backslash-bearing patterns fail explicitly. Evaluator5 corrects variable-supplied bracket expressions formerly treated literally. A clean prior4 probe establishes actual earlier results; migration retains source/output/files/credit as history plus exact original-byte backup/export/recheck/reset. Unsupported current behavior never inherits current success.
- Independent review caught the shared lexer's effect on the separate Linux terminal. Restored its explicit bare-wildcard rejection; quoted/escaped literal filenames still work. A full Tree regression proves failed touch/rm/ls/mixed-part patterns leave state unchanged. No terminal language expansion was silently added.
- Validation:253 Rust tests, formatter/Clippy,21 Python checks,554 disposable Bash comparisons/zero differences, all28 palettes and unchanged bundled/coverage validation. Eight new tests cover semantics, independent dynamic-programming short-pattern agreement, resource/tick/Stop behavior and actual earlier-credit migration; a ninth protects the literal terminal. Independent25 targeted tests and554 reference comparisons pass with no unresolved material finding.
- Actual native Fedora XWayland script regression passed editor/Run/Stop, focused resource feedback/history, exact backup, repair/current acceptance, save/reload/edit invalidation and light/dark150% scrolling. This is unchanged workspace integration, not new authored glob teaching or novice acceptance; dependent practical batch content follows. Default-Wayland investigation, accessibility and actual Windows/macOS desktop acceptance remain open. No release publication.


### M27 candidate / CI follow-up

Sourcea3d7a9d212663f91a134b231525971fb0cfeda80 passed all3 native CI runners in39846874265 (253Rust tests). Ubuntu/Windows artifacts downloaded and checksum-verified under ignored dist/native-ci-a3d7a9d. Fedora clean-source candidate prepared; INSTALLATION.md records all identities. No additional M27 Fedora installed-runtime claim is made; M26 installed native stress acceptance and M28 normal development-native flows are separate evidence. macOS archive remains notice-gated, and actual Windows/macOS desktop acceptance remains open.

## 2026-10-22 — M28: reviewed file-selection and safe batch unit

- Three distinct assignments: changing-file manifest, ordered duplicate-preserving cross-file incident harvesting, and a captured-source/collision-safe handover with explicit input rejection and file counts. Catalog58/fourteen;29 scripting assignments/seven units. No new syntax, dependency, schema field, host execution or altered existing IDs/versions. Native editor/optional help and separate saved example drafts remain the accepted workspace.
- Introductions teach selection, literal filename quoting, hidden/no-match behavior, filter status versus batch status, capture-before-output and preflight-before-mutation. Different .note/TODO/list examples and explicit experiments stay ungraded. Challenge retains complete independent work. Source arguments, output and full resulting/preserved file maps are graded across22 deterministic cases; legitimate alternate control flow/function interfaces pass, and first-case hardcoding fails.
- Review caught a missing empty-destination graded case. It replaced a redundant separate empty-source case; normal success now also contains an empty source. Removing the empty guard fails. Corrected self-referential prerequisite guidance, linked exact foundational file-test/return lessons and split the capstone task into three readable paragraphs. Existing total-count assumptions were updated without changing original-nine assertions. No remaining material content issue after independent14 targeted tests/601 Bash comparisons and rendered review.
- Validation:260 Rust tests, formatter/Clippy,21 Python tooling/boundary checks, all28 palettes,58/fourteen bundled validation and unchanged official objective fingerprints. Five new content tests cover exact example experiments, partial/wrong work/current invalidation, late collision/new .log report isolation, larger24-source behavior and original55-attempt/draft reset/resume isolation. Two native-egui tests check intro/example/assignment/Challenge widths at desktop/narrow150 and keyboard Ctrl+Enter plus Esc/Tab focus. All601 controlled disposable Bash comparisons agree.
- Actual native Fedora XWayland all-three flow passed introductions, isolated examples/experiments, wrong retries, keyboard repair/current Run, completion/save/reload, keyboard draft invalidation and selected reset. Reviewed full desktop and light/dark150% scrolled assignments/Challenge plus unit-library baseline/wrapping/cards. This is agent-render/input evidence, not novice, screen-reader or Windows/macOS desktop acceptance. Default-Wayland startup observation remains open. Screens stay ignored local artifacts.
- Added only two narrowly partial LPIC-1 103.3.K4/T19 rows: authored simple * selection. Granular ledger96 Partial/1144 Open across1240 overlapping references. ? capability/reference tests are not authored coverage; advanced/bracket/directory patterns and wildcard file-terminal operations remain absent. Full curriculum/completion checklist remains open. Continue independent required application/content work; no release publication.


### M28 installed candidates / CI follow-up

Run39849265414 atae484460b456bb43cbb180f6b326775fd6770a68 passed all3 native runners with260Rust tests/58 assignments/fourteen units. Ubuntu/Windows candidates downloaded and sidecars hash-verified; identities are in INSTALLATION.md. Fedora clean candidate installed/upgraded in the isolated /tmp root; outside-checkout bundled validation and desktop entry pass, exact synthetic sentinel retained (retained-file check). Installed all-three file-batch native flows passed editor/examples/experiments/wrong retry/current repair, save/reload/invalidation/reset, light-dark150% and unit library. Default-Wayland investigation, macOS notice gate and actual Windows/macOS desktop acceptance stay open; no release published.

## 2026-10-22 — M29: pure IPv4 static-forwarding capability

- Specified then implemented typed bounded devices/interfaces/links, conventional private/documentation endpoint assignments, connected routes and fully specified static routes. Prefix math0–32 includes host routes; interface1–30 and original subnet-lab /31-/32 exclusions remain explicit. Longest prefix precedes distance; connected0/static1–254,255 ineligible. Immediate on-link next hops only; no recursion, interface-only static routes, ECMP, ARP packets, real ICMP, transport, dynamic protocols, NAT/ACL or host network/process/I/O.
- Exact current adjacency, forwarding and both-direction paths diagnose absent neighbors, duplicates, network/broadcast faults and loops. Neighbor absence does not withdraw an eligible static route. Endpoint verification names the intended device/interface independently of current IP ownership; raw address probes cannot alone earn future endpoint credit. Weak-host ingress/source-policy limits and supplied operational state are documented. No state/progress/UI integration yet, no new assignment/coverage:58/fourteen and96 Partial/1144 Open remain unchanged.
- ExistingIPv6 owner audit: raw reachability could terminate on an intermediate relocated target, but all42 assignment and42 practice placement attacks fail current full goals because every other address slot is preserved and the flexible slot excludes its peer. No current authored-credit bypass was demonstrated. New IPv4 binding prevents relying on those incidental constraints alone; future goal adapters must still check required fixed/subnet addresses, same_shape and positive service cases.
- Validation:270 Rust tests, formatter/Clippy,21 Python tooling/boundary checks,264 independent Python prefix comparisons/zero differences, all28 palettes and unchanged bundled/coverage checks. Ten new tests cover prefix/octet boundaries, selection/default/host/distance, reply/forwarding, missing-neighbor versus withdrawal/backup, equal-cost exclusions, duplicate/moved endpoints, current breakage, strict duplicate/map/array/byte decoding, private/documentation scope and maximum12-device/96-interface/384-route topology with11-hop paths both ways.
- Independent review repeated10 routing tests and264 math comparisons. Fixed a connected-route diagnostic that incorrectly claimed static eligibility; specific connected/static messages now differ. Rejected cross-interface configured self next hops, up or down, with regression coverage. No remaining material finding in capability scope. Native/grading/content integration is not yet reviewed or accepted; it follows next. No new runtime/screenshots/platform claim, dependency or release publication.


### M29 candidate / native CI follow-up

Sourcea1245b4431786ddccb88dba7480c7d69303465d6 passed all3 native CI runners in [39852335952](https://github.com/drakahari/dlms_learning_lab/actions/runs/39852335952),270 Rust tests/58 assignments/fourteen units. Fedora clean-source candidate built and hash-verified; INSTALLATION.md records its identity. No additional installed-runtime acceptance is claimed for this pure capability step; M28 installed flows are separate evidence. macOS archive notice gate and actual Windows/macOS desktop acceptance remain open.

## 2026-10-22 — M30: native IPv4 routing integration, before authored content

- Added separate typed State/Goal, bounded declarative requirements and pack validation. Reachability names intended endpoint/interface, static-prefix tasks require that actual selected static network entry to serve the endpoint, and controlled failure cases clone current state. Required fixed/subnet assignments remain explicit. No raw IP ownership inference, host operations, new dependency or fake coverage.
- Atomic native address/route editors retain malformed bounded drafts independently of applied state. Divergent pending edits invalidate current success in workspace/library/resume; selected reset preserves other work. Optional omitted/defaulted version1 workspace fields isolate saved practice topology/draft/view/results from assignments, without changing older IDs/versions or fabricating credit. Import pins physical shape to authored assignment/example and rejects foreign metadata.
- Accepted native Introduction → Try example → Assignment renders through the normal state dispatcher. Full task/tools remain in Challenge. One Check action and compact first-two retry feedback; detailed results/paths/help optional. Review found one-interface forwarding could be expressed by adapters but not UI; fixed with optional router controls and a keyboard/pointer regression. Fixed valid empty optional walkthrough help indexing. Consolidated practice status to one summary after rendered review.
- Validation:281 Rust tests, formatter/Clippy,21 Python tooling/boundary checks, unchanged objective fingerprints/coverage and all28 palettes/notices. Eight integration and three native-egui tests cover alternate hosts/routes, partial/wrong/reserved/duplicate addresses, missing return paths, selected-prefix grading, endpoint movement, cloned failure cases, parser/adapter/shape/import rejection, atomic edits, practice isolation, prior58 attempts, malformed draft round trips/reset/resume, focus, scaling and one-interface router expressivity. Independent review repeated18 routing/workspace and two then-current UI tests plus adversarial probes, with no remaining material finding.
- Actual native Fedora XWayland development-fixture flow passes keyboard editor/checks, educational retries, completion/invalidation/save/reload, Tab focus/reset, route editing and resized150% light/dark scrolling/Challenge. Final renders reviewed including the consolidated practice summary. Fixture is explicitly development-only, absent from catalog/coverage.58/fourteen and96Partial/1144Open stay unchanged. No novice/physical-network/screen-reader/Windows/macOS desktop claim. Default-Wayland startup investigation remains open. Authored routing unit follows; no installer/release publication.


## 2026-10-22 — M31: reviewed IPv4 static-route unit

- Added six distinct authored assignments: network construction, reply-path repair, overlapping-prefix detour with working default preservation, deliberate host exception preserving a peer, equal-prefix floating backup and a two-client resilient branch handover. Catalog64/fifteen. Native meaningful tasks and skippable Introduction → Try example → Assignment; independent examples and concrete experiments, Challenge assistance on request. No number-variant/fixture inflation, dependency, shell semantics or changes to original IDs/versions/progress.
- Independent content review caught two real goal gaps: a broader backup default could replace an explicitly requested /24, and an existing reserve return route could mask the capstone’s broken primary. Added default-empty bounded conditions to StaticRouteFor with cloned-state evaluation, and normal R2 office-prefix/egress outcomes. Both attacks now fail for the intended reason while alternate distances/route order/free client addresses pass. Fixed source-address prerequisite teaching and a hint that overstated host-route limitations. Router-origin source selection and asymmetric-reply limits remain explicit.
- Actual size tests caught Check below the desktop on larger topologies. Kept selected-neighbor evidence prominent, whole topology optional, and aligned wide route rows with stacked narrow forms. Native menu focus waits for enabled popup contents/pointer release; pure keyboard and mouse opening both support Tab/Enter. Pending raw drafts suppress applied-only status on resume as well as current library completion. No redesign of the original IPv4 assignment.
- Validation:289 Rust tests, formatter/Clippy,21 Python tooling checks, all28 palettes/notices,64/fifteen content validation and unchanged official objective fingerprints. Six content/two UI additions cover exact example experiments, partial/wrong/alternate work, selected prefix under failure, primary return repair, endpoint movement, invalidation, shape/import bounds, exact prior58 attempts and reset/resume; layout matrix includes1200/150%, table-cutoff widths and720/150%. Existing full broken-attempt persistence regression now includes the new state family. Earlier601 Bash comparisons are unchanged, not rerun/claimed for this routing-only batch.
- Full native Fedora XWayland all-six flow passes intro/example/assignment/retry/reference-draft Check/completion, invalidation/save/reload/reset, actual manual keyboard route entry and menu selection, long stacked150% scroll-to-Check, stable light/dark Challenge and unit-library wrapping. Screens1920×1080 and946×929 reviewed; two harness assumptions (one fixed scroll amount and same-stage theme capture) were corrected before final acceptance. /tmp/dlms-m31-native-accepted.log records final PASS. Independent review confirmed final renders and no remaining material finding.
- Extended explicit existing Partial evidence for IPv4/route-prefix/next-hop/default/host/floating behavior;96Partial/1144Open across1240 references stay unchanged. No IOS/dynamic metrics/protocol/equipment conformance, novice/screen-reader or Windows/macOS desktop acceptance. Full completion checklist and default-Wayland startup investigation remain open. Next: strengthen first-time subnet/VLSM teaching in the approved complete workspaces, following the owner’s fundamentals question. No release publication.

## 2026-10-22 — M32: teach the existing subnet and VLSM foundations

- Kept both complete accepted assignments, IDs, versions, goals, alternate solutions and original progress. Added skippable plain-language introductions and separate editable examples for address/mask/prefix grouping, network/broadcast/usable hosts, a different workstation, then capacity/alignment/containment/non-overlap. Challenge retains independent full tasks. No new lab, dependency, shell behavior or objective-coverage inflation; catalog remains 64/fifteen and 96 Partial/1144 Open references.
- Bounded declarative teaching/examples and optional saved workspace metadata use current math/state grading. Original, example and numerical-practice drafts/checks/undo IDs are separate; examples cannot earn assignment credit. New attempts start Introduction, older attempts reopen Assignment with teaching available. Legacy absent workstation fields and credit retain existing honest migration behavior.
- Independent review exposed an existing import integrity gap: changing an original subnet or lowering VLSM host requirements could earn credit. Decode and current completion now pin original supplied inputs while preserving answer text and legacy field omission; fixture shape is pinned separately. Distinct examples depend on actual IP/prefix/pool/host-count inputs, not renamed/reordered labels. Team names are bounded. No known valid historical fixed-input changes were found; rejected imports retain original bytes.
- Validation: 299 Rust tests, formatter/Clippy with warnings denied, 21 Python tooling/boundary checks, all 28 palette notices, unchanged official objective fingerprints/coverage and 64/fifteen bundled validation. Seven backend/three native-egui additions cover prefixes 0–30 and octet boundaries, exact taught experiments, alternate and partial/wrong work, stale success, example/numerical/assignment isolation, foreign/malformed metadata, tampered originals, legacy omission/read bytes, separate undo history, selected reset/resume, both modes, focus and desktop/narrow 150% geometry. Prior nine initial states/goals/solutions/IDs/versions compare unchanged.
- Final native Fedora XWayland flow passes both fresh introductions/examples, typed mistakes/retry, full assignments, completion/invalidation, save/reload/reset and reachable resized 150% light/dark controls. Native rendering caught pathological block-tile wrapping and redundant VLSM result labels; both fixed and reviewed. Scaled allocation feedback and optional bit-weight/address-versus-mask explanation were inspected. Reference assignment values were populated from authored references before native Check. A queued-event harness bug was fixed before final PASS; it was not a learner navigation defect. Local ignored addressing-* captures are 1920×1080 and 946×929.
- Independent review found no remaining material issue. This is agent code/content/render review, not novice, physical keyboard/equipment, screen-reader or Windows/macOS desktop acceptance. Full curriculum and default-Wayland startup investigation remain open. Earlier 601 Bash comparisons were not rerun for this teaching-only batch. No release publication. M31 clean installed candidate and successful three-platform native CI identities/checksums are now recorded in INSTALLATION.md; macOS notice gate remains unresolved.


## 2026-10-22 — M33: teach the existing service recovery task

- Preserved the original meaningful service diagnosis/configuration/startup task, ID/version/state/goal/reference, editor and terminal. Added skippable plain-language service/port/configuration/current-run/boot teaching, separately explained command examples and an independent practice service with a different target port. New Learn attempts start Introduction; prior attempts retain Assignment entry and all drafts/history/credit. Challenge retains the complete task with assistance on request. No new authored lab, dependency, shell feature or coverage inflation;64/fifteen and96 Partial/1144 Open remain.
- Small service-specific saved example state, editor, command menu/input/transcript and exact checks stay separate from assignment and native Undo history. Insert and Use example port populate only their respective fields; saving, restarting, enabling and checking are explicit. Pending original service drafts now suppress current success without changing saved work. Existing specific diagnostics gained expected/actual port and failed/stopped/stale-run distinctions; status cards no longer show raw None.
- Review caught silent prefix truncation that hid recent long journal diagnostics: example entries now retain the bounded UTF-8 tail with an explicit omission marker. A supported compatibility-helper edit now refreshes only a clean example editor, preserving genuine unsaved text. Helpers remain explicitly simulator-only reference material, absent from normal command teaching. Port/journal strings are bounded consistently with preexisting accepted UI/helper limits. Foreign or incompatible metadata is rejected; old missing journals stay empty.
- Validation:310 Rust tests, formatter/Clippy,21 Python tooling/boundary checks, all28 palette notices, unchanged objective inventory/coverage and64/fifteen bundled validation. Eight backend/three native-egui additions exercise the taught workflow/experiment, failed/partial/stale/alternate repairs, helper coherence, latest Unicode diagnostics, unsupported/path input, bounds/imports, exact legacy reads/drafts, independent editor+terminal Undo, selected reset/resume, focus, modes and desktop/narrow150% geometry. Existing original nine regression suite remains intact.
- Actual Fedora XWayland flow passes fresh introduction, status/journal example diagnosis, explicit example save/restart/enable/check, fully typed independent assignment repair, is-active/is-enabled verification, success and unsaved-draft invalidation, exact resume and actual Reset/Confirm. Captures at1920×1080 and946×929 include example/assignment and light/dark150% scroll controls. Independent code/content/render review found no remaining material issue. This is agent acceptance for the scoped implementation, not novice, physical keyboard, screen-reader, actual-systemd, full curriculum or Windows/macOS desktop acceptance. Earlier601 Bash comparisons unchanged, not rerun. No release publication.
- Recorded M32 installed candidate and successful three-platform CI/checksum evidence. macOS notice gate, default-Wayland startup investigation, remaining original teaching and broader curriculum/platform acceptance stay open. Next independent work includes those readiness gates and further coherent content/capability milestones; completion checklist remains Open.


## 2026-10-22 — M34: correct native desktop identity and recheck startup

- Supplied the missing native app ID `org.dlms.LearningLab`, matching the existing desktop entry. The pinned Wayland adapter emits it only from an explicit viewport ID; protocol traces confirm the previous absence and corrected request. Independent review verified optional eframe persistence is disabled and custom progress/preferences stay at their existing paths. No dependency, theme, simulation, activation/security setting or curriculum change.
- A bounded pre-change default-Wayland startup probe already advanced five screenshots. Both post-change complete default-Wayland service and subsequent addressing flows pass current grading, separate drafts, actual typed service repair/queries, invalidation, save/reload/reset and desktop/narrow150% light/dark views. Therefore record current operation and identity correctness, **not** an established cure for the historical focus/frame-callback stall. The cause remains unresolved. XWayland is the previously verified application-local fallback.
- Native build, formatter and Clippy pass. Prior310 Rust tests/64 labs/fifteen units remain the unchanged baseline; no redundant backend suite was rerun for a window identifier. Independent source/protocol/render review found no material identity/persistence issue. Screen-reader/physical-input and Windows/macOS desktop acceptance remain open. Official upstream dispatch archive/tree still lacks the required original notice, so macOS redistribution gating remains intact without guessed licensing or removed accessibility features. No release publication; continue independent readiness and curriculum work.


### M33/M34 candidate follow-up

M33 native CI39864315623 passed all three runners; Ubuntu/Windows archives were downloaded and their sidecars verified. M34 clean-source Fedora candidate upgraded the isolated installation, retained exact synthetic progress bytes, passed outside-checkout64/fifteen content and desktop validation, and completed the full installed default-Wayland original-nine/theme-sample/seven-intro-script walkthrough with150% light/dark, input/Run/Stop and invalidation. An initial90-second timeout occurred after the final capture; the120-second rerun completed and is the accepted log. Candidate identities/checksums and precise limits are in INSTALLATION.md. File-sentinel retention is not parsed migration acceptance; theme captures alone are not fresh manual review of every palette. macOS notice and actual Windows/macOS desktop gates remain Open. No public release.

## 2026-10-22 — M35: name shared native controls without changing workspaces

- Added contextual native AccessKit names to shared search/filter/theme/text-size controls, lab actions, VLSM rows, Linux terminals, configuration/script editors, isolated examples, appearance samples and backup filename. Preserve widget IDs, actual roles/values/selections/disabled states, visible layout, focus/events, all28 themes and progress. No dependency, scenario, objective or simulation change;64/fifteen and96Partial/1144Open remain.
- Controlled baseline confirmed adjacent labels/placeholders did not name Port/Text size. Four new emitted-tree tests cover48 common assignments at desktop/narrow150%, example scopes, nondefault filters and contextual actions, retained multiline source/value, disabled samples and backup filename. Two initial test expectations assumed single-line roles; corrected expectations to the existing accurate multiline role without changing production semantics. Full314 Rust tests, formatter and all-target Clippy pass. Prior independent code/semantic review found no material issue; see ACCESSIBILITY.md for exact scope.
- Current native default-Wayland addressing and service flows pass isolated examples, actual input, educational retry, full assignments, current invalidation, exact resume/selected reset and150% light/dark resizing/scrolling. Reviewed current2560×1440 and1268×632 representative captures. Agent event/render review is not novice or assistive-technology acceptance. The subnet introduction is clearer but remains text-heavy; beginner comprehension still needs observed novice work, not an inference from passing tests.
- Specialized IPv6/switching/routing and optional host-bit semantic context, live feedback announcements, OS accessibility bridge, actual reader/physical input and Windows/macOS desktop acceptance remain Open. Historical Wayland stall cause remains unresolved despite current passing flows. Full curriculum/installation acceptance remains Open; no release publication. Continue independent completion work against the checklist.

## 2026-10-23 — M36: shared native control geometry and aligned configuration rows

- Reproduced the reported IPv4 placeholder-at-top and VLAN numeric/flowing-row defects on the native Fedora renderer before changing production layout. All single-line controls now share font-derived heights, native centered galley/hint/cursor layout and symmetric padding; buttons/selectors/labels share metrics. Multiline script editors retain top alignment. Persistent200% joins existing presets; original progress and28 themes remain. No scenario, goal, dependency or learner execution change;64/fifteen and96Partial/1144Open stay unchanged.
- VLAN settings use reserved wide grid columns and narrow labeled rows for port/connection, state, mode, access/native and applicable allowed lists. Narrow access rows omit irrelevant allowed controls and redundant headings. Per-port fields retain actual focused ID, draft and Ctrl-Z across reflow. New grid parent namespaces differ from earlier binaries only in session UI state; serialized progress has no widget IDs. Paired workstation cards share padded title/address/status slots with centered connectors; no text was shrunk.
- Validation:318 Rust tests from a clean package build; final narrow-row refinement reran all58 UI tests. Four new tests cover actual editor/placeholder paint geometry, font metrics16/24/32, real character clicking/selection/replacement, all64 assignments at200% wide/narrow, real port-field focus/Undo and appearance persistence. Formatter, all-target Clippy,21 Python tooling checks, all28 palettes/notices, bundled64/fifteen and unchanged objective-ledger checks pass. Earlier601 Bash comparisons were not rerun because no shell semantics changed.
- Native before/after matrices capture100/150/200%,1920×1080 and946×929, light/dark, blank/populated/incorrect feedback, focused fields, aligned VLAN rows and reachable scrolled Check. Representative actual renders reviewed independently; no remaining material issue. Comparison rebuilding briefly left a stale main executable in the shared target; visual review caught it. Rejected that after run, rebuilt own package, verified controls.rs in dependency identity and regenerated the accepted captures. Future baseline targets must be separate. See CONTROL_LAYOUT.md; screenshots remain ignored local synthetic artifacts.
- This is agent geometry/event/render acceptance, not novice, physical-device, screen-reader or Windows/macOS desktop acceptance. Full curriculum, specialized accessibility and platform/notice gates remain Open. Continue targeted IPv4 wording/example-action refinements, then capability-first curriculum milestones using these shared components. No release publication.
- Final default-Wayland matrix completes72 after captures with paired before evidence; full normal-app native flow also passes original-nine/library/filters/28 theme samples/seven introductory scripts, typed source/Run/Stop/invalidation and150% light/dark. Reviewed representative current terminal, selected-text sample and library renders. Captures alone do not establish fresh manual acceptance of every palette/scale. `/tmp/dlms-m36-final-native.log` and `/tmp/dlms-m36-full-native.log` are the final passing flows. Independent final source/test/render review found no material issue.

## 2026-10-23 — M37: targeted subnet teaching and honest example actions

- Kept the accepted complete /27 assignment, topology, five aligned fields and single Check. The separate /26 introduction explicitly connects32 bits, six host bits,64-address blocks, starts0/64/128/192,139 within128–191 and full network/broadcast/usable addresses. Optional supporting material correctly explains a /20 spanning two octets; /31-/32 remain excluded. No permanent panel or wizard.
- Renamed the solved-field action Fill worked example for subnet/VLSM and added short mathematically computed reference explanations only after requesting it. No implicit Check or assignment credit. Review found stale requested help after main Reset/restore; explicit session-only lifecycle and tests fix both while retaining imported drafts. Reset confirmation is brought into view rather than hidden above focused expanded help. All64 IDs/versions/initial states/goals/reference solutions compare unchanged; legacy progress/schema untouched.
- Confirmed Understand the task and sample-input/file-reset language throughout bundled instruction content; simplified related file/text/IPv6 developer wording. No shell capability or curriculum expansion:64/fifteen and96Partial/1144Open remain. See TEACHING_REFINEMENTS.md.
- Validation:321 Rust tests, formatter, all-target Clippy with warnings denied,21 Python tooling checks, all28 palettes/notices and unchanged objective checks. Added computed math/nonmutation and real UI fill/reset/restore/Challenge checks. Prior alternate/wrong/partial, draft/legacy retention and octet-boundary regressions remain passing;601 earlier Bash comparisons were not rerun for this teaching-only batch.
- Full native Fedora XWayland addressing flow passes fresh introductions, separate examples, typed mistakes/retry, both full assignments, current invalidation, save/reload/reset and150% light/dark scroll controls (`/tmp/dlms-m37-complete-native.log`). Corrected settled resize/scroll/hover harness timing after rejected captures. Independent review caught a prefix-help screenshot with help still closed; a separate assertion-backed recapture verifies actual requested help. Native render review is agent evidence, not novice/physical-device/screen-reader or Windows/macOS desktop acceptance. Default-Wayland intermittent pre-capture stall and platform/notice gates remain Open. No release publication. Continue capability-first DNS work before dependent content.

## 2026-10-23 — M38: bounded authoritative DNS capability before content

- Added a pure typed DNS module with A/AAAA/CNAME/MX/apex-NS RRsets, case/trailing-dot normalization, consistent minimum RRset TTL, bounded alias following, NXDOMAIN versus no-data/empty-nonterminal/refusal/failure and explicit unresolved external targets. Configured, loaded-primary and secondary snapshots are separate. Primary publication accepts an equal serial; secondary updates use32-bit RFC1982 comparison, including wrap and undefined half-range. Invalid publication/update leaves served state intact. No sockets, host files/processes, dependency or native/desktop change.
- Independent standards review caught a readiness distinction: structural zone checks cannot imply that NS/MX targets have address records. Added separate bounded typed owner/type/target checks for local A/AAAA presence, missing addresses and unknown external targets. Documented authority as original-name evidence rather than success, including failed aliases; no wire-packet, SMTP or network-readiness claim. Kept the original single-record seed and its exercise constraint unchanged. See DNS_SUBSET.md.
- Validation:333 Rust tests including twelve new capability tests; formatter and all-target Clippy pass, all28 palette sources/notices and objective checks unchanged. Tests cover RRset alternatives, malformed/unsupported names/imports,64-record/16-alias/byte/TTL bounds, loops/conflicts, current repair/rebreak, MX priorities, primary/secondary staleness and atomic error paths. Independent review reran12 tests, added four controlled alias/import probes and compared1,012,036 serial pairs against an independently written arithmetic equation; no remaining material finding. This is standards/model evidence, not interoperability with BIND or another actual server.
- Exported capability only: no State/Goal/native workspace, new authored lab or coverage promotion. Catalog remains64/fifteen and96Partial/1144Open. Native rendering was unchanged and not redundantly retested. Earlier601 Bash comparisons remain historical, not rerun. Next implement bounded native record editing/querying, declarative current goals and isolated teaching practice before reviewed LPIC-2 content. Full completion/platform/notice gates remain Open; no public release.

## 2026-10-23 — M39: native DNS editor, outcomes and isolated practice before content

- Added a separate AuthoritativeDns state/goal, bounded resource-record grammar, explicit saved/primary/secondary publication and in-memory query controls. Current behavioral goals grade RRsets, positive served results, intended NS/MX address records and alias paths/canonical owners; intermediate aliases/case/order/duplicates remain valid. TTL-choice assessment is explicitly absent. Native Learn/Challenge, skippable introduction/example/assignment and version1 optional drafts fit the accepted architecture. No dependency, host I/O, learner shell, original scenario change or fabricated progress.
- Origin-pinned imports and pending malformed/different drafts suppress current credit. Example fill changes only practice editor text; no automatic save/check/assignment credit. Selected reset/resume and separate native Ctrl-Z histories preserve other work. Review exposed a fixture bypass (matching portal address hid a wrong canonical host), an editor12k/48k mismatch, Unicode serial drafts that blocked saving, and secondary controls omitted for alias outcomes. Each has an isolated regression. All64 earlier IDs/versions/initials/goals/reference actions compare unchanged.
- Validation:347 Rust tests, including eight integration and six new native-egui tests; formatter/all-target Clippy pass. Tests cover partial/wrong/deleted/stale/alternate work, malformed/unsupported lines, atomic Save/reload, typed current goals,64 previous attempts, bounded import/identity/metadata, long ASCII/Unicode save-resume, actual Tab/Ctrl-Z and wide/narrow150/200% geometry. Independent review reran all backend and DNS UI checks with no remaining material finding. Existing601 Bash comparisons unchanged/not rerun; no shell semantics changed.
- Actual Fedora XWayland full flow passes isolated examples, explicit save/reload/secondary updates, typed alias lookup and malformed/correct record edits, grading/invalidation, exact save/reload and Reset/Confirm. Review rejected clipped dropdown captures and a shrunken desktop name field despite passing earlier geometry tests; final real reserved columns/shared cells, picker metrics, finite editor width and native rerenders correct them. Final full-flow and focused allocation-only logs are distinct: /tmp/dlms-m39-native-accepted.log and /tmp/dlms-m39-layout-native.log. First120-second timeout is not full-flow acceptance. See DNS_WORKSPACE.md.
- Capability/workspace only: development fixture is absent from catalog/units/coverage.64/fifteen and96Partial/1144Open remain; upcoming six distinct forward-DNS lessons require separate content review/acceptance. No novice/physical-device/screen-reader/real-DNS/Windows/macOS desktop claim. M37 three-platform native CI39907206563 succeeded for861d55e; CI alone is not runtime/installer acceptance. Default-Wayland intermittent startup and macOS notice/platform gates remain Open. No public release; continue independent curriculum work.

## 2026-10-23 — M40: six reviewed forward-DNS assignments

- Bundled a coherent native unit: add a dual-stack host, repair two looping aliases, restore canonical primary/backup mail discovery, retire one RRset member safely, diagnose an equal-serial stale secondary and complete a combined service handover. Catalog70/sixteen units (LPIC-2 now9). No numeric-variant/fixture inflation, new dependency/capability, host operations or original64 scenario/progress change. Every task has skippable teaching, separate editable practice, concrete experiment, progressive help and independent Challenge work.
- Independent review found direct-record/preserved-peer bypasses through same-answer CNAMEs. Exact configured RRset constraints now supplement publication/served/alias outcomes without freezing case/order/duplicates/TTL or legitimate intermediate alias repairs. First teaching gained a complete AAAA line and NS definition; loader policy is explicitly simulator-specific, and reload instructions match actual controls. All six suggested experiments and repairs plus23 relocation attacks were independently checked. See DNS_LESSONS.md.
- Validation:356 Rust tests, formatter/all-target Clippy,21 Python tooling checks, all28 palettes/notices and70/sixteen bundled validation. Eight new content tests plus one all-six modes/scale UI test cover taught/alternate/wrong/partial/preserved/stale/alias work, reset/resume,64 prior attempts, draft isolation and200% geometry; existing all-authored size checks include70. Fixed two baseline assumptions in tests: the new state needed a modeled broken-state mutation and the total catalog is70, while the29-script behavioral suite remains intact. No shell semantics changed;601 historical Bash comparisons not rerun.
- Native Fedora XWayland all-six walkthrough passes actual example Save/reload/update/Check controls, reference-populated assignment drafts with explicit publication/check, incorrect initial feedback, completion/invalidation, exact resume/selected reset and normal unit library. Captured every intro/assignment and representative example/completion/light/dark screens; independent rendered review found no material layout/navigation issue. The log explicitly discloses reference-populated inputs; M39 separately verified typed mistakes and actual Reset/Confirm. Current native scaled evidence is M39 representative workspace plus new all-six150/200% geometry tests, not six scaled native walkthroughs. /tmp/dlms-m40-native.log is the final PASS.
- Issuer LPIC-2 v4.5 rechecked2026-10-23, with checked RFC3596 AAAA and BIND loader-policy references. Only207.2.K4 moves to narrowly Partial forward host addition; existing reload/forward-record slices strengthen with item-specific evidence.97Partial/1143Open across1240 overlapping references; no full-objective, BIND-tool/reverse/transfer/security or exam-readiness claim. All completion/platform/notice gates remain explicit. No public release. Next continue independent shell/network/Linux capability/content and installation/accessibility work against COMPLETION.md, not another UI redesign.

## 2026-10-23 — M41: installable70-lab candidates and precise continuation evidence

- Built the clean committed46791cc Fedora candidate with target notices/source/build identity and checksums. Fresh isolated install, actual upgrade from34937f6c4ce8, manifest uninstall and reinstall pass; desktop entry and outside-checkout --version/70-lab/sixteen-unit validation pass. Exact original-nine synthetic progress bytes survived. This is file-retention evidence; parsed migration/draft acceptance remains separately tested.
- Installed native executable completed the all-six authored DNS flow outside the checkout with isolated smoke progress, disclosed reference-populated assignments, actual save/publication/check controls, invalidation, exact resume/selected resets and normal unit library. /tmp/dlms-m40-installed-native.log closes PASS. No fresh claim for every installed theme/lab/scale, novice or assistive technology.
- Native CI39912625664 succeeded on Ubuntu24.04, Windows2025 and macOS15 for46791cc (356 tests and applicable checks). Downloaded Ubuntu/Windows archives/sidecars match independently computed hashes and embedded source identity. Exact candidate paths/hashes and acceptance limits are in INSTALLATION.md. macOS archive notice gate, actual Windows/macOS desktop installation/runtime/signing and intermittent default-Wayland startup remain Open; no OS security/theme/compositor settings changed and no release published.
- Documentation-only continuation checkpoint records delivered work, remaining scope and external gates. The next Bash indexed-array contract received read-only design/fidelity review and controlled disposable Bash probes; installed GNU5.3.9 original Arrays manual was inspected after online fetch timeout. It is explicitly unimplemented/untaught: no evaluator revision, dependency, lab or coverage change. See SHELL_ARRAYS.md and CHECKPOINT.md. Full product completion is not claimed;70/sixteen and97Partial/1143Open stay honest. Continue independent implementation against COMPLETION.md.

## 2026-10-23 — M42: investigate native Wayland startup before further curriculum

- Inspected the current checkpoint,70/sixteen catalog,1240-reference97Partial/1143Open map and pinned native stack. Historical trace stops after a Mesa EGL surface frame request/commit and keyboard leave. No failing process is available now; a winit redraw-gate, focus or compositor cause is not established. Protocol XML permits withholding callbacks for obscured surfaces; that is a hypothesis boundary, not a diagnosis. No backend/vsync/security/theme/compositor change or speculative cure.
- Current direct-Wayland development evidence:15 full DNS-layout launches,40 first-render/readback probes,20 initial driver,10 hardened driver and20 quiet(no lifecycle/protocol logging) startup launches all pass. Packaged46791cc Fedora binary separately passes12 direct-Wayland DNS-layout launches outside checkout,6.47–6.92seconds each,11 quiet. Native alignment matrix passes100/150/200%, wide/narrow/light/dark, centered empty/populated/focused IPv4 text, real VLAN columns/stacked rows and reachable scrolling; root/independent render review found no material issue.
- Added opt-in content-free lifecycle observations and a bounded repeat-start/readback tool. Review hardened generated evidence paths, GDB configuration, output caps, timeout/interrupt child cleanup and strict deadline classification. Synthetic profiles now have random TempDir guards with0700 Unix permissions; real progress/preferences branch unchanged.68 UI tests, five new diagnostic boundary tests, formatter and all-target Clippy pass. The existing356-test baseline plus one UI test gives357 Rust tests; this scoped pass ran the68 UI tests, not an unnecessary repeat of unchanged core tests. Python tooling count is now26 including the two prior filesystem-harness tests. No new dependency, authored lab, coverage or evaluator revision.
- The startup cause remains unresolved/unreproduced; diagnostics will capture a bounded own-process stack/protocol if it recurs without bypassing permission controls. Independent simulation/content work proceeds. See NATIVE_STARTUP.md and tools/native_startup.py. Next implement reviewed arrays; current code still evaluator5 with arrays unsupported. No release publication; platform/notice/novice/reader gates remain Open.

### M43 — bounded typed Bash indexed arrays (2026-10-23)

- Implemented the reviewed contract before dependent content: typed scalar/dense-array runtime values, adjacency-aware compound assignments, quoted whole-list expansion, constant canonical indices/counts, old-binding atomic replacement, element0 scalar/read/for behavior, dynamic typed local restoration and pipeline isolation. Reserved base names and unsupported forms fail honestly. No new dependency or runtime host/process/network/file API.
- Added preflight retained-storage checks, array slot accounting and cumulative work charges for pipeline copies; Stop remains cooperative. Evaluator6 retains prior source/output/credit as separate history and exact non-overwriting before-shell-v6 backups; existing envelope and Case.env remain compatible. No legacy answer or array mastery is invented.
- Validation:371 Rust tests (14 new focused cases),624 controlled disposable Bash5.3.9 comparisons (23 new), formatter/all-target Clippy and24 Python tooling tests pass. Independent review caught composed-parameter and whitespace-adjacency bugs, now fixed/regression-tested; further copy-budget review strengthened preflights/work charging. The trusted development oracle no longer sets HOME; it uses a controlled environment/cwd and checked-in fixtures only. GNU online pages timed out again; installed original manual provenance remains explicit.
- Native UI unchanged; catalog remains70/sixteen (19CCNA,13LPIC-1,9LPIC-2,29Bash),97Partial/1143Open. Next author a substantive array unit using the accepted isolated introduction/example/assignment workspace. Full curriculum, packaged-current verification and external platform/notice/accessibility/novice gates remain Open.

## 2026-10-23 — M44: four practical Bash indexed-list assignments

- Added exact dispatch queues, whole-list mutation repair, two-group package inventories and a preview/save handover project, each with prerequisite teaching, a different isolated runnable example/experiment, complete task, progressive optional guidance and multiple behavioral cases. Alternative positional/function solutions pass; no exact array-source requirement. Existing70 IDs/versions/initial states/goals unchanged. Catalog74/seventeen (19CCNA,13LPIC-1,9LPIC-2,33Bash); no certification evidence promotion,97Partial/1143Open.
- Independent review caught an omitted-empty-destination grading gap in the initial eight-case draft. Added actual destination/source empty-name grading and omitted-guard negatives; capstone now10 cases. Shared bounded MAX_CASES12 applies to packs and saved records, with positive12/negative13 checks; old records/envelope compatible, one-case-at-a-time limits unchanged. Cleaned learner jargon and preserved separate practice/assignment drafts, partial work and reset/resume.
- Validation:379 Rust tests,657 controlled disposable Bash5.3.9 comparisons, formatter/all-target Clippy,24 Python tooling tests, all28 palettes and coverage integrity pass. Six new content and two UI tests exercise alternate/wrong/hardcoded/partial/current invalidation, all70 old attempts, example experiments, real egui keyboard Run/focus and100/150/200% widths/modes. Existing fixed-count fixtures were reconciled without weakening their prior-state checks.
- Native acceptance remains partial: actual Wayland first introduction captured2560×1440 and reviewed readable/contained. A harness transition needed correction. Subsequent native launches cannot connect: selected wayland-1 exists but socket connect returns ECONNREFUSED; X0 also refuses and X1024/1025 deny access. No security/display/theme settings changed, no other display was commandeered. Corrected all-four rendered workflow and installed-current acceptance remain Pending; tests are not a novice/physical-device/reader study. See SHELL_ARRAY_LESSONS.md/NATIVE_STARTUP.md. Continue independent implementation; no release publication.

## 2026-10-23 — M45: process foundation and array native acceptance follow-through

- Added pure bounded processes.rs before dependent content: fixed virtual identities/PIDs/parents/sessions, typed active/stopped/pending/exited state, current ps/pgrep projections, supported named/numeric signals, same-session CONT exception, absolute niceness/permissions and partial runtime errors. Malformed/unsupported input cannot mutate; no host lookup/process/file/network API or dependency. Duplicate/oversize process maps reject early, imported shapes have a pinning helper and required bounded-decoder boundary. No State/Goal/UI or new process curriculum is counted.
- Eleven focused tests and seven controlled owned-child Linux observations pass; independent read-only review caught SIG-prefixed numeric acceptance and it is now rejected/regression-tested. Original project ps/pgrep/kill/signal/renice manuals rendered2026-10-23. Reference tool accepts no arguments or learner input, inspects only its own disposable sleep children, cleans them up, and avoids publishing raw identity/PIDs; niceness-ceiling observations are explicitly skipped. Exact omissions in PROCESS_SUBSET.md.
- The desktop returned without app/system-setting changes. Exact installed346ffa0 ran all4 array examples and reference assignments but old harness clicked a long editor’s offscreen full text rectangle during capstone invalidation. Switched test clicks to the visible viewport and added a focused real-edit/current-success regression. Corrected dev full native array flow and separate Stop/repair/resume now pass; root/independent review accepts desktop and light150/dark200 captures.391 current Rust tests, formatter/Clippy pass;657 Bash comparisons/24 tooling/28 palettes remain unchanged from M44. Final clean installed corrected-harness retest remains required.
- M44 source346ffa0 is committed/pushed after owner restored GitHub sign-in. One push retry encountered automatic approval-review timeout, then permitted retry succeeded. Its Linux candidate, checksum/notices/clean install/upgrade/uninstall/reinstall/CLI/file-retention evidence are recorded in INSTALLATION.md; not a full curriculum/platform completion claim. Catalog74/seventeen and97Partial/1143Open unchanged. Continue process workspace/integration and substantive LPIC-1 content, then other track gaps; no release publication.

M45 additional native library capture passes for the new unit at dark desktop100% and light narrow150%, including clean card/action alignment and scrolling. Native header text retains its top padding at both scales. This is render evidence, not a novice study.

## 2026-10-23 — M46: integrate native process practice and verify the packaged array milestone

- Added typed Process State/Goal, validated declarative teaching, independent saved practice tables/commands, current outcome grading, generic role/prompt and the accepted native terminal introduction/example/assignment flow. Canonical string PID keys fix the internally tagged serde buffer path; direct/from_value/State tests prevent regression. Preservation now checks observed reparenting; feedback explains ended vs paused and actual niceness. Progress pins assignment/example identities and rejects invalid save-side metadata/type pairings before writes. Existing74 scenarios/versions/goals and earlier progress remain unchanged. No dependency or host learner execution.
-402 Rust tests pass:12 process core,7 workspace/import/save/reset/old-attempt cases,3 new native-egui focus/layout/isolation cases, all prior regressions. Formatter/all-target Clippy pass. Native development fixture passes typed separate practice/inspection, wrong TERM diagnosis, actual Reset/Confirm, alias STOP/current Check/Step0 completion, CONT invalidation, exact draft/save/load and Challenge light150/dark200. Root/independent source/render review finds no material issue. This fixture is explicitly not bundled or counted; authored process unit follows.657 prior Bash comparisons and28 palettes are unchanged, not gratuitously rerun.
- Clean017e875 Fedora candidate upgraded the isolated346ffa0 install, validates exact build/hash/74labs/17units outside checkout and retains exact synthetic data bytes. Installed all-four array flow, separate Stop and library modes pass3/3; ten quiet native default-Wayland readback launches pass10/10. Evidence/identities/hashes in INSTALLATION.md. This closes the corrected installed-harness gate, not the historical intermittent frame-callback cause. No app/system backend/theme/security workaround, release or platform/novice/reader acceptance claim.
- Catalog remains74/seventeen (19CCNA,13LPIC-1,9LPIC-2,33Bash);97Partial/1143Open. LPIC-1 official current page rechecked2026-10-23:5.0/101-500/102-500; process/priority103.5/103.6 remain Open until accepted authored teaching. Next build distinct ownership-aware pause, pending termination, priority and safe family-handover tasks with isolated examples/alternatives/negative/partial/reset/resume/native checks. Continue all other track gaps after that; full completion checklist Open.

## 2026-10-23 — M47: four substantive Linux process assignments

- Bundled an eighteenth unit: ownership-aware pause, pending termination, absolute existing-process niceness and a multi-action parent/child handover. Introductions teach prerequisites; different-PID examples and suggested edits remain isolated and saved. Full editable terminal tasks, optional help and independent Challenge are preserved. Existing74 scenario IDs/versions/initials/goals stay unchanged; no dependency or host learner execution.
- Independent review reproduced STOP→TERM falsely retaining pause success. A typed no-pending-termination requirement plus truthful task/teaching now rejects it; current-state and preservation negatives include requested unchanged niceness and reparented children. New9 content tests exercise taught/alternate/wrong/partial/read-only paths, suggested edits, reset/resume/import, prior74 compatibility and concise per-process root feedback. All-authored200% layout tests now explicitly enter process Assignment; the generic broken-attempt regression now mutates a real preserved process setting rather than sending a script comment to this terminal.
- Actual normal bundled Fedora flow passes all4 introductions, typed practice, full assignments, incorrect feedback, Reset/Confirm, successful authored reference repairs, Check/Step0 completion, state invalidation, exact saved reload, Challenge and library/light150/dark200 reflow. Root render review caught repeated Reset explanations for one ended PID; feedback now emits the first failing cause per process while keeping all grader outcomes. The final package must recapture that correction; prior development captures precede it. Inputs are authored references through injected native events, not novice/physical-device/accessibility acceptance.
- Catalog78/eighteen units:19CCNA,17LPIC-1,9LPIC-2,33Bash. Eleven narrowly Partial103.5/103.6 references plus one Supporting-only default/inherited-priority explanation:108Partial/1Supporting/1131Open of1240 overlapping references. Official LPIC-1 current version5.0/101-500/102-500 rechecked2026-10-23. Jobs, sorting, monitoring/spawning/nice/full scheduling and most remaining certification/shell families stay absent. See PROCESS_LESSONS.md. Full completion checklist remains Open.

## 2026-10-23 — M48: reviewed LLDP capability foundation

- Committedbd3a90c adds explicit one-hop LLDP transmit/receive, typed advertisements, manual bounded time/TTL cache, graceful withdrawal, immutable identity/graph reconfiguration, bounded validated restore and current-state/preservation requirements. No host I/O, background network clock, IOS parser, CDP or new dependency. No authored scenario or coverage promotion. Primary sources/precise simplifications are in DISCOVERY_SUBSET.md.
-30 focused tests pass with independent source review. Review corrected a valid maximum-size persisted-state rejection by separating256KiB topology and1MiB snapshot limits; worst-case escaping roundtrips are tested. Current-neighbor outcomes reject stale, expired, nondeliverable or changed-identity evidence. Literal absence must be guarded by whole expected Configuration or a same-attachment positive neighbor to preserve Link.up; granular global/port guards alone are insufficient. Explicit counterexample and authoring gate recorded.
- Combined worktree verification:441 Rust tests, all-target Clippy/formatter,24 Python tools,28 palettes/notices and coverage integrity pass. Final16 process content/workspace and30 discovery targeted reruns pass after the rendered feedback/example and link-preservation refinements. Native M47 all-four normal-catalog flows and independent representative render review pass; actual installed new78-lab candidate remains the next acceptance step. Older installed017e875 is still the verified74-lab artifact. Continue native/persistence discovery integration and real authored CCNA content, then remaining four-track gaps.

## 2026-10-23 — M49: native LLDP integration and exact installed process acceptance

- Added bounded identity-pinned saved discovery drafts, typed authoring/control actions, plain current feedback and separate validated teaching/practice. Normal native configuration route uses shared centered fields, explicit responsive port columns, applied topology cards, Apply/Send/Check and an editable manual clock; optional record details distinguish chassis/port IDs from descriptions. Existing78 scenario definitions/progress remain unchanged. No dependency/host learner execution.
- Review caught and fixed a pack accepted without teaching although its native route required it; missing metadata now rejects before UI. Native geometry exposed width growth from wrapped rows at200%; one captured parent-column width and explicit narrow labels fix it. Final render review corrected unsupported arrow glyphs and a misleading Port description caption. Invalid editable names/descriptions now receive field-specific atomic diagnostics. Clock request persists but is neither applied nor graded as configuration.
-469 Rust tests pass; final all-target Clippy/formatter and coverage validation pass. New15 editor/teaching,10 pack/progress and3 native-egui tests cover separate practice, real text edits/current invalidation, malformed/unsupported/large input, coordinated identity rewrites, save-side byte retention, all78 prior attempts, bounded invalid-draft actual save/load and selected reset. Independent25 model/integration+3 UI tests pass. Final actual native development-fixture flow and desktop/light150/dark200 screenshots pass root/independent review. The fixture is unbundled/not counted; no LLDP objective promotion. Withdraw remains a core/authoring action without a native learner control; do not author required inaccessible steps.
- Clean e2b0200 Fedora candidate now actually passes installed all4 process flows with final corrections, full72-capture shared IPv4/VLAN100/150/200% alignment and10 quiet default-Wayland startups/readbacks outside checkout; install/upgrade/uninstall/reinstall/desktop validation and exact synthetic data bytes pass. Source/archive/binary hashes in INSTALLATION.md. Native CI39976264683 succeeds on all3 runners; Ubuntu/Windows candidates downloaded, hashes/embedded revisions verified. No actual Windows/macOS desktop acceptance or macOS redistributable archive; notice/signing gates remain. Historical intermittent startup cause remains unresolved, not cured by passing probes.
- Catalog78/eighteen:19CCNA,17LPIC-1,9LPIC-2,33Bash;108Partial/1Supporting/1131Open. Next accept four distinct LLDP assignments staged by the helper, then continue remaining certification/shell capabilities and substantive curriculum. Full completion checklist remains Open; packaging/test counts do not close it.

## 2026-10-23 — M50: four authored LLDP discovery tasks

- Bundled a coherent nineteenth unit: receive-direction recovery; stale device/port/TTL inventory refresh; transmit-only policy retaining inbound/healthy neighbors; multi-attachment handover. Separate different-identity examples and runnable changes teach before full native assignments. All goals combine current/absence evidence with whole configuration including Link.up; alternate action orders/refresh counts pass, physical/receiver shortcuts and stale/unrelated work fail. Existing78 definitions/progress remain unchanged. No generated variants, new dependency or host learner execution.
-480 Rust tests pass, including9 content and2 new shared-card/all-authored UI regressions; final Clippy/formatter,24 Python tools,28 palettes/notices and coverage integrity pass. Independent9 content+2 layout tests and source/render/coverage review pass. Actual native all4 flow passes teaching/practice, initial/partial/full work, native Apply/Send/Check, Step0 completion, exact saved reload, expiry invalidation, selected reset, Challenge/library and light150/dark200. Authored reference drafts are populated programmatically; no novice/physical-device study is implied.
- Render review caught repeated diagrams crowding controls; a shared compact hub-and-spoke diagram now renders each device once. Shared topology text wraps using font/layout measurements, equal row/card heights and centered connectors, with truthful narrow/non-star pair fallback. Long-label regression checks painted text containment and connector attachment. The approved IPv4 workflow and shared centered input rules are unchanged. Final exact installed candidate/shared-alignment retest remains next.
- Rechecked current official CCNA v1.1 PDF and Cisco exam overview2026-10-23. Only2.3 receives narrow LLDP Partial evidence; CDP, IOS, automatic scheduling/full TLVs/802.1AB and physical acceptance remain absent, futurev2.0 stays separately planned. Catalog82/nineteen (23CCNA,17LPIC-1,9LPIC-2,33Bash);109Partial/1Supporting/1130Open. Most full-course requirements remain Open. See DISCOVERY_LESSONS.md.
- Next clean commit/package/actual installed verification, then continue Bash cooperative command-substitution work against a separately reviewed contract/reference observations. No shell evaluator change or new scripting coverage in this milestone. Platform/notice gates and no-public-release boundary persist.


## 2026-10-23 — M50 installed acceptance and M51 focused usability follow-through

- Clean a741b65 Fedora candidate actually passes isolated upgrade/install/uninstall/reinstall, desktop validation, exact build/content/hashes, synthetic old-progress file retention, all-four installed LLDP lesson flow and the 72-capture shared IPv4/VLAN alignment matrix at 100/150/200%, wide/narrow/light-dark. Ten quiet default-Wayland startup/readback launches pass; historical startup cause remains unresolved. CI39986036392 passes all three runners. Exact identities/evidence are in INSTALLATION.md; no Windows/macOS desktop inference or public release.
- Installed render review exposed feedback below the viewport after Check. M51 makes shared Check actions reveal existing results, without new panels, altered grading/drafts, passive-render scroll requests or exposed optional solutions. Native topology cards now emit current semantic image descriptions without geometry/focus changes; actual screen-reader acceptance remains pending. Recovery guide now correctly names evaluator6 and explicitly leaves substitution/revision7 disabled. No curriculum/coverage change:82/nineteen,23CCNA/17LPIC-1/9LPIC-2/33Bash,109Partial/1Supporting/1130Open.
- 81 native-egui tests, Clippy and build pass; source review accepts changes. Actual native all-four LLDP recapture passes; root reviewed fresh initial-feedback/completion with visible results and unchanged controls. Separate examples, saved reload/reset/current invalidation and narrow light150/dark200 pass. Other native workspace families and later passive manual-scroll interaction were not separately recaptured in this focused pass. Clean installed M51 follow-through remains next. No shell semantics or new capability activation in this pass.


## 2026-10-23 — M52: reviewed parser foundation, capture execution still gated

- Integrated accepted isolated9e5e985 as2162c39: typed original-source substitution bodies, bounded nested scanning and a shared immutable instruction arena with stable relocated targets/main isolation. The public VM rejects captures before any instruction; revision stays6, separate terminal remains unsupported. No new runtime, dependency, persistence schema, authored lab or objective promotion. Existing82/nineteen and109Partial/1Supporting/1130Open unchanged.
-495 main all-target Rust tests pass. Prior isolated99 scoped tests/Clippy/fmt and exact657 supported native outcomes plus33 authored records/182 cases match the genuine frozen evaluator6 baseline with zero differences. Independent parser/source/artifact review accepted; it did not rerun Cargo during root native work. Subsequent cooperative command preparation/child execution/resource/migration work remains unaccepted and gated; no new Bash fidelity claim.
- M51 clean eff98f0 candidate now actually passes isolated upgrade froma741, install/uninstall/reinstall, desktop validation, outside-checkout identity/hash/content, exact synthetic old-progress bytes, all-four installed LLDP flow/visible feedback plus narrow light150/dark200 and10/10 quiet Wayland startup/readbacks. Exact hashes/scopes in INSTALLATION.md. No release, historical startup cure, novice/reader/physical-device or Windows/macOS desktop inference. Continue independent typed persistent-mount capability before dependent LPIC-2 content; plans do not count as labs.


## 2026-10-23 — M53: typed persistent-mount capability before content

- Added independent pure virtual storage state: bounded immutable UUID/type inventory and device aliases, saved six-field fstab subset, separate live mounts, atomic target mount/unmount, simulator validation and independent automatic-session tests under fixed alias permutation. Explicit ro/rw and auto/noauto semantics, last-wins options, resolved duplicate-volume checks and detailed line/type/source diagnostics. Existing Mount seed/scenarios/progress unchanged; no new dependency, host device/files/kernel/process API, lesson or coverage promotion.82/nineteen and109Partial/1Supporting/1130Open stay unchanged.
-512 all-target Rust tests pass; final UUID-case-only correction reruns all17 model tests, probe and8 fixed parser comparisons. Final Clippy/formatter,27 Python tools,28 palettes/notices and coverage integrity pass. Independent final17 Rust+3 tool boundary+8findmnt comparisons and source review accepted. Fixes include pre-allocation constructor bounds, fixed alias-key permutation, noauto-skip validity distinction and exact UUID string matching. Hardware read-only fallback is deliberately deferred. See STORAGE_SUBSET.md for complete scope.
- findmnt2.41.5 reads only private synthetic tables with canonicalization/evaluation disabled and explicit columns; no real mounts, devices or personal paths. Comparison proves fields, not kernel policy. Primary mount/blkid/findmnt references pinnedv2.41.2; unavailable pinned fstab path is reported, upstream-rendered current manual version separately recorded. No native/teaching/integration/goal acceptance yet: these are required before four planned distinct mount assignments count as implemented. Full curriculum/platform/notice gates remain Open; no release.
- Shell M2 remains isolated/uncommitted: review found pending-buffer accounting and an unnecessary function-argument clone despite golden equality. Helper is correcting them plus explicitly diagnosing a preexisting finite-input snapshot/truncation boundary. New semantics must ship with evaluator7 and actual6→7 preservation/migration checks, not silently as6. Main retains only gated parser foundation; no capture execution/new script content is claimed.


## 2026-10-23 — M54: native saved/live mount investigation before content

- Added typed PersistentMounts goals, complete saved/live/fresh requirements, strict pack validation, separate Learn introduction/practice/assignment, focused configuration editor/investigation terminal and optional saved drafts. Every point needs saved/live outcomes, a positive current mount and changed-alias test; every clone rechecks saved identity/policy including skipped noauto entries. Source/state results grade semantics, not command text. Original82 definitions/progress unchanged; new private fixture remains unbundled/not counted. No dependency, host device/process access or new shell execution.
-531 all-target Rust tests pass; final85 UI rerun, Clippy/formatter,27 Python tools,28 palettes/notices and coverage integrity pass. Independent15 integration+4UI checks pass; both demonstrated malformed-goal/manual-source exploit packs now reject. Review caught and fixed duplicate goal-map keys, skipped-source identity, hidden clone identities, allocation/buffer bounds and repetitive source feedback. See STORAGE_WORKSPACE.md.
- Actual final Fedora full native fixture flow passes isolated fill-only practice, explicit save/terminal events, wrong/malformed/partial/reference work, Step0 completion/current invalidation, exact saved reload, selected reset, Challenge and light150/dark200, including actual Check/visible feedback at200 and an explicit identity-rich fresh-session test. Synthetic reference text is supplied, not novice/physical-device acceptance. Logs /tmp/dlms-m54-native-toolbar-final.log and final short-value follow-through /tmp/dlms-m54-theme-name-native.log. Root/independent representative renders accepted.
- Shared alignment follow-through passes all72 native IPv4/VLAN100/150/200%, wide/narrow/light-dark captures (`/tmp/dlms-m54-alignment-toolbar-final.log`); root reviewed centered placeholders/values/focus, numeric VLAN columns and narrow reflow. Native debugging corrected new font-measurement lock ordering and a scroll pointer outside the actual viewport; former64-point minimum scrolling and surplus vertical margins could consume the tiny tiled body. Font-derived responsive padding preserves a whole native control row; normal18-point margins/horizontal spacing/font sizes unchanged. A real reserved theme-picker cell keeps Settings on its row where measured widths fit; selected theme displays its actual name, with groups/full names retained in the28-option menu and attribution. No desktop/theme/security/backend changes or historical startup-cure claim.
-82/nineteen (23CCNA,17LPIC-1,9LPIC-2,33Bash),109Partial/1Supporting/1130Open unchanged. Next independent review/tests/native acceptance of four proposed substantive storage assignments staged in /tmp; they are not counted yet. Latest actually installed candidate remains clean eff98f0/82; this source integration is not inside that artifact. M3 shell captures remain private/gated and require evaluator7 preservation before activation. Full completion/platform/notice gates remain Open; no release.

## 2026-10-23 — M55: four substantive persistent-mount lessons

- Added stable identity, saved/live policy repair, manual-only access and a multi-fault handover, retaining the complete assignment/editor/terminal and separate teaching/example states. Supplied UUID tickets distinguish purpose from inventory; all saved rows, live outcomes, healthy peers and fresh alias permutations remain graded. Original82 definitions/progress unchanged. Catalog86/twenty units:23CCNA/17LPIC-1/13LPIC-2/33Bash. Longer optional duration filters expose20–35minute tasks without changing old filter values.
- Content author proposed lessons; root independently reviewed prerequisite teaching, actual state transitions, four independent alternatives,33 plausible negatives, partial work and example experiments/repairs. Seven content regressions cover prior82/new resume, independent drafts, selected reset/current invalidation and composed search/favorite/time filters. One UI matrix covers all four across teaching views/modes/100/150/200 and widths. Full catalog regression now tests the visible no-wrap editor viewport rather than its intentionally scrollable child contents; broken-attempt persistence includes invalid saved mount rows.
-539 all-target Rust tests, Clippy/fmt,27 Python tools,28 palettes and coverage integrity pass. Native four-lesson flow PASS `/tmp/dlms-m55-native.log`: actual Save/terminal/Check events, ungraded examples, initial feedback, complete results atStep0, exact resume/current invalidation, selected reset, Challenge and narrow light150/dark200. Reference editor drafts supplied programmatically. Root reviewed actual introduction, complete capstone task, policy feedback and narrow200 primary action. No novice/reader/physical-input acceptance claim.
- Current official LPIC-2 v4.5 source rechecked2026-10-23; complete inventory retains2026-10-22 fingerprints. Only203.1.K3 UUID and203.1.T5 bounded blkid newly gain Partial practice; existing fstab/mount evidence strengthens.111Partial/1Supporting/1128Open of1240 overlapping refs.203.2 maintenance, swap, systemd mounts, full utility syntax and real boot remain absent. See STORAGE_LESSONS.md. Full completion stays Open.
- No dependency, host learner execution, desktop change, installer release or public release. Latest actually installed artifact remains M51eff98f0/82 until clean M55 candidate installation tests below. Isolated shell7 migration/resource work continues with independent review; no premature public activation under evaluator6.

### M55 installed follow-through and alignment evidence correction

- Clean a6a6043 candidate actually installed/upgraded fromeff98f0 outside checkout; uninstall/reinstall, identity/hash/content, desktop validation and exact synthetic old-progress bytes pass. Installed all-four storage flow and10/10quiet default-Wayland startup/readbacks pass. Exact artifact hashes in INSTALLATION.md. CI40009847858 dispatched for this source; result pending.
- Actual screenshot review caught old alignment harness wheel targeting the header at200% in short tiled windows. Corrected capture-only code to use the existing actual-main-viewport reveal helper for fields and Check; production controls/geometry/grading are unchanged. All72 corrected development captures complete; root reviewed populated200% IPv4 and VLAN controls with centered values/labels visible, plus wide150 table columns and blanks/focus. Old printed capture PASS did not establish invisible field acceptance. Installed recapture of corrected harness remains next. No startup-cure or novice/physical-device claim.

## 2026-10-23 — owner-prioritized Fedora build memory safeguards

- Inspected existing configuration (none in project/user/isolated Cargo paths). Added project two-job Cargo/default two-worker test-harness limits; dev runner also supplies explicit flags/env. One shared OS lock spans all same-repository worktrees. Path/config/abbreviated-output bypasses are refused; resolved owned locations must remain on the verified disk volume. Markers/inherited POSIX lock protect abrupt wrapper loss; sampler failures fail closed and bounded cancellation targets only the owned process group.
- Future temporary/cache/native/package output uses ignored main `.build-artifacts` on NVMe-backed Btrfs. Copied the existing isolated Rust1.99.1 toolchain/cache there, retaining sources. Candidate staging/compilation and metadata capture share the guard; large hashes stream. CI limits are explicit. AGENTS.md/BUILD_WORKFLOW.md/README retain this policy for later agents. No new crate/package/global machine setting.
-44 Python tests pass and independent read-only source review accepts Linux/POSIX scope. Necessary ordinary Clippy and591 Rust tests passed with two jobs/two runtime harness workers and disk temporary data. Minimum observed available memory10.76/10.85GiB respectively; memory PSI averages0 and OOM-kill counter delta0. Swap remains high; no safety guarantee or stress-test claim. Exact scope and records in BUILD_SAFETY_EVIDENCE.md.
- Inventoried1913 DLMS /tmp entries/12.051GiB, checked running references and preserved uncertainty. Owner explicitly approved only seven named old ELF executables; removed exactly1,513,992,192 allocated bytes (1.410GiB) after fresh identity/process checks, preserving their sources and all other data. Local exact-path proposal/result and full CSV/JSON remain ignored. No wildcard removal or cargo clean.
- Application remains86/twenty units (23CCNA/17LPIC-1/13LPIC-2/33Bash),111Partial/1Supporting/1128Open. Shell7 local integration and native backup-controller/content gates remain separate; new capture lessons and pure ACL foundation do not count as delivered curriculum. Continue the authorized build through this workflow. No release.


## 2026-10-23 — M56: public bounded capture, exact progress preservation and native acceptance

- Integrated independently reviewed cooperative preparation/runtime and public evaluator7. Typed original-source captures run in the same bounded in-process VM; child variables/functions/arguments are isolated, virtual files and finite input remain shared, all descendants share execution/memory/output limits and responsive Stop. No host lookup, dependency or terminal-language expansion. Unsupported forms remain diagnosed. Genuine frozen6 artifacts retain657 observations and33 authored records/182cases; five intentional old resource/input differences remain explicit history rather than silently changing evaluator6.
- Exact first-write workspace/shell snapshots now reject unrelated occupied valid-JSON slots; target-derived transition receipts survive Reset/practice-only/repeat writes, are validated/stripped at portable boundaries, and never claim cryptographic protection. Confirmed corrupt/future-target restore preserves exact bounded raw originals. Independent29 migration/backup/race/failure/cancellation checks accepted; normal all-target suite591passed under safe two-job/two-test-thread workflow.
- Final87 native-egui tests, all-target Clippy and formatter pass. Actual final native Fedora flow verifies frozen33 old6 records/exact first bytes/history, current7 keyboard repair/Run, independent examples, active child-loop Stop leaving no graded result, save/reload/retry, light150/dark200 reachable Run and backup export/review/cancel retaining exact main bytes. Log `.build-artifacts/logs/m56-final-native.log`; captures in ignored disk native-runs/run-iixww7is. Root reviewed actual chooser/review and large-text controls. Not a novice/physical-device/screen-reader/Windows/macOS study; active backup-worker cancellation is backend-tested rather than claimed as a native interaction.
- Native review exposed a real chooser footer problem: recovery files/long folders could push Check backup below the viewport. Font-derived reserved footer space and bounded scrollable Settings correct this without new panels; matrix regression covers100/150/200 and two window sizes. Existing centered fields, script-editor alignment and accepted workspaces remain intact.
- Catalog stays86/twenty units,23CCNA/17LPIC-1/13LPIC-2/33Bash;111Partial/1Supporting/1128Open. Four capture lessons remain staged until independent content and native review. Next clean disk-backed candidate/actual installed acceptance, then substantive content and reviewed ACL routing integration. No release or full-product completion claim.


### M56 installed acceptance and safe-build observations

- Clean f4d7bc0 Fedora candidate actually upgraded from974381f, uninstalled/reinstalled and verified outside checkout with desktop metadata, embedded identity/content, exact hashes and separate synthetic original-nine progress-byte retention. Installed public7 native Run/Stop/history/example/resume/export/review/cancel flow passes; root reviewed actual renders. Ten quiet default-Wayland launches pass10/10. INSTALLATION.md records exact archive/binary hashes and scope; no historical startup-cure or other-platform inference.
- Candidate compilation used two jobs under the shared guard,151 memory samples: minimum available11.96GiB, maxsome/full PSIavg10 0.04,OOMcounterdelta0. Installed native25samples minimum12.89GiB, maxPSI1.30,OOMdelta0. Swap remained nearly full; observed low-pressure success is not a safety guarantee. All new large staging/installation/evidence is on project disk, not RAM/tmp.
- Independent read-only source review accepts f4d7bc0 recovery sizing/synchronization and preserved callbacks. Staged four capture lessons passed7 independent normal-Cargo content tests; native rendering, full integration and bundling still pending. Catalog remains86/twenty; no new coverage claim.


## 2026-10-23 — M57: four substantive command-output and reporting assignments

- Added normal offline unit `shell-command-capture`: independent list preview, intact evidence publication, helper findings/status diagnosis and a guarded preview/save batch capstone. Each teaches before the complete task with different editable examples and concrete experiments; examples remain ungraded/separate. Multiple supported solutions pass, including a non-capture capstone alternative. No existing scenario definition, new dependency, host execution or UI redesign. Catalog90/twenty-one units:23CCNA/17LPIC-1/13LPIC-2/37Bash.
- Root independently computed all33 whole-outcome expectations using separate line/set/string/guard models. Seven content tests cover references/two alternatives,16 plausible incorrect programs, partial cases, first-input hardcodes, unrelated-file damage, edits, examples/repairs and exact preservation of all86 earlier attempts. New UI tests check bundled assignments at100/150/200 and correct nonzero-result feedback. A status1 investigation can be correct; feedback now asks for the status specified by that case.
-601 unique Rust tests pass across lib,89 native-egui UI tests and every integration target. The initial all-target run exposed stale catalog-prefix/count assumptions; later targeted runs corrected them, extended old/new preservation checks and supplied four missing concrete example edits. All original failed logs remain. Unchanged89 UI tests were not repeated merely to obtain a single green wrapper invocation. Final all-target Clippy, formatter,44 Python tool tests,28-theme provenance and coverage integrity pass. Safe two-job/two-runtime-worker/shared-lock workflow used throughout.
- Actual final native normal-bundle all-four flow PASS `.build-artifacts/logs/m57-bundled-native.log`: introductions, example Run/change, full assignments, keyboard partial/repair/Run, Step0 completion, exact save/resume and example retention, success invalidation, selected Reset confirmation, library filter and Challenge light150/dark200 reachable Run. No fixture injection adds the lessons; the normal application already bundles them. Earlier pre-bundle run and actual renders remain in disk evidence. Root reviewed introductions, complete capstone, partial feedback and large-text controls. Synthetic events/reference drafts are not beginner/physical-device/screen-reader/WindowsmacOS acceptance.
- Rechecked current official LPIC-1 v5.0/105.2 on2026-10-23, confirmed K2 wording fingerprint; only105.2.K2 gains narrowly Partial practice, parent/K3 evidence strengthens.112Partial/1Supporting/1127Open of1240 overlapping references, not completion percentages. No mail, actual shebang/executable/ownership, broader shell or full-objective claim. SHELL_CAPTURE_LESSONS.md records exact source/readability/fidelity limits. Independent read-only integration/privacy/coverage review accepted after root's count fix.
- Latest installed artifact remains verified M56f4d7bc0/86 until a clean M57 candidate is built and actually tested; no premature package or release claim. Continue reviewed ACL configuration/grading foundation and Linux account core before dependent curriculum; most full-product requirements remain Open.


### M57 installed follow-through

- Clean7ffbfff candidate actually upgraded fromf4d7bc0 and passed uninstall/reinstall, desktop validation, outside-checkout identity/content/hash and separate original-nine synthetic progress-byte retention. The installed all-four new lesson flow passes, including native keyboard/Run, examples, partial/correct feedback, saved resume, selected reset/current invalidation and Challenge light150/dark200. Root reviewed actual renders. Ten quiet Wayland startup/readbacks pass10/10; historical cause remains unresolved. Exact hashes/scope in INSTALLATION.md. No release or novice/reader/other-platform claim.
- Safe two-job candidate compilation41 samples: minimum available12.12GiB, PSIavg10max0,OOMdelta0. Installed native91samples minimum12.81GiB, PSI0,OOMdelta0. Swap remains nearly full; not a memory guarantee. All large temporary/install/evidence output stays project disk-backed. CI40028251720 at this exact source has Ubuntu success; Windows/macOS pending at last check.

## 2026-10-23 — M58: reviewed ACL foundations and bounded application file input

- Integrated independently reviewed pure access-list classifier, endpoint-bound transit integration and native-document/fresh-grading adapter asc079f18/74fd8ce/92d9840. Supports ordered standard/extended IPv4 wildcard rules, explicit TCP/UDP ports, direction-bound router filtering, guarded no-ACL baselines, required healthy crossing traffic, optional specified denial cause, exact topology/host pins and malformed draft retention. No cached Inspection is accepted as a grade. Parser/renderer/apply do no host I/O. Full-router replacement and narrow unsupported grammar/empty-list behavior are explicit, not full IOS emulation.
- Root independently reran107 main lib/ACL/editor/old-routing/content/workspace tests and all-target Clippy; all pass. Earlier isolated60 focused checks/source contracts and hash manifests remain. No State/Goal/progress/native screen or authored ACL scenario is integrated by these three pure-foundation commits. CCNA5.6 remains Open; finite required flows do not prove arbitrary hosts/ports. Catalog stays90/21 and112Partial/1Supporting/1127Open.
- The developer `--validate-pack` path previously read a whole file before its JSON-size check. It now uses a common application-local ordinary-file reader with a2,000,000-byte cap before UTF-8/JSON allocation; progress/backup retain the same bounded reader behavior through a compatibility wrapper. A checked ceiling rejects arithmetic overflow. Static symlink/directory/invalid UTF-8, exact/+1 limits, actual CLI and original-byte retention tests pass. This file-loading path is not exposed to learner commands/scripts; no host execution or new dependency.
- Final29 file/backup/shell-migration checks and all-target Clippy pass; four are new file-boundary tests,25 preserve existing backup/migration behavior. Independent read-only source review accepted the shared helper/CLI boundary. No broader native recapture was needed for this read-only developer flag; actual M57 installed UI acceptance remains separately scoped.
- Continue backend ACL workspace/teaching/pack/progress integration in isolated acl-backend-integration, then native/editor review before any content. Separate numeric-identity account core is reviewed and has74 isolated checks, but still awaits main acceptance/integration; no account labs/coverage yet. Full completion remains Open.

## 2026-10-23 — M60: typed native ACL workspace and reviewed account backend

- Added native full-task policy editor dispatch with separate introduction/practice/assignment, default-visible packet requirements, equal topology cards, optional references and one Check configuration. This uses an unbundled fixture; no new authored-lab or5.6 coverage credit yet.
- Independent source review caught aggregate raw-draft overflow and tab-focus handoff; both fixed before acceptance. Native screenshot review corrected vague partial-block feedback, a missing arrow glyph and duplicated malformed-document messages. Ordinary test-fixture construction also needed down/addressless interfaces rather than invalid interface-less devices; failures are retained in local logs.
-186 main combined foundation/backend/old-profile checks,10 ACL workspace checks,5 new UI checks and89 prior UI checks pass.3 IPv6 UI checks cover the plain-text probe display correction. Final all-targetClippy and formatter pass. Actual private Fedora native flow and rendered light/dark100/150/200 wide/tiled screens reviewed; see ACL_WORKSPACE.md. Shared centered single-line controls and prior alignment work are retained.
- Account backend b95157f is accepted in its isolated branch after independent source review,134 guarded checks and all-targetClippy. Reviewer caught a save guard comparing evolving assignment/example namespaces; corrected to immutable context separation with both-direction coincident-name/ID and export/confirmed-restore regressions. Root main integration/native account renderer are next, not claimed complete here.
- Source7ffbfff CI40028251720 completed successfully on all3native runners; Windows/macOS desktop runtime/installation/signing, macOS dispatch notice, physical accessibility and novice acceptance remain pending. Latest actually installed Fedora candidate stays7ffbfff/90labs; later source changes are not inside it. No release publication.
- All heavy work serialized through main safe runner,2Cargo jobs/2test threads, verified NVMe Btrfs artifacts. Eight root evidence intervals minimum available10.94–12.79GiB; some/full PSI avg10max0 and observed OOM counter deltas0. Swap remained effectively full, at times0bytes free. No memory guarantee or machine setting change. Private exact evidence `.build-artifacts/audits/m60-verification-memory.json`.

Next: integrate the reviewed account backend/native terminal workspace, then engine-test/source-review/native-walkthrough four distinct ACL and four account lesson proposals. These authored drafts are not catalog additions until all gates pass. Continue remaining curriculum and fidelity gaps against the completion checklist.


## 2026-10-23 — M61/M62: integrated account backend and four ACL assignments

- M61 cc7d355 merges the independently reviewed account backend, preserving both ACL and accounts goal/workspace/teaching paths and old progress.204 combined checks and all-target Clippy pass. Account native UI/content remain separate until their acceptance; no curriculum count from backend plumbing.
- M62 normal bundle adds four distinct ACL assignments with different separate examples: standard source filtering, extended service ordering, two-router attachment/direction diagnosis and constrained handover. Catalog94/22units,27CCNA/17LPIC-1/13LPIC-2/37Bash. Current CCNA5.6 alone gains narrow Partial practice;113Partial/1Supporting/1126Open overlapping references, not completion percentages. ACL_LESSONS.md records sources and omissions.
-12 authored-content tests cover31 assignment/22 example packets, valid alternatives, common wrong policies, source-port confusion, seven capstone omissions, every packet's applied-state invalidation/repair and all90 previous attempts.91 overlapping catalog regressions,6 focused UI and22 prototype checks pass. Initial Advanced metadata and generic old test-helper failures are retained and corrected through shared validation/goal-aware helpers, not fake command acceptance.
- Actual normal native all-four editor/picker/Check/example/partial/alternate/completion/resume/invalidation/reset flows pass. Final layout flow checks wide100%, Light150/Dark200 tiled and native library Advanced filtering. Root reviewed actual screenshots; shared read-only grid minimums correct capstone Check visibility without shrinking text. Logs m62-acl-unit-native-r1/m62-compact-native-r3, ignored native-runs/run-bvfj3ijt/run-gcl741rs. This is synthetic native technical acceptance, not novice/physical-device/screen-reader/Windows/macOS runtime acceptance.
- Source changes are outside installed7ffbfff/90 candidate until a new clean artifact is actually tested. Keep two-job/two-runtime-thread/shared-lock/project-disk safeguards. Continue account native/content integration and remaining four-track curriculum; no release or completion claim.


## 2026-10-23 — M63: specific account-state feedback

- Replaced generic failing account checks with fresh bounded first-mismatch guidance for identities, memberships, expiry metadata, file state and preserved configuration. Original Boolean grading predicates, protected context and passing messages remain unchanged; no file bodies/comments are disclosed. Missing supported usermod operands receive an invalid-input diagnostic while unknown syntax remains unsupported.
- Independent source review and139 overlapping account/lib/feedback/workspace/ACL/backup/pack/migration checks pass. Native inspection of these new messages is reserved for the next authored account unit; no authored lab/coverage credit here. No dependency, host tool, progress-schema or shell-evaluator change.


## 2026-10-23 — M64: native account integration and pure OSPF foundation

- Native account renderer is integrated as a9e238d, keeping separate terminal/input/undo scopes and Challenge visibility, fixing follow-newest output and soft-wrapping educational diagnostics. Root six account UI plus six ACL UI checks pass after routine both-path merge conflicts. Original failed long-output geometry run is retained; new feedback exposed the need for wrapping rather than weakening its visibility assertion.
- Pure OSPF compatibility/SPF foundation a74e705 has69 isolated library/core checks and Clippy, plus root source review. No actual packets, neighbor FSM, broadcast DR/BDR, forwarding, authored scenarios or coverage credit. Review identified an unjustified host-address restriction on router IDs; a narrow follow-up accepts all nonzero32-bit identities and keeps zero/missing/duplicates distinct. Focused and combined main verification records that correction separately.

## 2026-10-23 — M65: four local-record lifecycle assignments

- Normal bundled unit local-account-lifecycle follows access foundations: explicit onboarding/template home, exact membership/home GID repair, evidence-preserving retirement and stable-GID/expiry-metadata handover. Catalog98/twenty-three units:27CCNA/21LPIC-1/13LPIC-2/37Bash. Each teaches before the meaningful full task using distinct, separately saved runnable examples and concrete experiments. No host accounts, credentials, authentication or session behavior.
-10 authored-content tests pass references/10 alternate sequences/20 wrong solutions, omitted actions, partial states,31 individually breakable typed outcomes, protected context, example isolation, original94 attempts/legacy history, selected reset and confirmed backup restore. Six feedback tests and overlapping old integration/prototype/storage/shell regressions pass; two generic helpers needed explicit new account context handling rather than weakening the simulator. Original failed logs remain.
- Actual normal-bundle all-four Fedora native flows PASS m65-accounts-unit-native-r1.log, native-runs/run-na9heqqn: introductions, independent insert/run/change/repair, full terminal tasks, missing operands, partial/alternate results, keyboard Enter, Step0 completion, exact saved resume, fresh invalidation, repair, selected Reset and Challenge Light150/Dark200. Root inspected actual renders at2560×1440/1268×632. Follow-up display-only deduplication retains all scoring but removes an exactly repeated preservation error; targeted final fixture-native render passes m65-final-dedup-native-r2.log/run-mxr12o60; root reviewed the single specific membership error. No novice/physical-device/screen-reader/Windows/macOS acceptance inference.
- Rechecked official current LPIC-1 v5.0/107.1 on2026-10-23; eleven previously Open local-record/tool references gain narrow Partial practice, existing parent/membership evidence strengthens.124Partial/1Supporting/1115Open overlapping refs. Passwords/chage/PAM/login/special-purpose policy/full NSS/options remain absent. ACCOUNT_LESSONS.md records source retrieval limits and boundaries. Final combined all-target run passes761 Rust checks (including102 UI and23 OSPF); all-target Clippy, formatter and98-lab coverage integrity also pass. Final8-capture fixture native render confirms deduplicated feedback. Clean installed candidate acceptance remains pending at this entry; no premature package/completion claim.


## 2026-10-23 — M66: pure unit manager and repeatable installed acceptance

- Integrated corrected pure unit-manager foundation as ec08cd2. Root source review caught failed-Requires ordering through an unstarted intermediate; controlled exact-original red regression fails, corrected graph passes. Main104 lib/manager/OSPF/account-content/backup checks pass after preserving both module exports. No existing single-service goal, UI or progress conversion, authored scenario, full systemd/boot claim or new dependency. UNIT_MANAGER_SUBSET.md explains saved/loaded/runtime/boot separation, job versus readiness, strict loader and bounded fictional programs.
- New fixed Linux installed_acceptance tool runs through the main guard, replacing repeated private lifecycle scripts. It validates candidate/dist/install containment and disk type, verifies previous/current build identities/hashes, upgrades/uninstalls/reinstalls only inside a new private prefix and retains exact synthetic original-nine bytes. GUI/native startup/platform acceptance is separate. Independent review caught dangling-link/desktop false acceptance and child-root symlink escapes; fixed before actual use with targeted negative fixtures. A nonblocking Linux capture avoids inherited-pipe reader-thread teardown deadlocks, without detaching from the outer owned group.53 Python checks and final merged all-target Clippy pass, including a fixed own-child/descendant-held-output timeout fixture; no stress workload or learner script. Actual clean candidate evidence is recorded after execution, not inferred here.

- M66 follow-through: exact877 candidate installed lifecycle, all4accounts/all4ACL native flows,72IPv4/VLAN alignment captures and10/10quiet Wayland launches now pass. Representative actual renders reviewed. INSTALLATION.md records exact identity/hashes/evidence. This is scoped Fedora technical acceptance only; startup root cause, other-platform desktop and full curriculum remain unfinished.

## 2026-10-23 — M67: native OSPF configuration unit

- Reviewed backend94ce288 received native router/process/port settings, separate prerequisite/example/assignment views, one whole-configuration Check, fresh local mismatch feedback and optional worked/scope help. Real layout regression found picker overflow at200%; fixed reserved atomic widths, preserved shared text/cursor geometry and stable reflow IDs. Source review caught enable/type omissions in optional references; corrected. Disabled backup-modal background cannot apply or take focus. Passing preservation rows are secondary details; current failures remain visible. No new dependency or evaluator change.
- Four distinct normal assignments: identity/process recovery, P2P area/timer handover, directed ECMP with asymmetric reverse path, passive user-LAN/site handover. Full explicit tasks retain healthy context, accept alternatives and keep worked networks separate.8content tests execute12reference/alternate routes and21wrong/partial cases, independent metric/hop equations, current invalidation, examples and backup/resume/reset.23core and18adapter checks pass. Normal69account/course/OSPF/prototype/storage regressions pass; generic old broken-attempt helper needed a real goal-aware disabled OSPF process, not raw fallback or a weakened learner parser. Its original failure and a subsequent test field-name compile error are retained.
- All four normal native flows PASS,84actual captures in m67-ospf-authored-native-r1.log/run-12xazcls: real menus/text/toggles/Check/Reset, ungraded example fill/check/change/repair, full tasks, initial/partial/wrong/reference/alternate outcomes, pending/applied invalidation, malformed atomicity, exact both-draft resume, Step0 completion, selected reset, Challenge Light150/Dark200. Root reviewed actual100% task/intro/ECMP and large-light/dark screens. Private earlier43-stage/14-capture flow also passes; a closed-popup completion-frame harness defect was diagnosed from an actually open menu and corrected. No novice/physical-device/screen-reader/WindowsmacOS acceptance claim.
- Actual task review found alphabetical East/Backup initially obscured the task. Small polish now focuses a NEW/RESET attempt on its task router while retaining saved selection; a new example focuses its first explained action. Complex link readouts expose raw directed and LAN costs, without solved totals.8focused UI checks pass including96authored size/scale/view combinations, long/passive fields, keyboard focus, modal isolation and new/resumed selection. Earlier106UI+8content+18adapter run passes132overlapping checks. Final outcomes are recorded below; initial broad catalog-bookkeeping failures remain in the evidence. Failed lint/layout/native diagnostics remain local.
- Normal catalog102/24units:31CCNA/21LPIC-1/13LPIC-2/37Bash. Current CCNA v1.1/3.4 sources checked10-23; four narrow slices newly Partial,128Partial/1Supporting/1111Open references. No live neighbor/LSA/timer/convergence/DRBDR/IOS/forwarding/equipment claim, no futurev2 promotion, no full preparation or complete-app claim. Original98 scenarios/progress and all28 themes preserved. Most completion checklist work remains Open.

- Final current-source library/integration checks pass701 across68targets;110UI checks passed in the broad run before test-only catalog bookkeeping fixes (811distinct checks combined, no repeated UI count). Old ACL/account/discovery/storage/shell fixture totals were updated to102; original94/raw-wrapper and98-ID preservation scopes remain pinned. No learner parser/predicate was weakened. All-target Clippy passes. The final normal all-four native run also passes84captures in m67-ospf-authored-native-r2.log/run-35oa1lvn after task-default/cost-evidence polish. Actual review caught unsupported arrow glyphs in cost text; replaced them with plain words/ASCII and explicitly labelled applied settings. Twelve final wide100/light150/dark200 legibility captures pass m67-plain-cost-labels-native-r1.log/run-nd6f773j; root reviewed actual dark100/200 and light150 views, including centered focused text, responsive columns and readable applied costs. Final53Python tool checks,28-theme provenance and102-lab coverage integrity pass. Earlier arrow images/failures retained.


## 2026-10-23 — M68: integrated multi-unit configuration and closed terminal foundation

- Integrated independently reviewed workspace/progress hooks and bounded systemctl/journalctl implementation. Saved text, loaded definitions, successful start jobs, current program initialization and future-session links remain separate. The journal retains typed historical events only; neither a message nor command exit grants current success. Requested job status correctly permits a weak Wants failure; effective direct properties include installed/inverse/target relations without claiming transitive dependencies.
- Root main138 focused library/core/workspace/terminal/integration/OSPF-content/backup/shell-migration checks pass, followed by all-target Clippy with warnings denied. The first command used a nonexistent shell_migration test target and failed before compilation; corrected target shell_substitution_migration and retained the original diagnostic. Isolated independent130-check/Clippy gate also passed; these overlap, not268distinct checks. Exact root evidence m68-manager-terminal-main-r1.log and m68-manager-main-clippy-r1.log.
- No normal unit-manager activity/native renderer yet, no new objective credit or scenario count. Catalog remains102/24 (31/21/13/37). Original Service/7-evaluator/progress and28themes remain. A native editor/terminal proposal is source-only; its acceptance and authored lessons follow capability review. Most completion requirements remain Open.
- All heavy work used the shared lock,2Cargo jobs/2runtime threads and disk artifacts. Root ordinary focused run37samples minimum9.68GiB available, peak some/full PSIavg10 0.18%, observed OOMdelta0. Swap remains nearly full; no guarantee, stress workload or system changes. No public release.


### M68 installed follow-through

Clean29096be/102labs/24units passes actual installed identified877upgrade,uninstall/reinstall,outside-checkout identity/content/hash/desktop and exact separate synthetic original-nine bytes. Installed all-four OSPF flows pass84captures; shared IPv4/VLAN alignment passes72views at100/150/200 wide/narrow/light/dark, with representative root visual review. Ten quiet default-Wayland launches pass10/10. INSTALLATION.md records exact hashes/paths/scope. No startup-cause cure, novice/reader/physical-device/other-platform claim, release or full-product completion.

CI40049251314 stopped at formatter because the cherry-picked lib.rs module declarations needed ordering; local fmt reproduced this one-line ordering oversight, corrected it and passes m68-main-fmt-r2.log. Original failure logs retained. This is formatting only; the tested Fedora artifact stays identified29096be. Remote rerun follows the correction; remote tests were not run in the failed pipeline. Continue service native/editor validation and actual before7 capture before any case/evaluator change.


Before-case public7 capture: first guarded compile passes; first actual capture stops on a harness assertion that treated657/62 fixed-input Case placeholder fields as expected outputs. The evaluator and original inputs were unchanged. Preserve the partial observations and full original binary, without MANIFEST/COMPLETE or accepted baseline claim. Correct only the harness to record genuine outcomes and identify optional expectation-match metadata as non-authoritative; authored assignment/example acceptance and slice/source/hash/resource checks stay mandatory. No evaluator8 or new shell curriculum is accepted.


## 2026-10-23 — M69: native multi-unit workspace integration

- Integrated reviewed isolated service editor/terminal checkpoint as8b902b9, preserving both OSPF and UnitManager dispatch/help through three explicit merge resolutions. Optional bounded command sessions and active editor kind preserve separate assignment/example work, failed command drafts, history and old metadata absence. No normal manager curriculum yet;102labs/24units and coverage counts stay unchanged.
- Main750library/integration checks pass. The full UI run passed114/115 and caught the finalM67 long topology heading overflowing the minimum720-pixel viewport at200% text in an ECMP example. Shortened only the heading to Links and costs · applied; all8focused OSPF checks now pass, supplying115distinct UIchecks combined/865withmodels. Did not weaken bounds or shrink fonts. First failed all-target log and corrected focused run retained (m69-native-main-all-r1.log, m69-ospf-short-heading-r1.log, m69-main-model-integrations-r1.log).
- Independent first workspace gate143checks/Clippy/fmt pass. Later actual-input inspector/native plan source fc37b38 passed7focusedchecks and finalClippy/fmt after2style-only let-chain fixes. It remains to merge/run actualFedora native; neither source tests nor renderer metadata are novice/screen-reader/other-OS acceptance.
- Actual public7 before-case baseline now accepted:4124genuine observations,37runs/215outcomes/37examples,33exacthistorical6records/182outcomes, both102-attempt raw+realwriter profiles, separate pendingcontexts,4actualStop observations. Root independently verifies COMPLETE9a924e… andall14filehashes; accepted copy is ignored project-disk evidence, not learner progress. Fixed reference vectors contain placeholder Case expectations; r1's mistaken acceptance assertion failed and remains preserved. R2 records their actual results with explicitly non-authoritative match metadata, preserving authored/example/slice/source/resource gates. No8orBashoracle/curriculum claim.


### M69 native follow-through

- Inspector/private input harness merged as07c878a, retaining both oldOSPF smoke ranges and new12000service fixture through two explicit stage-dispatch merge resolutions. Main7focused checks, all-targetClippy/fmt and native build pass. Combined current main750library/integration+117UI checks pass (867distinct; repeated focused/core checks not added).
- Actual Fedora native private fixture passes17captures in m69-manager-native-r1.log/run-erni39j9. Real editor/picker/Save, terminal Run/Enter, failure/partial/reference/Before alternative, query retention, loaded-versus-current inputs, fresh/current invalidation, exact isolated drafts/Session/editor-kind resume and actual Reset/Cancel all pass. Root inspected actual renders. Light150/Dark200 Check/layout/scroll pass; native larger-scale typing remains an authored-unit-flow gate, not claimed here. No normal service-unit lesson/count/coverage or novice/physical/reader/other-OS claim.
- Learner feedback now says previous run/saved settings/stop and start, removing the unavailable internal initialize instruction.65relevant core/adapter/terminal/session checks pass afterward; the final native flow uses these strings. No predicate, parser, progress schema or shell evaluator change.
- Safe serial main broad-run363samples minimum9.58GiB available, peakPSI0.72%,OOMdelta0; native198samples minimum12.66GiB,PSIzero/OOMdelta0. Swap reached0free. Ordinary observations only, no safety guarantee or system-setting change. Exact ignored m69-memory-r1.json audit retained. Continue12authored service-content tests/review/normal native and fixed37-case Bash oracle before any case/evaluator8 changes.


## 2026-10-23 — M70: four normal service recovery assignments

- Integrated reviewed content c1c8e4b and native guide/input-plan54b8faa into normal catalog106/25units:31CCNA/21LPIC-1/17LPIC-2/37Bash. The unit progresses through saved/loaded/running repair, required preparation and real ordering, deliberately failed optional Wants, and a preserved multi-target handover. Introductions precede assessment; example names/values/state/drafts are separate, Insert never executes, and Challenge keeps the full task. No old scenario IDs/goals, themes, VM architecture or learner-host execution changed.
- Actual native r1 caught an important recovery assumption: an early start left preparation active with old input. The model was correct; start must not silently rerun active RemainAfterExit work. Added explicit stop/restart teaching and native actions plus a14th content regression proving stale snapshot failure and successful recovery/alternatives. No predicate was weakened. Failed r1 remains retained. Render review also fixed missing spaces in port/task prose and a developer-oriented topic label.
- Current771library/integration checks across74targets and120UI checks pass891distinct combined, Clippy/fmt/build pass. Coverage106 and all28theme-source/palette checks pass; existing fonts/shared metrics still center actual text/placeholders/cursors, and equivalent configuration columns retain responsive layout. No repeated focused checks added to the distinct total.
- **Actual normal all-four Fedora native PASS72captures:** m70-normal-service-native-r2.log, run-pejyy86y. Native menus/typing/Save/terminal/Check prove wrong/partial/reference/alternate/retry, fill-only practice/experiment/repair, stale-success invalidation, query retention, exact separate draft/Session/editor-kind resume/Undo, selected reset/cancel and Challenge. Each task actually edits/saves/retries at150%light and200%dark; root visually reviewed representative intro/full task/current completion/light feedback/dark200 screens. Default window was1268×632 under the existing window manager; layouts scroll/stack without font shrinking.
- Official current LPIC-2v4.5 sources checked10-23. Strengthened existing202.1/K1/T4 Partial rows only;128Partial/1Supporting/1111Open overlap references unchanged. Raw input snapshots conservatively compare text including comments; this limit, fixed fictional programs and lack of real boot/SysV/readiness are explicit. Most full-product requirements remain Open. Actual novice, physical keyboard/device, screen-reader and Windows/macOS desktop acceptance are not inferred from injected events or CI. Latest installed candidate remains separately identified; no release publication.

## 2026-10-23 — M71: bounded history readers and fixed Bash reference gate

- Reviewed reader d558de2 is main b16008d: decode at most32 typed history/receipt elements, refuse the next before its record/strings and never reserve an untrusted sequence hint. Combined-history policy, versions, exact existing serialized bytes and target-driven migration writes are unchanged.80scoped checks/Clippy/fmt pass and the current891main gate includes affected regressions. Combined primary+more validation can still decode33bounded records before refusal; no stronger allocation claim.
- Fixed development oracle is main61888a3: exact37 authored builtin-only inputs, closed C environment/no HOME/empty PATH, bounded pipes/files/time, inherited actual guard group/lock and no-clobber retained evidence. All73Python checks pass, including20mock/pipe tests; actual fixed Bash5.3.9 observations complete with manifest7af1b2e86a9cf9364c5ea170bcdfba4142a5c8cff0c5046efd0be57c55af02bd. Root and peer independently hash all157outputs/37rows/pins. Original input null/NOT RUN fields intentionally remain immutable; observations are separate. SHELL_CASE_REFERENCE.md records actual statuses/keywords/quote/locale decisions. No app runtime calls Bash.
- Isolated case implementation27d0a0e passed50focused checks; independent review then caught array-data delimiter, locale and diagnostic gaps. Corrected65da217 passes55focused checks/Clippy/fmt. These are isolated capability checks, not main8 activation, old-outcome compatibility, migration, native acceptance or new curriculum. Those gates continue under CHECKPOINT.md; main stays evaluator7.


## 2026-10-23 — M73: isolate build caches as well as build concurrency

- A guarded shell comparison failed before creating any output: its pinned source contained 102 labs, but Cargo linked the 106-lab library previously compiled from another worktree in the shared target. The catalog guard caught the mismatch. This was serial work under the memory lock; serial execution alone does not establish artifact provenance. Failed diagnostics and the old target are retained.
- The runner now reserves main `target/` for main and selects a stable path-hashed target for each canonical worktree. It rejects cross-checkout target overrides and target-namespace symlinks. Direct and runner-driven candidate builds require main, protecting its separate packaging cache. No old files are removed and no global settings change.
- All 76 Python checks pass, including new wrong-owner, equal-basename, canonical-alias, target-symlink and candidate-worktree cases. Independent source review found no material blocker; documentation distinguishes harmless requests resolving to the same target from aliases of the target directory itself. Cargo remains two jobs, tests two runtime threads, and one heavy invocation spans all targets/worktrees.
- The ordinary failed attempt is not accepted compatibility evidence. A fresh dedicated-target comparison rebuild is in progress; no result is invented. The previous observed 810-check merged-model run is retained but qualified rather than presented as clean-cache identity proof. Normal app content/progress/evaluator and all platform acceptance limits remain unchanged.


### M73 safe stop requested by owner

- The corrected comparison rebuilt in an exclusive worktree target and passed. Root independently verified all16 output hashes and4,124 outcomes, with3,968 unchanged and156 enumerated new-support/diagnostic changes. All37 existing script records/results and separate examples retain their behavior. Genuine earlier profiles are byte-identical. Stop tick metadata is now measured; the inaccurate prior capture and failed wrong-library attempt remain untouched.
- The isolated merged106/evaluator8 branch passes a clean-target810 model/integration checks plus121 UI checks (931 distinct combined). The UI run took434.36seconds. First UI compilation exposed a test-only observer API assumption; the corrected app-only probe observes a virtual child marker without engine/API/budget changes. The original failure remains. Final Clippy/build and the actual new native7-to8 flow are still pending; evaluator8 is not enabled on main.
- At the owner's stop request, the current UI invocation completed normally, both helpers became idle, and root verified no active/unfinished command marker and a free project lock. The held editor focus-band correction remains unapplied. A precise20-file worktree receipt and patch preserve the staged work. No further build/native/curriculum batch started.
- Three new scripting tasks remain unbundled source only. Review caught an incorrect default-read whitespace preservation claim; its correction had not started at stop. A private MAC-learning foundation also remains uncompiled and requires the recorded physical-health, diagnostic-size, JSON-input and tagged-arrival review fixes. None counts as implemented curriculum. Main retains106 labs, evaluator7 and the unchanged identified b73 development executable; the build-cache safeguard ab4d990 is pushed. CHECKPOINT.md records exact continuation steps and acceptance limits.


## 2026-10-24 — M75: reviewed case selection and genuine progress upgrade

- Added the bounded, cooperative original `case` parser/evaluator after fixed primary-reference observations and independent review. First-match, quote-aware scalar patterns, lazy reached alternatives, status, nested captures and bounded iterative matching share existing execution/memory/output limits. Unsupported forms fail honestly. No dependency, learner-host execution, existing assignment requirement or theme changed;106 labs/25 units remain.
- Actual clean-target before/after evidence verifies4,124 outcomes:3,968 unchanged and156 explicitly listed new-support/diagnostic changes. All37 existing authored script records/215 results and37 examples preserve behavior; genuine6/7 data retain exact bytes/revisions. Four stopped-run tick metadata fields were corrected in a separate retained capture, not rewritten as historical evidence. The shared-target wrong-library failure led to the already pushed exclusive-target safeguard.
- Clean isolated merged regression passes810 model/integration plus121 UI checks (931 distinct). Final affected plan check, Clippy with warnings denied, formatting and binary build pass. Nine constant feature-floor assertions required lint-only compile-time conversion, preserving predicates/messages; three initial lint failures remain. No full-suite reruns or focused repeats are added to the distinct count.
- Actual Fedora native final flow passes15 captures in run-_z5iwydb. Typed case8 solution, reached-child Stop, retry, earlier6/7 history, separate example, exact first backup/save-readback, real export/restore Cancel+Confirm and safety copy, selected Reset and Challenge pass. Root reviewed actual150%light/200%dark syntax diagnostics, completion results, history and restore. First behavioral pass photographed only editors at larger sizes; measurement-only hooks then exposed the existing result widgets for actual review. Layout/controls/grading are unchanged. Short1268×632 tiled windows need nested scrolling; lower retry reminders may need additional output scrolling. These are not physical-device keyboard, screen-reader, novice, cold-process restart or other-OS acceptance.
- Three reviewed source-only scripting tasks retain31 independently specified behavior cases and separate examples; they are not bundled or counted until real simulator/native checks. Corrected read trimming and explicit canonical-source guarantees keep the teaching honest. No certification row is promoted merely because a language capability exists. Linux/macOS packaging and broader curriculum gaps remain as previously recorded; no release publication.


## 2026-10-24 — M76: Ethernet learning capability before dependent content

- Added an original bounded pure-Rust MAC-learning model over the existing VLAN forest, with source-to-ingress learning, local-VLAN destination lookup, unknown/broadcast flooding, same-ingress filtering, source-only aging, selective clear and transactional configuration. Physical arrivals and modeled untagged endpoint acceptance are distinct. Independent cloned current-state probes accept equivalent relearning and invalidate after clear/expiry. This is a library foundation, not a new playable activity or coverage promotion.
- Independent review caught a disabled-port shortcut in negative grading, retained allocation after diagnostic truncation, compact-versus-pretty JSON bounds and tagged-arrival labeling. Corrected all four and added adverse-path tests. An edit accidentally removed the unchanged one-day advance constant; source review caught it and it was restored before first compile. No expected-outcome weakening.
- Eighteen new focused checks pass (one private capacity check and17 integration checks), all-target Clippy with warnings denied and formatting pass. The first filtered invocation ran only the helper; the separate unfiltered integration invocation ran all17. No filtered/skipped or repeated tests are counted as passes. All work used the shared guard, two jobs/two runtime threads and an exclusive disk target.
- Current catalog remains106/25 and objective statuses128Partial/1Supporting/1111Open. Gap descriptions for CCNA1.13/a–d now distinguish implemented core from pending native teaching/practice; fingerprints and Open statuses are preserved. Cisco sources rechecked10-24. Native adapter/workspace source is being built separately; no kernel/network/host process or dependency added. MAC learning is not a security boundary, physical-equipment experience or full IOS/STP/ARP implementation.


## 2026-10-24 — M77: case decisions used in substantial scripts

- Added three distinct authored assignments after the accepted evaluator8 capability: exact report-mode/help/error interfaces, ordered record routing with a continued rejection audit, and whole-plan validation before preview/apply publication. They add31 deterministic output/status/virtual-file cases, separate editable museum examples, concrete experiments/repairs, progressive help and conditional alternatives. No exact-source requirement, new dependency, host execution or new theme. Catalog109/26:31CCNA,21LPIC-1,17LPIC-2,40Bash.
- Independent contract-table calculations and source review caught default-IFS trimming assumptions before execution. Canonical record inputs now explicitly exclude edge spaces/tabs while preserving internal spaces/backslashes; source filenames are guaranteed distinct from outputs. Scope and author-data guards accompany the teaching. This does not implement custom IFS or real filesystem alias/crash-atomic guarantees.
- Isolated normal109 regression passes818 library/integration plus122 UI checks (940 distinct). All-target Clippy, formatting and build pass. Current-catalog maintenance fixes preserve frozen102/106 identity fixtures and genuine6/7 data; the older37 shared current-review rows are unchanged and three new rows appended. Focused repeats are not added to the distinct total.
- Actual native all-three flow passes in m77-case-lessons-native-r2.log, screenshots run-8ja5d440. It exercises introduction, separate ready-only example, explicit Run, experiment/repair, full task, wrong/partial feedback, reference and if/loop alternative, edit invalidation/recheck, actual Challenge selection, typing/Run at light150% and dark200%, exact save/readback, unfinished draft, favorites and selected reset. Root reviewed actual screens: specific retry advice and expected/actual output or file differences are readable; short tiled1268×632 windows require document/output scrolling. This is not a novice study, physical-device keyboard/screen-reader check, cold-process restart or other-OS acceptance.
- First native run exposed a deadlock in new capture-only geometry code: reading zoom inside a context-data lock reentered the egui lock. Moved that read before the lock; independent review, focused plan, build, actual R2 and final Clippy pass. Failed R1 diagnostics remain. Product layout/grading were not weakened. Existing models, drafts, favorites and examples retain their behavior.
- Combined main passes836 model/integration checks across78 targets, including the18 pure MAC checks, after merging both capabilities. The accepted109 UI source is unchanged. Coverage integrity validates109 and unchanged reference statuses.
- Only already Partial LPIC-1 105.2/K3 evidence is strengthened; no status/fingerprint promotion. Core MAC learning is independently tested but native MAC instruction/practice remains unaccepted. Candidate packaging and complete certification coverage remain open; no release publication.


## 2026-10-24 — M77 delivery follow-through

- Pushed the109/26 unit and accurate coverage reporting. Built identified normal5a968c7 and a clean Fedora candidate with all223 applicable notice/source packages. Actual102 upgrade, uninstall/reinstall, outside-checkout identity/content/desktop and exact original-nine synthetic progress retention pass. Exact launch/hash inventory is in INSTALLATION.md.
- The installed candidate passes all-three new scripting native workflows (45 captures),72 shared IPv4/VLAN alignment views at100/150/200 light/dark wide/narrow and10/10 quiet default-Wayland starts. Root reviewed actual instructions/examples/assignments/feedback and centered field/column/focus renders. Private receipts preserve exact hashes and ordinary memory observations; no OOM or sustained pressure occurred. Not a memory guarantee, novice study, physical-device/screen-reader or cold-process reopen result. The old Wayland callback cause remains unresolved.
- Full-tree formatting caught one line-wrap omission after the capture-lock correction. Fixed only that wrapping in pushedcee0432; local full-tree fmt passes. Native CI40101334715 at5a stopped onlyFormat on all three OS runners; corrected40101440051 passes Ubuntu; Windows/macOS hit the30-minute job limit in tests/release compile. The proposed60-minute limit retains every check and two workers and has no desktop-acceptance implication. No candidate is relabeled to the later source.
- The new MAC workspace separately passes10backend+7UI/plan checks and Clippy/fmt/build. Actual first native flow passes; rendered review caught and corrected a misleading inherited Check-applies instruction for its separate Apply settings flow, and compacted repeated example-action controls. Actual final R2 render acceptance passes17captures and the workspace is integrated in ea7c6b1. Authored content still follows; catalog and coverage counts do not change for that fixture.


## 2026-10-24 — M78: native MAC workspace before authored coverage

- Integrated workspaceaac951f through main ea7c6b1: native source/destination sends, current learned table, optional configuration/time/clear/trace, explicit Apply versus Check, saved malformed drafts and independent Introduction/practice/assignment. Closed typed pack-only actions validate reachability without teaching invented IOS commands. Profile identities/example origin are pinned, wrong-kind/oversized data rejected, and current grading accepts genuine alternate outcomes. No shell revision, dependency, theme, original scenario or desktop setting changed.
- Seventeen distinct new workspace checks pass (10backend,6UI and1native-plan), plus three affected existing VLAN UI checks; prior18pure-core checks remain separate. Clippy denying warnings, formatting and build pass. First compilation caught test-only nonexistent API/imports; modal diagnostics showed current responses disabled while post-pass getters returned old data. Tests now assert actual measured Response flags plus unchanged keyboard-driven state, retaining all safety assertions. Failed sources/logs remain.
- Actual native final R2 passes17 captures inrun-xrywx5uv. Root reviewed separate insert-without-execution, practice expiry/repair, full task, malformed/partial/current/reverse-order outcomes, actual150/200% editing and invalidation, reset/readback/Challenge and readable errors/completion. First render review caught a misleading inherited Check-applies message and repeated action caption; explicit contextual help and adjacent selector/Insert fix those without changing old VLAN wording/IDs. Short tiled windows scroll; this is not cold restart, novice, physical-device, screen-reader or other-platform acceptance.
- Foundation only: normal catalog109/26 and128Partial/1Supporting/1111Open stay unchanged. Four distinct authored MAC assignments follow separately; no fixture or numeric-variant inflation. Combined-main follow-through still follows the completed installed-candidate run.


## 2026-10-24 — M79: exact native restart verification, acceptance still open

- Integrated a guarded two-process native development probe. Both children use the
  normal profile lock, progress load, preference load and save paths against a
  marked private disk profile, with bounded process/output handling. Raw saved
  bytes, full JSON, portable progress and loaded migration receipts are compared
  separately; portable export intentionally omits those receipts. No personal
  profile, host learner execution, default application behavior or grade is changed.
- Current97 Python guard tests and13 Rust controller/serialization tests, all-target
  Clippy and build pass. Native R1 found a post-launch controller assertion; R2
  found an incorrectly equated portable/raw projection. Failed evidence remains.
  Actual R3 writer and new-process reader pass exact raw/decoded receipt, draft,
  favorite and appearance comparisons. Later Back navigation hits a fractional
  visibility assertion at150%; full end-to-end acceptance is still open. Eight
  captures and raw snapshots remain; no COMPLETE receipt is fabricated.
- The pending controller-only correction must retain real center visibility and
  offscreen rejection. Repeat on a fresh profile and later the installed candidate.
  None of these gates is a novice, screen-reader or other-platform runtime study.


### M79 follow-through: actual two-process Fedora flow accepted

- Controller geometry correctionb8c0175 allows at most one physical pixel at a
  rounded clip edge and requires the click center inside the real viewport.
  Four scale/offscreen tests retain bounded scrolling/failure and no queued
  offscreen click; all17 targeted tests, Clippy/fmt/build pass. Product layout,
  profile guards and all exact raw/decoded assertions are unchanged.
- Actual strict native R4 passes: writer exits0, a different reader process
  reacquires the normal profile lock, loads exact records/drafts/favorite/theme/
  scale and completes real navigation. Nine captures and full manifest63a655f3…
  remain in ignored tmp/native-reopen-m79-worktree-r4. Root reviewed actual
  resumed library, independent practice and current completion. Earlier failed
  runs remain intact. Repeat on the next installed candidate; this worktree
  binary is not relabeled as the prior installed5a candidate.
- Cross-platform run40111060414 stopped at a merge-only line-wrap omission.
  Corrected formatting-onlyfec8cfe; no runtime check was disabled. Other-platform
  desktop, physical-device/screen-reader and novice acceptance remain pending.


## 2026-10-24 — M81–M84: reviewed teaching data and bounded Linux extensions

- Four distinct Ethernet assignments are registered in normal source113/27
  (35CCNA/21LPIC-1/17LPIC-2/40Bash), after VLAN foundations and before static
  routing. Their9 content checks accept reference and alternate plans, reject
  misconceptions and partial work, independently break each requirement, isolate
  practice and retain real6/7 history/drafts/favorites. Early failures were test
  catalog loading and stale draft assumptions; no state predicate was weakened.
  The normal merged branch passes857 model/integration checks and Clippy/fmt.
- Native R1 passes first-three full flows and capstone practice/partial work,
  but correctly leaves capstone incomplete after its controller misses the last
  item in a clipped source dropdown. All37 requirements stay intact. Actual
  screenshots reveal two smaller clarity/alignment issues: distinguish rejected
  Send from a still-correct configuration, and align dropdown labels with the
  shared field column. Fixes and full native acceptance remain pending. No new
  authored coverage is promoted yet; the installed5a candidate still has109 labs.
- Original PTR capability adds no dependency, host lookup, new authored activity
  or objective promotion. Twelve new tests plus77 existing library/DNS checks
  and all-target Clippy/fmt pass; independent review rechecked RFC1035/3596/2181.
  The main combined869 model/integration checks pass before the find merge.
  Six forward-DNS limitations texts now accurately describe editor PTR support;
  all other scenario JSON values are identical. Their8 content tests and9 MAC
  course-order/content tests pass after those metadata-only edits.
- Bounded virtual find core697754e passes42 scoped checks (12new,30existing),
  all-target Clippy and formatting, with no host reference execution or dependent
  lessons yet. Read-only traversal preserves Tree serialization, reports partial
  permission failures honestly and discloses the readable/nonsearchable boundary.
  The fixed GNU comparison remains a separate gate; all152 existing fixture
  commands, including the original79, are preserved. See FIND_SUBSET.md and
  DNS_REVERSE_SUBSET.md for exact scope and remaining teaching work.
- Ordinary serialized root model checks observe at least21.33GiB available,
  PSIavg10 zero and no OOM counter increase. Two jobs/two test threads and disk
  artifacts remain mandatory; these observations are not a stress test or safety
  guarantee. No release, desktop setting or unrelated application is changed.


### M84 reference and delivery follow-through

- Actual rebuilt MAIN GNU comparison R2 passes152 existing commands and32 new
  find observations (31 matched path/status/error comparisons, one explicitly
  unsupported traversal boundary). The first run is retained. It exposed an
  overlarge maxdepth range; the model now rejects2147483648 and4294967295 before
  lookup and accepts2147483647. Btrfs directory link counts are explicitly outside
  equality alongside directory sizes; regular-file link counts/sizes, modes,
  types and names remain checked. Source/ELF/GNU pins and before/after metadata
  are in manifest67ee7757… under filesystem-find-reference-d2vakr7o.
- The strict synthetic reopen fixtures now create their required artifact-parent
  shape independently of CI TMPDIR and canonicalize only a known fresh fixture
  base on macOS. Production containment is byte-identical.112 Python checks,
  12 find checks and17 reopen/geometry checks, Clippy/fmt pass before integration.
- Windows CI40111561758 atc41352a passes and its109/26 candidate is downloaded.
  Archive, PE and all25 embedded scenario/unit source hashes agree, along with
  the Rust standard-library notice and137-package inventory. Actual Windows
  desktop acceptance/signing is not claimed. Ubuntu/macOS CI had the fixture
  failures above; fresh corrected-source CI is still required.
- Actual MAC R2 again passes first-three full tasks but catches a stricter popup
  controller issue in the capstone example: an inherited paint clip is wider
  than the actual menu, directing its wheel outside the popup. Correct actual
  content bounds and a genuine off-center ComboBox regression are pending;
  no requirement or grade is weakened. Render-only layout measurement separately
  reentered a Context lock; its one-file correction is distinct from the old
  unanswered Wayland callback and awaits actual verification.


### M86 integrated Ethernet teaching and capability follow-through

- Normal catalog113/27:35CCNA,21LPIC-1,17LPIC-2,40Bash. Four substantive Ethernet
  assignments add narrow current1.13/a–d evidence, not full-objective coverage.
  Main889 models/integrations pass across83 targets. Whole UI run159pass/2fail
  exposes two old109 current-catalog assertions; correction and scoped rerun are
  recorded separately. Genuine historical fixtures remain unchanged.
- Corrected native capstone exact-plan suffix passes15 captures; render-only
  matrix passes24 fullscreen/windowed100/150/200 light/dark views. First-three
  full flows are accepted separately. Earlier failed full runs remain; complete
  installed-candidate all-four acceptance is still required. See MAC_LESSONS.md.
- Shared label widths, actual popup content bounds and capture Context discipline
  preserve full tasks and meaningful tools. One missing-glyph prose fix changes
  no grade/state/action. Legacy-three teaching claims are corrected rather than
  silently treating older seeds as meeting the newer introduction pattern.
- Typed exact DirectoryEntries grading is integrated after8new+56affected checks.
  Three Find lessons remain source-only, unregistered. PTR and Find capabilities
  alone gain no authored curriculum credit. Packaging/platform acceptance and
  the substantial remaining curriculum stay open.
- Follow-through:26/26 native-plan checks now pass, including both failures.
  All-target Clippy/fmt,112 Python guards, coverage and palette provenance pass.
  The original failed full UI run is retained;159 unaffected plus the corrected
  two give161 distinct UI checks, without counting overlapping reruns as new.
- Actual clean113/27 Fedora candidate73777e0 now passes install/109upgrade/clean
  uninstall-reinstall/outside-checkout verification and exact synthetic original-nine
  retention. Installed full four MAC flows57captures, shared alignment72views,
  quiet10/10 and strict two-process cold reopening9captures all pass. Root reviewed
  actual renders. Exact identities and limitations are in INSTALLATION.md.
  Fresh three-platform CI40120315031 is running; no release is published.


### M87 honest current credit for older configuration editors

- Older DNS and data-volume workspaces now treat explicitly changed unsaved
  fields as pending. Their completion panel/library current credit cannot borrow
  success from an older applied value. Missing optional legacy drafts preserve
  applied-state credit; reverting a draft restores it. Save remains explicit,
  and malformed Save never mutates the applied model.
- Full893 model/integration checks across84 targets pass, including4 new tests
  for blank/malformed/different drafts, saved wrong state, exact read-only resume
  and unrelated-attempt preservation. The extended existing actual-egui editor
  regression passes Save/reject/reopen/revert/completion checks. Clippy/fmt pass.
  One development UI invocation had a mistyped build-revision environment value;
  it was rejected as identity evidence and rerun with the exact observed revision.
  The separately identified73777e0 installation candidate is unaffected.
- No scenario, Goal, model State, progress schema, history, course count or
  dependency changes. The model gate observes minimum22.46GiB available,
  peak PSIavg10 at0.18%, no OOM counter increase. Two workers and one shared
  heavy-work lock remain mandatory. This source fix is not inside73777e0.
- Independent read-only source review finds no material issue: only explicit
  drafts invalidate, comparisons match the existing editors, missing legacy
  values retain credit and wrong applied values still fail the original goals.


### M88–M89 source and integration work in progress

- Three distinct Find-content proposals pass55 targeted checks/Clippy before
  registration. Normal116 helper integration then passes903 model checks/85
  targets; main still113 pending integration. Exact inventory, owner500/700
  alternatives, partial search and scoped private handover are actual outcomes,
  not find-command history credentials. The first backup-inspect test failure
  is retained; its exact expected projection changes only the separately rerun
  practice result/revision, retaining all other fields and genuine6/7 history.
- Client teaching d403077 integrates through7809b5f after70 scoped gates. No count,
  model State/Goal/version/evaluator/dependency change. Main combined/native gates
  remain pending. Root's independent source review added supplied-context pinning
  and font-derived Run sizing before acceptance. See CLIENT_TEACHING.md.
- CI40120315031 finishes Ubuntu PASS and macOS tests/release compile PASS. Windows
  has one developer-fixture failure; no actual desktop failure is inferred.
  Rust1.99.1 documents Windows canonicalize extended-length paths and verbatim
  separator differences: https://doc.rust-lang.org/std/fs/fn.canonicalize.html
  and https://doc.rust-lang.org/std/path/enum.Prefix.html (checked2026-10-24).
  Root prepares test-only fresh-fixture correction with all assertions intact;
  production reopen prefix remains exact28148b37…. New Windows gate is pending.
- Find actual-egui Enter reproduction catches a pending practice command running
  while reset confirmation is open. R1 retains4pass/1fail; narrow fix/regression
  underway before native acceptance. Native controller/phase and font-fit checks
  are not a novice, physical keyboard, screen-reader or other-platform study.
- Test-only Windows fixture correction passes the focused Fedora regression;
  production-prefix bytes remain identical. MAIN merged client/DNS/Mount/backup
  checks pass22; Windows execution of the correction still needs fresh CI.


### M90 combined integration in progress

- Main integrates both reviewed new teaching flows and normal116/28 catalog,
 35CCNA/24LPIC-1/17LPIC-2/40Bash. First installed116 candidate remains pending;
  the accepted113 package retains its exact identity. No added curriculum credit
  is inferred from library count or new native controllers.
- Combined model R1 passes913/914. Its sole failure is an IPv6 test fixture
  unintentionally retaining newly added incompatible client teaching. Clearing
  only that fixture metadata plus explicitly testing incompatible rejection
  preserves validation. R2 passes9IPv6/10Find/11client;914distinct model checks
  accounted without relabeling the original failed run. Full combined UI running.
- Plain unit-summary/permission-number spacing changes preserve every Find
  initial/goal/version/reference/practice/outcome/command value. Frozen113/109/
 106/genuine6/7 sources remain unchanged. Independent Find/LPI review approves
  only narrow103.3.K5 promotion after native acceptance;104.7 stays Open.

- Combined UI R1 finishes171pass/2fail: the new client's two-row diagram and
  redundant gateway label push Check to y927–962 at1200×850, so its grading-test
  click is also ignored. The shared one-row/stacked chain with gateway in its
  client card fixes both unchanged original-nine regressions. All7 focused
  client checks pass, including editing/Undo/modal safety and painted text-fit.
  No fonts, field IDs, model or goal changes.173distinct UI checks accounted;
  R1 is retained, never relabeled all-green. Independent source review approves.
- Final Clippy/fmt,112 workflow guards, coverage and pinned palette provenance
  pass. Main914 distinct models and173 UI are technical gates, not novice or
  platform acceptance. First actual Find3/client runs remain pending on the next
  clean installed candidate; existing113 packaged evidence stays separate.


### M91 installed116 review and shared opening correction in progress

- Clean1608a18 candidate116/28 is built/installed. Identified113 upgrade,
  uninstall/reinstall and exact original-nine synthetic bytes pass. Quiet Wayland
 10/10 and one bounded protocol probe pass; historical callback cause unresolved.
- Actual installed Find3 flow passes46 captures: separate teaching/practice/task,
  wrong/partial/reference/manual alternatives, current invalidation, exact raw
  save/readback, reset/cancel/Challenge and Light150/Dark200 native typing. These
  functional results do not by themselves establish visual/novice acceptance.
- Screenshot review catches opening auto-scroll past the task and growing blank
  terminal space as outer scroll changes screen coordinates. Narrow shared fix
  keeps view opening at the task and sizes terminal relative to unscrolled content.
  The real egui opening/scroll regression passes100/150/200; full174UI rerun
  passes631.80s. Independent source review accepts coordinate/focus behavior.
  The guarded run observes minimum24.00GiB available, PSIavg10some/full0%, OOM
  delta0; this is an ordinary observation, not a memory guarantee.
  No model, field alignment, font, grading, progress schema or dependency changes.
  Installed1608 artifacts remain immutable; corrected screens require a new build.
- Curriculum overview corrects stale blanket OSPF absence. Existing four bounded
  configuration/route-reasoning labs do not model a neighbor FSM, LSA exchange,
  convergence or broadcast elections. Narrow Find coverage promotion remains
  withheld until visual acceptance; matrix stays133Partial/1Supporting/1106Open.
- Original DNS/Mount teaching adapters remain source-only under independent
  review;21 prospective tests are unrun. Getopts is a researched capability
  contract only, not an implemented builtin or authored curriculum. Source-only
  parallel review never launches a second heavy command.


### M91 safe pause — owner request, 2026-10-24

- Code milestone deb55b92a2407a0068b8ba83dff9b71568562893 is pushed. Full174 UI,
  all-target Clippy/fmt/diff pass. Source models/Goals/evaluator remain unchanged.
- Clean deb55 Fedora116/28 candidate built; actual identified1608 upgrade,
  outside-checkout identity/content/hash/desktop, uninstall/reinstall and exact
  original-nine synthetic bytes pass. ArchiveSHA256
  c12be61f95c506a455dcc534e533894fa0700ce1600fd9fb54c15bf94aa033df;
  binarySHA256f3e6bd5110a1962b590bd1236e59abb7b24179e7f4b0d2e8adc45d09c21000f5.
- Exact installed client full flow23 captures (run-aeptboln) and Find3 full flow46
  (run-2z9qjy5k) PASS. Root inspected actual task opening, compact topology,
  feedback and Light150/Dark200 fields. Before run-9i7bn02d/after run-2z9qjy5k
  show task restored at the top and stable smaller terminal. Wrong/partial/
  alternative/current-invalidation, separate drafts, save/readback/resume,
  selected reset/cancel and Challenge exercised with actual native injected input.
- New deb55 shared72-view alignment, repeated quiet startup and strict two-process
  cold reopen remain pending. Earlier113 acceptance and1608 quiet10/10/protocol
  evidence stay separately identified. Historical callback cause unresolved.
- CI40128237589 at1608a18 now passes all3 native runners. Unsigned Windows116
  archive retained and independently verified:27 embedded source packs exact,
 137 dependency packages,391 notice/source hashes and Rust standard-library
  notice. No Windows/macOS desktop execution; deb55's later UI fix is not inside
  that archive. macOS compile passed, packaging still notice-gated.
- Source-reviewed legacy teaching R4 (17paths/21unrun checks) and NoAnswer R1
  (4paths/12unrun) remain frozen in private worktrees/archived snapshots. Getopts
  remains research plus50 proposed cases; its clean worktree has no implementation.
  None is counted as delivered content or objective coverage. Matrix remains
 133Partial/1Supporting/1106Open; narrow Find reporting update is pending.
- Owner requested pause. The already-running finite Find check finished and was
  reaped; both helpers stopped, no active/unfinished command markers remain, and
  no new heavy command launched after the request. Main reporting checkpoint is
  docs/CHECKPOINT.md. Private receipt m91-paused-status SHA256
 39665eec28bd96f1bf1d051f529d3cf8e0780abe849dbfb50ce667312c37de46.


### M92 resumed installed116 acceptance and narrow Find evidence

- Owner resumed2026-10-25. Exact installed deb55b92a240 source/binary unchanged.
  Shared72-view alignment PASS(run-80rrah26), quiet Wayland10/10 PASS and strict
  real two-process cold reopening9 PASS(native-reopen-m92-installed-r1,
  manifestf40fb92c1ffb57282ffb24e0c1990e82258cfdb1f024aca24e070afc2ad61238).
  Root reviewed actual100/150/200 fields/placeholders/feedback/VLAN columns and
  cold-loaded separate practice. Historical Wayland cause remains unresolved.
- Actual installed gateway23/Find46 flows from M91 retain their identity.
  Scoped package acceptance is now116/28,35CCNA/24LPIC1/17LPIC2/40Bash, not
  complete curriculum or physical-device/accessibility/novice/other-OS acceptance.
- Official LPIC-1v5.0/103.3 rechecked2026-10-25. Only103.3.K5 gains narrow
  Partial; existing103.3/K1/T2 evidence strengthens. Size/time/exec/delete/links/
  full find/FHS104.7 remain unimplemented;1240 source fingerprints untouched.
  Coverage validator PASS134Partial/1Supporting/1105Open.
- Private acceptance receiptc1d1a6f5993cb1c2d82d53de6f736145d366c667a096d76722662cce8ff78482.
  All builds/runs serialized through MAIN runner,2jobs/2testthreads, disk artifacts.
  Next source integration is reviewed legacy teaching R4 and NoAnswer R1; their
  prospective checks remain unrun at this reporting commit. Helpers source-only.


### M93 original DNS/storage teaching and honest negative DNS outcomes

- Integrates reviewed R4 teaching and NoAnswer R1, preserving original116 IDs,
  versions/initials/Goals/solutions/unit order,28themes and saved drafts/histories.
  New attempts get separate Introduction/practice/full assignment; legacy attempts
  first reopen their assignment. Existing pending/context guards stay intact.
- Combined941 model/integration checks across89targets PASS. Full UI R1 finishes
 179pass/1fail: the new test directly clicked offscreen popup choices. A stronger
  selection assertion reproduces wrong index4 vs5; original logs retained. Actual
  clipped-popup scroll/click correction +pre-experiment success assertion pass all
 5 seed checks. Reviewed native controller adds3 passing focused checks,8 total;
 183distinct UI checks accounted without calling the original full run all-green.
- Clippy R1 reports items-after-test-module only; relocating the new test module
  preserves code/expectations and all-target-Dwarnings R2 passes. fmt/diff pass.
- Native controller25000 uses actual Start/Insert-only/Save/Run/Check/typed fields,
  exact serving/saved experiment observations, distinct drafts/reset/resume and
  scaled light/dark input; tests are not actual native acceptance yet. No engine
  or reference State publication. New NoAnswer adds no authored coverage/count.
- Source audit corrects a prior incomplete teaching-gap report: original Unix
  permissions/traversal and Logs still have optional guides but no independent
  Introduction/practice metadata. They remain real working tasks; adapting their
  teaching is still open. Do not claim all original nine meet the full pattern.
- Reverse-DNS three-task data and fixed getopts oracle sources remain unregistered/
  unrun worktree handoffs. No shell revision/default change or host learner
  execution. Next is a clean candidate and actual DNS/Mount screen/flow review.


### M94 actual native review corrections and startup evidence

- Installed97b lifecycle R2 PASS; R1 mistaken directory suffix failed before
  install and is retained. Default native attempt stalls; own live bounded stack
  and separate15s protocol probe localize Mesa EGL swap/Wayland dispatch waiting
  after updates0/1 and a committed callback request without observed completion.
  Underlying cause remains unproven. Only own synthetic child closed/reaped; no
  GPU/theme/compositor/security/service change. Docs NATIVE_STARTUP.md records it.
- Same ELF process-local XWayland startup PASS1/1. Full seed flow reaches all
  two-lesson checks/33 captures then FAILS exit101 at asynchronous close: final
  step incorrectly advanced outside plan. No clean-exit acceptance is claimed.
- Controller now holds final step; repeated real close-phase test passes. Nine
  focused UI/controller checks pass;16 DNS/Mount model checks pass. A first scoped
  command filtered backend names with seed_ and ran only1 of16; the unfiltered
  R2 backend run explicitly passes all16. No inflated totals.
- Actual render review removes duplicated BIND disclosure and gives specific
  negative practice diagnoses instead of positive-goal labels. All grading bools,
  passing messages, context/pending guards, fixtures/commands/Goals remain exact.
  Capture reveals the whole practice-feedback block; no product layout change.
- All-target Clippy/fmt final PASS. New native rerun still required; Wayland
  failed/pending is separate from XWayland teaching acceptance. Catalog116/28.


### M95 actual shell baseline before further semantics

- Fixed metadata/source-pinned collector committed2c89bb9; correction2d484f9
  fixes a developer-only relative marker lookup, preserving the identical VM,
  fixtures, deadline/tick limits and post-Stop immobility checks. Exact helper
  regression passes at1/16/32/64; five Rust example tests and scoped Clippy/fmt pass.
- R1 partial actual8 outcomes are retained as failed, no COMPLETE. R2 completed
  at2d484f9 with manifest d6b5777260f45c9551f1df013c531cbd0547b3f234edfa369e913c8b49cca827.
  Root independently rehashed221 outputs. Actual40scripts/246 assignment cases/
 40examples/50fixed inputs ran at all four slices, with measured Stop and genuine
 6/7 histories, exact old7 backup, real8 writes/export/restore/reset/conflict paths.
-50 actual Bash5.3.9 fixed builtin comparisons completed in disposable project-disk
  fixtures under closed environment and bounded guarded execution. Differences
  remain unclassified before9. No new shell capability, evaluator9, learner-host
  execution, curriculum count or objective promotion. Original116/28 unchanged.


### M95 installed teaching follow-through

Clean30d75e7 candidate passes identified97b upgrade/lifecycle and exact old-nine
synthetic retention. Actual full DNS/storage two-lesson flow exits0 after33captures,
with pending/rejected edits, specific false feedback, alternatives/invalidation,
separate drafts/resume/reset/Challenge and light150/dark200. Root reviewed actual
screens. Process-local XWayland quiet starts10/10; default Wayland first launch
fails15s (0/1), same observed EGL/Wayland wait, missing-callback cause unproven.
Private audit75935337... hashes captures and memory observations. No cure or full
platform/novice claim. See INSTALLATION.md for immutable identity/checksums.

### M96 reverse-DNS tasks and original terminal teaching integration

- Normal source119/29:35CCNA,24LPIC1,20LPIC2,40Bash. New distinct tasks are IPv4
  reverse-owner/type repair, full IPv6 nibble rollout, and retired/multi-PTR/two-copy
  handover. Source-reviewed20question table matches independent arithmetic.
  Bounded record editor, separate practice and behavior/state grades preserve
  unrelated named data, accept TTL/order/normalization/serial alternatives and
  require positive publication alongside both retired-copy negatives.
- Original permissions/traversal/log tasks gain explicit typed isolated examples,
  explained commands and the accepted three-view pattern. IDs/versions/initials/
  Goals/solutions and old progress stay intact. New terminal source uses existing
  engines, no host execution or dependency. Practice scope differs from broader
  legacy imports; bounded readers/context guards never narrow old assignment state.
- Model/content R3 PASS279checks/23targets. R1 fixture-constant compile and R2
 104-versus107 current-kind count failures remain. New11PTR+15legacy model checks
  cover references, alternatives, partial/wrong/current invalidation, context,
  malformed/maximum inputs and exact original116 specs/unit relative order/history.
- Four legacy UI tests, five PTR phase/geometry/editor/Close tests and nine old seed
  regressions pass. New false practice prose uses actual diagnoses after independent
  review;15 strengthened models pass. Repeated4UI review PASS. All-targetClippy
  R3 passes after two style fixes; no allow/goal/expectation weakening. First PTR
  compile's Confirm-field typo retained in failed log.193 UI checks accounted
  across prior full/scoped runs, not a new full193 all-green run.
- No objective promotion yet; matrix134Partial/1Supporting/1105Open remains.
  Latest scoped installed candidate116 at30d75e7; new119 and original terminal
  teaching still require actual native screens/flows and their own candidate.


### M97 finite shell-scope reference collector

A second peer-reviewed developer-only fixed collector admits five builtin sources,
with no source/path/binary/profile input or VM/history/catalog dispatch. Exact
same-FD Bash/manual/source pins, explicit preparation versus approval, MAIN shared
guard,2s/process30s whole bound, capped raw evidence, closed environment, quiescent
rehash and no-clobber COMPLETE preserve the existing safety boundary. All15new
unit guards pass within144Python; initial mocked checked_pins root-signature error
is corrected with an explicit root assertion. Actual capture remains pending,
expectations null, no getopts/revision9/current mastery or extra authored lab.


M97 actual follow-through: fixed5 capture ated8b1f2 COMPLETE with manifest
f15f29582e1302622f444e2ae5c7df72fa2e6bc9dbcdaef6a55dd92276dea19a;
root rehashed26 outputs. Actual unequal local scanner offsets restore parent c
after b in all four scope variations, and unknown clustered option advances
without a permanent end latch. No expected VM result/source-ID implementation,
new before9 baseline, learner credit or curriculum count is inferred. Controlled
reference Bash remains entirely separate from learner execution.


### M98 installed PTR acceptance and terminal/native diagnostic gates

Installed72f119/29 lifecycle passes30d upgrade and exact original-nine retention.
Actual three-task PTR run exits0 with50captures; root reviewed relevant normal,
150light,200dark and fullscreen150 screens. Reference inputs, separate practice,
wrong/partial/alternate/current models, negative secondary query, malformed Save,
draft separation/readback/Continue/reset/Challenge all pass. Seven existing
Partial207.2 rows strengthen only; counts/fingerprints remain134/1/1105.

A fixed original-three terminal native controller is staged with passive feedback
geometry only. Five phase/typing/Run/Close tests pass; initial geometry test sampled
before deferred zoom settled, corrected with warm-up while preserving strict
actual sample/30-attempt guards. Actual native flow pending. A Linux smoke-only
exactenv no-vsync diagnostic leaves normal options unchanged; its options test
passes.199distinct UI tests accounted across prior full/scoped evidence, not a
new full199 pass. All-targetClippy/fmt and coverage validator pass. No normal
Wayland fix, real network/process learner execution, new dependency or release.


### M99 application-only Wayland presentation policy

Installedc799 lifecycle119/29 passes upgrade72f and exact original-nine progress.
SameELF interval1 fails two separate15s starts, before and after interval0 warm-up;
requested interval0 passes3protocol+10quiet starts. Owned stacks retain the EGL/
Wayland dispatch wait. Full3legacy terminal teaching passes43captures/cleanexit
on diagnostic Wayland. This evidence motivates requesting interval0 on Linux
Wayland while preserving other-backend defaults, not claiming universal driver
repair. Pinned source confirms event-driven idle and best-effort interval setter.
Three exact environment/backend policy tests pass; all-targetClippy/fmt pass.
201distinct UI tests accounted across full/scoped evidence; no new full201run.
New normal-policy package/startup/cold/RunStop gates remain pending.


### M100 normal Wayland follow-through and stable catalog probes

Installed1181 passes119/29 lifecycle/defaultWayland1traced+10quiet launches without
diagnostic override and strict two-process normalprivateprofile reopening9. The
application-only policy avoids the observed interval-dependent wait on this host;
no universaldriver/cross-platform/novice claim. All previousfailed controls remain.

Review before scriptStop finds PTRinclude insertion shifted older native array
probe indexes. Appending newpack restores exact old116 order while unit navigation
remains byID. Strengthened originalorder+PTR+course16checks pass. Stop4924/4925
now run in consecutive frames like existing cancellation probes; scripting Stop3UI
regressions pass. Actual corrected-package arrayStop/shared alignment are next.
No grading model, learner answer, theme, dependency or desktop setting changes.


M100 arrayStop native R1 on installede850 FAILS at4925: job still active, so the
raw click on newly appeared Stop did not cancel. Root replaces only the harness
click with shared completion-first hover/click settling. A new exact real-egui
production-phase regression types/Run/drives4924/4925 and proves no gradedRecord;
it passes, as do3 existingStopUI checks. Failed native artifacts remain. No
product simulation behavior change; actual newcandidate retry is still required.

## 2026-10-25 — M100: installed normal-policy Stop and alignment acceptance

Candidate ca1929b7397529853573b432a16b4757ead2d13a passes119-lab/29-unit
installed upgrade, uninstall/reinstall and exact original-nine synthetic progress
retention. Actual normal-policy native Wayland Stop retry exits0: the running
loop remains responsive, Stop awards no record, keyboard repair/current Run and
saved reload pass. The previous e850 native failure is retained; the correction
settles the newly visible test button before clicking, without changing product
Stop or grading behavior. Exact production-phase egui regression and prior three
Stop UI checks pass.

The same installed binary passes72 shared alignment captures: blank/populated,
malformed/partial IPv4 answers and VLAN controls across100/150/200%, wide/narrow,
Light/Dark. Representative actual images reviewed show centered text/placeholders,
aligned columns, responsive stacking and scrolling. Local audit
m100-installed-final-native.json pins binary/archive, logs and74 screenshots.
This is not physical-keyboard, screen-reader, novice or other-OS acceptance.

Next M101 integrates the independently reviewed bounded getopts scanner and
actual8 progress fixtures, with all model/content/migration gates required before
new curriculum credit. The catalog remains119 authored assignments, not a claim
of complete certification coverage. No public release.

## 2026-10-25 — M101: bounded option scanner and real8 history source gate

Implemented evaluator9 getopts as typed scanner/evaluator state, with short-option
clusters, required values, explicit reset, local ownership and inherited isolated
capture/pipeline state. Exact input identity and all hidden copies share existing
instruction/work/memory limits. No new dependency or host execution. The original
256 ordinary-name allowance is preserved plus three fixed special-variable slots.
Unsupported declarations and scanner reuse fail explicitly. All119 authored
scenario identities/versions and40 script definitions/246 cases/40 examples stay
unchanged. There is no new curriculum credit from the foundation.

The full library/integration R1 run passed991 checks with one stale test sentinel
failure: synthetic future8 became legitimate old history. Archived original test
source; the correction rejects both current and future history revisions while
retaining original6 bytes and32/33-capacity checks. Scoped18 retry passes, giving
992 distinct checks across fullR1+scopedR2, not a claimed single green full rerun.
This includes55 retained actual Bash observations with disclosed exclusions/
diagnostic differences, alternate solutions, limits, Stop, exact genuine8 bytes,
6/7/8 histories, real first-write backup, cancel/conflict/restore/reset and no
fabricated credit. All202 binary/UI checks pass. ClippyR1/R2 found constant-version
test assertion style only; compile-time assertions retain the checks, final
all-target ClippyR3, formatter and diff checks pass.

Single runner/jobs2/testthreads2: the ordinary full model run observed at least
21.14GiB available memory, peak full PSI avg10=0.18%, zero OOM delta. The full UI
run retained at least23GiB, full PSI0 and zero OOM delta. Disk-backed NVMe artifacts
and the shared lock remained in use; no desktop/global setting was changed.

The coarse curriculum index is reconciled with granular parent scopes, lab IDs
and gaps, independently reviewed. All165 statuses/proposals/prerequisites and29
upcoming rows stay unchanged; granular coverage bytes remain exact. README now
points to the accepted119-lab ca1929b Fedora build, which still uses evaluator8.
Next: genuine8→9 actual installed native acceptance, reviewed scripting projects,
and guarded pure DHCPv4 foundation tests. Windows/macOS, physical accessibility,
novice acceptance and substantial remaining curriculum stay open. No release.

## 2026-10-25 — M102: pure bounded DHCPv4 foundation gate

Integrated the independently reviewed R2 single-logical-server model. It keeps
pending offers, server bindings and client-held leases separate; allocates within
finite ranges with reservations/exclusions, selects direct or relay scope, retains
actual granted options, and distinguishes new/current/renewed acknowledgements,
transport silence, NAK and release. Explicit clock advancement never invents
traffic. Collection, import, token/time and cross-octet arithmetic limits are
tested. Foreign copied-subnet/segment snapshots are rejected without freezing
valid wrong policy values needed for troubleshooting.

Guarded pure-module gate passes25 DHCP and68 library checks, including malformed
input, duplicate ownership, removed-original foreign copies, resource bounds and
atomic errors. All-target Clippy and formatting pass. RFC2131/2132/1542 and vendor
selection references, exact educational policies and fidelity exclusions are in
DHCPV4_SUBSET.md. No wire protocol, full server syntax, host networking, dependency,
engine State/Goal, scenario, UI or progress changes. Catalog119 and coverage counts
remain unchanged. Native adapters and authored practice are the next independent
implementation work, not presumed complete because this library passes tests.

## 2026-10-25 — M103: exact native migration phases prepared and tested

Added a closed developer-smoke mode for the retained actual writer8 profile,
alongside original writer7. Normal app selection cannot import either fixture.
The shared controller checks complete original per-ID Record chains, original7
corpus, exact pre9 backup, existing receipts, independent example, actual native
input targets, current Run/Stop/retry, export/restore and selected reset/Challenge.
The accepted119 catalog and all scenario/progress definitions remain unchanged.

Root source review caught an assignment/practice mismatch before runtime; the
practice now uses existing distinct Welcome/Mina/ROOM data. Independent review
caught asynchronous Finish advancement; the controller holds after requesting
Close. Seven actual-egui-phase checks pass, including two-mode first-save backups,
caller-versus-persisted receipt distinction, explicit example Run, inherited child
scanner Stop and repeated Finish. Initial guarded-copy/zero-test attempt is not
acceptance; five-of-six R2 and corrected seven R3 results are retained. All-target
Clippy and formatter pass. Full prior202 UI/992 model gates remain scoped source
evidence; actual identified evaluator9 installed flow is the next mandatory gate.

The earlier installed ca1929b candidate independently passes ten quiet starts,
one actual Wayland protocol/readiness startup and a strict two-process NORMAL-
private-profile reopen. No diagnostic interval override, personal files or desktop
settings. Source and native acceptance remain distinguished. No release.

## 2026-10-25 — M104: three substantive option-interface projects, source gate

Registered three distinct assignments after model validation: a flexible report
interface, independent repeated-job parsers, and a two-level validate-all batch
publisher.122 source labs/30units =35CCNA/24LPIC1/20LPIC2/43Bash; this is not full
curriculum or new objective-status credit. References and two supported alternatives
pass all36 independent cases at1/16/64 slices. Wrong/partial/hardcoded/file-damage
programs fail; separate runnable museum examples, meaningful changes/repair,
current-source invalidation, draft isolation, reset/resume/export/restore pass.
Input grammar and portable versus Bash-specific features are explicit.

Source review corrected quoted spaced filenames in practice; first VM gate caught
three conditional alternatives using excluded elif. Equivalent nested conditions
fix only those programs, never the engine or expected contracts. Full registered
96-target model run has1020PASS/7 stale-integration failures; all seven targets
pass59 checks after exact current counts, original-unit filtering, new experiments/
reviews and correct actual-vs-source-file ordering.1027 distinct checks now pass
across fullR1+scopedR2, not a claimed full green rerun. Old119 IDs/versions/specs,
original116 order,40 original scripts/246cases/40examples, actual6/7/8 history and
116-context old profile stay intact; new requirements remain absent/ungraded.

The5 actual-egui native-flow checks pass after fixing test preconditions, normal
related-result selection and per-phase test frame budget. Production VM limits
and15s waits are unchanged. All16 whole-catalog layout checks, current7 history
checks, original case-plan, final10 content checks, all-targetClippyR2 and format/
diff pass. The earlier lint failure was an elidable lifetime annotation only.
Current coverage validator reports122 IDs and unchanged134Partial/1Supporting/
1105Open of1240 overlapping references. No percentage or mastery claim.

The preceding119-lab b5f candidate passes installed upgrade, actual8→9 native
16captures/clean exit,11 Wayland startups and strict two-process reopen. Its local
audit is411e34d50c069b27dcd7769c2f73d141f5e4342e56c6164a09a45cbd0eebee8e.
This122 source batch now requires its own installed three-lesson native review.
DHCPbackend and Debian package management remain separate source/research work,
not playable credit. No public release or new dependency.

### M104 installed122 verification and visual refinement

Source1e7c0eb candidate passes122/30 lifecycle, b5 upgrade, exact original-nine
bytes and all3 actual native getopts flows at normal Wayland with no diagnostic
override. Real Intro/practice/experiment/repair, wrong/partial feedback, references
and manual alternatives, edits/current retry,150Light/200Dark, separate unfinished
drafts/real Continue and selected ResetCancelConfirm pass with clean exit.11
startups and two-process NORMAL-private-profile reopening also pass.

Actual visual review found the capstone's310-word task consumed the initial
946×929 view. The approved workspace and complete assignment are retained; only
task/guide wording is grouped and reduced (roughly150/180/200words). All conditions,
formats, state expectations, references, cases and grammar remain unchanged.
The text guard correctly caught lost literal-format phrasing on the first retry;
restored explicit one-line/one-argument and literal quotes/backslashes wording.
Content10 and actual-egui5 tests pass. A new identified installed native render
retry is required before calling the compact version accepted. This is not a
new scenario, solution simplification, UI redesign or release.

### M104 compact candidate accepted

Candidatecd31af6 upgrades the earlier122 build and passes lifecycle122/30 with
exact original-nine synthetic bytes. All3 normal getopts flows pass again,51
captured frames/clean exit. Actual task/feedback/readability review confirms the
more compact default assignments and visible editor, including Light150/Dark200
output/controls.11 Wayland startups and strict two-process NORMAL-private-profile
reopen pass. Local corrected audit0a7b87a3531db278f693c231d9ab0d6e7dd3e5e7245b9a2d6948b544e71db940
pins the51 images, binary/archive and startup/cold evidence; initial derived
metadata's57-frame typo is retained/corrected explicitly, no raw observation
changed. The catalog is now accepted122/30=35/24/20/43. Objective statuses remain
134Partial/1Supporting/1105Open. Continue with reviewed DHCPbackend gate and
remaining curriculum; do not equate this candidate with completion.

## 2026-10-25 — M105: DHCP backend and progress gate

The reviewed pure DHCP core now has a typed engine/Goal adapter, bounded raw policy
drafts, isolated teaching/practice state, goal-aware actions and validated
progress/export/restore. Applied policy, offers, server bindings and actual held
client configuration remain distinct. Fresh allocation contexts use the learner's
policy and real returned offer tokens, independently of current lease success.
No terminal aliases, host network action, new curriculum count or objective credit.

Initial68 library plus24 adapter checks pass. The full registered97-target
library/integration run passes1051 checks with no failures, including current122
content and real6/7/8 migration fixtures. Independent review then identified
duplicate grading in the proposed native screen. A shared fresh current-result
helper preserves all context/draft guards and rejects pending drafts before probes;
no persisted success cache. Its new regression and four existing targets pass113
checks after the refinement. The first refinement compile caught a test-only name
shadow; corrected without changing expectations. Full97 targets were before that
refinement, not represented as a second full run.

One ordinary maximal4-context/32-total-client observation measured13.972ms
clone+check in development,58476-byte core JSON. This is not a frame-latency
guarantee. The full run observed minimum22.74GiB available, zero max some/full
PSI avg10 and no OOM counter increase, with2 Cargo jobs/2 runtime threads; no
stress test. Frozen backend and the helper received independent source review.
Native R2's11 interaction proposals and actual renderer acceptance remain next.
Normal catalog stays122/30=35/24/20/43; no implementation credit from a fixture.
All-target ClippyR2/fmt/diff pass; R1 flagged one nested-if style warning, fixed
without altering the validation predicate.

### M105 native DHCP component and actual render refinement

The incremental component keeps the full task, optional introduction/separate
practice, native pool/relay editor, topology and explicit client/time actions.
One fresh result vector drives completion and feedback. Initial focused11 and
full225 application UI checks pass; the full run observed minimum23.74GiB
available, zero some/full PSI avg10 and no OOM increase. The full225 was before
the visual refinement, not relabeled as a full current228 run.

Actual first native flow passed17 frames, but review found repeated binding
details and a sequential editor unnecessarily far down. Compact policy/client
columns now stack on narrow views; server detail is optional and Apply is
secondary. Shared reserved label/value rows and remaining-width dropdowns fix
narrow overflow and Grid's invisible measurement focus loss. Explicit field IDs
keep Undo across reflow. Current14 focused checks/alltargetClippy/fmt/diff pass;
no old bounds/modal assertions were relaxed. Additional tests cover real focus/
Undo across resize and long48-byte names/64-byte values at six sizes.

Actual compact normal-Wayland flow again passes17 captures/exit0. Reviewed
Intro/practice, the100% split,150Light and200Dark feedback/fields; actual window
946×929, not the requested1200×850. Current/wrong/partial/pending/offer-only,
practice separation, actual typed changes, explicit clock, save/readback, warm
resume and selected Reset pass. Receipt0102502fffdffcb657464a665668eeb98499710ff3e8c25144e4e250974f3099.
Development ELFaa0254ec2e11957ba876861cf3d274b65c6473316688133ab234ce7446a9c038.
No packaged/cold/novice/physical-device/screen-reader/other-platform acceptance
from this component. Catalog122 and objective statuses unchanged. Continue the
reviewed three authored DHCP tasks, then remaining Linux/network/shell gaps.

## 2026-10-25 — M107: bounded Debian package foundation

A reviewed pure module separates published/cached metadata, local archives,
installed records, ownership and actual virtual payload text. Debian version
ordering, transitive ANDed dependencies, repair, retained configuration, exact
removal/purge and protected unmanaged paths are modeled for a closed script-free
subset. Logical operations are not real dpkg transactions/APT resolver/RPM
behavior; no command alias, host operation, State/Goal/UI/pack/progress or
curriculum credit is added. Primary references/limitations accompany the module.

First guarded gate:68libPASS/18of19corePASS. The failed unknown-field assertion
found serde's internally tagged unit variant accepting extra fields. Empty
struct RefreshIndex retains identical JSON but rejects extras; original negative
assertion retained, positive/duplicate-tag/all5variant negatives added. All19
and68lib then PASS (m106-m107-content-core-r2); alltargetClippyR2/fmt/diff pass
after a predicate-preserving nested-if style correction. Optional development
dpkg is absent, so no host comparisons are claimed and no package was installed
or run. No new dependency. Catalog stays122 until separate DHCP integration.

## 2026-10-25 — M108: pure steady-state spanning-tree foundation

The independently reviewed module calculates one supplied P2P instance with
bounded composite identifiers, receiving-port costs, deterministic root/port
selection, down/disconnected components and fresh forwarding-tree paths. A
separate forest check validates every result. There are no protocol packets,
PVST/VLAN instances, convergence, PortFast or guard claims, no State/Goal/native
workspace or new authored lab/coverage credit. Shared MAC representation only;
no dependency or host network execution.

All20 core checks and68 library checks PASS after exact four-path source/base
preflight (m106-reconciled-targets-m108-stp-r1). The tests include256 independent
small-graph/cost/root configurations, path enumeration and a separate cycle
oracle, asymmetric/tie/disconnected/down/maximal cases, malformed input and
atomic rejection. All-target ClippyR3/fmt/diff PASS after two identical-predicate
if-let-chain style fixes; its R2 warning evidence is retained. No real equipment
or rendered STP lesson acceptance follows from this pure foundation. The normal
source catalog's pending DHCP125 milestone remains separately gated.

## 2026-10-25 — M109: honest excluded-clause diagnostic

Bare unquoted `elif` in a command-head position now reports Unsupported with a
nested-if suggestion during whole-script parsing, before virtual writes. It is
still excluded syntax. Quoted/escaped/expanded command names, comments, ordinary
arguments, file names, variables and loop/case data retain their existing roles.
No evaluator revision, new syntax acceptance, reference observation, history or
mastery is invented.

The independently prepared three-path change was root-reviewed and integrated
only after exact base/source/34 historical-module and fixture hash checks. All4
new diagnostic/data-role tests PASS in the full library/integration R2; all43
authored scripting tasks and genuine6/7/8 history checks retain their exact
expected cases. All-target ClippyR3/fmt/diff PASS. No host Bash or learner-host
command was run. The subset guide also reconciles its stale getopts source-only
status with actual installed122-lab revision9 acceptance.

## 2026-10-25 — M106: registered DHCP unit and acceptance tooling

Three distinct normal assignments now teach held-option repair, relay/wrong-mask
diagnosis and a constrained lease-pool handover. Each has prerequisite teaching,
a different runnable example/experiment/repair, the full native configuration
assignment and alternate valid policies/action orders. Current clients plus
independent new-client orders are graded; extra valid reservations are accepted.
Normal source125/31=38CCNA/24LPIC-1/20LPIC-2/43Bash. The old122 complete source
specifications/order and30-unit prefix remain exact; no old attempt is invented.

All13 content checks PASS. Initial content failures exposed an invalid
Discover-while-held setup and incorrect teaching about early renewal; RFC2131
was rechecked and only procedural/prose assumptions corrected, not Goals/core or
expected experiment tables. Three post-completion explanations now give distinct
reasoning. All prior actual6/7/8 records remain exact. The full100-target model
run passed1081/1088; seven old integration assumptions then corrected and their
seven complete targets passed74 checks. Explicit DHCP context handling preserves
raw mutation refusal; the old116-attempt fixture must still lack all9 later IDs.
This is a full run plus targeted retry, not a full green rerun. Ordinary run
memory minimum22.76GiB, zero max PSIavg10/OOM increase, jobs2/testthreads2.

Current23 DHCP/cold-controller tests PASS, including all3 flows at observed
946x929/density1.15 geometry. Count-dependent headers now keep stable identities
and Add focuses the new field without Apply; scopes preserve text/Undo across
reflow. All125 assignments pass200% horizontal-control/VLAN-reflow checks at
1920/720 widths, plus4 shared font/library/course/filter checks.149 Python guards,
alltargetClippyR3/fmt/diff and palette/coverage integrity pass. No full237 UI
suite is claimed; the older full225 run preceded compact refinements.

Actual native R2 passed first2 full lessons and reached correct capstone200%
state, then rejected an oversized whole-panel capture target. Optional worked
explanation and an explicit result-to-navigation geometry target keep normal
large-text scrolling; no font shrink, grading weakening or hidden failing rows.
Native R1/R2 failures/screens retained. The final installed R3 flow, startup and
cold-process acceptance are still pending; last accepted package remainscd31/122.
Source registration is not objective credit:134Partial/1Supporting/1105Open stays.

The closed cold tool adds a real normal-private-XDG DHCP case with actual controls,
full raw/portable/provenance/draft comparison before navigation, real repairs,
private safe writes and exact default-script wire preservation. Peer review caught
a missing portable-projection check and an unchanged test tamper; both corrected
with meaningful negative guards before execution. Three Rust/six Python new
checks pass. Real two-process/GPU evidence remains a separate pending gate.
No new dependency, host command, release or novice/other-platform acceptance.

### M106 installed candidate86ca and clipped-control correction

Clean86ca Fedora candidate passes installed125/31 upgrade/lifecycle and exact
synthetic progress retention. Archive c8ce64a093041f62ceb05742f950a18a122888827dd3a342bc25b27f2d0393d6;
ELF7eadafee389f51afd8326ace77f1248ceffa1a1fab1f00ea94c5bc76e42fbdd4.
1 protocol+10 quiet normal Wayland startups pass~0.8–0.9s without a diagnostic
override. Both actual normal-private-profile two-process cold cases pass:
DHCP writer37.674s/reader14.699s with12 captures and unchanged default scripting
case. Full raw/portable/receipts/drafts/favorite/appearance are checked before
navigation; actual repair follows. First wrongly placed cold output was refused
before child launch, then corrected to the required disk TMPDIR; guard retained.

Installed native alignment passes72 captures:36 at actual1920x1080 fullscreen,
36 at946x929 tiled,100/150/200%, light/dark. Representative IPv4 placeholders,
populated errors/topology, VLAN numeric columns/stacking and focus were reviewed.
These are86ca renders, not screenshots from a later identity. The requested
1100x820 tile was not honored; actual dimensions are recorded.

The complete installed DHCP unit flow R3 still FAILED: first2 full lessons
passed, then capstone practice repair's Remove was visible in the outer viewport
but clipped by its inner list. A missed automated click left the extra exclusion;
the strict full-policy assertion correctly failed. This does not establish
complete native unit acceptance. All failed evidence remains.

The independently reviewed three-path correction records actual collection
control clips and shares bounded stable inner/outer scrolling with fields. It
requires exact selected-pool raw Add/Remove vector changes before acknowledging
a click, retaining the full applied-policy assertion. No product layout/IDs,
model, Goal, authored data or progress wire changes. All26 DHCP/controller/cold
checks pass R12, including3 new real clipped-button/fault-injection groups.
All-target ClippyR5/fmt/diff pass after one identical-predicate style change.
A fresh identified candidate and complete installed unit retry remain required;
no objective-status promotion, release or full-product claim.

### M106 final installed acceptance —14cf248

The corrected clean125/31 candidate passes installed upgrade/lifecycle, all3
complete DHCP native learning flows (63actual PNGs),11 normal-Wayland startups,
both real2-process DHCP/default-script cold cases and72 IPv4/VLAN alignment
views on that SAME ELF. Actual wide1920x1080/tiled946x929,100/150/200%, light/dark,
representative fields/focus/feedback/completion reviewed. All grading and policy
assertions retained; formerly clipped Remove now uses actual inner scrolling and
its exact list-change acknowledgement. Earlier failures/evidence remain.

Archive8e556d5813de3945dd8cd56fa923725a5f7626f5ff65bb4249d84df34f6f555b;
ELFd873a1879e76d6bc2c1104e03751a9218e6946a4ef14aad4eecdec5ad217f2c1;
private accepted receipt42db18ec35daa435700d3e86fc537360cc8db77f9700bfa302961f253f31fd53.
Only current CCNAv1.1 parents4.3/4.6 gain Partial practice; complete source
fingerprints/future version rows unchanged.136Partial/1Supporting/1103Open are
overlapping references, not completion percentage. No DNS resolver, IOS, actual
packet/client protocol, LPIC-2 daemon or exact OS-tool credit. Installed candidate
is a usable scoped milestone, not complete curriculum or other-platform/novice/
assistive-technology acceptance. Continue package, STP and Apache capability gates.

## 2026-10-25 — M110: bounded Debian package application and command adapter

Integrated independently reviewed backendR3/terminalR2 over the pure package
model. Optional family metadata, immutable assignment context, fresh installed/
config-files/absent and virtual-file predicates, isolated practice/drafts and
validated backup/reset remain within existing pack/profile caps. No host package
program, archive, download or maintainer script is used. Literal supported
dpkg-query/APT/dpkg spellings parse into typed operations; query output is derived
from current records and mutation output explicitly identifies simulation.
Unsupported shell syntax/selectors and excessive input/output fail honestly.

First guarded run:68 library+19 core+15 command checks pass;19/20 adapter checks
pass. The remaining test had expected ConfigFiles for an initially Installed
record. It now independently performs Remove before pinning the query baseline;
all20 adapter checks pass on retry. Initial state/Goals/expected predicates were
not changed. Eight additional progress/backup/workspace/content/history targets
pass77 checks. All-target ClippyR3/formatter/diff pass after two semantics-neutral
style fixes. Three older125 metadata completion paragraphs were explicitly
reconciled to committed M106 prose; no history, authored outcomes or unit order
regenerated. Frozen source receipts and initial failure remain private.

First ordinary gate min19.48GiB available, max memory PSIavg10 some/full0.3/0.3,
OOM counter unchanged; no sustained pressure, extra heavy invocation or stress
test. Native package workbench and authored unit remain separate gates: normal
catalog stays125/31 (38/24/20/43), coverage136Partial/1Supporting/1103Open unchanged.
No actual dpkg/APT reference observation, new curriculum credit or release claim.
Continue native package work, reviewed STP adapter and Apache foundation.

## 2026-10-25 — M111: reviewed spanning-tree adapter and static HTTP foundation

Integrated the bounded STP adapter/teaching metadata over both existing package
and DHCP hooks using exact frozen bases and explicit three-way conflict review.
Fresh root/port-role/cost/path predicates, complete physical-availability and
positive-connectivity guards reject isolation shortcuts. Raw numeric drafts stay
atomic; pending mismatches clear current success. Separate fixture identity,
practice, actual historical progress and selected reset are checked. The only
pure STP API change exposes its unchanged action validator within this crate,
so teaching admission checks borrowed fields before serialization.

Added independently reviewed pure Apache2.4 static-serving subset: closed vhost/
Directory parser, saved-versus-active configuration, live virtual file contents,
name/default selection, typed GET/HEAD/status/redirect observations and strict
limits. No app adapter, HTTP connection, server, TLS/proxy or curriculum credit.
Source review corrected duplicate raw-slash normalization, case-insensitive sole
DirectoryIndex disabled, and duplicate-map test isolation before first execution.

First guarded gate passes68 library/20 STP core/18 STP adapter/20 Apache checks;
all-target ClippyR1 passes, then8 existing integration targets pass80 checks.
Formatter/diff pass. Initial integration formatter caught an export placed before
crate inner documentation; moved the export below crate attributes before any
compilation, retaining forbid(unsafe_code). First run min19.26GiB available,
PSIavg10zero and OOM counter unchanged. Source/merge independently reviewed.
Native package and STP contracts accepted for isolated implementation. Four
distinct package assignments are drafted privately for review, unregistered and
uncredited. Normal catalog stays125/31 and objective matrix unchanged. Continue
native workbenches and substantive reviewed curriculum; Fedora candidate remains
14cf248, not this newer source. No release or platform/novice acceptance claim.

## 2026-10-25 — M112: four authored Debian package tasks and honest guidance

Authored distinct installation, stale-metadata/dependency upgrade, damaged-payload
repair and remove-versus-purge handover tasks, with prerequisite introductions,
different practice catalogs/paths and explicitly authored experiment outcomes.
They remain UNBUNDLED until native command sessions/annotations and content flows
are accepted. No lab-count/objective promotion. Reference solutions use supported
real command spellings; no internal author-action JSON is taught as Linux syntax.

The legacy backend pack policy still rejected every guide command after the
terminal capability landed. It now validates optional guide examples through the
actual closed parser, requiring even quoted help to be labeled a simulator helper.
Unknown commands, shell operators/selectors and author-action JSON still reject.
This changes content admission, not package execution or objective predicates.

Eight substantive content groups pass, including legitimate dependency-first,
multi-package and reverse-order alternatives, partial/no-op/query-only negatives,
stale metadata, purge losing local settings, current success invalidation/repair,
independent fill-only examples and exact experiment vectors, unsupported-input
atomicity, all-four reset/resume. Broader gate68library+80integration PASS; all-
target ClippyR1/format pass. Independent source review accepts final data/tests/
guard. Initial errors retained: wrong practice paths, missing positive installed
keeper, guide/step cardinality, old reject-all policy, one test-format compile typo.
Validators, old authored outcomes and expected experiment vectors were not weakened.
Min22.16GiB available, max PSIavg10some/full0.36/0.36, OOMdelta0; no sustained
pressure or stress test. See DEBIAN_PACKAGE_LESSONS.md. Continue native work.

## 2026-10-25 — M113/M114: native STP workbench and four authored graph tasks

Native bridge/port/link settings, current root/role/cost/path observations, atomic
Apply, one primary Check, optional help and separate practice integrate without
IOS imitation. Root review caught repeated Apply failing to mark state dirty after
an intervening autosave; fixed actual Apply dirty flag and added save/readback
regression. Independent source review accepted. Three canonical MAC display calls
and one smoke State qualification fixed first compilation, without model changes.

Actual nativeR1/R2/R3 FAILED after4screens: practice feedback requested reveal on
every checked frame, overriding wheel movement. Pointer-column targeting alone
did not fix it. Event-only reveal plus idle-render regression fixes the actual
UX; original outcomes/visibility checks retained. R4 reached14screens then caught
a controller setup error: inserted favorite did not mark dirty after autosave.
Corrected setup, aligned headless render→save→SmokeTick order, and added full
controller regression. Final15headless groups PASS; ClippyR2/fmt/diff PASS.

Final development-native R5 exits0 with16 actual946x929 screenshots, native
density1.15,100/150/200%, Light/Dark, real fields/pickers/Apply/Check, isolated
example experiment/repair, wrong/partial/reference/alternate/current invalidation,
warm exact-attempt/favorite resume and ResetCancel/Confirm. Representative actual
final images reviewed. ELF69bafc057c9eae362ebfc773b0580a9776f9bab177e11506827e89e0046748e5;
private receipt reviews/spanning-tree-foundation/development-native-r5/RESULT.json.
Min23.62GiB available/PSIavg10zero/OOMdelta0. Technical fixture only; not packaged,
normal authored-unit, cold-process, actual Favorite-button, novice or other-OS
acceptance. Earlier failures/evidence remain.

Four distinct graph assignments authored separately: root election, receiving
local cost, parallel sending-port tie and four-bridge handover. Six content groups
PASS with20core/18adapter, including alternatives, plausible mistakes, all-link
isolation refusal, partial repair, current invalidation, pending-draft resume and
isolated exact experiment vectors. Fixed prerequisite metadata and test-only draft
row offset; authored vectors unchanged. Source independently reviewed. Both this
unit and four package tasks remain UNBUNDLED; normal125/31 and coverage unchanged.
Continue package native integration/annotations, registration/native full-content
flows and Apache/remaining four-track work. No release.

## 2026-10-25 — M115: native isolated package command sessions

Integrated independently reviewed13-path workbench over current STP/DHCP: native
closed command console, bounded scrolling transcript/status/diagnostics, same
font-derived input/button sizes, independent raw assignment/example drafts and
read-only supplied-file inspector. Optional faithful command annotations require
same parsed operation AND same reached guarded effect; Insert never executes.
Current package outcomes remain state-based; unsubmitted commands/history don't
invalidate honest applied credit or confer success. Old None wire/context/bounds
remain protected. No host command/archives/downloads or new dependency.

First gate68lib/8content/15terminal/20adapter pass;9/10Session pass, then all10
Session pass after test-only explicit-inspection correction. Old Script results
are legitimately rechecked by backup inspection: ordinary resume must remain
exact, while post-inspection old Attempts must equal an independent old-only
inspection control. Selected full Attempt/both Sessions/favorites before reset
remain exact; original genuine histories/production code unchanged.

Eight UI groups pass. Initial painted-Run check used a stale prior-frame button
rect during smooth scrolling; corrected to strict same-frame containment, no
visual/tolerance weakening. Combined23package/STP UI/controller checks pass,
9broader targets77pass, alltargetClippyR2/fmt/diff pass after unused import and
identical-predicate style fix. Source integration/test deltas independently
reviewed. New content prose clarifies independent sample workstations/destructive
Reset needs, and defines bridge/root in everyday terms; no Goal/initial/case change.
Normal catalog still125/31. Next: faithful annotations, register eight reviewed
source assignments with exact old125/31-prefix/history proof, then actual native
content flows and candidate gates. Package native source is not yet rendered
acceptance. ApacheR2/20prospective tests source accepted; Bash public9 collector/
fixed reference proposal precedes any evaluator10 change. No release.


## 2026-10-25 — M116 in progress: register eight reviewed assignments

Source catalog133/33 =42CCNA/28LPIC-1/20LPIC-2/43Bash. Appended four distinct
Debian-package tasks and four STP tasks/two units, with exact original125 typed
specifications/order,31-unit prefix,282 Bash cases and genuine progress fixtures
preserved. Six optional command annotations match parsed actions and reached
effects. First registration gate81 checks across eight targets passes.

Normal package controller passes four headless groups plus nine package UI checks;
Challenge offers optional hints and a mode-salted collapsed completion explanation
without unsolicited walkthrough. Actual development-native four-lesson run passes
69 captures, observed946×929 and1920×1080, Light150/Dark200 real typing, independent
examples/experiment/repair, wrong/partial/reference/alternate/current invalidation,
real Favorite/Back/Continue, warm exact raw/portable/receipt save/readback and
ResetCancel/Confirm. Representative actual Intro/assignment/error renders reviewed.
ELF27c0d57771f8e090cec1d3da0e796982a03b99f64d32713bb0f2dde0f02f0e0f; private
receipt reviews/package-management/development-normal133-native-r1/RESULT.json.
Min23.36GiB available, PSIavg10zero, OOMdelta0. No real dpkg/APT/host shell,
novice, cold-process or packaged133 acceptance. AlltargetClippy/fmt/diff pass at
this stage. Normal STP controller integration/gates and new candidate remain
pending; coverage136Partial/1Supporting/1103Open unchanged.


M116 normal STP follow-through: first controller3/4 passed, strict popup visibility
caught an earlier larger Area width after zoom. Shared popup-child fitting fixes
actual option layout, without changing the strict predicate; retry4/4 passes.
NativeR1 sandboxNoCompositor retained separately; approved desktop R2 connected
normally, then was deliberately interrupted after30 captures when rendered review
found label-length-shifted Bridge/Port pickers. Shared form_row aligns them and
plain root-port text replaces None. All19 STP UI/controller groups pass; strengthened
width/scalematrix passes. No model/Goal/State/fixtures/grading changes.

Final development-native R3 passes48actual images/allfour normal tasks at measured
946×929/1920×1080,100/150/200%, Light/Dark. Actual Fill-only/Apply/Check, practice
experiment/repair, wrong/partial/reference/alternate/current invalidation and repair,
real priority/cost typing, full raw/portable/receipt warm save/readback, actual
Favorite/Back/Continue, ResetCancel/Confirm and Challenge. Representative final
assignment/partial/Light150 renders reviewed. ELFb49b6b50bb380c5b752f03403de63e135e9e5fdd08692131a6ccaeffc5589722;
private receipt reviews/spanning-tree-foundation/development-normal133-native-r3.
Min23.51GiB/PSIavg10zero/OOMdelta0. Warm/synthetic/reference-driven, not packaged,
cold-process, novice, physical-device/screen-reader/Windows/macOS acceptance.
Broader current-count maintenance and candidate gates continue; coverage unchanged.


M116 full regression: first complete run1469PASS/15FAIL, including260/272 UI
(1144.79s). Failures were older122 current-catalog setup, context-free test scope
for new goal-required types, broken-state test dispatch and the exact old-writer
missing-ID set. Corrected current counts; retained legacy110 context-free targets
with explicit4package/4STP exclusions and original command/state assertions. Added
real goal-aware dpkg-P/BridgeOnline mutations to broken-state save/resume for all
133; genuine116 attempts/40definitions/246cases remain pinned, with exact17later
IDs absent. No expected vectors or genuine fixtures regenerated.

A cfg(test)-only OnceLock retains one normally validated immutable catalog and
clones owned Scenario/Unit data per fresh application. Production loaders remain
verbatim and uncached. New isolation test proves separate contexts/profiles/
progress/favorites/native drafts and deep catalog mutation isolation. No grade
or expected-output cache, disabled tests, or concurrency increase. Final complete
all-target/no-fail-fast rerun1485PASS/0FAIL,116harnesses; all273 UI groups PASS
in77.53s. This is test runtime, not application startup or a novice study. Min
22.76GiB available/PSIavg10zero/OOMdelta0. Private receipt
reviews/catalog133/full-regression-r2/RESULT.json. AlltargetClippyR4/fmt/diff PASS; public-tree checks
continue before commit/candidate; installed133 and conditional coverage promotion
remain pending.


M116 committed/pushed38a8098. Clean Fedora44 candidate built at that exact source,
ELF6d0257e58ccec397f8d374224b0a6b68bb1816365c29541e0da0995327185ddd,
archive df94f20838a5b712787a4067ed59edb264975434e12acca31d067a6b7c43aa60.
Target223 dependency notices/source gate passed; min21.72GiB/PSI0/OOMdelta0.
Owner requested safe pause before packaged acceptance. Candidate12414 reaped,
workers paused, no heavy child remains. Installed/lifecycle/startup/cold/alignment/
new8 flows on this exact release ELF remain pending;14cf remains last scoped
installed acceptance. Matrix unchanged. Private ApacheR4/NATR2 source accepted
but21/20groups UNRUN, neither applied. Public9collectorR3 repin has not begun:
only clean38a worktree created; acceptedR2 input/source/tooling remains unrun.
Precise hashes/commands and independent next steps in CHECKPOINT.md. No release.


## 2026-10-26 — M117 resumed installed checks and baseline tooling

Owner resumed. Same38a installed ELF6d0257e5 passes lifecycle upgrade14cf,133/33
content/manifest/hash, desktop validation, uninstall/reinstall and exact synthetic
original-nine progress retention. NormalWayland1protocol+10quietstarts PASS
0.865–0.915s, no driver/desktop override; script/DHCP real2process reopening PASS.
72sharedalignment captures pass; representative actual large/small light/dark
IPv4 placeholders/populated/topology and VLAN columns reviewed. All4 normal
package lessons pass69installed captures/realcontrols, isolated practice, wrong/
partial/alternate/currentinvalidation, rawSessions warmreadback, navigation/reset
and large-text typing. STP installed4 pending; no final133/coverage acceptance yet.

Reviewed collectorR3 repins only133/33 source/catalog closure and two new packs,
with43/282/43 input block byteexactR2. Source55a6e782/peer a4412596;11owned files
applied after exactpreconditions.176Python (27newguards),fiveRustprobe tests incl
digest,alltargetClippy/probe ordinarybuild pass. Root holds sole shared heavy
lease; no Bash/metadata/fullVM producer/capture yet. No parser/evaluator revision
or new curriculum from tooling. Pin preparation and independent actual approval
precede every fixed reference capture; real-byte/deadline safeguards unchanged.


Installed38a STP four-task run completes48actual captures, run-f4mokd4m, with
reference/alternate/partial/currentinvalidation, isolatedexamples, actual150Light/
200Darktyping, fullwarm raw/portable/receipt/readback, navigation/Favorite/reset.
Representative final renders reviewed. SameELF6d0257e5 now passes complete scoped
133/33 lifecycle/startup/cold/alignment/new8 acceptance. Receipt
reviews/catalog133/installed-38a-acceptance-r1/RESULT.json SHA6edf1592b456d4d48195bd68adbae08ec903f557d40a90728d934e1f6af3c49e.
STP min23.59GiB/PSI0/OOM0. Only currentCCNA2.5/a/b andLPIC1-102.4 promoted
Partial;140/1/1099 overlapping refs, all1240 source fingerprints unchanged.
No full RapidPVST+/Debian/RPM/futureexam/platform/novice claims or release.

First actual public9 capture public9-133-8f9-r1 ABORTED after1568VM-only rows at
full history-map comparison (original116 vs current133 keys), before Bash phase.
No COMPLETE; raw partials/controls retained. Four-file sourcepeer correction
ee3e7205/peer0433279e preserves exact116 old histories plus17explicit empties
at construction/load/inspect/restore and full currentkeysets; all other fullfield/
rawbyte checks retained with bounded mismatch diagnostics. Actual profile-helper
regression and independent Python guard added; no VM/data/expected-history or
recordrevision changes. New gates/re-pinned fresh-ID retry remain separate.


History correction gates PASS: newactual profile() regression runs from actual
referenceVM9 Records, checks complete real helper save/load/inspect/restore/reset
and rejects fabricated addedhistory/oldhistory loss/keyset replacement. All177
Python andsixRustprobe tests,Clippy pass. No production engine/inputs/fixtures
changed. New approved-source/binary pins and new no-clobber ID required for actual
retry; oldR1partial remains non-COMPLETE. Scoped installed38a evidence/crosswalk
and correcteddeveloperprobe can now be committed together before retry.


ActualcaptureR2 public9-133-717-r2 completes1570VM rows/0stderr/fullRustprofile
then independentPythonprojection aborts on idlessStop row, beforeBash/COMPLETE.
Sourcepeer two-file classifier correction6b1db4d1/ba7a0d17 keeps authoritative
verify_vm unchanged, restricts projection to7namedtypes with strictID/slice/actual/
duplicate/unknown checks.178Python PASS; read-only retained actual1570/fullprofile
validation PASS. No input/probe/VM/history/record revision changes, no accepted
baseline frompartials. Commit/newpincontrol/newID fullretry next.


## 2026-10-26 — M118 unchanged scripting baseline accepted; next capability gate

Public9 R3 actual capture completes1570 VM rows/380 vectors and28 fixed closed
Bash5.3.9 observations, current43/282/43 authored inputs, genuine6/7/8 histories,
actual9 outcomes, full133 map, unfinished drafts, restore/reset/Stop. Manifest
8987f4290cf58bdaa7bd7981f4cbeae9454fb4155bff47f4f254c0d8140aac00;
independent final review40ef01401082e4a41adcfc1f3f555a3a7d3adde15655a3e8bdd3dca727cf51be.
All138 pins/135 outputs independently rehashed. Exact138 old source/probe/manual/
Bash pins preserved immutably before next source/build. Min23.585GiB/PSI0/OOM0.
PriorR1/R2 remain failed/non-COMPLETE, no outcomes fabricated or retagged.

Source-only scoped evaluator10 implementation authorized after this gate. Root
integrates reviewed Apache backendR4 (13paths, preserving current test-only UI
catalog cache); first21/resource tests pending. Neither task adds normal labs or
coverage yet. No new dependencies, host learner execution, public release or
platform/novice acceptance claim. Installed133 candidate38a stays available.


M118 Apache backend first gate:20/21 pure/app tests PASS after fixing a test-only
14-byte malformed port to a bounded4-byte malformed value; overlimit negative now
explicitly preserves the prior saved target. First Clippy caught a one-character
push_str in the new test; changed to push without semantic difference. Immutable
RequestView batch optimization source-reviewed: public precedence/status vectors
unchanged, borrowed lifetime prevents stale observations, no persisted cache.
69 library +20 pure +21 app checks PASS; max32 batch91.05→30.77ms unoptimized.
Native renderer/private4-task authorship next; no bundled count/coverage promotion.

All-target Clippy/fmt/diff PASS. Affected82 checks PASS (21Apache app,7backup,
10package Sessions,20package app,6progress sequences,18STP app). New Apache125
snapshot exactly matches the existing STP125 fixture; test now reuses it, while
redundant copy stays private. No baseline deleted. Gate observations minimum
19.71GiB available, max somePSI0.62%, OOMdelta0; no sustained-pressure refusal.
Private receipt reviews/apache-static/main-backend-gates-r1/RESULT.json. Native
source worker remains source-only; public catalog133 and candidate38a unchanged.


## 2026-10-26 — M119 scoped line reading and pipeline status

Implemented source-reviewed literal-empty IFS read policy and explicit pipefail
in evaluator10. Original parser architecture, ordinary default paths, budgets,
cooperative Stop and in-process isolation remain. No environment-prefix framework,
custom splitting, errexit, host process or dependency added. Fresh runs start
pipefail off; functions share it, captures/stages inherit and isolate it. Read
arguments/ordered redirects see original default IFS before the temporary policy.

All15 new VM groups,10 getopts,3 portable migration/digest and8 existing migration
groups PASS. New fixtures contain only fixed controlled actual Bash observations;
actual saved9 Records remain private. Independent root checks rehashed all28 actual
observations/source/input/provenance. New9 opt-in private migration checks PASS:
full133 attempts, actual43 Record9, real6/7/8 histories, drafts, exact raw target/
before slots, receipts, copy races, reset-first, practice-only, export/cancel/
restore and active-child Stop. Source review corrected full-progress assertions
to include main-only receipts, not just portable serialization.

Full all-target/no-fail-fast R1 PASS1535/0FAIL/9ignored,120harnesses. Those ignored9
were separately run and passed. Later Clippy caught duplicated test helper modules;
use one cfgtest library module, no lint waiver. Affected74library+8migration checks
PASS after cleanup. After10 comparison5groups PASS, including4slices for380vectors:
357 exact unchanged outcomes+ticks,23 intentional changes equal actualBash, all
1570 historical rows classified and oldmetadata still9. No new capture/COMPLETE/
Record9 was generated or relabeled. All185Python and final all-targetClippy PASS.
Actual explicit historical feature request under10 refuses before a compiler child.

First VM gate caught a test function named reserved builtin enable; only its test
name changed. Python's input-table test was coupled to the old compiled-module
closure; it now verifies the original declared packs directly, while the producer
still rejects expanded modules and a synthetic negative proves that rejection.
Whole public9 producer/input files and accepted evidence remain exact and frozen.
Private receipts evaluator10-main-gates-r1/RESULT.json and ACTUAL-COMPARISON.json;
full R1 minimum21.99GiB available/maxPSI0.18%/OOM0, no sustained pressure.

Catalog stays133/33 =42/28/20/43; no new objectives promoted. Four unbundled Apache
assignments pass8content groups after independent review closed alias-private
leakage with both-name current/next-start guards (50assignment/50example rows).
Native source handed off10paths/13UNRUNchecks. Three safe-automation assignments
are privately authored with31cases and independent content review underway.
Actual native/newunit integration and packaged acceptance remain next, not complete.


## 2026-10-26 — M120 meaningful static-web and reliable-text units

Normal source inventory is 140 assignments in 35 units: 42 CCNA, 28 LPIC-1,
24 LPIC-2 and 46 Bash/Shell. Existing 133 authored definitions, versions, unit
metadata/order and 31 pack hashes remain exact; the seven additions are appended.
No generated variants count as labs. New profiles are not invented when loading
old progress. All three scripts use the accepted editor and separate examples;
all four web tasks have an editor, live virtual files and a read-only request
workspace. No dependencies, host execution, listeners or network actions added.

The scripting tasks preserve literal input (including final partial records),
distinguish missing pipeline input from valid empty output, and validate before
publishing a handover. Their 10/10/12 deterministic cases check whole outcomes
and file maps; genuinely different methods pass and hardcoded/partial shortcuts
fail. The capstone practice uses Notice/Notices, distinct from assignment
Entry/Total, after a teaching regression caught the copied interface. Examples
and assignment drafts remain separate through navigation, resume and reset.

Web tasks cover alias/default routing, ordered directory pages/live files,
path-based access, and saved/active/next-start handover. Fifty assignment outcomes
and 50 separate example outcomes are assertions within four labs. Independent
review found an alias-private-file leak; a regression now fails both affected
rows without banning valid duplicate-name configuration. Actual screen review
then improved feedback to name the first differing response field/cause, instead
of the misleading same-status 'declared values' message. Check labels and separate
example names were corrected. Insert/Reset focuses the editor after the click;
shared retention wording and native control metrics remain consistent.

Development ELF beaa4e9f passed all seven full native Fedora flows: 68 web and
72 script captures, plus 72 shared alignment captures at 100/150/200% light/dark.
Representative renders were reviewed independently, including actual focus,
errors, complete tasks and narrow stacking. Apache/alignment measured 946x929
and 1920x1080; scripting remained 946x929 despite larger requests. Exact warm
raw/portable/receipt readback, alternatives, invalidation, selected reset and
active Run/Stop passed. These are trusted technical inputs, not novice,
physical-keyboard, screen-reader, cold-process or other-platform acceptance.
Practice/file detail text outside a captured viewport was not claimed reviewed.
A later capture regression requires the complete Expected/Actual pair inside its
paint clip using an actual inner wheel event; it changes no Attempt data.

The first expanded-catalog full gate exposed five stale test assumptions; exact
historical projections and new-family goal-aware checks fixed them. Full R2 then
passed 1573 checks, zero failures and ten private-evidence checks skipped. Those
ten were explicitly run: library/after10/migration 74+5+8 all pass, including
actual9 and genuine6/7/8 histories, old exact bytes, receipts and new attempts
remaining absent. Python 185 passed. First final UI-polish gate passed 296 and
failed one new test sampling its asynchronous result too early; a bounded real
frame wait corrected the test, and all seven controller groups pass. The final
full regression after diagnostic/text/footer changes passes 1577 checks, zero
failures and ten explicitly private checks skipped. All-target Clippy/fmt and
coverage validation pass; the CSV uses allowed integration-test evidence paths
without widening the validator. Full R3 minimum available memory was22.17GiB,
maximum PSIavg10 zero and OOM delta zero; no pressure refusal.

Only LPIC-2 208.1 parent/K1/K3/K7 gains narrowly Partial practice after the native
baseline. All 1240 source fingerprints/positions stay exact; named tools,
installation, logs, authentication, dedicated-IP vhosts, explicit Redirect,
modules, tuning, HTTPS and proxy work remain open. Current overlapping matrix:
144 Partial, one Supporting and 1095 Open, not a completion percentage. The latest
accepted installed candidate remains 133/33 at38a; final polished 140 candidate
and SAME-ELF installed acceptance are next. No public release.


### M120 installed follow-through and capture timing correction

Clean d781 Fedora candidate built with223 dependency notices. Its installed
lifecycle/upgrade38a/uninstall/reinstall/desktop/original-nine byte retention,
11Wayland starts (ten quiet0.865–0.915s), script/DHCP real two-process reopening
and all four web workflows (68PNGs) pass on the same ELF b3373734. Refined actual
feedback names the wrong site and distinguishes no-index403 from access denial.
The scripting flow failed at first Stop: no live job at phase37067. It is retained
as failure, not accepted or relabeled. Source review found Run acknowledgement
still used the ordinary350ms delay. Only phases37066/37266/37466 now join exact
Stop/observation pacing; normal app execution and all budgets are unchanged.
The added actual-pacer regression plus all eight controller groups pass; adjacent
65/69 remain paced. Independent source review9c15190e accepts this scope. A new
clean candidate and SAME-ELF installed checks remain required. No release.


### M120 final same-ELF installed140 acceptance / M121 checkpoint (2026-10-26)

Source ddb886b4d41852822a3497daabcc21bc37d80e90 pushed; exact Fedora44 candidate
`dist/dlms-learning-lab-0.1.0-gddb886b4d418-x86_64-unknown-linux-gnu-fedora-44-candidate`.
Open from any cwd with:
`"$HOME/Documents/dlms_learning_lab/dist/dlms-learning-lab-0.1.0-gddb886b4d418-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab"`.
ELF90690062330eaa4b4a946c6eaaa282b5e221c42f770f672155025afbc24d0dc8;
archive bb1c89b52c1d5b043b2fb7fbdf542c73e3cf86910915723b4417aa3978f756ea.
Installed verified-ddb886b4d418-hqjy9lq7/application/dlms-learning-lab is identical.
All children reaped exit0: lifecycle upgrade from d781/uninstall/reinstall/desktop/
original-nine exact bytes,1protocol+10quiet Wayland starts (0.865–0.916s), real
script/DHCP two-process reopen, three scripting flows72PNGs run-j9ce6mal, four
web flows68PNGs run-8jgdspl2 and shared72alignment run-67dqqlph. All212 images
retained privately; root inspected representative actual errors, focus, Light150
editor and Dark200 fields. Actual scripting946x929, Apache/alignment946x929 and
1920x1080; do not claim requested wider scripting size was observed. New-seven
warm exact readback is distinct from script/DHCP cold reopening. Earlier d781
Stop failure retained, not relabeled. Final narrow capture-pacing correction has
actual-frame regression; no normal product execution/budget change.
Private receipt reviews/catalog140/installed-ddb-acceptance-r1/RESULT.json,
SHA88d8434c189936113ba68dc5b9baaba038b364e60fca90850bab692dd0a2c2c8.
Linux223 notice/source gate PASS. Nine guarded memory logs in final candidate/
installed window:1284samples, minimum21.64GiB available, some/full PSIavg10max0.18%,
OOMcounter unchanged0. No pressure refusal, shared2job/2thread workflow unchanged.
Historical Wayland exact-driver cause remains unproved; eleven starts are scoped
observations. Full curriculum, physical-device, screen-reader, novice and
Windows/macOS desktop acceptance remain open. No public release.

Current inventory140/35=42CCNA/28LPIC-1/24LPIC-2/46Bash, scripting46/314behavioral
cases/46separate examples. Matrix144Partial/1Supporting/1095Open of1240 overlapping
references, not completion percentage. No NAT/RPM authored labs or objective credit.
Native CI40381197535 checks exact ddb: Ubuntu success, Windows Clippy failure under
investigation, macOS still running at this record. CI is not desktop acceptance.

M121 isolated .build-artifacts/worktrees/nat-current140 at d781, exclusive target
nat-current140-8cccbf2286e30c9a; no MAIN integration. Reviewed pure BasicNAT20PASS,
core Clippy PASS. Reviewed adapter R3 compiled: backend21PASS/1FAIL/1ignored;
failure genuine-history test setup expected6/7/8 but v8 fixture alone lacks all.
ccna_discovery repairing actual immutable fixture composition, no weakened field
retention or fabricated histories. Max trial68calls/4clones observed1.752ms (one
technical sample, not benchmark). Root only heavy slot; worker source-only.
Next gate corrected frozen source, scoped Clippy, explicit actual9 private test,
all affected/full regressions, then coherent backend integration preserving ddb
capture pacing. Only afterward native adapter/teaching/distinct NAT assignments.
RPM EVR/query contract R2 remains source proposal, no implementation/reference
execution grant. Continue other independent work; do not stop at packaging.


### M122 native Windows build follow-through (2026-10-26)

Exact ddb CI40381197535 Windows tests passed; Clippy stopped at unused_mut in
examples/getopts_before9_capture.rs:229. The builder is only mutated for Unix
mode0700. Scope the mutable shadow to cfg(unix), preserving private permissions
there and identical non-Unix create behavior, with no lint suppression. Original
historical producer source/binary/hash evidence remains retained and unchanged;
this edited development helper is not relabeled original evidence. MAIN guarded
all-target Clippy passed after the correction. A new exact-source CI rerun is
required; Fedora ddb candidate bytes and scoped acceptance remain unchanged.
No Windows/macOS desktop runtime claim or public release.


### M121 Basic NAT backend integrated and gated (2026-10-26)

Reviewed original pure coreR2 plus adapterR5 integrated into MAIN only after the
isolated full1619PASS0FAIL11ignored and explicit20core+23adapter PASS including
actual9; source/peer receipts28b059ce/b29f3fa9. MAIN original base checks retained
ddb capture pacing and7dde Windows correction. All34 authored scenario files
stay byte-exact; normal140/35 (42CCNA/28LPIC-1/24LPIC-2/46Bash),314script cases,
46examples, EVAL10,28themes and old progress remain unchanged. Added optional
nil-skipped nat_draft, typed State/Goal and strict same-inventory admission, current
mapping/policy checks, separate copied-policy trials and bounded raw/native API.
Check never creates a current mapping. No native renderer, teaching or NAT card.
CCNA4.1 staysOpen, matrix144Partial/1Supporting/1095Open and all1240issuer pins
unchanged. Planning text now correctly distinguishes static/pool objective wording
from the old generic NAT/PAT-session label, without awarding practice credit.

MAIN full alltargets/no-fail-fast:1620PASS/0FAIL/11ignored in125harnesses, log
m121-main-nat-full-r1.log. All eleven private checks explicitly exercised in
lib74+Nat23+after10comparison5+migration8 PASS,0ignored. Actual6/7/8/9 records,
raw originals/backups/receipts, full old140 contexts and selected reset/restore
retained; no histories generated or revision relabeling. Python185PASS,
alltargetClippy and coverage guard PASS. Formatter check found one multiline
assert block; whitespace-only collapse then fmt/diff PASS. Final20+23Nat test
rerun postformat PASS; no repeated full1620 claimed for that whitespace edit.
Full MAIN448memory samples minimum19.60GiB available, some/full PSIavg10max0,
OOMcounter unchanged0; only2Cargo jobs/2testthreads under sharedlock. Finalmax
trial shape68calls/4copies; isolated final single sample1.813ms is not benchmark.
Private integration receipt reviews/nat-foundation/main-backend-integration-r1/
RESULT.json retains source/log hashes. Failed R3 history setup and R4 struct-vs-
JSON key-order byte comparison remain retained; test helpers now compare every
field value and array order while separate actual rawbyte assertions remain exact.
Two Clippy findings corrected without waivers or budget/test reduction.

Next renderer-only unbundled stage: contract98c5a8f4/manifest890773bf under peer
review; exact complete-input ephemeral key must feed library counts/filter/badges/
prereqs and workspace, no peridle trial replay or stale success. ExplicitCheck
still fresh, no broad cacheplatform. Reuse shared forms/rows/spacing/popup clips,
stable row identity/modal guards and real address observations. Then separately
review teaching/context/caps, three distinct tasks, normal integration and actual
Fedora native/install acceptance. Do not grant objective credit at pure/UI stage.

RPM pure source R1 ready privateworktree rpm-current140 at7dde, manifest9d73b80d;
23prospective groups/15independent EVR vectors, allUNRUN at this checkpoint. Root
and ccna_discovery source-review next before sole guarded compiler/reference gate.
No package CLI/sidecars/UI/normal lessons yet; no new dependencies or hostRPM calls.
NativeCI40386416552 runs7dde; previous40381197535 ddb Ubuntu/macOS jobs passed,
Windows tests passed but Clippy failed; cfgUnix correction in7dde under rerun.
Ubuntu ddb archive retained dist/native-ci-ddb886b/ubuntu, SHA
ba54642f6da67f95b94eb81186f7a26c649dc8dcc710906eda6c729de5b86ecc.
This is CI build evidence, not desktop acceptance. Current fully scoped Fedora
owner command remains ddb in INSTALLATION.md. All private images/progress/logs
remain ignored. Full curriculum, Windows/macOS desktop, physical-device, reader,
novice, macOS originaldispatch notice/signing remainOPEN. No publicrelease.


M122 follow-through: CI40386416552 at7dde3e45cbc1b8cc7b9f47094cfdf7411eadba86
completed all3nativejobsSUCCESS. Linux/Windows candidates downloaded on project
disk with private command-local TMPDIR/TMP/TEMP; sidecar/hash and bounded embedded
revision verification PASS. Ubuntu archive45ce3ffe5cba68416d6070e58c2160cc56fcb1891845e7bb9d92d31f427e9b24;
Windows4b82d722cbee629e8cd6c118dfad9c3775c4c5e377b6d268ac4008fca7280b43,
paths in INSTALLATION.md. Both140/35, before90b unbundledNAT. macOS compile/tests
PASS but originaldispatch notice stillblocks candidate. No otherOS desktop,
screenreader/novice claim or release. Oldddb CI Windows failure retained honestly.

M123 RPM firstguarded puregate compiled,22PASS/1FAIL/0ignored (sourceR1), log
m123-rpm-foundation-first-r1.log. Tokennegative fixture used6files/archive but
independent wirecount is8173, below8192;5files7020,7files9326. Fixing fixture with
6near-limit valid/7token-only invalid; strict token>MAX andstrings<MAX assertions
remain. No model/limits reduced; allother22pass. Sourceworker R2 pendingfreeze,
rootownssole heavy slot; no source/reference/hostRPM executed byworkers.
Natnativeworktree cleanprivate9cc496466117e73f1ef7477e2e5adc7641472b3d exactly
matchesMAIN90b tree, preservingprivateR5ad33458 andallsource/logsnapshots. Only3
expected add/add conflicts resolved EXACTMAIN (2subsetdocs/oneformatterblock).
Exclusive warmtargetunchanged, no crossworktreetarget sharing. Base receipt
reviews/nat-foundation/renderer-base-main90b-r1/BASE.json SHA352ab1e88080b24f1e46e17a7c29161f50fcf4847050c83afd756931f65b67ec.
RenderercontractR1 peer d00c44f0 requested concrete perkey/aggregate memo budget,
cache-unavailable truth and explicit cheap idle path. R2 delta pending; no native
implementation grant yet. Continue authorized work, no owner routine approvals.


### M123 pure RPM foundation integrated (2026-10-26)

Pure RPM sourceR3 b4c3a7bb and independent peer82e53b46 accepted,5source snapshots+
47 protected pins exact. Copy only4new files+tinylibexport into MAIN, preserving
both Nat modules and allcurrent code; never copyolderworkerlib. Implements original
bounded EVR/provider/source/manifest observations and explicit logical script-free
archive sets, not real RPM aliases/transactions. New23 checks PASS, scoped Clippy
PASS. MAIN lib74+Nat20+Natadapter23+RPM23=140 scopedchecks PASS0FAIL0ignored;
alltargetClippy17.95s, fmt/diff andcoverage guard PASS. Latest full1620PASS precedes
this puremodule addition; no full1643count claimed. Current140labs/35units and
all34scenario files,28themes,314scriptcases/46examples andsavedprogress unchanged.

FirstguardedR1:22PASS1FAIL because the supposed token-overflow fixture had8173,
19below8192. Independent JSON-only shape arithmetic established6files8173 and
7files9326 (strings28549/32513, depth8). Corrected6near-limitvalid/7invalid keeps
strict overflow/string-independence checks and adds exacttuples. FinalR2/R3 all23
PASS. One ordinary map-keys Clippy cleanup, no waiver, model/budget/test reduction
or generated golden observations. Firstcompile61samples min22.06GiB/PSIavg10max
0.18/OOMdelta0; MAINintegration29samples min19.87GiB/PSI0/OOM0; Clippy19samples
min20.87GiB/PSI0/OOM0. Shared2job/2thread workflow/exclusive targets preserved.
Private result reviews/rpm-package-foundation/main-integration-r1/RESULT.json.

Combined State S<=128KiB/N<=8192/count/depth, compact1MiB/raw2MiB closure enforced
before content clones. Plans use borrowed references before charging prospective
final State; atomic publication follows. Actual file discrepancies and dependency
problems are queryable states. Whole-database health/name-architecture constraints
are explicitly logical-subset restrictions, not universal rpm verdicts. Modified/
missing config, sidecar decisions, ownership transfers, solver, signatures and
archive parsing remain unsupported; no command aliases/native UI/lessons. All11
LPIC1v5.0 102.5refs remainOpen; planning boundary corrected, no statuses/fingerprints
or futureversion rows changed. Documentation-derived EVR vectors are not realRPM
observations. Fixed VERSION-only rpmsort oracle proposal05fcb0de is under startup/
provenance/isolation source audit, neverexecuted. ActualELF embeddednote identifies
rpm6.0.2-1.fc44; this is distinct from justreadingmanual6.0.2. FullEVR composition
and equality are not proved by sorted VERSION labels. No newdeps or hostchanges.

Parallel next work: Nat renderercontractR2 a218e8b4/manifestd210a398/peerf4893790
ROOT ACK; SOURCE-ONLY implementation grant at exact9cc(Main90b) privateworktree,
no heavy/MAIN writes. Own narrowUI/memo/tests/hooks andDefinitionPartialEq only,
no normalcards/teaching/schema yet.16entries/4MiBchargekey/16MiBaggregate LRU,
normal3 closure<=6180KiB, preclone bounds, exactkey cheapHIT; validuncacheabledata
uses fresh truthful sharedfallback once/unchangedframe rather than falsegrade.
Sameframe mutations/Check refresh finalinputs evenifextraevaluationneeded. First
source/event/maxfixture gates before rootnativecontroller and renderedscreens.
Then separatetypedteaching/context contract and3distinct authoredtasks. No new
NAT/RPM lab counts/coverage until actual practice gates. Keeprootpacing/EVAL10/
legacybytes andnormalapplication. Ordinaryimplementationcontinues, no release.


### M124/M125 live continuation checkpoint (2026-10-26)

MAIN clean codefd7be1cc347d12fd767c6ab3e4a2b77c57550db7 pushed; normal140/35,
acceptedinstalledFedora ddb unchanged. No newNAT/RPM authored labs or credit.
All localheavycommands reaped EXCEPT root Clippysession94264 at this write,
logm125-nat-renderer-clippy-r7.log. No workerheavygrants. Keep sharedMAINrunner,
2jobs/2threads/exclusivetargets/diskTMP andallretainedfailed evidence.

M125unbundledNatUI sourceprivateworktree nat-current140 HEAD9cc exactMAIN90b,
warmexclusive targetnat-current140-8cccbf2286e30c9a. SourceR1e0882d03/peer76f627a4
caught&BoxDefinition mismatch;R2one.as_ref binding6e6014fb/peerreadback. First
R2compilerE0500 librarywholeSelfborrow, correctedR3freeMemohelper0eaf6f62/
peer9c143cfb. R3UI6PASS2FAIL; memo6PASS andseparatehistory1PASS. R4diagnostics+
realheaderreveal2a0d2528 yielded7PASS1FAIL: staticlisttargetx240..573 butparent
clipx0..1172 gavewheelx593 overRIGHTcolumn. No productlayout defect proven.
R5passivestaticScrollArea.inner_rect metric+stricthelper1401a298 fixesactual
childwheelroute; all8UI PASS. Visibilityassertions/40bound retained, no direct
reset/statebypass. SourceR6FieldSpec +iteratorcleanupbbd45228 retainsIDs/labels/
widths/caps. Complete binregression313PASS0FAIL0ignored92.94s,96memorysamples,
min22.81GiB/PSI0/OOM0. AlltargetClippyR6 rejected onlytestfilter.next_back style;
rootchanged torfind (samepredicate/order), R7Clippy94264 currentlyRUNNING.
No production/state/grade/budget changes since313pass. R6snapshots8owned+
283protected remainfrozen; rootR7test-onlydelta stillneedsreceipt andfinalgate.
No rendererMAINintegration or actualnewnativecaptureyet.

Rendererexactmemo16entry/4MiBkey/16MiBaggregate with512KiBframebudget; fulltyped
key/history/pending/foreign guards, no idlevalidation/serialization/snapshotclone,
truthfuluncacheablefreshfallback. All7actualmemo groups pass. Nativecontroller
contractd0dfaeb8/6484a0aa/peer99fde940 acceptedSOURCE design for38000..38071
closed--ui-smoke DLMS_SMOKE_STAGE=nat-workspace only, privateauthoredtechnical
fixture, REALStart/fields/Apply/probes/Check/Clear, strictwholeAttempt/readback/
receipts/favorites/clock, light150dark200/observedgeometry, heldFinish. NOcontroller
implementation grantyet untilR7lint/finalsourcebase receipt. No status/subtype
filtering allowed; onlyexistingtop-levelmigration_receipts serializationdifference.
Thennativepaintreview before3separateteaching/normalNATassignments; notstopatpreview.

M124independentall46Bash sourceauditfound5teach-beforetest gaps: $#/-ne/filename=
inthetextcapstone; $#/-lt beforebundle repair; &&/= filebatch capstone; testvs[]
caseinterface; printf-formatreuse arraypriority. FiveTeaching-onlydataamendments
prepared in shell-teaching-140 basefd7, NO app/UI/grade/schema/prereq/version/
fixture change. Two newdefaultexampleguardprefixes retainEXACToriginalfixture/
normaloutcome; existing savedassignment/exampledrafts/results NEVERreplaced.
Publicclosedmanifest4f852c80 has13exactteachingops/9byteranges, reconstructsall5
originalpackbytes toimmutableold31SHA. R2frozen18efc82b underWORKTREE's own
.build-artifacts/reviews/teaching-audit-46-r1/remediation-source-r2. Stricttest-only
helpers +8guardadaptations preserveALLoriginalindexes/source-inputs/records/
producers/attestations; no runtimepatchlayer/blanketTeachingignore. 5newcurrentVM
and1Pythonmetadata test sources UNRUN; actualoldobservationsneverrewritten.
Peer2d9ff68c verified17source/261protected/19MB andexactraw+typedreversal. Only
sourcefindingduplicate_mod SHA import; authorR3dedup pending (reuseexisting
rootdigest orproof::sha, no waiver/oldfilechanges). Firstguardedcompile/testQUEUED
followingNatheavy; needmetadata/pins/sourcepeerreadback andallaffectedhistorical/
private migration gates, thenteachingnative/screens. Sourceworkeronly.

M126RPMVERSIONreferencehelper sourceR1frozen0cf9c7bb/patch5d2b4345 inrpm-current140
(base7dde +exactpureRPMR3).5owned helper/tools-tests/fixed20VERSIONpairs/tinydev
allowlist/docs;23fake-processgroups UNRUN. No actualrpmsortprocesshaseverrun.
Independentteaching-audit sourcepeerverified5owned+52protected, no materialbug;
nonblockingdoc precision30seconddeadlinevsboundedcleanupelapsed. ELFembedded
package6.0.2-1.fc44/buildid c556addb.../SHA54bc254c...; upstreamwrapperno direct
config/macro/DBcalls, downstream/transitiveclosure notabsoluteproof. Rootaccepts
trustedFedoraDEVtool withfixedliteralstdin/nooptions/files/closedHOMEenvcwd,
sameFD ELF pin/40sequential2s/30deadline/512KiBtotal/sourcepinrechecks/noclobber
COMPLETE. Actualcapture mustdefaulttoolssandboxONLY, no escalation/securitybypass.
No EVRcomposition/equality/referenceconformanceclaim from VERSIONsorting; ties
pending. Root/peerreview+fakeprocessgatesbeforeexplicitcapturegrant. SourceR1
frozen whileNatpeerwork, no helperMAINintegrationyet. App/terminal4taskoutline
061ad516 pendinglatercontract, CLIaliases/configsidecars/solver remainunsupported.


### M125 renderer gates / M126 fixed reference gate (2026-10-26)

M125 source/event gates complete privately: fullbin313PASS0FAIL92.94s, finalalltarget
ClippyR7PASS4.14s and8finalUI PASS1.45s. Rootprivatecommit6e169411c6cb342811214d73008a307d6e626755
on nat-current140 is CLEAN acceptedrendererbase, stillNOTMAIN/unbundled. Rootgate
reviews/nat-foundation/renderer-root-gate-r7/RESULT.json SHA0398da9f11086f9961bc7357dea7b178efa452f1d8d0ec41c4fe058f8623281b.
No activeheavycommand at this write. Renderer-sourceR1..R6/failedcompiler/lint/
viewportdiagnostics allretained. OnlyR7runtime delta fromfull313 isNONE; rfindis
test-iterator-only cleanup, documentstatusreconciled. All15newUI/memo groups pass.

ROOTgranted source-only technicalcontroller undercontractd0dfaeb8/6484a0aa,
peer99fde940, toccna_discovery onexact6e169privatebase. Controllerunderimplementation,
no heavy/actualnative yet. Explicitadditionalowned scopes: passivetab/ThemeSamples
metrics scripting.rs; actualsafe chooserCancel indata.rs+metric (noSave-asCancel),
contractamendment4bd1270a/ef204aba; readonlyPresentation.observed_effect getter
only. Cancelmustnotwritefile/progress andmustworkwithinvalidfilename, usemeasured
footerrow reserve; existingX remains. Allothercontrollerboundaries/staticinner
viewport/wholeprogressreadback/pacing/Closehold/normal140 unchanged. Sourcefreeze,
root/peer/gatesbeforeactualFedoraWindow, thenthreeauthoredNATtasks notjustpreview.

M126 fixedVERSIONoracle integratedMAIN source afterroot+independentreview;
all208Pythonfake/workflowchecksPASS bothprivate7.802s andMAIN7.849s. Docdeadline
precisioncorrected:30secondlaunch/completiondeadline, syncfilesystem/cleanup can
extendelapsed. No actualrpmsortprocesshaseverrun at this entry. Provenance/5source
pins retained source0cf9c7bb; RootMAINfakegate receipt
reviews/rpm-package-foundation/version-oracle-main-r1/RESULT.json SHA5ae1402a3d07f04287af2bb2e15601b5ab02801d09ab244dd964d46d590c332b.
HelperexecutableRoottrust exactFedoraELF54bc254c..., fixed20VERSIONlabels/bothorders,
40sequential/nooptions/files/sourceoverrides, privateHOME/TMP/XDG/emptyPATH/C locale,
2sperchild/30deadline/512KiBtotal, sameFD+beforeafterpins/noclobberCOMPLETE. Fake
processcheckscoverboundaries/timeout/cleanup/publication andrequire no realutility.
No arbitrarycommand/learnerinput orfullEVR/equalityproof. Rootnextcommit/push then
explicitfixedcaptureid underDEFAULTsandboxONLY; refusalmustNOTescalate/bypass.
No appdependency or learner-host execution introduced. SourceMainnormal140/35,
allcurrentobjectivestatuses/themes/progressunchanged. No release.

M1245Teaching-onlyamendments remainprivate shell-teaching-140. Peer2d9ff68c accepts
exact5originalpackreconstruction/13closedleaves/8historicalguards and261protected
files; sourceR3digestdedupinprogress (oldreaderthinSHAaccessorallowedifneeded,
NO producer/attestation/data/recordchanges). FirstguardedfreshVM+allaffected
historical/private/progressgatesqueuedafterM126capture under sharedlock; no tests
or native acceptance yet. Independent RPMhelperreviewby teachingaudit foundno
materialcodeissue; deadlinewordingcorrected. Continueauthorizedimplementation.


## 2026-10-26 — M124 teaching gates and M126 reference-domain correction

Independent review of all46 scripting assignments found five prerequisite gaps:
argument-count/numeric/text guards in the text and bundle capstones, conditional
AND/text comparison in file batches, the `test` spelling in case dispatch, and
printf format reuse for arrays. Five Teaching objects now introduce those concepts;
two separate practice defaults demonstrate their guards without changing fixture
inputs or declared outcomes. All assignment IDs, versions, prerequisites, starters,
objectives, cases and reference solutions remain exact. Existing practice drafts
and results remain intact; reset receives the revised default.

The closed test-only amendment pins all13 teaching leaves and nine raw byte edits,
reconstructing each original pack to its immutable historical SHA. Full nonteaching
scenario comparisons and original captured evaluator evidence remain enforced.
Focused6 and affected88 tests pass in the isolated checkout, including actual
writer8 work and the explicit private public9/current comparison. All-target Clippy
and186 Python checks pass there. MAIN full regression and native review remain
pending at this entry; no novice usability study is inferred.

The first40-observation Fedora rpmsort collection completed under the default
sandbox, but its16 mismatches were investigated rather than copied into the model.
Matching upstream wrapper review showed bare labels were sorted as package names.
All outputs/completion hashes were reverified and preserved. The corrected fixed
NVR fixture keeps all20 original version expectations and fixes only equal name /
release framing; its new fake gates and capture remain pending. Equality, fullEVR
and package operations are not established by sorting. See RPM_VERSION_REFERENCE.md.
No new lab count/coverage, host learner execution, dependency or release is added.


M124 MAIN complete regression:1653PASS/0FAIL/11ignored across127 harnesses.
The full run recorded434 memory samples: minimum21.26GiB available, peak some/full
PSI avg10 both0.18%, OOM delta0, swap-in117pages and swap-out0. This is ordinary
workload evidence, not a memory guarantee. The runner used2 Cargo jobs /2 test
threads and project-disk temporary/target directories. MAIN Python checks after
integrating the independently reviewed fixed NVR reference fixture pass211/211.
The separate guarded reference capture is still pending; no agreement claim yet.

M125 first controller gate compiled, then1/6 event groups passed. Four hit a
strict bounded reveal of a control extending3px below its actual child viewport;
the separate chooser test left Settings open. Exact diagnostics remain private.
The source author is investigating metric/layout versus event fixture causes;
no assertion, visibility bound, state transition or original smoke pacing was
relaxed. The controller/renderer remains private and supplies no normal NAT lab.


M124 final MAIN quality follow-up: all11 private-evidence checks passed explicitly
under the guarded runner, including actual9 histories, backup/restore, selected
reset, NAT independent context and public9/current comparison. MAIN all-target
Clippy passes9.08s and211 Python workflow checks pass7.797s. No private historical
capture or outcome was regenerated. Native teaching review remains the next gate.
See SHELL_TEACHING_AMENDMENTS.md for the bounded correction and saved-work behavior.


## 2026-10-26 — M124 actual native review and M126 fixed-NVR evidence

Actual Fedora review passes all18 existing assignments across the five affected
units: text4, reusable4, file-batches3, arrays4, case3. The same final development
ELF has SHA2565b170f94f2318bc912528f6c5918a5ad0c19994ab59d5bf094c864286f62af62;
text was rerun after the shared capture correction to keep this identity consistent.
Private runs: text `run-s94fzs_s`23PNGs; reusable `run-2magbin4`23; batches
`run-s0y3xdu8`22; arrays `run-wvehl4cj`; case `run-ojh1jo_p`45. References and
injected app events exercise separate ungraded practice, assignments, useful wrong/
partial feedback, retry/alternates, current invalidation, resume/reset and supported
light150/dark200 views. This is not a novice study or physical-device/reader/otherOS
acceptance. An independent reviewer inspected four100% teaching images without a
material finding. No claim that every image was manually reviewed.

The first reusable native run failed after its new longer introduction: the old
capture clicked the below-viewport primary example button. Reused the existing
bounded visible-control helper; no product UI/grade/pacing redesign. New real-event
regression passes100/150/200 and the complete reusable native rerun passes. Final
all-target Clippy passes1.66s. The earlier complete1653 run precedes this additional
capture-only regression; it is not relabeled1654.

Shared alignment repeated on that same development ELF:72 captures in
`run-s1eufzs6`, actual1920×1080/946×929 at100/150/200 in light/dark. Root manually
reviewed wide100 IPv4 placeholders and VLAN columns, wide150 light VLAN, narrow200
light VLAN and dark IPv4 populated/error states. Centered single-line text and
labels, equal topology cards/connectors, numeric columns and clean stacked layouts
remain visible; scrolling keeps large text. New installed-candidate acceptance is
still required. No existing screenshot or diagnostic was removed.

M126 corrected fixed-NVR capture from78d5e1b completed40 observations:28 strict
orders agree,12 tie controls inconclusive,0 strict mismatches. All162 output hashes
and completion marker reverified. Manifest375537c30568f0104dc90202469563d62e2b5d8be0330f27ebd21e7f4139b280;
private receipt `reviews/rpm-package-foundation/version-oracle/actual-via-nvr-r2`.
No original expectation, model algorithm or wrong-domain capture was rewritten.
This is VERSION ordering only, not fullEVR/equality or package transaction proof.

M125 technical native R4 flow passes on Fedora with24 captures in `run-3_s8mmmq`,
actual946×929 and observedPPP1.15/1.725/2.3 for100/150/200. Six event groups and
all-target Clippy4.33s pass. Earlier source/runtime failures remain retained; real
scroll animation needed bounded settling, not weakened visibility or pacing.
Manual render review found a one-map viewport hiding its second field, repeated
feedback prefixes and a missing arrow glyph. R5 fixes and actual wide capture are
under source-only work before normal integration. This is not an authored NAT unit.
The reviewed three-task teaching contract explicitly bounds all worked/experiment/
repair action lists1..16 with pre-body17th rejection. No count/coverage promotion.


## 2026-10-26 — M125 native integration, still no authored NAT credit

R5 passes8 event groups, Clippy and the actual72-stage Fedora flow with24PNG in
`run-_146_j9a`. Observed1920×1080 fullscreen100 and946×929 windowed150/200,
PPP1.15/1.725/2.3. Root inspected initial and partial wide plus both focused narrow
views. One map's fields are visible together, status wording is not duplicated,
and ASCII arrows render. No model/bool/trial/pacing/strict-visibility weakening.
R1/R2 failed events, R3 oscillation trace, R4 technical native images remain retained.

Integrated13 exact renderer/controller paths into MAINc868a19, preserving the newer
five teaching amendments and Root's visible-example helper/test. All659 other
tracked files initially rehashed unchanged; explicit documentation updates follow.
MAIN UI322/0,97.54s; all-target Clippy14.21s; model/backend43/0 including actual9
history/backup/reset pass. No complete1677/full-project count is claimed: the
last whole-project1653 run predates the UI addition. No current issuer/source
fingerprint or objective status changed.4.1 remains Open; planned static/pool/
handover assignments and their teaching/practice are not authored coverage yet.
The additional Settings Close and chooser Cancel are real controls; no click-through
or hidden flag switch substitutes for them in native checks.


M125 MAIN12d87f5 is pushed. Its actual technical native confirmation passes24
captures in `run-plsqb4ho`:1920×1080/fullscreen100 and946×929/windowed150/200,
strict controls/partial/alternate/current-invalidation/reset/warm readback. MAIN UI
memory114samples min22.26GiB, some PSI peak0.36%, OOM delta0. No new candidate or
curriculum claim. Fresh nat-teaching-143 was created from that exact source; peer
accepted its first typed teaching/practice source13paths/664protected, contractR3.
First guarded lib/integration run is underway, with an exclusive cold target and
2jobs/2runtime threads. Normal count remains140 until three real tasks are authored.

RPM first app source15paths was independently reviewed before compilation. Two
medium fixes are pending: direct Session/Outcome NUL acceptance differed from JSON
reload, and PreserveInitialPackage wrongly demanded pristine initial payloads
rather than preserving supplied current state. No runtime pass is claimed yet.
Pure EVR/provides/transaction logic and all659 protected source files remain exact.
Reciprocal new-family metadata and NAT memo integration need explicit later tests.


### M127/M128 resumed guarded gates (2026-10-26)

NAT teaching source R5 (`321f6916`) has13 backend checks passing, including the
retained actual9 migration replay,32 focused native-event/memo checks passing
(7.26s), and all-target Clippy passing (8.78s). The full331 UI regression remains
in progress at this entry. Earlier failures are retained: example action JSON
used an endpoint name as a schema key, one test malformed string exceeded the
existing bound, and a cache test omitted the newly charged Lesson option. Fixes
preserve production bounds, eviction assertions and grading. Two final let-chain
lint fixes preserve hint/reset behavior. Three authored tasks remain private
drafts, unregistered and uncredited; normal140/35 is unchanged.

RPM app source R2 scoped gate121PASS/0FAIL/5ignored (library/query-view, pure
core, terminal, workspace and teaching). Independent source review corrections
for NUL validation and preserving initially customized or missing payloads are
covered. Clippy exposed test-only construction/style corrections; the first
div_ceil correction needed an explicit usize type, retained as R5. Root verified
15 owned and659 protected hashes before rerun. No production/core semantics or
quotas changed in those lint corrections. Native contract peer review requested
public borrowed inspection instead of a crate-private query view and an explicit
one-entry-per-activity memory proof. No new UI or RPM authored coverage claimed.

M127 final source R5 private full331 UI PASS98.55s. MAIN exact13-path integration
preserves661 other source files and the three intentional documentation updates;
12 default+1 explicit actual9 integration checks and32 focused UI/memo checks pass
(7.15s). The private full UI and Clippy results belong to exact same source; MAIN
confirmation is scoped, not another claimed whole-project regression. No normal
NAT registration or native teaching acceptance yet.

M128 R5 all-target Clippy PASS3.84s, scoped121PASS/0FAIL/5ignored and explicit
five private checks PASS including actual9. Native contract R2 peer-ACK resolves
public binary-inspector access and counts one exact activity entry containing both
assignment/practice reports in the complete four-context bound. MAIN integration
and renderer/content gates remain pending.

M127 complete MAIN source73b93e4 regression:1698PASS/0FAIL/12ignored across128
harnesses. Receipt reviews/nat-foundation/main-full-regression-r1/RESULT.json;
log SHA d4a72e61dad49aa1ff180915e27939a9c84cc6bcc85695efe499534ad69d82fc.
441 memory samples: min21.25GiB available, some/full PSI avg10 peak0, OOM delta0,
swapin70pages/out0. Default ignored tests are not passed-test evidence; earlier
private checks remain separately versioned. No stress test or memory guarantee.

RPM merged-source17paths/669protected is root/peer-ACK (9f8e6985) in fresh
rpm-native-140 based on73b93e4. Existing Nat typed Lesson/Learning, complete memo
key and actual capture fix are preserved. Reciprocal individually valid foreign
metadata is rejected before current credit, save/decode/prepare and Nat snapshot.
Three new mixed-context groups and one warm-memo group are still pending execution;
the first guarded cold-target model run is underway. No renderer/credit yet.

Three distinct NAT tasks passed independent source/content review. Review caught
an actual misconception: total dynamic policy capacity after static reservations
is different from globals still unallocated after active maps exist. R2 prose
fixes that distinction without changing goal semantics or expected outcomes.
Unregistered content-test staging/native workflows remain future gates;140/35
and coverage144Partial/1Supporting/1095Open remain unchanged.

M128 merged source gates pass178default/0failed/7ignored,33 native-event/memo
checks, allseven private checks explicitly and all-target Clippy37.77s. Private
exact sourcefbac26e anchors renderer development. MAIN copied17 paths after exact
73b93e4 preimages;667 other files remain exact plus two intentional status docs.
MAIN repeats178+33+7 passing checks (UI7.19s). No new authored RPM count, renderer
acceptance or package candidate. Main model memory min19.27GiB/some PSI peak0.9%,
OOM0; no sustained threshold or safety refusal.

First real unregistered NAT content gate compiles then all eight tests fail at
one shared pack admission: handover has four optional guide entries but three
step labels. Existing loader correctly rejects the mismatch. The author is
aligning the authored labels; no bounds, validator, state or expected vector
will be relaxed. The failed log is retained as m127-nat-content-first-r1.log.

NAT content R2 fixes only handover’s four step labels to match its four existing
guides. All eight content groups now PASS3.01s, including33 reviewed plans, two
valid workflows per task, separate worked/adverse/repair practice, cross-fixture
hardcode rejection and full backup/reset retention. Clippy PASS0.31s. Private
base489a044 records exact source before normal143/36 registration. Root and peer
accept the normal three-task native plan4c63b462 as source design only; actual
controller, rendered screens and packaged acceptance remain pending.

MAIN8f5e5fa is pushed. The remaining six historical private checks pass, bringing
all13 current private checks to explicit PASS across the scoped gates.211 Python
checks PASS7.827s. Coverage integrity passes1240 references:144Partial/1Supporting/
1095Open,140 normal labs. RPM gap descriptions now correctly mention implemented
command/practice backends while retaining every Open status and source fingerprint.
The objective-mapping narrative also corrects its stale no-native-NAT description.
No practical coverage promotion or new packaged acceptance is claimed.

### M127 normal143 MAIN integration and broader regression (2026-10-26)

Private final registration005964e5/dc4cb6c passes unchanged lib70+accounts17,
six-target73 model/catalog checks,35 native-widget checks, all12 explicit private
checks and Clippy3.43s. Test-only corrections preserved original42CCNA in the old
140 prefix, used child-first outer/inner scrolling with unchanged visibility caps,
and added exact current missing IDs while retaining immutable old captures.

MAIN merged51 paths from the accepted content/registration sets. Only pack.rs
and ui/tests.rs needed mechanical three-way merge to retain newer RPM fields and
the existing exhaustive State arm. One RPM test’s CURRENT count changes140→143.
The full all-target/no-fail-fast run produced1743PASS/2FAIL/13ignored in134
harnesses. Both failures were test metadata: the old116 unit projection needed
to explicitly assert/remove appended basic-nat, and the current missing-after116
set needed the three new IDs. Capture bytes, old unit fields, goals and real
records were unchanged. Both complete targets rerun16PASS/0FAIL. All13 MAIN
private checks explicitly PASS; all-target Clippy PASS20.09s. This combined
evidence is not an all-clean relabeling of the retained initial full-run log.
Memory464samples min21.42GiB, some/full PSI peak0.18%, OOM delta0.

Source catalog is143/36 (45CCNA/28LPIC-1/24LPIC-2/46Bash). Normal NAT rendered
lesson captures and a new packaged candidate are pending, so CCNA4.1 stays Open.
Latest installed acceptance remains140ddb. RPM renderer source is under revision
after first-compile private API and independent review findings; no UI/data
acceptance is claimed. The next NFS pure capability is isolated/source-only,
with no normal scenarios, engine/progress/UI registration or objective credit.

### M129 bounded NFS core integration (2026-10-26)

Added a pure in-process NFSv3/AUTH_SYS ACCESS-style model, with separate editor,
saved configuration, applied exports and live mounts. Numeric root/group squash,
owner/group/other permissions, path search and existing virtual-file reads/writes
are explicit state operations. Whole-document parser refusals and incompatible
active-mount edits are disclosed simulator limitations, not Linux behavior.
No host filesystem, mount, RPC, network, utility execution or dependency added.

Independent review caught internally tagged unit variants accepting unknown
fields and imprecise unsquashed-root wording. R3 uses strict empty-struct wire
variants and explicitly tests read-only mode denial before unsupported root
capability checks. Frozen source d6c63bfd: four owned paths,685 protected exact.
Private73 library+22 NFS checks pass; four ignored historical progress-safety
checks run explicitly and pass. All-target Clippy passes13.74s. Exact four-path
MAIN integration passes73+22 checks. Existing143 labs, progress schema and28
themes remain unchanged. All16 objective209.2 rows stay Open; only their gap
text now acknowledges the implemented foundation. No NFS learner workspace,
authored assignments, curriculum credit or packaged acceptance is claimed.
