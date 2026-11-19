//! Bounded normal-flow tuples, never a TCP stack or host I/O.
//! NEW/ESTABLISHED describe observed directions, not TCP/application completion.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, net::Ipv4Addr};
pub const MAX_CONNECTIONS: usize = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Transport {
    Tcp,
    Udp,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub address: Ipv4Addr,
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
    pub fn valid(&self) -> bool {
        self.source.address != self.destination.address
            && [self.source.address, self.destination.address]
                .iter()
                .all(|a| {
                    !a.is_unspecified()
                        && !a.is_multicast()
                        && !a.is_broadcast()
                        && !a.is_loopback()
                        && !a.is_link_local()
                })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Class {
    New,
    Established,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Original,
    Reply,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: u64,
    /// Incoming original tuple, before destination translation.
    pub original: Tuple,
    /// After destination translation, before source translation: filter-stage tuple.
    pub forward: Tuple,
    /// After source translation, before leaving: the reply arrives as its reverse.
    pub wire: Tuple,
    pub seen_reply: bool,
    #[serde(deserialize_with = "optional_device")]
    pub masquerade_device: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    /// Original destination endpoint changed, independent of which direction is observed.
    pub dnat: bool,
    pub id: Option<u64>,
    pub class: Class,
    pub direction: Direction,
    pub filter: Tuple,
    pub outgoing: Tuple,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Table {
    pub next_id: u64,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Entry,64>")]
    pub entries: Vec<Entry>,
}
fn optional_device<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    struct Optional;
    impl<'de> serde::de::Visitor<'de> for Optional {
        type Value = Option<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("null or one bounded interface name")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
        fn visit_some<D: serde::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
            let name = crate::cron::text::<D, 15>(d)?;
            if !device(&name) {
                return Err(serde::de::Error::custom(
                    "Use one bounded supplied interface name",
                ));
            }
            Ok(Some(name))
        }
    }
    d.deserialize_option(Optional)
}
fn device(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 15
        && s != "."
        && s != ".."
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
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
            return Err("Connection table identifier or capacity limit.".into());
        }
        let (mut ids, mut keys) = (BTreeSet::new(), BTreeSet::new());
        for e in &self.entries {
            if e.id == 0
                || e.id >= self.next_id
                || !ids.insert(e.id)
                || !e.original.valid()
                || !e.forward.valid()
                || !e.wire.valid()
                || e.original.transport != e.forward.transport
                || e.original.transport != e.wire.transport
                || e.forward.source != e.original.source
                || e.wire.destination != e.forward.destination
                || e.masquerade_device.as_deref().is_some_and(|s| !device(s))
            {
                return Err(
                    "Connection entry violates the declared numeric tuple/hook-order bounds."
                        .into(),
                );
            }
            if !keys.insert(e.original) || !keys.insert(e.wire.reverse()) {
                return Err(
                    "Ambiguous original/reply lookup keys; no connection was accepted.".into(),
                );
            }
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 65536 {
            return Err("Connection table exceeds64KiB.".into());
        }
        Ok(())
    }
    pub fn preview(&self, incoming: Tuple) -> Result<Observation, String> {
        self.validate()?;
        if !incoming.valid() {
            return Err("Normal-flow exercises require supplied unicast tuples; other packet behavior is unmodeled.".into());
        }
        for e in &self.entries {
            if e.original == incoming {
                return Ok(Observation {
                    dnat: e.original.destination != e.forward.destination,
                    id: Some(e.id),
                    class: if e.seen_reply {
                        Class::Established
                    } else {
                        Class::New
                    },
                    direction: Direction::Original,
                    filter: e.forward,
                    outgoing: e.wire,
                });
            }
            if e.wire.reverse() == incoming {
                return Ok(Observation {
                    dnat: e.original.destination != e.forward.destination,
                    id: Some(e.id),
                    class: Class::Established,
                    direction: Direction::Reply,
                    filter: e.forward.reverse(),
                    outgoing: e.original.reverse(),
                });
            }
        }
        Ok(Observation {
            dnat: false,
            id: None,
            class: Class::New,
            direction: Direction::Original,
            filter: incoming,
            outgoing: incoming,
        })
    }
    /// Caller confirms a new mapping only after the modeled original path/filter accepts it.
    /// First-packet rejection must not fabricate a confirmed connection.
    pub fn confirm(
        &mut self,
        original: Tuple,
        forward: Tuple,
        wire: Tuple,
        masquerade_device: Option<String>,
    ) -> Result<u64, String> {
        self.validate()?;
        if self.entries.len() == MAX_CONNECTIONS {
            return Err("Simulated connection table is full; no mapping changed.".into());
        }
        let mut next = self.clone();
        let id = next.next_id;
        next.next_id = id
            .checked_add(1)
            .ok_or("Connection identifier limit; no mapping changed")?;
        next.entries.push(Entry {
            id,
            original,
            forward,
            wire,
            seen_reply: false,
            masquerade_device,
        });
        next.validate()?;
        *self = next;
        Ok(id)
    }
    /// Called at the declared conntrack observation point, before a later filter verdict.
    /// A seen reply belongs to an already confirmed entry even when that reply is filtered.
    pub fn observe(&mut self, incoming: Tuple) -> Result<Observation, String> {
        let result = self.preview(incoming)?;
        if result.direction == Direction::Reply {
            let id = result.id.ok_or("Reply requires a confirmed tuple")?;
            self.entries
                .iter_mut()
                .find(|e| e.id == id)
                .ok_or("Unknown connection")?
                .seen_reply = true;
        }
        Ok(result)
    }
    pub fn forget_masquerade(&mut self, name: &str) -> Result<usize, String> {
        self.validate()?;
        if !device(name) {
            return Err("Use one supplied interface name.".into());
        }
        let before = self.entries.len();
        self.entries
            .retain(|e| e.masquerade_device.as_deref() != Some(name));
        Ok(before - self.entries.len())
    }
}
