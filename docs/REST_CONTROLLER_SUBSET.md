## Current milestone441 R2 —2026-11-12

441labs/173units: CCNA104, LPIC-1 145, LPIC-2 112, Bash80. Two REST assignments
follow JSON inventories. Corrected candidate passes exact selected identity,
protected upgrade, installed165-action/26PNG flow and actual NFS cold reopening.
Readable errors resolve R1review finding; model/grading/progress remain unchanged.
See [checkpoint](CHECKPOINT.md) for exact launch command, hashes, tests and limits.
All439prior scenario values/172units preserved. Full curriculum/platform/device/
novice acceptance remains open. Continue the existing client-DNS roadmap; its current
private preparation is uncompiled and earns no new lab/coverage credit. Older entries
below are historical source/package records.

# Simulated REST controller — capability in development

This is the existing CCNA6.5 roadmap slice, following JSON inventory/VLAN practice.
Source catalog441labs/173units includes two new registered lessons. Model9/session6/
authored4/registry1tests pass; UI/native review and a separate candidate are in
progress. This is partial practical coverage, not complete objective coverage. No new dependency or host action.

## Boundary and resource schema

`rest_controller` is a pure finite request/resource reducer. It never opens a
socket, starts a server, sends HTTP, reads host files or invokes a command. Resource
paths are names in an in-memory map, never filesystem paths. An immutable source
defines one/two Ethernet switches, at most eight endpoints and twelve named access
port bindings. Applied resources project through the existing typed JSON inventory
adapter into actual simulated Ethernet/VLAN behavior. Trunks/equipment stay supplied.

`/profiles` lists or creates named access profiles. Each `/profiles/NAME` has required
`up` (boolean), `vlan` (integer1–4094), optional `description` (printable string up
to160bytes). `/bindings` lists fixed port resources; `/bindings/NAME` has one required
`profile` name. A missing profile or an unsupported VLAN on a bound switch refuses
the entire change. Resource names use lowercase ASCII letters, digits and hyphens,
at most32characters. Names, descriptions and errors never become instructions.

## Defined request behavior

This models method/path/Authorization/Content-Type/body metadata, not HTTP wire
parsing. Field names are fixed native controls. Methods are case-sensitive; media
types and Bearer scheme are case-insensitive. Only the documented exact media
types, without parameters, are accepted. No URL/query/percent-encoded route, host,
arbitrary header, TLS, cache, redirect, conditional request, Basic/OAuth or Cisco
device API is claimed. Unknown methods return501; incompatible known methods
return405withAllow; unknown resource names return404. The origin refuses a GET or
DELETE body rather than pretending to define its meaning.

| Method | Resource | Result |
| --- | --- | --- |
| GET | collection or existing item | 200withactual current JSON; read-only |
| POST | `/profiles` | create body with additional `name`;201withLocation |
| PUT | profile item | replace full representation, or create it;204or201 |
| PUT | existing binding | replace its full representation;204 |
| PATCH | existing item | JSON Merge Patch;204 |
| DELETE | unused profile | remove it;204withno body |

Duplicate creation and referenced deletion return409. Bad JSON or duplicate keys
return400; wrong media type415; a valid JSON value that violates this origin's typed
schema or network constraints422. Every failed change is atomic. No partial update
leaks into the network. A successful no-op reports no state change. HTTP success is
request feedback, never assignment completion; the latter must grade current state.

`Bearer lab-reader` and `Bearer lab-operator` are supplied synthetic identity labels,
not secrets or real authorization infrastructure. Missing/invalid identity gives401
with a Bearer challenge; reader mutation gives403. GET is allowed for both. Do not
claim this implements token issuing, encryption, OAuth or secure credential storage.

## PATCH, bounds and compatibility

The original bounded JSON Merge Patch implementation updates object members,
removes members named with null, and replaces non-object values/arrays as whole
values. The final result must still satisfy the flat resource schema: an array/null
cannot become a profile; removing required up/vlan/profile refuses atomically.
An omitted optional description remains under PATCH and disappears under PUT;
PATCH description:null removes it. Full resource description:null is not a stored
description in this origin. Duplicate keys refuse at every depth rather than choosing
an ambiguous last value. JSON itself supports more forms than this resource schema.

Request body4KiB, method16bytes, other metadata96bytes each; parser depth8and128JSON
values, profiles12, bindings12. Bound source maps reject duplicate identifiers.
Each request performs finite parsing/map operations and one finite Ethernet
projection; there is no script evaluator or unbounded loop. Responses are transient,
serialize-only, and must never deserialize as completion authority. Sessions replay at most64state-changing successful requests/16KiB against authored
source, not a deserialized resource snapshot or response. Example and assignment
drafts/history remain separate; current grading reruns the actual network. Existing
progress schemas/evaluator19 and all439prior scenario values remain unchanged.

## References and evidence

Primary sources checked2026-11-13UTC; original implementation and original examples,
no RFC code or pseudocode copied:

