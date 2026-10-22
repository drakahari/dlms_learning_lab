# Shell subset, version 1

Defined before the introductory unit (checked 2026-10-21). This is an in-process educational interpreter, **not Bash**. No shell/process, host file or network API exists in the interpreter. Only virtual UTF-8 text files in a flat `/work` namespace exist; directories and nested paths are unsupported. Linux terminal labs retain their separate documented command models.

## Accepted language

- Scripts use LF line breaks. Commands separated by newline or `;`, comments at a word boundary, backslash escaping/line continuation, literal single quotes and expanding double quotes.
- Named scalar assignments (assignment-only commands); `$name`, `${name}`, `$1`…`$9`, `$#`, `$?`. Undefined parameters expand to empty. Unquoted expansion uses default space/tab/newline field splitting. Sole `"$@"` expands each positional argument separately, including empty arguments. Other uses of `$@` are rejected.
- `|`, `&&`, `||`; pipeline status is the last stage. Pipeline variable changes are isolated; virtual file writes persist. No pipefail/set -e.
- `if …; then …; [else …;] fi`, `for name in words; do …; done`, `while …; do …; done`, nesting within the limits below.
- Text redirects `<`, `>`, `>>`, `2>`, `2>>`, `>&2`, `1>&2`, `2>&1`, evaluated left to right. Redirected paths must expand to exactly one word. Compound-command redirection is not supported.
- Builtins: `printf` with literal `%s`, `%%`, `\n`, `\t`, `\\` formats (format repeats for excess arguments); `echo` without flags or backslashes; `cat PATH`; `grep -F LITERAL [PATH]` (literal lines, status 0/1/2); `read -r NAME` (one variable, default whitespace trimming, EOF status); `test`/`[` with `-n`, `-z`, `-f`, `=`, `!=`, `-eq`, `-ne`, `-lt`, `-le`, `-gt`, `-ge`, or one-string truth; `true`, `false`, `:`, `exit [0…255]`.
- The first `#!/bin/bash` or `#!/bin/sh` line is a comment, not a process launcher. No startup files or inherited host environment. Each test starts with its declared environment, arguments, stdin and virtual files. Each Run starts fresh; editor work remains saved.

## Honest exclusions

Reject unsupported operators/syntax: command/arithmetic substitution, backticks, globbing, tilde/brace expansion, parameter operators, functions, arrays, subshells, groups, background jobs, here-documents/strings, `case`, `until`, `select`, `break`, `continue`, `local`, `export`, `set`, `eval`, `exec`, `source`/`.`. No external commands, filesystem metadata/links/devices, executable files, locale or binary/NUL data. Unquoted wildcard values are rejected rather than pretending globbing occurred. `[[ ]]` and Bash-only syntax are not taught in unit 1. All unit syntax is shared with POSIX shells; the reference oracle is Bash in its default mode with controlled inputs.

## Bounded execution

Maximum source 16 KiB; 4096 tokens; 24 levels of syntax nesting; 128 words/arguments and 64 redirects per command; 16 pipeline stages; 256 scalar variables; 64 files; 256 KiB aggregate virtual text/variables; 32 KiB captured output per case; 2,000 VM instructions per case. UI advances a bounded instruction slice per frame and offers Stop; interruption discards the current grading run. There is no wall-clock wait, sleep or blocking input. Input comes only from a finite fixture. Resource/unsupported diagnostics invalidate success. A stopped or edited script cannot retain current success. These are educational limits, not OS resource accounting.

## Fidelity evidence

`tools/bash_reference.py` runs **only checked-in controlled fixtures**, from a fresh temporary directory with an empty controlled environment and a timeout. It does not accept learner scripts or personal paths. It compares stdout, stderr, exit status and resulting text files with the Rust interpreter probe. Real Bash is a development oracle only; it is absent from the runtime interpreter. See the milestone verification report for the fixtures actually run and remaining exclusions.

Integrity clarification (2026-10-22): `grep -F` rejects patterns beginning with `-` until real `-e`/`--` option parsing is implemented. Integer comparisons accept signed 64-bit decimal operands; malformed or out-of-range operands return an explicit unsupported diagnostic instead of pretending to implement Bash’s recoverable test error. Stored diagnostics are bounded to 1,000 UTF-8 bytes so any supported-length script remains resumable after failure.
