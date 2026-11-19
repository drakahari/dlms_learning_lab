//! Typed current-state MBR outcomes; native and platform acceptance are separate.
use super::core::mbr::Kind;
use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    /// Task purpose, never an MBR on-disk partition name.
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub purpose: String,
    #[serde(deserialize_with = "disk_paths")]
    pub eligible_disks: Vec<String>,
    pub role: Kind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supplied_slot: Option<u8>,
    pub minimum_bytes: u64,
    pub maximum_bytes: u64,
    pub aligned_start: bool,
    pub content: DesiredContent,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "bounded::sequence::<_,Target,MAX_TARGETS>")]
    pub targets: Vec<Target>,
}
impl Plan {
    pub fn validate(&self, initial: &core::State) -> Result<(), String> {
        model(initial)?;
        if initial.mbr_profile.is_none()
            || self.targets.is_empty()
            || self.targets.len() > MAX_TARGETS
        {
            return Err(
                "MBR tasks need the explicit source profile and 1–8 payload requirements".into(),
            );
        }
        let mut purposes = BTreeSet::new();
        for t in &self.targets {
            if let Some(slot) = t.supplied_slot {
                if t.eligible_disks.len() != 1 {
                    return Err("A supplied slot needs one exact supplied disk".into());
                }
                let entry = initial
                    .disks
                    .get(&t.eligible_disks[0])
                    .and_then(|d| d.mbr.as_ref())
                    .and_then(|t| t.entries.iter().find(|e| e.partition.number == slot));
                let Some(e) = entry else {
                    return Err("Required supplied MBR slot does not exist".into());
                };
                let d = &initial.disks[&t.eligible_disks[0]];
                let expected_hint = if matches!(t.content, DesiredContent::Swap { .. }) {
                    core::TypeHint::LinuxSwap
                } else {
                    core::TypeHint::LinuxData
                };
                if e.kind != t.role
                    || e.partition.type_hint != expected_hint
                    || (e.partition.content == core::Content::Blank && (!d.writable || d.in_use))
                    || e.partition.bytes().is_err()
                    || e.partition
                        .bytes()
                        .is_ok_and(|n| !(t.minimum_bytes..=t.maximum_bytes).contains(&n))
                    || (t.aligned_start
                        && !e
                            .partition
                            .start
                            .is_multiple_of(initial.disks[&t.eligible_disks[0]].optimal_alignment))
                    || (e.partition.content != core::Content::Blank
                        && !content_matches(&e.partition.content, &t.content))
                {
                    return Err("The supplied MBR payload would need unsupported geometry or content replacement".into());
                }
            }
            let (floor, label) = match &t.content {
                DesiredContent::Ext4 { label } => (1024 * 1024, label),
                DesiredContent::Swap { label } => (10 * core::BLOCK_BYTES, label),
                DesiredContent::Blank => {
                    return Err("This MBR task profile requires initialized payloads".into());
                }
            };
            if !name_ok(&t.purpose, 36)
                || !purposes.insert(&t.purpose)
                || t.role == Kind::Extended
                || t.eligible_disks.is_empty()
                || t.eligible_disks.len() > 4
                || t.eligible_disks.iter().collect::<BTreeSet<_>>().len() != t.eligible_disks.len()
                || t.eligible_disks
                    .iter()
                    .any(|p| !disk_ok(p) || !initial.disks.contains_key(p))
                || t.minimum_bytes < floor
                || t.minimum_bytes > t.maximum_bytes
                || t.maximum_bytes > (1_u64 << 32) * core::SECTOR_BYTES
                || !t.minimum_bytes.is_multiple_of(core::SECTOR_BYTES)
                || !t.maximum_bytes.is_multiple_of(core::SECTOR_BYTES)
                || label.as_ref().is_none_or(|l| {
                    l.is_empty()
                        || l.len() > 16
                        || !l
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
                })
            {
                return Err("Invalid MBR payload role, purpose, disk, capacity, label or duplicate requirement".into());
            }
        }
        Ok(())
    }
}

fn capacity_text(bytes: u64) -> String {
    const MIB: u64 = 1024 * 1024;
    if bytes.is_multiple_of(MIB) {
        format!("{} MiB", bytes / MIB)
    } else {
        format!("{bytes} bytes")
    }
}

