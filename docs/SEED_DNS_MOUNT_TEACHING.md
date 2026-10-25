# Original DNS and mount teaching adapters

Implementation2026-10-24, integrated/model and headless UI verified2026-10-25;
actual native acceptance on installed30d75e7 now passes via process-local XWayland. These are introductions and independent practice for existing
`lpic2-dns` and `lpic2-storage`, not new lessons. The116-lesson catalog, IDs,
versions, initial assignment States, Goals, solutions and unit membership stay
unchanged. No evaluator revision, dependency, host action or coverage promotion.

The modules `seed_dns_teaching` and `seed_mount_teaching` use explicit small typed
practice snapshots. They reconstruct only the original in-process Dns or Mount
state for existing commands. They do not substitute AuthoritativeDns or
PersistentMounts and do not embed generic State deserialization. A private scalar
text/transcript utility is shared; there is no plugin registry or new execution
framework. The original native assignment editor and terminal remain in use.

DNS practice supplies `mirror.lab.test`, saved and served198.51.100.4 at serial41.
Native Save prepares .44/42; validation and reload are explicit, then dig observes
the serving .44. The experiment saves .45/43 without reload. Serving stays .44/42,
but both existing Goal rows fail: the saved address is wrong and the saved serial
differs. A query alone is not evidence that the whole configuration is correct.
The worked repair saves .44/44 and reloads; other valid serials/orders and any
state satisfying the unchanged Goal remain acceptable. Restoring the already
served address and loaded serial can also restore current success without a new
reload event. No history credential or RFC1982 wrap rule is added.

Real DNS compares ASCII letters without regard to case. This tiny existing seed
uses exact supplied lowercase map keys; unknown/case-varied keys retain existing
behavior. New teaching states that limitation plainly. The annotated
`named-checkzone lab.test` action is explicitly simulator-only: real BIND also
requires a zone-file argument. No general BIND zone-file parser or cache is implied.

Mount practice supplies known /dev/vde2, wrong saved /dev/vdf2,2000 MiB capacity,
1850 used and450 disposable cache, mounted read-only. Removing only that marked
cache leaves1400/2000 MiB,70%. The existing goal threshold remains80%, with no
exact70% or unmount-history predicate. The experiment unmounts after success;
only the current mounted/read-write row fails. Mounting with the correct saved
pointer repairs it. An alternate can unmount/mount before cleanup. The existing
terminal's permissive saved-pointer token remains distinct from stricter native
Save validation. No real device, inode, filesystem repair or boot action exists.

Teaching-bound assignment import/current checks pin only what existing commands
cannot change: the authored DNS record key set with loaded keys a subset, and
Mount's capacity plus known device. Mutable addresses/serials, used/cache,
configured pointer and mount flags are not pinned. Unrelated legacy custom packs
without these teaching fields retain their old validation. The original State and
Goal definitions and engine grammar are untouched.

Each saved practice has its exact authored immutable fixture marker, independent
current snapshot, raw editor and command drafts, bounded transcript, view,
selection and nonauthoritative checked flag. Decode and every operation compare
against Lesson.fixture. Coincident mutable example/assignment values are valid;
they do not identify a context. Invalid raw address/serial/device text is retained
and Save is atomic. Terminal mutations refresh only previously clean editor
drafts. New attempts open Introduction; older attempts missing optional metadata
first open Assignment. Loading does not add or rewrite metadata. The M87 pending
assignment-draft guard stays unchanged. Selected reset and Reset example only
have separate scopes; canceled/failed restore never prepares replacement examples.

Introduction, Try example and Assignment are skippable native views. Each example
action explains its purpose before Insert. Inserting settings or a command fills
only the relevant input. Save and Run are explicit; example checks derive current
existing Goal rows and never grant assignment credit. Assignment uses the old
native Save fields and Check work. Example/assignment command and editor IDs are
separate. Inactive reset/settings/theme/data states disable execution and defer
focus. Shared font-derived controls and natural transcript scrolling avoid a
fixed empty pane. Challenge retains the full assignment with optional help.

