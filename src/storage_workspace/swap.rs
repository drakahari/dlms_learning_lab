//! Outcome checks for source-enabled swap, not command-string grading.
use crate::storage::{self, State, swap as core};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    Configured {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        area: String,
        automatic: bool,
        priority: u16,
    },
    Unconfigured {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        area: String,
    },
    Active {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        area: String,
        priority: u16,
    },
    Inactive {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        area: String,
    },
    Fresh {
        #[serde(deserialize_with = "storage::map")]
        devices: BTreeMap<String, String>,
        #[serde(deserialize_with = "storage::map")]
        expected: BTreeMap<String, Option<u16>>,
    },
}
pub struct Fresh {
    pub inventory: core::Inventory,
    pub active: BTreeMap<String, u16>,
    pub outcomes: Vec<core::Outcome>,
}
fn mapped(
    facts: &core::Inventory,
    devices: &BTreeMap<String, String>,
) -> Result<core::Inventory, String> {
    facts.validate().map_err(|e| e.to_string())?;
    let aliases = facts
        .areas
        .iter()
        .map(|a| a.device.as_str())
        .collect::<BTreeSet<_>>();
    let ids = facts
        .areas
        .iter()
        .map(|a| a.id.as_str())
        .collect::<BTreeSet<_>>();
    if devices.keys().map(String::as_str).collect::<BTreeSet<_>>() != aliases
        || devices
            .values()
            .map(String::as_str)
            .collect::<BTreeSet<_>>()
            != ids
    {
        return Err("Fresh swap policy must only permute the exact supplied device aliases; no new or duplicate area identities".into());
    }
    let mut facts = facts.clone();
    for area in &mut facts.areas {
        area.device = devices
            .iter()
            .find(|(_, id)| **id == area.id)
            .unwrap()
            .0
            .clone();
    }
    facts.validate().map_err(|e| e.to_string())?;
    Ok(facts)
}
pub fn fresh(state: &State, devices: &BTreeMap<String, String>) -> Result<Fresh, String> {
    let facts = state
        .inventory()
        .swap()
        .ok_or("This lesson does not enable swap")?;
    let inventory = mapped(facts, devices)?;
    inventory
        .validate_against(state.inventory())
        .map_err(|e| e.to_string())?;
    let mut active = BTreeMap::new();
    let outcomes = core::activate_all(&inventory, &mut active, state.configuration())
        .map_err(|e| e.to_string())?;
    Ok(Fresh {
        inventory,
        active,
        outcomes,
    })
}
pub fn valid(inventory: &storage::Inventory, requirements: &[&Requirement]) -> bool {
    let Some(facts) = inventory.swap() else {
        return requirements.is_empty();
    };
    if facts.validate_against(inventory).is_err() || requirements.is_empty() {
        return false;
    }
    let ids = facts
        .areas
        .iter()
        .map(|a| a.id.clone())
        .collect::<BTreeSet<_>>();
    let mut saved = BTreeSet::new();
    let mut live = BTreeSet::new();
    let mut cases = BTreeSet::new();
    let mut positive = false;
    for r in requirements {
        match r {
            Requirement::Configured { area, priority, .. } => {
                if *priority > 32767 || !ids.contains(area) || !saved.insert(area.clone()) {
                    return false;
                }
            }
            Requirement::Unconfigured { area } => {
                if !ids.contains(area) || !saved.insert(area.clone()) {
                    return false;
                }
            }
            Requirement::Active { area, priority } => {
                if *priority > 32767 || !ids.contains(area) || !live.insert(area.clone()) {
                    return false;
                }
                if !facts
                    .areas
                    .iter()
                    .any(|a| a.id == *area && a.available && a.prepared)
                {
                    return false;
                }
                positive = true;
            }
            Requirement::Inactive { area } => {
                if !ids.contains(area) || !live.insert(area.clone()) {
                    return false;
                }
            }
            Requirement::Fresh { devices, expected } => {
                if mapped(facts, devices).is_err()
                    || expected.keys().cloned().collect::<BTreeSet<_>>() != ids
                    || expected.values().flatten().any(|p| *p > 32767)
                    || expected.iter().any(|(id, p)| {
                        p.is_some()
                            && !facts
                                .areas
                                .iter()
                                .any(|a| a.id == *id && a.available && a.prepared)
                    })
                    || !expected.values().any(Option::is_some)
                    || !cases.insert(format!("{devices:?}"))
                {
                    return false;
                }
            }
        }
    }
    let original = facts
        .areas
        .iter()
        .map(|a| (a.device.clone(), a.id.clone()))
        .collect::<BTreeMap<_, _>>();
    saved == ids
        && live == ids
        && positive
        && !cases.is_empty()
        && requirements
            .iter()
            .any(|r| matches!(r,Requirement::Fresh{devices,..}if *devices!=original))
}
impl Requirement {
    pub fn check(&self, state: &State) -> (bool, String) {
        let Some(facts) = state.inventory().swap() else {
            return (
                false,
                "This lesson does not enable swap requirements".into(),
            );
        };
        match self {
            Self::Configured {
                area,
                automatic,
                priority,
            } => {
                let rows = match state.swap_entries() {
                    Ok(rows) => rows,
                    Err(e) => return (false, format!("Saved swap policy: {e}")),
                };
                let row = rows
                    .iter()
                    .find(|e| facts.resolve(&e.source).is_ok_and(|a| a.id == *area));
                match row {
                    None => (
                        false,
                        format!(
                            "Saved {area}: no matching area entry; compare the supplied UUID with the first fstab field"
                        ),
                    ),
                    Some(e) if e.priority != *priority => (
                        false,
                        format!(
                            "Saved {area}: priority is {}, task requires {priority}. Edit pri in the saved row; that alone will not change its active priority",
                            e.priority
                        ),
                    ),
                    Some(e) if e.automatic != *automatic => (
                        false,
                        format!(
                            "Saved {area}: {} required. A remaining noauto token skips bulk activation even if auto appears later; remove noauto to request automatic activation. It does not prevent an explicit request",
                            if *automatic {
                                "automatic activation"
                            } else {
                                "noauto"
                            }
                        ),
                    ),
                    Some(_) => (
                        true,
                        format!(
                            "Saved {area}: {} with priority {priority}",
                            if *automatic { "automatic" } else { "manual" }
                        ),
                    ),
                }
            }
            Self::Unconfigured { area } => match state.swap_entries() {
                Err(e) => (false, format!("Saved swap policy: {e}")),
                Ok(rows) => {
                    let exists = rows
                        .iter()
                        .any(|e| facts.resolve(&e.source).is_ok_and(|a| a.id == *area));
                    (
                        !exists,
                        format!(
                            "Saved policy: remove the {area} row, rather than merely deactivating it now"
                        ),
                    )
                }
            },
            Self::Active { area, priority } => match state.active_swap().get(area) {
                None => (
                    false,
                    format!(
                        "Active {area}: not active. Saving a row does not activate the area; inspect the active table, then use a supported activation command"
                    ),
                ),
                Some(now) if now != priority => (
                    false,
                    format!(
                        "Active {area}: priority is {now}, task requires {priority}. Swapon -a skips an already-active area; in this no-workload activity deactivate it, then activate with the intended priority"
                    ),
                ),
                Some(_) => (true, format!("Active {area}: priority {priority}")),
            },
            Self::Inactive { area } => (
                !state.active_swap().contains_key(area),
                format!(
                    "Active table: {area} must be inactive; removing its saved row does not deactivate it"
                ),
            ),
            Self::Fresh { devices, expected } => {
                let f = match fresh(state, devices) {
                    Ok(f) => f,
                    Err(e) => return (false, format!("Fresh swap-policy test: {e}")),
                };
                if let Some(core::Outcome::Failed { line, reason }) = f
                    .outcomes
                    .iter()
                    .find(|o| matches!(o, core::Outcome::Failed { .. }))
                {
                    return (
                        false,
                        format!("Fresh swap-policy test line{line}: {reason}"),
                    );
                }
                for (area, want) in expected {
                    if f.active.get(area) != want.as_ref() {
                        return (
                            false,
                            format!(
                                "Fresh swap-policy test {area}: expected {}, found {} after supplied device aliases changed. Check stable UUID identity, auto/noauto and pri",
                                want.map(|p| format!("active priority {p}"))
                                    .unwrap_or_else(|| "inactive".into()),
                                f.active
                                    .get(area)
                                    .map(|p| format!("active priority {p}"))
                                    .unwrap_or_else(|| "inactive".into())
                            ),
                        );
                    }
                }
                (true,"Fresh swap-policy test: automatic areas and priorities survive the supplied alias permutation; manual/retired areas stay inactive. This is an independent model copy, not a system boot".into())
            }
        }
    }
}
pub const HELP: &str = "Source-enabled swap only: swapon --show --bytes; swapon -a; swapon -p N DEVICE; swapon -p N -U UUID; swapoff DEVICE; swapoff -U UUID. N is explicit0–32767; higher is preferred. Source facts contain prepared, no-workload areas only. Use the native editor for six-field source none swap options 0 0 rows, with one pri=N and optional auto/noauto/defaults. Saving does not activate/reprioritize; -a skips active/noauto. No host swap, paging, memory policy, swap files, automatic negative priorities, discard, nofail or general utility forms. Unsupported syntax refuses honestly.";
pub fn command(state: &mut State, parts: &[&str]) -> Result<String, String> {
    if state.inventory().swap().is_none() {
        return Err("Swap is not enabled by this lesson".into());
    }
    let pri = |s: &str| {
        s.parse::<u16>()
            .ok()
            .filter(|n| s.bytes().all(|b| b.is_ascii_digit()) && *n <= 32767)
            .ok_or("Use explicit priority0–32767".to_owned())
    };
    match parts {
        ["swapon", "--show", "--bytes"] => state.swap_listing().map_err(|e| e.to_string()),
        ["swapon", "-a"] => {
            let out = state.swap_on_all().map_err(|e| e.to_string())?;
            let errors = out
                .iter()
                .filter_map(|o| match o {
                    core::Outcome::Failed { line, reason } => {
                        Some(format!("swapon: saved line{line}: {reason}"))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            if errors.is_empty() {
                Ok(String::new())
            } else {
                Err(format!(
                    "{}\nSimulator note: some activation requests failed; inspect the active table. Other successful requests remain active. No full utility exit-status emulation.",
                    errors.join("\n")
                ))
            }
        }
        ["swapon", "-p", n, device] => state
            .swap_on(&core::source(device).map_err(|e| e.to_string())?, pri(n)?)
            .map(|_| String::new())
            .map_err(|e| e.to_string()),
        ["swapon", "-p", n, "-U", u] => state
            .swap_on(
                &core::source(&format!("UUID={u}")).map_err(|e| e.to_string())?,
                pri(n)?,
            )
            .map(|_| String::new())
            .map_err(|e| e.to_string()),
        ["swapoff", device] => state
            .swap_off(&core::source(device).map_err(|e| e.to_string())?)
            .map(|_| String::new())
            .map_err(|e| e.to_string()),
        ["swapoff", "-U", u] => state
            .swap_off(&core::source(&format!("UUID={u}")).map_err(|e| e.to_string())?)
            .map(|_| String::new())
            .map_err(|e| e.to_string()),
        _ => Err(format!("Unsupported swap arguments. {HELP}")),
    }
}
