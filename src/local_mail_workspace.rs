//! Separate editor, saved mail files and alias index for bounded local delivery.
use crate::{cron, cron_workspace as bounded, local_mail as core};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const HELP: &str = "Edit only the supplied simulated /etc/aliases and .forward files. Save file validates the chosen document. Rebuild aliases represents newaliases and refreshes the lookup from the saved aliases file. Run delivery test sends separate supplied sample messages through the bounded local-recipient model. No host mail or shell command runs.";
type Checks = Vec<(bool, String)>;

fn names<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    #[derive(Deserialize)]
    struct Name(#[serde(deserialize_with = "cron::text::<_,32>")] String);
    bounded::sequence::<D, Name, 8>(deserializer)
        .map(|values| values.into_iter().map(|name| name.0).collect())
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Files {
    #[serde(deserialize_with = "cron::text::<_,4096>")]
    pub aliases: String,
    #[serde(deserialize_with = "cron::map::<_,8,4096>")]
    pub forwards: BTreeMap<String, String>,
}

impl Files {
    fn shape(&self) -> Result<(), String> {
        if self.aliases.len() > core::MAX_TEXT
            || self.aliases.contains('\0')
            || self.forwards.len() > 8
            || self.forwards.iter().any(|(owner, value)| {
                !core::name(owner) || value.len() > core::MAX_TEXT || value.contains('\0')
            })
        {
            return Err("Keep the supplied virtual mail documents within their limits.".into());
        }
        Ok(())
    }

    fn directory(&self, users: &[String], index: &str) -> Result<core::Directory, String> {
        self.shape()?;
        let aliases = core::parse_aliases(index)?;
        let forwards = self
            .forwards
            .iter()
            .map(|(user, text)| Ok((user.clone(), core::parse_forward(text)?)))
            .collect::<Result<_, String>>()?;
        let directory = core::Directory {
            users: users.iter().cloned().collect(),
            aliases,
            forwards,
        };
        directory.validate()?;
        Ok(directory)
    }

    fn saved_shape(&self, users: &[String]) -> Result<(), String> {
        self.directory(users, &self.aliases).map(|_| ())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    #[serde(deserialize_with = "cron::text::<_,64>")]
    pub label: String,
    #[serde(deserialize_with = "cron::text::<_,32>")]
    pub recipient: String,
    #[serde(deserialize_with = "names")]
    pub expected: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "cron::text::<_,64>")]
    pub id: String,
    #[serde(deserialize_with = "cron::text::<_,1600>")]
    pub instructions: String,
    #[serde(deserialize_with = "names")]
    pub users: Vec<String>,
    pub initial: Files,
    #[serde(deserialize_with = "cron::text::<_,4096>")]
    pub initial_index: String,
    pub reference: Files,
    pub focus: Document,
    #[serde(deserialize_with = "bounded::sequence::<_,Probe,8>")]
    pub probes: Vec<Probe>,
}

impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || self.instructions.trim().is_empty()
            || self.instructions.len() > 1600
            || self.instructions.contains('\0')
            || !(1..=8).contains(&self.users.len())
            || self.users.iter().collect::<BTreeSet<_>>().len() != self.users.len()
            || self
                .initial
                .forwards
                .keys()
                .ne(self.reference.forwards.keys())
            || matches!(&self.focus, Document::Forward(user) if !self.initial.forwards.contains_key(user))
            || !(2..=8).contains(&self.probes.len())
        {
            return Err(
                "Supply distinct local users, bounded documents and at least two routing probes."
                    .into(),
            );
        }
        self.initial.saved_shape(&self.users)?;
        self.reference.saved_shape(&self.users)?;
        self.initial.directory(&self.users, &self.initial_index)?;
        let known: BTreeSet<_> = self.users.iter().collect();
        for probe in &self.probes {
            if probe.label.trim().is_empty()
                || probe.label.len() > 64
                || probe.label.chars().any(char::is_control)
                || !core::name(&probe.recipient)
                || probe.expected.is_empty()
                || probe.expected.len() > 8
                || probe.expected.iter().any(|user| !known.contains(user))
                || probe.expected.iter().collect::<BTreeSet<_>>().len() != probe.expected.len()
            {
                return Err("Each sample message needs a recipient and distinct supplied destination mailboxes.".into());
            }
        }
        let initial = self.evaluate(&self.initial, &self.initial_index)?;
        let initial_saved = self.evaluate(&self.initial, &self.initial.aliases)?;
        let reference = self.evaluate(&self.reference, &self.reference.aliases)?;
        if (initial.iter().all(|row| row.0) && initial_saved.iter().all(|row| row.0))
            || reference.iter().any(|row| !row.0)
        {
            return Err(
                "Provide an incomplete starter and a reachable local delivery policy.".into(),
            );
        }
        Ok(())
    }

    fn evaluate(&self, saved: &Files, index: &str) -> Result<Checks, String> {
        let directory = saved.directory(&self.users, index)?;
        self.probes
            .iter()
            .map(|probe| {
                let actual = directory.deliver(&probe.recipient)?;
                let expected: BTreeSet<_> = probe.expected.iter().cloned().collect();
                let missing = expected
                    .difference(&actual.mailboxes)
                    .cloned()
                    .collect::<Vec<_>>();
                let extra = actual
                    .mailboxes
                    .difference(&expected)
                    .cloned()
                    .collect::<Vec<_>>();
                let matched = missing.is_empty() && extra.is_empty() && actual.failures.is_empty();
                let mut feedback = format!("{} sent to {}.", probe.label, probe.recipient);
                if !missing.is_empty() {
                    feedback.push_str(&format!(" Missing delivery to {}.", missing.join(", ")));
                }
                if !extra.is_empty() {
                    feedback.push_str(&format!(" Unwanted delivery to {}.", extra.join(", ")));
                }
                for failure in &actual.failures {
                    feedback.push_str(&format!(" {failure}"));
                }
                Ok((matched, feedback))
            })
            .collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub saved: Files,
    #[serde(deserialize_with = "cron::text::<_,4096>")]
    pub index: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Document {
    Aliases,
    Forward(#[serde(deserialize_with = "cron::text::<_,32>")] String),
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub documents: Files,
    pub saved: Files,
    #[serde(deserialize_with = "cron::text::<_,4096>")]
    pub index: String,
    pub selected: Document,
    pub tested: Option<Snapshot>,
}

impl Draft {
    pub fn new(task: &Task) -> Self {
        Self {
            documents: task.initial.clone(),
            saved: task.initial.clone(),
            index: task.initial_index.clone(),
            selected: task.focus.clone(),
            tested: None,
        }
    }

    fn shape(&self) -> Result<(), String> {
        self.documents.shape()?;
        self.saved.shape()?;
        if self.index.len() > core::MAX_TEXT || self.index.contains('\0') {
            return Err("The saved alias index is outside the simulation’s limits.".into());
        }
        if self
            .tested
            .as_ref()
            .is_some_and(|record| record.saved != self.saved || record.index != self.index)
        {
            return Err(
                "The saved mail test does not match the current simulated files and alias index."
                    .into(),
            );
        }
        Ok(())
    }

    pub fn bound(&self, task: &Task) -> Result<(), String> {
        self.shape()?;
        if self
            .documents
            .forwards
            .keys()
            .ne(task.initial.forwards.keys())
            || self.saved.forwards.keys().ne(task.initial.forwards.keys())
            || matches!(&self.selected, Document::Forward(user) if !self.documents.forwards.contains_key(user))
        {
            return Err("Only the supplied virtual mail files can be edited.".into());
        }
        self.saved.saved_shape(&task.users)?;
        self.saved.directory(&task.users, &self.index).map(|_| ())
    }

    pub fn text(&self) -> &str {
        match &self.selected {
            Document::Aliases => &self.documents.aliases,
            Document::Forward(user) => self.documents.forwards.get(user).map_or("", String::as_str),
        }
    }

    pub fn edit(&mut self, text: String) -> Result<(), String> {
        if text.len() > core::MAX_TEXT || text.contains('\0') {
            return Err("Keep this simulated mail document within 4 KiB with no NUL.".into());
        }
        let target = match &self.selected {
            Document::Aliases => &mut self.documents.aliases,
            Document::Forward(user) => self
                .documents
                .forwards
                .get_mut(user)
                .ok_or("Choose one of the supplied forwarding files.")?,
        };
        if *target != text {
            *target = text;
            self.tested = None;
        }
        Ok(())
    }

    pub fn save_file(&mut self, task: &Task) -> Result<(), String> {
        self.bound(task)?;
        let mut saved = self.saved.clone();
        match &self.selected {
            Document::Aliases => saved.aliases = self.documents.aliases.clone(),
            Document::Forward(user) => {
                saved
                    .forwards
                    .insert(user.clone(), self.documents.forwards[user].clone());
            }
        }
        saved.saved_shape(&task.users)?;
        self.saved = saved;
        self.tested = None;
        Ok(())
    }

    pub fn rebuild_aliases(&mut self, task: &Task) -> Result<(), String> {
        self.bound(task)?;
        self.saved.directory(&task.users, &self.saved.aliases)?;
        self.index = self.saved.aliases.clone();
        self.tested = None;
        Ok(())
    }

    pub fn test(&mut self, task: &Task) -> Result<(), String> {
        self.bound(task)?;
        task.evaluate(&self.saved, &self.index)?;
        self.tested = Some(Snapshot {
            saved: self.saved.clone(),
            index: self.index.clone(),
        });
        Ok(())
    }

    pub fn checks(&self, task: &Task) -> Checks {
        if let Err(error) = self.bound(task) {
            return vec![(false, error)];
        }
        if self.tested.is_none() {
            return vec![(
                false,
                "Run the delivery test after your latest change.".into(),
            )];
        }
        let mut checks = vec![(
            self.documents == self.saved,
            "Save the edited virtual files before submitting the delivery result.".into(),
        )];
        match (
            task.evaluate(&self.saved, &self.index),
            task.evaluate(&self.saved, &self.saved.aliases),
        ) {
            (Ok(current), Ok(saved)) => {
                for (current, saved) in current.into_iter().zip(saved) {
                    checks.push((
                        current.0 && saved.0,
                        if !current.0 {
                            current.1
                        } else if !saved.0 {
                            format!("The saved aliases would fail after rebuilding. {}", saved.1)
                        } else {
                            current.1
                        },
                    ));
                }
            }
            (Err(error), _) | (_, Err(error)) => checks.push((false, error)),
        }
        checks
    }

    pub fn deliveries(&self, task: &Task) -> Result<Option<Vec<(String, core::Delivery)>>, String> {
        self.bound(task)?;
        if self.tested.is_none() {
            return Ok(None);
        }
        let directory = self.saved.directory(&task.users, &self.index)?;
        task.probes
            .iter()
            .map(|probe| {
                directory
                    .deliver(&probe.recipient)
                    .map(|delivery| (probe.label.clone(), delivery))
            })
            .collect::<Result<_, String>>()
            .map(Some)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    pub initial: Task,
    pub example: Task,
}

impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || self.initial.id == self.example.id
            || self
                .initial
                .users
                .iter()
                .any(|user| self.example.users.contains(user))
        {
            return Err("Use separate local users and tasks for practice and assignment.".into());
        }
        self.initial.validate()?;
        self.example.validate()
    }

    pub fn checks(&self, session: &Session) -> Checks {
        if let Err(error) = session.bound(self) {
            return vec![(false, error)];
        }
        session.assignment.checks(&self.initial)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "cron::text::<_,64>")]
    pub context: String,
    pub view: crate::script_teaching::View,
    pub assignment: Draft,
    pub practice: Draft,
}

