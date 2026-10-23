//! Fixed-context account execution and separate ungraded terminal practice.
//! No host accounts, credentials, processes, authentication or clock APIs.
use crate::{
    accounts::{self, Output, Request, Requirement, State},
    pack::CommandExample,
    progress::Attempt,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};

pub const HELP: &str = "Virtual administrator only: id [USER]; id -Gn USER; getent passwd|group|shadow [KEY]; cat PATH; ls -ld PATH or ls -l|-ln DIRECTORY; groupadd -g GID GROUP; groupmod -n NEW GROUP; groupdel GROUP; explicit useradd -N -u UID -g GROUP [-G LIST] -d HOME -s SHELL -m|-M [-k SKEL] USER; usermod [-a] -G LIST USER; usermod -g GROUP USER; usermod -e DATE USER; chown USER:GROUP PATH; userdel [-r] USER. show/help are simulator helpers. Literal arguments only. Account expiry is metadata; no host accounts, passwords, login, PAM, sessions or process model.";
pub const MAX_COMMAND_BYTES: usize = 2048;
pub const MAX_TRANSCRIPT_ROWS: usize = 40;
pub const MAX_TRANSCRIPT_ROW_BYTES: usize = 36_000;
pub const MAX_TRANSCRIPT_BYTES: usize = 131_072;

