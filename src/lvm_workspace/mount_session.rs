//! Optional source-bound mount practice. Older sources and progress omit it.
use super::{mount_flow as flow, *};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Runtime {
    pub assignment: flow::Live,
    pub example: flow::Live,
}
impl Runtime {
    pub fn new(d: &Definition) -> Result<Option<Self>, String> {
        match (&d.mount_task, &d.lesson.mount_practice) {
            (None, None) => Ok(None),
            (Some(t), Some(p)) => Ok(Some(Self {
                assignment: flow::Live::new(&t.context, &d.initial)?,
                example: flow::Live::new(&p.context, &d.lesson.fixture)?,
            })),
            _ => Err("Mount teaching needs both an assignment and separate practice".into()),
        }
    }
    pub fn shape(&self) -> Result<(), String> {
        for l in [&self.assignment, &self.example] {
            if !bounded::within(l, 80 * 1024)
                || l.draft.len() > 4096
                || l.draft.lines().count() > 64
                || l.draft
                    .chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
                || l.mounted.version != 1
            {
                return Err("Mount drafts exceed their saved-work format or bounds".into());
            }
            l.mounted.storage.validate().map_err(|e| e.to_string())?;
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
    let (Some(t), Some(p)) = (&d.mount_task, &d.lesson.mount_practice) else {
        return Err("Mount teaching needs both a typed assignment and separate practice".into());
    };
    let l = &d.lesson;
    if d.config_scan_task.is_some()
        || l.config_scan_practice.is_some()
        || d.device_reference_task.is_some()
        || l.device_reference_practice.is_some()
        || d.snapshot_task.is_some()
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
            .any(|k| l.fixture.devices.contains_key(k))
        || d.initial
            .groups
            .keys()
            .any(|k| l.fixture.groups.contains_key(k))
    {
        return Err("Mount lessons need clear concepts, separate example storage and no mixed legacy/reference/scan/snapshot goals".into());
    }
    model(&d.initial)?;
    model(&l.fixture)?;
    t.shape(&d.initial)?;
    p.shape(&l.fixture)?;
    Ok(())
}
pub(super) fn bound(d: &Definition, s: &Session) -> Result<(), String> {
    match(&d.mount_task,&d.lesson.mount_practice,&s.mount_runtime){
        (None,None,None)=>Ok(()),
        (Some(t),Some(p),Some(r))=>{r.assignment.validate(&t.context,&d.initial,&s.current)?;r.example.validate(&p.context,&d.lesson.fixture,&s.example)},
        _=>Err("Mount work does not match its authored assignment and practice; previous work retained".into()),
    }
}
impl Definition {
    pub fn mount_context(&self, practice: bool) -> Option<&flow::Context> {
        if practice {
            self.lesson.mount_practice.as_ref().map(|p| &p.context)
        } else {
            self.mount_task.as_ref().map(|t| &t.context)
        }
    }
    pub fn mount_helper(&self, index: usize) -> bool {
        self.lesson
            .mount_practice
            .as_ref()
            .and_then(|p| p.worked.get(index))
            .is_some_and(|s| !matches!(s.operation, flow::Operation::Command { .. }))
    }
}
impl Session {
    pub fn edit_mount(
        &mut self,
        d: &Definition,
        text: String,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        let c = d
            .mount_context(practice)
            .ok_or("No supplied mount editor")?;
        let mut next = self.clone();
        let r = next.mount_runtime.as_mut().ok_or("Missing mount work")?;
        if practice {
            r.example
                .set_draft(c, &d.lesson.fixture, &next.example, text)?;
            next.example_checked = false;
        } else {
            r.assignment.set_draft(c, &d.initial, &next.current, text)?;
            next.checked = false;
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn mount_action(
        &mut self,
        d: &Definition,
        op: &flow::Operation,
        practice: bool,
    ) -> Result<String, String> {
        d.bound(self)?;
        let c = d.mount_context(practice).ok_or("No supplied mount task")?;
        let mut next = self.clone();
        let r = next.mount_runtime.as_mut().ok_or("Missing mount work")?;
        let output = if practice {
            r.example
                .apply(c, &d.lesson.fixture, &mut next.example, op)?
        } else {
            r.assignment.apply(c, &d.initial, &mut next.current, op)?
        };
        let rows = if practice {
            next.example_checked = false;
            &mut next.example_transcript
        } else {
            next.checked = false;
            &mut next.transcript
        };
        if rows.len() == 32 {
            rows.remove(0);
        }
        rows.push(Row {
            input: op.description(),
            output: output.clone(),
        });
        d.bound(&next)?;
        *self = next;
        Ok(output)
    }
    pub fn save_mount(&mut self, d: &Definition, practice: bool) -> Result<String, String> {
        d.bound(self)?;
        let r = self.mount_runtime.as_ref().ok_or("Missing mount work")?;
        let text = if practice {
            r.example.draft.clone()
        } else {
            r.assignment.draft.clone()
        };
        self.mount_action(d, &flow::Operation::SavePolicy { text }, practice)
    }
}
