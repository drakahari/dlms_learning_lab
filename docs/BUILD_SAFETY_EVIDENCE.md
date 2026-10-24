# Fedora build-workflow verification — 2026-10-23

Owner-prioritized safeguard work; not application/curriculum completion or a stress
test. Independent source review accepted the Linux/POSIX runner. The reviewer did
not rerun the execution evidence below. Windows/macOS resource-control/runtime
acceptance remains pending.

- Linux reported31.1GiB MemTotal, about12GiB initially available, nearly full8GiB
  zram, and zero current memory PSI averages. The machine's64GB physical total
  must not be treated as Linux build headroom.
- No project/user/isolated-Cargo configuration was present in the inspected paths.
  Added project jobs=2 and a separate two-worker Rust test-harness default.
- Verified `.build-artifacts/tmp` and toolchain copies on `/dev/nvme1n1p3`, Btrfs
  `/home`; `/tmp` is tmpfs. Original cache/toolchain directories were retained.
- The copied compiler reports Rust1.99.1/Cargo1.99.1. Copy completion/version and
  executable-hash checks precede use; actual ordinary compilation/tests used it.
-44 Python tooling tests pass, including real competing-process lock refusal,
  disk/containment and flag checks, controlled sampler failure, fake-pressure
  cancellation, cleanup exceptions, an exited leader with an ignoring descendant,
  and abrupt wrapper death retaining a lock/marker. These tiny controlled processes
  create no real memory pressure and target no unrelated applications/agents.
- Ordinary all-target Clippy passed:35 one-second memory observations, minimum
  MemAvailable10.76GiB, maximum some/full PSI avg10=0, OOM-kill counter delta0.
- The necessary ordinary Rust test invocation passed591 tests. Actual command was
  `cargo test --jobs 2 --offline --all-targets -- --test-threads=2`, with both limits
  also present in child environment and project-disk TMPDIR/CARGO_TARGET_DIR.
  284 observations: minimum MemAvailable10.85GiB, some/full PSI avg10 maxima0,
  OOM-kill delta0. Swap used remained high,6.90→6.92GiB. Concurrent external
  workload changes were not attributed to this build or modified.

Logs, command records and memory JSONL are under ignored `.build-artifacts/logs`;
full exact-path inventories/cleanup records are under `.build-artifacts/audits`.
The ordinary Rust observation preceded the final abrupt-death/cleanup hardening;
those final paths are covered by the44 Python tests and independent source review.
No claim is made that a full Rust run exercised a real pressure incident.

The initial snapshot found1913 DLMS-named immediate `/tmp` entries totaling
12.051GiB allocated (a sum of per-entry disk usage, not a physical-RAM guarantee).
After the owner approved the exact seven-file proposal, fresh cwd/exe/open-file/
mapped-file checks found no active references or unreadable process entries.
Only those seven old ELF review executables were removed:1,513,992,192 allocated
bytes (1.410GiB). Matching Rust sources, diagnostics, original toolchains, worktrees,
privacy-review evidence, frozen evaluator6 baselines, staged lessons and installed
acceptance fixtures remain. No broad deletion or cargo clean was performed.

Future pressure refusal/cleanup is a conservative guard, not an OOM guarantee.
No GPU reservation, swap, OOM policy, LM Studio/model/monitor, service, global shell
or mount setting was changed. The packaging guard and source tests are verified;
a newly packaged artifact using this guard remains part of the next clean
application milestone, not an already completed installation acceptance claim.

## M67 ordinary test/native observations

The necessary132-overlapping-check combined run recorded277samples with minimum11.96GiB available, PSIavg10some/full maxima0 and OOM-counter delta0. The following41-sample native flow had minimum12.51GiB available, peak some/full PSIavg100.32% and OOMdelta0; swap free fell to4096bytes. These serialized2-Cargo-job/2-test-worker observations are not a stress test or memory guarantee. No GPU/swap/OOM/model/desktop setting changed. Ignored exact audit: `.build-artifacts/audits/m67-memory-r1.json`. Later full-unit/final runs retain separate command/memory logs.


M68 main focused manager/terminal integration:37 ordinary samples, minimum9.68GiB MemAvailable, peak some/full PSIavg10 0.18%, observed oom_kill counter delta0. Shared runner/Cargo2/test2/project disk retained. Nearly full swap remains a risk; no safety guarantee or stress test. Exact ignored command and sample logs: m68-manager-terminal-main-r1.log and20261024T011452.295379Z-test-memory.jsonl.


M68 clean Fedora candidate:66samples minimum11.49GiB available,PSIavg10zero,OOMdelta0. Actual installed OSPF/alignment/quiet runs:307/69/10samples, minimum12.28/12.60/12.59GiB, peakPSI0.90/0/0%,OOMdelta0. Swap reached0free bytes during OSPF, with no sustained guard threshold. No other workloads/settings were changed or interrupted. Evidence .build-artifacts/audits/m68-memory-r1.json, disk-backed native/install/log directories. All runs serial under the2job/2test-worker shared guard; not a guarantee or stress workload.


M69 merged main ordinary broad compile/UI interval363samples: minimum9.58GiB available, peak some/full PSIavg10 0.72%,OOMdelta0. Final actual private manager native198samples: minimum12.66GiB,PSIzero/OOMdelta0. Swap reached0free bytes. Runs remain serial main-guarded2jobs/2test workers; project disk staging/evidence, no stress/system changes or guarantee. Detailed per-run ignored audit m69-memory-r1.json.
