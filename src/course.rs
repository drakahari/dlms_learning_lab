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
pub fn load(text: &str, scenarios: &[Scenario]) -> Result<Vec<Unit>, String> {
    if text.len() > 256_000 {
        return Err("Unit index exceeds size limit".into());
    }
    let index: Index = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if index.version != 1 || index.units.is_empty() || index.units.len() > 100 {
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
