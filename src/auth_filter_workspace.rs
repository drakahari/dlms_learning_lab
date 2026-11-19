//! Source-bound native editor and virtual command history. No host operation.
use crate::{
    auth_filter_task::{self as task, Check, Task},
    log_filter as core,
    script_teaching::View,
};
use serde::{Deserialize, Serialize};
type Result<T> = std::result::Result<T, String>;
const CONSOLE: usize = 32 * 1024;
fn bounded<T: Serialize>(v: &T, max: usize) -> bool {
    serde_json::to_vec(v).is_ok_and(|x| x.len() <= max)
}
fn text(s: &str, max: usize) -> bool {
    s.len() <= max && !s.contains('\0')
}
fn context(t: &Task) -> Result<String> {
    serde_json::to_string(t).map_err(|e| e.to_string())
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Run {
    pub case: usize,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub filter: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Event {
    Save {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Run {
        case: usize,
        #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
        input: String,
    },
    Check,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Action {
    Edit {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
        input: String,
    },
    Select {
        case: usize,
    },
    Save,
    Run,
    Check,
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
    #[serde(deserialize_with = "crate::cron::text::<_,1024>")]
    pub command: String,
    pub selected: usize,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub console: String,
    pub last_run: Option<Run>,
    pub checked: bool,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Event,64>")]
    pub history: Vec<Event>,
}
fn observed_name(value: core::Disposition) -> &'static str {
    match value {
        core::Disposition::Matched => "matched",
        core::Disposition::Ignored => "ignored",
        core::Disposition::Missed => "missed",
    }
}
fn command(t: &Task, saved: &str, case: usize, input: &str) -> Result<(String, Option<Run>)> {
    if !text(input, 1024) || (!input.is_ascii() || input.bytes().any(|b| b.is_ascii_control())) {
        return Err("Use one ASCII command within 1 KiB.".into());
    }
    if input.bytes().any(|b| b";|&<>$`\"'\\".contains(&b)) {
        return Err("Unsupported shell syntax. This virtual command bar accepts only the documented commands and unquoted declared paths; it is not a shell.".into());
    }
    let source = t.cases.get(case).ok_or("Choose a supplied log sample.")?;
    let words: Vec<_> = input.split_whitespace().collect();
    if let ["cat", path] = words.as_slice() {
        return match *path {
            p if p == t.filter_path => Ok((saved.to_owned(), None)),
            p if p == t.log_path => Ok((source.lines.join("\n") + "\n", None)),
            _ => Err(
                "Only the two declared virtual files exist in this activity; no host path is read."
                    .into(),
            ),
        };
    }
    if words.first() != Some(&"fail2ban-regex") {
        return Err("Unsupported command. Use cat FILE or fail2ban-regex --usedns=no LOG FILTER, optionally with --print-all-matched/--print-all-missed/--print-all-ignored. No daemon or firewall command runs.".into());
    }
    let mut paths = Vec::new();
    let mut dns = false;
    let mut print = [false; 3];
    let mut at = 1;
    while at < words.len() {
        match words[at] {
            "--usedns=no" if !dns=>dns=true,
            "--usedns" if !dns&&words.get(at+1)==Some(&"no")=>{dns=true;at+=1;},
            "--print-all-matched" if !print[0]=>print[0]=true,
            "--print-all-ignored" if !print[1]=>print[1]=true,
            "--print-all-missed" if !print[2]=>print[2]=true,
            s if s.starts_with('-')=>return Err("Unsupported or repeated option. This offline subset uses --usedns=no and optional print-all result flags only.".into()),
            s=>paths.push(s),
        }
        at += 1;
    }
    if !dns || paths != [t.log_path.as_str(), t.filter_path.as_str()] {
        return Err("Use fail2ban-regex --usedns=no with this activity’s declared LOG path followed by FILTER path. Other files, inline patterns, options and name lookup are not modeled.".into());
    }
    let filter = core::Filter::parse(saved).map_err(task::fault)?;
    let rows = core::inspect(&filter, &source.lines).map_err(task::fault)?;
    let count = |kind| rows.iter().filter(|r| r.disposition == kind).count();
    let mut output = format!(
        "Simulated fail2ban-regex · offline numeric-IPv4 filter test\nLog: {} · sample: {}\nFilter: {}\n\nResults\n=======\nFailregex: {} total\nIgnoreregex: {} total\nLines: {} lines, {} ignored, {} matched, {} missed\n",
        t.log_path,
        source.label,
        t.filter_path,
        count(core::Disposition::Matched),
        count(core::Disposition::Ignored),
        rows.len(),
        count(core::Disposition::Ignored),
        count(core::Disposition::Matched),
        count(core::Disposition::Missed)
    );
    for (show, kind) in print.into_iter().zip([
        core::Disposition::Matched,
        core::Disposition::Ignored,
        core::Disposition::Missed,
    ]) {
        if show {
            output.push_str(&format!("\n{} lines:\n", observed_name(kind)));
            for row in rows.iter().filter(|r| r.disposition == kind) {
                output.push_str(&format!("{}: {}\n", row.line, row.text));
            }
        }
    }
    Ok((
        output,
        Some(Run {
            case,
            filter: saved.to_owned(),
        }),
    ))
}
impl Scope {
    fn seed(t: &Task) -> Result<Self> {
        t.validate()?;
        Ok(Self {
            source_task: context(t)?,
            saved: t.initial_filter.clone(),
            editor: t.initial_filter.clone(),
            command: format!(
                "fail2ban-regex --usedns=no {} {}",
                t.log_path, t.filter_path
            ),
            selected: 0,
            console: String::new(),
            last_run: None,
            checked: false,
            history: Vec::new(),
        })
    }
    fn event(&mut self, t: &Task, event: &Event) -> Result<()> {
        match event {
            Event::Save { text: body } => {
                if !text(body, core::MAX_FILTER) {
                    return Err("Keep filter text within 2 KiB without NUL.".into());
                }
                if self.saved != *body {
                    self.saved = body.clone();
                    self.checked = false;
                    self.last_run = None;
                }
            }
            Event::Run { case, input } => {
                if *case >= t.cases.len() {
                    return Err("Choose a supplied log sample.".into());
                }
                let (result, run) = match command(t, &self.saved, *case, input) {
                    Ok(v) => v,
                    Err(e) => (format!("{e}\n"), None),
                };
                let output = format!(
                    "{}$ {}\n{}{}",
                    self.console,
                    input,
                    result,
                    if result.ends_with('\n') { "" } else { "\n" }
                );
                if output.len() > CONSOLE {
                    return Err("This virtual console reached 32 KiB; the command was not applied. Export progress before resetting this selected activity if you need to retain its history.".into());
                }
                self.console = output;
                if let Some(run) = run {
                    self.last_run = Some(run);
                }
            }
            Event::Check => self.checked = true,
        }
        Ok(())
    }
    fn bound(&self, t: &Task) -> Result<()> {
        if self.source_task != context(t)?
            || !bounded(self, 256 * 1024)
            || !text(&self.editor, 2048)
            || !text(&self.command, 1024)
            || self.selected >= t.cases.len()
            || self.history.len() > 64
        {
            return Err("Saved filter work does not match its source task or simulation limits; retain the original progress.".into());
        }
        let mut replay = Self::seed(t)?;
        for event in &self.history {
            replay.event(t, event)?;
        }
        if (
            self.saved.as_str(),
            self.console.as_str(),
            &self.last_run,
            self.checked,
        ) != (
            replay.saved.as_str(),
            replay.console.as_str(),
            &replay.last_run,
            replay.checked,
        ) {
            return Err("Saved filter observations differ from bounded history replay; retain the original progress.".into());
        }
        Ok(())
    }
    pub fn apply(&mut self, t: &Task, action: &Action) -> Result<()> {
        self.bound(t)?;
        let mut next = self.clone();
        let event = match action {
            Action::Edit { text: body } => {
                if !text(body, 2048) {
                    return Err("Keep the filter editor within 2 KiB without NUL.".into());
                }
                if next.editor != *body {
                    next.editor = body.clone();
                }
                None
            }
            Action::Command { input } => {
                if !text(input, 1024) {
                    return Err("Keep the unrun virtual command within 1 KiB without NUL.".into());
                }
                next.command = input.clone();
                None
            }
            Action::Select { case } => {
                if *case >= t.cases.len() {
                    return Err("Choose a supplied log sample.".into());
                }
                next.selected = *case;
                None
            }
            Action::Save => Some(Event::Save {
                text: next.editor.clone(),
            }),
            Action::Run => Some(Event::Run {
                case: next.selected,
                input: next.command.clone(),
            }),
            Action::Check => Some(Event::Check),
        };
        if let Some(event) = event {
            if next.history.len() >= 64 {
                return Err("This activity reached 64 recorded actions; no new action was applied. Export progress before resetting this selected activity.".into());
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
        if !self.checked {
            return vec![Check {
                passed: false,
                label: "Unchecked filter".into(),
                detail: "Save your filter, then Check work to evaluate every supplied test sample."
                    .into(),
            }];
        }
        if self.editor != self.saved {
            return vec![Check{passed:false,label:"Unsaved filter changes".into(),detail:"Save virtual filter before checking. Test commands read the saved text; your editor draft is still preserved.".into()}];
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
            || self.assignment.log_path == self.example.log_path
            || self.assignment.filter_path == self.example.filter_path
        {
            return Err("Use a source-owned assignment with separate example identity, virtual log and filter paths.".into());
        }
        let paths = [
            &self.assignment.log_path,
            &self.assignment.filter_path,
            &self.example.log_path,
            &self.example.filter_path,
        ];
        if paths
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != 4
            || self.worked.is_empty()
            || self.worked.len() > 8
        {
            return Err("Use four distinct virtual paths and a bounded worked example.".into());
        }
        let mut example = Scope::seed(&self.example)?;
        for action in &self.worked {
            example.apply(&self.example, action)?;
        }
        if !example.checks(&self.example).iter().all(|c| c.passed) {
            return Err(
                "The authored worked example must reach every stated example outcome.".into(),
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
    /// Shape/replay checking for bounded import. Only the authored catalog may
    /// establish current objective credit; embedded task text is not authority.
    pub fn shape(&self) -> Result<()> {
        if !bounded(self, 512 * 1024)
            || self.assignment.source_task.len() > 65536
            || self.example.source_task.len() > 65536
        {
            return Err("Keep saved filter work within its declared bounds.".into());
        }
        let assignment: Task = serde_json::from_str(&self.assignment.source_task)
            .map_err(|e| format!("Malformed saved filter context: {e}"))?;
        let example: Task = serde_json::from_str(&self.example.source_task)
            .map_err(|e| format!("Malformed saved example context: {e}"))?;
        assignment.validate()?;
        example.validate()?;
        if assignment.id == example.id
            || assignment.log_path == example.log_path
            || assignment.filter_path == example.filter_path
        {
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
