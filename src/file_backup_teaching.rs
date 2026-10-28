//! Separate, resumable backup practice. No assignment mutation or credit path.
use crate::{file_backup_terminal::Store, file_backup_workspace as work, script_teaching::View};
use serde::{Deserialize, Serialize};
pub const MAX_LESSON: usize = 2 * 1024 * 1024;
pub const MAX_LEARNING: usize = 3 * 1024 * 1024;
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Concept {
    #[serde(deserialize_with = "work::text::<_,120>")]
    pub title: String,
    #[serde(deserialize_with = "work::text::<_,1600>")]
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    #[serde(deserialize_with = "work::text::<_,4096>")]
    pub command: String,
    #[serde(deserialize_with = "work::text::<_,800>")]
    pub explanation: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    #[serde(deserialize_with = "work::text::<_,1600>")]
    pub description: String,
    #[serde(deserialize_with = "work::list::<_,Command,16>")]
    pub commands: Vec<Command>,
    #[serde(deserialize_with = "work::list::<_,bool,54>")]
    pub expected: Vec<bool>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub definition: work::Definition,
    #[serde(deserialize_with = "work::text::<_,1600>")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "work::list::<_,Concept,6>")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "work::text::<_,1600>")]
    pub example_description: String,
    #[serde(deserialize_with = "work::list::<_,Command,16>")]
    pub example_commands: Vec<Command>,
    pub try_change: Experiment,
    #[serde(deserialize_with = "work::list::<_,Command,16>")]
    pub repair: Vec<Command>,
}
fn fits<T: Serialize>(value: &T, cap: usize) -> bool {
    struct Counter {
        bytes: usize,
        cap: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.bytes = self
                .bytes
                .checked_add(b.len())
                .filter(|n| *n <= self.cap)
                .ok_or_else(|| std::io::Error::other("Backup teaching byte bound"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter { bytes: 0, cap }, value).is_ok()
}
fn prose(s: &str, cap: usize) -> bool {
    !s.trim().is_empty() && s.len() <= cap && !s.chars().any(char::is_control)
}
fn valid_commands(commands: &[Command]) -> bool {
    !commands.is_empty()
        && commands.len() <= 16
        && commands
            .iter()
            .all(|c| prose(&c.command, 4096) && prose(&c.explanation, 800))
}
fn overlap(a: &str, b: &str) -> bool {
    a == b
        || a.strip_prefix(b).is_some_and(|t| t.starts_with('/'))
        || b.strip_prefix(a).is_some_and(|t| t.starts_with('/'))
}
impl Lesson {
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > MAX_LESSON {
            return Err("Backup lesson exceeds2MiB".into());
        }
        let lesson: Self =
            serde_json::from_slice(bytes).map_err(|e| format!("Invalid backup lesson: {e}"))?;
        lesson.validate()?;
        Ok(lesson)
    }
    pub fn bounded(&self) -> bool {
        self.definition.validate().is_ok()
            && prose(&self.prerequisite_note, 1600)
            && prose(&self.example_description, 1600)
            && !self.concepts.is_empty()
            && self.concepts.len() <= 6
            && self
                .concepts
                .iter()
                .all(|c| prose(&c.title, 120) && prose(&c.text, 1600))
            && valid_commands(&self.example_commands)
            && prose(&self.try_change.description, 1600)
            && valid_commands(&self.try_change.commands)
            && valid_commands(&self.repair)
            && self.try_change.expected.len() <= 54
            && fits(self, MAX_LESSON)
    }
    pub fn separate_from(&self, assignment: &Store) -> bool {
        let example = &self.definition.initial;
        self.bounded()
            && assignment.valid()
            && example
                .trees
                .keys()
                .chain(std::iter::once(&example.storage))
                .all(|a| {
                    assignment
                        .trees
                        .keys()
                        .chain(std::iter::once(&assignment.storage))
                        .all(|b| !overlap(a, b))
                })
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err("Invalid bounded backup teaching".into());
        }
        let initial = self.definition.checks(&self.definition.initial);
        if initial.iter().all(|r| r.0) || initial.len() != self.try_change.expected.len() {
            return Err(
                "Backup practice needs a substantive initial task and exact experiment outcomes"
                    .into(),
            );
        }
        let mut state = self.definition.initial.clone();
        for c in &self.example_commands {
            state
                .execute(&c.command)
                .map_err(|e| format!("Backup example cannot run: {e}"))?;
        }
        if self.definition.checks(&state).iter().any(|r| !r.0) {
            return Err("Backup example does not reach its declared goal".into());
        }
        let worked = state.clone();
        for c in &self.try_change.commands {
            state
                .execute(&c.command)
                .map_err(|e| format!("Backup experiment cannot run: {e}"))?;
        }
        let actual = self
            .definition
            .checks(&state)
            .into_iter()
            .map(|r| r.0)
            .collect::<Vec<_>>();
        if state == worked || actual != self.try_change.expected || actual.iter().all(|p| *p) {
            return Err("Backup experiment needs a real explained adverse change".into());
        }
        for c in &self.repair {
            state
                .execute(&c.command)
                .map_err(|e| format!("Backup repair cannot run: {e}"))?;
        }
        if self.definition.checks(&state).iter().any(|r| !r.0) {
            return Err("Backup practice repair does not restore its goal".into());
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub context: work::Definition,
    pub version: u32,
    pub view: View,
    pub state: Store,
    #[serde(deserialize_with = "draft")]
    pub command: String,
    #[serde(deserialize_with = "transcript")]
    pub transcript: Vec<String>,
    pub example_command: usize,
    pub checked: bool,
}
fn draft<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    bounded_string::<D, 4096>(d)
}
fn bounded_string<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> serde::de::Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "bounded UTF-8 text <={N}bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > N || s.contains('\0') {
                return Err(E::custom("Backup draft/transcript bound"));
            }
            Ok(s.into())
        }
    }
    d.deserialize_str(Text::<N>)
}
struct TranscriptEntry(String);
impl<'de> Deserialize<'de> for TranscriptEntry {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        bounded_string::<D, 40000>(d).map(Self)
    }
}
fn transcript<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    use serde::de::Error as _;
    let rows = work::list::<D, TranscriptEntry, 16>(d)?
        .into_iter()
        .map(|v| v.0)
        .collect::<Vec<_>>();
    if rows.iter().map(String::len).sum::<usize>() > 65536 {
        return Err(D::Error::custom("Backup transcript total bound"));
    }
    Ok(rows)
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Result<Self, String> {
        lesson.validate()?;
        #[derive(Serialize)]
        struct New<'a> {
            context: &'a work::Definition,
            version: u32,
            view: View,
            state: &'a Store,
            command: &'static str,
            transcript: Vec<String>,
            example_command: usize,
            checked: bool,
        }
        let proposed = New {
            context: &lesson.definition,
            version: 1,
            view: View::Introduction,
            state: &lesson.definition.initial,
            command: "",
            transcript: Vec::new(),
            example_command: 0,
            checked: false,
        };
        if !fits(&proposed, MAX_LEARNING) {
            return Err("Backup example exceeds its saved size limit".into());
        }
        Ok(Self {
            context: lesson.definition.clone(),
            version: 1,
            view: View::Introduction,
            state: lesson.definition.initial.clone(),
            command: String::new(),
            transcript: Vec::new(),
            example_command: 0,
            checked: false,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.context.validate().is_ok()
            && self.context.bound_state(&self.state)
            && self.state.valid()
            && self.command.len() <= 4096
            && !self.command.contains('\0')
            && self.example_command < 16
            && self.transcript.len() <= 16
            && self
                .transcript
                .iter()
                .all(|s| s.len() <= 40000 && !s.contains('\0'))
            && self.transcript.iter().map(String::len).sum::<usize>() <= 65536
            && fits(self, MAX_LEARNING)
    }
    pub fn valid_for(&self, lesson: &Lesson) -> bool {
        self.valid()
            && self.context == lesson.definition
            && lesson.bounded()
            && self.example_command < lesson.example_commands.len()
            && lesson.definition.bound_state(&self.state)
    }
    pub fn execute(&mut self, lesson: &Lesson) -> Result<String, String> {
        if !self.valid_for(lesson) {
            return Err(
                "Backup example differs from its supplied workspace; saved work is retained".into(),
            );
        }
        if self.command.trim().is_empty() {
            return Err("Type one command in the example terminal first".into());
        }
        let result = self.state.execute(&self.command);
        self.checked = false;
        let output = result.as_ref().map_or_else(|e| e.as_str(), |s| s.as_str());
        self.transcript
            .push(format!("simulated root $ {}\n{output}", self.command));
        while self.transcript.len() > 16
            || self.transcript.iter().map(String::len).sum::<usize>() > 65536
        {
            self.transcript.remove(0);
        }
        result
    }
}

