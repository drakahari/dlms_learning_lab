//! Teaching and native drafts for the original single-client Network model only.
//! No routing engine, host networking or command interpreter is introduced.
use crate::{
    engine::{self, Goal, State},
    pack::CommandExample,
    progress::Attempt,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
const MAX_TRANSCRIPT: usize = 32;
const MAX_TRANSCRIPT_BYTES: usize = 131_072;
fn text<'de, D: serde::Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
    struct V<const N: usize>;
    impl<'de, const N: usize> serde::de::Visitor<'de> for V<N> {
        type Value = String;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "at most {N} bytes of text")
        }
        fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
            if s.len() > N {
                Err(E::custom("Client text exceeds its byte limit"))
            } else {
                Ok(s.into())
            }
        }
        fn visit_string<E: serde::de::Error>(self, s: String) -> Result<String, E> {
            if s.len() > N {
                Err(E::custom("Client text exceeds its byte limit"))
            } else {
                Ok(s)
            }
        }
    }
    d.deserialize_string(V::<N>)
}
fn text64<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    text::<D, 64>(d)
}
fn text16<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    text::<D, 16>(d)
}
fn text4096<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    text::<D, 4096>(d)
}
#[derive(Deserialize)]
struct Line(#[serde(deserialize_with = "text8192")] String);
fn text8192<'de, D: serde::Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    text::<D, 8192>(d)
}
fn transcript<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
    Ok(
        crate::progress::bounded_saved_sequence::<D, Line, MAX_TRANSCRIPT>(d)?
            .into_iter()
            .map(|x| x.0)
            .collect(),
    )
}
fn concepts<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Concept>, D::Error> {
    crate::progress::bounded_saved_sequence::<D, Concept, 6>(d)
}
fn actions<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<ExampleAction>, D::Error> {
    crate::progress::bounded_saved_sequence::<D, ExampleAction, 12>(d)
}
/// Scalar-only copy of the existing Network state. It cannot deserialize a foreign State kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Client {
    pub address: Ipv4Addr,
    pub prefix: u8,
    pub gateway: Ipv4Addr,
    pub router: Ipv4Addr,
    pub lan_prefix: u8,
    pub target: Ipv4Addr,
}
impl Client {
    pub fn from_state(state: &State) -> Option<Self> {
        if let State::Network {
            address,
            prefix,
            gateway,
            router,
            lan_prefix,
            target,
        } = state
        {
            Some(Self {
                address: *address,
                prefix: *prefix,
                gateway: *gateway,
                router: *router,
                lan_prefix: *lan_prefix,
                target: *target,
            })
        } else {
            None
        }
    }
    pub fn state(self) -> State {
        State::Network {
            address: self.address,
            prefix: self.prefix,
            gateway: self.gateway,
            router: self.router,
            lan_prefix: self.lan_prefix,
            target: self.target,
        }
    }
    pub fn valid(&self) -> bool {
        self.prefix <= 30 && self.lan_prefix <= 30
    }
    pub fn same_problem(&self, fixture: &Self) -> bool {
        self.valid()
            && fixture.valid()
            && self.router == fixture.router
            && self.lan_prefix == fixture.lan_prefix
            && self.target == fixture.target
    }
    pub fn execute(&mut self, input: &str) -> Result<String, String> {
        if !self.valid() || input.len() > 4096 {
            return Err("Invalid client state or command exceeds 4,096 bytes".into());
        }
        let mut candidate = self.state();
        let result = engine::execute(&mut candidate, input);
        if result.is_ok() {
            let next =
                Self::from_state(&candidate).ok_or("Client command changed the activity kind")?;
            if !next.same_problem(self) {
                return Err("Keep the supplied router, LAN plan and remote server".into());
            }
            *self = next;
        }
        result
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "text64")]
    pub address: String,
    #[serde(deserialize_with = "text16")]
    pub prefix: String,
    #[serde(deserialize_with = "text64")]
    pub gateway: String,
}
impl Draft {
    pub fn new(c: &Client) -> Self {
        Self {
            address: c.address.to_string(),
            prefix: c.prefix.to_string(),
            gateway: c.gateway.to_string(),
        }
    }
    pub fn valid(&self) -> bool {
        self.address.len() <= 64 && self.prefix.len() <= 16 && self.gateway.len() <= 64
    }
    pub fn apply(&self, current: &Client) -> Result<Client, String> {
        if !self.valid() || !current.valid() {
            return Err("Invalid or oversized client draft".into());
        }
        let address = self
            .address
            .trim()
            .parse()
            .map_err(|_| "Client address: enter four IPv4 numbers separated by dots")?;
        let prefix = self
            .prefix
            .trim()
            .parse::<u8>()
            .map_err(|_| "Prefix: enter a whole number from 0 to 30")?;
        if prefix > 30 {
            return Err("Prefix: enter a whole number from 0 to 30".into());
        }
        let gateway = self
            .gateway
            .trim()
            .parse()
            .map_err(|_| "Default gateway: enter a full IPv4 address")?;
        Ok(Client {
            address,
            prefix,
            gateway,
            ..*current
        })
    }
    pub fn matches(&self, current: &Client) -> bool {
        self.apply(current).is_ok_and(|c| c == *current)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Apply { draft: Draft },
    Command { command: CommandExample },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleAction {
    pub title: String,
    pub explanation: String,
    pub action: Action,
}
impl ExampleAction {
    fn valid(&self, fixture: &Client) -> bool {
        !self.title.trim().is_empty()
            && self.title.len() <= 100
            && !self.explanation.trim().is_empty()
            && self.explanation.len() <= 1200
            && match &self.action {
                Action::Apply { draft } => draft.valid() && draft.apply(fixture).is_ok(),
                Action::Command { command } => {
                    !command.simulator_only
                        && !command.command.is_empty()
                        && command.command.len() <= 256
                        && !command.explanation.trim().is_empty()
                        && command.explanation.len() <= 600
                        && inspection(&command.command, fixture)
                }
            }
    }
    fn perform(&self, c: &mut Client) -> Result<(), String> {
        match &self.action {
            Action::Apply { draft } => {
                *c = draft.apply(c)?;
                Ok(())
            }
            Action::Command { command } => c.execute(&command.command).map(|_| ()),
        }
    }
}
fn inspection(command: &str, c: &Client) -> bool {
    match command
        .split_ascii_whitespace()
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["ip", "addr"] | ["ip", "route"] => true,
        ["ping", ip] => ip
            .parse::<Ipv4Addr>()
            .is_ok_and(|ip| ip == c.router || ip == c.target),
        _ => false,
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    #[serde(deserialize_with = "concepts")]
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub fixture: Client,
    #[serde(deserialize_with = "actions")]
    pub example_actions: Vec<ExampleAction>,
    pub try_change: String,
    pub try_settings: Draft,
    pub repair_settings: Draft,
}
impl Lesson {
    pub fn goal(&self) -> Goal {
        Goal::Network {
            address: self.fixture.address,
        }
    }
    pub fn bounded(&self) -> bool {
        self.fixture.valid()
            && !self.concepts.is_empty()
            && self.concepts.len() <= 6
            && !self.example_actions.is_empty()
            && self.example_actions.len() <= 12
            && self.try_settings.valid()
            && self.repair_settings.valid()
            && [
                &self.prerequisite_note,
                &self.example_description,
                &self.try_change,
            ]
            .iter()
            .all(|s| !s.trim().is_empty() && s.len() <= 1600)
            && self.concepts.iter().all(|c| {
                !c.title.trim().is_empty()
                    && c.title.len() <= 100
                    && !c.text.trim().is_empty()
                    && c.text.len() <= 1200
            })
            && self.example_actions.iter().all(|a| a.valid(&self.fixture))
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err("Invalid or oversized client teaching metadata".into());
        }
        let mut c = self.fixture;
        if engine::solved(&c.state(), &self.goal()) {
            return Err("Client example must begin with work remaining".into());
        }
        for a in &self.example_actions {
            a.perform(&mut c)?;
        }
        if !engine::solved(&c.state(), &self.goal()) {
            return Err("Client example actions do not reach the supplied outcome".into());
        }
        c = self.try_settings.apply(&c)?;
        let checks = engine::checks(&c.state(), &self.goal());
        if checks.iter().all(|c| c.0) || !checks.iter().any(|c| c.0) {
            return Err("Client experiment must leave a meaningful partial result".into());
        }
        c = self.repair_settings.apply(&c)?;
        if !engine::solved(&c.state(), &self.goal()) {
            return Err("Client example repair does not restore its outcome".into());
        }
        Ok(())
    }
    pub fn valid_for(&self, initial: &State) -> bool {
        let Some(original) = Client::from_state(initial) else {
            return false;
        };
        original.valid()
            && self.validate().is_ok()
            && self.fixture.address != original.address
            && self.fixture.router != original.router
            && self.fixture.target != original.target
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u8,
    pub view: View,
    pub fixture: Client,
    pub client: Client,
    pub draft: Draft,
    #[serde(deserialize_with = "text4096")]
    pub command: String,
    #[serde(deserialize_with = "transcript")]
    pub transcript: Vec<String>,
    pub selected_action: usize,
    pub checked: bool,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Result<Self, String> {
        lesson.validate()?;
        Ok(Self {
            version: 1,
            view: View::Introduction,
            fixture: lesson.fixture,
            client: lesson.fixture,
            draft: Draft::new(&lesson.fixture),
            command: String::new(),
            transcript: Vec::new(),
            selected_action: 0,
            checked: false,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.client.same_problem(&self.fixture)
            && self.draft.valid()
            && self.command.len() <= 4096
            && self.transcript.len() <= MAX_TRANSCRIPT
            && self.transcript.iter().all(|s| s.len() <= 8192)
            && self.transcript.iter().map(String::len).sum::<usize>() <= MAX_TRANSCRIPT_BYTES
            && self.selected_action < 12
    }
    pub fn valid_for(&self, lesson: &Lesson) -> bool {
        self.valid()
            && lesson.bounded()
            && self.fixture == lesson.fixture
            && self.selected_action < lesson.example_actions.len()
    }
    pub fn fill(&mut self, lesson: &Lesson) -> Result<(), String> {
        if !self.valid_for(lesson) {
            return Err("Client example fixture or bounds differ".into());
        }
        let Some(draft) = lesson.example_actions.iter().find_map(|a| {
            if let Action::Apply { draft } = &a.action {
                Some(draft)
            } else {
                None
            }
        }) else {
            return Err("Client example has no native settings".into());
        };
        self.draft = draft.clone();
        self.checked = false;
        Ok(())
    }
    pub fn insert(&mut self, lesson: &Lesson) -> Result<(), String> {
        if !self.valid_for(lesson) {
            return Err("Client example fixture or selection differs".into());
        }
        match &lesson.example_actions[self.selected_action].action {
            Action::Apply { draft } => {
                self.draft = draft.clone();
                self.checked = false
            }
            Action::Command { command } => self.command = command.command.clone(),
        };
        Ok(())
    }
    pub fn apply(&mut self) -> Result<(), String> {
        if !self.valid() {
            return Err("Invalid client example workspace".into());
        }
        let next = self.draft.apply(&self.client)?;
        if next != self.client {
            self.checked = false;
        }
        self.client = next;
        self.draft = Draft::new(&next);
        Ok(())
    }
    pub fn submit(&mut self) -> Result<(), String> {
        if !self.valid() {
            return Err("Invalid or oversized client example workspace".into());
        }
        let command = std::mem::take(&mut self.command);
        if command.trim().is_empty() {
            return Ok(());
        }
        let before = self.client;
        let result = self.client.execute(&command);
        // Refresh only formerly clean native drafts, preserving unfinished edits.
        if self.draft.matches(&before) {
            self.draft = Draft::new(&self.client);
        }
        if self.client != before {
            self.checked = false;
        }
        self.record(format!("practice # {command}"));
        match &result {
            Ok(output) => self.record(output.clone()),
            Err(error) => self.record(format!("Command failed: {error}")),
        };
        result.map(|_| ())
    }
    fn record(&mut self, text: String) {
        let mut end = text.len().min(8192);
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let bounded = text[..end].to_string();
        self.transcript.push(bounded);
        while self.transcript.len() > MAX_TRANSCRIPT
            || self.transcript.iter().map(String::len).sum::<usize>() > MAX_TRANSCRIPT_BYTES
        {
            self.transcript.remove(0);
        }
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(lesson) || !self.draft.matches(&self.client) {
            return vec![(false,"Example settings are not applied; Apply client settings before checking these edits".into())];
        }
        engine::checks(&self.client.state(), &lesson.goal())
    }
}
pub fn prepare(attempt: &mut Attempt, lesson: &Lesson, new_attempt: bool) -> Result<(), String> {
    if lesson.validate().is_err() || Client::from_state(&attempt.state).is_none() {
        return Err("Client teaching needs the original single-client activity".into());
    }
    if let Some(w) = &attempt.workspace
        && (w.client_draft.as_ref().is_some_and(|d| !d.valid())
            || w.client_learning
                .as_ref()
                .is_some_and(|l| !l.valid_for(lesson)))
    {
        return Err("Invalid or foreign client teaching workspace".into());
    }
    let w = attempt
        .workspace
        .as_mut()
        .ok_or("Prepare the workspace first")?;
    if w.client_draft.is_none() {
        let c = Client::from_state(&attempt.state).unwrap();
        w.client_draft = Some(Draft::new(&c));
    }
    if w.client_learning.is_none() {
        let mut l = Learning::new(lesson)?;
        if !new_attempt {
            l.view = View::Assignment;
        }
        w.client_learning = Some(l);
    }
    Ok(())
}
pub fn pending_draft(attempt: &Attempt) -> bool {
    let Some(c) = Client::from_state(&attempt.state) else {
        return false;
    };
    attempt
        .workspace
        .as_ref()
        .and_then(|w| w.client_draft.as_ref())
        .is_some_and(|d| !d.matches(&c))
}
/// Atomic native editor; existing terminal syntax and Network goals are unchanged.
pub fn apply_assignment(state: &mut State, draft: &Draft) -> Result<(), String> {
    let current =
        Client::from_state(state).ok_or("No single-client configuration in this activity")?;
    let candidate = draft.apply(&current)?;
    *state = candidate.state();
    Ok(())
}
