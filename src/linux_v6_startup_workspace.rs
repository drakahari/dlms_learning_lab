//! Separate, replay-validated practice and assignment drafts for the provided loader.
pub use crate::linux_v6_startup_task::Task;
use crate::{
    linux_v6_startup as core,
    linux_v6_workspace::{Check, Trial},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, String>;
const CONSOLE: usize = 128 * 1024;
fn within<T: Serialize>(value: &T, bytes: usize) -> bool {
    serde_json::to_vec(value).is_ok_and(|v| v.len() <= bytes)
}
fn text_ok(text: &str) -> bool {
    text.len() <= 32768
        && text
            .bytes()
            .all(|b| b.is_ascii_graphic() || matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum History {
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        input: String,
    },
    SaveFile {
        #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
        text: String,
    },
}
fn replay(world: &mut core::State, h: &History) -> Result<core::Output> {
    match h {
        History::Command { input } => world
            .command(input)
            .map_err(|e| format!("[{:?}] {}", e.class, e.message)),
        History::SaveFile { text } => {
            let previous = world.saved_filter.clone();
            world.save_draft(text).map_err(|e| e.message)?;
            Ok(core::Output {
                text:
                    "Saved the virtual file. Running rules and loader enablement are unchanged.\n"
                        .into(),
                exit: 0,
                changed: previous != *text,
            })
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Startup {
    pub world: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Check,64>")]
    pub checks: Vec<Check>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub world: core::State,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,History,96>")]
    pub history: Vec<History>,
    #[serde(deserialize_with = "crate::cron::text::<_,512>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,131072>")]
    pub output: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub editor: String,
    pub selected_case: usize,
    pub last_trial: Option<Trial>,
    pub last_startup: Option<Startup>,
    pub checked: bool,
}
fn log(s: &mut Scope, text: &str) {
    s.output.push_str(text);
    if s.output.len() > CONSOLE {
        let mut at = s.output.len() - CONSOLE;
        while !s.output.is_char_boundary(at) {
            at += 1;
        }
        s.output.drain(..at);
    }
}
impl Scope {
    pub fn new(t: &Task) -> Self {
        Self {
            world: t.initial.clone(),
            history: vec![],
            input: String::new(),
            output: String::new(),
            editor: t.initial.saved_filter.clone(),
            selected_case: 0,
            last_trial: None,
            last_startup: None,
            checked: false,
        }
    }
    pub fn shape(&self) -> Result<()> {
        self.world.validate().map_err(|e| e.message)?;
        if !within(self, 768 * 1024)
            || !within(&self.history, 256 * 1024)
            || self.history.len() > 96
            || self.input.len() > 512
            || self.output.len() > CONSOLE
            || !text_ok(&self.editor)
        {
            return Err(
                "Saved firewall work exceeds its limits; previous work is retained.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<()> {
        self.shape()?;
        t.bound(&self.world)?;
        if self.selected_case >= t.traffic.cases.len() {
            return Err("Choose a supplied traffic case.".into());
        }
        let mut world = t.initial.clone();
        for h in &self.history {
            replay(&mut world, h)?;
        }
        if world != self.world {
            return Err(
                "Saved firewall state differs from its supported edits; previous work is retained."
                    .into(),
            );
        }
        if let Some(trial) = &self.last_trial
            && *trial != t.trial(&self.world, self.selected_case)?
        {
            return Err("Saved traffic feedback no longer matches the current rules.".into());
        }
        if let Some(startup) = &self.last_startup {
            let (world, checks) = t.startup(&self.world)?;
            if *startup != (Startup { world, checks }) {
                return Err(
                    "Saved startup feedback no longer matches the file and loader settings.".into(),
                );
            }
        }
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Vec<Check> {
        if let Err(e) = self.bound(t) {
            return vec![Check {
                passed: false,
                label: "Saved workspace".into(),
                detail: e,
            }];
        }
        let mut checks = t.checks(&self.world);
        checks.push(Check{passed:self.editor==self.world.saved_filter,label:"Save the current file draft".into(),detail:"Unsaved editor changes are separate from the file the loader reads. Save your intended rules before checking the complete configuration.".into()});
        checks
    }
    pub fn apply(&mut self, t: &Task, a: &Action) -> Result<()> {
        self.apply_inner(t, a, false)
    }
    pub fn reference(&mut self, t: &Task, a: &Action) -> Result<()> {
        self.apply_inner(t, a, true)
    }
    fn apply_inner(&mut self, t: &Task, a: &Action, strict: bool) -> Result<()> {
        self.bound(t)?;
        let mut next = self.clone();
        match a {
            Action::Input { text } => {
                if text.len() > 512 {
                    return Err("Keep one command within 512 bytes.".into());
                }
                next.input = text.clone();
                next.checked = false;
            }
            Action::EditFile { text } => {
                if !text_ok(text) {
                    return Err(
                        "Keep this simulated rules file within 32 KiB of ASCII text.".into(),
                    );
                }
                next.editor = text.clone();
                next.checked = false;
                next.last_startup = None;
            }
            Action::ReloadEditor => {
                next.editor = next.world.saved_filter.clone();
                next.checked = false;
            }
            Action::SelectCase { case } => {
                if *case >= t.traffic.cases.len() {
                    return Err("Choose a supplied traffic case.".into());
                }
                next.selected_case = *case;
                next.last_trial = None;
            }
            Action::RunCase => {
                next.last_trial = Some(t.trial(&next.world, next.selected_case)?);
            }
            Action::RunStartup => {
                let (world, checks) = t.startup(&next.world)?;
                next.last_startup = Some(Startup { world, checks });
            }
            Action::Check => {
                next.checked = true;
            }
            Action::Command { .. } | Action::SaveFile => {
                let h = match a {
                    Action::Command { input } => History::Command {
                        input: input.clone(),
                    },
                    Action::SaveFile => History::SaveFile {
                        text: next.editor.clone(),
                    },
                    _ => unreachable!(),
                };
                let mut world = next.world.clone();
                next.checked = false;
                next.last_trial = None;
                next.last_startup = None;
                if let Action::Command { input } = a {
                    log(&mut next, &format!("$ {input}\n"));
                }
                match replay(&mut world, &h) {
                    Ok(out) => {
                        if strict && out.exit != 0 {
                            return Err(format!(
                                "Worked command exited {}: {}",
                                out.exit, out.text
                            ));
                        }
                        if world != next.world {
                            if next.history.len() == 96 {
                                return Err("This attempt reached 96 configuration changes. Its work is retained; no further change was applied.".into());
                            }
                            // A CLI save can refresh an unchanged editor, never erase an unsaved draft.
                            if next.editor == next.world.saved_filter {
                                next.editor = world.saved_filter.clone();
                            }
                            next.history.push(h);
                            next.world = world;
                        }
                        log(&mut next, &out.text);
                        if out.exit != 0 {
                            log(&mut next, &format!("[Exit status {}]\n", out.exit));
                        }
                        if matches!(a, Action::Command { .. }) {
                            next.input.clear();
                        }
                    }
                    Err(e) => {
                        if strict {
                            return Err(e);
                        }
                        log(&mut next, &format!("[Not applied] {e}\n"));
                        if let Action::Command { input } = a {
                            next.input = input.clone();
                        }
                    }
                }
            }
        }
        next.bound(t)?;
        *self = next;
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Input {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        text: String,
    },
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,512>")]
        input: String,
    },
    EditFile {
        #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
        text: String,
    },
    SaveFile,
    ReloadEditor,
    SelectCase {
        case: usize,
    },
    RunCase,
    RunStartup,
    Check,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,32>")]
    pub worked: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.assignment.validate()?;
        self.example.validate()?;
        if self.source_id.is_empty()
            || self.source_id.len() > 64
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || !within(self, 1536 * 1024)
            || self.worked.is_empty()
            || self.worked.len() > 32
            || self.assignment.id == self.example.id
            || self.assignment.initial.router.host.profile.ipv6
                == self.example.initial.router.host.profile.ipv6
        {
            return Err("Use distinct assignment and example networks, a stable source identifier and bounded worked actions.".into());
        }
        for t in [&self.assignment, &self.example] {
            if Scope::new(t).checks(t).iter().all(|c| c.passed) {
                return Err("Each firewall task needs a genuine repair.".into());
            }
        }
        let mut s = Scope::new(&self.example);
        for a in &self.worked {
            s.reference(&self.example, a)?;
        }
        if !s.checked || !s.checks(&self.example).iter().all(|c| c.passed) {
            return Err(
                "The worked actions must reach every example outcome and check the result.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        if s.source_id != self.source_id || !within(s, 1536 * 1024) {
            return Err("Saved firewall work belongs to another lesson or exceeds its bound; previous work is retained.".into());
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
    pub fn solved(&self, s: &Session) -> bool {
        self.bound(s).is_ok()
            && s.assignment.checked
            && s.assignment
                .checks(&self.assignment)
                .iter()
                .all(|c| c.passed)
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn shape(&self) -> Result<()> {
        if self.source_id.is_empty()
            || self.source_id.len() > 64
            || !self
                .source_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || !within(self, 1536 * 1024)
        {
            return Err("Invalid saved firewall lesson identity or size.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment),
            practice: Scope::new(&d.example),
        })
    }
    pub fn action(&mut self, d: &Definition, a: &Action, practice: bool) -> Result<()> {
        d.bound(self)?;
        let mut next = self.clone();
        if practice {
            next.practice.apply(&d.example, a)?;
        } else {
            next.assignment.apply(&d.assignment, a)?;
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<()> {
        d.bound(self)?;
        if practice {
            self.practice = Scope::new(&d.example);
        } else {
            self.assignment = Scope::new(&d.assignment);
        }
        Ok(())
    }
}
