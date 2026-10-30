# Installation candidates and platform acceptance

These are **development milestone candidates**, not a complete curriculum or a public release. `BUILD_INFO.json` identifies source revision, dirty status, compiler, target, build OS, embedded content hashes and binary checksum. Archive checksums sit beside each candidate. Do not describe an unsigned build as platform accepted.

## Linux / Fedora

Current local Fedora candidate: **5233a151bfe3**,190 labs/50 units
(57CCNA,40LPIC-1,36LPIC-2,57Bash). Source
`5233a151bfe312082159e351477cfb3d2a750e12`. Packaging is not curriculum or
platform completion. Open from any terminal directory:

```sh
"$HOME/Documents/dlms_learning_lab/dist/dlms-learning-lab-0.1.0-g5233a151bfe3-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab"
```

The actual installed executable passes upgrade fromf242519, identity/content/
checksum and desktop-entry validation, uninstall/reinstall and exact synthetic
original-nine progress-byte retention. Four complete NTP tasks pass1389actions/
94capturefiles (76unique images), with wrong/partial/alternate work, current
invalidation, separate examples, reset/warm resume, keyboard and20 observed
scale/theme/width settings. Two fully exited NFS processes restore separate
unfinished assignment/example drafts, mounts/files, favorites/preferences and
whole observed progress, then repair and regrade both goals. Ten serialized
Wayland starts pass ready/first-update/capture without timeout(0.965–1.017s,
including deliberate test pacing, not startup performance measurements).

