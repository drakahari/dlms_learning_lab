//! Typed current-state retirement goals; no host I/O or reference execution.
use super::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "ids::<_,8>")]
    pub retired_volumes: Vec<String>,
    #[serde(deserialize_with = "ids::<_,4>")]
    pub retired_groups: Vec<String>,
    #[serde(deserialize_with = "ids::<_,8>")]
    pub cleared_pvs: Vec<String>,
}
fn ids<'de, D: serde::Deserializer<'de>, const N: usize>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Id(#[serde(deserialize_with = "crate::cron::text::<_,64>")] String);
    bounded::sequence::<_, Id, N>(d).map(|v| v.into_iter().map(|v| v.0).collect())
}
impl Plan {
    pub(super) fn valid(&self, initial: &lvm::State) -> bool {
        if initial.provisioning != Some(lvm::provision::Profile::Lifecycle)
            || initial.validate().is_err()
            || self.retired_volumes.len() > 8
            || self.retired_groups.len() > 4
            || self.cleared_pvs.len() > 8
            || self.retired_volumes.len() + self.retired_groups.len() + self.cleared_pvs.len() == 0
        {
            return false;
        }
        let mut seen = BTreeSet::new();
        for id in &self.retired_volumes {
            if id.starts_with("/dev/")
                || !seen.insert(id)
                || !initial
                    .volume(id)
                    .is_ok_and(|(_, v)| v.filesystem.is_none())
            {
                return false;
            }
        }
        let mut groups = BTreeSet::new();
        for id in &self.retired_groups {
            let Some(g) = initial.groups.get(id) else {
                return false;
            };
            if !groups.insert(id)
                || g.volumes
                    .keys()
                    .any(|lv| !self.retired_volumes.contains(&format!("{id}/{lv}")))
            {
                return false;
            }
        }
        let mut pvs = BTreeSet::new();
        for id in &self.cleared_pvs {
            let Some(d) = initial.devices.get(id) else {
                return false;
            };
            if !pvs.insert(id)
                || d.role != lvm::DeviceRole::PhysicalVolume
                || (!d.group.is_empty() && !self.retired_groups.contains(&d.group))
            {
                return false;
            }
        }
        self.expected(initial).validate().is_ok()
    }
    fn expected(&self, initial: &lvm::State) -> lvm::State {
        let mut s = initial.clone();
        for id in &self.retired_volumes {
            let (g, lv) = id.split_once('/').unwrap();
            s.groups.get_mut(g).unwrap().volumes.remove(lv);
        }
        for id in &self.retired_groups {
            s.groups.remove(id);
            for d in s.devices.values_mut().filter(|d| d.group == *id) {
                d.group.clear();
            }
        }
        for id in &self.cleared_pvs {
            s.devices.get_mut(id).unwrap().role = lvm::DeviceRole::Blank;
        }
        s
    }
    pub(super) fn outcomes(
        &self,
        initial: &lvm::State,
        current: &lvm::State,
    ) -> Vec<(bool, String)> {
        if !self.valid(initial) || !same_problem(initial, current) {
            return vec![(false,"This saved work does not match the supplied retirement task; previous work retained".into())];
        }
        let mut rows = Vec::new();
        for id in &self.retired_volumes {
            let missing = current.volume(id).is_err();
            rows.push((
                missing,
                format!(
                    "{id}: retire this unused raw LV; current {}",
                    if missing { "absent" } else { "still present" }
                ),
            ));
        }
        for id in &self.retired_groups {
            let missing = !current.groups.contains_key(id);
            rows.push((
                missing,
                format!(
                    "{id}: retire the empty group; current {}",
                    if missing { "absent" } else { "still present" }
                ),
            ));
        }
        for id in &self.cleared_pvs {
            let d = &current.devices[id];
            let removed = d.role == lvm::DeviceRole::Blank;
            let status = if removed {
                "no modeled PV label".into()
            } else if d.group.is_empty() {
                "still an unowned labeled PV".into()
            } else {
                format!("still belongs to {}", d.group)
            };
            rows.push((
                removed,
                format!("{id}: clear only its unused LVM label; current {status}"),
            ));
        }
        let expected = self.expected(initial);
        let groups = self
            .retired_volumes
            .iter()
            .filter_map(|id| id.split_once('/').map(|p| p.0))
            .filter(|g| !self.retired_groups.iter().any(|id| id == g))
            .collect::<BTreeSet<_>>();
        for g in groups {
            let want = expected.free_extents(g).unwrap();
            let have = current.free_extents(g).ok();
            rows.push((
                have == Some(want),
                format!(
                    "{g}: return unused extents to the pool; expected {want} free, current {}",
                    have.map_or_else(|| "group missing".into(), |n| n.to_string())
                ),
            ));
        }
        // Ignore only the specifically requested pending removals when checking
        // unrelated protection. Incomplete retirement must not falsely blame data.
        let mut kept = current.groups.clone();
        for id in &self.retired_groups {
            kept.remove(id);
        }
        for id in &self.retired_volumes {
            let (g, lv) = id.split_once('/').unwrap();
            if let Some(group) = kept.get_mut(g) {
                group.volumes.remove(lv);
            }
        }
        let mut expected_groups = initial.groups.clone();
        for id in &self.retired_groups {
            expected_groups.remove(id);
        }
        for id in &self.retired_volumes {
            let (g, lv) = id.split_once('/').unwrap();
            if let Some(group) = expected_groups.get_mut(g) {
                group.volumes.remove(lv);
            }
        }
        let protected = kept == expected_groups
            && initial.devices.iter().all(|(id, d)| {
                if self.cleared_pvs.contains(id) || self.retired_groups.contains(&d.group) {
                    true
                } else {
                    current.devices.get(id) == Some(d)
                }
            });
        rows.push((protected,"Keep all unrelated groups, LVs, files, devices and allocations unchanged; do not add extra storage".into()));
        for (id, d) in &initial.devices {
            if self.retired_groups.contains(&d.group) && !self.cleared_pvs.contains(id) {
                let now = &current.devices[id];
                rows.push((
                    now.role == lvm::DeviceRole::PhysicalVolume && now.group.is_empty(),
                    format!(
                        "{id}: retain its PV label but release ownership of retired group {}",
                        d.group
                    ),
                ));
            }
        }
        rows
    }
}

