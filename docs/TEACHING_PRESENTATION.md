# Approved teaching and presentation standard

Owner-approved2026-11-01, based on the refined native CCNA magic-number pilot.
The owner used **Show worked answer**: this is assisted review, **not independent
novice-comprehension acceptance**. Mockups are design guidance, not implemented
screens. No platform, screen-reader or novice-study acceptance follows from this
approval. Keep existing simulators, objectives, starters, grading and progress.

## Three learning screens

**Introduction** states one clear learning objective, concise prerequisite context,
numbered concept sections and an annotated worked example before inviting action.
Keep essential explanations visible. Put optional depth in expandable help. Use a
table, command/code example or diagram when it clarifies the subject. Subnet ranges
use equal-width aligned blocks, rather than widths determined by their labels.

**Try example** offers separate, meaningful guided practice. Keep concise action
guidance beside the actual command picker, editor, configuration control, file
operation or calculation. Name controls exactly. Explain what to change and what
to observe; show intermediate steps where useful. Identify ungraded status once.
Distinguish expected preview, actual results, unchecked work and explicitly shown
worked answers. Edits must invalidate stale results and update dynamic guidance.

**Assignment** preserves independent work and intentional starter errors. State the
task and scannable requirements without exposing answers. Give unambiguous labels
and neutral format cues. Retain the appropriate workspace, helpful diagnostics,
retry, completion and saved drafts. Challenge hides unsolicited guidance.

## Native implementation contract

`src/ui/lesson_presentation.rs` supplies reusable objective/section headings,
left-aligned prose, worked examples, requirements and guided-action rows. Existing
workspace modules retain their IDs, editors, command entry, simulation actions,
checks and focus handling. Use these components around those tools; do not force
all subjects into subnetting fields. `ccna_presentation` remains a compatibility
adapter during the rollout so existing callers are reconciled deliberately.

Per-lab presentation data lives in `scenarios/presentation/{ccna,lpic1,lpic2,bash}.json`.
It is UI-only declarative prose, never engine instructions or an execution route.
Every entry retains its source scenario ID. Requirements must preserve all original
conditions; supporting notes never change the grade. Review original concepts,
worked/practice data, actual controls and assignment requirements for each entry.
Do not mark a lab updated solely because it inherits a shared renderer.

The inventory uses the actual registered normal catalog plus the four approved
pilot lessons. Statuses: **pending**, **updated**, **already meets**, **blocked**.
Record a concrete review note for Introduction, Try example and Assignment, or
an explicit reason a view is inapplicable. Native sampling checks representative
workspace families; it is not a claim that every lab received a human walkthrough.
Changes to curricula or grading require separate reporting; do not hide them here.

## Visual rules

Use headings, whitespace, subtle dividers, selective emphasis and aligned columns.
Prose stays left aligned without stretched spaces; commands, addresses and actual
output use monospace. Control sizes come from existing shared font/padding metrics.
Reflow columns into readable stacking at narrow widths/enlarged text; scroll rather
than shrink fonts or clip controls. Preserve native keyboard behavior and themes.
Avoid repeated disclaimers, generic inspect/apply instructions and dense dashboards.

## Representative implementation evidence

Refined pilot's actual packaged screenshots (local ignored evidence):

- [Introduction](../.build-artifacts/native-runs/run-jhkb64jx/screenshots/workspaces/ccna-guidance-introduction-dark-100.png)
- [Guided practice](../.build-artifacts/native-runs/run-jhkb64jx/screenshots/workspaces/ccna-guidance-guided-practice-dark-100.png)
- [Assignment](../.build-artifacts/native-runs/run-jhkb64jx/screenshots/workspaces/ccna-guidance-assignment-dark-100.png)

The approved reference set is `learning-lab-reference-05ICWH` in Downloads:
introduction/try-example/assignment are proposed mockups; image.png/image(1).png
are older application screenshots showing the readability problem. Never relabel
these images as new implementation.

Final packaged rollout evidence (not mockups):

- [Subnet Introduction: aligned blocks](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-ccna-magic-blocks-introduction-dark-100.png)
- [Terminal guided practice](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-lpic1-file-navigation-example-dark-100.png)
- [Service configuration practice](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-lpic2-service-example-dark-100.png)
- [Script Introduction](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-shell-functions-introduction-dark-100.png)
- [Independent script Assignment](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-shell-functions-assignment-dark-100.png)
- [Light theme, 200%: actual versus expected results](../.build-artifacts/native-runs/run-84so98rl/screenshots/workspaces/teaching-shell-functions-example-light-200-scrolled.png)

