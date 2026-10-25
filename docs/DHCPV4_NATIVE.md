# Native DHCP workspace

Native component stage2,2026-10-25. Backend gates are recorded in
[DHCPV4_WORKSPACE.md](DHCPV4_WORKSPACE.md). The current14 focused native-egui
interaction tests and the actual Fedora technical-fixture flow pass. Full225 UI
checks passed before the compact render refinement;14 scoped checks, all-target
Clippy and renderer checks close that refinement. No lesson is registered and
no activity/objective count changes.

The source handoff began atb5f5f83 with an exact current122/M104 UI overlay and
frozen backend. Integration checked all eight source hashes and all four existing
UI base hashes before publishing any file. Current capture/case/getopts and real
Writer7/Writer8 history code remains intact. Only the incremental component is
added; whole older shared files are never substituted.

One existing-scroll workspace contains the full task, a compact selected-client
logical path, actual state readouts, explicit client controls, a focused policy
editor and one Check work/feedback area. Policy and client tools share columns
when wide, then stack when narrow. Server-binding detail is available on request;
client-held parameters remain visible. Apply is secondary and Check is primary.
It uses shared topology, font-derived height, singleline, label/value and dropdown
components, with no text-size reduction. No terminal, dashboard, fake
Linux/IOS syntax or new global app setting is introduced. Introduction and the
separate worked example are optional; Challenge displays the full assignment and
hides optional guidance. Help has distinct lab/mode IDs, so an opened Learn
explanation does not leak into Challenge.

The client selector changes context only. The diagram identifies the supplied
logical server, physical segment/direct-or-relay requirement and selected client's
current state. Readouts distinguish Offer/token/deadline/offered parameters, server
binding and client-held lease with actual parameters, remaining time and explicit
T1/T2/expiry timestamps. Window labels do not claim that automatic renewal occurred.
Policy and held values are shown separately; a served ACK does not prove routed
connectivity or DNS response. Fixed path availability and relay-facing addresses
are labeled supplied facts and cannot be edited as a shortcut.

Discover, Request, Renew, Rebind and Release use typed core actions on the selected
client. Request reads the actual current offer token. Discover is unavailable for
a client that already holds a lease; Request needs an offer, and Rebind needs its
actual time window. Renew/Release need a current lease. Advance has bounded editable
seconds and moves only simulated time. Every outcome/error is plain text, not raw
enums, booleans or a color-only state. Action feedback is revealed through the
existing shared feedback helper so a no-reply diagnosis cannot remain below a long
form unnoticed.

The selected pool editor covers network/prefix, inclusive range, router, DNS,
lease time, exclusions and reservations. DNS and potentially long lists use
collapsible structured rows; exclusion/reservation bodies have bounded inner
scroll areas. New pools receive a unique generated pool-N identity; this version
has no arbitrary pool-ID rename field. Pools and relay records can be added or
removed from the draft. Relay fields refer to the selected client's segment.
Well-formed wrong values remain editable, and no desired goal values are used to
fill assignment fields. New/reset selection identifies a current task client and
its current matching policy scope, not a required answer address/option.

Apply parses and changes policy only. It never discovers, requests, renews or
rewrites held parameters. Check work does not apply the draft: it independently
checks pending policy plus current clients and bounded fresh policy contexts.
One freshly guarded current_dhcpv4_checks vector supplies both feedback and
completion; current_solved wraps that helper. Pending or malformed drafts return
an Apply failure before probes. No persisted grade cache is used. The
example has its own simulation/draft/context and actual controls. Filling its
annotated Apply step changes only policy fields; runtime steps use the visible
client controls. The supplied experiment and repair are explained and performed
explicitly. Example completion never earns assignment credit.

Raw edits retain the backend's64-byte fields/aggregate256KiB admission. TextEdit
accepts a bounded temporary edit, then an over-limit field/aggregate is rejected
whole with an explicit notice and previous draft retained; no accepted partial
prefix is silently published. Valid malformed text stays saveable and Apply
returns a bounded diagnosis without changing simulation. Input IDs include lab,
assignment/example scope and immutable pool/relay-segment/field, preventing cross-lab/example and cross-relay-record undo.
Auto-focus waits for pointer release. All workspace mutation, Fill, Apply,
exchange, time and Check controls are visibly disabled behind Reset confirmation,
Settings, Appearance samples or data modal; keyboard Check also requires enabled UI and a focused
actual DHCP text field.

