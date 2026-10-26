//! A closed Apache2.4 configuration/static-request model, not a server or wire parser.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, MapAccess, SeqAccess, Visitor},
};
use std::{collections::BTreeMap, fmt, io::Write, marker::PhantomData};

pub const MAX_CONFIG: usize = 16_384;
pub const MAX_LINES: usize = 256;
pub const MAX_LINE: usize = 512;
pub const MAX_VHOSTS: usize = 8;
pub const MAX_ALIASES: usize = 64;
pub const MAX_DIRECTORIES: usize = 32;
pub const MAX_ENTRIES: usize = 128;
pub const MAX_FILES: usize = 96;
pub const MAX_FILE: usize = 4096;
pub const MAX_FILE_TEXT: usize = 262_144;
pub const MAX_PATH: usize = 160;
pub const MAX_STATE_JSON: usize = 2_097_152;
pub const MAX_REQUEST_JSON: usize = 8192;
pub const MAX_ACTION_JSON: usize = 131_072;
pub const MAX_DIAGNOSTIC: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum FaultKind {
    Limit,
    Syntax,
    Unsupported,
    Invalid,
    Context,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fault {
    pub kind: FaultKind,
    pub message: String,
}
fn fault(kind: FaultKind, message: &str) -> Fault {
    let mut n = message.len().min(MAX_DIAGNOSTIC);
    while !message.is_char_boundary(n) {
        n -= 1;
    }
    let mut text = String::with_capacity(n);
    text.push_str(&message[..n]);
    Fault {
        kind,
        message: text,
    }
}
impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Fault {}
type Result<T> = std::result::Result<T, Fault>;
fn limit(message: &str) -> Fault {
    fault(FaultKind::Limit, message)
}
fn unsupported(message: &str) -> Fault {
    fault(FaultKind::Unsupported, message)
}
fn syntax(message: &str) -> Fault {
    fault(FaultKind::Syntax, message)
}

struct Bounded<const N: usize>(String);
impl<'de, const N: usize> Deserialize<'de> for Bounded<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct S<const N: usize>;
        impl<const N: usize> Visitor<'_> for S<N> {
            type Value = Bounded<N>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most {N} UTF-8 bytes")
            }
            fn visit_str<E: serde::de::Error>(
                self,
                s: &str,
            ) -> std::result::Result<Self::Value, E> {
                if s.len() > N {
                    return Err(E::custom("Apache text limit"));
                }
                Ok(Bounded(s.to_owned()))
            }
        }
        d.deserialize_str(S::<N>)
    }
}
fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> std::result::Result<String, D::Error> {
    Ok(Bounded::<N>::deserialize(d)?.0)
}
fn map<'de, D: Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    struct M<V, const N: usize>(PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for M<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} distinct bounded path keys")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(Bounded(k)) = a.next_key::<Bounded<MAX_PATH>>()? {
                if out.len() >= N || out.contains_key(&k) {
                    return Err(A::Error::custom("Apache map count/duplicate limit"));
                }
                out.insert(k, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M::<V, N>(PhantomData))
}
fn files<'de, D: Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, String>, D::Error> {
    struct M;
    impl<'de> Visitor<'de> for M {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("bounded distinct virtual file contents")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            let mut total = 0usize;
            while let Some(Bounded(k)) = a.next_key::<Bounded<MAX_PATH>>()? {
                if out.len() >= MAX_FILES || out.contains_key(&k) {
                    return Err(A::Error::custom("Apache file count/duplicate limit"));
                }
                let Bounded(v) = a.next_value::<Bounded<MAX_FILE>>()?;
                total = total
                    .checked_add(v.len())
                    .filter(|n| *n <= MAX_FILE_TEXT)
                    .ok_or_else(|| A::Error::custom("Apache aggregate file limit"))?;
                out.insert(k, v);
            }
            Ok(out)
        }
    }
    d.deserialize_map(M)
}
fn hosts<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    struct S;
    impl<'de> Visitor<'de> for S {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most two bounded Host fields")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while out.len() < 2 {
                match a.next_element::<Bounded<260>>()? {
                    Some(Bounded(s)) => out.push(s),
                    None => return Ok(out),
                }
            }
            // Reject a third element at its deserializer entry, before reading its value.
            struct Reject;
            impl<'de> Deserialize<'de> for Reject {
                fn deserialize<D: Deserializer<'de>>(_: D) -> std::result::Result<Self, D::Error> {
                    Err(D::Error::custom("Apache Host count limit"))
                }
            }
            let _ = a.next_element::<Reject>()?;
            Ok(out)
        }
    }
    d.deserialize_seq(S)
}
struct Counter {
    n: usize,
    cap: usize,
}
impl Write for Counter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.n = self
            .n
            .checked_add(b.len())
            .filter(|n| *n <= self.cap)
            .ok_or_else(|| std::io::Error::other("Apache JSON size limit"))?;
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn json_size<T: Serialize + ?Sized>(v: &T) -> Result<usize> {
    let mut c = Counter {
        n: 0,
        cap: MAX_STATE_JSON,
    };
    serde_json::to_writer(&mut c, v).map_err(|_| limit("Apache serialized size bound"))?;
    Ok(c.n)
}
fn identity(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}
fn unreserved(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b"-._~".contains(&b)
}
fn path(s: &str) -> bool {
    if s == "/" {
        return true;
    }
    s.starts_with('/')
        && s.len() <= MAX_PATH
        && !s.ends_with('/')
        && s[1..].split('/').count() <= 8
        && s[1..]
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != ".." && p.bytes().all(unreserved))
}
fn parent(s: &str) -> &str {
    match s.rsplit_once('/') {
        Some(("", _)) => "/",
        Some((p, _)) => p,
        None => "/",
    }
}
fn under(child: &str, ancestor: &str) -> bool {
    ancestor == "/"
        || child == ancestor
        || child
            .strip_prefix(ancestor)
            .is_some_and(|tail| tail.starts_with('/'))
}
fn dns_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 253
        && s.parse::<std::net::Ipv4Addr>().is_err()
        && s.split('.').all(|p| {
            !p.is_empty()
                && p.len() <= 63
                && p.as_bytes()[0].is_ascii_alphanumeric()
                && p.as_bytes()[p.len() - 1].is_ascii_alphanumeric()
                && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
        })
}
fn source_bounds(s: &str) -> Result<()> {
    if s.len() > MAX_CONFIG
        || s.split_terminator('\n').count() > MAX_LINES
        || s.split_terminator('\n')
            .any(|l| l.strip_suffix('\r').unwrap_or(l).len() > MAX_LINE)
    {
        return Err(limit("Config exceeds 16KiB,256 lines or512 bytes per line"));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntryKind {
    Directory,
    File,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    #[serde(deserialize_with = "text::<_,32>")]
    pub fixture: String,
    #[serde(deserialize_with = "text::<_,32>")]
    pub server: String,
    #[serde(deserialize_with = "text::<_,MAX_PATH>")]
    pub config_path: String,
    #[serde(deserialize_with = "map::<_,EntryKind,MAX_ENTRIES>")]
    pub entries: BTreeMap<String, EntryKind>,
}
impl Inventory {
    pub fn validate(&self) -> Result<()> {
        if !identity(&self.fixture)
            || !identity(&self.server)
            || !path(&self.config_path)
            || self.config_path == "/"
        {
            return Err(fault(
                FaultKind::Invalid,
                "Invalid Apache fixture,server or virtual config path",
            ));
        }
        if self.entries.len() > MAX_ENTRIES
            || self
                .entries
                .values()
                .filter(|v| **v == EntryKind::File)
                .count()
                > MAX_FILES
        {
            return Err(limit("Apache inventory count bound"));
        }
        if self.entries.get("/") != Some(&EntryKind::Directory)
            || self.entries.contains_key(&self.config_path)
        {
            return Err(fault(
                FaultKind::Context,
                "Tree needs directory root and a separate config slot",
            ));
        }
        for p in self.entries.keys() {
            if !path(p) || (p != "/" && self.entries.get(parent(p)) != Some(&EntryKind::Directory))
            {
                return Err(fault(
                    FaultKind::Context,
                    "Invalid virtual path or missing directory parent",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Active {
    #[serde(deserialize_with = "text::<_,MAX_CONFIG>")]
    pub source: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    pub inventory: Inventory,
    pub saved: String,
    pub files: BTreeMap<String, String>,
    pub active: Option<Active>,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            inventory: Inventory,
            #[serde(deserialize_with = "text::<_,MAX_CONFIG>")]
            saved: String,
            #[serde(deserialize_with = "files")]
            files: BTreeMap<String, String>,
            active: Option<Active>,
        }
        let w = Wire::deserialize(d)?;
        let s = Self {
            inventory: w.inventory,
            saved: w.saved,
            files: w.files,
            active: w.active,
        };
        s.validate().map_err(D::Error::custom)?;
        Ok(s)
    }
}
impl State {
    pub fn new(
        inventory: Inventory,
        saved: String,
        files: BTreeMap<String, String>,
    ) -> Result<Self> {
        let s = Self {
            inventory,
            saved,
            files,
            active: None,
        };
        s.validate()?;
        Ok(s)
    }
    fn validated_config(&self) -> Result<Option<Configuration>> {
        self.inventory.validate()?;
        source_bounds(&self.saved)?;
        if self.files.len() > MAX_FILES {
            return Err(limit("Apache file count bound"));
        }
        let mut total = 0usize;
        for (p, t) in &self.files {
            if !path(p) || self.inventory.entries.get(p) != Some(&EntryKind::File) {
                return Err(fault(FaultKind::Context, "Foreign virtual file contents"));
            }
            if t.len() > MAX_FILE {
                return Err(limit("Virtual file exceeds4KiB"));
            }
            total = total
                .checked_add(t.len())
                .filter(|n| *n <= MAX_FILE_TEXT)
                .ok_or_else(|| limit("Aggregate virtual file text exceeds256KiB"))?;
        }
        if self.files.len()
            != self
                .inventory
                .entries
                .values()
                .filter(|v| **v == EntryKind::File)
                .count()
        {
            return Err(fault(
                FaultKind::Context,
                "Missing fixed virtual file contents",
            ));
        }
        if let Some(a) = &self.active {
            source_bounds(&a.source)?;
        }
        json_size(self)?;
        self.active.as_ref().map(|a| parse(&a.source)).transpose()
    }
    pub fn validate(&self) -> Result<()> {
        self.validated_config().map(|_| ())
    }
    pub fn valid_in(&self, expected: &Inventory) -> Result<()> {
        expected.validate()?;
        if &self.inventory != expected {
            return Err(fault(FaultKind::Context, "Foreign Apache inventory"));
        }
        self.validate()
    }
    pub fn decode(bytes: &[u8], expected: &Inventory) -> Result<Self> {
        if bytes.len() > MAX_STATE_JSON {
            return Err(limit("Apache raw State exceeds2MiB"));
        }
        expected.validate()?;
        let s: Self = serde_json::from_slice(bytes)
            .map_err(|_| fault(FaultKind::Invalid, "Invalid bounded Apache State JSON"))?;
        s.valid_in(expected)?;
        Ok(s)
    }
    fn replacement<T: Serialize + ?Sized, U: Serialize + ?Sized>(
        &self,
        old: &T,
        new: &U,
    ) -> Result<()> {
        let n = json_size(self)?
            .checked_sub(json_size(old)?)
            .and_then(|n| n.checked_add(json_size(new).ok()?))
            .ok_or_else(|| limit("Apache replacement size overflow"))?;
        if n > MAX_STATE_JSON {
            return Err(limit("Apache replacement exceeds State size bound"));
        }
        Ok(())
    }
    /// All validation and projected-size checks finish before any changed text is cloned.
    pub fn apply(&mut self, action: &Action) -> Result<bool> {
        action.validate()?;
        self.validate()?;
        match action {
            Action::SaveConfig { text } => {
                self.replacement(&self.saved, text)?;
                if self.saved == *text {
                    return Ok(false);
                }
                self.saved = text.clone();
            }
            Action::WriteFile { path, text } => {
                let old = self.files.get(path).ok_or_else(|| {
                    fault(
                        FaultKind::Context,
                        "Write requires a fixed regular-file identity",
                    )
                })?;
                let total: usize = self.files.values().map(String::len).sum();
                if total - old.len() + text.len() > MAX_FILE_TEXT {
                    return Err(limit("Write exceeds aggregate file text bound"));
                }
                self.replacement(old, text)?;
                if old == text {
                    return Ok(false);
                }
                self.files.insert(path.clone(), text.clone());
            }
            Action::Start {} | Action::Reload {} => {
                if matches!(action, Action::Start {}) && self.active.is_some() {
                    return Ok(false);
                }
                if matches!(action, Action::Reload {}) && self.active.is_none() {
                    return Err(unsupported("Logical Reload requires a running server"));
                }
                parse(&self.saved)?;
                #[derive(Serialize)]
                struct BorrowedActive<'a> {
                    source: &'a str,
                }
                self.replacement(
                    &self.active,
                    &Some(BorrowedActive {
                        source: &self.saved,
                    }),
                )?;
                if self.active.as_ref().is_some_and(|a| a.source == self.saved) {
                    return Ok(false);
                }
                self.active = Some(Active {
                    source: self.saved.clone(),
                });
            }
            Action::Stop {} => {
                if self.active.is_none() {
                    return Ok(false);
                }
                self.active = None;
            }
        }
        Ok(true)
    }
    pub fn check_config(&self) -> Result<Configuration> {
        self.validate()?;
        parse(&self.saved)
    }
    pub fn active_config(&self) -> Result<Option<Configuration>> {
        self.validated_config()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    SaveConfig {
        #[serde(deserialize_with = "text::<_,MAX_CONFIG>")]
        text: String,
    },
    WriteFile {
        #[serde(deserialize_with = "text::<_,MAX_PATH>")]
        path: String,
        #[serde(deserialize_with = "text::<_,MAX_FILE>")]
        text: String,
    },
    Start {},
    Stop {},
    Reload {},
}
impl Action {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::SaveConfig { text } => source_bounds(text),
            Self::WriteFile { path: p, text } => {
                if !path(p) {
                    return Err(fault(FaultKind::Invalid, "Invalid virtual file path"));
                }
                if text.len() > MAX_FILE {
                    return Err(limit("Virtual file exceeds4KiB"));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_ACTION_JSON {
            return Err(limit("Apache raw action bound"));
        }
        let a: Self = serde_json::from_slice(bytes)
            .map_err(|_| fault(FaultKind::Invalid, "Invalid bounded Apache action JSON"))?;
        a.validate()?;
        Ok(a)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Access {
    Granted,
    Denied,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Directory {
    pub path: String,
    pub access: Access,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct VirtualHost {
    pub name: String,
    pub aliases: Vec<String>,
    pub document_root: String,
    /// Empty means DirectoryIndex disabled, not an omitted default.
    pub indexes: Vec<String>,
    pub directories: Vec<Directory>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Configuration {
    pub listen: u16,
    pub root_access: Access,
    pub hosts: Vec<VirtualHost>,
}
#[derive(Default)]
struct HostBuilder {
    port: u16,
    name: Option<String>,
    aliases: Option<Vec<String>>,
    root: Option<String>,
    indexes: Option<Vec<String>>,
    dirs: Vec<Directory>,
}
struct DirBuilder {
    path: String,
    access: Option<Access>,
    options: bool,
    overrides: bool,
    global: bool,
}
fn tokens(mut line: &str) -> Result<Vec<&str>> {
    let mut out = Vec::new();
    while !line.is_empty() {
        line = line.trim_start_matches([' ', '\t']);
        if line.is_empty() {
            break;
        }
        if out.len() >= 10 {
            return Err(limit("Too many config arguments"));
        }
        if line.starts_with('"') {
            let tail = &line[1..];
            let end = tail
                .find('"')
                .ok_or_else(|| syntax("Unclosed config quote"))?;
            out.push(&tail[..end]);
            line = &tail[end + 1..];
            if !line.is_empty() && !line.starts_with([' ', '\t']) {
                return Err(unsupported(
                    "Adjacent quoted arguments are outside this config subset",
                ));
            }
        } else {
            let end = line.find([' ', '\t']).unwrap_or(line.len());
            let token = &line[..end];
            if token.contains('"') {
                return Err(unsupported(
                    "Quotes must enclose one entire literal argument",
                ));
            }
            out.push(token);
            line = &line[end..];
        }
    }
    Ok(out)
}
fn port(s: &str) -> Result<u16> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(syntax("Port requires decimal digits"));
    }
    s.parse::<u16>()
        .ok()
        .filter(|p| *p != 0)
        .ok_or_else(|| syntax("Port requires an integer1..65535"))
}
fn access(t: &[&str]) -> Result<Access> {
    match t {
        [_, "all", "granted"] => Ok(Access::Granted),
        [_, "all", "denied"] => Ok(Access::Denied),
        _ => Err(unsupported(
            "Only one Require all granted|denied is supported",
        )),
    }
}
fn new_dir(p: &str, global: bool) -> Result<DirBuilder> {
    if !path(p) || (global && p != "/") {
        return Err(unsupported(
            "Only canonical literal Directory paths and the global root baseline are supported",
        ));
    }
    Ok(DirBuilder {
        path: p.into(),
        access: None,
        options: false,
        overrides: false,
        global,
    })
}
/// Parses only the disclosed closure. Valid Apache forms outside it are Unsupported.
pub fn parse(source: &str) -> Result<Configuration> {
    source_bounds(source)?;
    if source
        .bytes()
        .any(|b| b.is_ascii_control() && !b"\t\r\n".contains(&b))
    {
        return Err(syntax("Control byte in config"));
    }
    let mut listen = None;
    let mut root_access = None;
    let mut hosts = Vec::new();
    let mut host: Option<HostBuilder> = None;
    let mut dir: Option<DirBuilder> = None;
    let mut dir_count = 0usize;
    let mut alias_count = 0usize;
    for raw in source.split('\n') {
        let line = raw
            .strip_suffix('\r')
            .unwrap_or(raw)
            .trim_matches([' ', '\t']);
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.contains(['\\', '\'', '#']) || line.contains("${") {
            return Err(unsupported(
                "Escapes, single quotes,inline comments and interpolation are unsupported",
            ));
        }
        if line.starts_with('<') {
            if !line.ends_with('>') {
                return Err(syntax("Unclosed config container"));
            }
            let t = tokens(&line[1..line.len() - 1])?;
            let Some(tag) = t.first() else {
                return Err(syntax("Empty config container"));
            };
            if tag.eq_ignore_ascii_case("VirtualHost") {
                if dir.is_some() || host.is_some() || t.len() != 2 {
                    return Err(syntax("VirtualHost nesting/arity"));
                }
                if hosts.len() >= MAX_VHOSTS {
                    return Err(limit("At most8 VirtualHosts"));
                }
                let p = t[1].strip_prefix("*:").ok_or_else(|| {
                    unsupported("Only one explicit *:PORT address per vhost is supported")
                })?;
                host = Some(HostBuilder {
                    port: port(p)?,
                    ..Default::default()
                });
            } else if tag.eq_ignore_ascii_case("/VirtualHost") {
                if dir.is_some() || t.len() != 1 {
                    return Err(syntax("Unclosed Directory or invalid VirtualHost closer"));
                }
                let h = host
                    .take()
                    .ok_or_else(|| syntax("Unexpected VirtualHost closer"))?;
                hosts.push((
                    h.port,
                    VirtualHost {
                        name: h.name.ok_or_else(|| {
                            unsupported("This subset requires explicit ServerName")
                        })?,
                        aliases: h.aliases.unwrap_or_default(),
                        document_root: h.root.ok_or_else(|| {
                            unsupported("This subset requires explicit DocumentRoot")
                        })?,
                        indexes: h.indexes.unwrap_or_else(|| vec!["index.html".into()]),
                        directories: h.dirs,
                    },
                ));
            } else if tag.eq_ignore_ascii_case("Directory") {
                if dir.is_some() || t.len() != 2 {
                    return Err(syntax("Directory nesting/arity"));
                }
                if dir_count >= MAX_DIRECTORIES {
                    return Err(limit(
                        "At most32 Directory sections including the root baseline",
                    ));
                }
                if let Some(h) = &host {
                    if h.dirs.len() >= 4 {
                        return Err(limit("At most4 Directory sections per vhost"));
                    }
                    if h.dirs.iter().any(|d| d.path == t[1]) {
                        return Err(unsupported(
                            "Repeated same-path Directory sections are outside this subset",
                        ));
                    }
                } else if root_access.is_some() {
                    return Err(unsupported(
                        "Only one global root Directory baseline is supported",
                    ));
                }
                dir = Some(new_dir(t[1], host.is_none())?);
                dir_count += 1;
            } else if tag.eq_ignore_ascii_case("/Directory") {
                if t.len() != 1 {
                    return Err(syntax("Invalid Directory closer"));
                }
                let d = dir
                    .take()
                    .ok_or_else(|| syntax("Unexpected Directory closer"))?;
                let a = d
                    .access
                    .ok_or_else(|| unsupported("Each Directory needs one explicit Require rule"))?;
                if d.global {
                    if !d.options || !d.overrides {
                        return Err(unsupported(
                            "Root baseline needs Options None and AllowOverride None",
                        ));
                    }
                    root_access = Some(a);
                } else {
                    host.as_mut()
                        .ok_or_else(|| syntax("Missing Directory host"))?
                        .dirs
                        .push(Directory {
                            path: d.path,
                            access: a,
                        });
                }
            } else {
                return Err(unsupported("Unmodeled Apache container"));
            }
            continue;
        }
        let t = tokens(line)?;
        let Some(key) = t.first() else {
            continue;
        };
        if let Some(d) = &mut dir {
            if key.eq_ignore_ascii_case("Require") {
                if d.access.is_some() {
                    return Err(unsupported(
                        "Repeated Require/container authorization is unsupported",
                    ));
                }
                d.access = Some(access(&t)?);
            } else if key.eq_ignore_ascii_case("Options") && d.global {
                if t.as_slice() != [*key, "None"] || d.options {
                    return Err(unsupported("Only one root Options None is supported"));
                }
                d.options = true;
            } else if key.eq_ignore_ascii_case("AllowOverride") && d.global {
                if t.as_slice() != [*key, "None"] || d.overrides {
                    return Err(unsupported("Only one root AllowOverride None is supported"));
                }
                d.overrides = true;
            } else {
                return Err(unsupported("Unmodeled Directory directive"));
            }
        } else if let Some(h) = &mut host {
            if key.eq_ignore_ascii_case("ServerName") {
                if h.name.is_some() || t.len() != 2 || !dns_name(t[1]) {
                    return Err(unsupported("One literal DNS ServerName is required"));
                }
                h.name = Some(t[1].into());
            } else if key.eq_ignore_ascii_case("ServerAlias") {
                if h.aliases.is_some()
                    || !(2..=9).contains(&t.len())
                    || t[1..].iter().any(|s| !dns_name(s))
                {
                    return Err(unsupported(
                        "One ServerAlias list of1..8 literal DNS names is supported",
                    ));
                }
                alias_count += t.len() - 1;
                if alias_count > MAX_ALIASES {
                    return Err(limit("Aggregate alias bound"));
                }
                h.aliases = Some(t[1..].iter().map(|s| (*s).into()).collect());
            } else if key.eq_ignore_ascii_case("DocumentRoot") {
                if h.root.is_some() || t.len() != 2 || !path(t[1]) {
                    return Err(unsupported(
                        "One canonical absolute DocumentRoot is required",
                    ));
                }
                h.root = Some(t[1].into());
            } else if key.eq_ignore_ascii_case("DirectoryIndex") {
                if h.indexes.is_some() || !(2..=5).contains(&t.len()) {
                    return Err(unsupported(
                        "Only one DirectoryIndex list with1..4 basenames is supported",
                    ));
                }
                if t.len() == 2 && t[1].eq_ignore_ascii_case("disabled") {
                    h.indexes = Some(Vec::new());
                } else {
                    if t[1..].iter().any(|s| {
                        s.is_empty()
                            || s.len() > 64
                            || *s == "."
                            || *s == ".."
                            || s.eq_ignore_ascii_case("disabled")
                            || !s.bytes().all(unreserved)
                    }) {
                        return Err(unsupported(
                            "DirectoryIndex accepts only literal file basenames or sole disabled",
                        ));
                    }
                    h.indexes = Some(t[1..].iter().map(|s| (*s).into()).collect());
                }
            } else {
                return Err(unsupported("Unmodeled VirtualHost directive"));
            }
        } else if key.eq_ignore_ascii_case("Listen") {
            if listen.is_some() || t.len() != 2 {
                return Err(unsupported("Exactly one simple Listen PORT is supported"));
            }
            listen = Some(port(t[1])?);
        } else {
            return Err(unsupported("Unmodeled global Apache directive"));
        }
    }
    if host.is_some() || dir.is_some() {
        return Err(syntax("Unclosed config container"));
    }
    let listen = listen.ok_or_else(|| unsupported("This subset requires one explicit Listen"))?;
    let root_access = root_access
        .ok_or_else(|| unsupported("This subset requires the explicit root Directory baseline"))?;
    if hosts.is_empty() {
        return Err(unsupported(
            "This subset requires at least one explicit VirtualHost",
        ));
    }
    if hosts.iter().any(|(p, _)| *p != listen) {
        return Err(unsupported(
            "Mixed ports/main-server fallback are outside this one-port subset",
        ));
    }
    Ok(Configuration {
        listen,
        root_access,
        hosts: hosts.into_iter().map(|(_, h)| h).collect(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Method {
    #[serde(rename = "GET")]
    Get,
    #[serde(rename = "HEAD")]
    Head,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    #[serde(deserialize_with = "text::<_,32>")]
    pub server: String,
    pub port: u16,
    pub method: Method,
    #[serde(deserialize_with = "hosts")]
    pub hosts: Vec<String>,
    #[serde(deserialize_with = "text::<_,256>")]
    pub path: String,
}
impl Request {
    pub(crate) fn shape(&self) -> Result<()> {
        if !identity(&self.server) || self.port == 0 {
            return Err(fault(FaultKind::Invalid, "Invalid virtual destination"));
        }
        if self.hosts.len() > 2 || self.hosts.iter().any(|h| h.len() > 260) || self.path.len() > 256
        {
            return Err(limit("Virtual request field/count bound"));
        }
        Ok(())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > MAX_REQUEST_JSON {
            return Err(limit("Virtual request raw size bound"));
        }
        let r: Self = serde_json::from_slice(bytes)
            .map_err(|_| fault(FaultKind::Invalid, "Invalid bounded request JSON"))?;
        r.shape()?;
        Ok(r)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum HttpStatus {
    Ok,
    MovedPermanently,
    BadRequest,
    Forbidden,
    NotFound,
}
impl HttpStatus {
    pub fn code(self) -> u16 {
        match self {
            Self::Ok => 200,
            Self::MovedPermanently => 301,
            Self::BadRequest => 400,
            Self::Forbidden => 403,
            Self::NotFound => 404,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Cause {
    File,
    DirectorySlash,
    InvalidHost,
    AccessDenied,
    MissingResource,
    NoIndex,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Response {
    pub status: HttpStatus,
    pub cause: Cause,
    pub vhost: Option<String>,
    pub resource: Option<String>,
    pub redirect_path: Option<String>,
    pub body: String,
    pub representation_bytes: Option<usize>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum Outcome {
    NoListener,
    Http(Response),
}
fn response(
    status: HttpStatus,
    cause: Cause,
    h: Option<&VirtualHost>,
    p: Option<&str>,
) -> Response {
    Response {
        status,
        cause,
        vhost: h.map(|h| h.name.clone()),
        resource: p.map(str::to_owned),
        redirect_path: None,
        body: String::new(),
        representation_bytes: None,
    }
}
// None is a malformed field within the syntax we actually check. Valid but excluded
// reg-name/IPv6 forms return Unsupported, never a fabricated HTTP400.
fn host_name(raw: &str) -> Result<Option<&str>> {
    let s = raw.trim_matches([' ', '\t']);
    if s.bytes()
        .any(|b| b.is_ascii_control() || b.is_ascii_whitespace() || b"/\\?#@<>\"".contains(&b))
    {
        return Ok(None);
    }
    if s.is_empty() || !s.is_ascii() || s.contains(['%', '[', ']']) || s.matches(':').count() > 1 {
        return Err(unsupported("Host form is outside the literal DNS subset"));
    }
    let name = if let Some((n, p)) = s.split_once(':') {
        if n.is_empty() {
            return Ok(None);
        }
        if p.is_empty() {
            return Err(unsupported("Empty Host port is outside this subset"));
        }
        if !p.bytes().all(|b| b.is_ascii_digit()) {
            return Ok(None);
        }
        if p.parse::<u16>().is_err() {
            return Err(unsupported("Host port exceeds the supported integer shape"));
        }
        n
    } else {
        s
    };
    if !dns_name(name) {
        return Err(unsupported(
            "Host reg-name is valid or unclassified outside this DNS subset",
        ));
    }
    Ok(Some(name))
}
fn request_path(raw: &str) -> Result<&str> {
    // Check the original spelling before removing one legitimate final slash.
    // In particular, "//" must not collapse into the supported root "/".
    if raw.contains("//") {
        return Err(unsupported(
            "Duplicate slashes are outside canonical request paths",
        ));
    }
    let p = if raw == "/" {
        raw
    } else {
        raw.strip_suffix('/').unwrap_or(raw)
    };
    if !path(p) {
        return Err(unsupported(
            "Only canonical ASCII origin-form paths are supported",
        ));
    }
    Ok(p)
}
fn joined(root: &str, tail: &str) -> Result<String> {
    if tail == "/" {
        return Ok(root.to_owned());
    }
    let prefix = if root == "/" { "" } else { root };
    if prefix.len() + tail.len() > MAX_PATH {
        return Err(unsupported("Mapped virtual path exceeds the path bound"));
    }
    let mut out = String::with_capacity(prefix.len() + tail.len());
    out.push_str(prefix);
    out.push_str(tail);
    if !path(&out) {
        return Err(unsupported("Mapped virtual path exceeds the depth bound"));
    }
    Ok(out)
}
fn permission(c: &Configuration, h: &VirtualHost, resource: &str, directory: bool) -> Access {
    let anchor = if directory {
        resource
    } else {
        parent(resource)
    };
    h.directories
        .iter()
        .filter(|d| under(anchor, &d.path))
        .max_by_key(|d| d.path.len())
        .map_or(c.root_access, |d| d.access)
}
fn file_response(r: &Request, h: &VirtualHost, p: &str, text: &str) -> Outcome {
    let mut out = response(HttpStatus::Ok, Cause::File, Some(h), Some(p));
    out.representation_bytes = Some(text.len());
    if r.method == Method::Get {
        out.body = text.to_owned();
    }
    Outcome::Http(out)
}
/// Pure request observation: no time, filesystem, process, socket or grading state.
pub fn request(state: &State, r: &Request) -> Result<Outcome> {
    r.shape()?;
    RequestView::new(state)?.query(r)
}

/// One immutable, validated observation batch. It cannot outlive or mutate its
/// borrowed state and cannot be serialized or restored as grading authority.
pub(crate) struct RequestView<'a> {
    state: &'a State,
    config: Option<Configuration>,
}
impl<'a> RequestView<'a> {
    pub(crate) fn new(state: &'a State) -> Result<Self> {
        Ok(Self {
            state,
            config: state.validated_config()?,
        })
    }
    pub(crate) fn query(&self, r: &Request) -> Result<Outcome> {
        r.shape()?;
        request_in(self.state, self.config.as_ref(), r)
    }
}
fn request_in(state: &State, config: Option<&Configuration>, r: &Request) -> Result<Outcome> {
    let Some(c) = config else {
        return Ok(Outcome::NoListener);
    };
    if r.server != state.inventory.server || r.port != c.listen {
        return Ok(Outcome::NoListener);
    }
    if r.hosts.len() != 1 {
        return Ok(Outcome::Http(response(
            HttpStatus::BadRequest,
            Cause::InvalidHost,
            None,
            None,
        )));
    }
    let Some(name) = host_name(&r.hosts[0])? else {
        return Ok(Outcome::Http(response(
            HttpStatus::BadRequest,
            Cause::InvalidHost,
            None,
            None,
        )));
    };
    let h = c
        .hosts
        .iter()
        .find(|h| {
            h.name.eq_ignore_ascii_case(name)
                || h.aliases.iter().any(|a| a.eq_ignore_ascii_case(name))
        })
        .unwrap_or(&c.hosts[0]);
    let url = request_path(&r.path)?;
    if state.inventory.entries.get(&h.document_root) == Some(&EntryKind::File) {
        return Err(unsupported(
            "DocumentRoot naming a regular file is outside this subset",
        ));
    }
    let mapped = joined(&h.document_root, url)?;
    let kind = state.inventory.entries.get(&mapped);
    let allowed = permission(c, h, &mapped, kind == Some(&EntryKind::Directory));
    if allowed == Access::Denied {
        if kind.is_none() {
            return Err(unsupported(
                "Denied nonexistent-resource status precedence is not modeled",
            ));
        }
        return Ok(Outcome::Http(response(
            HttpStatus::Forbidden,
            Cause::AccessDenied,
            Some(h),
            Some(&mapped),
        )));
    }
    match kind {
        None => Ok(Outcome::Http(response(
            HttpStatus::NotFound,
            Cause::MissingResource,
            Some(h),
            Some(&mapped),
        ))),
        Some(EntryKind::File) => {
            if r.path.ends_with('/') {
                return Ok(Outcome::Http(response(
                    HttpStatus::NotFound,
                    Cause::MissingResource,
                    Some(h),
                    Some(&mapped),
                )));
            }
            Ok(file_response(r, h, &mapped, &state.files[&mapped]))
        }
        Some(EntryKind::Directory) => {
            if !r.path.ends_with('/') {
                if r.path.len() >= 256 {
                    return Err(limit("Redirect target path bound"));
                }
                let mut out = response(
                    HttpStatus::MovedPermanently,
                    Cause::DirectorySlash,
                    Some(h),
                    Some(&mapped),
                );
                out.redirect_path = Some(format!("{}/", r.path));
                return Ok(Outcome::Http(out));
            }
            for index in &h.indexes {
                let p = joined(&mapped, &format!("/{index}"))?;
                match state.inventory.entries.get(&p) {
                    Some(EntryKind::Directory) => {
                        return Err(unsupported(
                            "DirectoryIndex directory/subrequest behavior is not modeled",
                        ));
                    }
                    Some(EntryKind::File) => return Ok(file_response(r, h, &p, &state.files[&p])),
                    None => (),
                }
            }
            Ok(Outcome::Http(response(
                HttpStatus::Forbidden,
                Cause::NoIndex,
                Some(h),
                Some(&mapped),
            )))
        }
    }
}

#[cfg(test)]
mod batch_tests {
    use super::*;

    #[test]
    fn immutable_batch_matches_public_requests_and_retains_failure_precedence() {
        let raw: serde_json::Value = serde_json::from_str(include_str!(
            "../tests/fixtures/apache-static-workspace-preview.json"
        ))
        .unwrap();
        let mut state: State =
            serde_json::from_value(raw["scenarios"][0]["initial"]["simulation"].clone()).unwrap();
        let make = |host: &str, path: &str| Request {
            server: state.inventory.server.clone(),
            port: 8080,
            method: Method::Get,
            hosts: vec![host.into()],
            path: path.into(),
        };
        let mut requests = Vec::new();
        for host in [
            "report.example.test",
            "observer.example.test",
            "unknown.example.test",
            "report.example.test:9999",
            "bad host",
            "[::1]",
            "host_unsupported",
        ] {
            for path in [
                "/",
                "/index.html",
                "/private/secret.txt",
                "/missing",
                "/private",
                "/private/",
                "/index.html/",
                "//",
                "/../private",
                "/?query",
            ] {
                let r = make(host, path);
                requests.push(r.clone());
                requests.push(Request {
                    method: Method::Head,
                    ..r
                });
            }
        }
        let mut bad = make("report.example.test", "/");
        bad.port = 0;
        requests.push(bad.clone());
        for active in [true, false] {
            if !active {
                state.apply(&Action::Stop {}).unwrap();
            }
            let view = RequestView::new(&state).unwrap();
            for r in &requests {
                assert_eq!(view.query(r), request(&state, r));
            }
        }
        state.files.remove("/srv/report/index.html");
        assert!(RequestView::new(&state).is_err());
        let error = request(&state, &bad).unwrap_err();
        assert_eq!(
            error,
            bad.shape().unwrap_err(),
            "request shape precedes invalid State"
        );
        bad.port = 8080;
        assert_eq!(
            request(&state, &bad).unwrap_err(),
            state.validated_config().unwrap_err()
        );
    }
}
