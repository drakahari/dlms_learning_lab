//! Declarative unit order and advisory prerequisites; never a mastery or access lock.
use crate::{
    pack::Scenario,
    progress::{self, Progress},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Unit {
    pub id: String,
    pub track: String,
    pub title: String,
    pub summary: String,
    pub prerequisites: Vec<String>,
    pub labs: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Index {
    version: u32,
    units: Vec<Unit>,
}
/// Finite metadata capacity; the 256,000-byte and per-unit bounds remain separate.
pub const MAX_UNITS:usize=256;
pub fn load(text: &str, scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    if text.len() > 256_000 {
        return Err("Unit index exceeds size limit".into());
    }
    let index: Index = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if index.version != 1 || index.units.is_empty() || index.units.len() > MAX_UNITS {
        return Err("Unsupported unit index".into());
    }
    let ids: BTreeSet<_> = index.units.iter().map(|u| u.id.as_str()).collect();
    if ids.len() != index.units.len() {
        return Err("Duplicate unit identifier".into());
    }
    let mut covered = BTreeSet::new();
    for u in &index.units {
        if u.id.is_empty()
            || u.id.len() > 80
            || !u
                .id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || u.title.is_empty()
            || u.title.len() > 160
            || u.summary.is_empty()
            || u.summary.len() > 1000
            || u.labs.is_empty()
            || u.labs.len() > 200
            || u.prerequisites.len() > 8
            || u.prerequisites.iter().any(|p| !ids.contains(p.as_str()))
        {
            return Err(format!("Invalid unit {}", u.id));
        }
        for id in &u.labs {
            let s = scenarios
                .iter()
                .find(|s| &s.id == id)
                .ok_or("Unit references an unknown lab")?;
            if s.track != u.track || s.unit != u.id || !covered.insert(id) {
                return Err("Lab must belong to exactly one unit of its track".into());
            }
        }
        let mut pending = u.prerequisites.clone();
        let mut seen = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if id == u.id {
                return Err("Cyclic unit prerequisites".into());
            }
            if seen.insert(id.clone()) {
                pending.extend(
                    index
                        .units
                        .iter()
                        .find(|u| u.id == id)
                        .unwrap()
                        .prerequisites
                        .clone(),
                );
            }
        }
    }
    if covered.len() != scenarios.len() {
        return Err("Every authored lab needs a unit".into());
    }
    Ok(index.units)
}
pub fn bundled(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    load(include_str!("../scenarios/units-v1.json"), scenarios)
}
/// Separate review index; preserves all existing unit objects, IDs and prerequisites.
fn accepted_index() -> Result<Index, String> {
    let mut index: Index =
        serde_json::from_str(include_str!("../scenarios/ccna-magic-units-v1.json"))
            .map_err(|e| e.to_string())?;
    let base: Index = serde_json::from_str(include_str!("../scenarios/units-v1.json"))
        .map_err(|e| e.to_string())?;
    if index.version != 1 || base.version != 1 {
        return Err("Unsupported pilot unit index".into());
    }
    index.units.extend(base.units);
    let gateways: Index = serde_json::from_str(include_str!("../scenarios/hsrp-units-v1.json"))
        .map_err(|e| e.to_string())?;
    if gateways.version != 1 {
        return Err("Unsupported gateway unit index".into());
    }
    index.units.extend(gateways.units);
    Ok(index)
}
pub fn accepted209(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    load(
        &serde_json::to_string(&accepted_index()?).map_err(|e| e.to_string())?,
        scenarios,
    )
}
/// Append current reviewed curriculum without changing accepted unit objects.
pub fn accepted213(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let mut units = accepted_index()?.units;
    let extra: Index =
        serde_json::from_str(include_str!("../scenarios/continuation-units-v1.json"))
            .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported continuation unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}
pub fn next(unit: &Unit, current: &str, scenarios: &[Scenario], p: &Progress) -> Option<usize> {
    let position = unit.labs.iter().position(|id| id == current)?;
    // Continue forward, then offer earlier unfinished work. Never jump certification tracks.
    unit.labs
        .iter()
        .skip(position + 1)
        .chain(unit.labs.iter().take(position))
        .filter_map(|id| scenarios.iter().position(|s| &s.id == id))
        .find(|i| {
            p.attempts
                .get(&scenarios[*i].id)
                .is_none_or(|a| !progress::current_solved(a, &scenarios[*i]))
        })
}

/// Current review index; prior213 unit objects remain exact.
pub fn accepted221(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted213()?;
    let mut units = accepted213(&prior)?;
    let extra: Index = serde_json::from_str(include_str!("../scenarios/syslog-units-v1.json"))
        .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported syslog unit index".into());
    }
    units.extend(extra.units);
    let capacity: Index = serde_json::from_str(include_str!("../scenarios/capacity-units-v1.json"))
        .map_err(|e| e.to_string())?;
    if capacity.version != 1 {
        return Err("Unsupported capacity unit index".into());
    }
    units.extend(capacity.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Append repository evidence without changing any previous unit objects.
pub fn accepted225(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted221()?;
    let mut units = accepted221(&prior)?;
    let extra: Index = serde_json::from_str(include_str!("../scenarios/repository-units-v1.json"))
        .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported repository unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Append the host-networking unit, retaining prior225 content and prerequisites.
pub fn accepted229(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted225()?;
    let mut units = accepted225(&prior)?;
    let extra: Index = serde_json::from_str(include_str!("../scenarios/linux-host-units-v1.json"))
        .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported host unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Debug/recovery projects reuse evaluator15 and preserve all existing unit objects.
pub fn accepted231(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted229()?;
    let mut units = accepted229(&prior)?;
    let extra: Index = serde_json::from_str(include_str!(
        "../scenarios/shell-debug-recovery-units-v1.json"
    ))
    .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported debugging/recovery unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Classic Linux logging fragments, separate from existing Cisco source policies.
pub fn accepted235(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted231()?;
    let mut units = accepted231(&prior)?;
    let extra: Index =
        serde_json::from_str(include_str!("../scenarios/linux-logging-units-v1.json"))
            .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported Linux logging unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Current startup-policy practice retains the complete historical235 catalog.
pub fn accepted238(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted235()?;
    let mut units = accepted235(&prior)?;
    let extra: Index =
        serde_json::from_str(include_str!("../scenarios/linux-startup-units-v1.json"))
            .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported startup-policy unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Per-hop policy practice appends its own unit; historical238 objects stay exact.
pub fn accepted241(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted238()?;
    let mut units = accepted238(&prior)?;
    let extra: Index = serde_json::from_str(include_str!("../scenarios/qos-units-v1.json"))
        .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported QoS unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Application-specific selection follows the complete existing backup unit.
pub fn accepted242(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted241()?;
    let mut units = accepted241(&prior)?;
    let extra: Index =
        serde_json::from_str(include_str!("../scenarios/backup-selection-units-v1.json"))
            .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported backup-selection unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Linux server configuration is additive; all242 previous labs and66 units remain exact.
pub fn accepted244(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted242()?;
    let mut units = accepted242(&prior)?;
    let extra: Index = serde_json::from_str(include_str!("../scenarios/linux-dhcp-units-v1.json"))
        .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported DHCP configuration unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Persistent Ethernet configuration follows the existing host networking prerequisites.
pub fn accepted247(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted244()?;
    let mut units = accepted244(&prior)?;
    let extra: Index = serde_json::from_str(include_str!("../scenarios/linux-nm-units-v1.json"))
        .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported persistent-networking unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

pub fn accepted248(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted247()?;
    let mut units = accepted247(&prior)?;
    let extra: Index =
        serde_json::from_str(include_str!("../scenarios/removable-mount-units-v1.json"))
            .map_err(|e| e.to_string())?;
    if extra.version != 1 {
        return Err("Unsupported removable-mount unit index".into());
    }
    units.extend(extra.units);
    load(
        &serde_json::to_string(&Index { version: 1, units }).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Local address-book practice follows the existing host-resolver prerequisites.
pub fn accepted250(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior = crate::pack::accepted248()?;
    let mut units = accepted248(&prior)?;
    let extra: Index = serde_json::from_str(include_str!("../scenarios/client-name-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version != 1 { return Err("Unsupported client-name unit index".into()); }
    units.extend(extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

pub fn accepted252(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted250()?;let mut units=accepted250(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/resolver-search-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported resolver-search unit index".into());}
    units.extend(extra.units);load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Repository file-provider investigation follows the existing Debian package unit.
pub fn accepted253(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let prior=crate::pack::accepted252()?;let mut units=accepted252(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/debian-contents-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported Debian Contents unit index".into());}
    units.extend(extra.units);load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Source selection and partial-refresh recovery follow package/Contents work.
pub fn accepted255(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted253()?;let mut units=accepted253(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/debian-sources-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported source-metadata unit index".into());}
    units.extend(extra.units);load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Tracing uses foundation-only syntax and precedes more complex composition.
pub fn accepted257(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted255()?;let mut units=accepted255(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/shell-tracing-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported tracing unit index".into());}
    let at=units.iter().position(|u|u.id=="shell-foundations").ok_or("Missing tracing prerequisite unit")?+1;
    units.splice(at..at,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Child settings follow the filesystem foundation; shell basics remain an explicit prerequisite.
pub fn accepted259(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted257()?;let mut units=accepted257(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/environment-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported environment unit index".into());}
    let at=units.iter().position(|u|u.id=="linux-file-foundations").ok_or("Missing environment prerequisite unit")?+1;
    units.splice(at..at,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Caller-aware removable sessions follow the required/optional startup unit.
pub fn accepted261(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted259()?;let mut units=accepted259(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/user-mount-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported user-mount unit index".into());}
    let at=units.iter().position(|u|u.id=="linux-removable-startup").ok_or("Missing caller-mount prerequisite unit")?+1;
    units.splice(at..at,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Incompatibility repair follows the original package administration progression.
pub fn accepted263(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted261()?;let mut units=accepted261(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/debian-conflict-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported package-conflict unit index".into());}
    let at=units.iter().position(|u|u.id=="debian-package-administration").ok_or("Missing package-conflict prerequisite unit")?+1;
    units.splice(at..at,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Supplied DHCP address settings follow the static saved-profile progression.
pub fn accepted265(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted263()?;let mut units=accepted263(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/linux-nm-dhcp-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported automatic Ethernet unit index".into());}
    let at=units.iter().position(|u|u.id=="linux-persistent-networking").ok_or("Missing automatic Ethernet prerequisite unit")?+1;
    units.splice(at..at,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Offline consistency follows link identity and caller-aware mount foundations.
pub fn accepted267(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted265()?;let mut units=accepted265(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/fs-integrity-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported filesystem-consistency unit index".into());}
    let mounts=units.iter().position(|u|u.id=="linux-removable-callers").ok_or("Missing consistency mount prerequisite")?;
    let links=units.iter().position(|u|u.id=="linux-link-handover").ok_or("Missing consistency file-identity prerequisite")?;
    let at=mounts.max(links)+1;
    units.splice(at..at,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Startup-file practice follows exported-value foundations.
pub fn accepted269(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted267()?;let mut units=accepted267(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/shell-startup-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported shell startup unit index".into());}
    let at=units.iter().position(|u|u.id=="linux-shell-environment").ok_or("Missing startup environment prerequisite")?+1;
    units.splice(at..at,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Careful file projects follow exact snapshots and the debugging foundations.
pub fn accepted272(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted269()?;let mut units=accepted269(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/shell-file-lifecycle-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported file-lifecycle unit index".into());}
    let snapshot=units.iter().position(|u|u.id=="shell-snapshot-rollout").ok_or("Missing file-lifecycle snapshot prerequisite")?;
    let tracing=units.iter().position(|u|u.id=="shell-tracing-diagnosis").ok_or("Missing file-lifecycle debugging prerequisite")?;
    units.splice(snapshot.max(tracing)+1..snapshot.max(tracing)+1,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Configuration decisions follow package compatibility and explicit upgrade foundations.
pub fn accepted275(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted272()?;let mut units=accepted272(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/debian-conffile-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported configuration-upgrade unit index".into());}
    let at=units.iter().position(|u|u.id=="debian-package-conflicts").ok_or("Missing configuration-upgrade prerequisite")?+1;
    units.splice(at..at,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// RAID1 follows device/media identity and persistent-mount foundations.
pub fn accepted277(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted275()?;let mut units=accepted275(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/raid1-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported mirror-recovery unit index".into());}
    let disk=units.iter().position(|u|u.id=="linux-disk-preparation").ok_or("Missing disk preparation prerequisite")?;
    let mount=units.iter().position(|u|u.id=="persistent-mounts").ok_or("Missing persistent-mount prerequisite")?;
    let at=disk.max(mount)+1;units.splice(at..at,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

/// Named lookups follow both positional lists and record-processing foundations.
mod metadata;

/// Current metadata, validated against this caller's supplied scenarios on every call.
pub fn accepted279(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current279_index()?;
    load(
        &serde_json::to_string(&index).map_err(|e| e.to_string())?,
        scenarios,
    )
}


/// Exact preceding shared-library unit index, validated against its caller.
pub fn accepted281(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current281_index()?;
    load(
        &serde_json::to_string(&index).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Exact preceding file-location unit index, validated against its caller.
pub fn accepted283(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current283_index()?;
    load(
        &serde_json::to_string(&index).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Exact preceding managed-configuration unit index.
pub fn accepted284(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current284_index()?;
    load(
        &serde_json::to_string(&index).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Exact preceding listener-audit unit index.
pub fn accepted285(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current285_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

/// Exact preceding worker-search unit index.
pub fn accepted286(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current286_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

/// Exact preceding account-default unit index.
pub fn accepted287(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index=metadata::current287_index()?;
    load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)
}

/// Exact preceding command-eligibility unit index.
pub fn accepted288(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let index=metadata::current288_index()?;
    load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)
}
/// Exact preceding private-index unit index.
pub fn accepted289(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let index=metadata::current289_index()?;
    load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)
}
/// Exact preceding 290-lab unit index; validates each supplied catalog.
pub fn accepted290(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let index=metadata::current290_index()?;
    load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)
}
/// Exact preceding292-lab unit snapshot, validating the supplied scenario catalog.
pub fn accepted292(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let index=metadata::current292_index()?;
    load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)
}
/// Exact verified 294-lab unit snapshot, validating each caller catalog.
pub fn accepted294(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let index=metadata::current294_index()?;
    load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)
}
/// Exact verified transport unit index.
pub fn accepted296(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let index=metadata::current296_index()?;
    load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)
}
/// Exact verified298-lab unit index, validating each caller.
pub fn accepted298(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current298_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
/// Exact verified300/98 unit index.
pub fn accepted300(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current300_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
/// Exact verified303/100 unit index.
pub fn accepted303(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current303_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
/// Exact verified304/101 unit index; no later requirements or credit.
pub fn accepted304(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current304_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
/// Exact verified305/102 unit index; later content earns no inherited credit.
pub fn accepted305(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current305_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
/// Exact verified307/103 unit index; later units are independent.
pub fn accepted307(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current307_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
/// Exact verified310/104 index before swap policy.
pub fn accepted310(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current310_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted312(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current312_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted315(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current315_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted317(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current317_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted319(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current319_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted321(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current321_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted323(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current323_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
/// Exact preceding111-unit index; later lessons earn no inherited credit.
pub fn accepted326(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current326_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted329(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current329_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted333(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current333_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted331(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current331_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted337(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current337_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted335(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current335_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted338(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current338_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted340(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current340_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted341(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current341_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted342(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current342_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted343(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current343_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted346(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current346_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted349(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current349_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted351(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current351_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted354(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current354_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted356(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current356_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted358(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current358_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted360(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current360_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted380(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current380_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact verified 142-unit catalog.
pub fn accepted381(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current381_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

/// Exact verified 143-unit catalog.
pub fn accepted382(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current382_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

pub fn accepted383(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current383_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

pub fn accepted384(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current384_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

pub fn accepted385(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current385_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

pub fn accepted386(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current386_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

pub fn accepted387(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current387_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

pub fn accepted390(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current390_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

pub fn accepted392(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current392_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted394(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current394_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted396(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current396_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted399(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current399_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted402(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current402_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted404(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current404_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted406(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current406_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted409(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current409_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted412(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current412_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted415(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current415_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted417(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current417_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted419(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current419_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
/// Exact162unit metadata retained before WLAN-profile curriculum integration.
pub fn accepted421(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current421_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted487(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current487_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted499(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current499_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted501(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current501_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted511(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current511_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted513(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current513_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted515(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current515_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted517(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current517_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted519(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current519_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted521(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current521_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted523(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current523_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted526(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current526_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted528(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current528_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted530(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current530_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted532(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current532_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted534(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current534_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted536(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current536_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted538(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current538_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted540(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current540_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted541(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current541_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn accepted542(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current542_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}
pub fn ccna_pilot(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current543_index()?;
    load(
        &serde_json::to_string(&index).map_err(|e| e.to_string())?,
        scenarios,
    )
}

/// Exact129unit metadata retained before directory-data lessons.
pub fn accepted362(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current362_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact130unit metadata retained before utility lessons.
pub fn accepted364(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current364_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted372(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current372_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

// Preserve this cfg(test) oracle until one real guarded comparison has passed.
// Its function body is byte-identical to the source-hashed original ccna_pilot.
/// Exact165unit metadata for current428catalog.
pub fn accepted428(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current428_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted434(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current434_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted436(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current436_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact181unit objects retained before standalone-sharing integration.
pub fn accepted455(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current455_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted458(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current458_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact183unit metadata retained before guest-image support.
pub fn accepted461(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current461_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted485(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current485_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted490(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current490_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted494(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current494_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted493(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current493_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact202unit values retained before offline authentication-filter practice.
pub fn accepted503(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current503_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact203 unit values retained before IDS-rule curriculum.
pub fn accepted505(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current505_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted509(scenarios:&[Scenario])->Result<Vec<Unit>,String>{load(&serde_json::to_string(&metadata::current509_index()?).map_err(|e|e.to_string())?,scenarios)}

#[cfg(test)]
fn legacy279_course_index(scenarios:&[Scenario])->Result<Vec<Unit>,String>{
    let prior=crate::pack::accepted277()?;let mut units=accepted277(&prior)?;
    let extra:Index=serde_json::from_str(include_str!("../scenarios/shell-associative-units-v1.json")).map_err(|e|e.to_string())?;
    if extra.version!=1{return Err("Unsupported named-map unit index".into());}
    let arrays=units.iter().position(|u|u.id=="shell-arrays").ok_or("Missing named-map array prerequisite")?;
    let text=units.iter().position(|u|u.id=="shell-text").ok_or("Missing named-map text prerequisite")?;
    let at=arrays.max(text)+1;units.splice(at..at,extra.units);
    load(&serde_json::to_string(&Index{version:1,units}).map_err(|e|e.to_string())?,scenarios)
}

pub fn accepted366(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current366_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted369(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current369_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted368(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current368_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted367(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current367_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted371(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current371_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted370(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current370_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Current learner navigation, including every registered continuation unit.
pub fn accepted373(scenarios:&[Scenario])->Result<Vec<Unit>,String>{load(&serde_json::to_string(&metadata::current373_index()?).map_err(|e|e.to_string())?,scenarios)}

/// Exact139units retained before literal GRUB menu work.
pub fn accepted376(scenarios:&[Scenario])->Result<Vec<Unit>,String>{load(&serde_json::to_string(&metadata::current376_index()?).map_err(|e|e.to_string())?,scenarios)}

pub fn current(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> { ccna_pilot(scenarios) }

pub fn accepted378(scenarios:&[Scenario])->Result<Vec<Unit>,String>{load(&serde_json::to_string(&metadata::current378_index()?).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted424(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current424_index()?;
    load(
        &serde_json::to_string(&index).map_err(|e| e.to_string())?,
        scenarios,
    )
}

pub fn accepted426(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
 let index=metadata::current426_index()?;
 load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)
}

pub fn accepted430(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current430_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted432(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current432_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted437(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current437_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted438(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current438_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted439(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current439_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact173unit metadata retained before combined client-name diagnostics.
pub fn accepted441(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current441_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact174unit metadata retained before SSH administration integration.
pub fn accepted442(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current442_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact176unit metadata retained before modal-file editing integration.
pub fn accepted445(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current445_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact177 unit metadata retained before standard-editor handover.
pub fn accepted448(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current448_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact178unit objects retained before file-transfer curriculum integration.
pub fn accepted449(scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    let index = metadata::current449_index()?;
    load(&serde_json::to_string(&index).map_err(|e| e.to_string())?, scenarios)
}

/// Exact179 unit values retained before the campus architecture unit.
pub fn accepted454(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current454_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted452(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current452_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact184unit metadata retained before SMTP routing integration.
pub fn accepted464(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current464_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted470(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current470_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}
pub fn accepted467(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current467_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted472(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current472_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted475(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current475_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted477(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current477_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted478(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current478_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted481(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current481_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted482(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current482_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted489(scenarios:&[Scenario])->Result<Vec<Unit>,String>{load(&serde_json::to_string(&metadata::current489_index()?).map_err(|e|e.to_string())?,scenarios)}

pub fn accepted497(scenarios:&[Scenario])->Result<Vec<Unit>,String>{let index=metadata::current497_index()?;load(&serde_json::to_string(&index).map_err(|e|e.to_string())?,scenarios)}

/// Exact204unit metadata retained before PoE.
pub fn accepted507(scenarios:&[Scenario])->Result<Vec<Unit>,String>{load(&serde_json::to_string(&metadata::current507_index()?).map_err(|e|e.to_string())?,scenarios)}

#[cfg(test)]
mod metadata_regression {
    use super::*;

    #[test]
    fn current279_metadata_matches_actual_legacy_once_and_validates_each_caller() {
        // One genuine old-path execution, not a JSON/Python-generated expected result.
        // Keep all historical accessors unchanged; legacy function BODY must match original.
        let scenarios = crate::pack::accepted279().expect("current validated catalog");
        assert_eq!(scenarios.len(), 279);
        let old_start=std::time::Instant::now();
        let legacy = legacy279_course_index(&scenarios).expect("actual unchanged old path");
        assert_eq!(legacy.len(), 84);
        let legacy_bytes = serde_json::to_vec(&legacy).unwrap();
        let old_time=old_start.elapsed();
        let new_start=std::time::Instant::now();
        let actual = accepted279(&scenarios).expect("new path");
        let new_time=new_start.elapsed();
        eprintln!("COURSE_METADATA_TIMING old={old_time:?} current={new_time:?}");
        assert_eq!(serde_json::to_vec(&actual).unwrap(), legacy_bytes);
        let raw = metadata::current279_index().unwrap();
        assert_eq!(
            serde_json::to_vec(&raw).unwrap(),
            serde_json::to_vec(&Index { version: 1, units: legacy }).unwrap(),
            "exact raw Index serialization, all Unit fields and order"
        );

        // Repeat calls below must validate the supplied catalog, even after a valid call.
        assert!(accepted279(&[]).is_err());
        assert!(accepted279(&scenarios[..scenarios.len() - 1]).is_err());
        let mut changed = scenarios.clone();
        changed[0].id.push_str("-renamed");
        assert!(accepted279(&changed).is_err());
        changed = scenarios.clone();
        changed[0].track = "not-the-original-track".into();
        assert!(accepted279(&changed).is_err());
        changed = scenarios.clone();
        changed[0].unit = "not-the-original-unit".into();
        assert!(accepted279(&changed).is_err());
        changed = scenarios.clone();
        let mut extra = scenarios[0].clone();
        extra.id.push_str("-extra");
        changed.push(extra);
        assert!(accepted279(&changed).is_err());
        changed = scenarios.clone();
        changed.push(scenarios[0].clone());
        assert!(accepted279(&changed).is_err());

        // Caller order never affected Unit order and remains supported.
        changed = scenarios.clone();
        changed.reverse();
        assert_eq!(serde_json::to_vec(&accepted279(&changed).unwrap()).unwrap(), legacy_bytes);
        // This API validates Unit membership, not arbitrary Scenario teaching/goal content.
        changed[0].title = "A caller-specific title".into();
        assert_eq!(serde_json::to_vec(&accepted279(&changed).unwrap()).unwrap(), legacy_bytes);
        assert_eq!(serde_json::to_vec(&accepted279(&scenarios).unwrap()).unwrap(), legacy_bytes);
    }
}












