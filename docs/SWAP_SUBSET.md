# Source-enabled swap policy: bounded implemented subset

2026-11-06. Two registered lessons implement this narrow subset in source312. The
scoped312-lab Fedora review package T165212Z includes it; no full Linux conformance. The existing storage model gains optional source-held
swap facts and active entries; absent fields retain old serialized shapes. An old
lesson cannot acquire swap merely through imported active state. Existing authored
inventory equality remains authoritative. Removable/caller models refuse the new
facts explicitly; their tools do not implement this extension.

## Deliberate subset

Up to four supplied areas: exact bounded identities, device aliases and UUIDs,
usable size (4KiB multiples, at most1TiB), availability and prepared signature.
No actual disk, filesystem or memory is accessed. The supplied usable size is not
a measured raw partition capacity. Active entries store explicit priority0–32767;
higher numbers mean higher priority. No paging workload is simulated. Read-only
reports label usage zero within that finite model, not as a host measurement.

The opt-in parser reuses the established mount parser and preserves original line
numbers. Swap rows use six literal fields: source, `none`, `swap`, options, `0`,
`0`. Source is a supplied `/dev` alias or `UUID=...`; options are `defaults`,
`auto`/`noauto` (a remaining noauto token prevents bulk activation regardless of auto ordering) and one required `pri=N`. Repeated priorities, other
options, escaped/quoted fields, inline comments and duplicate area policy are
refused as unsupported. Real swapon may ignore unknown options: the simulator's
refusal is not a universal Linux error rule. The target is unused for swap;
`none` and trailing zeros are this activity's required conventional format.
Kernel-selected negative priorities and broader fstab forms are unmodeled.

Saving only changes policy text. Activation stores the requested priority;
changing saved `pri` does not reprioritize a live area. Bulk activation skips
already-active areas and `noauto` entries. An explicit request may activate a
`noauto` area. To change live priority in this subset, deactivate, then activate.
These operations assume supplied areas have no paging workload; no real swapoff
memory-pressure, syscall, OOM or safe-operational guarantee is simulated.

Bulk activation first validates complete supported syntax/duplicate policy. That
preparse refusal is a disclosed simulator restriction. Once syntax is admitted,
unknown, unavailable or unprepared entries fail individually, retaining other
successful activations. Diagnostics/outcomes cannot claim a failed row succeeded
or roll back earlier successes. Do not copy swapoff's documented exit codes onto
swapon; no full utility exit-status conformance is claimed here.

No swap files/holes, Btrfs extents, zram/zswap, discard, compression, encryption,
hibernation, page allocation, real boot/systemd units, live resource measurement,
kernel defaults, automatic priority changes or host commands. Partition preparation
remains the separate existing disk activity. This extension will not run mkswap.
The existing native editor and literal terminal implement this subset; state-based grading checks current and independent alias-permuted policy outcomes.

## Actual checks and status

The unit is registered and scoped Fedora packaged-native verified at312/105;
see final evidence in SWAP_POLICY_LESSONS.md. R47all13core/binding groups, R46workspace5,
R48authored/registration6 and R49actual UI/capture8 pass. Retained storage17,
workspace15 and removable10 pass. All310old scenarios/104units remain exact.
R40native129actions/45captures exits0; normal teaching/workspace/partial feedback,
light150 and actual dark200field images were opened. The latest capstone teaching
text and noauto-source correction pass the actual rendered-text/widget checks;
final packaged native review subsequently passed (R86/run-opmih1s7). This is technical, assisted review.

The old mount parser keeps its established option semantics. Only the new swap
rows use noauto-presence behavior, checked against util-linux v2.41.2 source; both
contradictory orders are tested. No host swap reference was run. Source-code review
is not kernel/runtime conformance. A new uppercase test initially assumed an
impossible old inventory; R45retains12pass/1fixture failure. The corrected test
verifies the existing lowercase guard, the new canonical guard and overlapping
identity refusal. No old assertion or validation/resource boundary was relaxed.

See SWAP_POLICY_LESSONS.md for the teaching progression, actual evidence and
retained earlier setup/capture failures. No whole-inclusive Rust, installed/cold
lifecycle, physical-device, full keyboard-only, all28theme, screen-reader, novice
or Windows/macOS/Omarchy acceptance is implied. Original progress/install and
historical shell proof remain unchanged; no dependencies or host operations.

## Primary references

Read2026-11-06; original implementation/prose, upstream sources referenced rather
than copied into the engine:

- [swapon/swapoff manual](https://man7.org/linux/man-pages/man8/swapon.8.html)
- [fstab manual](https://man7.org/linux/man-pages/man5/fstab.5.html)
- [util-linux v2.41.2 swapon logic](https://github.com/util-linux/util-linux/blob/v2.41.2/sys-utils/swapon.c)
- [util-linux v2.41.2 option retrieval](https://github.com/util-linux/util-linux/blob/v2.41.2/libmount/src/fs.c)
- [LPI LPIC-1 v5.0/104.3](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
- [LPI LPIC-2 v4.5/203.1](https://www.lpi.org/our-certifications/exam-201-202-objectives/)

No host swapon/swapoff/mkswap/mount/reference operation or memory-pressure test was
run. Native, platform, physical-device, screen-reader and novice acceptance are
separate gates; passing a pure model test cannot substitute for a usable lesson.
