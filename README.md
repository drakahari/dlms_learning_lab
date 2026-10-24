# DLMS Learning Lab

A native, offline Rust desktop prototype for networking, Linux administration and scripting. **113 authored assignments:** 35 CCNA,21 LPIC-1,17 LPIC-2 and40 Bash/Shell. Illustrative practice, **not full exam coverage**. Independent of DLMS_next and the old dlms_lab.

## Open the Fedora development build

From this repository, after building:

```sh
./target/debug/dlms-learning-lab
```

The executable embeds its scenario pack and fonts; it can run outside this directory. No browser, web server, VM, container, account, profile path or infrastructure setup is required. A working graphical desktop and OpenGL driver are needed.

Try these first:

1. **Read an IPv4 address** — start with Introduction, try a separate example, then solve the complete workstation assignment.
2. **Share a report safely** — permission repair with multiple valid solutions.
3. **Bring the service back** — learn service basics, try a separate editor/terminal example, then diagnose, repair, restart and enable.

## Build and verify

Use stable Rust (tested with 1.99.1 on Fedora 44):

```sh
python3 tools/dev.py build --locked --offline
python3 tools/dev.py test --locked --offline --all-targets
python3 tools/dev.py clippy --locked --offline --all-targets -- -D warnings
cargo fmt --all -- --check
python3 tools/dev.py native ./target/debug/dlms-learning-lab --validate-pack "$PWD/scenarios/starter-v1.json"
```

Use the [project build workflow](docs/BUILD_WORKFLOW.md): two Cargo jobs, two test threads, one shared heavy-work lock across worktrees, disk-backed temporary/cache data and lightweight memory-pressure observations. The local isolated Rust toolchain was copied into ignored `.build-artifacts/toolchains`; its original temporary copies were retained. No system packages, global shell settings, mounts, GPU reservations, swap or model settings were changed. First-time developer toolchain preparation is described in that guide; installed learners need none of it.

The checked-in Cargo.lock pins dependencies. First-time builds need crates.io access; the application operates offline. On another Fedora machine, inspect missing native dependencies before adding packages. This machine already supplied the C linker, X11/Wayland and OpenGL runtime libraries; no extra development packages were needed. egui uses bundled fonts and dynamically loads xkbcommon.

## This milestone

Four [Ethernet learning and recovery assignments](docs/MAC_LESSONS.md) now follow
VLAN foundations: build directed paths, repair an older table, reason about
source refresh/expiry and hand over two groups across three switches. Each has
an introduction, separate runnable practice and a full native assignment.
Model checks and native component flows pass; the exact next installed-candidate
all-four run is still a delivery gate. The last packaged Fedora candidate has109
labs; its identity and verified Windows candidate are in INSTALLATION.md.


The [OSPF settings and route-cost unit](docs/OSPF_LESSONS.md) adds four native
configuration assignments: repair router identities/process enablement, match
P2P parameters, restore equal-cost paths and hand over a passive user LAN.
Different practice networks stay ungraded. Current parameter/route calculations
are modeled; live neighbor exchange, DR/BDR, convergence and real packet delivery
remain unimplemented.


The accepted native workspace design remains the basis of all labs. The header adds a native theme picker with samples under **Settings → Appearance → Theme samples**: all five stock DLMS palettes and all 22 official Omarchy v4.0.4 palettes, plus the existing Navy default (**28 choices**). Selection and text size persist independently of progress. No desktop settings are changed. Exact inventory, source revision, contrast adaptation and MIT notices: [themes](docs/THEMES.md).

The original **Bring the service back** task now explains background services, saved configuration versus the current run, and startup at boot before independent diagnosis. A separate editor/terminal example keeps its own state and history; inserting a command never executes it. [Service teaching](docs/SERVICE_TEACHING.md).

The **LPIC-1** course now starts with six [file-management foundations](docs/LINUX_FILES.md): navigation, quoted names, copy versus move, precise cleanup, creation defaults and a handover capstone. Introductions and isolated editable terminal practice teach the commands before full state-graded assignments. Existing permission/log labs remain available.

