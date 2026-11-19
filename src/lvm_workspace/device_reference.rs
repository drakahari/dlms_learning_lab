//! Typed application-reference outcomes for an opt-in source profile. Existing
//! LVM definitions/sessions omit these separate configurations and remain unchanged.
use super::*;
use crate::lvm::{device_path, device_reference as config};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub caller: String,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub original_volume: String,
    /// Application references should prefer /dev/VG/LV. Allowing either form is
    /// for a task explicitly assessing observed-name diagnosis, not a stable ABI.
    pub canonical_required: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    #[serde(deserialize_with = "bounded::sequence::<_,Target,4>")]
    pub targets: Vec<Target>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub renaming: Option<rename::Plan>,
}
impl Plan {
    pub fn validate(
        &self,
        initial: &lvm::State,
        configuration: &config::Configuration,
    ) -> Result<(), String> {
        model(initial)?;
        configuration.validate().map_err(|e| e.to_string())?;
        if initial.identity.is_none()||initial.snapshots.is_some()||initial.pv_capacity.is_some()
            ||self.targets.is_empty()||self.targets.len()>4||initial.groups.contains_key("mapper")
            // The retained name-context relation orders consumer paths. Keep this
            // prerequisite slice to one supplied open consumer until a separate
            // multi-consumer relation is proven; never silently approximate it.
            ||initial.activation.as_ref().is_none_or(|a|a.open.len()>1)
            ||self.renaming.as_ref().is_some_and(|p|!p.valid(initial))
        {
            return Err("Reference tasks need a bounded identity-enabled source, at most one supplied open consumer,1–4targets and optional valid renames; other profiles remain separate".into());
        }
        let mut seen = BTreeSet::new();
        for t in &self.targets {
            let Some(caller) = configuration.callers.get(&t.caller) else {
                return Err("Reference requirement names an unknown supplied application".into());
            };
            if !seen.insert(&t.caller)
                || !initial
                    .identity
                    .as_ref()
                    .unwrap()
                    .volumes
                    .contains_key(&t.original_volume)
                || initial.read_file(&t.original_volume, &caller.file).is_err()
            {
                return Err("Each application needs one distinct requirement for an original readable filesystem volume and its supplied virtual file".into());
            }
        }
        Ok(())
    }
}
/// Admission allows bad or dangling references to be repaired. It never means
/// completion, and it cannot turn config fields into a host path/execution route.
pub fn same_context(
    initial: &lvm::State,
    current: &lvm::State,
    before: &config::Configuration,
    now: &config::Configuration,
) -> bool {
    before.same_roster(now)
        && initial.identity.is_some()
        && current.identity.is_some()
        && initial.snapshots.is_none()
        && current.snapshots.is_none()
        && rename::same_problem(initial, current)
}
fn no_renames_changed_storage(initial: &lvm::State, current: &lvm::State) -> bool {
    let mut normalized = current.clone();
    if let (Some(before), Some(after)) = (&initial.identity, &mut normalized.identity) {
        after.next = before.next;
    }
    normalized == *initial
}
pub fn outcomes(
    initial: &lvm::State,
    current: &lvm::State,
    before: &config::Configuration,
    now: &config::Configuration,
    plan: &Plan,
) -> Vec<(bool, String)> {
    if plan.validate(initial, before).is_err() || !same_context(initial, current, before, now) {
        return vec![(false,"Application-reference work differs from its supplied storage/configuration context; previous work retained".into())];
    }
    let identity = initial.identity.as_ref().unwrap();
    let mut rows = vec![];
    for target in &plan.targets {
        let caller = &now.callers[&target.caller];
        let row = match now.inspect(current, &target.caller) {
            Err(e) => (
                false,
                format!(
                    "{}: saved reference cannot read its virtual data: {e}",
                    caller.label
                ),
            ),
            Ok(read) if read.identity != Some(identity.volumes[&target.original_volume]) => (
                false,
                format!(
                    "{}: the reference reads another volume; inspect the supplied application purpose and current storage names",
                    caller.label
                ),
            ),
            Ok(read) if target.canonical_required && read.form != device_path::Form::Canonical => (
                false,
                format!(
                    "{}: the data is readable, but the saved reference still uses internal mapper encoding; use the recommended /dev/VG/LV form",
                    caller.label
                ),
            ),
            Ok(read)
                if initial
                    .read_file(&target.original_volume, &caller.file)
                    .ok()
                    != Some(read.content) =>
            {
                (
                    false,
                    format!(
                        "{}: the intended original volume no longer supplies its original file data",
                        caller.label
                    ),
                )
            }
            Ok(_) => (
                true,
                format!(
                    "{}: saved reference reads the intended original data in the required form",
                    caller.label
                ),
            ),
        };
        rows.push(row);
    }
    let unrequested = before.callers.iter().all(|(key, c)| {
        plan.targets.iter().any(|t| t.caller == *key) || now.callers[key].path == c.path
    });
    rows.push((
        unrequested,
        "Keep unrelated saved application references unchanged".into(),
    ));
    if let Some(names) = &plan.renaming {
        rows.extend(names.outcomes(initial, current));
    } else {
        rows.push((no_renames_changed_storage(initial,current),"Keep all original names, devices, allocations, filesystem data and availability; no extra or recreated storage".into()));
    }
    rows
}
