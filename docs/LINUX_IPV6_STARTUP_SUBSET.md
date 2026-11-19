## Post-build verification —2026-11-15

494/198candidate passed private upgrade/reinstall, installed native100actions/18captures
and normal cold reopening. Ten installed and eight cold original screenshots actually
inspected. Protected launch and identities are in CHECKPOINT.md; selected-input freeze
ENDED after readback/retention. Curriculum/coverage status unchanged; other-platform,
physical-device, screen-reader and independent novice acceptance remain pending.
Build-time pending annotations below are historical. Next:212.2FTPidentity/chroot.

# IPv6 firewall files and a provided startup loader

Implementation continuation,2026-11-15. The verified493-lab candidate is retained
separately. The capstone is registered and source/native verified in the494-lab/198-unit
development catalog. Final package/lifecycle acceptance is pending. No full exam
objective or curriculum completion is claimed.

The next212.1 router slice reuses the existing IPv6 tracked filter, actual supplied
path facts and whole-address/port policy comparison. It composes three distinct
things: the running table, a saved rules file and one explicitly provided custom
oneshot loader. It does not implement arbitrary systemd units or a Linux boot.

## Learner activity

`lpic2-ipv6-startup-policy-recovery` builds on scoped IPv6 router configuration
and tracked router filtering. The learner investigates an obsolete current rule,
a separately unsafe saved policy and disabled startup selection. They repair
FORWARD while preserving the supplied INPUT/OUTPUT, validate/save their intended
file, enable the provided loader and verify both current and fresh-startup policy.
An active loader or a successful syntax check alone cannot earn completion.

The introduction teaches the distinctions before presenting a worked example on
different networks and a different service port. Separate guided practice includes
an intentional saved-file regression. The assignment retains complete independent
requirements, editor/terminal/traffic tools and optional help. Its starter exposes
neither a solved policy nor a fill-assignment action.

## Supported data and commands

The only writable simulated path is `/etc/lab-ipv6-filter.rules`. The native editor
and `ip6tables-save -f` affect that file, not the host filesystem. Saving permits
unfinished ASCII syntax so it can be diagnosed and resumed. Loading is a separate
operation. A command-line save refreshes an unchanged editor; it never overwrites
an unsaved editor draft. **Reload saved file** explicitly discards that draft.

The snapshot contains one `*filter`, all three built-in chain policies with
`[0:0]`, supported ordered `-A` rules and `COMMIT`. Blank lines, whole-line `#`
comments, whitespace, CRLF, chain-declaration order and equivalent supported rules
are accepted. It uses the existing numeric IPv6, interface, TCP/UDP port, ICMPv6
header and NEW/ESTABLISHED conntrack subset. Nonzero counters, custom chains,
extra tables, unknown extensions, shell syntax and trailing commands are refused.
A malformed restore cannot partially replace the running table.

Supported commands are `ip6tables`' existing filter forms; `ip6tables-save`
with optional `-t filter` and/or `-f` for the supplied file; `ip6tables-restore`
with optional `--test`/`-t` and that positional file; and `cat` of that file.
Syntax testing does not commit the rules. Read-only supplied network/settings
inspection is available; even idempotent host-setting writes are refused here.

The fixed `lab-ipv6-filter.service` is supplied by the exercise. It is **not** the
name of a universal Linux firewall service. The read-only example definition uses
`Type=oneshot` and `RemainAfterExit=yes`. The model implements only its declared
loader effect through typed parsing; its ExecStart text is never executed.
Supported systemctl verbs: cat, status, is-active, is-enabled, enable, disable,
start, restart and stop, for this unit only. Nonzero inspection/load exit status
is visible. Enable/disable select startup separately from current activity. Start
of an already active oneshot does not reload. Restart loads the file. Stop retains
rules because this provided unit has no ExecStop. Failed loads retain current rules
and record Failed status; active (exited) only means that loading succeeded.

## Startup and grading boundaries

The separate startup trial assumes supplied static addresses/manual routes are
already persistent. It resets supplied IPv6 settings and loads source-owned flags,
starts with an ACCEPT filter table and invokes the provided loader only if enabled.
It then runs the same full policy comparisons and fresh sample conversations.
Current work remains unchanged. There is no general service ordering, reboot,
distribution firewall integration, transient boot-security claim, dynamic device
lifecycle, RA/SLAAC/NDP timing or real packet/kernel/subprocess access.

