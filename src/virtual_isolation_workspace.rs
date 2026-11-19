//! Source-bound represented workload and VRF assignments. Pure in-process state.
use crate::{script_teaching::View, virtual_routing as vrf, virtual_workloads as hosts};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Name {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub id: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Pair {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub from: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub to: String,
    pub reachable: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Exercise {
    Workloads {
        facts: hosts::Facts,
        initial: hosts::Configuration,
        reference: hosts::Configuration,
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Name,4>")]
        services: Vec<Name>,
    },
    Routing {
        facts: vrf::Facts,
        initial: vrf::Configuration,
        reference: vrf::Configuration,
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Pair,16>")]
        pairs: Vec<Pair>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "configuration", deny_unknown_fields)]
pub enum Draft {
    Workloads(hosts::Configuration),
    Routing(vrf::Configuration),
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub exercise: Exercise,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Check {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}
fn item(label: impl Into<String>, passed: bool, detail: impl Into<String>) -> Check {
    Check {
        label: label.into(),
        passed,
        detail: detail.into(),
    }
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes().next().is_some_and(|b| b.is_ascii_lowercase())
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl Task {
    pub fn initial(&self) -> Draft {
        match &self.exercise {
            Exercise::Workloads { initial, .. } => Draft::Workloads(initial.clone()),
            Exercise::Routing { initial, .. } => Draft::Routing(initial.clone()),
        }
    }
    pub fn reference(&self) -> Draft {
        match &self.exercise {
            Exercise::Workloads { reference, .. } => Draft::Workloads(reference.clone()),
            Exercise::Routing { reference, .. } => Draft::Routing(reference.clone()),
        }
    }
    pub fn configuration(&self, d: &Draft) -> Result<(), String> {
        match (&self.exercise, d) {
            (Exercise::Workloads { facts, .. }, Draft::Workloads(c)) => {
                facts.validate_configuration(c)
            }
            (Exercise::Routing { facts, .. }, Draft::Routing(c)) => facts.validate_configuration(c),
            _ => Err("This draft belongs to a different represented-network workspace.".into()),
        }
    }
    fn shape(&self) -> Result<(), String> {
        if !identifier(&self.id) {
            return Err("Supply a bounded stable task identifier.".into());
        }
        self.configuration(&self.initial())?;
        self.configuration(&self.reference())?;
        match &self.exercise {
            Exercise::Workloads {
                facts, services, ..
            } => {
                if services.is_empty() || services.len() > 4 {
                    return Err("Supply one to four named workload-service groups.".into());
                }
                let mut seen = BTreeSet::new();
                for s in services {
                    if !identifier(&s.id)
                        || !seen.insert(&s.id)
                        || !facts.workloads.iter().any(|w| w.service == s.id)
                    {
                        return Err(
                            "Service requirements must name distinct supplied workload groups."
                                .into(),
                        );
                    }
                }
                if facts.workloads.iter().any(|w| !seen.contains(&w.service)) {
                    return Err(
                        "Every supplied workload group needs an outcome requirement.".into(),
                    );
                }
            }
            Exercise::Routing { facts, pairs, .. } => {
                if !(4..=16).contains(&pairs.len())
                    || !pairs.iter().any(|p| p.reachable)
                    || !pairs.iter().any(|p| !p.reachable)
                {
                    return Err(
                        "Supply meaningful intended endpoint paths and separation cases.".into(),
                    );
                }
                let mut seen = BTreeSet::new();
                for p in pairs {
                    if p.from == p.to
                        || p.from == facts.router
                        || p.to == facts.router
                        || !facts.network.devices.contains_key(&p.from)
                        || !facts.network.devices.contains_key(&p.to)
                        || !seen.insert((&p.from, &p.to))
                    {
                        return Err("Path cases must name distinct supplied endpoint pairs, not internal routing instances.".into());
                    }
                }
                let covered: BTreeSet<_> = pairs
                    .iter()
                    .filter(|p| p.reachable)
                    .flat_map(|p| [&p.from, &p.to])
                    .collect();
                if covered
                    != facts
                        .network
                        .devices
                        .keys()
                        .filter(|id| *id != &facts.router)
                        .collect()
                {
                    return Err("Every supplied endpoint needs an intended working-path case, not only rejection cases.".into());
                }
            }
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shape()?;
        if self.checks(&self.reference()).iter().any(|c| !c.passed) {
            return Err("The source reference must reach every modeled outcome.".into());
        }
        if self.checks(&self.initial()).iter().all(|c| c.passed) {
            return Err("The independent starter must need a meaningful repair.".into());
        }
        Ok(())
    }
    pub fn checks(&self, d: &Draft) -> Vec<Check> {
        if let Err(e) = self.shape().and_then(|_| self.configuration(d)) {
            return vec![item("Configuration", false, e)];
        }
        match (&self.exercise, d) {
            (
                Exercise::Workloads {
                    facts, services, ..
                },
                Draft::Workloads(c),
            ) => {
                let observations = match facts.observe(c, &hosts::Experiment::Normal) {
                    Ok(v) => v,
                    Err(e) => return vec![item("Normal operation", false, e)],
                };
                let mut checks: Vec<_> = observations
                    .into_iter()
                    .map(|o| item(format!("{} on {}", o.workload, o.host), o.ready, o.detail))
                    .collect();
                for host in &facts.hosts {
                    for service in services {
                        let e = hosts::Experiment::HostOutage {
                            host: host.id.clone(),
                        };
                        let ready = facts.service_ready(c, &e, &service.id).unwrap_or(false);
                        checks.push(item(format!("{} during {} outage",service.id,host.id),ready,if ready{"At least one supplied replica remains ready in this plan."}else{"No replica remains ready. Inspect its host placement, kernel/runtime compatibility and stated planning budget."}));
                    }
                }
                checks
            }
            (Exercise::Routing { facts, pairs, .. }, Draft::Routing(c)) => {
                if let Err(e) = facts.project(c) {
                    return vec![item("Routing contexts", false, e)];
                }
                pairs.iter().map(|p|{
     let label=format!("{} to {}",p.from,p.to);
     match facts.probe(c,&p.from,&p.to){
      Ok(result)=>{let passed=result.reached_requested==p.reachable;let detail=if result.reached_requested{format!("{} reached intended endpoint {} at {}. {}",p.from,p.to,result.address,if p.reachable{"Required path works."}else{"This endpoint must remain in the other routing context; inspect interface membership."})}else{format!("Address {} answered at {}, not intended endpoint {}. {}",result.address,result.actual,p.to,if p.reachable{"Check which interfaces share a routing table."}else{"A same-numbered reply is not access to the other tenant's endpoint."})};item(label,passed,detail)},
      Err(e)=>item(label,!p.reachable,format!("Intended endpoint {} was not reached: {e}",p.to)),
     }
    }).collect()
            }
            _ => unreachable!(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Selection {
    Workloads {
        experiment: hosts::Experiment,
    },
    Routing {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        from: String,
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        to: String,
    },
}
impl Selection {
    fn validate(&self, t: &Task) -> Result<(), String> {
        match (self, &t.exercise) {
            (Self::Workloads { experiment }, Exercise::Workloads { facts, .. }) => match experiment
            {
                hosts::Experiment::Normal => Ok(()),
                hosts::Experiment::HostOutage { host } => {
                    if facts.hosts.iter().any(|h| h.id == *host) {
                        Ok(())
                    } else {
                        Err("Choose a supplied outage host.".into())
                    }
                }
            },
            (Self::Routing { from, to }, Exercise::Routing { facts, .. }) => {
                if from != &facts.router
                    && to != &facts.router
                    && facts.network.devices.contains_key(from)
                    && facts.network.devices.contains_key(to)
                {
                    Ok(())
                } else {
                    Err("Choose two different supplied endpoints.".into())
                }
            }
            _ => Err("Choose a trial appropriate to this workspace.".into()),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub configuration: Draft,
    pub selection: Selection,
    pub checked: bool,
    #[serde(skip)]
    pub actual: Option<Vec<String>>,
}
impl Attempt {
    pub fn new(t: &Task) -> Result<Self, String> {
        t.validate()?;
        Ok(Self {
            configuration: t.initial(),
            selection: match &t.exercise {
                Exercise::Workloads { .. } => Selection::Workloads {
                    experiment: hosts::Experiment::Normal,
                },
                Exercise::Routing { pairs, .. } => Selection::Routing {
                    from: pairs[0].from.clone(),
                    to: pairs[0].to.clone(),
                },
            },
            checked: false,
            actual: None,
        })
    }
    pub fn context(&self, t: &Task) -> Result<(), String> {
        t.configuration(&self.configuration)?;
        self.selection.validate(t)?;
        if self
            .actual
            .as_ref()
            .is_some_and(|v| v.len() > 64 || v.iter().map(String::len).sum::<usize>() > 32768)
        {
            return Err("Trial observations exceed their display bound.".into());
        }
        Ok(())
    }
    pub fn edit(&mut self, t: &Task, c: Draft) -> Result<(), String> {
        t.configuration(&c)?;
        if c != self.configuration {
            self.configuration = c;
            self.checked = false;
            self.actual = None;
        }
        Ok(())
    }
    pub fn choose(&mut self, t: &Task, s: Selection) -> Result<(), String> {
        s.validate(t)?;
        if self.selection != s {
            self.selection = s;
            self.actual = None;
        }
        Ok(())
    }
    pub fn run(&mut self, t: &Task) -> Result<(), String> {
        self.actual = None;
        self.context(t)?;
        let result = match (&t.exercise, &self.configuration, &self.selection) {
            (
                Exercise::Workloads { facts, .. },
                Draft::Workloads(c),
                Selection::Workloads { experiment },
            ) => facts.observe(c, experiment).map(|rows| {
                rows.into_iter()
                    .map(|r| {
                        format!(
                            "{} on {} — {}: {}",
                            r.workload,
                            r.host,
                            if r.ready {
                                "Ready in this plan"
                            } else {
                                "Unavailable in this plan"
                            },
                            r.detail
                        )
                    })
                    .collect()
            }),
            (
                Exercise::Routing { facts, .. },
                Draft::Routing(c),
                Selection::Routing { from, to },
            ) => facts.probe(c, from, to).map(|p| {
                let mut lines = vec![format!(
                    "Requested {to} at {}. Actual destination: {}. {}",
                    p.address,
                    p.actual,
                    if p.reached_requested {
                        "Reached the requested endpoint."
                    } else {
                        "This is a different endpoint with the same address."
                    }
                )];
                for h in p.probe.forward.hops {
                    let render = |id: &str| {
                        p.routing_instances
                            .get(id)
                            .map(|(router, context)| format!("{router} [{context}]"))
                            .unwrap_or_else(|| id.into())
                    };
                    lines.push(format!(
                        "{} / {} -> {} / {}",
                        render(&h.device),
                        h.interface,
                        render(&h.peer),
                        h.peer_interface
                    ));
                }
                lines
            }),
            _ => Err("Trial does not match this workspace.".into()),
        };
        match result {
            Ok(lines) => {
                self.actual = Some(lines);
                Ok(())
            }
            Err(e) => {
                self.actual = Some(vec![format!("Not reached / not completed: {e}")]);
                Err(e)
            }
        }
    }
    pub fn check(&mut self, t: &Task) -> Result<(), String> {
        self.context(t)?;
        self.checked = true;
        Ok(())
    }
    pub fn solved(&self, t: &Task) -> bool {
        self.checked
            && self.context(t).is_ok()
            && t.checks(&self.configuration).iter().all(|c| c.passed)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub version: u32,
    pub assignment: Task,
    pub example: Task,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub schema_version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub version: u32,
    pub view: View,
    pub assignment: Attempt,
    pub practice: Attempt,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if !identifier(&self.id) || self.version == 0 || self.assignment.id == self.example.id {
            return Err(
                "Supply a stable versioned lesson with separate practice and assignment tasks."
                    .into(),
            );
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if std::mem::discriminant(&self.assignment.exercise)
            != std::mem::discriminant(&self.example.exercise)
        {
            return Err("Practice must teach the assignment's workspace type.".into());
        }
        if crate::cron_workspace::size(self, 262144).is_none() {
            return Err("Represented-network source exceeds256KiB.".into());
        }
        Ok(())
    }
    pub fn new_session(&self) -> Result<Session, String> {
        self.validate()?;
        Ok(Session {
            schema_version: 1,
            id: self.id.clone(),
            version: self.version,
            view: View::Introduction,
            assignment: Attempt::new(&self.assignment)?,
            practice: Attempt::new(&self.example)?,
        })
    }
    pub fn context(&self, s: &Session) -> Result<(), String> {
        if s.schema_version != 1 || s.id != self.id || s.version != self.version {
            return Err("Saved work belongs to another represented-network lesson/version.".into());
        }
        s.assignment.context(&self.assignment)?;
        s.practice.context(&self.example)?;
        if crate::cron_workspace::size(s, 131072).is_none() {
            return Err("Saved represented-network work exceeds128KiB.".into());
        }
        Ok(())
    }
}
