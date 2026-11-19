# Private indexed-discovery development subset

2026-11-05. Pure capability only: no State/Goal, native workspace, progress or
lesson integration. Catalog/latest verified Fedora candidate stay288/91. That
candidate predates this module. Evaluator19 and existing labs/progress are unchanged.
There is no dependency, host utility fallback or learner-host execution route.

## Model and authority

`src/file_index.rs` takes a caller-held Context and original Tree, and requires
carried context and current caller/group/home/umask metadata to match. A future
adapter must obtain these references from its admitted scenario, never from an
imported current state. The core deliberately does not certify whole-assignment
preservation, draft state, practice identity or completion.

Context selects one supplied root, configuration and opaque database identity.
Config/database are distinct and outside the scan subtree; database identity may
not collide with an ordinary Tree entry. The database stores typed saved names,
not fake plocate binary bytes or executable content. Its private fields are
validated before use. Decoding a structurally valid snapshot is not evidence of
freshness or mastery. A future wrapper must explain the opaque database boundary
and must not claim cat/cp interoperability with real database bytes.

`rebuild` returns a candidate Index only after a complete bounded scan. It does
not mutate files. `execute_bound` publishes that candidate only on success;
errors retain the exact previous snapshot. `matches_current` recomputes current
names/config, ignoring command history and generation counters. `locate` reads
saved names, so deleted names may remain and new names may be absent. `-e` also
checks current existence without refreshing the index.

## Supported literal forms

```text
updatedb -l 0 -o SUPPLIED_DB -U SUPPLIED_ROOT --config-file SUPPLIED_CONFIG
locate -d SUPPLIED_DB LITERAL
locate -e -d SUPPLIED_DB LITERAL
```

The parser deliberately supports reordered explicit option pairs, one literal
query, and the fixed source-selected identities. No defaults, multiple DBs/queries,
LOCATE_PATH, glob/regex/casefold, escape/expansion, pipelines or shell composition.
Unknown syntax fails explicitly. The existing literal tokenizer never invokes the
shell interpreter. Found/NoMatch/Rebuilt are typed model outcomes, not promises of
exact real stdout/stderr or process exit codes.

Config permits one double-quoted PRUNEPATHS list, blank/comment lines and whitespace.
No duplicate assignments, shell evaluation, escapes, filesystem/mount pruning or
paths containing spaces. Prune paths are canonical strict descendants of the root;
ordinary-file exclusions are refused. Order, duplicates and redundant descendants
normalize to equivalent effective exclusions. Saved wire data must use that same
canonical form. Pruning a directory omits its descendants, retaining its own name.

Only bounded canonical ASCII paths and ordinary users. File read permission is
not required merely to index a name. Scanning needs directory read/search access;
incomplete scans refuse publication. This refusal boundary does not emulate every
real updatedb partial-error behavior. Inaccessible-parent existing-only queries
are explicitly unsupported. Command-location metadata belongs to another activity.

## Resource and admission limits

Context8KiB, index128KiB, config4KiB and command4KiB. Paths<=1024bytes/depth32;
<=64nodes/files and16users with existing Tree validity/text limits, plus this
module’s256KiB aggregate text cap. Index<=64names/64KiB, pruning<=16directories,
query<=128bytes, output<=32KiB and work<=16384units. Combined maxima can exceed the
work budget and are refused honestly. Raw decoders bound bytes before parsing;
finite string/sequence readers bound fields. This does not claim a new allocation
bound for upstream Tree deserialization. No file bodies are copied to discard them.

Root verified all19retained proposal files/14live read pins before admission
(manifest98be441a). R1 retained command-location16 and new index19 pass; one new
root-fixture assertion failed because Tree.valid already excludes root. The fixture
and early explicit ordinary-user diagnostic were corrected. R2 all20index groups
pass: staleness, pruning, equivalent configs, atomic failures, source mismatches,
opaque imports, permissions and limits. All-target ClippyR3 passes57.34s and CURRENT-only guard/PythonR4 passes770
tests12.181s.
No utility comparison, native flow or authored coverage is claimed yet.

## Primary references and open fidelity gates

