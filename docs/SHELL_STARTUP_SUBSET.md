# Bash startup-file simulation subset

2026-11-04. Two lessons are integrated in source; focused/native acceptance is
in progress. The verified267/79 review candidate remains unchanged.
The existing evaluator17 and all ordinary script/earlier environment behavior
remain in place. No new dependency or host execution is introduced.

## Selected files

All paths identify supplied virtual data. A file present in this small map is
readable; a missing map entry is absent. Unknown paths and permission-state
modeling are refused, rather than pretending to cover the host filesystem.

| Supplied invocation | Files considered, in order |
| --- | --- |
| Interactive login | `/etc/profile`, then the first present readable `~/.bash_profile`, `~/.bash_login`, `~/.profile`. An empty higher-priority file still wins. `--noprofile` suppresses these files. |
| Interactive non-login | `~/.bashrc`; `--norc` suppresses it. |
| Noninteractive, non-login | The supplied startup setting `BASH_ENV=/work/startup.env`, if set and present; otherwise no startup file. This does not search PATH. Other nonempty values explicitly fail as unsupported. |

A login does not automatically read .bashrc. A real profile commonly sources it
explicitly, but the dot/source builtin is not implemented by this activity. Do
not teach that link as automatic. Invoked-as-sh/POSIX mode, noninteractive login,
remote/privileged sessions, distro-specific additions, logout files, aliases,
history, PATH command resolution, arbitrary BASH_ENV expansion and unreadable
files are outside this slice. Fixed virtual home/control inputs must not become
personal paths or learner setup requirements.

## Executed settings

Each selected file is a real, separately parsed program for the existing bounded
in-process VM. Ordinary variables, quoting/expansion within the supported17
subset, export attributes, a literal-format printf, tests and if/else or &&/||
are supported. Supported explicit exit stops the whole simulated session.
Functions, loops, arrays, pipelines, redirections, computed command heads,
printf options/computed formats and commands outside this settings subset fail
clearly. These constructs remain available where supported in other scripting
activities; no existing lesson or parser behavior is removed.

HOME is supplied as `/home/learner` inside the simulation. Other supplied HOME
values and changes to it are refused. BASH_ENV is a typed startup-selection input;
reading/writing that special name inside scripts remains outside evaluator17.
HOME/BASH_ENV, locale and special shell-control settings are not writable in this
activity. The environment is fixed to C/POSIX. The supplied session command runs
after startup unless an error or explicit exit prevents it. Selecting a file is
not proof that its settings were applied: actual output, status, variable values
and export attributes must come from execution, never a canned success label.

One VM retains variable values, export attributes, exit status, output and
resource budgets between file programs. Empty/comment-only files return zero.
Text must not be concatenated to make an unfinished quote/control block in one
file accidentally consume the next file. Every selected input is preflighted
before execution: this is deliberately stricter than real Bash, which can run an
earlier file before discovering a syntax error in a later one. No atomicity of
ordinary real shell startup is inferred.

All6source files together hold24KiB; each file8KiB; admitted file-map JSON64KiB.
Selected startup source plus the supplied session command is16KiB; the latter
at most2KiB. The VM's existing global execution, expansion, memory and32KiBoutput
limits remain shared across files. All retained program arenas/source labels are
accounted, and each tick executes at most16VM instructions before yielding.
Stop prevents unfinished results from being published. No real dot command,
shell, startup file, user account, process, service or network is used.

## Evidence and references

The library compiles;9model groups and21retained environment/workspace groups
pass. Independent read-only review found a quoted export-name control bypass;
constant-word normalization and negative/legitimate-quoting regressions fixed it.
An initial OPTERR concern was retracted after checking the upstream manual: Bash
resets it to1 on invocation, matching existing17 behavior. A fixed18-recipe
comparison collector/probe is being prepared; its guard tests pass, but no actual
Bash observation/comparison is claimed yet. Authored tasks, behavior grading, native UI,
progress binding and controlled real-Bash comparisons are not implemented yet.
No conformance, new curriculum, novice or platform acceptance is claimed here.

Primary definitions checked2026-11-04 from the installed upstream **GNU Bash5.3
manual, dated18May2025**, `/usr/share/doc/bash/bashref.html`, section6.2. Browser
requests for the corresponding GNU pages timed out; those failures are retained,
not represented as successful web retrieval. Authoritative URLs:

- [GNU Bash startup files](https://www.gnu.org/software/bash/manual/html_node/Bash-Startup-Files.html).
- [GNU Bash invocation](https://www.gnu.org/software/bash/manual/html_node/Invoking-Bash.html).
- [GNU Bourne-shell builtins](https://www.gnu.org/software/bash/manual/html_node/Bourne-Shell-Builtins.html).

The manual is consulted for behavior, not copied/bundled as this project's text.
Future reference tests must use fixed reviewed builtin-only programs and
project-disk disposable files. Never launch a normal login Bash that might read
the machine's system or personal startup scripts. Separate primary-doc selection
checks from an actual controlled comparison of supported execution semantics.

## Implemented bindings and evidence status

The source-bound native editor now compiles. Ten core groups, seven workspace/
storage groups and21 retained environment groups pass. Eighteen fixed manually
sourced-settings comparisons match actual Bash and have separate independent
raw/readback review. The original observation and comparison receipts remain
immutable; adding a workspace does not relabel them as whole-app acceptance.
Quoted HOME export-attribute changes are refused after quote normalization.
OPTERR resets to1 at invocation, as specified by the upstream Bash manual.
Authored lesson tests and actual native verification remain in progress.

Each task admits2–6 bounded sessions (262KiB total definition), with six login
variants or five interactive/script variants in the authored unit. Every editable
startup context is exercised with at least two supplied values; a guessed value
for a fallback file or script-only startup file cannot satisfy the whole task.

Scoped packaged Fedora checks now pass for the two authored lessons; see
SHELL_STARTUP_LESSONS.md. Evaluator17 remains unchanged. Full105.1 and real
startup/session integration are not established by these bounded settings tasks.
