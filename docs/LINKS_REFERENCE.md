# Fixed inode/link development comparison design

Checked2026-10-29. The bounded pure link model passes16 integration groups
(M444). M453 passes all424 Python checks, including28 fixed-reference guards.
M455 matches all eight fixed Linux sequences and six actual simulator-only
exclusions; M456 verifies all retained outputs without replay. The first failed
attempt remains intact. These are development-fixture results, not full POSIX,
GNU CLI, native lesson, exam or platform acceptance. The learner application
never imports these reference tools.

The prerequisite is the separately reviewed LINKS_SUBSET foundation, core SHA
`eb872623fa04ecd256d5fe5d7c9b1e9db14b319919e3fc34ac6ed5760068a56f`
(the original core with only an unused import removed; no semantic change).
The probe imports that original module directly. It computes observations through
its real typed APIs; it does not recognize command strings or return expected
fixture values. Its embedded identity table pins core/probe/digest/fixture/Cargo
manifest/lock bytes. It accepts no source, operation, fixture, path or case CLI.

## What the eight comparisons mean

| Fixed sequence | Independently observed fields |
| --- | --- |
| Shared alias and separate copy | Alias identity relation, regular link counts, shared write and independent copy contents |
| Unlink first then final name | Remaining identity/data/link count; removed names fail |
| Move relative symbolic link | Symbolic identity and raw target persist; a new containing directory changes the resolved data |
| Dangling target arrives and disappears | lstat/readlink remain usable while followed reads reflect current target existence |
| Internal directory symbolic prefix | Prefix traversal reaches the real file; final no-follow/follow and directory-required slash differ |
| Copy through symbolic source | Copy is a new regular identity with captured bytes; later source changes appear through the link only |
| Three fixed file errors | Existing hard-link destination, regular-file prefix/slash and missing name; original data/identities stay intact |
| Literal names and UTF-8 data | Spaces/leading dash remain literal names; byte sizes, shared identity and raw link text |

Only identity **relations** are compared, never numeric inode/device values or
allocation/reuse order. Regular link counts and UTF-8 byte sizes are meaningful;
directory link counts, storage sizes, physical allocation/CoW, ownership, modes,
permissions, timestamps and open descriptors after last unlink are not compared.
The copy reference explicitly reads a known regular target into a separate new
file; it is not a claim of complete GNU `cp` option/metadata behavior.

The VM deliberately refuses replacing an existing move destination. Linux
rename normally replaces it. Therefore only a positively unused destination is
compared; the adapter's absence check is **not** a race-safe general rename-no-
replace implementation. No such concurrency/CLI claim follows from this fixture.

Six additional VM observations cover an absolute missing target, root-parent
escape input, unsupported hard links to directories/symbolic links, a cycle and
an oversized target. They never enter the host. Existing16 core groups retain
physical symlink-before-parent, path limits and codec/atomicity coverage; this
small comparison does not replace them or imply those cases ran on Linux.

## Collector/worker boundary

The adapter is never a standalone tool; its direct main still exits immediately.
**Its directory-FD argument is not an approval boundary.** Only the fixed worker
may load it, after all checks below. The runner adds only
`links_reference.py`, preserving all prior tools; worker/adapter cannot be selected
through that allowlist. Independent source review and Root's sole lease are still
required before any tests, build, prepare or capture. Source guards are not actual
filesystem acceptance.

1. Run only from MAIN through its shared-lock runner with two Cargo jobs/two Rust
   threads. Use MAIN's fixed default `target/debug/examples/links_reference`.
   No caller-selected checkout, binary, output directory, source, operation,
   arbitrary path or fixture selectors. The public modes are prepare, capture and verify. Build and reference invocations remain serial.
2. Prepare launches no process and performs no link/unlink/rename or fixture
   operations. Hash all original foundation/probe/fixture/adapter/test/doc,
   Cargo/lock/config, runner/helper dependencies, copied compiler/toolchain and
   ordinary ELF bytes. Pin Python executable, Linux release/platform and default
   target provenance. Verify the six compiled identity rows, core version1,
   exact8+6 partitions and every prediction-null field after the later probe run.
   No complete application-source reproducibility claim is made from six inputs.
