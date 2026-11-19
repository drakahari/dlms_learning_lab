//! Bounded desired-state plans for supplied port profiles and bindings.
//! No Terraform providers, Ansible modules, remote apply, or learner host input.
use crate::rest_controller::{Spec, Store};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difference {
    Create,
    Update,
    Remove,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeRow {
    pub resource: String,
    pub difference: Difference,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    baseline: Store,
    desired: Store,
    spec: Spec,
    pub changes: Vec<ChangeRow>,
}

fn plan_index<T: PartialEq>(
    before: &std::collections::BTreeMap<String, T>,
    after: &std::collections::BTreeMap<String, T>,
    folder: &str,
    rows: &mut Vec<ChangeRow>,
) {
    for (name, previous) in before {
        let difference = match after.get(name) {
            None => Difference::Remove,
            Some(current) if current != previous => Difference::Update,
            _ => continue,
        };
        rows.push(ChangeRow {
            resource: format!("/{folder}/{name}"),
            difference,
        });
    }
    for name in after.keys().filter(|name| !before.contains_key(*name)) {
        rows.push(ChangeRow {
            resource: format!("/{folder}/{name}"),
            difference: Difference::Create,
        });
    }
}

impl Plan {
    pub fn compare(spec: &Spec, actual: &Store, desired: &Store) -> Result<Self, String> {
        spec.project(actual)?;
        spec.project(desired)?;
        let mut changes = Vec::new();
        plan_index(
            &actual.profiles,
            &desired.profiles,
            "profiles",
            &mut changes,
        );
        plan_index(
            &actual.bindings,
            &desired.bindings,
            "bindings",
            &mut changes,
        );
        changes.sort_by(|a, b| a.resource.cmp(&b.resource));
        Ok(Self {
            spec: spec.clone(),
            baseline: actual.clone(),
            desired: desired.clone(),
            changes,
        })
    }

    pub fn apply(self, spec: &Spec, actual: &mut Store) -> Result<(), String> {
        if self.spec != *spec || self.baseline != *actual {
            return Err(
                "Current virtual settings changed after preview. Rebuild the plan before applying."
                    .into(),
            );
        }
        *actual = self.desired;
        Ok(())
    }

    pub fn intact_profile(&self, name: &str) -> bool {
        self.baseline.profiles.get(name) == self.desired.profiles.get(name)
    }

    pub fn intact_binding(&self, name: &str) -> bool {
        self.baseline.bindings.get(name) == self.desired.bindings.get(name)
    }
}
