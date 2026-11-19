# Bounded QoS simulation subset

Existing roadmap item: CCNA current v1.1/4.7. Primary Cisco v1.1 objective
PDF and RFC2474 read2026-11-02. RFC4594 direct retrieval returned HTTP429 and is
not accepted evidence. Do not cite/reproduce it as inspected. No new track.

The pure deterministic single-egress packet model was tested before dependent lessons.
No packets, sockets, host commands, time source or device configurations.
Classification and outgoing DSCP marking are different operations from queue
selection. A higher DSCP number does not mean higher priority: scheduling needs
an explicit mapping. Marking at one hop cannot guarantee end-to-end service.

Narrow first model: supplied equal-size packets, integer virtual slots, one fixed
link, bounded canonical flow identifiers, explicit arrival order, DSCP0..63,
FIFO versus two-queue nonpreemptive strict priority, finite waiting buffers and
tail drop. Report per-packet original/outgoing DSCP, queue, arrival, departure,
waiting slots or reason for dropping. Never invent real delay/throughput claims.
State arrival-before-service tie order, per-class FIFO, buffer accounting and
unused capacity plainly. Bound both input and replay work; derive outcomes rather
than recognize intended policy strings. Validate duplicate IDs, context, limits,
unknown flows, priority-map values and all goal reachability.

Use multiple deterministic arrival patterns per assignment, retain negative and
alternate policy evidence, source-bound separate practice/current assignment,
stale-result invalidation, protected progress and the approved native tables/
topology/result workspace. Teach prerequisites and a different worked system
before independent policy repair. Do not author dependents until this parser/
model/grade foundation is actually tested.

Explicitly absent initially: variable packet serialization, real clocks or
traffic, IOS syntax, ECN/RED/WRED, fair/weighted queuing, policing, shaping,
token buckets, TCP adaptation, end-to-end guarantees and vendor equivalence.
Those portions of4.7 remain Open; a classification/queuing slice cannot complete
that parent. A later policer/shaper needs its own reviewed semantics before credit.

Retain existing code and historical source/probe/seal families. Any new current
source family requires its own closed disjoint QA layer and frozen-reference
negative guard; do not rewrite old observations or collectors. Main runner only,
shared lock, Cargo2/runtime2 and disk artifacts. Never build this alongside a
candidate build. The initial foundation gate preceded authoring; three lessons are now registered.
See [lesson evidence](QOS_LESSONS.md) for current source/native/package status.

## Current foundation gate — 2026-11-02

Pure `qos` and `qos_workspace` libraries are exported.11 core and8 multi-workload/
state tests pass;4 actual authored-draft source checks also pass. Bounds include
32-byte identifiers,4 flows,128 packets,arrival0..63,64 shared waiting places,
64KiB fixture/policy JSON and16KiB resumable session JSON. Replays derive actual
labels, queue, departure, waiting or tail drop; no canned expected-command route.
Task validators require evidence for every stream in every workload and at least
two deterministic workloads. Actual source drafts use three patterns per task.

Assignments grade resulting behavior across every workload. Correct case results
remain correct when another case fails. Invalid-but-bounded partial queue settings
can be resumed, then repaired. Different nonpriority labels and valid queue sizes
can pass. Practice is independent and cannot award assignment credit; edits clear
the tested snapshot. Stored output, diagnostics or a fabricated passed field are
never accepted as grades. Reference policy JSON is data-only authoring input, not
a taught IOS or shell command. There is no host/network API in these models.

## Integration and known boundaries — 2026-11-02

Review catalog241/65 includes three QoS lessons with typed State/Goal, source-bound
save/restore, native marking/queue controls and actual per-case replay. Development
native widget flows passed twice; candidate verification is recorded separately
in QOS_LESSONS.md. These are a practical slice of CCNA4.7, not its completion.

Retained initial failures: the worked handover report label10 collided with its
interactive priority mapping, making zero-wait unsatisfiable; it is now18 in that
separate worked pattern. The grader was not changed. Initial View serialization
used the wrong case; it now uses the schema's lower-case introduction. Actual
screens exposed unsupported arrow glyphs; new headings use plain text.

A queue's capacity counts waiting places, including arrivals considered before
selection in that slot. A just-arrived packet can therefore be tail-dropped
before selection if its queue is already full. This is a disclosed deterministic
teaching abstraction, not a claim about physical-router internal buffering.
No byte rates, variable packet sizes, real timestamps or end-to-end SLA are modeled.
