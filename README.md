# DLMS Learning Lab

A native, offline Rust desktop prototype for networking and Linux practice. Thirty authored scenarios: the accepted nine across CCNA, LPIC-1 and LPIC-2, plus seven shell foundations, four shell composition and four reusable-function/safe-composition assignments, plus six Linux file-management foundations. Illustrative practice, **not full exam coverage**. Independent of DLMS_next and the old dlms_lab.

## Open the Fedora development build

From this repository, after building:

```sh
./target/debug/dlms-learning-lab
```

The executable embeds its scenario pack and fonts; it can run outside this directory. No browser, web server, VM, container, account, profile path or infrastructure setup is required. A working graphical desktop and OpenGL driver are needed.

Try these first:

1. **Read an IPv4 address** — subnet boundaries and a second workstation.
2. **Share a report safely** — permission repair with multiple valid solutions.
3. **Bring the service back** — diagnose configuration, restart, and enable separately.

## Build and verify

Use stable Rust (tested with 1.99.1 on Fedora 44):

```sh
cargo build --locked
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo fmt --all -- --check
./target/debug/dlms-learning-lab --validate-pack scenarios/starter-v1.json
```

This development session used an isolated Rust toolchain under `/tmp`; it made no system package changes. If Rust is absent from PATH while that toolchain remains available:

```sh
CARGO_HOME=/tmp/dlms-cargo RUSTUP_HOME=/tmp/dlms-rustup /tmp/dlms-cargo/bin/cargo build --locked --offline
```

The checked-in Cargo.lock pins dependencies. First-time builds need crates.io access; the application operates offline. On another Fedora machine, inspect missing native dependencies before adding packages. This machine already supplied the C linker, X11/Wayland and OpenGL runtime libraries; no extra development packages were needed. egui uses bundled fonts and dynamically loads xkbcommon.

## This milestone

The accepted native workspace design remains the basis of all labs. The header adds a native theme picker with samples under **Settings → Appearance → Theme samples**: all five stock DLMS palettes and all 22 official Omarchy v4.0.4 palettes, plus the existing Navy default (**28 choices**). Selection and text size persist independently of progress. No desktop settings are changed. Exact inventory, source revision, contrast adaptation and MIT notices: [themes](docs/THEMES.md).

The **LPIC-1** course now starts with six [file-management foundations](docs/LINUX_FILES.md): navigation, quoted names, copy versus move, precise cleanup, creation defaults and a handover capstone. Introductions and isolated editable terminal practice teach the commands before full state-graded assignments. Existing permission/log labs remain available.

The new **Bash/Shell** track teaches reliable output, quoted arguments, input, guards/status, report redirection, multi-file loops and a safe-report capstone, followed by separate output channels, intersecting pipeline filters, end-of-input repair resilient batch auditing, reusable functions, per-item return status, argument consumption and a preflight bundle capstone. Scripts are written in a native editor and run against 3–7 deterministic sets of sample inputs with output/file-state grading. Run/Stop, case inspection and specific retry feedback stay together. Learn/Challenge retain the same assignments. Learn adds a skippable, reopenable introduction and a separately saved **Try example** workspace, with different example values and no completion credit. Challenge opens independent assignment work. Walkthroughs, progressive hints and worked solutions remain optional. See the [foundations teaching audit](docs/SCRIPT_TEACHING.md) and [composition content review](docs/SHELL_COMPOSITION.md) and [reusable-interface unit](docs/SHELL_REUSABLE.md). Drafts survive resume; edits invalidate old results. Tab indents the code editor, Escape releases its focus, and Ctrl+Enter runs.

The in-process interpreter implements a documented [shell subset](docs/SHELL_SUBSET.md), with a real lexer/parser/VM, virtual files and bounded cooperative execution. It never passes learner input to Bash, a subprocess, the host filesystem or network. Real Bash appears **only** in the disposable development-oracle fixtures. No new dependencies were added.

The [curriculum plan](docs/CURRICULUM.md) and [165-row coverage map](docs/CURRICULUM.csv) record current official versions, sources/check dates, implemented slices, distinct planned assignments, prerequisites, capabilities and fidelity gaps. Upcoming CCNA v2.0 is tracked separately from currently offered v1.1. Planned scenarios are not catalog placeholders or full-exam preparation claims.

## Use

Choose a track, unit and topic, or search titles/goals. Start/continue course opens the first unfinished assignment; experienced learners can open any lab. Unit prerequisites are advice, not locks. Completion stays within the current unit. Favorites and Recent / continue keep navigation focused. Difficulty, completion and duration live under **More filters**, with an active-filter summary when collapsed. Cards have aligned actions; pagination appears only above 12 results. `--developer` exposes a clearly marked 900-item synthetic catalog for navigation testing; those entries are not playable curriculum.

