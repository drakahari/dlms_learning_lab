//! Typed inventory schema applied only to a finite supplied Ethernet fabric.
use crate::switching::{Fabric, Mode};
use serde::{
    Deserialize, Serialize,
    de::{Error, MapAccess, Visitor},
};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const MAX_DOCUMENT: usize = 8192;

fn enabled<'de, D: serde::Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
    struct Enabled;
    impl<'de> Visitor<'de> for Enabled {
        type Value = bool;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("up as true or false without quotes")
        }
        fn visit_bool<E: Error>(self, value: bool) -> Result<bool, E> {
            Ok(value)
        }
    }
    d.deserialize_bool(Enabled)
}
fn vlan<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u16, D::Error> {
    struct Vlan;
    impl<'de> Visitor<'de> for Vlan {
        type Value = u16;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("vlan as an integer from 1 to 4094 written without quotes, a decimal point or exponent")
        }
        fn visit_u64<E: Error>(self, value: u64) -> Result<u16, E> {
            if !(1..=4094).contains(&value) {
                return Err(E::custom("vlan needs an integer from 1 to 4094"));
            }
            Ok(value as u16)
        }
    }
    d.deserialize_u16(Vlan)
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PortRecord {
    #[serde(deserialize_with = "crate::cron::text::<_,32>")]
    pub name: String,
    #[serde(deserialize_with = "enabled")]
    pub up: bool,
    #[serde(deserialize_with = "vlan")]
    pub vlan: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SwitchRecord {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub name: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,PortRecord,6>")]
    pub ports: Vec<PortRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub note: Option<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,SwitchRecord,2>")]
    pub switches: Vec<SwitchRecord>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub fabric: Fabric,
    #[serde(deserialize_with = "editable")]
    pub editable: BTreeMap<String, BTreeSet<String>>,
}

fn editable<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<String, BTreeSet<String>>, D::Error> {
    #[derive(Deserialize)]
    struct SwitchName(#[serde(deserialize_with = "crate::cron::text::<_,64>")] String);
    #[derive(Deserialize)]
    struct PortName(#[serde(deserialize_with = "crate::cron::text::<_,32>")] String);
    #[derive(Deserialize)]
    struct Ports(
        #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,PortName,6>")]
        Vec<PortName>,
    );
    struct Editable;
    impl<'de> Visitor<'de> for Editable {
        type Value = BTreeMap<String, BTreeSet<String>>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most two distinct editable switches with six named access ports each")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(SwitchName(name)) = map.next_key()? {
                if out.len() == 2 || out.contains_key(&name) {
                    return Err(A::Error::custom("Editable switch limit or duplicate name"));
                }
                let Ports(ports) = map.next_value()?;
                let count = ports.len();
                let ports: BTreeSet<_> = ports.into_iter().map(|port| port.0).collect();
                if ports.len() != count {
                    return Err(A::Error::custom("Duplicate editable access port"));
                }
                out.insert(name, ports);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Editable)
}

impl Spec {
    pub fn validate(&self) -> Result<(), String> {
        self.fabric.validate()?;
        if !(1..=2).contains(&self.editable.len())
            || self.fabric.switches.len() > 2
            || self.fabric.hosts.len() > 8
        {
            return Err("Supply one or two small switches and at most eight endpoints.".into());
        }
        for (name, ports) in &self.editable {
            let switch = self
                .fabric
                .switches
                .get(name)
                .ok_or("Unknown editable switch.")?;
            if !(1..=6).contains(&ports.len())
                || ports.iter().any(|port| {
                    port.len() > 32
                        || !matches!(
                            switch.ports.get(port).map(|port| &port.mode),
                            Some(Mode::Access { .. })
                        )
                })
            {
                return Err(
                    "Only the supplied access ports can be edited in this inventory.".into(),
                );
            }
        }
        Ok(())
    }

    pub fn apply(&self, text: &str) -> Result<Fabric, String> {
        self.validate()?;
        if text.len() > MAX_DOCUMENT || text.contains('\0') {
            return Err("Keep the inventory within 8 KiB with no NUL.".into());
        }
        let document: Document = serde_json::from_str(text).map_err(|error| {
            let category = match error.classify() {
                serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                    "JSON syntax or numeric-range error"
                }
                serde_json::error::Category::Data => "Document field error",
                serde_json::error::Category::Io => "Document read error",
            };
            format!(
                "{category} at line {}, column {}. {error}",
                error.line(),
                error.column()
            )
        })?;
        if document
            .note
            .as_ref()
            .is_some_and(|note| note.len() > 160 || note.chars().any(char::is_control))
        {
            return Err(
                "Document field error. note accepts null or one printable string up to 160 bytes."
                    .into(),
            );
        }
        let mut next = self.fabric.clone();
        let mut switches = BTreeSet::new();
        for record in document.switches {
            if !switches.insert(record.name.clone()) {
                return Err(format!(
                    "Duplicate switch record {}. Keep one record per supplied switch.",
                    record.name
                ));
            }
            let editable = self.editable.get(&record.name).ok_or_else(|| {
                format!(
                    "Unknown switch {}. Keep the supplied equipment names.",
                    record.name
                )
            })?;
            let switch = next
                .switches
                .get_mut(&record.name)
                .ok_or("Supplied switch is unavailable.")?;
            let mut ports = BTreeSet::new();
            for entry in record.ports {
                if !ports.insert(entry.name.clone()) {
                    return Err(format!(
                        "Duplicate port {} on {}. Keep one record per supplied access port.",
                        entry.name, record.name
                    ));
                }
                if !editable.contains(&entry.name) {
                    return Err(format!(
                        "Port {} on {} is outside this inventory. Trunks and other equipment stay supplied.",
                        entry.name, record.name
                    ));
                }
                if !switch.vlans.contains(&entry.vlan) {
                    return Err(format!(
                        "VLAN {} is outside the supplied active VLAN database on {}.",
                        entry.vlan, record.name
                    ));
                }
                let port = switch
                    .ports
                    .get_mut(&entry.name)
                    .ok_or("Supplied port is unavailable.")?;
                port.up = entry.up;
                port.mode = Mode::Access { vlan: entry.vlan };
            }
            if &ports != editable {
                return Err(format!(
                    "Incomplete {} inventory. Include each supplied editable access port exactly once.",
                    record.name
                ));
            }
        }
        if switches != self.editable.keys().cloned().collect() {
            return Err(
                "Incomplete inventory. Include every supplied editable switch exactly once.".into(),
            );
        }
        next.validate()?;
        Ok(next)
    }
}
