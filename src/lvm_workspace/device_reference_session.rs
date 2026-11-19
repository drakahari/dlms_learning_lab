//! Optional source-bound reference drafts. Existing sources serialize no new data.
//! Only the virtual State evaluator runs commands; configuration text is inert.
use super::{device_reference_flow as flow, *};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub assignment: flow::Live,
    pub example: flow::Live,
}
impl Runtime {
    pub fn new(d: &Definition) -> Result<Option<Self>, String> {
        match (
            &d.device_reference_task,
            &d.lesson.device_reference_practice,
        ) {
            (None, None) => Ok(None),
            (Some(t), Some(p)) => Ok(Some(Self {
                assignment: flow::Live::new(&t.context)?,
                example: flow::Live::new(&p.context)?,
            })),
            _ => Err("Reference assignment and separate practice must both be supplied".into()),
        }
    }
    pub fn shape(&self) -> Result<(), String> {
        for live in [&self.assignment, &self.example] {
            live.configuration.validate().map_err(|e| e.to_string())?;
            if live.drafts.keys().ne(live.configuration.callers.keys())
                || live
                    .drafts
                    .values()
                    .any(|d| !lvm::device_reference::valid_reference_text(&d.0))
                || !bounded::within(live, 16 * 1024)
            {
                return Err("Reference draft roster/text exceeds its bounded configuration".into());
            }
        }
        Ok(())
    }
}
fn prose(s: &str, n: usize) -> bool {
    !s.trim().is_empty()
        && s.len() <= n
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
}
pub(super) fn definition_shape(d: &Definition) -> Result<(), String> {
    let (Some(task), Some(practice)) = (
        &d.device_reference_task,
        &d.lesson.device_reference_practice,
    ) else {
        return Err(
            "Application reference teaching needs both a typed task and separate practice".into(),
        );
    };
    let l = &d.lesson;
    if d.snapshot_task.is_some()
        || l.snapshot_practice.is_some()
        || !d.targets.is_empty()
        || !l.targets.is_empty()
        || d.plan.is_some()
        || l.plan.is_some()
        || d.retirement.is_some()
        || l.retirement.is_some()
        || d.availability.is_some()
        || l.availability.is_some()
        || d.renaming.is_some()
        || l.renaming.is_some()
        || !l.worked.is_empty()
        || !l.experiment.is_empty()
        || !l.repair.is_empty()
        || !bounded::within(d, MAX_DEFINITION)
        || l.concepts.is_empty()
        || l.concepts.len() > 6
        || [&l.prerequisite_note, &l.description, &l.try_change]
            .iter()
            .any(|s| !prose(s, 1600))
        || l.concepts
            .iter()
            .any(|c| !prose(&c.title, 100) || !prose(&c.text, 1200))
        || d.initial
            .devices
            .keys()
            .any(|p| l.fixture.devices.contains_key(p))
        || d.initial
            .groups
            .keys()
            .any(|g| l.fixture.groups.contains_key(g))
    {
        return Err("Reference teaching needs bounded concepts, separate storage and no mixed legacy or snapshot goals".into());
    }
    task.shape(&d.initial)?;
    practice.shape(&l.fixture)
}
pub(super) fn bound(d: &Definition, s: &Session) -> Result<(), String> {
    match (&d.device_reference_task, &d.lesson.device_reference_practice, &s.reference_runtime) {
        (None,None,None) => Ok(()),
        (Some(t),Some(p),Some(r)) => {
            r.assignment.validate(&t.context, &d.initial, &s.current)?;
            r.example.validate(&p.context, &d.lesson.fixture, &s.example)
        }
        _ => Err("Reference drafts do not match their authored assignment/practice; previous work retained".into()),
    }
}
/// Internal native/declarative packet, never a Linux command or host path action.
pub const PREFIX: &str = "simulated-lvm-reference ";
pub fn encode(op: &flow::Operation) -> Result<String, String> {
    let value = match op {
        flow::Operation::Command { text } => text.clone(),
        _ => format!(
            "{PREFIX}{}",
            serde_json::to_string(op).map_err(|e| e.to_string())?
        ),
    };
    if value.len() > 2048 || value.contains(['\0', '\n', '\r']) {
        return Err("Reference action exceeds its bounded packet".into());
    }
    Ok(value)
}
pub fn decode(text: &str) -> Result<flow::Operation, String> {
    if text.len() > 2048 || text.contains(['\0', '\n', '\r']) {
        return Err("Reference action exceeds its bounded packet".into());
    }
    if let Some(json) = text.strip_prefix(PREFIX) {
        serde_json::from_str(json).map_err(|e| format!("Invalid simulator reference action: {e}"))
    } else {
        Ok(flow::Operation::Command { text: text.into() })
    }
}
impl Definition {
    pub fn reference_context(&self, practice: bool) -> Option<&flow::Context> {
        if practice {
            self.lesson
                .device_reference_practice
                .as_ref()
                .map(|p| &p.context)
        } else {
            self.device_reference_task.as_ref().map(|t| &t.context)
        }
    }
    pub fn reference_helper(&self, index: usize) -> bool {
        self.lesson
            .device_reference_practice
            .as_ref()
            .and_then(|p| p.worked.get(index))
            .is_some_and(|s| !matches!(s.operation, flow::Operation::Command { .. }))
    }
}
impl Session {
    pub fn apply_reference(
        &mut self,
        d: &Definition,
        op: &flow::Operation,
        practice: bool,
    ) -> Result<String, String> {
        self.execute(d, &encode(op)?, practice)
    }
    pub fn edit_reference(
        &mut self,
        d: &Definition,
        caller: &str,
        text: String,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        let c = d
            .reference_context(practice)
            .ok_or("No reference editor for this lesson")?;
        let mut next = self.clone();
        let r = next
            .reference_runtime
            .as_mut()
            .ok_or("Missing reference work")?;
        let (live, initial, current) = if practice {
            (&mut r.example, &d.lesson.fixture, &next.example)
        } else {
            (&mut r.assignment, &d.initial, &next.current)
        };
        live.set_draft(c, initial, current, caller, text)?;
        if practice {
            next.example_checked = false;
        } else {
            next.checked = false;
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn save_reference(
        &mut self,
        d: &Definition,
        caller: &str,
        practice: bool,
    ) -> Result<String, String> {
        d.bound(self)?;
        let r = self
            .reference_runtime
            .as_ref()
            .ok_or("Missing reference work")?;
        let live = if practice { &r.example } else { &r.assignment };
        let path = live
            .drafts
            .get(caller)
            .ok_or("No supplied reference field")?
            .0
            .clone();
        self.apply_reference(
            d,
            &flow::Operation::SaveReference {
                caller: caller.into(),
                path,
            },
            practice,
        )
    }
}
