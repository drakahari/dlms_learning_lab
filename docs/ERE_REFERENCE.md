# Fixed ERE development comparison

This is a developer tool, never the learner engine. It compares only the34
reviewed synthetic boolean cases in `tests/fixtures/ere-fixed-reference.json`.
Eight excluded patterns run only in the in-process probe; they never reach GNU.
The original fixture retains null observations. Actual results go to ignored,
private, disk-backed evidence. No curriculum or full POSIX/GNU claim follows.

The standalone `examples/ere_reference.rs` directly includes the reviewed
`src/ere.rs` and existing SHA helper. It does not import the application's library,
shell, catalog, evaluator or learner input. Its compiled identity embeds hashes
of the module, own source, fixed fixture, SHA helper, Cargo manifest and lockfile,
plus Cargo's build-directory identity. It accepts no input or selectors; only
`--identity` or all42 fixed observations. Cargo may build the package library as
a build prerequisite, but the probe's predicate is its directly included module,
not a potentially stale same-package library/evaluator.

The collector records the closed source/helper dependency list, compiler-cache
identity and actual probe/GNU executable hashes. Cargo.lock records dependency
resolution. This is not a bit-for-bit reproducible toolchain or audited standard
library claim. Source pins and a prepared ELF alone are not an execution result.

## Reviewed sequence

Use the MAIN runner for every build, Python gate and reference invocation. It
owns the shared lock,2 Cargo jobs,2 test threads, disk temporary directory and
memory observations. Do not run alongside another heavy invocation.

```sh
python3 tools/dev.py build --offline --example ere_reference
python3 tools/dev.py python-tests
python3 tools/dev.py tool ere_reference.py --mode prepare
```

Prepare launches no child. It requires a built fixed probe, reads/hashes bounded
source and binary files, then writes
`.build-artifacts/reviews/ere-reference/captures/ere34-r2-PROPOSED.json`.
An independent reviewer must check exact source and executable pins, including
compiled-source proof, then create the corresponding `ere34-r2-APPROVED.json`
with exactly the same fields and literal JSON `root_reviewed: true`. No code here
creates that approval or changes the flag automatically. Owner authorization for
ordinary implementation is not a reason to bypass this technical review.

After independent approval:

```sh
python3 tools/dev.py tool ere_reference.py --mode capture
```

There are no source, pattern, file, profile, worktree, binary, fixture, output or
capture-ID selectors. Missing/drifted/extra approval fields, changed source/ELFs,
fixture substitution, aliases, a wrong target or absent shared guard refuse the
run. Fresh output only: no existing proposal/capture is overwritten or removed.
A repeated attempt requires a reviewed fixed capture-identity source amendment;
retain earlier failures and observations. No stress test or host learner code is
introduced.

## Execution and evidence

The existing reviewed helpers provide same-FD executable pinning, finite
nonblocking streams, inherited lock FD, timeout cleanup of only owned children,
private no-clobber files, bounded reads and durable completion publication. Their
transitive project helper sources are included in the approval pins. Old helpers
and collectors are unchanged. This tool adds only one fixed runner allowlist name.

Capture executes the identity and fixed in-process probe, verifies embedded source
hashes, then runs `/usr/bin/grep -E -q -- FIXED_PATTERN` for each supported case.
Each child receives one fixed input line plus LF on stdin, an empty disposable
private cwd and a closed C-locale environment without loader/preload/profile
settings. No shell or filename operand is involved. Exclusions never execute on
the host. A real GNU version observation and same-held-FD executable copies are
retained privately; they are not redistributable project artifacts or releases.

Both expected/VM/GNU booleans must agree. GNU status0 means match,1 means no
match; any other status, stderr, unexpected stdout or VM diagnostic is a failure.
The collector retains each actual row, raw stdout/stderr, status and mismatch;
a timeout/limit status that the bounded helper did not return is recorded as
unknown with an incomplete/error diagnostic, never invented.
it does not convert an error into a negative match. Each GNU process has a2-second
limit, the probe15 seconds, and the whole run120 seconds. Stream limits are16KiB
per GNU stream and512KiB per probe stream; total evidence cap512MiB also covers
retained binaries/sources. Memory is bounded by chunked same-FD hashing/copying;
no entire executable is loaded into a Python bytes value.

A prepublication exception retains partial streams and `FAILED.json` and publishes
no COMPLETE. Reporting errors after publication cannot revoke an existing COMPLETE.
A completely observed run publishes a manifest even on mismatch, with
`accepted: false`. A durable COMPLETE proves publication only; callers must check
`accepted`, all34 matches, all8 honest exclusions and source identities. No native,
scripting, curriculum, novice or other-platform acceptance is inferred.

The new Python tests use synthetic reports and mocked process execution. They
exercise guard behavior only and cannot count as actual GNU/VM comparisons.
Source-only preparation has not run Rust/Python/reference commands or generated
approval. Those actual gates belong to the implementation ledger when performed.

## Held-executable retry identity

The first fixed attempt, `ere34-r1`, remains preserved and is not an accepted
comparison: its version guard incorrectly required the literal name `grep`.
GNU uses the invoked program basename, and same-held-FD execution legitimately
reported `7 (GNU grep) 3.12`. The fixed retry is `ere34-r2`, with fresh prepared
pins and independent approval. The guard now requires the exact numeric basename
of the pinned `/proc/self/fd/N` executable, successful status, empty stderr and a
complete LF-framed numeric GNU grep version header without control characters.
It does not accept an arbitrary banner or substitute another executable. All34
supported comparisons and eight exclusions, probe source, fixture, timeouts and
resource limits are unchanged. Guard mocks are not actual comparison evidence.


## Actual fixed comparison,2026-10-29

M403's original14 guards pass. The first actual capture stopped before GNU
matching cases because the version banner used the held descriptor basename7
rather than the literal name grep. Its raw logs/FAILED marker remain unchanged.
R4 validates the exact held descriptor basename and clean full version header;
two new positive/negative groups retain all14 old bodies. M409 all393Python
checks pass. Fresh fixed ere34-r2 source/compiler/ELF/GNU pins were independently
streamed by Root before literal-only approval; no old evidence was overwritten.

M411 actual default-sandbox capture exits0:34 supported cases match GNU grep3.12
and eight VM-only cases return honest diagnostics without host execution. Raw
version is `7 (GNU grep) 3.12` because execution uses the approved held FD. All42
rows/probe observations,37 child raw/status pairs,16 retained source files and
both executed ELF copies pass independent readback9079b8ed. Original authored
predictions remain null; actual outcomes stay ignored on project disk.

Manifest SHA256:
`528033a6525f2a09217df951abf5c9f30481ee79861e8dfdd45059e15d8668c7`.
Probe1147360bytes SHA256:
`f3a65a365019b35a10e3402a66fd243523622b77535abf8cea942bbe9f41e9a2`.
GNU grep165528bytes SHA256:
`f0320b67dd6fe51276a6b0c57dac81d0fa5950dee0275ee30364e6b459b88af4`.

The captured core remains42bc6a56, distinct from the prospective cooperative
ERE14 utility. This is boolean C-byte fixed-case evidence, not full GNU/POSIX
conformance, script-utility integration, current14 migration, responsive Stop,
new curriculum or platform acceptance. Earlier source-only wording describes
the contract before this observation; retained source preserves its actual bytes.
