## Verified bounded integration

The329/112isolated review package now passes exact byte readback and actual native
checks (R63–R67). See LVM_COW_LESSONS.md/CHECKPOINT.md for identity, scope and actual
screens. The intermediate under-proof notes below remain chronological evidence;
this is still classic raw-record simulation, not a full LVM/kernel/FS claim.

# Bounded classic COW records — integration under verification

2026-11-07. Working registration329labs/112units includes three LPIC-2snapshot
lessons under proof; the latest VERIFIED package remains326/111. Pure, physical,
typed-goal, guided-flow, source-session and authored behavior gates pass. Actual
native/rendered review, final CURRENT-only QA, source freeze and new package gates
are pending. No full204.3coverage, real-device conformance or novice acceptance.

The absent-by-default State.snapshots profile is disjoint from existing optional
LVM profiles. Its source-bound typed assignment and separate practice are required;
injecting raw snapshot state into an old definition does not admit it. Session
admission protects immutable storage identity while allowing recoverable mistakes;
completion additionally checks required record ages, reservations, original point
identities, exact unrelated data and absence of extra storage. A name alone cannot
preserve an old view after removal/recreation. Structural/idle validation never
runs reference solutions. Applications/record copies are explicit simulator tools.

A virtual origin has explicit non-overlapping byte ranges for supplied fixed-size
printable ASCII records. Spaces, tabs and newlines are allowed; variable lengths,
binary/control data, added/deleted records and filesystem metadata are excluded.
This is raw virtual record storage, not ext4 or an actual device. No host I/O,
processes, kernel, network, permissions or system changes are used.

Snapshots begin with no copied data. On the first write touching a chunk, a sparse
before-image captures every supplied record fragment in that chunk. Other reads
continue to use current origin data until their chunk is copied. Repeated writes
do not overwrite an existing image or charge twice. Shared and spanning records
exercise real chunk relationships; this is not a whole-file backup in disguise.
Different snapshot ages retain distinct before-images. Implicit empty regions stay
zero; no giant virtual-device array is allocated by production code.

Chunk sizes are powers of two from4KiB through512KiB. The pure model requires
sector-aligned COW capacity and reserves one chunk for metadata as a disclosed
teaching convention, not actual LVM metadata accounting. The opt-in profile enforces actual modeled LV extent sizes and reservations
for its supported create/extend/remove commands. Space exhaustion invalidates that snapshot while the origin
write continues. An invalid snapshot cannot be read or revived by adding space;
removal/recreation represents a new time. Refused inputs leave the model unchanged.

Bounds:16GiB virtual address range,32records/128KiB actual origin text,4snapshots,
128sparse images each,64KiB maximum record and1MiB serialized model. Streaming
map/text bounds and byte/offset checks precede use. No unbounded loops or output.
Atomic candidate-state cloning is transient error handling; persistent snapshots
store only their copied chunks. The test oracle may use full bounded byte arrays
for an independent comparison; production snapshots do not.

Focused proof: R20pure9 includes72deterministic updates against an independent
bounded byte-array oracle; R21physical9 tests real virtual extent reservations;
R19goals6 tests source/data/age/identity/partial/current-invalidation; R25flow8
tests explicit stimuli, isolation, genuine mistakes and repair/reset; R32session6
tests explicit admission, inert inserts, actual progress decode, drafts and legacy
behavior; R34authored6 tests all three full assignments/examples, alternatives,
wrong timing, invalid views and persistence. These counts do not imply native or
whole-application acceptance. Failed diagnostics remain retained privately.

Authoritative references checked2026-11-07. The official LPI page still declares
objectives4.5 and exams201-450/202-450;204.3includes snapshot maintenance. This
batch targets only the bounded classic-COW slice, never the whole objective.
- [LPI official201/202objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [Upstream lvextend(8)](https://man7.org/linux/man-pages/man8/lvextend.8.html)
- [Upstream lvremove(8)](https://man7.org/linux/man-pages/man8/lvremove.8.html)
- [Upstream lvcreate(8)](https://man7.org/linux/man-pages/man8/lvcreate.8.html)
- [Red Hat LVM guide](https://docs.redhat.com/en/documentation/red_hat_enterprise_linux/8/html/system_design_guide/configuring_and_managing_logical_volumes)

No actual LVM or device conformance comparison has been run. Thin snapshots,
merge/deferred activation, filesystems, automatic extension and general LVM behavior
remain unsupported. Future native application-write stimuli must be explicitly
simulator-only and must never be presented as real Linux utilities.

Physical coupling: each snapshot requires a distinct raw COW LV in its origin's
group; its reservation matches actual virtual extent allocation. The origin and
unrelated LVs cannot be resized/removed through this profile. Commands are parsed
and evaluated in process. Scope and remaining gates are in CHECKPOINT.md.
