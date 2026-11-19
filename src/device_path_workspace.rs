//! Source-bound cable repair, derived Ethernet domains and IPv4 round trips.
use crate::{
    device_path::Facts,
    switching::{Cable, Jack},
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

pub const HELP: &str = "Patch only the supplied virtual cables. Each jack accepts one cable. All switch ports are enabled, untagged VLAN 1. The router's two interfaces are separate Ethernet endpoints. Check paths evaluates actual simulated broadcast visibility and IPv4 requests and replies. No real traffic, shell or network runs. Loops, MAC aging, ARP timing and physical measurements are outside this activity.";
pub const PREFIX: &str = "DEVICE PATH WIRING\n";
type Checks = Vec<(bool, String)>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Probe {
    Local {
        from: String,
        to: String,
    },
    Separate {
        from: String,
        to: String,
    },
    Routed {
        from: String,
        destination: Ipv4Addr,
        via: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "crate::cron::text::<_,1600>")]
    pub instructions: String,
    pub facts: Facts,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Cable,8>")]
    pub initial: Vec<Cable>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Cable,8>")]
    pub reference: Vec<Cable>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Probe,6>")]
    pub probes: Vec<Probe>,
}

fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

impl Task {
    pub fn choices(&self) -> Vec<Jack> {
        self.facts
            .fabric
            .hosts
            .keys()
            .map(|host| Jack::Host { host: host.clone() })
            .chain(
                self.facts
                    .fabric
                    .switches
                    .iter()
                    .flat_map(|(switch, value)| {
                        value.ports.keys().map(move |port| Jack::Switch {
                            switch: switch.clone(),
                            port: port.clone(),
                        })
                    }),
            )
            .collect()
    }

    fn shape(&self, cables: &[Cable]) -> Result<(), String> {
        let choices = self.choices();
        if cables.len() != self.facts.fabric.cables.len()
            || cables
                .iter()
                .zip(&self.facts.fabric.cables)
                .any(|(cable, original)| {
                    cable.id != original.id
                        || !choices.contains(&cable.a)
                        || !choices.contains(&cable.b)
                })
        {
            return Err("Use only the supplied cable identities and device jacks.".into());
        }
        Ok(())
    }

    pub fn outcome(&self, cables: &[Cable]) -> Checks {
        let (fabric, network) = match self.shape(cables).and_then(|()| self.facts.project(cables)) {
            Ok(value) => value,
            Err(error) => {
                return vec![(
                    false,
                    format!(
                        "Wiring could not be tested. {error} Each jack needs its own connection."
                    ),
                )];
            }
        };
        let mut checks = Vec::new();
        for probe in &self.probes {
            let result = match probe {
                Probe::Local { from, to } => {
                    let passed = fabric
                        .visibility(from, to)
                        .is_ok_and(|visibility| visibility.mutual());
                    let detail = if passed {
                        "both endpoints share the switched network."
                    } else {
                        "no two-way local path. Check their cables and switch ports."
                    };
                    (
                        passed,
                        format!(
                            "Local Ethernet {} / {}: {detail}",
                            self.facts.fabric.hosts[from].label, self.facts.fabric.hosts[to].label
                        ),
                    )
                }
                Probe::Separate { from, to } => {
                    let passed = fabric
                        .visibility(from, to)
                        .is_ok_and(|visibility| visibility.isolated());
                    let detail = if passed {
                        "neither endpoint receives the other's Ethernet broadcasts."
                    } else {
                        "the networks are joined at Layer 2. Find a cable bypassing the router boundary."
                    };
                    (
                        passed,
                        format!(
                            "Broadcast separation {} / {}: {detail}",
                            self.facts.fabric.hosts[from].label, self.facts.fabric.hosts[to].label
                        ),
                    )
                }
                Probe::Routed {
                    from,
                    destination,
                    via,
                } => match network.probe(from, *destination) {
                    Ok(report) => {
                        let passed = report.forward.hops.iter().any(|hop| &hop.device == via)
                            && report.reply.hops.iter().any(|hop| &hop.device == via);
                        let outward = report
                            .forward
                            .hops
                            .iter()
                            .map(|hop| hop.device.as_str())
                            .chain(std::iter::once(report.forward.destination_device.as_str()))
                            .collect::<Vec<_>>()
                            .join(" -> ");
                        let returning = report
                            .reply
                            .hops
                            .iter()
                            .map(|hop| hop.device.as_str())
                            .chain(std::iter::once(report.reply.destination_device.as_str()))
                            .collect::<Vec<_>>()
                            .join(" -> ");
                        let detail = if passed {
                            "Both use the required router."
                        } else {
                            "A working path must use the required router in both directions."
                        };
                        (
                            passed,
                            format!(
                                "IPv4 {from} to {destination}: outward [{outward}], reply [{returning}]. {detail}"
                            ),
                        )
                    }
                    Err(error) => {
                        let first = error
                            .split(". ")
                            .next()
                            .unwrap_or(&error)
                            .trim_end_matches('.');
                        (
                            false,
                            format!(
                                "IPv4 {from} to {destination}: {first}. Check the cable path to this device's gateway and the server-side router connection."
                            ),
                        )
                    }
                },
            };
            checks.push(result);
        }
        checks
    }

