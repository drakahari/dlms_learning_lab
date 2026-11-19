//! Bounded source-owned exposure requirements; no scenario registered yet.
use crate::{
    linux_filter as filter, linux_filter_policy, linux_router_path as path,
    linux_router_workspace as router, processes, service_exposure as core,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbeCase {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,160>")]
    pub label: String,
    pub viewpoint: usize,
    pub port: u16,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,core::PortState,3>")]
    pub accepted: Vec<core::PortState>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u8,
    pub initial: core::World,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,router::Policy,2>")]
    pub policies: Vec<router::Policy>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,processes::Requirement,32>")]
    pub requirements: Vec<processes::Requirement>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,ProbeCase,16>")]
    pub cases: Vec<ProbeCase>,
}
impl Task {
    fn facts(&self) -> path::Facts {
        // Comparison uses declared interface names. This is not route discovery.
        path::Facts {
            links: vec![path::Link {
                name: self.initial.device.clone(),
                address: self.initial.server,
                prefix: 32,
                up: true,
            }],
            routes: vec![],
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        self.initial.validate().map_err(|e| e.message)?;
        if self.version != 1
            || self.policies.is_empty()
            || self.policies.len() > 2
            || self.requirements.is_empty()
            || self.requirements.len() > 32
            || self.cases.is_empty()
            || self.cases.len() > 16
            || !self
                .cases
                .iter()
                .any(|c| c.accepted == [core::PortState::Open])
            || !self
                .cases
                .iter()
                .any(|c| !c.accepted.is_empty() && !c.accepted.contains(&core::PortState::Open))
            || serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 256 * 1024
        {
            return Err("Use a version1 bounded investigation with positive access, negative outcomes and process preservation.".into());
        }
        let mut chains = std::collections::BTreeSet::new();
        let mut repair = false;
        for p in &self.policies {
            if p.chain == filter::Chain::Forward || !chains.insert(p.chain.name()) {
                return Err(
                    "This single-host task compares distinct INPUT/OUTPUT policies only.".into(),
                );
            }
            repair |= linux_filter_policy::compare_filter(
                &self.initial.filter,
                p.chain,
                &self.facts(),
                &p.domains,
                &p.allowed,
            )
            .map_err(|e| e.message)?
            .is_some();
        }
        if !self
            .requirements
            .iter()
            .all(|r| r.valid(&self.initial.processes))
        {
            return Err(
                "Process requirements must refer to supplied real virtual owners/endpoints.".into(),
            );
        }
        repair |= self
            .requirements
            .iter()
            .any(|r| !r.check(&self.initial.processes).0);
        let mut ids = std::collections::BTreeSet::new();
        for c in &self.cases {
            if c.id.is_empty()
                || !c
                    .id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                || c.label.trim().is_empty()
                || c.port == 0
                || c.viewpoint >= self.initial.viewpoints.len()
                || c.accepted.is_empty()
                || c.accepted.len() > 3
                || c.accepted
                    .iter()
                    .enumerate()
                    .any(|(i, s)| c.accepted[..i].contains(s))
                || !ids.insert(&c.id)
            {
                return Err("Probe cases need distinct identifiers, labels, supplied observers and valid TCP ports.".into());
            }
            repair |= !c.accepted.contains(
                &self
                    .initial
                    .scan(c.viewpoint, &[c.port])
                    .map_err(|e| e.message)?[0]
                    .state,
            );
        }
        if !repair {
            return Err("An exposure assignment must require a real policy or owner repair, not an already solved workspace.".into());
        }
        Ok(())
    }
    pub fn checks(&self, w: &core::World) -> Vec<router::Check> {
        if let Err(detail) = self.validate() {
            return vec![router::Check {
                passed: false,
                label: "Source task".into(),
                detail,
            }];
        }
        if !w.same_problem(&self.initial) {
            return vec![router::Check{passed:false,label:"Supplied server and identities".into(),detail:"The saved workspace differs from this task's supplied server, observer or process/listener metadata. Reset this attempt or restore its matching backup.".into()}];
        }
        let mut checks = vec![];
        for p in &self.policies {
            let (passed,detail)=match linux_filter_policy::compare_filter(&w.filter,p.chain,&self.facts(),&p.domains,&p.allowed) {
                Ok(None)=>(true,"Every supported header in the required source/port domains has the intended filter decision.".into()),
                Ok(Some(m))=>(false,format!("For {}, current {:?} differs from required {:?}. A few successful probes do not prove that a whole client range is protected.",router::packet_text(&m.packet),m.actual,m.expected)),
                Err(e)=>(false,e.message),
            };
            checks.push(router::Check {
                passed,
                label: format!("{} access policy", p.chain.name()),
                detail,
            });
        }
        for r in &self.requirements {
            let (passed, detail) = r.check(&w.processes);
            let label = match r {
                processes::Requirement::Listener { endpoint, present } => format!(
                    "{} listener {}:{}",
                    if *present { "Retain" } else { "Remove" },
                    endpoint.address,
                    endpoint.port,
                ),
                processes::Requirement::Unchanged { pid, expected } => {
                    format!("Preserve {} (process {pid})", expected.comm,)
                }
                processes::Requirement::Phase { pid, expected } => format!(
                    "{} (process {pid}) must be {}",
                    self.initial.processes.processes[pid].comm,
                    match expected {
                        processes::ExpectedPhase::Active => "active",
                        processes::ExpectedPhase::Stopped => "paused",
                        processes::ExpectedPhase::Exited => "ended",
                    },
                ),
                processes::Requirement::NoPendingTermination { pid } => {
                    format!("No pending ending request for process {pid}",)
                }
                processes::Requirement::Niceness { pid, .. } => {
                    format!("Priority of process {pid}",)
                }
            };
            checks.push(router::Check {
                passed,
                label,
                detail,
            });
        }
        for c in &self.cases {
            let (passed, detail) = match w.scan(c.viewpoint, &[c.port]) {
                Ok(rows) => {
                    let actual = rows[0].state;
                    (
                        c.accepted.contains(&actual),
                        format!(
                            "Required {}; actual {} from {}. {}",
                            c.accepted
                                .iter()
                                .map(|s| s.name())
                                .collect::<Vec<_>>()
                                .join(" or "),
                            actual.name(),
                            w.viewpoints[c.viewpoint].id,
                            actual.explanation()
                        ),
                    )
                }
                Err(e) => (false, e.message),
            };
            checks.push(router::Check {
                passed,
                label: c.label.clone(),
                detail,
            });
        }
        checks
    }
}
