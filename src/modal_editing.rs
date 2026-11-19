//! Initial bounded modal controller. No filesystem, host editor or lesson bridge.
//! This first subset is intentionally small; unsupported keys/commands diagnose.
use crate::modal_text::{Buffer, MAX_TEXT};
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Normal,
    Insert,
    Command,
    SearchForward,
    SearchBackward,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum Key {
    Character(char),
    Escape,
    Enter,
    Backspace,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Pending {
    None,
    Delete,
    Yank,
    FirstLine,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
enum Goal {
    Column(usize),
    End,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Register {
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    text: String,
    linewise: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Search {
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    text: String,
    forward: bool,
}
fn plain_search(text: &str) -> bool {
    !text.is_empty()
        && text.len() <= 128
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b" _-=:,@%".contains(&b))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Editor {
    version: u32,
    buffer: Buffer,
    #[serde(deserialize_with = "crate::cron::text::<_,16384>")]
    file_bytes: String,
    mode: Mode,
    #[serde(deserialize_with = "crate::cron::text::<_,128>")]
    command: String,
    pending: Pending,
    goal: Option<Goal>,
    register: Option<Register>,
    search: Option<Search>,
    #[serde(deserialize_with = "crate::cron::text::<_,256>")]
    notice: String,
    closed: bool,
}
fn canonical(text: &str) -> Result<String, String> {
    Buffer::new(text)?;
    let mut text = text.to_owned();
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    if text.len() > MAX_TEXT {
        return Err("The logical file with its line ending exceeds16 KiB.".into());
    }
    Ok(text)
}
impl Editor {
    pub fn new(file_bytes: &str) -> Result<Self, String> {
        Ok(Self {
            version: 1,
            buffer: Buffer::new(&canonical(file_bytes)?)?,
            file_bytes: file_bytes.into(),
            mode: Mode::Normal,
            command: String::new(),
            pending: Pending::None,
            goal: None,
            register: None,
            search: None,
            notice: String::new(),
            closed: false,
        })
    }
    /// Authored recovery starter: keep saved virtual bytes and a distinct unsaved
    /// buffer. This is data initialization, not execution of an editor script.
    pub fn with_draft(file_bytes: &str, draft: &str) -> Result<Self, String> {
        let mut e = Self::new(file_bytes)?;
        let draft = canonical(draft)?;
        e.buffer.begin_group()?;
        e.buffer.replace(0..e.contents().len(), &draft)?;
        e.buffer.seek(0)?;
        e.buffer.finish_group()?;
        e.normalise()?;
        e.validate()?;
        Ok(e)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.buffer.validate()?;
        if self.version != 1
            || canonical(&self.file_bytes)? != self.buffer.written()
            || !self.buffer.contents().is_empty() && !self.buffer.contents().ends_with('\n')
            || self.command.len() > 128
            || !self.command.bytes().all(|b| (32..=126).contains(&b))
            || !matches!(
                self.mode,
                Mode::Command | Mode::SearchForward | Mode::SearchBackward
            ) && !self.command.is_empty()
            || self.mode != Mode::Normal && self.pending != Pending::None
            || self.buffer.group_active() != (self.mode == Mode::Insert)
            || self.closed && (self.mode != Mode::Normal || self.pending != Pending::None)
            || !self.buffer.contents().is_empty()
                && self.buffer.cursor() >= self.buffer.contents().len()
        {
            return Err("Invalid bounded modal editor state.".into());
        }
        if self.notice.len() > 256
            || !self.notice.bytes().all(|b| (32..=126).contains(&b))
            || matches!(self.goal,Some(Goal::Column(c)) if c>=MAX_TEXT)
            || self.search.as_ref().is_some_and(|s| !plain_search(&s.text))
            || self.register.as_ref().is_some_and(|r| {
                r.text.is_empty()
                    || Buffer::new(&r.text).is_err()
                    || (r.linewise && !r.text.ends_with('\n'))
                    || (!r.linewise && r.text.contains('\n'))
            })
        {
            return Err("Invalid modal search, movement goal or unnamed register.".into());
        }
        let (start, end) = self.line();
        if self.mode == Mode::Normal && end > start && self.buffer.cursor() >= end {
            return Err("Normal cursor must point to a character or an empty line.".into());
        }
        Ok(())
    }
    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn contents(&self) -> &str {
        self.buffer.contents()
    }
    pub fn file_bytes(&self) -> &str {
        &self.file_bytes
    }
    pub fn closed(&self) -> bool {
        self.closed
    }
    pub fn modified(&self) -> bool {
        self.buffer.modified_since_written()
    }
    pub fn cursor(&self) -> Result<(usize, usize), String> {
        self.validate()?;
        self.buffer.position()
    }
    pub fn command(&self) -> &str {
        &self.command
    }
    pub fn notice(&self) -> &str {
        &self.notice
    }
    pub fn pending_label(&self) -> &str {
        match self.pending {
            Pending::None => "",
            Pending::Delete => "d",
            Pending::Yank => "y",
            Pending::FirstLine => "g",
        }
    }
    fn first_nonblank(&mut self) -> Result<(), String> {
        let (start, end) = self.line();
        let at = self.contents()[start..end]
            .bytes()
            .position(|b| b != b' ')
            .map_or(start, |n| start + n);
        self.buffer.seek(at)?;
        self.goal = None;
        Ok(())
    }
    fn vertical(&mut self, down: bool) -> Result<(), String> {
        let (start, end) = self.line();
        let target = if down {
            if end + 1 >= self.contents().len() {
                return Ok(());
            }
            end + 1
        } else {
            if start == 0 {
                return Ok(());
            }
            self.contents()[..start - 1]
                .rfind('\n')
                .map_or(0, |n| n + 1)
        };
        let goal = self
            .goal
            .unwrap_or(Goal::Column(self.buffer.cursor() - start));
        let stop = self.contents()[target..]
            .find('\n')
            .map_or(self.contents().len(), |n| target + n);
        let at = match goal {
            Goal::Column(col) => target + col.min(stop.saturating_sub(target + 1)),
            Goal::End => stop.saturating_sub(1).max(target),
        };
        self.buffer.seek(at)?;
        self.goal = Some(goal);
        Ok(())
    }
    fn put(&mut self, after: bool) -> Result<(), String> {
        let r = self
            .register
            .clone()
            .ok_or("The unnamed register is empty; copy or delete some text first.")?;
        self.buffer.begin_group()?;
        if r.linewise {
            let (start, end) = self.line();
            if self.contents().is_empty() {
                self.buffer.insert("\n")?;
            }
            let at = if after {
                if end == 0 && start == 0 && self.contents() == "\n" {
                    1
                } else {
                    end + 1
                }
            } else {
                start
            };
            self.buffer.seek(at)?;
            self.buffer.insert(&r.text)?;
            self.buffer.seek(at)?;
            self.first_nonblank()?;
        } else {
            let (_, end) = self.line();
            let at =
                (self.buffer.cursor() + usize::from(after && self.buffer.cursor() < end)).min(end);
            self.buffer.seek(at)?;
            self.buffer.insert(&r.text)?;
            self.buffer.seek(at + r.text.len() - 1)?;
            self.goal = None;
        }
        self.buffer.finish_group()?;
        Ok(())
    }
    fn find(&mut self, text: &str, forward: bool) -> Result<(), String> {
        if !plain_search(text) {
            return Err("This subset searches plain letters, digits, spaces and _-=:,@% only. Regular expressions, delimiters and offsets are not supported.".into());
        }
        let mut matches = self
            .contents()
            .as_bytes()
            .windows(text.len())
            .enumerate()
            .filter_map(|(n, b)| (b == text.as_bytes()).then_some(n));
        let at = self.buffer.cursor();
        let found = if forward {
            matches.clone().find(|n| *n > at).or_else(|| matches.next())
        } else {
            matches
                .clone()
                .rfind(|n| *n < at)
                .or_else(|| matches.next_back())
        };
        self.search = Some(Search {
            text: text.into(),
            forward,
        });
        self.goal = None;
        if let Some(at) = found {
            self.buffer.seek(at)?;
        } else {
            self.notice =
                "Text not found. Check its spelling and capitalization; the file was not changed."
                    .into();
        }
        Ok(())
    }
    fn finish_search(&mut self) -> Result<(), String> {
        let text = if self.command.is_empty() {
            self.search
                .as_ref()
                .ok_or("Enter search text first; no earlier search is available.")?
                .text
                .clone()
        } else {
            self.command.clone()
        };
        self.find(&text, self.mode == Mode::SearchForward)?;
        self.command.clear();
        self.mode = Mode::Normal;
        Ok(())
    }
    fn line(&self) -> (usize, usize) {
        let text = self.buffer.contents();
        let at = self.buffer.cursor();
        let start = text[..at].rfind('\n').map_or(0, |n| n + 1);
        let end = text[at..].find('\n').map_or(text.len(), |n| at + n);
        (start, end)
    }
    fn normalise(&mut self) -> Result<(), String> {
        let text = self.buffer.contents();
        let at = self.buffer.cursor().min(text.len().saturating_sub(1));
        self.buffer.seek(at)?;
        let (start, end) = self.line();
        if end > start {
            self.buffer.seek(at.min(end - 1))?;
        }
        Ok(())
    }
    fn begin_insert(&mut self, after: bool) -> Result<(), String> {
        self.buffer.begin_group()?;
        if after && !self.contents().is_empty() {
            let (_, end) = self.line();
            self.buffer.seek((self.buffer.cursor() + 1).min(end))?;
        }
        self.mode = Mode::Insert;
        Ok(())
    }
    fn insert(&mut self, text: &str) -> Result<(), String> {
        if self.contents().is_empty() {
            self.buffer.insert("\n")?;
            self.buffer.seek(0)?;
        }
        self.buffer.insert(text)
    }
    fn finish_insert(&mut self) -> Result<(), String> {
        let (start, _) = self.line();
        let at = self.buffer.cursor();
        if at > start {
            self.buffer.seek(at - 1)?;
        }
        self.buffer.finish_group()?;
        self.mode = Mode::Normal;
        self.normalise()
    }
    fn write(&mut self) -> Result<(), String> {
        self.buffer.mark_written()?;
        self.file_bytes = self.buffer.contents().into();
        Ok(())
    }
    fn ex(&mut self) -> Result<(), String> {
        let command = self.command.trim().to_owned();
        match command.as_str(){
            "w"=>self.write()?,
            "wq"=>{self.write()?;self.closed=true;},
            "q"=>{if self.modified(){return Err("There are unsaved edits. Use :w to write them or :q! to discard them.".into());}self.closed=true;},
            "q!"=>{self.buffer=Buffer::new(&canonical(&self.file_bytes)?)?;self.closed=true;},
            _=>return Err("Unsupported editor command. This initial subset supports :w, :wq, :q and :q! only; no shell escapes, scripts or arbitrary files.".into()),
        }
        self.command.clear();
        self.mode = Mode::Normal;
        self.normalise()
    }
    fn apply(&mut self, key: Key) -> Result<(), String> {
        self.notice.clear();
        match (self.mode, key) {
            (Mode::Insert, Key::Escape) => self.finish_insert()?,
            (Mode::Insert, Key::Enter) => self.insert("\n")?,
            (Mode::Insert, Key::Backspace) => {
                let at = self.buffer.cursor();
                if at > 0 {
                    self.buffer.replace(at - 1..at, "")?;
                } else {
                    self.notice = "Start of file: there is no earlier character to erase.".into();
                }
            }
            (Mode::Insert, Key::Character(c)) => self.insert(&c.to_string())?,
            (Mode::Command | Mode::SearchForward | Mode::SearchBackward, Key::Escape) => {
                self.command.clear();
                self.mode = Mode::Normal;
            }
            (Mode::Command | Mode::SearchForward | Mode::SearchBackward, Key::Backspace) => {
                if self.command.pop().is_none() {
                    self.command.clear();
                    self.mode = Mode::Normal;
                }
            }
            (Mode::Command, Key::Enter) => self.ex()?,
            (Mode::SearchForward | Mode::SearchBackward, Key::Enter) => self.finish_search()?,
            (Mode::Command | Mode::SearchForward | Mode::SearchBackward, Key::Character(c)) => {
                if self.command.len() == 128 {
                    return Err("Editor entry exceeds128 characters; Escape cancels it.".into());
                }
                self.command.push(c);
            }
            (Mode::Normal, Key::Escape) => self.pending = Pending::None,
            (Mode::Normal, Key::Character(c)) => {
                if self.pending != Pending::None {
                    match (self.pending,c) {
                        (Pending::Delete,'d') | (Pending::Yank,'y')=>{
                            let(start,end)=self.line();let stop=(end+1).min(self.contents().len());
                            self.register=Some(Register{text:if self.contents().is_empty(){"\n".into()}else{self.contents()[start..stop].into()},linewise:true});
                            if self.pending==Pending::Delete && !self.contents().is_empty(){self.buffer.replace(start..stop,"")?;self.normalise()?;self.first_nonblank()?;}
                        },
                        (Pending::FirstLine,'g')=>{self.buffer.seek(0)?;self.first_nonblank()?;},
                        _=>return Err("Only dd, yy and gg are supported operator sequences; Escape cancels a pending key.".into()),
                    }
                    self.pending = Pending::None;
                } else {
                    match c {
                        'i'=>{self.goal=None;self.begin_insert(false)?;},
                        'a'=>{self.goal=None;self.begin_insert(true)?;},
                        'o'=>{
                            self.goal=None;self.buffer.begin_group()?;let(_,end)=self.line();
                            let empty=self.contents().is_empty();let at=if empty{0}else{end+1};
                            self.buffer.seek(at)?;self.buffer.insert(if empty{"\n\n"}else{"\n"})?;
                            self.buffer.seek(if empty{1}else{at})?;self.mode=Mode::Insert;
                        },
                        'h'=>{let(start,_)=self.line();let at=self.buffer.cursor();if at>start{self.buffer.seek(at-1)?;self.goal=None;}},
                        'l'=>{let(_,end)=self.line();let at=self.buffer.cursor();if at+1<end{self.buffer.seek(at+1)?;self.goal=None;}},
                        'j'=>self.vertical(true)?,
                        'k'=>self.vertical(false)?,
                        '0'=>{let(start,_)=self.line();self.buffer.seek(start)?;self.goal=None;},
                        '$'=>{let(start,end)=self.line();self.buffer.seek(end.saturating_sub(1).max(start))?;self.goal=Some(Goal::End);},
                        'G'=>{let end=self.contents().len().saturating_sub(1);self.buffer.seek(end)?;self.first_nonblank()?;},
                        'x'=>{
                            let at=self.buffer.cursor();let(_,end)=self.line();if at==end{return Err("There is no character to delete on this empty line.".into());}
                            self.register=Some(Register{text:self.contents()[at..at+1].into(),linewise:false});
                            self.buffer.replace(at..at+1,"")?;self.normalise()?;self.goal=None;
                        },
                        'd'=>self.pending=Pending::Delete,
                        'y'=>self.pending=Pending::Yank,
                        'g'=>self.pending=Pending::FirstLine,
                        'p'=>self.put(true)?,
                        'P'=>self.put(false)?,
                        'u'=>{self.buffer.undo()?;self.normalise()?;self.goal=None;},
                        ':'=>{self.mode=Mode::Command;self.command.clear();},
                        '/'=>{self.mode=Mode::SearchForward;self.command.clear();},
                        '?'=>{self.mode=Mode::SearchBackward;self.command.clear();},
                        'n'=>{let last=self.search.clone().ok_or("Enter a / or ? search before repeating it with n.")?;self.find(&last.text,last.forward)?;},
                        _=>return Err("Unsupported Normal key. Available: h j k l 0 $ gg G i a o x dd yy p P u / ? n and :commands; counts, other operators and regular expressions are not supported.".into()),
                    }
                }
            }
            (Mode::Normal, Key::Backspace) => {
                return Err("Normal-mode Backspace is outside this subset; use h to move left or i to insert text.".into());
            }
            (Mode::Normal, Key::Enter) => {
                return Err("Normal-mode Enter is not supported; use j to move down.".into());
            }
        }
        Ok(())
    }
    pub fn key(&mut self, key: Key) -> Result<(), String> {
        self.validate()?;
        if self.closed {
            return Err("The virtual editor is closed; reopen its file first.".into());
        }
        if matches!(key,Key::Character(c) if !((' '..='~').contains(&c))) {
            return Err(
                "Only printable ASCII is supported here; use the explicit Enter or Escape key."
                    .into(),
            );
        }
        let mut next = self.clone();
        next.apply(key)?;
        next.validate()?;
        *self = next;
        Ok(())
    }
    /// Native paste/text input while inserting. Contents are data, never keys,
    /// colon commands, a script or a host process. One Insert group remains open.
    pub fn insert_text(&mut self, text: &str) -> Result<(), String> {
        self.validate()?;
        if self.closed || self.mode != Mode::Insert {
            return Err("Enter Insert mode with i, a or o before pasting text.".into());
        }
        Buffer::new(text)?;
        if text.is_empty() {
            return Ok(());
        }
        let mut next = self.clone();
        next.notice.clear();
        next.insert(text)?;
        next.validate()?;
        *self = next;
        Ok(())
    }
    /// Bounded test/authoring replay, not Vimscript. Each accepted key commits;
    /// a diagnostic retains earlier accepted keys, like interactive editing.
    pub fn replay(&mut self, keys: &str) -> Result<(), String> {
        if keys.len() > 2048 {
            return Err("Fixed key replay exceeds its limit.".into());
        }
        for c in keys.chars() {
            self.key(match c {
                '\x1b' => Key::Escape,
                '\r' => Key::Enter,
                '\x08' => Key::Backspace,
                _ => Key::Character(c),
            })?;
        }
        Ok(())
    }
}

/// Selected linked model inputs for a development-only fixed comparison probe.
/// This is not the full compiler closure or a cryptographic certification claim.
pub const MODEL_INPUTS: &[(&str, &[u8])] = &[
    ("src/lib.rs", include_bytes!("lib.rs")),
    ("src/modal_editing.rs", include_bytes!("modal_editing.rs")),
    ("src/modal_text.rs", include_bytes!("modal_text.rs")),
    ("src/cron.rs", include_bytes!("cron.rs")),
    ("src/cron_workspace.rs", include_bytes!("cron_workspace.rs")),
    ("Cargo.toml", include_bytes!("../Cargo.toml")),
    ("Cargo.lock", include_bytes!("../Cargo.lock")),
];
