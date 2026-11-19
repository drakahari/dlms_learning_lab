//! Additive identity-bound context. Legacy named-object guards stay unchanged.
use super::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Change {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "bounded::sequence::<_,Change,4>")]
    pub groups: Vec<Change>,
    #[serde(deserialize_with = "bounded::sequence::<_,Change,8>")]
    pub volumes: Vec<Change>,
}
impl Plan {
    pub(super) fn valid(&self, s: &lvm::State) -> bool {
        if s.identity.is_none()
            || s.validate().is_err()
            || self.groups.len() > 4
            || self.volumes.len() > 8
            || self.groups.len() + self.volumes.len() == 0
        {
            return false;
        }
        let mut seen = BTreeSet::new();
        for c in &self.groups {
            if !seen.insert(c.source.as_str())
                || !s.groups.contains_key(&c.source)
                || c.source == c.name
                || lvm::validate_command(&format!("vgrename {} {}", c.source, c.name)).is_err()
            {
                return false;
            }
        }
        seen.clear();
        for c in &self.volumes {
            if !seen.insert(c.source.as_str())
                || c.source.starts_with("/dev/")
                || s.volume(&c.source).is_err()
                || !lvm::provision::valid_volume_name(&c.name)
                || c.source.split_once('/').is_some_and(|(_, v)| v == c.name)
            {
                return false;
            }
        }
        // Final names must be unique across the whole supplied inventory, not
        // merely across requested targets. Temporary intermediate names are OK.
        let mut gs = BTreeSet::new();
        let mut vs = BTreeSet::new();
        for (g, d) in &s.groups {
            if !gs.insert(self.group_name(g)) {
                return false;
            }
            for v in d.volumes.keys() {
                if !vs.insert(self.volume_name(g, v)) {
                    return false;
                }
            }
        }
        true
    }
    fn group_name<'a>(&'a self, source: &'a str) -> &'a str {
        self.groups
            .iter()
            .find(|c| c.source == source)
            .map_or(source, |c| c.name.as_str())
    }
    fn volume_name(&self, group: &str, volume: &str) -> String {
        let source = format!("{group}/{volume}");
        let v = self
            .volumes
            .iter()
            .find(|c| c.source == source)
            .map_or(volume, |c| c.name.as_str());
        format!("{}/{v}", self.group_name(group))
    }
    pub(super) fn outcomes(
        &self,
        initial: &lvm::State,
        current: &lvm::State,
    ) -> Vec<(bool, String)> {
        if !self.valid(initial) || !same_problem(initial, current) {
            return vec![(
                false,
                "Rename task/session differs; previous work retained".into(),
            )];
        }
        let before = initial.identity.as_ref().unwrap();
        let now = current.identity.as_ref().unwrap();
        let mut rows = Vec::new();
        let mut storage =
            before.groups.len() == now.groups.len() && before.volumes.len() == now.volumes.len();
        let mut availability = true;
        for (g, d) in &initial.groups {
            let actual = now
                .groups
                .iter()
                .find(|(_, id)| **id == before.groups[g])
                .map(|(n, _)| n.as_str());
            let want = self.group_name(g);
            rows.push((
                actual == Some(want),
                format!(
                    "Original group {g}: required name {want}; current {}",
                    actual.unwrap_or("original group missing")
                ),
            ));
            storage &= actual.is_some_and(|n| current.groups[n].extent_bytes == d.extent_bytes);
            for (v, data) in &d.volumes {
                let old = format!("{g}/{v}");
                let id = before.volumes[&old];
                let want = self.volume_name(g, v);
                let actual = now
                    .volumes
                    .iter()
                    .find(|(_, i)| **i == id)
                    .map(|(n, _)| n.as_str());
                rows.push((
                    actual == Some(want.as_str()),
                    format!(
                        "Original LV {old}: required path {want}; current {}",
                        actual.unwrap_or(
                            "original LV missing; recreating a name does not restore it"
                        )
                    ),
                ));
                storage &= actual.is_some_and(|n| {
                    let (group, lv) = n.split_once('/').unwrap();
                    now.groups[group] == before.groups[g]
                        && current.groups[group].volumes[lv] == *data
                });
                availability &= actual.is_some_and(|n| {
                    current.activation.as_ref().unwrap().active.get(n)
                        == initial.activation.as_ref().unwrap().active.get(&old)
                });
                if initial.activation.as_ref().unwrap().active[&old]
                    && let Some(fs) = &data.filesystem
                {
                    for (path, file) in &fs.files {
                        let text = actual.and_then(|n| current.read_file(n, path).ok());
                        rows.push((
                            text == Some(file.contents.as_str()),
                            format!(
                                "Reader for original {old}:{path}: {}",
                                if text == Some(file.contents.as_str()) {
                                    "original contents available"
                                } else {
                                    "unavailable or contents changed"
                                }
                            ),
                        ));
                    }
                }
            }
        }
        storage &= initial.devices.iter().all(|(path, d)| {
            let a = &current.devices[path];
            a.bytes == d.bytes
                && a.role == d.role
                && if d.group.is_empty() {
                    a.group.is_empty()
                } else {
                    now.groups.get(&a.group) == before.groups.get(&d.group)
                }
        });
        rows.push((storage,"Keep original allocations, files, capacities and PV membership; no extra or recreated storage".into()));
        rows.push((
            availability,
            "Preserve each LV's active/inactive state and supplied open consumer".into(),
        ));
        rows
    }
}
pub(super) fn same_problem(initial: &lvm::State, current: &lvm::State) -> bool {
    let (Some(before), Some(now)) = (&initial.identity, &current.identity) else {
        return false;
    };
    if model(initial).is_err()
        || model(current).is_err()
        || initial.provisioning != current.provisioning
        || initial.devices.keys().ne(current.devices.keys())
        || now.next < before.next
    {
        return false;
    }
    // Existing identities cannot change their kind, and new objects cannot reuse
    // any retired source ID. Neither command history nor display text is trusted.
    let old_groups = before.groups.values().copied().collect::<BTreeSet<_>>();
    let old_volumes = before.volumes.values().copied().collect::<BTreeSet<_>>();
    if now
        .groups
        .values()
        .any(|id| *id < before.next && !old_groups.contains(id))
        || now
            .volumes
            .values()
            .any(|id| *id < before.next && !old_volumes.contains(id))
    {
        return false;
    }
    let group_name = |id: &u32| {
        now.groups
            .iter()
            .find(|(_, v)| *v == id)
            .map(|(n, _)| n.as_str())
    };
    let volume_name = |id: &u32| {
        now.volumes
            .iter()
            .find(|(_, v)| *v == id)
            .map(|(n, _)| n.as_str())
    };
    for (path, d) in &initial.devices {
        let a = &current.devices[path];
        if d.bytes != a.bytes
            || match d.role {
                lvm::DeviceRole::Foreign => d != a,
                lvm::DeviceRole::Blank => a.role == lvm::DeviceRole::Foreign,
                lvm::DeviceRole::PhysicalVolume => {
                    a.role == lvm::DeviceRole::Foreign
                        || (!d.group.is_empty()
                            && initial.groups[&d.group]
                                .volumes
                                .values()
                                .any(|v| v.filesystem.is_some())
                            && (a.role != d.role
                                || group_name(&before.groups[&d.group]) != Some(a.group.as_str())))
                }
            }
        {
            return false;
        }
    }
    for (group, g) in &initial.groups {
        let Some(new_name) = group_name(&before.groups[group]) else {
            if g.volumes.values().any(|v| v.filesystem.is_some()) {
                return false;
            }
            continue;
        };
        let a = &current.groups[new_name];
        if a.extent_bytes != g.extent_bytes {
            return false;
        }
        for (name, v) in &g.volumes {
            let id = before.volumes[&format!("{group}/{name}")];
            let Some(new_path) = volume_name(&id) else {
                if v.filesystem.is_some() {
                    return false;
                }
                continue;
            };
            let (new_group, new_lv) = new_path.split_once('/').unwrap();
            let after = &current.groups[new_group].volumes[new_lv];
            if new_group != new_name
                || !after.extents.starts_with(&v.extents)
                || after.filesystem.is_some() != v.filesystem.is_some()
                || v.filesystem
                    .as_ref()
                    .zip(after.filesystem.as_ref())
                    .is_some_and(|(f, a)| f.files.keys().ne(a.files.keys()))
            {
                return false;
            }
        }
    }
    // Consumers follow their original object, never its former pathname.
    let a = initial.activation.as_ref().unwrap();
    let b = current.activation.as_ref().unwrap();
    let original_open = a.open.iter().map(|v| before.volumes[v]).collect::<Vec<_>>();
    let current_open = b.open.iter().map(|v| now.volumes[v]).collect::<Vec<_>>();
    original_open == current_open
}
