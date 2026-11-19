# Linux time-source diagnosis

The LPIC-1 assignment `lpic1-chrony-source-diagnosis` follows Linux networking
and service startup. Learners inspect the current source, distinguish valid
replies from an eligible clock, repair the saved server list and apply it.
The approved primary must be selected, the documented backup must remain
configured and the retired server must be removed. The display time zone
remains a supplied read-only fact.

The separate practice starts with a reachable server that has too few
usable measurements. The source listing shows `^?` even though the reach
register is nonzero. After applying the edited source list, `chronyc tracking`
reports the new reference and local stratum. Edits invalidate checked results,
and invalid configuration leaves the applied snapshot intact.

## Supported model

The simulated file accepts up to three supplied IPv4 `server` lines, optional
`iburst`, and blank or full-line `#` comments. The inspection subset is
`chronyc sources` and `chronyc tracking`, with optional `-n`. Results show
selected fields from fixed source observations and the actual applied source
list. Cases with more than one eligible peer are outside the exercise.

The simulator does not poll servers, resolve names, combine clock estimates,
adjust a host clock, change a real service, configure a hardware clock, or
handle ntpd, NTS or pool configuration. The assignment exercises one part of
LPIC-1 108.1. Wider system date, time zone, hardware clock and NTP administration
remain separate.

## Fedora development verification

Three model tests and four authored and persistence tests passed, alongside the
current-catalog regression. The exact comparison retained all 385 preceding
scenarios and 146 unit values. Tests cover alternate server order, an omitted
backup, stale results, invalid configuration and separate practice.

The focused native walkthrough completed 44 actions and captured nine screens.
It exercised the worked example, keyboard input, an independent configuration,
missing-backup feedback, retry and progress readback. The console now collapses
when no inspection has run, so the editor and result stay closer at 200%.
Reviewed screens use dark at 100% and 150%, and Light at narrow 200%. The captures
are in `.build-artifacts/native-runs/run-9w6luhub/screenshots/workspaces/`.
Final all-target Clippy, 1,045 Python tests, strict coverage validation and the
386-lab teaching inventory passed. The Fedora package passed private upgrade, reinstall, the installed Chrony
workflow and NFS cold reopening. Separate-process Chrony draft reopening
remains unverified.

## Sources

Checked on 2026-11-10.

- [LPIC-1 exam objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
- [chrony 4.8 configuration reference](https://chrony-project.org/doc/4.8/chrony.conf.html)
- [chronyc 4.8 reference](https://chrony-project.org/doc/4.8/chronyc.html)

Installed screenshots are at
`.build-artifacts/native-runs/run-u6hnvbvr/screenshots/workspaces`.
