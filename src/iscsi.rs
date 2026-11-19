//! Bounded iSCSI initiator model operating on supplied portal and device facts.
pub mod command;

use self::command::Command;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fmt::{self, Write},
    net::Ipv4Addr,
    str::FromStr,
};

type Result<T> = std::result::Result<T, String>;
const MAX_EXPORTS: usize = 12;
const MAX_DEVICES: usize = 4;
const DEFAULT_PORT: u16 = 3260;

fn exports<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Export>, D::Error> {
    crate::cron_workspace::sequence::<_, Export, MAX_EXPORTS>(d)
}

fn devices<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Device>, D::Error> {
    crate::cron_workspace::sequence::<_, Device, MAX_DEVICES>(d)
}

fn records<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Vec<Record>, D::Error> {
    crate::cron_workspace::sequence::<_, Record, MAX_EXPORTS>(d)
}

fn connections<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Connection>, D::Error> {
    crate::cron_workspace::sequence::<_, Connection, MAX_EXPORTS>(d)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Portal {
    pub address: Ipv4Addr,
    pub port: u16,
}

impl FromStr for Portal {
    type Err = String;

    fn from_str(value: &str) -> Result<Self> {
        let (ip, port) = value
            .split_once(':')
            .map_or((value, None), |(ip, port)| (ip, Some(port)));
        let address = ip
            .parse()
            .map_err(|_| "Use a literal supplied IPv4 portal.")?;
        let port = match port {
            Some(p) if !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()) => p
                .parse::<u16>()
                .ok()
                .filter(|n| *n != 0)
                .ok_or("Use a portal port from 1 to 65,535.")?,
            None => DEFAULT_PORT,
            _ => return Err("Use IPv4 or IPv4:PORT without a portal group suffix.".into()),
        };
        Ok(Self { address, port })
    }
}

impl fmt::Display for Portal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.address, self.port)
    }
}

pub fn valid_target(value: &str) -> bool {
    value.len() <= 96
        && value
            .strip_prefix("iqn.")
            .is_some_and(|rest| !rest.is_empty())
        && value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b".:-".contains(&b))
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    #[serde(deserialize_with = "crate::cron::text::<_,96>")]
    pub target: String,
    pub portal: Portal,
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {}", self.target, self.portal)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    pub lun: u8,
    #[serde(deserialize_with = "crate::cron::text::<_,3>")]
    pub name: String,
    #[serde(deserialize_with = "crate::cron::text::<_,18>")]
    pub wwn: String,
}

