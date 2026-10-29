//! Bounded NTP dependency teaching model, not an NTP wire/clock implementation.
//! All observations derive from virtual state; see NTP_SUBSET.md.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::net::Ipv4Addr;

pub const VERSION: u32 = 1;
pub const MAX_NODES: usize = 16;
pub const MAX_DELIVERIES: usize = MAX_NODES * MAX_NODES;
pub const MAX_STATE_BYTES: usize = 32_768;
pub const HELP: &str = "Virtual single-upstream NTP only: ntp server IPv4; no ntp server IPv4; show ntp status; show ntp associations; show running-config. Configure commands apply to the selected simulated device. A second different server is unsupported: remove the first before adding another. No real clock, packet exchange, polling history, reach register, delay/offset, multipeer selection, ntp master, authentication, DNS, IPv6 or host commands. Status is an immediately recomputed dependency check, not proof of correct UTC.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Unknown,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;
fn err(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}

// Bound collection admission, rejecting duplicate keys before inserting them.
fn nodes<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<Ipv4Addr, Node>, D::Error> {
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = BTreeMap<Ipv4Addr, Node>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("up to16 unique virtual NTP nodes")
        }
        fn visit_map<A: serde::de::MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut result = BTreeMap::new();
            while let Some(k) = a.next_key::<Ipv4Addr>()? {
                if result.len() >= MAX_NODES || result.contains_key(&k) {
                    return Err(serde::de::Error::custom(
                        "NTP node bound or duplicate address",
                    ));
                }
                result.insert(k, a.next_value()?);
            }
            Ok(result)
        }
    }
    d.deserialize_map(V)
}
fn deliveries<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Delivery>, D::Error> {
    struct V;
    impl<'de> serde::de::Visitor<'de> for V {
        type Value = Vec<Delivery>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("up to256 directed NTP delivery fixtures")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut result = Vec::new();
            while let Some(item) = a.next_element::<Delivery>()? {
                if result.len() >= MAX_DELIVERIES {
                    return Err(serde::de::Error::custom("NTP delivery bound"));
                }
                result.push(item);
            }
            Ok(result)
        }
    }
    d.deserialize_seq(V)
}
fn label<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    struct V;
    impl serde::de::Visitor<'_> for V {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an ASCII node label up to32 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if !valid_label(s) {
                return Err(E::custom("Invalid bounded NTP node label"));
            }
            Ok(s.into())
        }
    }
    d.deserialize_str(V)
}
fn valid_label(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
}
fn valid_ip(ip: Ipv4Addr) -> bool {
    !ip.is_unspecified()
        && !ip.is_multicast()
        && !ip.is_broadcast()
        && !ip.is_loopback()
        && ip.octets()[0] != 0
        && ip.octets()[0] < 240
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub version: u32,
    #[serde(deserialize_with = "nodes")]
    pub nodes: BTreeMap<Ipv4Addr, Node>,
    #[serde(deserialize_with = "deliveries")]
    pub deliveries: Vec<Delivery>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    #[serde(deserialize_with = "label")]
    pub label: String,
    pub online: bool,
    pub role: Role,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Role {
    /// Supplied environment evidence, not a learner-settable clock or UTC proof.
    Reference {
        stratum: u8,
        healthy: bool,
    },
    Device {
        upstream: Option<Ipv4Addr>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Delivery {
    pub client: Ipv4Addr,
    pub server: Ipv4Addr,
    pub request_delivered: bool,
    pub reply_delivered: bool,
    pub udp123_allowed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FailureReason {
    Offline,
    NoUpstream,
    UnknownServer,
    RequestUnreachable,
    ReplyUnreachable,
    Udp123Blocked,
    SourceUnsynchronized,
    Cycle,
    StratumExhausted,
}
impl FailureReason {
    pub fn explanation(&self) -> &'static str {
        match self {
            Self::Offline => "A device in the configured source chain is offline",
            Self::NoUpstream => "No time source is configured",
            Self::UnknownServer => "The configured source is not present in this virtual topology",
            Self::RequestUnreachable => "The request cannot reach the configured source",
            Self::ReplyUnreachable => "The source reply cannot return to this client",
            Self::Udp123Blocked => "UDP port123 is blocked on this modeled exchange",
            Self::SourceUnsynchronized => "The supplied source is not synchronized",
            Self::Cycle => {
                "The configured source chain loops back to a device already in the chain"
            }
            Self::StratumExhausted => {
                "Using a stratum15 source would make this client stratum16, which is unsynchronized"
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Failure {
    pub reason: FailureReason,
    pub at: Ipv4Addr,
    pub chain: Vec<Ipv4Addr>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status {
    pub synchronized: bool,
    pub serving: bool,
    pub stratum: u8,
    /// Immediate selected upstream, never substituted with the distant root.
    pub reference: Option<Ipv4Addr>,
    /// Includes this node, all actual upstream dependencies, then supplied source.
    pub chain: Vec<Ipv4Addr>,
    pub failure: Option<Failure>,
}
impl State {
    pub fn validate(&self) -> Result<()> {
        if self.version != VERSION {
            return Err(err(ErrorKind::Invalid, "Unsupported NTP state version"));
        }
        if self.nodes.is_empty()
            || self.nodes.len() > MAX_NODES
            || self.deliveries.len() > MAX_DELIVERIES
        {
            return Err(err(ErrorKind::Limit, "NTP node/delivery bound"));
        }
        let mut labels = BTreeSet::new();
        for (ip, node) in &self.nodes {
            if !valid_ip(*ip) || !valid_label(&node.label) || !labels.insert(&node.label) {
                return Err(err(
                    ErrorKind::Invalid,
                    "Invalid or duplicate virtual node identity",
                ));
            }
            match &node.role {
                Role::Reference { stratum, .. } if !(1..=15).contains(stratum) => {
                    return Err(err(
                        ErrorKind::Invalid,
                        "Supplied reference stratum must be1..15",
                    ));
                }
                Role::Device { upstream: Some(ip) } if !valid_ip(*ip) => {
                    return Err(err(
                        ErrorKind::Invalid,
                        "Use a unicast non-loopback IPv4 upstream",
                    ));
                }
                _ => {}
            }
        }
        let mut pairs = BTreeSet::new();
        for d in &self.deliveries {
            if d.client == d.server
                || !self.nodes.contains_key(&d.client)
                || !self.nodes.contains_key(&d.server)
                || !pairs.insert((d.client, d.server))
            {
                return Err(err(
                    ErrorKind::Invalid,
                    "Delivery fixtures require unique directed known distinct node pairs",
                ));
            }
        }
        Ok(())
    }
    pub fn from_json(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_STATE_BYTES {
            return Err(err(ErrorKind::Limit, "NTP saved-state byte bound"));
        }
        let state: Self = serde_json::from_slice(bytes)
            .map_err(|e| err(ErrorKind::Invalid, format!("Invalid NTP saved state: {e}")))?;
        state.validate()?;
        Ok(state)
    }
    pub fn to_json(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let b = serde_json::to_vec(self).map_err(|e| err(ErrorKind::Invalid, e.to_string()))?;
        if b.len() > MAX_STATE_BYTES {
            return Err(err(ErrorKind::Limit, "NTP saved-state byte bound"));
        }
        Ok(b)
    }
    pub fn status(&self, ip: Ipv4Addr) -> Result<Status> {
        self.validate()?;
        if !self.nodes.contains_key(&ip) {
            return Err(err(ErrorKind::Unknown, "Unknown selected virtual node"));
        }
        Ok(self.resolve(ip, &mut Vec::new()))
    }
    pub fn statuses(&self) -> Result<BTreeMap<Ipv4Addr, Status>> {
        self.validate()?;
        Ok(self
            .nodes
            .keys()
            .map(|ip| (*ip, self.resolve(*ip, &mut Vec::new())))
            .collect())
    }
    fn failed(reason: FailureReason, at: Ipv4Addr, chain: Vec<Ipv4Addr>) -> Status {
        Status {
            synchronized: false,
            serving: false,
            stratum: 16,
            reference: None,
            chain: chain.clone(),
            failure: Some(Failure { reason, at, chain }),
        }
    }
    fn resolve(&self, ip: Ipv4Addr, path: &mut Vec<Ipv4Addr>) -> Status {
        // Validation caps all nodes at16; cycles are diagnosed before recursion.
        if path.contains(&ip) {
            let mut chain = path.clone();
            chain.push(ip);
            return Self::failed(FailureReason::Cycle, ip, chain);
        }
        debug_assert!(path.len() < MAX_NODES);
        path.push(ip);
        let node = &self.nodes[&ip];
        let result = if !node.online {
            Self::failed(FailureReason::Offline, ip, path.clone())
        } else {
            match &node.role {
                Role::Reference { stratum, healthy } => {
                    if *healthy {
                        Status {
                            synchronized: true,
                            serving: true,
                            stratum: *stratum,
                            reference: None,
                            chain: path.clone(),
                            failure: None,
                        }
                    } else {
                        Self::failed(FailureReason::SourceUnsynchronized, ip, path.clone())
                    }
                }
                Role::Device { upstream: None } => {
                    Self::failed(FailureReason::NoUpstream, ip, path.clone())
                }
                Role::Device {
                    upstream: Some(server),
                } => {
                    let mut endpoint = path.clone();
                    endpoint.push(*server);
                    if !self.nodes.contains_key(server) {
                        Self::failed(FailureReason::UnknownServer, *server, endpoint)
                    } else if server == &ip {
                        Self::failed(FailureReason::Cycle, ip, endpoint)
                    } else {
                        let delivery = self
                            .deliveries
                            .iter()
                            .find(|d| d.client == ip && d.server == *server);
                        let blocked = match delivery {
                            None => Some(FailureReason::RequestUnreachable),
                            Some(d) if !d.request_delivered => {
                                Some(FailureReason::RequestUnreachable)
                            }
                            Some(d) if !d.reply_delivered => Some(FailureReason::ReplyUnreachable),
                            Some(d) if !d.udp123_allowed => Some(FailureReason::Udp123Blocked),
                            _ => None,
                        };
                        if let Some(reason) = blocked {
                            Self::failed(reason, *server, endpoint)
                        } else {
                            let upstream = self.resolve(*server, path);
                            if !upstream.synchronized {
                                upstream
                            } else if upstream.stratum >= 15 {
                                Self::failed(
                                    FailureReason::StratumExhausted,
                                    *server,
                                    upstream.chain,
                                )
                            } else {
                                Status {
                                    synchronized: true,
                                    serving: true,
                                    stratum: upstream.stratum + 1,
                                    reference: Some(*server),
                                    chain: upstream.chain,
                                    failure: None,
                                }
                            }
                        }
                    }
                }
            }
        };
        path.pop();
        result
    }
    /// Only configuration changes; fixture health/delivery facts stay untouched.
    pub fn configure(&mut self, client: Ipv4Addr, upstream: Option<Ipv4Addr>) -> Result<()> {
        self.validate()?;
        if upstream.is_some_and(|ip| !valid_ip(ip)) {
            return Err(err(
                ErrorKind::Invalid,
                "Use a unicast non-loopback IPv4 upstream",
            ));
        }
        let node = self
            .nodes
            .get_mut(&client)
            .ok_or_else(|| err(ErrorKind::Unknown, "Unknown selected virtual node"))?;
        let Role::Device { upstream: current } = &mut node.role else {
            return Err(err(
                ErrorKind::Unsupported,
                "Supplied reference-source evidence is read-only",
            ));
        };
        *current = upstream;
        Ok(())
    }
    pub fn execute(&mut self, client: Ipv4Addr, input: &str) -> Result<String> {
        self.validate()?;
        if input.len() > 512 {
            return Err(err(ErrorKind::Limit, "NTP command byte bound"));
        }
        if input.chars().any(|c| c.is_control() && !matches!(c, '\t')) {
            return Err(err(
                ErrorKind::Unsupported,
                "One command per input; no scripts or control characters",
            ));
        }
        let words: Vec<_> = input.split_whitespace().collect();
        if !self.nodes.contains_key(&client) {
            return Err(err(ErrorKind::Unknown, "Unknown selected virtual node"));
        }
        match words.as_slice() {
            ["show", "ntp", "status"] => self.show_status(client),
            ["show", "ntp", "associations"] => self.show_associations(client),
            ["show", "running-config"] => Ok(match self.nodes[&client].role {
                Role::Device { upstream: Some(ip) } => {
                    format!("! Virtual NTP subset\nntp server {ip}\n")
                }
                _ => "! Virtual NTP subset: no client association\n".into(),
            }),
            ["ntp", "server", address] => {
                let ip = address
                    .parse::<Ipv4Addr>()
                    .map_err(|_| err(ErrorKind::Invalid, "Enter an IPv4 server address"))?;
                if !valid_ip(ip) {
                    return Err(err(
                        ErrorKind::Invalid,
                        "Use a unicast non-loopback IPv4 upstream",
                    ));
                }
                if let Role::Device {
                    upstream: Some(existing),
                } = self.nodes[&client].role
                    && existing != ip
                {
                    return Err(err(
                        ErrorKind::Unsupported,
                        "This subset supports one upstream. Remove the current server first; real IOS supports multiple associations.",
                    ));
                }
                self.configure(client, Some(ip))?;
                Ok("Virtual NTP client association configured; inspect status to verify the source.\n".into())
            }
            ["no", "ntp", "server", address] => {
                let ip = address
                    .parse::<Ipv4Addr>()
                    .map_err(|_| err(ErrorKind::Invalid, "Enter an IPv4 server address"))?;
                if !valid_ip(ip) {
                    return Err(err(
                        ErrorKind::Invalid,
                        "Use a unicast non-loopback IPv4 upstream",
                    ));
                }
                if let Role::Device {
                    upstream: Some(existing),
                } = self.nodes[&client].role
                {
                    if existing == ip {
                        self.configure(client, None)?;
                    }
                } else if matches!(self.nodes[&client].role, Role::Reference { .. }) {
                    return Err(err(
                        ErrorKind::Unsupported,
                        "Supplied reference-source evidence is read-only",
                    ));
                }
                Ok("Virtual matching association removed, if configured.\n".into())
            }
            _ => Err(err(ErrorKind::Unsupported, HELP)),
        }
    }
    fn show_status(&self, ip: Ipv4Addr) -> Result<String> {
        let s = self.status(ip)?;
        let reference = s.reference.map(|v| v.to_string()).unwrap_or_else(|| {
            if matches!(self.nodes[&ip].role, Role::Reference { .. }) && s.synchronized {
                "supplied reference evidence".into()
            } else {
                "none".into()
            }
        });
        let mut out = format!(
            "Virtual NTP dependency status (no measured clock)\nClock is {}, stratum {}, reference is {}\nServing downstream: {}\n",
            if s.synchronized {
                "synchronized in this model"
            } else {
                "unsynchronized"
            },
            s.stratum,
            reference,
            if s.serving { "Yes" } else { "No" }
        );
        if let Some(f) = s.failure {
            out.push_str(&format!(
                "Diagnosis: {} at {}.\n",
                f.reason.explanation(),
                f.at
            ));
        }
        out.push_str(&format!(
            "Source chain: {}\n",
            s.chain
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" -> ")
        ));
        Ok(out)
    }
    fn show_associations(&self, ip: Ipv4Addr) -> Result<String> {
        let mut out="Virtual NTP associations (no sampled reach/delay/offset fields)\naddress          st  selection\n".to_string();
        if let Role::Device {
            upstream: Some(server),
        } = self.nodes[&ip].role
        {
            let local = self.status(ip)?;
            let delivered = self
                .deliveries
                .iter()
                .find(|d| d.client == ip && d.server == server)
                .is_some_and(|d| d.request_delivered && d.reply_delivered && d.udp123_allowed);
            let remote = if delivered && self.nodes.get(&server).is_some_and(|n| n.online) {
                Some(self.status(server)?)
            } else {
                None
            };
            let st = remote
                .map(|s| s.stratum.to_string())
                .unwrap_or_else(|| "--".into());
            out.push_str(&format!(
                "{server:<16} {st:>2}  {}\n",
                if local.synchronized {
                    "* configured, synchronized source"
                } else {
                    "~ configured, not selected"
                }
            ));
        } else {
            out.push_str("(no configured client associations)\n");
        }
        Ok(out)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement {
    pub client: Ipv4Addr,
    pub allowed_references: BTreeSet<Ipv4Addr>,
    pub must_serve: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub client: Ipv4Addr,
    pub passed: bool,
    pub message: String,
}
/// Recomputed current-state goals, accepting any explicitly allowed upstream.
pub fn check(state: &State, requirements: &[Requirement]) -> Result<Vec<Check>> {
    state.validate()?;
    if requirements.is_empty() || requirements.len() > MAX_NODES {
        return Err(err(
            ErrorKind::Invalid,
            "Provide1..16 current-state NTP requirements",
        ));
    }
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for r in requirements {
        if !seen.insert(r.client)
            || r.allowed_references.is_empty()
            || r.allowed_references.len() > MAX_NODES
            || r.allowed_references
                .iter()
                .any(|ip| !state.nodes.contains_key(ip))
        {
            return Err(err(
                ErrorKind::Invalid,
                "Invalid, duplicate or foreign NTP goal identity",
            ));
        }
        let s = state.status(r.client)?;
        let allowed = s
            .reference
            .is_some_and(|ip| r.allowed_references.contains(&ip));
        let passed = s.synchronized && allowed && (!r.must_serve || s.serving);
        out.push(Check {
            client: r.client,
            passed,
            message: if passed {
                "Current source and synchronization requirements met.".into()
            } else if let Some(f) = s.failure {
                format!(
                    "{} at {}. Inspect the configured source chain and retry.",
                    f.reason.explanation(),
                    f.at
                )
            } else {
                "This device uses a different source than the assignment allows.".into()
            },
        });
    }
    Ok(out)
}

/// Restore/authoring boundary: learners may change device upstreams, not the
/// supplied identities, availability, reference evidence or delivery fixtures.
pub fn environment_matches(current: &State, initial: &State) -> Result<bool> {
    current.validate()?;
    initial.validate()?;
    if current.deliveries != initial.deliveries || current.nodes.len() != initial.nodes.len() {
        return Ok(false);
    }
    Ok(initial.nodes.iter().all(|(ip, expected)| {
        current.nodes.get(ip).is_some_and(|actual| {
            actual.label == expected.label
                && actual.online == expected.online
                && match (actual.role, expected.role) {
                    (Role::Device { .. }, Role::Device { .. }) => true,
                    (
                        Role::Reference {
                            stratum: a,
                            healthy: b,
                        },
                        Role::Reference {
                            stratum: c,
                            healthy: d,
                        },
                    ) => a == c && b == d,
                    _ => false,
                }
        })
    }))
}
/// Future persisted-attempt admission must retain the author's supplied world.
pub fn check_with_environment(
    current: &State,
    initial: &State,
    requirements: &[Requirement],
) -> Result<Vec<Check>> {
    if !environment_matches(current, initial)? {
        return Err(err(
            ErrorKind::Invalid,
            "The saved attempt changes supplied reference or network evidence; keep the original and recover configuration separately",
        ));
    }
    check(current, requirements)
}
