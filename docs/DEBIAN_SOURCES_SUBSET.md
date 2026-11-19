# Debian source selection and cached metadata

Source255/73 in development2026-11-02; two new source-selection tasks are
registered and under verification. They are not yet part of a verified candidate.
The253lab/72unit Fedora review candidate remains immutable.

## What is modeled

`debian_sources` parses an explicitly limited deb822 **selection excerpt**:
Types: deb, URIs, Suites, Components and Enabled: yes/no. Field names are case
insensitive. Supported whitespace-separated values, continuation lines, whole-
line comments and blank-separated stanzas keep their normal relationships.
A source target is an ordered URI/suite/component combination. Repeated targets
share an index without changing the first occurrence's order. Source addresses
are canonical offline identities; no address is opened.

`debian_source_cache` separates saved configuration, supplied publications and
cached snapshots. Saving valid source fields changes which targets are eligible
immediately, without fetching. Retained cache bytes for a disabled/removed target
are not query evidence. Enabling a target without a cache leaves it missing until
an explicit simulated refresh. Refresh copies supplied publications independently;
an unavailable target reports failure and retains its old matching cache. Older
matching evidence can still be inspected. Exact package-name queries derive rows
from active matching caches, retain source/snapshot provenance and filter by the
supplied native architecture plus architecture-independent packages.

This is **metadata evidence**, not APT policy candidate selection or dependency
solving. Saving/refreshing/querying never installs software or changes installed
package records. Cached current/retained/missing states are distinct from whether
the learner's eventual assignment meets its requirements.

## Explicit limits

- At most8KiB/128lines/four stanzas, two URIs/two suites/three components per
  stanza; at most24deduplicated active target keys. Four stanzas permit separate
  required targets plus a retained disabled entry; active keys remain bounded.
- Narrow ordinary lowercase HTTPS host/path labels. No credentials, ports,
  address literals, percent escapes, traversal, query/fragment or file URI opening.
- At most six supplied targets, three immutable snapshots per target and24rows
  per snapshot. Aggregate metadata text is limited to64KiB; raw definition JSON
  to256KiB and saved-workspace JSON to32KiB. Strings/lists are bounded during
  deserialization; duplicate/foreign cache references and unknown JSON fields
  must be rejected. Snapshot versions use the existing Debian version type.
- Not a complete `.sources` file or security configuration lesson. Real sources
  also need appropriate authentication/trust policy. Signed-By, Trusted, key
  files, repository signatures, transport, source packages, architecture options,
  flat repositories, expressions, legacy one-line syntax and other fields are
  intentionally unsupported with a diagnostic. Do not teach disabling signature
  checks or removing trust configuration as a real repair.
- No host APT, binary archives, network, filesystem, subprocess, full package
  policy/solver or general Debian administration conformance. This capability
  does not close objective102.4or count a parser as practical coverage.

## References and acceptance boundary

Checked2026-11-02(local), Debian trixie APT3.0.3:
[sources.list(5)](https://manpages.debian.org/trixie/apt/sources.list.5.en.html),
[tagged source-list implementation](https://github.com/Debian/apt/blob/3.0.3/apt-pkg/sourcelist.cc),
[tagged cache construction](https://github.com/Debian/apt/blob/3.0.3/apt-pkg/pkgcachegen.cc).
These are source/document comparisons, not observed host APT behavior. No apt-get,
apt-file or dpkg executable is available in this Fedora environment; no dependency
installation was attempted. The original package/Contents lessons and their
published limitations remain unchanged. Actual native editor, saved draft/session,
current-outcome grader, teaching, independent task and focused content tests must
exist before a new scenario is marked implemented.
