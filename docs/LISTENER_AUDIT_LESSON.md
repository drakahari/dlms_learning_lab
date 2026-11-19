# Investigate an endpoint before ending its owner

2026-11-05. One new LPIC-1 task, `lpic1-listener-audit`, follows process control
and host networking. Current source285labs/88units:72CCNA,94LPIC-1,45LPIC-2,
74Bash/Shell. All284prior scenarios and87unit values/order remain exact.
This is substantive supplied-endpoint investigation, not full security-audit or
exam preparation coverage. Evaluator19, existing labs and progress schema stay intact.

## Teaching and assignment

Introduction explains protocol/address/port, wildcard versus loopback, numeric
ss options, PID versus name, owner visibility, paused versus closed sockets and
supplied queue metadata. A Morgan worked example has different programs/PIDs and
endpoints. Eight runnable commands inspect, pause, observe the still-bound socket,
continue, retire only the unwanted owner and verify protected work. A reversible
STOP/CONT experiment shows why a bound protected endpoint does not prove its
owner is active. Practice has its own table, command draft and transcript.

The independent Alex assignment asks learners to investigate a wildcard preview
listener, retire only its owner and preserve a same-name loopback instance plus
approved TCP/UDP endpoints and all unrelated processes. Its default task does not
provide a PID or termination command. Twelve actual state predicates require the
unwanted endpoint absent AND owner ended, approved endpoints present AND their
owners unchanged. Equivalent supported TERM/INT/KILL approaches pass, as does
correct handling of a paused/pending-termination owner. Pausing the target, ending
every name match, ending a protected owner or changing priority cannot pass.
Current-state changes invalidate success; no command spelling or proof-of-reading
is graded. Failed endpoint owners gain specific paused/ended/pending/priority
feedback without changing predicate truth or historical empty-table messages.

## Model and persistence boundaries

A bounded source-pinned roster adds IPv4 TCP LISTEN/UDP UNCONN endpoints to the
existing process table. At most64entries, positive ports/existing owner PIDs,
bounded descriptor IDs, canonical IPv4 addresses and no conflicting/shared binds.
Old tables deserialize an empty default and serialize without the new field,
retaining original wire bytes. The whole roster is immutable source context.
Only live-owner phases determine visibility: STOP and pending TERM retain sockets;
Exited removes every owned row from the projection. No real sockets are created.

Only disclosed numeric/listening TCP/UDP ss flags are supported, including
separated/reordered forms; unsupported filters/options/commands fail honestly.
Rows and headers use calculated monospace column widths. All supplied endpoints
remain visible; process details require the finite root/owner visibility policy.
Recv-Q is fixed empty, TCP Send-Q is supplied backlog and UDP Send-Q is zero.
No actual packets, queue evolution, established connections, IPv6, namespaces,
capabilities, firewall/reachability, shared sockets, supervisor restart, boot
policy or application-specific signal handling is claimed. No netstat/nmap/lsof/
fuser command or scanning capability is introduced by this ss-only lesson.

The additional save-boundary regression exposed a real pre-existing omission:
current grading/decode pinned process identity, while saving lacked that source
check. Normal catalog-aware saving now checks endpoint-bearing authored/current/
practice context before replacing bytes, including erased/retargeted rosters,
foreign practice, changed activity/version and unknown source. Uncatalogued raw
saving refuses structurally endpoint-bearing work; without a catalog it cannot
identify an origin after both rosters are erased. It is not source authority.
Normal application save/load always uses the source-aware path. Existing empty
process tables keep their prior save behavior. New task drafts and unrelated
progress/favorites survive save/resume, and reset affects only the selected task.

## Focused evidence

- R1new14capability and retained12process groups pass.
- R2retained9content and7workspace groups pass.
- R3new test initially failed compilation (Progress has no Clone); R4then caught
  the actual missing source-aware save guard. Both original logs are retained.
  R5fix passes; R6all15enhanced capability/progress groups pass2.52s, including
  byte-preserving refusal for assignment/practice/family/version/source forgeries.
- R7three existing native-widget process regressions pass2.72s.
- R8eight authored/alternate/mistake/source/progress groups pass4.46s; R9after
  preservation-feedback improvement retains all8plus15capability passes.
- R10new UI test had a module-name qualification compile error, corrected without
  weakening assertions. R11four UI/provider/actual-capture groups pass4.59s.
- R13development-native56actions pass through real terminal Enter/Check/Reset/
  mode controls, wrong/partial/alternate grading, save/read and light150/dark200.
  Visual review caught mislabeled teaching previews left in Challenge mode;
  those files remain unaccepted evidence. R14five UI/capture groups pass4.58s,
  including refusal to label a Challenge assignment as an Introduction/example.
  A corrected actual native capture remains pending until opened.

Independent exact-source readback found no blocker in source-aware save/load/
grading, new feedback or authored bindings. Private review manifest
`6e8cd9059b413f7f2a6c18d38cf48055f26c2a860660acb6b2d10a0101c48cb5`.
No full inclusive rerun, physical-device, screen-reader, complete keyboard-only,
Windows/macOS/Omarchy or independent novice acceptance follows from these checks.

