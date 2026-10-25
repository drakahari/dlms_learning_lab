# Closed Debian package command source proposal

2026-10-25, SOURCE ONLY. Fifteen prospective command tests and all prerequisites
remain unrun in this worktree. The original pure core has its separately recorded
gate; backendR3 is a source proposal, not a compiled app feature. No native UI,
lesson, unit, objective status, progress schema, evaluator revision or dependency
is added here. No host package program, real archive, compiler, reference fixture
or VM has been executed for this adapter.

## Purpose and honesty

This adapter gives the independent backend genuine Debian command spellings over
its fictional script-free package universe. It accepts literal argv, derives
query output from current data and computes real current changes through the
existing typed core. The internal `packages {JSON}` bridge remains authoring-only;
it is not advertised as learner syntax. Package commands require immutable Goal
context. Context-free engine execution still allows only simulator help/show.

This is a closed successful-final-state model. It does not simulate real dpkg
transactions/rollback, unpack/configure states, maintainer scripts, privilege,
repository transport, signing/trust or full APT selection/resolution. Successful
mutations return explicitly labeled `DLMS simulation:` summaries derived from the
current Plan's record/file/cache/artifact changes. They do not forge fetch URLs,
package sizes, hashes, PIDs, elapsed time, security checks or dpkg progress phases.
No query, command sequence or displayed string authorizes task completion.

## Supported real forms

- `dpkg-query -W [NAME]`, aliases `--show`, with optional `-f='FORMAT'`, separate
  `-f FORMAT`, `--showformat=FORMAT` or separate long-option value. Either query
  option order is accepted, duplicates rejected. Default output is package name,
  tab and version. Custom fields are only Package/binary:Package, Version,
  Architecture and db:Status-Status; supported escapes are backslash, tab and
  newline. Database status renders installed/config-files, without inventing
  wanted/error flags. Empty matches return1. Literal formats containing dollar
  fields use single quotes; shell expansion is not implemented.
- `dpkg-query -S ABS_PATH`, or `dpkg -S/--search ABS_PATH`: exact canonical path
  without pattern characters. An installed package's recorded owner remains
  queryable when its ordinary payload is missing. A missing owner returns1.
  Retained configuration-only ownership queries explicitly remain Unsupported
  pending more precise file-list evidence; no missing-owner answer is invented.
- `apt-cache --important depends NAME`, with the flag before/after `depends`:
  immediate cached Depends only. A compatible cached candidate must exist;
  installed-newer fallback and missing/provider metadata reject rather than
  fabricate APT's broader policy output. Offline reads remain possible. The
  command does not claim installed dependency health.
- `apt-get --error-on=any update`: strict refresh of the one supplied source.
  Installed records, artifact availability and virtual files are not refreshed.
  Plain update's warning/partial-result policy is not approximated.
- `apt-get -y/--yes/--assume-yes install NAME[=VERSION]...`:1–8 exact requests,
  one exposed cached candidate and bounded ANDed Depends closure. Install of an
  identical installed artifact is a no-op, not a payload repair. Explicit versions
  require exact exposed text, with no downgrade/alternate-version lookup.
- Add `--reinstall` only for one `NAME=VERSION` whose exact installed artifact is
  also the cached candidate. **All currently Installed records** must have
  satisfied Depends, including unrelated broken imported records. The core's
  exact Reinstall is never used to hide APT upgrade or global repair semantics.
- `dpkg -r/--remove NAME`, `dpkg -P/--purge NAME`: one exact low-level operation.
  Reverse dependency checks, retained conffiles and recreated unowned-file safety
  come from the core. An absent record produces a derived successful no-op.
  These are not aliases for APT's dependent-removal proposals.
- `dpkg --compare-versions V1 lt/le/eq/ne/ge/gt V2`: admitted nonempty Debian
  versions, empty stdout and a0/1 boolean status. Other real operators/empty
  versions remain an explicitly unsupported subset, not guessed comparisons.

APT trailing package `+`/`-` selectors are Unsupported even when quoted and before
an optional `=VERSION`. No accidental passage through the core's package-name
character set may turn a selector into an exact installation request.
The check applies only to NAME after splitting `NAME=VERSION`; a valid exposed
version ending in `+`, such as `1.0+`, remains an exact version, including quoted
operands. A prospective positive test installs and reinstalls that version.

