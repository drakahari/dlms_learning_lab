# DLMS Learning Lab

A native, offline Rust desktop prototype for networking and Linux practice. Nine authored scenarios across CCNA, LPIC-1 and LPIC-2. Illustrative practice, **not full exam coverage**. Independent of DLMS_next and the old dlms_lab.

## Open the Fedora development build

From this repository, after building:

```sh
./target/debug/dlms-learning-lab
```

The executable embeds its scenario pack and fonts; it can run outside this directory. No browser, web server, VM, container, account, profile path or infrastructure setup is required. A working graphical desktop and OpenGL driver are needed.

Try these first:

1. **Read an IPv4 address** — guided subnet boundaries.
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

## Use

Choose a track, search or filter by topic, difficulty, completion, and duration. Favorites and recent/continue provide focused shortcuts. The library shows aligned filters and consistent card actions, uses two card columns when space allows, and paginates only when more than 12 results match. Start with `--developer` to expose the secondary developer option for 900 **synthetic metadata entries**; these previews cannot launch and are not finished labs.

IPv4 uses four labeled native answer fields and a subnet bit diagram. VLSM uses an allocation table. Linux skills use a simulated terminal with scrollable output; the service lab also has a simulated `/etc/atlas.conf` editor. Save that editor explicitly before leaving, then restart the service to apply the saved port.

Learn mode shows one section at a time with the reason for the action, separate command examples, and **Insert command** buttons that populate input without executing it. Challenge hides the walkthrough and examples; hints remain opt-in. `test-access`, `config set`, DNS editing helpers and the client address shortcut are labeled simulator-only controls. Command references, limitations and sources are secondary disclosures.

**Check work** tests current values. All objectives passing immediately shows one completion panel with results, a worked explanation and Back to library / Next lab actions, even at walkthrough section one. Later damage removes current completion. Walkthrough position is optional guidance, not a completion requirement. Wide workspaces place instruction beside practice; narrow windows stack them with scrolling. Terminal input sits directly below its output instead of being pinned to the window bottom. Tab/Shift+Tab move focus, Enter submits terminal commands, and the header’s percentage selector controls text size.

State, current section, mode, hints, transcript and favorites save locally after actions. IPv4/VLSM fields save as typed, including unfinished or malformed drafts. Back to library preserves the attempt. Reset requires an inline confirmation and resets only that lab’s attempt. Reset keeps favorites and other labs.

Storage uses the OS local application-data directory, normally `$XDG_DATA_HOME/dlmslearninglab/progress-v1.json` or `~/.local/share/dlmslearninglab/progress-v1.json` on Linux (the exact directory follows the `directories` crate). Saves replace the file through a synced temporary file in the same directory. Unsupported/corrupt saves are preserved and the session becomes read-only with a visible explanation. Concurrent application instances are not supported; last writer wins.

Existing version-1 progress loads without a reset or file rewrite. Scenario IDs, versions and three-section positions are unchanged. Optional service failure/journal fields default safely for old saves; old attempts have no fabricated journal history. Downgrading to the earlier binary after new service fields are saved is not supported.

The public tree excludes learner progress, recordings, screenshots, credentials, environment files and local tool configuration. The compatibility fixture in `tests/fixtures/` is synthetic test data. The simulator never passes learner commands to OS APIs. Storage paths come from application configuration, not the simulated terminal.

See [architecture and authoring](docs/ARCHITECTURE.md), [sources and simplifications](docs/CONTENT.md), and [verification and limits](docs/VERIFICATION.md).

## Status and license

Native Fedora development prototype. Windows and macOS use the same native Rust stack but have **not** been runtime accepted. No releases, installers, or distribution packages have been published.

**License decision pending the repository owner.** No project license has been selected; public visibility does not grant a reuse license. Dependency licenses remain their respective authors’ terms.
