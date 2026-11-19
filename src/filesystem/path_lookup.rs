//! Readonly supplied external-name PATH lookup. Never loads or runs a file.
use super::Tree;
use serde::{Deserialize, Deserializer, Serialize};
use std::{collections::BTreeSet, io::Write};
pub const VERSION: u32 = 1;
pub const MAX_NODES: usize = 64;
pub const MAX_NAMES: usize = 4;
pub const MAX_COMPONENTS: usize = 16;
pub const MAX_PATH: usize = 16_384;
pub const MAX_DIRECTORY: usize = 1024;
pub const MAX_PROFILE: usize = 128 * 1024;
pub const MAX_OBSERVATION: usize = 256 * 1024;
pub const MAX_STEPS: u32 = 4 * 16 * 65;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Invalid,
    Unsupported,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
type Result<T> = std::result::Result<T, Error>;
fn error(kind: ErrorKind, message: &str) -> Error {
    Error {
        kind,
        message: message.into(),
    }
}

struct Text<const N: usize>(String);
impl<'de, const N: usize> Deserialize<'de> for Text<N> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        crate::cron::text::<D, N>(d).map(Self)
    }
}
fn text_list<'de, D: Deserializer<'de>, const N: usize, const M: usize>(
    d: D,
) -> std::result::Result<Vec<String>, D::Error> {
    crate::cron_workspace::sequence::<D, Text<N>, M>(d)
        .map(|v| v.into_iter().map(|s| s.0).collect())
}
fn names<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    text_list::<D, 48, 4>(d)
}
fn directories<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<String>, D::Error> {
    text_list::<D, 1024, 16>(d)
}
fn traces<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<Candidate>, D::Error> {
    crate::cron_workspace::sequence::<D, Candidate, 16>(d)
}
fn reports<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<Report>, D::Error> {
    crate::cron_workspace::sequence::<D, Report, 4>(d)
}
fn directory(s: &str) -> bool {
    super::canonical(s) && !s.contains(':')
}
fn external_name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 48
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        && ![
            "printf", "echo", "cat", "grep", "test", "true", "false", "read", "export", "declare",
            "local", "set", "unset", "shift", "return", "exit", "eval", "exec", "source",
            "command", "type", "alias", "unalias", "hash", "cd", "pwd",
        ]
        .contains(&s)
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub version: u32,
    pub tree: Tree,
    #[serde(deserialize_with = "directories")]
    pub trusted_directories: Vec<String>,
    #[serde(deserialize_with = "names")]
    pub names: Vec<String>,
}
struct ByteCount(usize);
impl Write for ByteCount {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        if b.len() > MAX_PROFILE.saturating_sub(self.0) {
            return Err(std::io::Error::other("Lookup profile byte limit"));
        }
        self.0 += b.len();
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Profile {
    pub fn validate(&self) -> Result<()> {
        if self.tree.command_location.is_some() { return Err(error(ErrorKind::Invalid,"Readonly lookup profiles refuse operational command-location metadata")); }
        if self.version != VERSION
            || self.tree.nodes.len() > MAX_NODES
            || self.tree.files.len() > MAX_NODES
            || self.names.is_empty()
            || self.names.len() > MAX_NAMES
            || self.trusted_directories.len() > MAX_COMPONENTS
        {
            return Err(error(
                ErrorKind::Invalid,
                "Invalid lookup profile version or collection bound",
            ));
        }
        if !self.tree.valid() || self.tree.files.values().any(|s| !s.is_empty()) {
            return Err(error(
                ErrorKind::Invalid,
                "Lookup needs an ordinary-user readonly tree of empty artifact metadata",
            ));
        }
        let mut seen = BTreeSet::new();
        if self
            .names
            .iter()
            .any(|n| !external_name(n) || !seen.insert(n))
        {
            return Err(error(
                ErrorKind::Invalid,
                "Supply one to four distinct ASCII external basenames; reserved query names in this activity are excluded",
            ));
        }
        let mut seen = BTreeSet::new();
        if self.trusted_directories.iter().any(|d| {
            !directory(d)
                || !seen.insert(d)
                || !self.tree.nodes.get(d).is_some_and(|n| n.directory)
                || self.tree.allows(&self.tree.user, d, 2)
        }) {
            return Err(error(
                ErrorKind::Invalid,
                "Trusted directories must be distinct supplied canonical directories without write permission for this caller",
            ));
        }
        serde_json::to_writer(&mut ByteCount(0), self)
            .map_err(|_| error(ErrorKind::Limit, "Lookup profile exceeds128KiB"))?;
        Ok(())
    }
    pub fn decode(raw: &[u8]) -> Result<Self> {
        strict_document(raw, MAX_PROFILE)?;
        let p: Self = serde_json::from_slice(raw)
            .map_err(|_| error(ErrorKind::Invalid, "Invalid bounded lookup profile JSON"))?;
        p.validate()?;
        Ok(p)
    }
    /// The later workspace must compare to its held authored profile, not merely
    /// accept another structurally valid filesystem or trusted-directory roster.
    pub fn same_source(&self, expected: &Self) -> bool {
        self.validate().is_ok() && expected.validate().is_ok() && self == expected
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CandidateState {
    Missing,
    NotDirectory,
    NotRegularFile,
    Denied,
    NotExecutable,
    Selected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Candidate {
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub path: String,
    pub state: CandidateState,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Decision {
    Selected {
        #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
        path: String,
    },
    NotFound,
    Denied,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[serde(deserialize_with = "crate::cron::text::<_,48>")]
    pub name: String,
    pub decision: Decision,
    #[serde(deserialize_with = "traces")]
    pub candidates: Vec<Candidate>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Policy {
    Trusted,
    Untrusted {
        #[serde(deserialize_with = "directories")]
        directories: Vec<String>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    pub path: String,
    #[serde(deserialize_with = "directories")]
    pub components: Vec<String>,
    pub policy: Policy,
    #[serde(deserialize_with = "reports")]
    pub reports: Vec<Report>,
    /// Bounded candidate-entry evaluations, not time or real machine work.
    pub steps: u32,
}
impl Observation {
    /// This only proves a consistent pure lookup for this profile/PATH. The
    /// later adapter must also bind path to its ACTUAL inherited snapshot.
    pub fn matches(&self, profile: &Profile) -> bool {
        resolve(profile, Some(&self.path)).is_ok_and(|o| o == *self)
    }
    pub fn decode(raw: &[u8], profile: &Profile) -> Result<Self> {
        strict_document(raw, MAX_OBSERVATION)?;
        let o: Self = serde_json::from_slice(raw).map_err(|_| {
            error(
                ErrorKind::Invalid,
                "Invalid bounded lookup observation JSON",
            )
        })?;
        if !o.matches(profile) {
            return Err(error(
                ErrorKind::Invalid,
                "Lookup observation differs from its source profile and declared PATH",
            ));
        }
        Ok(o)
    }
}
fn charge(steps: &mut u32) -> Result<()> {
    if *steps >= MAX_STEPS {
        return Err(error(ErrorKind::Limit, "Lookup traversal budget exceeded"));
    }
    *steps += 1;
    Ok(())
}
fn examine(tree: &Tree, path: &str, steps: &mut u32) -> Result<CandidateState> {
    let mut at = "/".to_owned();
    // Every ancestor needs search, not read. Private allows uses the same
    // owner-first/group/other class rule as the public Tree access queries.
    for part in
        std::iter::once("").chain(path[1..].split('/').take(path[1..].split('/').count() - 1))
    {
        if !part.is_empty() {
            if at != "/" {
                at.push('/');
            }
            at.push_str(part);
        }
        charge(steps)?;
        let Some(node) = tree.nodes.get(&at) else {
            return Ok(CandidateState::Missing);
        };
        if !node.directory {
            return Ok(CandidateState::NotDirectory);
        }
        if !tree.allows(&tree.user, &at, 1) {
            return Ok(CandidateState::Denied);
        }
    }
    charge(steps)?;
    let Some(node) = tree.nodes.get(path) else {
        return Ok(CandidateState::Missing);
    };
    if node.directory {
        return Ok(CandidateState::NotRegularFile);
    }
    if !tree.allows(&tree.user, path, 1) {
        return Ok(CandidateState::NotExecutable);
    }
    Ok(CandidateState::Selected)
}
pub fn resolve(profile: &Profile, path: Option<&str>) -> Result<Observation> {
    profile.validate()?;
    let Some(path) = path else {
        return Err(error(
            ErrorKind::Unsupported,
            "No inherited PATH: this activity does not invent an unset-PATH default search",
        ));
    };
    if path.len() > MAX_PATH {
        return Err(error(ErrorKind::Limit, "PATH exceeds16KiB"));
    }
    let mut components = Vec::new();
    for component in path.split(':') {
        if components.len() == MAX_COMPONENTS {
            return Err(error(ErrorKind::Limit, "PATH exceeds16components"));
        }
        if component.is_empty() || !directory(component) {
            return Err(error(
                ErrorKind::Unsupported,
                "This activity supports only nonempty canonical absolute PATH directories; real shells also allow current-directory and relative entries",
            ));
        }
        for name in &profile.names {
            let candidate = if component == "/" {
                format!("/{name}")
            } else {
                format!("{component}/{name}")
            };
            if !super::canonical(&candidate) {
                return Err(error(
                    ErrorKind::Unsupported,
                    "A candidate exceeds the supported absolute path size or depth",
                ));
            }
        }
        components.push(component.to_owned());
    }
    let mut seen = BTreeSet::new();
    let mut unsafe_dirs = Vec::new();
    for dir in &components {
        if !profile.trusted_directories.contains(dir) && seen.insert(dir.clone()) {
            unsafe_dirs.push(dir.clone());
        }
    }
    let policy = if unsafe_dirs.is_empty() {
        Policy::Trusted
    } else {
        Policy::Untrusted {
            directories: unsafe_dirs,
        }
    };
    let mut reports = Vec::new();
    let mut steps = 0;
    for name in &profile.names {
        let mut candidates = Vec::new();
        let mut decision = Decision::NotFound;
        let mut denied = false;
        for dir in &components {
            let path = if dir == "/" {
                format!("/{name}")
            } else {
                format!("{dir}/{name}")
            };
            let state = examine(&profile.tree, &path, &mut steps)?;
            denied |= matches!(
                state,
                CandidateState::Denied
                    | CandidateState::NotExecutable
                    | CandidateState::NotRegularFile
            );
            candidates.push(Candidate {
                path: path.clone(),
                state,
            });
            if state == CandidateState::Selected {
                decision = Decision::Selected { path };
                break;
            }
        }
        if matches!(decision, Decision::NotFound) && denied {
            decision = Decision::Denied;
        }
        reports.push(Report {
            name: name.clone(),
            decision,
            candidates,
        });
    }
    Ok(Observation {
        path: path.into(),
        components,
        policy,
        reports,
        steps,
    })
}

// Small JSON preflight without retaining value trees, for NEW entry points only.
// Serde may allocate bounded scratch while decoding escaped strings.
// It rejects duplicate keys, deep/excess collections and long strings before
// typed construction, under the outer raw-byte cap. It is not a general parser.
fn strict_document(raw: &[u8], cap: usize) -> Result<()> {
    use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
    struct Seed<'a> {
        left: &'a mut usize,
        depth: u8,
    }
    impl<'de> DeserializeSeed<'de> for Seed<'_> {
        type Value = ();
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
            if *self.left == 0 || self.depth > 16 {
                return Err(serde::de::Error::custom("Lookup JSON work/depth bound"));
            }
            *self.left -= 1;
            d.deserialize_any(self)
        }
    }
    impl<'de> Visitor<'de> for Seed<'_> {
        type Value = ();
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("bounded unique lookup JSON")
        }
        fn visit_unit<E: serde::de::Error>(self) -> std::result::Result<(), E> {
            Ok(())
        }
        fn visit_bool<E: serde::de::Error>(self, _: bool) -> std::result::Result<(), E> {
            Ok(())
        }
        fn visit_u64<E: serde::de::Error>(self, _: u64) -> std::result::Result<(), E> {
            Ok(())
        }
        fn visit_i64<E: serde::de::Error>(self, _: i64) -> std::result::Result<(), E> {
            Ok(())
        }
        fn visit_f64<E: serde::de::Error>(self, _: f64) -> std::result::Result<(), E> {
            Ok(())
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> std::result::Result<(), E> {
            // The PATH value is allowed16KiB. Typed fields impose narrower caps.
            if s.len() > MAX_PATH {
                return Err(E::custom("Lookup JSON string bound"));
            }
            Ok(())
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> std::result::Result<(), A::Error> {
            let mut n = 0;
            while a
                .next_element_seed(Seed {
                    left: &mut *self.left,
                    depth: self.depth + 1,
                })?
                .is_some()
            {
                n += 1;
                if n > 64 {
                    return Err(serde::de::Error::custom("Lookup JSON array bound"));
                }
            }
            Ok(())
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> std::result::Result<(), A::Error> {
            let mut seen = BTreeSet::new();
            while let Some(Text(key)) = a.next_key::<Text<1024>>()? {
                if seen.len() >= 64 || !seen.insert(key) {
                    return Err(serde::de::Error::custom("Duplicate/excess lookup JSON key"));
                }
                a.next_value_seed(Seed {
                    left: &mut *self.left,
                    depth: self.depth + 1,
                })?;
            }
            Ok(())
        }
    }
    if raw.len() > cap {
        return Err(error(ErrorKind::Limit, "Lookup JSON raw byte bound"));
    }
    let mut left = 4096;
    let mut d = serde_json::Deserializer::from_slice(raw);
    Seed {
        left: &mut left,
        depth: 0,
    }
    .deserialize(&mut d)
    .map_err(|_| {
        error(
            ErrorKind::Invalid,
            "Invalid, duplicate or excessive lookup JSON",
        )
    })?;
    d.end()
        .map_err(|_| error(ErrorKind::Invalid, "Trailing lookup JSON"))
}