No dpkg -i/unpack/configure/verify/audit/list/status/full file-list facade, generic
apt, APT remove/purge/upgrade/autoremove/fix-broken/pinning/force/config overrides,
archive pathname commands, RPM/yum/dnf or arbitrary host command is supported.
Payload findings remain separately labeled typed data; no fake md5/verify output
is produced. Archive IDs remain descriptors, not fabricated filesystem archives.

## Bounds, publication and current outcomes

Input2048 UTF8 bytes is checked before trim/tokenization/context work. One literal
command allows64 argv words; physical newlines, NUL/other controls, operators,
parameter/command substitution, unquoted filename expansion and unknown flags
are rejected. Quoting/escaping uses the existing literal-argv helper, not a shell.
Format256 bytes, package64, admitted version128, canonical path160, requests8.
Bounded parsing completes before any State clone/mutation. Parsed commands have
private fields and no Deserialize route for injecting unchecked requests.

Queries stream into a32KiB bounded output builder without truncation; combined
stdout/stderr stays within that bound. Diagnostics have fresh UTF8 allocations
at most512 bytes. Query statuses are0/1/2, APT supported operation/error statuses
0/100, dpkg mutation success/error0/2. A parser/context/subset failure is labeled
as a simulator diagnostic, not a claim about every real utility error message.

All queries and mutations repeat current immutable Catalog/Goal admission. A
read-only Plan sizes the derived mutation response; actual goal-aware Apply then
recomputes its action from the same current state and publishes once. There is
no persisted approval token or post-publication output failure. Existing app
State/Draft/Definition and global pack/profile caps are unchanged. Pure core
maxima can still exceed app admission and are rejected without publication.

Ordinary terminal command proposals do not constitute pending configuration.
Read-only/unsupported commands preserve truthful current credit; successful real
mutations can invalidate it, and a legitimate repair can restore it. Actual
fresh package/file predicates and explicit preservation remain the only grading
inputs. No teaching, progress or current-state guard was weakened for commands.
A future native console needs its own bounded assignment/example command drafts,
transcript and UI/modal/focus acceptance; none is introduced by this source stage.

## Prospective evidence and next gate

The original source proposal contained fifteen tests covering real record formats/statuses, missing payload ownership,
cached/offline dependencies, strict refresh, automatic closure versus dependency-
first alternatives, no-op Install versus repair, exact-current candidate guard,
unrelated broken Depends, quoted selectors, Remove/Purge and recreated-file safety,
version exit statuses, unsupported/shell/option isolation, input/format/output
ceilings and fresh Goal/isolation/context behavior. All expectations are authored
from the declared model/primary manuals, not observed package programs.

Before compilation, peer/root review must check the source diff against exact
backendR3 snapshots and CLI contractR2. After an explicit sole-heavy-slot grant,
use MAIN tools/dev.py with this checkout's automatic exclusive target for the
new terminal/backend/core tests and relevant progress/history regressions, then
Clippy/fmt. Native controls, practical lessons and any coverage claim are separate
milestones. Do not install a host utility to manufacture a reference result.

Primary manuals were checked read-only2026-10-25: Debian trixie apt-cache(8)/
apt-get(8) APT3.0.3 and dpkg(1)/dpkg-query(1) dpkg1.22.22. Exact URLs and the
source-to-command limitations are recorded in ignored CLI contractR2. No upstream
implementation code was imported. Frozen genuine6/7/8 fixture bytes, original125
prerequisite source data and pure core remain unchanged in this worktree.

MAIN first gate passes all15 terminal checks plus68 library/19 pure core; backend
20 passes after correcting one test-only Installed-versus-ConfigFiles setup.
Commands parse/evaluate against actual bounded virtual state, with no host
program/archive execution. Native workbench and authored content remain pending.

Broader MAIN gate: eight existing progress/backup/workspace/content/history targets
pass after integration (m110-package-progress-regressions-r3); all-target ClippyR3
and formatter pass. One context-guard style correction and an array-only test
allocation correction preserve predicates/expected outcomes. First ordinary gate
observed min19.48GiB available, max memory PSIavg10 some/full0.3/0.3, no OOM
increase; no sustained pressure or extra heavy invocation. No stress test.
