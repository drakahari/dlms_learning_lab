//! Finite offline OpenVPN2.6-style peer configuration; not a TLS implementation.
//! No file/socket/process API; credential fields are supplied facts, not X.509 verification.
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Class {
    Syntax,
    Unsupported,
    Configuration,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub class: Class,
    pub line: Option<usize>,
    pub message: String,
}
fn error(class: Class, message: impl Into<String>) -> Error {
    Error {
        class,
        line: None,
        message: message.into(),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
    Server,
    Client,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Network {
    pub address: Ipv4Addr,
    pub prefix: u8,
}
impl Network {
    pub fn validate(self) -> Result<()> {
        if self.prefix > 32 {
            return Err(error(
                Class::Configuration,
                "An IPv4 route prefix must be between 0 and 32.",
            ));
        }
        if !(crate::linux_filter::Cidr {
            network: self.address,
            prefix: self.prefix,
        })
        .valid()
        {
            return Err(error(
                Class::Configuration,
                "Use the network boundary in a route, not an address inside it.",
            ));
        }
        Ok(())
    }
    pub fn interval(self) -> Result<(u32, u32)> {
        self.validate()?;
        let start = u32::from(self.address);
        // General routes include /31 and /32; the introductory host exercise
        // intentionally has a narrower prefix domain and is not reused here.
        let size = 1_u64 << (32 - self.prefix);
        Ok((start, (u64::from(start) + size - 1) as u32))
    }
    pub fn contains(self, address: Ipv4Addr) -> bool {
        self.interval()
            .is_ok_and(|(a, b)| (a..=b).contains(&u32::from(address)))
    }
}
fn ip(s: &str) -> Result<Ipv4Addr> {
    s.parse().map_err(|_| {
        error(
            Class::Syntax,
            "Use a dotted-decimal IPv4 address; names and IPv6 are outside this subset.",
        )
    })
}
fn unicast(a: Ipv4Addr) -> bool {
    !a.is_unspecified()
        && !a.is_loopback()
        && !a.is_multicast()
        && !a.is_broadcast()
        && a.octets()[0] < 224
}
fn port(s: &str) -> Result<u16> {
    s.parse::<u16>()
        .ok()
        .filter(|p| *p > 0)
        .ok_or_else(|| error(Class::Syntax, "Use a numeric port from 1 to 65535."))
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s != "."
        && s != ".."
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn virtual_path(s: &str) -> bool {
    s.starts_with("/etc/openvpn/") && s.len() <= 160 && s.split('/').skip(1).all(name)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Remote {
    pub address: Ipv4Addr,
    pub port: Option<u16>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub role: Role,
    pub port: u16,
    pub remote: Option<Remote>,
    pub local_tunnel: Ipv4Addr,
    pub remote_tunnel: Ipv4Addr,
    pub ca: String,
    pub cert: String,
    pub key: String,
    pub expected_role: Option<Role>,
    pub expected_name: Option<String>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Network,32>")]
    pub routes: Vec<Network>,
}
impl Config {
    pub fn validate(&self) -> Result<()> {
        if self.port == 0
            || self.role == Role::Client && self.remote.is_none()
            || !unicast(self.local_tunnel)
            || !unicast(self.remote_tunnel)
            || self.local_tunnel == self.remote_tunnel
            || [&self.ca, &self.cert, &self.key]
                .iter()
                .any(|p| !virtual_path(p))
            || self.expected_name.as_ref().is_some_and(|n| !name(n))
            || self
                .remote
                .as_ref()
                .is_some_and(|r| !unicast(r.address) || r.port == Some(0))
            || self.routes.len() > 32
        {
            return Err(error(
                Class::Configuration,
                "Peer values exceed the documented configuration subset.",
            ));
        }
        let mut seen = vec![];
        for n in &self.routes {
            n.validate()?;
            if seen.contains(n) {
                return Err(error(
                    Class::Unsupported,
                    "Duplicate routes are outside the supported subset.",
                ));
            }
            seen.push(*n);
        }
        Ok(())
    }
}
pub fn parse(text: &str) -> Result<Config> {
    if text.len() > 8192 || text.lines().count() > 128 {
        return Err(error(
            Class::Limit,
            "Keep this peer configuration within 8 KiB and 128 lines.",
        ));
    }
    if !text.is_ascii()
        || text
            .bytes()
            .any(|b| b == 0 || b.is_ascii_control() && !b"\r\n\t".contains(&b))
    {
        return Err(error(
            Class::Syntax,
            "Use ordinary ASCII configuration text without control characters.",
        ));
    }
    let (
        mut role,
        mut local_port,
        mut remote,
        mut tunnel,
        mut ca,
        mut cert,
        mut key,
        mut expected_role,
        mut expected_name,
    ) = (None, None, None, None, None, None, None, None, None);
    let (mut dev, mut proto) = (false, false);
    let mut routes = vec![];
    let mut seen = std::collections::BTreeSet::new();
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        let parsed = (|| {
            if line.bytes().any(|b| b"'\"\\#;".contains(&b)) {
                return Err(error(
                    Class::Unsupported,
                    "Quoted parameters, escapes and inline comments are outside this finite subset. Use a whole-line comment and plain parameters.",
                ));
            }
            let w: Vec<_> = line.split_whitespace().collect();
            let verb = w[0];
            let a = &w[1..];
            let supported = matches!(
                verb,
                "dev"
                    | "proto"
                    | "port"
                    | "remote"
                    | "tls-server"
                    | "tls-client"
                    | "ifconfig"
                    | "ca"
                    | "cert"
                    | "key"
                    | "remote-cert-tls"
                    | "verify-x509-name"
                    | "route"
            );
            if !supported {
                return Err(error(
                    Class::Unsupported,
                    format!(
                        "{verb} is not implemented. No directive, include, script or plugin is executed."
                    ),
                ));
            }
            if verb != "route" && !seen.insert(verb.to_string()) {
                return Err(error(
                    Class::Unsupported,
                    "Repeated scalar directives are outside this subset; keep one explicit value.",
                ));
            }
            let arity = |n: usize| -> Result<()> {
                if a.len() != n {
                    Err(error(
                        Class::Syntax,
                        format!("{verb} needs {n} parameter(s)."),
                    ))
                } else {
                    Ok(())
                }
            };
            match verb {
                "dev" => {
                    arity(1)?;
                    if a[0] != "tun" {
                        return Err(error(
                            Class::Unsupported,
                            "Only the supplied TUN point-to-point device is modeled; no TAP/device is created.",
                        ));
                    }
                    dev = true;
                }
                "proto" => {
                    arity(1)?;
                    if !matches!(a[0], "udp" | "udp4") {
                        return Err(error(
                            Class::Unsupported,
                            "Only numeric IPv4 UDP transport is modeled; TCP and IPv6 transport are unsupported.",
                        ));
                    }
                    proto = true;
                }
                "port" => {
                    arity(1)?;
                    local_port = Some(port(a[0])?);
                }
                "remote" => {
                    if !(1..=2).contains(&a.len()) {
                        return Err(error(Class::Syntax, "Use remote IPv4 [port]."));
                    }
                    let address = ip(a[0])?;
                    if !unicast(address) {
                        return Err(error(
                            Class::Configuration,
                            "Use the supplied unicast peer address.",
                        ));
                    }
                    remote = Some(Remote {
                        address,
                        port: if a.len() == 2 {
                            Some(port(a[1])?)
                        } else {
                            None
                        },
                    });
                }
                "tls-server" | "tls-client" => {
                    arity(0)?;
                    if role.is_some() {
                        return Err(error(
                            Class::Configuration,
                            "A peer must have one TLS role, not both.",
                        ));
                    }
                    role = Some(if verb == "tls-server" {
                        Role::Server
                    } else {
                        Role::Client
                    });
                }
                "ifconfig" => {
                    arity(2)?;
                    let (l, r) = (ip(a[0])?, ip(a[1])?);
                    if !unicast(l) || !unicast(r) || l == r {
                        return Err(error(
                            Class::Configuration,
                            "Use distinct supplied unicast local and remote tunnel addresses.",
                        ));
                    }
                    tunnel = Some((l, r));
                }
                "ca" | "cert" | "key" => {
                    arity(1)?;
                    if !virtual_path(a[0]) {
                        return Err(error(
                            Class::Unsupported,
                            "Choose a supplied virtual file directly under /etc/openvpn/ or its named subdirectories. No host path is read.",
                        ));
                    }
                    match verb {
                        "ca" => ca = Some(a[0].to_string()),
                        "cert" => cert = Some(a[0].to_string()),
                        _ => key = Some(a[0].to_string()),
                    }
                }
                "remote-cert-tls" => {
                    arity(1)?;
                    expected_role = Some(match a[0] {
                        "server" => Role::Server,
                        "client" => Role::Client,
                        _ => {
                            return Err(error(
                                Class::Syntax,
                                "Use remote-cert-tls server or client.",
                            ));
                        }
                    });
                }
                "verify-x509-name" => {
                    arity(2)?;
                    if a[1] != "name" || !name(a[0]) {
                        return Err(error(
                            Class::Unsupported,
                            "Only an exact supplied common name with verify-x509-name NAME name is supported.",
                        ));
                    }
                    expected_name = Some(a[0].to_string());
                }
                "route" => {
                    arity(2)?;
                    let address = ip(a[0])?;
                    let mask = u32::from(ip(a[1])?);
                    let prefix = mask.leading_ones() as u8;
                    if mask.count_ones() != u32::from(prefix) {
                        return Err(error(
                            Class::Syntax,
                            "The route mask must have contiguous network bits.",
                        ));
                    }
                    let n = Network { address, prefix };
                    n.validate()?;
                    if routes.contains(&n) {
                        return Err(error(
                            Class::Unsupported,
                            "Duplicate route directives are outside this subset; keep one explicit route.",
                        ));
                    }
                    if routes.len() == 32 {
                        return Err(error(Class::Limit, "Use at most 32 explicit routes."));
                    }
                    routes.push(n);
                }
                _ => unreachable!(),
            }
            Ok(())
        })();
        if let Err(mut e) = parsed {
            e.line = Some(number + 1);
            return Err(e);
        }
    }
    let need = |label: &str| {
        error(
            Class::Configuration,
            format!("Add the required {label} directive; review the supplied peer settings."),
        )
    };
    if !dev || !proto {
        return Err(need(" dev tun and proto udp "));
    }
    let role = role.ok_or_else(|| need(" TLS role "))?;
    let (local_tunnel, remote_tunnel) = tunnel.ok_or_else(|| need(" ifconfig "))?;
    if role == Role::Client && remote.is_none() {
        return Err(need(" remote "));
    }
    Ok(Config {
        role,
        port: local_port.unwrap_or(1194),
        remote,
        local_tunnel,
        remote_tunnel,
        ca: ca.ok_or_else(|| need(" ca "))?,
        cert: cert.ok_or_else(|| need(" cert "))?,
        key: key.ok_or_else(|| need(" key "))?,
        expected_role,
        expected_name,
        routes,
    })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Material {
    Ca {
        id: String,
    },
    Certificate {
        subject: String,
        issuer: String,
        key_id: String,
        role: Role,
        valid: bool,
    },
    Key {
        key_id: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VirtualFile {
    pub path: String,
    pub material: Material,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Peer {
    pub label: String,
    pub endpoint: Ipv4Addr,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Supplied {
    pub peers: [Peer; 2],
    pub underlay_available: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,VirtualFile,16>")]
    pub files: Vec<VirtualFile>,
}
impl Supplied {
    pub fn validate(&self) -> Result<()> {
        if self.peers[0].endpoint == self.peers[1].endpoint
            || self
                .peers
                .iter()
                .any(|p| !unicast(p.endpoint) || !name(&p.label))
            || self.files.is_empty()
            || self.files.len() > 16
        {
            return Err(error(
                Class::Configuration,
                "Supply two distinct named IPv4 peers and a bounded virtual credential roster.",
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        for f in &self.files {
            if !virtual_path(&f.path) || !seen.insert(&f.path) {
                return Err(error(
                    Class::Configuration,
                    "Virtual credential paths must be unique ordinary supplied names.",
                ));
            }
            let valid = match &f.material {
                Material::Ca { id } => name(id),
                Material::Certificate {
                    subject,
                    issuer,
                    key_id,
                    ..
                } => name(subject) && name(issuer) && name(key_id),
                Material::Key { key_id } => name(key_id),
            };
            if !valid {
                return Err(error(
                    Class::Configuration,
                    "Keep supplied identity facts bounded and explicitly named.",
                ));
            }
        }
        Ok(())
    }
    fn material(&self, path: &str) -> Result<&Material> {
        self.files.iter().find(|f|f.path==path).map(|f|&f.material).ok_or_else(||error(Class::Configuration,format!("{path} is not a supplied virtual credential file; no host lookup is performed.")))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PairCheck {
    pub label: String,
    pub passed: bool,
    pub evaluated: bool,
    pub detail: String,
}
fn check(
    v: &mut Vec<PairCheck>,
    label: impl Into<String>,
    passed: bool,
    detail: impl Into<String>,
) {
    v.push(PairCheck {
        label: label.into(),
        passed,
        evaluated: true,
        detail: detail.into(),
    });
}
pub fn peer_checks(s: &Supplied, c: &[Config; 2]) -> Result<Vec<PairCheck>> {
    s.validate()?;
    for peer in c {
        peer.validate()?;
    }
    let mut checks = vec![];
    check(
        &mut checks,
        "Supplied underlay",
        s.underlay_available,
        "The outside path is supplied evidence, not a discovered or tested network.",
    );
    check(
        &mut checks,
        "Complementary TLS roles",
        c[0].role != c[1].role,
        "The two configured roles must be server and client.",
    );
    check(
        &mut checks,
        "Reciprocal tunnel addresses",
        c[0].local_tunnel == c[1].remote_tunnel && c[1].local_tunnel == c[0].remote_tunnel,
        "Each peer's local tunnel address must be the other peer's remote tunnel address.",
    );
    for i in 0..2 {
        let other = 1 - i;
        let own = &c[i];
        let peer = &c[other];
        let label = &s.peers[i].label;
        let remote_ok = own.remote.as_ref().is_none_or(|r| {
            r.address == s.peers[other].endpoint && r.port.unwrap_or(own.port) == peer.port
        });
        check(
            &mut checks,
            format!("{label}: remote endpoint/port"),
            remote_ok,
            "The remote address and destination port must select the supplied other peer's configured listener.",
        );
        let Material::Ca { id: trusted } = s.material(&own.ca)? else {
            return Err(error(
                Class::Configuration,
                "ca must select a supplied authority record.",
            ));
        };
        let Material::Certificate { key_id, .. } = s.material(&own.cert)? else {
            return Err(error(
                Class::Configuration,
                "cert must select a supplied certificate record.",
            ));
        };
        let Material::Key { key_id: private } = s.material(&own.key)? else {
            return Err(error(
                Class::Configuration,
                "key must select a supplied key record.",
            ));
        };
        check(
            &mut checks,
            format!("{label}: own certificate/key"),
            key_id == private,
            "The supplied own certificate and private-key metadata must refer to the same key pair.",
        );
        let Material::Certificate {
            subject,
            issuer,
            role,
            valid,
            ..
        } = s.material(&peer.cert)?
        else {
            return Err(error(
                Class::Configuration,
                "The other cert must select a supplied certificate record.",
            ));
        };
        check(
            &mut checks,
            format!("{label}: supplied peer validity and authority"),
            *valid && issuer == trusted,
            "The peer must have a supplied valid certificate fact under the selected authority. This is metadata evaluation, not signature or clock verification.",
        );
        if let Some(expected) = own.expected_role {
            check(
                &mut checks,
                format!("{label}: requested peer purpose"),
                expected == *role,
                "The requested remote-cert-tls purpose must match the supplied peer certificate fact.",
            );
        } else {
            checks.push(PairCheck{label:format!("{label}: peer purpose not requested"),passed:false,evaluated:false,detail:"No remote-cert-tls check is configured. This is not evidence of a verified peer purpose.".into()});
        }
        if let Some(expected) = &own.expected_name {
            check(
                &mut checks,
                format!("{label}: requested peer name"),
                expected == subject,
                "The exact requested peer name must match the supplied certificate fact. Authority and identity are different checks.",
            );
        } else {
            checks.push(PairCheck{label:format!("{label}: peer name not requested"),passed:false,evaluated:false,detail:"No exact peer-name check is configured. This is not evidence of a verified peer identity.".into()});
        }
    }
    Ok(checks)
}
/// Normalize the exact union redirected into a point-to-point tunnel. This is not
/// a host route table, route injection or DCO; every supported route uses this peer.
pub fn route_intervals(c: &Config) -> Result<Vec<(u32, u32)>> {
    let mut rows = c
        .routes
        .iter()
        .copied()
        .map(Network::interval)
        .collect::<Result<Vec<_>>>()?;
    rows.sort_unstable();
    let mut out: Vec<(u32, u32)> = vec![];
    for (a, b) in rows {
        if let Some(last) = out.last_mut()
            && u64::from(a) <= u64::from(last.1) + 1
        {
            last.1 = last.1.max(b);
            continue;
        }
        out.push((a, b));
    }
    Ok(out)
}
pub fn equivalent_routes(c: &Config, required: &[Network]) -> Result<bool> {
    if required.is_empty() || required.len() > 32 {
        return Err(error(
            Class::Configuration,
            "Use a bounded explicit required destination set.",
        ));
    }
    let mut expected = c.clone();
    expected.routes = required.to_vec();
    Ok(route_intervals(c)? == route_intervals(&expected)?)
}
