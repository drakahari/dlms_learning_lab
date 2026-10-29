# Memory-conscious development workflow

Added2026-10-23 after the owner reported system-wide exhaustion during concurrent
Rust builds/linking and accumulated RAM-backed temporary artifacts. Linux sees
about31.1GiB on this desktop, not the machine's64GiB physical total. Other workloads
share that memory. Nothing here changes GPU reservations, swap, OOM policy,
LM Studio, its connect-only monitor, services, mounts or global shell settings.
Two Cargo jobs reduce risk; they do not guarantee sufficient memory.

## Commands for this checkout and delegated worktrees

Use the main checkout's runner for builds, tests, packaging and native/reference
checks. It takes one shared OS lock derived from Git's common directory, so a
worktree does not get an independent heavy-build allowance. A second invocation
refuses to launch work rather than competing or stopping another process.

```sh
python3 tools/dev.py build --locked --offline
python3 tools/dev.py test --locked --offline --all-targets
python3 tools/dev.py clippy --locked --offline --all-targets -- -D warnings
python3 tools/dev.py python-tests
python3 tools/dev.py candidate
python3 tools/dev.py native ./target/debug/dlms-learning-lab --ui-smoke
python3 tools/dev.py tool verify_coverage.py
```

Set a smoke-stage environment variable only for that invocation when needed.
Never pass learner-supplied scripts to host tools. The reference tools retain
their fixed, disposable development-fixture restrictions.

Native capture code must not call other egui Context methods from inside
`ctx.data` or `ctx.data_mut` closures. Read geometry, zoom and pixels-per-point
first, then store only the precomputed values. Context re-entry has caused two
capture-only lock failures; it is separate from the historical Wayland callback
issue. Exercise actual capture-phase code in a regression, not only its case
list, before running a new native measurement stage.

Every worktree mutation must set its explicit absolute working directory. Check
the selected checkout before applying a patch; never rely on an inherited shell
cwd. A setup error briefly applied a source patch in main on2026-10-25; the exact
patch was immediately reversed and30 pinned main files independently rehashed.
No build ran against that temporary change. Retain the base/source receipts and
verify the intended checkout before continuing.

For a same-project worktree, invoke the main runner with `--cwd WORKTREE` before
the action. Each canonical checkout now has an exclusive Cargo target. Main
keeps `target/`; a worktree uses `.build-artifacts/targets/NAME-PATHHASH` on the
main project disk. The runner prints the exact path. Do not share a target even
when builds are serial: Cargo can reuse a newer same-package library from a
different worktree, combining a new test executable with the wrong bundled data.
An actual comparison caught this on2026-10-23 before producing results. Old
shared targets and failed diagnostics remain untouched. `--target-dir` only
accepts requests resolving to the automatically selected path; the target
namespace itself cannot be a symlink alias. Cross-worktree/main overrides are
refused. Candidate builds run only from main and keep their
separate packaging target. The common heavy-work lock still serializes ALL targets.

```sh
python3 /path/to/dlms_learning_lab/tools/dev.py --cwd /path/to/worktree test --offline --test acl
```

`.cargo/config.toml` sets build.jobs=2 and separately supplies
RUST_TEST_THREADS=2. The runner also sets those values, passes `--jobs 2` and
appends `--test-threads=2` to Rust test runs. Upward CLI overrides are refused.
CI has explicit two-job/two-test-thread settings on each separate native runner.
Fixed reference tools currently require the main checkout/default target so a
worktree cannot accidentally test the wrong probe. New output options are
normalized against the selected working directory and checked on disk; ambiguous
or abbreviated forms are rejected. Do not call raw Cargo/rustc to evade the shared lock. A necessary standalone
compiler/probe run must be coordinated through this runner or an explicitly
reviewed extension to it; it must not overlap another heavy command.

UI unit-test setup validates the immutable bundled catalog once per test binary
and clones that owned baseline into each fresh application. This cfg(test)-only
cache shares no context, profile, progress, draft, grade or expected outcome.
Normal/development/candidate GUI binaries still run their original pack/unit
validation; direct loader and integration tests remain uncached. Keep two runtime
threads and the shared heavy-work lock; this is not permission to add concurrency.
The isolation regression and measured complete-run evidence belong in the ledger.