See [rollout verification](TEACHING_ROLLOUT_VERIFICATION.md) for the candidate
identity, complete sample list, checks and acceptance limits. The inventory and
guide bundled at build time are retained; these final screenshot links were
recorded afterward without modifying the candidate/archive.

## Raw-record storage lessons

The COW unit retains real simulated LVM command practice alongside clearly named
simulator application updates and actual record readers/copies. Typed worked
operations distinguish inert command insertion from an explicit non-command
application action. Independent requirements stay a literal checklist; no helper
is taught as a Linux utility. See [COW teaching/evidence](LVM_COW_LESSONS.md).
Development-native inspection is technical evidence, not novice acceptance.

## Reusable component and content API

- `lead`, `section`, `prose`, `rich` and `monospace` prelayout left-aligned text;
  column layouts cannot stretch their word spacing.
- `concepts` uses reviewed essential concepts/context where supplied. Original
  longer explanations remain in optional detail, rather than being discarded.
- `worked` and `worked_rows` pair concrete example values with reasons. Long
  command/configuration lines scroll horizontally so visual wrapping does not
  masquerade as required newlines. Small worked scripts reserve their measured
  font height; long scripts keep a bounded scrollable reference.
- `practice_step` renders the lab-specific action and observation next to its
  workspace controls. An expected refusal or a harmless navigation change must
  be described as such; a generic “repair the changed setting” fallback is not
  acceptable for every experiment.
- `assignment` uses reviewed checklist clauses without changing their required
  characters (presentation whitespace may improve readability). Independent
  starters and outcome graders remain in the original scenario/engine.
- `range_blocks` gives equal-purpose diagram cells equal width/padding and aligned
  labels, reflowing whole cells at narrow widths.

These are ordinary native egui components and immutable bundled prose, not a
plugin system or a new interpreter. The overlay does not execute actions, alter
scenario IDs/versions, migrate progress or award credit. Its review statuses are
separate from the learner’s actual completion history.

## Validation and delivery boundary

Use focused behavior/layout tests and representative native Fedora screens for
terminals, script/configuration editors, topology/allocation and file operations,
including a light/dark theme and enlarged text. Reuse valid earlier simulator and
progress evidence. All heavy commands use MAIN `tools/dev.py`, one common lock,
jobs2/test threads2 and disk-backed artifacts. No full-suite repetition per lab.
Deliver a separate review candidate with protected progress; no installed-app
replacement, push, merge or public release is authorized for this rollout.

## Compact worked commands for the backup-selection capstone

UI-only presentation entries may opt in with `command_picker: true` (default
false). The new backup-selection capstone alone uses it; the existing four
backup lessons and other tracks retain their appearance. The native picker
browses all explained commands while keeping the terminal visible. Selecting
only changes the remembered example choice; Insert command fills the practice
draft and never runs it. Experiment and repair commands come from the existing
typed lesson data and stay visible beside their guidance. Shared `code_value`
reserves native font/scrollbar space for long commands without shrinking text.

## Capturing the scannable assignment layout

Capture checks must inspect the actual goal and separate requirement labels, not
search for a removed all-in-one objective paragraph. The optional native capture
adapter records the same Label galleys and responses used by the renderer. A final
paint check must independently witness every required row with readable fonts,
nontransparent glyphs, its own clipping rectangle and no opaque overlap.
Unavailable or discarded viewports earn no evidence; wait for a real repaint.

Use measured vertical distances for screenshot scroll positions. At200%text,
long requirements may need several overlapping captures; never shrink the lesson
or omit requirements to fit an old screenshot count. Keep an explicit inventory
and bounded scrolling, then verify the union of actual visible rows. Focused
regressions cover this behavior; a headless paint test is not a native Fedora PNG,
keyboard-device acceptance or a novice comprehension study.

For navigation and capture, keep the actual lead-label response separate from the
complete goal-plus-requirements extent. A tall task must remain scrollable at200%;
a visible heading is a valid navigation anchor, not evidence that all requirements
were visible in one screenshot. Whole-task evidence must retain actual row/glyph
coverage. Synthetic clicks must keep the measured window size and settle after
scrolling before pressing the intended existing control.

## Recursive-tree adaptation (2026-11-06)

The two new ordinary-directory assignments reuse the accepted terminal components,
not the subnet worksheet. Essential mask/copy/removal explanations precede separate
worked practice commands; specific action/effect captions replace generic disclaimers.
Actual output remains separate from observations; one Check work shows only failed
outcomes first with all outcomes optional. Source-held state regrading clears stale
success after file/mask changes. The actual R20Enter/Check/reset/save-read flow and
R18widget bounds/focus tests support this adaptation, not novice-comprehension
acceptance. Local actual screenshots:

