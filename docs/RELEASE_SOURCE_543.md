# DLMS Learning Lab 543-lab release source

The frozen catalog has 543 unique authored labs across 224 units.
It includes 129 CCNA, 158 LPIC-1, 176 LPIC-2, and 80 Bash/Shell labs.
This records a source freeze and packaging prerequisites. Native package
verification is a separate step.

## Source selection

The freeze is based on Git
`6b54e34869e031e50d6cf0854cce3402f65c6fd4` and the private source
manifest with SHA-256
`2583b080c2c0404f1a608888c39447e9baa285b507cd69cb070217620a35e6d4`.
The release tree retains the frozen runtime, registered content, teaching
data, compilation and test inputs, packaging code, and notices. The
README now describes the frozen catalog. In the selected documentation,
repository paths specific to the developer's Fedora account use a
neutral example path.

The local checkpoint, completion ledger, and implementation diary updates
remain in the development checkout. The release tree retains the parent
versions of those tracked files. Three superseded CCNA pilot capture
reports also remain local. The screenshots, profiles, backups,
diagnostics, build targets, and review installers stay outside the
release source. Tests retain authored simulation fixtures needed to
verify progress and scope.

## macOS notice provenance

Checked November 18, 2026. macOS packaging remains blocked by the
missing original notice for the locked `dispatch 0.2.0` Rust crate.

- The published 10,229-byte crate has SHA-256
  `bd0c93bb4b0c6d9b77f4435b0ae98c24d17f1c45b2ff844c6151a07256ca923b`,
  matching Cargo.lock and both retained crate copies. Its VCS metadata
  and the upstream 0.2.0 tag identify
  `82d6c7a5b75dc0c71c3f46f87bb6c16a476f7748`.
  [Registry version record](https://crates.io/api/v1/crates/dispatch/0.2.0) ·
  [Upstream tags](https://api.github.com/repos/SSheldon/rust-dispatch/tags?per_page=20)
- Nine published original files match the pinned Git blob IDs.
  The manifest declares MIT without a license-file. All 12 crate
  members and the pinned 14-blob tree were checked for original notice
  evidence, and the exact-revision license endpoint returned HTTP 404.
  The checks found no authoritative copyright and permission notice.
  [Pinned upstream tree](https://api.github.com/repos/SSheldon/rust-dispatch/git/trees/82d6c7a5b75dc0c71c3f46f87bb6c16a476f7748?recursive=1) ·
  [Exact-revision license endpoint](https://api.github.com/repos/SSheldon/rust-dispatch/license?ref=82d6c7a5b75dc0c71c3f46f87bb6c16a476f7748)
- The exact-version upstream request has no visible authoritative
  reply. The existing manifest entry with no notices and the
  fail-closed collector are preserved.
  [Upstream issue](https://github.com/SSheldon/rust-dispatch/issues/18)

An original notice explicitly tied to this exact source would close
the notice gate. A reviewed compatible dependency change would instead
require a new common source revision and focused validation. macOS
Developer ID signing, notarization, and native runtime acceptance are
separate requirements.

## Packaging status

- Fedora x86-64 uses the existing native tarball and per-user installer.
  Release compilation and isolated installation, upgrade, and startup
  checks for 543 remain pending. The 541-lab Fedora review is preserved.
- Ubuntu 24.04 amd64 has an existing native CI row and uses the same
  tarball format. Build for its glibc baseline. The retained Fedora 541
  executable imports glibc 2.43 symbols, while Ubuntu 24.04 has glibc
  2.39. The Omarchy x86-64 laptop can try the exact resulting archive
  after checking its architecture and OpenGL/Wayland runtime.
  Confirm compatibility by testing that archive on Hyprland.
  [Ubuntu 24.04 libc](https://packages.ubuntu.com/noble/libc6) ·
  [Omarchy manual](https://omarchy.org/manual/)
- Windows x64 has an existing native MSVC CI row and ZIP packaging with
  an optional per-user install script. Native 543 build, progress reopen
  and upgrade, and Authenticode signing are pending.
- macOS Apple Silicon has an existing native CI row and an app-bundle
  ZIP design. The original notice, signing, notarization, desktop
  verification, and a prior Mac baseline for genuine upgrade testing
  remain open.

For each eventual package, bind the source commit, dependency lock,
architecture, and binary and archive hashes. Confirm 543 labs and 224
units. Open it normally in a disposable profile, complete a sample lab,
quit and reopen to confirm progress, then upgrade over a genuine native
predecessor and confirm progress again. Run the primary Fedora checks
with the project's memory guard and one Cargo job.

[Native workflow](../.github/workflows/native-candidates.yml) ·
[Build safety and packaging](BUILD_WORKFLOW.md) ·
[Historical platform evidence](INSTALLATION.md)
