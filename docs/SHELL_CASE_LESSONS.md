# Case dispatch lessons — evaluator8 and Fedora native checks accepted

Original source and curriculum text,2026-10-23; acceptance updated2026-10-24.
Original authorship follows the project's existing licensing boundary; no new
licensing decision is asserted.

Three tasks build distinct usable scripts:

| Task | Practical behavior | Distinct evidence |
| --- | --- | --- |
| Report-mode interface | Exact help/card modes, argument/error contract and two read-only views |11 cases: brief/full, intact text, both help aliases, spaced unknown mode, missing/empty/extra arguments and help arity |
| Prioritized work routing | Ordered full-record routes, output replacement/append, continued error audit and counts |8 cases: overlapping routes, several rejects, held/blanks, empty input, literal space/percent/wildcard/backslash/colon data, EOF tail and repeated records |
| Validated release publication | Whole-plan validation; read-only preview; exact replacement plus appended ledger after success |12 cases: preview/apply, empty/all-held plan, invalid record after valid work, unknown prefix, missing input, mode/argument guards and wildcard-looking filename/unterminated tail |

Concepts and a working separate example precede each full assignment. An example
edit changes one concrete thing and has a complete independent expected outcome
and repair. Examples use museum records/modes/files different from assignment
fixtures. Default/error behavior and byte/quoting limits are taught before they
are assessed. Optional hints progress from design to concrete methods; there is
no mandatory exact source string or case keyword requirement. Supported if/loop
solutions and different valid function/capture arrangements are accepted by
complete output/status/final-file behavior.

The portable core is first-match case/esac, | alternatives and ;;. Quoted wildcard
bytes are literal, the selector stays a whole scalar, and explicit default arms
handle unknown inputs. Simple * and ? use C/POSIX bytes, not Unicode character
classes. No bracket patterns, extglob, fallthrough, locale emulation or full-shell
claim is taught. Commands inside arms keep the documented existing subset.
Task-specific path identity is limited to supplied canonical flat virtual names;
no host path, real filesystem alias or crash-atomic publication claim is made.

The first task's assessment fields are intact values, including multiline
SUMMARY. The second routes the whole text, does not remove prefixes, counts
blank lines as ignored, continues after rejected records and clears old route
files even for empty input. The final task rejects the first bad record before
exposing any partial plan; empty successful apply still replaces the publication
and appends a source line. Its no-final-LF file rule follows prior command-capture
teaching and is explicit in the task, rather than a hidden formatting trick.

Sources and checks: installed original GNU Bash reference Edition5.3, updated
2025-05-18, rehashed/read2026-10-23 (SHA256
b91026acffb159bd1e842e3116a95539bf858e2b57cbfa74df9e3a53379177a4).
Official Conditional Constructs/Pattern Matching/Command Substitution URLs are
listed per lesson. Fresh GNU web fetches timed out and POSIX Issue8 shell-page
fetch returned403; successful live retrieval is not invented. The separately
retained actual Bash5.3.9 C-locale fixed37 observations justify the capability
contract, not these new authored program outcomes. No implementation/manual text
was copied into the original teaching. GNU cut is used only by optional behavior
alternatives within its already accepted delimiter/field subset.

First isolated checks on accepted evaluator8 passed:8 authored content tests and
18 affected teaching/capture/workspace regressions,26 total, plus all-target
Clippy with warnings denied and scoped formatting. These include372 reference/
alternate observations at four tick slices and36 separate example/change/repair
observations; complete output/status/file expectations are independently authored.
All106 prior activity identities,37 real current script records, unfinished
assignment/example drafts and favorites are preserved by the content test. Actual
6/7 history migration remains the separately accepted evaluator8 suite.

After normal registration in the isolated branch,818 distinct library/integration
tests and122 binary/UI tests pass:940 total, including all109-activity layout
matrices. Clippy with warnings denied, changed-source formatting and the build
pass.
The built executable validates109 activities/26 units (31CCNA/21LPIC-1/17LPIC-2/
40Bash). Frozen102/106 identity fixtures and genuine6/7 observations remain intact.
The actual Fedora native three-lesson flow passed on2026-10-24 with45 retained
captures,15 per lesson, at the window manager's actual1268×632-pixel window size. It covered
introduction, separate example without execution on opening, example Run/change/
repair, full assignment, wrong and partial feedback, reference and equivalent
conditional/loop solutions, edit invalidation, same-process exact save/load,
selected reset and preserved other attempts/favorites. Challenge typing and Run
passed in Light150% and Dark200%; these were actual input events and rendered
outputs, rather than only headless layout checks. Render review covered all three
introductions, the interface and publication assignments, each wrong/partial
feedback view and the larger-scale completed outputs. Detailed mismatch/advice
was readable in the output panel; additional result rows remain scrollable.

The first native attempt failed before captures because the smoke geometry helper
read egui zoom while holding its data lock. Moving the zoom read before the data
closure fixed the recursive lock; the rebuilt second attempt completed normally.
The failed attempt and successful build/run/captures are retained privately. This
capture-helper correction changed no learner layout, script or grading outcome.
The940 distinct automated tests remain the stated total; build, native checks and
repeated plan checks are separate evidence, not extra distinct tests.

No cold application restart, novice, screen-reader or other-platform acceptance
is claimed. Save/load above occurs within the same running app. This unit
reinforces bounded LPIC scripting behavior; no certification objective promotion,
full-shell or whole-objective completion claim follows from these checks.

## Record input format

The routing and publication inputs use canonical records: no leading/trailing
space or tab, with empty records permitted. Default-IFS `read -r` trims those
edges but retains backslashes and internal spaces. This matches existing
`shell-input`, `shell-eof` and `shell-number-records` instruction. The percent
publication fixture uses `ready:100% ready`, with an internal space. Task1 direct
arguments/environment/raw file output are not restricted. A metadata-driven
author-data guard covers assignment inputs and separate example/experiment
inputs. Custom-IFS whitespace-preserving input is deferred. These lessons use the existing
read behavior; they introduce no new input-expansion capability.

Visible prerequisites use actual lab titles; machine IDs are unchanged. Each test resets its simulated files and retains the script draft.
Read-based assignments and examples explicitly guarantee that the supplied source
filename differs from their output filenames. Review metadata pins those output
names, and the tested author-data guard rejects assignment/example source-output
collisions. This guarantees the stated source preservation without adding an
alias/filesystem claim or changing programs, cases or expected outcomes.
