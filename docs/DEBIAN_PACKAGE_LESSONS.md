# Debian package unit — registered, development native verified

Four distinct assignments are bundled in `scenarios/debian-packages-v1.json`.
The catalog is133 labs/33 units. All four package lessons pass model/content,
development-native and installed Fedora38a8098 flows, including separate practice,
alternatives, current invalidation, retry and warm resume. Full lifecycle/startup/
cold-reopen/alignment checks use that same installed ELF; novice and other-platform
acceptance remain pending. Only a narrow102.4 parent slice is Partial practice.

| Task | Practical outcome | Distinct reasoning |
| --- | --- | --- |
| `lpic1-package-install` | Install a requested report version, preserving neighbouring tools and notes | Installed records, available software and dependencies are different |
| `lpic1-package-metadata` | Refresh stale metadata and install a newer tool with its newer dependency | Refreshing is not upgrading; dependency-first and automatic closure both work |
| `lpic1-package-payload` | Repair ordinary files at the same version while retaining local configuration | Recorded ownership and installed status do not prove intact files; ordinary same-version install is a no-op |
| `lpic1-package-handover` | Remove one tool but retain its settings; purge another; preserve working dependents | Retirement policies differ; deleting everything fails the positive preservation requirements |

Prerequisites progress through these four tasks. Each introduction teaches its
new concepts and literal command spellings before assessment. A different gallery
example uses its actual separate paths and catalog identities. Practice changes
and repairs have independently authored expected result vectors. Commands in the
assignment reference are real supported spellings; internal JSON actions are not
taught as Linux syntax. Optional guidance is admitted through the same bounded
parser, including a required simulator-only label for console help.

Eight content checks pass: actual command references, legitimate multi-package/
dependency-first/reversed-order alternatives, queries without repair, stale list,
partial changes, no-op install, purge losing customized configuration, fresh
success invalidation and retry, example isolation, exact experiment results,
unsupported-command atomicity, save/reload and selected reset. The broader gate
passes68 library and80 integration checks, then all-target Clippy/format checks.
This evidence is model/content verification, not beginner or native acceptance.

Initial source review caught practice goals referring to nonexistent paths; the
authoring data now derives paths from its own supplied artifacts. Validation also
caught the capstone missing an explicit positive Installed keeper, and guide/step
cardinality. Those data errors were corrected without weakening validators or
regenerating expected behavior. A test-format compilation typo was corrected.
The original adapter's reject-all guide policy was replaced by actual parser
validation once the real command subset existed; unsupported commands still fail.
All initial evidence is retained privately.

The app never invokes dpkg/APT or a host shell. This is script-free final-state
simulation over declared text payloads, not downloads, archive/maintainer-script
execution, full dependency resolution, security verification, real transaction
rollback or Fedora RPM administration. See [the subset](DEBIAN_PACKAGES_SUBSET.md),
[app adapter](DEBIAN_PACKAGES_WORKSPACE.md) and
[command boundary](DEBIAN_PACKAGES_TERMINAL.md).

Primary sources checked2026-10-25: [LPIC-1 v5.0,102.4](https://www.lpi.org/our-certifications/exam-101-102-objectives/),
[APT3.0.3 apt-get](https://manpages.debian.org/trixie/apt/apt-get.8.en.html),
[dpkg1.22.22](https://manpages.debian.org/trixie/dpkg/dpkg.1.en.html) and
[dpkg-query](https://manpages.debian.org/trixie/dpkg/dpkg-query.1.en.html).
Only a narrow future102.4 slice is proposed; no full exam preparation claim.


2026-10-26: all four normal lessons also pass on installed Fedora38a8098, the
same ELF6d0257e5 used for lifecycle, startup, cold-reopening and alignment checks.
Package69/STP48 actual captures and representative rendered review establish the
scoped installed milestone; earlier source-only/development notes are historical.
No actual package utilities/BPDUs/protocol/other-platform or novice acceptance.
See INSTALLATION.md for complete identity and retained limits.
