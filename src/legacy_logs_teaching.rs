//! Independent fixed-text practice for original Logs, not the shell VM or Tree.
use crate::{
    engine::{self, Goal, State},
    legacy_access_teaching::limits,
    pack::CommandExample,
    progress::Attempt,
    script_teaching::{Concept, View},
    seed_dns_teaching::limits as prose,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
fn path(s: &str) -> bool {
    s.len() <= 256
        && s.starts_with("/var/log/")
        && !s.contains("..")
        && !s.ends_with('/')
        && !s.contains("//")
        && !s.chars().any(char::is_whitespace)
        && !s
            .bytes()
            .any(|c| c.is_ascii_whitespace() || c.is_ascii_control() || b"\\'\"".contains(&c))
}
fn files<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeMap<String, String>, D::Error> {
    use serde::de::{Error as _, MapAccess, Visitor};
    struct Files;
    impl<'de> Visitor<'de> for Files {
        type Value = BTreeMap<String, String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most 8 bounded fixed log files")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = BTreeMap::new();
            let mut total = 0;
            while let Some(key) = a.next_key::<limits::Text<256>>()? {
                if out.len() == 8 {
                    return Err(A::Error::custom("Practice file count limit"));
                }
                if out.contains_key(&key.0) {
                    return Err(A::Error::custom("Duplicate practice file"));
                }
                let value = a.next_value::<limits::Text<16384>>()?;
                if value.0.len() > 65536 - total {
                    return Err(A::Error::custom("Practice source-text aggregate limit"));
                }
                total += value.0.len();
                out.insert(key.0, value.0);
            }
            Ok(out)
        }
    }
    d.deserialize_map(Files)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    #[serde(deserialize_with = "files")]
    pub files: BTreeMap<String, String>,
    #[serde(deserialize_with = "limits::text::<_,32768>")]
    pub result: String,
}
impl Snapshot {
    /// Standalone practice import only; Progress retains its existing raw cap.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() > 1024 * 1024 {
            return Err("Practice snapshot exceeds 1 MiB".into());
        }
        let value: Self = serde_json::from_slice(bytes)
            .map_err(|_| "Malformed bounded practice snapshot".to_owned())?;
        if !value.valid() {
            return Err("Invalid bounded practice snapshot".into());
        }
        Ok(value)
    }
    pub fn valid(&self) -> bool {
        !self.files.is_empty()
            && self.files.len() <= 8
            && self.result.len() <= limits::OUTPUT
            && self
                .files
                .iter()
                .all(|(p, s)| path(p) && s.len() <= 16384 && !s.contains('\0'))
            && self.files.values().map(String::len).sum::<usize>() <= 65536
    }
    pub fn from_state(s: &State) -> Option<Self> {
        let State::Logs { files, result } = s else {
            return None;
        };
        if files.is_empty()
            || files.len() > 8
            || result.len() > limits::OUTPUT
            || !files
                .iter()
                .all(|(p, s)| path(p) && s.len() <= 16384 && !s.contains('\0'))
            || files.values().map(String::len).sum::<usize>() > 65536
        {
            return None;
        }
        Some(Self {
            files: files.clone(),
            result: result.clone(),
        })
    }
    pub fn state(&self) -> Result<State, String> {
        if !self.valid() {
            return Err("Invalid log practice snapshot".into());
        }
        Ok(State::Logs {
            files: self.files.clone(),
            result: self.result.clone(),
        })
    }
    pub fn same_problem(&self, fixture: &Self) -> bool {
        self.valid() && fixture.valid() && self.files == fixture.files
    }
    pub fn execute(&mut self, command: &str) -> Result<String, String> {
        if command.len() > limits::INPUT {
            return Err("Practice command exceeds 4096 bytes".into());
        }
        let mut candidate = self.state()?;
        let output = engine::execute(&mut candidate, command).map_err(|s| prose::message(&s))?;
        let next = Self::from_state(&candidate).ok_or("Log practice exceeds its bounded result")?;
        if !next.same_problem(self) || output.len() > limits::OUTPUT {
            return Err("Log practice source/output context changed".into());
        }
        *self = next;
        Ok(output)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "limits::text::<_,256>")]
    pub path: String,
    #[serde(deserialize_with = "limits::text::<_,64>")]
    pub needle: String,
}
impl Task {
    pub fn valid_for(&self, fixture: &Snapshot) -> bool {
        fixture.valid()
            && path(&self.path)
            && !self.needle.is_empty()
            && self.needle.len() <= 64
            && !self.needle.chars().any(char::is_whitespace)
            && fixture.files.get(&self.path).is_some_and(|s| {
                s.lines().filter(|line| line.contains(&self.needle)).count() >= 2
                    && s.lines().any(|line| !line.contains(&self.needle))
            })
    }
    pub fn goal(&self, fixture: &Snapshot) -> Result<Goal, String> {
        if !self.valid_for(fixture) {
            return Err("Invalid log practice task".into());
        }
        Ok(Goal::Logs {
            path: self.path.clone(),
            needle: self.needle.clone(),
        })
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "prose::text1600")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "prose::concepts")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "prose::text1600")]
    pub example_description: String,
    pub fixture: Snapshot,
    pub task: Task,
    #[serde(deserialize_with = "limits::commands")]
    pub commands: Vec<CommandExample>,
    #[serde(deserialize_with = "limits::sequence::<_,12>")]
    pub reference: Vec<String>,
    #[serde(deserialize_with = "prose::text1600")]
    pub try_change: String,
    #[serde(deserialize_with = "limits::sequence::<_,4>")]
    pub try_commands: Vec<String>,
    #[serde(deserialize_with = "limits::sequence::<_,6>")]
    pub repair_commands: Vec<String>,
}
impl Lesson {
    pub fn bounded(&self) -> bool {
        self.task.valid_for(&self.fixture)
            && limits::prose_valid(
                &self.prerequisite_note,
                &self.concepts,
                &self.example_description,
                &self.try_change,
            )
            && limits::commands_valid(&self.commands)
            && limits::sequence_valid(&self.reference, 12)
            && limits::sequence_valid(&self.try_commands, 4)
            && limits::sequence_valid(&self.repair_commands, 6)
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err("Invalid bounded log teaching".into());
        }
        let goal = self.task.goal(&self.fixture)?;
        let mut p = self.fixture.clone();
        if engine::solved(&p.state()?, &goal) {
            return Err("Log example starts solved".into());
        }
        for c in &self.commands {
            p.execute(&c.command)?;
        }
        p = self.fixture.clone();
        for c in &self.reference {
            p.execute(c)?;
        }
        if !engine::solved(&p.state()?, &goal) {
            return Err("Log reference must reach current text task".into());
        }
        for c in &self.try_commands {
            p.execute(c)?;
        }
        if engine::solved(&p.state()?, &goal) {
            return Err("Log experiment must break current result".into());
        }
        for c in &self.repair_commands {
            p.execute(c)?;
        }
        if !engine::solved(&p.state()?, &goal) {
            return Err("Log repair must restore current result".into());
        }
        Ok(())
    }
    pub fn valid_for(&self, initial: &State, goal: &Goal) -> bool {
        let (State::Logs { files, .. }, Goal::Logs { path, needle }) = (initial, goal) else {
            return false;
        };
        self.validate().is_ok()
            && files.contains_key(path)
            && self.task.needle != *needle
            && self.fixture.files.keys().all(|p| !files.contains_key(p))
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub current: Snapshot,
    pub selected_action: usize,
    #[serde(deserialize_with = "prose::text4096")]
    pub command: String,
    #[serde(deserialize_with = "limits::transcript")]
    pub transcript: Vec<String>,
    pub checked: bool,
}
impl Learning {
    pub fn new(l: &Lesson) -> Result<Self, String> {
        l.validate()?;
        Ok(Self {
            version: 1,
            view: View::Introduction,
            current: l.fixture.clone(),
            selected_action: 0,
            command: String::new(),
            transcript: vec![],
            checked: false,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.current.valid()
            && self.selected_action < 12
            && self.command.len() <= limits::INPUT
            && limits::valid_transcript(&self.transcript)
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        self.valid()
            && l.bounded()
            && self.current.same_problem(&l.fixture)
            && self.selected_action < l.commands.len()
    }
    pub fn insert(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Foreign log practice".into());
        }
        self.command = l.commands[self.selected_action].command.clone();
        Ok(())
    }
    pub fn submit(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Invalid or foreign log practice".into());
        }
        let command = std::mem::take(&mut self.command);
        if command.trim().is_empty() {
            return Ok(());
        }
        let before = self.current.clone();
        let output = self.current.execute(&command);
        if self.current != before {
            self.checked = false;
        }
        let text = match &output {
            Ok(s) => s.clone(),
            Err(s) => format!("Command failed: {s}"),
        };
        limits::record(
            &mut self.transcript,
            &format!("practice # {command}\n{text}"),
        );
        output.map(|_| ())
    }
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(l) {
            return vec![(false, "Invalid or foreign log practice".into())];
        }
        engine::checks(
            &self.current.state().expect("bounded practice"),
            &l.task.goal(&l.fixture).expect("bounded task"),
        )
        .into_iter()
        .map(|(pass, message)| {
            let message = if pass {
                message
            } else if self.current.result.is_empty() {
                "No text is selected by the current query. Check the path, literal text and letter case, then retry.".into()
            } else if self.current.result.lines().any(|line| !line.contains(&l.task.needle)) {
                "The result still includes lines outside the requested filter. Refine the literal text query and retry.".into()
            } else {
                "The selected result differs from the requested matching lines. Check for missing, repeated or reordered lines.".into()
            };
            (pass, message)
        })
        .collect()
    }
}
pub fn prepare(a: &mut Attempt, l: &Lesson, new_attempt: bool) -> Result<(), String> {
    if !matches!(a.state, State::Logs { .. }) {
        return Err("Log teaching belongs to the original log task".into());
    }
    let w = a.workspace.as_mut().ok_or("Prepare workspace first")?;
    if w.legacy_logs_learning
        .as_ref()
        .is_some_and(|p| !p.valid_for(l))
    {
        return Err("Foreign log example retained".into());
    }
    if w.legacy_logs_learning.is_none() {
        let mut p = Learning::new(l)?;
        if !new_attempt {
            p.view = View::Assignment;
        }
        w.legacy_logs_learning = Some(p);
    }
    Ok(())
}
