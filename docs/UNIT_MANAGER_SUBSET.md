# Virtual unit manager — first pure capability contract

Specified2026-10-23 before implementation. Source-only capability milestone;
not a new app State/Goal, CLI, GUI, scenario, startup claim or objective coverage.
Old `State::Service` and its teaching/progress remain unchanged. No dependency,
host filesystem, subprocess, systemd D-Bus, service, socket, timer or network API
is used. Every path/name below belongs solely to supplied in-memory data.

## Purpose and separate layers

Provide enough genuine behavior for later unit/configuration editor lessons:
saved unit text, manager-loaded definitions, dispatched/executed program state,
application configuration, boot-install links and the selected default target
are separate. Editing one layer does not silently update another. Existing
single-service exercises are not changed or relabeled as complete systemd.

Version1 immutable Inventory fixes a fixture ID, at most16 unit names,16 virtual
config paths,8 fictional program descriptors and8 artifact slots. Unit names are
1–64 ASCII bytes ending `.service` or `.target`; templates, aliases, instances,
escaped names and other unit kinds are unsupported. Program keys are canonical
absolute `/sim/bin/...` identifiers, never executable host paths. Config paths
are fixed canonical absolute virtual paths (128-byte/depth8 bound); `..`, links,
new path creation and host access are impossible. Artifact IDs are names, not
filesystem paths. Full inventory equality is the later import/goal context pin.

State contains exact-key saved unit/config maps, a subset of loaded unit source
snapshots, runtime states, artifact values/provenance, boot Wants links, and one
known default `.target`. Saved text may be malformed and must round-trip; no
automatic repair/truncation. Loaded/runtime snapshots must independently parse
and resolve against Inventory. Future imports cannot authorize new descriptors,
paths, unit IDs or executable forms. No cached report or boolean is a credential.

## Unit document grammar

One full virtual document per fixed unit. At most8192 UTF-8 bytes,256 physical
lines and512 bytes/line. Blank lines and whole-line `#`/`;` comments are allowed.
Trim spaces/tabs around a key/value; no inline comments, continuation, quoting,
escaping, substitution, `%` specifiers, environment variables or command syntax.
Only these sections/keys are accepted, in any section order without duplicates:

```ini
[Unit]
Description=Readable short description
Requires=prepare.service
Wants=optional.service
After=prepare.service
Before=another.service

[Service]
Type=oneshot
ExecStart=/sim/bin/prepare
RemainAfterExit=yes

[Install]
WantedBy=practice.target
```

List keys accept up to16 distinct known unit names separated by spaces/tabs.
Self Requires/Wants references are explicitly unsupported; nonself pull cycles
are allowed and do not imply an ordering cycle.
Repeated list keys append/deduplicate, as systemd documents; an empty assignment
resets that list. Scalar duplicates and repeated section headers are explicitly
Unsupported, rather than silently approximating systemd's full merge semantics.
`Type` is explicit `simple` or `oneshot`. `ExecStart` is exactly one registered
fictional descriptor path without arguments/prefixes; a real executable, shell,
extra token or unknown descriptor fails Unsupported. `RemainAfterExit` is yes/no,
defaults no and is supported only for oneshot in this slice. Target documents
have Unit/Install sections only, no Service/Target section, and need Description
so an empty unit is not mistaken for masking. A service requires Description,
Type and ExecStart. Description is nonempty/at most160 UTF-8 bytes, no controls.

Unknown keys/options/sections and unsupported shapes are honest diagnostics.
There are no drop-ins, search-path precedence, conditions, masks, ExecStartPre/
Post/Stop/Reload, Environment/User, success-code lists, restart policies, timeouts,
notify/dbus/forking/idle, sockets, timers, mount units or dependencies on real
special targets. Service/target default boot/shutdown dependencies are outside
this abstract *post-initialization* manager. No kernel, bootloader, SysV or full
system initialization is represented. Within this slice targets do complement
their Wants/Requires dependencies with After ordering; no `DefaultDependencies`
override is accepted. These limits must stay visible in later optional help.

## Fictional programs, not command recognition

Two closed declarative descriptor variants are proposed:

- **ValueTask** for oneshot: read the descriptor's fixed config path/key, verify
  an optional prerequisite artifact value, and publish the nonempty value into
  its fixed artifact slot. One producer descriptor per slot avoids write races.
