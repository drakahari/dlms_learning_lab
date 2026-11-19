//! Proposed finite IPv4 address-match lists. Not integrated or curriculum credit.
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    pub address: Ipv4Addr,
    pub prefix: u8,
}
impl Network {
    pub fn parse(text: &str) -> Result<Self, String> {
        let (address, prefix) = match text.split_once('/') {
            Some((address, prefix)) => {
                if prefix.is_empty()
                    || prefix.len() > 2
                    || !prefix.bytes().all(|b| b.is_ascii_digit())
                {
                    return Err("Use a full IPv4 address with a prefix from /0 through /32.".into());
                }
                (
                    address,
                    prefix.parse::<u8>().map_err(|_| "Invalid IPv4 prefix")?,
                )
            }
            None => (text, 32),
        };
        let address = address.parse::<Ipv4Addr>().map_err(|_| "Use a complete dotted-decimal IPv4 address; named ACLs, IPv6 and abbreviated networks are unsupported here.")?;
        let value = Self { address, prefix };
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.prefix > 32 {
            return Err("An IPv4 ACL prefix must be from /0 through /32.".into());
        }
        if u32::from(self.address) & self.mask() != u32::from(self.address) {
            return Err("This subset requires the network base before /prefix. Host addresses without a prefix mean one /32 address.".into());
        }
        Ok(())
    }
    fn mask(&self) -> u32 {
        u32::MAX
            .checked_shl(32 - u32::from(self.prefix))
            .unwrap_or(0)
    }
    pub fn interval(&self) -> Result<(u32, u32), String> {
        self.validate()?;
        let start = u32::from(self.address);
        Ok((start, start | !self.mask()))
    }
    pub fn contains(&self, address: Ipv4Addr) -> bool {
        self.interval()
            .is_ok_and(|(a, b)| (a..=b).contains(&u32::from(address)))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Acl {
    Any,
    None,
    Networks { networks: Vec<Network> },
}
// Empty struct variants, rather than unit variants, make Serde reject every
// extra field for any/none. The public serialized shape stays {"kind":"any"}.
impl<'de> Deserialize<'de> for Acl {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
        enum Raw {
            Any {},
            None {},
            Networks {
                #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Network,32>")]
                networks: Vec<Network>,
            },
        }
        let value = match Raw::deserialize(deserializer)? {
            Raw::Any {} => Self::Any,
            Raw::None {} => Self::None,
            Raw::Networks { networks } => Self::Networks { networks },
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}
impl Acl {
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Networks { networks } = self {
            if networks.len() > 32 {
                return Err("Keep an address-match list within 32 entries.".into());
            }
            for network in networks {
                network.validate()?;
            }
        }
        Ok(())
    }
    pub fn intervals(&self) -> Result<Vec<(u32, u32)>, String> {
        self.validate()?;
        let mut rows = match self {
            Self::Any => return Ok(vec![(0, u32::MAX)]),
            Self::None => return Ok(vec![]),
            Self::Networks { networks } => networks
                .iter()
                .map(Network::interval)
                .collect::<Result<Vec<_>, _>>()?,
        };
        rows.sort_unstable();
        let mut result: Vec<(u32, u32)> = vec![];
        for (start, end) in rows {
            if let Some(last) = result.last_mut()
                && u64::from(start) <= u64::from(last.1) + 1
            {
                last.1 = last.1.max(end);
            } else {
                result.push((start, end));
            }
        }
        Ok(result)
    }
    pub fn permits(&self, address: Ipv4Addr) -> Result<bool, String> {
        let address = u32::from(address);
        Ok(self
            .intervals()?
            .iter()
            .any(|&(a, b)| (a..=b).contains(&address)))
    }
    /// Intersection of effective permission lists, normalized over all IPv4
    /// values rather than inferred from a few sampled clients. No host access.
    pub fn intersection(lists: &[&Self]) -> Result<Vec<(u32, u32)>, String> {
        if lists.len() > 8 {
            return Err("At most eight access lists can be combined.".into());
        }
        let mut result = vec![(0, u32::MAX)];
        for list in lists {
            let next = list.intervals()?;
            let mut merged = vec![];
            let (mut a, mut b) = (0, 0);
            while a < result.len() && b < next.len() {
                let low = result[a].0.max(next[b].0);
                let high = result[a].1.min(next[b].1);
                if low <= high {
                    merged.push((low, high));
                }
                if result[a].1 < next[b].1 {
                    a += 1;
                } else {
                    b += 1;
                }
            }
            result = merged;
        }
        Ok(result)
    }
    pub fn equivalent(&self, other: &Self) -> Result<bool, String> {
        Ok(self.intervals()? == other.intervals()?)
    }
}
