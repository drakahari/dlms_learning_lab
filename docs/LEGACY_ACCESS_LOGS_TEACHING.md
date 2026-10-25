# Teaching for the original access and log tasks

Source-only implementation, 2026-10-25. Nineteen prospective backend/native-view
tests are **unrun**. No compiler, VM, reference or native acceptance run has been
performed. This adds teaching to three existing tasks; it adds no scenario,
unit, coverage status, dependency, host command or public release claim.

`legacy_access_teaching` owns a concrete small Unix-practice snapshot; it does
not replace `State::Unix`, the newer `Filesystem` Tree or the accounts model.
`legacy_logs_teaching` owns a fixed text-catalog snapshot and current selected
result; it does not replace `State::Logs` or the scripting VM. No generic State
or Goal is deserialized as practice authority. Each module converts its validated
typed snapshot to a bounded temporary old State, calls the existing in-process
engine once, validates the candidate and publishes only a valid result. Failed
commands leave the snapshot unchanged; the submitted text and bounded diagnostic
remain in its practice transcript.

The isolated branch starts from reviewed main97b9092. It remains separate while
MAIN records its actual before9 baseline. These two future library modules are
not silently added to a frozen reference attestation.

## Three existing assignments, separate practice

| Existing task | Independent practice | Concrete experiment |
| --- | --- | --- |
| Share a report safely | Dana/reviewers, /srv/brief.txt; group660 or ownership600 can grant private read/write | chmod440 removes write; repair current access |
| Unlock the project directory | Evan/inspectors, /srv/review/plan.txt; group membership and directory search are independent | chmod700 removes parent search; restore710 or750 |
| Find the failing requests | worker.log with two FAIL lines and a different audit log | cat whole log overwrites the filtered result; refilter |

Each Introduction explains the role, command and limits before use. Practice has
an explained command selector, **Insert action** (input only), **Run**/Enter,
fresh **Check example**, and **Reset example only**. It has its own users/files,
snapshot, command, transcript, selected action and view. The full assignment
retains the existing terminal and **Check work**. There is no new settings wizard
or Save configuration button for these terminal-only models. Learn/Challenge,
optional help, modal execution guards, focus/Undo namespaces and font-derived
Run controls follow the existing native flow. No new native acceptance controller
is authored in this batch.

New attempts open Introduction. Existing attempts without new metadata open
Assignment, retaining their state, previous command and history. Resetting one
example does not reset assignment state; selected assignment Reset leaves other
attempts and favorites intact. Practice checks and checked display flags never
award assignment completion.

## Original grading and command behavior remain

All116 original IDs, versions, initial states, typed Goals, solutions, units and
the complete unit map are preserved by an explicit fixture. The only old objective
prose clarification is authorized for traversal: “Give blair read access to
/srv/project/notes.txt without public file permissions. Repair group membership
and traversal as needed.” Its Goal stays private:true/exclusive:false. Existing
Alex/editor access is allowed; no Blair-only condition or exact mode/group event
is added. Historic Goals are decoded to typed values before comparison so an
omitted exclusive:false default is not a false modification.

The access model chooses owner, then matching group, otherwise other bits;
classes are not combined. A known path needs parent search plus requested file
bits, without an unasked directory listing/write requirement. Mutations are
virtual-administrator actions and bypass ownership/traversal restrictions; the
simulator-only `test-access` probe uses ordinary class/search logic and no root
access override. `namei` is a compact simulated component view. Numeric IDs are
derived display labels, not stable account records. Only known users/groups
exist; `usermod -aG` appends one existing group and refreshes modeled sessions
immediately. No host account, credential, login, ACL, SELinux or special-bit claim.
Every authored `test-access` annotation must set simulator_only:true.

The Logs terminal still tokenizes whitespace: no quoting, expansion, regex,
wildcard discovery, redirection or general shell. Its `find /var/log -name NAME`
is an exact-basename catalog query. `find`, `show` and `help` preserve selected
result; successful cat/grep/tail queries replace it. A rejected pipeline is
atomic. grep is a literal case-sensitive substring filter, not a severity-field
or whole-word parser. tail accepts0..10000 under the unchanged engine. Filtering
joins lines without a final newline; a no-match query stores empty text while
displaying a message. This is not a byte-exact GNU stdout/exit-status model.
Only current selected text is graded; running find or following one exact
command order is not a credential. Equivalent supported filters remain valid.

## New practice bounds and context

These are new practice limits, **not tighter assignment import rules**:

- Access: at most64 entries,16 users,16 unique group names/user; 32-byte simple
  lowercase ASCII account/group names;256-byte canonical absolute paths without
  whitespace/control/quoting characters; modes000–777, known owner/group and
  existing directory parents. Collection/string readers reject bounded typed
  overflow and duplicate map keys without size_hint reservations.
- Log source: at most8 files,16KiB each and64KiB total;256-byte canonical
  /var/log/ paths and32KiB current result. The file reader charges each bounded
  value before retaining it in the aggregate. Supplied text is immutable within
  practice and pinned exactly to the authored fixture; result remains mutable.
