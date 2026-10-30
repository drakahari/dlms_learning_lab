# Fixed GNU grep / simulated shell development comparison

This is a development fixture, never a learner execution engine. The application
uses its bounded in-process interpreter and virtual files. It does not run GNU
commands, Bash, host scripts or subprocesses from learner input.

## Scope and evidence status

The exact 24 recipes in `tests/fixtures/shell-ere-reference.json` compare the
actual evaluator14 `grep -E` utility with direct GNU grep under `LC_ALL=C` and
`LANG=C`. Comparisons include stdout, stderr, integer exit status and complete
unchanged file contents, against both independent predictions and each other.
The inputs cover grouped/numbered/inverted/quiet matching, flag order, empty
patterns/input/lines, final LF handling, UTF-8 byte behavior, ordinary CR and
Unicode separators, classes, repeats, nullable loops, literal punctuation,
finite stdin, dash-leading names and read-only virtual file inputs.

**This library-probe repair and its three new synthetic guard groups are
source-only and UNRUN.** Original18 guards have separate RootM517 evidence;
probe build, prepare, Root pin approval, actual24 comparisons and sealed
verification remain pending. No reference observations are inferred from guards.
`observed`, `host_execution` and `source_identity` remain null in the fixed
fixture. Source review, synthetic guards and the earlier pure ERE34 comparison
are not utility14 observations. No curriculum, platform, complete GNU/POSIX/Bash
or novice acceptance is implied.

The six `vm_only` items are reason metadata, not six executable test fixtures.
They have no GNU observations and earn no invented pass counts. In particular,
the work-limit entry does not contain the large input needed for a resource
exercise. They explain deliberately different diagnostics, unsupported grammar,
resource/Stop limits, finite-snapshot stdin consumption, virtual missing-file
wording and non-C selector restrictions. No excluded source is passed to GNU.

## Probe and source closure

`examples/shell_ere_reference.rs` imports the actual `dlms_learning_lab::shell`
library APIs. It does not path-load, copy, stub or rearrange the shell engine.
It has only fixed outcomes or `--identity`; it accepts no source, filename,
pattern, stdin, fixture, worktree or engine selector. The exact24 case runner and
its seven-instruction slices remain unchanged.

At the observed r1 source revision, its compiled identity embeds a literal396-file conservative superset:
all282 `src` files at that revision,50 scenario files and59 assets, plus Cargo.toml/lock,
probe, digest helper and fixed fixture. Some included UI/test source is unused
by this library example; its presence is provenance, not UI execution. The real
library's scenario/theme data and non-test developer_digest are included. There
is no local build.rs. This is closed project-input evidence, not complete
third-party dependency, build-environment or toolchain reproducibility.

The collector pins and retains the same396 names plus13 existing transport/
review inputs (409 sources total). Prepare/capture checks the current fixed
source-family roster; unlisted, missing or aliased inputs require a new review.
Retained verification never scans later current modules or reopens current docs
or the current fixture: it validates the fixed fixture and every source from the
approved retained snapshot. Future source/docs changes cannot rewrite that old
compiled identity. Core18 source pins, including the reviewed single-leading-./
grep fix, remain strict; no historical source afterimage is restored to pass.

The previous23-file direct-module claim was incomplete and failed compilation.
This source repair is still uncompiled/unrun; actual build identity and all24
reference outcomes remain pending Root gates and exact approval.

## Fixed transport and retention

`tools/shell_ere_reference.py` reuses the existing reviewed execution, inherited
lock, bounded stream capture, kill/reap, private files, budget, held executable,
GNU version, approval and publication helpers. It does not create another
subprocess runner. All imported local helpers are pinned and retained.

Only the exact `/usr/bin/grep` held descriptor and fixed row argv are executed.
No shell is involved. Three file-input recipes create at most two small regular
files in their own fresh disk-backed private directories. Names are fixed leaf
names; dash and dash-leading names are ordinary data. Traversal, extra entries,
symlinks, hard aliases and unexpected mutation fail. No personal files, real
network targets, actual service/device changes or learner-selected paths exist.

GNU timeout is two seconds per child; probe timeout is 15 seconds; whole capture
is 120 seconds. Separate stdout/stderr bounds are 16 KiB per GNU stream and
512 KiB per probe stream. Total retained evidence is bounded at 512 MiB, with
256 MiB maximum ELF size. These are tiny fixed comparisons, not a stress test.
The old256-entry evidence-directory premise is replaced by a bound derived
from the exact expected files plus allowed directories, so all409 retained
sources fit. This entry-count bound changes; byte/ELF/output/memory/process/
deadline and directory-depth bounds do not increase.
The MAIN runner's memory/PSI guard remains mandatory; there is no claim that a
Cargo job limit or these output bounds guarantee free desktop memory.

Failed raw streams and unknown timeout status are preserved without COMPLETE.
The exact held probe/GNU bytes, source files, compiler identities, approved
control, raw streams, RESULT/STATUS, file contents and ordered comparisons are
retained. Verification launches nothing and recomputes source/executable hashes,
complete evidence inventory, literal approval, raw/RESULT/STATUS agreement,
actual compiled identity, GNU held-FD version, all 24 full tuples and predictions.
It does not trust a self-reported `accepted` flag.

## Ordered actual gate, after source review and integration

Use only the MAIN `python3 tools/dev.py` runner; all build/test/reference work
shares one lock, Cargo maximum2 and test maximum2. The tool has fixed modes and
capture ID `shell-ere24-r1`, no arbitrary paths or capture selectors.

