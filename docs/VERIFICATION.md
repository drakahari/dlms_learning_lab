# Integrated workspace verification

Fedora desktop, 2026-10-21: native Rust/egui/eframe Glow renderer on COSMIC Wayland. Rust 1.99.1, pinned egui/eframe 0.33.3. No added dependencies or system packages. Nine authored labs; no installer or release publication.

## Checks

- Offline development build, `cargo fmt --all -- --check` and Clippy with warnings denied.
- **46 tests**: nine normal-UI checks, eight IPv4 math/legacy-metadata checks, 22 existing engine/content/catalog/progress checks, and seven workspace/migration/editor checks.
- Bundled declarative pack validation: all nine initial states unsolved and reference solutions reachable.

The checks cover all nine success paths and state breakage after success, wrong and partial native answers, malformed drafts, service saved/runtime/boot distinctions, DNS staged/loaded data, mount persistence and capacity, alternate Unix ownership/permission repairs, alternate VLSM layouts and larger valid allocations, log pipelines, unsupported commands and no shell escape. New row diagnostics distinguish format, alignment, capacity, containment and overlap. Unix feedback names a blocked traversal component or the applicable permission class.

IPv4 tests cover all conventional prefixes /0…/30, known octet-crossing results, all 256 last-octet host candidates in the supplied /27 (29 unused usable choices), duplicate/reserved/outside/malformed hosts, second hosts at /23 and /30, and 500 deterministic numerical cases. /31 and /32 remain excluded. Original and numerical-practice state are independent. Legacy prediction/review metadata remains loadable; it is no longer a walkthrough gate.

UI checks cover both Learn and Challenge at 1200×850 and 720×560 with 100%/150% text, aligned IPv4 fields, visible desktop Check actions, keyboard focus/Tab/Shift+Tab/Enter, optional assistance, scroll access to Insert command without execution, native editor Save, reset confirmation/isolation, resume of unfinished commands and drafts, and library topic/search/favorites/filter/pagination behavior with 900 synthetic entries. These tests verify mechanics/geometry rather than beginner usability acceptance.

## What ran and was visually inspected

The normal native application’s `--ui-smoke` capture harness used fresh synthetic data under `/tmp/dlms-workspaces-smoke`. It captured **every lab in Learn and Challenge**, the library, topic filtering, extra filters, service save/restart/enable/status, filtered logs and subsequent invalidation, the checked second-host topology and a later conflicting host, the requested binary explanation, 150% text and real tiled/full-screen transitions. The renderer/input path uses injected egui text, keyboard and pointer events. Service restart/enable commands are submitted through the same in-memory parser; this is not a physical mouse/keyboard study or manual execution of every alternate sequence.

Screenshots were inspected for task visibility, field/button alignment, readable state, optional-help defaults and resizing. Relevant local ignored captures:

| View | File under `screenshots/workspaces/` |
| --- | --- |
| Library | `00-library.png` |
| All nine initial workspaces | `<scenario-id>-learn.png` and `<scenario-id>-challenge.png` |
| Valid entered second workstation | `ipv4-verified-topology.png` |
| Editing clears previous success/status | `ipv4-edit-invalidates-success.png` |
| Checked conflicting host | `ipv4-checked-invalid-host.png` |
| Explicit network/host split and zero/one rules | `ipv4-binary-explanation.png` |
| Working service and boot configuration | `service-configured-and-verified.png` |
| Terminal practice and invalidation | `terminal-filtered-log.png`, `terminal-state-invalidates-success.png` |
| Larger text | `library-150-percent.png`, `ipv4-150-percent.png`, `service-150-percent.png` |
| Large navigation fixture | `developer-large-catalog.png` |

## Progress and private-data checks

A synthetic all-nine-lab legacy fixture still loads without rewriting the original. Tests verify exact legacy answers, transcript, hints, mode and completion, partial malformed drafts, original/practice isolation (including return from wider host-bit subnets), all-nine workspace-draft round trips, future-version rejection and selected-attempt reset. Completed legacy IPv4 analyses receive an explicitly disclosed compatible second-host migration; incomplete analyses receive a blank fifth field. A byte-for-byte pre-workspace backup is verified, including non-overwrite on later saves. Normal learner files were not used by the native harness.

The public diff excludes actual learner progress/backups, generated screenshots, recordings, local configuration and credentials. Screenshot artifacts remain local and ignored. Repository license selection remains the owner’s pending decision.

## Not verified / limitations

- Windows/macOS builds and runtime acceptance, installers and distribution packages.
- Physical-device, screen-reader/AccessKit, IME, high-contrast-theme and multi-monitor DPI acceptance, or a beginner usability study.
- Concurrent application writers and downgrade to older binaries after a new-format save. Small windows and expanded optional explanations may require vertical scrolling; long terminal lines scroll horizontally.
- Full Linux/Cisco command behavior, real networking, processes, sockets, filesystems, ACLs/SELinux or complete certification coverage. Configuration files are deliberately limited field editors, not full text editors. Sources and simplifications remain in the pack and secondary UI references.
