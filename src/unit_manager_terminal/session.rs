//! Bounded saved command drafts/history. Display data never supplies objective credit.
use super::{DiagnosticKind, MAX_COMMAND_BYTES, MAX_OUTPUT_BYTES, Outcome, execute};
use crate::{unit_manager::State, unit_manager_workspace::Definition};
use serde::{
    Deserialize, Deserializer, Serialize,
    de::{Error as _, SeqAccess, Visitor},
};
use std::fmt;
pub const MAX_ENTRIES: usize = 16;
pub const MAX_SESSION_BYTES: usize = 131_072;
pub const MAX_ERROR_BYTES: usize = 1024;
fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "at most{N} UTF-8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() <= N {
                Ok(s.into())
            } else {
                Err(E::custom("Virtual service terminal text limit"))
            }
        }
    }
    d.deserialize_str(Text::<N>)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    #[serde(deserialize_with = "text::<_, MAX_COMMAND_BYTES>")]
    pub command: String,
    #[serde(deserialize_with = "text::<_, MAX_OUTPUT_BYTES>")]
    pub stdout: String,
    #[serde(deserialize_with = "text::<_, MAX_ERROR_BYTES>")]
    pub stderr: String,
    pub status: Option<u8>,
    pub diagnostic: Option<DiagnosticKind>,
    pub display_escaped: bool,
}
impl Entry {
    fn bytes(&self) -> usize {
        self.command.len() + self.stdout.len() + self.stderr.len()
    }
    pub fn valid(&self) -> bool {
        self.command.len() <= MAX_COMMAND_BYTES
            && self.stdout.len() <= MAX_OUTPUT_BYTES
            && self.stderr.len() <= MAX_ERROR_BYTES
            && self
                .stdout
                .chars()
                .chain(self.stderr.chars())
                .all(|c| !c.is_control() || matches!(c, '\n' | '\t'))
            && match (self.status, self.diagnostic) {
                (Some(0 | 1 | 3), None) => true,
                (None, Some(_)) => self.stdout.is_empty() && !self.stderr.is_empty(),
                _ => false,
            }
    }
}
fn entries<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<Entry>, D::Error> {
    struct Entries;
    impl<'de> Visitor<'de> for Entries {
        type Value = Vec<Entry>;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("at most16 entries and128KiB retained terminal text")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            let mut bytes = 0usize;
            while out.len() < MAX_ENTRIES {
                match a.next_element::<Entry>()? {
                    Some(e) => {
                        bytes = bytes
                            .checked_add(e.bytes())
                            .ok_or_else(|| A::Error::custom("Terminal text limit"))?;
                        if !e.valid() || bytes > MAX_SESSION_BYTES {
                            return Err(A::Error::custom("Invalid bounded terminal entry"));
                        }
                        out.push(e);
                    }
                    None => return Ok(out),
                }
            }
            if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(A::Error::custom("Terminal entry count limit"));
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Entries)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Session {
    pub version: u32,
    pub command: String,
    pub entries: Vec<Entry>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            version: 1,
            command: String::new(),
            entries: vec![],
        }
    }
}
impl<'de> Deserialize<'de> for Session {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Data {
            version: u32,
            #[serde(deserialize_with = "text::<_,MAX_COMMAND_BYTES>")]
            command: String,
            #[serde(deserialize_with = "entries")]
            entries: Vec<Entry>,
        }
        let v = Data::deserialize(d)?;
        let s = Self {
            version: v.version,
            command: v.command,
            entries: v.entries,
        };
        if !s.valid() {
            return Err(D::Error::custom("Invalid bounded virtual terminal session"));
        }
        Ok(s)
    }
}
impl Session {
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.command.len() <= MAX_COMMAND_BYTES
            && self.entries.len() <= MAX_ENTRIES
            && self.entries.iter().all(Entry::valid)
            && self
                .entries
                .iter()
                .map(Entry::bytes)
                .sum::<usize>()
                .checked_add(MAX_COMMAND_BYTES)
                .is_some_and(|n| n <= MAX_SESSION_BYTES)
    }
    /// Only a genuine bounded submitted command can append an observation. Invalid
    /// direct state/session inputs publish nothing. Partial syntax remains editable.
    pub fn submit(&mut self, definition: &Definition, state: &mut State) -> Result<bool, String> {
        if !self.valid() || !definition.same_assignment(state) {
            return Err("Invalid virtual service terminal context; work retained".into());
        }
        let mut candidate = state.clone();
        let result = execute(definition, &mut candidate, &self.command);
        let (entry, clear) = match result {
            Ok(Outcome {
                stdout,
                stderr,
                status,
                display_escaped,
                ..
            }) => (
                Entry {
                    command: self.command.clone(),
                    stdout,
                    stderr,
                    status: Some(status),
                    diagnostic: None,
                    display_escaped,
                },
                true,
            ),
            Err(e) => (
                Entry {
                    command: self.command.clone(),
                    stdout: String::new(),
                    stderr: e.message,
                    status: None,
                    diagnostic: Some(e.kind),
                    display_escaped: false,
                },
                false,
            ),
        };
        if !entry.valid() {
            return Err(
                "Virtual terminal result exceeds its supported display shape; work retained".into(),
            );
        }
        // Always reserve the full command draft budget, so later typing never
        // makes an otherwise valid saved transcript impossible to resume/save.
        let command_bytes = MAX_COMMAND_BYTES;
        let mut bytes =
            self.entries.iter().map(Entry::bytes).sum::<usize>() + entry.bytes() + command_bytes;
        let mut drop = 0;
        while self.entries.len() - drop >= MAX_ENTRIES || bytes > MAX_SESSION_BYTES {
            let Some(old) = self.entries.get(drop) else {
                return Err("Virtual terminal retention limit; work retained".into());
            };
            bytes -= old.bytes();
            drop += 1;
        }
        let changed = candidate != *state;
        self.entries.drain(..drop);
        self.entries.push(entry);
        if clear {
            self.command.clear();
        }
        *state = candidate;
        Ok(changed)
    }
}
