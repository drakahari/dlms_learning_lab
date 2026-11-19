//! Source-held assembly assignments with isolated examples and replayed progress.
use crate::{cron_workspace as bounded, raid_assembly as core, script_teaching::View};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const CHECK: &str = "@raid-assembly-check";
pub const HELP: &str = core::HELP;
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub target: core::Name,
    pub uuid: core::Uuid,
    pub file: core::Name,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Activity {
    pub context: core::Context,
    pub initial: core::Runtime,
    #[serde(deserialize_with = "bounded::sequence::<_,Requirement,2>")]
    pub requirements: Vec<Requirement>,
}
impl Activity {
    pub fn validate(&self) -> Result<()> {
        self.initial.validate(&self.context)?;
        let mut seen = BTreeSet::new();
        if self.requirements.len() != self.context.identities.len()
            || !bounded::within(self, 192 * 1024)
        {
            return Err(
                "Every supplied mirror needs a bounded meaningful assembly requirement".into(),
            );
        }
        for t in &self.requirements {
            if !seen.insert(&t.target.0) || !self.context.target(&t.target.0) {
                return Err("Assembly requirements need distinct supplied targets".into());
            }
            let id = self.context.identity(&t.uuid)?;
            self.context
                .replicas
                .read_file(&id.array.0, &t.file.0)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub fn checks(&self, r: &core::Runtime) -> Vec<(bool, String)> {
        if let Err(e) = self.validate().and_then(|_| r.validate(&self.context)) {
            return vec![(false, e)];
        }
        let p = core::policy(&self.context, &r.saved.0);
        let mut rows = vec![];
        for t in &self.requirements {
            let saved = p
                .as_ref()
                .map_err(ToString::to_string)
                .and_then(|p| p.selection(&self.context, &t.target.0));
            let saved_ok = saved.as_ref().is_ok_and(|u| *u == t.uuid);
            rows.push((saved_ok,format!("{} · Next assembly from saved policy: {}",t.target.0,match saved{Err(e)=>e,Ok(_)if !saved_ok=>"selects another existing array UUID; compare the original member pair rather than the device name".into(),_=>"selects both original members and their intended array UUID".into()})));
            let active = r.live.iter().find(|b| b.target == t.target);
            let correct = active.is_some_and(|b| b.uuid == t.uuid);
            rows.push((correct,format!("{} · Active array: {}",t.target.0,if correct{"both original synchronized members are active"}else if active.is_some(){"another array identity is active here; saved configuration changes do not replace it"}else{"stopped; assemble the existing clean members, then inspect /proc/mdstat and --detail"})));
            let expected = self
                .context
                .replicas
                .read_file(&self.context.identity(&t.uuid).unwrap().array.0, &t.file.0)
                .unwrap();
            let actual = self.context.read(&r.live, &t.target.0, &t.file.0);
            let good = correct && actual.as_ref().is_ok_and(|v| *v == expected);
            rows.push((good,format!("{} · Original data: {}",t.target.0,if good{"the native sample-data view reads the intended original replica content"}else{"not readable from the intended active identity; assembling another mirror does not restore this dataset"})));
        }
        rows
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Save {
        text: core::Text,
    },
    Command {
        #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
        text: String,
    },
}
impl Operation {
    pub fn label(&self) -> String {
        match self {
            Self::Save { .. } => format!("Save {}", core::CONFIG),
            Self::Command { text } => text.clone(),
        }
    }
    fn shape(&self, a: &Activity) -> Result<()> {
        match self {
            Self::Save { text } => core::text(&text.0),
            Self::Command { text } => core::command_shape(&a.context, text),
        }
    }
    fn apply(&self, a: &Activity, r: &mut core::Runtime) -> Result<core::Outcome> {
        self.shape(a)?;
        match self {
            Self::Save { text } => r.save(&a.context, text.0.clone()),
            Self::Command { text } => r.execute(&a.context, text),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Step {
    pub operation: Operation,
    #[serde(deserialize_with = "crate::cron::text::<_,640>")]
    pub explanation: String,
    pub status: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Definition {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub source_id: String,
    pub assignment: Activity,
    pub example: Activity,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,20>")]
    pub reference: Vec<Step>,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,16>")]
    pub example_steps: Vec<Step>,
    pub experiment: Step,
    #[serde(deserialize_with = "bounded::sequence::<_,Step,8>")]
    pub repair: Vec<Step>,
}
fn id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
impl Definition {
    pub fn validate(&self) -> Result<()> {
        if self.version != 1
            || !id(&self.source_id)
            || !bounded::within(self, 384 * 1024)
            || self.reference.is_empty()
            || self.reference.len() > 20
            || self.example_steps.is_empty()
            || self.example_steps.len() > 16
            || self.repair.is_empty()
            || self.repair.len() > 8
        {
            return Err("Unknown or oversized assembly teaching definition".into());
        }
        self.assignment.validate()?;
        self.example.validate()?;
        if self
            .assignment
            .context
            .replicas
            .devices
            .keys()
            .any(|k| self.example.context.replicas.devices.contains_key(k))
            || self.assignment.context.identities.iter().any(|i| {
                self.example
                    .context
                    .identities
                    .iter()
                    .any(|v| v.uuid == i.uuid)
            })
        {
            return Err(
                "Practice requires separate component names, original data and identities".into(),
            );
        }
        for (steps, a) in [
            (&self.reference, &self.assignment),
            (&self.example_steps, &self.example),
            (&self.repair, &self.example),
        ] {
            for s in steps {
                Self::step(s, a)?;
            }
        }
        Self::step(&self.experiment, &self.example)
    }
    fn step(s: &Step, a: &Activity) -> Result<()> {
        if s.status > 1
            || s.explanation.trim().is_empty()
            || s.explanation.len() > 640
            || s.explanation
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
        {
            return Err(
                "Every assembly teaching action needs a concise explanation and expected status"
                    .into(),
            );
        }
        s.operation.shape(a)
    }
    pub fn bound(&self, s: &Session) -> Result<()> {
        self.validate()?;
        if !s.shape() || s.origin != self.source_id || s.example_step >= self.example_steps.len() {
            return Err("Saved assembly work does not match its held task/version/selection; original retained".into());
        }
        s.assignment.bound(&self.assignment)?;
        s.practice.bound(&self.example)
    }
    pub fn checks(&self, s: &Session) -> Vec<(bool, String)> {
        if let Err(e) = self.bound(s) {
            vec![(false, e)]
        } else {
            s.assignment.checks(&self.assignment)
        }
    }
    pub fn validate_reachability(&self) -> Result<()> {
        self.validate()?;
        for (a, steps, practice) in [
            (&self.assignment, &self.reference, false),
            (&self.example, &self.example_steps, true),
        ] {
            let mut w = Work::new(a)?;
            if a.checks(&w.runtime).iter().all(|r| r.0) {
                return Err("An assembly assignment must require meaningful repair".into());
            }
            for s in steps {
                let o = w.apply(a, &s.operation)?;
                if o.status != s.status {
                    return Err(format!(
                        "{}: expected {}, actual {}: {}",
                        s.operation.label(),
                        s.status,
                        o.status,
                        o.output
                    ));
                }
            }
            w.checked = true;
            if w.checks(a).iter().any(|r| !r.0) {
                return Err(format!(
                    "Assembly teaching method misses requirements: {:?}",
                    w.checks(a)
                ));
            }
            if practice {
                if w.apply(a, &self.experiment.operation)?.status != self.experiment.status
                    || a.checks(&w.runtime).iter().all(|r| r.0)
                {
                    return Err(
                        "Practice experiment must break a meaningful requirement as described"
                            .into(),
                    );
                }
                for s in &self.repair {
                    if w.apply(a, &s.operation)?.status != s.status {
                        return Err("Practice repair does not run as documented".into());
                    }
                }
                w.checked = true;
                if w.checks(a).iter().any(|r| !r.0) {
                    return Err("Practice repair does not restore the required behavior".into());
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub action: String,
    pub result: core::Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Work {
    pub runtime: core::Runtime,
    pub draft: core::Text,
    #[serde(deserialize_with = "bounded::sequence::<_,Operation,64>")]
    pub history: Vec<Operation>,
    #[serde(deserialize_with = "bounded::sequence::<_,Row,32>")]
    pub transcript: Vec<Row>,
    #[serde(deserialize_with = "crate::cron::text::<_,2048>")]
    pub command: String,
    pub checked: bool,
}
fn record(rows: &mut Vec<Row>, op: &Operation, result: core::Outcome) {
    if rows.len() == 32 {
        rows.remove(0);
    }
    rows.push(Row {
        action: op.label(),
        result,
    });
}
impl Work {
    pub fn new(a: &Activity) -> Result<Self> {
        a.validate()?;
        Ok(Self {
            runtime: a.initial.clone(),
            draft: a.initial.saved.clone(),
            history: vec![],
            transcript: vec![],
            command: String::new(),
            checked: false,
        })
    }
    pub fn bound(&self, a: &Activity) -> Result<()> {
        if !bounded::within(self, 512 * 1024)
            || self.history.len() > 64
            || self.transcript.len() > 32
            || self.command.len() > 2048
            || !self.command.is_ascii()
            || self
                .command
                .bytes()
                .any(|b| b.is_ascii_control() && b != b'\t')
        {
            return Err("Saved assembly work exceeds its bounded command/history capacity".into());
        }
        core::text(&self.draft.0)?;
        self.runtime.validate(&a.context)?;
        let mut actual = a.initial.clone();
        let mut rows = vec![];
        for op in &self.history {
            let result = op.apply(a, &mut actual)?;
            record(&mut rows, op, result);
        }
        if actual != self.runtime || rows != self.transcript {
            return Err("Saved assembly effects/results do not match this task and its actual actions; previous data retained".into());
        }
        Ok(())
    }
    pub fn edit(&mut self, a: &Activity, text: String) -> Result<()> {
        self.bound(a)?;
        core::text(&text)?;
        self.draft = core::Text(text);
        self.checked = false;
        Ok(())
    }
    pub fn apply(&mut self, a: &Activity, op: &Operation) -> Result<core::Outcome> {
        self.bound(a)?;
        if self.history.len() == 64 {
            return Err("This attempt reached 64 actions. Export work if needed, then reset this selected attempt for a new investigation".into());
        }
        let mut next = self.clone();
        let out = op.apply(a, &mut next.runtime)?;
        if let Operation::Save { text } = op {
            next.draft = text.clone();
        }
        next.history.push(op.clone());
        record(&mut next.transcript, op, out.clone());
        next.checked = false;
        next.bound(a)?;
        *self = next;
        Ok(out)
    }
    pub fn checks(&self, a: &Activity) -> Vec<(bool, String)> {
        if !self.checked {
            return vec![(false,"Not checked · choose Check work to compare saved policy, active identity and original data".into())];
        }
        let mut out=vec![(self.draft==self.runtime.saved,"Save or revert the configuration draft before checking; the editor and saved policy are separate".into())];
        out.extend(a.checks(&self.runtime));
        out
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub version: u32,
    #[serde(deserialize_with = "crate::cron::text::<_,64>")]
    pub origin: String,
    pub view: View,
    pub example_step: usize,
    pub assignment: Work,
    pub practice: Work,
}
impl Session {
    pub fn new(d: &Definition) -> Result<Self> {
        d.validate()?;
        Ok(Self {
            version: 1,
            origin: d.source_id.clone(),
            view: View::Introduction,
            example_step: 0,
            assignment: Work::new(&d.assignment)?,
            practice: Work::new(&d.example)?,
        })
    }
    pub fn shape(&self) -> bool {
        self.version == 1
            && id(&self.origin)
            && self.example_step < 16
            && bounded::within(self, 1024 * 1024)
    }
    pub fn activity(d: &Definition, practice: bool) -> &Activity {
        if practice { &d.example } else { &d.assignment }
    }
    pub fn work(&self, practice: bool) -> &Work {
        if practice {
            &self.practice
        } else {
            &self.assignment
        }
    }
    pub fn work_mut(&mut self, practice: bool) -> &mut Work {
        if practice {
            &mut self.practice
        } else {
            &mut self.assignment
        }
    }
    pub fn apply(
        &mut self,
        d: &Definition,
        op: &Operation,
        practice: bool,
    ) -> Result<core::Outcome> {
        d.bound(self)?;
        let o = self
            .work_mut(practice)
            .apply(Self::activity(d, practice), op)?;
        d.bound(self)?;
        Ok(o)
    }
    pub fn edit(&mut self, d: &Definition, text: String, practice: bool) -> Result<()> {
        d.bound(self)?;
        self.work_mut(practice)
            .edit(Self::activity(d, practice), text)
    }
    pub fn save(&mut self, d: &Definition, practice: bool) -> Result<core::Outcome> {
        let op = Operation::Save {
            text: self.work(practice).draft.clone(),
        };
        self.apply(d, &op, practice)
    }
    pub fn run(&mut self, d: &Definition, input: &str, practice: bool) -> Result<String> {
        d.bound(self)?;
        if input == CHECK {
            self.work_mut(practice).checked = true;
            return Ok(if self
                .work(practice)
                .checks(Self::activity(d, practice))
                .iter()
                .all(|r| r.0)
            {
                "Current work meets every requirement"
            } else {
                "Work needs changes; inspect the feedback and retry"
            }
            .into());
        }
        self.apply(d, &Operation::Command { text: input.into() }, practice)
            .map(|v| v.display())
    }
}
