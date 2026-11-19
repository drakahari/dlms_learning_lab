//! One bounded pre-forward selection over a supplied access-only switch.
//! Not a DHCP parser, lease database or security policy by itself.
use super::{Mac, Observation, State};
use crate::switching::{Fabric, Jack, Mode};
use std::collections::BTreeSet;

pub const MAX_PORTS: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessOutputs {
    switch: String,
    ports: BTreeSet<String>,
}
impl AccessOutputs {
    pub fn new(fabric: &Fabric, switch: &str, ports: BTreeSet<String>) -> Result<Self, String> {
        // Check supplied identifiers before retaining owned policy data.
        Self::validate_parts(fabric, switch, &ports)?;
        Ok(Self {
            switch: switch.into(),
            ports,
        })
    }
    fn validate_parts(
        fabric: &Fabric,
        switch: &str,
        ports: &BTreeSet<String>,
    ) -> Result<(), String> {
        fabric.validate()?;
        if fabric.switches.len() != 1 {
            return Err("This per-frame output subset supports one supplied access switch.".into());
        }
        let s = fabric
            .switches
            .get(switch)
            .ok_or("Choose the supplied access switch.")?;
        if s.ports.len() > MAX_PORTS || ports.len() > MAX_PORTS {
            return Err("Per-frame access output selection supports at most16 ports.".into());
        }
        if s.ports
            .values()
            .any(|p| !matches!(p.mode, Mode::Access { .. }))
        {
            return Err("Trunks are outside this per-frame access output subset.".into());
        }
        if ports.iter().any(|p| !s.ports.contains_key(p)) {
            return Err("Output selection refers to an unknown supplied port.".into());
        }
        Ok(())
    }
    pub fn validate_for(&self, fabric: &Fabric) -> Result<(), String> {
        Self::validate_parts(fabric, &self.switch, &self.ports)
    }
    pub(super) fn permits(&self, switch: &str, port: &str) -> bool {
        self.switch == switch && self.ports.contains(port)
    }
    fn source_is_attached(&self, fabric: &Fabric, host: &str) -> bool {
        let endpoint = Jack::Host { host: host.into() };
        fabric.cables.iter().any(|c| {
            let peer = if c.a == endpoint { &c.b } else if c.b == endpoint { &c.a } else { return false; };
            matches!(peer, Jack::Switch { switch, port } if switch == &self.switch && fabric.switches[switch].ports.contains_key(port))
        })
    }
}

impl State {
    /// Consult this frame's allowed outputs before emitting Forward/arrival.
    /// Existing destination lookup, VLAN, NIC and physical-state rules still apply.
    pub fn send_frame_on_access_ports(
        &mut self,
        source: &str,
        source_mac: Mac,
        destination: Mac,
        outputs: &AccessOutputs,
    ) -> Result<Observation, String> {
        self.validate()?;
        outputs.validate_for(&self.fabric)?;
        if !self.host_addresses.contains_key(source)
            || !outputs.source_is_attached(&self.fabric, source)
        {
            return Err(
                "Choose a supplied workstation directly attached to this access switch.".into(),
            );
        }
        if !source_mac.unicast() {
            return Err(
                "The metadata source must be a complete nonzero unicast MAC address.".into(),
            );
        }
        if !destination.unicast() && destination != Mac::BROADCAST {
            return Err(
                "Unsupported multicast: use an individual or broadcast destination.".into(),
            );
        }
        let mut trial = self.clone();
        let observation = trial.propagate_with_static(
            source,
            destination,
            None,
            &mut Vec::new(),
            source_mac,
            Some(outputs),
        )?;
        trial.validate()?;
        *self = trial;
        Ok(observation)
    }
}
