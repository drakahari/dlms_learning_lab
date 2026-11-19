//! State-based raw allocation plans, separate from existing growth targets.
use super::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    pub extent_bytes: u64,
    #[serde(deserialize_with = "device_list")]
    pub devices: Vec<String>,
    pub reserve_extents: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Volume {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub bytes: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "bounded::sequence::<_,Group,2>")]
    pub groups: Vec<Group>,
    #[serde(deserialize_with = "bounded::sequence::<_,Volume,4>")]
    pub volumes: Vec<Volume>,
}
fn device_list<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Path(#[serde(deserialize_with = "crate::cron::text::<_,64>")] String);
    bounded::sequence::<_, Path, 8>(d).map(|rows| rows.into_iter().map(|p| p.0).collect())
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && !s.starts_with(['-', '.'])
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-+.".contains(&c))
}
impl Plan {
    pub(super) fn valid(&self, initial: &lvm::State) -> bool {
        if initial.provisioning != Some(lvm::provision::Profile::Linear)
            || self.groups.is_empty()
            || self.groups.len() > 2
            || self.volumes.is_empty()
            || self.volumes.len() > 4
        {
            return false;
        }
        let mut groups = BTreeSet::new();
        let mut devices = BTreeSet::new();
        for g in &self.groups {
            if !name(&g.name)
                || !groups.insert(&g.name)
                || !(lvm::MIB..=64 * lvm::MIB).contains(&g.extent_bytes)
                || !g.extent_bytes.is_power_of_two()
                || g.devices.is_empty()
                || g.devices.len() > 8
                || g.reserve_extents > 4096
                || initial
                    .groups
                    .get(&g.name)
                    .is_some_and(|b| b.extent_bytes != g.extent_bytes)
            {
                return false;
            }
            for p in &g.devices {
                if p.len() > 64
                    || !devices.insert(p)
                    || !initial.devices.get(p).is_some_and(|d| {
                        d.role != lvm::DeviceRole::Foreign
                            && (d.group.is_empty() || d.group == g.name)
                    })
                {
                    return false;
                }
            }
            if initial
                .devices
                .iter()
                .any(|(p, d)| d.group == g.name && !g.devices.contains(p))
            {
                return false;
            }
        }
        let mut volumes = BTreeSet::new();
        for v in &self.volumes {
            let Some((g, n)) = v.id.split_once('/') else {
                return false;
            };
            let Some(group) = self.groups.iter().find(|x| x.name == g) else {
                return false;
            };
            if !name(g)
                || !lvm::provision::valid_volume_name(n)
                || !volumes.insert(&v.id)
                || v.bytes == 0
                || v.bytes > 128 * 64 * lvm::MIB
                || v.bytes % group.extent_bytes != 0
                || initial
                    .groups
                    .get(g)
                    .is_some_and(|x| x.volumes.contains_key(n))
            {
                return false;
            }
        }
        let extra_groups = self
            .groups
            .iter()
            .filter(|g| !initial.groups.contains_key(&g.name))
            .count();
        initial.groups.len() + extra_groups <= lvm::MAX_GROUPS
            && initial
                .groups
                .values()
                .map(|g| g.volumes.len())
                .sum::<usize>()
                + self.volumes.len()
                <= lvm::MAX_VOLUMES
    }
    pub(super) fn outcomes(
        &self,
        initial: &lvm::State,
        current: &lvm::State,
    ) -> Vec<(bool, String)> {
        if !self.valid(initial) || !same_problem(initial, current) {
            return vec![(
                false,
                "This saved work does not match the supplied device plan; previous work retained"
                    .into(),
            )];
        }
        let mut rows = Vec::new();
        for g in &self.groups {
            let actual = current
                .devices
                .iter()
                .filter(|(_, d)| d.group == g.name)
                .map(|(p, _)| p.as_str())
                .collect::<BTreeSet<_>>();
            let expected = g
                .devices
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            rows.push((
                current.groups.contains_key(&g.name) && actual == expected,
                if current.groups.contains_key(&g.name) {
                    format!("{}: use exactly these supplied devices: {}; current members: {}",g.name,g.devices.join(", "),actual.iter().copied().collect::<Vec<_>>().join(", "))
                } else {
                    format!("{}: group missing. Create it on the supplied devices {} before allocating its LVs.",g.name,g.devices.join(", "))
                },
            ));
            let extent = current.groups.get(&g.name).map(|g| g.extent_bytes);
            rows.push((
                extent == Some(g.extent_bytes),
                format!(
                    "{}: use {} MiB extents; current {}",
                    g.name,
                    g.extent_bytes / lvm::MIB,
                    extent.map_or_else(
                        || "group missing".into(),
                        |n| format!("{} MiB", n / lvm::MIB)
                    )
                ),
            ));
            let free = current.free_extents(&g.name).ok();
            rows.push((
                free.is_some_and(|n| n >= g.reserve_extents),
                format!(
                    "{}: keep at least {} free extents; current {}",
                    g.name,
                    g.reserve_extents,
                    free.map_or_else(|| "group missing".into(), |n| n.to_string())
                ),
            ));
        }
        for v in &self.volumes {
            let bytes = current.lv_bytes(&v.id).ok();
            let raw = current
                .volume(&v.id)
                .is_ok_and(|(_, lv)| lv.filesystem.is_none());
            rows.push((
                bytes == Some(v.bytes) && raw,
                format!(
                    "{}: create a raw LV of {} MiB ({} bytes); current {}",
                    v.id,
                    v.bytes / lvm::MIB,
                    v.bytes,
                    bytes.map_or_else(
                        || "volume missing".into(),
                        |n| format!(
                            "{} MiB ({n} bytes); {}",
                            n / lvm::MIB,
                            if raw {
                                "no filesystem"
                            } else {
                                "unexpected filesystem"
                            }
                        )
                    )
                ),
            ));
        }
        let preserved = initial.groups.iter().all(|(name, g)| {
            current
                .groups
                .get(name)
                .is_some_and(|a| g.volumes.iter().all(|(id, v)| a.volumes.get(id) == Some(v)))
        }) && initial.devices.iter().all(|(p, d)| {
            self.groups.iter().any(|g| g.devices.contains(p)) || current.devices.get(p) == Some(d)
        });
        rows.push((
            preserved,
            "Keep all originally supplied LVs, files and unrelated devices unchanged".into(),
        ));
        let extras = current.groups.iter().all(|(vg, g)| {
            (initial.groups.contains_key(vg) || self.groups.iter().any(|x| x.name == *vg))
                && g.volumes.keys().all(|lv| {
                    initial
                        .groups
                        .get(vg)
                        .is_some_and(|x| x.volumes.contains_key(lv))
                        || self.volumes.iter().any(|x| x.id == format!("{vg}/{lv}"))
                })
        });
        rows.push((
            extras,
            "Create only the requested groups and LVs; remove unintended new raw allocations"
                .into(),
        ));
        rows
    }
}
