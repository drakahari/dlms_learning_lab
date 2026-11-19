# Shared-library investigation simulation

This is a bounded, in-process metadata model for part of LPIC-1 v5.0 objective
102.3. It never reads an executable, loads a library or runs a host utility.
The supplied virtual platform is little-endian x86-64 with a declared inventory;
its default directories are not a claim about every Linux distribution.

## Learner operations

| Operation | What changes or is observed |
| --- | --- |
| `ldd PROGRAM` | Inspect the supplied program’s complete dependency graph using a clean environment, virtual cache and supplied default directories. |
| `env LD_LIBRARY_PATH=DIRS ldd PROGRAM` | Inspect with a one-off search override. It does not save application settings or edit configuration. |
| `ldconfig -p` | Display the current virtual cache’s names and paths. |
| `ldconfig` | Explicitly rebuild the virtual cache from the saved directory list. Unsaved editor text is not used. |
| Save configuration | Save the editor’s virtual `/etc/ld.so.conf`. This does not rebuild the cache. |
| Apply launch settings | Save this supplied program’s explicitly simulated search override. This control is not a real Linux configuration-file format. |
| Inspect launch settings | Observe the chosen program with its applied simulated override. An unapplied draft is not inspected. |
| Check configuration | Recompute all assignment outcomes and preservation requirements, including unapplied drafts. |

`DIRS` contains supplied absolute directories separated by colons. A quoted
semicolon separator is supported; unquoted shell composition is refused.
The parser handles literal quoting only. Expansion, redirection, pipelines,
other options, configuration includes and globs are unsupported and diagnosed.
Missing dependencies appear as missing metadata, not as an invented success.
Observations distinguish supported operations, unsupported behavior, invalid
input, unknown supplied items and limits. No GNU exit status is fabricated.

## Resolution and fidelity boundaries

The model walks direct and transitive requested names, retaining already loaded
requested-name/SONAME selections before searching again. For admitted fixtures,
lookup considers the explicit environment, cache and default directories.
Cycles and repeated dependencies are bounded. Catalog objects, aliases, declared
architecture and cache contents are validated before use. Cache validation
recomputes its entries from its source configuration; saved booleans do not make
an invalid snapshot trustworthy.

A SONAME is a name, not proof of binary compatibility. No ELF parsing, symbol or
relocation checks, ABI guarantee, actual loading, program execution, installation,
link repair or real cache bytes are implemented. RPATH/RUNPATH, secure execution,
hwcaps, preload/dlopen, general architecture fallback and ambiguous version/link
selection are excluded. Fixtures requiring these rules refuse rather than
silently approximating them. Full objective 102.3 remains partial, particularly
installing missing libraries. Source review is not an executed loader-conformance
comparison.

## Limits and persistence

Metadata has at most 24 objects, 8 programs, 16 directories, 8 direct needs per object,
depth 16 and 128 visited dependency edges. Serialized model 64 KiB; configuration 4 KiB;
search list 1 KiB; command 2 KiB; output 32 KiB; transcript 12 rows; 64 explicit rebuilds.
Workspace 160 KiB, Session 512 KiB, Definition 256 KiB and complete source/Attempt 1 MiB.
These are deliberate learning-simulator limits, not Linux limits.

Source-bound assignment and example inventories, objective plans and initial
application settings remain distinct. Drafts, applied settings, configuration,
cache, history, view and selected worked action are persisted independently.
Native inspection displays are ephemeral and show unrun after reopening; they
never award credit. Changing a meaningful setting clears the checked gate, and
current success always recomputes actual outcomes. Practice cannot complete an
assignment. Selected example reset preserves assignment work; assignment reset
requires the app’s existing confirmation.

Scoped-search grading preserves the original configuration, cache presence,
directories and entries. Rebuild generation is an operation counter, and a past
comment in a valid cache-source snapshot is not a different library selection.
Ignoring those two nonfunctional differences in outcome comparison never relaxes
cache recomputation or immutable source binding. Cache handover additionally
requires a current rebuilt cache and removal of real supplied temporary overrides.

## Authoring

`shared_libraries.rs` owns pure typed resolution; `shared_libraries_workspace.rs`
owns plans, bounded command parsing, independent work and grading. Its `binding`
module validates the actual authored Scenario/Attempt at pack and persistence
boundaries. `ui/shared_libraries_page.rs` renders native controls and queues typed
edits. It cannot execute a real utility.

A versioned lesson provides a bounded `Definition`, a fresh actual `Session::new`
wire representation, distinct unfinished practice data, essential concepts and
worked actions. `verify_worked` executes the example’s closed model operations;
the pack validator also executes the assignment’s reference actions and grades
resulting state. JSON `shared-libraries` action records are an authoring codec,
not Linux commands to teach learners. Native fills are practice-only and never
auto-save or run. Preserve source IDs and version requirements deliberately when
changing an already shipped objective; do not infer new learner mastery.

## Sources checked 2026-11-05

- [LPI’s released objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/): v5.0, 101-500/102-500; 102.3 includes dependency identification and installation.
- Linux man-pages 6.19: [ld.so](https://man7.org/linux/man-pages/man8/ld.so.8.html), [ldconfig](https://man7.org/linux/man-pages/man8/ldconfig.8.html), [ldd](https://man7.org/linux/man-pages/man1/ldd.1.html).
- Documented glibc 2.42 implementation rules: [dl-load.c](https://raw.githubusercontent.com/bminor/glibc/glibc-2.42/elf/dl-load.c), [ldconfig.c](https://raw.githubusercontent.com/bminor/glibc/glibc-2.42/elf/ldconfig.c).

The attempted 2.43 source-tag and Sourceware URLs were unavailable in the recorded
source review. This uses accessible 2.42 rules and does not identify the host’s
loader version or claim latest-release conformance. No GNU source code is copied.
