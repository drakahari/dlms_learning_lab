# Text records and reliable audits

Authored2026-10-22 on the reviewed M17 utility capability. Four distinct assignments use portable shell syntax with an explicitly documented GNU text-utility subset. They do not require Bash local scope; unit prerequisites are the earlier stream/pipeline foundations.

| Assignment | Reasoning and practical result | Cases |
| --- | --- | --- |
| Extract an inventory view | Select existing asset/owner fields, preserve empty/space-containing values, skip note lines without a delimiter |6: normal, notes/empty selected fields, extra/missing fields with a different spaced filename, edge spaces/percent, unterminated record, empty input |
| Make a stable unique manifest | Preserve literal names while removing separated duplicates in deterministic whole-line text order |6: unsorted repeats, case/numeric-looking names, significant spaces, missing final newline, empty input, UTF-8 with a different filename |
| Count locations without losing repeats | Extract locations before sorting/grouping; preserve duplicate evidence and GNU count formatting |6: nonadjacent locations, one group, singleton groups, empty locations, case/space distinction with a spaced filename, empty input |
| Audit duplicate asset identities | Detect repeated IDs in different rows, publish a report after validation and preserve inputs/old output on failure |8: nonadjacent duplicates, all-singletons, extra argument with valid input, spaced IDs, identical input/output, missing source with old/absent output, empty output-name argument |

Every lesson supplies skippable prerequisite teaching, a separate editable example with different data and a tested suggested experiment. Learn and Challenge keep the same complete assignment/editor and Run/Stop controls. Help and worked reasoning remain optional. A count is an observed record count, not a reason to sort the finished count prefixes or remove duplicates first.

Cases vary supplied filenames as well as text, defeating programs that ignore the file argument. Tests compare resulting output, status, errors and every virtual file; alternate supported programs pass. Negative checks include plausible partial pipelines, hardcoded first-case output/filename, premature output creation and omitted argument validation. An empty-source heading-only audit is separately verified. Existing attempt IDs/versions and prior credit remain intact; reset affects only the selected attempt and both of its drafts.

Fidelity limits: fixed C/POSIX byte ordering, ASCII field delimiters, LF/UTF-8 text, one virtual input or finite stdin, and bounded documented flags. These are simple delimiter-separated records, **not CSV with quoting/escaping**. Supplied canonical single filenames have no directory parts/leading hyphen; textual equality does not establish general real-filesystem alias safety. No numeric/key/locale sorting, regex, awk/sed or host utility execution is claimed. See [SHELL_SUBSET.md](SHELL_SUBSET.md).

The official GNU cut/sort/uniq manual sections and current LPIC-1 5.0 objective103.2 were checked2026-10-22; exact URLs are bundled in scenarios and the objective map. GNU direct-page requests timed out; installed original coreutils manual and controlled disposable GNU comparisons supplied the technical verification. This is narrow partial103.2 practice inside scripting workspaces, not full LPIC filter coverage or Linux-terminal tool integration.
