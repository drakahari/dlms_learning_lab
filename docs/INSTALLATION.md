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

## M8 native CI evidence (not final product acceptance)

[Run 39793992548](https://github.com/drakahari/dlms_learning_lab/actions/runs/39793992548), source `db54a2cd7b82d58e3eea811ce48bd4c049a9f684`: all three native jobs passed Rust tests/Clippy and applicable Python checks. macOS compiled but produced no installation candidate because of the notice gate. Ubuntu/Windows candidate archives were downloaded and their sidecar hashes verified locally. They contain the 24-assignment M8 curriculum, not subsequent work.

| Candidate | SHA-256 |
| --- | --- |
| `dlms-learning-lab-0.1.0-gdb54a2cd7b82-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz` | `3e4ac6abbfeee4239f3c9a4fa3be8ef69f87b0d3e85456f83777bcb23ec6d154` |
| `dlms-learning-lab-0.1.0-gdb54a2cd7b82-x86_64-pc-windows-msvc-windows-candidate.zip` | `93f40a7361380c7884b14a499e2410ce8356548884f4e35a7aff591604d8f487` |

Local retained copies: ignored `dist/native-ci-db54a2c/`. CI retention is 14 days. No public release was published; Windows/macOS desktop runtime, upgrades and security prompts remain unverified.

### Refreshed M11 candidates

[Run39800447563](https://github.com/drakahari/dlms_learning_lab/actions/runs/39800447563) at `71734c65df33aa2dda88596258a90f2d7c4bce0b`: all native CI jobs passed; runtime acceptance remains separate. These include30 authored assignments/seven units and the IPv6 capability foundation, with no authored IPv6 lesson counted. Downloaded archives and sidecars were verified and retained under ignored `dist/native-ci-71734c6/`.

| Candidate | SHA-256 |
| --- | --- |
| Fedora44 x86_64 `...-g71734c65df33-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz` | `8067a2191cfa5d1b6a64e44a7686e8dc0c32f1acf5bdcaa2413ac68ace99318e` |
| Ubuntu24.04 x86_64 `...-g71734c65df33-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz` | `6d706a134c8fb5d4c415d85f241e65bac0c93830252c283c536adce88175c743` |
| Windows x86_64 `...-g71734c65df33-x86_64-pc-windows-msvc-windows-candidate.zip` | `c6a97ac99374911d6657c0e955d6262bbd36a71d8e222dd049f9973586de9f52` |

The full filename prefix is `dlms-learning-lab-0.1.0`. No macOS archive is supplied while its notice gate remains unresolved. Fedora candidate was installed/opened outside the checkout under an isolated test root with a synthetic profile, including light150 native layout; no Windows/macOS desktop run is claimed.


### M13/M14 refreshed candidates

[Run39810533865](https://github.com/drakahari/dlms_learning_lab/actions/runs/39810533865) at `c6d8afe61585` passed all three native build/test jobs. These candidates contain36 assignments/eight units and precede M15 evaluator changes. Downloaded Ubuntu/Windows checksums were verified under ignored `dist/native-ci-c6d8afe/`.

| Candidate | SHA-256 |
| --- | --- |
| Fedora44 `...-g82fbf868f5b3-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz` | `d2f8b45b0f6a3e2110b9d6790b928ae475cf5b203780a44129f658b2e4644ef0` |
| Ubuntu24.04 `...-gc6d8afe61585-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz` | `0fcec4003ee102d5f6aed626003d9d3590ec21fa4d9ef5e0038de39530d31826` |
| Windows `...-gc6d8afe61585-x86_64-pc-windows-msvc-windows-candidate.zip` | `8c3bd51c5d15bb790a94369be102f7e9addbcdc52de6dcea23b990ff9ce452e6` |

The full prefix is `dlms-learning-lab-0.1.0`. Fedora installed/upgraded/opened outside the checkout under an isolated root; synthetic progress was retained and native IPv6/light150 screens inspected. These are milestone identities, not the final complete course or Windows/macOS desktop acceptance. macOS packaging remains gated above.


### M18 clean owner-test candidates

Source `90039eefdddb0c1cb99482841664a3b602d4b2e1`,43 assignments/ten units, evaluator revision2. [Run39821972644](https://github.com/drakahari/dlms_learning_lab/actions/runs/39821972644) passed181 Rust tests and applicable native build/tooling checks on all three runners. Ubuntu/Windows archives were downloaded and their sidecar hashes verified under ignored `dist/native-ci-90039ee/`. They precede M19 arithmetic/order corrections.

| Candidate | SHA-256 |
| --- | --- |
| Fedora44 `...-g90039eefdddb-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz` | `f6cd92838924b57fe78c1a4f24fba4b6214f41fe06b5880a422dd61ebf47f199` |
| Ubuntu24.04 `...-g90039eefdddb-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz` | `073dbb6be7d4498ca373a5d007a8150716d28a43428bfb8465395807d72bf107` |
| Windows `...-g90039eefdddb-x86_64-pc-windows-msvc-windows-candidate.zip` | `93a106c27aac0425294581a6dff7117a9d66f1f8ab2f522fa7837229f49af56e` |

Full prefix: `dlms-learning-lab-0.1.0`. Fedora archive was checksum-verified, extracted and installed/upgraded under an isolated /tmp root. Installed CLI validated43/ten outside the checkout; desktop-file-validate passed and the exact synthetic progress sentinel stayed intact. The installed native app then passed all4 text-lesson synthetic flows, including larger text/light/dark views. These are controlled installation/render/input checks; the sentinel is a retained-file test, not an old profile opened for acceptance. Real Windows/macOS desktop installation/runtime/accessibility and macOS notice/signing gates remain open. No release published.


### M20 Fedora installation candidate

Source `94b0dbe4883a5124033faf8c86e32abf21d74e9a` contains47 authored assignments/eleven units, evaluator3. Fedora44 archive `dlms-learning-lab-0.1.0-g94b0dbe4883a-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz` SHA-256 `4453f212087bd8ff99f545ffccc60bbf0a66832a017be94d921112dcdb37064f`; binary `d49476cd39c19a8a567ae82c3174e0bc384eba1908aae5c7abf10d9c72338194`. Checksum-verified, extracted and installed/upgraded the isolated /tmp Fedora test root. Outside-checkout CLI validation and desktop-file-validate passed; the exact synthetic progress sentinel was retained (a file-retention check). Installed native full counter-unit flow passed all4 lessons, wrong retries, keyboard repair/current runs, save/reload/invalidation and light/dark150 rendering. No personal profile or release publication.

Initial native CI run39828321927 at this source passed Ubuntu/macOS but failed a Windows restore-dialog UI assertion. Windows is not marked validated for this source; fixture diagnostics/geometry follow-up is under investigation. Earlier successful candidate identities remain above. macOS notice/signing and actual Windows/macOS desktop acceptance remain open.
