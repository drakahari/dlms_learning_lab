# Bounded environment subset — evaluator17

Implemented core and typed parent/child activity,2026-11-04 local. The source
catalog registers two LPIC-1 lessons; native integration validation and a new
candidate remain pending. The last verified candidate is257/74, evaluator16.
Keep these statuses separate. The genuine16 baseline remains sealed and immutable.

## Values and inheritance

A shell variable holds a value in the current script. An export attribute marks
a value to be passed to a subsequently started program. Exporting does not change
the parent's value or permanently configure an operating system. A set empty
string and an unset name are different states. Exported-but-unset names have an
attribute but contribute no name/value pair to the child's environment.

The implementation reuses the existing parser, resumable expansion
and VM. Supplied ordinary `Case.env` names begin exported. Assignment and `read`
preserve an existing ordinary variable's attribute. A bounded read-only observation
exposes actual scalar values/attributes and derive the ordinary child map.
It does not change the historical Outcome/Record format or trust a success boolean.

Supported declaration forms are literal ordinary `export NAME` and
`export NAME=value`, with one or more bounded operands. Assignment RHS expansion
uses declaration context: spaces and wildcard text remain part of the value,
rather than ordinary argument splitting/globbing. Quoted forms remain preferable
for readability. `export -n NAME` removes only the attribute, preserving the
parent value; **-n is Bash-specific**, not a portable-shell feature.

The first supported program shape is global scalar assignments, supported simple
commands and if/else or &&/|| decisions. Functions, pipeline stages, loops, case,
arrays, command substitution and prefixed commands need faithful attribute scope
handling before this environment exercise can accept them. Reject these shapes
honestly; ordinary non-environment support remains available. Do not implement a
second shell parser. Reject computed declaration heads/names, listing/options not
explicitly supported, arrays and special/reserved variable names. New declaration
operations combined with xtrace are initially unsupported until compared.

This is not PATH command search, startup-file execution, aliases, history,
`declare -x`, `local -x`, readonly, `set -a`, an `env` command launcher or general
temporary `NAME=value command` semantics. Preserve the existing exact `IFS= read -r`
exception as a read policy. No external program or learner source runs on the host.

## Assignment adapter and grading

Use a separate typed environment activity record and the existing VM/editor
components. Run the learner setup, observe its actual parent state, derive the
child map, then execute a small immutable supplied child-report script in another
bounded in-process VM. Share an explicit finite job budget; Stop must remain
responsive. Learner code cannot select a host child or modify the supplied report.

Grade the actual parent values/attributes, exact permitted child visibility,
child behavior, statuses and unchanged virtual files across multiple deterministic
cases. Printing look-alike child output without exporting must fail. Equivalent
assignment/export order and supported declaration forms should pass. Preserve
separate practice drafts, stale-result invalidation, safe resume and honest
historical credit. Do not add new mastery to an old attempt during migration.

## Sources and future work

Authoritative definitions: GNU Bash [Environment](https://www.gnu.org/software/bash/manual/html_node/Environment.html),
[Bourne Shell Builtins](https://www.gnu.org/software/bash/manual/html_node/Bourne-Shell-Builtins.html)
and [Command Execution Environment](https://www.gnu.org/s/bash/manual/html_node/Command-Execution-Environment.html),
retrieved2026-11-02 local; official indexed GNU text rechecked2026-11-04 (direct GNU fetch timed out). LPI current combined objective page was retrieved2026-11-04 and confirms v5.0/103.1. These facts are paraphrased; no GNU implementation or
restricted LPI lesson text is copied. Planned practice supports a narrow slice of
LPIC-1 v5.0 103.1, preceding105.1; full environment/startup objectives stay partial.
The28fixed disposable Bash comparisons verified the supported declaration/state
behavior against the retained core snapshot; see the evidence limits below. Their oracle is never the learner execution engine. No native platform,
physical-device, screen-reader or novice acceptance follows from this contract.

## Scoped evidence and limits

Twelve core groups, nine typed activity groups and one memory-ledger group pass.
Twenty-eight controlled Bash observations match the in-process outcomes and
selected parent/export states, independently checked against retained source and
executed probe. Four compatibility groups cover641genuine16 observations and
158synthetic progress contexts, including90unrelated contexts and two drafts.
No historical producer/source/seal was rewritten. These are scoped comparisons,
not a claim of general Bash or operating-system process fidelity.

Current integration has actual parent values, child visibility, separate expected
practice output, actual setup/child statuses, cooperative Run/Stop and independent
example drafts. Persisted records retain observations but cannot supply current
credit until their saved source actually re-executes. A nonserialized in-process
consistency token detects changed record/source data; this is not a signature or
security boundary. Import validation and task identities remain mandatory.

On2026-11-04 the guarded runner stopped a UI-test compilation after sustained
memory PSI; min13.928GiB available, maximum full/some avg10=12.83%, OOMdelta0.
The owned process was reaped; no unrelated workload/settings changed. That run
is not a test pass. Native widgets, latest source gates and packaged acceptance
remain pending until completed and recorded in ENVIRONMENT_LESSONS.md.
