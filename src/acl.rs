//! Pure bounded IPv4 ACL classification. No host I/O or forwarding effects.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    net::Ipv4Addr,
};

pub const MAX_LISTS: usize = 8;
pub const MAX_RULES: usize = 64;
pub const MAX_JSON_BYTES: usize = 512 * 1024;
pub const MAX_PACKET_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
    UnknownList,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
struct Id(String);
impl<'de> Deserialize<'de> for Id {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Name;
        impl Visitor<'_> for Name {
            type Value = Id;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("an ASCII identifier of 1 to 32 bytes")
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Id, E> {
                if !identifier(value) {
                    return Err(E::custom("Invalid ACL identifier"));
                }
                Ok(Id(value.to_owned()))
            }
        }
        d.deserialize_str(Name)
    }
}
fn id<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    Id::deserialize(d).map(|id| id.0)
}
fn lists<'de, D: Deserializer<'de>>(d: D) -> Result<BTreeMap<String, AccessList>, D::Error> {
    struct Lists;
    impl<'de> Visitor<'de> for Lists {
        type Value = BTreeMap<String, AccessList>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most eight unique ACLs")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Id(key)) = a.next_key()? {
                if out.len() >= MAX_LISTS || out.contains_key(&key) {
                    return Err(A::Error::custom("Duplicate ACL or list limit exceeded"));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Lists)
}
fn rules<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Rule>, D::Error> {
    struct Rules;
    impl<'de> Visitor<'de> for Rules {
        type Value = Vec<Rule>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most 64 ordered ACL rules")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            let mut ids = BTreeSet::new();
            while let Some(rule) = a.next_element::<Rule>()? {
                if out.len() >= MAX_RULES || !ids.insert(rule.id.clone()) {
                    return Err(A::Error::custom("Duplicate rule ID or rule limit exceeded"));
                }
                out.push(rule);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Rules)
}

/// A one bit in wildcard ignores the corresponding address bit, including in base.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wildcard {
    pub base: Ipv4Addr,
    pub wildcard: Ipv4Addr,
}
impl Wildcard {
    pub fn host(base: Ipv4Addr) -> Self {
        Self {
            base,
            wildcard: Ipv4Addr::UNSPECIFIED,
        }
    }
    pub fn any() -> Self {
        Self {
            base: Ipv4Addr::UNSPECIFIED,
            wildcard: Ipv4Addr::BROADCAST,
        }
    }
    pub fn matches(self, address: Ipv4Addr) -> bool {
        (u32::from(address) ^ u32::from(self.base)) & !u32::from(self.wildcard) == 0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Ports {
    Any {},
    Eq { port: u16 },
    Range { first: u16, last: u16 },
}
impl Ports {
    pub fn valid(self) -> bool {
        !matches!(self, Self::Range { first, last } if first > last)
    }
    fn matches(self, port: u16) -> bool {
        match self {
            Self::Any {} => true,
            Self::Eq { port: expected } => port == expected,
            Self::Range { first, last } => (first..=last).contains(&port),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PacketProtocol {
    Icmp {},
    Tcp {
        source_port: u16,
        destination_port: u16,
    },
    Udp {
        source_port: u16,
        destination_port: u16,
    },
    Other {
        number: u8,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Packet {
    pub source: Ipv4Addr,
    pub destination: Ipv4Addr,
    pub protocol: PacketProtocol,
}
impl Packet {
    pub fn validate(&self) -> Result<(), Error> {
        if matches!(self.protocol, PacketProtocol::Other { number: 1 | 6 | 17 }) {
            return Err(error(
                ErrorKind::Invalid,
                "Use the structured ICMP, TCP or UDP packet variant for protocol 1, 6 or 17.",
            ));
        }
        Ok(())
    }
    pub fn from_json(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_PACKET_BYTES {
            return Err(error(
                ErrorKind::Limit,
                "ACL packet data exceeds 512 bytes.",
            ));
        }
        let packet: Self = serde_json::from_slice(bytes).map_err(|_| {
            error(
                ErrorKind::Invalid,
                "Malformed or unsupported ACL packet fields.",
            )
        })?;
        packet.validate()?;
        Ok(packet)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Protocol {
    Ip {},
    Icmp {},
    Tcp {
        source_port: Ports,
        destination_port: Ports,
    },
    Udp {
        source_port: Ports,
        destination_port: Ports,
    },
}
impl Protocol {
    fn valid(self) -> bool {
        match self {
            Self::Ip {} | Self::Icmp {} => true,
            Self::Tcp {
                source_port,
                destination_port,
            }
            | Self::Udp {
                source_port,
                destination_port,
            } => source_port.valid() && destination_port.valid(),
        }
    }
    fn matches(self, packet: PacketProtocol) -> bool {
        match (self, packet) {
            (Self::Ip {}, _) | (Self::Icmp {}, PacketProtocol::Icmp {}) => true,
            (
                Self::Tcp {
                    source_port,
                    destination_port,
                },
                PacketProtocol::Tcp {
                    source_port: s,
                    destination_port: d,
                },
            )
            | (
                Self::Udp {
                    source_port,
                    destination_port,
                },
                PacketProtocol::Udp {
                    source_port: s,
                    destination_port: d,
                },
            ) => source_port.matches(s) && destination_port.matches(d),
            _ => false,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Standard,
    Extended,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Permit,
    Deny,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selector {
    Standard {
        source: Wildcard,
    },
    Extended {
        source: Wildcard,
        destination: Wildcard,
        protocol: Protocol,
    },
}
impl Selector {
    fn valid(self, kind: Kind) -> bool {
        match (kind, self) {
            (Kind::Standard, Self::Standard { .. }) => true,
            (Kind::Extended, Self::Extended { protocol, .. }) => protocol.valid(),
            _ => false,
        }
    }
    fn matches(self, packet: &Packet) -> bool {
        match self {
            Self::Standard { source } => source.matches(packet.source),
            Self::Extended {
                source,
                destination,
                protocol,
            } => {
                source.matches(packet.source)
                    && destination.matches(packet.destination)
                    && protocol.matches(packet.protocol)
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    #[serde(deserialize_with = "id")]
    pub id: String,
    pub action: Action,
    pub selector: Selector,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccessList {
    pub kind: Kind,
    #[serde(deserialize_with = "rules")]
    pub rules: Vec<Rule>,
}
impl AccessList {
    pub fn validate(&self) -> Result<(), Error> {
        if self.rules.len() > MAX_RULES {
            return Err(error(ErrorKind::Limit, "An ACL supports at most 64 rules."));
        }
        if self.rules.is_empty() {
            return Err(error(
                ErrorKind::Unsupported,
                "Empty ACL behavior is outside this classifier; provide at least one rule.",
            ));
        }
        let mut ids = BTreeSet::new();
        for rule in &self.rules {
            if !identifier(&rule.id) || !ids.insert(rule.id.as_str()) {
                return Err(error(
                    ErrorKind::Invalid,
                    "Rule IDs must be valid and unique within the ACL.",
                ));
            }
            if !rule.selector.valid(self.kind) {
                return Err(error(
                    ErrorKind::Invalid,
                    "ACL class and rule class must agree, and port ranges must be ordered.",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema_version: u32,
    #[serde(deserialize_with = "lists")]
    pub lists: BTreeMap<String, AccessList>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Cause {
    /// Stable per-list rule ID and zero-based row position in the current list.
    Matched {
        rule: String,
        row: usize,
    },
    ImplicitDeny,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decision {
    pub list: String,
    pub action: Action,
    pub cause: Cause,
}
impl Policy {
    pub fn validate(&self) -> Result<(), Error> {
        if self.schema_version != 1 {
            return Err(error(
                ErrorKind::Unsupported,
                "Unsupported ACL policy version.",
            ));
        }
        if self.lists.len() > MAX_LISTS {
            return Err(error(
                ErrorKind::Limit,
                "A policy supports at most eight ACLs.",
            ));
        }
        for (name, list) in &self.lists {
            if !identifier(name) {
                return Err(error(
                    ErrorKind::Invalid,
                    "Invalid ACL name; use 1 to 32 ASCII identifier characters.",
                ));
            }
            list.validate()?;
        }
        Ok(())
    }
    pub fn from_json(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_JSON_BYTES {
            return Err(error(ErrorKind::Limit, "ACL policy data exceeds 512 KiB."));
        }
        let policy: Self = serde_json::from_slice(bytes).map_err(|_| error(ErrorKind::Invalid, "Malformed or unsupported ACL policy fields, identifiers, duplicate keys or collection limits."))?;
        policy.validate()?;
        Ok(policy)
    }
    /// Classify metadata only. A denial is not proof of routed-path isolation.
    pub fn evaluate(&self, list: &str, packet: &Packet) -> Result<Decision, Error> {
        self.validate()?;
        packet.validate()?;
        if !identifier(list) {
            return Err(error(ErrorKind::Invalid, "Invalid requested ACL name."));
        }
        let acl = self.lists.get(list).ok_or_else(|| {
            error(
                ErrorKind::UnknownList,
                format!("ACL '{list}' is not defined; missing attachment behavior is unsupported."),
            )
        })?;
        let (action, cause) = acl
            .rules
            .iter()
            .enumerate()
            .find(|(_, rule)| rule.selector.matches(packet))
            .map(|(row, rule)| {
                (
                    rule.action,
                    Cause::Matched {
                        rule: rule.id.clone(),
                        row,
                    },
                )
            })
            .unwrap_or((Action::Deny, Cause::ImplicitDeny));
        Ok(Decision {
            list: list.to_owned(),
            action,
            cause,
        })
    }
}
