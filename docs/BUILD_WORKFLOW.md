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
python3 tools/dev.py python-tests --pattern test_find_size_reference.py
python3 tools/dev.py candidate
python3 tools/dev.py native ./target/debug/dlms-learning-lab --ui-smoke
python3 tools/dev.py tool verify_coverage.py --catalog teaching-review
```

Expensive all-target checks can use a stricter one-job limit, still through MAIN:

```sh
python3 tools/dev.py --cargo-jobs 1 clippy --locked --offline --all-targets -- -D warnings
```

This per-call option only accepts1or2for Cargo build/check/test/clippy and candidate. It changes
both the child command and recorded CARGO_BUILD_JOBS; Rust tests remain capped at
two runtime threads. Raw Cargo/job/test/config overrides remain refused. Native
and reference actions reject this option rather than silently ignoring
it. The default/project maximum stays2; common lock, disk targets, sampler and
pressure cancellation are unchanged. Prefer one job for expensive all-target
checks after the2026-11-12guarded Clippy run reached5.880GiB available for three
samples (fullPSImax0.18%, OOMdelta0). Headroom recovered to about15GiB after the
owned child exited. No other process, GPU, swap, service or desktop setting changed.
This is not a memory reservation or permission to overlap work.

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
RUST_TEST_THREADS=2. The runner also sets those values, passes `--jobs 2` by default (or the reviewed per-call one-job setting) and
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

The candidate script also takes the shared lock when invoked directly and
defaults to two Cargo jobs. Use
`python3 tools/dev.py --cargo-jobs 1 candidate --presentation-review` for a
single-job local review build. Both paths stage on the common project disk from
the main checkout and hash
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

Catchable POSIX SIGTERM/SIGHUP now unwind that same owned-child cleanup. A signal
during Popen is deferred until the returned child is known; handlers are restored
after the CLI workflow. SIGKILL remains uncatchable, so inherited-lock/marker
recovery still requires exact process inspection. Long native/test sessions use
an allocated tool terminal and regular handle polling; neither that nor a partial
capture proves completion. Preserve interrupted logs and verify the actual exit.

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

The new developer-only `loop_control_bash_oracle.py` is likewise a fixed-fixture
exception, not an execution route for learner scripts. It requires MAIN/default
runner, the common inherited lock, one exact detached evaluator13 worktree and
its exclusive target, all898 reviewed source paths and the compiled probe's
identity. Its24 pinned cases partition16 builtin-only Bash comparisons from eight
explicit VM-only exclusions. Every host comparison uses disposable disk fixtures,
stripped environment, empty search path, bounded time/output/files and a retained
executable descriptor. There is no arbitrary script, worktree, engine or path
selector. All genuine evaluator12 outputs,296 source inputs and retained producer
are verified before work. `--mode prepare` launches nothing; capture requires
independent exact-pin approval. A completed capture with mismatches is retained
and rejected, never treated as acceptance. Synthetic guard tests, preparing the
control and building the probe are not actual Bash comparison evidence.

Its collector-local source-identity reader accepts an ordinary detached HEAD,
ordinary loose branch ref, or bounded packed refs only when the loose ref is
absent. It refuses aliases, malformed/duplicate records and non-LF control
separators. Do not rewrite Git refs or alter immutable historical helpers to make
a new capture tool pass. Existing historical producers and receipts stay exact.


The genuine evaluator13 producer similarly requires the nondefault
`historical-evaluator13-capture` feature and actual revision13. Its fixed
normal182 baseline was observed and independently sealed before future14 work;
see SHELL_EVALUATOR13_BASELINE.md. The new guarded
`tools/dev.py tool scripting_before14_verify.py --mode verify` checks retained
499 API observations,147-context original/draft profiles,708 source inputs and
the actual executed producer without replay. Preserve original9–13 producers,
raw observations and source/ELF bundles. Explicit old producer compilation after
a revision advance is refused; no feature selects an earlier evaluator.
New language still needs migration/negative/current-state tests; the seal itself
is not new evaluator or curriculum acceptance. All memory/lock/disk rules above
remain mandatory for this tool and every worker.


The genuine evaluator14-before15 harness was actually captured on2026-10-30
against normal193 after its installed Fedora acceptance. The nondefault
`historical-evaluator14-capture` feature requires actual14; it never selects
older semantics. M565537API outcomes and preserved150-context original/draft
profiles are independently sealed (6046f5fb/9089741a). Keep all737retained source
inputs, executed producer and old9–13 histories immutable.
`tools/dev.py tool scripting_before15_verify.py --mode verify` verifies retained
bytes without replay. Do not overwrite/recreate its fixed original destination,
rerun old producers after an advance, invent observations or treat a baseline as
new mastery. BEFORE15_CAPTURE.md records actual source/ELF/control/limits.

Package README generation now validates real platform launch/resources and
writes relative standalone instructions. Disposable Linux/Windows/macOS layout
fixtures test only this packaging logic, not native platform acceptance. Existing
193 candidate/archive stays immutable with its documented old-README limitation;
the correction applies only to later newly built candidates.


The fixed `tools/dev.py tool debug_recovery_reference.py` compares only two
SHA-pinned authored project recipes with40 static synthetic cases. It has no
caller source/path/profile selector, requires the inherited MAIN lock, stages
only under project-disk reference-runs, strips child environment and bounds each
Bash child’s time/CPU/address space/output/files. This development-only comparison
uses real Bash as an oracle, never as learner execution. It neither changes the
evaluator nor reruns/rewrites any sealed historical producer. Retain its original
observations and review source pins before any future recipe change.

The genuine evaluator15-before16 producer is fixed to the actual255 review
catalog,66scripts/549cases/66separate examples. Its nondefault
historical-evaluator15-capture feature requires actual15; it cannot select older
semantics. The genuine capture and156-context profiles were independently read
back and sealed before a future advance; see SHELL_EVALUATOR15_BASELINE.md.
Keep its496 compiled bindings,502 retained inputs, held-descriptor ELF and all
raw observations/profiles/backups immutable. The new guarded
`tools/dev.py tool scripting_before16_verify.py --mode verify` only reads retained
bytes. Old9–14 producers and closures remain unchanged. Future comparisons or
progress migrations do not manufacture mastery from baseline/reference scripts.

The genuine evaluator16-before17 producer was actually run and independently
sealed against257/74 before environment semantics:573graded+68example outcomes,
158-context synthetic profiles,90unrelated contexts and two unfinished drafts.
See SHELL_EVALUATOR16_BASELINE.md. The new historical-evaluator16-capture feature
requires actual16, never selects old semantics, and is excluded from ordinary
default targets. The before17 verifier checks retained bytes without replay.
Keep original9–16 source/ELF/observations/profiles immutable. No new mastery or
future17 acceptance follows merely from this compatibility baseline.


## Project-only debug data reduction —2026-11-04

A necessary two-job widget-test compilation was safely interrupted by the
pressure monitor (full/some PSIavg10 reached12.83%; no OOM increase). Root-crate
`dev` and `test` package overrides now use `debug = "line-tables-only"`: file/line
backtraces remain, while type/variable debug data is omitted. Dependencies,
release profile, debug assertions and overflow checks retain their settings.
This changes build diagnostics, not grading or tests. See the official
[Cargo debug/profile documentation](https://doc.rust-lang.org/cargo/reference/profiles.html#debug),
checked2026-11-04. Do not override limits or bypass the monitor; record the next
ordinary run's actual memory/PSI and stop on sustained pressure again.


The fixed `native_reopen.py --case user-mounts` adds only the two authored
caller-mount tasks to the existing private two-process harness. MAIN/common lock,
explicit review catalog,96bounded native phases,120second child deadline, capped
logs/evidence and fresh project-disk profiles remain mandatory. Writer uses real
editor/caller/Mount controls; reader observes exact ordinary loaded history and
three drafts before any verification frame, then repairs the shared task through
native controls. It never accepts a supplied script, mountpoint or profile path
for host execution. This development route is not a new learner feature.


The new fixed `startup_reference.py` compares18 reviewed builtin-only settings
recipes through explicit sourcing in disposable project-disk fixtures. It does
NOT test Bash's automatic login/rc selection; that distinction is in its reports.
Prepare modes launch no child. Actual observation and VM comparison require
independent exact source/executable approval; an observation seal follows actual
readback, never proposal data. MAIN/default target/common inherited lock, held
binary descriptors, stripped environment, fixed no-profile/no-rc Bash, resource/
time/output limits and retained evidence remain mandatory. The EXIT observer is
a developer fixture only, not a learner trap capability. No caller can select a
script, file path, profile or source tree. Existing environment and historical
collectors/seals remain unchanged. Use the main dev runner for every phase.

## Actual17 before18 prerequisite (preparation, no capture credit yet)

The new nondefault historical-evaluator17-capture target is a fixed developer
producer, not a selector for old semantics. Both runner and source assert actual17;
ordinary default all-target checks exclude it. It has no source/profile/path/engine
CLI. The new fixed before18 tool requires MAIN/default target and the inherited
lock. Prepare launches nothing; capture needs independent exact-source/ELF approval.
Never run the example manually. No actual17 baseline is established until real
execution, full source/raw/profile readback and a separate independent seal. Keep
all old9–16producers, controls, source, ELFs and observations immutable.

The inherited pair remains158contexts with zero new Script attempts; preserve90
unrelated contexts and two unfinished assignment/example drafts. A separate
explicitly synthetic typed fixture covers the newer Environment/StartupFiles
formats; imported runtime proof must remain absent until actual rerun. This adds
no learner credit, utility support or new curriculum. Next flat-file utility
semantics must wait for the actual17 seal. See CHECKPOINT.md for current status.

Actual17R2 capture has now completed and passed independent raw/source/ELF/profile
readback. See SHELL_EVALUATOR17_BASELINE.md for identities and limits. Retain
544inputs/538bindings,18outputs, the exact411186000-byte ELF and all unacceptedR1
artifacts. `tool scripting_before18_verify.py --mode verify` reads sealed bytes
only. The historical17feature must refuse replay after any evaluator advance;
never overwrite/relabel this observation or infer learner mastery from it.

## Actual18 before19 compatibility

The nondefault historical-evaluator18-capture example and fixed before19 tool
require genuine18 for capture. The actual277catalog baseline is now independently
sealed:679ordered outcomes,158zero-addition inherited contexts, separate new
synthetic profiles and32typed phases. See SHELL_EVALUATOR18_BASELINE.md.
Retained verify reads bytes without replay; original producers, source, executable
and evidence remain immutable. A Cargo feature never selects old semantics.
Continue using MAIN runner, shared lock, two jobs/runtime threads and disk artifacts.

### Uncommitted review-build lifecycle identity

The owner currently forbids commits/publication. A `-dirty` review candidate may
enter the private installation fixture only with explicit
`--candidate-manifest-sha256 HASH --previous-manifest-sha256 HASH`, taken from
independently retained build receipts. The bounded ordinary manifests must match
those exact hashes; declared source metadata must match all bundled scenario
hashes and the protected review profile/catalog. Same-revision upgrades require
different actual executable hashes and a differing common source entry; an
added/deleted metadata row alone is insufficient. Both entire installed manifests
are verified. This is controlled review-build identity, not a release or an
independent reconstruction of every historical source file. Original retention,
private-prefix, desktop, timeout and uninstall checks still apply.

## Fixed managed-configuration authoring comparison

`tools/dev.py tool managed_config_reference.py` accepts no script/path selector.
It compares only the exact SHA-pinned canonical/alternate twelve-case fixture and
separate two-call example, using held Bash/cat/grep descriptors, empty PATH/no HOME,
private disk files, bounded streams/time/CPU/address space and the inherited lock.
Independent source review precedes execution; actual raw readback establishes only
that fixed composition. Never connect learner drafts to it or rewrite historical
producers/seals. Runtime interpreter remains in-process evaluator19.


The fixed `links_inode_reference.py` uses one reviewed generated world and only
six supported directory queries plus two explicit exclusions and a version call.
It accepts no learner command, arbitrary path or script. Prepare launches nothing;
observe/compare require independently approved exact source/ELF/observation pins.
Original0755 GNU ls and the compiled in-process probe use held descriptors, bounded
child-only NNP/limits and disk fixtures under the MAIN shared lock. Preserve the
actual source/ELF/raw evidence after source advances; never rerun a historical
control against new content. See LINKS_REFERENCE.md for the narrow actual results.

## Stage large root targets after low-headroom cancellation —2026-11-05

A first three-integration-target compile for the additive directory-policy model
was interrupted by the normal guard: minimum available memory5.599GiB, some/full
PSIavg10=0 and no OOM increase. No runtime tests passed in that interrupted run.
The owned child was reaped and both active/unfinished markers were absent. Keep
that evidence; never remove markers or interrupt another workload to retry.

After headroom recovered to11.1GiB, splitting invalidated root targets worked:
library-only42.10s (minimum6.170GiB), then app-only19.17s (minimum9.032GiB), then
one integration-test target. All still use MAIN's shared lock, two Cargo jobs,
two runtime threads, disk temporary data and unmodified pressure thresholds.
This avoids overlapping the large library/application/test root compilations;
it is not a higher allowance or a guarantee. Start the next command only after
an actual successful exit and ordinary headroom reassessment.

```sh
python3 tools/dev.py build --locked --offline --lib
python3 tools/dev.py build --locked --offline --bin dlms-learning-lab
python3 tools/dev.py test --locked --offline --test sticky_directory
```

Run other necessary integration targets separately when a core invalidation has
just triggered this pressure pattern. Do not repeat the same broad compilation
immediately, lower protections, run raw Cargo, change GPU/swap/model/services or
claim that passing a library build means runtime/teaching tests passed.

## Stage Clippy's large roots too —2026-11-05

After the sticky lesson integration, a direct all-target Clippy run stopped at
5.647GiB available (22samples, some/fullPSIavg10max0.72%, OOMdelta0). It was not
a lint/test pass. Owned-child cleanup left both current marker paths absent;
historical recovery copies/diagnostics were retained. No global setting changed.

Compilation/packaging were held during lightweight ledger checks. After ordinary
headroom recovered to11.6GiB with currentPSI0, checking the library alone passed
20.16s/min8.725GiB; then the app passed7.85s. Three new integration targets were
checked separately. Resuming all-target Clippy only after those roots were cached
passed49.23s/min8.270GiB/maximumfullPSIavg10=0.18%/OOMdelta0. The initial failure
remains in evidence. This is an observed mitigation, not a memory reservation.

```sh
python3 tools/dev.py clippy --locked --offline --lib -- -D warnings
python3 tools/dev.py clippy --locked --offline --bin dlms-learning-lab -- -D warnings
python3 tools/dev.py clippy --locked --offline --test directory_policy_workspace -- -D warnings
python3 tools/dev.py clippy --locked --offline --test directory_policy_source -- -D warnings
python3 tools/dev.py clippy --locked --offline --test directory_policy_lessons -- -D warnings
python3 tools/dev.py clippy --locked --offline --all-targets -- -D warnings
```

Do not retry a broad compile immediately after cancellation or bypass the normal
pressure gate. Reassess actual headroom and stage invalidated large roots; if
pressure returns, stop launching further heavy work and report it. Keep one MAIN
lock, two Cargo jobs/runtime threads and disk-backed artifacts. Never interrupt
another workload, reduce GPU reservations or change swap/OOM/model settings.

## Fixed ordinary-directory comparison

`tools/dev.py tool recursive_tree_reference.py` has only prepare/compare and an
exact control digest; no caller script, path, command or executable selector.
The fixed8cases use owned project-disk fixtures, retained source and held original
GNU/probe descriptors, stripped environment, empty stdin, child-only NNP/limits
and Popen child umask. No parent/global mask/security setting changes. R11actual
8matches/3model-only exclusions are retained under recursive-tree-r1. This is root
source-reviewed development evidence, not an independent seal or full GNU fidelity.
Never feed learner input to it, overwrite old attempts, or rerun the old control
after source changes. See RECURSIVE_TREE_SUBSET.md for identities and omissions.

### Focused Python checks

The guarded runner now accepts `python-tests --pattern test_NAME.py` for one existing
ordinary file in tools/tests. Default no-argument execution remains the inclusive
suite. Unknown flags, wildcards, missing files and aliases refuse rather than
silently starting the full suite. The same MAINlock, disk temporary paths and
memory sampler apply. Newly authored tests initialize their own project-tool import
path so focused execution does not depend on alphabetical full-suite import order.
R14three command-construction guards and R15five fixed-size-reference guards pass;
R12/R13initial import failures remain retained. No learner execution route is added.

## Fixed offline bond keyfile check

`python3 tools/dev.py tool nm_bond_keyfile_reference.py` accepts no arguments or
learner input. Under the MAIN lock it runs exactly three fixed synthetic
`nmcli --offline connection add` fixtures and a version query against a retained
installed executable descriptor, with bounded streams/time and project-disk data.
This checks serialization/normalization only: no daemon, profile installation,
interface activation, packets or host settings. It is never the learner engine.
Retain actual captures and diagnostics; tests of guards are separate from the
observed fixed fixtures. See BOND_CONTROL_SUBSET.md for the limited fidelity claim.

The separate fixed `nm_bridge_keyfile_reference.py` has the same no-argument,
constant-three-fixture, inherited MAINlock, held-executable, finite stream/time and
project-disk boundary. It checks only manual bridge controller, bound Ethernet port
and unbound candidate serialization. It never installs profiles, activates devices
or runs learner input; see BRIDGE_CONTROL_SUBSET.md for actual observed scope.

## Interrupted tool-terminal candidate evidence

On2026-11-08an owned candidate terminal reportedexit1afterCtrl-C, its monitor
stopped and a command marker remained. This is not normal runner cleanup or a
successful build. The exact marker/log/snapshot were retained. No new heavy work
started until the common kernel lock was acquired nonblocking and the exact
marker was archived under that lock after process-state inspection; per-tool
PID namespaces limit host visibility. The private r62-lock-recovery.json records
those limits. Never infer staleness from a missing PID alone, automate removal,
bypass a held lock or signal an unrelated application. Prefer polling an ordinary
bounded build to completion. All normal shared-lock, two-job/two-runtime-thread,
private disk and pressure guards remain required after recovery.

## Native action polling and disappearing controls —2026-11-10

An optimized Wayland capture produced1200redraws in218mswhile waiting for150ms
settlement. Count eligible action polls, not passive redraws; retain the absolute
15second deadline on every frame,1200active-poll limit and paced unfinished phases.
Keep strict20reveal attempts/one-physical-pixel clipping. Do not raise guards to
hide a failed capture. Use the existing visible-click helper to drain a delivered
click before requiring geometry: Confirm reset and Back can disappear on the next
layout pass. Regression must click the actual widget, let that layout settle,
then acknowledge the click; asserting only the old rectangle misses this race.
Read theme/context values before a fonts_mutclosure as well as before ctx.data
closures; context re-entry while holding a font lock can deadlock. R1–3failed
artifacts and exact before/after regressions remain private. These fixed developer
checks never run learner commands or replace original progress.

## Monitor-aware native capture geometry

The opt-in capture_size helper for the wired-device and retained mail workflows
reads reported monitor geometry and OS density after text zoom and fullscreen
have settled. Viewport size commands use UI points, so convert the requested
native-pixel size using the current pixels-per-point. Record both requested and
actual dimensions. Wayland can retain a smaller tiled window even after a valid
request. Require consecutive stable actual geometry inside the bounded narrow
size and above the app minimum. A full-size or oversized window must fail.

The actual946×929, native1.15,200% fixture exposed the old fixed-size assumption.
Do not raise the15second action deadline,1200active-poll limit,20reveal attempts
or one-physical-pixel click-visibility tolerance to force a pass. Capture complete
individual controls/cards and scroll normally when a page exceeds the viewport.
These are isolated application-window checks and do not change desktop settings.


## Nested-editor capture scrolling —2026-11-11

The opt-in JSON capture route uses the measured outer scrollbar gutter for
parent-page wheel events. A point at the content's left edge can fall inside a
full-width nested editor and scroll text instead of the lesson. Revalidate the
actual control clip after settlement, immediately before screenshot request.
Capture-only focus clearing prevents an earlier keyboard action competing with
navigation. Keep the15second deadline,1200active polls,20reveal attempts and
one-physical-pixel visibility limit. Actual complete200%editor views and a clipped-
editor/queued-gutter regression support this correction; partial R1images remain.

## Fixed measurement-report comparisons —2026-11-11

`python3 tools/dev.py tool measurement_scripts_reference.py` admits only three
hash-pinned authored canonical/alternate fixtures and their separate examples.
It requires the inherited MAIN/default-target lock and retains bounded results in
private project-disk fixtures. The75 observed Bash5.3.9 comparisons exercise
quoted fields, arithmetic, validation, EOF/rebaselining, output files and status.
The tool has no learner source/path selector and never becomes app execution.
Five guards test fixture/tool identity and containment. Existing sealed historical
collectors, observations and compiled source remain unchanged; this is a new fixed
composition comparison, not a replacement migration baseline. Keep all build,
reference, test and native invocations serial under this runner.

## Hold reviewed inputs during identity/native checks —2026-11-13

Do not edit source/data/tool inputs while a source-identity suite or native capture
runs. Loaded pin tables can predate a late edit even if the on-disk table is then
refreshed. Reap the child, retain the failed result, review only current inputs,
then run the affected guards. Historical collectors/producers/seals remain fixed.
The leaf/spine R1Python result retains this failure; R2inclusive1284and fresh final
source guards pass. Candidate freezes additionally include bundled docs/supplements
until final readback/retention. Never re-attest an old package against later main.

Coverage inventory reads at most2MBper file, verifies/discards exact historical
origins using closed fingerprints plus selected-value equality, and retains the
32MBunintegrated draft/unit-data budget. Historical copies are not new drafts.
Negative tests retain oversized/changed-origin refusal; no selected limits raised.

## Focused library tests after the2026-11-14guard stop

A one-job combined --lib/integration test build was stopped after three samples
below6GiB available(5.924/5.823/5.684). No OOM; do not relabel it a passed run.
The218sample log is retained privately as20261114T190824.007450Z-test-memory.jsonl.
After headroom recovered, the identical new region-math assertions ran through a
dedicated integration harness using the actual source file, plus the normal linked
production API. This avoids compiling the entire library-test binary. The first
scoped18test run passed in45.01sbuild time, min9.269GiB/maxfullPSI0.18/OOMdelta0.
A later scoped23run adds typed policy binding. These are focused model checks,
not an inclusive Rust runtime suite. Keep exact assertions and source identity;
do not substitute a simplified model, bypass the shared lock/guard or raise jobs/
threads. Use MAIN runner Cargo1/runtime2and named integration targets.

```sh
python3 tools/dev.py --cargo-jobs 1 test --locked --offline --test linux_filter --test linux_filter_regions --test linux_router_path --test linux_filter_policy
```

The normal application/candidate still links the actual library. New registered
lessons must separately pass source-context, persistence, content and native checks.
If pressure returns, stop launching additional work and record it. A smaller target
reduces risk; it is not a memory guarantee or permission to interrupt other work.

## Fixed IPv6 administration reopen case

`tools/dev.py tool native_reopen.py --case v6-admin` adds one fixed authored
saved-settings task to the existing normal two-process harness. It creates all
work through native controls, restores raw progress/preferences before repair,
and distinguishes saved file, unsaved editor, pending Load command, current flags
and supplied observations. No caller script/file selector or host sysctl/network
action. The scoped saved-settings trial is not an OS reboot or firewall boot test.
Keep MAINlock, disk profiles, existing deadlines/caps and actual exit readback.
If a sandboxed launch reports NoCompositor while the Wayland socket exists, retain
that failure and request authorized display access; do not change compositor,
security policy or desktop settings or silently relabel that failed launch.


## Fixed provided IPv6 loader reopen case

`python3 tools/dev.py tool native_reopen.py --case v6-startup --app EXACT_APP`
uses the same guarded two-normal-process harness, fresh project-disk four-XDG
profile and bounded120second phases. It verifies saved rules, separate unsaved
editors, queued enable commands and actual traffic/startup feedback before native
Save/Enable/Restart/Test/Check repair. ExecStart remains inert supplied text; no
host firewall/service/network action. Keep source/data/tool inputs fixed during
capture and use an allocated tool terminal with regular polling. The first current
cold run received SIGTERM with healthy memory; its source is unknown, diagnostics
remain, and the allocated-terminal retry passed without raised caps or bypasses.

## Fixed supplied-server exposure reopen case

`python3 tools/dev.py tool native_reopen.py --case service-exposure --app EXACT_APP`
uses the existing MAIN-lock/two-normal-process harness and private four-XDG profile.
The fixed authored client-policy task preserves actual filter/listener state, selected
observer, current scan report, source snapshot and separate queued server/observer
commands. Reader compares raw progress/preferences before native Run/Check repair.
It accepts no caller target, command, script or personal profile. Existing120second
phase deadlines and file/output/evidence caps remain unchanged. A passing plan test
is not an observed native reopen; retain actual process exits and original images.
No scanner, socket, host firewall/service, dependency or desktop change is involved.


## Fixed RPM configuration content comparison —2026-11-15

Build the fixed probe through MAIN Cargo1, then run the no-selector tool serially:

```sh
python3 tools/dev.py --cargo-jobs 1 build --locked --offline --example rpm_configuration_probe
python3 tools/dev.py tool rpm_configuration_reference.py
```

This development-only tool compares14fixed inert content cases using three tiny
script-free archives, fresh private payload paths and an explicit private RPM
database under .build-artifacts/reference-runs. Archive file lists and absence of
scriptlets/triggers are checked before installation. Personal macro/rc defaults
are replaced with explicit system sources; plugins and transaction scripts/triggers
are disabled. Child-only limits/no-new-privileges, RPM worker/thread counts1, fixed
system utility PATH, private HOME/TEMP and the MAIN lock/pressure guard remain.
There are no caller source/path/case selectors and no learner/host RPM database
bridge. Artifacts/diagnostics remain retained, never in RAM /tmp. The current
RPM6.0.2comparison matches14cases; it is content evidence, not full RPM conformance
or an independently sealed historical baseline. The immutable existing version
oracle is unchanged. Earlier local setup failures remain; no privilege, memory
limit or system setting was changed to obtain the passing comparison.


## Fixed offline filter comparison —2026-11-15

Build through MAIN Cargo1, then run the no-selector comparison serially:

```sh
python3 tools/dev.py --cargo-jobs 1 build --locked --offline --example log_filter_probe
python3 tools/dev.py tool log_filter_reference.py
```

Only twelve reviewed SHA-pinned short definitions/48inert lines are evaluated by
Python re; a held original fixed Rust probe reports actual in-process observations.
No learner pattern/path/script selector exists. MAINlock, bounded streams, private
disk fixtures and child-only NNP/CPU/address-space/file-descriptor limits remain.
The actual observations match48lines; source/ELF/raw outputs are retained privately.
This is a current fixed common-ASCII comparison, not actual fail2ban-regex/daemon
conformance or an independently sealed baseline. Historical collectors/seals stay
unchanged; never re-attest this old capture against later source. A pin-shape setup
failure before the child is retained. No dependency, service or system setting changed.
