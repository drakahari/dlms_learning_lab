# Verification and known limits

Checked 2026-10-21 on Fedora Linux 44, COSMIC Wayland desktop, x86_64, Rust 1.99.1. Native egui/eframe 0.33.3 with Glow/OpenGL, X11 and Wayland support. No system packages were installed. Rust, rustfmt and Clippy were downloaded into an isolated `/tmp` toolchain; runtime graphics libraries were already present.

## Results

- `cargo build --locked --offline`: passed; runnable `target/debug/dlms-learning-lab`.
- `cargo test --locked --offline`: **18 passed**, zero failures (16 core/content/storage/catalog tests, two actual egui layout/catalog tests).
- `cargo clippy --locked --offline --all-targets -- -D warnings`: passed.
- `cargo fmt --all -- --check`: passed.
- `--validate-pack scenarios/starter-v1.json`: passed, **nine authored scenarios**.
- Native Fedora smoke: passed using the real Wayland window and renderer. Injected egui text input, Enter submission, Tab and Shift+Tab focus transitions all assert successfully. Native resize verified from a 1268×632 tiled framebuffer to 2560×1440 full-screen at 125% scale and back. COSMIC constrained requested floating dimensions, so the test uses a brief full-screen transition to force real resize events. An XWayland smoke run also rendered and passed keyboard/scaling checks; it had the same compositor sizing constraint.
- Headless egui UI layout tests: 720, 980 and 1400 logical widths, command-bar fit/focus, 900-entry pagination and empty-search recovery. Native screenshot inspection confirmed readable dark text contrast, visible focus, wrapped lesson text and pinned terminal/actions. The 125% native capture was visually inspected. Headless tests exercise layout code; they do not replace device-level desktop acceptance.

Tests exercise unsolved/success paths for all nine labs, alternate VLSM and permission/ownership solutions, capacity and overlap failures, subnet arithmetic across all IPv4 prefixes, owner precedence and directory traversal, service loaded configuration versus boot enablement, DNS serial/reload, writable mounts and safe cleanup, log filtering/pipelines, shell-escape rejection, state-based completion invalidation, saved broken-state resume, selected-attempt reset with favorites/other attempts retained, malformed/future saves retained, invalid pack/schema/IDs/objectives and unreachable solutions, and synthetic catalog filtering.

`--ui-smoke` writes only the application framebuffer to ignored local files:

- `screenshots/01-library.png`
- `screenshots/02-workspace.png`
- `screenshots/03-resized-125-percent.png`
- `screenshots/04-synthetic-catalog.png`

These captures and the isolated smoke progress are **not committed**. Normal learner progress is outside the repository. Source-control checks exclude credentials, environment files, protected local configuration, progress, recordings, binaries and screenshots.

## Limits

- Windows and macOS were not built or run here; no runtime acceptance is claimed. Native desktop support is an architectural target. No installers, releases or auto-update channel exist yet.
- Keyboard testing injects events at the native egui input boundary; it is not physical-device automation. No screen-reader, IME, high-contrast OS theme or multi-monitor DPI acceptance was performed. AccessKit is enabled, but accessibility still needs human testing.
- Arbitrary floating-window sizes were constrained by COSMIC. Narrow layouts were checked through egui at explicit sizes; real native resizing was checked through the full-screen transition.
- Only the documented command subsets and models exist. Each scenario discloses its omissions; see CONTENT.md. No full shell, operating system, router, DNS daemon or block device is emulated.
- Nine real authored labs only. The 900 synthetic entries are labeled, opt-in and unplayable. Pack editing requires a rebuild; external pack installation is deferred.
- Local save versioning is strict with no migrations yet. Corrupt/incompatible originals are preserved; those sessions are read-only. Concurrent instances use last-writer-wins storage and are unsupported. A visible storage error means progress is only in memory until a save succeeds.
- No project license was chosen. Owner decision is still required before licensed distribution.