- Raw commands4096B; query output32KiB. Transcript32 entries,40KiB each and128KiB
  total, with an explicit omission marker when older output is evicted. Current
  scored results are not truncated to meet transcript limits.
- Snapshot::decode rejects raw input over1MiB, returns fixed short errors and
  validates the resulting typed snapshot. The1MiB ceiling accommodates worst
  legal JSON escaping:64KiB source +32KiB result may occupy about576KiB before
  punctuation/pretty formatting. Compact and pretty control-byte regressions
  check this case. This optional pure decoder does not
  change the application's existing2MiB pack/16MB progress readers. General serde
  fields use typed bounds; outer existing Content may already be buffered. No
  global streaming/zero-allocation claim is made.
- Six concepts,12 explained actions/reference commands,4 experiment commands,
  6 repair commands; bounded text descriptions and annotations. Directly
  constructed values validate before bounded clones/replay. Returned command
  errors are fresh UTF8-safe strings capped512B, without dumping source bodies.

Access practice pins path keys/kinds, account keys, existing group universe and
primary-group slots. Modes, owner/group choices and supplementary membership
remain editable, so both ownership and membership repairs are accepted. These
are invariants of the supported commands, not a full mutable-configuration
answer. Root and /srv scaffold paths may be shared; example target paths and
nonadministrator identities differ from the assignment. Separateness is checked
against authored initial context, never by requiring evolving independent
namespaces to remain disjoint.

Old Unix/Logs assignments keep their original validation and result bounds.
New practice validation is applied only to new optional learning metadata.
Tests deliberately import a larger valid old Unix map/account name and an old
99999-byte Logs result/more file keys, beyond new practice limits. Absence of
learning metadata remains valid. With metadata, saved examples validate their
own bounded type/kind; decode pins practice to the authored Lesson. No event
history or checked flag authenticates success.

## Source test and fixture scope

Eight access tests and seven log tests cover reachable teaching sequences,
Insert-only state isolation, owner precedence, private/exclusive distinction,
directory search without unnecessary read/write, alternative repairs, current
invalidation, malformed/oversize/duplicate decode boundaries, max-shape pretty
roundtrip, transcript eviction, unchanged old imports, context rejection, exact
save/load and selected reset. Actual frozen6/7 histories and all existing
contexts remain exact. Backup inspect/restore uses a separate new-learning-only
slice without script results, so it can compare every key without hiding the
shared inspector's deliberate script rechecks.

Four native-view source tests exercise fresh/old view defaults, actual Insert/
Run/typing/reference/experiment/repair, inactive Enter and the real private data
chooser, and100/150/200 percent wide/narrow Learn/Challenge controls. They are
headless source proposals until run; even passing headless tests will not prove
actual native, novice, accessibility or other-platform acceptance.

Two genuine fixtures in tests/filesystem.rs clone the original Unix scenario
then replace State/Goal with Filesystem. They now clear only the new incompatible
access teaching metadata, preserving their original intended goal/roundtrip
assertions. Separate wrong-kind teaching rejection controls remain explicit.
Assignment-only UI/smoke helper fixtures deliberately skip the new skippable
introductions; fresh-introduction tests use the normal launch path.

Source basis: original engine and validation, current lesson/unit metadata and
existing progress tests, all inspected2026-10-25. The current
[LPI objectives5.0](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
and [Linux path_resolution(7)](https://man7.org/linux/man-pages/man7/path_resolution.7.html)
were checked during contract review. Fresh GNU coreutils/grep retrieval timed
out; no new successful GNU reference run or broad utility conformance is claimed.
All code and practice text here are original; existing project notices remain.


M96 integration gates: all15 new backend checks pass within the279-check,
23-target combined model run. Four real-egui checks pass for separate drafts,
actual Insert/Run/experiments, inactive controls and100/150/200 reflow. Nine prior
seed regressions pass. Independent source review accepts bounds/context/old
progress and catches affirmative wording on failed practice rows; root maps only
false messages to blocked access/public access/empty/extra/incomplete log
explanations, keeping exact grading booleans and passing labels. The strengthened
15 model checks pass; repeated four UI checks pass; actual native review remains pending. Original116 Goals/States/solutions/versions stay exact; only the
already authorized traversal objective prose is clarified.


M98 actual installedc799 native follow-through: all3 original terminal tasks exit0
with43captures on Wayland using the isolated interval0 diagnostic. Actual library
Start/Continue, Introductions, Insert-only practice, typed Run/reference/experiment/
repair, wrong/partial/alternate/current invalidation, readonly queries, independent
unfinished command drafts/save/readback/navigation resume/reset/Challenge and
light150/dark200 wide/narrow inputs pass. Root reviewed permission Introduction;
additional representative render review and normal-policy candidate checks remain.
Private audit0e20d4265c1d9640919ec6fbfd73924054fc2ccd2fbfcf1319943b6c7858475c
pins43captures and same-binary startup results. No novice/physical acceptance.