The immutable public-evaluator9 capture example is a historical-only target behind
the nondefault `historical-evaluator9-capture` Cargo feature. Ordinary all-target
checks exclude that one old producer; its accepted original source, executable
and evidence remain preserved privately. Explicit selection of that target,
feature or `--all-features` is refused before compilation unless the selected
checkout really declares evaluator9. A feature never switches current evaluator
semantics. The evaluator10 producer similarly requires the nondefault
`historical-evaluator10-capture` feature and actual revision10. Its fixed original
capture, source bundle and producer are retained privately before revision11;
`tools/dev.py tool parameters_before11_verify.py verify` checks historical bytes
without requiring current semantics to remain10. Do not rerun/overwrite the
original capture or attest it against changed source. Use ordinary default-feature
gates for current development; do not bypass either refusal to relabel historical
observations. New comparison and private
migration tests run current in-process simulation against hash-pinned old evidence.

The genuine evaluator11 before12 producer similarly has its own nondefault
`historical-evaluator11-capture` feature. Explicit compilation requires actual
revision11; it never selects an older engine. Its fixed synthetic fixture uses
only the reviewed52 authored scripts/384 cases/52 separate examples and the
pinned genuine10 private profiles, adding only3 actually executed script contexts.
No Ether/Cron answers or personal progress are fabricated. The new fixed tool
`tools/dev.py tool scripting_before12_verify.py --mode prepare` launches nothing;
`--mode capture` requires independent approval of exact source/producer pins and
refuses an existing destination. Only a genuinely observed and separately sealed
manifest/attestation authorizes `--mode verify` after a future evaluator advance.
Keep original9/10/11 producers, compiled source, executables and observations
immutable. A source proposal or unsealed capture is not migration acceptance.

The developer-only `case_classes_bash_oracle.py` is a narrow, fixed-fixture
exception for the reviewed evaluator12 worktree. It still enters through MAIN's
default-target runner and common lock. It derives that one worktree's exclusive
target, validates its complete reviewed source inventory and compiled probe
identity, and requires independently approved exact pins before capture. It has
no arbitrary source, path, probe or checkout selector. The48 builtin-only cases
use disposable directories and a stripped C-locale environment; every mismatch
is retained. Preparing a control or passing synthetic guard tests is not actual
Bash comparison acceptance. Existing collectors' Main/default-target restrictions
and historical producers remain unchanged. This tool never executes learner work.

## Disk location and toolchain

The dedicated ignored directory is `.build-artifacts` under the main checkout.
On this Fedora session it is on `/dev/nvme1n1p3`, Btrfs `/home` (NVMe-backed).
`/tmp` is tmpfs and is rejected for new artifact roots. The runner verifies Linux
mount information, refuses RAM filesystems/overlay for the artifact location,
and assigns TMPDIR, TMP and TEMP only inside its own process/children. Native
Windows/macOS volume/runtime acceptance is still pending.

- `tmp/`: compiler, Python, native smoke-profile and packaging staging files.
- `toolchains/cargo` and `toolchains/rustup`: copies of the already installed,
  isolated Rust1.99.1 development toolchain and cached sources. The original
  `/tmp/dlms-cargo` and `/tmp/dlms-rustup` remain untouched pending cleanup approval.
- `logs/`: per-invocation command/environment limits and memory JSONL observations.
- `audits/`: local exact-path temporary-artifact inventory/cleanup proposal.
- `targets/`: isolated worktree targets; main `target/` and `dist/` already reside
  on the same disk volume and remain supported.

For a fresh development checkout, install the tested Rust toolchain through your
normal approved process first. This runner does not install toolchains or packages.
Provide an existing isolated Cargo/Rustup cache to the copy helper; it refuses
RAM-backed future output and never removes the source.

The one-time copy command is `tools/bootstrap_toolchain_cache.py` with explicit
existing source directories; it never replaces a destination or removes a source.
A completed READY.json, rustup executable hash and compiler/version checks precede
use. Normal builds reuse these local copies and operate offline. No packages or
new dependencies are installed by this change.

The candidate script also takes the shared lock when invoked directly, passes
`--jobs 2`, stages on the common project disk from the main checkout, and hashes
large artifacts in1MiB chunks. It refuses a dirty release identity as before.
It does not publish a release. Existing candidates are preserved.

## Memory observations and pressure response

Every runner invocation samples MemAvailable, total/free swap, memory PSI and
pswpin/pswpout/oom_kill counters approximately once per second. It records no
other application's command line, model settings or personal file contents.
Initial available memory below6GiB, full PSI avg10>=5%, or some PSI avg10>=15%
refuses a new command. Three consecutive pressured samples flag sustained
pressure. No additional command is started; a running runner-owned child/group
is interrupted and reaped. No unrelated application or agent is interrupted.
These are conservative workflow thresholds, not a reservation or OOM predictor.