## Primary references and objective scope

[Current LPI exam101/102 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
rechecked2026-11-05: version5.0,101-500/102-500. Direct narrow109.3ss practice;
110.1open-port reasoning and110.2runtime retirement are supporting only. This does
not credit the named nmap/netstat tools or persistent disabling of unused services.

Upstream iproute2 [ss(8) source](https://git.kernel.org/pub/scm/network/iproute2/iproute2.git/tree/man/man8/ss.8)
was inspected through the installed original manual on2026-11-05; the compressed
input SHA256 is retained privately. Only original paraphrased teaching is bundled.
No real ss or network reference command was executed. Existing process-signal
subset and prior reference evidence are reused within their actual boundaries.

R16correctly rendered the normal three views, but its enlarged practice-controls
capture refused because a practice Run was not exposed to the harness (there is
no Check work in practice). R17six guards pass4.54s after measuring the existing
Run button and selecting it by actual view. New worked-table values are shortened
to fit their columns; full explanations remain adjacent. The failed run and its
images remain retained. A complete corrected native run is still pending.

Corrected R19development-native58actions/22PNG completes. Actual Introduction
(with aligned shortened worked values), Try example and light200assignment
controls opened from run-py_61_p_. Unsupported capture assumptions were fixed in
the harness; learner workflows and grading did not change. Early unaccepted
images/logs remain retained. Final packaged bytes/screens remain pending.

- [Introduction](../.build-artifacts/native-runs/run-py_61_p_/screenshots/workspaces/teaching-lpic1-listener-audit-introduction-dark-100.png)
- [Separate practice](../.build-artifacts/native-runs/run-py_61_p_/screenshots/workspaces/teaching-lpic1-listener-audit-example-dark-100.png)
- [Fresh independent assignment](../.build-artifacts/native-runs/run-py_61_p_/screenshots/workspaces/listener-assignment.png)
- [Paused is not closed](../.build-artifacts/native-runs/run-py_61_p_/screenshots/workspaces/listener-stop-is-not-closed.png)

Final R20authored8/capability15/retainedmanaged4/map7 pass, including exact old
scenario/unit preservation. R21LPIC-1 and R22whole-catalog content inventories
pass. All-target R23Clippy passes56.97s. This is focused current evidence plus
retained prior suites, not a fresh one-shot inclusive Rust/UI acceptance claim.

R24Python ran746checks with one correct failure: generated teaching inventory
was still284. R25regenerated the actual285inventory (280updated+5alreadymeets);
no test/requirement was weakened. The corrected rerun is retained separately.

Final R26Python746passes12.225s; current R29coverage validates285registered
lessons, no unregistered drafts and204partial/7supporting/1029open overlapping
issuer rows. Two earlier ledger checks correctly refused a UI-source test-path
entry outside the verifier's admitted evidence root, then inconsistent supporting
parent status. Only those new rows were corrected; all refusals remain intact.
Inventory280updated+5alreadymeets. Source/data now frozen for separate packaging;
actual package identity and native/lifecycle evidence must follow.

## Packaged285Fedora candidate

R30candidate `dist/teaching-review-20261105T150031Z/` has569declared source/data
hashes and258packaged documents matching MAIN before subsequent notes. Actual
R31content validates285/88. R32packaged native completes58actions/22PNG with
correct view assertions; actual normal three views, paused-owner feedback,
light150terminal and light200controls opened from run-dqv8gmt2.

- [Packaged Introduction](../.build-artifacts/native-runs/run-dqv8gmt2/screenshots/workspaces/teaching-lpic1-listener-audit-introduction-dark-100.png)
- [Packaged Try example](../.build-artifacts/native-runs/run-dqv8gmt2/screenshots/workspaces/teaching-lpic1-listener-audit-example-dark-100.png)
- [Fresh assignment](../.build-artifacts/native-runs/run-dqv8gmt2/screenshots/workspaces/listener-assignment.png)
- [Protected-owner feedback](../.build-artifacts/native-runs/run-dqv8gmt2/screenshots/workspaces/listener-protected-owner-paused.png)

R33private284→285upgrade passes exact manifest/binary/content, desktop validation,
uninstall/reinstall and unchanged synthetic original-nine bytes. Normal installed
app and original learner progress are untouched. New-listener-specific cold
process is not claimed; actual warm native save/read and disk persistence tests
are distinct. The fixed installed NFS cold regression remains pending at this
point. No complete curriculum or cross-platform/accessibility acceptance follows.

Installed R34NFS cold process save/exit/reopen passes with restored drafts and
fresh grading; repaired screenshot opened from native-reopen-e3dc2c57f147815950f6c067.
This is the shared-persistence regression, not new-listener-specific cold proof.
R30build411samples: min11.071GiBavailable/PSIavg10max0/OOMdelta0; swap nearly full.
Exact runnable candidate and identities are recorded in CHECKPOINT.md.
