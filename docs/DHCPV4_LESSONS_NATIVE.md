# Fixed three-lesson DHCP native acceptance proposal

Checked2026-10-25. The normal catalog contains all three lessons. All five
native-egui controller tests and15 DHCP workspace interaction tests pass;
actual Fedora rendering and packaged acceptance are separate gates. The current
native flow uses the real bundled scenarios and actual controls, not injected
successful simulator state.

The controller was independently source-reviewed before integration. Incremental
hooks were merged into the accepted compact workspace, retaining its shared
field components, stable focus scopes and responsive split/stack layout.

## Scope and real controls

`DLMS_SMOKE_STAGE=dhcpv4-lessons` selects31000 only with `--ui-smoke`. The existing
technical30000..30095 flow stays unchanged. The new closed range31000..31246 has
82 phases per lesson and one Finish phase. Finish requests Close and holds its
phase while the window closes asynchronously. Normal mode ignores the selector.
The actual native controller requires the three IDs to exist in the application's
catalog; it never appends practice scenarios. The cfg(test) helper also requires the normal registered IDs and compares the
authored fixtures; it never appends or replaces scenarios.

A bounded queue drives existing client/pool/action menus, typed fields, collection
headers, Add/Remove excluded-address/reservation buttons, Apply, explicit
Discover/Request/Renew/Release/Advance and Check. Menus use actual popup clipping,
bounded scrolling and separate hover/press/release frames. Fields must own actual
TextEdit focus before Ctrl+A/text events. Request uses the UI's actual current
returned offer, never an authored token installed into state. The queue rejects
unsupported pool/relay creation, toggle changes, different DNS list lengths,
more than4 exclusions/reservations and more than256 queued operations. It is a
fixed acceptance helper for these three tasks, not a generic authoring/editor
platform. Suspended queues have1800-frame/120-second bounds.

Passive geometry records cover existing menus and collection controls. Interaction
checks also exposed a real identity bug: including the collection count in a
collapsible header's identity closed the section when a row was added. Headers
now have stable identities across count changes and responsive reflow. Adding a
DNS, exclusion or reservation row focuses its new field without applying policy.
Completion keeps its result and next action visible; the worked explanation is
default-collapsed optional help. No panels or learner workflows were added. Native text editing and grading stay
unchanged.

No engine execution, policy replacement, reference simulation or solved State is
used as an input substitute. Typed authored actions describe the UI queue only;
policy equality assertions check the actual result of typing and Apply. Current
state and grading are read only for evidence. Metadata operations such as opening
normal activities, private save/readback, library filters, theme/scale and favorites
follow the existing fixed-controller pattern.

## Bounded evidence flow

For each lesson the controller visits:

1. Normal-sized dark100 Introduction, separate example and Fill-only proof.
2. Actual annotated-action selection, Apply/exchanges, Check, distinct experiment,
   exact declared failed rows, repair and separate assignment-state equality.
3. Full assignment and current initial failure; a plausible wrong attempt and
   partial outcome. The relay wrong attempt explicitly obtains a /16 grant after
   fixing delivery, proving an ACK alone is insufficient. Reset is a real
   confirmation action before the independent reference attempt.
4. Reference through actual fields/buttons and Check; pending router edit
   immediately invalidates completion, Apply leaves correct old held parameters
   while new-client checks fail, and restoring policy repairs them. The helper
   asserts all current rows pass in this particular new-client-only failure.
5. Real lease-duration typing, Apply and successful Check in Light150 and Dark200,
   with geometry settled after actual viewport/scale changes. Before labeling
   either capture, exact draft text and applied work-pool duration must equal the
   newly typed value:104/112,88/96 and104/112 seconds for the three tasks. Held
   leases must remain byte-for-value unchanged by this policy-only Apply. Actual
   content rect, pixels-per-point and zoom are logged; a requested window size is
   not asserted to have been honored by the window manager.
6. A malformed-but-bounded unfinished duration, normal private save and exact
   portable progress readback, real Back/card navigation, pending warm resume,
   Reset cancellation, selected Reset confirmation, full supported alternate,
   Check, Challenge and library return.

The warm-resume comparison retains every Attempt field. It explicitly projects
only the one normal `visited` update to the saved clock+1 and checks the clock,
rather than ignoring metadata or learning fields. Other attempts, favorites,
both drafts and selected example/assignment state are checked; selected Reset
must restore only that lesson's authored initial/practice fixture. This does not
claim a second OS process or cold reopen. Normal backup/progress provenance is
not replaced or relabeled.

Five passing headless tests exercise actual SmokeTick opening/geometry and
repeated delayed Finish; the first real example's fill/actions/experiment/repair;
the full capstone controller including real list creation, scales, pending save,
resume, Reset and alternate; and the relay controller including wrong-mask
feedback and repair. Headless tests inspect the controller's capture-phase return
values without pretending that egui emitted real GPU screenshots. The tests do not provide GPU screenshot evidence. Actual root-guarded native
execution, retained screenshots and render review remain separate gates. No novice, physical DHCP/routed/DNS service,
Windows/macOS or performance acceptance is claimed.

All Context geometry, zoom and pixels-per-point reads occur outside data/data_mut
closures. Context storage holds only bounded controller metadata and observed
private data; it is never authoritative simulator state or completion credit.

## Current guarded source evidence

-13 registered content checks pass (`m106-registered-dhcp-content-r1`).
-20 interaction checks pass (`m106-dhcp-native-actual-density-r9`):15 workspace
  plus5 lesson-controller tests, including partial/wrong results, actual list
  editing, alternate solutions, warm save/resume, selected reset and text scales.
-All-target Clippy and native development build pass. No full post-change UI
  suite is claimed; the earlier225-test full UI run preceded compact refinements.
-Controller scroll/click helpers wait for actual geometry, route inner scrolling
  through the field column and allow one input batch per rendered frame. These
  developer-only constraints prevent false clicks during egui measurement passes;
  they do not set field contents, scroll state or simulator outcomes directly.

The first actual native run exposed an oversized completion area at200% text on
the observed946x929 window with native density1.15. Default-collapsing its worked
explanation retains the complete result and Next action. A full three-lesson
controller regression now uses that observed logical viewport at100/150/200%
(`m106-all-three-observed-density-r10`). This is a geometry regression, not a claim
that a headless test produced a native-density GPU render. The failed actual run
and its screenshots remain retained.

Actual R2 passed the first two full lessons, then its capstone200% capture
required the complete operation notice and completion area to fit at once. The
application correctly allows scrolling. A passive result-summary rectangle now
lets that successful capture include objective/result/navigation together when
the older notice makes the whole area taller than the viewport. All grading and
failed-row visibility assertions remain intact. The20 observed-geometry checks
pass R10 and the combined23 DHCP/cold-controller checks pass R11. New150/200
focused-field captures inspect input/selection geometry, separately from result
captures. Actual final candidate rendering remains required.

All149 Python tooling guards and all-target Clippy pass. The catalog-wide actual
egui check covers all125 assignments at200% text/1920 and720 widths plus VLAN
typing/reflow (190.64seconds); four focused course/filter/font-metric checks also
pass. These are headless geometry/input checks, not new native PNG evidence.