Router facts are immutable in this activity. Source validation checks positive
paths against actual supplied peers, scoped gateways/onward prefixes and return
paths. Current/fresh-startup FORWARD grading compares complete supported numeric
address/port/state regions, not command strings or displayed endpoints alone.
Sample cases separately exercise actual request/reply/follow-up direction state,
unsolicited/wrong-port/UDP refusals, selected ICMPv6 headers and link-local scope.
INPUT/OUTPUT preservation is explicitly part of the assignment. Fresh traffic
cases reset simulated connections; this is not a real TCP stack or RELATED model.

Files are bounded to32KiB/256lines, commands512bytes, mutation histories96entries
and256KiB, console128KiB, and workspaces/replayed feedback have explicit size caps.
Unsupported operations fail honestly. Examples, assignment, saved file and unsaved
editors stay separate. Checks recompute current outcomes; edits clear stale success.
Saved state must reproduce through supported source-bound history, including failed
loader operations. Cached traffic/startup feedback is verified against current work.

## References and evidence

Checked2026-11-15: upstream iptables1.8.13 manuals obtained2026-08-04, upstream
commit date2026-07-10; systemd262 online service documentation. These describe real
interfaces; the narrower declarations above are intentional simulation limits.

- [Netfilter save manual](https://man7.org/linux/man-pages/man8/iptables-save.8.html)
- [Netfilter restore manual](https://man7.org/linux/man-pages/man8/iptables-restore.8.html)
- [systemd service manual](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html)
- [LPI v4.5 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)

Five snapshot tests, eight loader-composition tests and ten whole-policy/workspace
tests passed before source/catalog integration. They include valid alternatives,
unsafe defaults, sample-only rules, malformed/truncated files, unsupported escapes,
failed-load atomicity, unsaved drafts, state replay, reset and stale-success failure.
This is focused technical evidence, not an inclusive runtime suite or independent
novice, screen-reader, physical-device or non-Fedora acceptance. Five authored/source/catalog checks and nine preceding administration regressions
also pass. All493previous scenario/197unit values and146previous LPIC-2teaching
values remain exact. Seven native widget/capture/cold-plan checks pass. Actual
R2native100actions/18captures and normal cold writerPID418379(exit0,92.593s) and
readerPID418942(exit0,67.534s) pass. Nine R2originals (all three learning screens,
initial/malformed/success feedback and light150/dark200 controls) and all eight cold
originals were inspected. The first editor regression failed and was fixed with
TextEdit’s real no-wrap layout, preserving cursor/selection/native focus and fixed
viewport/Save geometry. R1screens remain as before evidence. The first cold command
omitted its required --app and launched nothing; R2received SIGTERM after16s with
min18.886GiB/fullPSI0/OOMdelta0 and bounded owned-child cleanup. Its origin is
unknown. R3used the documented allocated terminal; no deadlines/protections raised.
Final packaged lifecycle and installed native results remain pending.


Development images (actual captures, private ignored evidence):

- [Introduction](../.build-artifacts/native-runs/run-_b0a32p1/screenshots/workspaces/v6-startup-introduction.png)
- [Guided editor and terminal](../.build-artifacts/native-runs/run-_b0a32p1/screenshots/workspaces/v6-startup-example-editor.png)
- [Independent assignment](../.build-artifacts/native-runs/run-_b0a32p1/screenshots/workspaces/v6-startup-assignment.png)
- [Before: wrapped rules,200%](../.build-artifacts/native-runs/run-uqb2o7z1/screenshots/workspaces/v6-startup-dark200-editor.png)
- [After: horizontal rule editing,200%](../.build-artifacts/native-runs/run-_b0a32p1/screenshots/workspaces/v6-startup-dark200-editor.png)
- [Light150% saved rules](../.build-artifacts/native-runs/run-_b0a32p1/screenshots/workspaces/v6-startup-light150-editor.png)
- [Cold restored command and separate draft](../.build-artifacts/tmp/native-reopen-ff01b370a0ad51d12b357f56/read/reopen-v6-startup-reader-assignment-command.png)


Final pre-package gates:37distinct focused Rust model/content/regression checks,
seven UI checks and538focused Python checks pass. All-target ClippyR2, strict
inventory494(489updated/5already) and strict coverage533partial/24supporting/683open
with zero draft IDs pass. No new inclusive runtime suite. The retained493factory
was moved before test modules after a real lint failure; no warning suppressed.
The candidate input snapshot now starts; installed acceptance remains pending.
