# Current course metadata loading

The current279course path used a chain of historical snapshot helpers. Each step loaded and fully validated an overlapping scenario catalog just to obtain unit labels and placement. It repeated authored solution execution during navigation setup. No learner script ran on the host, but the duplicated in-process validation delayed first rendering.

The current path now composes the same bundled unit fragments as metadata, then invokes existing `course::load` once against the caller's already validated scenarios. `pack::ccna_pilot` still performs full authored scenario validation. All historical public accessors, the pack loader, Unit/Index wire schemas and the final unit/caller validator are unchanged. There is no cache keyed incompletely to a caller and no progress migration or shell revision change.

An actual regression executes the original current function body (preserved byte-for-byte as a test-only legacy oracle), then compares exact serialized bytes for all84Unit values/order and the raw Index. It rejects empty, truncated, renamed, extra, duplicate, wrong-track and wrong-unit callers after a successful call; valid reversed caller ordering still works. A changed caller title remains accepted because unit membership validation never assessed unrelated teaching/Goal data. Every current call validates its own supplied catalog.

Observed on this Fedora desktop: old course construction75.119090573seconds; new construction1.853851milliseconds, using the same already loaded279scenario vector. The complete test, including initial authored loading and the old-path observation, passes in78.71seconds. These are unit-graph timings, not claimed cold application launch times or cross-platform benchmarks. Native/candidate launch validation follows separately.

No unit values, ordering, prerequisites, objectives, grading or draft/progress formats changed. If multiple bundled unit fragments become malformed simultaneously, the first diagnostic ordering can differ because redundant prefix validation is omitted; every current index still receives full schema, identity, coverage, track/unit, duplicate and cycle checks through the unchanged final validator.

For future curriculum batches, extend the appropriate metadata-only composition and scenario registration together. Keep earlier technical snapshots stable, use the existing final validator and add preservation tests. Do not reintroduce pack/VM execution merely to compose Unit metadata. The independently retained source/proposal and actual regression log live privately under associative-stage/course-performance-proposal and course-metadata-r1.log.

## Bounded catalog growth beyond100 units —2026-11-06

The named-driver handover exposed the original100-unit admission limit (NamesR17
failed at101). The runtime now admits at most256units; the independent256,000-byte
JSON limit,80-byte IDs,160-byte titles,1000-byte summaries,200labs/unit,8prerequisites,
unique ownership and iterative cycle checks remain unchanged. A clearly synthetic
metadata test accepts exactly256, rejects257, and retains byte/per-unit/cycle
negative checks. This is parser capacity, not new curriculum or a target lab count.
No old scenario/unit object, learner progress requirement or simulator bound is
changed. Current separate101st unit preserves the previous100objects/order exactly.