ELF76188480B SHA256
`80242c1500d4c300d90b2430c06724fec92c468ad709d810b053b153dab155ce`.
Archive46285100B SHA256
`04f982da507f75cee0e233dc931491b6591873a25a1a624851e3d525209298e1`.
See [the190 milestone](CATALOG190_MILESTONE.md#installed-fedora-candidate5233a15)
for exact bounded evidence, original failed driver diagnosis and source identity.
A finite startup test cannot rule out every intermittent failure; the historical
long Physical143 termination cause remains unproved. Actual Windows/macOS,
novice, physical-device and screen-reader acceptance remain pending. No release
is published. Earlier171 evidence remains historical at
[FEDORA_065032D_VERIFICATION.md](FEDORA_065032D_VERIFICATION.md); older candidates
and diagnostics are retained.

### Historical154-lab installed milestone

Retained earlier scoped Fedora candidate: **9a90f275e133**,154 authored labs/39 units
(45CCNA,32LPIC-1,28LPIC-2,49Bash). Source commit
`9a90f275e133655f981cd79dd49d93f5246d7377` contains the reviewed UI fixes and
five-size verification. The later documentation-only commit does not change this
executable. Curriculum, grading, evaluator10, progress formats and all28 themes
remain unchanged. Open from any terminal directory:

```sh
"$HOME/Documents/dlms_learning_lab/dist/dlms-learning-lab-0.1.0-g9a90f275e133-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab"
```

The actual installed regular ELF passes upgrade from00d69d3, clean uninstall/
reinstall, desktop validation, outside-checkout identity/content checks and exact
synthetic original-nine progress-byte retention. On this same final ELF, all ten
serialized native stages pass: NFS editor/Save alignment40 captures, script case/
run feedback7, four complete NFS flows106, three command-construction flows72,
three reliable-text flows72, original IPv4/VLAN alignment72, quiet/traced startup
and original script/DHCP two-process reopening. All369 flow/alignment PNGs are
indexed; this is not a claim that every image was individually viewed.

The NFS alignment sweep observes1920×1080 wide and946×929 narrow windows at all
five supported sizes100/115/125/150/200%, with both light/dark themes. Root reviews
representative final frames; an independent reviewer inspected all40 sweep frames
and the retained before image. Full lesson flows also retain wrong/partial work,
alternatives, invalidation, isolated drafts, reset, Challenge and warm full progress
readback. Named Case Pass/Fail is now distinct from whole-run failure; NFS Save
rows remain aligned through scrollbar changes. The unchanged comparison/clipping
and nonzero-status teaching regressions pass. Narrow200% documents require editor
scrolling; Save controls remain visible. No text is shrunk to fit.

Ten quiet Wayland launches/readbacks pass 0.865–0.916s; one separately traced
Wayland launch passes. Eleven successes do not prove an intermittent failure
impossible. Cold-process reopening covers the original script/DHCP cases, not
every NFS or newly authored scenario. This is injected-event technical acceptance,
not novice, physical-device, screen-reader or Windows/macOS desktop acceptance.
All28 theme checks pass; actual installed theme sampling is light/dark, not every
palette. Expanded optional NFS inventory is not newly visually reviewed.

ELF SHA-256: `96c3cf50503d1228342999887e0168d466389de79c7079e7dd6336b714f6333f`.
Archive SHA-256: `0d757e92fdfc268931976178ca5ddf2f1c342661dedffcdbd24fc58f34f4f9fa`.
Archive: `dist/dlms-learning-lab-0.1.0-g9a90f275e133-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz`.
Private receipt: reviews/ui-polish154-final/RESULT.json,
SHA-256 `2f8522a536641f25f449f03d5433b21065aa23d55ebb4967c9dd0183b6368430`. Before/after originals and all frame/log hashes stay
in ignored project-disk evidence, not the public repository. A tool sometimes
delivered partial previews despite populated, unchanged PNG bytes; read-only pixel/
hash comparisons and peer reviews retain that delivery caveat rather than calling
the stored capture blank or claiming every preview was uncaveated.

The Linux223 dependency notices and exact source archives pass the candidate gate.
All checks use the shared two-job/two-thread runner and disk-backed artifacts.
Across final source/build/install/native checks, minimum available memory was
21.20GiB; maximum some/full PSI avg10 was 0/0, OOM counter delta0.
These observations are not a memory reservation or stress-test guarantee.
No public release or new cross-platform runtime acceptance. Older candidates and
diagnostics remain intact. NAT80 acceptance remains on the previous00d candidate;
it was not rerun as part of this UI-only final pass.

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

The historical development workaround for this earlier build was `env -u WAYLAND_DISPLAY ./target/debug/dlms-learning-lab`. Current normal Wayland instructions appear at the top; no override is required. For the candidate, extract the Fedora archive and run its executable directly, or use the install.sh instructions above. No Rust/toolchain is required by candidate users. Candidates are unsigned development builds, no public release; full curriculum, actual Windows/macOS install/reopen/upgrade/uninstall, signing/notarization and owner release acceptance remain Open. Preserve OS security checks.

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

### M55 persistent-mount curriculum candidate

Clean source `a6a6043b377fc8bcbd8140f2e8b4b55f7c53fdbf`,86 authored assignments/twenty units, Fedora44 x86_64:

```sh
./dist/dlms-learning-lab-0.1.0-ga6a6043b377f-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Archive SHA-256:`19bf67961f6d83f6d8ac94ca75eede6077f3c070ce2175a699ef7b349df1dac5`. Binary SHA-256:`33237a165bce7cf12790f1960709abd92cf151fb27cec3e15ca4d460d849376a`.

Actual isolated upgrade fromeff98f0, clean install/uninstall/reinstall, desktop-file validation, outside-checkout identity/content/hash and exact synthetic old-progress-byte preservation pass. The installed launcher runs all four storage lessons through separate examples, explicit Save/terminal/Check, wrong-state feedback, completion, reset/resume/current invalidation, Challenge and narrow light150/dark200. Ten quiet default-Wayland startup/readbacks pass10/10. Reference editor drafts are supplied by the harness; no novice/physical-device/reader acceptance is inferred. Linux target notices/source closure passes. Native CI run40009847858 tests this exact source; consult its actual status, not a predicted result.

Evidence:`/tmp/dlms-m44-install-06kni51t/m55-acceptance.json`, `/tmp/dlms-m55-installed-native.log`, `/tmp/dlms-m55-packaged-quiet/results.json`. Initial72 alignment capture events completed, but actual screenshot review found the old harness sometimes pointed its wheel over the header in a short200% window, leaving fields off-screen. That run does **not** prove those field views passed. A development-only harness correction reuses viewport-aware scrolling; all72 corrected captures pass and actual200% fields/labels and VLAN values are visible and centered (`/tmp/dlms-m55-corrected-alignment.log`). Packaging/installed recapture of that correction is pending. Production control geometry did not change. Short tiled946×439 windows require scrolling at large text; text is not reduced. The historical intermittent callback cause remains unresolved. Full curriculum and external platform/notice/accessibility gates remain Open; no public release.


## M56 verified evaluator7 Fedora candidate — 2026-10-23

Clean source `f4d7bc013c37a9ccef3ee6b21313e315b4360906`,86 authored assignments/twenty units. This is a milestone candidate; full curriculum completion remains Open.

```sh
./dist/dlms-learning-lab-0.1.0-gf4d7bc013c37-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Archive SHA-256:`28200b9960a65a69bd2952fd4ef68466df84b4051ba985d4235852e9f402602e`. Binary SHA-256:`bda9e7f25f9ad409d1e25850e88f91915865d190f9d8aad1bd234ed289bd59e0`. Linux223dependency notices/source archives, Rust/fonts and all28themes resolve. No release published.

Actually installed outside checkout under the ignored disk-backed `.build-artifacts/installations/m56-pf5qdors/`. Upgrade from974381f, clean uninstall/reinstall, desktop-file validation, installed identity/content/hash and exact separate synthetic original-nine progress bytes pass. The installed launcher passes the complete public7 native flow: genuine frozen33 old6 records/exact first backup/history, current7 keyboard capture repair/Run, independent example, active-child Stop without graded result, resume/retry, light150/dark200 reachable Run and backup export/review/cancel preserving exact current bytes. Root reviewed native renders. Ten quiet default-Wayland startup/readbacks pass10/10; harness pacing is not a performance measurement and does not establish the historical startup cause/cure.

Evidence: `.build-artifacts/logs/m56-{candidate,installed-lifecycle,installed-native,installed-quiet}.log`, installation `acceptance.json` and `quiet-startup/results.json`, native screenshots in `.build-artifacts/native-runs/run-zt7pao1m/screenshots/workspaces/`. This focused installed pass does not recapture the whole shared72-view matrix;974381f supplied its actual installed alignment evidence and final87 UI regressions preserve those controls. Synthetic fixtures/injected input are not beginner, physical-device, screen-reader or Windows/macOS desktop acceptance. Those gates, macOS original dispatch notice/signing and most curriculum remain pending. New staged capture lessons are not inside this86-lab artifact.


## M57 verified 90-lab Fedora candidate — 2026-10-23

Clean source `7ffbfffe4b63b6fafe30aca59a3b4e551b6dc49e`,90 authored assignments/21 units:23CCNA,17LPIC-1,13LPIC-2,37Bash/Shell. Includes the four command-output/reporting lessons. This remains a milestone candidate, not full curriculum completion or a public release.

```sh
./dist/dlms-learning-lab-0.1.0-g7ffbfffe4b63-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Archive SHA-256: `b0973ffe176a574fc0aad745ba287d8a83b1b5d23ed421ebf2749965512eb343`.
Executable SHA-256: `afaea1630b0ecc6312ae76a4ec3eeffdd1974dbd214b53c047f5d67d59f0d111`.
Linux223 dependency source/notice archives plus Rust, fonts and all28 themes resolve.

Actual disk-backed isolated installation upgraded fromf4d7bc0, uninstalled/reinstalled, validated the desktop entry and verified installed build/content/hashes outside checkout. Separate synthetic original-nine progress bytes remained exact. The installed launcher passed all four newly authored lessons through introduction, independent example Run/change, full assignment, keyboard partial/repair/Run, completion, retained drafts/results, selected Reset, current-success invalidation, library filter and Challenge light150/dark200. Root reviewed installed native renders. Ten quiet default-Wayland launch/readbacks pass10/10; artificial harness pacing is not a performance benchmark, and the historical startup cause remains unresolved.

Evidence: `.build-artifacts/installations/m57-97r6dbbw/{acceptance.json,quiet-startup/results.json}`, `.build-artifacts/logs/m57-{candidate,installed-lifecycle,installed-native,installed-quiet}.log`, and `.build-artifacts/native-runs/run-q408nhdp/screenshots/workspaces/`. Existing shared alignment evidence remains the actual installed974381f72-view matrix plus M57's89 native-egui regressions; this focused installed flow does not claim to recapture all workspaces. No beginner study, physical-device/screen-reader or Windows/macOS desktop acceptance is inferred.

[Native CI40028251720](https://github.com/drakahari/dlms_learning_lab/actions/runs/40028251720) tests this exact source; Ubuntu has passed, Windows/macOS were still running at the last status check. CI compile/tests are separate from actual desktop acceptance; macOS original dispatch notice/signing gates remain. Later pure ACL foundations and the developer pack-file reader fix are not inside this artifact. They do not change the authored90 count or establish new practical coverage by themselves.

## M66 — 98-lab Fedora candidate; scoped installed acceptance passed

Clean source `87755c93d7b36fb6ade18e7ea578d063e743955d`,98 authored labs/23 units (27CCNA,21LPIC-1,13LPIC-2,37Bash), evaluator7. Candidate:

```sh
./dist/dlms-learning-lab-0.1.0-g87755c93d7b3-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Archive SHA-256 `acb495d5a4be690a7094df28cdf47fcdfc06244ceaa2878dc452da3163e2b2f5`; executable SHA-256 `4c310a85b18608bd0482318495304d14d87a472443a103be39792d31538b806f`. All223 applicable Linux dependency packages/notices and source resources resolved; no new dependency or public release. Pure OSPF/unit-manager foundations add no playable curriculum by themselves.

The fixed main-guard `installed_acceptance.py` tool actually verified upgrade from an identified7ffbfff executable, clean uninstall/reinstall, outside-checkout build/content/hash, desktop-entry validation and exact separate synthetic original-nine progress-file bytes. It uses a new private project-disk prefix, not personal progress. Evidence `.build-artifacts/logs/m66-installed-lifecycle-r1.log`, `.build-artifacts/installations/verified-87755c93d7b3-ci3j2yo2/acceptance.json`. The actual installed launcher subsequently passed all four account and all four ACL lesson flows: separate examples, full assignments, wrong/partial/alternate solutions, keyboard actions, current-success invalidation, exact saved resume, repair, selected Reset and Challenge Light150/Dark200. Logs m66-installed-accounts-r1.log and m66-installed-acl-r1.log; native captures run-henrjxlv and run-1p6pfwzl. The installed72-capture IPv4/VLAN alignment matrix passed100/150/200% text, wide/narrow and light/dark (m66-installed-alignment-r1.log, run-ecakpznb); inspected captures retain centered single-line text and reserved/reflowing VLAN columns. Ten quiet native default-Wayland launch/readbacks passed10/10, without lifecycle/protocol logging (m66-installed-quiet-r1.log, the same installation's quiet-startup/results.json). Their0.965–0.966s includes deliberate harness pacing, not a performance benchmark. These are synthetic-profile renderer/input checks, not novice, physical-device or screen-reader acceptance.

Candidate build55 memory samples: minimum available11.74GiB, peak some/full PSIavg10 zero, observed OOM-counter delta0. Earlier761-check combined run355 samples: minimum11.20GiB, peak PSI zero/OOMdelta0. Swap remained nearly full. These ordinary serialized2-job/2-test-worker runs are not a stress test or guarantee; no desktop/GPU/swap/model setting changed. Detailed ignored audit m65-m66-memory.json.

[Native CI40041702686](https://github.com/drakahari/dlms_learning_lab/actions/runs/40041702686) checks this exact source; all three native build/test jobs passed. Its eventual build/test results do not establish Windows/macOS desktop runtime acceptance. macOS original dispatch notice/signing/notarization, actual other-platform installation/reopening/uninstall, screen reader, physical-device and novice acceptance remain pending. The historical intermittent Wayland callback cause remains unresolved; no cure is claimed. Most full-product curriculum/completion requirements remain Open.


## M68 — installed102-lab Fedora candidate

Clean source `29096be81982de2812b2e25c609703bf40688472`,102 authored labs/24units (31CCNA,21LPIC-1,13LPIC-2,37Bash), evaluator7:

```sh
./dist/dlms-learning-lab-0.1.0-g29096be81982-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Archive SHA-256 `7e0c4624d593f797d65f64b267c4bc9128978d487a7686d6e85da42b791023ce`; binary SHA-256 `7df8724919b05fd532c80bfd7ce9d3343ef9585fa21671e48c3e9b86466cecea`. All223 applicable Linux notice/source packages resolve. No installer/release publication. The normal OSPF unit is playable; the newer multi-unit service core/editor foundation is not itself authored coverage.

The actual installed launcher in `.build-artifacts/installations/verified-29096be81982-2eflmz82/bin/` passes identified877 upgrade, clean uninstall/reinstall, outside-checkout identity/content/hash, desktop validation and exact separate synthetic original-nine-byte retention. Evidence: m68-installed-lifecycle-r1.log and that prefix's acceptance.json. All four normal OSPF introduction/example/assignment flows pass84actual native captures (m68-installed-ospf-r1.log, run-cgz5szfn), including keyboard/pickers, wrong/partial/alternate configurations, fill-only isolation, success invalidation, malformed drafts, exact save/resume, selected reset and Challenge Light150/Dark200. The72-view IPv4/VLAN alignment matrix passes wide/narrow100/150/200 light/dark (m68-installed-alignment-r1.log, run-htak7fy9); root reviewed actual centered placeholders/values and reserved/stacked VLAN controls. Ten quiet default-Wayland launch/readbacks pass10/10 (m68-installed-quiet-r1.log, prefix quiet-startup/results.json), without lifecycle/protocol logs.0.965–0.966s includes harness pacing, not a speed benchmark. The historical unanswered-frame cause remains unresolved.

Guarded ordinary candidate66samples minimum11.49GiB available/PSIzero/OOMdelta0; installed OSPF307samples minimum12.28GiB, peak some/full PSIavg10 0.90%,OOMdelta0. Alignment/quiet69/10samples minimum12.60/12.59GiB,PSIzero/OOMdelta0. Swap was effectively full, including0bytes free in the OSPF interval. These results are scoped observations, not a safety guarantee. All artifacts remain project disk-backed.

[Native CI40049251314](https://github.com/drakahari/dlms_learning_lab/actions/runs/40049251314) at this exact source stopped at formatting: cherry-picked module declarations needed rustfmt ordering. No remote runtime/build acceptance is inferred; the formatting-only correction and rerun are separate evidence. Fedora's actual package checks above passed. Windows/macOS desktop, physical-device, screen-reader and novice acceptance, macOS original dispatch notice and signing/notarization remain pending. Most curriculum/completion criteria remain Open.


## M77 — installed109-lab Fedora candidate

Clean source `5a968c735523574ffffcb9ab4804409846f29993`,109 authored assignments/26units
(31CCNA,21LPIC-1,17LPIC-2,40Bash), evaluator8:

```sh
./dist/dlms-learning-lab-0.1.0-g5a968c735523-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Archive SHA-256 `3031a439608f47d2a1d5f09bdcb9d4b1ae722cb573344ff6cc9c6f195f42246e`;
binary SHA-256 `202c7b993f7517e697a7203e42a8cb511ff2c4c5591ecb2b1ee2dee5d70552cd`.
All223 applicable Linux notice/source packages resolve. This is an owner-test
milestone candidate, not completed curriculum or a public release.

The actual installed launcher in the private project-disk prefix verified-5a968c735523-mzx5b592
passes identified102 upgrade, clean uninstall/reinstall, outside-checkout identity/content/hash,
desktop validation and exact separate synthetic original-nine progress bytes. All three new
case-dispatch native flows pass45 captures: separate example/experiment/repair, full task,
wrong/partial/reference/conditional alternatives, current edit invalidation, actual typing
and Run at light150%/dark200%, save/readback, selected Reset and Challenge. The installed
72-view IPv4/VLAN matrix passes wide/narrow100/150/200% light/dark. Root reviewed actual
centered empty/populated fields, focus/cursor, equal topology cards and reserved/stacked
VLAN controls. Ten quiet default-Wayland startup/readbacks pass10/10. Their approximately
one-second intervals include harness pacing, not a speed benchmark or startup-cause diagnosis.

Evidence: m77-installed-{lifecycle,case-lessons,alignment,quiet}-r1 logs, native runs
run-k9iyd4_w/run-795xb4yi and ignored audits/m77-installed-runtime-r1.json
(SHA256def749e0234e805e8e3b01ac98e77b1e5d47a37f1eb0255ef3c66a00f894b702).
The development executable at that acceptance embedded5a968c7/109/26; SHA256
ac9db3f9abbfb205785154d1edb3b56d6850510b57ebda737db31f3831448c79.
Later integration tests rebuild the development target; that historical hash is
not a claim about its current bytes. The installed candidate remains unchanged.
A subsequent rustfmt-only source change iscee0432; candidate identity remains5a968c7.
The5a968c7 native CI run40101334715 stopped only atFormat on all three runners;
the corrected-source40101440051 run passed Ubuntu tests/Clippy/candidate creation.
Windows hit30minutes during Rust tests; macOS passed its test/Clippy stages but
hit30minutes during release compilation. Both annotations explicitly report the
30-minute job limit; neither is desktop acceptance or a completed current candidate.
The workflow now permits a bounded60minutes, retaining two jobs/test workers and
every check. Its new source rerun is separate evidence.

Ordinary guarded combined model tests observed21.37GiB minimum available, peak some/full
PSIavg10 0.18% and OOM-counter delta0. Candidate building observed22.63GiB minimum/PSIzero;
installed case/alignment runs23.66/23.49GiB minimum/PSIzero/OOMdelta0. Swap use was small
but nonzero in this interval; no global settings or other applications were changed.
These are serialized two-job/two-test-worker observations, not a stress test or guarantee.

Historical intermittent Wayland callback cause, cold-process progress reopening, physical
input/screen-reader/novice acceptance and actual Windows/macOS desktop lifecycle remain
open. macOS original dispatch notice/signing/notarization gates remain. The pure MAC
model in this candidate is not itself an additional authored lab. No release published.


Additional M77 installed follow-through passes all four multi-unit service lessons,72captures inrun-uxkjmlot (m77-installed-unit-manager-r1.log). Root reviewed each introduction and full editor/terminal assignment plus representative wrong feedback/large-text views. Actual flows include separate examples/experiments, diagnosis, wrong/partial/current/alternate outcomes, query purity,150/200% Save/terminal repair, exact drafts/readback, Undo and selected Reset/Challenge. No host service was accessed. This strengthens the same5a968c7 candidate acceptance; it does not add lab titles or prove other-platform, novice, screen-reader or cold-process reopening.


## M84 — Windows CI candidate, desktop acceptance pending

[Native CI40111561758](https://github.com/drakahari/dlms_learning_lab/actions/runs/40111561758)
at exact sourcec41352a8c003bda8f4216860ac2604cc8ce6e463 passes the Windows2025
job, including its tests, Clippy and candidate creation. The downloaded archive
and sidecar agree, its PE executable matches the build manifest, and all25
embedded scenario/unit source packs match their declared hashes and that exact
Git revision. This candidate contains109 labs/26units (31CCNA/21LPIC-1/17LPIC-2/
40Bash), not the later113-lab work. Its target inventory has137 dependency packages.

Local owner-test archive:
`dist/native-ci-c41352a/windows/dlms-learning-lab-0.1.0-gc41352a8c003-x86_64-pc-windows-msvc-windows-candidate.zip`
(36108503bytes).
Archive SHA256`eda9eafb71f484d900159edbf035c84953e045dabbb5ec789dd519cc579f4b73`;
executable SHA256`de9a032815d3fdeb6cc3bf5839bede1d95111c5a37e67336cf4759da84766383`.
Private receipt is audits/m84-windows-c413-candidate.json. No Windows executable
was run on Fedora, no desktop installation/reopen/upgrade/uninstall acceptance
is inferred, and the candidate is unsigned. No release was published.

The same CI run finds synthetic-path assumptions in two reopen tests: Ubuntu's
ordinary temp directory lacks the deliberately required artifact-parent shape,
and macOS exposes a temp parent through a symlink. Test fixtures are being
corrected without relaxing production containment or skipping the tests.
A subsequent full CI rerun is required; neither failed job is reported as passed.


## Verified Fedora113 milestone — 2026-10-24

Clean source `73777e0f094fc51964a4c8e693098d814bb9c0f8`,113 authored labs/27units
(35CCNA,21LPIC-1,17LPIC-2,40Bash), evaluator8,28themes. Open from this checkout:

```sh
./dist/dlms-learning-lab-0.1.0-g73777e0f094f-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

ArchiveSHA256 `b325b68b45faeb93dac21d6f457c23ef1e4b3828c27b0522deb891a27306cfeb`;
binarySHA256 `918e939109dfa782f5bf4142eedb923ddd71742c2bc5248aa8fd8f717e1d0c16`.
All223 applicable Linux dependency/source notices and Rust notice resolve.
This is an owner-test milestone candidate, not a release or complete course.

The actual private installed application in verified-73777e0f094f-ozlvlmhd passes
identified109 upgrade, clean uninstall/reinstall, outside-checkout identity/content/
hash and desktop validation. Exact separate synthetic original-nine progress bytes
remain unchanged. Its full all-four MAC teaching/assignment flow passes57 captures
(run-wvi9hoys), including independent practice, wrong/partial/alternate repairs,
37 capstone requirements, real150%Light/200%Dark edits, current invalidation,
selected Reset Cancel/Confirm, readback, Challenge and library.

The installed72-view IPv4/VLAN matrix passes at100/150/200%, actual2560×1440 and
1268×632, Light/Dark (run-b_j8cmsf). Root inspected blank/populated/focused values,
specific feedback and reserved/stacked VLAN columns. Larger text/short windows
scroll; text is not shrunk. Ten quiet default-Wayland launches/readbacks pass10/10;
approximately one second includes harness pacing and is not a performance measure
or historical callback-cause diagnosis.

The same installed ELF passes strict real two-process cold reopening. Writer and
reader preserve exact observed data, unfinished assignment and separate example
drafts, current run, favorite, theme/scale and migration receipts. Nine captures
and manifest `d4e0204ce0d0db5ab97ae5abf12f602357bbcc8ff2c5a1af702861b7d8671e6e`
remain under ignored tmp/native-reopen-m86-installed-r1. Root inspected real loaded
library, independent draft and current completion screens. No personal profile is
used. Evidence logs are m86-installed-{lifecycle,mac-lessons,alignment,quiet,cold-reopen}-r1;
private delivery receiptSHA256 `4b1334b1bbf0f9698644fa650deb9d244f02164696b5ff193a85aa6149268828`.

Fresh CI40120315031 checks this exact source; its outcome is pending at this entry.
Windows/macOS desktop, physical keyboard/device, screen-reader and novice acceptance
remain pending. The historical Wayland stall remains unreproduced, not diagnosed.
macOS dispatch notice/signing gates remain; no installer security setting is bypassed,
no credentials generated and no public release published. Later source-only seed
pending-draft feedback refinements are not inside this immutable candidate.


### M91 paused owner-test milestone (116 labs, 28 units)

Clean Fedora source `deb55b92a2407a0068b8ba83dff9b71568562893`:

```sh
./dist/dlms-learning-lab-0.1.0-gdeb55b92a240-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Archive SHA256 `c12be61f95c506a455dcc534e533894fa0700ce1600fd9fb54c15bf94aa033df`;
binary SHA256 `f3e6bd5110a1962b590bd1236e59abb7b24179e7f4b0d2e8adc45d09c21000f5`.
Actual installed upgrade/lifecycle/exact synthetic progress checks pass. Full
installed gateway23 and Find46 captures pass, including alternative/incorrect/
partial/current-state results, independent drafts and Light150/Dark200 input.
Shared174 UI regressions and Clippy/fmt pass. Exact deb55 native72-view alignment,
repeated quiet starts and strict two-process cold reopen remain pending at the
owner-requested pause; earlier identities are not relabeled. The historical
intermittent Wayland callback cause remains unresolved. These checks do not
establish novice, physical keyboard, screen-reader or other-platform acceptance.

CI40128237589 at prior source1608a18 passes Ubuntu/Windows/macOS tests and
applicable compile/candidate steps. Downloaded unsigned Windows116 candidate:
`dist/native-ci-1608a18/windows/dlms-learning-lab-0.1.0-g1608a18b0aa0-x86_64-pc-windows-msvc-windows-candidate.zip`,
archiveSHA256 `f2ea12b08eecfe16294cf9f07df45f7c0962ab8bcc6122949e6fc848c14c1503`.
All27 source packs embedded in its PE match exact Git bytes/manifests;137 package
notices/source archives and the Rust standard-library notice were verified. It
predates deb55's scroll correction and has NOT run on a Windows desktop. macOS
compiled but packaging still requires the unresolved original dispatch notice.
No release was published. Detailed resume state is in CHECKPOINT.md.


M92 follow-through2026-10-25: the exact deb55 candidate above now also passes
72 shared native alignment captures,10/10 quiet Wayland starts and strict two
real-process cold reopening9 captures. Root reviewed actual blank/populated fields,
feedback, stacked VLAN controls and cold-loaded independent practice. This
supersedes the pending scoped checks above without relabeling any earlier build.
Historical startup cause and physical-device/screen-reader/novice/Windows/macOS
desktop acceptance remain open. Catalog remains116/28; no new public release.


## M95 current scoped Fedora candidate

Clean source30d75e743ac9a4dc289fbae43e26675dbecfc055,116 labs/28 units:
`dist/dlms-learning-lab-0.1.0-g30d75e743ac9-x86_64-unknown-linux-gnu-fedora-44-candidate`.
Archive SHA256e442beac21532181d24c705932eb6e511cb0f0cb918a65de8718646f8e54b9cc;
binary SHA256a3cba9dfe98eac4310f397d756660828b0ecd604682c1d2d659c6995c2f120f3.
Actual private installed prefix verified-30d75e743ac9-gjoj1ztt passes97b upgrade,
identity/content/hash/desktop and clean uninstall/reinstall, retaining exact
original-nine synthetic bytes (4b74228e2c3f7d090c345222c53b8bb7ed6cca922c546fedcb83def73a88d81e).
Full original DNS/storage teaching flow exits0 with33 captures; actual errors,
partial/alternate/current checks, pending/rejected edits, separated draft resume,
reset, Challenge and light150/dark200 inputs pass. Root reviewed representative
screens. Native XWayland quiet startup/readback passes10/10; default Wayland
fails at first15s launch (0/1), with EGL/Wayland dispatch stack. The missing
callback cause is unresolved; this does not establish default Wayland reliability.
A process-local fallback can be used without changing session settings:

```sh
# Run from the repository folder, or use the executable's full absolute path.
env -u WAYLAND_DISPLAY -u WAYLAND_SOCKET ./dist/dlms-learning-lab-0.1.0-g30d75e743ac9-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab
```

Private receipt audits/m95-installed-acceptance.json hashes every capture and
records build/native memory observations: minimum22.94/24.29GiB available,
PSIavg10 some/full0%, OOM delta0. These scoped checks do not establish a complete
curriculum or novice/accessibility/other-OS acceptance. New source119/29 PTR
integration requires its own candidate and native checks; this116 artifact is
immutable. No public release is published.


## M97 installed119 reverse-DNS milestone

Source72fca8dfb165db576e1c6c329b4ad9809d37055d,119 labs/29 units;
`dist/dlms-learning-lab-0.1.0-g72fca8dfb165-x86_64-unknown-linux-gnu-fedora-44-candidate`.
Archive SHA25677502d07df25523d853daa2cfc3d1e0dfed66abe41dd261b2fe44614b7469a59;
binary af2b32ff7be898effe72bced2560c03cccc31564d5fa1ba3a2d20264cb2ee5ab.
Actual private prefix verified-72fca8dfb165-_hn0ykib passes30d upgrade,
identity/content/desktop/hash/uninstall/reinstall and exact original-nine
synthetic progress retention. All three normal reverse-DNS native flows exit0
with50captures; actual controls, wrong/partial/alternate/current states, separate
practice/drafts/resume/reset/Challenge, light150/dark200 inputs and fullscreen150
are reviewed. This was process-local XWayland; current default Wayland reliability
and new original-terminal native teaching are still pending. Earlier30d startup
0/1Wayland and10/10XWayland evidence is not relabeled as72f evidence. No release.


## M99 default-Wayland scoped candidate

Source11811b76aca48eb87c9232871614334002e59452,119labs/29units:
`dist/dlms-learning-lab-0.1.0-g11811b76aca4-x86_64-unknown-linux-gnu-fedora-44-candidate`.
Archive SHA256e33277ef72ce006e5f2cbe5f4aa1b809d11f25d0387448adba778ea7cfe725e5;
binary bca3c79a753b212588aeaf4ee0e86ad2771876708e6e58c217d77553583eb9fb.
Installedprefixverified-11811b76aca4-jqzlw650 passes c799upgrade and the same
identity/content/hash/desktop/uninstall/reinstall/original-nine retention gates.
New normalLinuxWayland requestedinterval0 passes1protocol+10quiet startups and
strict real two-process private NORMAL-profile reopen9, diagnostic override absent.
This is scoped Fedora44 acceptance, not a general driver cure. The subsequent
source-only original-order/arrayStop harness correction requires its own identified
candidate before arrayStop acceptance; normal app progress/IDs/content unchanged.
No release is published. Windows/macOS runtime/novice/screen-reader stay pending.

## Current122-lab Fedora milestone — 2026-10-25

Candidate sourcecd31af6e78e66c44b5f4fffeeaf65016e625a6ac:
`dist/dlms-learning-lab-0.1.0-gcd31af6e78e6-x86_64-unknown-linux-gnu-fedora-44-candidate`.
Archive SHA256 `b738abf1db9dbc44c772e150b357f8423367c559ff31f2d080201cd453aaecbc`;
executable SHA256 `abceac7b97072a8e6628f0e4dafcc3b2fd7499583a2e8dff8e889196e74514b1`.
Upgrade from1e7, private installation, uninstall/reinstall and exact original-nine
synthetic progress retention pass. All3 new option-interface native flows pass
with51frames;11 native Wayland startup checks and two-process normal-private-XDG
reopen pass. No diagnostic interval override or personal profile is used. Native
render review includes narrow100%, Light150 andDark200; current controls retain
prior shared native wide/narrow alignment evidence. This is scoped122-lab
technical acceptance, not complete curriculum, physical/assistive/novice testing,
Windows/macOS desktop acceptance or a public release.

## M106 accepted125-lab Fedora candidate

Source14cf24839d3ca2223d2b47e888bda4b3a35b51ea,31units and all28themes.
Candidate directory `dist/dlms-learning-lab-0.1.0-g14cf24839d3c-x86_64-unknown-linux-gnu-fedora-44-candidate`.

| Artifact | SHA-256 |
| --- | --- |
| Fedora `.tar.gz` | `8e556d5813de3945dd8cd56fa923725a5f7626f5ff65bb4249d84df34f6f555b` |
| Installed ELF | `d873a1879e76d6bc2c1104e03751a9218e6946a4ef14aad4eecdec5ad217f2c1` |

Actual private installed lifecycle passes125/31: upgrade from86ca, outside-checkout
identity/content/hash/desktop validation, uninstall/reinstall and exact original-
nine synthetic progress-byte retention. Release build stayed above22.13GiB
available memory with zero sampled PSIavg10/OOM increase; two jobs, project-disk
temporary files, no stress test or guarantee under other workloads.

All3 authored DHCP tasks pass the complete installed normal native flow through
actual controls: teaching/separate practice, wrong/partial/reference/alternate,
current invalidation, explicit exchanges, actual150Light/200Dark typing, warm
readback/resume/Reset/Challenge.63 actual PNGs, clean exit0. Earlier failed
86ca capstone capture/inner-list click attempts are retained; the corrected
driver requires real inner visibility and exact list-change acknowledgement.
No simulation/Goal/layout/data weakening.

1 protocol+10 quiet normal-Wayland launches pass~0.8–0.9seconds, no diagnostic
override. Both real2-process normal-private-XDG cases pass: DHCP and unchanged
scripting. Full raw/portable/receipts/independent drafts/favorite/appearance are
compared before navigation, then repaired through native controls. This verifies
synthetic progress and actual process reopening, not personal learner data.

72 actual IPv4/VLAN alignment images cover100/150/200% and light/dark:36 at
1920x1080 fullscreen,36 at946x929 tiled. The requested1100x820 tile was not
honored. Representative placeholder/populated text, label/number columns, focus
and narrow stacking were reviewed; not every image was individually inspected.
Private acceptance receipt42db18ec35daa435700d3e86fc537360cc8db77f9700bfa302961f253f31fd53
pins archive/ELF/lifecycle/startup/cold manifests/63+72images. No screenshots or
synthetic progress files are published in Git.

Current scoped coverage becomes136Partial/1Supporting/1103Open of1240 overlapping
references, not percent complete. Actual Windows/macOS runtime, signing and
notarization, physical-device keyboard/screen-reader/novice comprehension remain
pending. Linux notices were collected; the separate macOS dispatch original-
notice gate remains unresolved. No public release or full-product claim.


## Historical M116 candidate preparation; acceptance paused at that time

Source38a8098c8820b487e54e4d65aa377597ddfc7e04 builds a Fedora44 candidate with
133 authored assignments/33 units. The owner requested a pause immediately after
building. It has NOT yet passed packaged lifecycle, native startup/reopening,
alignment or the eight new lesson flows on this exact release ELF. The last scoped
installed-accepted candidate remains14cf248 above. No release was published.

Open the new candidate for inspection from any terminal directory:

```sh
"$HOME/Documents/dlms_learning_lab/dist/dlms-learning-lab-0.1.0-g38a8098c8820-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab"
```

ELF SHA-256: `6d0257e58ccec397f8d374224b0a6b68bb1816365c29541e0da0995327185ddd`.
Archive SHA-256: `df94f20838a5b712787a4067ed59edb264975434e12acca31d067a6b7c43aa60`.
Builder included223 target dependency packages with notices/source; no new
dependency. Ordinary build minimum21.72GiB available, PSIavg10zero/OOMdelta0,
with the shared two-job workflow; not a stress or universal-memory guarantee.
Exact private candidate receipt: reviews/catalog133/fedora-candidate-38a-r1.
Full source/native-development evidence and next acceptance commands are in
CHECKPOINT.md. This artifact is a development candidate, not complete curriculum.


## M122 refreshed native CI candidates — 2026-10-26

[Run40386416552](https://github.com/drakahari/dlms_learning_lab/actions/runs/40386416552)
at7dde3e45cbc1b8cc7b9f47094cfdf7411eadba86 passed all three native jobs. The
Windows-only unused_mut failure in prior40381197535 was corrected without
suppressing the lint or changing Unix private-directory permissions. Native
headless tests/Clippy, applicable Python/coverage/notice gates and Linux/Windows
candidate creation passed. macOS compiled/tests passed; its originaldispatch
notice gate still prevents a candidate. These archives contain140 assignments/
35units and the accepted GUI, before the later unbundled NAT backend. They are
unsigned owner-test candidates, not desktop runtime acceptance or a public release.

| Retained candidate | SHA-256 |
| --- | --- |
| `dist/native-ci-7dde3e4/ubuntu/dlms-learning-lab-0.1.0-g7dde3e45cbc1-x86_64-unknown-linux-gnu-ubuntu-24.04-candidate.tar.gz` | `45ce3ffe5cba68416d6070e58c2160cc56fcb1891845e7bb9d92d31f427e9b24` |
| `dist/native-ci-7dde3e4/windows/dlms-learning-lab-0.1.0-g7dde3e45cbc1-x86_64-pc-windows-msvc-windows-candidate.zip` | `4b82d722cbee629e8cd6c118dfad9c3775c4c5e377b6d268ac4008fca7280b43` |

Downloaded to verified project disk with per-command disk TMPDIR/TMP/TEMP.
Sidecar hashes and bounded embedded BUILD_INFO revision checked without running
or extracting those applications. Private receipt reviews/ci-7dde3e4-candidates-r1.
Actual Windows/macOS install/reopen/upgrade/uninstall, keyboard/scaling/accessibility
and OS-security acceptance remain pending. No signing credentials were generated,
no security policy bypassed and no release published. At that milestone, the installed Fedora
accepted command was ddb886b; its separate checksum/evidence remains historical.


## Historical140 acceptance record

Historical140 scoped Fedora candidate: **ddb886b4d418**,140 authored labs/35 units
(42CCNA,28LPIC-1,24LPIC-2,46Bash). Open directly from any terminal directory:

```sh
"$HOME/Documents/dlms_learning_lab/dist/dlms-learning-lab-0.1.0-gddb886b4d418-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab"
```

The same installed ELF passes upgrade from d781/uninstall/reinstall/desktop
validation and exact synthetic original-nine progress retention; eleven normal
Wayland starts; actual two-process script/DHCP reopening;72 shared alignment
captures; and all seven new lesson flows (68 static-web and72 scripting captures).
The earlier133 candidate separately passed all eight package/STP flows; those are
not claimed repeated in this installed pass. Representative100/150/200% light/dark
renders, fields/columns/focus/feedback reviewed. Apache/alignment measured946x929
and1920x1080; scripting measured946x929 even when a wider size was requested.
No diagnostic backend override, desktop changes or personal-progress inspection.
Technical synthetic inputs are not novice, physical-device, screen-reader or
other-platform acceptance. New-seven resume uses exact warm raw/portable/receipt
readback; script/DHCP cold reopening is a distinct actual two-process check.

ELF SHA-256: `90690062330eaa4b4a946c6eaaa282b5e221c42f770f672155025afbc24d0dc8`.
Archive SHA-256: `bb1c89b52c1d5b043b2fb7fbdf542c73e3cf86910915723b4417aa3978f756ea`.
Private accepted receipt: reviews/catalog140/installed-ddb-acceptance-r1/RESULT.json,
SHA-256 `88d8434c189936113ba68dc5b9baaba038b364e60fca90850bab692dd0a2c2c8`.
Linux223 notices and exact source archives passed the candidate gate. This is an
unsigned development milestone with incomplete curriculum/platform acceptance.
The retained d781 candidate failed the scripting Stop capture; it is not relabeled
accepted. Its replacement changes only exact capture Run-acknowledgement pacing,
with an actual-frame regression; normal execution and budgets are unchanged.
Historical acceptance entries below retain their own identities.

## Historical154 candidate before the UI fixes — 00d69d3

The following describes the earlier executable only. Its two UI follow-ups are
corrected in the current9a90f275e133 candidate above.

Current scoped Fedora candidate: **00d69d3c09ea**,154 authored labs/39 units
(45CCNA,32LPIC-1,28LPIC-2,49Bash). Source commit
`00d69d3c09ea3ae050148ac421e2ec9556ae3716` is pushed; the following
delivery-documentation commit does not change this executable. Open from any
terminal directory, including your home directory:

```sh
"$HOME/Documents/dlms_learning_lab/dist/dlms-learning-lab-0.1.0-g00d69d3c09ea-x86_64-unknown-linux-gnu-fedora-44-candidate/dlms-learning-lab"
```

The actual installed regular executable passes upgrade from87121151e82e, clean
uninstall/reinstall, desktop validation, outside-checkout identity/content checks
and exact synthetic original-nine progress-byte retention. The same ELF passes
all four new NFS lesson flows106 captures, all three new scripting flows72, all
three NAT flows80, and shared alignment72 captures. Wrong/partial/reference/two
alternative solutions, current-success invalidation, isolated examples, warm full
raw/portable/receipt readback, reset and Challenge are exercised. Actual light/dark
100/150/200% screens were reviewed. NFS/NAT/alignment measured1920x1080 and946x929;
scripting measured946x929 even where wider sizes were requested. Requested window
sizes are not acceptance evidence. Independent render reviews viewed25NFS/21Bash
frames; Root also inspected representative networking/alignment screens. All330
flow/alignment captures are indexed, not claimed all individually viewed.

Ten quiet Wayland launches/readbacks pass0.865–0.916s; one separately traced
Wayland launch passes. Eleven successes do not prove intermittent failure
impossible. Actual two-process original script/DHCP reopening also passes.
New-seven resume is warm readback, not seven cold-process tests. Expanded optional
NFS inventory was source/event tested but not visually reviewed in this pass.
These are synthetic technical checks, not novice, physical-device, screen-reader
or Windows/macOS runtime acceptance. No desktop/backend override or personal
progress inspection. All28 palettes remain; installed visual sampling is light/dark.

Two nonblocking UI follow-ups remain: script output comparison can show the
selected passing case while other cases fail (the per-case list and grading are
correct; select a failing input to compare it), and an NFS editor scrollbar can
offset adjacent Save rows. This is not full curriculum or perfect UX acceptance.

ELF SHA-256: `8b6861790f8371da789b6b6187a9484a00e14fe13cda13fa176ac4bce2f18fbb`.
Archive SHA-256: `be6480590bd0dfa94092f8f50f883c34e4e4bcb478323b4e9161147d9749b540`.
Archive: `dist/dlms-learning-lab-0.1.0-g00d69d3c09ea-x86_64-unknown-linux-gnu-fedora-44-candidate.tar.gz`.
Private receipt: reviews/catalog154/installed-00d-acceptance-r1/RESULT.json,
SHA-256 `eab3b0c281d070e0bce5e63cd05807650daf686815c925b1d69e176ba4590ca6`.
Linux223 dependency notices and exact source archives pass the candidate gate.
All work used the shared two-job/two-thread runner and disk-backed artifacts.
Build minimum21.76GiB available; installed checks minimum23.79GiB, PSI0/OOM0.
Those observations are not a memory reservation or stress-test guarantee.
No release or new cross-platform runtime acceptance. Older candidates remain intact.