/// Create only missing separate practice; never replace an existing learner draft.
pub fn prepare(
    attempt: &mut crate::progress::Attempt,
    lesson: &Lesson,
    new_attempt: bool,
) -> Result<(), String> {
    crate::progress::validate_file_backup_workspace(attempt)?;
    let crate::engine::State::FileBackup { simulation } = &attempt.state else {
        return Err("Backup example needs a backup assignment".into());
    };
    if !lesson.separate_from(simulation) {
        return Err("Backup example paths overlap the assignment".into());
    }
    if let Some(learning) = attempt
        .workspace
        .as_ref()
        .and_then(|w| w.file_backup_learning.as_ref())
    {
        return if learning.valid_for(lesson) {
            Ok(())
        } else {
            Err("Saved backup example differs; previous work retained".into())
        };
    }
    let mut learning = Learning::new(lesson)?;
    if !new_attempt {
        learning.view = View::Assignment;
    }
    // Conservative borrowed admission BEFORE any caller field is changed. This
    // counts even the complete default workspace overhead when one already exists.
    let admitted = work::size(attempt, work::MAX_ATTEMPT)
        .and_then(|a| {
            work::size(&learning, work::MAX_ATTEMPT).and_then(|b| {
                work::size(&crate::workspace::WorkspaceData::new(), work::MAX_ATTEMPT)
                    .and_then(|c| a.checked_add(b)?.checked_add(c)?.checked_add(64))
            })
        })
        .is_some_and(|n| n <= work::MAX_ATTEMPT);
    if !admitted {
        return Err(
            "Backup example exceeds the saved attempt size limit; previous work retained".into(),
        );
    }
    attempt
        .workspace
        .get_or_insert_with(crate::workspace::WorkspaceData::new)
        .file_backup_learning = Some(learning);
    Ok(())
}
