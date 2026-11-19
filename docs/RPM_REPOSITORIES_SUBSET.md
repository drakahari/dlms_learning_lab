# Current curriculum composition — 2026-11-02

The pure model now supports four source-integrated native repository-evidence
lessons. See REPOSITORY_LESSONS.md for actual gates and pending acceptance; the
original pure-foundation/source-only statuses below remain historical. No new
Zypper/Yum CLI, transport, signature or installation behavior was added.

---

# Current integration evidence — 2026-11-02

The reviewed pure repository metadata and cache provenance model is now exported by the application
library. All19 focused engine tests pass through MAIN's two-job/two-thread runner
(`.build-artifacts/continuation/resume-20261102/rpm-library-gate.log`). These are
capability tests, **not integrated lessons or practical objective coverage**.
No host filesystem, package-manager or network execution is introduced.
The retained proposal below describes its original review state; its UNRUN
labels are historical. Native lessons, progress admission and teaching remain
separate gates.

---

# Bounded repository and metadata-cache foundation

Source-only proposal, 2026-10-30. All19 prospective integration checks are UNRUN. No export, UI, CLI, Goal, scenario, progress migration or curriculum registration is added. The accepted RPM implementation is a read-only dependency. No objective promotion or actual Zypper conformance claim follows.

## Modeled behavior

Source contains a fully validated existing RPM Catalog plus one or two immutable supplied repository endpoints. Each endpoint declares one opaque URI, a supplied delivery-health fact, one published snapshot and at most two historical snapshot IDs. Catalog target stays x86_64, matching the core's existing target constraint; payload packages are noarch. At most12 archives, eight package names, two snapshots and32 allowed virtual text paths are admitted.

State holds the whole immutable Source, one or two configured alias/URI/enabled records and up to two cache observations. A cache records its actual endpoint identity and snapshot, not a canned successful-query flag. Unknown but well-shaped URI text can be represented as a repairable configuration problem. URI text is never resolved, opened or sent anywhere.

Typed SetEnabled, SetUri and single-alias Refresh operations work only on existing configured aliases. Default Refresh excludes disabled sources even when explicitly named: the typed simulator returns Disabled and retains the full state/cache. Enable before retrying; no include-disabled option is modeled or exposed. Source delivery failure leaves the old state/cache exact. Refresh changes metadata only, never RPM installed records, local archives or virtual package files. No repository addition/removal, parser for .repo files or transactions exist yet.

A SetUri operation retains prior cache provenance for inspection. If it identifies a different endpoint, query rows from that cache are withheld until explicit refresh. This is a conservative source-provenance MODEL POLICY, not a claim about exact libzypp URI-change cache retention or search behavior. The cache, configured URI and current immutable endpoint facts are always separately observable.

Typed exact-name queries return borrowed archive rows with actual alias, endpoint and cached snapshot. They do not select the newest version, solve dependencies or invent installed status. Disabled/unknown-source/no-cache/wrong-source omissions are explicit. Older metadata remains usable and labeled; an unavailable endpoint does not prohibit inspecting an existing valid cache. Querying never silently refreshes. Later native command work must faithfully implement its documented no-refresh/detailed/installed-result behavior before claiming a Zypper command is supported.

## Admission, resources and persistence

Every public view/action validates the complete State and expected Source. Context equality is exact across every archive, file, dependency, endpoint, snapshot and delivery fact; unchanged IDs alone never suffice. Both sides are independently validated. Decode requires a supplied expected Source. serde unknown/duplicate fields, duplicate aliases/cache identities and foreign endpoint/snapshot IDs are refused. Bounded visitors check strings/list counts before accepting nested items. Raw JSON is limited to128KiB before decoding; compact Source to32KiB and State to48KiB. Malformed direct serde values cannot bypass validation by calling a view or action.

A capped writer measures actual JSON without building a second serialized tree. Constructor validates the borrowed Source and small runtime before its one source clone. Actions clone only two tiny configuration/cache lists, size the borrowed full candidate projection, and publish together after all checks; no full archive/payload clone per action. No I/O, subprocess, unsafe, dependency or global limit change.