The **Persistent mounts and UUIDs** unit adds four [storage assignments](docs/STORAGE_LESSONS.md): stable identity, saved versus live access policy, a manual-only volume, and a multi-fault handover. A native configuration editor and simulated administrator terminal keep diagnosis and verification together. Fresh-session tests change only virtual device names; no actual mount, boot or device operation occurs.

The **Processes and priority** unit adds four [Linux terminal assignments](docs/PROCESS_LESSONS.md): choose the right owner before pausing, finish pending termination, change an existing batch process’s priority, and repair a process-family handover. Teaching and practice use a separate table; all processes and signals stay virtual.

The existing subnet and VLSM labs now teach address groups, prefixes/masks, usable hosts, capacity, alignment and non-overlap before their unchanged full assignments. Learn has skippable introductions and separate editable examples; existing attempts reopen their saved assignment with Introduction available. [Teaching and validation](docs/ADDRESSING_TEACHING.md).

The **CCNA** course adds six [IPv6 static-network foundations](docs/IPV6_LESSONS.md): unique workstation addressing, prefix/gateway repair, interface-scoped next hops, route specificity, backup failover and a branch capstone. A small topology and selected-device native address/route editor keep the full assignment visible. Examples have independent saved drafts; Check configuration applies and tests only virtual state. This is narrow partial IPv6/static-routing practice, not complete CCNA coverage.

The **LLDP neighbors, inventory and handover** unit adds four [discovery assignments](docs/DISCOVERY_LESSONS.md): repair a receiving direction, refresh changed inventory, stop one transmitter while retaining reception, and complete a multi-link handover. Native settings, explicit simulated advertisements and current cache checks stay together. CDP, IOS and real packet handling remain outside this bounded model.

The **Filename selection and safe batch reports** unit adds three [script assignments](docs/SHELL_FILE_BATCHES.md): a changing-file manifest, cross-file incident collection and a collision-safe handover. Separate examples teach selection before writing; scripted filename matching stays inside the virtual file map.

The **VLAN groups, trunks and handover** unit adds four [native switching assignments](docs/VLAN_LESSONS.md): access membership, trunks versus active/allowed groups, native classification and a multi-fault handover. Edit native port tables and inspect simulated broadcasts; no Cisco terminal, STP or real network is involved. Each lesson has isolated examples and tests both working groups and separation.

The new **Bash/Shell** track teaches reliable output, quoted arguments, input, guards/status, report redirection, multi-file loops and a safe-report capstone, followed by separate output channels, intersecting pipeline filters, end-of-input repair resilient batch auditing, reusable functions, per-item return status, argument consumption and a preflight bundle capstone. Scripts are written in a native editor and run against 3–12 deterministic sets of sample inputs with output/file-state grading. Run/Stop, case inspection and specific retry feedback stay together. Learn/Challenge retain the same assignments. Learn adds a skippable, reopenable introduction and a separately saved **Try example** workspace, with different example values and no completion credit. Challenge opens independent assignment work. Walkthroughs, progressive hints and worked solutions remain optional. See the [foundations teaching audit](docs/SCRIPT_TEACHING.md) and [composition content review](docs/SHELL_COMPOSITION.md) and [reusable-interface unit](docs/SHELL_REUSABLE.md). Drafts survive resume; edits invalidate old results. Tab indents the code editor, Escape releases its focus, and Ctrl+Enter runs.

The **Command output, reliable decisions and report previews** unit adds four [capture assignments](docs/SHELL_CAPTURE_LESSONS.md): intact multiline lists, preserved evidence, meaningful helper status and a guarded preview/save report with an inspection ledger. Separate examples teach before full editor tasks; multiple behavioral cases accept equivalent supported solutions. A nonzero result can correctly mean that an investigation found a problem. Captures remain bounded simulated execution; the terminal does not gain shell expansion.

The **Clear dispatch and safe publication** unit adds three [case-dispatch assignments](docs/SHELL_CASE_LESSONS.md): a reliable report-mode interface, prioritized record routing and a validated preview/apply publication. Separate examples teach first-match decisions and error handling before complete editor assignments. Multiple output, exit-status and virtual-file cases accept supported alternatives, including conditional solutions.

