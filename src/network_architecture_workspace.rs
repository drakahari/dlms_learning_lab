//! Source-held campus assignments over the bounded structural graph. No I/O.
use crate::{network_architecture as core, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Named {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub id: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Uplinks {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub node: String,
    pub minimum: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub from: String,
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub to: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,120>")]
    pub label: String,
    pub experiment: core::Experiment,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_paths: Option<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub facts: core::Facts,
    pub initial: core::Configuration,
    pub reference: core::Configuration,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Named,32>")]
    pub editable: Vec<Named>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Uplinks,16>")]
    pub uplinks: Vec<Uplinks>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Pair,8>")]
    pub pairs: Vec<Pair>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Case,8>")]
    pub cases: Vec<Case>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Check {
    pub passed: bool,
    pub detail: String,
}
fn check(passed: bool, detail: impl Into<String>) -> Check {
    Check {
        passed,
        detail: detail.into(),
    }
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}
impl Task {
    fn shape(&self) -> Result<(), String> {
        if !identifier(&self.id)
            || self.editable.is_empty()
            || self.editable.len() > 32
            || self.uplinks.is_empty()
            || self.uplinks.len() > 16
            || self.pairs.is_empty()
            || self.pairs.len() > 8
            || !(2..=8).contains(&self.cases.len())
        {
            return Err("Supply a bounded, named layout task with editable cables, uplinks, access pairs and normal/failure experiments.".into());
        }
        self.facts.validate()?;
        self.facts.validate_draft(&self.initial)?;
        self.facts.validate_draft(&self.reference)?;
        let mut editable = BTreeSet::new();
        for item in &self.editable {
            if !self.facts.wires.iter().any(|w| w.id == item.id) || !editable.insert(&item.id) {
                return Err("Editable cables must name distinct supplied wires.".into());
            }
        }
        let mut uplinks = BTreeSet::new();
        for u in &self.uplinks {
            let node = self
                .facts
                .node(&u.node)
                .ok_or("Uplink requirement names an unknown device.")?;
            let eligible = matches!(
                (self.facts.design, node.role),
                (core::Design::SpineLeaf, core::Role::Leaf)
                    | (core::Design::TwoTier, core::Role::Access)
                    | (
                        core::Design::ThreeTier,
                        core::Role::Access | core::Role::Distribution
                    )
            );
            if !eligible || !(1..=8).contains(&u.minimum) || !uplinks.insert(&u.node) {
                return Err("Uplink goals need distinct devices with an upstream role and a count from 1 to 8.".into());
            }
        }
        let mut pairs = BTreeSet::new();
        for pair in &self.pairs {
            if pair.from == pair.to
                || !pairs.insert((&pair.from, &pair.to))
                || [&pair.from, &pair.to].iter().any(|n| {
                    self.facts.node(n).is_none_or(|n| {
                        n.role
                            != if self.facts.design == core::Design::SpineLeaf {
                                core::Role::Leaf
                            } else {
                                core::Role::Access
                            }
                    })
                })
            {
                return Err("Path goals must name distinct supplied access devices.".into());
            }
        }
        let mut cases = BTreeSet::new();
        let mut normal = 0;
        for c in &self.cases {
            if !identifier(&c.id)
                || c.id.len() > 48
                || !cases.insert(&c.id)
                || c.label.trim().is_empty()
                || c.label.len() > 120
                || c.label.chars().any(char::is_control)
            {
                return Err("Use distinct experiment IDs and readable labels.".into());
            }
            if c.minimum_paths.is_some_and(|n| {
                !(1..=8).contains(&n) || self.facts.design != core::Design::SpineLeaf
            }) {
                return Err("Alternative-count goals require the canonical leaf/spine model and a count from 1 to 8.".into());
            }
            self.facts.validate_experiment(&c.experiment)?;
            normal += usize::from(c.experiment.unavailable.is_empty());
        }
        if normal != 1 {
            return Err(
                "Include exactly one normal layout and at least one stated failure experiment."
                    .into(),
            );
        }
        self.owned(&self.reference)?;
        Ok(())
    }
    fn owned(&self, c: &core::Configuration) -> Result<(), String> {
        self.facts.validate_draft(c)?;
        for initial in &self.initial.connections {
            if !self.editable.iter().any(|w| w.id == initial.wire)
                && c.connections.iter().find(|w| w.wire == initial.wire) != Some(initial)
            {
                return Err(format!("Keep protected cable {} unchanged.", initial.wire));
            }
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if self.evaluate(&self.initial).iter().all(|c| c.passed) {
            return Err(
                "The initial assignment already passes; supply meaningful repair work.".into(),
            );
        }
        if !self.evaluate(&self.reference).iter().all(|c| c.passed) {
            return Err(
                "The authored reference cannot reach every layout/uplink/failure goal.".into(),
            );
        }
        Ok(())
    }
    fn evaluate(&self, c: &core::Configuration) -> Vec<Check> {
        if let Err(e) = self.owned(c) {
            return vec![check(false, e)];
        }
        let pair = &self.pairs[0];
        let baseline =
            match self
                .facts
                .inspect(c, &core::Experiment::default(), &pair.from, &pair.to)
            {
                Ok(o) => o,
                Err(e) => return vec![check(false, e)],
            };
        let mut result = vec![check(
            baseline.layout_issues.is_empty(),
            if baseline.layout_issues.is_empty() {
                "All supplied cables are complete and follow the tier/building roles.".into()
            } else {
                baseline.layout_issues.join(" ")
            },
        )];
        for u in &self.uplinks {
            match self.facts.upstream(c, &u.node) {
                Ok(parents) => {
                    let passed = parents.len() >= u.minimum as usize;
                    let name = &self.facts.node(&u.node).unwrap().label;
                    let mut detail = format!(
                        "{name}: {} distinct upstream devices; at least {} required.",
                        parents.len(),
                        u.minimum
                    );
                    if !passed {
                        detail.push_str(
                            " Two cables to one device still share that device's failure.",
                        );
                    }
                    result.push(check(passed, detail));
                }
                Err(e) => result.push(check(false, e)),
            }
        }
        for case in &self.cases {
            for pair in &self.pairs {
                match self
                    .facts
                    .inspect(c, &case.experiment, &pair.from, &pair.to)
                {
                    Ok(o) => {
                        let count = o.spine_alternatives.as_ref().map(Vec::len);
                        let passed = o.hierarchy.is_some()
                            && case
                                .minimum_paths
                                .is_none_or(|n| count.is_some_and(|count| count >= n as usize));
                        let mut detail = if let Some(path) = o.hierarchy {
                            format!(
                                "Hierarchy path {} ({} cables).",
                                path.nodes
                                    .iter()
                                    .map(|id| self.facts.node(id).unwrap().label.as_str())
                                    .collect::<Vec<_>>()
                                    .join(" > "),
                                path.wires.len()
                            )
                        } else if o.physical.is_some() {
                            "A physical path exists, but it does not follow the required tier/building roles or relies on another access device as transit.".into()
                        } else {
                            "No structural path remains. Check the cable endpoints and the devices/cables unavailable in this experiment.".into()
                        };
                        if let Some(minimum) = case.minimum_paths {
                            detail.push_str(&format!(" {} distinct internal spine alternatives; at least {minimum} required. Parallel cables to one spine do not create another independent spine.",count.unwrap_or(0)));
                        }
                        result.push(check(
                            passed,
                            format!(
                                "{} · {} to {}: {detail}",
                                case.label,
                                self.facts.node(&pair.from).unwrap().label,
                                self.facts.node(&pair.to).unwrap().label
                            ),
                        ));
                    }
                    Err(e) => result.push(check(false, format!("{}: {e}", case.label))),
                }
            }
        }
        result
    }
    pub fn checks(&self, c: &core::Configuration) -> Vec<Check> {
        if let Err(e) = self.validate() {
            return vec![check(false, e)];
        }
        self.evaluate(c)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub draft: core::Configuration,
    pub checked: bool,
}
impl Attempt {
    pub fn edit(&mut self, t: &Task, draft: core::Configuration) -> Result<(), String> {
        t.owned(&draft)?;
        if self.draft != draft {
            self.draft = draft;
            self.checked = false;
        }
        Ok(())
    }
    pub fn check(&mut self, t: &Task) -> Result<Vec<Check>, String> {
        t.validate()?;
        t.owned(&self.draft)?;
        self.checked = true;
        Ok(t.evaluate(&self.draft))
    }
    pub fn solved(&self, t: &Task) -> bool {
        self.checked && t.checks(&self.draft).iter().all(|c| c.passed)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub schema_version: u32,
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub assignment: Task,
    pub example: Task,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub schema_version: u32,
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub view: View,
    pub assignment: Attempt,
    pub practice: Attempt,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.version == 0
            || !identifier(&self.id)
            || self.assignment.id == self.example.id
            || self.assignment.facts == self.example.facts
        {
            return Err(
                "Use version1, a bounded identity and distinct assignment/worked-example contexts."
                    .into(),
            );
        }
        self.assignment.validate()?;
        self.example.validate()?;
        Ok(())
    }
    pub fn new_session(&self) -> Result<Session, String> {
        self.validate()?;
        Ok(Session {
            schema_version: 1,
            version: self.version,
            id: self.id.clone(),
            view: View::Introduction,
            assignment: Attempt {
                draft: self.assignment.initial.clone(),
                checked: false,
            },
            practice: Attempt {
                draft: self.example.initial.clone(),
                checked: false,
            },
        })
    }
    pub fn context(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        s.validate()?;
        if s.schema_version != 1 || s.version != self.version || s.id != self.id {
            return Err("This saved layout belongs to another task/version.".into());
        }
        self.assignment.owned(&s.assignment.draft)?;
        self.example.owned(&s.practice.draft)?;
        Ok(())
    }
}
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.version == 0
            || !identifier(&self.id)
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 32 * 1024
        {
            return Err(
                "Unknown campus-workspace version/identity or saved work exceeds its bound.".into(),
            );
        }
        for a in [&self.assignment, &self.practice] {
            if a.draft.connections.len() > core::MAX_WIRES
                || a.draft.connections.iter().any(|c| {
                    c.wire.len() > 48
                        || !identifier(&c.wire)
                        || [&c.a, &c.b].into_iter().flatten().any(|j| {
                            j.node.len() > 48
                                || j.port.len() > 48
                                || !identifier(&j.node)
                                || !identifier(&j.port)
                        })
                })
            {
                return Err("Saved cable inventory exceeds its identity/size bounds.".into());
            }
        }
        Ok(())
    }
    pub fn checks(&self, d: &Definition, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = d.context(self) {
            return vec![(false, e)];
        }
        let a = self.attempt(practice);
        let t = if practice { &d.example } else { &d.assignment };
        if !a.checked {
            return vec![(
                false,
                "Layout not checked for these cable endpoints yet.".into(),
            )];
        }
        t.checks(&a.draft)
            .into_iter()
            .map(|r| (r.passed, r.detail))
            .collect()
    }
    pub fn attempt(&self, practice: bool) -> &Attempt {
        if practice {
            &self.practice
        } else {
            &self.assignment
        }
    }
    pub fn attempt_mut(&mut self, practice: bool) -> &mut Attempt {
        if practice {
            &mut self.practice
        } else {
            &mut self.assignment
        }
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.context(self)?;
        let t = if practice { &d.example } else { &d.assignment };
        *self.attempt_mut(practice) = Attempt {
            draft: t.initial.clone(),
            checked: false,
        };
        Ok(())
    }
}
