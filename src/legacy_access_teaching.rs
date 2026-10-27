//! Bounded separate practice for the original Unix access model, not Filesystem.
use crate::{
    engine::{self, Goal, State},
    pack::CommandExample,
    progress::Attempt,
    script_teaching::{Concept, View},
    seed_dns_teaching::limits as prose,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Text/collection helpers only; no generic State or execution framework.
pub(crate) mod limits {
    use serde::{
        Deserialize, Deserializer,
        de::{Error as _, MapAccess, SeqAccess, Visitor},
    };
    use std::{collections::BTreeMap, fmt};
    pub const INPUT: usize = 4096;
    pub const OUTPUT: usize = 32768;
    pub const LINE: usize = 40960;
    pub const TOTAL: usize = 131072;
    pub struct Text<const N: usize>(pub String);
    impl<'de, const N: usize> Deserialize<'de> for Text<N> {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            struct T<const N: usize>;
            impl<const N: usize> Visitor<'_> for T<N> {
                type Value = Text<N>;
                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "text bounded to {N} bytes")
                }
                fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
                    if v.len() > N {
                        Err(E::custom("Practice text exceeds byte limit"))
                    } else {
                        Ok(Text(v.to_owned()))
                    }
                }
                fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Self::Value, E> {
                    self.visit_str(&v)
                }
            }
            d.deserialize_string(T::<N>)
        }
    }
    pub fn text<'de, D: Deserializer<'de>, const N: usize>(d: D) -> Result<String, D::Error> {
        Ok(Text::<N>::deserialize(d)?.0)
    }
    pub fn map<
        'de,
        D: Deserializer<'de>,
        V: Deserialize<'de>,
        const COUNT: usize,
        const KEY: usize,
    >(
        d: D,
    ) -> Result<BTreeMap<String, V>, D::Error> {
        struct M<V, const C: usize, const K: usize>(std::marker::PhantomData<V>);
        impl<'de, V: Deserialize<'de>, const C: usize, const K: usize> Visitor<'de> for M<V, C, K> {
            type Value = BTreeMap<String, V>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "at most {C} unique bounded entries")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut result = BTreeMap::new();
                while let Some(key) = a.next_key::<Text<K>>()? {
                    if result.len() == C {
                        return Err(A::Error::custom("Practice map entry limit"));
                    }
                    if result.contains_key(&key.0) {
                        return Err(A::Error::custom("Duplicate practice map key"));
                    }
                    result.insert(key.0, a.next_value()?);
                }
                Ok(result)
            }
        }
        d.deserialize_map(M::<V, COUNT, KEY>(std::marker::PhantomData))
    }
    pub fn transcript<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
        struct NextLine {
            allowed: bool,
            remaining: usize,
        }
        impl<'de> serde::de::DeserializeSeed<'de> for NextLine {
            type Value = String;
            fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
                if !self.allowed {
                    return Err(D::Error::custom("Practice transcript count limit"));
                }
                struct LineText(usize);
                impl Visitor<'_> for LineText {
                    type Value = String;
                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("one bounded transcript entry")
                    }
                    fn visit_str<E: serde::de::Error>(self, text: &str) -> Result<String, E> {
                        if text.len() > LINE || text.len() > self.0 {
                            return Err(E::custom("Practice transcript aggregate limit"));
                        }
                        Ok(text.to_owned())
                    }
                    fn visit_string<E: serde::de::Error>(self, text: String) -> Result<String, E> {
                        self.visit_str(&text)
                    }
                }
                d.deserialize_string(LineText(self.remaining))
            }
        }
        struct Lines;
        impl<'de> Visitor<'de> for Lines {
            type Value = Vec<String>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("bounded practice transcript")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
                let mut out = Vec::new();
                let mut total = 0;
                while let Some(line) = a.next_element_seed(NextLine {
                    allowed: out.len() < 32,
                    remaining: TOTAL - total,
                })? {
                    total += line.len();
                    out.push(line);
                }
                Ok(out)
            }
        }
        d.deserialize_seq(Lines)
    }
    pub fn valid_transcript(lines: &[String]) -> bool {
        lines.len() <= 32
            && lines.iter().all(|l| l.len() <= LINE)
            && lines.iter().map(String::len).sum::<usize>() <= TOTAL
    }
    pub fn record(lines: &mut Vec<String>, s: &str) {
        const OMIT: &str = "[Earlier practice output omitted]\n";
        let mut n = s.len().min(LINE);
        while !s.is_char_boundary(n) {
            n -= 1;
        }
        let mut line = s[..n].to_owned();
        let mut omitted = n < s.len();
        while lines.len() >= 32 || lines.iter().map(String::len).sum::<usize>() + line.len() > TOTAL
        {
            lines.remove(0);
            omitted = true;
        }
        if omitted {
            let mut end = line.len().min(LINE - OMIT.len());
            while !line.is_char_boundary(end) {
                end -= 1;
            }
            line = format!("{OMIT}{}", &line[..end]);
        }
        while lines.iter().map(String::len).sum::<usize>() + line.len() > TOTAL {
            lines.remove(0);
        }
        lines.push(line);
    }
    #[derive(Deserialize)]
    struct Command(
        #[serde(deserialize_with = "crate::seed_dns_teaching::limits::command")]
        crate::pack::CommandExample,
    );
    pub fn commands<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Vec<crate::pack::CommandExample>, D::Error> {
        Ok(
            crate::progress::bounded_saved_sequence::<D, Command, 12>(d)?
                .into_iter()
                .map(|x| x.0)
                .collect(),
        )
    }
    pub fn sequence<'de, D: Deserializer<'de>, const N: usize>(
        d: D,
    ) -> Result<Vec<String>, D::Error> {
        Ok(
            crate::progress::bounded_saved_sequence::<D, Text<4096>, N>(d)?
                .into_iter()
                .map(|x| x.0)
                .collect(),
        )
    }
    pub fn commands_valid(commands: &[crate::pack::CommandExample]) -> bool {
        !commands.is_empty()
            && commands.len() <= 12
            && commands.iter().all(|c| {
                c.command.len() <= 256
                    && !c.command.trim().is_empty()
                    && c.explanation.len() <= 600
                    && !c.explanation.trim().is_empty()
                    && c.simulator_only
                        == (c.command.split_whitespace().next() == Some("test-access"))
            })
    }
    pub fn prose_valid(
        note: &str,
        concepts: &[crate::script_teaching::Concept],
        description: &str,
        experiment: &str,
    ) -> bool {
        [note, description, experiment]
            .iter()
            .all(|s| s.len() <= 1600 && !s.trim().is_empty())
            && !concepts.is_empty()
            && concepts.len() <= 6
            && concepts.iter().all(|c| {
                c.title.len() <= 100
                    && !c.title.trim().is_empty()
                    && c.text.len() <= 1200
                    && !c.text.trim().is_empty()
            })
    }
    pub fn sequence_valid(v: &[String], max: usize) -> bool {
        !v.is_empty() && v.len() <= max && v.iter().all(|s| s.len() <= 4096 && !s.trim().is_empty())
    }
}
fn name(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 32
        && s.bytes().enumerate().all(|(i, c)| {
            c.is_ascii_lowercase() || c == b'_' || (i > 0 && (c.is_ascii_digit() || c == b'-'))
        })
}
fn path(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 256
        && s.starts_with('/')
        && !s.contains("..")
        && !s.chars().any(char::is_whitespace)
        && !s.bytes().any(|c| {
            c.is_ascii_control() || c.is_ascii_whitespace() || c == b'\\' || c == b'\'' || c == b'"'
        })
        && (s == "/" || (!s.ends_with('/') && !s.contains("//") && !s.split('/').any(|p| p == ".")))
}
fn users_valid(users: &BTreeMap<String, Vec<String>>) -> bool {
    !users.is_empty()
        && users.len() <= 16
        && users.iter().all(|(u, g)| {
            name(u)
                && !g.is_empty()
                && g.len() <= 16
                && g.iter().all(|s| name(s))
                && g.iter().collect::<BTreeSet<_>>().len() == g.len()
        })
}
fn fields(mode: u16, owner: &str, group: &str, users: &BTreeMap<String, Vec<String>>) -> bool {
    mode <= 0o777
        && name(owner)
        && name(group)
        && users.contains_key(owner)
        && users.values().any(|gs| gs.iter().any(|g| g == group))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub mode: u16,
    #[serde(deserialize_with = "limits::text::<_,32>")]
    pub owner: String,
    #[serde(deserialize_with = "limits::text::<_,32>")]
    pub group: String,
    pub directory: bool,
}
fn nodes<'de, D: serde::Deserializer<'de>>(d: D) -> Result<BTreeMap<String, Entry>, D::Error> {
    limits::map::<D, Entry, 64, 256>(d)
}
fn users<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> Result<BTreeMap<String, Vec<String>>, D::Error> {
    #[derive(Deserialize)]
    struct Groups(#[serde(deserialize_with = "groups")] Vec<String>);
    fn groups<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<String>, D::Error> {
        Ok(
            crate::progress::bounded_saved_sequence::<D, limits::Text<32>, 16>(d)?
                .into_iter()
                .map(|s| s.0)
                .collect(),
        )
    }
    Ok(limits::map::<D, Groups, 16, 32>(d)?
        .into_iter()
        .map(|(k, v)| (k, v.0))
        .collect())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    #[serde(deserialize_with = "nodes")]
    pub nodes: BTreeMap<String, Entry>,
    #[serde(deserialize_with = "users")]
    pub users: BTreeMap<String, Vec<String>>,
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
        users_valid(&self.users)
            && !self.nodes.is_empty()
            && self.nodes.len() <= 64
            && self.nodes.get("/").is_some_and(|e| e.directory)
            && self.nodes.iter().all(|(p, n)| {
                path(p)
                    && fields(n.mode, &n.owner, &n.group, &self.users)
                    && (p == "/"
                        || self
                            .nodes
                            .get(
                                p.rsplit_once('/')
                                    .map(|(s, _)| if s.is_empty() { "/" } else { s })
                                    .unwrap(),
                            )
                            .is_some_and(|n| n.directory))
            })
    }
    pub fn from_state(s: &State) -> Option<Self> {
        let State::Unix { nodes, users } = s else {
            return None;
        };
        if !users_valid(users)
            || nodes.is_empty()
            || nodes.len() > 64
            || !nodes.get("/").is_some_and(|n| n.directory)
            || !nodes.iter().all(|(p, n)| {
                path(p)
                    && fields(n.mode, &n.owner, &n.group, users)
                    && (p == "/"
                        || nodes
                            .get(
                                p.rsplit_once('/')
                                    .map(|(s, _)| if s.is_empty() { "/" } else { s })
                                    .unwrap(),
                            )
                            .is_some_and(|n| n.directory))
            })
        {
            return None;
        }
        Some(Self {
            nodes: nodes
                .iter()
                .map(|(p, n)| {
                    (
                        p.clone(),
                        Entry {
                            mode: n.mode,
                            owner: n.owner.clone(),
                            group: n.group.clone(),
                            directory: n.directory,
                        },
                    )
                })
                .collect(),
            users: users.clone(),
        })
    }
    pub fn state(&self) -> Result<State, String> {
        if !self.valid() {
            return Err("Invalid access practice snapshot".into());
        }
        Ok(State::Unix {
            nodes: self
                .nodes
                .iter()
                .map(|(p, n)| {
                    (
                        p.clone(),
                        engine::Node {
                            mode: n.mode,
                            owner: n.owner.clone(),
                            group: n.group.clone(),
                            directory: n.directory,
                        },
                    )
                })
                .collect(),
            users: self.users.clone(),
        })
    }
    pub fn same_problem(&self, fixture: &Self) -> bool {
        self.valid()
            && fixture.valid()
            && self.nodes.keys().eq(fixture.nodes.keys())
            && self
                .nodes
                .iter()
                .all(|(p, n)| n.directory == fixture.nodes[p].directory)
            && self.users.keys().eq(fixture.users.keys())
            && self
                .users
                .iter()
                .all(|(user, groups)| groups.first() == fixture.users[user].first())
            && self.users.values().flatten().collect::<BTreeSet<_>>()
                == fixture.users.values().flatten().collect::<BTreeSet<_>>()
    }
    pub fn execute(&mut self, command: &str) -> Result<String, String> {
        if command.len() > limits::INPUT {
            return Err("Practice command exceeds 4096 bytes".into());
        }
        let mut candidate = self.state()?;
        let output = engine::execute(&mut candidate, command).map_err(|e| prose::message(&e))?;
        let next =
            Self::from_state(&candidate).ok_or("Practice result exceeds its bounded snapshot")?;
        if !next.same_problem(self) || output.len() > limits::OUTPUT {
            return Err("Practice output/context exceeds supported bounds".into());
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
    #[serde(deserialize_with = "limits::text::<_,32>")]
    pub user: String,
    pub read: bool,
    pub write: bool,
    pub private: bool,
    pub exclusive: bool,
}
impl Task {
    pub fn valid_for(&self, fixture: &Snapshot) -> bool {
        fixture.valid()
            && path(&self.path)
            && name(&self.user)
            && (self.read || self.write)
            && fixture.users.contains_key(&self.user)
            && fixture.nodes.get(&self.path).is_some_and(|n| !n.directory)
    }
    pub fn goal(&self, fixture: &Snapshot) -> Result<Goal, String> {
        if !self.valid_for(fixture) {
            return Err("Invalid access practice task".into());
        }
        Ok(Goal::Access {
            path: self.path.clone(),
            user: self.user.clone(),
            read: self.read,
            write: self.write,
            private: self.private,
            exclusive: self.exclusive,
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
            return Err("Invalid bounded access teaching".into());
        }
        let goal = self.task.goal(&self.fixture)?;
        let mut practice = self.fixture.clone();
        for c in &self.commands {
            practice.execute(&c.command)?;
        }
        practice = self.fixture.clone();
        if engine::solved(&practice.state()?, &goal) {
            return Err("Access example starts solved".into());
        }
        for c in &self.reference {
            practice.execute(c)?;
        }
        if !engine::solved(&practice.state()?, &goal) {
            return Err("Access reference does not reach task".into());
        }
        for c in &self.try_commands {
            practice.execute(c)?;
        }
        if engine::solved(&practice.state()?, &goal) {
            return Err("Access experiment must break current task".into());
        }
        for c in &self.repair_commands {
            practice.execute(c)?;
        }
        if !engine::solved(&practice.state()?, &goal) {
            return Err("Access repair must restore current task".into());
        }
        Ok(())
    }
    pub fn valid_for(&self, initial: &State, goal: &Goal) -> bool {
        let (State::Unix { nodes, users }, Goal::Access { path, user, .. }) = (initial, goal)
        else {
            return false;
        };
        self.validate().is_ok()
            && nodes.contains_key(path)
            && users.contains_key(user)
            && !nodes.contains_key(&self.task.path)
            && !users.contains_key(&self.task.user)
            && self
                .fixture
                .users
                .keys()
                .filter(|u| u.as_str() != "root" && u.as_str() != "nobody")
                .all(|u| !users.contains_key(u))
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
            return Err("Foreign access practice".into());
        }
        self.command = l.commands[self.selected_action].command.clone();
        Ok(())
    }
    pub fn submit(&mut self, l: &Lesson) -> Result<(), String> {
        if !self.valid_for(l) {
            return Err("Invalid or foreign access practice".into());
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
            return vec![(false, "Invalid or foreign access practice".into())];
        }
        engine::checks(
            &self.current.state().expect("bounded practice"),
            &l.task.goal(&l.fixture).expect("bounded task"),
        )
        .into_iter()
        .enumerate()
        .map(|(index, (pass, message))| {
            let message = if pass {
                message
            } else if index == 0 {
                prose::message(&format!(
                    "Requested access for {} to {} is blocked. Inspect the file's access class and each parent directory's search permission.",
                    l.task.user, l.task.path
                ))
            } else if l.task.exclusive {
                "Another modeled user still has access, or other-class file permissions remain. Inspect those access rules and retry.".into()
            } else {
                "The file still has other-class permission bits. Remove public access while keeping the requested user's access.".into()
            };
            (pass, message)
        })
        .collect()
    }
}
pub fn prepare(a: &mut Attempt, l: &Lesson, new_attempt: bool) -> Result<(), String> {
    crate::workspace::reject_nfs_metadata(a)?;
    if !matches!(a.state, State::Unix { .. }) {
        return Err("Access teaching belongs to the original access task".into());
    }
    let w = a.workspace.as_mut().ok_or("Prepare workspace first")?;
    if w.legacy_access_learning
        .as_ref()
        .is_some_and(|p| !p.valid_for(l))
    {
        return Err("Foreign access example retained".into());
    }
    if w.legacy_access_learning.is_none() {
        let mut p = Learning::new(l)?;
        if !new_attempt {
            p.view = View::Assignment;
        }
        w.legacy_access_learning = Some(p);
    }
    Ok(())
}