- [Private-copy introduction](../.build-artifacts/native-runs/run-pi0fxitr/screenshots/workspaces/tree-private-introduction.png)
- [Separate guided terminal](../.build-artifacts/native-runs/run-pi0fxitr/screenshots/workspaces/tree-private-practice.png)
- [Independent complete task](../.build-artifacts/native-runs/run-pi0fxitr/screenshots/workspaces/tree-private-assignment.png)
- [Broken permission feedback](../.build-artifacts/native-runs/run-pi0fxitr/screenshots/workspaces/tree-retire-broken-feedback.png)

These R20images precede the small command-caption refinement. Final packaged images
will be identified separately; none is a proposed mockup or owner novice study.

## Configuration/scan continuation

The opt-in LVM configuration workspace reuses the bounded multiline editor with
Tab focus navigation; old script/code editors retain Tab indentation. Save stores
a draft, Run scan derives actual virtual reads, and Check answers grades each
supplied inventory. A previous run stays explicitly out of date after edits.
Worked fills edit only separate example drafts; they neither Save nor Run.
Renderer-owned section numbering must not be duplicated in content titles.
Use one curated practice sequence, falling back to source exploration only when
no curated entry exists. Source boundaries/references remain optional help.

## LVM-backed mount configuration

The new unit uses the shared font/padding-based multiline editor with Tab focus
navigation, explicit configuration Save, actual current-mount rows and the
existing investigation terminal. Introduction teaches the six fields before
editing; Try example fills only its own draft; Assignment keeps intentional
saved/live mismatches and original-data requirements. Concise initial feedback
leaves every complete objective result in optional expansion. Earlier workspaces
retain their presentation. See [unit evidence](LVM_MOUNT_LESSONS.md); current
Fedora development evidence is not independent novice acceptance.

## AutoFS configuration and investigation

The local-file AutoFS unit uses the same three-screen standard around a native
file picker/editor, explicit Save file, actual backing-mount rows and a terminal.
Annotated example actions remain beside the relevant editor/command; filling a
file or inserting a command never applies it. Expected failure labels differ
from actual run status. Direct/master refresh, indirect new-lookup behavior and
existing-mount expiry are explained before assessment. An idle check is clearly
a simulator activity, not a Linux command. Independent requirements are literal
and worked solutions stay closed by default. See [AutoFS lessons](AUTOFS_LESSONS.md).

## Existing-mirror assembly practice

The clean RAID1 assembly capstone reuses the configuration editor, centered command
entry and bounded actual-output terminal. Teach member identity and saved versus
active policy first; retain an expected failed attempt and a wrong-future-UUID
experiment in the separate example. Fill never saves, Insert never runs. A native
sample-data view is labeled simulator inspection; do not invent a Linux command.
Independent UUID answers remain absent from the default starter/checklist. See
RAID_ASSEMBLY_LESSONS.md for the scoped implementation/acceptance evidence.

## Bounded prerequisite additions

The mixed-IP bridge adds `scenarios/presentation/lpic1_foundations.json`, version1,
for two explicit Introduction-only notes (`lpic1-host-address` and
`lpic1-host-routing`). The four base per-track review records remain unchanged
for old lessons. The native loader and inventory validate the note schema,
bounded size, exact IDs, separate worked rows and source links; these notes do
not become tracks/labs or change practice, requirements or grading. Keep readers
aware of this versioned object rather than assuming every presentation JSON file
is a lesson array. See [mixed-IP teaching](MIXED_IP_LESSONS.md).

## Native request/resource lessons

Controller practice keeps the request editor, actual response and independently
graded network outcome distinct. Derive guidance from current request parts and
resource state: introduce a body repair before it succeeds, and retirement before
the old resource is deleted. Changing a draft clears current success; a2xxresponse
alone never completes a task. Worked and assignment histories stay separate. See
[bounded REST teaching/evidence](REST_CONTROLLER_SUBSET.md). Existing screens and
shared control metrics remain unchanged. Assisted technical checks are not a novice
study or full REST/auth acceptance.

## Client-name diagnosis capstone

The opt-in combined source/search workspace places a current-draft, selected-sample
diagnostic beside its explanation and the virtual file editor beside repair guidance.
Configuration rows scroll horizontally so a soft wrap cannot look like another hosts
record. The original Source/Search workspaces retain their appearance and behavior.
Expected requirements, actual command output, unsupported syntax, out-of-date output
and all-sample grading remain distinct. Only explicit Run lookups checks the assignment.
See [teaching and validation](CLIENT_DIAGNOSTIC_LESSON.md); no novice study is inferred.

