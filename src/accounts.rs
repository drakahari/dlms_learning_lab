//! Bounded, in-process account records and numeric virtual ownership.
//! No host account, filesystem, process, authentication or clock APIs.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const MAX_USERS: usize = 32;
pub const MAX_GROUPS: usize = 32;
pub const MAX_ENTRIES: usize = 256;
pub const MAX_TEXT_BYTES: usize = 262_144;
pub const MAX_FILE_BYTES: usize = 65_536;
pub const MAX_STATE_BYTES: usize = 4_194_304;
pub const MAX_OUTPUT_BYTES: usize = 32_768;
pub const MAX_REQUIREMENTS: usize = 64;
pub const MAX_ERROR_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
    Missing,
    Conflict,
    Protected,
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
type Result<T> = std::result::Result<T, Error>;
fn err(kind: ErrorKind, text: impl Into<String>) -> Error {
    let text = text.into();
    let truncated = text.len() > MAX_ERROR_BYTES;
    let mut end = text.len().min(if truncated {
        MAX_ERROR_BYTES - "…".len()
    } else {
        MAX_ERROR_BYTES
    });
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    // Retain a new small allocation, never a truncated multi-megabyte capacity.
    let mut message = String::with_capacity(if truncated { MAX_ERROR_BYTES } else { end });
    message.push_str(&text[..end]);
    if truncated {
        message.push('…');
    }
    Error { kind, message }
}
fn invalid() -> Error {
    err(
        ErrorKind::Invalid,
        "Invalid bounded account state or argument",
    )
}
fn unsupported() -> Error {
    err(
        ErrorKind::Unsupported,
        "Unsupported account command: see the bounded account help",
    )
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes()
            .next()
            .is_some_and(|b| b.is_ascii_lowercase() || b == b'_')
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-".contains(&b))
}
fn path(s: &str) -> bool {
    s == "/"
        || (s.starts_with('/')
            && s.len() <= 128
            && !s.ends_with('/')
            && s[1..].split('/').count() <= 8
            && s[1..].split('/').all(|p| {
                !p.is_empty()
                    && p != "."
                    && p != ".."
                    && p.bytes()
                        .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
            }))
}
fn within(p: &str, root: &str) -> bool {
    p == root || p.strip_prefix(root).is_some_and(|s| s.starts_with('/'))
}
fn parent(p: &str) -> &str {
    p.rsplit_once('/')
        .map_or("/", |(p, _)| if p.is_empty() { "/" } else { p })
}
fn home(s: &str) -> bool {
    path(s) && parent(s) == "/home"
}
fn decimal(s: &str) -> Option<u32> {
    if s.is_empty() || (s.len() > 1 && s.starts_with('0')) || !s.bytes().all(|b| b.is_ascii_digit())
    {
        None
    } else {
        s.parse().ok()
    }
}
fn human_id(id: u32) -> bool {
    (1000..=60000).contains(&id)
}
fn short<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> std::result::Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> serde::de::Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} UTF-8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<String, E> {
            if s.len() > N {
                Err(E::custom("Account string limit"))
            } else {
                Ok(s.to_owned())
            }
        }
    }
    d.deserialize_str(Text::<N>)
}
fn text32<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    short::<D, 32>(d)
}
fn text80<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    short::<D, 80>(d)
}
fn text128<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    short::<D, 128>(d)
}
fn contents<'de, D: serde::Deserializer<'de>>(d: D) -> std::result::Result<String, D::Error> {
    short::<D, MAX_FILE_BYTES>(d)
}

fn map<'de, D: serde::Deserializer<'de>, V: Deserialize<'de>, const N: usize>(
    d: D,
) -> std::result::Result<BTreeMap<String, V>, D::Error> {
    use serde::de::{Error as _, MapAccess, Visitor};
    struct Entries<V, const N: usize>(std::marker::PhantomData<V>);
    impl<'de, V: Deserialize<'de>, const N: usize> Visitor<'de> for Entries<V, N> {
        type Value = BTreeMap<String, V>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} distinct bounded account keys")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            while let Some(k) = a.next_key::<Key>()? {
                // Inspect a key/count BEFORE decoding the next potentially large value.
                if out.len() >= N || out.contains_key(&k.0) {
                    return Err(A::Error::custom(
                        "Duplicate account key or collection limit",
                    ));
                }
                out.insert(k.0, a.next_value()?);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Entries::<V, N>(std::marker::PhantomData))
}
struct Key(String);
impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        short::<D, 128>(d).map(Self)
    }
}
fn users<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, User>, D::Error> {
    map::<D, User, MAX_USERS>(d)
}
fn groups<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, Group>, D::Error> {
    map::<D, Group, MAX_GROUPS>(d)
}
fn entries<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeMap<String, Entry>, D::Error> {
    use serde::de::{Error as _, MapAccess, Visitor};
    struct Entries;
    impl<'de> Visitor<'de> for Entries {
        type Value = BTreeMap<String, Entry>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("bounded virtual entries")
        }
        fn visit_map<A: MapAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            let mut total = 0usize;
            while let Some(k) = a.next_key::<Key>()? {
                if out.len() >= MAX_ENTRIES || out.contains_key(&k.0) {
                    return Err(A::Error::custom("Duplicate/oversized virtual entries"));
                }
                let e: Entry = a.next_value()?;
                total += e.bytes();
                if total > MAX_TEXT_BYTES {
                    return Err(A::Error::custom("Aggregate virtual text limit"));
                }
                out.insert(k.0, e);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Entries)
}
fn outcomes<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Vec<Requirement>, D::Error> {
    use serde::de::{Error as _, SeqAccess, Visitor};
    struct Outcomes;
    impl<'de> Visitor<'de> for Outcomes {
        type Value = Vec<Requirement>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("bounded account outcomes")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while out.len() < MAX_REQUIREMENTS {
                match a.next_element()? {
                    Some(r) => out.push(r),
                    None => return Ok(out),
                }
            }
            // Reject before decoding a65th potentially large outcome value.
            if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(A::Error::custom("Account outcome count limit"));
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Outcomes)
}
fn set<'de, D: serde::Deserializer<'de>, T: Deserialize<'de> + Ord, const N: usize>(
    d: D,
) -> std::result::Result<BTreeSet<T>, D::Error> {
    use serde::de::{Error as _, SeqAccess, Visitor};
    struct Items<T, const N: usize>(std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de> + Ord, const N: usize> Visitor<'de> for Items<T, N> {
        type Value = BTreeSet<T>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} unique items")
        }
        fn visit_seq<A: SeqAccess<'de>>(
            self,
            mut a: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut out = BTreeSet::new();
            while let Some(x) = a.next_element::<T>()? {
                if out.len() >= N || !out.insert(x) {
                    return Err(A::Error::custom("Duplicate account set member or limit"));
                }
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Items::<T, N>(std::marker::PhantomData))
}
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Small(String);
impl<'de> Deserialize<'de> for Small {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        short::<D, 128>(d).map(Self)
    }
}
fn strings<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeSet<String>, D::Error> {
    set::<D, Small, MAX_ENTRIES>(d).map(|s| s.into_iter().map(|s| s.0).collect())
}
fn memberships<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<BTreeSet<u32>, D::Error> {
    set::<D, u32, 16>(d)
}

