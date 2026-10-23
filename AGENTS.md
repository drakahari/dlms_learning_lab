# DLMS Learning Lab development constraints

Work only on this project. Preserve the approved native Rust/egui, offline,
simulation-only application and existing progress. No learner-host execution,
old-project edits, desktop/GPU/swap/service/model changes or public release.

## Build and test memory safety — mandatory for every agent/worktree

- Read `docs/BUILD_WORKFLOW.md` before starting a build/test/reference/native run.
- Use the MAIN checkout's `python3 tools/dev.py` runner, including for isolated
  worktrees. It owns one shared OS lock across this repository's worktrees.
  Never run a second memory-heavy build/test invocation alongside it. A busy
  lock means retry later; do not bypass it or interrupt another agent/application.
- Cargo maximum is **2 jobs**. Rust test maximum is **2 runtime threads**;
  Cargo jobs do not limit test parallelism. Do not override either upward.
- Future large build/cache/package/test data belongs on the verified disk volume:
  `.build-artifacts/`, `target/`, or `dist/` in the main checkout. Never use RAM
  `/tmp` for new large artifacts. The runner supplies private disk TMPDIR/TMP/TEMP
  and the copied local toolchain; do not change global shell or mount settings.
- The runner records available memory, swap and PSI. It refuses low headroom or
  pressure and stops only its own child on sustained pressure. Stop launching
  additional work and report pressure; two jobs are not a memory guarantee.
- Do not delete existing DLMS `/tmp` artifacts, diagnostics, baselines, drafts or
  installers without the owner's approval of exact paths. No wildcard cleanup
  or indiscriminate `cargo clean`. Preserve uncertain and pending evidence.
- Generated caches, measurements and private evidence stay ignored. Do not add
  credentials, progress, recordings or screenshots to public commits.

Ordinary implementation milestones remain authorized. Keep CHECKPOINT.md and
coverage evidence honest; packaging/test counts alone do not complete curriculum
or Windows/macOS, accessibility or novice acceptance.