Resource closure: Source serialization is <=32768B by admission. Two configurations contain aliases <=32B and URI text <=160B each; URI validation excludes controls/backslash/quote so it cannot expand through JSON escaping. Two caches contain three <=32B safe identifier strings each. Runtime/State wrappers and all four records fit within1536B beyond Source, conservatively; therefore every valid combination remains below34304B, safely inside48KiB. An oversized Source is refused before cloning. Tiny action projections remain checked anyway. The underlying RPM Catalog already bounds every nested payload/requirement/provide; additional app limits narrow that universe. Query rows are <=2 repositories ×12 archive references; omissions <=2. All loops have structural bounds, with no interpreter/time/host activity.

State itself has no completion, transcript, prior grade, clock or learner answer. Roundtrip preserves actual cache/configuration; resetting future activity must restore its supplied initial State rather than invent a refresh. A future source-bound workspace must budget whole Attempt/profile and isolate examples before any registration.

## Verification plan

The standalone integration target imports the current read-only RPM library and includes the proposed new module directly; no library export is needed for the first gate. Nineteen groups cover missing/stale/current cache, atomic default disabled-refresh refusal and enabled retry, alternate enable/URI-repair orders before refresh, wrong/unknown source and repair, unavailable endpoint with usable old cache, two repositories/provenance, breaking success, every Source field, alias/URI mutation refusal, duplicate/foreign metadata, raw/nested JSON limits, escaped-source cap, roundtrip, malformed source, direct poisoned serde State and a full escaped valid source/runtime growth control. These are source-written assertions, not runtime observations. Root must compile/run them under the MAIN memory-safe runner after independent source review, then add negative/semantic controls for any discovered mismatch before CLI/content.

## Primary sources and fidelity limits

Checked2026-10-30:

- [LPI LPIC-1 v5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/),102.5: T6 Zypper is Open in the current coverage map; no credit is added here.
- [Official openSUSE Leap16 Zypper manual](https://manpages.opensuse.org/Leap-16.0/zypper/zypper.8.en.html): global --no-refresh and configuration/cache concepts. Its explicit-operand wording alone does not establish a disabled override.
- [Official upstream refresh implementation](https://raw.githubusercontent.com/openSUSE/zypper/master/src/commands/repos/refresh.cc), lines254–279 checked2026-10-30: disabled sources are skipped unless an explicit plus-content intent includes them, even when the alias was specified. This is a mutable-master source read, not a version-pinned runtime comparison; no upstream implementation code is copied. The model exposes no include-disabled intent.
- [SUSE SLES15SP7 Administration Guide](https://documentation.suse.com/sles/15-SP7/html/SLES-all/cha-sw-cl.html),9.1.6–9.1.7: repository management and package queries. No prose or code is copied.
- [RPM6.0 manual](https://rpm.org/docs/6.0.x/man/rpm.8): retain query/verification distinction of existing model. All old RPM arithmetic/transaction boundaries remain unchanged.

No network, RPM-MD XML, digest/signature/trust, clocks, actual .repo grammar, service/plugin, libzypp dependency resolution, priority/vendor selection, mirrors/credentials, package downloads/transactions, configuration sidecars, patch/dist-upgrade, snapshot system, real Zypper output/status or YUM/DNF/rpm2cpio behavior is implemented. Opaque URI validation is a bounded model token check, not a real URI parser or host security policy. The program never executes a URI or command. Full objective102.5 remains incomplete.


## R2 correction to the frozen R1 source proposal

Independent primary-source review found that R1 incorrectly inferred that merely naming a disabled alias allows its default refresh. The upstream refresh implementation explicitly filters disabled sources unless include-disabled behavior is requested. R1 remains frozen and its two incorrect positive tests are not accepted behavior; no tests or real utility were run to establish that inference.

R2 adds only a typed Disabled refusal before runtime copies/publication, replaces the invalid disabled-success and refresh-before-enable positive groups with default-refusal/enabled-retry and valid enable/URI-repair independent-order groups, and appends one old-cache byte-retention/refusal/retry group. Sixteen other full test bodies are unchanged. Eighteen prospective groups become nineteen, all UNRUN. This is not a numeric Zypper exit status, and no CLI or include-disabled behavior is added. All other policy/cache/source/resource boundaries are unchanged.
