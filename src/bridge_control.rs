//! Typed source-bound bridge profiles and owned current/future selections.
//! No host commands, daemon, device, packet or filesystem operations.
use crate::{bond_control::Binding, bridge_fabric as fabric, nm_profiles as key};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
type Result<T> = std::result::Result<T, key::Error>;
fn err(kind: key::ErrorKind, line: usize, message: impl Into<String>) -> key::Error {
    key::Error {
        kind,
        line,
        message: message.into(),
    }
}
fn name(n: &str) -> bool {
    !n.is_empty()
        && n.len() <= 15
        && !matches!(n, "." | "..")
        && n.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
fn file(n: &str) -> bool {
    !n.starts_with('.')
        && n.len() <= 64
        && n.ends_with(".nmconnection")
        && n.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
}
fn uuid(n: &str) -> bool {
    n.len() == 36
        && n.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
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
        let roles = std::iter::once(&self.controller)
            .chain(&self.members)
            .collect::<Vec<_>>();
        if self.version != 1
            || self.members.len() != 2
            || roles
                .iter()
                .any(|b| !file(&b.file) || !uuid(&b.uuid) || !name(&b.interface))
            || roles.iter().map(|b| &b.file).collect::<BTreeSet<_>>().len() != 3
            || roles
                .iter()
                .map(|b| b.uuid.to_ascii_lowercase())
                .collect::<BTreeSet<_>>()
                .len()
                != 3
            || roles
                .iter()
                .map(|b| &b.interface)
                .collect::<BTreeSet<_>>()
                .len()
                != 3
        {
            return Err(err(
                key::ErrorKind::Invalid,
                0,
                "Supply one distinct bridge controller and two distinct NIC names/files/UUIDs; nested controllers and discovery are outside scope",
            ));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Member {
    pub name: String,
    pub uuid: String,
    pub label: String,
    pub autoconnect: bool,
    pub controller: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    source: Source,
    controller: key::BridgeSettings,
    members: BTreeMap<String, Member>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Current {
    pub controller: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,fabric::Interface,2>")]
    pub members: Vec<fabric::Interface>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub plan: Plan,
    pub current: Current,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ControlProfile<'a> {
    Controller(&'a key::BridgeSettings),
    Member(&'a Member),
}
#[derive(Clone, Debug)]
struct Value {
    text: String,
    line: usize,
}
type Groups = BTreeMap<String, BTreeMap<String, Value>>;
fn groups(text: &str) -> Result<Groups> {
    if text.len() > key::MAX_TEXT || text.lines().count() > key::MAX_LINES {
        return Err(err(
            key::ErrorKind::Limit,
            0,
            "Bridge port keyfile exceeds16KiB/256lines",
        ));
    }
    if !text.is_ascii() {
        return Err(err(
            key::ErrorKind::Unsupported,
            0,
            "ASCII keyfiles only in this bounded bridge activity",
        ));
    }
    if text
        .bytes()
        .any(|b| b.is_ascii_control() && !b"\n\r\t".contains(&b))
    {
        return Err(err(
            key::ErrorKind::Syntax,
            0,
            "Control characters are not keyfile text, including in comments",
        ));
    }
    let mut out = Groups::new();
    let mut group = String::new();
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        let row = raw.trim();
        if row.is_empty() || row.starts_with('#') {
            continue;
        }
        if row.starts_with('[') {
            let g = row
                .strip_prefix('[')
                .and_then(|r| r.strip_suffix(']'))
                .filter(|s| !s.is_empty())
                .ok_or_else(|| {
                    err(
                        key::ErrorKind::Syntax,
                        line,
                        "Put a closed section header on its own line",
                    )
                })?;
            if ![
                "connection",
                "ethernet",
                "802-3-ethernet",
                "bridge-port",
                "proxy",
                "ipv4",
                "ipv6",
            ]
            .contains(&g)
            {
                return Err(err(
                    key::ErrorKind::Unsupported,
                    line,
                    format!("[{g}] is outside the bridge Ethernet-port subset"),
                ));
            }
            group = g.into();
            out.entry(group.clone()).or_default();
            continue;
        }
        let (k, v) = row
            .split_once('=')
            .filter(|_| !group.is_empty())
            .ok_or_else(|| {
                err(
                    key::ErrorKind::Syntax,
                    line,
                    "Put key=value under its named section",
                )
            })?;
        let k = k.trim();
        let v = v.trim();
        if k.len() > 48 || v.len() > 256 {
            return Err(err(
                key::ErrorKind::Limit,
                line,
                "Keyfile property exceeds its bounded length",
            ));
        }
        if k.is_empty()
            || !k
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
        {
            return Err(err(
                key::ErrorKind::Syntax,
                line,
                "Use a plain property name before =",
            ));
        }
        let allowed = match group.as_str() {
            "connection" => [
                "id",
                "uuid",
                "type",
                "interface-name",
                "autoconnect",
                "controller",
                "port-type",
            ]
            .contains(&k),
            "ipv4" => k == "method",
            "ipv6" => matches!(k, "method" | "addr-gen-mode"),
            _ => false,
        };
        if !allowed {
            return Err(err(
                key::ErrorKind::Unsupported,
                line,
                format!("[{group}] {k} is outside the declared bridge-port subset"),
            ));
        }
        if v.contains('\\') {
            return Err(err(
                key::ErrorKind::Unsupported,
                line,
                "Escapes and continued keyfile values are outside this subset",
            ));
        }
        out.get_mut(&group).unwrap().insert(
            k.into(),
            Value {
                text: v.into(),
                line,
            },
        );
    }
    Ok(out)
}
fn get<'a>(g: &'a Groups, group: &str, k: &str) -> Option<&'a Value> {
    g.get(group).and_then(|g| g.get(k))
}
fn required<'a>(g: &'a Groups, group: &str, k: &str) -> Result<&'a Value> {
    get(g, group, k)
        .filter(|v| !v.text.is_empty())
        .ok_or_else(|| {
            err(
                key::ErrorKind::Invalid,
                0,
                format!("Supply [{group}] {k} explicitly in this activity"),
            )
        })
}
fn member(g: &Groups, b: &Binding, controller: &str) -> Result<Member> {
    let label = required(g, "connection", "id")?;
    if label.text.len() > 64 {
        return Err(err(
            key::ErrorKind::Limit,
            label.line,
            "Use a short connection label",
        ));
    }
    for (k, want) in [
        ("uuid", b.uuid.as_str()),
        ("interface-name", b.interface.as_str()),
    ] {
        let v = required(g, "connection", k)?;
        if if k == "uuid" {
            !uuid(&v.text) || !v.text.eq_ignore_ascii_case(want)
        } else {
            v.text != want
        } {
            return Err(err(
                key::ErrorKind::Invalid,
                v.line,
                format!("Retain the supplied {k} for {}", b.file),
            ));
        }
    }
    let ty = required(g, "connection", "type")?;
    if !matches!(ty.text.as_str(), "ethernet" | "802-3-ethernet") {
        return Err(err(
            key::ErrorKind::Unsupported,
            ty.line,
            "A bridge port needs a provided Ethernet profile, not another controller",
        ));
    }
    let parent = match (
        get(g, "connection", "controller"),
        get(g, "connection", "port-type"),
    ) {
        (None, None) => None,
        (Some(c), Some(t)) => {
            if c.text != controller {
                return Err(err(
                    key::ErrorKind::Invalid,
                    c.line,
                    "Use the supplied literal bridge name; UUID lookup/discovery is outside scope",
                ));
            }
            if t.text != "bridge" {
                return Err(err(
                    key::ErrorKind::Unsupported,
                    t.line,
                    "This port must use port-type=bridge; other controller kinds are separate",
                ));
            }
            Some(c.text.clone())
        }
        _ => {
            return Err(err(
                key::ErrorKind::Unsupported,
                0,
                "Set or remove controller and port-type together; inferred partial binding is not modeled",
            ));
        }
    };
    if parent.is_none() && g.contains_key("bridge-port") {
        return Err(err(
            key::ErrorKind::Unsupported,
            0,
            "An unbound Ethernet candidate has no bridge-port settings",
        ));
    }
    for group in ["ipv4", "ipv6"] {
        let value = if parent.is_some() {
            get(g, group, "method")
        } else {
            Some(required(g, group, "method")?)
        };
        if let Some(v) = value
            && v.text != "disabled"
        {
            return Err(err(
                key::ErrorKind::Unsupported,
                v.line,
                "Bound ports may omit IP groups or use disabled methods. An unbound candidate needs disabled IP; configure the host address on the bridge controller",
            ));
        }
    }
    if let Some(v) = get(g, "ipv6", "addr-gen-mode")
        && v.text != "default"
    {
        return Err(err(
            key::ErrorKind::Unsupported,
            v.line,
            "Only omitted/default addr-gen-mode is supported with non-applicable port IPv6",
        ));
    }
    let autoconnect = match get(g, "connection", "autoconnect") {
        None => true,
        Some(v) => match v.text.as_str() {
            "true" => true,
            "false" => false,
            _ => {
                return Err(err(
                    key::ErrorKind::Unsupported,
                    v.line,
                    "Use true or false for port autoconnect",
                ));
            }
        },
    };
    Ok(Member {
        name: b.interface.clone(),
        uuid: b.uuid.to_ascii_lowercase(),
        label: label.text.clone(),
        autoconnect,
        controller: parent,
    })
}
impl Plan {
    pub fn from_files(source: &Source, files: &BTreeMap<String, String>) -> Result<Self> {
        source.validate()?;
        if files.len() > key::MAX_PROFILES
            || files
                .iter()
                .any(|(n, t)| !file(n) || t.len() > key::MAX_TEXT)
        {
            return Err(err(
                key::ErrorKind::Limit,
                0,
                "Use at most8bounded virtual keyfiles with flat supplied filenames",
            ));
        }
        let read = |b: &Binding| {
            files.get(&b.file).ok_or_else(|| {
                err(
                    key::ErrorKind::Invalid,
                    0,
                    format!("Retain the provided profile file {}", b.file),
                )
            })
        };
        let controller = key::parse_bridge_settings(read(&source.controller)?)?;
        if controller.ipv4.interface != source.controller.interface
            || !controller
                .ipv4
                .uuid
                .eq_ignore_ascii_case(&source.controller.uuid)
        {
            return Err(err(
                key::ErrorKind::Invalid,
                0,
                "Retain the supplied bridge controller interface and UUID",
            ));
        }
        let mut members = BTreeMap::new();
        for b in &source.members {
            let p = member(&groups(read(b)?)?, b, &source.controller.interface)?;
            members.insert(b.interface.clone(), p);
        }
        Ok(Self {
            source: source.clone(),
            controller,
            members,
        })
    }
    pub fn source(&self) -> &Source {
        &self.source
    }
    pub fn controller(&self) -> &key::BridgeSettings {
        &self.controller
    }
    pub fn members(&self) -> impl Iterator<Item = (&str, &Member)> {
        self.members.iter().map(|(n, p)| (n.as_str(), p))
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
    pub fn selected(&self, current: &Current) -> Result<fabric::Current> {
        let members = current
            .members
            .iter()
            .map(|n| n.0.as_str())
            .collect::<BTreeSet<_>>();
        if current.members.len() > 2
            || members.len() != current.members.len()
            || (!current.controller && !members.is_empty())
            || members.iter().any(|n| {
                !self.members.get(*n).is_some_and(|p| {
                    p.controller.as_deref() == Some(self.source.controller.interface.as_str())
                })
            })
        {
            return Err(err(
                key::ErrorKind::Invalid,
                0,
                "Current bridge membership must list distinct provided bound ports with a selected controller",
            ));
        }
        Ok(fabric::Current {
            up: current.controller,
            members: current.members.clone(),
        })
    }
    pub fn automatic(&self) -> Result<Current> {
        let forced = self.controller.ipv4.autoconnect && self.controller.autoconnect_ports;
        if self.members.values().any(|p| {
            p.autoconnect
                && (!forced
                    || p.controller.as_deref() != Some(self.source.controller.interface.as_str()))
        }) {
            return Err(err(
                key::ErrorKind::Unsupported,
                0,
                "Independent port-driven activation/controller discovery is outside this settled subset; this does not mean real ports cannot autoconnect",
            ));
        }
        let current = Current {
            controller: self.controller.ipv4.autoconnect,
            members: if forced {
                self.members
                    .iter()
                    .filter(|(_, p)| {
                        p.controller.as_deref() == Some(self.source.controller.interface.as_str())
                    })
                    .map(|(n, _)| fabric::Interface(n.clone()))
                    .collect()
            } else {
                vec![]
            },
        };
        self.selected(&current)?;
        Ok(current)
    }
}
impl Snapshot {
    pub fn supplied(plan: Plan, current: Current) -> Result<Self> {
        plan.selected(&current)?;
        Ok(Self { plan, current })
    }
    pub fn automatic(plan: Plan) -> Result<Self> {
        let current = plan.automatic()?;
        Self::supplied(plan, current)
    }
    pub fn projection(&self) -> Result<fabric::Current> {
        self.plan.selected(&self.current)
    }
}
