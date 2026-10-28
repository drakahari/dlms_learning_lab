# LPIC reconciliation and next implementation frontier

Reconciled against current Main164 source, checked2026-10-28. The normal catalog
has36 LPIC-1 and28 LPIC-2 authored assignments, including four recurring-job
assignments. Related52 Bash assignments are not counted again as Linux-track
labs. No parent is complete. Native/candidate results for the latest ten source
additions remain separately pending in CATALOG164_MILESTONE.md. The original
4bc390a/private158 source epochs remain in Git and retained private evidence.

Official objective pages still identify [LPIC-1 5.0,101-500/102-500](https://www.lpi.org/our-certifications/exam-101-102-objectives/) and [LPIC-2 4.5,201-450/202-450](https://www.lpi.org/our-certifications/exam-201-202-objectives/). Both objective pages were fetched; overview URLs timed out. No new objective version was found in those pages. Existing granular fingerprints remain the 2026-10-22 snapshot, not silently rehashed or redated.

## Exhaustive row reconciliation

| Track | Parents | Knowledge rows | Named tools/terms | Total overlapping references |
| --- | --- | --- | --- | --- |
| LPIC-1 |28 Open /14 Partial |151 Open /29 Partial /1 Supporting |327 Open /48 Partial |598 |
| LPIC-2 |35 Open /6 Partial |180 Open /14 Partial |288 Open /8 Partial |531 |

All 83 parent rows and 1129 granular rows were enumerated. Every coarse parent
credits exactly the same scenario-ID set as its granular parent. Every credited
scenario exists; every prerequisite on all64 LPIC assignments resolves; every
authored LPIC assignment has at least one granular mapping. These are consistency
checks, not proof that each credited scope is sufficient for the whole objective.
The existing unit/content tests, native evidence and item-specific remaining
limitations remain authoritative. Open rows are not implemented simply because
a related model or document exists. Partial rows must retain their explicit gaps.

Canonical exhaustive knowledge/tool inventories are [SKILL_COVERAGE.csv](SKILL_COVERAGE.csv);
all distinct planned assignments, capabilities and limitations are in
[CURRICULUM.csv](CURRICULUM.csv). Retained original154 reconciliation source identities (historical, not current hashes):

- SKILL_COVERAGE.csv SHA256 `2202e95216ac220c103c74724a8f93250ede7aef747f9756a03eaefe00024201`.
- CURRICULUM.csv SHA256 `eaf247a23721df03fd5f619e7b1e52b049a58802f14011621929a3b8b8a8d71d`.

The following includes every LPIC parent and its actual current narrow credit and
remaining proposed task/prerequisites.107.2 gains only the six explicitly reviewed
recurring-job/access/configuration slices; at/timer/CLI requirements stay Open.

## LPIC-1 v5.0

| Objective | Current slice | Credited authored IDs | Required proposed task / prerequisites |
| --- | --- | --- | --- |
| 101.1 | Open | None | Identify a missing storage driver / Terminal and file basics |
| 101.2 | Open | None | Trace a stalled boot / Terminal and file basics |
| 101.3 | Open | None | Recover the intended startup target / Terminal and file basics |
| 102.1 | Open | None | Plan disks for a growing log server / Terminal and file basics |
| 102.2 | Open | None | Repair an alternate boot entry / Terminal and file basics |
| 102.3 | Open | None | Diagnose a missing shared library / Terminal and file basics |
| 102.4 | Partial | lpic1-package-install;lpic1-package-metadata;lpic1-package-payload;lpic1-package-handover | Resolve a Debian package conflict / Terminal and file basics |
| 102.5 | Partial | lpic1-rpm-provider;lpic1-rpm-epoch;lpic1-rpm-payload;lpic1-rpm-handover | Pin a working RPM update / Terminal and file basics |
| 102.6 | Open | None | Recognize a guest device mismatch / Terminal and file basics |
| 103.1 | Partial | shell-handover;shell-arguments;lpic1-file-navigation;lpic1-file-names | Diagnose a surprising shell value / Terminal and file basics |
| 103.2 | Partial | lpic1-logs;shell-pipelines;shell-cut-inventory;shell-sort-manifest;shell-uniq-counts;shell-text-capstone | Normalize a noisy inventory stream / Terminal and file basics |
| 103.3 | Partial | lpic1-file-navigation;lpic1-file-names;lpic1-copy-move;lpic1-file-cleanup;lpic1-file-handover;lpic1-search-release-set;lpic1-search-incomplete-intake;lpic1-search-private-handover | Reorganize a project without losing data / Terminal and file basics |
| 103.4 | Partial | shell-streams;shell-pipelines;shell-report;shell-loop | Separate report data from diagnostics / Terminal and file basics |
| 103.5 | Partial | lpic1-process-pause;lpic1-process-pending;lpic1-process-priority;lpic1-process-handover | Find a runaway process family / Linux file foundations and ordinary-user access policy |
| 103.6 | Partial | lpic1-process-priority;lpic1-process-handover | Restore interactive workload priority / Linux file foundations and ordinary-user access policy |
| 103.7 | Open | None | Select structured failures with regex / Terminal and file basics |
| 103.8 | Open | None | Repair a setting in a modal editor / Terminal and file basics |
| 104.1 | Open | None | Choose and create a suitable filesystem / Terminal and file basics |
| 104.2 | Open | None | Recover an inconsistent filesystem / Terminal and file basics |
| 104.3 | Open | None | Repair a persistent removable mount / Terminal and file basics |
| 104.5 | Partial | lpic1-permissions;lpic1-traversal;lpic1-team-brief;lpic1-drop-directory;lpic1-update-existing;lpic1-retire-handover | Give a colleague least-privilege access / Terminal and file basics |
| 104.6 | Open | None | Repair a broken link deployment / Terminal and file basics |
| 104.7 | Open | None | Place and locate an application file / Terminal and file basics |
| 105.1 | Partial | shell-functions;shell-return;shell-bundle | Fix a startup environment override / Terminal and file basics |
| 105.2 | Partial | shell-validation;shell-loop;shell-eof;shell-batch-audit;shell-return;shell-bundle;shell-capture-preview;shell-capture-evidence;shell-capture-decision;shell-capture-handover;shell-case-interface;shell-case-routing;shell-case-publication | Build a guarded multi-file report / Terminal and file basics |
| 106.1 | Open | None | Diagnose a display configuration mismatch / Terminal and file basics |
| 106.2 | Open | None | Choose a remote desktop arrangement / Terminal and file basics |
| 106.3 | Open | None | Choose accessible workstation settings / Terminal and file basics |
| 107.1 | Partial | lpic1-account-onboarding;lpic1-account-membership;lpic1-account-retirement;lpic1-account-team-handover;lpic1-traversal | Onboard a user with correct group access / Terminal and file basics |
| 107.2 | Partial | lpic1-cron-weekday-report;lpic1-cron-environment-output;lpic1-cron-access-retirement;lpic1-cron-calendar-handover | Schedule a report without overlapping runs / Terminal and file basics; duration/locking remains required |
| 107.3 | Open | None | Repair a locale-sensitive report / Terminal and file basics |
| 108.1 | Open | None | Diagnose clock drift and time sources / Terminal and file basics |
| 108.2 | Partial | lpic2-service | Route and rotate useful logs / Terminal and file basics |
| 108.3 | Open | None | Trace a failed local notification / Terminal and file basics |
| 108.4 | Open | None | Recover a paused print queue / Terminal and file basics |
| 109.1 | Open | None | Choose a route for mixed IP traffic / Terminal and file basics |
| 109.2 | Open | None | Make a client network repair persistent / Terminal and file basics |
| 109.3 | Open | None | Localize a connectivity break / Terminal and file basics |
| 109.4 | Open | None | Repair resolver search-order mistakes / Terminal and file basics |
| 110.1 | Open | None | Audit unwanted access and listeners / Terminal and file basics |
| 110.2 | Open | None | Restrict remote administration safely / Terminal and file basics |
| 110.3 | Open | None | Validate encrypted exchange and trust / Terminal and file basics |

## LPIC-2 v4.5

| Objective | Current slice | Credited authored IDs | Required proposed task / prerequisites |
| --- | --- | --- | --- |
| 200.1 | Open | None | Find the actual resource bottleneck / LPIC-1 foundation track |
| 200.2 | Open | None | Forecast the next capacity upgrade / LPIC-1 foundation track |
| 201.1 | Open | None | Choose a compatible kernel component / LPIC-1 foundation track |
| 201.2 | Open | None | Plan a reproducible kernel build / LPIC-1 foundation track |
| 201.3 | Open | None | Recover a failed module load / LPIC-1 foundation track |
| 202.1 | Partial | lpic2-service;lpic2-unit-loaded-recovery;lpic2-unit-preparation-order;lpic2-unit-optional-wants;lpic2-unit-target-handover | Restore service dependencies and boot behavior / LPIC-1 foundation track |
| 202.2 | Open | None | Recover a broken root boot path / LPIC-1 foundation track |
| 202.3 | Open | None | Repair a UEFI boot chain / LPIC-1 foundation track |
| 203.1 | Partial | lpic2-storage;lpic2-mount-identity;lpic2-mount-policy;lpic2-mount-manual;lpic2-mount-handover | Recover a data volume with the right mount / LPIC-1 foundation track |
| 203.2 | Open | None | Diagnose inode exhaustion / LPIC-1 foundation track |
| 203.3 | Open | None | Tune filesystem behavior for a workload / LPIC-1 foundation track |
| 204.1 | Open | None | Replace a failed mirror member / LPIC-1 foundation track |
| 204.2 | Open | None | Diagnose a storage path mismatch / LPIC-1 foundation track |
| 204.3 | Open | None | Extend a volume while preserving data / LPIC-1 foundation track |
| 205.1 | Open | None | Restore dual-stack server connectivity / LPIC-1 foundation track |
| 205.2 | Open | None | Repair a bonded VLAN uplink / LPIC-1 foundation track |
| 205.3 | Open | None | Trace intermittent network failures / LPIC-1 foundation track |
| 206.1 | Open | None | Stage and verify a source installation / LPIC-1 foundation track |
| 206.2 | Open | None | Restore a consistent incremental backup / LPIC-1 foundation track |
| 206.3 | Open | None | Communicate a planned maintenance window / LPIC-1 foundation track |
| 207.1 | Partial | lpic2-dns;lpic2-dns-dualstack;lpic2-dns-aliases;lpic2-dns-mail;lpic2-dns-rrsets;lpic2-dns-secondary;lpic2-dns-handover | Restore recursive versus authoritative behavior / LPIC-1 foundation track |
| 207.2 | Partial | lpic2-dns;lpic2-dns-dualstack;lpic2-dns-aliases;lpic2-dns-mail;lpic2-dns-rrsets;lpic2-dns-secondary;lpic2-dns-handover;lpic2-dns-reverse-investigation;lpic2-dns-reverse-ipv6;lpic2-dns-reverse-handover | Repair forward and reverse zone consistency / LPIC-1 foundation track |
| 207.3 | Open | None | Restrict DNS transfer and recursion / LPIC-1 foundation track |
| 208.1 | Partial | lpic2-web-routing;lpic2-web-pages;lpic2-web-access;lpic2-web-handover | Diagnose an authenticated site using request and error logs / LPIC-1 foundation track |
| 208.2 | Open | None | Repair a TLS identity chain / LPIC-1 foundation track |
| 208.3 | Open | None | Fix a proxy access-policy leak / LPIC-1 foundation track |
| 208.4 | Open | None | Recover a reverse proxy backend / LPIC-1 foundation track |
| 209.1 | Open | None | Repair a team Samba share / LPIC-1 foundation track |
| 209.2 | Partial | lpic2-nfs-reference;lpic2-nfs-publication;lpic2-nfs-handover;lpic2-nfs-capstone | Restrict an NFS export correctly / LPIC-1 foundation track |
| 210.1 | Open | None | Repair an exhausted DHCP scope / LPIC-1 foundation track |
| 210.2 | Open | None | Diagnose authentication stack order / LPIC-1 foundation track |
| 210.3 | Open | None | Fix an LDAP client search base / LPIC-1 foundation track |
| 210.4 | Open | None | Delegate directory access safely / LPIC-1 foundation track |
| 211.1 | Open | None | Trace a mail routing loop / LPIC-1 foundation track |
| 211.2 | Open | None | Filter and deliver local mail reliably / LPIC-1 foundation track |
| 211.3 | Open | None | Repair mailbox authentication and TLS / LPIC-1 foundation track |
| 212.1 | Open | None | Repair forwarding and firewall rules / LPIC-1 foundation track |
| 212.2 | Open | None | Constrain a file-transfer account / LPIC-1 foundation track |
| 212.3 | Open | None | Recover key-based administration / LPIC-1 foundation track |
| 212.4 | Open | None | Assess exposed services from evidence / LPIC-1 foundation track |
| 212.5 | Open | None | Repair a VPN route and peer policy / LPIC-1 foundation track |

## Actual authored prerequisite inventory

These IDs and prerequisites come from the versioned packs, not inferred lesson
titles. Empty prerequisites mean no enforced prior lab; introductions must still
teach the skills they assess. Experienced learners may open assignments directly.

| Track / authored ID | Title | Actual scenario prerequisites |
| --- | --- | --- |
| LPIC-1 / `lpic1-account-membership` | Repair team membership without changing unrelated ownership | `lpic1-account-onboarding` |
| LPIC-1 / `lpic1-account-onboarding` | Create an account with the right home template | None declared |
| LPIC-1 / `lpic1-account-retirement` | Retire an account while preserving the right evidence | `lpic1-account-membership` |
| LPIC-1 / `lpic1-account-team-handover` | Handover a team without losing numeric ownership | `lpic1-account-retirement` |
| LPIC-1 / `lpic1-copy-move` | Keep a snapshot and process a receipt | `lpic1-file-names` |
| LPIC-1 / `lpic1-creation-mask` | Create a private working area by default | `lpic1-file-cleanup` |
| LPIC-1 / `lpic1-drop-directory` | Configure a drop folder without browsing | `lpic1-team-brief` |
| LPIC-1 / `lpic1-file-cleanup` | Clean a staging folder without losing the release | `lpic1-copy-move` |
| LPIC-1 / `lpic1-file-handover` | Prepare a safe project handover | `lpic1-creation-mask` |
| LPIC-1 / `lpic1-file-names` | Package notes with spaces in their names | `lpic1-file-navigation` |
| LPIC-1 / `lpic1-file-navigation` | Find a project and prepare its handover | None declared |
| LPIC-1 / `lpic1-logs` | Find the failing requests | None declared |
| LPIC-1 / `lpic1-package-handover` | Retire two tools with different retention policies | `lpic1-package-payload` |
| LPIC-1 / `lpic1-package-install` | Install a tool without disturbing its neighbours | None declared |
| LPIC-1 / `lpic1-package-metadata` | Refresh a stale list and meet a new dependency | `lpic1-package-install` |
| LPIC-1 / `lpic1-package-payload` | Repair damaged files without losing local settings | `lpic1-package-metadata` |
| LPIC-1 / `lpic1-permissions` | Share a report safely | None declared |
| LPIC-1 / `lpic1-process-handover` | Complete a safe process handover | `lpic1-process-pending`; `lpic1-process-priority` |
| LPIC-1 / `lpic1-process-pause` | Pause the right importer | `lpic1-file-navigation`; `lpic1-team-brief` |
| LPIC-1 / `lpic1-process-pending` | End the paused archiver | `lpic1-process-pause` |
| LPIC-1 / `lpic1-process-priority` | Let the batch copy yield | `lpic1-process-pause` |
| LPIC-1 / `lpic1-retire-handover` | Retire a read-only handover safely | `lpic1-update-existing`; `lpic1-file-cleanup` |
| LPIC-1 / `lpic1-rpm-epoch` | Update a client when epoch changes the order | `lpic1-rpm-provider` |
| LPIC-1 / `lpic1-rpm-handover` | Hand an API over before retiring its provider | `lpic1-rpm-payload` |
| LPIC-1 / `lpic1-rpm-payload` | Repair recorded payload without upgrading it | `lpic1-rpm-epoch` |
| LPIC-1 / `lpic1-rpm-provider` | Find a provider beyond its package name | `lpic1-file-navigation` |
| LPIC-1 / `lpic1-search-incomplete-intake` | Recover an incomplete intake investigation | `lpic1-search-release-set`; `lpic1-drop-directory` |
| LPIC-1 / `lpic1-search-private-handover` | Complete a private support handover | `lpic1-search-incomplete-intake`; `lpic1-update-existing`; `lpic1-retire-handover` |
| LPIC-1 / `lpic1-search-release-set` | Build the right release evidence set | `lpic1-file-handover`; `lpic1-logs` |
| LPIC-1 / `lpic1-team-brief` | Share a brief with the review group | `lpic1-creation-mask`; `lpic1-file-names` |
| LPIC-1 / `lpic1-traversal` | Unlock the project directory | None declared |
| LPIC-1 / `lpic1-update-existing` | Update files inside a read-only directory | `lpic1-drop-directory`; `lpic1-copy-move` |
| LPIC-2 / `lpic2-dns` | Repair the internal DNS answer | None declared |
| LPIC-2 / `lpic2-dns-aliases` | Repair a looping service alias | `lpic2-dns-dualstack` |
| LPIC-2 / `lpic2-dns-dualstack` | Add a service with IPv4 and IPv6 | None declared |
| LPIC-2 / `lpic2-dns-handover` | Complete a DNS service handover | `lpic2-dns-secondary` |
| LPIC-2 / `lpic2-dns-mail` | Restore primary and backup mail discovery | `lpic2-dns-aliases` |
| LPIC-2 / `lpic2-dns-reverse-handover` | Retire an old address and hand over both copies | `lpic2-dns-reverse-ipv6` |
| LPIC-2 / `lpic2-dns-reverse-investigation` | Repair names returned for connection-log addresses | `lpic2-dns-handover` |
| LPIC-2 / `lpic2-dns-reverse-ipv6` | Publish reverse names for an IPv6 rollout | `lpic2-dns-reverse-investigation` |
| LPIC-2 / `lpic2-dns-rrsets` | Retire an obsolete address safely | `lpic2-dns-mail` |
| LPIC-2 / `lpic2-dns-secondary` | Bring a stale secondary up to date | `lpic2-dns-rrsets` |
| LPIC-2 / `lpic2-mount-handover` | Complete a stable storage handover | `lpic2-mount-identity`; `lpic2-mount-policy`; `lpic2-mount-manual` |
| LPIC-2 / `lpic2-mount-identity` | Identify and mount the assigned volume | None declared |
| LPIC-2 / `lpic2-mount-manual` | Keep maintenance storage manual but available | `lpic2-mount-identity`; `lpic2-mount-policy` |
| LPIC-2 / `lpic2-mount-policy` | Make saved and live mount policies agree | `lpic2-mount-identity` |
| LPIC-2 / `lpic2-nfs-capstone` | Complete the two-share publication handover | `lpic2-nfs-handover` |
| LPIC-2 / `lpic2-nfs-handover` | Hand a share to its replacement client | `lpic2-nfs-publication` |
| LPIC-2 / `lpic2-nfs-publication` | Diagnose and publish an existing result | `lpic2-nfs-reference` |
| LPIC-2 / `lpic2-nfs-reference` | Publish a restricted reference share | `lpic2-mount-policy`; `lpic1-traversal` |
| LPIC-2 / `lpic2-service` | Bring the service back | None declared |
| LPIC-2 / `lpic2-storage` | Recover the data volume | None declared |
| LPIC-2 / `lpic2-unit-loaded-recovery` | Recover the definition a running service actually uses | `lpic2-service` |
| LPIC-2 / `lpic2-unit-optional-wants` | Keep a failed optional helper from blocking the core | `lpic2-unit-preparation-order` |
| LPIC-2 / `lpic2-unit-preparation-order` | Make preparation a real prerequisite | `lpic2-unit-loaded-recovery` |
| LPIC-2 / `lpic2-unit-target-handover` | Hand over a multi-unit service without damaging its observer | `lpic2-unit-optional-wants` |
| LPIC-2 / `lpic2-web-access` | Keep public pages open and staff notes private | `lpic2-web-pages` |
| LPIC-2 / `lpic2-web-handover` | Hand over a release that survives its next start | `lpic2-web-access` |
| LPIC-2 / `lpic2-web-pages` | Choose a landing page and publish a correction | `lpic2-web-routing` |
| LPIC-2 / `lpic2-web-routing` | Route site names without changing the default site | None declared |

| LPIC-1 / `lpic1-cron-weekday-report` | Publish a weekday report on schedule | `lpic1-file-navigation`, `shell-handover` |
| LPIC-1 / `lpic1-cron-environment-output` | Repair a noninteractive report and its output | `lpic1-cron-weekday-report`, `shell-report` |
| LPIC-1 / `lpic1-cron-access-retirement` | Delegate scheduling and retire an installed job | `lpic1-cron-environment-output`, `lpic1-account-onboarding` |
| LPIC-1 / `lpic1-cron-calendar-handover` | Hand over a system report without losing peer jobs | `lpic1-cron-weekday-report`, `lpic1-cron-environment-output`, `lpic1-cron-access-retirement` |

## Recurring administration: implemented narrow unit

Four distinct tasks now follow the recorded prerequisites:

1. `lpic1-cron-weekday-report`: user table, weekday/minute boundaries, fresh source data; requires `lpic1-file-navigation`, `shell-handover`.
2. `lpic1-cron-environment-output`: ordered variables, literal percent escaping and append retention; requires the first task and `shell-report`.
3. `lpic1-cron-access-retirement`: allow/deny management policy versus removal of an already installed table; requires the previous task and `lpic1-account-onboarding`.
4. `lpic1-cron-calendar-handover`: system-table owner, day-of-month/weekday OR and removal of an obsolete user table; requires all preceding tasks.

Each keeps a full assignment, separate example, native table editor, explicit
virtual Run/Stop/output and sixteen independent sample dates/files. The precise
source boundary is CRON_SUBSET.md. The parent, K1/K2 and /etc/crontab/cron.allow/
cron.deny slices are Partial only. Editor Install/Remove is not Linux crontab
CLI practice; that tool row stays Open. One-off at jobs, anacron, systemd timers,
spool/directory administration, runtime duration/overlap and locking remain
missing. The planned non-overlap task is not marked finished by this subset.
Main native/candidate receipts are separately pending in CATALOG164_MILESTONE.md.

## Following Linux frontiers

LPIC-2 206.2 backup remains wholly Open for authored practical credit. A separate
virtual archive foundation and unregistered drafts are in development; selected
restore, snapshot lineage and damaged-backup recovery still need an integrated,
verified teaching unit. Existing user-progress backup is an
application safety feature, not Linux backup curriculum. Subsequent batches must
also address Open disk/filesystem/LVM/RAID, kernel/boot, logging/time, networking,
security/authentication and remaining site services. The complete parent table
and granular rows above retain every gap; this unit does not redefine completion.