- [RFC9110 HTTP semantics](https://www.rfc-editor.org/rfc/rfc9110.html).
- [RFC7396 JSON Merge Patch](https://www.rfc-editor.org/rfc/rfc7396.html).
- [Current CCNA1.1 objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),6.5.

Focused tests in `tests/rest_controller.rs` cover real network projection, equivalent
PUT/PATCH, creation/binding/retirement, auth/status distinctions, malformed/typed
errors, optional/required deletion, atomicity, resource bounds and escape-shaped
input. Record actual execution separately; test source alone is not passed evidence.
The two native request lessons are registered: ccna-rest-change-request and
ccna-rest-profile-handover, after JSON network inventories. Focused core9/session6/
authored4/registry1, UI scale/feedback/capture3, popup1, keyboard1and guidance-order1
pass. Development-native R23completed165actions/26PNG at100/150Navy and200Light.
Actual Introduction, Assignment, guided JSON repair before the answer, retirement
before deletion and enlarged light fields inspected. R11/R15failures remain: egui's
last popup option was clipped4points; a bounded capture-only wheel now targets the
actual menu column, with regression. No guard/deadline/tolerance was raised.

Actual refusal statuses, partial results, equivalent PUT/name sequences, broken-
again results, independent example/assignment states, Challenge and Save/readback
pass. A response is not persisted or invented on reopening. No physical narrow
native window or dedicated two-process REST reopening yet. Inclusive Python1211, stronger focused4, all-targetClippy and strict coverage/
inventory pass. A separate441candidate is next;439R1remains the latest verified
package. No full REST/auth objective, platform or independent novice acceptance.

Representative actual development screens (private ignored evidence):

- [Introduction](../.build-artifacts/native-runs/run-z2o2veap/screenshots/workspaces/rest-update-introduction.png).
- [Guidance before the JSON answer](../.build-artifacts/native-runs/run-z2o2veap/screenshots/workspaces/rest-update-json-guidance-before-answer.png).
- [Independent handover](../.build-artifacts/native-runs/run-z2o2veap/screenshots/workspaces/rest-handover-assignment.png).
- [Retirement guidance before deletion](../.build-artifacts/native-runs/run-z2o2veap/screenshots/workspaces/rest-handover-retire-guidance-before-delete.png).
- [Light200% centered fields](../.build-artifacts/native-runs/run-z2o2veap/screenshots/workspaces/rest-update-light200-authorization.png).

## Packaged R1 review and readable-error refinement

441R1passes protected439→441lifecycle, installed165actions/26PNG, actual two-process
NFS reopen/repair and final identity readback.2805selected files/104704492bytes retained
before R1freeze ENDED. Receipt status deliberately leaves feedback readability
pending: the raw JSON line hid expected boolean horizontally. R1candidate and all
proof remain immutable; it is not the delivery-ready441candidate.

The REST-only renderer now decodes an error into a wrapping sentence and keeps its
actual raw API response behind optional detail. Wrong up/VLAN types receive plain
language; request-limit errors also wrap. Success GET/resource output remains
actual data. No response becomes grading authority. Current5workspace UI regressions
pass, including actual painted text without opening/scrolling raw JSON, both themes,
normal/enlarged sizes and request-limit feedback. One-job all-targetClippy passes.
All model/engine/progress/scenario/course source hashes match retainedR1exactly;
only REST renderer/test/capture source changed. Separate441R2is being prepared.
No new curriculum, scoring requirement, dependency or user-data migration.

R1selected bytes and error screenshot are preserved at
`.build-artifacts/continuation/resume-20261112/next-rest-api/candidate-r1`. Its NFS
cold write/read PIDs2694662/2695165exited0in68.392/31.022seconds; restored unfinished
and repaired150%Light screens were inspected. R1is a useful technical baseline,
not the final readable-feedback candidate. An in-flight review-metadata edit
invalidated PythonR41; the exact added prose was reverted. Current metadata now
matches retainedR1byte-for-byte. Keep failed evidence; rerun fixed inputs before
R2freeze. Source model/session/grade data remain unchanged.

The corrected source now passes all5workspace UI regressions, one-job all-target
Clippy, fixed-input Python1211and strict coverage/inventory. Current presentation
bytes match retainedR1; no model/data/grade/schema change. Core/session/content/
registry20remain validated against unchanged source. Separate441R2package gates
are next. Never reinterpret the immutable R1readability-pending receipt as R2proof.

## Corrected packaged result

441R2passes protectedR1→R2lifecycle, installed165actions/26PNG and actual NFS cold
reopening. Eight packaged originals and both restored/repaired NFS images inspected.
The complete wrapped boolean explanation is visible beside the body; optional raw
JSON is closed by default. See CHECKPOINT.md for exact identities/limits. R1readability-
pending receipt remains immutable; R2has its own frozen inputs/final readback and
selected retention. No full curriculum, remote HTTP, platform or novice claim.

- [Packaged readable error](../.build-artifacts/native-runs/run-_aw2yl1v/screenshots/workspaces/rest-update-type-error.png).
- [Packaged Introduction](../.build-artifacts/native-runs/run-_aw2yl1v/screenshots/workspaces/rest-update-introduction.png).
- [Packaged independent assignment](../.build-artifacts/native-runs/run-_aw2yl1v/screenshots/workspaces/rest-handover-assignment.png).