- **PortDaemon** for simple: read a fixed config path/key as an ASCII decimal
  port1–65535 and verify an optional prerequisite artifact value. Its running
  metadata retains that value; no real listener, bind conflict or network is
  claimed. Same-port programs are not pretending to reserve host sockets.

Each application config is at most4096 UTF-8 bytes. It contains one expected
`KEY=VALUE` plus blank/whole-line comments; unknown/duplicate/missing keys,
invalid text/port or empty/over256-byte value produces a program diagnostic.
Saving bad config is allowed. A program snapshot retains its unit source,
consumed config and observed optional artifact input; program result is derived
again from that snapshot, not a success string. Unknown programs/types are never
treated as successful execution. No descriptor can name arbitrary read/write
paths, change Inventory or invoke other code.

## Start jobs, ordering and readiness

Typed Start collects the selected unit and transitive loaded Requires/Wants
pull-ins plus installed Wants links. After/Before alone do not pull another unit
in. Pull-in cycles terminate via a visited set; requirement cycles alone are not
incorrectly called ordering cycles. Unknown fixed IDs fail validation; a known
but unloaded pulled unit yields a failed job, not a fabricated success.

Topological *layers* come only from After/Before edges among jobs, including
targets' complemented Wants/Requires order. Explicit ordering cycles fail with a
bounded named diagnostic before state publication. Lexical ID order is only
stable report presentation: jobs in the same layer read one pre-layer snapshot,
so one name sorting earlier cannot accidentally guarantee readiness for another.
Report layers expose the actual transaction order. The transitive
`ordered_before` inspector describes only a path in the complete declared graph;
it may pass through inactive units absent from a particular start transaction.
Scheduling and failed-Requires propagation both use only edges whose endpoints
belong to that transaction, so an unstarted intermediary supplies no ordering or
readiness guarantee. The chosen layer projection is a disclosed deterministic teaching
schedule, not a claim about every possible real systemd concurrency interleaving.

An ordered failed Requires activation prevents dependent dispatch; failed Wants
does not. A Requires edge alone does not supply ordering or readiness. The core
must never replace Requires with an implicit After. Already active starts do not
re-run the program. A successful oneshot completes its job after its body; it is
active/exited only with RemainAfterExit=yes, otherwise inactive after success.
Requires does not falsely require a successful no-remain oneshot to stay active.

Simple dispatch completes the start job before its application initialization.
Runtime is explicitly Dispatched (manager start done, program readiness pending).
Typed AdvancePrograms evaluates pending simple programs using the then-current
virtual config/artifact inputs. It can produce Running or Failed while the
earlier start job was Done. After=simple therefore is never advertised as an
application readiness guarantee. Application failure does not implement BindsTo
or automatically stop a requiring unit. These are manual simulator events, not
real fork/exec/PIDs, timing, readiness sockets or process supervision.

## Safe actions and reload/install boundaries

All actions first validate directly constructed state and scalar/count limits.
Structural/parse/order errors retain the previous state exactly; at most one
bounded candidate clone is published. A job transaction can legitimately publish
successful and failed jobs together, with typed per-job reasons: transactional
publication does not mean all programs must succeed.

- SaveUnit/SaveConfig change only the fixed saved raw slot, accepting bounded
  mistakes. They do not change loaded/running/boot state.
- DaemonReload parses/resolves all saved documents and checks the ordering graph
  before replacing all loaded definitions together. It never restarts programs
  or consumes their config. This all-or-nothing invalid-file/cycle policy is a
  stricter disclosed editor boundary, not full systemd loader/error recovery.
- Enable derives WantedBy links from saved documents after a successful implicit
  daemon reload; adds links without removing old links and does not start units.
  Disable removes all modeled links to that unit, also reloads, and does not stop
  it. Also/Alias/RequiredBy/presets/runtime links/--now are unsupported.
- SetDefault selects a known target for a future session only; it is not isolate
  and does not switch or stop the running set.
- Stop performs immediate fictional termination of the requested unit and active
  reverse-Requires dependents, in reverse ordering layers. Wants does not imply
  stop propagation. No ExecStop/signals/cgroups/busy process semantics are claimed.
  Full restart/try-restart/reload/isolate/mask are not added; Stop then Start is
  the explicit available way to reread an application config in this core.
