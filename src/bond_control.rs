//! Source-enabled, bounded NetworkManager control keyfiles for active-backup.
//! Pure parsing and settled selection. No daemon, filesystem, process or packets.
use crate::nm_profiles::{Error, ErrorKind, MAX_LINES, MAX_PROFILES, MAX_TEXT};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, line: usize, message: impl Into<String>) -> Error {
    Error {
        kind,
        line,
        message: message.into(),
    }
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 15
        && !matches!(s, "." | "..")
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}
fn filename(s: &str) -> bool {
    !s.starts_with('.')
        && s.len() <= 64
        && s.ends_with(".nmconnection")
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub file: String,
    #[serde(deserialize_with = "crate::cron::text::<_,36>")]
    pub uuid: String,
    #[serde(deserialize_with = "crate::cron::text::<_,15>")]
    pub interface: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub version: u32,
    pub controller: Binding,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Binding,2>")]
    pub members: Vec<Binding>,
}
impl Source {
    pub fn validate(&self) -> Result<()> {
        let bindings = std::iter::once(&self.controller)
            .chain(&self.members)
            .collect::<Vec<_>>();
        if self.version != 1
            || self.members.len() != 2
            || bindings
                .iter()
                .any(|b| !filename(&b.file) || !uuid(&b.uuid) || !name(&b.interface))
            || bindings
                .iter()
                .map(|b| &b.file)
                .collect::<BTreeSet<_>>()
                .len()
                != 3
            || bindings
                .iter()
                .map(|b| b.uuid.to_ascii_lowercase())
                .collect::<BTreeSet<_>>()
                .len()
                != 3
            || bindings
                .iter()
                .map(|b| &b.interface)
                .collect::<BTreeSet<_>>()
                .len()
                != 3
        {
            return Err(error(
                ErrorKind::Invalid,
                0,
                "Supply one distinct bond controller and two distinct physical-member identities/files; discovery/nested controllers are outside this subset",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Controller {
    pub name: String,
    pub label: String,
    pub autoconnect: bool,
    pub autoconnect_ports: bool,
    pub primary: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub controller: Option<String>,
    pub name: String,
    pub label: String,
    pub autoconnect: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    pub source: Source,
    pub controller: Controller,
    pub members: BTreeMap<String, Member>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub controller: bool,
    pub members: BTreeSet<String>,
    pub active: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InterfaceName(#[serde(deserialize_with = "crate::cron::text::<_,15>")] pub String);
/// Source-held current membership is explicit, never fabricated from new drafts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Current {
    pub controller: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,InterfaceName,2>")]
    pub members: Vec<InterfaceName>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub plan: Plan,
    pub current: Current,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlProfile<'a> {
    Controller(&'a Controller),
    Member(&'a Member),
}
impl Snapshot {
    pub fn supplied(
        plan: Plan,
        current: Current,
        carrier: &BTreeMap<String, bool>,
    ) -> Result<Self> {
        let s = Self { plan, current };
        s.selection(carrier)?;
        Ok(s)
    }
    pub fn automatic(plan: Plan, carrier: &BTreeMap<String, bool>) -> Result<Self> {
        let selection = plan.automatic(carrier)?;
        Self::supplied(
            plan,
            Current {
                controller: selection.controller,
                members: selection.members.into_iter().map(InterfaceName).collect(),
            },
            carrier,
        )
    }
    pub fn selection(&self, carrier: &BTreeMap<String, bool>) -> Result<Selection> {
        let members = self
            .current
            .members
            .iter()
            .map(|n| n.0.clone())
            .collect::<BTreeSet<_>>();
        if self.current.members.len() > 2 || members.len() != self.current.members.len() {
            return Err(error(
                ErrorKind::Invalid,
                0,
                "Retain at most two distinct supplied current member NICs",
            ));
        }
        self.plan
            .selected(self.current.controller, &members, carrier)
    }
}
#[derive(Clone, Debug)]
struct Value {
    line: usize,
    text: String,
}
type Groups = BTreeMap<String, BTreeMap<String, Value>>;
fn groups(text: &str) -> Result<Groups> {
    if text.len() > MAX_TEXT || text.lines().count() > MAX_LINES {
        return Err(error(
            ErrorKind::Limit,
            0,
            "Control keyfile exceeds16KiB/256lines",
        ));
    }
    let mut out = Groups::new();
    let mut current = String::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        let n = i + 1;
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.chars().any(|c| c.is_control() && c != '\t') {
            return Err(error(
                ErrorKind::Syntax,
                n,
                "Control characters are not keyfile content",
            ));
        }
        if line.starts_with('[') {
            let group = line
                .strip_prefix('[')
                .and_then(|s| s.strip_suffix(']'))
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    error(
                        ErrorKind::Syntax,
                        n,
                        "Use a section such as [connection] on its own line",
                    )
                })?;
            if ![
                "connection",
                "bond",
                "bond-port",
                "proxy",
                "ethernet",
                "802-3-ethernet",
                "ipv4",
                "ipv6",
            ]
            .contains(&group)
            {
                return Err(error(
                    ErrorKind::Unsupported,
                    n,
                    format!("[{group}] is outside the bond-control keyfile subset"),
                ));
            }
            current = group.into();
            out.entry(current.clone()).or_default();
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .filter(|_| !current.is_empty())
            .ok_or_else(|| {
                error(
                    ErrorKind::Syntax,
                    n,
                    "Put key=value inside its named section",
                )
            })?;
        let key = key.trim();
        if key.is_empty()
            || !key
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        {
            return Err(error(ErrorKind::Syntax, n, "Use a plain key before ="));
        }
        let allowed = match current.as_str() {
            "connection" => [
                "id",
                "uuid",
                "type",
                "interface-name",
                "autoconnect",
                "autoconnect-ports",
                "controller",
                "port-type",
            ]
            .contains(&key),
            "bond" => ["mode", "miimon", "primary", "primary_reselect"].contains(&key),
            "ipv4" => key == "method",
            "ipv6" => matches!(key, "method" | "addr-gen-mode"),
            _ => false,
        };
        if !allowed {
            return Err(error(
                ErrorKind::Unsupported,
                n,
                format!(
                    "[{current}] {key} is outside this control-profile subset; bond options are individual keys, not an options=command string"
                ),
            ));
        }
        out.get_mut(&current).unwrap().insert(
            key.into(),
            Value {
                line: n,
                text: value.trim().into(),
            },
        );
    }
    Ok(out)
}
fn get<'a>(g: &'a Groups, group: &str, key: &str) -> Option<&'a Value> {
    g.get(group).and_then(|s| s.get(key))
}
fn required<'a>(g: &'a Groups, group: &str, key: &str) -> Result<&'a Value> {
    get(g, group, key)
        .filter(|v| !v.text.is_empty())
        .ok_or_else(|| {
            error(
                ErrorKind::Invalid,
                0,
                format!("Supply [{group}] {key} explicitly in this activity"),
            )
        })
}
fn boolean(g: &Groups, key: &str, default: bool) -> Result<bool> {
    match get(g, "connection", key) {
        None => Ok(default),
        Some(v) => match v.text.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(error(
                ErrorKind::Unsupported,
                v.line,
                format!("Use true or false for {key} in this keyfile subset"),
            )),
        },
    }
}
fn identity(g: &Groups, b: &Binding) -> Result<()> {
    let id = required(g, "connection", "id")?;
    if id.text.len() > 120 {
        return Err(error(
            ErrorKind::Limit,
            id.line,
            "Use a short connection label",
        ));
    }
    for (key, expected) in [
        ("uuid", b.uuid.as_str()),
        ("interface-name", b.interface.as_str()),
    ] {
        let v = required(g, "connection", key)?;
        let matches = if key == "uuid" {
            uuid(&v.text) && v.text.eq_ignore_ascii_case(expected)
        } else {
            v.text == expected
        };
        if !matches {
            return Err(error(
                ErrorKind::Invalid,
                v.line,
                format!(
                    "Retain the supplied {key} for {}; this activity does not create another controller or NIC",
                    b.file
                ),
            ));
        }
    }
    Ok(())
}
fn ip_methods(g: &Groups, bound: bool) -> Result<()> {
    for group in ["ipv4", "ipv6"] {
        let value = if bound {
            get(g, group, "method")
        } else {
            Some(required(g, group, "method")?)
        };
        if let Some(v) = value
            && v.text != "disabled"
        {
            return Err(error(
                ErrorKind::Unsupported,
                v.line,
                "Omit IP methods on a bound port or use disabled. Unbound/controller profiles need disabled methods; the VLAN profiles hold their own IPv4 addresses in this subset",
            ));
        }
    }
    if let Some(v) = get(g, "ipv6", "addr-gen-mode")
        && v.text != "default"
    {
        return Err(error(
            ErrorKind::Unsupported,
            v.line,
            "Only omitted/default addr-gen-mode with disabled or non-applicable port IPv6 is supported",
        ));
    }
    Ok(())
}
impl Plan {
    pub fn from_files(source: &Source, files: &BTreeMap<String, String>) -> Result<Self> {
        source.validate()?;
        if files.len() > MAX_PROFILES
            || files
                .iter()
                .any(|(n, t)| !filename(n) || t.len() > MAX_TEXT)
        {
            return Err(error(
                ErrorKind::Limit,
                0,
                "Use at most8bounded virtual keyfiles with flat supplied filenames",
            ));
        }
        let read = |b: &Binding| -> Result<Groups> {
            let text = files.get(&b.file).ok_or_else(|| {
                error(
                    ErrorKind::Invalid,
                    0,
                    format!("Retain the supplied control file {}", b.file),
                )
            })?;
            let g = groups(text)?;
            identity(&g, b)?;
            Ok(g)
        };
        let c = read(&source.controller)?;
        ip_methods(&c, false)?;
        if c.contains_key("bond-port") {
            return Err(error(
                ErrorKind::Unsupported,
                0,
                "A bond-port setting belongs to a bound Ethernet port, not the controller",
            ));
        }
        let kind = required(&c, "connection", "type")?;
        if kind.text != "bond" {
            return Err(error(
                ErrorKind::Unsupported,
                kind.line,
                "The controller profile needs type=bond; other controller types are outside this activity",
            ));
        }
        for key in ["controller", "port-type"] {
            if let Some(v) = get(&c, "connection", key) {
                return Err(error(
                    ErrorKind::Unsupported,
                    v.line,
                    "Nested controllers are outside this activity",
                ));
            }
        }
        if c.contains_key("ethernet") || c.contains_key("802-3-ethernet") {
            return Err(error(
                ErrorKind::Unsupported,
                0,
                "The bond controller is separate from its Ethernet member profiles",
            ));
        }
        let mode = required(&c, "bond", "mode")?;
        if !matches!(mode.text.as_str(), "active-backup" | "1") {
            return Err(error(
                ErrorKind::Unsupported,
                mode.line,
                "Only active-backup (mode1) is modeled; omission on a real system does not imply this mode",
            ));
        }
        let monitor = required(&c, "bond", "miimon")?;
        if monitor.text.parse::<u32>() != Ok(100) {
            return Err(error(
                ErrorKind::Unsupported,
                monitor.line,
                "This settled snapshot activity requires miimon=100. Zero disables MII monitoring; ARP monitoring and other intervals are not modeled",
            ));
        }
        if let Some(v) = get(&c, "bond", "primary_reselect")
            && !matches!(v.text.as_str(), "always" | "0")
        {
            return Err(error(
                ErrorKind::Unsupported,
                v.line,
                "Only primary_reselect=always (0), also the omitted default, is supported here",
            ));
        }
        let primary = required(&c, "bond", "primary")?;
        if !source.members.iter().any(|b| b.interface == primary.text) {
            return Err(error(
                ErrorKind::Invalid,
                primary.line,
                "Choose a supplied physical member as primary; a bond or VLAN name is not that NIC",
            ));
        }
        let auto_ports = required(&c, "connection", "autoconnect-ports")?;
        let autoconnect_ports = match auto_ports.text.as_str() {
            "1" => true,
            "0" => false,
            _ => {
                return Err(error(
                    ErrorKind::Unsupported,
                    auto_ports.line,
                    "Choose explicit autoconnect-ports=1 or0; inherited global defaults are not guessed",
                ));
            }
        };
        let controller = Controller {
            name: source.controller.interface.clone(),
            label: required(&c, "connection", "id")?.text.clone(),
            autoconnect: boolean(&c, "autoconnect", true)?,
            autoconnect_ports,
            primary: primary.text.clone(),
        };
        let mut members = BTreeMap::new();
        for b in &source.members {
            let p = read(b)?;
            if p.contains_key("bond") {
                return Err(error(
                    ErrorKind::Unsupported,
                    0,
                    "A member uses an Ethernet port profile, not another [bond] configuration",
                ));
            }
            if get(&p, "connection", "autoconnect-ports").is_some() {
                return Err(error(
                    ErrorKind::Unsupported,
                    0,
                    "autoconnect-ports belongs to the controller, not a member",
                ));
            }
            let kind = required(&p, "connection", "type")?;
            if !matches!(kind.text.as_str(), "ethernet" | "802-3-ethernet") {
                return Err(error(
                    ErrorKind::Unsupported,
                    kind.line,
                    "A provided bond member needs an Ethernet port profile",
                ));
            }
            let attachment = match (
                get(&p, "connection", "controller"),
                get(&p, "connection", "port-type"),
            ) {
                (None, None) => None,
                (Some(parent), Some(typ)) => {
                    if parent.text != source.controller.interface {
                        return Err(error(
                            ErrorKind::Invalid,
                            parent.line,
                            "Use the supplied literal bond controller name; UUID lookup/discovery is outside this activity",
                        ));
                    }
                    if typ.text != "bond" {
                        return Err(error(
                            ErrorKind::Unsupported,
                            typ.line,
                            "Only port-type=bond is supported for a bound member",
                        ));
                    }
                    Some(parent.text.clone())
                }
                _ => {
                    return Err(error(
                        ErrorKind::Unsupported,
                        0,
                        "Set or remove controller and port-type together; incomplete/inferred port binding is outside this subset",
                    ));
                }
            };
            ip_methods(&p, attachment.is_some())?;
            if attachment.is_none() && p.contains_key("bond-port") {
                return Err(error(
                    ErrorKind::Unsupported,
                    0,
                    "An unbound Ethernet candidate has no bond-port settings",
                ));
            }
            members.insert(
                b.interface.clone(),
                Member {
                    controller: attachment,
                    name: b.interface.clone(),
                    label: required(&p, "connection", "id")?.text.clone(),
                    autoconnect: boolean(&p, "autoconnect", true)?,
                },
            );
        }
        Ok(Self {
            source: source.clone(),
            controller,
            members,
        })
    }
    pub fn profile(&self, filename: &str) -> Option<ControlProfile<'_>> {
        if filename == self.source.controller.file {
            Some(ControlProfile::Controller(&self.controller))
        } else {
            self.source
                .members
                .iter()
                .find(|b| b.file == filename)
                .and_then(|b| self.members.get(&b.interface))
                .map(ControlProfile::Member)
        }
    }
    /// Explicit membership is useful for supplied current snapshots. Nothing is
    /// inferred from edited future eligibility or end-to-end reachability.
    pub fn selected(
        &self,
        controller: bool,
        members: &BTreeSet<String>,
        carrier: &BTreeMap<String, bool>,
    ) -> Result<Selection> {
        self.source.validate()?;
        if self.members.len() != 2
            || !name(&self.controller.name)
            || self.controller.name != self.source.controller.interface
            || self.members.keys().collect::<BTreeSet<_>>()
                != self.source.members.iter().map(|b| &b.interface).collect()
            || self.members.contains_key(&self.controller.name)
            || !self.members.contains_key(&self.controller.primary)
            || self.members.iter().any(|(n, p)| n != &p.name || !name(n))
            || carrier.keys().ne(self.members.keys())
            || members.iter().any(|n| {
                !self.members.contains_key(n)
                    || self.members[n].controller.as_deref() != Some(self.controller.name.as_str())
            })
            || (!controller && !members.is_empty())
        {
            return Err(error(
                ErrorKind::Invalid,
                0,
                "Supply carrier for exactly the two known members and an explicit valid current membership",
            ));
        }
        let up = |n: &String| members.contains(n) && carrier[n];
        let active = if !controller {
            None
        } else if up(&self.controller.primary) {
            Some(self.controller.primary.clone())
        } else {
            self.members.keys().find(|n| up(n)).cloned()
        };
        Ok(Selection {
            controller,
            members: members.clone(),
            active,
        })
    }
    pub fn automatic(&self, carrier: &BTreeMap<String, bool>) -> Result<Selection> {
        let forced = self.controller.autoconnect && self.controller.autoconnect_ports;
        if self.members.values().any(|p| {
            p.autoconnect
                && (!forced || p.controller.as_deref() != Some(self.controller.name.as_str()))
        }) {
            return Err(error(
                ErrorKind::Unsupported,
                0,
                "Independent port-driven activation/controller discovery is outside this settled subset. With no forced controller activation, use the supplied autoconnect=false member profiles; this is a simulator limitation, not a rule that real ports cannot autoconnect",
            ));
        }
        // NetworkManager connection.autoconnect-ports=1 explicitly activates
        // matching port profiles even if their own autoconnect is false.
        let members = if forced {
            self.members
                .iter()
                .filter(|(_, p)| p.controller.as_deref() == Some(self.controller.name.as_str()))
                .map(|(n, _)| n.clone())
                .collect()
        } else {
            BTreeSet::new()
        };
        self.selected(self.controller.autoconnect, &members, carrier)
    }
}
