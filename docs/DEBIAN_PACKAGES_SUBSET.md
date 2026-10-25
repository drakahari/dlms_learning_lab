# Logical Debian package foundation

Pure-module gate,2026-10-25. Nineteen integration checks and68 library checks pass
after the strict action-decoding correction below. No comparison with a real
package utility is claimed: optional development `dpkg` is not installed here. Expected version vectors
and package data are explicitly authored from primary documentation, not captured
`dpkg` observations. No command adapter, application State/Goal, persistence,
workspace, lesson, objective status or curriculum count is introduced.

## Boundary and separate sources of truth

The model handles a closed set of fictional, script-free packages with ordinary
regular text payloads. Actions are **logical restricted-package operations**.
They do not implement real `dpkg` transactions, intermediate unpacked/configured
states or rollback. In particular, there is no `dpkg -i`, `apt`, `rpm`, `yum` or
`dnf` command alias. The atomic publication guarantee below belongs to this pure
model, not to a claim that real package administration rolls back all effects.

An immutable `Catalog` contains the architecture, repository reachability fact,
currently published snapshot, supplied artifacts, up to three named metadata
snapshots, and explicitly allowed unmanaged paths. A future task must pin the
entire catalog by `validate_context`; importing another valid catalog must not
change the problem. The separate mutable `Runtime` stores a cached snapshot ID,
locally available artifact IDs, package records, and current virtual file bytes.
The published snapshot is not the cached snapshot. Cached metadata is not a
locally available artifact, and an installed database record is not evidence that
its payload still exists or has its original contents.

`RefreshIndex` changes only the cached snapshot when the supplied repository is
reachable. Artifact acquisition is a declarative availability change: a locally
available artifact can be reused offline; otherwise the exact artifact must be
exposed by cached metadata and the supplied repository must be reachable. There
are no DNS, HTTP, signature, download, archive extraction, privilege or host-file
operations. The public catalog is authoring context; the learner query API keeps
cached candidate metadata separate from local-artifact metadata and payload.

Only `Installed` and `ConfigFiles` records exist. Dependency satisfaction requires
`Installed`; a retained configuration record does not satisfy Depends. Missing or
modified payloads and unmet Depends are valid operational states for diagnosis
and repair. Structural validation does not silently repair them or make a task
pass. Wrong architecture, foreign identities, paths outside the supplied universe
and ambiguous multiple ownership are import errors in this subset.

## Version and dependency policy

Versions preserve their exact text. `Version::compare` implements Debian's epoch,
upstream-version and Debian-revision ordering for the declared ASCII subset:
numeric epoch (missing is zero), upstream beginning with a digit, final hyphen
separating revision (missing is zero), tilde before everything including end of
text, letters before other punctuation, and numeric runs compared numerically
without integer conversion. Leading zeros in a run do not change its numeric
value. This is not SemVer or RPM EVR. Exact text equality remains distinct from
semantic version comparison; equivalent artifact identities for the same package
and architecture are rejected as ambiguous.

The declared bounds are 1–128 ASCII bytes per complete version and an epoch that
fits a nonnegative 32-bit integer. The upstream characters are ASCII letters,
digits, `.`, `+`, `-`, `~`; revision omits `-`. More general Debian version syntax,
including an embedded colon after an epoch, is unsupported. Upper/lowercase and
punctuation have the Debian ordering within the admitted subset.

A package has up to eight ANDed Depends entries with optional `<<`, `<=`, `=`,
`>=`, `>>` equivalents represented as typed `Lt`, `Le`, `Eq`, `Ge`, `Gt`.
Install takes one to eight explicit package requests, with optional semantic exact
version. There is one candidate per name in the cached snapshot; this model does
not claim a general APT candidate/resolver algorithm. It retains a satisfying
installed dependency, otherwise selects that dependency's cached candidate. It
follows dependencies transitively, then requires every installed package's Depends
to be satisfied. Explicit safe dependency-first installation can reach the same
result as automatic closure. Mutually dependent, script-free packages may be
admitted when their final installed records satisfy all constraints; no real
configuration-order or maintainer-script behavior is implied.

No alternative dependency syntax, Pre-Depends, virtual providers, Conflicts,
Breaks/Replaces, architecture qualifiers, Multi-Arch, recommendations, holds,
pinning, downgrade, automatic removal of peers, backtracking or force option is
modeled. Unsupported/failing plans leave the exact original state unchanged.
An already installed identical artifact is an Install no-op; it does not repair
corrupt payload bytes. `Reinstall` uses the exact installed artifact and separately
checks its availability.

## Files, conffiles and retirement

Recorded ownership is queried from package manifests, independently of whether a
file is present. Ordinary payload verification compares complete current text to
the supplied artifact text; it is not a cryptographic provenance/signature check.
Reinstall restores ordinary payloads and does not overwrite a modified conffile.
No directories, links, devices, modes, owners, generated files, triggers or scripts
are modeled.

For an upgrade/reinstall with an unchanged conffile path set:

- An unchanged packaged default can advance to the new default.
- A locally modified file is preserved when the packaged default did not change.
- A locally removed conffile stays absent.
- If both the local content and packaged default changed, the action reports an
  unsupported choice and publishes nothing. No automatic prompt answer, sidecar
  file or guessed replacement is invented.

Changing the conffile path set and adopting existing unowned payload text are also
unsupported. Multiple recorded owners and proposed file collisions are refused.
A normal payload path recreated after removal is unowned; purging the retained
configuration record must not delete that recreated file.

