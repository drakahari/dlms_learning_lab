//! Independent teaching for the original one-record Dns model, not AuthoritativeDns.
use crate::{
    engine::{self, Goal, State},
    pack::CommandExample,
    progress::Attempt,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, net::Ipv4Addr};

// Shared scalar/sequence resource helpers only. No state/plugin abstraction.
pub(crate) mod limits {
    use serde::{Deserialize, Deserializer};
    pub const TRANSCRIPT_COUNT: usize = 32;
    pub const TRANSCRIPT_BYTES: usize = 131_072;
    fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
        struct Text<const N: usize>;
        impl<'de, const N: usize> serde::de::Visitor<'de> for Text<N> {
            type Value = String;
            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "at most {N} bytes of text")
            }
            fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<String, E> {
                if s.len() > N {
                    Err(E::custom("Seed teaching text exceeds its byte limit"))
                } else {
                    Ok(s.into())
                }
            }
            fn visit_string<E: serde::de::Error>(self, s: String) -> Result<String, E> {
                if s.len() > N {
                    Err(E::custom("Seed teaching text exceeds its byte limit"))
                } else {
                    Ok(s.as_str().to_owned())
                }
            }
        }
        d.deserialize_string(Text::<N>)
    }
    pub fn text15<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        text::<D, 15>(d)
    }
    pub fn text100<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        text::<D, 100>(d)
    }
    pub fn text600<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        text::<D, 600>(d)
    }
    pub fn text1200<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        text::<D, 1200>(d)
    }
    pub fn text1600<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        text::<D, 1600>(d)
    }
    pub fn ipv4<'de, D: Deserializer<'de>>(d: D) -> Result<std::net::Ipv4Addr, D::Error> {
        use serde::de::Error as _;
        text15(d)?
            .parse()
            .map_err(|_| D::Error::custom("Invalid example IPv4 address"))
    }
    #[derive(Deserialize)]
    struct Address(#[serde(deserialize_with = "ipv4")] std::net::Ipv4Addr);
    pub fn optional_ipv4<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<std::net::Ipv4Addr>, D::Error> {
        Ok(Option::<Address>::deserialize(d)?.map(|a| a.0))
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct BoundedConcept {
        #[serde(deserialize_with = "text100")]
        title: String,
        #[serde(deserialize_with = "text1200")]
        text: String,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct BoundedCommand {
        #[serde(deserialize_with = "text256")]
        command: String,
        #[serde(deserialize_with = "text600")]
        explanation: String,
        simulator_only: bool,
    }
    pub fn command<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<crate::pack::CommandExample, D::Error> {
        let c = BoundedCommand::deserialize(d)?;
        Ok(crate::pack::CommandExample {
            command: c.command,
            explanation: c.explanation,
            simulator_only: c.simulator_only,
        })
    }
    pub fn text253<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        text::<D, 253>(d)
    }
    pub fn text256<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        text::<D, 256>(d)
    }
    pub fn text4096<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        text::<D, 4096>(d)
    }
    fn text8192<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
        text::<D, 8192>(d)
    }
    #[derive(Deserialize)]
    struct Line(#[serde(deserialize_with = "text8192")] String);
    pub fn transcript<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
        Ok(
            crate::progress::bounded_saved_sequence::<D, Line, TRANSCRIPT_COUNT>(d)?
                .into_iter()
                .map(|x| x.0)
                .collect(),
        )
    }
    pub fn concepts<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Vec<crate::script_teaching::Concept>, D::Error> {
        Ok(
            crate::progress::bounded_saved_sequence::<D, BoundedConcept, 6>(d)?
                .into_iter()
                .map(|c| crate::script_teaching::Concept {
                    title: c.title,
                    text: c.text,
                })
                .collect(),
        )
    }
    pub fn message(s: &str) -> String {
        let mut n = s.len().min(512);
        while !s.is_char_boundary(n) {
            n -= 1;
        }
        s[..n].to_owned()
    }
    pub fn record(lines: &mut Vec<String>, s: &str) {
        const MARKER: &str = "[Earlier example output omitted]\n";
        let mut end = s.len().min(8192);
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        let mut line = s[..end].to_owned();
        let mut omitted = end < s.len();
        while lines.len() >= TRANSCRIPT_COUNT
            || lines.iter().map(String::len).sum::<usize>() + line.len() > TRANSCRIPT_BYTES
        {
            lines.remove(0);
            omitted = true;
        }
        if omitted {
            let mut end = line.len().min(8192 - MARKER.len());
            while !line.is_char_boundary(end) {
                end -= 1;
            }
            line = format!("{MARKER}{}", &line[..end]);
            while !lines.is_empty()
                && lines.iter().map(String::len).sum::<usize>() + line.len() > TRANSCRIPT_BYTES
            {
                lines.remove(0);
            }
        }
        lines.push(line);
    }
    pub fn transcript_valid(lines: &[String]) -> bool {
        lines.len() <= TRANSCRIPT_COUNT
            && lines.iter().all(|s| s.len() <= 8192)
            && lines.iter().map(String::len).sum::<usize>() <= TRANSCRIPT_BYTES
    }
}
fn name_valid(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 253
        && name.ends_with(".lab.test")
        && name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-')
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Zone {
    #[serde(deserialize_with = "limits::text253")]
    pub name: String,
    #[serde(deserialize_with = "limits::ipv4")]
    pub saved: Ipv4Addr,
    #[serde(deserialize_with = "limits::optional_ipv4")]
    pub served: Option<Ipv4Addr>,
    pub serial: u32,
    pub loaded_serial: u32,
}
impl Zone {
    pub fn valid(&self) -> bool {
        name_valid(&self.name)
    }
    pub fn from_state(s: &State) -> Option<Self> {
        let State::Dns {
            records,
            loaded,
            serial,
            loaded_serial,
        } = s
        else {
            return None;
        };
        if records.len() != 1 || loaded.len() > 1 {
            return None;
        }
        let (name, address) = records.first_key_value()?;
        if !name_valid(name) || address.len() > 15 || loaded.keys().any(|key| key != name) {
            return None;
        }
        let saved = address.parse().ok()?;
        let served = match loaded.get(name) {
            Some(a) if a.len() <= 15 => Some(a.parse().ok()?),
            Some(_) => return None,
            None => None,
        };
        Some(Self {
            name: name.clone(),
            saved,
            served,
            serial: *serial,
            loaded_serial: *loaded_serial,
        })
    }
    pub fn state(&self) -> Result<State, String> {
        if !self.valid() {
            return Err("Invalid DNS example".into());
        }
        Ok(State::Dns {
            records: BTreeMap::from([(self.name.clone(), self.saved.to_string())]),
            loaded: self
                .served
                .map(|a| BTreeMap::from([(self.name.clone(), a.to_string())]))
                .unwrap_or_default(),
            serial: self.serial,
            loaded_serial: self.loaded_serial,
        })
    }
    pub fn same_problem(&self, fixture: &Self) -> bool {
        self.valid() && fixture.valid() && self.name == fixture.name
    }
    pub fn execute(&mut self, command: &str) -> Result<String, String> {
        if !self.valid() || command.len() > 4096 {
            return Err("Invalid DNS example or command exceeds 4,096 bytes".into());
        }
        let mut candidate = self.state()?;
        let result = engine::execute(&mut candidate, command);
        if result.is_ok() {
            let next =
                Self::from_state(&candidate).ok_or("DNS command changed the one-record example")?;
            if !next.same_problem(self) {
                return Err("Keep the supplied example record name".into());
            }
            *self = next;
        }
        result.map_err(|e| limits::message(&e))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "limits::text256")]
    pub address: String,
    #[serde(deserialize_with = "limits::text256")]
    pub serial: String,
}
impl Draft {
    pub fn new(z: &Zone) -> Self {
        Self {
            address: z.saved.to_string(),
            serial: z.serial.to_string(),
        }
    }
    pub fn valid(&self) -> bool {
        self.address.len() <= 256 && self.serial.len() <= 256
    }
    pub fn matches(&self, z: &Zone) -> bool {
        self.address == z.saved.to_string() && self.serial == z.serial.to_string()
    }
    pub fn apply(&self, z: &Zone) -> Result<Zone, String> {
        if !self.valid() || !z.valid() {
            return Err("Invalid or oversized DNS example settings".into());
        }
        let mut candidate = z.state()?;
        crate::workspace::edit_dns(&mut candidate, &self.address, &self.serial)
            .map_err(|e| limits::message(&e))?;
        Zone::from_state(&candidate).ok_or("DNS settings changed the example context".into())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Save {
        draft: Draft,
    },
    Command {
        #[serde(deserialize_with = "limits::command")]
        command: CommandExample,
    },
}
impl Action {
    fn bounded(&self, fixture: &Zone) -> bool {
        match self {
            Self::Save { draft } => draft.valid(),
            Self::Command { command } => {
                if command.command.is_empty()
                    || command.command.len() > 256
                    || command.explanation.trim().is_empty()
                    || command.explanation.len() > 600
                {
                    return false;
                }
                let words: Vec<_> = command.command.split_ascii_whitespace().collect();
                let teaching = words == ["named-checkzone", "lab.test"];
                command.simulator_only == teaching
                    && match words.as_slice() {
                        ["cat", "/etc/named/lab.zone"]
                        | ["named-checkzone", "lab.test"]
                        | ["systemctl", "reload", "named"] => true,
                        ["dig", name] => *name == fixture.name,
                        _ => false,
                    }
            }
        }
    }
    pub fn perform(&self, z: &mut Zone) -> Result<(), String> {
        match self {
            Self::Save { draft } => {
                *z = draft.apply(z)?;
                Ok(())
            }
            Self::Command { command } => z.execute(&command.command).map(|_| ()),
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleAction {
    #[serde(deserialize_with = "limits::text100")]
    pub title: String,
    #[serde(deserialize_with = "limits::text1200")]
    pub explanation: String,
    pub action: Action,
}
fn examples<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<ExampleAction>, D::Error> {
    crate::progress::bounded_saved_sequence::<D, _, 12>(d)
}
fn experiments<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Action>, D::Error> {
    crate::progress::bounded_saved_sequence::<D, _, 4>(d)
}
fn repairs<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Action>, D::Error> {
    crate::progress::bounded_saved_sequence::<D, _, 6>(d)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    #[serde(deserialize_with = "limits::text1600")]
    pub prerequisite_note: String,
    #[serde(deserialize_with = "limits::concepts")]
    pub concepts: Vec<Concept>,
    #[serde(deserialize_with = "limits::text1600")]
    pub example_description: String,
    pub fixture: Zone,
    #[serde(deserialize_with = "limits::ipv4")]
    pub required_address: Ipv4Addr,
    #[serde(deserialize_with = "examples")]
    pub example_actions: Vec<ExampleAction>,
    #[serde(deserialize_with = "limits::text1600")]
    pub try_change: String,
    #[serde(deserialize_with = "experiments")]
    pub try_actions: Vec<Action>,
    #[serde(deserialize_with = "repairs")]
    pub repair_actions: Vec<Action>,
}
impl Lesson {
    pub fn goal(&self) -> Result<Goal, String> {
        if !self.fixture.valid() {
            return Err("Invalid DNS example identity".into());
        }
        Ok(Goal::Dns {
            name: self.fixture.name.clone(),
            address: self.required_address,
        })
    }
    pub fn bounded(&self) -> bool {
        self.fixture.valid()
            && !self.concepts.is_empty()
            && self.concepts.len() <= 6
            && self.concepts.iter().all(|c| {
                !c.title.trim().is_empty()
                    && c.title.len() <= 100
                    && !c.text.trim().is_empty()
                    && c.text.len() <= 1200
            })
            && [
                &self.prerequisite_note,
                &self.example_description,
                &self.try_change,
            ]
            .iter()
            .all(|s| !s.trim().is_empty() && s.len() <= 1600)
            && !self.example_actions.is_empty()
            && self.example_actions.len() <= 12
            && self.example_actions.iter().all(|a| {
                !a.title.trim().is_empty()
                    && a.title.len() <= 100
                    && !a.explanation.trim().is_empty()
                    && a.explanation.len() <= 1200
                    && a.action.bounded(&self.fixture)
            })
            && !self.try_actions.is_empty()
            && self.try_actions.len() <= 4
            && self.try_actions.iter().all(|a| a.bounded(&self.fixture))
            && !self.repair_actions.is_empty()
            && self.repair_actions.len() <= 6
            && self.repair_actions.iter().all(|a| a.bounded(&self.fixture))
    }
    pub fn validate(&self) -> Result<(), String> {
        if !self.bounded() {
            return Err("Invalid or oversized seed DNS teaching".into());
        }
        let mut z = self.fixture.clone();
        if engine::solved(&z.state()?, &self.goal()?) {
            return Err("DNS example must begin incomplete".into());
        }
        for a in &self.example_actions {
            a.action.perform(&mut z)?;
        }
        if !engine::solved(&z.state()?, &self.goal()?) {
            return Err("DNS example is not reachable".into());
        }
        for a in &self.try_actions {
            a.perform(&mut z)?;
        }
        if engine::solved(&z.state()?, &self.goal()?) {
            return Err("DNS experiment must leave work remaining".into());
        }
        for a in &self.repair_actions {
            a.perform(&mut z)?;
        }
        if !engine::solved(&z.state()?, &self.goal()?) {
            return Err("DNS repair does not restore the example".into());
        }
        Ok(())
    }
    pub fn valid_for(&self, initial: &State, goal: &Goal) -> bool {
        let State::Dns {
            records, loaded, ..
        } = initial
        else {
            return false;
        };
        let Goal::Dns { name, address } = goal else {
            return false;
        };
        name_valid(name)
            && records.len() == 1
            && records.contains_key(name)
            && loaded.len() <= 1
            && loaded.keys().all(|key| records.contains_key(key))
            && self.fixture.name != *name
            && self.required_address != *address
            && self.validate().is_ok()
    }
}
/// Only commands' immutable problem identity; editable values/serials stay free.
pub fn assignment_context(current: &State, initial: &State) -> bool {
    let (
        State::Dns {
            records, loaded, ..
        },
        State::Dns {
            records: original, ..
        },
    ) = (current, initial)
    else {
        return false;
    };
    records.len() == 1
        && original.len() == 1
        && original.keys().next().is_some_and(|name| name_valid(name))
        && records.keys().eq(original.keys())
        && loaded.len() <= 1
        && loaded.keys().all(|key| original.contains_key(key))
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u8,
    pub view: View,
    pub fixture: Zone,
    pub current: Zone,
    pub draft: Draft,
    #[serde(deserialize_with = "limits::text4096")]
    pub command: String,
    #[serde(deserialize_with = "limits::transcript")]
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
            fixture: lesson.fixture.clone(),
            current: lesson.fixture.clone(),
            draft: Draft::new(&lesson.fixture),
            command: String::new(),
            transcript: Vec::new(),
            selected_action: 0,
            checked: false,
        })
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && self.current.same_problem(&self.fixture)
            && self.draft.valid()
            && self.command.len() <= 4096
            && limits::transcript_valid(&self.transcript)
            && self.selected_action < 12
    }
    pub fn valid_for(&self, l: &Lesson) -> bool {
        self.valid()
            && l.bounded()
            && self.fixture == l.fixture
            && self.selected_action < l.example_actions.len()
    }
    pub fn insert(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("DNS example context or selection differs".into());
        }
        match &l.example_actions[self.selected_action].action {
            Action::Save { draft } => {
                self.draft = draft.clone();
                self.checked = false;
            }
            Action::Command { command } => self.command = command.command.clone(),
        }
        Ok(())
    }
    pub fn save(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("DNS example context differs".into());
        }
        let next = self.draft.apply(&self.current)?;
        if next != self.current {
            self.checked = false;
        }
        self.current = next;
        self.draft = Draft::new(&self.current);
        Ok(())
    }
    pub fn submit(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Invalid or foreign DNS example".into());
        }
        let command = std::mem::take(&mut self.command);
        if command.trim().is_empty() {
            return Ok(());
        }
        let before = self.current.clone();
        let result = self.current.execute(&command);
        if self.draft.matches(&before) {
            self.draft = Draft::new(&self.current);
        }
        if before != self.current {
            self.checked = false;
        }
        limits::record(&mut self.transcript, &format!("practice # {command}"));
        match &result {
            Ok(output) => limits::record(&mut self.transcript, output),
            Err(error) => limits::record(&mut self.transcript, &format!("Command failed: {error}")),
        };
        result.map(|_| ())
    }
    pub fn checks(&self, l: &Lesson) -> Vec<(bool, String)> {
        if !self.valid_for(l) {
            return vec![(false, "Invalid or foreign DNS example".into())];
        }
        if !self.draft.matches(&self.current) {
            return vec![(
                false,
                "Unsaved example settings; Save configuration before checking these edits".into(),
            )];
        }
        engine::checks(
            &self.current.state().expect("validated DNS example"),
            &l.goal().expect("bounded lesson"),
        )
        .into_iter()
        .enumerate()
        .map(|(index, (pass, message))| {
            if pass {
                return (pass, message);
            }
            let diagnosis = match index {
                0 => format!(
                    "Saved address {} differs from the practice task. Inspect the saved record.",
                    self.current.saved
                ),
                1 => match self.current.served {
                    None => "No answer is loaded for this name. Inspect the saved record and serving copy.".into(),
                    Some(address) if address != l.required_address => format!(
                        "Serving address {address} differs from the practice task. Inspect the saved record and whether its changes are active."
                    ),
                    Some(_) => format!(
                        "Serving serial {} differs from saved serial {}. The saved and serving copies are not synchronized.",
                        self.current.loaded_serial, self.current.serial
                    ),
                },
                _ => message,
            };
            (pass, limits::message(&diagnosis))
        })
        .collect()
    }
}
pub fn prepare(a: &mut Attempt, l: &Lesson, new_attempt: bool) -> Result<(), String> {
    if !matches!(a.state, State::Dns { .. }) || l.validate().is_err() {
        return Err("Seed DNS teaching needs the original DNS activity".into());
    }
    let w = a.workspace.as_mut().ok_or("Prepare the workspace first")?;
    if w.seed_dns_learning
        .as_ref()
        .is_some_and(|x| !x.valid_for(l))
    {
        return Err("Foreign or invalid DNS example".into());
    }
    if w.seed_dns_learning.is_none() {
        let mut x = Learning::new(l)?;
        if !new_attempt {
            x.view = View::Assignment;
        }
        w.seed_dns_learning = Some(x);
    }
    Ok(())
}
