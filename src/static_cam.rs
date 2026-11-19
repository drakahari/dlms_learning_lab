//! Supplied configured destination bindings over the existing VLAN forest.
//! No host networking, commands, timers, security enforcement or hidden state.
use crate::{
    mac_learning::{Mac, Observation, State},
    switching::{Fabric, Mode},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const MAX_BINDINGS: usize = 64;
pub const MAX_JSON: usize = 32 * 1024;
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_./:-".contains(&b))
        && !s.split('/').any(|p| p.is_empty() || p == "." || p == "..")
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub switch: String,
    pub vlan: u16,
    pub address: Mac,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub port: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Rows", into = "Rows")]
pub struct Table {
    rows: Vec<Binding>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Rows {
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Binding,64>")]
    rows: Vec<Binding>,
}
impl TryFrom<Rows> for Table {
    type Error = String;
    fn try_from(r: Rows) -> Result<Self, String> {
        Self::new(r.rows)
    }
}
impl From<Table> for Rows {
    fn from(t: Table) -> Self {
        Self { rows: t.rows }
    }
}
impl Table {
    pub fn new(rows: Vec<Binding>) -> Result<Self, String> {
        let mut keys = BTreeSet::new();
        if rows.len() > MAX_BINDINGS
            || rows.iter().any(|r| {
                !name(&r.switch)
                    || !name(&r.port)
                    || !(1..=4094).contains(&r.vlan)
                    || !r.address.unicast()
                    || !keys.insert((&r.switch, r.vlan, r.address))
            })
        {
            return Err("Use at most64 unique switch/VLAN/unicast-MAC bindings with bounded supplied port names.".into());
        }
        Ok(Self { rows })
    }
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_JSON {
            return Err("Static forwarding configuration exceeds32KiB.".into());
        }
        serde_json::from_str(raw).map_err(|_|"Invalid static forwarding data: use bounded complete unicast addresses and unique switch/VLAN/MAC keys.".into())
    }
    pub fn rows(&self) -> &[Binding] {
        &self.rows
    }
    pub fn lookup(&self, switch: &str, vlan: u16, address: Mac) -> Option<&Binding> {
        self.rows
            .iter()
            .find(|r| r.switch == switch && r.vlan == vlan && r.address == address)
    }
    pub fn validate_for(&self, fabric: &Fabric) -> Result<(), String> {
        fabric.validate()?;
        for row in &self.rows {
            let sw = fabric
                .switches
                .get(&row.switch)
                .ok_or("Static binding refers to an unknown supplied switch.")?;
            if !sw.vlans.contains(&row.vlan) {
                return Err("Static binding VLAN must exist on its supplied switch.".into());
            }
            let port = sw
                .ports
                .get(&row.port)
                .ok_or("Static binding refers to an unknown supplied port.")?;
            let carries = match &port.mode {
                Mode::Access { vlan } => *vlan == row.vlan,
                Mode::Trunk { allowed, .. } => allowed.contains(&row.vlan),
            };
            if !carries {
                return Err("Static destination port does not carry this VLAN in the supplied configuration.".into());
            }
        }
        Ok(())
    }
    /// Display derives from configuration plus actual dynamic state. A static
    /// row never acquires a fabricated learned timestamp or dynamic age.
    pub fn effective_rows(&self, state: &State) -> Result<Vec<EffectiveRow>, String> {
        state.validate()?;
        self.validate_for(state.fabric())?;
        let mut rows: Vec<_> = self
            .rows
            .iter()
            .map(|r| EffectiveRow {
                binding: r.clone(),
                origin: Origin::Configured,
            })
            .collect();
        rows.extend(
            state
                .entries()
                .iter()
                .filter(|r| self.lookup(&r.switch, r.vlan, r.address).is_none())
                .map(|r| EffectiveRow {
                    binding: Binding {
                        switch: r.switch.clone(),
                        vlan: r.vlan,
                        address: r.address,
                        port: r.port.clone(),
                    },
                    origin: Origin::Learned { at: r.learned_at },
                }),
        );
        Ok(rows)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    Configured,
    Learned { at: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveRow {
    pub binding: Binding,
    pub origin: Origin,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceObservation {
    pub binding: Binding,
    pub ingress: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub forwarding: Observation,
    /// A configured source key was retained instead of dynamically learned.
    /// A different ingress is not itself a security drop in this foundation.
    pub configured_sources: Vec<SourceObservation>,
}
