//! Represented workload admission/host failure, no I/O.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kernel {
    Linux,
    Windows,
    FreeBsd,
}
impl Kernel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Linux => "Linux",
            Self::Windows => "Windows",
            Self::FreeBsd => "FreeBSD",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Environment {
    VirtualMachine { kernel: Kernel },
    LinuxContainer,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Host {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub id: String,
    pub kernel: Kernel,
    pub hypervisor: bool,
    pub container_runtime: bool,
    /// A stated planning budget, not a real allocator or universal overcommit rule.
    pub memory_mib: u32,
    pub cpu_units: u8,
    pub up: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workload {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub id: String,
    pub environment: Environment,
    pub memory_mib: u32,
    pub cpu_units: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub service: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Host,4>")]
    pub hosts: Vec<Host>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Workload,8>")]
    pub workloads: Vec<Workload>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Placement {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub workload: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub host: String,
    pub up: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Placement,8>")]
    pub placements: Vec<Placement>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Experiment {
    Normal,
    HostOutage {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        host: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub workload: String,
    pub host: String,
    pub ready: bool,
    pub detail: String,
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
impl Facts {
    pub fn validate(&self) -> Result<(), String> {
        if !(2..=4).contains(&self.hosts.len()) || !(2..=8).contains(&self.workloads.len()) {
            return Err("Supply two to four represented hosts and two to eight workloads.".into());
        }
        let mut ids = BTreeSet::new();
        for h in &self.hosts {
            if !name(&h.id)
                || !ids.insert(&h.id)
                || !(256..=131072).contains(&h.memory_mib)
                || !(1..=64).contains(&h.cpu_units)
                || (h.container_runtime && h.kernel != Kernel::Linux)
            {
                return Err("Invalid host identity, planning budget or unsupported non-Linux container runtime.".into());
            }
        }
        ids.clear();
        for w in &self.workloads {
            if !name(&w.id)
                || !name(&w.service)
                || !ids.insert(&w.id)
                || !(64..=65536).contains(&w.memory_mib)
                || !(1..=32).contains(&w.cpu_units)
            {
                return Err("Invalid workload identity, service or resource request.".into());
            }
        }
        Ok(())
    }
    pub fn validate_configuration(&self, c: &Configuration) -> Result<(), String> {
        self.validate()?;
        if c.placements.len() != self.workloads.len() {
            return Err("Place every supplied workload exactly once.".into());
        }
        let mut seen = BTreeSet::new();
        for p in &c.placements {
            if !seen.insert(&p.workload)
                || !self.workloads.iter().any(|w| w.id == p.workload)
                || !self.hosts.iter().any(|h| h.id == p.host)
            {
                return Err("A placement must name one known workload and host; duplicate placements are not supported.".into());
            }
        }
        Ok(())
    }
    pub fn observe(&self, c: &Configuration, e: &Experiment) -> Result<Vec<Observation>, String> {
        self.validate_configuration(c)?;
        let outage = match e {
            Experiment::Normal => None,
            Experiment::HostOutage { host } => {
                if !self.hosts.iter().any(|h| h.id == *host) {
                    return Err("Choose a supplied host for the outage.".into());
                }
                Some(host)
            }
        };
        let mut totals: BTreeMap<&str, (u32, u16)> = BTreeMap::new();
        for p in &c.placements {
            if p.up {
                let w = self.workloads.iter().find(|w| w.id == p.workload).unwrap();
                let t = totals.entry(&p.host).or_default();
                t.0 += w.memory_mib;
                t.1 += u16::from(w.cpu_units);
            }
        }
        Ok(self
            .workloads
            .iter()
            .map(|w| {
                let p = c.placements.iter().find(|p| p.workload == w.id).unwrap();
                let h = self.hosts.iter().find(|h| h.id == p.host).unwrap();
                let budget = totals.get(h.id.as_str()).copied().unwrap_or_default();
                let fault = if !p.up {
                    Some("The workload is switched off.")
                } else if !h.up || outage == Some(&h.id) {
                    Some("Its represented physical host is unavailable.")
                } else if matches!(w.environment, Environment::VirtualMachine { .. }) && !h.hypervisor {
                    Some("This host has no supplied hypervisor for a virtual machine.")
                } else if matches!(w.environment, Environment::LinuxContainer)
                    && (!h.container_runtime || h.kernel != Kernel::Linux)
                {
                    Some("This Linux container needs a supplied Linux host kernel and container runtime.")
                } else if budget.0 > h.memory_mib || budget.1 > u16::from(h.cpu_units) {
                    Some("The placement exceeds this exercise's host memory or CPU-unit planning budget.")
                } else {
                    None
                };
                let detail = fault.map(str::to_owned).unwrap_or_else(|| match w.environment {
                    Environment::VirtualMachine { kernel } => format!(
                        "Ready in this plan: its own {} guest kernel runs through {}'s supplied hypervisor.",
                        kernel.label(), h.id
                    ),
                    Environment::LinuxContainer => format!(
                        "Ready in this plan: its isolated process environment shares {}'s Linux kernel.",
                        h.id
                    ),
                });
                Observation { workload: w.id.clone(), host: h.id.clone(), ready: fault.is_none(), detail }
            })
            .collect())
    }

    pub fn service_ready(
        &self,
        c: &Configuration,
        e: &Experiment,
        service: &str,
    ) -> Result<bool, String> {
        if !self.workloads.iter().any(|w| w.service == service) {
            return Err("Unknown supplied service group.".into());
        }
        let results = self.observe(c, e)?;
        Ok(self
            .workloads
            .iter()
            .filter(|w| w.service == service)
            .any(|w| results.iter().any(|o| o.workload == w.id && o.ready)))
    }
}