pub(super) fn same_problem(initial: &core::State, current: &core::State) -> bool {
    for (id, before) in &initial.disks {
        let after = &current.disks[id];
        if let Some(table) = &before.mbr {
            let Some(now) = &after.mbr else {
                return false;
            };
            for e in &table.entries {
                let Some(q) = now
                    .entries
                    .iter()
                    .find(|q| q.partition.number == e.partition.number)
                else {
                    return false;
                };
                let (p, a) = (&e.partition, &q.partition);
                if e.kind != q.kind
                    || e.ebr != q.ebr
                    || p.number != a.number
                    || p.name != a.name
                    || p.start != a.start
                    || p.end != a.end
                    || p.type_hint != a.type_hint
                    || (p.content != core::Content::Blank && p.content != a.content)
                {
                    return false;
                }
                if p.content == core::Content::Blank && invented_content(&a.content) {
                    return false;
                }
            }
        }
        if let Some(table) = &after.mbr {
            for e in &table.entries {
                let old = before.mbr.as_ref().is_some_and(|t| {
                    t.entries
                        .iter()
                        .any(|q| q.partition.number == e.partition.number)
                });
                if !old && invented_content(&e.partition.content) {
                    return false;
                }
            }
        }
    }
    true
}
fn invented_content(c: &core::Content) -> bool {
    matches!(c, core::Content::Foreign { .. })
        || matches!(c, core::Content::Ext4 { files, .. } if !files.is_empty())
}
fn structure_fits(t: &Target, id: &str, entry: &core::mbr::Entry, d: &core::Disk) -> bool {
    let p = &entry.partition;
    entry.kind == t.role
        && t.supplied_slot.is_none_or(|slot| p.number == slot)
        && t.eligible_disks.iter().any(|p| p == id)
        && p.bytes()
            .is_ok_and(|n| (t.minimum_bytes..=t.maximum_bytes).contains(&n))
        && (!t.aligned_start || p.start.is_multiple_of(d.optimal_alignment))
}
fn fits(t: &Target, id: &str, entry: &core::mbr::Entry, d: &core::Disk) -> bool {
    structure_fits(t, id, entry, d)
        && content_matches(&entry.partition.content, &t.content)
        && entry.partition.type_hint
            == match t.content {
                DesiredContent::Swap { .. } => core::TypeHint::LinuxSwap,
                _ => core::TypeHint::LinuxData,
            }
}
fn augment(
    t: usize,
    edges: &[Vec<usize>],
    visited: &mut [bool],
    owners: &mut [Option<usize>],
) -> bool {
    for &p in &edges[t] {
        if visited[p] {
            continue;
        }
        visited[p] = true;
        if owners[p].is_none() || augment(owners[p].unwrap(), edges, visited, owners) {
            owners[p] = Some(t);
            return true;
        }
    }
    false
}
pub fn role_name(k: Kind) -> &'static str {
    match k {
        Kind::Primary => "primary",
        Kind::Logical => "logical",
        Kind::Extended => "extended container",
    }
}
/// Polynomial bounded matching: a payload can satisfy only one requirement.
/// Transcript strings, historical checked flags and exact sector answers play no part.
pub fn outcomes(initial: &core::State, current: &core::State, plan: &Plan) -> Vec<(bool, String)> {
    if plan.validate(initial).is_err() || !super::same_problem(initial, current) {
        return vec![(
            false,
            "The MBR source, supplied geometry or original data differs; no success accepted"
                .into(),
        )];
    }
    let parts = current
        .disks
        .iter()
        .flat_map(|(id, d)| {
            d.mbr.iter().flat_map(move |table| {
                table
                    .entries
                    .iter()
                    .filter(|e| e.kind != Kind::Extended)
                    .map(move |e| (id.as_str(), d, e))
            })
        })
        .collect::<Vec<_>>();
    let edges = plan
        .targets
        .iter()
        .map(|t| {
            parts
                .iter()
                .enumerate()
                .filter_map(|(i, (id, d, e))| fits(t, id, e, d).then_some(i))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut owners = vec![None; parts.len()];
    for t in 0..plan.targets.len() {
        augment(t, &edges, &mut vec![false; parts.len()], &mut owners);
    }
    let mut assigned = vec![None; plan.targets.len()];
    for (p, t) in owners.iter().enumerate() {
        if let Some(t) = t {
            assigned[*t] = Some(p);
        }
    }
    // Recognize requested allocations before formatting, without awarding their
    // content requirements. Completed objects keep singleton edges; unfinished
    // work cannot disguise an extra allocation after all goals pass.
    let fully_matched = assigned.iter().all(Option::is_some);
    let structural_edges = plan
        .targets
        .iter()
        .enumerate()
        .map(|(t, goal)| {
            assigned[t].map_or_else(
                || {
                    parts
                        .iter()
                        .enumerate()
                        .filter_map(|(i, (id, d, e))| structure_fits(goal, id, e, d).then_some(i))
                        .collect::<Vec<_>>()
                },
                |i| vec![i],
            )
        })
        .collect::<Vec<_>>();
    let mut potential = vec![None; parts.len()];
    for t in 0..plan.targets.len() {
        augment(
            t,
            &structural_edges,
            &mut vec![false; parts.len()],
            &mut potential,
        );
    }
    let matched = potential
        .iter()
        .enumerate()
        .filter_map(|(i, owner)| owner.map(|_| (parts[i].0, parts[i].2.partition.number)))
        .collect::<BTreeSet<_>>();
    let used = if fully_matched {
        matched.iter().map(|(id, _)| *id).collect::<BTreeSet<_>>()
    } else {
        plan.targets
            .iter()
            .flat_map(|t| t.eligible_disks.iter().map(String::as_str))
            .collect::<BTreeSet<_>>()
    };
    let mut rows = Vec::new();
    for (t, found) in plan.targets.iter().zip(&assigned) {
        let purpose = t.supplied_slot.map_or_else(
            || t.purpose.clone(),
            |slot| format!("{} · supplied {}{slot}", t.purpose, t.eligible_disks[0]),
        );
        if let Some(i) = found {
            let (id, _, e) = parts[*i];
            rows.push((
                true,
                format!(
                    "{}: {id}{} has the required {} role, capacity, alignment and {}",
                    purpose,
                    e.partition.number,
                    role_name(t.role),
                    required_content(&t.content)
                ),
            ));
        } else {
            let closest = parts
                .iter()
                .filter(|(id, _, _)| t.eligible_disks.iter().any(|d| d == id))
                .max_by_key(|(_, d, e)| {
                    (
                        content_matches(&e.partition.content, &t.content),
                        e.kind == t.role,
                        e.partition
                            .bytes()
                            .is_ok_and(|n| (t.minimum_bytes..=t.maximum_bytes).contains(&n)),
                        e.partition.start.is_multiple_of(d.optimal_alignment),
                    )
                });
            let observed = closest.map_or_else(|| "No matching payload is present; a container or GPT entry is not this MBR payload".into(), |(id, _, e)| format!("Observed {id}{}: {}, {}, {} hint, {}",e.partition.number,role_name(e.kind),capacity_text(e.partition.bytes().unwrap()),type_hint_name(e.partition.type_hint),actual_content(&e.partition.content)));
            rows.push((
                false,
                format!(
                    "{}: need a distinct {} payload of {}–{}{} with {}. {observed}.",
                    purpose,
                    role_name(t.role),
                    capacity_text(t.minimum_bytes),
                    capacity_text(t.maximum_bytes),
                    if t.aligned_start {
                        ", an aligned start"
                    } else {
                        ""
                    },
                    required_content(&t.content)
                ),
            ));
        }
    }
    let mut protection = Vec::new();
    for (id, before) in &initial.disks {
        let after = &current.disks[id];
        if !used.contains(id.as_str()) {
            if before != after {
                protection.push(format!("Unrequested disk {id} changed"));
            }
            continue;
        }
        if before.table != after.table {
            protection.push(format!("Unrequested GPT allocations on {id} changed"));
        }
        if let Some(table) = &after.mbr {
            for e in &table.entries {
                let old = before.mbr.as_ref().and_then(|t| {
                    t.entries
                        .iter()
                        .find(|q| q.partition.number == e.partition.number)
                });
                if matched.contains(&(id.as_str(), e.partition.number)) {
                    continue;
                }
                if let Some(old) = old {
                    if old != e {
                        protection.push(format!("Supplied {id}{} changed", e.partition.number));
                    }
                } else if e.kind != Kind::Extended
                    || !plan.targets.iter().any(|t| {
                        t.role == Kind::Logical && t.eligible_disks.iter().any(|p| p == id)
                    })
                    || (fully_matched
                        && !table.entries.iter().any(|q| {
                            q.kind == Kind::Logical
                                && matched.contains(&(id.as_str(), q.partition.number))
                        }))
                {
                    protection.push(format!("Unrequested {id}{} allocation", e.partition.number));
                }
            }
        }
    }
    rows.push((protection.is_empty(),if protection.is_empty() {"Supplied original data, unrelated media and unrequested allocations remain unchanged".into()}else{format!("{}. This preparation-only model cannot delete or reformat allocations; reset and replay the intended layout.",protection.join("; "))}));
    rows
}
