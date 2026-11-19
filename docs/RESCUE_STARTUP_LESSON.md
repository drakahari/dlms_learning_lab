# Repair a startup from rescue

The LPIC-2 exercise `lpic2-rescue-startup-repair` follows temporary GRUB recovery
and the service-manager unit. It begins after kernel handoff and teaches a
different part of system recovery. The learner observes which units actually
start in the simulated ordinary and rescue sessions.

In guided practice, a required startup check rejects its saved configuration.
The ordinary target cannot reach its required jobs. The supplied rescue target
can still start its own recovery tools. The learner repairs and saves the
startup configuration, then checks a separate ordinary startup while the
current rescue tools stay available. The independent assignment uses a
different set of services, files, values and port.

Starting a session uses the saved unit files and configuration, and resets the
simulated runtime for that session. The temporary recovery selection keeps the
saved default. The grade checks the current rescue outcome, the later ordinary
startup and application outcome, and preservation of all unrelated configuration.
A working but wrong application input fails the ordinary verification.

## Scope and sources

This uses a bounded set of supplied systemd-style service and target units.
It does not run host programs, open a real listener, alter mounts, start a login
shell, or model systemctl isolate. Emergency mode, filesystem repair and actual
systemd/OS startup remain separate scope. This is a practical slice of LPIC-2
objective 202.2. Sources checked 2026-11-10.

- [LPIC-2 objectives](https://www.lpi.org/our-certifications/exam-201-202-objectives/)
- [systemd special targets](https://www.freedesktop.org/software/systemd/man/262/systemd.special.html)
- [systemd unit dependencies](https://www.freedesktop.org/software/systemd/man/262/systemd.unit.html)

## Fedora development checks

Four new Rust tests cover the ordinary and rescue graphs, an alternate
configuration, wrong values, isolation, persistence, and exact retention of
384 scenarios and 145 units. Sixty-one retained unit-manager tests passed.
The final checks also passed all-target Clippy.

Each of four native walkthroughs completed 84 actions with nine captures.
The first review found repeated guidance. The revised screens show one
current practice action with optional worked help. Further inspection exposed
generic restart advice for a separate startup test. The final screen names
the actual wrong value and the failed application job, and directs learners to
check the saved files. Native evidence includes normal text, Light at 150%
and narrow Dark at 200%, alternate input, invalidation, retry and save/readback.
Originals remain private under
`.build-artifacts/native-runs/run-b2qexz_6/screenshots/workspaces/`.

## Fedora candidate evidence

The separate review package passed private upgrade and reinstall, the installed
84-action rescue startup flow, the retained 34-action temporary GRUB flow and
NFS cold reopening. Installed rescue captures are in
`.build-artifacts/native-runs/run-1tu776la/screenshots/workspaces/`. The launch path and checksums are in
[the checkpoint](CHECKPOINT.md).