New strings/sequences are bounded during typed construction; direct APIs also
validate before cloning. Practice has no unbounded map. Raw fields<=256bytes,
command4096, transcript<=32 entries/8192bytes each/131072bytes aggregate, retained
new error/notice messages<=512 UTF8-safe bytes in fresh allocations. Lesson
concepts<=6, default actions<=12, experiment<=4, repair<=6. Existing tagged Action
Content can already be buffered under pack2MiB/profile16MB caps; no pack-wide
streaming claim. Worst legal compact/pretty learning roundtrips have source tests
below1MiB; these bounds checks pass in the combined model gate.

Proposed focused checks cover original-nine immutable payloads, old/new first
open, known DNS keys and disclosed case boundary, explicit simulator-only command
annotation, correct serving-versus-saved experiment, independent mounted state,
alternate methods, invalid Save atomicity, literal/aggregate decode bounds,
fixture and physical identity pins, mutable coincidences and exact bytes after a
failed save. A full synthetic profile test retains actual6/7 main history, creates
real current8 outcomes through the public VM, preserves all116 contexts and both
unfinished seed drafts through save/load/export/inspection/confirmed restore and
selected reset. Inspection's known two-field refresh of older script example
results is independently computed; other fields remain exact. Native headless
source tests exercise real Insert/Save/Run, experiments/repair, raw-draft Undo
isolation, inactive Enter and scale/view reflow. Combined MAIN941 model/integration checks pass. Full UI R1 is179pass/1fail: a
test clicked offscreen popup options without proving selection. A real-popup
scroll/click correction and stronger pre-experiment completion assertion preserve
all expected outcomes; five scoped seed checks pass. Native controller adds three
passing checks (eight focused total), actual native capture remains pending.
Clippy R1 found test-module placement only; moving it preserves behavior and R2
passes. Original failures are retained, not relabeled.

Primary references checked2026-10-24: [RFC4343](https://www.rfc-editor.org/rfc/rfc4343.html)
for real DNS ASCII case comparison, and current official
[BIND9.20.30 manual](https://bind9.readthedocs.io/en/stable/manpages.html#named-checkzone-zone-file-validation-tool)
for the zone/file arguments. Existing original scenario references and teaching
limitations remain. No upstream code or manual prose is copied. This adds no
claim of complete BIND, Linux storage, LPIC coverage, novice, accessibility or
Windows/macOS acceptance.


M94 rendered follow-through: installed97b's full XWayland flow reached both
Finish checks and33 captures but exited101 because the controller advanced past
its final step before native close completed. The failed run is retained, not
accepted from its earlier PASS text. A terminal-step hold and repeated actual
close-phase regression fix the controller only. Nine scoped UI/controller checks
pass; all16 model tests pass with stronger negative-feedback assertions.
Actual screens also exposed duplicate BIND disclosure and positive-goal wording
after Needs attention. The fixed caption remains, outer action explanation now
teaches validation purpose; false practice rows diagnose current saved/serving/
serial or capacity/device/mount problems. Engine booleans, guards and passing
messages are unchanged. New actual native acceptance remains pending.


M95 installed follow-through: clean30d75e743ac9 passes identified97b upgrade,
content/desktop/hash checks and uninstall/reinstall with exact original-nine
synthetic progress retained. The full two-seed native flow exits0 after33 captures
(run-iksgx1_d); root reviewed actual negative feedback and light150/dark200
focused fields. Wrong/partial/alternate/current checks, separated unfinished
drafts/save/readback/navigation resume, reset and Challenge all pass. This is
injected native input, not a novice study, cold-process or physical-device test.
Ten additional process-local XWayland startup/readbacks pass10/10. Default
Wayland first launch times out15s (0/1), again in EGL/Wayland dispatch; its cause
is not cured. No desktop/theme/service/security changes. Exact identities and
hashes are in INSTALLATION.md; prior failed97b flow remains retained.
