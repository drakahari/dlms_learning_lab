//! Isolated practice for the existing single-service configuration/terminal exercise.
use crate::{
    engine::{self, Goal, State},
    pack::{self, CommandExample},
    progress::Attempt,
    script_teaching::{Concept, View},
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lesson {
    pub prerequisite_note: String,
    pub concepts: Vec<Concept>,
    pub example_description: String,
    pub required_port: u16,
    pub fixture: State,
    pub commands: Vec<CommandExample>,
    pub try_change: String,
}
impl Lesson {
    pub fn goal(&self) -> Goal {
        Goal::Service {
            port: self.required_port,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.required_port == 0
            || !matches!(self.fixture, State::Service { .. })
            || pack::validate_state(&self.fixture).is_err()
            || engine::solved(&self.fixture, &self.goal())
            || self.concepts.is_empty()
            || self.concepts.len() > 6
            || self.concepts.iter().any(|c| {
                c.title.trim().is_empty()
                    || c.title.len() > 100
                    || c.text.trim().is_empty()
                    || c.text.len() > 1200
            })
            || [
                &self.prerequisite_note,
                &self.example_description,
                &self.try_change,
            ]
            .iter()
            .any(|s| s.trim().is_empty() || s.len() > 1600)
            || self.commands.is_empty()
            || self.commands.len() > 12
            || self.commands.iter().any(|c| {
                c.command.is_empty()
                    || c.command.len() > 256
                    || c.explanation.trim().is_empty()
                    || c.explanation.len() > 600
                    || c.simulator_only
                    || !real_command(&c.command)
            })
        {
            return Err("Invalid single-service teaching content".into());
        }
        // Only non-mutating, genuinely supported inspection or service-management examples.
        let mut state = self.fixture.clone();
        engine::edit_service_port(&mut state, &self.required_port.to_string())?;
        for command in &self.commands {
            engine::execute(&mut state, &command.command)?;
        }
        engine::execute(&mut state, "systemctl restart atlas")?;
        engine::execute(&mut state, "systemctl enable atlas")?;
        if !engine::solved(&state, &self.goal()) {
            return Err("Service teaching example is unreachable".into());
        }
        Ok(())
    }
}
fn real_command(command: &str) -> bool {
    let parts: Vec<_> = command.split_ascii_whitespace().collect();
    match parts.as_slice() {
        ["cat", "/etc/atlas.conf"] | ["journalctl", "-u", "atlas"] => true,
        ["systemctl", action, "atlas"] => matches!(
            *action,
            "status"
                | "start"
                | "restart"
                | "stop"
                | "enable"
                | "disable"
                | "is-active"
                | "is-enabled"
        ),
        _ => false,
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Learning {
    pub version: u32,
    pub view: View,
    pub state: State,
    pub port_draft: String,
    pub command: String,
    pub transcript: Vec<String>,
    pub checked: Option<State>,
    pub command_index: usize,
}
impl Learning {
    pub fn new(lesson: &Lesson) -> Self {
        let State::Service { port, .. } = &lesson.fixture else {
            unreachable!()
        };
        Self {
            version: 1,
            view: View::Introduction,
            state: lesson.fixture.clone(),
            port_draft: port.clone(),
            command: String::new(),
            transcript: Vec::new(),
            checked: None,
            command_index: 0,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && matches!(self.state, State::Service { .. })
            && pack::validate_state(&self.state).is_ok()
            && self.port_draft.len() <= 256
            && self.command.len() <= 4096
            && self.transcript.len() <= 80
            && self.transcript.iter().all(|s| s.len() <= 8192)
            && self.command_index < 12
            && self.checked.as_ref().is_none_or(|s| {
                matches!(s, State::Service { .. }) && pack::validate_state(s).is_ok()
            })
    }
    pub fn save_port(&mut self) -> Result<(), String> {
        engine::edit_service_port(&mut self.state, self.port_draft.trim())?;
        if let State::Service { port, .. } = &self.state {
            self.port_draft = port.clone();
        }
        Ok(())
    }
    pub fn submit(&mut self) -> Result<(), String> {
        if self.command.len() > 4096 {
            return Err("Command exceeds the simulator’s 4,096-byte limit".into());
        }
        let command = std::mem::take(&mut self.command);
        if command.trim().is_empty() {
            return Ok(());
        }
        let old_port = if let State::Service { port, .. } = &self.state {
            Some(port.clone())
        } else {
            None
        };
        let result = engine::execute(&mut self.state, &command);
        if let (Some(old), State::Service { port, .. }) = (old_port, &self.state)
            && self.port_draft == old
        {
            self.port_draft = port.clone();
        }
        self.record(format!("practice # {command}"));
        match &result {
            Ok(output) if !output.is_empty() => self.record(output.clone()),
            Err(message) => self.record(format!("Command failed · {message}")),
            _ => {}
        }
        result.map(|_| ())
    }
    fn record(&mut self, mut line: String) {
        if line.len() > 8192 {
            const MARKER: &str = "[Earlier output omitted by the simulator’s 8 KiB entry limit]\n";
            let mut start = line.len() - (8192 - MARKER.len());
            while !line.is_char_boundary(start) {
                start += 1;
            }
            line = format!("{MARKER}{}", &line[start..]);
        }
        self.transcript.push(line);
        if self.transcript.len() > 80 {
            self.transcript.remove(0);
        }
    }
    pub fn has_unsaved_port(&self) -> bool {
        matches!(&self.state,State::Service{port,..}if port!=&self.port_draft)
    }
    pub fn check(&mut self) {
        self.checked = Some(self.state.clone());
    }
    pub fn checks(&self, lesson: &Lesson) -> Vec<(bool, String)> {
        if self.checked.as_ref() != Some(&self.state) || self.has_unsaved_port() {
            return Vec::new();
        }
        engine::checks(&self.state, &lesson.goal())
    }
    pub fn solved(&self, lesson: &Lesson) -> bool {
        let c = self.checks(lesson);
        !c.is_empty() && c.iter().all(|c| c.0)
    }
}
pub fn prepare(a: &mut Attempt, lesson: &Lesson, new_attempt: bool) {
    if crate::workspace::is_etherchannel_workspace(a) || crate::workspace::is_cron_workspace(a) {
        return;
    }
    if crate::workspace::has_nfs_metadata(a) || crate::workspace::has_cron_metadata(a) {
        return;
    }
    let w = a.workspace.as_mut().expect("workspace prepared");
    if w.service_learning.is_none() {
        let mut l = Learning::new(lesson);
        if !new_attempt {
            l.view = View::Assignment;
        }
        w.service_learning = Some(l);
    }
}
