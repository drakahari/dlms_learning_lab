//! Current-state availability goals over source-bound virtual storage.
use super::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    pub active: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "bounded::sequence::<_,Group,4>")]
    pub groups: Vec<Group>,
}
impl Plan {
    pub(super) fn valid(&self, s: &lvm::State) -> bool {
        let Some(availability) = &s.activation else {
            return false;
        };
        if s.validate().is_err() || self.groups.is_empty() || self.groups.len() > 4 {
            return false;
        }
        let mut names = BTreeSet::new();
        for target in &self.groups {
            let Some(group) = s.groups.get(&target.name) else {
                return false;
            };
            if !names.insert(&target.name)
                || group.volumes.is_empty()
                || (!target.active
                    && availability
                        .open
                        .iter()
                        .any(|id| id.split_once('/').is_some_and(|(g, _)| g == target.name)))
            {
                return false;
            }
        }
        true
    }
    pub(super) fn outcomes(
        &self,
        initial: &lvm::State,
        current: &lvm::State,
    ) -> Vec<(bool, String)> {
        if !self.valid(initial) || !same_problem(initial, current) {
            return vec![(
                false,
                "Availability source/session differs; previous work retained".into(),
            )];
        }
        let now = current.activation.as_ref().unwrap();
        let before = initial.activation.as_ref().unwrap();
        let mut rows = Vec::new();
        for target in &self.groups {
            let Some(_group) = current.groups.get(&target.name) else {
                rows.push((false,format!("{}: original group is missing; restore the supplied allocation rather than recreating data",target.name)));
                continue;
            };
            let originals = &initial.groups[&target.name].volumes;
            let active = originals
                .keys()
                .filter(|v| now.active.get(&format!("{}/{v}", target.name)) == Some(&true))
                .count();
            let ok = originals
                .keys()
                .all(|v| now.active.get(&format!("{}/{v}", target.name)) == Some(&target.active));
            rows.push((
                ok,
                format!(
                    "{}: make its LVs {}; current {active}/{} active.",
                    target.name,
                    if target.active {
                        "available"
                    } else {
                        "inactive"
                    },
                    originals.len()
                ),
            ));
            if target.active {
                for (name, v) in &initial.groups[&target.name].volumes {
                    if let Some(fs) = &v.filesystem {
                        for (path, file) in &fs.files {
                            let id = format!("{}/{name}", target.name);
                            let result = current.read_file(&id, path);
                            let (ok, status) = match result {
                                Ok(text) if text == file.contents => {
                                    (true, "available; original contents retained".into())
                                }
                                Ok(_) => {
                                    (false, "readable, but the supplied contents changed".into())
                                }
                                Err(e) => (false, format!("unavailable: {e}")),
                            };
                            rows.push((
                                ok,
                                format!("Simulated application reader {id}:{path}: {status}"),
                            ));
                        }
                    }
                }
            }
        }
        for (id, value) in &before.active {
            if self
                .groups
                .iter()
                .any(|g| id.split_once('/').is_some_and(|(name, _)| name == g.name))
            {
                continue;
            }
            let actual = now.active.get(id);
            rows.push((
                actual == Some(value),
                format!(
                    "Unrelated {id}: preserve its supplied {} state; current {}",
                    if *value { "active" } else { "inactive" },
                    match actual {
                        Some(true) => "active",
                        Some(false) => "inactive",
                        None => "LV missing",
                    }
                ),
            ));
        }
        let protected = initial.groups == current.groups && initial.devices == current.devices;
        rows.push((protected,"Keep every original allocation, file, device and open-consumer fact unchanged; no new storage".into()));
        rows
    }
}
