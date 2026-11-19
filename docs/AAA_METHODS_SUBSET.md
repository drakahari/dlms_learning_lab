# Ordered authentication methods: bounded foundation

Designed2026-11-09 for the existing CCNA5.8 AAA-denial roadmap gap, after local
access foundations. This contract and pure model add **no registered lab or new
coverage credit**. The verified358-lab review candidate predates this source.
Authentication, authorization and accounting remain distinct concepts.

## Supported meaning

An explicit ordered list contains one to four methods: the existing local-case
training database or a supplied named training identity service. The latter is a
case-sensitive metadata directory with explicit available/unavailable fixture
state. It is not RADIUS, TACACS+, LDAP, a server process or a wire-protocol engine.
Do not infer a real server's case, transport, timeout or encryption behavior from
this declared teaching boundary.

A responding method compares actual supplied account records and training words.
It returns an authenticated identity or an explicit unknown-name/wrong-word
rejection. Either response stops the list. Only unavailability advances to the
next method. Exhausting unavailable methods leaves an unavailable result, never
an authenticated identity or a credential rejection. Missing service definitions,
malformed input and invalid configuration are diagnostics, not simulated outages.
The trace names each actually evaluated method and result; no token is echoed.

A separate explicit local EXEC-authorization operation looks up an authenticated
identity's local record to return its configured level1or15. No matching local
record means that local authorization cannot provide access; an unauthenticated
attempt never receives rights. This models only the specified local profile
lookup. It does not implement remote authorization attributes, arbitrary command
permissions, a persistent EXEC session or any accounting mechanism.

Only supplied training words belong in these records. They are plain lesson
metadata and locally saved drafts, not personal credentials or Cisco hashes.
No crypto, host shell/files/accounts/services, sockets or network requests exist
in the evaluator. Local-case semantics remain unchanged in existing lessons.

## Bounds and diagnostics

Schema version1; at most four named services, sixteen identities per service,
one to four unique methods, supported1–32ASCII names and1–128printable ASCII
configured tokens. An empty submitted token is an ordinary wrong token. Unknown
methods/fields, duplicate service names/identity records, unsupported versions,
invalid names/tokens and excess values fail explicitly. The deliberately unique
method-list rule is a simulator subset restriction, not a claim about every IOS
configuration. Direct map decoding rejects an excess/duplicate key before its
body; arrays and strings use existing bounded deserializers. JSON entry points
reject inputs above64KiB before decoding. Only bounded typed state is evaluated;
there is no learner-controlled loop or external execution.

## Primary references actually inspected2026-11-09

All chapters are Catalyst9300 IOS XE Dublin17.12, updated2023-07-28:

- [Authentication](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/configuring_authentication.html): method order and the distinction between an unanswered request and a returned refusal.
- [Authorization](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/configuring_authorization.html): authorization follows authentication; EXEC attributes and command authorization differ.
- [Local authentication/authorization](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/configuring_local_authentication_and_authorization.html): explicit local EXEC lookup; local-only setup does not provide accounting.
- [Accounting](https://www.cisco.com/c/en/us/td/docs/switches/lan/catalyst9300/software/release/17-12/configuration_guide/sec/b_1712_sec_9300_cg/configuring_accounting.html): activity/session records are separate; Cisco RADIUS does not support command accounting. A future generic simulator journal must not claim protocol interoperability.

An older15-MT authentication URL could not be accessed and is not an implementation
source. No vendor prose/implementation imported. No new dependency or architectural
platform is added. This model must pass its focused behavioral/boundary tests
before a source-held task, editor, authored lesson or objective credit is added.
Existing358labs/progress/themes/teaching remain unchanged by this foundation.

## Complete-document adapter under verification

The adapter reads `aaa authentication login default` followed by supported
`group NAME` and/or `local-case` methods in explicit order. Group names identify
only the source-supplied training-service boundary above; this does not implement
server configuration, transport, RADIUS/TACACS+ packets or cryptography. No other
method, bypass or guessed fallback is accepted. Missing/duplicate lists and
unsupported settings produce line/category diagnostics.

The unchanged local-document parser still validates AAA scope, explicit local
EXEC authorization, local records and incoming management policy. An internal
normalization preserves original line positions; the separate parsed method list
owns authentication evaluation, so normalization does not make a central attempt
use local authentication. Original text controls, line lengths, tokens and total
limits are checked before normalization. Candidate358 and existing local-only
parser/lessons remain unchanged.

R03 passes12method tests and11retained local-record tests. R04Clippy identified
one collapsible conditional; correcting its structure preserves behavior and
assertions. R08–10staged/all-targetClippy pass49.71salltarget; R11fullPython passes.
R12–14compile and test the new document adapter next. No authored5.8lesson or
source-held task/session/native acceptance follows from these foundation gates.

Document gate R14 passes all29focused checks (7new adapter,12method and10retained
local-document checks). Actual method order, central-versus-local words, explicit
denial versus unavailability, local authorization, unsupported methods, scope,
diagnostic lines, raw limits and unchanged supplied state are covered. R11 passes
all968Python checks; historical collectors and observed evidence remain intact.

## Source-held task/session layer under verification

Tasks supply immutable incoming topology/policy, training directories, trial
availability conditions, required local-profile inventory and expected method/
identity/authorization outcomes. Availability conditions are declared simulator
fixtures, not observed network timeouts. Grading evaluates each actual incoming
and identity case across every supplied line; a wrong upstream layer cannot
satisfy an expected credential refusal. Requiring the actual responding method
prevents an all-local bypass from earning central-authentication credit.

Positive cases require both service and local authentication, every permitted
profile, and declared outage/refusal conditions. References must reach all
requirements and starters must remain unfinished. Equivalent supported line/rule
configuration can pass. Source errors, malformed/unsupported documents and broken
paths are never interpreted as valid login denials.

Separate bounded practice/assignment scopes retain draft/applied documents and
selectors. Editing invalidates checks and observations; transient trial output is
not imported as a new execution. Checks are recomputed from current source data,
not a saved success flag. Filling a worked example edits only practice, without
applying or grading. Reset affects one scope. This layer is still unregistered:
no current catalog, learner progress schema, Goal/native path or5.8credit change.
R15–17 compile and test its synthetic fixtures before dependent lesson integration.

Task gate R18 passes27checks (8source-held task,12method,7document); references,
partial failures, behavioral alternatives, actual trace layers and all-local bypass
rejection work. R17 retained a test-only indexed borrow diagnostic; cloning the
duplicate availability fixture before pushing preserves the original assertion.
R19 passes all15task/session checks: separate practice, inert worked fill, partial
feedback, success invalidation, atomic malformed apply, unchanged applied state,
invalid draft resume, source/origin bounds, forged flags, selectors and selected
reset. Trial traces contain identities/stages without echoed training words.
Staged Clippy/Python gates follow; native/source adapters and authored lessons are
still pending and cannot be counted as implemented coverage.
