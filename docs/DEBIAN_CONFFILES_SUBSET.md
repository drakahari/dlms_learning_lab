# Explicit conffile upgrade decisions — implemented subset

Implemented2026-11-04 for the existing102.4 roadmap slice. Three integrated
lessons use explicit native configuration decisions. Ordinary package actions,
their serialized forms and old progress remain unchanged. No host package operation, script, process, service or network access.

## Admitted operation

Add one explicit logical `UpgradeConffiles` action for a selected installed
package, using the existing cached-candidate resolution and complete package
planner. It represents native per-file prompt decisions, not a dpkg/APT command
or a global force flag. The terminal retains its currently documented subset;
`dpkg -i` remains unsupported. There is no arbitrary override/configuration route.

The request binds package, previous archive, incoming archive and exactly the
unchanged managed-conffile set (1–4 present regular text files). Snapshot each
local value, including an empty string; reject stale identities/text before
publication. Bound snapshot text to2048UTF8 bytes each and the complete action
to the existing16KiB limit. Decisions are a closed KeepLocal/InstallNew enum,
required only for true conflicts and forbidden for automatic outcomes. Missing
files, new/obsolete conffile adoption, downgrades, confmiss/confdef/confask and
maintainer-generated configuration remain explicit unsupported cases.

Let O be previous packaged text, L the present local text and N the incoming
packaged text. For this opt-in operation: L=N needs no decision; L=O installs N;
N=O preserves L. Only when all three differ does a decision apply:

| Decision | Live configuration | .dpkg-dist | .dpkg-old |
| --- | --- | --- | --- |
| KeepLocal | L | N | Absent |
| InstallNew | N | Absent | L |

The package record advances to the incoming archive even when L is kept. Other
payload, availability, architecture, dependency, conflict and ownership checks
remain mandatory. Existing implicit Install intentionally retains its older
refusal for dual local/default changes (including L=N); this new operation does
not silently alter that contract.

## Conservative sidecar admission

For the first slice refuse pre-existing .dpkg-old/.dpkg-dist/.dpkg-new/.dpkg-tmp
neighbors of the managed paths, including protected or unrelated files. Refuse
any derived sidecar colliding with any supplied manifest path, current owner,
proposed sidecar or bound. This is a simulator restriction: real dpkg can replace
or remove previous sidecars, so existing-sidecar/repeated-upgrade handling is
not claimed here. No basename suffix implies ownership.

New activity catalogs must explicitly admit derived paths in Catalog.extra_paths.
Borrow their catalog-owned strings after checking the bounded suffix; do not
borrow a temporary generated String or broaden arbitrary path access. The
existing borrowed Prepared map, bounds and materialization can remain intact.
Sidecars remain ordinary unowned virtual files. The new activity adapter refuses
Remove/Purge of a package while associated sidecars remain; it must not announce
a successful real-equivalent purge leaving those files behind. Old activities
retain their exact prior actions and limitations.

All checks finish in the existing whole-transaction in-memory Prepared plan
before publishing any records, payload or configuration. Do not raise file,
work, text, action or progress limits. This model guarantee is not real dpkg
rollback, atomic disk publication, durability, maintainer-script execution,
permissions, symlinks, ACLs or crash recovery.

## Native and progress contract

Use an explicit source-bound ConffileUpgrade outcome requirement to opt new
activities in; do not infer permission from a filename suffix. Grade actual
installed archive, live configurations, backup/default sidecars and protected
siblings. A selected dropdown or saved success flag is not authority. Equivalent
supported ways to reach the same required state remain valid.

Keep the existing terminal/inspector. A compact relevant native comparison and
per-file choice draft prepares the upgrade; a separate apply action changes state.
Changing selection/local values makes the draft stale. Never auto-apply on load.
Keep example and assignment drafts separate. Existing definitions and state
encoding remain valid, with new optional draft data omitted when absent. No
answers or mastery are invented for legacy profiles. Only new stable IDs can
receive this new objective; old lab goals do not change.

## Primary sources and known evidence limits

Checked2026-11-04:

- Debian dpkg1.22.22 manual, dated2026-03-06:
  https://manpages.debian.org/trixie/dpkg/dpkg.1.en.html
- Debian Policy4.7.4.1, configuration-file handling:
  https://www.debian.org/doc/debian-policy/ap-pkg-conffiles.html
- dpkg1.22.22 configure.c / remove.c:
  https://sources.debian.org/src/dpkg/1.22.22/src/main/configure.c
  https://sources.debian.org/src/dpkg/1.22.22/src/main/remove.c
  Distribution VCS identity58e5927f9f2103e94574d92edbd93e41ab93384a.

Primary source behavior was independently inspected. Real decisions can replace
regular sidecars; purge scans configuration-related backups, motivating the
honest refusal above. No dpkg executable comparison or complete dpkg/APT
conformance is established. Implement original behavior from this contract;
copy no GPL implementation text or binaries into the MIT application.

Core, workspace, draft, control, content and representative native checks pass.
The three registered lessons have separate guided examples, actual state grading
and source-bound saved drafts. Development native route run-6dro1_0c completed
120interactions and36captures. Packaged repetition is pending; see
DEBIAN_CONFFILE_LESSONS.md for evidence and explicit acceptance limits.
