//! Source-bound local-notice drafts; results derive from replay, never cached strings.
use crate::{maintenance_notice as core, script_teaching::View};
use core::{Action, Document, Facts, World, bulletin::Bulletin};
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub id: String,
    pub facts: Facts,
    pub required: Bulletin,
    pub requires_restart: bool,
    pub minimum_lead: u32,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,8>")]
    pub starter: Vec<Action>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,64>")]
    pub reference: Vec<Action>,
}
fn ident(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub label: String,
    pub correct: bool,
    pub feedback: String,
}
fn bulletin_check(label: String, raw: &str, expected: &Bulletin) -> Check {
    match Bulletin::parse(raw) {
        Err(e) => Check {
            label,
            correct: false,
            feedback: e,
        },
        Ok(actual) => {
            let errors = actual.differences(expected);
            Check {
                label,
                correct: errors.is_empty(),
                feedback: if errors.is_empty() {
                    "The service, maintenance window and contact match the request.".into()
                } else {
                    errors.join(" ")
                },
            }
        }
    }
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        self.facts.validate()?;
        self.required.validate()?;
        if !ident(&self.id)
            || self.starter.len() > 8
            || self.reference.is_empty()
            || self.reference.len() > 64
            || self.required.starts <= self.facts.minute
            || self.requires_restart && !(6..=180).contains(&self.minimum_lead)
            || !self.requires_restart && self.minimum_lead != 0
            || self.requires_restart && self.required.starts - self.facts.minute < self.minimum_lead
        {
            return Err("Supply a bounded, future same-day maintenance request and an attainable notice period.".into());
        }
        let initial = World::replay(&self.facts, &self.starter)?;
        if self.check_world(&initial).iter().all(|c| c.correct) {
            return Err(
                "The independent notice starter must retain meaningful unfinished work.".into(),
            );
        }
        let mut history = self.starter.clone();
        history.extend(self.reference.clone());
        let reference = World::replay(&self.facts, &history)?;
        if !self.check_world(&reference).iter().all(|c| c.correct) {
            return Err("The declared notice method does not satisfy every recipient, file and timing requirement.".into());
        }
        Ok(())
    }
    pub fn check_world(&self, world: &World) -> Vec<Check> {
        let mut checks = vec![
            bulletin_check(
                "Future successful logins · /etc/motd".into(),
                &world.files.motd,
                &self.required,
            ),
            Check {
                label: "Pre-login identification · /etc/issue".into(),
                correct: world.files.issue == self.facts.initial.issue,
                feedback: if world.files.issue == self.facts.initial.issue {
                    "The existing pre-login notice is preserved.".into()
                } else {
                    "The request preserves the original pre-login notice; changing motd must not replace issue.".into()
                },
            },
        ];
        for login in &self.facts.logins {
            let label = format!("Current session · {} on {}", login.user, login.tty);
            let last = world
                .deliveries
                .iter()
                .rev()
                .find(|event| event.terminals.contains(&login.tty));
            checks.push(match last {
                None=>Check {label,correct:false,feedback:"No live notice reached this terminal. Saving a login message does not notify an already logged-in user.".into()},
                Some(event)=>bulletin_check(label,&event.body,&self.required),
            });
        }
        let (correct, feedback) = if world.stopped {
            (false,"The recorded host already reached its virtual restart; this assignment needs a verified plan before that event.".into())
        } else if self.requires_restart {
            match &world.restart {
                None=>(false,"No restart is currently scheduled. Cancelling the inherited request is only part of the handover.".into()),
                Some(restart) if restart.at!=self.required.starts=>(false,"The pending restart does not match the approved notice's start. Check the recording clock and relative delay.".into()),
                Some(restart) if restart.at.saturating_sub(world.minute)<self.minimum_lead=>(false,format!("Fewer than {} simulated minutes remain. The handover requires that notice period before the restart.",self.minimum_lead)),
                Some(_)=>(true,"The pending restart matches the approved window and retains the required notice period.".into()),
            }
        } else if world.restart.is_some() {
            (false,"This notice-only task does not authorize a restart. Cancel the simulated schedule and deliver the requested notice.".into())
        } else if world.minute >= self.required.starts {
            (false, "The announced start has passed. This task checks the notice before that window; reset this attempt to try again.".into())
        } else {
            (
                true,
                "The notice is in place before the window and no unrequested restart is pending."
                    .into(),
            )
        };
        checks.push(Check {
            label: "Current maintenance plan".into(),
            correct,
            feedback,
        });
        checks
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Editor {
    Issue,
    Motd,
    Stdin,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecord {
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub command: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub stdin: String,
    pub before: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub task: String,
    #[serde(default, deserialize_with = "crate::cron::text::<_,1024>")]
    pub command: String,
    #[serde(default)]
    pub last_run: Option<RunRecord>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,64>")]
    pub history: Vec<Action>,
    pub selected: Editor,
    pub documents: core::Files,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub stdin: String,
    pub stdin_pending: bool,
    pub checked: bool,
}
impl Draft {
    pub fn new(task: &Task) -> Result<Self> {
        task.validate()?;
        let world = World::replay(&task.facts, &task.starter)?;
        Ok(Self {
            task: task.id.clone(),
            command: String::new(),
            last_run: None,
            history: task.starter.clone(),
            selected: Editor::Motd,
            documents: world.files,
            stdin: String::new(),
            stdin_pending: false,
            checked: false,
        })
    }
    pub fn world(&self, task: &Task) -> Result<World> {
        task.validate()?;
        if self.task != task.id
            || self.history.len() > 64
            || !self.history.starts_with(&task.starter)
            || self.stdin.len() > 4096
            || self.stdin.contains('\0')
            || self.documents.issue.len() > 4096
            || self.documents.motd.len() > 4096
            || self.documents.issue.contains('\0')
            || self.documents.motd.contains('\0')
        {
            return Err("This saved notice draft is outside its source task or size bounds; retain the original progress file.".into());
        }
        if !self.stdin_pending && !self.stdin.is_empty() {
            let sent = self.history.iter().rev().find_map(|action| {
                if let Action::Broadcast { text } = action {
                    Some(text)
                } else {
                    None
                }
            });
            if sent != Some(&self.stdin) {
                return Err("Saved input is marked sent but does not match its actual broadcast history; keep the original progress file.".into());
            }
        }
        if self.command.len() > 1024 || self.command.contains('\0') {
            return Err("Saved command draft exceeds its literal terminal bounds.".into());
        }
        if let Some(record) = &self.last_run {
            if record.before < task.starter.len()
                || record.before > self.history.len()
                || record.command.len() > 1024
                || record.command.contains('\0')
                || record.stdin.len() > 4096
                || record.stdin.contains('\0')
            {
                return Err("Saved terminal observation does not fit this notice history.".into());
            }
            let prior = World::replay(&task.facts, &self.history[..record.before])?;
            if let Ok(output) =
                core::command::run(&task.facts, &prior, &record.command, &record.stdin)
                && let Some(action) = output.action
                && self.history.get(record.before) != Some(&action)
            {
                return Err("Saved terminal observation claims an action absent from the actual notice history.".into());
            }
        }
        World::replay(&task.facts, &self.history)
    }
    pub fn text(&self) -> &str {
        match self.selected {
            Editor::Issue => &self.documents.issue,
            Editor::Motd => &self.documents.motd,
            Editor::Stdin => &self.stdin,
        }
    }
    pub fn edit(&mut self, value: String) -> Result<()> {
        if value.len() > 4096 || value.contains('\0') {
            return Err("Keep this simulated notice draft within 4 KiB with no NUL.".into());
        }
        let current = match self.selected {
            Editor::Issue => &mut self.documents.issue,
            Editor::Motd => &mut self.documents.motd,
            Editor::Stdin => &mut self.stdin,
        };
        if *current != value {
            *current = value;
            self.checked = false;
            if self.selected == Editor::Stdin {
                self.stdin_pending = true;
            }
        }
        Ok(())
    }
    pub fn discard_stdin(&mut self) {
        self.stdin.clear();
        self.stdin_pending = false;
        self.checked = false;
    }
    fn commit(&mut self, task: &Task, action: Action) -> Result<()> {
        let mut world = self.world(task)?;
        if self.history.len() == 64 {
            return Err(
                "The bounded action history is full; reset only this attempt to explore again."
                    .into(),
            );
        }
        world.apply(&task.facts, &action)?;
        match &action {
            Action::Save {
                document: Document::Issue,
                text,
            } => self.documents.issue = text.clone(),
            Action::Save {
                document: Document::Motd,
                text,
            } => self.documents.motd = text.clone(),
            Action::Broadcast { text } => {
                self.stdin = text.clone();
                self.stdin_pending = false;
            }
            _ => {}
        }
        self.history.push(action);
        self.checked = false;
        Ok(())
    }
    pub fn save(&mut self, task: &Task) -> Result<()> {
        let document=match self.selected {Editor::Issue=>Document::Issue,Editor::Motd=>Document::Motd,Editor::Stdin=>return Err("The live composer is simulated stdin. Run wall to send it; it is not a saved login file.".into())};
        self.commit(
            task,
            Action::Save {
                document,
                text: self.text().into(),
            },
        )
    }
    pub fn run(
        &mut self,
        task: &Task,
        raw: &str,
    ) -> std::result::Result<String, core::command::Diagnostic> {
        use core::command::{Diagnostic, Kind};
        let world = self.world(task).map_err(|message| Diagnostic {
            kind: Kind::State,
            message,
        })?;
        let output = core::command::run(&task.facts, &world, raw, &self.stdin)?;
        if let Some(action) = output.action {
            self.commit(task, action).map_err(|message| Diagnostic {
                kind: Kind::State,
                message,
            })?;
        }
        Ok(output.stdout)
    }
    pub fn run_and_record(
        &mut self,
        task: &Task,
    ) -> std::result::Result<String, core::command::Diagnostic> {
        let record = RunRecord {
            command: self.command.clone(),
            stdin: self.stdin.clone(),
            before: self.history.len(),
        };
        let output = self.run(task, &record.command);
        let reproducible = output.is_ok()
            || self.world(task).ok().is_some_and(|world| {
                core::command::run(&task.facts, &world, &record.command, &record.stdin).is_err()
            });
        self.last_run = reproducible.then_some(record);
        output
    }
    pub fn last_output(
        &self,
        task: &Task,
    ) -> Option<std::result::Result<String, core::command::Diagnostic>> {
        let r = self.last_run.as_ref()?;
        let prior = World::replay(&task.facts, self.history.get(..r.before)?).ok()?;
        Some(core::command::run(&task.facts, &prior, &r.command, &r.stdin).map(|o| o.stdout))
    }
    pub fn apply_action(&mut self, task: &Task, action: Action) -> Result<()> {
        self.commit(task, action)
    }
    pub fn advance(&mut self, task: &Task, minutes: u32) -> Result<()> {
        self.commit(task, Action::Advance { minutes })
    }
    pub fn checks(&self, task: &Task) -> Result<Vec<Check>> {
        let world = self.world(task)?;
        let mut checks = task.check_world(&world);
        let pending = self.documents != world.files || self.stdin_pending;
        checks.push(Check{label:"Saved files and sent message".into(),correct:!pending,feedback:if pending{"Save changed login files. Send the edited live input with wall, or discard that unsent input before checking.".into()}else{"No unsaved notice changes or unsent input remain.".into()}});
        Ok(checks)
    }
    pub fn check(&mut self, task: &Task) -> Result<()> {
        self.checks(task)?;
        self.checked = true;
        Ok(())
    }
    pub fn solved(&self, task: &Task) -> bool {
        self.checked && self.checks(task).is_ok_and(|c| c.iter().all(|c| c.correct))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub initial: Task,
    pub example: Task,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate()?;
        self.example.validate()?;
        if self.initial.id == self.example.id
            || self.initial.facts == self.example.facts
            || self.initial.required == self.example.required
        {
            return Err(
                "Use different worked data and a distinct source-bound practice task.".into(),
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub view: View,
    pub assignment: Draft,
    pub practice: Draft,
}
impl Session {
    pub fn new(def: &Definition) -> Result<Self> {
        def.validate()?;
        Ok(Self {
            version: 1,
            view: View::Introduction,
            assignment: Draft::new(&def.initial)?,
            practice: Draft::new(&def.example)?,
        })
    }
    pub fn validate(&self, def: &Definition) -> Result<()> {
        self.shape()?;
        if self.version != 1 {
            return Err(
                "Unsupported notice progress version; keep the original for recovery.".into(),
            );
        }
        def.validate()?;
        self.assignment.world(&def.initial)?;
        self.practice.world(&def.example)?;
        Ok(())
    }
    pub fn reset_assignment(&mut self, def: &Definition) -> Result<()> {
        self.assignment = Draft::new(&def.initial)?;
        Ok(())
    }
    pub fn reset_practice(&mut self, def: &Definition) -> Result<()> {
        self.practice = Draft::new(&def.example)?;
        Ok(())
    }
    pub fn solved(&self, def: &Definition) -> bool {
        self.validate(def).is_ok() && self.assignment.solved(&def.initial)
    }
}

pub const HELP: &str = core::command::HELP;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Reference {
    Apply { action: Action },
    Check,
}
impl Definition {
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = s.validate(self) {
            return vec![(false, e)];
        }
        let (d, t) = if practice {
            (&s.practice, &self.example)
        } else {
            (&s.assignment, &self.initial)
        };
        if !d.checked {
            return vec![(
                false,
                "Not checked. Save changed files, send the live message and choose Check notices."
                    .into(),
            )];
        }
        match d.checks(t) {
            Ok(checks) => checks
                .into_iter()
                .map(|c| (c.correct, format!("{}: {}", c.label, c.feedback)))
                .collect(),
            Err(e) => vec![(false, e)],
        }
    }
}
impl Session {
    pub fn shape(&self) -> Result<()> {
        if self.version != 1 {
            return Err("Unsupported notice progress version; original retained.".into());
        }
        for d in [&self.assignment, &self.practice] {
            if !ident(&d.task)
                || d.history.len() > 64
                || d.documents.issue.len() > 4096
                || d.documents.motd.len() > 4096
                || d.stdin.len() > 4096
                || [&d.documents.issue, &d.documents.motd, &d.stdin]
                    .iter()
                    .any(|s| s.contains('\0'))
            {
                return Err("Saved notice drafts exceed their source-independent bounds.".into());
            }
            if d.command.len() > 1024
                || d.command.contains('\0')
                || d.last_run.as_ref().is_some_and(|r| {
                    r.before > d.history.len()
                        || r.command.len() > 1024
                        || r.stdin.len() > 4096
                        || r.command.contains('\0')
                        || r.stdin.contains('\0')
                })
            {
                return Err("Saved terminal draft or observation exceeds its bounds.".into());
            }
            for action in &d.history {
                let body = match action {
                    Action::Save { text, .. }
                    | Action::Broadcast { text }
                    | Action::Schedule { text, .. }
                    | Action::Cancel { text } => Some(text),
                    Action::Advance { .. } => None,
                };
                if body.is_some_and(|text| text.len() > 4096 || text.contains('\0')) {
                    return Err("Saved notice action exceeds the plain-text limit.".into());
                }
            }
        }
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 1024 * 1024 {
            return Err("Saved notice activity exceeds1MiB.".into());
        }
        Ok(())
    }
    pub fn execute_reference(&mut self, def: &Definition, raw: &str) -> Result<String> {
        self.validate(def)?;
        let mut next = self.clone();
        if let Some(raw) = raw.strip_prefix("notice-action ") {
            if raw.len() > 8192 {
                return Err("Simulator notice reference exceeds its size bound.".into());
            }
            let action: Reference = serde_json::from_str(raw).map_err(
                |_| "Invalid simulator-only notice operation; this helper is not a Linux command.",
            )?;
            match action {
                Reference::Apply { action } => {
                    next.assignment.apply_action(&def.initial, action)?
                }
                Reference::Check => next.assignment.check(&def.initial)?,
            }
            next.validate(def)?;
            *self = next;
            Ok("Applied simulator-only notice operation.".into())
        } else {
            let output = next
                .assignment
                .run(&def.initial, raw)
                .map_err(|e| e.to_string())?;
            next.validate(def)?;
            *self = next;
            Ok(output)
        }
    }
}
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::MaintenanceNotice { simulation },
        crate::engine::Goal::MaintenanceNotice { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Notice state and objective must use the same authored source.".into());
    };
    definition.validate()?;
    if **simulation != Session::new(definition)? {
        return Err(
            "Notice work must start from separate declared assignment/practice drafts.".into(),
        );
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::MaintenanceNotice { .. })
        || matches!(s.initial, crate::engine::State::MaintenanceNotice { .. })
        || matches!(s.goal, crate::engine::Goal::MaintenanceNotice { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let def = source(s)?;
    let crate::engine::State::MaintenanceNotice { simulation } = &a.state else {
        return Err("Saved notice work belongs to another activity.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible notice metadata; preserve the original progress file.".into());
    }
    simulation.validate(def)
}