impl Device {
    fn valid(&self) -> bool {
        let name = self.name.as_bytes();
        matches!(name, [b's', b'd', letter] if letter.is_ascii_lowercase())
            && self.wwn.strip_prefix("0x5").is_some_and(|rest| {
                rest.len() == 15
                    && rest
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
    pub key: Key,
    pub discovery: Portal,
    pub group: u16,
    pub login_available: bool,
    #[serde(deserialize_with = "devices")]
    pub devices: Vec<Device>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    #[serde(deserialize_with = "exports")]
    pub exports: Vec<Export>,
}

impl Inventory {
    pub fn validate(&self) -> Result<()> {
        let mut keys = BTreeSet::new();
        let mut names = BTreeSet::new();
        let mut identities = BTreeSet::new();
        if self.exports.is_empty() || self.exports.len() > MAX_EXPORTS {
            return Err("Supply between one and twelve virtual iSCSI exports.".into());
        }
        for entry in &self.exports {
            let mut luns = BTreeSet::new();
            if !valid_target(&entry.key.target)
                || entry.key.portal.port == 0
                || entry.discovery.port == 0
                || !keys.insert(&entry.key)
                || entry.devices.is_empty()
                || entry.devices.len() > MAX_DEVICES
                || entry.devices.iter().any(|disk| {
                    !disk.valid()
                        || !luns.insert(disk.lun)
                        || !names.insert(&disk.name)
                        || !identities.insert(&disk.wwn)
                })
            {
                return Err("Use unique virtual target/portal keys, disk names and WWNs, with unique LUNs per export.".into());
            }
        }
        Ok(())
    }

    pub fn export(&self, key: &Key) -> Option<&Export> {
        self.exports.iter().find(|e| &e.key == key)
    }

    pub fn device(&self, key: &Key, lun: u8) -> Option<&Device> {
        self.export(key)?.devices.iter().find(|d| d.lun == lun)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Startup {
    #[default]
    Manual,
    Automatic,
}

impl FromStr for Startup {
    type Err = String;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "manual" => Ok(Self::Manual),
            "automatic" => Ok(Self::Automatic),
            _ => Err("Use manual or automatic for node.startup.".into()),
        }
    }
}

impl fmt::Display for Startup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Manual => "manual",
            Self::Automatic => "automatic",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub key: Key,
    pub startup: Startup,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Connection {
    pub key: Key,
    pub id: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Initiator {
    #[serde(deserialize_with = "records")]
    pub nodes: Vec<Record>,
    #[serde(deserialize_with = "connections")]
    pub active: Vec<Connection>,
    pub next_id: u32,
}

impl Default for Initiator {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            active: Vec::new(),
            next_id: 1,
        }
    }
}

impl Initiator {
    pub fn validate(&self, inventory: &Inventory) -> Result<()> {
        let mut names = BTreeSet::new();
        let mut active = BTreeSet::new();
        let mut ids = BTreeSet::new();
        if self.nodes.len() > MAX_EXPORTS
            || self.active.len() > MAX_EXPORTS
            || self.next_id == 0
            || self
                .nodes
                .iter()
                .any(|n| inventory.export(&n.key).is_none() || !names.insert(&n.key))
            || self.active.iter().any(|s| {
                !names.contains(&s.key)
                    || inventory.export(&s.key).is_none_or(|e| !e.login_available)
                    || !active.insert(&s.key)
                    || s.id == 0
                    || s.id >= self.next_id
                    || !ids.insert(s.id)
            })
        {
            return Err("The virtual node database and active sessions must refer to supplied exports and unique session IDs.".into());
        }
        Ok(())
    }

    pub fn node(&self, key: &Key) -> Option<&Record> {
        self.nodes.iter().find(|n| &n.key == key)
    }

    pub fn session(&self, key: &Key) -> Option<&Connection> {
        self.active.iter().find(|s| &s.key == key)
    }

    pub fn startup_candidates(&self) -> impl Iterator<Item = &Key> {
        self.nodes
            .iter()
            .filter(|n| n.startup == Startup::Automatic)
            .map(|n| &n.key)
    }

    pub fn execute(&mut self, inventory: &Inventory, command: Command) -> Result<String> {
        match command {
            Command::Help => Ok(command::HELP.into()),
            Command::Discover(portal) => self.discover(inventory, portal),
            Command::Nodes => {
                if self.nodes.is_empty() {
                    return Err(
                        "iscsiadm: No records found. Discover the supplied portal first.".into(),
                    );
                }
                Ok(self
                    .nodes
                    .iter()
                    .map(|n| entry_line(inventory, &n.key))
                    .collect::<Vec<_>>()
                    .join("\n"))
            }
            Command::Show(key) => {
                let record = self.node(&key).ok_or_else(|| missing_node(&key))?;
                Ok(format!(
                    "node.name = {}\nnode.tpgt = {}\nnode.startup = {}\nnode.conn[0].address = {}\nnode.conn[0].port = {}",
                    key.target,
                    group(inventory, &key),
                    record.startup,
                    key.portal.address,
                    key.portal.port
                ))
            }
            Command::Login(key) => self.login(inventory, &key),
            Command::Logout(key) => {
                let index = self
                    .active
                    .iter()
                    .position(|s| s.key == key)
                    .ok_or_else(|| format!("iscsiadm: No active session for {key}."))?;
                self.active.remove(index);
                Ok(format!("Logout of {key} successful."))
            }
            Command::SetStartup(key, startup) => {
                let record = self
                    .nodes
                    .iter_mut()
                    .find(|n| n.key == key)
                    .ok_or_else(|| missing_node(&key))?;
                record.startup = startup;
                Ok(format!(
                    "Saved node.startup = {startup} for {key}. Existing sessions are unchanged."
                ))
            }
            Command::LoginAutomatic => {
                let keys: Vec<_> = self.startup_candidates().cloned().collect();
                if keys.is_empty() {
                    return Err("iscsiadm: No records with node.startup = automatic.".into());
                }
                let mut rows = Vec::new();
                let mut failed = false;
                for key in keys {
                    match self.login(inventory, &key) {
                        Ok(line) => rows.push(line),
                        Err(error) if self.session(&key).is_some() => rows.push(error),
                        Err(error) => {
                            rows.push(error);
                            failed = true;
                        }
                    }
                }
                if failed {
                    Err(rows.join("\n"))
                } else {
                    Ok(rows.join("\n"))
                }
            }
            Command::Sessions { devices } => self.describe_sessions(inventory, devices),
            Command::DeviceIds => {
                let mut output = String::from("NAME WWN");
                let mut disks: Vec<_> = self
                    .active
                    .iter()
                    .filter_map(|s| inventory.export(&s.key))
                    .flat_map(|e| &e.devices)
                    .collect();
                disks.sort_by_key(|d| &d.name);
                for disk in disks {
                    let _ = write!(output, "\n{:4} {}", disk.name, disk.wwn);
                }
                Ok(output)
            }
        }
    }

    fn discover(&mut self, inventory: &Inventory, portal: Portal) -> Result<String> {
        let found: Vec<_> = inventory
            .exports
            .iter()
            .filter(|e| e.discovery == portal)
            .collect();
        if found.is_empty() {
            return Err(format!(
                "iscsiadm: No SendTargets response from {portal} in this workspace."
            ));
        }
        for entry in &found {
            if self.node(&entry.key).is_none() {
                self.nodes.push(Record {
                    key: entry.key.clone(),
                    startup: Startup::Manual,
                });
            }
        }
        Ok(found
            .into_iter()
            .map(|e| entry_line(inventory, &e.key))
            .collect::<Vec<_>>()
            .join("\n"))
    }

    fn login(&mut self, inventory: &Inventory, key: &Key) -> Result<String> {
        if self.node(key).is_none() {
            return Err(missing_node(key));
        }
        if self.session(key).is_some() {
            return Err(format!("iscsiadm: Session for {key} is already active."));
        }
        if !inventory.export(key).is_some_and(|e| e.login_available) {
            return Err(format!(
                "iscsiadm: Login to {key} failed in this workspace. Check the target and portal together."
            ));
        }
        let id = self.next_id;
        self.next_id = id
            .checked_add(1)
            .ok_or("Session ID limit reached. Reset this exercise to start again.")?;
        self.active.push(Connection {
            id,
            key: key.clone(),
        });
        Ok(format!("Login to {key} successful."))
    }

    fn describe_sessions(&self, inventory: &Inventory, devices: bool) -> Result<String> {
        if self.active.is_empty() {
            return Err("iscsiadm: No active sessions.".into());
        }
        let mut output = String::new();
        for session in &self.active {
            let Some(entry) = inventory.export(&session.key) else {
                continue;
            };
            if devices {
                let _ = writeln!(
                    output,
                    "Target: {}\n    Current Portal: {},{}\n    SID: {}\n    iSCSI Session State: LOGGED_IN\n    Attached SCSI devices:",
                    session.key.target, session.key.portal, entry.group, session.id
                );
                for disk in &entry.devices {
                    let _ = writeln!(
                        output,
                        "        Lun: {}\n            Attached scsi disk {}    State: running",
                        disk.lun, disk.name
                    );
                }
            } else {
                let _ = writeln!(
                    output,
                    "tcp: [{}] {},{} {} (non-flash)",
                    session.id, session.key.portal, entry.group, session.key.target
                );
            }
        }
        Ok(output.trim_end().into())
    }
}

fn group(inventory: &Inventory, key: &Key) -> u16 {
    inventory.export(key).map_or(0, |e| e.group)
}

fn entry_line(inventory: &Inventory, key: &Key) -> String {
    format!("{},{} {}", key.portal, group(inventory, key), key.target)
}

fn missing_node(key: &Key) -> String {
    format!("iscsiadm: No node record for {key}. Check the discovery result for both values.")
}

pub mod session;
pub mod source;
pub mod workspace;