`Remove` is exact low-level removal: a remaining installed package declaring a
Depends on that package blocks removal, even if the version constraint was already
broken. This does not emulate APT's dependent-removal proposals. Otherwise ordinary
payloads are removed; conffiles and a `ConfigFiles` record are retained. A package
with no conffiles loses its record. `Purge` removes the selected package's remaining
managed conffiles (and ordinary payload if still Installed), never arbitrary
neighbors, peer packages, cached metadata or archive availability. A `ConfigFiles`
record with no conffile manifest is outside this model's state shape.

## Bounded mutation and untrusted input

`State::new` and catalog validation preflight collection and scalar bounds before
derived sets or state cloning. The typed Action API repeats bounds for directly
constructed values. JSON objects reject duplicate keys and unknown fields. Bounded
sequence visitors reject an extra typed body before constructing it; bounded text
visitors check bytes before creating the owned copy. JSON input limits are applied
by `State::decode` (4 MiB) and `Action::decode` (16 KiB).

| Item | Limit |
| --- | --- |
| Package names / supplied artifact IDs | 24 names / 72 artifacts, at most 3 artifacts per name |
| Architecture | one supplied amd64 or arm64 target; matching artifacts or all |
| Metadata snapshots / candidates | 3 snapshots / 24 candidates each |
| Payload rows | 32 per artifact; 256 in the catalog total |
| Virtual files / unmanaged allowed paths | 288 files / 32 additional allowed paths |
| Dependencies / requested packages | 8 per artifact / 8 explicit requests |
| Identity / path / file content | 64 / 160 / 2048 UTF-8 bytes; no NUL in file text |
| Catalog payload / runtime file text | independently 256 KiB |
| Compact catalog / complete State | 2 MiB / 4 MiB |
| Logical action work | 10,000 charged worklist/dependency/payload steps |
| Error message | fresh allocation containing at most 512 UTF-8 bytes |

Structural loops are bounded even outside the charged worklist (e.g. pairwise
identity checks among at most 72 artifacts). Internal planning keeps borrowed
IDs/text, checks prospective aggregate file and serialized-state sizes with a
counting writer, and only then materializes a bounded candidate. `apply` publishes
it once after validation. Any error retains exact original catalog/runtime bytes.
`plan` is a read-only explanation, not a token authorizing later state; `apply`
recomputes from the current State and Action. Queries do not mutate state.

These are logical data/work bounds, not a claim about allocator overhead, real
package size/performance or complete hostile-JSON streaming memory behavior. The
serde_json decoder can have its own bounded raw-input scratch. There is no broad
unbounded `serde_json::Value` in production. Worst-escaped compact and pretty State
roundtrips pass the guarded checks; outer app profile/pack acceptance is not claimed.

## Verified foundation checks and remaining gate

The declared fixture is synthetic and includes an installed damaged executable,
customized config, a shared dependency, a peer, a retired leaf, a foreign
architecture and old/current snapshots. Nineteen passing tests cover independent
version vectors, ordering properties, query distinctions, corruption/reinstall,
transitive and alternate safe installation, offline/stale caches, broken Depends,
exact removal, retained configs/purge safety, conffile choices, collision/architecture/
version errors, immutable context, duplicate/unknown JSON, direct/streaming limits,
and maximum admitted escaped serialization. No expected value was obtained by
running a host package manager.

The first guarded gate passed68 library and18/19 integration checks. Its failed
assertion exposed serde ignoring extra fields on an internally tagged unit
variant despite the enum attribute. `RefreshIndex {}` is now an empty struct
variant, retaining the same JSON wire shape. The original rejection assertion
remains, with positive wire, duplicate-tag and all-five-variant unknown-key
regressions added. All19 plus68 library checks then pass. All-target ClippyR2
and formatting pass after a style-only nested-if correction.

These are primary-document-derived checks, not a host `dpkg` oracle. No package
manager was installed or executed. A command adapter, current-state goals, native
workspace, progress integration, taught assignments and controlled real reference
comparisons where available still need their own reviewed gates. No practical
coverage or authored-lab count follows from this pure foundation alone.

## Primary references

Read-only source research checked 2026-10-25; exact source receipt is retained
privately in `reviews/package-management/design-r1/SOURCES.json`. No host package
utility/reference capture was run.

- [Debian Policy 5.6.12, Version](https://www.debian.org/doc/debian-policy/ch-controlfields.html#version), policy 4.7.4.1 observed.
- [Debian Policy 7, package relationships](https://www.debian.org/doc/debian-policy/ch-relationships.html).
- [Debian Policy 10.7, configuration files](https://www.debian.org/doc/debian-policy/ch-files.html#configuration-files).
- [dpkg(1), Debian trixie 1.22.22](https://manpages.debian.org/trixie/dpkg/dpkg.1.en.html), package states/install/remove/purge/verify distinctions.
- [dpkg-query(1)](https://manpages.debian.org/trixie/dpkg/dpkg-query.1.en.html) and [dpkg-deb(1)](https://manpages.debian.org/trixie/dpkg/dpkg-deb.1.en.html), installed records versus archive queries.
- [apt-get(8), Debian trixie APT 3.0.3](https://manpages.debian.org/trixie/apt/apt-get.8.en.html) and [apt-cache(8)](https://manpages.debian.org/trixie/apt/apt-cache.8.en.html), index refresh, cached queries and resolver scope.
- [LPI LPIC-1 version 5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/), 102.4/102.5. This foundation does not itself establish objective practice or RPM capability.