Installed plocate, updatedb and updatedb.conf manuals were read via filesystem on
2026-11-05. Official [plocate manual](https://plocate.sesse.net/plocate.1.html) and
[updatedb manual](https://plocate.sesse.net/updatedb.8.html) were also checked. The
config HTML fetch timed out; its installed original is retained privately.
Upstream landing page lists1.1.25 dated2026-09-06; installed executable version has
not been observed. No source/manual copyright text is imported into authored code.

Before teaching this subset, compare fixed disposable fixtures for scan-root and
pruned-directory inclusion, stored/current leaf distinctions and output ordering.
Root inclusion/order currently express model contracts, not verified utility
claims. Real privileged/global visibility, caches, compressed DB format, links,
mounts, timers and complete locate behavior remain unsupported. The installed
plocate is setgid: any controlled comparison must review that identity explicitly,
use only a contained private visibility-disabled database and never infer global
visibility behavior. No security setting or installed binary is changed.

Next integration must pair separate practice/assignment sources, keep drafts versus
saved config versus applied index distinct, grade current semantic freshness and
whole protected file state, and validate restore/reset. None is credited merely
because these pure-core tests pass. Windows/macOS, accessibility and independent
novice acceptance remain open.

## First actual comparison — failed, retained (post-gate note)

Actual22restricted host launches and17queries completed under CONTROLaf153150.
All22proof markers verify child-only NNP; both tools report1.1.25. Independent
readback verifies all source/tool/ELF/argv/env/status/snapshots and raw DB data.
Original comparison13/17passes, four all-name rows fail. This is not acceptance.
The observed query results omit the scan root and group sorted immediate names
before recursively visiting each child directory, unlike the original model’s
root-inclusive flat ordering. FixedR1 evidence and old producer remain retained.

Root checked official release source privately (archive SHA68c1d5fbb11864403ae39c1a5937f13afd80b03e68041368831ef58cb613578e) from
https://plocate.sesse.net/download/plocate-1.1.25.tar.gz to confirm traversal facts;
no GPL implementation text is copied into the original Rust model or curriculum.
The bounded saved-name graph now follows that ordering and rejects root/disconnected/
flat-order imported records. No learner has this provisional index state: there is
no State/Goal/progress/lab integration yet, so existing progress remains untouched.
R11corrected22tests and a fresh deeper/prefix-siblingR2 reference remain pending.
Do not treat the earlier proposed ordering text above as an accepted utility claim.

## Corrected narrow reference acceptance

Fresh R2 adds nested and near-prefix sibling leaves, retaining all original query/
mutation/guard checks. Corrected22core tests pass. Python793/12.295s and all-target
Clippy29.15s pass. R16actual22host launches/17queries pass their restrictions and
raw evidence checks; R17model matches all17 outputs exactly, including the deeper
ordering. Independent readback confirms all655compiled input commitments,
663retained source/tool pins, actual probe107fa1dc and raw restriction proof.
Receipt SHA2568a58869d8bc0dad8f1c27c6a3b36bb939d43de25bf54c265470aafb1a371fe89;
comparison SHA256d8d9addaee2771299c4ec202e7ab1829e8ab01023232b6dc687b458eb6e3f757.
Original R1failure remains unaccepted and immutable. No output sorting weakened
that comparison; the model itself now emits the observed directory-record order.

This proves only the fixed ordinary-user private-l0 subset on installed1.1.25.
It does not prove privileged/global visibility, full permissions, binary DB
interoperability, application/persistence/native teaching or curriculum completion.
Application wrapper and one substantive indexed-handover lesson are the next work;
no new authored lab or objective credit has yet been registered.


## Application integration after the narrow reference gate

The dedicated FileIndex source-bound editor/terminal lesson now exists in source,
with separate Session drafts and catalog-held Definition authority. It does not
change the core accepted against R2 or reinterpret the earlier retained failure.
See FILE_INDEX_LESSON.md and CHECKPOINT.md for actual app/native gates and pending
candidate acceptance. Earlier “no integration” statements above are historical
gate notes. Existing288objects and saved progress remain compatible; current
source289/92 adds one distinct handover, not generated numerical variants.
Official LPI page rechecked2026-11-05: current5.0/101-500,102-500;104.7 retains
find/locate/updatedb/whereis/which/type and the default configuration path.
Only the implemented narrow slices receive evidence; whereis stays Open.
