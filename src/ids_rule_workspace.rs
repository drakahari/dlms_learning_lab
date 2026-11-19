//! Separate, replay-checked rule editor scopes. No host commands or packet I/O.
use crate::{
    ids_rule_task::{Check, Task},
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, String>;

fn bounded<T: Serialize>(value: &T, max: usize) -> bool {
    serde_json::to_vec(value).is_ok_and(|v| v.len() <= max)
}
fn text(value: &str) -> bool {
    value.len() <= 2048 && !value.contains('\0')
}
fn context(t: &Task) -> Result<String> {
    serde_json::to_string(t).map_err(|e| e.to_string())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Edit {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Select {
        case: usize,
    },
    Save,
    Replay,
    Check,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Save {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Replay {
        case: usize,
    },
    Check,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub case: usize,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub rule: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    #[serde(deserialize_with = "crate::cron::text::<_,65536>")]
    pub source_task: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub saved: String,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub editor: String,
    pub selected: usize,
    pub last_run: Option<Run>,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,64>")]
    pub history: Vec<Event>,
}
impl Scope {
    fn seed(t: &Task) -> Result<Self> {
        t.validate()?;
        Ok(Self {
            source_task: context(t)?,
            saved: t.initial_rule.clone(),
            editor: t.initial_rule.clone(),
            selected: 0,
            last_run: None,
            checked: false,
            history: vec![],
        })
    }
    fn event(&mut self, t: &Task, event: &Event) -> Result<()> {
        match event {
            Event::Save { text: body } => {
                if !text(body) {
                    return Err("Keep the rule file within 2 KiB without NUL.".into());
                }
                if self.saved != *body {
                    self.saved = body.clone();
                    self.checked = false;
                    self.last_run = None;
                }
            }
            Event::Replay { case } => {
                if *case >= t.cases.len() {
                    return Err("Choose a supplied record set.".into());
                }
                // Keep the tested input even on a parse error; the diagnostic is
                // derived afresh, never trusted from imported progress.
                self.last_run = Some(Run {
                    case: *case,
                    rule: self.saved.clone(),
                });
            }
            Event::Check => self.checked = true,
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<()> {
        if self.source_task != context(t)?
            || !bounded(self, 256 * 1024)
            || !text(&self.editor)
            || self.selected >= t.cases.len()
            || self.history.len() > 64
        {
            return Err("Saved rule work differs from its source task or simulation limits; retain the original progress.".into());
        }
        let mut replay = Self::seed(t)?;
        for event in &self.history {
            replay.event(t, event)?;
        }
        if (&self.saved, &self.last_run, self.checked)
            != (&replay.saved, &replay.last_run, replay.checked)
        {
            return Err("Saved rule observations differ from bounded history replay; retain the original progress.".into());
        }
        Ok(())
    }
    pub fn apply(&mut self, t: &Task, action: &Action) -> Result<()> {
        self.bound(t)?;
        let mut next = self.clone();
        let event = match action {
            Action::Edit { text: body } => {
                if !text(body) {
                    return Err("Keep the rule editor within 2 KiB without NUL.".into());
                }
                next.editor = body.clone();
                None
            }
            Action::Select { case } => {
                if *case >= t.cases.len() {
                    return Err("Choose a supplied record set.".into());
                }
                next.selected = *case;
                None
            }
            Action::Save => Some(Event::Save {
                text: next.editor.clone(),
            }),
            Action::Replay => Some(Event::Replay {
                case: next.selected,
            }),
            Action::Check => Some(Event::Check),
        };
        if let Some(event) = event {
            if next.history.len() >= 64 {
                return Err("This activity reached 64 recorded actions; no action was applied. Export progress before resetting this selected activity.".into());
            }
            next.event(t, &event)?;
            next.history.push(event);
        }
        next.bound(t)?;
        *self = next;
        Ok(())
    }
    pub fn checks(&self, t: &Task) -> Vec<Check> {
        if let Err(error) = self.bound(t) {
            return vec![Check {
                passed: false,
                label: "Saved context".into(),
                detail: error,
            }];
        }
        let message = if self.editor != self.saved {
            Some((
                "Unsaved rule changes",
                "Save the virtual rule before checking. Replay reads the saved file; your editor draft is preserved.",
            ))
        } else if !self.checked {
            Some((
                "Unchecked rule",
                "Save your rule, then Check answers to evaluate every supplied record set.",
            ))
        } else {
            None
        };
        if let Some((label, detail)) = message {
            return vec![Check {
                passed: false,
                label: label.into(),
                detail: detail.into(),
            }];
        }
        t.checks(&self.saved)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,8>")]
    pub worked: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        self.assignment.validate()?;
        self.example.validate()?;
        if self.source_id != self.assignment.id
            || self.assignment.id == self.example.id
            || self.assignment.rule_path == self.example.rule_path
            || self.worked.is_empty()
            || self.worked.len() > 8
        {
            return Err("Use separate assignment/example identities and virtual rule files, with a bounded reachable worked example.".into());
        }
        let mut example = Scope::seed(&self.example)?;
        for action in &self.worked {
            example.apply(&self.example, action)?;
        }
        if !example.checks(&self.example).iter().all(|c| c.passed) {
            return Err(
                "The authored worked example must reach every required example outcome.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        self.validate()?;
        s.assignment.bound(&self.assignment)?;
        s.example.bound(&self.example)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub view: View,
    pub assignment: Scope,
    pub example: Scope,
}
impl Session {
    /// Import shape is not source authority or assignment credit.
    pub fn shape(&self) -> Result<()> {
        if !bounded(self, 512 * 1024)
            || self.assignment.source_task.len() > 65536
            || self.example.source_task.len() > 65536
        {
            return Err("Keep saved rule work within its declared bounds.".into());
        }
        let assignment: Task = serde_json::from_str(&self.assignment.source_task)
            .map_err(|e| format!("Malformed saved rule task: {e}"))?;
        let example: Task = serde_json::from_str(&self.example.source_task)
            .map_err(|e| format!("Malformed saved rule example: {e}"))?;
        assignment.validate()?;
        example.validate()?;
        if assignment.id == example.id || assignment.rule_path == example.rule_path {
            return Err("Keep assignment and example contexts separate.".into());
        }
        self.assignment.bound(&assignment)?;
        self.example.bound(&example)
    }
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            view: View::Introduction,
            assignment: Scope::seed(&d.assignment)?,
            example: Scope::seed(&d.example)?,
        })
    }
    pub fn action(&mut self, d: &Definition, action: &Action, practice: bool) -> Result<()> {
        d.bound(self)?;
        if practice {
            self.example.apply(&d.example, action)
        } else {
            self.assignment.apply(&d.assignment, action)
        }
    }
    pub fn reset_example(&mut self, d: &Definition) -> Result<()> {
        d.bound(self)?;
        self.example = Scope::seed(&d.example)?;
        Ok(())
    }
}
