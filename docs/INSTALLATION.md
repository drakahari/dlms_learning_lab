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


### M20 CI follow-up acceptance

[Run39829793024](https://github.com/drakahari/dlms_learning_lab/actions/runs/39829793024) at `bd14db0dcf1ab81672e87d5a6885d6b0e7895fe9` passed all three native runners after the async-dialog fixture follow-up. It includes203 Rust tests/47 assignments/eleven units and precedes M21 access capabilities/M22 content. Downloaded archives were sidecar-checksum verified under ignored `dist/native-ci-bd14db0/`.

| Candidate | SHA-256 |
| --- | --- |
| `dlms-learning-lab-0.1.0-gbd14db0dcf1a-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz` | `f0d6d5afa16d2c3916114db926b23d26a0907e4c50d47ed376fc275d81641f1a` |
| `dlms-learning-lab-0.1.0-gbd14db0dcf1a-x86_64-pc-windows-msvc-windows-candidate.zip` | `fda9ce66dd69e5c74c7d93a63e97adf18b5158d459a01dff7f2cfdcdabdb81dc` |

These CI results do not establish actual Windows/macOS desktop installation/runtime/accessibility acceptance. macOS packaging remains notice-gated. The Fedora94b0dbe candidate above is separately installed/render-tested; no release published.


### M22 clean owner-test candidates

Source `19aad147796b289c8aa89d9371dcaea372c9155c` contains51 assignments/twelve units. [Run39833026248](https://github.com/drakahari/dlms_learning_lab/actions/runs/39833026248) passed all three native CI runners (216 Rust tests and applicable build/tooling checks). Ubuntu/Windows archives were downloaded and sidecar hashes verified in ignored `dist/native-ci-19aad14/`.

| Candidate | SHA-256 |
| --- | --- |
| Fedora44 `dlms-learning-lab-0.1.0-g19aad147796b-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz` | `1dd7badf44ac9d228445c1104652dc9dd59449a92d8dac06dd9569b80b92136d` |
| `dlms-learning-lab-0.1.0-g19aad147796b-x86_64-pc-windows-msvc-windows-candidate.zip` | `b89c52eba6b670c24703b481d8611b7a0a0355e2b24881726558536ca6071e5a` |
| `dlms-learning-lab-0.1.0-g19aad147796b-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz` | `9142e7a2edfc12cf428f827d5012b62a746530d6b296fe490f4132ba92c8c90e` |

The clean Fedora archive was extracted and installed/upgraded in the isolated /tmp root. Installed CLI validated51/twelve outside the checkout; desktop-file-validate passed and the exact synthetic progress sentinel remained intact (file retention, not parsed legacy-profile acceptance). Installed native all-four access lesson flows passed, including light/dark150% views. Windows/macOS desktop runtime/install/accessibility and macOS notice/signing gates remain open. No release publication.


### Native startup evidence to investigate after M25

The full four-lesson M25 Fedora native flow completed, but extra default-Wayland resumed-draft runs stopped before the smoke harness’s first timed action. The same synthetic resume flow passed through existing XWayland. An app-only Wayland trace ends after committing a frame, with a pending callback after keyboard focus left; the session was active and unlocked. This suggests occlusion/repaint scheduling in the timed harness, but does not establish a root cause. Do not treat those stalled runs as startup acceptance. No user desktop/session setting was changed, and no security control was bypassed. Continue independent work and investigate the harness/window startup behavior before final acceptance.


### M25 installed candidates (before evaluator4 expansion correction)

Source `85d0274ebbacf9e65376d6b44fc054a83b95b77e`,55 assignments/thirteen units, evaluator3. [Run39841022031](https://github.com/drakahari/dlms_learning_lab/actions/runs/39841022031) passed239 Rust tests and applicable build/tooling checks on all three native CI runners. Downloaded Ubuntu/Windows archives and sidecars were hash-verified under ignored `dist/native-ci-85d0274/`. They precede M26 resource tightening and are development milestone candidates, not final release acceptance.

| Candidate | SHA-256 |
| --- | --- |
| Fedora44 `dlms-learning-lab-0.1.0-g85d0274ebbac-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz` | `504e6a758d391ff6ddf1fe7344d787c31732f6077a28c58edf3f50183428f927` |
| `dlms-learning-lab-0.1.0-g85d0274ebbac-x86_64-pc-windows-msvc-windows-candidate.zip` | `35159be57612b225eae9f862b89337a47f68646983f8bb5bcf43ad76cbc71d23` |
| `dlms-learning-lab-0.1.0-g85d0274ebbac-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz` | `23447655b2f97aaa3cb16a20944b49a360537040771dc123ebb1ae33b675d223` |

Fedora binary SHA-256 `145d2290d8dc7e53cccb5d6a1f23601ca6e5971faafb14c0b2296f398fe7f647`. Archive verified/extracted and installed/upgraded in the isolated /tmp acceptance root. Outside-checkout CLI validated55/thirteen and desktop-file-validate passed; exact synthetic sentinel remained (retained-file check). Installed native all4 VLAN flows passed outside checkout through existing XWayland, including keyboard repairs, checks, reset/resume, invalidation, malformed-entry feedback and150% light/dark views. The default-Wayland extra-start observation above remains unresolved; a smoke-only pre-map fullscreen experiment was ineffective and removed. No system/session/theme setting was changed. Actual Windows/macOS desktop runtime and macOS notice/signing gates remain open. No release publication.


### M26 installed candidates (before evaluator5 filename matching)

Source44c1f406f15c68c4bab73f9bed3c482271dc4b6b,55/thirteen, evaluator4. [Run39845121080](https://github.com/drakahari/dlms_learning_lab/actions/runs/39845121080) passed244 Rust tests and applicable native CI checks on all three runners. Ubuntu/Windows sidecars downloaded and locally hash-verified under ignored dist/native-ci-44c1f40.

| Build | Candidate archive | SHA-256 |
| --- | --- | --- |
| Fedora44 | dlms-learning-lab-0.1.0-g44c1f406f15c-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz | 561c2f8faf1c4b4930b541809ffa3e065c287ed5991d76e254578c5f9d51ad0f |
| Ubuntu24.04 | dlms-learning-lab-0.1.0-g44c1f406f15c-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz | 7a09c72f712cf632eac72644111a2e97258008f7903c580c89c65fbfc94e60de |
| Windows | dlms-learning-lab-0.1.0-g44c1f406f15c-x86_64-pc-windows-msvc-windows-candidate.zip | 069861a156b3a3675f123f6c283aeb2c745bc98af40f19013780f1bc0035c577 |

Fedora binary SHA-256 c98f4827bbe5bfcd281ea11085ab16beb7b6584e48b81a1d0605652d57dcd87f. Clean archive installed/upgraded in the isolated /tmp test root; bundled validation55/thirteen, desktop entry validation and exact sentinel retention pass (retained-file check). Installed native stress flow outside checkout passed current limit feedback/history, exact backup, keyboard editor/Run/Stop, repair/all3cases, reload/edit invalidation and light/dark150%. These are synthetic development checks, not novice/platform acceptance. Default-Wayland startup investigation and actual Windows/macOS desktop acceptance remain open. macOS archive remains notice-gated. No release published.


### M27 prepared candidates (before M28 authored file-batch unit)

Sourcea3d7a9d212663f91a134b231525971fb0cfeda80,55/thirteen, evaluator5. [Run39846874265](https://github.com/drakahari/dlms_learning_lab/actions/runs/39846874265) passed253 Rust tests and applicable native checks on all3 CI runners. Ubuntu/Windows sidecars downloaded and locally hash-verified under ignored dist/native-ci-a3d7a9d. Fedora prepared from clean source; this identity is build/hash evidence, not a new installed-runtime acceptance run.

| Build | Candidate archive | SHA-256 |
| --- | --- | --- |
| Fedora44 | dlms-learning-lab-0.1.0-ga3d7a9d21266-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz | 16c63c31aebc9917f9ca8a928d1bc5e0735346912e25d087a9b1778e2a1d79fe |
| Ubuntu24.04 | dlms-learning-lab-0.1.0-ga3d7a9d21266-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz | 8589a6d2969712d5ce8d631f74f782ddae759b8b9c73cad8afedf1e6992dcde5 |
| Windows | dlms-learning-lab-0.1.0-ga3d7a9d21266-x86_64-pc-windows-msvc-windows-candidate.zip | 5c2f774a4a87b67dae5f2d00948b6f7557bf220705b2631e8a5c5077e352fd60 |

Fedora binary SHA-256 e638446f14eda469d686eff080e5f72fb97c91fab9d4eb4d55d83dd887a8a23e. No M27 candidate release was published. macOS archive notice gate, actual Windows/macOS desktop acceptance and the default-Wayland startup investigation remain open.


### M28 installed candidates (58 authored assignments)

Sourceae484460b456bb43cbb180f6b326775fd6770a68,58/fourteen, evaluator5. [Run39849265414](https://github.com/drakahari/dlms_learning_lab/actions/runs/39849265414) passed260 Rust tests and applicable native checks on all3 runners. Downloaded Ubuntu/Windows sidecars were locally hash-verified under ignored dist/native-ci-ae48446.

| Build | Candidate archive | SHA-256 |
| --- | --- | --- |
| Fedora44 | dlms-learning-lab-0.1.0-gae484460b456-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz | 2d7d966099a7f5e9e924986a9e87cb60131e67afac07ed6203ba0b2bce51bb42 |
| Ubuntu24.04 | dlms-learning-lab-0.1.0-gae484460b456-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz | 0a1771743e083f254732c427c811ce5932a1579f833ada0c18d2100375381061 |
| Windows | dlms-learning-lab-0.1.0-gae484460b456-x86_64-pc-windows-msvc-windows-candidate.zip | 153efc0cc3839ad20a7529450c5eae36df8f339c97efc0f9e09e58eaef498a48 |

Fedora binary SHA-2562523d408a182e9d5747a4740ac1b710acb01b8eb787e8b7408d5e10c42e2561a. Clean archive installed/upgraded in the isolated /tmp acceptance root; bundled validation58/fourteen, desktop entry validation and exact synthetic sentinel retention pass (retained-file check). Installed native all-three file-batch flow passed outside checkout, including examples/experiments, keyboard repairs/current grading, save/reload/edit invalidation, selected reset and light/dark150%/unit library. This is a development milestone, not completed-curriculum or novice/platform acceptance. Default-Wayland investigation, actual Windows/macOS desktop acceptance and macOS archive notices remain open. No release published.


### M29 prepared Fedora capability candidate

Sourcea1245b4431786ddccb88dba7480c7d69303465d6,58/fourteen, evaluator5. [Run39852335952](https://github.com/drakahari/dlms_learning_lab/actions/runs/39852335952) passed270 Rust tests and applicable checks on all3 native CI runners. Fedora clean candidate `dlms-learning-lab-0.1.0-ga1245b443178-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz` has SHA-256 `6d32fe01d60bb1f67c5a59eb9365b44f3f7ab7acf38e0f16d9474fd66f704770`; binary `e07ef599ca0d244649534a9d176883e7b50caf0d0ef67922691ea036e86957f8`. This is build/hash evidence, not additional installed-runtime acceptance. No release published; macOS archive notice gate, actual Windows/macOS desktop acceptance and default-Wayland startup investigation remain open.


### M31 installed routing candidate

Source `b778fb567f8cacb500b62c3d627cafd1ec849626`,64 authored assignments/fifteen units. [Native CI39858444151](https://github.com/drakahari/dlms_learning_lab/actions/runs/39858444151) passed on Ubuntu24.04, Windows2025 and macOS15. Archives/hash manifests were downloaded and verified locally; native CI is build/test evidence, not actual Windows/macOS desktop acceptance.

| Platform | Candidate | SHA-256 |
| --- | --- | --- |
| Fedora44 | dlms-learning-lab-0.1.0-gb778fb567f8c-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz | 71d20d9b022ab5aa8e5c6fd33ce3730a46d5e5b349d0eac0a76763ecf96cf7c8 |
| Ubuntu24.04 | dlms-learning-lab-0.1.0-gb778fb567f8c-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz | 7147b52c628f028f8e146a921e0e82f5406f255ce870df4c2435c64a8e50f917 |
| Windows | dlms-learning-lab-0.1.0-gb778fb567f8c-x86_64-pc-windows-msvc-windows-candidate.zip | 0c9fe4df55c3166d50e44c89faa74c472156e2d93ded5517270177eae3f4a2f9 |

Fedora binary SHA-256 `565b485d8db497e9d393df0f9ac03f5595ef77b749995a0325b596e4ee6ae50e`. Extracted and upgraded the prior isolated `/tmp` install; validated its desktop file and bundled64/15 content outside the checkout, retained the exact synthetic progress sentinel and passed the full installed all-six routing native flow. Sentinel retention proves file preservation, not parsed legacy-progress acceptance (covered separately by model/UI tests). This candidate predates M32 teaching improvements. No public release. macOS notice gate, Windows/macOS actual runtime acceptance and default-Wayland startup investigation remain open.
