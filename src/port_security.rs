//! Bounded secure access-port admission composed with real virtual forwarding.
//! No IOS process, sockets, packet bytes, host interfaces or authentication.
use crate::{
    mac_learning::{Mac, Observation, State},
    static_cam::{Binding, Table},
    switching::{Fabric, Jack, Mode},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MAX_PORTS: usize = 8;
pub const MAX_SECURE: usize = 64;
pub const MAX_COUNTER: u32 = 1_000_000;
pub const MAX_JSON: usize = 768 * 1024;
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationMode {
    Protect,
    Restrict,
    Shutdown,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Port {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub name: String,
    pub enabled: bool,
    pub maximum: u8,
    pub mode: ViolationMode,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Mac,16>")]
    pub configured: Vec<Mac>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Mac,16>")]
    pub dynamic: Vec<Mac>,
    pub admin_up: bool,
    pub error_disabled: bool,
    pub violations: u32,
}
impl Port {
    pub fn default_for(name: &str) -> Self {
        Self {
            name: name.into(),
            enabled: false,
            maximum: 1,
            mode: ViolationMode::Shutdown,
            configured: vec![],
            dynamic: vec![],
            admin_up: true,
            error_disabled: false,
            violations: 0,
        }
    }
    pub fn operational(&self) -> bool {
        self.admin_up && !self.error_disabled
    }
    pub fn count(&self) -> usize {
        self.configured.len() + self.dynamic.len()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct World {
    schema_version: u8,
    baseline: State,
    active: State,
    switch: String,
    ports: Vec<Port>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    schema_version: u8,
    baseline: State,
    active: State,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    switch: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Port,8>")]
    ports: Vec<Port>,
}
impl<'de> Deserialize<'de> for World {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let w = Wire::deserialize(d)?;
        let s = Self {
            schema_version: w.schema_version,
            baseline: w.baseline,
            active: w.active,
            switch: w.switch,
            ports: w.ports,
        };
        s.validate().map_err(serde::de::Error::custom)?;
        Ok(s)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Configured,
    DynamicSecure,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecureRow {
    pub port: String,
    pub vlan: u16,
    pub address: Mac,
    pub origin: Origin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cause {
    Capacity,
    SecuredElsewhere { port: String },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    RestrictViolation,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Unavailable {
        reason: String,
    },
    Violation {
        port: String,
        source: Mac,
        cause: Cause,
        mode: ViolationMode,
        counter: u32,
        notice: Option<Notice>,
    },
    Admitted {
        learned: Option<SecureRow>,
        forwarding: Observation,
    },
}
impl World {
    pub fn new(baseline: State, switch: &str, managed: &[&str]) -> Result<Self> {
        let s = Self {
            schema_version: 1,
            active: baseline.clone(),
            baseline,
            switch: switch.into(),
            ports: managed.iter().map(|p| Port::default_for(p)).collect(),
        };
        s.validate()?;
        Ok(s)
    }
    pub fn from_json(raw: &str) -> Result<Self> {
        if raw.len() > MAX_JSON {
            return Err("Secure-port saved data exceeds 768 KiB.".into());
        }
        serde_json::from_str(raw)
            .map_err(|_| "Invalid or unsupported bounded secure-port state.".into())
    }
    pub fn baseline(&self) -> &State {
        &self.baseline
    }
    pub fn forwarding(&self) -> &State {
        &self.active
    }
    pub fn switch(&self) -> &str {
        &self.switch
    }
    pub fn ports(&self) -> &[Port] {
        &self.ports
    }
    pub fn port(&self, name: &str) -> Option<&Port> {
        self.ports.iter().find(|p| p.name == name)
    }
    pub fn secure_rows(&self) -> Vec<SecureRow> {
        self.ports
            .iter()
            .flat_map(|p| {
                let vlan = self.vlan(&p.name);
                p.configured
                    .iter()
                    .map(move |m| SecureRow {
                        port: p.name.clone(),
                        vlan,
                        address: *m,
                        origin: Origin::Configured,
                    })
                    .chain(p.dynamic.iter().map(move |m| SecureRow {
                        port: p.name.clone(),
                        vlan,
                        address: *m,
                        origin: Origin::DynamicSecure,
                    }))
            })
            .collect()
    }
    fn vlan(&self, port: &str) -> u16 {
        match self.baseline.fabric().switches[&self.switch].ports[port].mode {
            Mode::Access { vlan } => vlan,
            Mode::Trunk { .. } => 0,
        }
    }
    fn projected(&self) -> Fabric {
        let mut fabric = self.baseline.fabric().clone();
        for p in &self.ports {
            fabric
                .switches
                .get_mut(&self.switch)
                .unwrap()
                .ports
                .get_mut(&p.name)
                .unwrap()
                .up = p.operational();
        }
        fabric
    }
    /// A private forwarding-only projection. Never display its Configured label
    /// for learned secure rows: use secure_rows()'s actual provenance instead.
    fn bindings(&self) -> Result<Table> {
        Table::new(
            self.secure_rows()
                .into_iter()
                .map(|r| Binding {
                    switch: self.switch.clone(),
                    vlan: r.vlan,
                    address: r.address,
                    port: r.port,
                })
                .collect(),
        )
    }
    fn project(&mut self) -> Result<()> {
        let fabric = self.projected();
        if self.active.fabric() != &fabric {
            self.active.configure(fabric, self.active.aging_seconds())?;
        }
        Ok(())
    }
    pub fn validate(&self) -> Result<()> {
        self.baseline.validate()?;
        self.active.validate()?;
        let sw = self
            .baseline
            .fabric()
            .switches
            .get(&self.switch)
            .ok_or("Unknown supplied secure switch.")?;
        if self.schema_version != 1
            || self.ports.is_empty()
            || self.ports.len() > MAX_PORTS
            || !self.baseline.entries().is_empty()
            || self.baseline.now() != 0
            || self.active.aging_seconds() != self.baseline.aging_seconds()
            || !self
                .baseline
                .fabric()
                .hosts
                .keys()
                .all(|h| self.baseline.address(h) == self.active.address(h))
        {
            return Err("Keep one supplied initial fabric and 1–8 managed access ports; unsupported secure-port version/state.".into());
        }
        let mut names = BTreeSet::new();
        let mut keys = BTreeSet::new();
        let mut total = 0;
        for p in &self.ports {
            let port = sw.ports.get(&p.name).ok_or("Unknown managed port.")?;
            let Mode::Access { vlan } = port.mode else {
                return Err(
                    "This subset secures directly attached static access ports, not trunks.".into(),
                );
            };
            let jack = Jack::Switch {
                switch: self.switch.clone(),
                port: p.name.clone(),
            };
            let attached = self.baseline.fabric().cables.iter().any(|c| {
                (c.a == jack && matches!(c.b, Jack::Host { .. }))
                    || (c.b == jack && matches!(c.a, Jack::Host { .. }))
            });
            if !names.insert(&p.name)
                || !port.up
                || !sw.vlans.contains(&vlan)
                || !attached
                || !(1..=16).contains(&p.maximum)
                || p.configured.len() > 16
                || p.dynamic.len() > 16
                || p.count() > usize::from(p.maximum)
                || p.violations > MAX_COUNTER
                || (!p.enabled && p.count() != 0)
                || (!p.admin_up && (!p.dynamic.is_empty() || p.error_disabled))
            {
                return Err("Use unique supplied live access attachments, maximum 1–16, and bounded valid secure state.".into());
            }
            for m in p.configured.iter().chain(&p.dynamic) {
                if !m.unicast() || !keys.insert((vlan, *m)) {
                    return Err("A complete unicast address can belong to only one secure port in each VLAN.".into());
                }
            }
            total += p.count();
        }
        if total > MAX_SECURE || self.active.fabric() != &self.projected() {
            return Err("Secure entries exceed 64 or operational forwarding disagrees with current port state.".into());
        }
        self.bindings()?.validate_for(self.active.fabric())?;
        if serde_json::to_vec(self)
            .map_err(|_| "Cannot encode secure state.")?
            .len()
            > MAX_JSON
        {
            return Err("Secure-port state exceeds 768 KiB.".into());
        }
        Ok(())
    }
    fn commit(&mut self, mut trial: Self) -> Result<()> {
        trial.project()?;
        trial.validate()?;
        *self = trial;
        Ok(())
    }
    pub fn configure(
        &mut self,
        name: &str,
        enabled: bool,
        maximum: u8,
        mode: ViolationMode,
        configured: Vec<Mac>,
    ) -> Result<()> {
        self.validate()?;
        if configured.len() > 16 {
            return Err("Use at most 16 configured identities on a port.".into());
        }
        if !(1..=16).contains(&maximum) {
            return Err("Maximum: use a whole number from 1 to 16.".into());
        }
        if !enabled && !configured.is_empty() {
            return Err(
                "Remove configured addresses before disabling security in this subset.".into(),
            );
        }
        if configured.iter().any(|m| !m.unicast()) {
            return Err("Configured addresses must be complete unicast MACs, not broadcast or multicast destinations.".into());
        }
        if configured.iter().collect::<BTreeSet<_>>().len() != configured.len() {
            return Err("Remove duplicate configured MAC addresses from this port.".into());
        }
        let mut trial = self.clone();
        let p = trial
            .ports
            .iter_mut()
            .find(|p| p.name == name)
            .ok_or("Choose a managed access port.")?;
        p.enabled = enabled;
        p.maximum = maximum;
        p.mode = mode;
        p.dynamic.retain(|m| enabled && !configured.contains(m));
        p.configured = configured;
        if p.count() > usize::from(maximum) {
            return Err(format!(
                "This edit needs {} secure slots, but Maximum is {maximum}. The learned secure addresses still occupy slots; Shut down this port to clear them, apply the intended binding, then Bring up.",
                p.count()
            ));
        }
        // Policy changes do not reset an error-disabled latch or counter.
        self.commit(trial)
    }
    /// Down then Up is the explicit manual recovery sequence. Changing policy
    /// alone never invents recovery; learned secure entries clear on Down.
    pub fn set_admin(&mut self, name: &str, up: bool) -> Result<()> {
        self.validate()?;
        let mut trial = self.clone();
        let p = trial
            .ports
            .iter_mut()
            .find(|p| p.name == name)
            .ok_or("Choose a managed access port.")?;
        p.admin_up = up;
        if !up {
            p.dynamic.clear();
            p.error_disabled = false;
        }
        self.commit(trial)
    }
    pub fn advance(&mut self, seconds: u64) -> Result<()> {
        self.validate()?;
        let mut trial = self.clone();
        trial.active.advance(seconds)?;
        self.commit(trial)
    }
    /// One bounded synchronous frame; no loop, clock, history or host execution.
    pub fn send(&mut self, host: &str, source: Mac, destination: Mac) -> Result<Outcome> {
        self.validate()?;
        if !source.unicast() || (!destination.unicast() && destination != Mac::BROADCAST) {
            return Err("Use a complete unicast source and an individual or broadcast destination; multicast is unsupported.".into());
        }
        let endpoint = Jack::Host { host: host.into() };
        let h = self
            .baseline
            .fabric()
            .hosts
            .get(host)
            .ok_or("Choose a supplied workstation attachment.")?;
        let attached = self.baseline.fabric().cables.iter().find_map(|c| {
            if c.a == endpoint {
                Some((c, &c.b))
            } else if c.b == endpoint {
                Some((c, &c.a))
            } else {
                None
            }
        });
        let Some((cable, Jack::Switch { switch, port })) = attached else {
            return Ok(Outcome::Unavailable {
                reason: "This workstation has no supplied switch attachment.".into(),
            });
        };
        let actual = &self.active.fabric().switches[switch].ports[port];
        if !h.up || !cable.up || !actual.up {
            return Ok(Outcome::Unavailable{reason:"The source workstation, cable or current port is down; no security learning or new violation occurred.".into()});
        }
        let mut trial = self.clone();
        let mut learned = None;
        if switch == &self.switch
            && let Some(index) = self.ports.iter().position(|p| p.name == *port && p.enabled)
        {
            let p = &self.ports[index];
            let vlan = self.vlan(port);
            let elsewhere = self
                .secure_rows()
                .into_iter()
                .find(|r| r.vlan == vlan && r.address == source && r.port != *port);
            let known = p.configured.contains(&source) || p.dynamic.contains(&source);
            let cause = elsewhere
                .map(|r| Cause::SecuredElsewhere { port: r.port })
                .or_else(|| {
                    (!known && p.count() >= usize::from(p.maximum)).then_some(Cause::Capacity)
                });
            if let Some(cause) = cause {
                let p = &mut trial.ports[index];
                if p.mode != ViolationMode::Protect {
                    p.violations = p.violations.saturating_add(1).min(MAX_COUNTER);
                }
                if p.mode == ViolationMode::Shutdown {
                    p.error_disabled = true;
                }
                let out = Outcome::Violation {
                    port: port.clone(),
                    source,
                    cause,
                    mode: p.mode,
                    counter: p.violations,
                    notice: (p.mode == ViolationMode::Restrict)
                        .then_some(Notice::RestrictViolation),
                };
                self.commit(trial)?;
                return Ok(out);
            }
            if !known {
                trial.ports[index].dynamic.push(source);
                if trial.secure_rows().len() > MAX_SECURE {
                    return Err("The simulator's 64-secure-entry resource limit was reached; no state changed.".into());
                }
                learned = Some(SecureRow {
                    port: port.clone(),
                    vlan,
                    address: source,
                    origin: Origin::DynamicSecure,
                });
            }
        }
        let bindings = trial.bindings()?;
        let forwarding = trial
            .active
            .send_frame_with_static(host, source, destination, &bindings)?
            .forwarding;
        self.commit(trial)?;
        Ok(Outcome::Admitted {
            learned,
            forwarding,
        })
    }
}