Near-full zram is reported honestly even when current PSI is zero and available
memory is adequate. Measurements from an ordinary necessary build/test run are
technical observations, not a stress test or proof of safety under every workload.
A command marker is written before spawning and remains if the wrapper dies
abruptly or owned-child cleanup fails. Future heavy work refuses that marker
until the exact process state is inspected; do not simply remove it to bypass
a running job. On POSIX the child also inherits the shared lock descriptor.
Normal/error/KeyboardInterrupt cleanup is bounded and targets its own process
group, including descendants. Sampler failures fail closed.

Fake-pressure unit tests exercise the refusal/cancellation path without creating
real memory pressure. Gate failures retain their evidence and require reassessing
headroom before retrying, not changing swap, GPU allocation or OOM behavior.

## Existing temporary data

After the owner approved the exact seven-file list, only those old ELF review
executables were removed:1,513,992,192 allocated bytes (1.410GiB). Matching sources,
diagnostics, all toolchain originals, worktrees and uncertain/pending evidence remain.
The process-reference/identity checks and exact result are recorded locally.
Full local inventories and any future proposed cleanup paths live under `.build-artifacts/audits/`. Retain diagnostics,
original evaluator baselines, worktrees, unreviewed curriculum, installed acceptance
fixtures and uncertain artifacts. Approval of an exact cleanup list is required
before removal; neither broad wildcard deletion nor `cargo clean` is authorized.

## Actual Linux candidate lifecycle

Use the fixed guarded tool with exact existing candidate directories and expected catalog counts:

```sh
python3 tools/dev.py tool installed_acceptance.py --candidate dist/EXACT-NEW-CANDIDATE --previous dist/EXACT-PREVIOUS-CANDIDATE --labs 98 --units 23
```

It verifies build-manifest/binary identity, installs only into a new private project-disk directory, upgrades an actually identified older executable, checks installed content/desktop metadata, uninstalls/reinstalls, and retains exact separate synthetic original-nine progress bytes. Resolved candidate/install roots cannot escape the project or use RAM filesystems. Leftover desktop entries and dangling application/launcher links fail acceptance. Fixed commands have nonblocking32KiB streaming capture and bounded timeout/pipe teardown under the parent's shared lock, memory observer and owned process group. Descendants never detach from that group; failed runs return to its cleanup. Per-command logs and acceptance.json remain under ignored .build-artifacts/installations. No personal progress, desktop theme, host service or learner script is accessed. This proves installer file retention on this host; native GUI/reopening/startup and other operating systems require separate checks.


## Retaining CI installation candidates

Keep downloaded archives and extraction output under ignored dist/, with private
metadata/verification under .build-artifacts/. For the gh download process as well
as the final destination, set disk temporary paths for that command only:

```sh
env TMPDIR="$PWD/.build-artifacts/tmp" TMP="$PWD/.build-artifacts/tmp" TEMP="$PWD/.build-artifacts/tmp" gh run download RUN_ID --name EXACT_ARTIFACT --dir dist/native-ci-REV/PLATFORM
```

Run from MAIN after verifying the disk location above; do not change global
shell variables. Serialize large retrieval/inspection with local heavy work.
Verify exact revision and sidecar SHA-256 before owner delivery. Inspect bounded
archive metadata without executing a Windows/macOS binary on Fedora; CI success
is not desktop acceptance. Installing, unpacking and native acceptance remain
separately guarded project-disk operations. No command here publishes a release.

The genuine evaluator12 before13 producer has its own nondefault
`historical-evaluator12-capture` feature and actual-revision12 guard. Its closed
Main171 fixture derives55 scripts/420 cases/55 separate examples from static
authored input pins. It derives prior context IDs from sealed genuine11 data and
adds only the three actually executed numeric-script fixture contexts; genuine
unfinished drafts, non-script Attempts, old records and receipts remain exact.
`tools/dev.py tool scripting_before13_verify.py --mode prepare` launches nothing.
Capture requires independent approval of exact producer/source/control pins and
a fresh project-disk destination; verification cannot authorize revision13 until
actual observations and retained source/binary have been independently sealed.
No arbitrary source/profile/engine/path selector exists. Original9/10/11 capture
producers/evidence remain immutable. Rebase the closed catalog index explicitly
if newer normal non-script content is integrated before capture.

The fixed Main171 capture was completed before the pending networking
integration and independently sealed on2026-10-29. Keep that original index,
compiled source and executable; do not rebase or regenerate this completed
capture for later catalogs. See SHELL_EVALUATOR12_BASELINE.md and the ledger for
its exact observations and verified identities.
