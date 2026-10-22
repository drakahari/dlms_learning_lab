# Expansion resource contract (M26)

Specified2026-10-22 before implementation. No new shell syntax, authored assignment, host capability or dependency. This tightens the existing in-process interpreter’s resource boundary before future globbing/substitution work.

## Finding and intended correction

The prior shared expansion layer bounded individual parameter text but concatenated word parts before checking the final argument list. Many individually legal quoted/bare/escaped parts could therefore build a large temporary field. Whitespace splitting also accumulated fields before the128-argument check. Direct assignment/local/redirection/while-input consumers call this layer separately and need the same guards. A valid source-length/instruction count alone does not bound character work inside one expansion.

Enforce before allocation/append:

- Each expanded word retains at most128 fields and256KiB of combined field text, including UTF-8 bytes. The complete ordinary command has the same count/byte bound. Empty quoted arguments remain real arguments; ordinary unquoted whitespace still splits/disappears correctly.
- Every expanded-text append, including literal/arithmetic output, respects256KiB. Parameter retrieval charges work before cloning. Sole quoted "$@" observes the same count/byte/work limits without changing empty-argument behavior.
- One VM run has a cumulative1,000,000-byte expansion-work budget. Charge source scans, parameter copies, expanded-text appends and field processing before performing that work. Nested arithmetic lookups, functions, pipeline preflight/stages, loops and later ticks share it. A new VM gets a new budget. This is a simulator work limit, not a Bash semantic limit or a wall-clock guarantee.
- Resource exhaustion is an honest focused limit diagnostic, never success or an unsupported-syntax fiction. A failure before redirection does not truncate a virtual destination. Existing virtual-state/output/instruction/parser/call/input-frame limits remain in force. Bounded temporary stage buffers are separate from the virtual-state budget; no claim of exact whole-process RSS is made.

Advance evaluator revision4 because stricter resource limits can reject some previously accepted runs. Preserve actual earlier source/output/files/credit as history; do not turn it into current success or discard it. First upgraded writes retain exact original bytes through the existing non-overwriting revision-specific backup; loading never rewrites the original.

## Development evidence and verification requirements

Independent controlled VM probe on revision3, source85d0274:1000 alternating quoted8KiB values assembled an8,192,000-byte field before the argument error. Reported peak RSS was11,140KiB versus3,004KiB for one part. This is one synthetic process measurement, not a universal timing/memory promise.

A clean revision3 probe was retained locally before changes (SHA-256 a110a55e1ee90e2e0a8bc9703eb172fee3998236de11ec2b74fe5e75f9fddf80). Three fixed whitespace/empty-quote fixtures (32/64/96KiB supplied text) earned the declared32 empty lines under that actual prior VM and matched Bash5.3.9 in disposable development directories. They establish honest prior credit that a tighter work budget must retain historically. They are development fixtures, not curriculum or learner recordings. Reference Bash never runs learner/imported input and never becomes the app’s execution engine.

Regressions must exercise multipart quoted/bare/escaped text, early field/count/byte bounds, aggregate command arguments, direct assignment/local/redirect/while-input consumers, sole quoted "$@", UTF-8 accounting, cumulative function/pipeline/loop/tick work, fresh-VM isolation, Stop, migration/reset/export and exact original-byte backup. Re-run the supported small Bash fixture corpus and existing authored methods/alternatives. Inspect native limit feedback/current-history/retry behavior; no new coverage is earned from this capability correction.

Reference: GNU Bash manual, Shell Expansions / Word Splitting / Double Quotes / Special Parameters, https://www.gnu.org/software/bash/manual/bash.html . Runtime shell resource budgets are this simulator’s disclosed policy, not attributed to Bash.


GNU web node fetches were unavailable during this audit (timeout/status error). Checked the corresponding official installed `/usr/share/info/bash.info.gz` Word Splitting and Special Parameters nodes instead, alongside disposable Bash5.3.9 comparisons. The `info` reader program was absent; the existing gzip/Python reader was sufficient, with no system package change. No unverified web content or secondary source is substituted.

## Completed evidence

244 Rust tests pass, including five new expansion regressions; formatter and Clippy pass. Independent focused review ran37 model/migration/feedback tests, all524 controlled Bash5.3.9 comparisons with zero differences, and the synthetic prior/current allocation probe. A repeated process measurement was10,992KiB prior versus3,848KiB current; the variation reinforces that these are observations, not an RSS contract. Final independent feedback/expansion rerun passed9 tests with no material finding.

Actual Fedora native XWayland flow used a separate synthetic /tmp profile and actual keyboard/editor/Run/Stop controls: retained actual revision3 results, current limit feedback on all three cases, exact pre-upgrade backup, loop cancellation, repair/current acceptance, save/reload, edit invalidation and historical credit. Reviewed desktop dark and resized light/dark150% scrolling. The first Stop harness check clicked before the newly enabled control entered egui’s previous-frame hit map; a settled frame fixed the test timing, with no production Stop behavior change. No novice, screen-reader, default-Wayland startup resolution or Windows/macOS desktop acceptance is claimed. Local ignored screenshots are `screenshots/workspaces/expansion-*.png`.
