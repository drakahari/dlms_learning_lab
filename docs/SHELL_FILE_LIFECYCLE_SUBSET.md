# Shell flat-file lifecycle contract — evaluator18

2026-11-04. Prerequisite: independently sealed genuine17 baseline in
SHELL_EVALUATOR17_BASELINE.md. This document does not establish a finished unit
or added objective coverage. No host file, process, network or service operation.

Admit only:

| Utility | Supported form | Behavior |
| --- | --- | --- |
| `mktemp` | `mktemp [--] TEMPLATE`, one explicit flat filename ending in exactly six `X` characters | Create an empty unused virtual file and print its name. Absolute `/work/…` and relative names use the existing fixed `/work` namespace; printed form follows the supplied form. No default template, directories, suffix flags or other options. |
| `mv` | `mv [--] SOURCE DEST`, one ordinary-file pair | Move the source bytes/name, replacing an existing ordinary destination. Missing source or identical normalized paths return a normal failure with a diagnostic. No directory/metadata/cross-device/interactive/backup forms. |
| `rm` | `rm [--] FILE`, one ordinary filename | Remove it. Missing file returns a normal failure. No force, recursive, directory, prompt or multiple-operand forms. |

Options must precede operands. `--` ends option interpretation; quote expanded
pathnames. Unsupported forms/path namespaces produce an honest simulator
limitation rather than silently approximating GNU behavior. Utility failures
have nonzero status/stderr and can be handled with supported shell conditions.

Temporary names use a bounded VM-owned deterministic sequence, shared across
command substitution/pipeline scopes. Collision search is bounded and charged;
the existing64-file/256KiB memory/32KiB output/instruction/expansion limits remain.
The real GNU utility uses security properties this model does not represent:
no real randomness, permissions/ownership, symlinks, races or adversarial users
are simulated. Teach use of the returned quoted name, never guessing filenames.
No exact temporary spelling should be graded or compared with real GNU output.

Before mutation, normalize names and enforce existing active-input guards on both
rename endpoints/removal. Refuse a target aliased by any actually opened current
or suspended output destination. The model stores output destinations by path,
not inode/descriptor; allowing such mutations would recreate/misroute output.
Future raw redirect words are not opened descriptors and are not blanket-blocked.

Redirections and earlier substitutions execute before a utility. Refusing a
mutation does not undo an earlier truncation or other virtual effects. No whole-
command rollback, durable/atomic real publication, locks, traps or crash recovery
is promised. Stop/limits end the bounded run; they do not emulate EXIT cleanup.

Primary sources: GNU coreutils9.10 installed upstream manual
`/usr/share/info/coreutils.info.gz`, mktemp/mv/rm invocation nodes (checked
2026-11-04); official web fetches timed out. Canonical sources:
https://www.gnu.org/software/coreutils/manual/html_node/mktemp-invocation.html
https://www.gnu.org/software/coreutils/manual/html_node/mv-invocation.html
https://www.gnu.org/software/coreutils/manual/html_node/rm-invocation.html

No GNU manual/code/binary is bundled by this change. A fixed controlled developer
comparison will use project-disk disposable fixtures; it must never become the
learner execution engine. Portable shell syntax and GNU utility extensions must
be distinguished in teaching. Outcome grading should accept supported alternate
solutions while checking protected inputs, final outputs/status and no leftovers;
it cannot prove security randomness or real crash safety from final file maps.

## Verified implementation boundary

Eleven pure evaluator groups pass: normal outcomes, literal names, collisions,
active input/output aliases, suspended captures, status recovery, unsupported
forms, file/instruction/memory bounds and cooperative Stop. Current18 reproduces
all641 separately sealed17 authored outcomes exactly and preserves the synthetic
inherited/typed progress pairs. Old source, producer binaries and observations
remain immutable; loaded17 results remain honest historical credit until rerun.

Fourteen fixed, disposable developer recipes matched actual Bash/GNU status,
stdout, diagnostic presence and full final-file maps. Independent pre-execution
source/executable review and separate raw readback retained approved source,
exact composed Bash scripts and executable identities. Reference report SHA256:
`f7069a68bdc978a70c37bd06bdab42e3186e0ca4a94f74a128365d930fd5f4c8`.
Exact GNU diagnostic wording, random temporary spelling, descriptor-alias
semantics and general GNU conformance are not claimed. These reference tools are
never called by the learner application. Three authored projects are documented
in SHELL_FILE_LIFECYCLE_LESSONS.md; capability evidence alone is not a lesson.
