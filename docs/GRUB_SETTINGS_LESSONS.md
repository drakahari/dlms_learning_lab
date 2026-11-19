# GRUB settings: generate and verify

LPIC-1 → **GRUB settings: generate and verify**, immediately after **GRUB menus:
choice and recovery**. Prerequisites are advisory. Existing IDs and progress stay
available; the new assignments have stable new IDs and receive no inherited credit.

| Assignment | Practical work |
| --- | --- |
| `lpic1-grub-persistent-default` | Generate a persistent rollback choice with a usable menu wait, then verify the supplied paths. |
| `lpic1-grub-argument-scope` | Repair common arguments affecting normal and recovery entries while retaining normal-only diagnostic suppression. |

Introduction explains source settings, supplied templates, generated output and
verification, with separate worked workshop settings. Try example uses a different
UUID, IDs, ordering or timing. Its four steps sit beside Edit, Generate, Inspect
and Test. Assignment preserves intentional errors and independent requirements.
Challenge keeps the full assignment without unsolicited worked settings.

## Defined simulation

Only four literal settings in the virtual `/etc/default/grub` are accepted:

| Setting | Meaning in this activity |
| --- | --- |
| `GRUB_DEFAULT` | Existing supplied ID, quoted title or zero-based position. Unset uses `0`; saved environments are unsupported. |
| `GRUB_TIMEOUT` | Integer seconds. Unset uses `5`; `0` starts immediately and `-1` waits for a selection. The simulator bounds values to −1 or 0–300. No real timer or boot occurs. |
| `GRUB_CMDLINE_LINUX` | Literal supported kernel arguments appended to every supplied Linux template. |
| `GRUB_CMDLINE_LINUX_DEFAULT` | Additional arguments appended only to normal templates, after common arguments. This class is independent of the selected `GRUB_DEFAULT`. |

Unspecified argument settings are empty. Later assignments replace earlier ones.
Single/double literal quotes, concatenated literal spans, escapes and ordinary
whitespace-delimited comments are supported. Control characters, sourcing,
expansion (including unquoted tilde), shell commands, exports, continuations and unknown keys fail explicitly.
Kernel arguments are limited to `ro`, `rw`, `quiet` and the previously modeled
ordinary/rescue `systemd.unit=` requests. The templates own image paths and root
UUIDs. Repeated ambiguous target options remain explicitly unsupported by the
shared early-boot model. Later `ro`/`rw` takes precedence within that subset.

Ordered source-held templates explicitly identify normal and recovery entries.
Generation appends settings to their typed image/root options, renders a bounded
literal menu and checks it through the existing parser. It never executes `/etc/grub.d`
scripts, discovers a host distribution, reads files, runs `grub-mkconfig`, installs
a loader or changes firmware. The read-only output includes a typed timeout; the
older literal-menu trace consumes the generated entry portion, not an invented
interpretation of arbitrary GRUB scripts. Full discovery, distro-specific output,
saved environments, Legacy, interactive loader, MBR/ESP and firmware remain open.

`quiet` presence and initial root mode are configuration outcomes. They do not
prove real console behavior, remounts, an actual kernel or complete exam preparation.

## Generation, grading and progress

**Generate menu · simulation** commits a bounded generated observation. **Test
generated menu · simulation** requires it to match the current settings. Editing
retains old output as explicitly outdated and clears current success; filling a
worked example does not generate or test it. Actual test results distinguish no
current generated menu from a generated configuration that fails its timer,
selected-release, initial-mode or per-entry quiet requirements.

Grade resulting selected handoff and every supplied path, not exact source text.
Equivalent selectors, quoting, comments and supported argument combinations pass.
Both normal and recovery paths must reach the supplied system with the requested
initial mode; repairing only the selected normal path cannot hide broken recovery.
Expected task requirements, actual generated output and tested outcomes stay distinct.

`GeneratorPolicy` is opt-in source metadata; `GeneratedRecord` stores only bounded
settings/output/timer observations. Old absent fields are omitted from serialization,
so all previous 378 scenario objects and 140 unit values remain exact. Saved data
cannot replace templates or inventory. Restore revalidates generated output against
its source-held facts. Unfinished invalid editing can resume alongside older valid
generated output. Assignment and practice drafts are separate; reset is selected.

## Evidence and limits

Initial core parser/generator checks: five groups. New authored/grading/registration/
progress checks: six groups. Native UI/capture regressions: four groups including
actual keyboard editing, generation, test, stale results, retry, Challenge, selected
reset, five supported text sizes, two widths and light/dark layouts. Older boot/menu
compatibility checks passed before registration. Actual backup/inspect/restore with safety copy, tamper refusal, retained regressions,
all-target Clippy, 1,018 Python and strict coverage/inventory pass. Exact packaged
results are tracked separately in CHECKPOINT.md.
Actual native R1/R2/R3 complete 58/61/63 actions and 15/16/17 captures respectively.
Individual source screens were opened for both tasks in all three views; visible
kernel facts, readiness labeling, early failure feedback and wrapped output were
refined from those images. Ready380R1 now passes exact content/native/NFS/private lifecycle/identity gates.
The packaged first-rows200%light capture was actually opened and shows both entries
without horizontal clipping. Paths, hashes and receipts are in CHECKPOINT.md; this
is not an installed-path GUI or generated-GRUB separate-process reopening claim.

No actual GRUB reference execution, independent novice study, screen-reader,
physical-device or additional-OS runtime acceptance is claimed. Owner review of
worked answers is assisted review. Nothing here changes original progress or the
installed app.

## Sources, checked 2026-11-10

- [LPI LPIC-1 objectives v5.0, 101-500/102-500](https://www.lpi.org/our-certifications/exam-101-102-objectives/): partial 102.2 configuration/generation practice; installation and interaction remain open.
- GNU GRUB Manual 2.14: [simple configuration](https://www.gnu.org/software/grub/manual/grub/html_node/Simple-configuration.html), [grub-mkconfig invocation](https://www.gnu.org/software/grub/manual/grub/html_node/Invoking-grub_002dmkconfig.html), [timeout](https://www.gnu.org/software/grub/manual/grub/html_node/timeout.html). Official indexed text supports selection, timer defaults and argument order; direct GNU fetches timed out. No full-generator conformance follows from these references.
- [Linux kernel parameters](https://docs.kernel.org/admin-guide/kernel-parameters.html), for the previously disclosed root/mode/target subset.

Original code and authored examples only; no external scripts or manual passages
are redistributed. Existing project and third-party notices remain intact.