The in-process interpreter implements a documented [shell subset](docs/SHELL_SUBSET.md), with a real lexer/parser/VM, virtual files and bounded cooperative execution. It never passes learner input to Bash, a subprocess, the host filesystem or network. Real Bash appears **only** in the disposable development-oracle fixtures. No new dependencies were added.

The [curriculum plan](docs/CURRICULUM.md) and [165-row coverage map](docs/CURRICULUM.csv) record current official versions, sources/check dates, implemented slices, distinct planned assignments, prerequisites, capabilities and fidelity gaps. Upcoming CCNA v2.0 is tracked separately from currently offered v1.1. The [granular evidence/gap ledger](docs/OBJECTIVE_MAPPING.md) pins1240 current objective/knowledge/tool references with explicit partial evidence and missing capabilities. Planned scenarios are not catalog placeholders or full-exam preparation claims.

## Use

Choose a track, unit and topic, or search titles/goals. Start/continue course opens the first unfinished assignment; experienced learners can open any lab. Unit prerequisites are advice, not locks. Completion stays within the current unit. Favorites and Recent / continue keep navigation focused. Difficulty, completion and duration live under **More filters**, with an active-filter summary when collapsed. Cards have aligned actions; pagination appears only above 12 results. `--developer` exposes a clearly marked 900-item synthetic catalog for navigation testing; those entries are not playable curriculum.

Every lab keeps its complete objective and appropriate workspace together. IPv4 has the approved compact five-field assignment, a two-workstation topology and one **Check configuration** action. Workstation 2 accepts any different usable address. Its topology shows the typed address and checked status, which clears when that answer changes. Progressive hints and a binary explanation are optional; **network bits | host bits** and the all-zero/all-one host rules are explicit when requested. Numerical variants and a separate host-bit activity remain under optional practice, without adding lab titles. /31 and /32 remain excluded.

VLSM retains an allocation table with specific row feedback and alternate valid plans. Networking, permission, traversal and log labs retain simulated terminals. Service, DNS and storage retain investigation and terminal verification alongside native configuration editors. **Save configuration** changes only the simulated saved file: restarting/reloading/mounting is separate. Check work evaluates saved state, not an unsaved draft. Editor drafts and unfinished commands survive leaving and reopening the lab.

Learn and Challenge keep the same assignments. Learn offers a short starting point; lengthy walkthroughs, commands, worked reasoning, references and advanced practice are optional. Challenge omits the starting examples and hides the walkthrough. Hints/references and explicitly requested reasoning remain available. Insert command only fills input; Run/Enter executes it in the simulator. Unknown commands explain the supported subset. `test-access`, `config set`, zone-editing helpers and the client-address shortcut are simulator controls, not full Linux commands.

Completion comes from current state and does not require visiting walkthrough sections. A later edit or command that breaks the state removes current success. Native forms, editors, terminal input/output and feedback stay together. Narrow windows and larger text scroll cleanly; fonts are not shrunk to fit. Tab/Shift+Tab navigate controls and Enter runs terminal commands. The header controls text size.

Back to library retains the selected track, search and filters. Reset asks for confirmation and clears only that attempt, including drafts, transcript and numerical practice; favorites and other labs remain. Local state saves after edits using a synced temporary file and atomic replacement. Corrupt/future saves are retained and opened read-only. A profile writer lock prevents concurrent app copies from silently replacing each other’s drafts. Settings → Your data provides checked backup export/restore; see [progress recovery](docs/PROGRESS_RECOVERY.md).

The [Bash scope unit](docs/SHELL_SCOPE.md) adds three practical debugging assignments: protect caller settings, trace nested filter visibility and repair a review builder that changes the wrong file. Explicitly Bash-specific `local` is taught before use; alternative supported repairs pass the same behavior tests.

Four [text-record lessons](docs/SHELL_TEXT.md) teach delimited field selection, stable unique manifests, accurate grouped counts and a duplicate-identity audit. Scripts use the same bounded simulator, separate examples and behavior grading.

