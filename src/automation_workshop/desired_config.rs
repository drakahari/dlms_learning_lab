//! Display and grade a finite, source-bound desired configuration plan.
use super::plan_editor::Plan;
use crate::rest_controller::{self as rest, Spec, Store};
use serde::{Deserialize, Serialize};

pub const DRAFT_LIMIT: usize = 8192;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathCase {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub from: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub to: String,
    pub permitted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub topology: Spec,
    pub initial: Store,
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub starter_document: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub retained_profile: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub incidental_removal: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub excluded_slot: String,
    pub approved_slots: [String; 2],
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,PathCase,8>")]
    pub paths: Vec<PathCase>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edit {
    #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
    pub draft: String,
    pub applied: Store,
    pub previewed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub name: String,
    pub passed: bool,
    pub detail: String,
}

fn finding(name: impl Into<String>, passed: bool, detail: impl Into<String>) -> Finding {
    Finding {
        name: name.into(),
        passed,
        detail: detail.into(),
    }
}

impl Task {
    pub fn new_edit(&self) -> Edit {
        Edit {
            draft: self.starter_document.clone(),
            applied: self.initial.clone(),
            previewed: false,
        }
    }

    pub fn plan(&self, applied: &Store, draft: &str) -> Result<Plan, String> {
        if draft.len() > DRAFT_LIMIT {
            return Err("Keep the simulated configuration document within 8 KiB.".into());
        }
        let wanted: Store = serde_json::from_str(draft)
            .map_err(|error| format!("Cannot read the supplied configuration. {error}"))?;
        Plan::compare(&self.topology, applied, &wanted)
    }

    pub fn validate(&self) -> Result<(), String> {
        let plan = self.plan(&self.initial, &self.starter_document)?;
        if ![
            self.retained_profile.as_str(),
            self.incidental_removal.as_str(),
            self.excluded_slot.as_str(),
        ]
        .into_iter()
        .all(rest::identifier)
            || self.retained_profile == self.incidental_removal
            || !self.initial.profiles.contains_key(&self.retained_profile)
            || !self.initial.profiles.contains_key(&self.incidental_removal)
            || !self.initial.bindings.contains_key(&self.excluded_slot)
            || self.approved_slots[0] == self.approved_slots[1]
            || self.approved_slots.iter().any(|slot| {
                slot == &self.excluded_slot || !self.initial.bindings.contains_key(slot)
            })
            || self
                .initial
                .bindings
                .values()
                .any(|row| row.profile == self.incidental_removal)
            || plan.intact_profile(&self.retained_profile)
            || !(3..=8).contains(&self.paths.len())
            || !self.paths.iter().any(|case| case.permitted)
            || !self.paths.iter().any(|case| !case.permitted)
        {
            return Err("Supply a bounded plan with an unwanted change, an independent resource and distinct path requirements.".into());
        }
        let initial = self.topology.project(&self.initial)?;
        let mut seen = std::collections::BTreeSet::new();
        for case in &self.paths {
            if case.from == case.to
                || !initial.hosts.contains_key(&case.from)
                || !initial.hosts.contains_key(&case.to)
                || !seen.insert((&case.from, &case.to))
            {
                return Err(
                    "Use distinct source-held path cases between the supplied workplaces.".into(),
                );
            }
        }
        Ok(())
    }

    pub fn outcomes(&self, edit: &Edit) -> Result<Vec<Finding>, String> {
        let assembled = self.topology.project(&edit.applied)?;
        let mut checks = vec![
            finding(
                "Preserve the recovery profile",
                edit.applied.profiles.get(&self.retained_profile)
                    == self.initial.profiles.get(&self.retained_profile),
                "Restore the supplied profile exactly if it disappeared from the desired document.",
            ),
            finding(
                "Retire the obsolete unused profile",
                !edit.applied.profiles.contains_key(&self.incidental_removal),
                "Remove only the obsolete profile described in the work order.",
            ),
            finding(
                "Keep the unrelated port’s binding",
                edit.applied.bindings.get(&self.excluded_slot)
                    == self.initial.bindings.get(&self.excluded_slot),
                "The unrelated port needs its earlier profile assignment.",
            ),
        ];
        let comparison = Plan::compare(&self.topology, &self.initial, &edit.applied)?;
        let unrelated = self
            .initial
            .bindings
            .keys()
            .filter(|name| !self.approved_slots.contains(name) && *name != &self.excluded_slot)
            .all(|name| comparison.intact_binding(name));
        let prior_profiles = self
            .initial
            .profiles
            .keys()
            .filter(|name| *name != &self.incidental_removal && *name != &self.retained_profile)
            .all(|name| comparison.intact_profile(name));
        let new_profiles_allowed = edit
            .applied
            .profiles
            .keys()
            .filter(|name| !self.initial.profiles.contains_key(*name))
            .all(|name| {
                self.approved_slots
                    .iter()
                    .any(|slot| edit.applied.bindings[slot].profile == *name)
            });
        checks.push(finding("Limit changes to the work order",
            unrelated && prior_profiles && new_profiles_allowed,
            "Keep every unrelated binding and still-needed profile. A new profile must serve an approved port."));
        let (ready, verification) = match self.plan(&edit.applied, &edit.draft) {
            Err(error) => (
                false,
                format!("{error} Correct the draft and preview it again."),
            ),
            Ok(plan) if !plan.changes.is_empty() => (
                false,
                "Apply the approved changes and preview again.".into(),
            ),
            Ok(_) if !edit.previewed => (
                false,
                "Preview the current document again after applying the intended change.".into(),
            ),
            Ok(_) => (true, "The current draft has no unapplied changes.".into()),
        };
        checks.push(finding(
            "Verify the applied plan has no remaining changes",
            ready,
            verification,
        ));
        for case in &self.paths {
            let visibility = assembled.visibility(&case.from, &case.to)?;
            checks.push(finding(
                format!("{} and {}", case.from, case.to),
                if case.permitted {
                    visibility.mutual()
                } else {
                    visibility.isolated()
                },
                format!(
                    "This pair should {} using the applied plan.",
                    if case.permitted {
                        "exchange traffic in both directions"
                    } else {
                        "remain separate in both directions"
                    }
                ),
            ));
        }
        Ok(checks)
    }
}

impl Edit {
    pub fn change(&mut self, draft: String) -> Result<(), String> {
        if draft.len() > DRAFT_LIMIT || draft.contains('\0') {
            return Err("Keep this configuration within 8 KiB and use text only.".into());
        }
        if draft != self.draft {
            self.draft = draft;
            self.previewed = false;
        }
        Ok(())
    }

    pub fn preview(&mut self, task: &Task) -> Result<(), String> {
        task.plan(&self.applied, &self.draft)?;
        self.previewed = true;
        Ok(())
    }

    pub fn apply(&mut self, task: &Task) -> Result<(), String> {
        if !self.previewed {
            return Err("Preview this draft before applying its changes.".into());
        }
        task.plan(&self.applied, &self.draft)?
            .apply(&task.topology, &mut self.applied)?;
        self.previewed = false;
        Ok(())
    }

    pub fn validate(&self, task: &Task) -> Result<(), String> {
        if self.draft.len() > DRAFT_LIMIT || self.draft.contains('\0') {
            return Err("The saved document exceeds the supported text limits.".into());
        }
        task.topology.project(&self.applied)?;
        if self.previewed {
            task.plan(&self.applied, &self.draft)?;
        }
        Ok(())
    }
}
