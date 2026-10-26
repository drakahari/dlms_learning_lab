//! Bounded per-mode display history and raw command draft; never grade authority.
use super::{DiagnosticKind, MAX_COMMAND_BYTES, MAX_OUTPUT_BYTES, execute};
use crate::{
    debian_packages::State,
    debian_packages_workspace::{self as work, Definition},
};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{DeserializeSeed, Error as _, SeqAccess, Visitor},
};
use std::{collections::BTreeSet, fmt};

pub const MAX_ENTRIES: usize = 32;
pub const MAX_RETAINED_TEXT: usize = 65_536;
pub const MAX_SESSION_JSON: usize = 524_288;
pub const MAX_INSPECTOR_TEXT: usize = 16_384;
const RESERVED_TEXT: usize = MAX_COMMAND_BYTES + 160;
fn string<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} UTF8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > N {
                return Err(E::custom("Package terminal text limit"));
            }
            Ok(s.to_owned())
        }
    }
    d.deserialize_str(Text::<N>)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "string::<_,MAX_COMMAND_BYTES>")]
    pub command: String,
    #[serde(deserialize_with = "string::<_,MAX_OUTPUT_BYTES>")]
    pub stdout: String,
    #[serde(deserialize_with = "string::<_,MAX_OUTPUT_BYTES>")]
    pub stderr: String,
    pub status: u8,
    pub diagnostic: Option<DiagnosticKind>,
}
impl Entry {
    fn bytes(&self) -> usize {
        self.command.len() + self.stdout.len() + self.stderr.len()
    }
    pub fn valid(&self) -> bool {
        self.command.len() <= MAX_COMMAND_BYTES
            && self.stdout.len() <= MAX_OUTPUT_BYTES
            && self.stderr.len() <= MAX_OUTPUT_BYTES
            && self.stdout.len() + self.stderr.len() <= MAX_OUTPUT_BYTES
            && match self.diagnostic {
                None => matches!(self.status, 0 | 1),
                Some(_) => {
                    matches!(self.status, 2 | 100)
                        && self.stdout.is_empty()
                        && !self.stderr.is_empty()
                        && self.stderr.len() <= 512
                }
            }
    }
}
fn entries<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Entry>, D::Error> {
    struct Entries;
    struct Next {
        allowed: bool,
    }
    impl<'de> DeserializeSeed<'de> for Next {
        type Value = Entry;
        fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Entry, D::Error> {
            if !self.allowed {
                return Err(D::Error::custom("Package transcript entry count limit"));
            }
            Entry::deserialize(d)
        }
    }
    impl<'de> Visitor<'de> for Entries {
        type Value = Vec<Entry>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("bounded package terminal entries")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Vec<Entry>, A::Error> {
            let mut out = Vec::new();
            let mut total = RESERVED_TEXT;
            while let Some(e) = a.next_element_seed(Next {
                allowed: out.len() < MAX_ENTRIES,
            })? {
                if !e.valid() {
                    return Err(A::Error::custom("Invalid package terminal entry"));
                }
                total = total
                    .checked_add(e.bytes())
                    .filter(|n| *n <= MAX_RETAINED_TEXT)
                    .ok_or_else(|| A::Error::custom("Package transcript aggregate limit"))?;
                out.push(e);
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Entries)
}
fn selected<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<work::Text<160>>::deserialize(d)?.map(|s| s.0))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Session {
    pub version: u32,
    pub command: String,
    pub entries: Vec<Entry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_path: Option<String>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            version: 1,
            command: String::new(),
            entries: Vec::new(),
            selected_path: None,
        }
    }
}
impl<'de> Deserialize<'de> for Session {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Raw {
            version: u32,
            #[serde(deserialize_with = "string::<_,MAX_COMMAND_BYTES>")]
            command: String,
            #[serde(deserialize_with = "entries")]
            entries: Vec<Entry>,
            #[serde(default, deserialize_with = "selected")]
            selected_path: Option<String>,
        }
        let r = Raw::deserialize(d)?;
        let s = Self {
            version: r.version,
            command: r.command,
            entries: r.entries,
            selected_path: r.selected_path,
        };
        if !s.valid() {
            return Err(D::Error::custom("Invalid bounded package terminal session"));
        }
        Ok(s)
    }
}
fn path(s: &str) -> bool {
    s.len() > 1
        && s.len() <= 160
        && s.starts_with('/')
        && !s.chars().any(char::is_control)
        && !s.contains('\\')
        && s[1..]
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
}
fn known(state: &State, path: &str) -> bool {
    state.catalog().extra_paths.contains(path)
        || state
            .catalog()
            .archives
            .values()
            .any(|a| a.files.contains_key(path))
}
impl Session {
    pub fn valid(&self) -> bool {
        if self.version != 1
            || self.command.len() > MAX_COMMAND_BYTES
            || self.entries.len() > MAX_ENTRIES
            || self.selected_path.as_ref().is_some_and(|p| !path(p))
            || !self.entries.iter().all(Entry::valid)
        {
            return false;
        }
        let total = self.entries.iter().try_fold(RESERVED_TEXT, |n, e| {
            n.checked_add(e.bytes()).filter(|n| *n <= MAX_RETAINED_TEXT)
        });
        total.is_some() && work::within(self, MAX_SESSION_JSON)
    }
    pub fn valid_for(&self, state: &State) -> bool {
        self.valid()
            && work::admitted(state)
            && self.selected_path.as_ref().is_none_or(|p| known(state, p))
    }
    pub fn decode(raw: &[u8], state: &State) -> Result<Self, String> {
        if raw.len() > MAX_SESSION_JSON || !work::admitted(state) {
            return Err("Package Session exceeds raw/context bounds".into());
        }
        let s: Self =
            serde_json::from_slice(raw).map_err(|_| "Invalid package Session JSON".to_owned())?;
        if !s.valid_for(state) {
            return Err("Foreign package inspector selection or transcript".into());
        }
        Ok(s)
    }
    /// A normal diagnostic is an observation and keeps the malformed command for
    /// retry. Admission failures publish neither state nor retained session.
    pub fn submit(&mut self, definition: &Definition, state: &mut State) -> Result<bool, String> {
        if !self.valid_for(state) || !definition.same_assignment(state) {
            return Err("Invalid package terminal context; previous work retained".into());
        }
        let mut candidate = state.clone();
        let (entry, clear, publish) = match execute(definition, &mut candidate, &self.command) {
            Ok(o) => (
                Entry {
                    command: self.command.clone(),
                    stdout: o.stdout,
                    stderr: o.stderr,
                    status: o.status,
                    diagnostic: None,
                },
                o.status == 0,
                true,
            ),
            Err(e) => (
                Entry {
                    command: self.command.clone(),
                    stdout: String::new(),
                    stderr: e.message,
                    status: e.status,
                    diagnostic: Some(e.kind),
                },
                false,
                false,
            ),
        };
        if !entry.valid() {
            return Err(
                "Package result exceeds retained terminal bounds; previous work retained".into(),
            );
        }
        let mut bytes =
            RESERVED_TEXT + self.entries.iter().map(Entry::bytes).sum::<usize>() + entry.bytes();
        let mut drop = 0;
        while self.entries.len() - drop >= MAX_ENTRIES || bytes > MAX_RETAINED_TEXT {
            let Some(old) = self.entries.get(drop) else {
                return Err(
                    "Package result exceeds transcript admission; previous work retained".into(),
                );
            };
            bytes -= old.bytes();
            drop += 1;
        }
        let mut next = Self {
            version: 1,
            command: if clear {
                String::new()
            } else {
                self.command.clone()
            },
            entries: self.entries[drop..].to_vec(),
            selected_path: self.selected_path.clone(),
        };
        next.entries.push(entry);
        if !next.valid_for(if publish { &candidate } else { state }) {
            return Err("Package result exceeds session admission; previous work retained".into());
        }
        let changed = publish && candidate != *state;
        if publish {
            *state = candidate;
        }
        *self = next;
        Ok(changed)
    }
}
/// Bounded actual known path identities, not expected Goal or reference payloads.
pub fn known_paths(state: &State) -> Result<Vec<String>, String> {
    if !work::admitted(state) {
        return Err("Package state exceeds inspector admission".into());
    }
    let all: BTreeSet<_> = state
        .catalog()
        .archives
        .values()
        .flat_map(|a| a.files.keys())
        .chain(state.catalog().extra_paths.iter())
        .collect();
    Ok(all.into_iter().cloned().collect())
}
/// Missing content and recorded ownership are distinct; no fictitious file or
/// archive output is generated. Large current files remain untouched.
pub fn current_file<'a>(state: &'a State, path: &str) -> Result<Option<&'a str>, String> {
    if !work::admitted(state) || !self::path(path) || !known(state, path) {
        return Err("Inspector needs one supplied virtual path".into());
    }
    let text = state.runtime().files.get(path).map(String::as_str);
    if text.is_some_and(|t| t.len() > MAX_INSPECTOR_TEXT) {
        return Err(
            "Current virtual file exceeds this inspector's16KiB display limit; file unchanged"
                .into(),
        );
    }
    Ok(text)
}