3. A bounded exact control file starts `root_reviewed: false`. Root reviews the
   entire static source/action table and source/compiler/ELF/platform pins; only
   the otherwise structurally identical approved control may set that literal true.
   Capture rejects missing/extra/changed inputs or helpers before any child or
   host fixture operation. Keep old controls/failures immutable. No automatic
   approval, environment credentials or permission bypass.
4. Use a fresh dedicated disk-backed owned capture root under
   `.build-artifacts/reviews/links-reference/captures/links8-r2/`; refuse any
   existing destination, ancestor symlink/alias, unexpected owner or RAM-backed
   filesystem. The fixed probe runs in an empty private disk cwd with empty stdin,
   stripped closed environment, retained executable FD and a three-second timeout.
   Verify its ELF/source identity before and after execution/retention.
5. Actual host cases use the separately pinned fixed Python worker, isolated
   from user startup/environment (`-I -S -B` and a closed environment), running as
   the ordinary owner. Never use a shell, command lookup, Bash, dynamic evaluation of strings,
   network, system services or learner scripts. Loading the one pinned trusted
   adapter from its sealed FD uses the standard Python module loader only. Its only operation is the exact
   eight-case plan. No JSON action interpreter: index/FD messages are a closed
   private parent protocol, with no optional subset or caller-supplied content.
6. The parent creates each brand-new700 case directory itself under that root,
   opens it O_DIRECTORY|O_NOFOLLOW and passes its held FD only to the approved
   worker. Check ordinary ancestors, ownership, unique inode and empty contents;
   verify identity/complete allowed-entry inventory and exact internal symbolic
   targets before/after every sequence. Never follow an unexpected entry. The
   adapter's fixed paths reject absolute, dot/dot-dot, unknown and backslash
   spellings. All allowed symbolic targets are short relative internal names;
   no host escape/traversal/cycle/limit tests are admitted. No environment/cwd
   values can redirect them. No personal file/profile or /tmp path is used.
7. Eight workers run serially, five seconds each, with the existing runner's
   memory/PSI protection, fixed process/file/output limits and a60-second overall
   deadline. Bound each worker to128 operations,16 entries,8KiB per regular file,
   64KiB aggregate data and8KiB stdout/stderr. These fixed sources use far less.
   A per-operation clock check cannot interrupt a blocking syscall; the separate
   parent timeout and child process-group termination are mandatory. Preserve
   partially written evidence after timeout; do not retry by widening bounds.
8. Persist raw probe/worker stdout, stderr, exit status and normalized observations
   immediately, including failures. Compare every observation field independently
   against the fixed prediction and the other engine. Do not infer success from
   process exit alone or silently normalize unexpected errno. Preserve all eight
   results and mismatches. A COMPLETE manifest is written only after all8+6
   partitions, input stability, output/file byte caps and retained source/ELF
   hashes pass; independent Root readback/sealing is a separate step.

The worker/adapter are copied from approved ordinary source descriptors into
anonymous Linux memfds capped at64KiB each and sealed against writes, growth,
shrinkage and further seal changes. Python executes the held source FD; its
isolated loader reads the held sealed adapter FD. No unsafe mutable-path fallback
exists. This tiny code snapshot is not a large RAM build/cache artifact; all
fixtures, outputs and retained executables/sources remain on the project disk.
The worker derives Main only from the exact fixed run-cwd hierarchy, checks the
actual collector parent/group, MAIN marker and inherited shared-lock inode,
approved control, interpreter/kernel/source identities and exact fresh fixture FD.
All child protocol fields are bounded indices/FDs, never paths/actions/source.

The collector retains approved source snapshots plus held probe/Python ELF bytes
**before any probe or host child**. Timeouts and mismatches therefore retain actual
code as well as partial streams/statuses and fixtures. All pairs must pass before
COMPLETE can be published. Verify launches no child, changes no fixture, checks
all74 ordinary output hashes and retained sources/executables, recomputes every
comparison and observes the remaining fixed filesystem tree without following a
final symlink. Regular hardlink counts/normalized alias groups detect later
content-preserving alias damage; raw inode numbers are never compared across
VM/Linux or published as portable identity. A structural verify reports
`NOT-OWNER-ACCEPTED`; Root's independent readback remains separate.