The developer-only closed `DLMS_SMOKE_STAGE=dhcpv4-workspace` uses30000..30095.
Normal non-smoke launches ignore this selector. The private technical fixture is
loaded only through the smoke/test helper and launched normally; it is not bundled
course data or a solved State. The controller clicks/types actual controls for:

- Introduction, separate fill-only policy, Apply, Offer-only Discover, Request,
  example Check, Release experiment and real reacquisition repair.
- Assignment's wrong relay, typed router/DNS/relay repair, pending and Offer-only
  failure, actual Request and positive current/fresh checks.
- Malformed duration rejection and valid repair; actual Light150 and Dark200
  duration typing, Apply and Renew; explicit time advance, Challenge, exact raw
  save/readback, pending-draft resume and Reset Cancel/Confirm.

Geometry/zoom/pixels-per-point are read before Context data closures. Actual
geometry-phase code and asynchronous Finish hold are exercised by a passing
SmokeTick regression. Click press/release acknowledgments are completed before
rechecking controls that may become disabled after Request/Release. Bounded
visibility/scroll/focus helpers are reused. Native Finish holds its phase until
Close completes. The script/engine APIs are never used to install a solution.
Existing normal progress save/load is used only within the existing private smoke
profile; no arbitrary path or personal/default profile is introduced.

Fourteen passing UI tests cover Introduction/practice/actions, wrong/partial/repair,
stale options, malformed raw save/resume and byte rejection, all four modal guards,
mode preservation, selected Reset, separate field IDs/Undo, deferred focus,18
width/scale/view cases, real capture-phase geometry/Finish, client selection and
explicit timers. Added checks require side-by-side/stacking behavior, retained
text/focus/Undo across reflow, and long48-byte pool names/64-byte raw fields at six
sizes followed by real editing. Raw values are retained; dropdown visual truncation
is not saved-data truncation.

Integration first caught a private fixture helper and unused import. Narrow200%
layout then exposed width calculations made before actual label allocation. The
pair component now reserves the label and uses the remaining row space, stacking
if needed. A Grid for each single row also performed an invisible first measurement
when switching columns; its disabled TextEdit surrendered focus. Simple reserved
rows avoid that measurement, with explicit pre-layout scope/pool/relay field IDs.
Wrapped dropdowns use remaining-before-wrap width and truncate their display,
preventing adjacent labels from widening the whole parent. No coordinate offset,
font shrink, modal bypass or relaxed bounds assertion is used.

A relay Undo test initially assumed each Undo changes text. A copied egui Undoer
showed a cursor-only checkpoint above the prior edit. The test now permits at most
three real Undo events while requiring the original text and checking the other
relay every frame. Product Undo is unchanged. Intermediate compile/test failures
remain local evidence; expectations were not replaced with observed wrong output.

The first actual Fedora flow passed17 captures but visual review rejected repeated
readouts pushing the policy below the fold. After the compact refinement, the
second actual flow again passes17 captures/exit0, with split100%, stacked150Light/
200Dark, actual typing/Apply/Renew, wrong/pending/offer-only checks, separated
practice, explicit clock, raw save/readback, warm resume and Reset Cancel/Confirm.
The actual tiled window was946×929 pixels; requested1200×850 is not claimed as an
actual larger-window run. Headless layout tests cover the additional widths.
Normal Wayland presentation policy was used; the diagnostic override was unset.

Private result receipt0102502fffdffcb657464a665668eeb98499710ff3e8c25144e4e250974f3099
pins source/binary and images. This is an identified development component test,
not a packaged/cold-process, novice, physical-keyboard, screen-reader or other-OS
acceptance claim. The normal catalog remains122. Authored DHCP teaching and
normal packaged workflows require their own subsequent gates.