/// Source-bound inventory context permits supported raw retirement/recreation,
/// while every original filesystem-bearing object keeps its allocation prefix.
/// Wrong but structurally valid changes are left for outcome feedback.
pub(super) fn same_problem(initial: &lvm::State, current: &lvm::State) -> bool {
    if model(initial).is_err()
        || model(current).is_err()
        || initial.provisioning != current.provisioning
        || initial.devices.keys().ne(current.devices.keys())
    {
        return false;
    }
    for (id, d) in &initial.devices {
        let now = &current.devices[id];
        if d.bytes != now.bytes
            || match d.role {
                lvm::DeviceRole::Foreign => d != now,
                lvm::DeviceRole::Blank => now.role == lvm::DeviceRole::Foreign,
                lvm::DeviceRole::PhysicalVolume => {
                    now.role == lvm::DeviceRole::Foreign
                        || (!d.group.is_empty()
                            && initial.groups[&d.group]
                                .volumes
                                .values()
                                .any(|v| v.filesystem.is_some())
                            && (d.group != now.group || d.role != now.role))
                }
            }
        {
            return false;
        }
    }
    for (id, g) in &initial.groups {
        let Some(now) = current.groups.get(id) else {
            if g.volumes.values().any(|v| v.filesystem.is_some()) {
                return false;
            }
            continue;
        };
        if g.extent_bytes != now.extent_bytes {
            return false;
        }
        for (name, v) in &g.volumes {
            if let Some(fs) = &v.filesystem {
                let Some(a) = now.volumes.get(name) else {
                    return false;
                };
                if !a.extents.starts_with(&v.extents)
                    || a.filesystem
                        .as_ref()
                        .is_none_or(|f| f.files.keys().ne(fs.files.keys()))
                {
                    return false;
                }
            }
        }
    }
    true
}
