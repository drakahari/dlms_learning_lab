//! Closed NFSv3/ACCESS-style teaching model. No host filesystem, RPC or command bridge.
//! It evaluates supplied numeric identities, not trusted authentication or full Linux opens.
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, SeqAccess, Visitor},
};
use std::{collections::BTreeSet, fmt, io::Write, marker::PhantomData, net::Ipv4Addr};

pub const MAX_STATE: usize = 262_144;
pub const MAX_INVENTORY: usize = 131_072;
pub const MAX_ACTION: usize = 65_536;
pub const MAX_DOCUMENT: usize = 8192;
pub const MAX_FILE: usize = 4096;
pub const MAX_CONTENTS: usize = 65_536;
pub const MAX_RETURNED: usize = 65_536;
pub const MAX_CALLS: usize = 128;
const MAX_ID: u32 = 65_534;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum FaultKind {
    Malformed,
    Context,
    Limit,
    SimulatorUnsupported,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fault {
    pub kind: FaultKind,
    pub message: String,
}
impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Fault {}
type Result<T> = std::result::Result<T, Fault>;
fn fail<T>(kind: FaultKind, message: &str) -> Result<T> {
    let mut n = message.len().min(512);
    while !message.is_char_boundary(n) {
        n -= 1;
    }
    Err(Fault {
        kind,
        message: message[..n].into(),
    })
}
fn require(ok: bool, kind: FaultKind, message: &str) -> Result<()> {
    if ok { Ok(()) } else { fail(kind, message) }
}
struct Count {
    bytes: usize,
    cap: usize,
}
impl Write for Count {
    fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
        let n = self
            .bytes
            .checked_add(input.len())
            .filter(|n| *n <= self.cap)
            .ok_or_else(|| std::io::Error::other("NFS compact limit"))?;
        self.bytes = n;
        Ok(input.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn size<T: Serialize + ?Sized>(value: &T, cap: usize) -> Option<usize> {
    let mut c = Count { bytes: 0, cap };
    serde_json::to_writer(&mut c, value).ok()?;
    Some(c.bytes)
}
fn compact<T: Serialize + ?Sized>(value: &T, cap: usize) -> Result<()> {
    require(
        size(value, cap).is_some(),
        FaultKind::Limit,
        "NFS compact aggregate exceeds limit",
    )
}
fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> std::result::Result<String, D::Error> {
    struct T<const N: usize>;
    impl<const N: usize> Visitor<'_> for T<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "non-NUL text up to {N} bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if s.len() > N || s.contains('\0') {
                Err(E::custom("NFS text bound"))
            } else {
                Ok(s.into())
            }
        }
    }
    d.deserialize_str(T::<N>)
}
struct Element<T> {
    allowed: bool,
    marker: PhantomData<T>,
}
impl<'de, T: Deserialize<'de>> DeserializeSeed<'de> for Element<T> {
    type Value = T;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<T, D::Error> {
        if !self.allowed {
            return Err(D::Error::custom("NFS collection limit before typed body"));
        }
        T::deserialize(d)
    }
}
fn list<'de, D: Deserializer<'de>, T: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<Vec<T>, D::Error> {
    struct Items<T, const N: usize>(PhantomData<T>);
    impl<'de, T: Deserialize<'de>, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = Vec<T>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most {N} items")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> std::result::Result<Vec<T>, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element_seed(Element {
                allowed: out.len() < N,
                marker: PhantomData,
            })? {
                out.push(v);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(PhantomData))
}
#[derive(Deserialize)]
struct PathText(#[serde(deserialize_with = "text::<_,160>")] String);
fn paths<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    Ok(list::<D, PathText, 4>(d)?
        .into_iter()
        .map(|s| s.0)
        .collect())
}
#[derive(Deserialize)]
struct IdText(#[serde(deserialize_with = "text::<_,32>")] String);
fn ids<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    Ok(list::<D, IdText, 16>(d)?.into_iter().map(|s| s.0).collect())
}
fn server_ids<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    Ok(list::<D, IdText, 2>(d)?.into_iter().map(|s| s.0).collect())
}
fn bounded(s: &str, n: usize) -> bool {
    s.len() <= n && !s.contains('\0')
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn components(s: &str, absolute: bool) -> bool {
    if s.is_empty() || s.len() > 160 || !s.is_ascii() || s.starts_with('/') != absolute {
        return false;
    }
    let tail = if absolute { &s[1..] } else { s };
    !tail.is_empty()
        && tail.split('/').count() <= 8
        && tail.split('/').all(|p| {
            !p.is_empty()
                && p != "."
                && p != ".."
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-.".contains(&b))
        })
}
fn path(s: &str) -> bool {
    components(s, true)
}
fn relative(s: &str) -> bool {
    components(s, false)
}
fn under(p: &str, root: &str) -> bool {
    p == root
        || p.strip_prefix(root)
            .is_some_and(|tail| tail.starts_with('/'))
}
fn parent(p: &str) -> &str {
    p.rsplit_once('/')
        .map_or("/", |(a, _)| if a.is_empty() { "/" } else { a })
}
fn document(s: &str) -> bool {
    bounded(s, MAX_DOCUMENT)
        && s.lines().all(|line| line.len() <= 512)
        && s.chars()
            .all(|c| !c.is_control() || matches!(c, '\t' | '\n' | '\r'))
        && s.as_bytes()
            .iter()
            .enumerate()
            .all(|(n, b)| *b != b'\r' || s.as_bytes().get(n + 1) == Some(&b'\n'))
}
fn unique<T: PartialEq>(v: &[T]) -> bool {
    v.iter().enumerate().all(|(n, x)| !v[..n].contains(x))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    Ro,
    Rw,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    #[serde(deserialize_with = "text::<_,32>")]
    pub id: String,
    pub address: Ipv4Addr,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    #[serde(deserialize_with = "text::<_,32>")]
    pub id: String,
    pub address: Ipv4Addr,
    #[serde(deserialize_with = "paths")]
    pub mountpoints: Vec<String>,
    #[serde(deserialize_with = "ids")]
    pub profiles: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    #[serde(deserialize_with = "text::<_,32>")]
    pub id: String,
    #[serde(deserialize_with = "text::<_,64>")]
    pub label: String,
    pub uid: u32,
    pub gid: u32,
    #[serde(deserialize_with = "list::<_,u32,16>")]
    pub groups: Vec<u32>,
}
impl Profile {
    fn valid(&self) -> bool {
        id(&self.id)
            && bounded(&self.label, 64)
            && !self.label.trim().is_empty()
            && self.uid <= MAX_ID
            && self.gid <= MAX_ID
            && self.groups.len() <= 16
            && self.groups.iter().all(|g| *g <= MAX_ID)
            && self.groups.windows(2).all(|g| g[0] < g[1])
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inode {
    #[serde(deserialize_with = "text::<_,160>")]
    pub path: String,
    pub uid: u32,
    pub gid: u32,
    pub mode: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Root {
    #[serde(deserialize_with = "text::<_,32>")]
    pub server: String,
    #[serde(deserialize_with = "text::<_,160>")]
    pub path: String,
    #[serde(deserialize_with = "list::<_,Inode,32>")]
    pub directories: Vec<Inode>,
    #[serde(deserialize_with = "list::<_,Inode,32>")]
    pub files: Vec<Inode>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inventory {
    #[serde(deserialize_with = "text::<_,32>")]
    pub fixture: String,
    #[serde(deserialize_with = "list::<_,Server,2>")]
    pub servers: Vec<Server>,
    #[serde(deserialize_with = "list::<_,Client,8>")]
    pub clients: Vec<Client>,
    #[serde(deserialize_with = "list::<_,Profile,16>")]
    pub profiles: Vec<Profile>,
    #[serde(deserialize_with = "list::<_,Root,8>")]
    pub roots: Vec<Root>,
}
impl Inventory {
    pub fn validate(&self) -> Result<()> {
        require(
            self.servers.len() <= 2
                && self.clients.len() <= 8
                && self.profiles.len() <= 16
                && self.roots.len() <= 8,
            FaultKind::Limit,
            "NFS inventory collection bound",
        )?;
        require(
            id(&self.fixture)
                && !self.servers.is_empty()
                && !self.clients.is_empty()
                && !self.profiles.is_empty()
                && !self.roots.is_empty(),
            FaultKind::Malformed,
            "NFS needs a supplied nonempty inventory",
        )?;
        for (n, s) in self.servers.iter().enumerate() {
            require(
                id(&s.id)
                    && !self.servers[..n]
                        .iter()
                        .any(|old| old.id == s.id || old.address == s.address),
                FaultKind::Context,
                "Duplicate or invalid supplied server",
            )?;
        }
        for (n, p) in self.profiles.iter().enumerate() {
            require(
                p.valid() && !self.profiles[..n].iter().any(|old| old.id == p.id),
                FaultKind::Malformed,
                "Invalid bounded numeric profile",
            )?;
        }
        let mut targets = 0usize;
        for (n, c) in self.clients.iter().enumerate() {
            require(
                c.mountpoints.len() <= 4 && c.profiles.len() <= 16,
                FaultKind::Limit,
                "NFS client collection bound",
            )?;
            targets += c.mountpoints.len();
            require(
                id(&c.id)
                    && !self.clients[..n]
                        .iter()
                        .any(|old| old.id == c.id || old.address == c.address)
                    && unique(&c.profiles)
                    && c.profiles
                        .iter()
                        .all(|p| id(p) && self.profiles.iter().any(|v| &v.id == p)),
                FaultKind::Context,
                "Invalid supplied client/profile context",
            )?;
            for (k, p) in c.mountpoints.iter().enumerate() {
                require(
                    path(p)
                        && !c.mountpoints[..k]
                            .iter()
                            .any(|q| under(p, q) || under(q, p)),
                    FaultKind::Context,
                    "Duplicate, nested or invalid client mount point",
                )?;
            }
        }
        require(
            targets <= 16,
            FaultKind::Limit,
            "NFS total mount point bound",
        )?;
        let mut dirs = 0usize;
        let mut files = 0usize;
        for (n, r) in self.roots.iter().enumerate() {
            require(
                r.directories.len() <= 32 && r.files.len() <= 32,
                FaultKind::Limit,
                "NFS filesystem count bound",
            )?;
            dirs += r.directories.len();
            files += r.files.len();
            require(
                id(&r.server)
                    && self.servers.iter().any(|s| s.id == r.server)
                    && path(&r.path)
                    && !self.roots[..n].iter().any(|q| {
                        q.server == r.server && (under(&r.path, &q.path) || under(&q.path, &r.path))
                    }),
                FaultKind::Context,
                "Invalid, nested or duplicate exportable root",
            )?;
            require(
                r.directories.iter().any(|d| d.path == r.path),
                FaultKind::Context,
                "Export root directory is missing",
            )?;
            for (is_file, nodes) in [(false, &r.directories), (true, &r.files)] {
                for (k, node) in nodes.iter().enumerate() {
                    require(
                        path(&node.path)
                            && under(&node.path, &r.path)
                            && (!is_file || node.path != r.path)
                            && node.uid <= MAX_ID
                            && node.gid <= MAX_ID
                            && node.mode <= 0o777,
                        FaultKind::Malformed,
                        "Invalid ordinary supplied inode",
                    )?;
                    require(
                        !is_file || (node.mode & 0o600 == 0o600 && node.mode & !0o666 == 0),
                        FaultKind::SimulatorUnsupported,
                        "Only owner-RW nonexecuting regular text files are supported",
                    )?;
                    require(
                        !nodes[..k].iter().any(|v| v.path == node.path)
                            && (!is_file || !r.directories.iter().any(|d| d.path == node.path)),
                        FaultKind::Context,
                        "Duplicate inode path",
                    )?;
                    if node.path != r.path {
                        require(
                            r.directories.iter().any(|d| d.path == parent(&node.path)),
                            FaultKind::Context,
                            "Every server-side path parent must be a supplied directory",
                        )?;
                    }
                }
            }
        }
        require(
            dirs <= 32 && files <= 32,
            FaultKind::Limit,
            "NFS total directory/file bound",
        )?;
        compact(self, MAX_INVENTORY)
    }
    fn server(&self, id: &str) -> Option<&Server> {
        self.servers.iter().find(|s| s.id == id)
    }
    fn client(&self, id: &str) -> Option<&Client> {
        self.clients.iter().find(|c| c.id == id)
    }
    fn root(&self, server: &str, path: &str) -> Option<&Root> {
        self.roots
            .iter()
            .find(|r| r.server == server && r.path == path)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prefix {
    pub network: Ipv4Addr,
    pub length: u8,
}
impl Prefix {
    fn mask(self) -> u32 {
        if self.length == 0 {
            0
        } else {
            u32::MAX << (32 - self.length)
        }
    }
    fn valid(self) -> bool {
        self.length <= 32 && u32::from(self.network) & !self.mask() == 0
    }
    pub fn contains(self, address: Ipv4Addr) -> bool {
        self.valid() && u32::from(address) & self.mask() == u32::from(self.network)
    }
    fn intersects(self, other: Self) -> bool {
        self.contains(other.network) || other.contains(self.network)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Options {
    pub access: Access,
    pub root_squash: bool,
    pub anon_uid: u32,
    pub anon_gid: u32,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            access: Access::Ro,
            root_squash: true,
            anon_uid: 65534,
            anon_gid: 65534,
        }
    }
}
impl Options {
    fn valid(self) -> bool {
        (1..=MAX_ID).contains(&self.anon_uid) && (1..=MAX_ID).contains(&self.anon_gid)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    pub selector: Prefix,
    pub options: Options,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Export {
    #[serde(deserialize_with = "text::<_,160>")]
    pub root: String,
    #[serde(deserialize_with = "list::<_,Rule,8>")]
    pub clients: Vec<Rule>,
}
fn export_shape(i: &Inventory, server: &str, exports: &[Export]) -> Result<()> {
    require(
        exports.len() <= 8,
        FaultKind::Limit,
        "NFS applied export count",
    )?;
    for (n, e) in exports.iter().enumerate() {
        require(
            path(&e.root)
                && i.root(server, &e.root).is_some()
                && !exports[..n].iter().any(|v| v.root == e.root),
            FaultKind::Context,
            "Applied export root context differs",
        )?;
        require(
            !e.clients.is_empty() && e.clients.len() <= 8,
            FaultKind::Limit,
            "NFS export needs one to eight client clauses",
        )?;
        for (k, r) in e.clients.iter().enumerate() {
            require(
                r.selector.valid() && r.options.valid(),
                FaultKind::Malformed,
                "Invalid export selector/options",
            )?;
            require(
                !e.clients[..k]
                    .iter()
                    .any(|v| v.selector.intersects(r.selector)),
                FaultKind::SimulatorUnsupported,
                "Overlapping client selectors are outside this subset; no invented precedence",
            )?;
        }
    }
    Ok(())
}
fn ipv4(s: &str) -> Result<Ipv4Addr> {
    require(
        s.len() <= 15,
        FaultKind::Malformed,
        "Invalid canonical IPv4 literal",
    )?;
    let ip = s.parse::<Ipv4Addr>().map_err(|_| Fault {
        kind: FaultKind::Malformed,
        message: "Invalid IPv4 literal".into(),
    })?;
    require(
        ip.to_string() == s,
        FaultKind::Malformed,
        "IPv4 literal is not canonical",
    )?;
    Ok(ip)
}
fn selector(s: &str) -> Result<Prefix> {
    require(
        s.len() <= 18,
        FaultKind::Malformed,
        "Invalid bounded IPv4 selector",
    )?;
    let p = if let Some((n, bits)) = s.split_once('/') {
        require(
            !bits.is_empty() && bits.bytes().all(|b| b.is_ascii_digit()),
            FaultKind::Malformed,
            "Invalid prefix length",
        )?;
        let length = bits.parse::<u8>().map_err(|_| Fault {
            kind: FaultKind::Malformed,
            message: "Invalid prefix length".into(),
        })?;
        require(
            length.to_string() == bits,
            FaultKind::Malformed,
            "Noncanonical prefix length",
        )?;
        Prefix {
            network: ipv4(n)?,
            length,
        }
    } else {
        Prefix {
            network: ipv4(s)?,
            length: 32,
        }
    };
    require(
        p.valid(),
        FaultKind::Malformed,
        "Prefix needs a canonical network and length zero to 32",
    )?;
    Ok(p)
}
fn options(raw: &str) -> Result<Options> {
    let mut o = Options::default();
    if raw.is_empty() {
        return Ok(o);
    }
    require(
        raw.split(',').count() <= 8,
        FaultKind::Limit,
        "Too many export options",
    )?;
    let mut seen = BTreeSet::new();
    for part in raw.split(',') {
        let key = match part {
            "ro" => {
                o.access = Access::Ro;
                "access"
            }
            "rw" => {
                o.access = Access::Rw;
                "access"
            }
            "sync" => "sync",
            "sec=sys" => "sec",
            "root_squash" => {
                o.root_squash = true;
                "squash"
            }
            "no_root_squash" => {
                o.root_squash = false;
                "squash"
            }
            _ => {
                let Some((k, v)) = part.split_once('=') else {
                    return fail(
                        FaultKind::SimulatorUnsupported,
                        "Export option is outside the closed subset",
                    );
                };
                require(
                    matches!(k, "anonuid" | "anongid"),
                    FaultKind::SimulatorUnsupported,
                    "Unknown export option",
                )?;
                require(
                    !v.is_empty() && v.len() <= 5 && v.bytes().all(|b| b.is_ascii_digit()),
                    FaultKind::Malformed,
                    "Invalid bounded anonymous numeric ID",
                )?;
                let n = v.parse::<u32>().map_err(|_| Fault {
                    kind: FaultKind::Malformed,
                    message: "Invalid anonymous numeric ID".into(),
                })?;
                match k {
                    "anonuid" => {
                        o.anon_uid = n;
                        "anonuid"
                    }
                    "anongid" => {
                        o.anon_gid = n;
                        "anongid"
                    }
                    _ => unreachable!("checked option key"),
                }
            }
        };
        require(
            seen.insert(key),
            FaultKind::SimulatorUnsupported,
            "Duplicate or conflicting export options are outside this subset",
        )?;
    }
    require(
        o.valid(),
        FaultKind::SimulatorUnsupported,
        "Anonymous UID/GID must be nonzero and at most 65534",
    )?;
    Ok(o)
}
pub fn parse_exports(i: &Inventory, server: &str, raw: &str) -> Result<Vec<Export>> {
    require(
        document(raw),
        FaultKind::Limit,
        "NFS export source byte/line/control bound",
    )?;
    i.validate()?;
    require(
        i.server(server).is_some(),
        FaultKind::Context,
        "Unknown supplied server",
    )?;
    let mut out = Vec::<Export>::new();
    for line in raw.lines() {
        let code = line.split('#').next().unwrap_or("").trim();
        if code.is_empty() {
            continue;
        }
        require(
            out.len() < 8,
            FaultKind::Limit,
            "Ninth export record refused before construction",
        )?;
        let mut words = code.split_ascii_whitespace();
        let root = words.next().unwrap();
        require(
            path(root) && i.root(server, root).is_some() && !out.iter().any(|e| e.root == root),
            FaultKind::Context,
            "Unknown or duplicate supplied export root",
        )?;
        let n = words.clone().count();
        require(
            (1..=8).contains(&n),
            FaultKind::Limit,
            "Export needs one to eight clauses",
        )?;
        let mut clients = Vec::new();
        for w in words {
            let (s, o) = if let Some((s, rest)) = w.split_once('(') {
                require(
                    rest.ends_with(')') && !rest[..rest.len() - 1].contains(['(', ')']),
                    FaultKind::SimulatorUnsupported,
                    "Unsupported client option grouping; parentheses must be adjacent",
                )?;
                (s, options(&rest[..rest.len() - 1])?)
            } else {
                (w, Options::default())
            };
            clients.push(Rule {
                selector: selector(s)?,
                options: o,
            });
        }
        out.push(Export {
            root: root.into(),
            clients,
        });
    }
    export_shape(i, server, &out)?;
    Ok(out)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MountSpec {
    #[serde(deserialize_with = "text::<_,32>")]
    pub server: String,
    #[serde(deserialize_with = "text::<_,160>")]
    pub root: String,
    #[serde(deserialize_with = "text::<_,160>")]
    pub mountpoint: String,
    pub access: Access,
}
fn mount_shape(i: &Inventory, client: &str, s: &MountSpec) -> Result<()> {
    require(
        id(&s.server)
            && path(&s.root)
            && path(&s.mountpoint)
            && i.root(&s.server, &s.root).is_some()
            && i.client(client)
                .is_some_and(|c| c.mountpoints.contains(&s.mountpoint)),
        FaultKind::Context,
        "Mount source/target differs from supplied context",
    )
}
pub fn parse_fstab(i: &Inventory, client: &str, raw: &str) -> Result<Vec<MountSpec>> {
    require(
        document(raw),
        FaultKind::Limit,
        "NFS client source byte/line/control bound",
    )?;
    i.validate()?;
    require(
        i.client(client).is_some(),
        FaultKind::Context,
        "Unknown supplied client",
    )?;
    let mut out = Vec::<MountSpec>::new();
    for line in raw.lines() {
        let code = line.split('#').next().unwrap_or("").trim();
        if code.is_empty() {
            continue;
        }
        require(
            out.len() < 4,
            FaultKind::Limit,
            "Fifth client mount record refused before construction",
        )?;
        require(
            code.split_ascii_whitespace().count() == 6,
            FaultKind::SimulatorUnsupported,
            "Only the disclosed six-field NFS mount records are supported",
        )?;
        let w = code.split_ascii_whitespace().collect::<Vec<_>>();
        require(
            w[2] == "nfs" && w[4] == "0" && w[5] == "0",
            FaultKind::SimulatorUnsupported,
            "Only nfs with fixed zero trailing fields is supported",
        )?;
        let Some((ip, root)) = w[0].split_once(':') else {
            return fail(
                FaultKind::SimulatorUnsupported,
                "Mount source needs a supplied literal IPv4 and absolute root",
            );
        };
        let address = ipv4(ip)?;
        let server = i
            .servers
            .iter()
            .find(|s| s.address == address)
            .ok_or_else(|| Fault {
                kind: FaultKind::Context,
                message: "Mount server address is not supplied".into(),
            })?;
        require(
            w[3].split(',').count() == 4,
            FaultKind::SimulatorUnsupported,
            "Mount options need vers=3,proto=tcp,sec=sys and ro or rw",
        )?;
        let mut found = BTreeSet::new();
        let mut access = Access::Ro;
        for o in w[3].split(',') {
            let key = match o {
                "vers=3" => "vers",
                "proto=tcp" => "proto",
                "sec=sys" => "sec",
                "ro" => {
                    access = Access::Ro;
                    "access"
                }
                "rw" => {
                    access = Access::Rw;
                    "access"
                }
                _ => {
                    return fail(
                        FaultKind::SimulatorUnsupported,
                        "Unsupported client mount option",
                    );
                }
            };
            require(
                found.insert(key),
                FaultKind::SimulatorUnsupported,
                "Duplicate client mount option",
            )?;
        }
        require(
            found.len() == 4
                && path(root)
                && path(w[1])
                && !out.iter().any(|s| s.mountpoint == w[1]),
            FaultKind::Context,
            "Invalid or duplicate mount target/options",
        )?;
        let s = MountSpec {
            server: server.id.clone(),
            root: root.into(),
            mountpoint: w[1].into(),
            access,
        };
        mount_shape(i, client, &s)?;
        out.push(s);
    }
    Ok(out)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerLayers {
    #[serde(deserialize_with = "text::<_,32>")]
    pub server: String,
    #[serde(deserialize_with = "text::<_,8192>")]
    pub draft: String,
    #[serde(deserialize_with = "text::<_,8192>")]
    pub saved: String,
    #[serde(deserialize_with = "list::<_,Export,8>")]
    pub applied: Vec<Export>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClientLayers {
    #[serde(deserialize_with = "text::<_,32>")]
    pub client: String,
    #[serde(deserialize_with = "text::<_,8192>")]
    pub draft: String,
    #[serde(deserialize_with = "text::<_,8192>")]
    pub saved: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mount {
    #[serde(deserialize_with = "text::<_,32>")]
    pub client: String,
    pub specification: MountSpec,
    pub binding: Options,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contents {
    #[serde(deserialize_with = "text::<_,32>")]
    pub server: String,
    #[serde(deserialize_with = "text::<_,160>")]
    pub path: String,
    #[serde(deserialize_with = "text::<_,4096>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    #[serde(deserialize_with = "list::<_,ServerLayers,2>")]
    pub servers: Vec<ServerLayers>,
    #[serde(deserialize_with = "list::<_,ClientLayers,8>")]
    pub clients: Vec<ClientLayers>,
    #[serde(deserialize_with = "list::<_,Mount,16>")]
    pub mounts: Vec<Mount>,
    #[serde(deserialize_with = "list::<_,Contents,32>")]
    pub contents: Vec<Contents>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct State {
    schema_version: u32,
    inventory: Inventory,
    runtime: Runtime,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StateWire {
    schema_version: u32,
    inventory: Inventory,
    runtime: Runtime,
}
impl<'de> Deserialize<'de> for State {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let w = StateWire::deserialize(d)?;
        let state = Self {
            schema_version: w.schema_version,
            inventory: w.inventory,
            runtime: w.runtime,
        };
        state.validate().map_err(D::Error::custom)?;
        Ok(state)
    }
}
fn matched<'a>(exports: &'a [Export], root: &str, address: Ipv4Addr) -> Option<&'a Options> {
    exports
        .iter()
        .find(|e| e.root == root)?
        .clients
        .iter()
        .find(|r| r.selector.contains(address))
        .map(|r| &r.options)
}
impl State {
    pub fn new(inventory: Inventory, runtime: Runtime) -> Result<Self> {
        let s = Self {
            schema_version: 1,
            inventory,
            runtime,
        };
        s.validate()?;
        Ok(s)
    }
    pub fn from_json(raw: &str) -> Result<Self> {
        require(
            raw.len() <= MAX_STATE,
            FaultKind::Limit,
            "Raw NFS State exceeds limit before decoding",
        )?;
        serde_json::from_str(raw).map_err(|_| Fault {
            kind: FaultKind::Malformed,
            message: "Invalid bounded NFS State JSON".into(),
        })
    }
    pub fn inventory(&self) -> &Inventory {
        &self.inventory
    }
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }
    pub fn same_context(&self, other: &Self) -> bool {
        self.inventory == other.inventory
    }
    pub fn validate(&self) -> Result<()> {
        require(
            self.schema_version == 1,
            FaultKind::Malformed,
            "Unsupported NFS State schema",
        )?;
        let r = &self.runtime;
        let i = &self.inventory;
        require(
            r.servers.len() <= 2
                && r.clients.len() <= 8
                && r.mounts.len() <= 16
                && r.contents.len() <= 32,
            FaultKind::Limit,
            "NFS runtime collection bound",
        )?;
        for s in &r.servers {
            require(
                bounded(&s.server, 32)
                    && document(&s.draft)
                    && document(&s.saved)
                    && s.applied.len() <= 8
                    && s.applied.iter().all(|e| e.clients.len() <= 8),
                FaultKind::Limit,
                "NFS server layer scalar/count bound",
            )?;
        }
        for c in &r.clients {
            require(
                bounded(&c.client, 32) && document(&c.draft) && document(&c.saved),
                FaultKind::Limit,
                "NFS client layer scalar bound",
            )?;
        }
        let mut total = 0usize;
        for c in &r.contents {
            require(
                bounded(&c.text, MAX_FILE),
                FaultKind::Limit,
                "NFS existing text bound",
            )?;
            total = total.checked_add(c.text.len()).ok_or_else(|| Fault {
                kind: FaultKind::Limit,
                message: "NFS contents overflow".into(),
            })?;
        }
        require(
            total <= MAX_CONTENTS,
            FaultKind::Limit,
            "NFS aggregate contents bound",
        )?;
        i.validate()?;
        require(
            r.servers.len() == i.servers.len() && r.clients.len() == i.clients.len(),
            FaultKind::Context,
            "NFS layers must cover exact supplied server/client sets",
        )?;
        for (n, s) in r.servers.iter().enumerate() {
            require(
                i.server(&s.server).is_some()
                    && !r.servers[..n].iter().any(|v| v.server == s.server),
                FaultKind::Context,
                "Foreign or duplicate server layers",
            )?;
            export_shape(i, &s.server, &s.applied)?;
        }
        for (n, c) in r.clients.iter().enumerate() {
            require(
                i.client(&c.client).is_some()
                    && !r.clients[..n].iter().any(|v| v.client == c.client),
                FaultKind::Context,
                "Foreign or duplicate client layers",
            )?;
        }
        require(
            r.contents.len() == i.roots.iter().map(|r| r.files.len()).sum::<usize>(),
            FaultKind::Context,
            "NFS contents must cover exact supplied regular files",
        )?;
        for (n, c) in r.contents.iter().enumerate() {
            require(
                id(&c.server)
                    && path(&c.path)
                    && i.roots
                        .iter()
                        .any(|r| r.server == c.server && r.files.iter().any(|f| f.path == c.path))
                    && !r.contents[..n]
                        .iter()
                        .any(|v| v.server == c.server && v.path == c.path),
                FaultKind::Context,
                "Foreign, missing or duplicate existing file content",
            )?;
        }
        for (n, m) in r.mounts.iter().enumerate() {
            mount_shape(i, &m.client, &m.specification)?;
            require(
                !r.mounts[..n].iter().any(|v| {
                    v.client == m.client && v.specification.mountpoint == m.specification.mountpoint
                }),
                FaultKind::Context,
                "Duplicate current mount",
            )?;
            let client = i.client(&m.client).expect("validated mount client");
            require(
                self.table(&m.specification.server)
                    .and_then(|t| matched(t, &m.specification.root, client.address))
                    == Some(&m.binding),
                FaultKind::Context,
                "Current mount binding differs from applied export policy",
            )?;
        }
        compact(self, MAX_STATE)
    }
    fn table(&self, server: &str) -> Option<&[Export]> {
        self.runtime
            .servers
            .iter()
            .find(|s| s.server == server)
            .map(|s| s.applied.as_slice())
    }
    fn server_index(&self, server: &str) -> Result<usize> {
        self.runtime
            .servers
            .iter()
            .position(|s| s.server == server)
            .ok_or_else(|| Fault {
                kind: FaultKind::Context,
                message: "Unknown supplied server layer".into(),
            })
    }
    fn client_index(&self, client: &str) -> Result<usize> {
        self.runtime
            .clients
            .iter()
            .position(|s| s.client == client)
            .ok_or_else(|| Fault {
                kind: FaultKind::Context,
                message: "Unknown supplied client layer".into(),
            })
    }
    // Exact escaped JSON substitution. No full-State candidate is allocated.
    fn replacement<T: Serialize + ?Sized, U: Serialize + ?Sized>(
        &self,
        old: &T,
        new: &U,
    ) -> Result<()> {
        let current = size(self, MAX_STATE).ok_or_else(|| Fault {
            kind: FaultKind::Limit,
            message: "NFS State aggregate".into(),
        })?;
        let old = size(old, MAX_STATE).ok_or_else(|| Fault {
            kind: FaultKind::Limit,
            message: "NFS old field aggregate".into(),
        })?;
        let new = size(new, MAX_STATE).ok_or_else(|| Fault {
            kind: FaultKind::Limit,
            message: "NFS proposed field aggregate".into(),
        })?;
        require(
            current
                .checked_sub(old)
                .and_then(|n| n.checked_add(new))
                .is_some_and(|n| n <= MAX_STATE),
            FaultKind::Limit,
            "Proposed NFS State exceeds compact bound before copying",
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    SetExportsDraft {
        #[serde(deserialize_with = "text::<_,32>")]
        server: String,
        #[serde(deserialize_with = "text::<_,8192>")]
        text: String,
    },
    SaveExportsDraft {
        #[serde(deserialize_with = "text::<_,32>")]
        server: String,
    },
    ApplySavedExports {
        #[serde(deserialize_with = "text::<_,32>")]
        server: String,
    },
    SetFstabDraft {
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
        #[serde(deserialize_with = "text::<_,8192>")]
        text: String,
    },
    SaveFstabDraft {
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
    },
    MountExplicit {
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
        specification: MountSpec,
    },
    MountSaved {
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
        #[serde(deserialize_with = "text::<_,160>")]
        mountpoint: String,
    },
    Unmount {
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
        #[serde(deserialize_with = "text::<_,160>")]
        mountpoint: String,
    },
    ReadExisting {
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
        #[serde(deserialize_with = "text::<_,160>")]
        mountpoint: String,
        #[serde(deserialize_with = "text::<_,32>")]
        profile: String,
        #[serde(deserialize_with = "text::<_,160>")]
        relative: String,
    },
    WriteExisting {
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
        #[serde(deserialize_with = "text::<_,160>")]
        mountpoint: String,
        #[serde(deserialize_with = "text::<_,32>")]
        profile: String,
        #[serde(deserialize_with = "text::<_,160>")]
        relative: String,
        #[serde(deserialize_with = "text::<_,4096>")]
        text: String,
    },
}
impl Action {
    pub fn from_json(raw: &str) -> Result<Self> {
        require(
            raw.len() <= MAX_ACTION,
            FaultKind::Limit,
            "Raw NFS Action exceeds limit before decoding",
        )?;
        let a: Self = serde_json::from_str(raw).map_err(|_| Fault {
            kind: FaultKind::Malformed,
            message: "Invalid bounded NFS Action JSON".into(),
        })?;
        a.validate()?;
        Ok(a)
    }
    fn validate(&self) -> Result<()> {
        match self {
            Self::SetExportsDraft { server, text } => {
                require(id(server), FaultKind::Context, "Invalid supplied server ID")?;
                require(document(text), FaultKind::Limit, "Export editor text bound")?;
            }
            Self::SaveExportsDraft { server } | Self::ApplySavedExports { server } => {
                require(id(server), FaultKind::Context, "Invalid supplied server ID")?
            }
            Self::SetFstabDraft { client, text } => {
                require(id(client), FaultKind::Context, "Invalid supplied client ID")?;
                require(document(text), FaultKind::Limit, "Client editor text bound")?;
            }
            Self::SaveFstabDraft { client } => {
                require(id(client), FaultKind::Context, "Invalid supplied client ID")?
            }
            Self::MountExplicit {
                client,
                specification,
            } => require(
                id(client)
                    && id(&specification.server)
                    && path(&specification.root)
                    && path(&specification.mountpoint),
                FaultKind::Context,
                "Invalid supplied mount context",
            )?,
            Self::MountSaved { client, mountpoint } | Self::Unmount { client, mountpoint } => {
                require(
                    id(client) && path(mountpoint),
                    FaultKind::Context,
                    "Invalid supplied mount context",
                )?
            }
            Self::ReadExisting {
                client,
                mountpoint,
                profile,
                relative,
            } => request_shape(client, mountpoint, profile, relative)?,
            Self::WriteExisting {
                client,
                mountpoint,
                profile,
                relative,
                text,
            } => {
                require(
                    bounded(text, MAX_FILE),
                    FaultKind::Limit,
                    "Virtual file replacement text bound",
                )?;
                request_shape(client, mountpoint, profile, relative)?;
            }
        }
        compact(self, MAX_ACTION)
    }
    fn trial_allowed(&self) -> bool {
        matches!(
            self,
            Self::MountExplicit { .. }
                | Self::MountSaved { .. }
                | Self::Unmount { .. }
                | Self::ReadExisting { .. }
                | Self::WriteExisting { .. }
        )
    }
}
fn request_shape(client: &str, mountpoint: &str, profile: &str, rel: &str) -> Result<()> {
    require(
        id(client) && path(mountpoint) && id(profile) && relative(rel),
        FaultKind::Context,
        "Invalid supplied file request context/path",
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DenialStage {
    MountAbsent,
    ExportAdmission,
    ClientReadOnly,
    ExportReadOnly,
    DirectorySearch,
    FilePermission,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Class {
    Owner,
    Group,
    Other,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub uid: u32,
    pub gid: u32,
    #[serde(deserialize_with = "list::<_,u32,16>")]
    pub groups: Vec<u32>,
}
fn mapping(p: &Profile, o: Options) -> Result<Mapping> {
    let uid = if o.root_squash && p.uid == 0 {
        o.anon_uid
    } else {
        p.uid
    };
    require(
        uid != 0,
        FaultKind::SimulatorUnsupported,
        "UnsquashedRootPrivileges: effective UID0 file access is outside this profile",
    )?;
    let gid = if o.root_squash && p.gid == 0 {
        o.anon_gid
    } else {
        p.gid
    };
    let mut groups = Vec::new();
    for &g in &p.groups {
        let g = if o.root_squash && g == 0 {
            o.anon_gid
        } else {
            g
        };
        if !groups.contains(&g) {
            groups.push(g)
        }
    }
    groups.sort_unstable();
    Ok(Mapping { uid, gid, groups })
}
fn bits(node: &Inode, m: &Mapping) -> (Class, u16) {
    if m.uid == node.uid {
        (Class::Owner, (node.mode >> 6) & 7)
    } else if m.gid == node.gid || m.groups.contains(&node.gid) {
        (Class::Group, (node.mode >> 3) & 7)
    } else {
        (Class::Other, node.mode & 7)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Decision {
    Changed {
        changed: bool,
    },
    Mounted {},
    Unmounted {},
    AllowedRead {
        mapping: Mapping,
        class: Class,
        #[serde(deserialize_with = "text::<_,4096>")]
        text: String,
    },
    AllowedWrite {
        mapping: Mapping,
        class: Class,
    },
    Denied {
        stage: DenialStage,
        mapping: Option<Mapping>,
        class: Option<Class>,
    },
}
fn denied(stage: DenialStage) -> Decision {
    Decision::Denied {
        stage,
        mapping: None,
        class: None,
    }
}
struct Charge {
    used: usize,
}
impl Charge {
    fn bytes(&mut self, n: usize) -> Result<()> {
        self.used = self
            .used
            .checked_add(n)
            .filter(|n| *n <= MAX_RETURNED)
            .ok_or_else(|| Fault {
                kind: FaultKind::Limit,
                message: "Joint NFS report/read-text limit".into(),
            })?;
        Ok(())
    }
    fn value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<()> {
        let n = size(v, MAX_RETURNED).ok_or_else(|| Fault {
            kind: FaultKind::Limit,
            message: "Joint NFS report/read-text limit".into(),
        })?;
        self.bytes(n)
    }
}
struct Accessed<'a> {
    index: usize,
    mapping: Mapping,
    class: Class,
    text: &'a str,
}
enum Permission<'a> {
    Denied(Decision),
    Allowed(Accessed<'a>),
}
impl State {
    fn permission(
        &self,
        client: &str,
        mountpoint: &str,
        profile: &str,
        rel: &str,
        write: bool,
    ) -> Result<Permission<'_>> {
        request_shape(client, mountpoint, profile, rel)?;
        let c = self.inventory.client(client).ok_or_else(|| Fault {
            kind: FaultKind::Context,
            message: "File request client is not supplied".into(),
        })?;
        require(
            c.mountpoints.iter().any(|p| p == mountpoint)
                && c.profiles.iter().any(|p| p == profile),
            FaultKind::Context,
            "File target/profile is foreign to this supplied client",
        )?;
        let p = self
            .inventory
            .profiles
            .iter()
            .find(|p| p.id == profile)
            .expect("validated supplied profile");
        let Some(m) = self
            .runtime
            .mounts
            .iter()
            .find(|m| m.client == client && m.specification.mountpoint == mountpoint)
        else {
            return Ok(Permission::Denied(denied(DenialStage::MountAbsent)));
        };
        let spec = &m.specification;
        require(
            spec.root
                .len()
                .checked_add(1)
                .and_then(|n| n.checked_add(rel.len()))
                .is_some_and(|n| n <= 160)
                && spec.root.split('/').skip(1).count() + rel.split('/').count() <= 8,
            FaultKind::Limit,
            "Joined virtual path exceeds bound before allocation",
        )?;
        let full = format!("{}/{rel}", spec.root);
        let root = self
            .inventory
            .root(&spec.server, &spec.root)
            .expect("validated mount root");
        let file = root
            .files
            .iter()
            .find(|f| f.path == full)
            .ok_or_else(|| Fault {
                kind: FaultKind::Context,
                message: "Request does not identify a supplied existing regular file".into(),
            })?;
        let Some(o) = self
            .table(&spec.server)
            .and_then(|t| matched(t, &spec.root, c.address))
        else {
            return Ok(Permission::Denied(denied(DenialStage::ExportAdmission)));
        };
        require(*o == m.binding, FaultKind::Context, "Stale export binding")?;
        if write && spec.access == Access::Ro {
            return Ok(Permission::Denied(denied(DenialStage::ClientReadOnly)));
        }
        if write && o.access == Access::Ro {
            return Ok(Permission::Denied(denied(DenialStage::ExportReadOnly)));
        }
        let mapped = mapping(p, *o)?;
        // Traverse only the supplied export root through the file's parent, shallowest first.
        let mut dirs = root
            .directories
            .iter()
            .filter(|d| under(parent(&full), &d.path))
            .collect::<Vec<_>>();
        dirs.sort_by_key(|d| d.path.len());
        for d in dirs {
            let (class, b) = bits(d, &mapped);
            if b & 1 == 0 {
                return Ok(Permission::Denied(Decision::Denied {
                    stage: DenialStage::DirectorySearch,
                    mapping: Some(mapped),
                    class: Some(class),
                }));
            }
        }
        let (class, b) = bits(file, &mapped);
        let need = if write { 2 } else { 4 };
        if b & need == 0 {
            return Ok(Permission::Denied(Decision::Denied {
                stage: DenialStage::FilePermission,
                mapping: Some(mapped),
                class: Some(class),
            }));
        }
        let index = self
            .runtime
            .contents
            .iter()
            .position(|v| v.server == spec.server && v.path == full)
            .expect("validated exact contents");
        Ok(Permission::Allowed(Accessed {
            index,
            mapping: mapped,
            class,
            text: &self.runtime.contents[index].text,
        }))
    }
    fn read_inner(
        &self,
        client: &str,
        mountpoint: &str,
        profile: &str,
        relative: &str,
        write_probe: bool,
        charge: &mut Charge,
    ) -> Result<Decision> {
        match self.permission(client, mountpoint, profile, relative, write_probe)? {
            Permission::Denied(d) => {
                charge.value(&d)?;
                Ok(d)
            }
            Permission::Allowed(a) => {
                if write_probe {
                    let d = Decision::AllowedWrite {
                        mapping: a.mapping,
                        class: a.class,
                    };
                    charge.value(&d)?;
                    Ok(d)
                } else {
                    // Count the exact borrowed result, including escaped file bytes, before copying it.
                    #[derive(Serialize)]
                    struct Borrowed<'a> {
                        kind: &'static str,
                        mapping: &'a Mapping,
                        class: Class,
                        text: &'a str,
                    }
                    charge.value(&Borrowed {
                        kind: "allowed_read",
                        mapping: &a.mapping,
                        class: a.class,
                        text: a.text,
                    })?;
                    Ok(Decision::AllowedRead {
                        mapping: a.mapping,
                        class: a.class,
                        text: a.text.into(),
                    })
                }
            }
        }
    }
    pub fn apply(&mut self, action: &Action) -> Result<Decision> {
        action.validate()?;
        self.validate()?;
        self.apply_inner(action, &mut Charge { used: 0 })
    }
    fn apply_inner(&mut self, action: &Action, charge: &mut Charge) -> Result<Decision> {
        match action {
            Action::SetExportsDraft { server, text } => {
                let n = self.server_index(server)?;
                let old = &self.runtime.servers[n].draft;
                self.replacement(old, text)?;
                let changed = old != text;
                self.runtime.servers[n].draft = text.clone();
                Ok(Decision::Changed { changed })
            }
            Action::SaveExportsDraft { server } => {
                let n = self.server_index(server)?;
                let s = &self.runtime.servers[n];
                self.replacement(&s.saved, &s.draft)?;
                let changed = s.saved != s.draft;
                self.runtime.servers[n].saved = self.runtime.servers[n].draft.clone();
                Ok(Decision::Changed { changed })
            }
            Action::ApplySavedExports { server } => {
                let n = self.server_index(server)?;
                let next = parse_exports(&self.inventory, server, &self.runtime.servers[n].saved)?;
                for m in self
                    .runtime
                    .mounts
                    .iter()
                    .filter(|m| m.specification.server == *server)
                {
                    let c = self
                        .inventory
                        .client(&m.client)
                        .expect("validated mount client");
                    require(
                        matched(&next, &m.specification.root, c.address) == Some(&m.binding),
                        FaultKind::SimulatorUnsupported,
                        &format!(
                            "ActiveMountConflict: explicitly unmount {}:{} before incompatible Apply",
                            m.client, m.specification.mountpoint
                        ),
                    )?;
                }
                self.replacement(&self.runtime.servers[n].applied, &next)?;
                let changed = self.runtime.servers[n].applied != next;
                self.runtime.servers[n].applied = next;
                Ok(Decision::Changed { changed })
            }
            Action::SetFstabDraft { client, text } => {
                let n = self.client_index(client)?;
                let old = &self.runtime.clients[n].draft;
                self.replacement(old, text)?;
                let changed = old != text;
                self.runtime.clients[n].draft = text.clone();
                Ok(Decision::Changed { changed })
            }
            Action::SaveFstabDraft { client } => {
                let n = self.client_index(client)?;
                let c = &self.runtime.clients[n];
                self.replacement(&c.saved, &c.draft)?;
                let changed = c.saved != c.draft;
                self.runtime.clients[n].saved = self.runtime.clients[n].draft.clone();
                Ok(Decision::Changed { changed })
            }
            Action::MountExplicit {
                client,
                specification,
            } => self.mount_inner(client, specification),
            Action::MountSaved { client, mountpoint } => {
                let n = self.client_index(client)?;
                let parsed = parse_fstab(&self.inventory, client, &self.runtime.clients[n].saved)?;
                let spec = parsed
                    .iter()
                    .find(|s| s.mountpoint == *mountpoint)
                    .ok_or_else(|| Fault {
                        kind: FaultKind::Context,
                        message: "No saved row for supplied mount point".into(),
                    })?;
                self.mount_inner(client, spec)
            }
            Action::Unmount { client, mountpoint } => {
                require(
                    self.inventory
                        .client(client)
                        .is_some_and(|c| c.mountpoints.contains(mountpoint)),
                    FaultKind::Context,
                    "Unmount context is not supplied",
                )?;
                let Some(n) =
                    self.runtime.mounts.iter().position(|m| {
                        m.client == *client && m.specification.mountpoint == *mountpoint
                    })
                else {
                    return Ok(denied(DenialStage::MountAbsent));
                };
                self.runtime.mounts.remove(n);
                Ok(Decision::Unmounted {})
            }
            Action::ReadExisting {
                client,
                mountpoint,
                profile,
                relative,
            } => self.read_inner(client, mountpoint, profile, relative, false, charge),
            Action::WriteExisting {
                client,
                mountpoint,
                profile,
                relative,
                text,
            } => match self.permission(client, mountpoint, profile, relative, true)? {
                Permission::Denied(d) => {
                    charge.value(&d)?;
                    Ok(d)
                }
                Permission::Allowed(a) => {
                    let n = a.index;
                    let decision = Decision::AllowedWrite {
                        mapping: a.mapping,
                        class: a.class,
                    };
                    charge.value(&decision)?;
                    let total = self
                        .runtime
                        .contents
                        .iter()
                        .map(|c| c.text.len())
                        .sum::<usize>();
                    require(
                        total - self.runtime.contents[n].text.len() + text.len() <= MAX_CONTENTS,
                        FaultKind::Limit,
                        "Proposed aggregate contents limit",
                    )?;
                    self.replacement(&self.runtime.contents[n].text, text)?;
                    self.runtime.contents[n].text = text.clone();
                    Ok(decision)
                }
            },
        }
    }
    fn mount_inner(&mut self, client: &str, spec: &MountSpec) -> Result<Decision> {
        mount_shape(&self.inventory, client, spec)?;
        require(
            !self
                .runtime
                .mounts
                .iter()
                .any(|m| m.client == client && m.specification.mountpoint == spec.mountpoint),
            FaultKind::SimulatorUnsupported,
            "Target already mounted: explicit Unmount is required",
        )?;
        let c = self.inventory.client(client).expect("validated client");
        let Some(&binding) = self
            .table(&spec.server)
            .and_then(|t| matched(t, &spec.root, c.address))
        else {
            return Ok(denied(DenialStage::ExportAdmission));
        };
        require(
            self.runtime.mounts.len() < 16,
            FaultKind::Limit,
            "Current mount count limit",
        )?;
        #[derive(Serialize)]
        struct Borrowed<'a> {
            client: &'a str,
            specification: &'a MountSpec,
            binding: Options,
        }
        #[derive(Serialize)]
        #[serde(untagged)]
        enum Item<'a> {
            Old(&'a Mount),
            New(Borrowed<'a>),
        }
        let mut proposed = self
            .runtime
            .mounts
            .iter()
            .map(Item::Old)
            .collect::<Vec<_>>();
        proposed.push(Item::New(Borrowed {
            client,
            specification: spec,
            binding,
        }));
        self.replacement(&self.runtime.mounts, &proposed)?;
        self.runtime.mounts.push(Mount {
            client: client.into(),
            specification: spec.clone(),
            binding,
        });
        Ok(Decision::Mounted {})
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Request {
    MountPresent {
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
        #[serde(deserialize_with = "text::<_,160>")]
        mountpoint: String,
    },
    Read {
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
        #[serde(deserialize_with = "text::<_,160>")]
        mountpoint: String,
        #[serde(deserialize_with = "text::<_,32>")]
        profile: String,
        #[serde(deserialize_with = "text::<_,160>")]
        relative: String,
    },
    WritePermission {
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
        #[serde(deserialize_with = "text::<_,160>")]
        mountpoint: String,
        #[serde(deserialize_with = "text::<_,32>")]
        profile: String,
        #[serde(deserialize_with = "text::<_,160>")]
        relative: String,
    },
}
impl Request {
    fn validate(&self) -> Result<()> {
        match self {
            Self::MountPresent { client, mountpoint } => require(
                id(client) && path(mountpoint),
                FaultKind::Context,
                "Invalid current mount request",
            ),
            Self::Read {
                client,
                mountpoint,
                profile,
                relative,
            }
            | Self::WritePermission {
                client,
                mountpoint,
                profile,
                relative,
            } => request_shape(client, mountpoint, profile, relative),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Expected {
    Mounted {},
    Unmounted {},
    Read {
        #[serde(deserialize_with = "text::<_,4096>")]
        text: String,
    },
    Write {},
    Denied {
        stage: DenialStage,
    },
}
impl Expected {
    fn matches(&self, d: &Decision) -> bool {
        match (self, d) {
            (Self::Mounted {}, Decision::Mounted {})
            | (Self::Unmounted {}, Decision::Unmounted {})
            | (Self::Write {}, Decision::AllowedWrite { .. }) => true,
            (Self::Read { text: a }, Decision::AllowedRead { text: b, .. }) => a == b,
            (Self::Denied { stage: a }, Decision::Denied { stage: b, .. }) => a == b,
            _ => false,
        }
    }
    fn validate(&self) -> Result<()> {
        if let Self::Read { text } = self {
            require(
                bounded(text, MAX_FILE),
                FaultKind::Limit,
                "Expected read text bound",
            )?
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum TrialBase {
    Applied {},
    Saved {
        #[serde(deserialize_with = "server_ids")]
        servers: Vec<String>,
        #[serde(deserialize_with = "text::<_,32>")]
        client: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrialStep {
    pub action: Action,
    pub expected: Expected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trial {
    pub base: TrialBase,
    #[serde(deserialize_with = "list::<_,TrialStep,16>")]
    pub steps: Vec<TrialStep>,
}
impl Trial {
    fn validate(&self) -> Result<()> {
        require(
            !self.steps.is_empty() && self.steps.len() <= 16,
            FaultKind::Limit,
            "Fresh trial requires one to sixteen steps",
        )?;
        if let TrialBase::Saved { servers, client } = &self.base {
            require(
                !servers.is_empty() && servers.len() <= 2,
                FaultKind::Limit,
                "Saved trial requires one to two selected servers",
            )?;
            require(
                id(client) && servers.iter().all(|s| id(s)) && unique(servers),
                FaultKind::Context,
                "Invalid selected saved-trial context",
            )?;
        }
        for s in &self.steps {
            s.action.validate()?;
            s.expected.validate()?;
            require(
                s.action.trial_allowed(),
                FaultKind::SimulatorUnsupported,
                "Fresh trial cannot install policies, save editors or replace inventory",
            )?;
        }
        compact(self, MAX_ACTION)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    #[serde(deserialize_with = "list::<_,Request,32>")]
    pub current: Vec<Request>,
    #[serde(deserialize_with = "list::<_,Trial,4>")]
    pub fresh: Vec<Trial>,
}
impl Evaluation {
    pub fn from_json(raw: &str) -> Result<Self> {
        require(
            raw.len() <= MAX_ACTION,
            FaultKind::Limit,
            "Raw evaluation exceeds limit before decoding",
        )?;
        let p: Self = serde_json::from_str(raw).map_err(|_| Fault {
            kind: FaultKind::Malformed,
            message: "Invalid bounded NFS evaluation JSON".into(),
        })?;
        p.validate()?;
        Ok(p)
    }
    fn validate(&self) -> Result<()> {
        require(
            self.current.len() <= 32 && self.fresh.len() <= 4,
            FaultKind::Limit,
            "Current/fresh evaluation collection bound",
        )?;
        let mut calls = self.current.len();
        for t in &self.fresh {
            require(
                t.steps.len() <= 16,
                FaultKind::Limit,
                "Fresh step count before typed work",
            )?;
            calls = calls.checked_add(1 + t.steps.len()).ok_or_else(|| Fault {
                kind: FaultKind::Limit,
                message: "Evaluation call overflow".into(),
            })?;
        }
        require(
            calls <= MAX_CALLS,
            FaultKind::Limit,
            "Combined current/fresh call budget",
        )?;
        for r in &self.current {
            r.validate()?
        }
        for t in &self.fresh {
            t.validate()?
        }
        compact(self, MAX_ACTION)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Observation {
    Decision { decision: Decision },
    Fault { fault: Fault },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct StepReport {
    pub passed: bool,
    pub observation: Observation,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TrialReport {
    pub passed: bool,
    pub initialization_fault: Option<Fault>,
    pub steps: Vec<StepReport>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub current: Vec<Observation>,
    pub fresh: Vec<TrialReport>,
}
impl State {
    pub fn evaluate_batch(&self, plan: &Evaluation) -> Result<Report> {
        plan.validate()?;
        self.validate()?;
        // Charge report wrappers and boolean/option keys as well as each exact payload.
        let mut charge = Charge { used: 0 };
        charge.bytes(
            32 + plan.current.len() * 40
                + plan.fresh.len() * 100
                + plan.fresh.iter().map(|t| t.steps.len() * 80).sum::<usize>(),
        )?;
        let mut current = Vec::new();
        for r in &plan.current {
            let decision = match r {
                Request::MountPresent { client, mountpoint } => {
                    let result =
                        if !self
                            .inventory
                            .client(client)
                            .is_some_and(|c| c.mountpoints.contains(mountpoint))
                        {
                            fail(FaultKind::Context, "Current mount request is foreign")
                        } else if self.runtime.mounts.iter().any(|m| {
                            m.client == *client && m.specification.mountpoint == *mountpoint
                        }) {
                            Ok(Decision::Mounted {})
                        } else {
                            Ok(denied(DenialStage::MountAbsent))
                        };
                    if let Ok(d) = &result {
                        charge.value(d)?
                    }
                    result
                }
                Request::Read {
                    client,
                    mountpoint,
                    profile,
                    relative,
                } => self.read_inner(client, mountpoint, profile, relative, false, &mut charge),
                Request::WritePermission {
                    client,
                    mountpoint,
                    profile,
                    relative,
                } => self.read_inner(client, mountpoint, profile, relative, true, &mut charge),
            };
            current.push(observe(decision, &mut charge)?);
        }
        let mut fresh = Vec::new();
        for trial in &plan.fresh {
            // A single scratch copy lives only until this trial report is complete.
            let initialized = self.trial_state(&trial.base);
            let mut scratch = match initialized {
                Ok(s) => s,
                Err(fault) => {
                    charge.value(&fault)?;
                    fresh.push(TrialReport {
                        passed: false,
                        initialization_fault: Some(fault),
                        steps: Vec::new(),
                    });
                    continue;
                }
            };
            let mut steps = Vec::new();
            let mut passed = true;
            for step in &trial.steps {
                let scoped = trial_scope(&scratch, &trial.base, &step.action);
                let result = match scoped {
                    Ok(()) => scratch.apply_inner(&step.action, &mut charge),
                    Err(f) => Err(f),
                };
                // Read/write decisions charge before copying/mutation; these small administrative results have no text.
                if !matches!(
                    step.action,
                    Action::ReadExisting { .. } | Action::WriteExisting { .. }
                ) && let Ok(d) = &result
                {
                    charge.value(d)?
                }
                let ok = result.as_ref().is_ok_and(|d| step.expected.matches(d));
                passed &= ok;
                steps.push(StepReport {
                    passed: ok,
                    observation: observe(result, &mut charge)?,
                });
            }
            fresh.push(TrialReport {
                passed,
                initialization_fault: None,
                steps,
            });
        }
        let report = Report { current, fresh };
        compact(&report, MAX_RETURNED)?;
        Ok(report)
    }
    fn trial_state(&self, base: &TrialBase) -> Result<Self> {
        // Parse and preflight every selected saved table before the sole bounded State copy.
        let mut tables = Vec::new();
        if let TrialBase::Saved { servers, client } = base {
            let n = self.client_index(client)?;
            let mounts = parse_fstab(&self.inventory, client, &self.runtime.clients[n].saved)?;
            require(
                mounts.iter().all(|m| servers.contains(&m.server)),
                FaultKind::Context,
                "Saved trial document references an unselected server",
            )?;
            for server in servers {
                let n = self.server_index(server)?;
                tables.push((
                    n,
                    parse_exports(&self.inventory, server, &self.runtime.servers[n].saved)?,
                ));
            }
        }
        // Clearing mounts reduces the candidate; selected table changes are counted jointly before cloning.
        let current = size(self, MAX_STATE).expect("validated bounded State");
        let mounts = size(&self.runtime.mounts, MAX_STATE).expect("validated mounts");
        let mut proposed = current - mounts + 2;
        if matches!(base, TrialBase::Saved { .. }) {
            for (n, s) in self.runtime.servers.iter().enumerate() {
                let old = size(&s.applied, MAX_STATE).expect("validated table");
                let new = tables
                    .iter()
                    .find(|(k, _)| *k == n)
                    .map_or(Some(2), |(_, t)| size(t, MAX_STATE))
                    .ok_or_else(|| Fault {
                        kind: FaultKind::Limit,
                        message: "Saved trial table aggregate".into(),
                    })?;
                proposed = proposed
                    .checked_sub(old)
                    .and_then(|v| v.checked_add(new))
                    .ok_or_else(|| Fault {
                        kind: FaultKind::Limit,
                        message: "Saved trial State size overflow".into(),
                    })?;
            }
        }
        require(
            proposed <= MAX_STATE,
            FaultKind::Limit,
            "Fresh proposed State aggregate before clone",
        )?;
        let mut scratch = self.clone();
        scratch.runtime.mounts.clear();
        if matches!(base, TrialBase::Saved { .. }) {
            for s in &mut scratch.runtime.servers {
                s.applied.clear()
            }
            for (n, t) in tables {
                scratch.runtime.servers[n].applied = t
            }
        }
        Ok(scratch)
    }
}
fn observe(result: Result<Decision>, charge: &mut Charge) -> Result<Observation> {
    match result {
        Ok(decision) => Ok(Observation::Decision { decision }),
        Err(fault) => {
            if fault.kind == FaultKind::Limit {
                return Err(fault);
            }
            charge.value(&fault)?;
            Ok(Observation::Fault { fault })
        }
    }
}
fn trial_scope(s: &State, base: &TrialBase, a: &Action) -> Result<()> {
    if let TrialBase::Saved { servers, client } = base {
        let action_client = match a {
            Action::MountExplicit { client, .. }
            | Action::MountSaved { client, .. }
            | Action::Unmount { client, .. }
            | Action::ReadExisting { client, .. }
            | Action::WriteExisting { client, .. } => client,
            _ => {
                return fail(
                    FaultKind::SimulatorUnsupported,
                    "Not a fresh trial operation",
                );
            }
        };
        require(
            action_client == client,
            FaultKind::Context,
            "Saved trial operation client differs from selected client",
        )?;
        if let Action::MountExplicit { specification, .. } = a {
            require(
                servers.contains(&specification.server),
                FaultKind::Context,
                "Saved trial operation server was not selected",
            )?;
        }
        require(
            s.inventory.client(client).is_some(),
            FaultKind::Context,
            "Saved trial client is foreign",
        )?;
    }
    Ok(())
}
