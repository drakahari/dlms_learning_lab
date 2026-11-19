//! Typed snapshot outcomes under proof. Native authoring remains closed.
use super::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Value {
    Original,
    Text {
        #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
        contents: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub record: String,
    pub expected: Value,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub volume: String,
    #[serde(deserialize_with = "bounded::sequence::<_,Record,32>")]
    pub records: Vec<Record>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    pub minimum_bytes: u64,
    pub maximum_bytes: u64,
    pub chunk_bytes: Option<u32>,
    #[serde(deserialize_with = "bounded::sequence::<_,Record,32>")]
    pub records: Vec<Record>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retire {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub volume: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reserve {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub group: String,
    pub minimum_bytes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "bounded::sequence::<_,Origin,4>")]
    pub origins: Vec<Origin>,
    #[serde(deserialize_with = "bounded::sequence::<_,Snapshot,8>")]
    pub snapshots: Vec<Snapshot>,
    #[serde(deserialize_with = "bounded::sequence::<_,Retire,8>")]
    pub retire: Vec<Retire>,
    #[serde(deserialize_with = "bounded::sequence::<_,Reserve,4>")]
    pub reserves: Vec<Reserve>,
}
fn full(origin: &str, name: &str) -> String {
    format!("{}/{name}", origin.split_once('/').unwrap().0)
}
fn bank<'a>(s: &'a lvm::State, origin: &str) -> Option<&'a lvm::cow::Bank> {
    s.snapshots.as_ref()?.origins.get(origin)
}
fn expected<'a>(initial: &'a lvm::cow::Bank, r: &'a Record) -> Option<&'a str> {
    match &r.expected {
        Value::Original => initial.current(&r.record).ok(),
        Value::Text { contents } => Some(contents),
    }
}
fn records_valid(bank: &lvm::cow::Bank, records: &[Record]) -> bool {
    let mut keys = BTreeSet::new();
    !records.is_empty()
        && records.len() <= 32
        && records.iter().all(|r| {
            keys.insert(&r.record)
                && expected(bank, r)
                    .is_some_and(|v| bank.validate_record_value(&r.record, v).is_ok())
        })
}
impl Plan {
    pub fn validate(&self, s: &lvm::State) -> Result<(), String> {
        model(s)?;
        if s.snapshots.is_none()
            || self.origins.len() > 4
            || self.snapshots.len() > 8
            || self.retire.len() > 8
            || self.reserves.len() > 4
            || self.snapshots.is_empty()
        {
            return Err(
                "Snapshot plans need a source profile and1–8complete snapshot outcomes".into(),
            );
        }
        let mut origin_keys = BTreeSet::new();
        for o in &self.origins {
            if !origin_keys.insert(&o.volume)
                || bank(s, &o.volume).is_none_or(|b| !records_valid(b, &o.records))
            {
                return Err("Unknown/duplicate origin or invalid fixed record requirements".into());
            }
        }
        let mut keys = BTreeSet::new();
        for t in &self.snapshots {
            let Some(b) = bank(s, &t.origin) else {
                return Err("Snapshot goal names an unknown supplied origin".into());
            };
            let path = full(&t.origin, &t.name);
            if !keys.insert(path.clone())
                || !lvm::provision::valid_volume_name(&t.name)
                || !records_valid(b, &t.records)
                || t.minimum_bytes == 0
                || t.minimum_bytes > t.maximum_bytes
                || t.maximum_bytes > lvm::cow::MAX_ORIGIN_BYTES
                || !t.minimum_bytes.is_multiple_of(lvm::MIB)
                || !t.maximum_bytes.is_multiple_of(lvm::MIB)
                || t.chunk_bytes
                    .is_some_and(|n| !(4096..=512 * 1024).contains(&n) || !n.is_power_of_two())
            {
                return Err(
                    "Invalid snapshot identity, capacity, chunk or record requirement".into(),
                );
            }
            if s.volume(&path).is_ok() && !b.snapshot_names().any(|n| n == t.name) {
                return Err(
                    "Required snapshot name collides with a supplied origin or unrelated LV".into(),
                );
            }
            if let Ok(status) = b.status(&t.name) {
                if !status.valid {
                    return Err("Do not claim recovery of an invalid initial view; retire it and name a new point in time".into());
                }
                if t.maximum_bytes < status.capacity_bytes
                    || t.chunk_bytes.is_some_and(|n| n != status.chunk_bytes)
                    || t.records
                        .iter()
                        .any(|r| b.read(&t.name, &r.record).ok().as_deref() != expected(b, r))
                {
                    return Err(
                        "An existing point in time must keep its data/chunks and cannot shrink"
                            .into(),
                    );
                }
            }
            if b.status(&t.name).is_ok_and(|r| !r.valid) {
                return Err("Do not claim recovery of an invalid initial view; retire it and name a new point in time".into());
            }
        }
        let mut counts = std::collections::BTreeMap::new();
        for t in &self.snapshots {
            *counts.entry(&t.origin).or_insert(0usize) += 1;
        }
        if counts.values().any(|n| *n > lvm::cow::MAX_SNAPSHOTS) {
            return Err("Too many required snapshots for one origin".into());
        }
        let mut gone = BTreeSet::new();
        for r in &self.retire {
            if !gone.insert(&r.volume)
                || keys.contains(&r.volume)
                || !s
                    .snapshots
                    .as_ref()
                    .unwrap()
                    .origins
                    .iter()
                    .any(|(origin, b)| b.snapshot_names().any(|n| full(origin, n) == r.volume))
            {
                return Err("Retire only distinct supplied snapshot stores, not origins or required new views".into());
            }
        }
        let mut groups = BTreeSet::new();
        for r in &self.reserves {
            if !groups.insert(&r.group)
                || !s.groups.contains_key(&r.group)
                || r.minimum_bytes == 0
                || !r.minimum_bytes.is_multiple_of(lvm::MIB)
            {
                return Err(
                    "Reserve goals need distinct supplied groups and a positive MiB amount".into(),
                );
            }
        }
        Ok(())
    }
}
/// Admission for a recoverable session, not proof of completion. Only actual COW
/// stores may be created/changed/removed; supplied raw origins, neighbor volumes,
/// devices, record layouts and VG geometry retain their exact source identity.
/// Deliberate data mistakes and extra snapshots stay editable and grade separately.
pub fn same_context(initial: &lvm::State, current: &lvm::State) -> bool {
    if model(initial).is_err()
        || model(current).is_err()
        || initial.devices != current.devices
        || initial.groups.keys().ne(current.groups.keys())
    {
        return false;
    }
    let (Some(old), Some(now)) = (&initial.snapshots, &current.snapshots) else {
        return false;
    };
    if old.origins.keys().ne(now.origins.keys()) {
        return false;
    }
    let stores = |m: &lvm::snapshot::Model| {
        m.origins
            .iter()
            .flat_map(|(origin, b)| b.snapshot_names().map(move |n| full(origin, n)))
            .collect::<BTreeSet<_>>()
    };
    let before_stores = stores(old);
    let after_stores = stores(now);
    for (g, before) in &initial.groups {
        let after = &current.groups[g];
        if before.extent_bytes != after.extent_bytes {
            return false;
        }
        for (n, v) in &before.volumes {
            if !before_stores.contains(&format!("{g}/{n}")) && after.volumes.get(n) != Some(v) {
                return false;
            }
        }
        for n in after.volumes.keys() {
            let path = format!("{g}/{n}");
            if !before.volumes.contains_key(n) && !after_stores.contains(&path) {
                return false;
            }
            // A removed initial COW name cannot be repurposed as an unrelated LV.
            if before_stores.contains(&path) && !after_stores.contains(&path) {
                return false;
            }
        }
    }
    old.origins
        .iter()
        .all(|(id, b)| now.origins[id].continues(b))
}
/// Exact source identity/layout/protected data, with only requested changes.
pub fn same_problem(initial: &lvm::State, current: &lvm::State, plan: &Plan) -> bool {
    if plan.validate(initial).is_err()
        || model(current).is_err()
        || initial.devices != current.devices
        || initial.groups.keys().ne(current.groups.keys())
    {
        return false;
    }
    let (Some(old), Some(now)) = (&initial.snapshots, &current.snapshots) else {
        return false;
    };
    if old.origins.keys().ne(now.origins.keys()) {
        return false;
    }
    let retired = plan
        .retire
        .iter()
        .map(|r| r.volume.as_str())
        .collect::<BTreeSet<_>>();
    let wanted = plan
        .snapshots
        .iter()
        .map(|t| full(&t.origin, &t.name))
        .collect::<BTreeSet<_>>();
    for (g, before) in &initial.groups {
        let after = &current.groups[g];
        if before.extent_bytes != after.extent_bytes {
            return false;
        }
        for (n, v) in &before.volumes {
            let path = format!("{g}/{n}");
            if retired.contains(path.as_str()) {
                continue;
            }
            let Some(a) = after.volumes.get(n) else {
                return false;
            };
            if wanted.contains(&path) {
                if a.filesystem != v.filesystem || !a.extents.starts_with(&v.extents) {
                    return false;
                }
            } else if a != v {
                return false;
            }
        }
        if after
            .volumes
            .keys()
            .any(|n| !before.volumes.contains_key(n) && !wanted.contains(&format!("{g}/{n}")))
        {
            return false;
        }
    }
    for (origin, before) in &old.origins {
        let after = &now.origins[origin];
        if !after.continues(before)
            || before.byte_len() != after.byte_len()
            || before.layout() != after.layout()
        {
            return false;
        }
        let changed = plan
            .origins
            .iter()
            .find(|o| o.volume == *origin)
            .map(|o| {
                o.records
                    .iter()
                    .map(|r| r.record.as_str())
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        if before
            .layout()
            .keys()
            .any(|r| !changed.contains(r.as_str()) && before.current(r) != after.current(r))
        {
            return false;
        }
        for n in before.snapshot_names() {
            let path = full(origin, n);
            if !retired.contains(path.as_str())
                && !after.keeps_snapshot(before, n, wanted.contains(&path))
            {
                return false;
            }
        }
        for n in after.snapshot_names() {
            let path = full(origin, n);
            if !before.snapshot_names().any(|old| old == n) && !wanted.contains(&path) {
                return false;
            }
        }
    }
    true
}
/// Current data/reservations, never walkthrough/check flags or command strings.
pub fn outcomes(initial: &lvm::State, current: &lvm::State, plan: &Plan) -> Vec<(bool, String)> {
    if plan.validate(initial).is_err() || model(current).is_err() || current.snapshots.is_none() {
        return vec![(
            false,
            "This snapshot state does not match its supported source task".into(),
        )];
    }
    let mut rows = Vec::new();
    for o in &plan.origins {
        let original = bank(initial, &o.volume).unwrap();
        let now = bank(current, &o.volume);
        let missing = o
            .records
            .iter()
            .filter(|r| now.and_then(|b| b.current(&r.record).ok()) != expected(original, r))
            .map(|r| r.record.as_str())
            .collect::<Vec<_>>();
        rows.push((
            missing.is_empty(),
            if missing.is_empty() {
                format!("{}: required current records match", o.volume)
            } else {
                format!(
                    "{}: inspect current records {}. They do not yet have the required content",
                    o.volume,
                    missing.join(", ")
                )
            },
        ));
    }
    for t in &plan.snapshots {
        let original = bank(initial, &t.origin).unwrap();
        let now = bank(current, &t.origin);
        let status = now.and_then(|b| b.status(&t.name).ok());
        let missing = t
            .records
            .iter()
            .filter(|r| {
                now.and_then(|b| b.read(&t.name, &r.record).ok()).as_deref()
                    != expected(original, r)
            })
            .map(|r| r.record.as_str())
            .collect::<Vec<_>>();
        let ok = status.is_some_and(|s| {
            s.valid
                && (t.minimum_bytes..=t.maximum_bytes).contains(&s.capacity_bytes)
                && t.chunk_bytes.is_none_or(|n| n == s.chunk_bytes)
        }) && missing.is_empty();
        let message = match status {
            None => format!(
                "{}: create the required view of {} with its own COW storage",
                t.name, t.origin
            ),
            Some(s) if !s.valid => format!(
                "{}: invalid; its point-in-time data is unavailable. Space added afterward cannot recover it",
                t.name
            ),
            Some(_) if !missing.is_empty() => format!(
                "{}: inspect saved records {}; this view does not contain the required point-in-time data",
                t.name,
                missing.join(", ")
            ),
            Some(s) if !ok => format!(
                "{}: check its reserved capacity and chunk size (currently {} MiB, {} KiB chunks)",
                t.name,
                s.capacity_bytes / lvm::MIB,
                s.chunk_bytes / 1024
            ),
            _ => format!(
                "{}: valid point-in-time records and reserved COW space match",
                t.name
            ),
        };
        rows.push((ok, message));
    }
    for r in &plan.retire {
        let absent = current.volume(&r.volume).is_err();
        rows.push((
            absent,
            format!(
                "{}: {}",
                r.volume,
                if absent {
                    "old snapshot store retired"
                } else {
                    "remove the old snapshot store without removing its origin"
                }
            ),
        ));
    }
    for r in &plan.reserves {
        let bytes = current.free_extents(&r.group).ok().and_then(|n| {
            current
                .groups
                .get(&r.group)
                .map(|g| u64::from(n) * g.extent_bytes)
        });
        rows.push((
            bytes.is_some_and(|n| n >= r.minimum_bytes),
            format!(
                "{}: keep at least {} MiB free for other work",
                r.group,
                r.minimum_bytes / lvm::MIB
            ),
        ));
    }
    rows.push((same_problem(initial,current,plan),"Original allocations, point-in-time views, unrelated data and unrequested storage remain protected".into()));
    rows
}
