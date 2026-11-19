# LVM configuration/scan prerequisite — no new lesson coverage

2026-11-07. Two new source lessons are registered under verification (333/114),
not yet a verified Fedora-native/package milestone. The preceding verified331
T153459Z review package stays immutable and does not contain this profile.
204.3.T6 remains Open until authored/native admission checks pass. No new
shell-evaluator revision, dependency or real LVM utility is introduced.

The pure parser/scan12, typed workspace13, source/session6 and actual egui6checks
pass. Retained reference5/snapshot6session checks pass; staged Clippy passes.
Before authoring, CURRENT-only literal108/ERE117guards passed and twenty exact
profile inputs were retained privately (config-r43-profile-proof.json).
Those checks do not establish an independent review, Fedora rendered acceptance,
full Linux conformance or a complete source closure. Authored checks and new
native evidence remain separately pending.

The optional configuration task, practice and runtime fields are omitted by old
sources/sessions. New work uses separate bounded editor drafts, saved policies
and actual multi-inventory scans. Normal learner shell/LVM commands remain refused
in this workspace. The virtual storage remains unchanged; its metadata is read
through the pure scanner. Typed authoring references are not Linux commands.

## Closed configuration grammar

One devices section with explicit use_devicesfile = 0 or 1 and a filter array of
1–8double-quoted rules. Both settings are required here so the model never guesses
a distribution's defaults. Whitespace, CRLF, comments, either setting order and
current accept/reject delimiter conventions are parsed as syntax. Each rule has
an a/r action and a bounded expression; internal alternation remains part of the
expression rather than being split into extra rules. Paired delimiters are
supported. Nothing is passed to a shell.

Other sections/settings, repeated settings, quoted backslash escapes, empty
expressions, trailing array commas, implicit defaults and non-ASCII configuration
are explicitly outside this unit. Missing punctuation is a syntax diagnostic;
well-formed behavior outside the subset is unsupported, not silently approximated.
Expression syntax uses the existing C-byte bounded subset; see SHELL_SUBSET.md.

Limits:4096configuration bytes,8rules,128expression bytes per rule, and existing
expression compilation/state/depth limits. Eight supplied simple /dev/NAME device
names and64KiB virtual inventory are admitted. One shared65536-transition matching
allowance covers every rule/device in a scan; it is not reset per device. A limit
failure returns no partial report and changes no storage. Scan output has at most
eight records and eight missing devices-file entries.

## Actual virtual behavior

The first matching rule for the one supplied path decides accept/reject. Unmatched
paths are accepted; the model does not invent an implicit safe rejection. An
accepted path reads actual virtual PV-label/group metadata, including ungrouped
PVs; rejected paths expose no read result. Foreign/blank devices have no modeled
PV label. Actual modeled VG renames change the returned group information.

With use_devicesfile enabled and a provided file present, its entries select the
devices and filters are ignored. A present empty file selects none. A missing
file falls back to filters; explicit use_devicesfile=0 selects filters even if a
file is supplied. Missing entries are reported without inventing devices or
reading anything outside the virtual inventory. This is a devname-only model;
real PVID/hardware-ID selection, sysfs, aliases, udev, symlinks, global_filter,
overrides and real devices-file serialization are absent. Scanning is read-only,
not formatting, provisioning, mounting or activating real storage.

## Sources checked2026-11-07

- [Upstream lvm.conf(5)](https://man7.org/linux/man-pages/man5/lvm.conf.5.html):
  configuration sections, assignments, comments and settings/override context.
- [Upstream filter implementation](https://raw.githubusercontent.com/lvmteam/lvm2/main/lib/filters/filter-regex.c):
  current rule delimiters, priority, unmatched behavior and device-file interaction.
  Consulted as a behavior reference; no upstream C code is copied or bundled.
- [Upstream lvmdevices(8)](https://man7.org/linux/man-pages/man8/lvmdevices.8.html):
  provided-file precedence, empty/missing file and filter fallback distinctions.

The referenced upstream main file can change; this records the check date, not an
invented immutable revision. No real LVM, host configuration or device comparison
was performed. Existing normal learner commands still refuse lvmconfig/lvmdevices.
The next gates are typed goals, separate drafts/practice, bounded source imports,
persistence, real native configuration/reader interactions and substantive authored
assignments before any new curriculum or partial-objective credit is registered.
