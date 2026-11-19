//! Finite Linux6.17 IPv6 forwarding/RA settings; all values are virtual.
use crate::linux_filter::{self, Class, Fault};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
type Result<T> = std::result::Result<T, Fault>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interface {
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub name: String,
    pub forwarding: bool,
    pub force_forwarding: bool,
    pub accept_ra: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u8,
    pub global_forwarding: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Interface,8>")]
    pub interfaces: Vec<Interface>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub text: String,
    pub changed: bool,
    /// A validated write event, including idempotent writes. The administration
    /// composition uses this to handle learned-router effects separately.
    pub forwarding_written: Option<bool>,
}
fn fault(class: Class, message: impl Into<String>) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
fn unsupported(message: &str) -> Fault {
    fault(Class::Unsupported, message)
}
impl State {
    pub fn validate(&self) -> Result<()> {
        let mut names = BTreeSet::new();
        if self.version != 1 || self.interfaces.is_empty() || self.interfaces.len() > 8 {
            return Err(fault(
                Class::Context,
                "Supply a version 1 profile with 1–8 fixed interfaces.",
            ));
        }
        for i in &self.interfaces {
            if i.name.is_empty()
                || i.name.len() > 15
                || matches!(i.name.as_str(), "all" | "default")
                || !i
                    .name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
                || !names.insert(&i.name)
                || i.accept_ra > 2
            {
                return Err(fault(
                    Class::Context,
                    "Use unique supplied simple interface names and accept_ra 0, 1, or 2.",
                ));
            }
        }
        Ok(())
    }
    fn interface(&self, name: &str) -> Result<&Interface> {
        self.interfaces
            .iter()
            .find(|i| i.name == name)
            .ok_or_else(|| {
                fault(
                    Class::Context,
                    "Unknown interface; use one of the supplied links.",
                )
            })
    }
    pub fn forwards_from(&self, ingress: &str) -> Result<bool> {
        self.validate()?;
        // Interface.forwarding is Host/Router behavior, not the ingress gate.
        let i = self.interface(ingress)?;
        Ok(self.global_forwarding || i.force_forwarding)
    }
    pub fn accepts_ra(&self, device: &str) -> Result<bool> {
        self.validate()?;
        let i = self.interface(device)?;
        Ok(i.accept_ra == 2 || (i.accept_ra == 1 && !i.forwarding))
    }
    pub(crate) fn get(&self, key: &str) -> Result<u8> {
        let parts = key
            .strip_prefix("net.ipv6.conf.")
            .ok_or_else(|| {
                unsupported("Only supplied IPv6 conf keys are modeled; no host sysctl ran.")
            })?
            .split('.')
            .collect::<Vec<_>>();
        let [device, setting] = parts.as_slice() else {
            return Err(unsupported(
                "Use net.ipv6.conf.INTERFACE.SETTING with a supplied simple interface name.",
            ));
        };
        if *device == "all" {
            return if *setting == "forwarding" {
                Ok(u8::from(self.global_forwarding))
            } else {
                Err(unsupported(
                    "Only all.forwarding is modeled; this is not a general proc filesystem.",
                ))
            };
        }
        if *device == "default" {
            return Err(unsupported(
                "Default settings affect newly created interfaces. This fixed-interface activity does not create interfaces or model that inheritance.",
            ));
        }
        let i = self.interface(device)?;
        match *setting {
            "forwarding" => Ok(u8::from(i.forwarding)),
            "force_forwarding" => Ok(u8::from(i.force_forwarding)),
            "accept_ra" => Ok(i.accept_ra),
            _ => Err(unsupported(
                "Supported interface settings: forwarding, force_forwarding and accept_ra (Linux6.17 profile).",
            )),
        }
    }
    fn set(&mut self, key: &str, value: u8) -> Result<()> {
        self.get(key)?;
        let (device, setting) = key
            .strip_prefix("net.ipv6.conf.")
            .unwrap()
            .split_once('.')
            .unwrap();
        if value > if setting == "accept_ra" { 2 } else { 1 } {
            return Err(fault(
                Class::Syntax,
                "Use 0 or 1 for forwarding flags; accept_ra accepts 0, 1, or 2.",
            ));
        }
        if device == "all" {
            self.global_forwarding = value == 1;
            for i in &mut self.interfaces {
                i.forwarding = value == 1;
                if value == 0 {
                    i.force_forwarding = false;
                }
            }
        } else {
            let i = self
                .interfaces
                .iter_mut()
                .find(|i| i.name == device)
                .unwrap();
            match setting {
                "forwarding" => i.forwarding = value == 1,
                "force_forwarding" => i.force_forwarding = value == 1,
                "accept_ra" => i.accept_ra = value,
                _ => unreachable!("get validated the closed settings set"),
            }
        }
        Ok(())
    }
    pub fn command(&mut self, line: &str) -> Result<Output> {
        self.validate()?;
        let words = linux_filter::words(line)?;
        let mut next = self.clone();
        let mut forwarding_written = None;
        let text = match words
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["sysctl", key] => format!("{key} = {}\n", next.get(key)?),
            ["sysctl", "-w", pair] => {
                let (key, value) = pair.split_once('=').ok_or_else(|| {
                    fault(
                        Class::Syntax,
                        "Use sysctl -w KEY=VALUE; no real settings are changed.",
                    )
                })?;
                let value = match value {
                    "0" => 0,
                    "1" => 1,
                    "2" => 2,
                    _ => {
                        return Err(fault(
                            Class::Syntax,
                            "Use a single decimal value 0, 1, or 2 within this setting's supported range.",
                        ));
                    }
                };
                next.set(key, value)?;
                if key.ends_with(".forwarding") {
                    forwarding_written = Some(value == 1);
                }
                format!("{key} = {}\n", next.get(key)?)
            }
            ["cat", path] => {
                let rest = path
                    .strip_prefix("/proc/sys/net/ipv6/conf/")
                    .ok_or_else(|| {
                        unsupported("Only the supplied virtual IPv6 conf files are readable here.")
                    })?;
                let parts = rest.split('/').collect::<Vec<_>>();
                let [device, setting] = parts.as_slice() else {
                    return Err(unsupported(
                        "Use one supplied conf/interface/setting path; traversal and arbitrary files are unsupported.",
                    ));
                };
                format!(
                    "{}\n",
                    next.get(&format!("net.ipv6.conf.{device}.{setting}"))?
                )
            }
            _ => {
                return Err(unsupported(
                    "Supported here: sysctl KEY, sysctl -w KEY=VALUE, and cat of one supplied virtual IPv6 conf file. No shell, host files or real settings are used.",
                ));
            }
        };
        next.validate()?;
        let changed = next != *self;
        *self = next;
        Ok(Output {
            text,
            changed,
            forwarding_written,
        })
    }
}
