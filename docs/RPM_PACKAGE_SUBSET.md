# Restricted RPM package foundation

2026-10-26. The pure foundation is independently source-reviewed and passes all23
isolated tests plus scoped library/test Clippy. The first run's token-fixture
mistake is retained: six files/archive produced8173 tokens,19 below8192. The
corrected near-cap valid control uses six; the token-only negative uses seven
(9326). Limits and strict assertions were preserved. One keys-only iteration lint
was corrected without a waiver. MAIN integration gates are recorded separately.
The first bare-label rpmsort collection tested the wrong input domain. A corrected
fixed-NVR collection agrees on28 strict VERSION orders;12 tie controls remain
inconclusive. See [RPM_VERSION_REFERENCE.md](RPM_VERSION_REFERENCE.md). Complete
EVR expectations remain documentation-derived; no equality/full-EVR proof follows. No package operation or DNF reference has run.
No engine State/Goal, application persistence, CLI, renderer, lesson, registry,
dependency or objective status change. All140 existing activities remain intact;
this library supplies no course credit or real RPM transaction conformance.

The independent accepted contract is private RPM contractR2 c1695d87. Primary
sources checked2026-10-26: [LPI v5.0 102.5](https://www.lpi.org/our-certifications/exam-101-102-objectives/),
[RPM6.0.2 EVR](https://rpm.org/docs/6.0.x/man/rpm-version.7),
[RPM6.0.2 queries/operations](https://rpm.org/docs/6.0.x/man/rpm.8),
[modern RPM spec](https://rpm.org/docs/6.0.x/manual/spec.html),
[architecture dependencies](https://rpm.org/docs/6.0.x/manual/arch_dependencies.html),
[DNF4](https://dnf.readthedocs.io/en/stable/command_ref.html) and
[DNF5](https://dnf5.readthedocs.io/en/latest/commands/repoquery.8.html).
No upstream implementation code is imported/copied. Expected vectors are authored
from behavior documentation, not observed utility results. The fixed rpmsort
collection is development-only and does not supply package-operation evidence.

## Boundary and separate authorities

Catalog is immutable supplied context: fixture origin, fixedx86_64 target, complete
script-free archive identities/content, named metadata snapshots/published identity
and allowed virtual paths. Runtime separately stores installed archive records,
local archive availability, cached metadata selection and current regular text
files. Context validation compares the ENTIRE Catalog; current data may legally
be broken (missing/changed files, unmet requirements). No cached observation or
serialized verification result becomes authority. Ordinary recorded ownership is
not the same observation as exact real rpm -qf path lookup; no command aliases.

Archive identity is name+architecture. Catalog can offer x86_64 and noarch, up to
three distinct EVRs per identity; semantically equivalent-EVR artifacts for one
identity are rejected as ambiguous. Runtime admits at most ONE artifact per NAME,
not parallel versions/multilib. Cross-arch coinstallation/replacement is explicitly
UnsupportedSubset, never an automatic architecture switch. ISA/ELF/SONAME and
cross-arch dependency behavior are not modeled.

Complete EVR is <=128ASCII bytes, optional u32 epoch, nonempty version-release
components containing alphanumerics and . _ + ~ ^ (at least one alphanumeric in
each component). Missing epoch is0. Comparison is epoch, then version, then
release; numeric runs ignore leading zeros, compare by length/digits without
large integer parse, and outrank alpha runs. Ordinary separators compare equally;
extra ordinary segments matter; tilde precedes base and caret lies after base but
before its next segment. Exact text equality is separate. Omitted release,
nonASCII and other punctuation are outside this conservative subset. rpmsort
sorting would not establish comparator equality; equality vectors remain authored
expectations pending a separately reviewed safe oracle.

Each package implicitly provides its own name at its complete EVR. Up to8 explicit
literal provides may be unversioned or exact-EVR. Up to8 ANDed requirements use
name/capability plus an optional complete-EVR relation. An unversioned provide
does not satisfy a versioned requirement. Different provider package names can
legitimately satisfy the same requirement. No boolean dependencies, scriptlet
qualifiers, auto dependency generation, conflicts/obsoletes, weak dependencies,
file-capability requirements or general solver. Modern versioned spec sections,
not historical tutorial claims about unversioned-only virtual packages, govern.

## Observations and logical operations

Installed/package-file metadata, installed versus cached-available providers,
dependencies, recorded owners and file discrepancies derive from actual supplied
metadata/current files. File issues compare contents, mode, owner, group and mtime
for regular UTF-8 virtual text. Missing/modified payloads do not erase installed
records. These are not cryptographic signatures, authentication, full rpm -V
columns or proof that a program runs. There is no host user database lookup.

InstallSet installs only explicitly local archives, refuses an existing package
name, and never fetches dependencies. UpgradeSet may install an absent package;
when replacing, requires same name/arch and strictly newer EVR. ReinstallExact
requires the exact installed local archive. EraseSet removes exact selected
identities and their ordinary/unmodified-config virtual files. Sets contain1..8
identities, final ownership and dependency constraints apply independent of set
order; no automatic provider selection. Downgrades, force flags, ownership
transfers/adopting unmanaged files and parallel installs are UnsupportedSubset.

WHOLE-DATABASE HEALTH IS A LOGICAL PLANNER RESTRICTION: after a proposed change,
every installed requirement must be satisfied, even if a different imported
record was already broken. That extra precondition returns UnsupportedSubset,
not an invented universal RPM error/status. Broken records remain inspectable
and an explicit repair set may restore health. No actual rpm transaction scan
scope, ordering/scripts/rollback or DNF/YUM/Zypper resolver behavior is claimed.
Failed plans preserve exact caller/State/files. The planner uses bounded borrowed
references for final records/files and charges combined S/N BEFORE full content
clones; only a fully admitted final state is published.

Configuration flags record ordinary/config/config_noreplace identities, but no
sidecar decision table is enabled. Any missing/modified installed configuration
(content OR modeled metadata) or changed configuration path/flag manifest makes
a replacing/reinstall/erase operation UnsupportedSubset. Unmodified defaults can
advance; unchanged config is removed on exact erase. No Debian ConfigFiles/purge
record is invented. Changed default versus locally customized content, rpmnew /
rpmsave / rpmorig, user prompts and sidecar collision policies require a later
independent table and gate before any such practice/CLI is exposed.

## Bounds and untrusted admission

Shape ceilings:24 name/arch catalog keys,72 archives,3 per key,3 snapshots,
8 requirements/provides per archive,16 files/archive,128 allowed/current paths,
24 installed names,8 explicit requests. Names/IDs/owners/groups <=32B, paths <=160B,
virtual file text <=4096B, notices bounded. JSON objects reject unknown/duplicate
keys; sequences/maps reject overflow without trusting size_hint, with the extra
body refused before decoding. String visitors check UTF8 bytes before owning a
copy. Direct constructors repeat shape/scalar checks before sets, counting or
plan clones. Generic tagged serde may still buffer within explicit raw admission;
this is not an arbitrary-stream raw-before-parse promise.

Combined ALL serialized strings (including static keys/tags/repeated Catalog+
Runtime occurrences) S<=131072bytes and ALL values/object-key tokens N<=8192.
Fixed schema depth<=16, integers <=20decimal bytes, no floats. S/N are charged by
a schema-local borrowed walk before allocation-heavy cloning; an allocation-free
counting writer independently enforces compact<=1048576bytes. Conservative compact
bound6S+26N<=999424, standard2-space pretty adds <70N, total<=1572864<2097152 raw
State cap. Action raw cap16384. Maxima are COUPLED, not a promise all per-field
maxima fit together. No global2MiB pack/16MB progress limit is increased. Proposed
max legal escaped/combined tests independently count actual serialized keys/values;
no size/performance result is claimed before the first guarded gate.

Corrected pure-model tests passed23/23 in the isolated checkout and MAIN.
MAIN's focused library/NAT/RPM gate passed140 checks and all-target Clippy passed.
The latest complete1620-check regression predates this RPM module; a new complete
regression has not been claimed. No command adapter, app engine State or actual
package operation is added; the tiny lib module export is for the pure test API. LPI102.5 also requires YUM/Zypper and signatures;
this foundation is not a practical/objective completion claim. Native teaching,
source/command annotations, saved draft admission and genuine-history app tests
require later explicitly reviewed contracts.