## Network file operations

Use the supplied endpoint table and a live source-to-destination read/write line
beside native service/direction/login/file inputs, with the real bounded router
document editor when policy is part of the task. Transfer status and independent
case results have different meanings; a successful wrong-file copy must fail the
assignment. Separate worked files, progressive investigation, original/replacement
roles and permission failures teach the concept without exposing assignment
answers by default. See [network-transfer teaching](NETWORK_TRANSFER_SUBSET.md).

## Campus architecture assignments

The new campus unit keeps the actual editable cable table, source-held equipment
roles and a selected-outage topology rather than turning architecture into a quiz.
Worked guide equipment, contextual repair/observation actions and independent
protected-cable requirements remain separate. Current physical and required-role
paths are distinct observations; explicit Check answers evaluates all stated cases.
Endpoint labels reflow above full-width native selectors when the shared font
metrics show that side labels would squeeze important device/port names. The
component restores internal keyboard focus after reflow or Escape, while respecting
Tab, deliberate pointer actions, other controls and disabled/modal states. No OS
focus or theme changes are made. See CAMPUS_ARCHITECTURE_SUBSET.md for authorship,
source boundaries, tests and the exact pending/observed native status.

## Internal alternatives in a topology task

The leaf/spine task opts into the existing cable workspace. Teach its rack/leaf/
spine concepts and a separate worked topology first; guide actual commissioning
and a recoverable missing-link experiment; keep the independent new-rack endpoints
unplugged. A selected rack pair/case changes observation, never grading scope.
Show actual distinct spine counts beside the observed paths and stated requirement.
After checking, summarize cases and show short failed actual/required counts;
keep complete path diagnostics expandable. Do not repeat a full paragraph for
every failed pair or imply one working path meets a multi-path requirement.
See [scope and actual screenshots](LEAF_SPINE_SUBSET.md). The same model/UI retains
old campus behavior; no all-track redesign or unsolicited solution is introduced.

## Standalone-sharing adapter

The SMB unit reuses this hierarchy with an appropriate server workspace:
accounts and Unix investigation for identity repair, a real multiline config
editor for policy, and selectable fresh client read/write trials beside it.
Essential concepts/worked guide values precede practice; its actual observations
and independent task requirements remain separate. Worked fill is example-only
and never saves, applies or executes. Failures name the access layer or actual
resolved file; default assignments do not expose a canonical configuration.

Related account rows reserve the same font-derived status column. The wide
minimum sums shared label/status/button/spacing/padding measurements; compact
windows stack without shrinking text. Single-line fields use the shared native
centered edit geometry; editors stay top-aligned. A Samba-only stable parent
gutter keeps the workspace from shifting as nested scrolling appears. Other
tracks retain their rendering. [Scoped native evidence](SAMBA_SHARE_SUBSET.md)
and CHECKPOINT distinguish implementation, package acceptance and untested
novice/physical-input/screen-reader/platform behavior.

## Virtual-workload and routing-context tools

The three-lesson **Virtual workloads and routing isolation** unit uses the same
three-view standard, with source-owned facts beside meaningful configuration tools.
Host cards show supplied kernel/runtime and planning budgets; placement selectors
and a host-outage experiment lead to a whole-task check. The vNIC activity reuses
the VLAN table and real Apply/Check/broadcast inspector. Routing-context selectors
show fixed peer addresses and distinguish requested from actual replying endpoint.
Guide examples have their own names, drafts and checks; table names or placements
may differ from the reference when behavior is correct. Configuration edits clear
stale success and actual observations. The opt-in dropdown-focus helper retains
Escape focus without intercepting Tab; existing screens do not change. See
[VIRTUAL_ISOLATION_SUBSET.md](VIRTUAL_ISOLATION_SUBSET.md) for actual screenshots,
finite Fedora checks and fidelity/acceptance limits.

## Guest-image package/module workspace

The source-bound guest-driver page opts into shared aligned form rows/dropdowns,
byte-bounded fitting single-line input and stable root scroll gutter. Wide windows
place native package evidence/actions beside module investigation; narrow windows
stack them. Only this new workspace opts into fitting_input; existing multiline
editors and prior tools retain their dimensions. Expected guide observations,
actual terminal/native-package history, unchecked state and per-outcome checks are
separate. Whole-task checks include current versus unloaded requests and protection
requirements; model helpers are not learner Linux commands. See GUEST_DRIVER_SUBSET.
