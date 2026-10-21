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

Choose a track, search or filter by topic, difficulty, completion, and duration. Favorites and recent/continue provide focused shortcuts. The library paginates at 12 entries. The opt-in developer checkbox generates 900 **synthetic metadata entries** for navigation testing; they cannot launch and are not finished labs.

Learn mode shows one step at a time. Challenge hides the steps. Type `help` or expand **Supported commands and simulation limits** before using the simulated terminal. Submit with Enter or Run. **Check work** reads current state; later damage removes completion. Tab/Shift+Tab move focus, Enter/Space activate controls, and the text scale slider adjusts readability. The command bar and Check work stay visible while lesson content scrolls.

State, current step, mode, hints, transcript and favorites save locally after actions. Back to library preserves the attempt. Reset requires an inline confirmation and resets only that lab’s attempt. Reset keeps favorites and other labs.

Storage uses the OS local application-data directory, normally `$XDG_DATA_HOME/dlmslearninglab/progress-v1.json` or `~/.local/share/dlmslearninglab/progress-v1.json` on Linux (the exact directory follows the `directories` crate). Saves replace the file through a synced temporary file in the same directory. Unsupported/corrupt saves are preserved and the session becomes read-only with a visible explanation. Concurrent application instances are not supported; last writer wins.

The public tree excludes progress, recordings, screenshots, credentials, environment files and local tool configuration. The simulator never passes learner commands to OS APIs. Storage paths come from application configuration, not the simulated terminal.

See [architecture and authoring](docs/ARCHITECTURE.md), [sources and simplifications](docs/CONTENT.md), and [verification and limits](docs/VERIFICATION.md).

## Status and license

Native Fedora development prototype. Windows and macOS use the same native Rust stack but have **not** been runtime accepted. No releases, installers, or distribution packages have been published.

**License decision pending the repository owner.** No project license has been selected; public visibility does not grant a reuse license. Dependency licenses remain their respective authors’ terms.