impl Session {
    pub fn new(definition: &Definition) -> Self {
        Self {
            version: definition.version,
            context: definition.initial.id.clone(),
            view: crate::script_teaching::View::Introduction,
            assignment: Draft::new(&definition.initial),
            practice: Draft::new(&definition.example),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.context.is_empty() || self.context.len() > 64 {
            return Err("Saved local-mail activity metadata is incompatible.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }

    pub fn bound(&self, definition: &Definition) -> Result<(), String> {
        self.validate()?;
        if self.version != definition.version || self.context != definition.initial.id {
            return Err("This saved local-mail work belongs to another lesson.".into());
        }
        self.assignment.bound(&definition.initial)?;
        self.practice.bound(&definition.example)
    }

    pub fn execute_reference(
        &mut self,
        definition: &Definition,
        input: &str,
    ) -> Result<String, String> {
        self.bound(definition)?;
        let files = input
            .strip_prefix("LOCAL MAIL WORK\n")
            .ok_or("Use the virtual mail editor for this activity.")?;
        if files.len() > 64 * 1024 {
            return Err("The authored mail repair exceeds its input limit.".into());
        }
        let files: Files =
            serde_json::from_str(files).map_err(|_| "Invalid authored mail repair.")?;
        let mut next = self.assignment.clone();
        next.documents = files;
        next.tested = None;
        next.selected = Document::Aliases;
        next.save_file(&definition.initial)?;
        next.rebuild_aliases(&definition.initial)?;
        let owners = next.documents.forwards.keys().cloned().collect::<Vec<_>>();
        for owner in owners {
            next.selected = Document::Forward(owner);
            next.save_file(&definition.initial)?;
        }
        next.selected = definition.initial.focus.clone();
        next.test(&definition.initial)?;
        let checks = next.checks(&definition.initial);
        let passed = checks.iter().filter(|row| row.0).count();
        self.assignment = next;
        Ok(format!(
            "{passed}/{} local-mail requirements matched.",
            checks.len()
        ))
    }
}

fn admitted(scenario: &crate::pack::Scenario) -> Result<&Definition, String> {
    let (
        crate::engine::State::LocalMail { simulation },
        crate::engine::Goal::LocalMail { definition },
    ) = (&scenario.initial, &scenario.goal)
    else {
        return Err("The local-mail task does not match its supplied workspace.".into());
    };
    if scenario.track != "LPIC-1"
        || definition.initial.id != scenario.id
        || definition.initial.instructions != scenario.objective
        || **simulation != Session::new(definition)
    {
        return Err(
            "The mail task, track, instructions and initial editor state must match.".into(),
        );
    }
    Ok(definition)
}

pub fn source(scenario: &crate::pack::Scenario) -> Result<&Definition, String> {
    let definition = admitted(scenario)?;
    definition.validate()?;
    Ok(definition)
}

pub fn context(
    attempt: &crate::progress::Attempt,
    scenario: &crate::pack::Scenario,
) -> Result<(), String> {
    let definition = admitted(scenario)?;
    let crate::engine::State::LocalMail { simulation } = &attempt.state else {
        return Err("This saved mail work belongs to another activity.".into());
    };
    if attempt.scenario_version != scenario.version
        || attempt.ipv4.is_some()
        || attempt.workspace.as_ref().is_some_and(|workspace| {
            serde_json::to_value(workspace).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("This saved mail activity has incompatible progress metadata.".into());
    }
    simulation.bound(definition)
}
