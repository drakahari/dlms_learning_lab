# Case selection: development reference boundary

Checked 2026-10-23. These actual reference observations preceded the bounded
revision 8 extension documented in [SHELL_CASE.md](SHELL_CASE.md). Revision 7
rejected `case`; its genuine results are retained separately. Reference evidence
alone does not add curriculum or establish native acceptance.

The developer-only `tools/case_bash_oracle.py` accepts exactly the37 hash-pinned
original fixtures in `tests/fixtures/shell-case-oracle-inputs.json`. It cannot
accept an arbitrary script, fixture path or Bash executable. The immutable input
file intentionally retains its proposal-era NOT RUN/null fields; actual
observations are separate retained records, never substituted into the inputs. It is absent from
the app's execution path. The ordinary app always uses the in-process simulator.

Run through the shared memory guard, with a new evidence ID:

```sh
python3 tools/dev.py tool case_bash_oracle.py --capture-id NEW-REVIEWED-ID
```

The collector uses private retained project-disk fixture directories, omitted
HOME, an empty PATH, a closed C-locale environment, no startup files, and explicit
GNU non-POSIX mode with extended patterns disabled. The only host scripts are
the reviewed finite builtin fixtures. Output, input, file count/size, process
time and the whole batch have limits. No child detaches from the guard's process
group or inherited lock. Failures retain incomplete evidence; the collector
never retries another fixture after uncertain cleanup. It never signals another
application. No real services, personal files or network commands are involved.

Actual fixed37 run

- Bash5.3.9(1)-release, x86_64-redhat-linux-gnu; the installed original Edition5.3
  manual and actual binary/source identities are pinned in the retained manifest.
- Capture `fixed37-bash539-r1`, manifest SHA-256
  `7af1b2e86a9cf9364c5ea170bcdfba4142a5c8cff0c5046efd0be57c55af02bd`.
  Root and an independent reviewer checked COMPLETE, all157 output hashes and
  all37 source/input/stream/status/final-file observations. These are development
  fixtures, not learner progress. They remain ignored under `.build-artifacts`.
- All73 Python checks passed, including20 collector mock/small-pipe checks for
  stream limits, timeout/cleanup, stdin short writes/EAGAIN/EPIPE, file bounds,
  closed environment and no-clobber completion publication. These tests do not
  themselves start Bash. The separate actual run supplies the observations.

Decisions supported by those observations

| Behavior | Actual GNU Bash observation |
| --- | --- |
| Matching | First matching arm, not the most specific; quoted wildcard bytes are literal |
| Selector | No field splitting or filename expansion; expanded whitespace and `*` remain data |
| Pattern context | `*` includes dot and slash; `?` includes newline and matches one C-locale byte, not a multibyte character |
| Lazy expansion | Selector executes once; later clauses do not expand after a match; this GNU alternative list expanded only its first matching alternative |
| Status | No match and a matched empty body return0; a selected `false` returns1 |
| Status visible inside body | Prior `false` remains1 without captures; a selector or reached pattern capture ending in `exit 7` makes the body's first `$?` see7 |
| Grammar and scope | Optional leading parentheses, final separator, nested case/function/capture, quoted `)` inside capture, `(esac)` as a pattern and return from an arm work |

Within-clause pattern expansion side-effect order is not a portable teaching
guarantee: POSIX Issue8 leaves it unspecified. GNU's observed order is a
deterministic compatibility choice only. Dynamic bracket/backslash patterns and
fallthrough work in the recorded Bash fixtures but are deliberately excluded
from the proposed first simulator subset. POSIX Issue8 includes `;&`; its
exclusion must not be described as proof that it is nonportable.

Primary references: the original installed GNU Bash manual, and the official
[Conditional Constructs](https://www.gnu.org/software/bash/manual/html_node/Conditional-Constructs.html),
[Pattern Matching](https://www.gnu.org/software/bash/manual/html_node/Pattern-Matching.html)
and [POSIX Issue8 shell language](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html)
sections2.9.4.3/2.14. Direct GNU requests timed out and POSIX returned403 during
the source review; the installed original and official indexed text were
consulted. Original observations establish only these fixtures, not full Bash
conformance. No upstream implementation or manual passage is copied into the
simulator.

Before activation: real parsing, quote provenance, case-aware capture delimiter
scanning, cooperative matching and shared budgets must pass review and tests.
The unchanged7 baseline must be compared, and genuine6/7 records retained through
an explicit evaluator8 transition. No outcomes may be regenerated and labeled as
earlier evidence. Supporting hundreds of worthwhile activities remains unfinished.
