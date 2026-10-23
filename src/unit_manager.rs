//! Closed, bounded virtual unit/job manager. No OS, process, filesystem or network API.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

pub const MAX_UNITS: usize = 16;
pub const MAX_PROGRAMS: usize = 8;
pub const MAX_CONFIGS: usize = 16;
pub const MAX_ARTIFACTS: usize = 8;
pub const MAX_UNIT_BYTES: usize = 8192;
pub const MAX_CONFIG_BYTES: usize = 4096;
pub const MAX_JSON_BYTES: usize = 4_194_304;
pub const MAX_ERROR_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Limit,
    Unsupported,
    Syntax,
    Missing,
    Ordering,
    Program,
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
type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, message: impl Into<String>) -> Error {
    let text = message.into();
    let cut = text.len() > MAX_ERROR_BYTES;
    let mut end = text.len().min(if cut {
        MAX_ERROR_BYTES - 3
    } else {
        MAX_ERROR_BYTES
    });
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    let mut message = String::with_capacity(end + if cut { 3 } else { 0 });
    message.push_str(&text[..end]);
    if cut {
        message.push('…');
    }
    Error { kind, message }
}
fn invalid(message: &str) -> Error {
    error(ErrorKind::Invalid, message)
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.as_bytes()[0].is_ascii_alphanumeric()
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
}
fn unit(s: &str) -> bool {
    identifier(s)
        && (s.ends_with(".service") || s.ends_with(".target"))
        && s.split('.').next().is_some_and(|s| !s.is_empty())
}
fn path(s: &str) -> bool {
    s.len() <= 128
        && s.starts_with('/')
        && s != "/"
        && !s.ends_with('/')
        && s.split('/').count() <= 9
        && s[1..].split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"_.-".contains(&c))
        })
}
fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> std::result::Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} UTF-8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if s.len() > N {
                Err(E::custom("Virtual unit text limit"))
            } else {
                Ok(s.into())
            }
        }
    }
    d.deserialize_str(Text::<N>)
}
fn text128<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    text::<D, 128>(d)
}
fn text256<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    text::<D, 256>(d)
}
fn source<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    text::<D, MAX_UNIT_BYTES>(d)
}
fn config<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    text::<D, MAX_CONFIG_BYTES>(d)
}
struct Key(String);
impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        text128(d).map(Self)
    }
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    struct Map<V, const N: usize>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Map<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} distinct fixed slots")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Key(key)) = a.next_key()? {
                if out.len() >= N || out.contains_key(&key) {
                    return Err(A::Error::custom("Duplicate virtual slot or map limit"));
                }
                out.insert(key, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Map::<V, N>(std::marker::PhantomData))
}
fn names<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<BTreeSet<String>, D::Error> {
    struct Names;
    impl<'de> Visitor<'de> for Names {
        type Value = BTreeSet<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most16 distinct names")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeSet::new();
            while out.len() < MAX_UNITS {
                let Some(Key(key)) = a.next_element()? else {
                    return Ok(out);
                };
                if !out.insert(key) {
                    return Err(A::Error::custom("Duplicate name"));
                }
            }
            if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(A::Error::custom("Name count limit"));
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Names)
}
struct Source(String);
impl<'de> Deserialize<'de> for Source {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        source(d).map(Self)
    }
}
struct Config(String);
impl<'de> Deserialize<'de> for Config {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        config(d).map(Self)
    }
}
fn documents<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error> {
    map::<D, Source, MAX_UNITS>(d).map(|m| m.into_iter().map(|(k, v)| (k, v.0)).collect())
}
fn configurations<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error> {
    map::<D, Config, MAX_CONFIGS>(d).map(|m| m.into_iter().map(|(k, v)| (k, v.0)).collect())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Need {
    #[serde(deserialize_with = "text128")]
    pub artifact: String,
    #[serde(deserialize_with = "text256")]
    pub value: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Program {
    ValueTask {
        #[serde(deserialize_with = "text128")]
        config: String,
        #[serde(deserialize_with = "text128")]
        key: String,
        #[serde(deserialize_with = "text128")]
        artifact: String,
        prerequisite: Option<Need>,
    },
    PortDaemon {
        #[serde(deserialize_with = "text128")]
        config: String,
        #[serde(deserialize_with = "text128")]
        key: String,
        prerequisite: Option<Need>,
    },
}
impl Program {
    fn config(&self) -> &str {
        match self {
            Self::ValueTask { config, .. } | Self::PortDaemon { config, .. } => config,
        }
    }
    fn key(&self) -> &str {
        match self {
            Self::ValueTask { key, .. } | Self::PortDaemon { key, .. } => key,
        }
    }
    fn need(&self) -> Option<&Need> {
        match self {
            Self::ValueTask { prerequisite, .. } | Self::PortDaemon { prerequisite, .. } => {
                prerequisite.as_ref()
            }
        }
    }
}
fn programs<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, Program>, D::Error> {
    map::<D, Program, MAX_PROGRAMS>(d)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    pub schema_version: u32,
    #[serde(deserialize_with = "text128")]
    pub fixture: String,
    #[serde(deserialize_with = "names")]
    pub units: BTreeSet<String>,
    #[serde(deserialize_with = "names")]
    pub config_paths: BTreeSet<String>,
    #[serde(deserialize_with = "programs")]
    pub programs: BTreeMap<String, Program>,
    #[serde(deserialize_with = "names")]
    pub artifact_ids: BTreeSet<String>,
}
impl Inventory {
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || !identifier(&self.fixture)
            || self.units.is_empty()
            || self.units.len() > MAX_UNITS
            || self.config_paths.len() > MAX_CONFIGS
            || self.programs.len() > MAX_PROGRAMS
            || self.artifact_ids.len() > MAX_ARTIFACTS
            || self.units.iter().any(|s| !unit(s))
            || !self.units.iter().any(|s| s.ends_with(".target"))
            || self.config_paths.iter().any(|s| !path(s))
            || self.artifact_ids.iter().any(|s| !identifier(s))
        {
            return Err(invalid("Invalid fixed virtual unit inventory or limits"));
        }
        let mut producers = BTreeSet::new();
        for (exec, p) in &self.programs {
            if !path(exec)
                || !exec.starts_with("/sim/bin/")
                || exec.len() > 96
                || !self.config_paths.contains(p.config())
                || !identifier(p.key())
                || p.key().len() > 32
                || p.need().is_some_and(|n| {
                    !self.artifact_ids.contains(&n.artifact)
                        || n.value.is_empty()
                        || n.value.len() > 256
                        || n.value.chars().any(char::is_control)
                })
            {
                return Err(invalid(
                    "Invalid fictional program/configuration/artifact reference",
                ));
            }
            if let Program::ValueTask { artifact, .. } = p
                && (!self.artifact_ids.contains(artifact) || !producers.insert(artifact))
            {
                return Err(invalid(
                    "Each virtual artifact needs at most one supplied producer descriptor",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServiceType {
    Simple,
    Oneshot,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnitType {
    Target,
    Service {
        kind: ServiceType,
        exec: String,
        remain_after_exit: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    pub description: String,
    pub requires: BTreeSet<String>,
    pub wants: BTreeSet<String>,
    pub after: BTreeSet<String>,
    pub before: BTreeSet<String>,
    pub wanted_by: BTreeSet<String>,
    pub kind: UnitType,
}
fn line_error(kind: ErrorKind, id: &str, line: usize, message: &str) -> Error {
    error(kind, format!("{id}, line{line}: {message}"))
}
pub fn parse_unit(inventory: &Inventory, id: &str, text: &str) -> Result<Definition> {
    inventory.validate()?;
    if !inventory.units.contains(id) {
        return Err(invalid("Unknown fixed unit ID"));
    }
    if text.len() > MAX_UNIT_BYTES {
        return Err(error(ErrorKind::Limit, "Unit document exceeds8192 bytes"));
    }
    let mut d = Definition {
        description: String::new(),
        requires: BTreeSet::new(),
        wants: BTreeSet::new(),
        after: BTreeSet::new(),
        before: BTreeSet::new(),
        wanted_by: BTreeSet::new(),
        kind: UnitType::Target,
    };
    let mut section = "";
    let mut sections = BTreeSet::new();
    let mut scalars = BTreeSet::new();
    let mut kind = None;
    let mut exec = None;
    let mut remain = None;
    for (i, raw) in text.lines().enumerate() {
        let line = i + 1;
        if line > 256 || raw.len() > 512 {
            return Err(line_error(
                ErrorKind::Limit,
                id,
                line,
                "Unit line/count limit",
            ));
        }
        if raw.chars().any(|c| c.is_control() && c != '\t') {
            return Err(line_error(
                ErrorKind::Unsupported,
                id,
                line,
                "Control characters are unsupported",
            ));
        }
        let s = raw.trim_matches([' ', '\t']);
        if s.is_empty() || s.starts_with(['#', ';']) {
            continue;
        }
        if s.starts_with('[') {
            if !matches!(s, "[Unit]" | "[Service]" | "[Install]") || !sections.insert(s) {
                return Err(line_error(
                    ErrorKind::Unsupported,
                    id,
                    line,
                    "Unsupported or repeated unit section",
                ));
            }
            if id.ends_with(".target") && s == "[Service]" {
                return Err(line_error(
                    ErrorKind::Unsupported,
                    id,
                    line,
                    "Targets have no Service or Target section",
                ));
            }
            section = s;
            continue;
        }
        let (key, value) = s.split_once('=').ok_or_else(|| {
            line_error(
                ErrorKind::Syntax,
                id,
                line,
                "Expected a supported KEY=VALUE assignment",
            )
        })?;
        let key = key.trim_matches([' ', '\t']);
        let value = value.trim_matches([' ', '\t']);
        let list = match (section, key) {
            ("[Unit]", "Requires") => Some(&mut d.requires),
            ("[Unit]", "Wants") => Some(&mut d.wants),
            ("[Unit]", "After") => Some(&mut d.after),
            ("[Unit]", "Before") => Some(&mut d.before),
            ("[Install]", "WantedBy") => Some(&mut d.wanted_by),
            _ => None,
        };
        if let Some(list) = list {
            if value.is_empty() {
                list.clear();
                continue;
            }
            for name in value.split_ascii_whitespace() {
                if !inventory.units.contains(name)
                    || key == "WantedBy" && !name.ends_with(".target")
                {
                    return Err(line_error(
                        ErrorKind::Missing,
                        id,
                        line,
                        "Dependency/install target must name a supplied unit of the supported kind",
                    ));
                }
                if name == id && matches!(key, "Requires" | "Wants") {
                    return Err(line_error(
                        ErrorKind::Unsupported,
                        id,
                        line,
                        "Self pull-in references are unsupported",
                    ));
                }
                if !list.contains(name) && list.len() >= MAX_UNITS {
                    return Err(line_error(
                        ErrorKind::Limit,
                        id,
                        line,
                        "Dependency count limit",
                    ));
                }
                list.insert(name.into());
            }
            continue;
        }
        if !scalars.insert((section, key)) {
            return Err(line_error(
                ErrorKind::Unsupported,
                id,
                line,
                "Repeated scalar assignments are unsupported",
            ));
        }
        match (section, key) {
            ("[Unit]", "Description") => {
                if value.is_empty() || value.len() > 160 || value.contains('%') {
                    return Err(line_error(
                        ErrorKind::Unsupported,
                        id,
                        line,
                        "Description must be plain nonempty text up to160 bytes",
                    ));
                }
                d.description = value.into();
            }
            ("[Service]", "Type") => {
                kind = Some(match value {
                    "simple" => ServiceType::Simple,
                    "oneshot" => ServiceType::Oneshot,
                    _ => {
                        return Err(line_error(
                            ErrorKind::Unsupported,
                            id,
                            line,
                            "Only explicit Type=simple or Type=oneshot is modeled",
                        ));
                    }
                })
            }
            ("[Service]", "ExecStart") => {
                if value.split_ascii_whitespace().count() != 1
                    || !inventory.programs.contains_key(value)
                {
                    return Err(line_error(
                        ErrorKind::Unsupported,
                        id,
                        line,
                        "ExecStart must be exactly one supplied fictional /sim/bin descriptor; no commands or arguments run",
                    ));
                }
                exec = Some(value.to_owned());
            }
            ("[Service]", "RemainAfterExit") => {
                remain = Some(match value {
                    "yes" => true,
                    "no" => false,
                    _ => {
                        return Err(line_error(
                            ErrorKind::Unsupported,
                            id,
                            line,
                            "RemainAfterExit accepts yes/no only",
                        ));
                    }
                })
            }
            _ => {
                return Err(line_error(
                    ErrorKind::Unsupported,
                    id,
                    line,
                    "Unsupported unit key/section; see the declared subset",
                ));
            }
        }
    }
    if d.description.is_empty() {
        return Err(error(
            ErrorKind::Syntax,
            format!("{id}: a nonempty Description is required in this teaching subset"),
        ));
    }
    if id.ends_with(".service") {
        let kind = kind.ok_or_else(|| {
            error(
                ErrorKind::Syntax,
                format!("{id}: explicit Type is required"),
            )
        })?;
        let exec =
            exec.ok_or_else(|| error(ErrorKind::Syntax, format!("{id}: ExecStart is required")))?;
        if kind == ServiceType::Simple && remain.is_some() {
            return Err(error(
                ErrorKind::Unsupported,
                "RemainAfterExit is supported only for oneshot here",
            ));
        }
        if !matches!(
            (kind, &inventory.programs[&exec]),
            (ServiceType::Simple, Program::PortDaemon { .. })
                | (ServiceType::Oneshot, Program::ValueTask { .. })
        ) {
            return Err(error(
                ErrorKind::Unsupported,
                "Service Type and fictional descriptor kind differ",
            ));
        }
        d.kind = UnitType::Service {
            kind,
            exec,
            remain_after_exit: remain.unwrap_or(false),
        };
    }
    Ok(d)
}

fn config_value<'a>(text: &'a str, key: &str) -> Result<&'a str> {
    if text.len() > MAX_CONFIG_BYTES {
        return Err(error(ErrorKind::Limit, "Application configuration limit"));
    }
    let mut value = None;
    for (i, line) in text.lines().enumerate() {
        if i >= 256 || line.len() > 512 {
            return Err(error(
                ErrorKind::Limit,
                "Application config line/count limit",
            ));
        }
        if line.chars().any(|c| c.is_control() && c != '\t') {
            return Err(error(
                ErrorKind::Program,
                "Unsupported control character in virtual program config",
            ));
        }
        let line = line.trim_matches([' ', '\t']);
        if line.is_empty() || line.starts_with(['#', ';']) {
            continue;
        }
        let (k, v) = line.split_once('=').ok_or_else(|| {
            error(
                ErrorKind::Program,
                "Virtual program expects its declared KEY=VALUE",
            )
        })?;
        if k.trim_matches([' ', '\t']) != key || value.is_some() {
            return Err(error(
                ErrorKind::Program,
                "Unknown or duplicate application configuration key",
            ));
        }
        let v = v.trim_matches([' ', '\t']);
        if v.is_empty() || v.len() > 256 {
            return Err(error(
                ErrorKind::Program,
                "Application value must contain1–256 bytes",
            ));
        }
        value = Some(v);
    }
    value.ok_or_else(|| error(ErrorKind::Program, format!("Missing application key {key}")))
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProgramResult {
    pub port: Option<u16>,
    pub value: Option<String>,
}
fn evaluate(program: &Program, configuration: &str, input: Option<&str>) -> Result<ProgramResult> {
    if let Some(need) = program.need() {
        if input != Some(need.value.as_str()) {
            return Err(error(
                ErrorKind::Program,
                "Prerequisite artifact is unavailable or wrong at program entry; pull-in alone does not guarantee ordering",
            ));
        }
    } else if input.is_some() {
        return Err(invalid(
            "Unexpected artifact input in saved program snapshot",
        ));
    }
    let value = config_value(configuration, program.key())?;
    match program {
        Program::ValueTask { .. } => Ok(ProgramResult {
            port: None,
            value: Some(value.into()),
        }),
        Program::PortDaemon { .. } => {
            let port = if value.bytes().all(|c| c.is_ascii_digit()) {
                value.parse::<u16>().ok().filter(|n| *n != 0)
            } else {
                None
            };
            port.map(|port| ProgramResult {
                port: Some(port),
                value: None,
            })
            .ok_or_else(|| {
                error(
                    ErrorKind::Program,
                    "Configured port must be a decimal number1–65535; no listener is opened",
                )
            })
        }
    }
}

fn snapshot_input_shape(program: &Program, input: Option<&SmallValue>) -> Result<()> {
    if input.is_some_and(|s| s.0.is_empty() || s.0.len() > 256 || s.0.chars().any(char::is_control))
        || (program.need().is_none() && input.is_some())
    {
        return Err(invalid(
            "Saved artifact input cannot have this shape for the supplied program",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    #[serde(deserialize_with = "source")]
    pub unit_source: String,
    #[serde(deserialize_with = "config")]
    pub configuration: String,
    #[serde(default)]
    pub artifact_input: Option<SmallValue>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SmallValue(#[serde(deserialize_with = "text256")] String);
impl SmallValue {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Runtime {
    Inactive {},
    TargetActive {},
    Dispatched {
        #[serde(deserialize_with = "source")]
        unit_source: String,
    },
    Running {
        snapshot: Snapshot,
    },
    Exited {
        snapshot: Snapshot,
    },
    Failed {
        snapshot: Snapshot,
    },
}
impl Runtime {
    pub fn active(&self) -> bool {
        matches!(
            self,
            Self::TargetActive {}
                | Self::Dispatched { .. }
                | Self::Running { .. }
                | Self::Exited { .. }
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    #[serde(deserialize_with = "text128")]
    pub producer: String,
    pub snapshot: Snapshot,
    pub value: SmallValue,
}
fn runtimes<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, Runtime>, D::Error> {
    map::<D, Runtime, MAX_UNITS>(d)
}
fn artifacts<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, Artifact>, D::Error> {
    map::<D, Artifact, MAX_ARTIFACTS>(d)
}
struct Names(BTreeSet<String>);
impl<'de> Deserialize<'de> for Names {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        names(d).map(Self)
    }
}
fn links<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, BTreeSet<String>>, D::Error> {
    map::<D, Names, MAX_UNITS>(d).map(|m| m.into_iter().map(|(k, v)| (k, v.0)).collect())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    schema_version: u32,
    inventory: Inventory,
    documents: BTreeMap<String, String>,
    configuration: BTreeMap<String, String>,
    loaded: BTreeMap<String, String>,
    runtime: BTreeMap<String, Runtime>,
    artifacts: BTreeMap<String, Artifact>,
    boot_links: BTreeMap<String, BTreeSet<String>>,
    default_target: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawState {
    schema_version: u32,
    inventory: Inventory,
    #[serde(deserialize_with = "documents")]
    documents: BTreeMap<String, String>,
    #[serde(deserialize_with = "configurations")]
    configuration: BTreeMap<String, String>,
    #[serde(deserialize_with = "documents")]
    loaded: BTreeMap<String, String>,
    #[serde(deserialize_with = "runtimes")]
    runtime: BTreeMap<String, Runtime>,
    #[serde(deserialize_with = "artifacts")]
    artifacts: BTreeMap<String, Artifact>,
    #[serde(deserialize_with = "links")]
    boot_links: BTreeMap<String, BTreeSet<String>>,
    #[serde(deserialize_with = "text128")]
    default_target: String,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let r = RawState::deserialize(d)?;
        let s = Self {
            schema_version: r.schema_version,
            inventory: r.inventory,
            documents: r.documents,
            configuration: r.configuration,
            loaded: r.loaded,
            runtime: r.runtime,
            artifacts: r.artifacts,
            boot_links: r.boot_links,
            default_target: r.default_target,
        };
        s.validate().map_err(D::Error::custom)?;
        Ok(s)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobKind {
    Start,
    Initialize,
    Stop,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JobResult {
    Done,
    AlreadyActive,
    Failed { kind: ErrorKind, message: String },
    DependencyFailed { unit: String },
}
impl JobResult {
    pub fn succeeded(&self) -> bool {
        matches!(self, Self::Done | Self::AlreadyActive)
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Job {
    pub unit: String,
    pub kind: JobKind,
    pub result: JobResult,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub requested: Option<String>,
    pub layers: Vec<Vec<String>>,
    pub jobs: Vec<Job>,
}
impl Report {
    pub fn requested_succeeded(&self) -> Option<bool> {
        if self.jobs.len() > MAX_UNITS || self.jobs.iter().any(|j| !unit(&j.unit)) {
            return None;
        }
        self.requested
            .as_ref()
            .filter(|id| unit(id))
            .and_then(|id| {
                self.jobs
                    .iter()
                    .find(|j| &j.unit == id)
                    .map(|j| j.result.succeeded())
            })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    SaveUnit {
        #[serde(deserialize_with = "text128")]
        unit: String,
        #[serde(deserialize_with = "source")]
        text: String,
    },
    SaveConfig {
        #[serde(deserialize_with = "text128")]
        path: String,
        #[serde(deserialize_with = "config")]
        text: String,
    },
    DaemonReload {},
    Enable {
        #[serde(deserialize_with = "text128")]
        unit: String,
    },
    Disable {
        #[serde(deserialize_with = "text128")]
        unit: String,
    },
    SetDefault {
        #[serde(deserialize_with = "text128")]
        target: String,
    },
    Start {
        #[serde(deserialize_with = "text128")]
        unit: String,
    },
    AdvancePrograms {},
    Stop {
        #[serde(deserialize_with = "text128")]
        unit: String,
    },
}

type Edges = BTreeSet<(String, String)>;
fn edges(
    definitions: &BTreeMap<String, Definition>,
    links: &BTreeMap<String, BTreeSet<String>>,
) -> Edges {
    let mut edges = Edges::new();
    for (id, d) in definitions {
        for before in &d.after {
            edges.insert((before.clone(), id.clone()));
        }
        for after in &d.before {
            edges.insert((id.clone(), after.clone()));
        }
        if matches!(d.kind, UnitType::Target) {
            for required in d
                .requires
                .iter()
                .chain(&d.wants)
                .chain(links.get(id).into_iter().flatten())
            {
                edges.insert((required.clone(), id.clone()));
            }
        }
    }
    edges
}
fn layers(nodes: &BTreeSet<String>, edges: &Edges) -> Result<Vec<Vec<String>>> {
    let mut remaining = nodes.clone();
    let mut out = vec![];
    while !remaining.is_empty() {
        let ready = remaining
            .iter()
            .filter(|n| !edges.iter().any(|(a, b)| b == *n && remaining.contains(a)))
            .cloned()
            .collect::<Vec<_>>();
        if ready.is_empty() {
            return Err(error(
                ErrorKind::Ordering,
                format!(
                    "Ordering cycle among supplied jobs: {}",
                    remaining.iter().cloned().collect::<Vec<_>>().join(", ")
                ),
            ));
        }
        for n in &ready {
            remaining.remove(n);
        }
        out.push(ready);
    }
    Ok(out)
}
fn precedes(before: &str, after: &str, edges: &Edges) -> bool {
    if before == after {
        return false;
    }
    let mut pending = vec![before.to_owned()];
    let mut visited = BTreeSet::new();
    while let Some(at) = pending.pop() {
        if !visited.insert(at.clone()) {
            continue;
        }
        for (_, next) in edges.iter().filter(|(from, _)| from == &at) {
            if next == after {
                return true;
            }
            pending.push(next.clone());
        }
    }
    false
}
fn known_report(requested: Option<String>, layers: Vec<Vec<String>>, jobs: Vec<Job>) -> Report {
    Report {
        requested,
        layers,
        jobs,
    }
}
impl State {
    pub fn new(
        inventory: Inventory,
        documents: BTreeMap<String, String>,
        configuration: BTreeMap<String, String>,
        loaded: BTreeMap<String, String>,
        default_target: String,
    ) -> Result<Self> {
        let s = Self {
            schema_version: 1,
            inventory,
            documents,
            configuration,
            loaded,
            runtime: BTreeMap::new(),
            artifacts: BTreeMap::new(),
            boot_links: BTreeMap::new(),
            default_target,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn from_json(text: &str) -> Result<Self> {
        if text.len() > MAX_JSON_BYTES {
            return Err(error(ErrorKind::Limit, "Virtual unit state JSON limit"));
        }
        serde_json::from_str(text).map_err(|e| {
            error(
                ErrorKind::Invalid,
                format!("Invalid virtual unit state: {e}"),
            )
        })
    }
    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }
    pub fn documents(&self) -> &BTreeMap<String, String> {
        &self.documents
    }
    pub fn configuration(&self) -> &BTreeMap<String, String> {
        &self.configuration
    }
    pub fn loaded_sources(&self) -> &BTreeMap<String, String> {
        &self.loaded
    }
    pub fn runtime(&self) -> &BTreeMap<String, Runtime> {
        &self.runtime
    }
    pub fn artifacts(&self) -> &BTreeMap<String, Artifact> {
        &self.artifacts
    }
    pub fn boot_links(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.boot_links
    }
    pub fn default_target(&self) -> &str {
        &self.default_target
    }
    pub fn same_context(&self, initial: &Self) -> bool {
        self.validate().is_ok() && initial.validate().is_ok() && self.inventory == initial.inventory
    }
    fn parse_sources(
        &self,
        sources: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, Definition>> {
        let mut definitions = BTreeMap::new();
        let mut producer_units = BTreeSet::new();
        for (id, source) in sources {
            let d = parse_unit(&self.inventory, id, source)?;
            if let UnitType::Service { exec, .. } = &d.kind
                && let Program::ValueTask { artifact, .. } = &self.inventory.programs[exec]
                && !producer_units.insert(artifact)
            {
                return Err(error(
                    ErrorKind::Unsupported,
                    "Multiple loaded units cannot publish the same fictional artifact",
                ));
            }
            definitions.insert(id.clone(), d);
        }
        layers(
            &self.inventory.units,
            &edges(&definitions, &self.boot_links),
        )?;
        Ok(definitions)
    }
    pub fn definitions(&self) -> Result<BTreeMap<String, Definition>> {
        self.validate()?;
        self.parse_sources(&self.loaded)
    }
    pub fn validate_documents(&self) -> Result<()> {
        self.validate()?;
        self.parse_sources(&self.documents).map(|_| ())
    }
    fn snapshot<'a>(
        &'a self,
        id: &str,
        snapshot: &'a Snapshot,
    ) -> Result<(Definition, &'a Program, ProgramResult)> {
        if snapshot.unit_source.len() > MAX_UNIT_BYTES
            || snapshot.configuration.len() > MAX_CONFIG_BYTES
            || snapshot
                .artifact_input
                .as_ref()
                .is_some_and(|s| s.0.len() > 256)
        {
            return Err(error(ErrorKind::Limit, "Execution snapshot limit"));
        }
        let d = parse_unit(&self.inventory, id, &snapshot.unit_source)?;
        let UnitType::Service { exec, .. } = &d.kind else {
            return Err(invalid(
                "A target cannot carry a fictional program snapshot",
            ));
        };
        let p = &self.inventory.programs[exec];
        snapshot_input_shape(p, snapshot.artifact_input.as_ref())?;
        let result = evaluate(
            p,
            &snapshot.configuration,
            snapshot.artifact_input.as_ref().map(SmallValue::as_str),
        )?;
        Ok((d, p, result))
    }
    pub fn validate(&self) -> Result<()> {
        // Cheap bounds/scalars precede graph derivation, snapshots or State clones.
        if self.schema_version != 1
            || self.documents.len() > MAX_UNITS
            || self.loaded.len() > MAX_UNITS
            || self.configuration.len() > MAX_CONFIGS
            || self.runtime.len() > MAX_UNITS
            || self.artifacts.len() > MAX_ARTIFACTS
            || self.boot_links.len() > MAX_UNITS
            || self
                .documents
                .iter()
                .chain(&self.loaded)
                .any(|(k, v)| !unit(k) || v.len() > MAX_UNIT_BYTES)
            || self
                .configuration
                .iter()
                .any(|(k, v)| !path(k) || v.len() > MAX_CONFIG_BYTES)
            || self.runtime.keys().any(|k| !unit(k))
            || self.artifacts.keys().any(|k| !identifier(k))
            || self
                .boot_links
                .iter()
                .any(|(k, v)| !unit(k) || v.len() > MAX_UNITS || v.iter().any(|s| !unit(s)))
        {
            return Err(error(
                ErrorKind::Limit,
                "Invalid virtual manager shape/text limits",
            ));
        }
        self.inventory.validate()?;
        if self.documents.keys().cloned().collect::<BTreeSet<_>>() != self.inventory.units
            || self.configuration.keys().cloned().collect::<BTreeSet<_>>()
                != self.inventory.config_paths
            || !self.inventory.units.contains(&self.default_target)
            || !self.default_target.ends_with(".target")
            || self
                .loaded
                .keys()
                .chain(self.runtime.keys())
                .any(|k| !self.inventory.units.contains(k))
            || self
                .artifacts
                .keys()
                .any(|k| !self.inventory.artifact_ids.contains(k))
            || self.boot_links.iter().any(|(target, units)| {
                !target.ends_with(".target")
                    || !self.inventory.units.contains(target)
                    || units.iter().any(|u| !self.inventory.units.contains(u))
            })
        {
            return Err(invalid(
                "Foreign or missing fixed unit/config/artifact/default context",
            ));
        }
        let _ = self.parse_sources(&self.loaded)?;
        for (id, r) in &self.runtime {
            if !matches!(r, Runtime::Inactive {}) && !self.loaded.contains_key(id) {
                return Err(invalid(
                    "Live/failed unit lacks a loaded manager definition",
                ));
            }
            match r {
                Runtime::Inactive {} => {}
                Runtime::TargetActive {} => {
                    if !id.ends_with(".target") {
                        return Err(invalid("Target runtime attached to a service"));
                    }
                }
                Runtime::Dispatched { unit_source } => {
                    if unit_source.len() > MAX_UNIT_BYTES
                        || !matches!(
                            parse_unit(&self.inventory, id, unit_source)?.kind,
                            UnitType::Service {
                                kind: ServiceType::Simple,
                                ..
                            }
                        )
                    {
                        return Err(invalid("Invalid pending simple-service source"));
                    }
                }
                Runtime::Running { snapshot } => {
                    let (d, _, _) = self.snapshot(id, snapshot)?;
                    if !matches!(
                        d.kind,
                        UnitType::Service {
                            kind: ServiceType::Simple,
                            ..
                        }
                    ) {
                        return Err(invalid("Running phase needs a simple-service execution"));
                    }
                }
                Runtime::Exited { snapshot } => {
                    let (d, _, _) = self.snapshot(id, snapshot)?;
                    if !matches!(
                        d.kind,
                        UnitType::Service {
                            kind: ServiceType::Oneshot,
                            remain_after_exit: true,
                            ..
                        }
                    ) {
                        return Err(invalid(
                            "Active/exited needs a successful remaining oneshot",
                        ));
                    }
                }
                Runtime::Failed { snapshot } => {
                    let d = parse_unit(&self.inventory, id, &snapshot.unit_source)?;
                    let UnitType::Service { exec, .. } = d.kind else {
                        return Err(invalid("Target marked as failed program"));
                    };
                    snapshot_input_shape(
                        &self.inventory.programs[&exec],
                        snapshot.artifact_input.as_ref(),
                    )?;
                    if snapshot.configuration.len() > MAX_CONFIG_BYTES
                        || snapshot
                            .artifact_input
                            .as_ref()
                            .is_some_and(|s| s.0.len() > 256)
                        || evaluate(
                            &self.inventory.programs[&exec],
                            &snapshot.configuration,
                            snapshot.artifact_input.as_ref().map(SmallValue::as_str),
                        )
                        .is_ok()
                    {
                        return Err(invalid(
                            "Failed runtime must contain a bounded genuinely failing program snapshot",
                        ));
                    }
                }
            }
        }
        for (id, a) in &self.artifacts {
            if a.value.0.len() > 256 || !self.loaded.contains_key(&a.producer) {
                return Err(invalid("Invalid artifact size/producer context"));
            }
            let (_, program, result) = self.snapshot(&a.producer, &a.snapshot)?;
            if !matches!(program,Program::ValueTask{artifact,..}if artifact==id)
                || result.value.as_deref() != Some(a.value.as_str())
            {
                return Err(invalid(
                    "Artifact differs from its supplied producer and consumed data",
                ));
            }
        }
        Ok(())
    }
    pub fn needs_daemon_reload(&self, id: &str) -> Result<bool> {
        self.validate()?;
        self.known(id)?;
        Ok(self.loaded.get(id) != self.documents.get(id))
    }
    pub fn execution_differs(&self, id: &str) -> Result<bool> {
        self.validate()?;
        self.known(id)?;
        let source = match self.runtime.get(id) {
            Some(Runtime::Dispatched { unit_source }) => Some((unit_source, None)),
            Some(
                Runtime::Running { snapshot }
                | Runtime::Exited { snapshot }
                | Runtime::Failed { snapshot },
            ) => Some((&snapshot.unit_source, Some(snapshot))),
            _ => None,
        };
        let Some((source, snapshot)) = source else {
            return Ok(false);
        };
        if self.loaded.get(id) != Some(source) {
            return Ok(true);
        }
        if let Some(snapshot) = snapshot {
            let d = parse_unit(&self.inventory, id, source)?;
            let UnitType::Service { exec, .. } = d.kind else {
                unreachable!()
            };
            let p = &self.inventory.programs[&exec];
            if self.configuration.get(p.config()) != Some(&snapshot.configuration) {
                return Ok(true);
            }
            if let Some(need) = p.need() {
                return Ok(self.artifacts.get(&need.artifact).map(|a| a.value.as_str())
                    != snapshot.artifact_input.as_ref().map(SmallValue::as_str));
            }
        }
        Ok(false)
    }
    pub fn result(&self, id: &str) -> Result<Option<ProgramResult>> {
        self.validate()?;
        self.known(id)?;
        match self.runtime.get(id) {
            Some(Runtime::Running { snapshot } | Runtime::Exited { snapshot }) => self
                .snapshot(id, snapshot)
                .map(|(_, _, result)| Some(result)),
            _ => Ok(None),
        }
    }
    fn known(&self, id: &str) -> Result<()> {
        if self.inventory.units.contains(id) {
            Ok(())
        } else {
            Err(error(
                ErrorKind::Missing,
                "Unit is not in this fixed virtual inventory",
            ))
        }
    }
    fn capture(
        &self,
        id: &str,
        source: &str,
        inputs: &BTreeMap<String, String>,
    ) -> Result<Snapshot> {
        let d = parse_unit(&self.inventory, id, source)?;
        let UnitType::Service { exec, .. } = d.kind else {
            return Err(invalid("No fictional program on target"));
        };
        let p = &self.inventory.programs[&exec];
        Ok(Snapshot {
            unit_source: source.into(),
            configuration: self.configuration[p.config()].clone(),
            artifact_input: p
                .need()
                .and_then(|n| inputs.get(&n.artifact).cloned())
                .map(SmallValue),
        })
    }
    fn input_values(&self) -> BTreeMap<String, String> {
        self.artifacts
            .iter()
            .map(|(id, a)| (id.clone(), a.value.0.clone()))
            .collect()
    }
    fn start(&mut self, id: &str) -> Result<Report> {
        let definitions = self.parse_sources(&self.loaded)?;
        let mut nodes = BTreeSet::new();
        let mut pending = vec![id.to_owned()];
        while let Some(unit) = pending.pop() {
            if !nodes.insert(unit.clone()) {
                continue;
            }
            if let Some(d) = definitions.get(&unit) {
                pending.extend(d.requires.iter().chain(&d.wants).cloned());
                pending.extend(self.boot_links.get(&unit).into_iter().flatten().cloned());
            }
        }
        // A path through an unstarted unit cannot order this transaction's jobs.
        // Use the same induced graph for both scheduling and failed-Requires gates.
        let order = edges(&definitions, &self.boot_links)
            .into_iter()
            .filter(|(before, after)| nodes.contains(before) && nodes.contains(after))
            .collect();
        let schedule = layers(&nodes, &order)?;
        let mut completed = BTreeMap::<String, JobResult>::new();
        let mut jobs = vec![];
        for wave in &schedule {
            let inputs = self.input_values();
            let mut effects = vec![];
            for unit in wave {
                let Some(d) = definitions.get(unit) else {
                    effects.push((
                        unit.clone(),
                        Runtime::Inactive {},
                        None,
                        JobResult::Failed {
                            kind: ErrorKind::Missing,
                            message:
                                "Known unit has no loaded definition; repair/load its saved file"
                                    .into(),
                        },
                    ));
                    continue;
                };
                if self.runtime.get(unit).is_some_and(Runtime::active) {
                    completed.insert(unit.clone(), JobResult::AlreadyActive);
                    jobs.push(Job {
                        unit: unit.clone(),
                        kind: JobKind::Start,
                        result: JobResult::AlreadyActive,
                    });
                    continue;
                }
                if let Some(required) = d.requires.iter().find(|r| {
                    precedes(r, unit, &order) && completed.get(*r).is_some_and(|j| !j.succeeded())
                }) {
                    effects.push((
                        unit.clone(),
                        Runtime::Inactive {},
                        None,
                        JobResult::DependencyFailed {
                            unit: required.clone(),
                        },
                    ));
                    continue;
                }
                match &d.kind {
                    UnitType::Target => effects.push((
                        unit.clone(),
                        Runtime::TargetActive {},
                        None,
                        JobResult::Done,
                    )),
                    UnitType::Service {
                        kind: ServiceType::Simple,
                        ..
                    } => effects.push((
                        unit.clone(),
                        Runtime::Dispatched {
                            unit_source: self.loaded[unit].clone(),
                        },
                        None,
                        JobResult::Done,
                    )),
                    UnitType::Service {
                        kind: ServiceType::Oneshot,
                        remain_after_exit,
                        ..
                    } => {
                        let snapshot = self.capture(unit, &self.loaded[unit], &inputs)?;
                        match self.snapshot(unit, &snapshot) {
                            Ok((_, Program::ValueTask { artifact, .. }, result)) => {
                                let published = Artifact {
                                    producer: unit.clone(),
                                    snapshot: snapshot.clone(),
                                    value: SmallValue(result.value.unwrap()),
                                };
                                let output = Some((artifact.clone(), published));
                                let runtime = if *remain_after_exit {
                                    Runtime::Exited { snapshot }
                                } else {
                                    Runtime::Inactive {}
                                };
                                effects.push((unit.clone(), runtime, output, JobResult::Done));
                            }
                            Ok(_) => return Err(invalid("Oneshot descriptor changed kind")),
                            Err(e) => effects.push((
                                unit.clone(),
                                Runtime::Failed { snapshot },
                                None,
                                JobResult::Failed {
                                    kind: e.kind,
                                    message: e.message,
                                },
                            )),
                        }
                    }
                }
            }
            for (unit, runtime, artifact, result) in effects {
                self.runtime.insert(unit.clone(), runtime);
                if let Some((id, a)) = artifact {
                    self.artifacts.insert(id, a);
                }
                completed.insert(unit.clone(), result.clone());
                jobs.push(Job {
                    unit,
                    kind: JobKind::Start,
                    result,
                });
            }
        }
        Ok(known_report(Some(id.into()), schedule, jobs))
    }
    fn advance(&mut self) -> Result<Report> {
        let inputs = self.input_values();
        let pending = self
            .runtime
            .iter()
            .filter_map(|(id, r)| {
                if let Runtime::Dispatched { unit_source } = r {
                    Some((id.clone(), unit_source.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let mut jobs = vec![];
        for (id, source) in pending {
            let snapshot = self.capture(&id, &source, &inputs)?;
            let (result, runtime) = match self.snapshot(&id, &snapshot) {
                Ok(_) => (JobResult::Done, Runtime::Running { snapshot }),
                Err(e) => (
                    JobResult::Failed {
                        kind: e.kind,
                        message: e.message,
                    },
                    Runtime::Failed { snapshot },
                ),
            };
            self.runtime.insert(id.clone(), runtime);
            jobs.push(Job {
                unit: id,
                kind: JobKind::Initialize,
                result,
            });
        }
        let layer = jobs.iter().map(|j| j.unit.clone()).collect::<Vec<_>>();
        Ok(known_report(
            None,
            if layer.is_empty() {
                vec![]
            } else {
                vec![layer]
            },
            jobs,
        ))
    }
    fn stop(&mut self, id: &str) -> Result<Report> {
        let definitions = self.parse_sources(&self.loaded)?;
        let mut nodes = BTreeSet::from([id.to_owned()]);
        loop {
            let before = nodes.len();
            for (unit, d) in &definitions {
                if self.runtime.get(unit).is_some_and(Runtime::active)
                    && d.requires.iter().any(|r| nodes.contains(r))
                {
                    nodes.insert(unit.clone());
                }
            }
            if nodes.len() == before {
                break;
            }
        }
        let reverse = edges(&definitions, &self.boot_links)
            .into_iter()
            .map(|(a, b)| (b, a))
            .collect();
        let schedule = layers(&nodes, &reverse)?;
        let mut jobs = vec![];
        for wave in &schedule {
            for unit in wave {
                self.runtime.insert(unit.clone(), Runtime::Inactive {});
                jobs.push(Job {
                    unit: unit.clone(),
                    kind: JobKind::Stop,
                    result: JobResult::Done,
                });
            }
        }
        Ok(known_report(Some(id.into()), schedule, jobs))
    }
    pub fn apply(&mut self, action: &Action) -> Result<Report> {
        self.validate()?;
        match action {
            Action::SaveUnit { unit, text } => {
                self.known(unit)?;
                if text.len() > MAX_UNIT_BYTES {
                    return Err(error(ErrorKind::Limit, "Unit document exceeds8192 bytes"));
                }
            }
            Action::SaveConfig { path, text } => {
                if !self.inventory.config_paths.contains(path) {
                    return Err(invalid("Unknown fixed configuration path"));
                }
                if text.len() > MAX_CONFIG_BYTES {
                    return Err(error(
                        ErrorKind::Limit,
                        "Application config exceeds4096 bytes",
                    ));
                }
            }
            Action::Enable { unit }
            | Action::Disable { unit }
            | Action::Start { unit }
            | Action::Stop { unit } => self.known(unit)?,
            Action::SetDefault { target } => {
                self.known(target)?;
                if !target.ends_with(".target") {
                    return Err(invalid("Default must name a supplied target"));
                }
            }
            _ => {}
        }
        let definitions = if matches!(
            action,
            Action::DaemonReload {} | Action::Enable { .. } | Action::Disable { .. }
        ) {
            Some(self.parse_sources(&self.documents)?)
        } else {
            None
        };
        if let Action::Enable { unit } = action
            && definitions.as_ref().unwrap()[unit].wanted_by.is_empty()
        {
            return Err(error(
                ErrorKind::Unsupported,
                "This unit has no WantedBy install targets; it is not enabled by inventing a link",
            ));
        }
        let mut next = self.clone();
        let mut report = known_report(None, vec![], vec![]);
        match action {
            Action::SaveUnit { unit, text } => {
                next.documents.insert(unit.clone(), text.clone());
            }
            Action::SaveConfig { path, text } => {
                next.configuration.insert(path.clone(), text.clone());
            }
            Action::DaemonReload {} => next.loaded = next.documents.clone(),
            Action::Enable { unit } => {
                next.loaded = next.documents.clone();
                for target in &definitions.as_ref().unwrap()[unit].wanted_by {
                    next.boot_links
                        .entry(target.clone())
                        .or_default()
                        .insert(unit.clone());
                }
            }
            Action::Disable { unit } => {
                next.loaded = next.documents.clone();
                for linked in next.boot_links.values_mut() {
                    linked.remove(unit);
                }
                next.boot_links.retain(|_, v| !v.is_empty());
            }
            Action::SetDefault { target } => next.default_target = target.clone(),
            Action::Start { unit } => report = next.start(unit)?,
            Action::AdvancePrograms {} => report = next.advance()?,
            Action::Stop { unit } => report = next.stop(unit)?,
        }
        next.validate()?;
        *self = next;
        Ok(report)
    }
    pub fn fresh_session(&self) -> Result<(Self, Report)> {
        self.validate()?;
        self.parse_sources(&self.documents)?;
        let mut next = self.clone();
        next.runtime.clear();
        next.artifacts.clear();
        next.loaded = next.documents.clone();
        let target = next.default_target.clone();
        let report = next.start(&target)?;
        next.validate()?;
        Ok((next, report))
    }
    /// Inspect a path in the complete declared graph, including inactive units.
    /// This is not a guarantee that a particular start transaction includes that
    /// path, nor that a program has become ready.
    pub fn ordered_before(&self, before: &str, after: &str) -> Result<bool> {
        self.validate()?;
        self.known(before)?;
        self.known(after)?;
        Ok(precedes(
            before,
            after,
            &edges(&self.parse_sources(&self.loaded)?, &self.boot_links),
        ))
    }
}