1. Preserve the sealed genuine13-before14 capture and all older evidence.
2. Compose accepted capability14 and all actual13 migration/current consumers.
   Run the actual core/resource/Stop, full inclusive Rust, Python and Clippy gates.
3. Build the fixed development example through the runner.
4. Run the fixed tool in `--mode prepare`; it launches zero children and writes
   root_reviewed:false pins. Root independently reviews the actual source,
   compiler, executable and inherited shared-runner context. No automatic
   approval is created. Root provides the exact `-APPROVED.json` with only
   root_reviewed changed to true.
5. Run `--mode capture` through the runner. All 24 matches are required; mismatch
   or limit failures retain diagnostics. Prior capture IDs are never overwritten.
6. Independently read the actual raw observations, then run `--mode verify`
   through the runner. Do not manufacture output files to meet a gate.

No code in this source proposal has run these steps. The separate command above
is not permission to bypass the lock, source review or Root pin approval.

## Separate VM-only acceptance obligations

Core/resource/Stop and compatibility gates are separate from this comparator.
RootM509/M512 report4 ownership,18 pure,16 utility and3 closed-subset groups
passed; this transport repair does not rerun or expand those observations.
All existing shell/history regressions remain required too. Relevant exact tests:

- `cooperative_cursor_matches_all_fixed_supported_cases_across_tiny_slices`
- `cooperative_owned_snapshot_utf8_ranges_and_shared_limits_fail_without_false_matches`
- `syntax_error_returns_two_and_script_can_handle_it_but_unsupported_syntax_cannot_pass`
- `missing_file_and_prior_acquisition_error_are_not_hidden_by_quiet_or_empty_pattern`
- `unsupported_option_and_pattern_forms_fail_instead_of_guessing`
- `single_virtual_file_dash_paths_and_finite_stdin_are_defined_without_host_access`
- `fixed_locale_rejects_unmodeled_settings_and_lc_all_has_precedence`
- `stop_drops_large_pending_match_and_prevents_following_publication`
- `global_regex_budget_cannot_be_reset_by_repeated_calls_or_command_substitution`
- `output_line_and_pattern_bounds_fail_honestly_with_no_false_completion`
- `extended_selection_completion_keeps_reserved_output_charged_through_emission`
- `stop_reaches_real_pending_nfa_after_framing_and_validation_without_later_publication`

These names identify real source tests, not passed results. Resource ownership,
Stop responsiveness and global limits across pipeline/substitution/function
contexts must be verified separately; GNU matching does not prove them.

## Authoritative behavior references

GNU grep manual: <https://www.gnu.org/software/grep/manual/grep.html> (pattern
syntax, matching control, output prefixes, exit status, character encoding).
The actual development gate records the installed `/usr/bin/grep --version`
through its held descriptor and pins `/usr/share/man/man1/grep.1.gz`; a URL or
installed manual alone is not an observed comparison. The earlier `ERE_SUBSET.md`
and `SHELL_SUBSET.md` disclose simulator-specific limitations. No GNU code is
copied, and this is not a GNU endorsement.


The integrated cooperative cursor remains the same named struct exposed through
a transparent public type alias. This avoids an unused re-export warning when
older standalone pure examples compile the module; those producer/helper sources
and retained observations are unchanged. The collector pins that current source
explicitly. No matcher, ownership, work or storage semantics change with the alias.


## Actual capability14 comparison acceptance —2026-10-30

After alltargetClippy,445Python and fullinclusive2,839tests/0fail/0ignored,
M523 builds the actual-library probe. Zero-child preparation and two independent
exact source/ELF/control reviews precede Root's flag-only approval. M525 observes
VM14 identity and396 compiled input hashes before24GNU child comparisons.
All24 full stdout/stderr/status/file tuples match:19status0 and5expected no-match
status1; allfive supplied fileinstances remain exact. Six documented reason-only
exclusions were not executed and earn no pass count.

Manifest `276c9982c94d40d423d8791ae4c358ebc3e324d9ffed2e93f5830f00971980d7`;
M526 retained verification and independent raw/source/ELF reviewdb079bcb pass.
Private evidence retains409source inputs,21,934,272B ELF3435cdb6 and literal
Rootapproval783ee820;553files/55dirs total36,643,252B. No observations are copied
into static prediction fixtures or manufactured from an accepted flag. Actual
source/streams/profiles remain ignored. This is bounded utility fidelity evidence,
not fullGNU/Bash conformance, curriculum193, native/platform/novice acceptance
or an observed evaluator14-before15 migration baseline. Normal193/native content
gates follow; this technical comparison is not a reason to stop implementation.


## Later native QA source closure

The normal193 native acceptance route adds exactly two later QA modules:
`src/ui/smoke/ere_evidence.rs` and `src/ui/tests/ere_evidence_native.rs`.
They were not compiled input bytes observed by the sealed r1 ELF. Current Python
source-closure checks account for them as an explicit, disjoint two-file union
with the original396 names, verifying both complete file sizes and SHA256 plus
ordinary single-link files. This preserves a closed current roster without
claiming398 inputs were compared with GNU or rewriting captured409-source data.

The r1 collector, probe, COMPILED396, SOURCES409, approved control, ELF, raw results
and retained snapshots remain unchanged. Its fresh prepare/capture source-roster
guard must refuse either new QA file; a synthetic guard exercises that refusal
before any child or control destination is created. Retained r1 verification
continues using its exact retained snapshot, independent of later current source,
unit registration, docs or QA files. Any future fresh comparison requires a new
separately reviewed source closure, compiled probe and Root-approved controls.
This native QA addition grants no new reference execution or fidelity claim.
