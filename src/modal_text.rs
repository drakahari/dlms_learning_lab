//! Bounded in-memory text/undo foundation for the planned modal editor.
//! No key grammar, vi conformance, file access, lesson or grading is implied yet.
use serde::{Deserialize, Serialize};
use std::ops::Range;
pub const MAX_TEXT: usize = 16_384;
pub const MAX_UNDO: usize = 32;
pub const MAX_JSON: usize = 1_200_000;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    contents: String,
    cursor: usize,
    revision: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Buffer {
    version: u32,
    clock: u64,
    written_revision: u64,
    current: Snapshot,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    written: String,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Snapshot,32>")]
    undo: Vec<Snapshot>,
    group: Option<Snapshot>,
    group_changed: bool,
}
fn text_valid(text: &str) -> bool {
    text.len() <= MAX_TEXT && text.bytes().all(|b| b == b'\n' || (32..=126).contains(&b))
}
impl Snapshot {
    fn valid(&self) -> bool {
        text_valid(&self.contents) && self.cursor <= self.contents.len()
    }
}
impl Buffer {
    pub fn new(contents: &str) -> Result<Self, String> {
        if !text_valid(contents) {
            return Err("This editing foundation supports at most 16 KiB of printable ASCII and line breaks; tabs, other controls and non-ASCII text are not supported yet.".into());
        }
        Ok(Self {
            version: 1,
            clock: 0,
            written_revision: 0,
            current: Snapshot {
                contents: contents.into(),
                cursor: 0,
                revision: 0,
            },
            written: contents.into(),
            undo: vec![],
            group: None,
            group_changed: false,
        })
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.clock == u64::MAX
            || self.current.revision > self.clock
            || self.written_revision > self.clock
            || !self.current.valid()
            || !text_valid(&self.written)
            || self.undo.len() > MAX_UNDO
            || self
                .undo
                .iter()
                .any(|s| !s.valid() || s.revision > self.clock)
            || self
                .group
                .as_ref()
                .is_some_and(|s| !s.valid() || s.revision > self.clock)
            || self.group.is_none() && self.group_changed
        {
            return Err("Invalid bounded text buffer or saved edit history.".into());
        }
        Ok(())
    }
    pub fn from_json(raw: &str) -> Result<Self, String> {
        if raw.len() > MAX_JSON {
            return Err("Saved text buffer exceeds its size limit.".into());
        }
        let value: Self = serde_json::from_str(raw).map_err(|e| e.to_string())?;
        value.validate()?;
        Ok(value)
    }
    pub fn contents(&self) -> &str {
        &self.current.contents
    }
    pub fn written(&self) -> &str {
        &self.written
    }
    pub fn cursor(&self) -> usize {
        self.current.cursor
    }
    /// Data comparison only; not a claim about vi's 'modified' option/history.
    pub fn differs_from_written(&self) -> bool {
        self.contents() != self.written
    }
    /// Track edits separately from byte equality, including edits that cancel
    /// within a group. Undo can return to the exact last-written revision.
    pub fn modified_since_written(&self) -> bool {
        self.current.revision != self.written_revision || self.group_changed
    }
    pub fn group_active(&self) -> bool {
        self.group.is_some()
    }
    fn next_revision(&self) -> Result<u64, String> {
        self.clock
            .checked_add(1)
            .filter(|r| *r < u64::MAX)
            .ok_or_else(|| "The edit revision limit was reached; no edit was applied.".into())
    }
    pub fn seek(&mut self, byte: usize) -> Result<(), String> {
        self.validate()?;
        if byte > self.contents().len() {
            return Err("Cursor is outside this text buffer.".into());
        }
        self.current.cursor = byte;
        Ok(())
    }
    pub fn position(&self) -> Result<(usize, usize), String> {
        self.validate()?;
        let before = &self.current.contents[..self.current.cursor];
        Ok((
            before.bytes().filter(|b| *b == b'\n').count(),
            before.rsplit('\n').next().unwrap_or("").len(),
        ))
    }
    fn remember(&mut self, snapshot: Snapshot) {
        if self.undo.len() == MAX_UNDO {
            self.undo.remove(0);
        }
        self.undo.push(snapshot);
    }
    pub fn begin_group(&mut self) -> Result<(), String> {
        self.validate()?;
        if self.group.is_some() {
            return Err("Finish the current edit group first.".into());
        }
        self.group = Some(self.current.clone());
        self.group_changed = false;
        Ok(())
    }
    pub fn finish_group(&mut self) -> Result<(), String> {
        self.validate()?;
        if self.group.is_none() {
            return Err("No edit group is active.".into());
        }
        let next = if self.group_changed {
            Some(self.next_revision()?)
        } else {
            None
        };
        let snapshot = self.group.take().ok_or("No edit group is active.")?;
        self.group_changed = false;
        if let Some(revision) = next {
            self.clock = revision;
            self.current.revision = revision;
            self.remember(snapshot);
        }
        Ok(())
    }
    /// Pure text replacement. A later modal controller supplies command semantics.
    /// Invalid ranges/text/limits leave contents, cursor and history exact.
    pub fn replace(&mut self, range: Range<usize>, text: &str) -> Result<(), String> {
        self.validate()?;
        if range.start > range.end || range.end > self.contents().len() {
            return Err("The selected text range is outside this buffer.".into());
        }
        if !text_valid(text) {
            return Err("Only printable ASCII and line breaks are supported in this bounded editing foundation.".into());
        }
        let size = self.contents().len() - (range.end - range.start) + text.len();
        if size > MAX_TEXT {
            return Err("The edit would exceed the 16 KiB text limit; it was not applied.".into());
        }
        if self.contents()[range.clone()] == *text {
            return Ok(());
        }
        let before = self.current.clone();
        let next = if self.group.is_none() {
            Some(self.next_revision()?)
        } else {
            None
        };
        let cursor = range.start + text.len();
        self.current.contents.replace_range(range, text);
        self.current.cursor = cursor;
        if let Some(revision) = next {
            self.clock = revision;
            self.current.revision = revision;
            self.remember(before);
        } else {
            self.group_changed = true;
        }
        Ok(())
    }
    pub fn insert(&mut self, text: &str) -> Result<(), String> {
        self.replace(self.cursor()..self.cursor(), text)
    }
    /// Return to an earlier buffer, without undoing previously written file bytes.
    pub fn undo(&mut self) -> Result<(), String> {
        self.validate()?;
        if self.group.is_some() {
            return Err("Finish the current edit before undoing it.".into());
        }
        self.current = self.undo.pop().ok_or("No earlier edit is retained.")?;
        Ok(())
    }
    /// Called only after a future controller successfully writes virtual bytes.
    /// This method performs no filesystem operation and never invents a newline.
    pub fn mark_written(&mut self) -> Result<(), String> {
        self.validate()?;
        if self.group.is_some() {
            return Err("Finish the current edit before writing it.".into());
        }
        self.written.clone_from(&self.current.contents);
        self.written_revision = self.current.revision;
        Ok(())
    }
}