/// Validate the authored context before any clone, and publish only a candidate
/// that remains compatible with the same immutable fixture.
pub fn execute_in_context(
    initial: &State,
    current: &mut State,
    input: &str,
) -> Result<Output, String> {
    if !current.valid_against(initial) {
        return Err("Account identities or protected context differ from the authored fixture; previous state retained".into());
    }
    if input.len() > MAX_COMMAND_BYTES {
        return Err("Account command exceeds2,048 bytes; previous state retained".into());
    }
    if input.trim() == "help" {
        return Ok(Output {
            stdout: HELP.into(),
            stderr: String::new(),
            status: 0,
        });
    }
    let request = accounts::parse(input).map_err(|e| e.to_string())?;
    if let Request::Query(q) = request {
        return current.query(&q).map_err(|e| e.to_string());
    }
    let mut candidate = current.clone();
    let output = candidate.execute(input).map_err(|e| e.to_string())?;
    if !candidate.valid_against(initial) {
        return Err("This command would reassign an authored account/group identity; previous state retained".into());
    }
    *current = candidate;
    Ok(output)
}
pub fn output_text(output: Output) -> Result<String, String> {
    let text = format!("{}{}", output.stdout, output.stderr);
    if output.status == 0 {
        Ok(text)
    } else {
        Err(format!(
            "{text}Simulator note: command exit status {}. A keyed getent query with no record returns2.",
            output.status
        ))
    }
}
pub(crate) fn requirements<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<Vec<Requirement>, D::Error> {
    use serde::de::{Error as _, SeqAccess, Visitor};
    struct Requirements;
    impl<'de> Visitor<'de> for Requirements {
        type Value = Vec<Requirement>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most64 bounded accounts outcomes")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while out.len() < accounts::MAX_REQUIREMENTS {
                match a.next_element()? {
                    None => return Ok(out),
                    Some(r) => out.push(r),
                }
            }
            if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(A::Error::custom("Accounts outcome count limit"));
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Requirements)
}
pub fn separate_states(assignment: &State, example: &State) -> bool {
    example.policy().fixture != assignment.policy().fixture
        && example.policy().today != assignment.policy().today
        && example
            .configuration()
            .users
            .iter()
            .filter(|(_, u)| u.uid != 0)
            .all(|(n, u)| {
                assignment
                    .configuration()
                    .users
                    .iter()
                    .all(|(other, v)| n != other && u.uid != v.uid)
            })
        && example
            .configuration()
            .groups
            .iter()
            .filter(|(_, g)| g.gid != 0)
            .all(|(n, g)| {
                assignment
                    .configuration()
                    .groups
                    .iter()
                    .all(|(other, v)| n != other && g.gid != v.gid)
            })
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example: Vec<String>,
    pub example_commands: Vec<CommandExample>,
    pub inputs: String,
    pub try_change: String,
    pub fixture: State,
    #[serde(deserialize_with = "requirements")]
    pub outcomes: Vec<Requirement>,
    pub try_actions: Vec<String>,
    #[serde(deserialize_with = "requirements")]
    pub try_outcomes: Vec<Requirement>,
    pub repair: Vec<String>,
}
fn commands_valid(commands: &[String]) -> bool {
    !commands.is_empty()
        && commands.len() <= 8
        && commands
            .iter()
            .all(|s| !s.trim().is_empty() && s.len() <= MAX_COMMAND_BYTES)
}
fn reached(initial: &State, state: &State, rs: &[Requirement]) -> bool {
    accounts::checks(initial, state, rs).iter().all(|c| c.0)
}
impl Lesson {
    pub fn validate(&self) -> Result<(), String> {
        if !self.fixture.valid()
            || self.concepts.is_empty()
            || self.concepts.len() > 6
            || self.concepts.iter().any(|c| {
                c.title.trim().is_empty()
                    || c.title.len() > 100
                    || c.text.trim().is_empty()
                    || c.text.len() > 1200
            })
            || [&self.prerequisite_note, &self.inputs, &self.try_change]
                .iter()
                .any(|s| s.trim().is_empty() || s.len() > 1600)
            || !commands_valid(&self.example)
            || !commands_valid(&self.try_actions)
            || !commands_valid(&self.repair)
            || self.example_commands.len() != self.example.len()
            || self
                .example_commands
                .iter()
                .zip(&self.example)
                .any(|(c, source)| {
                    c.command != *source
                        || c.explanation.trim().is_empty()
                        || c.explanation.len() > 800
                        || c.simulator_only != (source.trim() == "show" || source.trim() == "help")
                })
            || !accounts::valid_requirements(&self.fixture, &self.outcomes)
            || !accounts::valid_requirements(&self.fixture, &self.try_outcomes)
            || reached(&self.fixture, &self.fixture, &self.outcomes)
        {
            return Err("Invalid accounts teaching, annotated commands or example outcomes".into());
        }
        let mut state = self.fixture.clone();
        for command in &self.example {
            if execute_in_context(&self.fixture, &mut state, command)?.status != 0 {
                return Err("Account example must explain intentional nonzero results explicitly; default commands must succeed".into());
            }
        }
        if !reached(&self.fixture, &state, &self.outcomes) {
            return Err("Accounts example does not reach every declared outcome".into());
        }
        let before = state.clone();
        for command in &self.try_actions {
            if execute_in_context(&self.fixture, &mut state, command)?.status != 0 {
                return Err(
                    "Accounts try-change commands must have supported successful results".into(),
                );
            }
        }
        if state == before || !reached(&self.fixture, &state, &self.try_outcomes) {
            return Err(
                "Accounts try-change must alter state and reach its declared outcomes".into(),
            );
        }
        for command in &self.repair {
            if execute_in_context(&self.fixture, &mut state, command)?.status != 0 {
                return Err("Accounts example repair must succeed".into());
            }
        }
        if !reached(&self.fixture, &state, &self.outcomes) {
            return Err("Accounts example repair does not restore its outcomes".into());
        }
        Ok(())
    }
    pub fn separate_from(&self, assignment: &State) -> bool {
        separate_states(assignment, &self.fixture)
    }
}
fn bounded_text<'de, D: serde::Deserializer<'de>, const N: usize>(
    d: D,
) -> Result<String, D::Error> {
    struct Text<const N: usize>;
    impl<const N: usize> serde::de::Visitor<'_> for Text<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} UTF-8 bytes")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > N {
                Err(E::custom("Accounts practice text limit"))
            } else {
                Ok(s.into())
            }
        }
    }
    d.deserialize_str(Text::<N>)
}
fn command<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    bounded_text::<D, MAX_COMMAND_BYTES>(d)
}
struct Row(String);
impl<'de> Deserialize<'de> for Row {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        bounded_text::<D, MAX_TRANSCRIPT_ROW_BYTES>(d).map(Self)
    }
}
fn transcript<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    use serde::de::{Error as _, SeqAccess, Visitor};
    struct Rows;
    impl<'de> Visitor<'de> for Rows {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("bounded accounts practice transcript")
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            let mut bytes = 0;
            while out.len() < MAX_TRANSCRIPT_ROWS {
                match a.next_element::<Row>()? {
                    None => return Ok(out),
                    Some(row) => {
                        bytes += row.0.len();
                        if bytes > MAX_TRANSCRIPT_BYTES {
                            return Err(A::Error::custom("Accounts transcript aggregate limit"));
                        }
                        out.push(row.0);
                    }
                }
            }
            if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
                return Err(A::Error::custom("Accounts transcript row limit"));
            }
            Ok(out)
        }
    }
    d.deserialize_seq(Rows)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub simulation: State,
    pub example_command: usize,
    #[serde(deserialize_with = "command")]
    pub command: String,
    #[serde(deserialize_with = "transcript")]
    pub transcript: Vec<String>,
    pub checked: bool,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Self {
        Self {
            version: 1,
            view: View::Introduction,
            simulation: lesson.fixture.clone(),
            example_command: 0,
            command: String::new(),
            transcript: vec![],
            checked: false,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.simulation.valid()
            && self.example_command < 8
            && self.command.len() <= MAX_COMMAND_BYTES
            && self.transcript.len() <= MAX_TRANSCRIPT_ROWS
            && self
                .transcript
                .iter()
                .all(|s| s.len() <= MAX_TRANSCRIPT_ROW_BYTES)
            && self.transcript.iter().map(String::len).sum::<usize>() <= MAX_TRANSCRIPT_BYTES
    }
    pub fn valid_for(&self, lesson: &Lesson) -> bool {
        self.valid()
            && self.example_command < lesson.example.len()
            && self.simulation.valid_against(&lesson.fixture)
    }
    pub fn fill_example(&mut self, lesson: &Lesson) -> Result<(), String> {
        if !self.valid_for(lesson) {
            return Err("Accounts example context or selection differs from this lesson".into());
        }
        self.command = lesson.example[self.example_command].clone();
        Ok(())
    }
    pub fn execute(&mut self, lesson: &Lesson) -> Result<Output, String> {
        if !self.valid_for(lesson) {
            return Err("Invalid accounts practice context; previous state/input retained".into());
        }
        if self.command.trim().is_empty() {
            return Err("Enter one supported virtual account command".into());
        }
        let input = std::mem::take(&mut self.command);
        let result = execute_in_context(&lesson.fixture, &mut self.simulation, &input);
        let output = match &result {
            Ok(out) => format!(
                "{}{}{}",
                out.stdout,
                out.stderr,
                if out.status == 0 {
                    String::new()
                } else {
                    format!("Command exit status {}\n", out.status)
                }
            ),
            Err(e) => e.clone(),
        };
        self.transcript
            .push(format!("virtual admin # {input}\n{output}"));
        while self.transcript.len() > MAX_TRANSCRIPT_ROWS
            || self.transcript.iter().map(String::len).sum::<usize>() > MAX_TRANSCRIPT_BYTES
        {
            self.transcript.remove(0);
        }
        self.checked = false;
        result
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(lesson) {
            return vec![(
                false,
                "Accounts example identities or saved metadata differ from this lesson".into(),
            )];
        }
        accounts::checks(&lesson.fixture, &self.simulation, &lesson.outcomes)
    }
}
pub fn prepare(attempt: &mut Attempt, lesson: &Lesson, new_attempt: bool) {
    let w = attempt.workspace.as_mut().expect("workspace prepared");
    if w.accounts_learning.is_none() {
        let mut l = Learning::new(lesson);
        if !new_attempt {
            l.view = View::Assignment;
        }
        w.accounts_learning = Some(l);
    }
}