    pub fn validate(&self) -> Result<(), String> {
        self.facts.validate()?;
        self.shape(&self.initial)?;
        self.shape(&self.reference)?;
        self.facts.project(&self.initial)?;
        if !identifier(&self.id)
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || self.instructions.chars().any(char::is_control)
            || !(3..=6).contains(&self.probes.len())
        {
            return Err(
                "Supply a named device-path task and three to six path requirements.".into(),
            );
        }
        for probe in &self.probes {
            let valid = match probe {
                Probe::Local { from, to } | Probe::Separate { from, to } => {
                    from != to
                        && self.facts.fabric.hosts.contains_key(from)
                        && self.facts.fabric.hosts.contains_key(to)
                }
                Probe::Routed {
                    from,
                    destination,
                    via,
                } => {
                    self.facts.network.devices.contains_key(from)
                        && self
                            .facts
                            .network
                            .devices
                            .get(via)
                            .is_some_and(|device| device.forwarding)
                        && self.facts.network.devices.values().any(|device| {
                            device.interfaces.values().any(|interface| {
                                interface
                                    .address
                                    .as_ref()
                                    .is_some_and(|address| address.address == *destination)
                            })
                        })
                }
            };
            if !valid {
                return Err("A path requirement refers to unavailable equipment.".into());
            }
        }
        if !self
            .probes
            .iter()
            .any(|probe| matches!(probe, Probe::Local { .. }))
            || !self
                .probes
                .iter()
                .any(|probe| matches!(probe, Probe::Separate { .. }))
            || !self
                .probes
                .iter()
                .any(|probe| matches!(probe, Probe::Routed { .. }))
            || self.outcome(&self.initial).iter().all(|row| row.0)
            || self.outcome(&self.reference).iter().any(|row| !row.0)
        {
            return Err("Teach local switching, broadcast separation and routed round trips with an incomplete starter and a reachable repair.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Cable,8>")]
    pub cables: Vec<Cable>,
    pub checked: bool,
}
impl Draft {
    pub fn new(task: &Task) -> Self {
        Self {
            cables: task.initial.clone(),
            checked: false,
        }
    }
    pub fn edit(&mut self, task: &Task, cables: Vec<Cable>) -> Result<(), String> {
        task.shape(&cables)?;
        if self.cables != cables {
            self.cables = cables;
            self.checked = false;
        }
        Ok(())
    }
    pub fn test(&mut self, task: &Task) -> Result<(), String> {
        task.shape(&self.cables)?;
        self.checked = true;
        Ok(())
    }
    pub fn checks(&self, task: &Task) -> Checks {
        if !self.checked {
            return vec![(
                false,
                "Paths unchecked. Select Check paths after the final wiring change.".into(),
            )];
        }
        task.outcome(&self.cables)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.initial.id == self.example.id
            || self.initial.facts.network == self.example.facts.network
        {
            return Err("Keep separate worked and assignment device-path facts.".into());
        }
        self.initial.validate()?;
        self.example.validate()
    }
    pub fn checks(&self, session: &Session) -> Checks {
        if let Err(error) = session.bound(self) {
            return vec![(false, error)];
        }
        session.assignment.checks(&self.initial)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub context: String,
    pub view: crate::script_teaching::View,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn new(definition: &Definition) -> Self {
        Self {
            version: 1,
            context: definition.initial.id.clone(),
            view: crate::script_teaching::View::Introduction,
            assignment: Draft::new(&definition.initial),
            practice: Draft::new(&definition.example),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !identifier(&self.context)
            || self.assignment.cables.len() > 8
            || self.practice.cables.len() > 8
        {
            return Err("Saved cable activity metadata is incompatible.".into());
        }
        Ok(())
    }
    pub fn bound(&self, definition: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.context != definition.initial.id || self.version != definition.version {
            return Err("Saved cable work belongs to another task.".into());
        }
        definition.initial.shape(&self.assignment.cables)?;
        definition.example.shape(&self.practice.cables)
    }
    pub fn execute_reference(
        &mut self,
        definition: &Definition,
        input: &str,
    ) -> Result<String, String> {
        self.bound(definition)?;
        let text = input
            .strip_prefix(PREFIX)
            .ok_or("Use the virtual cable controls for this activity.")?;
        if text.len() > 8192 {
            return Err("Wiring edit exceeds the small activity limit.".into());
        }
        let mut next: Draft = serde_json::from_str(text).map_err(|error| error.to_string())?;
        next.test(&definition.initial)?;
        let checks = next.checks(&definition.initial);
        self.assignment = next;
        Ok(format!(
            "{}/{} paths passed.",
            checks.iter().filter(|row| row.0).count(),
            checks.len()
        ))
    }
}

fn admitted(scenario: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::DevicePath { simulation },
        crate::engine::Goal::DevicePath { definition },
    ) = (&scenario.initial, &scenario.goal)
    else {
        return Err("The supplied device-path task and workspace do not match.".into());
    };
    if scenario.track != "CCNA"
        || definition.initial.id != scenario.id
        || definition.initial.instructions != scenario.objective
        || **simulation != Session::new(definition)
    {
        return Err("Match the CCNA task, instructions and initial cable workspace.".into());
    }
    Ok(definition)
}
pub fn source(scenario: &crate::pack::Scenario) -> Result<&Definition, String> {
    let definition = admitted(scenario)?;
    definition.validate()?;
    Ok(definition)
}
pub fn context(
    attempt: &crate::progress::Attempt,
    scenario: &crate::pack::Scenario,
) -> Result<(), String> {
    let definition = admitted(scenario)?;
    let crate::engine::State::DevicePath { simulation } = &attempt.state else {
        return Err("Saved work belongs to another workspace.".into());
    };
    if attempt.scenario_version != scenario.version
        || attempt.ipv4.is_some()
        || attempt.workspace.as_ref().is_some_and(|workspace| {
            serde_json::to_value(workspace).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Device-path progress metadata is incompatible.".into());
    }
    simulation.bound(definition)
}
