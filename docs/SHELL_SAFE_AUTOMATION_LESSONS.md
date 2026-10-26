# Reliable text processing: three scripting assignments

2026-10-26. This guide describes the current three-lab `shell-safe-automation`
unit. Six content groups and seven actual egui controller groups have passed under
the guarded runner, including the corrected Stop acknowledgement. All three development-native Fedora flows passed72 captures before final
wording/capture polish; final same-ELF installed acceptance passed at ddb886b with
all three flows/72 captures, including real release Run/Stop. The retained d781
Stop-capture failure is not counted as acceptance. These checks do not establish full safe automation,
novice or accessibility acceptance, other operating systems, or real filesystem
safety.

The authored pack is `scenarios/shell-safe-automation-v1.json`, SHA256
`675a81721bdf3225e31c1074c799292236ee7012e73e3e1b133d28f95848c147`.
Its assignments contain 10, 10 and 12 behavioral input sets respectively. Grading
checks complete output, error output, exit status and simulated file contents;
it does not require a particular command string or method. Every input set starts
with its own sample files. Editing the script invalidates its current result.

## Keep every label while reading text

`shell-preserve-lines` builds on `shell-number-records` and `shell-eof`. The task
numbers every source record as `NUMBER: [TEXT]`, keeping meaningful edge spaces,
tabs, backslashes, percent signs, carriage returns and UTF-8 text. Blank records
count, an empty file has none, and a final nonempty record without a newline must
still appear. Bad argument count or a missing source produces the specified
usage error without changing files.

The introduction distinguishes a complete blank line from EOF. Ordinary
`read -r line` preserves backslashes but trims default-IFS edge spaces and tabs.
`IFS= read -r line` preserves those edges for that one read. A condition such as
`while IFS= read -r line || test -n "$line"` includes the final partial record;
quoted values and a fixed `printf` format keep the text out of formatting syntax.
The reviewed alternate uses literal `IFS=''` and a differently named counter.

The ten inputs exercise ordinary labels, edge spaces/tabs, backslashes/percent,
blank records, a final partial record, an empty file, UTF-8/carriage return,
missing and extra operands, and a missing source. Separate practice uses a
`Usage: practice SOURCE` interface and different sample text. Remove `IFS=` to
observe trimming; restore it, then remove the final-record condition to observe
the lost partial line. Neither experiment edits the assignment draft.

## Do not publish a pipeline that failed

`shell-pipeline-status` follows the first task, `shell-capture-decision` and
`shell-uniq-counts`. It selects literal uppercase `ERROR` records, sorts whole
lines in C-byte order, removes duplicates and publishes `errors.txt`. A valid
source with no matches publishes an empty report. A missing source must preserve
the previous report and every other file, report the specified error, and return
2. Supplied source filenames are distinct from `errors.txt` and `keep.txt`.

The introduction teaches that the last successful stage can hide an earlier
failure. Explicit `set -o pipefail` reports the rightmost nonzero stage status;
a new Run starts with the option off. In this subset, `grep -F` status 1 means
no matches and status 2 means a file error. Save `$?` immediately. Build and check
a candidate before opening the output: `>` can truncate before later validation.
Capture removes trailing newlines, so publication restores the required newline
only for nonempty captured records.

An equivalent reviewed solution checks `grep` separately, then sorts and deduplicates
only after that producer check. The ten inputs include duplicate records, no
match, an empty file, a spaced filename, a match inside a record, UTF-8 byte order,
a missing source, bad counts and matching records with meaningful edge whitespace.
Practice uses `WARN` and `warnings.txt`. Point its grep at `missing.log`, then
remove pipefail to see why a successful final stage can conceal the missing file.
Restore the working example before the full assignment.

## Publish a validated text handover

`shell-safe-handover` follows `shell-pipeline-status`, `shell-case-publication`
and `shell-local-heading`. It accepts exactly two nonempty flat basenames,
rejects slash paths, `.` and `..`, requires an existing source and forbids writing
to the source itself or `keep.txt`. After validation it writes `Entry: <TEXT>`
records and `Total: N`, preserving every other file. Invalid input changes no
file and returns the specified usage error.

The introduction explains capture-local counters, validation before redirection,
and restoring the capture's final newline. The reviewed alternate validates first,
then writes entries incrementally with append redirection and adds the total.
Both methods meet these finite supplied outcomes. The assignment does not claim
crash-safe replacement or concurrent-write protection.

The twelve inputs cover a complete handover, formatting characters/spaced names,
a final partial record, an empty source, blank/UTF-8 records, extra operands,
missing input, input/output collision, protected output, an empty output name,
and source/output traversal forms. The separate practice deliberately has a
different interface: `Notice: <TEXT>`, `Notices: N`, `Example published` and
`Usage: practice SOURCE OUTPUT`. The assignment retains `Entry`, `Total`,
`Published report` and its own usage message. Redirect the practice publication
to `keep.txt` to inspect the mismatch between validation and use; repair it, then
remove `IFS=` to inspect trimming. Practice results are ungraded and isolated.

## Reference and supported boundary

This is the evaluator10 in-process subset, with no host shell, files, subprocess,
or network. Only literal-empty prefixes directly on `read -r NAME` are admitted,
including literal empty quoted forms. A variable that expands to an empty IFS
value, a general environment prefix, or arbitrary IFS configuration is outside
this extension. The temporary read prefix does not alter later ordinary splitting.
Default `read` behavior remains available and unchanged.

Only explicit `set -o pipefail` and `set +o pipefail` are added here. Ordinary
functions share the current option; captures and pipeline stages inherit isolated
copies. A pipeline uses the setting at entry. There is no general `set`, errexit,
trap or export facility added by these lessons. Flat virtual UTF-8 files have
finite LF-delimited records; CR is text and NUL is excluded. Existing instruction,
output, source, capture and memory limits continue to apply, and Stop cancels
without retaining current success.

The semantic reference is the controlled GNU Bash5.3.9 evidence and the existing
installed GNU Bash5.3 manual. The pack links the GNU manual's Bash Builtins,
Pipelines and Command Substitution sections. The online GNU fetch timed out and
POSIX Issue8 pages returned403 on2026-10-26; no universal POSIX portability claim
is made. The accepted public9 observations and genuine older progress histories
remain historical evidence, not rewritten evaluator10 outcomes.

The native controller covers actual editor typing, Run/Stop, wrong and partial
results, independent alternatives, source invalidation, separate unfinished
practice/assignment drafts, warm save/load, favorites, reset Cancel/Confirm,
Challenge, Light150 and Dark200. Its geometry records actual egui dimensions and
scale; requested window sizes are not proof of physical window size. Actual
native renders and cold-process evidence are separate gates.
