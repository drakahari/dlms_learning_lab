//! Offline previews of per-device edits, shared controller profiles, and reconciliation.
//! All forwarding decisions come from the existing virtual Ethernet model.
use crate::{rest_controller as rest, switching as ethernet};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortSettings {
    pub up: bool,
    pub vlan: u16,
}

impl PortSettings {
    fn validate(self) -> Result<(), String> {
        if !(1..=4094).contains(&self.vlan) {
            return Err("Use a VLAN from 1 to 4094.".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    pub desired: rest::Store,
    #[serde(deserialize_with = "bounded_overrides")]
    pub local: BTreeMap<String, PortSettings>,
}

fn bounded_overrides<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<String, PortSettings>, D::Error> {
    struct Overrides;
    impl<'de> serde::de::Visitor<'de> for Overrides {
        type Value = BTreeMap<String, PortSettings>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("at most 12 distinct supplied slot overrides")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut rows: A,
        ) -> Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some(key) = rows.next_key::<String>()? {
                if result.len() >= 12 || !rest::identifier(&key) || result.contains_key(&key) {
                    return Err(serde::de::Error::custom(
                        "Too many overrides, invalid name or repeated slot",
                    ));
                }
                result.insert(key, rows.next_value()?);
            }
            Ok(result)
        }
    }
    d.deserialize_map(Overrides)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "method", rename_all = "snake_case", deny_unknown_fields)]
pub enum Change {
    Individual {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        slot: String,
        settings: PortSettings,
    },
    SharedProfile {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        profile: String,
        settings: PortSettings,
    },
    Rebind {
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        slot: String,
        #[serde(deserialize_with = "crate::cron::text::<_,32>")]
        profile: String,
    },
    Reconcile,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PortChange {
    pub slot: String,
    pub before: PortSettings,
    pub after: PortSettings,
    pub origin_changed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preview {
    source: rest::Spec,
    based_on: Inventory,
    next: Inventory,
    pub affected: Vec<PortChange>,
}

impl Inventory {
    pub fn installed(&self, spec: &rest::Spec) -> Result<ethernet::Fabric, String> {
        let mut running = spec.project(&self.desired)?;
        if self.local.len() > 12 {
            return Err("This workspace supports up to twelve supplied port overrides.".into());
        }
        for (id, settings) in &self.local {
            settings.validate()?;
            let slot = spec.ports.get(id).ok_or("Choose a supplied device slot.")?;
            let port = running
                .switches
                .get_mut(&slot.switch)
                .and_then(|switch| switch.ports.get_mut(&slot.port))
                .ok_or("The supplied slot is missing from the current switch inventory.")?;
            if !matches!(port.mode, ethernet::Mode::Access { .. }) {
                return Err("Only the supplied access ports can be changed here.".into());
            }
            port.up = settings.up;
            port.mode = ethernet::Mode::Access {
                vlan: settings.vlan,
            };
        }
        running.validate()?;
        Ok(running)
    }

    pub fn settings(&self, id: &str) -> Result<PortSettings, String> {
        if let Some(local) = self.local.get(id) {
            return Ok(*local);
        }
        let binding = self
            .desired
            .bindings
            .get(id)
            .ok_or("Choose a supplied device slot.")?;
        let p = self
            .desired
            .profiles
            .get(&binding.profile)
            .ok_or("The slot's controller profile is missing.")?;
        Ok(PortSettings {
            up: p.up,
            vlan: p.vlan,
        })
    }

    pub fn preview(&self, spec: &rest::Spec, change: &Change) -> Result<Preview, String> {
        self.installed(spec)?;
        let mut next = self.clone();
        match change {
            Change::Individual { slot, settings } => {
                settings.validate()?;
                if !spec.ports.contains_key(slot) {
                    return Err("Choose a supplied device slot.".into());
                }
                next.local.insert(slot.clone(), *settings);
            }
            Change::SharedProfile { profile, settings } => {
                settings.validate()?;
                let tuned = next
                    .desired
                    .profiles
                    .get_mut(profile)
                    .ok_or("Choose a supplied controller profile.")?;
                tuned.up = settings.up;
                tuned.vlan = settings.vlan;
                next.local.retain(|slot, _| {
                    next.desired
                        .bindings
                        .get(slot)
                        .is_some_and(|b| b.profile != *profile)
                });
            }
            Change::Rebind { slot, profile } => {
                if !next.desired.profiles.contains_key(profile) {
                    return Err("Choose an existing controller profile.".into());
                }
                let binding = next
                    .desired
                    .bindings
                    .get_mut(slot)
                    .ok_or("Choose a supplied device slot.")?;
                binding.profile.clone_from(profile);
                next.local.remove(slot);
            }
            Change::Reconcile => next.local.clear(),
        }
        next.installed(spec)?;
        let mut affected = Vec::new();
        for slot in spec.ports.keys() {
            let before = self.settings(slot)?;
            let after = next.settings(slot)?;
            let origin_changed = self.local.get(slot) != next.local.get(slot)
                || self.desired.bindings.get(slot) != next.desired.bindings.get(slot);
            if before != after || origin_changed {
                affected.push(PortChange {
                    slot: slot.clone(),
                    before,
                    after,
                    origin_changed,
                });
            }
        }
        Ok(Preview {
            source: spec.clone(),
            based_on: self.clone(),
            next,
            affected,
        })
    }

    pub fn apply(&mut self, spec: &rest::Spec, preview: Preview) -> Result<(), String> {
        if *spec != preview.source || *self != preview.based_on {
            return Err("The virtual network changed. Preview this rollout again.".into());
        }
        *self = preview.next;
        Ok(())
    }

    pub fn visible(
        &self,
        spec: &rest::Spec,
        source: &str,
        destination: &str,
    ) -> Result<bool, String> {
        self.installed(spec)?
            .visibility(source, destination)
            .map(|visible| visible.mutual())
    }

    pub fn isolated(
        &self,
        spec: &rest::Spec,
        source: &str,
        destination: &str,
    ) -> Result<bool, String> {
        self.installed(spec)?
            .visibility(source, destination)
            .map(|visible| visible.isolated())
    }
}
