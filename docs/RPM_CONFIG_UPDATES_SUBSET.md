## Current integration —2026-11-15

This opt-in capability now supports the two authored patch activities in the503lab/
202unit normal catalog. Its old default remains unchanged. The initial501-only
capability notes below are historical. See RPM_PATCH_LESSONS.md for current scoped
model/content/native evidence; candidate/cold verification is not yet claimed.

# Configuration-preserving RPM updates (capability work)

2026-11-15. Catalog remains501/201. No patch/IDS lab or new objective credit is
registered by this document or model. The verified501candidate stays immutable.

The existing RPM planner deliberately refuses modified/missing configuration and
manifest changes. Keep that default and every existing lesson unchanged. A separate
pure in-process transition may opt in to this smaller content-handling table:

| Current versus recorded/incoming contents | Supported update |
| --- | --- |
| Missing or identical to recorded default | Install the incoming default. |
| Already identical to incoming default | Use the incoming default; no sidecar. |
| Incoming equals recorded default; local contents differ | Keep the local file; no sidecar. |
| All three contents differ, config_noreplace | Keep local; stage incoming as .rpmnew. |
| All three contents differ, config | Preserve local as .rpmsave; install incoming. |
| Erase an unchanged or missing configuration | Remove it; no sidecar. |
| Erase a locally changed configuration | Preserve local as .rpmsave. |

This initial table requires the same configuration path/flag manifest and unchanged
recorded mode/owner/group for each configuration across archives. Local modification times may differ; locally changed mode/owner/group remain
unsupported; the existing separate metadata-restoration operation
can preserve contents while restoring recorded attributes. Missing files are admitted
only by this new transition. Existing sidecar collisions, undeclared sidecar paths,
manifest/type changes and all other old unsupported operations refuse atomically.
Ordinary files, version ordering, explicit local availability, package identities,
ownership and the disclosed whole-database dependency rule use the existing planner.
No resolver, signature, scriptlet, actual RPM/daemon/filesystem/network execution.

Only immutable source-owned new tasks may choose this transition. No learner action,
imported State flag or old lesson enables it. Bound inputs/outputs using existing
RPM state/action budgets; all files remain virtual regular text. This is not complete
RPM conformance or a silent change to the old console. Code/state grading and actual
controlled comparison must precede authored patch lessons.

Primary references checked2026-11-15:
https://rpm.org/docs/6.0.x/manual/spec.html
https://rpm.org/docs/6.0.x/man/rpm.8
https://access.redhat.com/solutions/57665
Functional syntax/behavior is paraphrased; no upstream implementation imported.
Fixed comparison via MAIN tools/dev.py tool rpm_configuration_reference.py matches
all14content cases against RPM6.0.2. Only three tiny script-free fixed archives,
private payload leaves and a private database are used. Personal macros/plugins
and transaction scriptlets/triggers are disabled; no learner source/path selector.
Source/probe/fixture identities and actual observations are retained privately at
.build-artifacts/reference-runs/rpm-configuration-bdbj7hdf. This is a current fixed
content comparison, not an independently sealed baseline or full RPM conformance.
Initial setup failures remain retained. RPM worker counts are explicitly1; private
resource ceilings and MAIN lock/pressure guard remain. No system configuration,
services, real package database, owner files or dependencies changed.
