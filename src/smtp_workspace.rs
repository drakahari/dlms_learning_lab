//! Source-bound mail requirements and separate practice/assignment state.
use crate::{
    script_teaching::View,
    smtp_config::{self as config, Config},
    smtp_equivalence, smtp_queue as queue,
    smtp_routing::{Outcome, Trace},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    net::Ipv4Addr,
};
pub const MAX_TASK: usize = 384 * 1024;
pub const MAX_SCOPE: usize = 512 * 1024;
pub const MAX_SESSION: usize = 1024 * 1024;
pub const MAX_ACTION: usize = 32 * 1024;
fn within<T: Serialize>(v: &T, cap: usize) -> bool {
    struct Budget(usize, usize);
    impl std::io::Write for Budget {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(b.len())
                .filter(|n| *n <= self.1)
                .ok_or_else(|| std::io::Error::other("Mail workspace size limit"))?;
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Budget(0, cap), v).is_ok()
}
fn id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"-_".contains(&b))
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum Expected {
    Delivery {
        node: String,
        mailboxes: BTreeSet<String>,
    },
    Reject {
        node: String,
    },
    Defer {
        node: String,
    },
}
impl Expected {
    fn validate(&self, f: &queue::Facts) -> Result<(), String> {
        let node = match self {
            Self::Delivery { node, .. } | Self::Reject { node } | Self::Defer { node } => node,
        };
        let n = f
            .world
            .nodes
            .get(node)
            .ok_or("A required mail outcome needs a supplied endpoint.")?;
        if let Self::Delivery { mailboxes, .. } = self
            && (mailboxes.is_empty()
                || mailboxes.len() > 8
                || mailboxes.iter().any(|u| !n.users.contains(u)))
        {
            return Err("Required deliveries need one to8supplied local mailboxes.".into());
        }
        Ok(())
    }
    pub fn matches(&self, t: &Trace) -> bool {
        match (self, &t.outcome) {
            (
                Self::Delivery {
                    node: wanted,
                    mailboxes: users,
                },
                Outcome::Local {
                    node,
                    mailboxes,
                    failures,
                },
            ) => node == wanted && mailboxes == users && failures.is_empty(),
            (Self::Reject { node: wanted }, Outcome::Rejected { node })
            | (Self::Defer { node: wanted }, Outcome::Deferred { node }) => node == wanted,
            _ => false,
        }
    }
    fn recorded(&self, s: &queue::Status) -> bool {
        match (self, s) {
            (
                Self::Delivery {
                    node: wanted,
                    mailboxes: users,
                },
                queue::Status::Delivered { node, mailboxes },
            ) => node == wanted && mailboxes == users,
            _ => false,
        }
    }
    pub fn describe(&self) -> String {
        match self {
            Self::Delivery { node, mailboxes } => format!(
                "Deliver on {node} to {}",
                mailboxes.iter().cloned().collect::<Vec<_>>().join(", ")
            ),
            Self::Reject { node } => format!("Reject relay permission at {node}"),
            Self::Defer { node } => format!("Temporarily defer relay permission at {node}"),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Probe {
    pub id: String,
    pub label: String,
    pub entry: String,
    pub client: Ipv4Addr,
    pub recipient: String,
    pub expected: Expected,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub facts: queue::Facts,
    /// All other settings must retain their supplied semantic behavior.
    pub editable: BTreeMap<String, BTreeSet<String>>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Probe,16>")]
    pub probes: Vec<Probe>,
    pub queued: BTreeMap<String, Expected>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub label: String,
    pub detail: String,
}
pub fn observed(t: &Trace) -> String {
    match &t.outcome {
        Outcome::Local {
            node,
            mailboxes,
            failures,
        } => format!(
            "Local result on {node}: {}.{}",
            if mailboxes.is_empty() {
                "no mailbox received mail".into()
            } else {
                format!(
                    "delivered to {}",
                    mailboxes.iter().cloned().collect::<Vec<_>>().join(", ")
                )
            },
            failures
                .iter()
                .take(2)
                .map(|s| format!(" Recipient problem: {s}."))
                .collect::<String>()
        ),
        Outcome::Rejected { node } => format!("Relay permission rejected at {node}."),
        Outcome::Deferred { node } => format!("Relay permission deferred at {node}."),
        Outcome::NoRoute { node } => {
            format!("No supplied next hop at {node}; that is not proof of a closed relay policy.")
        }
        Outcome::Unavailable { target, port } => {
            format!("No available supplied endpoint at {target}:{port}.")
        }
        Outcome::ConfigurationProblem { node, error } => format!("{node}: {}", error.message),
        Outcome::Loop { repeats, .. } => {
            format!("The represented relay path cycles through {repeats}.")
        }
        Outcome::Limit => "The finite route limit was reached.".into(),
    }
}
impl Task {
    pub fn validate(&self) -> Result<(), String> {
        if !within(self, MAX_TASK) {
            return Err("Mail task source exceeds384KiB.".into());
        }
        self.facts.validate().map_err(|e| e.message)?;
        if self.editable.is_empty()
            || self.editable.len() > self.facts.world.nodes.len()
            || self.probes.is_empty()
            || self.probes.len() > 16
            || self.queued.len() > self.facts.samples.len()
        {
            return Err("Mail tasks need explicit editable settings,1to16fresh behavioral probes and bounded queued requirements.".into());
        }
        for (node, fields) in &self.editable {
            if !self.facts.world.nodes.contains_key(node)
                || fields.is_empty()
                || fields.len() > 7
                || fields
                    .iter()
                    .any(|s| !config::PARAMETERS.contains(&s.as_str()))
            {
                return Err(
                    "Editable mail settings must name supplied nodes and supported parameters."
                        .into(),
                );
            }
        }
        let mut ids = BTreeSet::new();
        for p in &self.probes {
            if !id(&p.id)
                || !ids.insert(&p.id)
                || p.label.is_empty()
                || p.label.len() > 160
                || !self.facts.world.nodes.contains_key(&p.entry)
            {
                return Err(
                    "Mail probes need unique stable IDs, short labels and supplied entry servers."
                        .into(),
                );
            }
            crate::smtp_routing::Recipient::parse(&p.recipient).map_err(|e| e.message)?;
            p.expected.validate(&self.facts)?;
        }
        for (sample, wanted) in &self.queued {
            if !self.facts.samples.contains_key(sample)
                || !matches!(wanted, Expected::Delivery { .. })
            {
                return Err("Queued requirements must refer to original samples and real delivery outcomes.".into());
            }
            wanted.validate(&self.facts)?;
        }
        Ok(())
    }
    pub fn checks(&self, r: &queue::Runtime) -> Vec<Check> {
        if let Err(e) = self
            .validate()
            .and_then(|_| r.validate(&self.facts).map_err(|e| e.message))
        {
            return vec![Check {
                passed: false,
                label: "Read this mail workspace".into(),
                detail: e,
            }];
        }
        let mut rows = vec![];
        let mut bad_files = vec![];
        let mut protected = vec![];
        for (node, baseline) in &self.facts.initial_active {
            let baseline = Config::parse(baseline).unwrap();
            for (label, text) in [("saved", &r.files[node]), ("running", &r.active[node])] {
                match Config::parse(text) {
                    Err(e) => bad_files.push(format!("{node} {label}: {}", e.message)),
                    Ok(current) => {
                        for field in config::PARAMETERS {
                            if !self
                                .editable
                                .get(node)
                                .is_some_and(|fields| fields.contains(field))
                                && !smtp_equivalence::field(&baseline, &current, field)
                            {
                                protected.push(format!("{node} {label}: {field}"));
                            }
                        }
                    }
                }
            }
        }
        rows.push(Check {
            passed: bad_files.is_empty(),
            label: "Valid saved and running settings".into(),
            detail: if bad_files.is_empty() {
                "Every represented configuration uses supported valid settings.".into()
            } else {
                bad_files.into_iter().take(3).collect::<Vec<_>>().join("; ")
            },
        });
        rows.push(Check{passed:protected.is_empty(),label:"Keep other supplied settings".into(),detail:if protected.is_empty(){"Settings outside this task retain their supplied semantic behavior.".into()}else{format!("Restore the settings outside this task: {}. Equivalent address ranges and policy logic are accepted.",protected.into_iter().take(3).collect::<Vec<_>>().join("; "))}});
        let saved = self.facts.world.prepare(&r.files).unwrap();
        let active = self.facts.world.prepare(&r.active).unwrap();
        for p in &self.probes {
            let stored = saved.incoming(&p.entry, p.client, &p.recipient).unwrap();
            let running = active.incoming(&p.entry, p.client, &p.recipient).unwrap();
            let pass = p.expected.matches(&stored) && p.expected.matches(&running);
            rows.push(Check{passed:pass,label:p.label.clone(),detail:if pass{format!("As required: {}.",p.expected.describe())}else{format!("Required: {}. Saved: {} Running: {} Check the named destination role, trust boundary and restriction order; correct settings must be saved and reloaded.",p.expected.describe(),observed(&stored),observed(&running))}});
        }
        for (id, wanted) in &self.queued {
            let sample = &self.facts.samples[id];
            let stored = saved.queued(&sample.entry, &sample.recipient).unwrap();
            let running = active.queued(&sample.entry, &sample.recipient).unwrap();
            let recorded = wanted.recorded(&r.messages[id].status);
            let pass = recorded && wanted.matches(&stored) && wanted.matches(&running);
            rows.push(Check{passed:pass,label:format!("Queued sample: {}",sample.subject),detail:if pass{format!("Recorded and current delivery agree: {}.",wanted.describe())}else{format!("Required: {}. An empty queue alone is insufficient. Correct and reload the route, then retry pending mail; reset only sample mail to repeat a failed sample while retaining configuration work.",wanted.describe())}});
        }
        rows
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Run {
        #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
        input: String,
    },
    Edit {
        node: String,
        #[serde(deserialize_with = "crate::cron::text::<_,8192>")]
        text: String,
    },
    Save {
        node: String,
    },
    Reload {
        node: String,
    },
    Flush {
        node: String,
    },
    ResetSamples,
    Check,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub runtime: queue::Runtime,
    pub drafts: BTreeMap<String, String>,
    pub selected: String,
    #[serde(deserialize_with = "crate::cron::text::<_,4096>")]
    pub input: String,
    #[serde(deserialize_with = "crate::cron::text::<_,32768>")]
    pub output: String,
    pub checked: bool,
}
impl Scope {
    fn new(t: &Task) -> Result<Self, String> {
        Ok(Self {
            runtime: queue::Runtime::new(&t.facts).map_err(|e| e.message)?,
            drafts: t.facts.initial_files.clone(),
            selected: t.editable.keys().next().unwrap().clone(),
            input: String::new(),
            output: String::new(),
            checked: false,
        })
    }
    pub fn bound(&self, t: &Task) -> Result<(), String> {
        t.validate()?;
        if !within(self, MAX_SCOPE)
            || self.drafts.keys().ne(t.facts.world.nodes.keys())
            || self
                .drafts
                .values()
                .any(|s| s.len() > 8192 || s.lines().count() > 96)
            || !t.facts.world.nodes.contains_key(&self.selected)
            || self.input.len() > 4096
            || self.output.len() > 32768
        {
            return Err("Mail drafts, server selection or console data exceed the supplied workspace bounds.".into());
        }
        self.runtime.validate(&t.facts).map_err(|e| e.message)
    }
    pub fn checks(&self, t: &Task) -> Vec<Check> {
        if let Err(e) = self.bound(t) {
            return vec![Check {
                passed: false,
                label: "Read this saved work".into(),
                detail: e,
            }];
        }
        let mut rows = vec![Check {
            passed: self.drafts == self.runtime.files,
            label: "Save the editor changes".into(),
            detail: if self.drafts == self.runtime.files {
                "The editors match their saved simulated files.".into()
            } else {
                "Save or restore the edited configuration before checking. The running settings change only after a successful reload.".into()
            },
        }];
        rows.extend(t.checks(&self.runtime));
        rows
    }
    pub fn apply(&mut self, t: &Task, a: &Action) -> Result<Vec<Check>, String> {
        self.bound(t)?;
        if !within(a, MAX_ACTION) {
            return Err("Mail action exceeds32KiB.".into());
        }
        match a {
            Action::Run { input } => {
                let result =
                    crate::smtp_console::run(&t.facts, &mut self.runtime, &self.selected, input);
                crate::smtp_console::append(
                    &mut self.output,
                    &format!(
                        "{}$ {input}\n{}{}",
                        self.selected, result.stdout, result.stderr
                    ),
                );
                if !result.observations.is_empty() {
                    crate::smtp_console::append(
                        &mut self.output,
                        &format!(
                            "Simulator observations:\n{}\n",
                            result.observations.join("\n")
                        ),
                    );
                }
                crate::smtp_console::append(
                    &mut self.output,
                    &format!("[command exit status: {}]\n", result.status),
                );
                if result.changed {
                    self.checked = false;
                }
            }
            Action::Edit { node, text } => {
                if !self.drafts.contains_key(node) {
                    return Err("Choose a supplied server to edit.".into());
                }
                if text.len() > 8192 || text.lines().count() > 96 {
                    return Err("Keep this editor within8KiB and96lines.".into());
                }
                self.drafts.insert(node.clone(), text.clone());
                self.checked = false;
            }
            Action::Save { node } => {
                let text = self
                    .drafts
                    .get(node)
                    .ok_or("Choose a supplied server to save.")?
                    .clone();
                self.runtime
                    .save(&t.facts, node, &text)
                    .map_err(|e| e.message)?;
                self.checked = false;
            }
            Action::Reload { node } => {
                self.runtime.reload(&t.facts, node).map_err(|e| e.message)?;
                self.checked = false;
            }
            Action::Flush { node } => {
                self.runtime.flush(&t.facts, node).map_err(|e| e.message)?;
                self.checked = false;
            }
            Action::ResetSamples => {
                self.runtime
                    .reset_samples(&t.facts)
                    .map_err(|e| e.message)?;
                self.checked = false;
            }
            Action::Check => self.checked = true,
        }
        Ok(self.checks(t))
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Task,
    pub example: Task,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,24>")]
    pub reference: Vec<Action>,
    #[serde(deserialize_with = "crate::cron_workspace::sequence::<_,Action,24>")]
    pub example_steps: Vec<Action>,
}
impl Definition {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1
            || !id(&self.source_id)
            || !within(self, 1024 * 1024)
            || self.reference.is_empty()
            || self.reference.len() > 24
            || self.example_steps.is_empty()
            || self.example_steps.len() > 24
        {
            return Err("Mail definitions need schema1, a bounded stable identifier and finite authored methods.".into());
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self.assignment.facts == self.example.facts {
            return Err("Use a separately named/configured guided mail example, not the assignment's initial work.".into());
        }
        Ok(())
    }
    pub fn validate_reachability(&self) -> Result<(), String> {
        self.validate()?;
        let initial = Scope::new(&self.assignment)?;
        if initial.checks(&self.assignment).iter().all(|r| r.passed) {
            return Err("An independent mail assignment must require meaningful work.".into());
        }
        for (t, actions) in [
            (&self.assignment, &self.reference),
            (&self.example, &self.example_steps),
        ] {
            let mut s = Scope::new(t)?;
            for a in actions {
                s.apply(t, a)?;
            }
            if !s.checks(t).iter().all(|r| r.passed) {
                return Err("The authored method must reach every required mail outcome.".into());
            }
        }
        Ok(())
    }
    pub fn bound(&self, s: &Session) -> Result<(), String> {
        self.validate()?;
        s.validate()?;
        if s.version != 1 || s.source_id != self.source_id || !within(s, MAX_SESSION) {
            return Err(
                "Mail work belongs to a different source or schema, or exceeds1MiB.".into(),
            );
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
    pub fn current(&self, s: &Session) -> bool {
        self.bound(s).is_ok()
            && s.assignment.checked
            && s.assignment
                .checks(&self.assignment)
                .iter()
                .all(|r| r.passed)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            return vec![(false, e)];
        }
        if !s.assignment.checked {
            return vec![(false, "Check the current assignment configuration.".into())];
        }
        s.assignment
            .checks(&self.assignment)
            .into_iter()
            .map(|r| (r.passed, format!("{}: {}", r.label, r.detail)))
            .collect()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    pub source_id: String,
    pub view: View,
    pub assignment: Scope,
    pub practice: Scope,
}
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || !id(&self.source_id) || !within(self, MAX_SESSION) {
            return Err(
                "Unknown mail work version, identity or excessive saved data. Original retained."
                    .into(),
            );
        }
        for s in [&self.assignment, &self.practice] {
            if !id(&s.selected)
                || s.drafts.is_empty()
                || s.drafts.len() > 6
                || s.drafts
                    .iter()
                    .any(|(n, t)| !id(n) || t.len() > 8192 || t.lines().count() > 96)
                || s.runtime.version != 1
                || s.runtime.files.len() > 6
                || s.runtime.active.len() > 6
                || s.runtime.messages.len() > 16
                || !within(s, MAX_SCOPE)
                || s.input.len() > 4096
                || s.output.len() > 32768
                || s.input.contains('\0')
                || s.output.contains('\0')
            {
                return Err("Mail editors, selection or console data exceed the supported saved form. Original retained.".into());
            }
        }
        Ok(())
    }
    pub fn new(d: &Definition) -> Result<Self, String> {
        d.validate()?;
        Ok(Self {
            version: 1,
            source_id: d.source_id.clone(),
            view: View::Introduction,
            assignment: Scope::new(&d.assignment)?,
            practice: Scope::new(&d.example)?,
        })
    }
    pub fn action(
        &mut self,
        d: &Definition,
        a: &Action,
        practice: bool,
    ) -> Result<Vec<Check>, String> {
        d.bound(self)?;
        if practice {
            self.practice.apply(&d.example, a)
        } else {
            self.assignment.apply(&d.assignment, a)
        }
    }
    pub fn reset(&mut self, d: &Definition, practice: bool) -> Result<(), String> {
        d.bound(self)?;
        if practice {
            self.practice = Scope::new(&d.example)?;
        } else {
            self.assignment = Scope::new(&d.assignment)?;
        }
        Ok(())
    }
}