Six [forward-DNS lessons](docs/DNS_LESSONS.md) teach dual-stack host records, alias-loop repair, primary/backup mail discovery, safe address retirement, stale secondary diagnosis and a combined service handover. They use a native record editor and in-memory lookups, separate ungraded examples and current-state grading. Full BIND, reverse zones, recursion and real transfers remain outside this subset.

Shell evaluator upgrades retain earlier output and credit separately from current success. Run rechecks the saved draft; unsupported behavior cannot reuse earlier completion. The first upgraded write keeps an exact non-overwriting backup. [Recovery details](docs/PROGRESS_RECOVERY.md).

## Saved-progress compatibility

Existing version-1 saves load without a reset or read-time rewrite. All nine IDs, scenario versions, previous answers, walkthrough positions, hints, transcripts and optional numerical-practice data are retained. The complete IPv4 assignment adds an optional `second` field to the typed state. When reopening a completed legacy four-answer analysis, its earlier credit is retained separately and the second-workstation field is left blank. The library identifies the new requirement as remaining; it never supplies a learner answer. Values supplied by an older compatibility version remain saved but require a new explicit check before counting toward the complete assignment. Incomplete attempts retain every draft.

Optional versioned `workspace` data stores unfinished commands, editor drafts and check snapshots. Before the first upgraded write, an existing pre-workspace save is copied byte-for-byte to `progress-v1.before-workspace.json` beside the progress file. The backup is not replaced by later saves. Storage uses the OS local application-data directory (normally `$XDG_DATA_HOME/dlmslearninglab` or `~/.local/share/dlmslearninglab` on Linux). New metadata requires this newer binary; downgrade compatibility is not claimed.

Progress, backups, screenshots and recordings remain local and excluded from the public tree. No learner input is passed to an OS shell, process, host filesystem command or network operation. The compatibility fixtures and native captures use synthetic data.

Development checks also include `python3 tools/dev.py tool verify_palettes.py` and, after `python3 tools/dev.py build --locked --offline --example shell_probe`, `python3 tools/dev.py tool bash_reference.py`. The latter executes only checked-in controlled fixtures in private disk-backed temporary directories under the shared guard.

See [architecture and authoring](docs/ARCHITECTURE.md), [content sources and simplifications](docs/CONTENT.md), and [verification and limits](docs/VERIFICATION.md).

## Status and license

Native Fedora development prototype. Windows and macOS use the same native Rust stack but have **not** been runtime accepted. No releases or distribution packages have been published. Local development installation candidates are prepared separately; see [installation and platform gates](docs/INSTALLATION.md).

Original code and authored curriculum are licensed under [MIT](LICENSE), approved by the owner on 2026-10-22. Third-party licenses and attribution remain in [NOTICE](NOTICE) and the palette notices. Candidates include target-specific dependency/font/standard-library notices and original source archives. macOS notice provenance and actual cross-platform installation/runtime acceptance remain explicit gates.

Four [counter and totals assignments](docs/SHELL_COUNTS.md) add sequential virtual-file input, stable record numbering, whole-cent costs, guarded division/remainder and a preflight multi-file count capstone. Small valid integer inputs and read-only active-source limitations are explicit; no arbitrary numeric validation or floating-point behavior is claimed.

Four [access-policy assignments](docs/LINUX_ACCESS.md) continue LPIC-1 file foundations: group sharing, directory write/search versus browsing, updating another owner's existing files, and a metadata-preserving handover. The native terminal keeps the complete assignment visible with separate saved examples and optional guidance.

Four native IPv4 access-policy assignments now add source filtering, ordered service rules, direction diagnosis and a two-router handover. See [ACL_LESSONS.md](docs/ACL_LESSONS.md) for exact verification and limits. Source additions are separate from the older installed Fedora candidate listed in INSTALLATION.md.

Four [local-account assignments](docs/ACCOUNT_LESSONS.md) add explicit onboarding/template homes, exact membership repair, evidence-preserving retirement and a stable-GID team handover. Native simulated administrator terminals retain independent examples, current-state checks and supported alternate command sequences. No host accounts, passwords, authentication or login sessions are touched.
