# Installation candidates and platform acceptance

These are **development milestone candidates**, not a complete curriculum or a public release. `BUILD_INFO.json` identifies source revision, dirty status, compiler, target, build OS, embedded content hashes and binary checksum. Archive checksums sit beside each candidate. Do not describe an unsigned build as platform accepted.

## Linux / Fedora

Extract the candidate archive. Open a terminal in its extracted folder and run:

```sh
sh install.sh
```

This installs for the current user, without administrator access, and adds **DLMS Learning Lab** to the application menu. The stable launcher is `~/.local/bin/dlms-learning-lab`; application resources live under `~/.local/opt/dlms-learning-lab`. It uses standard Linux shell/coreutils only; no Rust, Python, profile path, network account or learner VM is required. You can also open the executable directly from the extracted folder.

For upgrade, close the app and run the new candidate’s `install.sh`. Existing progress stays in the separate application-data folder described in [PROGRESS_RECOVERY.md](PROGRESS_RECOVERY.md). Unrelated launchers, changed links and user-added resource files cause a refusal rather than silent replacement. Resolve that specific conflict before retrying. Never put learner backups into the application resource folder.

To remove the installed app:

```sh
sh ~/.local/opt/dlms-learning-lab/uninstall.sh
```

The uninstaller removes owned resources and shortcuts, retaining progress and user-added files. The installer's optional `--prefix`, `--bin-dir`, `--desktop-dir` are for development/testing/custom installations, not required setup. Locations must be absolute, separate/non-nested and free of desktop field-code/control characters. A changed intermediate symbolic link causes refusal before any removal.

Fedora 44 x86_64 is the actual native development/test desktop. Candidates built there need a compatible glibc, working X11 or Wayland and an OpenGL driver; other Linux distributions and software rendering are not automatically accepted. No system driver or desktop-theme setting is changed. An Ubuntu CI build has its own build OS and checksum and still needs desktop installation tests.

## Windows

Extract a Windows candidate and open `dlms-learning-lab.exe`. For a per-user installed copy, use the included `Install.ps1` through an allowed PowerShell session, or copy the extracted folder to your user’s Programs folder and create a Start-menu shortcut through Windows. Do not change execution policy or bypass security controls to run the helper. It refuses an existing target or unrelated shortcut.

For upgrade, close the application, retain/export a progress backup, and replace only the application resources in the Programs folder. Keep the separate application-data directory. For uninstall remove the application folder and its shortcut; keep application data if you want to retain work. The current helper deliberately refuses automated replacement of an existing installation. Clean install/reopen/manual upgrade/uninstall, protected progress, physical keyboard, scaling and accessibility acceptance still require a real Windows desktop. No signing credential is supplied or generated.

## macOS

The native builder can assemble a `.app` bundle. Intended path: extract, drag **DLMS Learning Lab.app** to Applications or your user’s Applications folder, open normally. Close before replacing the bundle on upgrade. Removing the bundle retains separate application data. Owner installation tests must follow Apple’s supported signing/security process; never disable Gatekeeper or bypass controls. Signing/notarization, an application icon and runtime/accessibility acceptance remain open.

**Current packaging gate:** `dispatch 0.2.0` is linked through the macOS native stack. Its published package says MIT, but no original license/copyright notice was found in its archive or pinned upstream revision. Packaging refuses to label a complete candidate while that notice is unavailable. The optional `hexf-parse 0.2.1` entry has a similar unavailable notice in the complete lock graph, but is not linked by any current native target. Neither is silently omitted or declared incompatible. Resolve original notice provenance before macOS candidate redistribution; no dependency or accessibility feature was removed to conceal the issue.

## Maintainer build and provenance

On each target OS, with the native compiler/toolchain and Python 3.11+ available:

```sh
cargo fetch --locked
python tools/build_candidate.py
```

`CARGO` and `RUSTC` may name an already available isolated toolchain. The builder refuses a dirty tree by default, uses locked offline sources, remaps source paths, bundles only tracked documentation and explicit application resources, includes dependency/font/Rust standard-library notices and every linked dependency’s checksum-verified original source archive, then emits an archive/checksum under ignored `dist/`. It never publishes releases. `--allow-dirty` labels development checks explicitly; those are not reviewed-source identities.

`tools/dependency_notices.py` collects the actual target’s normal/build graph, preserving original license expressions, texts, hashes and sources. MPL covered source is supplied as its exact original `.crate`; other source archives preserve embedded headers/notices. Missing/changed notices or archives stop candidate creation. Cached upstream notices use full source revisions in `assets/licenses/upstream/manifest-v1.json`, with no runtime network retrieval. Palette attribution remains unchanged in [THEMES.md](THEMES.md). Original code/curriculum use the owner-approved MIT license.

The manually dispatched `Native milestone checks and candidates` workflow tests/compiles on Ubuntu 24.04, Windows Server 2025 and macOS 15. It prepares Linux/Windows candidates only after their notice gates pass; macOS remains explicitly gated above. Actions and runner labels were checked against their official sources on 2026-10-22. Build/test output is not desktop runtime acceptance. No workflow creates a release, signs an app or changes a learner’s security settings.

## Acceptance still required

For each exact candidate checksum: clean user installation; opening offline; saved script and terminal drafts across restart; all current scenario types; restored backup; upgrade with an old profile; uninstall retaining progress; keyboard/focus and assistive technology; smaller/larger windows and text; light/dark states; OS security/signing behavior. Fedora evidence and actual CI results belong in the implementation ledger. Windows/macOS desktop checks cannot be inferred from Fedora, cross-compilation or CI.
