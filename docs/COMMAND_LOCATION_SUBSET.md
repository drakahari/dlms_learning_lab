# Command-location diagnostic subset

Development capability gate, 2026-11-05. The new command-location lesson is registered in source288/91;
latest verified Fedora build remains287/90 in CHECKPOINT.md. There is no progress
schema or shell-evaluator change. The lesson supplies only partial104.7 coverage. This independent native-terminal capability does not change the
Script evaluator19 or make external utilities executable.

The pure module supports one supplied external name per literal query:
`which NAME` finds its first executable PATH match; `type -a -p NAME` finds all
eligible matches in supplied directory order. Both operate on the **current**
virtual file metadata, so a changed execute/search permission changes the result.
No file contents, scripts, ELF or host utilities are loaded or executed.

The context has1–4known ASCII names and1–16distinct canonical absolute PATH
components. Empty/relative/duplicate/default search, composition/expansion,
unsupported flags, plain type, aliases, functions, builtins, hash tables and
whereis are refused honestly. The specific type combination avoids pretending
that `-P` alone always performs a fresh scan. This is not a complete shell or
GNU-which implementation.

A caller-held authored context and original tree must match carried metadata;
caller, group roster and home remain bound. A structurally valid imported context
does not authorize itself. The core reads actual current permissions and reuses
the unchanged existing first-match resolver; all-results makes at most16bounded
single-directory calls. It projects empty content values directly from keys,
preserving original file bytes and avoiding body clones merely to discard them.
The source-bound Definition permits only modes on supplied caller-owned command
files to change. Whole-tree comparison preserves file contents, ownership, callers,
users, PATH, directories and unrelated entries. Goal validation, execution, grading
and catalog-aware progress save/load require the held original definition.
Assignment and example identities are `<lab-id>-assignment` and `<lab-id>-example`;
they have disjoint command names and PATHs. Example success never grants credit.
A resumed workspace missing this context is rejected without refilling its draft.

Source/current trees must be valid and have at most64nodes/files. Existing tree
limits on users, groups, file sizes and aggregate text remain enforced before
projection. Context input64KiB, query2KiB, output32KiB and diagnostic512bytes are
bounded. Found/Missing are typed results; missing is not unsupported or successful.
Path stdout and simulator diagnostic are separate. No fabricated Bash126/127 or
exact utility-stderr equivalence is promised. Returned data is not a persisted
completion certificate; verification recomputes from held source and live tree.

Primary references checked2026-11-05: installed GNU Bash Reference Manual5.3
(18May2025), [type builtin](https://www.gnu.org/software/bash/manual/html_node/Bash-Builtins.html),
and the installed which manual. The independent read receipt retains their exact
identities; fixed bounded host reference and model comparison now match18query outcomes;
see COMMAND_LOCATION_LESSON.md for retained exact identities and limits.
[LPI current LPIC-1v5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
still list command discovery at104.7. Whereis fidelity remains separate work;
this module cannot close the whole objective. All text and code are original.

Actual pure-core gates:16new query/source/permission/bounds groups and unchanged
filesystem12/path-lookup16 pass; focused Clippy20.62s passes. These are in-process
model checks, not real Bash/which comparisons, native learner interaction, source-
aware authored coverage. Seventeen new tree/engine groups and six separate
source/practice/save/load/reset/forgery groups also pass. All-target Clippy passes;
Python765 guards pass before the additional directory-link regression. A fixed
reference probe and retained host executables match18controlled query outcomes. Independent pre-execution review
caught an inappropriate assumption that empty directories always have two links;
the collector now retains original descriptor identity across readonly queries.
R25 actual new-lesson native54actions/8PNG pass, including separate practice,
feedback, reset, disk save/read and enlarged light/dark input. These are technical
checks, not novice or accessibility acceptance.
