# Fixed public9 / safe-automation decision capture

2026-10-26. Reviewed R3 integrated over133/33. All176 Python guard tests and
five Rust probe tests (including digest self-checks), all-target Clippy and the
ordinary probe build pass. Actual metadata preparation, full VM/Bash producer
and observation publication remain pending at this checkpoint. No evaluator,
parser, record revision or curriculum is changed by this developer tooling.

The proposed order is a narrow literal-empty `IFS= read -r NAME` capability,
then explicit `set -o pipefail` / `set +o pipefail`, then two meaningful automation
projects (a third only if it adds distinct work). Dynamic/nonempty IFS, general
command environment prefixes, other `set` modes, errexit, traps, host files and
host learner execution remain outside this proposal. Pipefail observations are
Bash5.3-reference evidence only until primary POSIX Issue8 text is verified.
No GPL implementation source is imported.

## Closed inputs and genuine history

`automation-oracle-inputs.json` contains28 fixed builtin-only sources and finite
stdin/file inputs. It covers spaces/tabs/backslashes/percent/CR/UTF8, blank/empty
and unterminated read input, literal empty-quote forms, helper/capture/redirection
scope, default read controls, and pipeline status/toggle/function/child scope.
Dynamic-empty assignment and an unrelated prefix are explicitly deferred
observations. Quoted command head and prefixed read pipeline-stage behavior are
queued decisions, not promised support. No fixture has predicted outcomes.

`automation-vm-boundaries.json` contains8 separate finite VM-only input vectors:
ordinary-name256/257, retained expansion memory, whitespace work, output limit,
static incomplete-case before effects, active-input truncation guard and non-C
case matching. These inputs NEVER reach Bash. API Case expected fields are empty
scaffolding; no `Case::accepts` claim is made for these boundary/new-feature rows.

`automation-before10-authored-inputs.json` freezes all43 actual Script definitions,
282 assignment cases and43 separate examples. R2 froze them at d424ffbf/125;
R3 repins only the catalog/source closure to clean b17c462 (M116 code38a8098),
133 normal scenarios/33 units. Every script input field and ordered example bundle
remains byte-exact R2. Sources, env/stdin/args/files, expected authored outcomes,
versions and unit identities match their raw packs. The future probe must execute the public9
VM, assert existing assignment/example acceptance, and log outcomes at slices
1/16/32/64 with exact slicing equality. It creates43 revision9 Records ONLY from
those actual observations. It does not relabel older records or synthesize a
revision9 success from expected fields. Stop is ungraded: a real nested child must
write its virtual marker, measured ticks must be positive, and eight post-Stop
calls at each slice must leave the complete Outcome unchanged.

The exact existing synthetic writer8 fixture is the starting profile. Its genuine
33 revision6 histories,37 revision7 histories and40 actual revision8 Records are
retained in full equality. Actual9 records are separate current results. The
original before8 slot remains exact writer7 bytes; the first before9 slot must be
exact writer8 bytes, with actual target-derived receipts `[7,8,9]`. No intermediate
revision is invented. All current normal contexts start ungraded except the43
actually observed Script results. No non-script completion is fabricated.

Two new deliberately unfinished assignment/example drafts are added only to
`shell-handover` and `shell-capture-handover`; their previous genuine result cannot
satisfy changed source. Original writer8 also already has unfinished example
sources in `shell-arguments` and `shell-capture-preview`; those exact texts and
old result provenance remain intact. They are independently observed as separate
public9 recheck inputs, never silently replaced by an authored example. Restoring
an inspected backup may explicitly recheck current results; Python independently
projects the complete expected profile from logged VM outcomes and compares every
field. Only main-file `migration_receipts` is removed for portable serialization.
Both old and new unfinished texts, all histories, unrelated Attempts and favorites
remain exact. Reset-first and a different valid JSON occupied-slot failure retain
exact target/slot bytes. Export preview cancellation never changes the target.

## Admission and resource contract