/// UTC days since1970-01-01; day0 deliberately excluded. No real-time API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u32", into = "u32")]
pub struct EpochDay(u32);
impl TryFrom<u32> for EpochDay {
    type Error = Error;
    fn try_from(n: u32) -> Result<Self> {
        if (1..=2_932_896).contains(&n) {
            Ok(Self(n))
        } else {
            Err(invalid())
        }
    }
}
impl From<EpochDay> for u32 {
    fn from(d: EpochDay) -> u32 {
        d.0
    }
}
fn before_year(y: u32) -> u32 {
    let n = y - 1;
    n * 365 + n / 4 - n / 100 + n / 400
}
fn leap(y: u32) -> bool {
    y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400))
}
fn month_days(y: u32, m: u32) -> u32 {
    match m {
        2 => {
            if leap(y) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
impl EpochDay {
    pub fn parse(s: &str) -> Result<Self> {
        if let Some(n) = decimal(s) {
            return Self::try_from(n);
        }
        if s.len() != 10
            || !s.bytes().enumerate().all(|(i, b)| {
                if i == 4 || i == 7 {
                    b == b'-'
                } else {
                    b.is_ascii_digit()
                }
            })
        {
            return Err(invalid());
        }
        let y: u32 = s[..4].parse().map_err(|_| invalid())?;
        let m: u32 = s[5..7].parse().map_err(|_| invalid())?;
        let d: u32 = s[8..].parse().map_err(|_| invalid())?;
        if !(1970..=9999).contains(&y) || !(1..=12).contains(&m) || d == 0 || d > month_days(y, m) {
            return Err(invalid());
        }
        let n =
            before_year(y) - before_year(1970) + (1..m).map(|m| month_days(y, m)).sum::<u32>() + d
                - 1;
        Self::try_from(n)
    }
    pub fn number(self) -> u32 {
        self.0
    }
    pub fn iso(self) -> String {
        let mut y = 1970;
        let mut n = self.0;
        while n >= if leap(y) { 366 } else { 365 } {
            n -= if leap(y) { 366 } else { 365 };
            y += 1;
        }
        let mut m = 1;
        while n >= month_days(y, m) {
            n -= month_days(y, m);
            m += 1;
        }
        format!("{y:04}-{m:02}-{:02}", n + 1)
    }
    /// Exact metadata comparison; deliberately not an authentication decision.
    pub fn compare(self, today: Self) -> std::cmp::Ordering {
        self.cmp(&today)
    }
}
fn expiry(s: &str) -> Result<Option<EpochDay>> {
    if s.is_empty() || s == "-1" {
        Ok(None)
    } else {
        EpochDay::parse(s).map(Some)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct User {
    pub uid: u32,
    pub primary_gid: u32,
    #[serde(deserialize_with = "memberships")]
    pub supplementary: BTreeSet<u32>,
    #[serde(deserialize_with = "text128")]
    pub home: String,
    #[serde(deserialize_with = "text128")]
    pub shell: String,
    #[serde(deserialize_with = "text80")]
    pub comment: String,
    pub expires: Option<EpochDay>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub gid: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Kind {
    Directory,
    Text {
        #[serde(deserialize_with = "contents")]
        contents: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub uid: u32,
    pub gid: u32,
    pub mode: u16,
    pub data: Kind,
}
impl Entry {
    pub fn directory(&self) -> bool {
        matches!(self.data, Kind::Directory)
    }
    fn bytes(&self) -> usize {
        match &self.data {
            Kind::Directory => 0,
            Kind::Text { contents } => contents.len(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    #[serde(deserialize_with = "text32")]
    pub fixture: String,
    pub today: EpochDay,
    #[serde(deserialize_with = "strings")]
    pub protected_users: BTreeSet<String>,
    #[serde(deserialize_with = "strings")]
    pub protected_groups: BTreeSet<String>,
    #[serde(deserialize_with = "strings")]
    pub protected_entries: BTreeSet<String>,
    #[serde(deserialize_with = "strings")]
    pub skeletons: BTreeSet<String>,
    #[serde(deserialize_with = "strings")]
    pub shells: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    #[serde(deserialize_with = "users")]
    pub users: BTreeMap<String, User>,
    #[serde(deserialize_with = "groups")]
    pub groups: BTreeMap<String, Group>,
    #[serde(deserialize_with = "entries")]
    pub entries: BTreeMap<String, Entry>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "StateData")]
pub struct State {
    policy: Policy,
    configuration: Configuration,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StateData {
    policy: Policy,
    configuration: Configuration,
}
impl TryFrom<StateData> for State {
    type Error = Error;
    fn try_from(x: StateData) -> Result<Self> {
        Self::new(x.policy, x.configuration)
    }
}
impl State {
    pub fn new(policy: Policy, configuration: Configuration) -> Result<Self> {
        let s = Self {
            policy,
            configuration,
        };
        if s.valid() { Ok(s) } else { Err(invalid()) }
    }
    pub fn policy(&self) -> &Policy {
        &self.policy
    }
    pub fn configuration(&self) -> &Configuration {
        &self.configuration
    }
    pub fn from_json(s: &str) -> Result<Self> {
        if s.len() > MAX_STATE_BYTES {
            return Err(err(ErrorKind::Limit, "Account JSON input limit"));
        }
        serde_json::from_str(s)
            .map_err(|e| err(ErrorKind::Invalid, format!("Invalid account JSON: {e}")))
    }
    pub fn valid(&self) -> bool {
        Self::valid_configuration(&self.policy, &self.configuration)
    }
    // Borrow first: directly constructed outcome data has not necessarily passed
    // any decoder and must not be cloned until these bounds are established.
    fn valid_configuration(p: &Policy, c: &Configuration) -> bool {
        if !name(&p.fixture)
            || !p.protected_users.contains("root")
            || !p.protected_groups.contains("root")
            || !p.skeletons.contains("/etc/skel")
            || p.protected_users.len() > MAX_USERS
            || p.protected_groups.len() > MAX_GROUPS
            || p.protected_entries.len() > MAX_ENTRIES
            || p.skeletons.len() > 8
            || p.shells.is_empty()
            || p.shells.len() > 8
            || p.shells
                .iter()
                .any(|s| !path(s) || !s.starts_with("/bin/") && !s.starts_with("/usr/sbin/"))
            || p.skeletons.iter().any(|s| !path(s) || !within(s, "/etc"))
            || ["/", "/home", "/etc"]
                .iter()
                .any(|s| !p.protected_entries.contains(*s))
            || p.protected_entries
                .iter()
                .any(|s| !c.entries.contains_key(s))
            || p.protected_users.iter().any(|s| !c.users.contains_key(s))
            || p.protected_groups.iter().any(|s| !c.groups.contains_key(s))
            || p.skeletons
                .iter()
                .any(|s| !c.entries.get(s).is_some_and(Entry::directory))
            || c.users.is_empty()
            || c.users.len() > MAX_USERS
            || c.groups.is_empty()
            || c.groups.len() > MAX_GROUPS
            || c.entries.len() > MAX_ENTRIES
        {
            return false;
        }
        let mut uids = BTreeSet::new();
        let mut gids = BTreeSet::new();
        let mut homes = BTreeSet::new();
        for (n, g) in &c.groups {
            if !name(n)
                || !gids.insert(g.gid)
                || g.gid > 60000
                || (n == "root") != (g.gid == 0)
                || (!p.protected_groups.contains(n) && !human_id(g.gid))
            {
                return false;
            }
        }
        for (n, u) in &c.users {
            if !name(n)
                || !uids.insert(u.uid)
                || u.uid > 60000
                || (n == "root") != (u.uid == 0)
                || (!p.protected_users.contains(n) && !human_id(u.uid))
                || !gids.contains(&u.primary_gid)
                || u.supplementary.len() > 16
                || u.supplementary.iter().any(|g| !gids.contains(g))
                || !path(&u.home)
                || (!p.protected_users.contains(n) && !home(&u.home))
                || !homes.insert(&u.home)
                || c.entries.get(&u.home).is_some_and(|e| !e.directory())
                || !p.shells.contains(&u.shell)
                || u.comment.len() > 80
                || u.comment.chars().any(|c| c.is_control() || c == ':')
            {
                return false;
            }
        }
        let mut total = 0usize;
        for (s, e) in &c.entries {
            if !path(s)
                || e.uid > 60000
                || e.gid > 60000
                || e.mode > 0o777
                || e.bytes() > MAX_FILE_BYTES
                || ["/etc/passwd", "/etc/group", "/etc/shadow"].contains(&s.as_str())
                || (s != "/" && !c.entries.get(parent(s)).is_some_and(Entry::directory))
                || matches!(&e.data,Kind::Text{contents}if contents.contains('\0'))
            {
                return false;
            }
            total += e.bytes();
            if total > MAX_TEXT_BYTES {
                return false;
            }
        }
        ["/", "/home", "/etc"]
            .iter()
            .all(|p| c.entries.get(*p).is_some_and(Entry::directory))
    }
    /// Imported policy is never authority: callers pin the original fixture.
    pub fn valid_against(&self, initial: &Self) -> bool {
        self.valid()
            && self.policy == initial.policy
            && self.protected_same(initial)
            && initial.configuration.users.iter().all(|(n, u)| {
                self.configuration
                    .users
                    .get(n)
                    .is_none_or(|x| x.uid == u.uid)
            })
            && initial.configuration.groups.iter().all(|(n, g)| {
                self.configuration
                    .groups
                    .get(n)
                    .is_none_or(|x| x.gid == g.gid)
            })
    }
    fn protected_same(&self, old: &Self) -> bool {
        let p = &self.policy;
        let c = &self.configuration;
        let b = &old.configuration;
        p.protected_users
            .iter()
            .all(|n| c.users.get(n) == b.users.get(n))
            && p.protected_groups
                .iter()
                .all(|n| c.groups.get(n) == b.groups.get(n))
            && b.entries
                .iter()
                .filter(|(s, _)| {
                    p.protected_entries.contains(*s) || p.skeletons.iter().any(|r| within(s, r))
                })
                .all(|(s, e)| c.entries.get(s) == Some(e))
            && c.entries
                .keys()
                .filter(|s| p.skeletons.iter().any(|r| within(s, r)))
                .all(|s| b.entries.contains_key(s))
    }
    fn mutable_user(&self, n: &str) -> Result<&User> {
        if self.policy.protected_users.contains(n) {
            return Err(err(ErrorKind::Protected, "Protected simulated account"));
        }
        self.configuration
            .users
            .get(n)
            .ok_or_else(|| err(ErrorKind::Missing, "Unknown simulated account"))
    }
    fn gid(&self, s: &str) -> Result<u32> {
        self.configuration
            .groups
            .get(s)
            .map(|g| g.gid)
            .or_else(|| {
                decimal(s).filter(|g| self.configuration.groups.values().any(|v| v.gid == *g))
            })
            .ok_or_else(|| err(ErrorKind::Missing, "Unknown simulated group"))
    }
    fn uid(&self, s: &str) -> Result<u32> {
        self.configuration
            .users
            .get(s)
            .map(|u| u.uid)
            .or_else(|| {
                decimal(s).filter(|u| self.configuration.users.values().any(|v| v.uid == *u))
            })
            .ok_or_else(|| err(ErrorKind::Missing, "Unknown simulated user"))
    }
    fn group_list(&self, s: &str) -> Result<BTreeSet<u32>> {
        if s.is_empty() {
            return Ok(BTreeSet::new());
        }
        if s.len() > 1055 {
            return Err(err(ErrorKind::Limit, "Supplementary group list limit"));
        }
        let mut out = BTreeSet::new();
        for n in s.split(',') {
            out.insert(self.gid(n)?);
            if out.len() > 16 {
                return Err(err(ErrorKind::Limit, "At most16 supplementary groups"));
            }
        }
        Ok(out)
    }
    pub fn apply(&mut self, action: &Action) -> Result<()> {
        // All state is bounded; one bounded clone is the transaction preflight.
        let mut next = self.clone();
        next.change(action)?;
        if !next.protected_same(self) {
            return Err(err(
                ErrorKind::Protected,
                "The operation would change a protected record or entry",
            ));
        }
        if !next.valid() {
            return Err(invalid());
        }
        *self = next;
        Ok(())
    }
    fn change(&mut self, a: &Action) -> Result<()> {
        match a {
            Action::AddGroup { name: n, gid } => {
                if !name(n) || !human_id(*gid) {
                    return Err(invalid());
                }
                if self.configuration.groups.contains_key(n)
                    || self.configuration.groups.values().any(|g| g.gid == *gid)
                    || self.configuration.entries.values().any(|e| e.gid == *gid)
                {
                    return Err(err(
                        ErrorKind::Conflict,
                        "Group name/GID is occupied, including retained ownership",
                    ));
                }
                self.configuration
                    .groups
                    .insert(n.clone(), Group { gid: *gid });
            }
            Action::RenameGroup { group, new_name } => {
                if !name(new_name) {
                    return Err(invalid());
                }
                if self.policy.protected_groups.contains(group) {
                    return Err(err(ErrorKind::Protected, "Protected simulated group"));
                }
                if self.configuration.groups.contains_key(new_name) {
                    return Err(err(ErrorKind::Conflict, "Group name is occupied"));
                }
                let g = self
                    .configuration
                    .groups
                    .remove(group)
                    .ok_or_else(|| err(ErrorKind::Missing, "Unknown simulated group"))?;
                self.configuration.groups.insert(new_name.clone(), g);
            }
            Action::DeleteGroup { group } => {
                if self.policy.protected_groups.contains(group) {
                    return Err(err(ErrorKind::Protected, "Protected simulated group"));
                }
                let gid = self.gid(group)?;
                // Name operand only; no implicit numeric group deletion.
                if !self.configuration.groups.contains_key(group) {
                    return Err(invalid());
                }
                if self
                    .configuration
                    .users
                    .values()
                    .any(|u| u.primary_gid == gid)
                {
                    return Err(err(
                        ErrorKind::Conflict,
                        "A current account uses this primary group",
                    ));
                }
                self.configuration.groups.remove(group);
                for u in self.configuration.users.values_mut() {
                    u.supplementary.remove(&gid);
                }
            }
            Action::AddUser {
                name: n,
                user,
                create_home,
                skeleton,
            } => {
                if !name(n)
                    || !human_id(user.uid)
                    || !home(&user.home)
                    || !self.policy.shells.contains(&user.shell)
                    || user.supplementary.len() > 16
                    || user.comment.len() > 80
                    || user.comment.chars().any(|c| c.is_control() || c == ':')
                {
                    return Err(invalid());
                }
                if self.configuration.users.contains_key(n)
                    || self
                        .configuration
                        .users
                        .values()
                        .any(|u| u.uid == user.uid || u.home == user.home)
                    || self
                        .configuration
                        .entries
                        .values()
                        .any(|e| e.uid == user.uid)
                {
                    return Err(err(
                        ErrorKind::Conflict,
                        "User name/UID/home is occupied, including retained ownership",
                    ));
                }
                if self.configuration.entries.contains_key(&user.home) {
                    return Err(err(
                        ErrorKind::Unsupported,
                        "Adopting an existing home is outside this subset",
                    ));
                }
                if !self
                    .configuration
                    .groups
                    .values()
                    .any(|g| g.gid == user.primary_gid)
                    || user
                        .supplementary
                        .iter()
                        .any(|g| !self.configuration.groups.values().any(|x| x.gid == *g))
                {
                    return Err(err(
                        ErrorKind::Missing,
                        "Primary/supplementary group does not exist",
                    ));
                }
                if *create_home {
                    let skel = skeleton.as_deref().unwrap_or("/etc/skel");
                    if !self.policy.skeletons.contains(skel) {
                        return Err(err(
                            ErrorKind::Unsupported,
                            "Only a supplied skeleton is supported",
                        ));
                    }
                    let mut copied = Vec::new();
                    for (s, e) in &self.configuration.entries {
                        if within(s, skel) {
                            let mut e = e.clone();
                            e.uid = user.uid;
                            e.gid = user.primary_gid;
                            let target = format!("{}{}", user.home, &s[skel.len()..]);
                            if s == skel {
                                e.mode = 0o700;
                            }
                            copied.push((target, e));
                        }
                    }
                    for (s, e) in copied {
                        self.configuration.entries.insert(s, e);
                    }
                } else if skeleton.is_some() {
                    return Err(unsupported());
                }
                self.configuration.users.insert(n.clone(), user.clone());
            }
            Action::Supplementary {
                user,
                groups,
                append,
            } => {
                self.mutable_user(user)?;
                let gs = self.group_list(groups)?;
                let u = self.configuration.users.get_mut(user).unwrap();
                if *append {
                    u.supplementary.extend(gs);
                } else {
                    u.supplementary = gs;
                }
            }
            Action::PrimaryGroup { user, group } => {
                let u = self.mutable_user(user)?.clone();
                let gid = self.gid(group)?;
                if self
                    .configuration
                    .entries
                    .get(&u.home)
                    .is_some_and(|root| !root.directory() || root.uid != u.uid)
                {
                    return Err(err(
                        ErrorKind::Unsupported,
                        "Primary-GID changes with a foreign-owned/unsupported home are excluded",
                    ));
                }
                for (s, e) in &mut self.configuration.entries {
                    if within(s, &u.home) && e.gid == u.primary_gid {
                        e.gid = gid;
                    }
                }
                self.configuration.users.get_mut(user).unwrap().primary_gid = gid;
            }
            Action::Expire { user, date } => {
                self.mutable_user(user)?;
                let d = expiry(date)?;
                self.configuration.users.get_mut(user).unwrap().expires = d;
            }
            Action::Own {
                path: p,
                user,
                group,
            } => {
                if !path(p) {
                    return Err(invalid());
                }
                let uid = self.uid(user)?;
                let gid = self.gid(group)?;
                let e = self
                    .configuration
                    .entries
                    .get_mut(p)
                    .ok_or_else(|| err(ErrorKind::Missing, "Unknown virtual entry"))?;
                e.uid = uid;
                e.gid = gid;
            }
            Action::DeleteUser { user, remove_home } => {
                let u = self.mutable_user(user)?.clone();
                if *remove_home {
                    if !home(&u.home) {
                        return Err(unsupported());
                    }
                    if self
                        .configuration
                        .entries
                        .get(&u.home)
                        .is_some_and(|e| !e.directory() || e.uid != u.uid)
                    {
                        return Err(err(
                            ErrorKind::Unsupported,
                            "Removing foreign-owned/unsupported homes is excluded",
                        ));
                    }
                    self.configuration
                        .entries
                        .retain(|p, _| !within(p, &u.home));
                }
                self.configuration.users.remove(user);
            }
        }
        Ok(())
    }
    pub fn execute(&mut self, line: &str) -> Result<Output> {
        match parse(line)? {
            Request::Change(a) => {
                self.apply(&a)?;
                Ok(Output::success(String::new()))
            }
            Request::Query(q) => self.query(&q),
            Request::CreateUser(spec) => {
                let primary_gid = self.gid(&spec.primary_group)?;
                let supplementary = self.group_list(&spec.supplementary_groups)?;
                let user = User {
                    uid: spec.uid,
                    primary_gid,
                    supplementary,
                    home: spec.home,
                    shell: spec.shell,
                    comment: String::new(),
                    expires: None,
                };
                self.apply(&Action::AddUser {
                    name: spec.name,
                    user,
                    create_home: spec.create_home,
                    skeleton: spec.skeleton,
                })?;
                Ok(Output::success(String::new()))
            }
        }
    }
    pub fn query(&self, q: &Query) -> Result<Output> {
        let c = &self.configuration;
        let text = match q {
            Query::Identity { user, names_only } => {
                let n = user.as_deref().unwrap_or("root");
                let u = c
                    .users
                    .get(n)
                    .ok_or_else(|| err(ErrorKind::Missing, "Unknown simulated user"))?;
                let gs = std::iter::once(u.primary_gid)
                    .chain(
                        u.supplementary
                            .iter()
                            .copied()
                            .filter(|g| *g != u.primary_gid),
                    )
                    .collect::<Vec<_>>();
                if *names_only {
                    format!(
                        "{}\n",
                        gs.iter()
                            .map(|g| self.group_name(*g))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                } else {
                    format!(
                        "uid={}({n}) gid={}({}) groups={}\n",
                        u.uid,
                        u.primary_gid,
                        self.group_name(u.primary_gid),
                        gs.iter()
                            .map(|g| format!("{g}({})", self.group_name(*g)))
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                }
            }
            Query::Database { database, key } => {
                let mut lines = Vec::new();
                match database {
                    Database::Passwd | Database::Shadow => {
                        let mut us: Vec<_> = c.users.iter().collect();
                        us.sort_by_key(|(_, u)| u.uid);
                        for (n, u) in us {
                            if key.as_ref().is_none_or(|k| {
                                k == n
                                    || (*database == Database::Passwd && decimal(k) == Some(u.uid))
                            }) {
                                lines.push(if *database == Database::Passwd {
                                    format!(
                                        "{n}:x:{}:{}:{}:{}:{}\n",
                                        u.uid, u.primary_gid, u.comment, u.home, u.shell
                                    )
                                } else {
                                    format!(
                                        "{n}:!::::::{}:\n",
                                        u.expires.map(|d| d.0.to_string()).unwrap_or_default()
                                    )
                                });
                            }
                        }
                    }
                    Database::Group => {
                        let mut gs: Vec<_> = c.groups.iter().collect();
                        gs.sort_by_key(|(_, g)| g.gid);
                        for (n, g) in gs {
                            if key
                                .as_ref()
                                .is_none_or(|k| k == n || decimal(k) == Some(g.gid))
                            {
                                lines.push(format!(
                                    "{n}:x:{}:{}\n",
                                    g.gid,
                                    c.users
                                        .iter()
                                        .filter(|(_, u)| u.supplementary.contains(&g.gid))
                                        .map(|(n, _)| n.as_str())
                                        .collect::<Vec<_>>()
                                        .join(",")
                                ));
                            }
                        }
                    }
                }
                if key.is_some() && lines.is_empty() {
                    return Ok(Output {
                        stdout: String::new(),
                        stderr: String::new(),
                        status: 2,
                    });
                }
                lines.concat()
            }
            Query::Read { path: p } => {
                if let Some(db) = match p.as_str() {
                    "/etc/passwd" => Some(Database::Passwd),
                    "/etc/group" => Some(Database::Group),
                    "/etc/shadow" => Some(Database::Shadow),
                    _ => None,
                } {
                    return self.query(&Query::Database {
                        database: db,
                        key: None,
                    });
                }
                if !path(p) {
                    return Err(invalid());
                }
                match &c
                    .entries
                    .get(p)
                    .ok_or_else(|| err(ErrorKind::Missing, "Unknown virtual entry"))?
                    .data
                {
                    Kind::Text { contents } => contents.clone(),
                    Kind::Directory => {
                        return Err(err(ErrorKind::Conflict, "Cannot read a directory as text"));
                    }
                }
            }
            Query::List {
                path: p,
                directory,
                numeric,
            } => {
                if !path(p) {
                    return Err(invalid());
                }
                let e = c
                    .entries
                    .get(p)
                    .ok_or_else(|| err(ErrorKind::Missing, "Unknown virtual entry"))?;
                if !directory && !e.directory() {
                    return Err(err(ErrorKind::Conflict, "List operand is not a directory"));
                }
                let selected: Vec<_> = if *directory {
                    vec![(p, e)]
                } else {
                    c.entries
                        .iter()
                        .filter(|(s, _)| s.as_str() != p && parent(s) == p)
                        .collect()
                };
                selected
                    .into_iter()
                    .map(|(s, e)| {
                        let u = if *numeric {
                            e.uid.to_string()
                        } else {
                            self.user_name(e.uid)
                        };
                        let g = if *numeric {
                            e.gid.to_string()
                        } else {
                            self.group_name(e.gid)
                        };
                        format!("{} {u} {g} {} {s}\n", permissions(e), e.bytes())
                    })
                    .collect::<Vec<_>>()
                    .concat()
            }
            Query::Show => {
                let mut t = format!(
                    "Simulator account snapshot · {} · supplied UTC date {} (day{})\nNo login/authentication/session model. Shadow sentinel ! contains no credential.\n",
                    self.policy.fixture,
                    self.policy.today.iso(),
                    self.policy.today.0
                );
                for (n, u) in &c.users {
                    let expiry = u.expires.map_or_else(
                        || "none".to_string(),
                        |d| {
                            let relation = match d.compare(self.policy.today) {
                                std::cmp::Ordering::Less => "before",
                                std::cmp::Ordering::Equal => "on",
                                std::cmp::Ordering::Greater => "after",
                            };
                            format!("{} (day{}, {relation} supplied date)", d.iso(), d.0)
                        },
                    );
                    let supplementary = u
                        .supplementary
                        .iter()
                        .map(|g| self.group_name(*g))
                        .collect::<Vec<_>>()
                        .join(",");
                    t.push_str(&format!("{n}: UID{}, primary {} (GID{}), supplementary [{supplementary}], home {}, shell {}, account-expiry policy {expiry}\n",u.uid,self.group_name(u.primary_gid),u.primary_gid,u.home,u.shell));
                }
                t
            }
        };
        if text.len() > MAX_OUTPUT_BYTES {
            return Err(err(ErrorKind::Limit, "Account query output limit"));
        }
        Ok(Output::success(text))
    }
    fn user_name(&self, id: u32) -> String {
        self.configuration
            .users
            .iter()
            .find(|(_, u)| u.uid == id)
            .map_or_else(|| id.to_string(), |(n, _)| n.clone())
    }
    fn group_name(&self, id: u32) -> String {
        self.configuration
            .groups
            .iter()
            .find(|(_, g)| g.gid == id)
            .map_or_else(|| id.to_string(), |(n, _)| n.clone())
    }
}
fn permissions(e: &Entry) -> String {
    let mut s = String::from(if e.directory() { "d" } else { "-" });
    for bit in (0..9).rev() {
        s.push(if e.mode & (1 << bit) == 0 {
            '-'
        } else {
            match bit % 3 {
                2 => 'r',
                1 => 'w',
                _ => 'x',
            }
        });
    }
    s
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}
impl Output {
    fn success(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
            status: 0,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    AddGroup {
        name: String,
        gid: u32,
    },
    RenameGroup {
        group: String,
        new_name: String,
    },
    DeleteGroup {
        group: String,
    },
    AddUser {
        name: String,
        user: User,
        create_home: bool,
        skeleton: Option<String>,
    },
    Supplementary {
        user: String,
        groups: String,
        append: bool,
    },
    PrimaryGroup {
        user: String,
        group: String,
    },
    Expire {
        user: String,
        date: String,
    },
    Own {
        path: String,
        user: String,
        group: String,
    },
    DeleteUser {
        user: String,
        remove_home: bool,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Database {
    Passwd,
    Group,
    Shadow,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Query {
    Identity {
        user: Option<String>,
        names_only: bool,
    },
    Database {
        database: Database,
        key: Option<String>,
    },
    Read {
        path: String,
    },
    List {
        path: String,
        directory: bool,
        numeric: bool,
    },
    Show,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    Query(Query),
    Change(Action),
    CreateUser(CreateUser),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateUser {
    pub name: String,
    pub uid: u32,
    pub primary_group: String,
    pub supplementary_groups: String,
    pub home: String,
    pub shell: String,
    pub create_home: bool,
    pub skeleton: Option<String>,
}

fn words(line: &str) -> Result<Vec<String>> {
    if line.len() > 2048 {
        return Err(err(ErrorKind::Limit, "Account command line limit"));
    }
    let mut out = Vec::new();
    let mut word = String::new();
    let mut quoted = None;
    let mut active = false;
    for c in line.chars() {
        if c == '\0' || c == '\n' || c == '\r' || c.is_control() && c != '\t' {
            return Err(unsupported());
        }
        if let Some(q) = quoted {
            if c == q {
                quoted = None;
            } else {
                if q == '"' && matches!(c, '$' | '`' | '\\') {
                    return Err(unsupported());
                }
                word.push(c);
            }
            active = true;
        } else {
            match c {
                '\'' | '"' => {
                    quoted = Some(c);
                    active = true;
                }
                ' ' | '\t' => {
                    if active {
                        out.push(std::mem::take(&mut word));
                        active = false;
                        if out.len() > 32 {
                            return Err(err(ErrorKind::Limit, "Account token limit"));
                        }
                    }
                }
                '$' | '`' | '\\' | '|' | '&' | ';' | '<' | '>' | '(' | ')' | '*' | '?' | '['
                | ']' => return Err(unsupported()),
                _ => {
                    word.push(c);
                    active = true;
                }
            }
        }
    }
    if quoted.is_some() {
        return Err(err(ErrorKind::Invalid, "Unclosed literal account argument"));
    }
    if active {
        out.push(word);
    }
    if out.len() > 32 {
        return Err(err(ErrorKind::Limit, "Account token limit"));
    }
    Ok(out)
}
/// Parsing is lexical only. Name-to-ID resolution happens against the state.
pub fn parse(line: &str) -> Result<Request> {
    let w = words(line)?;
    let Some(cmd) = w.first().map(String::as_str) else {
        return Err(unsupported());
    };
    let a = &w[1..];
    let q = |q| Ok(Request::Query(q));
    let change = |a| Ok(Request::Change(a));
    match cmd {
        "show" if a.is_empty() => q(Query::Show),
        "id" => match a.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
            [] => q(Query::Identity {
                user: None,
                names_only: false,
            }),
            [n] if name(n) => q(Query::Identity {
                user: Some((*n).into()),
                names_only: false,
            }),
            ["-Gn" | "-nG", n] if name(n) => q(Query::Identity {
                user: Some((*n).into()),
                names_only: true,
            }),
            _ => Err(unsupported()),
        },
        "getent" if a.len() == 1 || a.len() == 2 => {
            let database = match a[0].as_str() {
                "passwd" => Database::Passwd,
                "group" => Database::Group,
                "shadow" => Database::Shadow,
                _ => return Err(unsupported()),
            };
            let key = a.get(1).cloned();
            if key
                .as_ref()
                .is_some_and(|s| !name(s) && decimal(s).is_none())
            {
                return Err(invalid());
            }
            q(Query::Database { database, key })
        }
        "cat" if a.len() == 1 => q(Query::Read { path: a[0].clone() }),
        "ls" if a.len() == 2 => match a[0].as_str() {
            "-ld" => q(Query::List {
                path: a[1].clone(),
                directory: true,
                numeric: false,
            }),
            "-l" | "-ln" => q(Query::List {
                path: a[1].clone(),
                directory: false,
                numeric: a[0] == "-ln",
            }),
            _ => Err(unsupported()),
        },
        "chown" if a.len() == 2 => {
            let (user, group) = a[0].split_once(':').ok_or_else(unsupported)?;
            change(Action::Own {
                path: a[1].clone(),
                user: user.into(),
                group: group.into(),
            })
        }
        "groupdel" if a.len() == 1 => change(Action::DeleteGroup {
            group: a[0].clone(),
        }),
        "userdel" => {
            let (o, n) = options(a, &[('r', false)])?;
            change(Action::DeleteUser {
                user: n,
                remove_home: o.contains_key(&'r'),
            })
        }
        "groupadd" => {
            let (o, n) = options(a, &[('g', true)])?;
            if o.len() != 1 {
                return Err(unsupported());
            }
            change(Action::AddGroup {
                name: n,
                gid: decimal(&o[&'g']).ok_or_else(invalid)?,
            })
        }
        "groupmod" => {
            let (o, n) = options(a, &[('n', true)])?;
            if o.len() != 1 {
                return Err(unsupported());
            }
            change(Action::RenameGroup {
                group: n,
                new_name: o[&'n'].clone(),
            })
        }
        "usermod" => {
            let (o, n) = options(a, &[('a', false), ('G', true), ('g', true), ('e', true)])?;
            if o.contains_key(&'G') && o.len() == 1 + usize::from(o.contains_key(&'a')) {
                change(Action::Supplementary {
                    user: n,
                    groups: o[&'G'].clone(),
                    append: o.contains_key(&'a'),
                })
            } else if o.len() == 1 && o.contains_key(&'g') {
                change(Action::PrimaryGroup {
                    user: n,
                    group: o[&'g'].clone(),
                })
            } else if o.len() == 1 && o.contains_key(&'e') {
                expiry(&o[&'e'])?;
                change(Action::Expire {
                    user: n,
                    date: o[&'e'].clone(),
                })
            } else {
                Err(unsupported())
            }
        }
        "useradd" => {
            let (o, n) = options(
                a,
                &[
                    ('N', false),
                    ('u', true),
                    ('g', true),
                    ('G', true),
                    ('d', true),
                    ('s', true),
                    ('m', false),
                    ('M', false),
                    ('k', true),
                ],
            )?;
            if !['N', 'u', 'g', 'd', 's'].iter().all(|c| o.contains_key(c))
                || o.contains_key(&'m') == o.contains_key(&'M')
                || o.contains_key(&'k') && !o.contains_key(&'m')
            {
                return Err(unsupported());
            }
            let uid = decimal(&o[&'u']).ok_or_else(invalid)?;
            Ok(Request::CreateUser(CreateUser {
                name: n,
                uid,
                primary_group: o[&'g'].clone(),
                supplementary_groups: o.get(&'G').cloned().unwrap_or_default(),
                home: o[&'d'].clone(),
                shell: o[&'s'].clone(),
                create_home: o.contains_key(&'m'),
                skeleton: o.get(&'k').cloned(),
            }))
        }
        _ => Err(unsupported()),
    }
}
fn options(a: &[String], supported: &[(char, bool)]) -> Result<(BTreeMap<char, String>, String)> {
    let mut out = BTreeMap::new();
    let mut i = 0;
    while i + 1 < a.len() {
        if a[i] == "-aG" {
            if !supported.contains(&('a', false))
                || !supported.contains(&('G', true))
                || i + 2 >= a.len()
                || out.insert('a', String::new()).is_some()
                || out.insert('G', a[i + 1].clone()).is_some()
            {
                return Err(unsupported());
            }
            i += 2;
            continue;
        }
        let opt = match a[i].as_str() {
            "-a" | "--append" => 'a',
            "-G" | "--groups" => 'G',
            "-g" | "--gid" => 'g',
            "-u" | "--uid" => 'u',
            "-d" | "--home-dir" => 'd',
            "-s" | "--shell" => 's',
            "-m" | "--create-home" => 'm',
            "-M" | "--no-create-home" => 'M',
            "-N" | "--no-user-group" => 'N',
            "-k" | "--skel" => 'k',
            "-r" | "--remove" => 'r',
            "-n" | "--new-name" => 'n',
            "-e" | "--expiredate" => 'e',
            _ => return Err(unsupported()),
        };
        let takes = supported
            .iter()
            .find(|(c, _)| *c == opt)
            .ok_or_else(unsupported)?
            .1;
        let value = if takes {
            i += 1;
            if i + 1 >= a.len() {
                return Err(unsupported());
            }
            a[i].clone()
        } else {
            String::new()
        };
        if out.insert(opt, value).is_some() {
            return Err(unsupported());
        }
        i += 1;
    }
    if i != a.len().saturating_sub(1) || !a.last().is_some_and(|n| name(n)) {
        return Err(unsupported());
    }
    Ok((out, a.last().unwrap().clone()))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Requirement {
    User {
        #[serde(deserialize_with = "text32")]
        name: String,
        expected: User,
    },
    Group {
        #[serde(deserialize_with = "text32")]
        name: String,
        expected: Group,
    },
    UserAbsent {
        #[serde(deserialize_with = "text32")]
        name: String,
        uid: u32,
    },
    GroupAbsent {
        #[serde(deserialize_with = "text32")]
        name: String,
        gid: u32,
    },
    Entry {
        #[serde(deserialize_with = "text128")]
        path: String,
        expected: Entry,
    },
    AbsentPath {
        #[serde(deserialize_with = "text128")]
        path: String,
    },
    Configuration {
        expected: Configuration,
    },
    ExpiryRange {
        #[serde(deserialize_with = "text32")]
        name: String,
        first: EpochDay,
        last: EpochDay,
    },
}
pub fn valid_requirements(initial: &State, requirements: &[Requirement]) -> bool {
    if !initial.valid() || requirements.is_empty() || requirements.len() > MAX_REQUIREMENTS {
        return false;
    }
    let possible_groups: BTreeSet<_> = initial
        .configuration
        .groups
        .values()
        .map(|g| g.gid)
        .chain(requirements.iter().filter_map(|r| {
            if let Requirement::Group { expected, .. } = r {
                Some(expected.gid)
            } else {
                None
            }
        }))
        .collect();
    requirements.iter().all(|r| match r {
        Requirement::Configuration { expected } => {
            State::valid_configuration(&initial.policy, expected)
                && State::new(initial.policy.clone(), expected.clone())
                    .is_ok_and(|s| s.valid_against(initial))
        }
        Requirement::User { name: n, expected } => {
            name(n)
                && initial
                    .configuration
                    .users
                    .get(n)
                    .is_none_or(|u| u.uid == expected.uid)
                && initial
                    .configuration
                    .users
                    .iter()
                    .all(|(other, u)| other == n || u.uid != expected.uid)
                && possible_groups.contains(&expected.primary_gid)
                && expected
                    .supplementary
                    .iter()
                    .all(|g| possible_groups.contains(g))
                && expected.supplementary.len() <= 16
                && expected.comment.len() <= 80
                && !expected.comment.chars().any(|c| c.is_control() || c == ':')
                && initial.policy.shells.contains(&expected.shell)
                && if initial.policy.protected_users.contains(n) {
                    initial.configuration.users.get(n) == Some(expected)
                } else {
                    human_id(expected.uid) && home(&expected.home)
                }
        }
        Requirement::Group { name: n, expected } => {
            name(n)
                && initial
                    .configuration
                    .groups
                    .get(n)
                    .is_none_or(|g| g.gid == expected.gid)
                && if initial.policy.protected_groups.contains(n) {
                    initial.configuration.groups.get(n) == Some(expected)
                } else {
                    human_id(expected.gid)
                        && initial
                            .policy
                            .protected_groups
                            .iter()
                            .all(|n| initial.configuration.groups[n].gid != expected.gid)
                }
        }
        Requirement::UserAbsent { name: n, uid } => {
            initial
                .configuration
                .users
                .get(n)
                .is_some_and(|u| u.uid == *uid)
                && !initial.policy.protected_users.contains(n)
        }
        Requirement::GroupAbsent { name: n, gid } => {
            initial
                .configuration
                .groups
                .get(n)
                .is_some_and(|g| g.gid == *gid)
                && !initial.policy.protected_groups.contains(n)
        }
        Requirement::Entry { path: p, expected } => {
            path(p)
                && !["/etc/passwd", "/etc/group", "/etc/shadow"].contains(&p.as_str())
                && expected.mode <= 0o777
                && expected.bytes() <= MAX_FILE_BYTES
                && expected.uid <= 60000
                && expected.gid <= 60000
                && !matches!(&expected.data,Kind::Text{contents}if contents.contains('\0'))
                && if initial.policy.protected_entries.contains(p)
                    || initial.policy.skeletons.iter().any(|r| within(p, r))
                {
                    initial.configuration.entries.get(p) == Some(expected)
                } else {
                    true
                }
        }
        Requirement::AbsentPath { path: p } => {
            path(p)
                && p != "/"
                && !initial.policy.protected_entries.contains(p)
                && !initial.policy.skeletons.iter().any(|r| within(p, r))
        }
        Requirement::ExpiryRange {
            name: n,
            first,
            last,
        } => {
            initial.configuration.users.contains_key(n)
                && !initial.policy.protected_users.contains(n)
                && first <= last
        }
    })
}
pub fn requirements_from_json(initial: &State, text: &str) -> Result<Vec<Requirement>> {
    if text.len() > MAX_STATE_BYTES {
        return Err(err(ErrorKind::Limit, "Account outcome JSON input limit"));
    }
    #[derive(Deserialize)]
    #[serde(transparent)]
    struct Outcomes(#[serde(deserialize_with = "outcomes")] Vec<Requirement>);
    let rs: Outcomes = serde_json::from_str(text)
        .map_err(|e| err(ErrorKind::Invalid, format!("Invalid account outcomes: {e}")))?;
    if !valid_requirements(initial, &rs.0) {
        return Err(invalid());
    }
    Ok(rs.0)
}
pub fn checks(initial: &State, state: &State, requirements: &[Requirement]) -> Vec<(bool, String)> {
    if !valid_requirements(initial, requirements) || !state.valid_against(initial) {
        return vec![(
            false,
            "Account fixture, identity or outcome requirements do not match".into(),
        )];
    }
    let c = &state.configuration;
    requirements.iter().map(|r|match r{
        Requirement::User{name,expected}=>(c.users.get(name)==Some(expected),format!("{name}: requested account identity, membership and policy")),
        Requirement::Group{name,expected}=>(c.groups.get(name)==Some(expected),format!("{name}: requested group identity")),
        Requirement::UserAbsent{name,uid}=>(!c.users.contains_key(name)&&!c.users.values().any(|u|u.uid==*uid),format!("{name}: account removed without reassigning its UID")),
        Requirement::GroupAbsent{name,gid}=>(!c.groups.contains_key(name)&&!c.groups.values().any(|g|g.gid==*gid),format!("{name}: group removed without reassigning its GID")),
        Requirement::Entry{path,expected}=>(c.entries.get(path)==Some(expected),format!("{path}: retained text, kind, mode and numeric ownership")),
        Requirement::AbsentPath{path}=>(!c.entries.contains_key(path),format!("{path}: entry absent")),
        Requirement::Configuration{expected}=>(c==expected,"All requested account/group records and virtual entries, including unaffected data".into()),
        Requirement::ExpiryRange{name,first,last}=>(c.users.get(name).and_then(|u|u.expires).is_some_and(|d|d>=*first&&d<=*last),format!("{name}: requested account-expiry metadata range")),
    }).collect()
}