After source integration/review and an explicit sole lease, the exact sequence is:

```sh
python3 tools/dev.py build --locked --offline --example links_reference
python3 tools/dev.py clippy --locked --offline --example links_reference -- -D warnings
python3 tools/dev.py python-tests
python3 tools/dev.py tool links_reference.py --mode prepare
# Root reviews exact proposed pins and writes the otherwise identical APPROVED control.
python3 tools/dev.py tool links_reference.py --mode capture
python3 tools/dev.py tool links_reference.py --mode verify
```

The successful build, all-target Clippy, Python and observed comparison gates
are recorded above. Original filesystem oracles, historical producers and their
evidence remain unchanged. Published source documentation was updated after
capture; the original compiled/pinned inputs remain in its retained bundle.

Retain all fixtures/diagnostics. Only the explicitly authored names removed
inside the new disposable cases are ever unlinked. No blanket cleanup, old /tmp
artifact deletion, `cargo clean`, permission repair or system setting change is
part of this design. If Linux cannot provide ordinary hardlinks/symlinks/FD-relative
APIs on this disk, report reference-unavailable and retain primary-doc comparison;
do not change mounts, privileges, security policy or the user's desktop.

The fixture is private to an ordinary development process, not an adversarial
same-user sandbox or an operating-system filesystem emulator. No reference
execution is routed into learner grading. Mock-only guards do not establish
successful Linux operations, worker timeouts or physical isolation.

## Primary sources and remaining evidence

Linux upstream man-pages checked2026-10-29: symlink target substitution and
containing-directory meaning at
https://man7.org/linux/man-pages/man2/symlink.2.html ; rename identity, existing-
destination and symbolic-entry behavior at
https://man7.org/linux/man-pages/man2/rename.2.html ; removal of names and last-
link/open-descriptor distinction at
https://man7.org/linux/man-pages/man2/unlink.2.html . The direct link(2) fetch
https://man7.org/linux/man-pages/man2/link.2.html timed out in this pass; the
previous source review and foundation guide record the existing reference.
Only paraphrases and original implementation/fixtures are used; no manual or GNU
implementation code is copied. This does not assert full POSIX, CLI, permission,
mount, binary-name, exam, native, platform or novice acceptance.


## First actual attempt retained; independent directory scans

Root's M452 attempt under `links8-r1` stopped at case0 before accepting the
reference: the worker returned correct typed alias/copy observations but an
empty final inventory, while the independent parent found the three actual
fixture entries. Its FAILED record, original control, raw results, source and
ELFs remain untouched. It is not a successful eight-case comparison.

The worker and parent obtain a fresh open description by opening literal
`.` relative to their held directory descriptor, with O_DIRECTORY, O_NOFOLLOW
and O_CLOEXEC. Each scan closes only its own descriptor, preserves the caller's
cursor and retains the same16-entry/depth/data/alias checks. No seek, path
fallback, weaker inventory equality or fixture rewrite repairs the failure.
The retry used new fixed `links8-r2` in collector and worker, fresh prepare,
independent pin review and exact Root approval. M455 completed with all eight
comparisons matching and all six actual VM-only outcomes correct.

The exact prior cursor producer was not instrumented. The fix removes dependence
on borrowed directory-stream position without assuming which earlier scan
produced it.

Three new guards check fresh relative-open flags and exact close on success,
error and excess entries; a mocked consumed-cursor test checks both repeated scanners use a fresh
description and retain all three files, contents and normalized hard-alias
groups. These guards create no actual fixture and perform no adapter graph,
probe, worker child or actual eight-case reference. All28 guards pass in M453; original25 source assertions remain unchanged.
The74 retained outputs and source/executable bundle have manifest SHA256
`5141c690ec4d12c887f78e62fd3b0e2603476a619b18bbb26e486fa415ff9a89`.
M456 verifies the complete bundle without executing a probe or reference worker.
Independent raw readback is recorded separately in the implementation ledger.