The only new guarded runner allowlist entry is `automation_before10_oracle.py`.
It accepts exactly one of `--prepare-id ID` or `--capture-id ID`; no source, pack,
profile, arbitrary environment, command, binary or path option exists. Unknown or
abbreviated options fail before any child. Actual preparation/capture require
MAIN's `.git` directory, MAIN/default target, inherited shared guard and2 jobs /
2 runtime threads. Worktree execution is refused. The compiled probe separately
attests all closed library sources, bundled packs and fixed input bytes; revision9,
linked catalog/script identity and actual source identity must agree. A cached
wrong library/probe cannot satisfy that attestation.

Preparation creates a no-clobber PROPOSED file with `root_reviewed:false`, hashes
metadata only, and executes neither Bash/version nor VM. Root must independently
recompute and create the exact APPROVED-ID control before capture. Preparation
itself has NOT been performed for this source proposal. The R3 source base is
clean b17c462, code38a8098, with85 library files and the exact133/33 registration.
Its compiled attestation adds the two newly registered package/STP pack files and
retains the current units/pack source bytes. UI/test-cache modules are excluded
because the probe links the library rather than the application. Source identity
remains strict: any later catalog/module change requires a new reviewed repin.
MAIN integration, guard tests and probe build pass. A clean committed execution
identity and independently reviewed actual pins still precede every capture.

Ordinary inputs and executable identities are read/hash-checked through bounded
same retained FDs with regular/no-symlink/size checks. Pin count<=256; initial and
actual aggregate pin bytes<=1.5GiB. The full probe binary is streamed in1MiB chunks
under a1GiB ceiling, not copied or truncated. Bash executable and original GNU5.3
manual identity are pinned independently. The actual bounded version probe must
report Bash5.3.9; the collector does not guess that result from a filename.

Capture creates a fresh0700 project-disk directory with0600 no-clobber files and
retains every fixture folder/map, raw stdout/stderr, partial failure and original
control. Environment is closed: C locale, empty PATH, private TMP/TMPDIR/TEMP,
no HOME/startup/function-import variables. Only the reviewed builtin sources are
passed to pinned Bash. Finite nonempty stdin is forwarded as its exact UTF8 bytes;
the inherited hardened helper handles partial writes/EAGAIN/EPIPE, strict output
caps, held pipes and owned-group quiescence. Reference children may not detach.

Per Bash case2s, version2s, identity10s, VM180s (probe deadline150s), whole
capture300s. Every launch first requires positive remaining whole time and uses
the smaller per-child/remaining timeout. Deadline expiry initiates bounded owned
cleanup and can never publish COMPLETE. Bash output64KiB per stream; VM32MiB per
stream /4MiB per stdout row (stderr must be empty for success); profile16MB each /
64MiB aggregate, checked against both preflight stat and actual same-FD pin sizes;
all retained outputs128MiB. The evidence budget charges actual profile pin bytes.
Excess, changed pins/outputs, unstable slices,
unquiescent children, missing rows or failed profile proof abort before COMPLETE.
Binary size is outside output capture and never charged as a fictitious copy.
All processes remain under MAIN runner's memory monitoring/shared lock.

COMPLETE is the no-clobber commit boundary only after all child processes are
quiescent, source/binary/input/manual/Bash pins and every output are rehashed, all
scope/profile/Stop checks pass, and prepublication directory sync succeeds.
Postcommit directory-sync/reporting failure is reported as a durability warning;
it cannot pretend no valid marker exists. Failures before publication preserve
STARTED/partial evidence without a success claim. No broad cleanup is authorized.

## Pending gates

The source adds27 Python guard/validator tests and four Rust probe checks plus
a digest self-check; all pass in the executed176-Python/five-Rust gates. They cover closed admission, actual aggregate pin growth,
non-overwrite publication, source/compiled identity, raw history/input preservation,
exact full-field recheck projection, finite stdin and Stop/stream boundaries.
They use fake validator rows/small synthetic files and do not execute Bash or the
capture producer. Any fake validator values are ephemeral test inputs, never
published observations or learning credit.

After source review, root chooses serialized MAIN integration/guard tests/probe
build, reviewed metadata pins, then a separately authorized actual fixed capture.
An independently rehashed COMPLETE and all raw records are required before any
empty-IFS/pipefail implementation. Changed public behavior will require revision10
and a separate exact before10 boundary preserving genuine6/7/8/9 history. Neither
future evaluator code, migration, authored projects nor native acceptance is part
of this source proposal.
