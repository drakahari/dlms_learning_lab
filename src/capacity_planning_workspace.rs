//! Finite capacity planning over supplied observations and workload cases.
use crate::{resource_measurements::capacity as core, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, String>;
pub const HELP: &str = "Capacity planner: supplied occupancy snapshots, fictional upgrade quotes and finite workload cases. A forecast assumes the selected growth continues; the workload test replays the supplied cases separately. Capacity arrives after the quote's lead time. Test plan checks the evidence window, required reserve, budget and every simulated day. All planning uses simulated data. It cannot order hardware, monitor your computer, resize disks or delete files.";
fn optional_offer<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    struct Value(#[serde(deserialize_with = "crate::cron::text::<_,48>")] String);
    Ok(Option::<Value>::deserialize(d)?.map(|v| v.0))
}
fn slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub minimum_reserve_kib: u64,
    pub budget: u32,
    pub horizon_days: u32,
    pub window_not_before_day: u32,
    pub minimum_window_days: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inputs {
    pub first: Option<usize>,
    pub last: Option<usize>,
    #[serde(deserialize_with = "optional_offer")]
    pub offer: Option<String>,
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub order_day: String,
    #[serde(deserialize_with = "crate::cron::text::<_,24>")]
    pub reserve_kib: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub id: String,
    pub series: core::Series,
    pub policy: Policy,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,core::Offer,8>")]
    pub offers: Vec<core::Offer>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,core::Demand,8>")]
    pub demands: Vec<core::Demand>,
    pub starter: Inputs,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub task_id: String,
    pub inputs: Inputs,
    pub selected_case: usize,
    pub checked: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseResult {
    pub id: String,
    pub days: Vec<core::Day>,
    pub first_failure: Option<u32>,
    pub minimum_free_kib: i128,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Assessment {
    pub checks: Vec<(bool, String)>,
    pub cases: Vec<CaseResult>,
}
impl Assessment {
    pub fn passed(&self) -> bool {
        !self.checks.is_empty() && self.checks.iter().all(|r| r.0)
    }
}
fn number(text: &str, label: &str) -> Result<u64> {
    let s = text.trim();
    if s.is_empty() {
        return Err(format!("{label}: enter a value before testing the plan."));
    }
    if !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!(
            "{label}: use a whole number with the units already shown; do not include a sign or unit suffix."
        ));
    }
    s.parse()
        .map_err(|_| format!("{label}: that number exceeds the simulator's size limit."))
}
impl Inputs {
    fn shape(&self) -> Result<()> {
        if self.first.is_some_and(|i| i >= 32)
            || self.last.is_some_and(|i| i >= 32)
            || self.offer.as_ref().is_some_and(|s| !slug(s))
            || [&self.order_day, &self.reserve_kib]
                .iter()
                .any(|s| s.len() > 24 || s.chars().any(char::is_control))
        {
            return Err("Invalid saved planning fields; original work retained.".into());
        }
        Ok(())
    }
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        self.series.validate().map_err(|e| e.to_string())?;
        let anchor = self.series.samples.last().unwrap();
        let p = &self.policy;
        if !slug(&self.id)
            || !(2..=8).contains(&self.offers.len())
            || !(2..=8).contains(&self.demands.len())
            || self
                .offers
                .iter()
                .map(|o| &o.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.offers.len()
            || self
                .demands
                .iter()
                .map(|d| &d.id)
                .collect::<BTreeSet<_>>()
                .len()
                != self.demands.len()
            || p.minimum_reserve_kib == 0
            || p.minimum_reserve_kib >= self.series.capacity_kib
            || p.budget > 1_000_000_000
            || !(1..=90).contains(&p.horizon_days)
            || anchor
                .day
                .checked_add(p.horizon_days)
                .is_none_or(|d| d > core::MAX_DAY)
            || p.minimum_window_days == 0
            || !self.series.samples.iter().any(|s| {
                s.day >= p.window_not_before_day
                    && anchor.day.saturating_sub(s.day) >= p.minimum_window_days
            })
            || self.offers.iter().any(|o| {
                !slug(&o.id)
                    || o.capacity_kib < self.series.capacity_kib
                    || o.capacity_kib > core::MAX_KIB
                    || o.lead_days > 90
                    || o.cost > 1_000_000_000
            })
        {
            return Err("Supply bounded observations, distinct offers/cases and a reachable planning policy.".into());
        }
        for demand in &self.demands {
            // Replay is also the authoritative validation of supplied demand.
            core::replay(
                &self.series,
                &self.offers[0],
                &core::Plan {
                    offer: self.offers[0].id.clone(),
                    order_day: anchor.day,
                },
                demand,
                p.minimum_reserve_kib,
                p.horizon_days,
            )
            .map_err(|e| e.to_string())?;
        }
        Draft::new(self).bound(self)
    }
    pub fn window(&self, d: &Draft) -> Result<core::Window> {
        let first = d
            .inputs
            .first
            .ok_or("Growth window: choose its starting observation.")?;
        let last = d
            .inputs
            .last
            .ok_or("Growth window: choose its ending observation.")?;
        let w = core::Window { first, last };
        self.series.growth(w).map_err(|e| e.to_string())?;
        Ok(w)
    }
    pub fn window_requirement(&self, d: &Draft) -> Result<core::Window> {
        let w = self.window(d)?;
        let samples = &self.series.samples;
        let p = &self.policy;
        if w.last != samples.len() - 1 {
            return Err(format!(
                "Growth window: it ends on day {}, but the latest supplied observation is day {}. Include the latest evidence.",
                samples[w.last].day,
                samples.last().unwrap().day
            ));
        }
        if samples[w.first].day < p.window_not_before_day {
            return Err(format!(
                "Growth window: it starts before day {}. Earlier activity belongs to the old operating period; use the period requested in the task.",
                p.window_not_before_day
            ));
        }
        if samples[w.last].day - samples[w.first].day < p.minimum_window_days {
            return Err(format!(
                "Growth window: compare at least {} days of the requested operating period, rather than a shorter interval.",
                p.minimum_window_days
            ));
        }
        Ok(w)
    }
    pub fn assess(&self, d: &Draft) -> Assessment {
        if let Err(e) = self.validate().and_then(|()| d.bound(self)) {
            return Assessment {
                checks: vec![(false, e)],
                cases: vec![],
            };
        }
        let mut checks = vec![match self.window_requirement(d) {
            Ok(_) => (
                true,
                "Growth window: current observations from the requested operating period.".into(),
            ),
            Err(e) => (false, e),
        }];
        let reserve=number(&d.inputs.reserve_kib,"Reserve").and_then(|n|{
            if n<self.policy.minimum_reserve_kib {Err(format!("Reserve: {n} KiB is below the required {} KiB. A plan must leave that space unused, not merely avoid a completely full volume.",self.policy.minimum_reserve_kib))}
            else if n>core::MAX_KIB {Err("Reserve: this exceeds the simulator's disclosed size limit.".into())}
            else {Ok(n)}
        });
        checks.push(match &reserve {
            Ok(_) => (
                true,
                "Reserve: meets the required free-space margin.".into(),
            ),
            Err(e) => (false, e.clone()),
        });
        let anchor = self.series.samples.last().unwrap();
        let day=number(&d.inputs.order_day,"Order day").and_then(|n|{
            if n<u64::from(anchor.day) {Err(format!("Order day: day {n} is in the past. This plan starts from day {}; it cannot backdate an order.",anchor.day))}
            else if n>u64::from(anchor.day+self.policy.horizon_days) {Err("Order day: choose a day inside the task's planning horizon.".into())}
            else {Ok(n as u32)}
        });
        checks.push(match &day {
            Ok(_) => (
                true,
                "Order day: a possible date from the current observation onward.".into(),
            ),
            Err(e) => (false, e.clone()),
        });
        let offer = d
            .inputs
            .offer
            .as_ref()
            .and_then(|id| self.offers.iter().find(|o| &o.id == id));
        checks.push(match offer{None=>(false,"Offer: choose a supplied quote, including keeping the current capacity if it can meet the task.".into()),Some(o)if o.cost>self.policy.budget=>(false,format!("Budget: this quote costs {} credits; the limit is{}. More capacity cannot justify exceeding the stated budget.",o.cost,self.policy.budget)),Some(_)=>(true,"Budget: the chosen quote is within the supplied budget.".into())});
        let mut cases = vec![];
        if let (Ok(reserve), Ok(day), Some(offer)) = (reserve, day, offer) {
            let plan = core::Plan {
                offer: offer.id.clone(),
                order_day: day,
            };
            for demand in &self.demands {
                match core::replay(
                    &self.series,
                    offer,
                    &plan,
                    demand,
                    reserve,
                    self.policy.horizon_days,
                ) {
                    Ok(days) => {
                        let first_failure = days.iter().find(|d| !d.reserve_met).map(|d| d.day);
                        let minimum_free_kib = days
                            .iter()
                            .map(|d| i128::from(d.capacity_kib) - i128::from(d.used_kib))
                            .min()
                            .unwrap();
                        checks.push(match first_failure{None=>(true,format!("Case {}: required reserve is retained on every simulated day.",demand.id)),Some(day)=>{let row=days.iter().find(|r|r.day==day).unwrap();(false,format!("Case {}: reserve first fails on day {day}. Occupied {} KiB plus your {reserve} KiB reserve exceeds the {} KiB available then. Check both arrival time and final capacity.",demand.id,row.used_kib,row.capacity_kib))}});
                        cases.push(CaseResult {
                            id: demand.id.clone(),
                            days,
                            first_failure,
                            minimum_free_kib,
                        });
                    }
                    Err(e) => checks.push((false, format!("Case {} was not run: {e}", demand.id))),
                }
            }
        } else {
            checks.push((false,"Workload cases not run: complete the reserve, date and offer fields, then test again.".into()));
        }
        Assessment { checks, cases }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Window {
        first: usize,
        last: usize,
    },
    Offer {
        #[serde(deserialize_with = "crate::cron::text::<_,48>")]
        id: String,
    },
    OrderDay {
        #[serde(deserialize_with = "crate::cron::text::<_,24>")]
        value: String,
    },
    Reserve {
        #[serde(deserialize_with = "crate::cron::text::<_,24>")]
        value: String,
    },
    Test,
}
impl Draft {
    pub fn new(t: &Task) -> Self {
        Self {
            task_id: t.id.clone(),
            inputs: t.starter.clone(),
            selected_case: 0,
            checked: false,
        }
    }
    pub fn shape(&self) -> Result<()> {
        if !slug(&self.task_id) || self.selected_case >= 8 {
            return Err("Invalid saved plan identity or case selection.".into());
        }
        self.inputs.shape()
    }
    pub fn bound(&self, t: &Task) -> Result<()> {
        self.shape()?;
        if self.task_id != t.id
            || self.selected_case >= t.demands.len()
            || [self.inputs.first, self.inputs.last]
                .into_iter()
                .flatten()
                .any(|i| i >= t.series.samples.len())
            || self
                .inputs
                .offer
                .as_ref()
                .is_some_and(|id| !t.offers.iter().any(|o| &o.id == id))
        {
            return Err(
                "Saved planning fields belong to another task or unavailable observation/offer."
                    .into(),
            );
        }
        Ok(())
    }
    pub fn apply(&mut self, t: &Task, action: &Action) -> Result<()> {
        self.bound(t)?;
        let mut next = self.clone();
        match action {
            Action::Window { first, last } => {
                next.inputs.first = Some(*first);
                next.inputs.last = Some(*last);
            }
            Action::Offer { id } => next.inputs.offer = Some(id.clone()),
            Action::OrderDay { value } => next.inputs.order_day = value.clone(),
            Action::Reserve { value } => next.inputs.reserve_kib = value.clone(),
            Action::Test => {}
        }
        next.checked = matches!(action, Action::Test);
        next.bound(t)?;
        *self = next;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,24>")]
    pub reference: Vec<Action>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,24>")]
    pub example_reference: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id || self.initial.series == self.example.series {
            return Err(
                "Use different worked observations and independent assignment data.".into(),
            );
        }
        for (t, actions) in [
            (&self.initial, &self.reference),
            (&self.example, &self.example_reference),
        ] {
            if actions.is_empty() || actions.len() > 24 {
                return Err("Provide a bounded reachable planning reference.".into());
            }
            let mut d = Draft::new(t);
            if t.assess(&d).passed() {
                return Err("The starter must retain meaningful planning work.".into());
            }
            for a in actions {
                d.apply(t, a)?;
            }
            if !d.checked || !t.assess(&d).passed() {
                return Err(
                    "Planning reference does not meet every policy and workload case.".into(),
                );
            }
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        s.validate()?;
        s.assignment.bound(&self.initial)?;
        s.practice.bound(&self.example)
    }
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        let (t, d) = if practice {
            (&self.example, &s.practice)
        } else {
            (&self.initial, &s.assignment)
        };
        if !d.checked {
            return vec![(
                false,
                "Plan is untested. Review its window and quote, then choose Test plan.".into(),
            )];
        }
        t.assess(d).checks
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn new(d: &Definition) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            assignment: Draft::new(&d.initial),
            practice: Draft::new(&d.example),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unsupported saved planning version; original work retained.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()?;
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 8192 {
            return Err("Saved plan exceeds its size limit.".into());
        }
        Ok(())
    }
    pub fn execute_reference(&mut self, d: &Definition, text: &str) -> Result<String> {
        d.bound(self)?;
        let raw = text.strip_prefix("plan-action ").ok_or("Use the native observation, quote and plan controls. plan-action is an internal simulator reference, not a Linux command.")?;
        if raw.len() > 4096 {
            return Err("Planning reference exceeds its size limit.".into());
        }
        let action: Action =
            serde_json::from_str(raw).map_err(|_| "Invalid simulator-only planning reference.")?;
        let mut next = self.clone();
        next.assignment.apply(&d.initial, &action)?;
        d.bound(&next)?;
        *self = next;
        Ok("Applied simulated planning operation.".into())
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::CapacityPlanning { simulation },
        crate::engine::Goal::CapacityPlanning { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Capacity-planning state and objective must agree.".into());
    };
    definition.validate()?;
    if **simulation != Session::new(definition) {
        return Err("Begin with separate planning drafts from the declared starters.".into());
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::CapacityPlanning { .. })
        || matches!(s.initial, crate::engine::State::CapacityPlanning { .. })
        || matches!(s.goal, crate::engine::Goal::CapacityPlanning { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let d = source(s)?;
    let crate::engine::State::CapacityPlanning { simulation } = &a.state else {
        return Err("Saved plan belongs to another activity.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible planning metadata; original work retained.".into());
    }
    d.bound(simulation)
}