Every lab keeps its complete objective and appropriate workspace together. IPv4 has the approved compact five-field assignment, a two-workstation topology and one **Check configuration** action. Workstation 2 accepts any different usable address. Its topology shows the typed address and checked status, which clears when that answer changes. Progressive hints and a binary explanation are optional; **network bits | host bits** and the all-zero/all-one host rules are explicit when requested. Numerical variants and a separate host-bit activity remain under optional practice, without adding lab titles. /31 and /32 remain excluded.

VLSM retains an allocation table with specific row feedback and alternate valid plans. Networking, permission, traversal and log labs retain simulated terminals. Service, DNS and storage retain investigation and terminal verification alongside native configuration editors. **Save configuration** changes only the simulated saved file: restarting/reloading/mounting is separate. Check work evaluates saved state, not an unsaved draft. Editor drafts and unfinished commands survive leaving and reopening the lab.

Learn and Challenge keep the same assignments. Learn offers a short starting point; lengthy walkthroughs, commands, worked reasoning, references and advanced practice are optional. Challenge omits the starting examples and hides the walkthrough. Hints/references and explicitly requested reasoning remain available. Insert command only fills input; Run/Enter executes it in the simulator. Unknown commands explain the supported subset. `test-access`, `config set`, zone-editing helpers and the client-address shortcut are simulator controls, not full Linux commands.

Completion comes from current state and does not require visiting walkthrough sections. A later edit or command that breaks the state removes current success. Native forms, editors, terminal input/output and feedback stay together. Narrow windows and larger text scroll cleanly; fonts are not shrunk to fit. Tab/Shift+Tab navigate controls and Enter runs terminal commands. The header controls text size.

Back to library retains the selected track, search and filters. Reset asks for confirmation and clears only that attempt, including drafts, transcript and numerical practice; favorites and other labs remain. Local state saves after edits using a synced temporary file and atomic replacement. Corrupt/future saves are retained and opened read-only. A profile writer lock prevents concurrent app copies from silently replacing each other’s drafts. Settings → Your data provides checked backup export/restore; see [progress recovery](docs/PROGRESS_RECOVERY.md).

## Saved-progress compatibility

Existing version-1 saves load without a reset or read-time rewrite. All nine IDs, scenario versions, previous answers, walkthrough positions, hints, transcripts and optional numerical-practice data are retained. The complete IPv4 assignment adds an optional `second` field to the typed state. When reopening a completed legacy four-answer analysis, its earlier credit is retained separately and the second-workstation field is left blank. The library identifies the new requirement as remaining; it never supplies a learner answer. Values supplied by an older compatibility version remain saved but require a new explicit check before counting toward the complete assignment. Incomplete attempts retain every draft.

Optional versioned `workspace` data stores unfinished commands, editor drafts and check snapshots. Before the first upgraded write, an existing pre-workspace save is copied byte-for-byte to `progress-v1.before-workspace.json` beside the progress file. The backup is not replaced by later saves. Storage uses the OS local application-data directory (normally `$XDG_DATA_HOME/dlmslearninglab` or `~/.local/share/dlmslearninglab` on Linux). New metadata requires this newer binary; downgrade compatibility is not claimed.

Progress, backups, screenshots and recordings remain local and excluded from the public tree. No learner input is passed to an OS shell, process, host filesystem command or network operation. The compatibility fixtures and native captures use synthetic data.

Development checks also include `python3 tools/verify_palettes.py` and, after `cargo build --locked --example shell_probe`, `python3 tools/bash_reference.py`. The latter executes only checked-in controlled fixtures in temporary directories.

See [architecture and authoring](docs/ARCHITECTURE.md), [content sources and simplifications](docs/CONTENT.md), and [verification and limits](docs/VERIFICATION.md).

## Status and license

Native Fedora development prototype. Windows and macOS use the same native Rust stack but have **not** been runtime accepted. No releases or distribution packages have been published. Local development installation candidates are prepared separately; see [installation and platform gates](docs/INSTALLATION.md).

Original code and authored curriculum are licensed under [MIT](LICENSE), approved by the owner on 2026-10-22. Third-party licenses and attribution remain in [NOTICE](NOTICE) and the palette notices. Candidates include target-specific dependency/font/standard-library notices and original source archives. macOS notice provenance and actual cross-platform installation/runtime acceptance remain explicit gates.
