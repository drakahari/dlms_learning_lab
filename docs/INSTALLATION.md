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


### M32 installed addressing-teaching candidate

Source `0959e9a037b5742104e040fce24a9eba8d479d5a`, unchanged64/fifteen. [Native CI39862188551](https://github.com/drakahari/dlms_learning_lab/actions/runs/39862188551) passed on all three native runners; downloaded Ubuntu/Windows checksum manifests match their archives.

| Platform | Candidate | SHA-256 |
| --- | --- | --- |
| Fedora44 | dlms-learning-lab-0.1.0-g0959e9a037b5-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz | 84c117a92302469069c5e1c0936a8d45c1c815931ac6043b17b562d2976a725c |
| Ubuntu24.04 | dlms-learning-lab-0.1.0-g0959e9a037b5-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz | 277888ec663b9c5b368d3a34a26a368454e99958aa122f4f4273658f76004197 |
| Windows | dlms-learning-lab-0.1.0-g0959e9a037b5-x86_64-pc-windows-msvc-windows-candidate.zip | 806638597538dd0b13d4d998248baba2a0d6a2e90c9b1d05a5642a3781b5ca3b |

Fedora binary SHA-256 `3864fddf50e1bf61ae789bf652f57313b89c9b8db985f8e223edfe7ac82123b1`. Safely extracted and upgraded the same isolated installation; validated desktop entry and bundled content outside the checkout, retained the exact synthetic progress sentinel, then passed both installed native subnet/VLSM teaching flows with light/dark150% and allocation feedback review. Parsed legacy progress is separately covered by backend/UI regressions. This candidate predates M33 service teaching. macOS archive notices, actual Windows/macOS desktop acceptance and default-Wayland startup remain open. No public release.


### Native identity follow-up (M34)

The native Wayland ID now explicitly matches `org.dlms.LearningLab.desktop`; custom learner-data paths stay unchanged. Fresh full default-Wayland service and addressing walkthroughs pass at desktop/narrow150% sizes. The baseline startup probe passed too, so this does not prove the historical intermittent stall was fixed. See NATIVE_STARTUP.md for the evidence and remaining investigation. Do not change compositor/security settings as a workaround. macOS notice provenance was rechecked at the upstream repository and remains unresolved; no license text was guessed and the packaging gate remains intact.

### M33 native CI and M34 installed Fedora follow-up

[Native CI39864315623](https://github.com/drakahari/dlms_learning_lab/actions/runs/39864315623), source `77e341ab703ce4e598c103c66f2d3caef5975da4`, passed all three native runners. Downloaded Ubuntu/Windows sidecars match their archives. These builds include service teaching; CI does not establish Windows/macOS desktop acceptance.

| Build | Archive | SHA-256 |
| --- | --- | --- |
| Fedora44, M33 | dlms-learning-lab-0.1.0-g77e341ab703c-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz | 27594de5dd4927b129ccd4b247016e11a55a66eb26d36d2af04e2a2f46bcf429 |
| Ubuntu24.04, M33 | dlms-learning-lab-0.1.0-g77e341ab703c-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz | 402b928c6a08bde479464ca2d4b818b52fd791c1ce8f5969a0b1f9f3bfdb7bdc |
| Windows, M33 | dlms-learning-lab-0.1.0-g77e341ab703c-x86_64-pc-windows-msvc-windows-candidate.zip | e05f1c63205d09b3a7fa71e818ab954308bea9e2adc4104f64ba69b80d1626b3 |
| Fedora44, M34 | dlms-learning-lab-0.1.0-g34937f6c4ce8-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz | d84352b0798d69154cfa975e3641f305e4862ab96d9d88d8e6e4634cb7f18510 |

M34 source `34937f6c4ce8df9c0040e2e11cd945abd116bdcd`, binary SHA-256 `e06df7e519573c73361e9fc0954f9dc8700bf9365a55ed2903b63a25309d7ba7`, was built from clean committed source and upgraded in the isolated `/tmp` installation. Outside-checkout64/fifteen validation, desktop-file validation and exact synthetic-progress sentinel retention pass. A full installed default-Wayland native flow passes the original nine, all28 theme sample captures and seven introductory scripting tasks, including input/Run/Stop, current invalidation, Challenge and resized150% light/dark. The earlier90-second run reached the final capture but timed out before the closing check; the120-second rerun completed and is the accepted evidence. Sentinel retention is a file-preservation check; parsed migration has separate backend/UI tests. No claim that every freshly captured palette was manually re-reviewed or that historical startup stalls are conclusively fixed. No public release; macOS notices and actual Windows/macOS desktop acceptance remain open.

### M40 installed70-lab candidate (2026-10-23)

Source `46791ccf6b5e9d633aa194d2cdb0d41352e5e759`,70 authored assignments/sixteen units, shell evaluator5. [Native CI39912625664](https://github.com/drakahari/dlms_learning_lab/actions/runs/39912625664) passed356 Rust tests and applicable checks on Ubuntu24.04, Windows2025 and macOS15. CI remains build/test evidence, not actual Windows/macOS desktop acceptance. Ubuntu/Windows artifacts and sidecars were downloaded into ignored `dist/native-ci-46791cc` and independently hashed; embedded BUILD_INFO revisions match. macOS compiles/tests but its candidate archive remains gated by the unresolved original dispatch notice.

| Build | Archive basename | SHA-256 |
| --- | --- | --- |
| Fedora44 | dlms-learning-lab-0.1.0-g46791ccf6b5e-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz | 5f83955563180700fa9940699588d154ece3be411fcce1aca6202ec23c5493d9 |
| Ubuntu24.04 | dlms-learning-lab-0.1.0-g46791ccf6b5e-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz | e62c66558b8ed9bf695d5b793abadd2b9cad76945274b2d15358d8c7dd82323f |
| Windows | dlms-learning-lab-0.1.0-g46791ccf6b5e-x86_64-pc-windows-msvc-windows-candidate.zip | d803b928ec9b17f3803468a7e382211700d836d5b308a8fdccf7b05b02152f29 |

Fedora binary SHA-256 `7b3fc774608c1e8e415a0334fbcf31e290478aa5f277c74462ea8283038097d6`. The clean committed-source candidate passed a fresh isolated install, actual upgrade from34937f6c4ce8, manifest uninstall and reinstall. Its desktop entry validates; outside-checkout --version and --validate-content identify46791cc and70/sixteen. Exact original-nine synthetic fixture bytes in a separate data folder survived these operations. That is file-preservation evidence, not proof of production-mode parsing or a novice migration study; parser/migration/draft compatibility has separate model/UI tests.

The installed executable then completed the all-six authored DNS native flow outside the checkout using its isolated smoke profile, with introductions/examples, actual Save/reload/update/Check controls, disclosed reference-populated assignments, initial incorrect feedback, completion/invalidation, exact resume, selected resets and the normal unit library. `/tmp/dlms-m40-installed-native.log` closes PASS. Representative installed captures live under `/tmp/dlms-m40-install-acceptance/screenshots/workspaces`. This adds installed runtime evidence to M36/M37/M39 native control/teaching checks; it is not fresh installed acceptance of every lab/theme/scale. The XWayland command prefix `env -u WAYLAND_DISPLAY` was used locally. Default-Wayland intermittent startup investigation stays Open; no desktop/compositor/security setting was changed.

From the checkout, the current development launch is `env -u WAYLAND_DISPLAY ./target/debug/dlms-learning-lab`. For the candidate, extract the Fedora archive and run its executable directly, or use the install.sh instructions above. No Rust/toolchain is required by candidate users. Candidates are unsigned development builds, no public release; full curriculum, actual Windows/macOS install/reopen/upgrade/uninstall, signing/notarization and owner release acceptance remain Open. Preserve OS security checks.

### M44 Fedora indexed-list candidate and exact acceptance limits

Source346ffa06c1fff2360e19040931c1e7064eab6308,74 authored labs/seventeen units, evaluator6. Candidate `dist/dlms-learning-lab-0.1.0-g346ffa06c1ff-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz` SHA-256 `b8f7bf851e100f5ea79ee0ef5058f9a09bfe99c5891ebbaecc107cafd290614c`; binary `b78372afb4d9cf2348435676228533e9f45009623b94fe161ba3be114cccb0e5`. Target closure223 dependency packages plus Rust/theme/source notices resolved for this Linux candidate; this does not resolve the macOS dispatch notice gate. No release published.

Fresh isolated installation, desktop-file validation, version/content validation outside checkout, uninstall/reinstall and upgrade from46791cc all pass. Exact original-nine synthetic progress bytes survive (file-retention evidence, not a personal-profile GUI claim). Evidence `/tmp/dlms-m44-install-06kni51t/acceptance.json`; launcher `/tmp/dlms-m44-install-06kni51t/bin/dlms-learning-lab`.

First packaged startup after build failed immediately before rendering: selected Wayland socket refused connections; strict diagnostic series stopped at0/1 after the first failure (three were requested). Once the display returned, installed native flow completed all4 isolated examples and reference assignments, checks and saved reload, but its capstone edit test clicked an offscreen full TextEdit rectangle. The viewport-corrected development harness and focused regression now pass full invalidation/reset/resume/Challenge and150/200% native rendering. **A new clean candidate with that corrected harness still requires the full installed retest**; do not call the old harness run a full PASS. See NATIVE_STARTUP.md/SHELL_ARRAY_LESSONS.md. Windows/macOS desktop, screen reader/novice and complete curriculum acceptance remain pending.

### M45 verified corrected Fedora candidate

Source017e875aeb771ed7a58d3a283dcc42464abefaf8,74/seventeen/evaluator6. `dist/dlms-learning-lab-0.1.0-g017e875aeb77-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz` SHA-256 `d2f5f9b03649f644671acaf2df97a04b00c457ed5bad0a28a8825c65d6b0c68c`; binary `7aac414856371e2a42b7cd361aafe8366fc78f7b1cbcfd8f3ff4cdb6bbf4cf3b`. Linux223-package target notices/source plus Rust/theme notices resolve; no new dependency or public release. M46 process integration follows this identity and is not claimed inside this artifact.

Upgraded the isolated installation from346ffa0, validated exact binary/build/content outside checkout and retained exact original-nine synthetic data bytes. All3 **installed** native modes now pass: complete all-four array intro/example/assignment/wrong/retry/completion/invalidation/reset/resume/Challenge, array-loop Stop/repair/resume, and dark100/light150 unit-library rendering. Actual light150/dark200 captures reviewed. Evidence `/tmp/dlms-m44-install-06kni51t/m45-acceptance.json`, `/tmp/dlms-m45-installed-arrays*.log`. Reference scripts were injected; this is not novice or physical-device acceptance.

Ten further quiet default-Wayland startup/readback launches pass10/10; `/tmp/dlms-m45-packaged-quiet/results.json`. Their0.965–0.966s durations include deliberate harness pacing and are not startup performance benchmarks. No lifecycle/protocol logging in these quiet runs. This supersedes the pending corrected-candidate test above; it does not establish a cure for the historical unanswered frame callback. The separate transient NoCompositor/connection-refused event and subsequent server availability remain recorded. Windows/macOS runtime, screen-reader/novice, macOS original dispatch notice/signing and full curriculum completion remain pending.

From the checkout open the exact verified build:

```sh
./dist/dlms-learning-lab-0.1.0-g017e875aeb77-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

## M47/M48 clean Fedora candidate —78 labs,2026-10-23

Source `e2b020064826f8befae64cc6cdc6ec2475a72d34` includes the reviewed process unit and the unintegrated LLDP capability foundation. It bundles78 authored labs/eighteen units, **not complete curriculum**. Linux target223 dependency archives/notices, Rust notice, fonts and28 theme notices resolve; no public release.

```
./dist/dlms-learning-lab-0.1.0-ge2b020064826-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Archive: `dist/dlms-learning-lab-0.1.0-ge2b020064826-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz`
SHA-256: `502c6a10a83b678717e2ae0a38edfab3b822160614bc241117aa982916d8b022`
Executable SHA-256: `d7e3e713f8a70ffff8bf99271cafe646305574def5bde9eb3c97fd3493f990f3`.

Actual isolated per-user Fedora install, upgrade from017e875, clean install/uninstall/reinstall, desktop-file validation, outside-checkout identity/content checks and exact synthetic progress-byte retention pass. The old fixture was technical test data, not the owner’s real profile. Installed launcher `/tmp/dlms-m44-install-06kni51t/bin/dlms-learning-lab`; local evidence `m47-acceptance.json` in that isolated root.

The **installed executable** passes the normal all-four process-unit flow, including final concise wrong-state feedback and final post-CONT inspection, plus the full72-capture IPv4/VLAN alignment matrix100/150/200%, wide/narrow/light/dark. Representative actual fields, populated values, labels, columns, focus outlines and scrolling were visually reviewed. Ten additional quiet default-Wayland startup/readback launches pass10/10;0.965–0.966s includes artificial test pacing and is not a performance benchmark. Logs `/tmp/dlms-m47-installed-process.log`, `/tmp/dlms-m47-installed-alignment.log`, `/tmp/dlms-m47-packaged-quiet/results.json`. These flows inject authored synthetic actions, not novice, physical-device or screen-reader acceptance. Historical unanswered-frame startup cause remains unresolved/unreproduced.

Native CI [39976264683](https://github.com/drakahari/dlms_learning_lab/actions/runs/39976264683) was dispatched at the same exact source. Final CI results/artifacts are recorded below; never infer Windows/macOS desktop acceptance. The existing macOS notice/signing gate remains separate. Subsequent in-progress discovery workspace source is not included in this candidate.

CI39976264683 completed successfully on all3 native runners. Its Windows and Ubuntu archives were downloaded under ignored `dist/native-ci-e2b0200/`, sidecar SHA-256 and embedded exact source revision checked locally. macOS compiled/tested, with packaging still blocked by the recorded dispatch notice; no desktop runtime inference.

| Sourcee2b0200 CI candidate | SHA-256 |
| --- | --- |
| Windows x86_64 MSVC `...-ge2b020064826-x86_64-pc-windows-msvc-windows-candidate.zip` | `39a1d2e58aa9adcb9a5176a11e574e0785d0ee3d948d7a2f11d3786b1d684563` |
| Ubuntu24.04 x86_64 `...-ge2b020064826-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz` | `ab6ed5f5b939422d237358582435ca2d8e7b2e95694f2b31abf5bd67b2f002ce` |


## M50 verified Fedora candidate — 82 labs, 2026-10-23

Clean source `a741b65558ca4ef43735594fb7af915f96cd84f1` bundles 82 authored labs in nineteen units. It is a tested milestone, **not curriculum completion**.

```sh
./dist/dlms-learning-lab-0.1.0-ga741b65558ca-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Archive SHA-256: `3b0694a6693d2565d16a3864ad9dcc0d89920699be1c6cc30488ad4cc188aaf4`.
Executable SHA-256: `f71491b5e1bf4aedf495cbe04874c5d98b976f0007b3693f1886af49831a159c`.
The archive has the same directory name plus `.tar.gz`. Linux target 223 dependency source archives/notices, Rust, fonts and all 28 theme notices resolve.

Actual isolated upgrade from e2b0200, clean install/uninstall/reinstall, desktop-file validation, outside-checkout build/content identity and exact original-nine synthetic progress-byte retention pass. The installed executable passes all four LLDP lessons with isolated examples, native Apply/Send/Check, partial/current grading, expiry invalidation, save/reload, selected reset and Challenge. Reference drafts are populated programmatically; this is not novice or physical-input acceptance. The installed 72-capture IPv4/VLAN matrix covers wide/narrow, light/dark and 100/150/200% text; representative fields, focus, labels, columns and scrolling were reviewed. Ten quiet default-Wayland startup/readback launches pass 10/10; artificial harness pacing means these are not performance measurements. Historical unanswered-frame startup cause remains unresolved/unreproduced.

Evidence: `/tmp/dlms-m44-install-06kni51t/m50-acceptance.json`, `/tmp/dlms-m50-installed-discovery.log`, `/tmp/dlms-m50-installed-alignment.log`, `/tmp/dlms-m50-packaged-quiet/results.json`. Native CI [39986036392](https://github.com/drakahari/dlms_learning_lab/actions/runs/39986036392) passes all three native runners at this exact source. CI compile/tests are not Windows/macOS desktop acceptance; macOS packaging remains gated by the recorded dispatch notice. M51's subsequent feedback-visibility/accessibility descriptions are not inside this artifact. No public release.


## M51 verified feedback/accessibility candidate — 2026-10-23

Clean source `eff98f09090d66dd0172b3ffa9ddc90aa188d4f8`, still 82 authored labs/nineteen units:

```sh
./dist/dlms-learning-lab-0.1.0-geff98f09090d-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Archive SHA-256: `00a3a02715e6653debf25329f512cf9cc99d005bdcec335185ee0ab173c73bbc`.
Executable SHA-256: `a00ba612fa215937786dd451dbaddb2afaa23823438b87a82cd864a74f9fbb9b`.
Linux target notices/source closure passes. Actual isolated upgrade from a741, clean install/uninstall/reinstall, desktop validation, outside-checkout identity/content/hash checks and exact synthetic old-progress bytes pass. The **installed** full four-LLDP flow passes; root reviewed fresh completion and retry screenshots with results visible after Check plus narrow light150/dark200. Reference drafts are programmatically populated; no novice/physical-device claim. Ten further quiet default-Wayland startup/readbacks pass10/10; artificial pacing is not a speed benchmark, and the historical failure remains unresolved/unreproduced.

Evidence: `/tmp/dlms-m44-install-06kni51t/m51-acceptance.json`, `/tmp/dlms-m51-installed-feedback.log`, `/tmp/dlms-m51-packaged-quiet/results.json`. This focused follow-through did not repeat the entire72-capture alignment matrix or every native workspace; M50 supplies that unchanged shared-field evidence and81 M51 UI tests check existing families. New diagram descriptions are checked through the native accessibility tree, not an actual screen reader. Subsequent gated parser/storage foundation source is not in this candidate. Full curriculum, Windows/macOS desktop/signing/notice gates remain Open; no release.
