//! Bounded identity-mapped IPv6 TCP/UDP direction tracking; no NAT/kernel/socket.
//! Ordinary numeric unicast profile, not global reachability; link-local tracking/zones are excluded.
pub use crate::linux_connections::{Class, Direction, Transport};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv6Addr};
pub const MAX_CONNECTIONS: usize = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub address: Ipv6Addr,
    pub port: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Tuple {
    pub transport: Transport,
    pub source: Endpoint,
    pub destination: Endpoint,
}
impl Tuple {
    pub fn reverse(self) -> Self {
        Self {
            source: self.destination,
            destination: self.source,
            ..self
        }
    }
    pub fn valid(self) -> bool {
        self.source.address != self.destination.address
            && [self.source.address, self.destination.address]
                .iter()
                .all(|a| {
                    matches!(
                        crate::ipv6::category(*a),
                        crate::ipv6::Category::UniqueLocal
                            | crate::ipv6::Category::Documentation
                            | crate::ipv6::Category::OtherUnicast
                    )
                })
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: u64,
    pub original: Tuple,
    pub seen_reply: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub next_id: u64,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Entry,64>")]
    pub entries: Vec<Entry>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub id: Option<u64>,
    pub class: Class,
    pub direction: Direction,
}
impl Default for Table {
    fn default() -> Self {
        Self {
            next_id: 1,
            entries: vec![],
        }
    }
}
impl Table {
    pub fn validate(&self) -> Result<(), String> {
        if self.next_id == 0 || self.entries.len() > MAX_CONNECTIONS {
            return Err("IPv6 connection identifier/capacity bound.".into());
        }
        let (mut ids, mut keys) = (BTreeSet::new(), BTreeSet::new());
        for e in &self.entries {
            if e.id == 0
                || e.id >= self.next_id
                || !ids.insert(e.id)
                || !e.original.valid()
                || !keys.insert(e.original)
                || !keys.insert(e.original.reverse())
            {
                return Err(
                    "IPv6 connection IDs, tuples or reverse lookup keys are inconsistent.".into(),
                );
            }
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 64 * 1024 {
            return Err("IPv6 connection table exceeds64KiB.".into());
        }
        Ok(())
    }
    fn position(&self, t: Tuple) -> Option<(usize, Direction)> {
        self.entries.iter().enumerate().find_map(|(i, e)| {
            if e.original == t {
                Some((i, Direction::Original))
            } else if e.original.reverse() == t {
                Some((i, Direction::Reply))
            } else {
                None
            }
        })
    }
    pub fn preview(&self, t: Tuple) -> Result<Observation, String> {
        self.validate()?;
        if !t.valid() {
            return Err("This tracker supports distinct ordinary IPv6 unicast TCP/UDP endpoints; link-local zones, raw/ICMP and local loops are outside its profile.".into());
        }
        if let Some((index, direction)) = self.position(t) {
            let e = &self.entries[index];
            Ok(Observation {
                id: Some(e.id),
                class: if e.seen_reply || direction == Direction::Reply {
                    Class::Established
                } else {
                    Class::New
                },
                direction,
            })
        } else {
            Ok(Observation {
                id: None,
                class: Class::New,
                direction: Direction::Original,
            })
        }
    }
    /// An observed reply remains observed even if a later route/filter gate rejects it.
    pub fn observe(&mut self, t: Tuple) -> Result<Observation, String> {
        let observation = self.preview(t)?;
        if observation.direction == Direction::Reply {
            let index = self
                .position(t)
                .ok_or("IPv6 reply lookup changed unexpectedly")?
                .0;
            self.entries[index].seen_reply = true;
        }
        Ok(observation)
    }
    /// Called only after an allowed original event; rejected events create no entry.
    pub fn confirm(&mut self, t: Tuple) -> Result<u64, String> {
        self.validate()?;
        if !t.valid() {
            return Err("Unsupported IPv6 connection endpoints; nothing confirmed.".into());
        }
        if self.position(t).is_some() {
            return Err(
                "This IPv6 conversation is already confirmed; no duplicate created.".into(),
            );
        }
        if self.entries.len() == MAX_CONNECTIONS {
            return Err(
                "The simulated IPv6 connection table is full; no new mapping or success accepted."
                    .into(),
            );
        }
        let id = self.next_id;
        let next_id = id
            .checked_add(1)
            .ok_or("IPv6 connection identifier limit; original state retained.")?;
        let mut next = self.clone();
        next.entries.push(Entry {
            id,
            original: t,
            seen_reply: false,
        });
        next.next_id = next_id;
        next.validate()?;
        *self = next;
        Ok(id)
    }
}
