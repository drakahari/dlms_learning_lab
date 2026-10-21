# Learning-experience revision: verification and limits

Checked 2026-10-21 on Fedora Linux 44, COSMIC Wayland desktop, x86_64, Rust 1.99.1. Native egui/eframe 0.33.3 with Glow/OpenGL. No new dependencies or system packages. Still nine authored scenarios; no installers or releases.

## Automated results

- `cargo build --locked --offline`: passed; `target/debug/dlms-learning-lab`.
- `cargo test --locked --offline`: **32 passed**, zero failures: 22 engine/content/storage/catalog tests and 10 egui UI tests.
- `cargo clippy --locked --offline --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `--validate-pack scenarios/starter-v1.json`: passed, nine scenarios.

Coverage includes incorrect/malformed answers, partial IPv4 and VLSM results, alternate valid solutions, completion at walkthrough section one, invalidation after subsequent damage, native draft resume, reset isolation, saved favorites/section/mode/history, all nine legacy version-1 attempts, retained corrupt/future saves, invalid packs/guidance, solution reachability, permission precedence/traversal, service runtime versus saved configuration/boot enablement, persistent service event history, DNS reload/serial, capacity/mounts, log filtering and shell-escape rejection. ls/id/systemctl/journalctl formatting has explicit assertions; `ls -l` versus `ls -ld` directory behavior is tested.

UI tests exercise native editor Save, Insert command without execution, Challenge withholding examples/explanations before success, native-field keyboard focus and Tab/Shift+Tab, output/input proximity, field bounds, hidden single-page pagination, developer-only 900-entry catalog, filtering and empty-result recovery. Library geometry is checked at 720, 980 and 1400 physical widths at 85%, 100%, 125% and 150% text size; standalone native workspace layouts run at 480, 720 and 1400 logical widths. Existing contexts are also resized and zoomed repeatedly to check reflow. Header and filter rectangles must fit; filter heights and row baselines match; card-action heights and alignment match. These checks caught and fixed a narrow header overflow.

## Native Fedora run and visual inspection

The real Wayland window rendered with its actual OpenGL backend. The native smoke harness injects events through egui’s native input hook and asserts:

- Typing a malformed IPv4 answer updates the actual native field; the wrong answer fails.
- Four correct values complete the objective while walkthrough position remains section one. Breaking a value invalidates success.
- VLSM renders three editable allocation rows.
- Terminal text plus Enter changes the simulated log result; Tab/Shift+Tab move and return focus.
- Clicking Save configuration updates only the modeled file; restart and enable then complete the service objective.
- A real resize from the 1268×632 tiled framebuffer to 2560×1440 full-screen and back succeeds. COSMIC constrained requested floating sizes, so full-screen transitions force real native resize events.
- The tiled window at 150% text size uses stacked workspaces; library filters fit without clipping, and scrolling reaches the native answer fields and Check work.

Application-only screenshots were captured **before editing** using the previous binary and **after editing** using the new binary. Captures were visually inspected, including aligned library controls, card actions, native wrong answers, section-one completion, VLSM, service editor/terminal, the 150% filter wrap, and scrolled stacked native fields.

| View | Local capture (ignored by Git) |
| --- | --- |
| Library before | `screenshots/before/01-library.png` |
| Workspace before | `screenshots/before/02-workspace.png` |
| Full-screen workspace before | `screenshots/before/03-resized-125-percent.png` |
| Library after | `screenshots/after/01-library.png` |
| Wrong IPv4 answer | `screenshots/after/02-ipv4-wrong.png` |
| Completion at section one | `screenshots/after/03-complete-at-section-one.png` |
| VLSM table | `screenshots/after/04-vlsm-table.png` |
| Service editor and terminal | `screenshots/after/05-service-editor.png` |
| Service completion | `screenshots/after/06-service-complete.png` |
| Library at 150% | `screenshots/after/07-library-150-percent.png` |
| Stacked workspace at 150% | `screenshots/after/08-stacked-ipv4-150-percent.png` |
| Scrolled native inputs at 150% | `screenshots/after/09-stacked-inputs-150-percent.png` |
| Developer catalog | `screenshots/after/10-developer-catalog.png` |

Screenshots and smoke progress are local and uncommitted. `--ui-smoke` starts fresh isolated progress under `/tmp/dlms-ui-v2-smoke`. Normal learner progress was not used by the harness. The checked-in version-1 compatibility fixture is explicitly synthetic, not a learner save or recording.

## Compatibility and limits

- IDs, scenario versions, goals and three-section walkthrough lengths are preserved. Old answer/allocation values, states, favorites, mode, hints, transcripts and positions load without a reset or read-time rewrite. New service failure/journal fields use safe defaults. Old history is not reconstructed. Malformed native drafts are accepted as saved draft state but fail grading; malformed authored initial allocations are still rejected.
- New service fields are written on later saves. Running the older binary against those newer saves is not supported; the earlier loader would reject the extra fields. No general migration framework was added.
- Windows/macOS were not built or run in this pass. Physical keyboard/mouse, IME, screen-reader, high-contrast OS theme and multi-monitor DPI acceptance were not performed. Input tests inject egui events; they do not claim physical-device testing. AccessKit remains enabled.
- Small windows require vertical scrolling. Output has its own vertical/horizontal scrolling, adjacent input and feedback. The UI does not claim full Linux/systemd/Cisco emulation. Numeric IDs, file sizes/dates and journal metadata are modeled teaching values; see CONTENT.md.
- External pack installation, concurrent-writer locking, installers, releases and full curriculum coverage remain outside this pass. Corrupt/incompatible progress stays preserved with a read-only explanation. Existing safe temporary-file replacement is unchanged.
- License selection remains the owner’s pending decision.
