# Network file-transfer teaching subset

2026-11-13: three authored lessons implement a bounded practical slice of current
CCNA v1.1 objective4.9. Source catalog452/179: CCNA107, LPIC-1 151, LPIC-2 114,
Bash80. A separate452 Fedora candidate is now verified; the old449 artifact is unchanged.
The unit follows `ccna-transport-conversations`: recovery copy, passive FTP archive
repair, then original/replacement handover. All449 old scenarios and178unit values/
relative order compare exactly. No old requirement, grade, profile or draft changed.

## Actual capability and limits

`network_transfer` composes the existing finite TCP opening, route and stateless
ACL inspection with typed virtual stores and file operations. A request selects a
supplied server, download/upload, two relative store names and (for FTP) a fictional
training login profile. It never accepts a real target, host path or command.

TFTP models octet read/write requests to UDP69, then DATA/ACK using source-held
client/server transfer ports. Exact512-byte multiples require a final empty DATA
block, including an empty file. These are typed application stages with port/path
inspection, **not a wire codec, random TID negotiation, real packets or full RFC
implementation**. FTP uses TCP21 control and a separate source-held passive data
endpoint; the client opens that connection even when file bytes come from the
server. Named login admission and read/write namespaces are modeled; USER/PASS
exchange and real authentication are not emulated. UTF8 fixture bytes and line
endings are preserved; NUL/binary data, netascii/ASCII conversion, active FTP,
TLS/FTPS/SFTP, timers, retransmission, windows and full protocol commands are
unsupported. Neither modeled protocol encrypts a transfer. No actual configuration
is activated. Real interrupted copies can leave partial files; this exercise
commits destination bytes only after every modeled stage succeeds.

Limits: one client/up to3servers,24files/store,1024bytes/file,64KiB total virtual
store text and16trace events per request. The workspace retains up to16distinct successful
plans (identical consecutive retries do not consume another slot),
up to4deterministic cases and bounded source-held router documents. Packet/file
reports are not deserialized as grading authority. Case files and current policy
are replayed from source. Last-result feedback, pending inputs and checked goals
remain distinct. Separate practice/assignment and source-bound progress integration are tested.
All execution stays bounded, offline and in process; there are no host files,
subprocesses, sockets, service changes or added dependencies.

## Evidence and primary sources

Initial8core tests pass: byte-preserving empty/511/512/513/1024-byte and split-UTF8
copies, correct request/reply ports, write ACK0, passive initiation versus data
direction, distinct login/permission/path failures, control-only and one-way path
failures, atomic refusal, malformed/traversal-like input and resource bounds.
An additional13 workspace and4 source/progress checks,6 authored/catalog checks
and exact fixed-authoring reproduction pass. These are not native or package
acceptance; a lab total is not a coverage percentage.

Primary sources opened2026-11-13; no implementation text imported:

- [CCNA v1.1 official objectives](https://learningcontent.cisco.com/documents/marketing/exam-topics/200-301-CCNA-v1.1.pdf),4.9: roles and functions of TFTP/FTP. Future v2.0 remains separately mapped.
- [RFC1350](https://www.rfc-editor.org/rfc/rfc1350.html): purpose, octet mode, request/transfer identifiers and DATA/ACK/end-of-file behavior.
- [RFC959](https://www.rfc-editor.org/rfc/rfc959.html): control/data separation, passive endpoint, retrieval/storage and representation modes.

No complete RFC/issuer coverage, real equipment, novice, physical-input,
screen-reader or other-platform acceptance is claimed. MAIN serial memory guards
and project-disk artifact/progress isolation remain mandatory.

## Teaching and native evidence

Each lesson has its own reviewed Introduction/worked table, separate guided files
and full independent checklist. Transfer file executes a bounded copy; Check work
inspects already-transferred files and current policy across samples. Pending
edits invalidate the check; typing the identical value does not break correct
physical state. Fill worked transfer is practice-only and never executes a copy.
The capstone accepts either copy order and either supplied writer. A successful
wrong-source transfer fails the task; a failed permission/path request leaves
files unchanged. Configuration is staged/archived, never activated.

Development R3 completed226 actual native actions/40 PNGs, including all3 learning
screens for each lesson, file/packet evidence, partial/wrong/correct work, repairs,
save/readback and selected reset. Actual dimensions:1920×1080 wide,946×929 narrow,
with Navy100/200% and Light150%. Six widget and two native-plan/capture tests pass;
placeholder/populated text centering and reflow cover100/115/125/150/200% in both
themes. Keyboard Tab navigation, unchanged-input retention, edited-input invalidation,
private resume and complete editor-viewport reveal have explicit regressions.
A later Tab-navigation/diagnostic-anchor-only refinement is source-tested and must
be verified by the new packaged run; it is not re-attested against the R3 ELF.

Original captures are local ignored evidence, not mockups or public assets:

- [TFTP Introduction](../.build-artifacts/native-runs/run-zi5vf2mo/screenshots/workspaces/transfer-0-introduction.png)
- [Guided file investigation](../.build-artifacts/native-runs/run-zi5vf2mo/screenshots/workspaces/transfer-0-practice-file-evidence.png)
- [FTP assignment/editor](../.build-artifacts/native-runs/run-zi5vf2mo/screenshots/workspaces/transfer-1-assignment.png)
- [Handover practice](../.build-artifacts/native-runs/run-zi5vf2mo/screenshots/workspaces/transfer-2-practice.png)
- [Light150 editor](../.build-artifacts/native-runs/run-zi5vf2mo/screenshots/workspaces/transfer-1-light150-editor.png)
- [Navy200 editor](../.build-artifacts/native-runs/run-zi5vf2mo/screenshots/workspaces/transfer-1-dark200-editor.png)

R1 exposed narrow endpoint columns and an incorrect capture assertion for retyping
an unchanged field. R2 exposed a viewport target clipped to the parent's expanded
clip margin, which could never settle inside the actual scroll area. The corrected
logical viewport and actual parent-gutter scrolling retain the20-reveal,15-second,
1200-poll and one-physical-pixel guards. Failed logs/images remain. R3's development
ELF SHA256 is `8787ed5c78312b365c2747b343be2470528f4ef633f1a85aa23442143d9c891e`;
this is not a packaged identity or independent seal. Final Python1271, current guard subsets175/184, selected Rust42, all-target
one-job Clippy and strict inventory/coverage pass. Package and fresh packaged
cold-flow verification are next in CHECKPOINT.md.

## Packaged452 review milestone

Exact identity/content, private449→452 upgrade/uninstall/reinstall/progress retention,
installed226actions/40captures and two-process NFS/editor-handover cold reopening
pass. Original packaged feedback, light150/narrow200 editors/fields and restored
unfinished work were inspected. Selected2912files/114900060bytes are retained,
not a full compiler closure or independent seal. CHECKPOINT.md contains the exact
protected launcher, hashes and acceptance limits. Package evidence was recorded
after build; do not mutate/re-attest its frozen snapshot to these later notes.

- [Packaged specific wrong-file feedback](../.build-artifacts/native-runs/run-x2eu2ry6/screenshots/workspaces/transfer-1-wrong-source.png)
- [Packaged guided practice](../.build-artifacts/native-runs/run-x2eu2ry6/screenshots/workspaces/transfer-2-practice.png)
- [Packaged Light150 editor](../.build-artifacts/native-runs/run-x2eu2ry6/screenshots/workspaces/transfer-1-light150-editor.png)
- [Packaged Navy200 fields](../.build-artifacts/native-runs/run-x2eu2ry6/screenshots/workspaces/transfer-2-dark200-fields.png)