- FreshSession returns a separate clone with runtime/artifacts cleared, saved
  definitions freshly loaded and the default target started. It never changes
  the caller or claims a real boot/kernel/system configuration operation.

Raw files, loaded source, process snapshot and install/default state have separate
query APIs. Current configuration mismatch/needs-reread inspection is derived
from snapshots and saved/loaded values. No wall clock, log of claimed success or
previous start report substitutes for these current values.

## Limits and planned verification

Version1,16 units/configs,8 descriptors/artifacts; unit8KiB/config4KiB,256 lines,
512-byte lines,16 unique references/list,160-byte description,256-byte artifact
value,512-byte retained diagnostics. Explicit aggregate limits apply before
clone/derived collections, and strict unique map/string visitors reject excess
before retaining another large value. Unit source snapshots obey the same bound.
State JSON entry point proposed4MiB, including worst legal escaped full state;
exact compact/pretty maximum-shape roundtrip must justify that ceiling before
acceptance. Direct serde users must supply an outer byte bound. Jobs/layers are
bounded by the16 fixed IDs; reports contain no unbounded journal or output.

Tests must cover repeated/empty list semantics; unsupported exec/syntax; same
layers versus true transitive order under name permutation; ordering and benign
pull cycles; Requires-with/without-order versus Wants failures; no-remain oneshot
success; simple dispatch success versus later config failure; config save versus
unit reload versus explicit stop/start; default/enable/disable independence;
missing loaded optional/required units; active reverse-Requires stop order;
strict duplicate/size/schema/name/descriptor/path/import validation; snapshot
forgery/foreign Inventory; maximum escaped state; rollback and per-job partial
publication. Later engine/UI/progress/content need independent integration gates.

## Authoritative sources and honest objective scope

Checked2026-10-23 against official versioned **systemd262** manuals:

- [systemd.unit](https://www.freedesktop.org/software/systemd/man/262/systemd.unit.html):
  pull-in versus ordering, ordered required-job failure, Wants continuation,
  reverse Requires on explicit stop and WantedBy install links.
- [systemd.service](https://www.freedesktop.org/software/systemd/man/262/systemd.service.html):
  simple fork/start boundary versus readiness, oneshot and RemainAfterExit.
- [systemctl](https://www.freedesktop.org/software/systemd/man/262/systemctl.html):
  daemon-reload versus application reload, implicit reload on enable/disable,
  neither starting/stopping runtime by default, default-target selection.
- [systemd.target](https://www.freedesktop.org/software/systemd/man/262/systemd.target.html):
  no Target section and default ordering complement for target dependencies.
- [LPI current LPIC-2 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/):
  v4.5,201-450/202-450;202.1 includes querying/modifying service behavior at targets
  and wider systemd/SysV/LSB startup knowledge. None is promoted by a pure module.
- [LPI LPIC-1 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/):
  v5.0;101.3 default/current targets remains a possible later narrow supporting
  slice, not real shutdown/reboot, user alerts or complete boot experience.

No upstream implementation/manual bodies are copied. This original pure model
is not systemd, Linux, a VM, a container, equipment/runtime acceptance or complete
LPIC coverage. Full202.1, SysV/LSB and boot/shutdown details remain Open.

## Optional retained events — tested backend extension

The subsequent terminal foundation adds an optional version1 typed journal to
this new State only. See [UNIT_MANAGER_TERMINAL.md](UNIT_MANAGER_TERMINAL.md).
Existing absent event data remains absent on serialization. The store is bounded
at256 events, contiguous checked u64 sequence/dropped counters, known unit IDs and
512-byte diagnostic detail; decoding validates shape and Inventory context. Job
start, dispatch, program initialization, ordered dependency failure and meaningful
stop results create events only on committed candidate actions. Invalid actions
or event counter overflow retain the exact original State. Queries, saved edits,
reload and enable/default changes do not fabricate program-output events. Fresh
sessions receive a separate window. No PID/time/real journald behavior or current
success is inferred from historical messages. The17 focused journal/terminal tests
and113 existing scoped regression checks pass, as does all-target Clippy. Native
UI/content adoption remains separate; these backend checks do not establish
release readiness or actual systemd/runtime acceptance.
