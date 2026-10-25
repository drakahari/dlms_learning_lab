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
