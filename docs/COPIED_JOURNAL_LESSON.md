# Recovering a copied systemd journal

The LPIC-1 assignment `lpic1-copied-journal-incident` follows Linux logging.
A copied journal contains several boot windows and services. Learners identify
the incident boot, retrieve its requested service errors and compare the result
with another boot or urgency threshold. Practice uses another archive, service
and threshold. The Introduction demonstrates the boot table and worked query
before practice. Assignment answers are hidden by default.

## Supported simulation

The in-process reader operates on immutable authored records, grouped into up
to eight boots with at most 32 entries each. It accepts the supplied recovery
directory, full boot IDs, exact service names and a single named or numerical
priority threshold. The directory names supplied data. Queries select actual
records by these fields. Short and long option forms, option order and optional
--no-pager are supported. Output can be selected and copied.

The learner lists boot windows, revises a query, inspects results and checks
the filter. Editing invalidates the previous output and grade. Practice and
assignment drafts stay separate. Saved output is reconstructed from source data
and the validated query.

The model does not decode binary journals or simulate rescue booting. It omits
time-range parsing, relative boot selectors, unit globs, special manager and
coredump matches, rotation, persistent-storage policy and host logs. It never
opens a learner path, starts a subprocess or queries a service.

## Fedora development evidence

Three model and five authored and persistence tests passed. They cover alternate
flags, wrong boot, wrong service, excessive and insufficient urgency, unsupported
syntax, path boundaries, separate practice, reset, resume and invalidation.
The tests retain all 386 preceding scenarios and 147 unit values.

Three focused native runs completed 45 actions and ten captures each. The first
review found that the worked explanation lacked the boot table needed to follow
its reasoning. The revised Introduction shows the expected table, and query
output is selectable. Final feedback describes both the entered and required
severity thresholds. Native save/readback and keyboard text entry passed.
Reviewed screens cover dark100/150 and narrow Light200. Final captures are at
`.build-artifacts/native-runs/run-0ugblwlp/screenshots/workspaces/`.
Final checks passed all-target Clippy, 1,049 Python tests, strict objective
coverage validation and the 387-lab teaching inventory. The 21 retained classic
logging tests passed. The coverage map now includes their already implemented
selector practice under 108.2 K1 and K2, alongside the new copied-journal K7
slice. This corrects reporting for existing labs. No grading requirement changed.

The Fedora package passed private upgrade and reinstall, its installed journal
workflow, retained Chrony work and separate-process NFS reopening. Cold reopening
of a copied-journal draft remains unverified.

## Primary sources

Checked on 2026-11-10.

- [LPIC-1 v5.0 objectives](https://www.lpi.org/our-certifications/exam-101-102-objectives/)
- [systemd 262 journalctl reference](https://www.freedesktop.org/software/systemd/man/latest/journalctl.html)

Installed screenshots are at
`.build-artifacts/native-runs/run-q4ha6jho/screenshots/workspaces`.
