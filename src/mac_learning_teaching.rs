//! Separate bounded native practice; no practice result can complete an assignment.
use crate::{
    mac_learning as mac, mac_learning_workspace as work,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub fixture: mac::State,
    pub example_solution: Vec<work::Action>,
    #[serde(deserialize_with = "work::requirements")]
    pub outcomes: Vec<work::Requirement>,
    pub try_change: String,
    pub try_actions: Vec<work::Action>,
    #[serde(deserialize_with = "work::requirements")]
    pub try_outcomes: Vec<work::Requirement>,
}
impl Lesson {
    fn bounded(&self) -> bool {
        self.fixture.validate().is_ok()
            && self.concepts.len() <= 6
            && !self.concepts.is_empty()
            && self.concepts.iter().all(|c| {
                !c.title.trim().is_empty()
                    && c.title.len() <= 100
                    && !c.text.trim().is_empty()
                    && c.text.len() <= 1200
            })
            && [
                &self.prerequisite_note,
                &self.example_description,
                &self.try_change,
            ]
            .iter()
            .all(|s| !s.trim().is_empty() && s.len() <= 1600)
            && [&self.example_solution, &self.try_actions]
                .iter()
                .all(|a| !a.is_empty() && a.len() <= 16 && a.iter().all(work::Action::valid))
            && work::valid_requirements(&self.outcomes, &self.fixture)
            && work::valid_requirements(&self.try_outcomes, &self.fixture)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err("Invalid or excessive MAC teaching content".into());
        }
        let mut s = self.fixture.clone();
        for a in &self.example_solution {
            a.apply(&mut s)?
        }
        if work::checks(&self.fixture, &s, &self.outcomes)
            .iter()
            .any(|c| !c.0)
        {
            return Err("MAC example does not reach its stated outcomes".into());
        }
        let before = s.clone();
        for a in &self.try_actions {
            a.apply(&mut s)?
        }
        if s == before
            || work::checks(&self.fixture, &s, &self.try_outcomes)
                .iter()
                .any(|c| !c.0)
        {
            return Err("MAC suggested change is a no-op or misses its stated outcomes".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u8,
    pub view: View,
    /// Immutable authored origin, never a grading source.
    pub fixture: mac::State,
    pub simulation: mac::State,
    pub draft: work::Draft,
    pub selected_action: usize,
    pub checked: bool,
}
impl Learning {
    pub fn new(l: &Lesson) -> Result<Self, String> {
        l.validate()?;
        Ok(Self {
            version: 1,
            view: View::Introduction,
            fixture: l.fixture.clone(),
            simulation: l.fixture.clone(),
            draft: work::Draft::new(&l.fixture)?,
            selected_action: 0,
            checked: false,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.selected_action < 16
            && work::same_problem(&self.simulation, &self.fixture)
            && self.draft.same_shape(&self.simulation)
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        self.valid()
            && l.bounded()
            && self.fixture == l.fixture
            && self.selected_action < l.example_solution.len()
    }
    pub fn fill_example(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Practice belongs to another MAC lesson".into());
        }
        l.validate()?;
        let mut d = self.draft.clone();
        d.configuration = crate::switching_workspace::Draft::new(l.fixture.fabric());
        d.aging = l.fixture.aging_seconds().to_string();
        let selected = self.draft.configuration.selected_switch.clone();
        if d.configuration.switches.contains_key(&selected) {
            d.configuration.selected_switch = selected;
        }
        for action in &l.example_solution {
            if matches!(
                action,
                work::Action::Settings { .. } | work::Action::Aging { .. }
            ) {
                action.insert(&l.fixture, &mut d)?;
            }
        }
        l.example_solution[self.selected_action].insert(&l.fixture, &mut d)?;
        self.draft = d;
        self.checked = false;
        Ok(())
    }
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(l) || !self.checked || !self.draft.matches(&self.simulation) {
            return vec![];
        }
        work::checks(&l.fixture, &self.simulation, &l.outcomes)
    }
}
pub fn prepare(a: &mut crate::progress::Attempt, l: &Lesson, new_attempt: bool) {
    if crate::workspace::has_nfs_metadata(a) {
        return;
    }
    let w = a.workspace.as_mut().expect("workspace prepared");
    if w.mac_learning.is_none() {
        let mut learning = Learning::new(l).expect("validated lesson");
        if !new_attempt {
            learning.view = View::Assignment
        }
        w.mac_learning = Some(learning)
    }
}
