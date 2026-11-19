//! Original bounded keyfile parser; no daemon, host files or network access.
use crate::ipv4_routing::{Assignment, Prefix};
use std::{collections::BTreeMap, net::Ipv4Addr};
pub const MAX_TEXT: usize = 16384;
pub const MAX_LINES: usize = 256;
pub const MAX_PROFILES: usize = 8;
pub const MAX_ROUTES: usize = 8;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Syntax,
    Unsupported,
    Invalid,
    Limit,
}
impl ErrorKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Syntax => "Syntax error",
            Self::Unsupported => "Unsupported in this simulator",
            Self::Invalid => "Invalid profile",
            Self::Limit => "Simulator limit",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub line: usize,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.line == 0 {
            write!(f, "{}: {}", self.kind.label(), self.message)
        } else {
            write!(
                f,
                "{} at line {}: {}",
                self.kind.label(),
                self.line,
                self.message
            )
        }
    }
}
fn error(kind: ErrorKind, line: usize, message: impl Into<String>) -> Error {
    Error {
        kind,
        line,
        message: message.into(),
    }
}
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    pub destination: Prefix,
    pub via: Ipv4Addr,
    pub metric: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vlan {
    pub parent: String,
    pub id: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub vlan: Option<Vlan>,
    pub id: String,
    pub uuid: String,
    pub interface: String,
    pub autoconnect: bool,
    pub priority: i16,
    pub address: Assignment,
    pub gateway: Option<Ipv4Addr>,
    pub route_metric: Option<u32>,
    pub never_default: bool,
    pub routes: Vec<Route>,
}
/// Saved configuration is distinct from an active, resolved Profile.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Addressing {
    Manual {
        address: Assignment,
        gateway: Option<Ipv4Addr>,
    },
    Automatic,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub vlan: Option<Vlan>,
    pub id: String,
    pub uuid: String,
    pub interface: String,
    pub autoconnect: bool,
    pub priority: i16,
    pub addressing: Addressing,
    pub route_metric: Option<u32>,
    pub never_default: bool,
    pub routes: Vec<Route>,
}
impl Settings {
    pub(crate) fn resolved(&self, address: Assignment, gateway: Option<Ipv4Addr>) -> Profile {
        Profile {
            vlan: self.vlan.clone(),
            id: self.id.clone(),
            uuid: self.uuid.clone(),
            interface: self.interface.clone(),
            autoconnect: self.autoconnect,
            priority: self.priority,
            address,
            gateway: if self.never_default { None } else { gateway },
            route_metric: self.route_metric,
            never_default: self.never_default,
            routes: self.routes.clone(),
        }
    }
}
/// Original static-only API; callers must explicitly opt into the wider settings model.
pub fn parse(text: &str) -> Result<Profile> {
    let settings = parse_settings(text)?;
    match &settings.addressing {
        Addressing::Manual { address, gateway } => Ok(settings.resolved(address.clone(), *gateway)),
        Addressing::Automatic => Err(error(
            ErrorKind::Unsupported,
            0,
            "This static parser does not resolve DHCP; use the supplied-lease workspace",
        )),
    }
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(n, b)| {
            if [8, 13, 18, 23].contains(&n) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
fn iface(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 15
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}
fn ip(s: &str, line: usize) -> Result<Ipv4Addr> {
    s.parse().map_err(|_| {
        error(
            ErrorKind::Invalid,
            line,
            "Use a dotted-decimal IPv4 address",
        )
    })
}
fn assignment(s: &str, line: usize) -> Result<Assignment> {
    let (address, prefix) = s.split_once('/').ok_or_else(|| {
        error(
            ErrorKind::Unsupported,
            line,
            "Use an explicit IPv4 address/prefix in this subset",
        )
    })?;
    let prefix = prefix
        .parse::<u8>()
        .map_err(|_| error(ErrorKind::Invalid, line, "Use a decimal IPv4 prefix"))?;
    if !(1..=30).contains(&prefix) {
        return Err(error(
            ErrorKind::Unsupported,
            line,
            "Conventional /1 through /30 interface addresses only; /31 and /32 are special cases outside this exercise",
        ));
    }
    let a = Assignment {
        address: ip(address, line)?,
        prefix,
    };
    if !crate::ipv4_routing::supported_address(a.address) {
        return Err(error(
            ErrorKind::Unsupported,
            line,
            "Only private/documentation addresses are modeled in this exercise; no real target is contacted",
        ));
    }
    if !a.usable() {
        return Err(error(
            ErrorKind::Invalid,
            line,
            "The configured interface address is the network or broadcast address",
        ));
    }
    Ok(a)
}
fn number(s: &str, line: usize) -> Result<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(error(
            ErrorKind::Invalid,
            line,
            "Use an unsigned decimal metric",
        ));
    }
    s.parse().map_err(|_| {
        error(
            ErrorKind::Limit,
            line,
            "Metric exceeds the unsigned 32-bit bound",
        )
    })
}
fn metric(s: &str, line: usize) -> Result<Option<u32>> {
    if s == "-1" {
        Ok(None)
    } else {
        number(s, line).map(Some)
    }
}
fn boolean(s: &str, line: usize) -> Result<bool> {
    match s {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(error(
            ErrorKind::Unsupported,
            line,
            "This subset accepts the keyfile boolean forms true and false",
        )),
    }
}
fn route(s: &str, line: usize) -> Result<Route> {
    let fields = s.split(',').map(str::trim).collect::<Vec<_>>();
    if !(2..=3).contains(&fields.len()) || fields[1].is_empty() {
        return Err(error(
            ErrorKind::Unsupported,
            line,
            "Use routeN=network/prefix,gateway[,metric]; direct routes/options are outside this subset",
        ));
    }
    let (a, p) = fields[0].split_once('/').ok_or_else(|| {
        error(
            ErrorKind::Unsupported,
            line,
            "Give a route's explicit destination prefix",
        )
    })?;
    let p = p
        .parse::<u8>()
        .map_err(|_| error(ErrorKind::Invalid, line, "Invalid route prefix"))?;
    let destination = Prefix::new(ip(a, line)?, p).ok_or_else(|| {
        error(
            ErrorKind::Invalid,
            line,
            "Route prefix must be /0 through /32",
        )
    })?;
    let via = ip(fields[1], line)?;
    let metric = if fields.len() == 3 && !fields[2].is_empty() {
        Some(number(fields[2], line)?)
    } else {
        None
    };
    Ok(Route {
        destination,
        via,
        metric,
    })
}
/// Saved bridge controller properties; no daemon or host activation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BridgeSettings {
    pub ipv4: Settings,
    pub stp: bool,
    pub autoconnect_ports: bool,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum ParseMode {
    Ethernet,
    Vlan,
    Bridge,
}
#[derive(Clone, Debug)]
struct BridgeProperties {
    stp: bool,
    autoconnect_ports: bool,
}
/// Parse one original keyfile. Unknown syntax is refused, never executed/ignored.
pub fn parse_settings(text: &str) -> Result<Settings> {
    parse_settings_mode(text, ParseMode::Ethernet).map(|(settings, _)| settings)
}
/// Explicit capability API; ordinary static/DHCP Ethernet parsing stays strict.
pub fn parse_vlan_settings(text: &str) -> Result<Settings> {
    parse_settings_mode(text, ParseMode::Vlan).map(|(settings, _)| settings)
}
/// Source-enabled bridge controller only; ports and activation are separate.
pub fn parse_bridge_settings(text: &str) -> Result<BridgeSettings> {
    let (ipv4, properties) = parse_settings_mode(text, ParseMode::Bridge)?;
    let properties = properties.ok_or_else(|| {
        error(
            ErrorKind::Invalid,
            0,
            "Expected bridge controller properties",
        )
    })?;
    Ok(BridgeSettings {
        ipv4,
        stp: properties.stp,
        autoconnect_ports: properties.autoconnect_ports,
    })
}
fn parse_settings_mode(
    text: &str,
    mode: ParseMode,
) -> Result<(Settings, Option<BridgeProperties>)> {
    let allow_vlan = mode == ParseMode::Vlan;
    let allow_bridge = mode == ParseMode::Bridge;
    if text.len() > MAX_TEXT {
        return Err(error(ErrorKind::Limit, 0, "Profile exceeds16KiB"));
    }
    if !text.is_ascii() {
        return Err(error(
            ErrorKind::Unsupported,
            0,
            "ASCII keyfiles only in this exercise; previous draft retained",
        ));
    }
    if text
        .bytes()
        .any(|b| b.is_ascii_control() && !b"\n\r\t".contains(&b))
    {
        return Err(error(
            ErrorKind::Syntax,
            0,
            "Control characters are not accepted, including in comments",
        ));
    }
    let mut group = "";
    let mut values = BTreeMap::new();
    for (index, raw) in text.lines().enumerate() {
        let line = index + 1;
        if line > MAX_LINES {
            return Err(error(ErrorKind::Limit, line, "Profile line limit exceeded"));
        }
        let row = raw.trim();
        if row.is_empty() || row.starts_with('#') {
            continue;
        }
        if row.starts_with('[') {
            let Some(name) = row.strip_prefix('[').and_then(|r| r.strip_suffix(']')) else {
                return Err(error(
                    ErrorKind::Syntax,
                    line,
                    "Close the section header with ]",
                ));
            };
            if ![
                "connection",
                "ipv4",
                "ipv6",
                "ethernet",
                "802-3-ethernet",
                "proxy",
            ]
            .contains(&name)
                && !(allow_vlan && name == "vlan")
                && !(allow_bridge && name == "bridge")
            {
                return Err(error(
                    ErrorKind::Unsupported,
                    line,
                    if allow_bridge {
                        "Only bridge controller, manual IPv4, disabled IPv6 and empty Ethernet/proxy sections are supported"
                    } else if allow_vlan {
                        "Only connection, VLAN, IPv4, disabled IPv6 and empty Ethernet/proxy sections are supported"
                    } else {
                        "Only connection, IPv4, disabled IPv6 and empty Ethernet/proxy sections are supported"
                    },
                ));
            }
            group = name;
            continue;
        }
        let (key, value) = row.split_once('=').ok_or_else(|| {
            error(
                ErrorKind::Syntax,
                line,
                "Use key=value under a supported section",
            )
        })?;
        let key = key.trim();
        let value = value.trim();
        if key.len() > 48 || value.len() > 256 {
            return Err(error(
                ErrorKind::Limit,
                line,
                "Profile property length limit exceeded",
            ));
        }
        if group.is_empty() {
            return Err(error(
                ErrorKind::Syntax,
                line,
                "Add a section header before its settings",
            ));
        }
        let known = match group {
            "connection" => {
                [
                    "id",
                    "uuid",
                    "type",
                    "interface-name",
                    "autoconnect",
                    "autoconnect-priority",
                ]
                .contains(&key)
                    || (allow_bridge && key == "autoconnect-ports")
            }
            "ipv4" => {
                [
                    "method",
                    "address1",
                    "gateway",
                    "route-metric",
                    "never-default",
                ]
                .contains(&key)
                    || [
                        "route1", "route2", "route3", "route4", "route5", "route6", "route7",
                        "route8",
                    ]
                    .contains(&key)
            }
            "vlan" => allow_vlan && ["id", "parent", "flags", "protocol"].contains(&key),
            "bridge" => allow_bridge && matches!(key, "stp" | "vlan-filtering"),
            "ipv6" => key == "method" || (allow_bridge && key == "addr-gen-mode"),
            _ => false,
        };
        if !known {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                format!("[{group}] {key} is outside the declared keyfile subset"),
            ));
        }
        if value.contains('\\') {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                "Escaped/keyfile continuation values are outside this subset",
            ));
        }
        // GLib-style repeated groups/keys retain the last assignment, not first.
        values.insert((group, key), (value, line));
    }
    let get = |g, k| values.get(&(g, k)).copied();
    let required = |g, k| -> Result<(&str, usize)> {
        get(g,k).ok_or_else(||error(ErrorKind::Unsupported,0,format!("This exercise requires [{g}] {k}; implicit generated profile identity/addressing is not modeled")))
    };
    let (id, line) = required("connection", "id")?;
    if id.is_empty() || id.len() > 64 {
        return Err(error(
            ErrorKind::Invalid,
            line,
            "Give the profile a nonempty name up to64ASCII bytes",
        ));
    }
    let (uid, line) = required("connection", "uuid")?;
    if !uuid(uid) {
        return Err(error(
            ErrorKind::Invalid,
            line,
            "Preserve a UUID in the supplied 8-4-4-4-12 hexadecimal format",
        ));
    }
    let (ty, line) = required("connection", "type")?;
    let supported = if allow_bridge {
        ty == "bridge"
    } else {
        ["ethernet", "802-3-ethernet"].contains(&ty) || (allow_vlan && ty == "vlan")
    };
    if !supported {
        return Err(error(
            ErrorKind::Unsupported,
            line,
            if allow_bridge {
                "This explicit parser needs a bridge controller; Ethernet port/VLAN/nested controller configuration is separate"
            } else if allow_vlan {
                "This parser supports Ethernet or VLAN profiles; bonding, bridging and Wi-Fi are outside this activity"
            } else {
                "Only Ethernet profiles are simulated; Wi-Fi and other types remain outside this unit"
            },
        ));
    }
    let bridge = if allow_bridge {
        let (ports, line) = required("connection", "autoconnect-ports")?;
        let autoconnect_ports = match ports {
            "1" => true,
            "0" => false,
            _ => {
                return Err(error(
                    ErrorKind::Unsupported,
                    line,
                    "Choose explicit autoconnect-ports=0 or1; inherited global defaults are outside this subset",
                ));
            }
        };
        let stp = get("bridge", "stp")
            .map(|(v, l)| boolean(v, l))
            .transpose()?
            .unwrap_or(true);
        if let Some((v, l)) = get("bridge", "vlan-filtering")
            && boolean(v, l)?
        {
            return Err(error(
                ErrorKind::Unsupported,
                l,
                "VLAN-filtering bridges are outside this untagged acyclic activity",
            ));
        }
        if let Some((v, l)) = get("ipv6", "addr-gen-mode")
            && v != "default"
        {
            return Err(error(
                ErrorKind::Unsupported,
                l,
                "Only omitted/default addr-gen-mode with disabled IPv6 is supported",
            ));
        }
        Some(BridgeProperties {
            stp,
            autoconnect_ports,
        })
    } else {
        None
    };
    let vlan = if ty == "vlan" {
        let (parent, line) = required("vlan", "parent")?;
        if !iface(parent) {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                "Use a supplied literal parent interface name; parent UUIDs and discovery are outside this activity",
            ));
        }
        let (id, line) = required("vlan", "id")?;
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
            return Err(error(
                ErrorKind::Invalid,
                line,
                "VLAN id needs a decimal number, not an IP octet or interface position",
            ));
        }
        let id = id.parse::<u16>().map_err(|_| {
            error(
                ErrorKind::Invalid,
                line,
                "VLAN ID exceeds the supported data range",
            )
        })?;
        if id == 0 {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                "Priority tag0 is real but outside this data-VLAN activity; use a supplied ID1–4094",
            ));
        }
        if id > 4094 {
            return Err(error(
                ErrorKind::Invalid,
                line,
                "Data VLAN IDs end at4094;4095 is reserved",
            ));
        }
        let (flags, line) = required("vlan", "flags")?;
        if flags != "1" {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                "Retain the supplied flags=1 header-reordering mode; other flags/defaults are outside this activity",
            ));
        }
        if let Some((protocol, line)) = get("vlan", "protocol")
            && protocol != "802.1Q"
        {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                "Only single802.1Q tags are modeled;802.1ad/nested tags are outside this activity",
            ));
        }
        Some(Vlan {
            parent: parent.into(),
            id,
        })
    } else {
        if values.keys().any(|(group, _)| *group == "vlan") {
            return Err(error(
                ErrorKind::Unsupported,
                0,
                "VLAN properties require connection.type=vlan; they are not silently ignored on Ethernet",
            ));
        }
        None
    };
    let (interface, line) = required("connection", "interface-name")?;
    if !iface(interface) {
        return Err(error(
            ErrorKind::Unsupported,
            line,
            "Use one literal interface name of at most15ASCII letters/digits/_/./-; interface patterns are outside this subset",
        ));
    }
    let (method, line) = required("ipv4", "method")?;
    if allow_bridge && method != "manual" {
        return Err(error(
            ErrorKind::Unsupported,
            line,
            "This bridge foundation requires one manual IPv4 address; DHCP/shared/link-local are outside scope",
        ));
    }

    if !matches!(method, "manual" | "auto") {
        return Err(error(
            ErrorKind::Unsupported,
            line,
            "Only manual or supplied-lease auto IPv4 is modeled; link-local/shared behavior is separate",
        ));
    }
    let (v6, line) = required("ipv6", "method")?;
    if v6 != "disabled" {
        return Err(error(
            ErrorKind::Unsupported,
            line,
            "This isolated IPv4 exercise needs its supplied IPv6 method=disabled; IPv6 auto/manual behavior is not simulated",
        ));
    }
    let addressing = if method == "auto" {
        if let Some((_, line)) = get("ipv4", "address1").or_else(|| get("ipv4", "gateway")) {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                "Automatic IPv4 with additional static addresses or a gateway override is outside this subset; remove those properties to use only the supplied DHCP outcome",
            ));
        }
        Addressing::Automatic
    } else {
        let (address, line) = required("ipv4", "address1")?;
        let fields = address.split([',', ';']).map(str::trim).collect::<Vec<_>>();
        if fields.len() > 2 {
            return Err(error(
                ErrorKind::Unsupported,
                line,
                "One primary address with an optional legacy gateway only",
            ));
        }
        let address = assignment(fields[0], line)?;
        let inline = if fields.len() == 2 && !fields[1].is_empty() {
            Some(ip(fields[1], line)?)
        } else {
            None
        };
        let gateway = match get("ipv4", "gateway") {
            Some(("", l)) => {
                return Err(error(
                    ErrorKind::Invalid,
                    l,
                    "An empty gateway value is not a valid keyfile gateway; omit it or use 0.0.0.0 to clear it",
                ));
            }
            Some((s, l)) => Some(ip(s, l)?),
            None => inline,
        };
        let gateway = gateway.filter(|ip| *ip != Ipv4Addr::UNSPECIFIED);
        Addressing::Manual { address, gateway }
    };
    let route_metric = get("ipv4", "route-metric")
        .map(|(v, l)| metric(v, l))
        .transpose()?
        .flatten();
    let autoconnect = get("connection", "autoconnect")
        .map(|(v, l)| boolean(v, l))
        .transpose()?
        .unwrap_or(true);
    let priority = if let Some((v, l)) = get("connection", "autoconnect-priority") {
        let n = v.parse::<i16>().map_err(|_| {
            error(
                ErrorKind::Invalid,
                l,
                "Autoconnect priority must be -999 through999",
            )
        })?;
        if !(-999..=999).contains(&n) {
            return Err(error(
                ErrorKind::Invalid,
                l,
                "Autoconnect priority must be -999 through999",
            ));
        }
        n
    } else {
        0
    };
    let never_default = get("ipv4", "never-default")
        .map(|(v, l)| boolean(v, l))
        .transpose()?
        .unwrap_or(false);
    let mut routes = vec![];
    for n in 1..=MAX_ROUTES {
        let key = format!("route{n}");
        if let Some((s, l)) = values.get(&("ipv4", key.as_str())).copied()
            && !s.is_empty()
        {
            routes.push(route(s, l)?);
        }
    }
    Ok((
        Settings {
            vlan,
            id: id.into(),
            uuid: uid.to_ascii_lowercase(),
            interface: interface.into(),
            autoconnect,
            priority,
            addressing: match addressing {
                Addressing::Manual { address, gateway } => Addressing::Manual {
                    address,
                    gateway: if never_default { None } else { gateway },
                },
                Addressing::Automatic => Addressing::Automatic,
            },
            route_metric,
            never_default,
            routes,
        },
        bridge,
    ))
}
