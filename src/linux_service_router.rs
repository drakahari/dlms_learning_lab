//! Compose a virtual services editor with the existing bounded IPv6 router.
//! No existing scenario opts in implicitly; all files and commands stay in memory.
use crate::{
    linux_filter::{Class, Fault, Output},
    linux_named_ports, linux_v6_workspace as router,
    script_teaching::View,
    service_names as names,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
type Result<T> = std::result::Result<T, String>;
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn within<T: Serialize>(v: &T, max: usize) -> bool {
    serde_json::to_vec(v).is_ok_and(|s| s.len() <= max)
}
fn editor_text(s: &str) -> bool {
    s.len() <= 32768
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
}
fn meanings(s: &str) -> Result<BTreeMap<(String, String), u16>> {
    let mut out = BTreeMap::new();
    for r in names::Database::parse(s)
        .map_err(|e| e.to_string())?
        .records()
    {
        for name in std::iter::once(&r.name).chain(&r.aliases) {
            out.insert(
                (name.clone(), names::protocol_name(r.protocol).into()),
                r.port,
            );
        }
    }
    Ok(out)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub version: u8,
    pub router: router::Task,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub initial_services: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub required_services: String,
}
impl Task {
    pub fn validate(&self) -> Result<()> {
        self.router.validate()?;
        if self.version != 1 || !within(self, 512 * 1024) {
            return Err("Use a version1, bounded services/router task.".into());
        }
        if meanings(&self.initial_services)? == meanings(&self.required_services)? {
            return Err("This task must require a genuine service mapping repair.".into());
        }
        Ok(())
    }
    pub fn checks(&self, s: &Scope) -> Vec<router::Check> {
        if let Err(e) = s.bound(self) {
            return vec![router::Check {
                passed: false,
                label: "Saved workspace".into(),
                detail: e,
            }];
        }
        let mut checks=vec![router::Check{passed:s.editor==s.saved_services,label:"Save the services file".into(),detail:if s.editor==s.saved_services{"The current editor text is saved in the simulated file."}else{"The editor has unsaved changes. Save /etc/services; existing numeric rules will still need inspection."}.into()}];
        let required = match meanings(&self.required_services) {
            Ok(v) => v,
            Err(e) => {
                return vec![router::Check {
                    passed: false,
                    label: "Source mapping".into(),
                    detail: e,
                }];
            }
        };
        let saved = match meanings(&s.saved_services) {
            Ok(v) => v,
            Err(e) => {
                return vec![router::Check {
                    passed: false,
                    label: "Saved mapping syntax".into(),
                    detail: e,
                }];
            }
        };
        for ((name, protocol), expected) in &required {
            let key = (name.clone(), protocol.clone());
            let actual = saved.get(&key);
            checks.push(router::Check{passed:actual==Some(expected),label:format!("Saved mapping · {name}/{protocol}"),detail:if actual==Some(expected){format!("The saved name or alias maps to required {expected}/{protocol}.")}else{format!("Expected {name}/{protocol} to map to port {expected}; actual {}. Compare the supplied listener and saved protocol-specific entry.",actual.map_or("no matching entry".into(),|p|format!("port {p}")))}});
        }
        checks.push(router::Check{passed:saved.keys().all(|k|required.contains_key(k)),label:"Keep the supplied service names".into(),detail:"Preserve the required names and aliases; do not leave additional unintended mappings. Comments, spacing, entry order and equivalent canonical/alias choices may differ.".into()});
        checks.extend(self.router.checks(&s.router));
        checks
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub router: router::Scope,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub editor: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub saved_services: String,
}
fn log(s: &mut router::Scope, text: &str) {
    s.output.push_str(text);
    if s.output.len() > router::MAX_CONSOLE {
        let mut at = s.output.len() - router::MAX_CONSOLE;
        while !s.output.is_char_boundary(at) {
            at += 1;
        }
        s.output.drain(..at);
    }
}
fn fault(class: Class, message: impl Into<String>) -> Fault {
    Fault {
        class,
        message: message.into(),
    }
}
fn lookup_fault(e: names::Fault) -> Fault {
    let class = match e.kind {
        names::FaultKind::Syntax => Class::Syntax,
        names::FaultKind::Unsupported => Class::Unsupported,
        names::FaultKind::Limit => Class::Limit,
        names::FaultKind::Invalid => Class::Context,
    };
    fault(class, e.to_string())
}

/// A narrow callback seam for the reused terminal. The editor supplies saved text,
/// never an arbitrary host path. All non-command actions keep existing semantics.
pub fn router_action(
    s: &mut router::Scope,
    t: &router::Task,
    saved: &str,
    a: &router::Action,
) -> Result<()> {
    s.bound(t)?;
    let router::Action::Command { line } = a else {
        return s.apply(t, a);
    };
    if line.len() > 2048 {
        return Err("Keep one simulated command within2KiB; previous work retained.".into());
    }
    s.checked = false;
    s.last_trial = None;
    log(s, &format!("$ {line}\n"));
    match command(s, t, saved, line) {
        Ok(out) => {
            log(s, &out.text);
            s.input.clear();
        }
        Err(e) => {
            log(s, &format!("[{:?}] {}\n", e.class, e.message));
            s.input = line.clone();
        }
    }
    s.bound(t)
}
fn command(
    s: &mut router::Scope,
    t: &router::Task,
    saved: &str,
    line: &str,
) -> std::result::Result<Output, Fault> {
    let words = crate::linux_filter::words(line)?;
    if matches!(words.first().map(String::as_str), Some("cat" | "getent")) {
        let out = names::execute(saved, line).map_err(lookup_fault)?;
        return Ok(Output {
            text: format!("{}[exit {}]\n", out.stdout, out.status),
            changed: false,
        });
    }
    let mut probe = s.clone();
    probe.last_trial = None;
    probe.checked = false;
    let out = if words.first().map(String::as_str) == Some("ip6tables") {
        let result = linux_named_ports::v6(&mut probe.state.filter, saved, line)?;
        let mut out = result.output;
        for r in result.resolutions {
            out.text.push_str(&format!(
                "[Simulator note] Resolved {} {} as {}/{} in this new rule.\n",
                r.option,
                r.name,
                r.port,
                names::protocol_name(r.protocol)
            ));
        }
        out
    } else {
        t.command(&mut probe.state, line)?
    };
    probe.bound(t).map_err(|e|fault(Class::Unsupported,format!("Keep the supplied router environment unchanged; only FORWARD rules are editable. {e}")))?;
    s.state = probe.state;
    Ok(out)
}
impl Scope {
    pub fn new(t: &Task) -> Result<Self> {
        t.validate()?;
        Ok(Self {
            router: router::Scope::new(&t.router)?,
            editor: t.initial_services.clone(),
            saved_services: t.initial_services.clone(),
        })
    }
    pub fn shape(&self) -> Result<()> {
        self.router.state.validate()?;
        if !within(self, 640 * 1024)
            || !editor_text(&self.editor)
            || !editor_text(&self.saved_services)
            || self.router.input.len() > 2048
            || self.router.output.len() > router::MAX_CONSOLE
            || self.router.selected_case >= 8
        {
            return Err(
                "Saved services/router work exceeds its declared limits; original retained.".into(),
            );
        }
        names::Database::parse(&self.saved_services).map_err(|e| e.to_string())?;
        if let Some(r) = &self.router.last_trial {
            if r.steps.len() > 12 || !id(&r.case_id) {
                return Err("Invalid saved traffic observation.".into());
            }
            r.connections.validate()?;
        }
        Ok(())
    }
    pub fn bound(&self, t: &Task) -> Result<()> {
        self.shape()?;
        self.router.bound(&t.router)
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
            Action::Edit { text } => {
                if !editor_text(text) {
                    return Err("Keep the services draft within32KiB and use ordinary text.".into());
                }
                next.editor = text.clone();
                next.router.checked = false;
                next.router.last_trial = None;
            }
            Action::Save => {
                names::Database::parse(&next.editor).map_err(|e| format!("File not saved: {e}"))?;
                next.saved_services = next.editor.clone();
                next.router.checked = false;
                next.router.last_trial = None;
            }
            Action::Router { action } => {
                if strict && let router::Action::Command { line } = action {
                    next.router.checked = false;
                    next.router.last_trial = None;
                    log(&mut next.router, &format!("$ {line}\n"));
                    let out = command(&mut next.router, &t.router, &next.saved_services, line)
                        .map_err(|e| e.message)?;
                    log(&mut next.router, &out.text);
                    next.router.input.clear();
                } else {
                    router_action(&mut next.router, &t.router, &next.saved_services, action)?;
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
    Edit {
        #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
        text: String,
    },
    Save,
    Router {
        action: router::Action,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
        if !id(&self.source_id)
            || !within(self, 1024 * 1024)
            || self.worked.is_empty()
            || self.worked.len() > 32
        {
            return Err("Supply a stable source and a bounded reachable worked example.".into());
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self.assignment.router.id == self.example.router.id
            || self.assignment.router.facts == self.example.router.facts
            || meanings(&self.assignment.required_services)?
                == meanings(&self.example.required_services)?
        {
            return Err("Keep example router and file values separate from the assignment.".into());
        }
        let mut example = Scope::new(&self.example)?;
        for a in &self.worked {
            example.reference(&self.example, a)?;
        }
        if !example.router.checked || !self.example.checks(&example).iter().all(|c| c.passed) {
            return Err(
                "Worked actions do not reach the saved-file and actual router requirements.".into(),
            );
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        if s.source_id != self.source_id || !within(s, 1536 * 1024) {
            return Err("Progress belongs to another services/router task or exceeds its bound; original retained.".into());
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
    pub fn solved(&self, s: &Session) -> bool {
        self.bound(s).is_ok()
            && s.assignment.router.checked
            && self
                .assignment
                .checks(&s.assignment)
                .iter()
                .all(|c| c.passed)
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            practice: Scope::new(&d.example)?,
        })
    }
    pub fn shape(&self) -> Result<()> {
        if !id(&self.source_id) || !within(self, 1536 * 1024) {
            return Err("Invalid services/router session identity or size.".into());
        }
        self.assignment.shape()?;
        self.practice.shape()
    }
    pub fn action(&mut self, d: &Definition, a: &Action, practice: bool) -> Result<()> {
        d.bound(self)?;
        let mut next = self.clone();
        let (t, s) = if practice {
            (&d.example, &mut next.practice)
        } else {
            (&d.assignment, &mut next.assignment)
        };
        s.apply(t, a)?;
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<()> {
        d.bound(self)?;
        if practice {
            self.practice = Scope::new(&d.example)?;
        } else {
            self.assignment = Scope::new(&d.assignment)?;
        }
        Ok(())
    }
}
