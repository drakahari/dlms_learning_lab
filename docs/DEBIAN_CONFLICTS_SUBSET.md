# Explicit package incompatibility

Implemented source capability2026-11-04; new lessons/native acceptance pending.
This extends the script-free final-state Debian engine. It executes no host
package tools, maintainer scripts, downloads or archives.

Archive metadata can contain up to8named `Conflicts` relationships, optionally
restricted by the existing Debian version comparator's `<< <= = >= >>` subset.
Empty relationships default to empty and are omitted on serialization, keeping
old archive and progress wire values unchanged. Either package's declaration
applies: installation checks the complete selected installed set, including
newly selected dependencies and retained installed counterparts. It checks
actual installed versions, not a newer cached candidate. A real package's own
name is ignored; configuration-only leftovers do not conflict.

Install/reinstall refuses a remaining conflict before publishing any state.
Inspection, metadata refresh and removals remain available on an authored broken
state. The model does not make full APT removal/resolver choices: learners must
explicitly retire incompatible software or choose a supported compatible version.
This is deliberately not real dpkg unpack/configure ordering or rollback.
A new ConflictsSatisfied requirement checks actual state separately from positive
installed/version, file-integrity and preservation requirements. Existing goals
are unchanged.

`apt-cache show NAME` reads one cached candidate and prints the available
Package/Version/Architecture and supported Depends/Conflicts fields. It changes
no state and does not display installed status. The default `dpkg-query -W`
shows database records and versions; configuration-only records can still appear.
The existing formatted status field is available when a caller needs to inspect
that distinction explicitly. Metadata fields or arbitrary output are not credit.

No Breaks, Replaces, Provides/virtual packages, Essential/deselection behavior,
multiarch, automatic removal/SAT/pinning, maintainer scripts, real binary payload
execution or archive handling. Unsupported relationships/syntax are refused.
The fixed universe remains24names/72artifacts, bounded payload/config text and
10,000logical work units; conflict scanning adds at most24×8clauses. Existing
command, transcript, pack and progress caps are not increased.

Sources checked2026-11-04:

- [LPI LPIC-1 v5.0,102.4](https://www.lpi.org/our-certifications/exam-101-102-objectives/).
- [Debian Policy4.7.4.1,7.1/7.4](https://www.debian.org/doc/debian-policy/ch-relationships.html#conflicting-binary-packages-conflicts).
- [Debian trixie apt-cache(8)](https://manpages.debian.org/trixie/apt/apt-cache.8.en.html).

69model/retained query/workspace checks pass, including8new policy groups;
4authored content/practice/progress groups pass before registration. Vectors are
authored from primary references, not observations of a host APT installation.
Broad102.4, real package administration and platform/novice acceptance remain open.
