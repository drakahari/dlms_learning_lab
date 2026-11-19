//! Fixed chrony source observations with literal configuration. No clock, DNS,
//! sockets, process or host filesystem APIs are used.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};

pub const MAX_CONFIG: usize = 2048;
pub const MAX_SOURCES: usize = 8;
pub const HELP: &str = "Literal chrony configuration: up to three supplied IPv4 server lines with optional iburst, blank lines and # comments. The sample compares zero or one eligible source. It does not model selection among multiple eligible servers, real polling, clock adjustment or DNS.";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Assessment {
    Eligible,
    Unreachable,
    Unsynchronized,
    InsufficientSamples,
}
impl Assessment {
    pub fn detail(self) -> &'static str {
        match self {
            Self::Eligible => "Reachable and eligible in this sample.",
            Self::Unreachable => "No valid response in the last eight polls.",
            Self::Unsynchronized => "The server responded but its own clock is unsynchronized.",
            Self::InsufficientSamples => {
                "The server responded but there are too few usable measurements."
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub seconds_ago: u16,
    pub offset_us: i32,
    pub uncertainty_us: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub address: Ipv4Addr,
    pub stratum: u8,
    pub poll: u8,
    pub reach: u8,
    pub assessment: Assessment,
    pub last_good: Option<Sample>,
}
impl Source {
    fn valid(self) -> bool {
        let plausible = self.poll <= 16
            && self.last_good.is_none_or(|sample| {
                sample.seconds_ago <= 99
                    && sample.offset_us.unsigned_abs() <= 1_000_000
                    && sample.uncertainty_us <= 1_000_000
            });
        plausible
            && match self.assessment {
                Assessment::Eligible => {
                    self.reach > 0 && (1..=14).contains(&self.stratum) && self.last_good.is_some()
                }
                Assessment::Unreachable => {
                    self.reach == 0 && self.stratum == 0 && self.last_good.is_none()
                }
                Assessment::Unsynchronized => self.stratum <= 16,
                Assessment::InsufficientSamples => {
                    self.reach > 0 && (1..=15).contains(&self.stratum)
                }
            }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConfiguredSource {
    pub address: Ipv4Addr,
    pub iburst: bool,
}

pub fn parse(source: &str) -> Result<Vec<ConfiguredSource>, String> {
    if source.len() > MAX_CONFIG || source.lines().count() > 32 {
        return Err("Keep the simulated chrony file within 2 KiB and 32 lines.".into());
    }
    if source
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
    {
        return Err("Unsupported control character in the chrony file.".into());
    }
    let mut entries = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, line) in source.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let words: Vec<_> = line.split_ascii_whitespace().collect();
        let (address, iburst) = match words.as_slice() {
            ["server", address] => (address, false),
            ["server", address, "iburst"] => (address, true),
            _ => {
                return Err(format!(
                    "Line {}: use server IPV4 with optional iburst.",
                    index + 1
                ));
            }
        };
        let address = address.parse::<Ipv4Addr>().map_err(|_| {
            format!(
                "Line {}: use a supplied dotted-decimal IPv4 address.",
                index + 1
            )
        })?;
        if entries.len() >= 3 {
            return Err("This exercise supports up to three configured servers.".into());
        }
        if !seen.insert(address) {
            return Err(format!("Line {}: list each server only once.", index + 1));
        }
        entries.push(ConfiguredSource { address, iburst });
    }
    if entries.is_empty() {
        return Err("Add at least one server line using a supplied address.".into());
    }
    Ok(entries)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub configured: Vec<Source>,
    pub selected: Option<Ipv4Addr>,
}
impl Observation {
    pub fn report_sources(&self) -> String {
        let mut output = String::from(
            "MS Name/IP address         Stratum Poll Reach LastRx Last sample\n===============================================================\n",
        );
        for source in &self.configured {
            let state = if self.selected == Some(source.address) {
                "^*"
            } else {
                "^?"
            };
            let (age, sample) = source.last_good.map_or_else(
                || ("-".into(), "-".into()),
                |value| {
                    (
                        value.seconds_ago.to_string(),
                        format!(
                            "{:+}us [{:+}us] +/- {}us",
                            value.offset_us, value.offset_us, value.uncertainty_us
                        ),
                    )
                },
            );
            output.push_str(&format!(
                "{state} {:<22} {:>7} {:>4} {:>5o} {:>6} {sample}\n",
                source.address.to_string(),
                source.stratum,
                source.poll,
                source.reach,
                age,
            ));
        }
        output
    }

    pub fn report_tracking(&self) -> String {
        let selected = self
            .configured
            .iter()
            .find(|s| self.selected == Some(s.address));
        match selected {
            None => "Reference ID    : 00000000\nStratum         : 0\nLeap status     : Not synchronised\n".into(),
            Some(source) => format!(
                "Reference ID    : {:08X} ({})\nStratum         : {}\nLeap status     : Normal\n",
                u32::from(source.address), source.address, source.stratum + 1,
            ),
        }
    }
}

pub fn observe(source: &str, inventory: &[Source]) -> Result<Observation, String> {
    if inventory.is_empty()
        || inventory.len() > MAX_SOURCES
        || inventory.iter().any(|peer| !peer.valid())
        || inventory
            .iter()
            .map(|peer| peer.address)
            .collect::<BTreeSet<_>>()
            .len()
            != inventory.len()
    {
        return Err("Invalid supplied chrony observations.".into());
    }
    let configured = parse(source)?
        .into_iter()
        .map(|entry| {
            inventory
                .iter()
                .copied()
                .find(|peer| peer.address == entry.address)
                .ok_or_else(|| {
                    format!(
                        "{} has no supplied observation. Choose a server from the lab inventory.",
                        entry.address
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut eligible = configured
        .iter()
        .filter(|peer| peer.assessment == Assessment::Eligible);
    let selected = eligible.next().map(|peer| peer.address);
    if eligible.next().is_some() {
        return Err("This exercise does not compare several eligible servers. Use a supplied selection with at most one eligible server.".into());
    }
    Ok(Observation {
        configured,
        selected,
    })
}
