//! Optional source-bound configuration profile. Old sources/sessions omit it.
//! Native typed operations never become commands or a host execution path.
use super::{config_scan as flow, *};
pub(super) fn fresh(d: &Definition) -> Result<Option<flow::Runtime>, String> {
    match (&d.config_scan_task, &d.lesson.config_scan_practice) {
        (None, None) => Ok(None),
        (Some(t), Some(p)) => flow::Runtime::new(t, p).map(Some),
        _ => Err("Configuration tasks need separate typed assignment and practice".into()),
    }
}
fn prose(s: &str, max: usize) -> bool {
    !s.trim().is_empty()
        && s.len() <= max
        && !s
            .chars()
            .any(|c| c.is_control() && !matches!(c, '\n' | '\t'))
}
pub(super) fn definition_shape(d: &Definition) -> Result<(), String> {
    let (Some(t), Some(p)) = (&d.config_scan_task, &d.lesson.config_scan_practice) else {
        return Err("Configuration teaching needs both assignment and separate practice".into());
    };
    let l = &d.lesson;
    if d.device_reference_task.is_some()
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
    {
        return Err("Configuration teaching needs bounded explanations, separate practice and no mixed legacy/storage-mutation goals".into());
    }
    t.shape()?;
    p.shape()?;
    if d.initial != t.context.cases[0].inventory || l.fixture != p.context.cases[0].inventory {
        return Err("Configuration teaching must bind each visible inventory to its first supplied scan case".into());
    }
    for case in t.context.cases.iter().chain(&p.context.cases) {
        model(&case.inventory)?;
        if case.inventory.provisioning.is_some()
            || case.inventory.activation.is_some()
            || case.inventory.identity.is_some()
            || case.inventory.pv_capacity.is_some()
            || case.inventory.snapshots.is_some()
        {
            return Err("This configuration profile admits only read-only supplied storage, without other LVM lesson profiles".into());
        }
    }
    fresh(d)?;
    Ok(())
}
pub(super) fn bound(d: &Definition, s: &Session) -> Result<(), String> {
    match (&d.config_scan_task,&d.lesson.config_scan_practice,&s.config_scan_runtime) {
        (None,None,None)=>Ok(()),
        (Some(t),Some(p),Some(r))=>{
            r.validate(t,p)?;
            if s.current!=d.initial||s.example!=d.lesson.fixture||!s.command.is_empty()||!s.example_command.is_empty(){
                return Err("Configuration work cannot mutate supplied storage or contain a terminal command draft".into());
            }Ok(())
        }
        _=>Err("Configuration drafts do not match their authored assignment and practice; previous work retained".into()),
    }
}
impl Definition {
    pub fn config_context(&self, practice: bool) -> Option<&flow::Context> {
        if practice {
            self.lesson
                .config_scan_practice
                .as_ref()
                .map(|p| &p.context)
        } else {
            self.config_scan_task.as_ref().map(|t| &t.context)
        }
    }
}
impl Session {
    pub fn config_edit(
        &mut self,
        d: &Definition,
        text: String,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        let c = d
            .config_context(practice)
            .ok_or("No supplied configuration task")?;
        let mut next = self.clone();
        let runtime = next
            .config_scan_runtime
            .as_mut()
            .ok_or("Missing configuration work")?;
        let live = if practice {
            &mut runtime.example
        } else {
            &mut runtime.assignment
        };
        live.edit(c, text)?;
        if practice {
            next.example_checked = false;
        } else {
            next.checked = false;
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn config_selection_edit(
        &mut self,
        d: &Definition,
        file: flow::Selection,
        practice: bool,
    ) -> Result<(), String> {
        d.bound(self)?;
        let c = d
            .config_context(practice)
            .ok_or("No supplied configuration task")?;
        let mut next = self.clone();
        let runtime = next
            .config_scan_runtime
            .as_mut()
            .ok_or("Missing configuration work")?;
        let live = if practice {
            &mut runtime.example
        } else {
            &mut runtime.assignment
        };
        live.edit_selection(c, file)?;
        if practice {
            next.example_checked = false;
        } else {
            next.checked = false;
        }
        d.bound(&next)?;
        *self = next;
        Ok(())
    }
    pub fn config_action(
        &mut self,
        d: &Definition,
        op: &flow::Operation,
        practice: bool,
    ) -> Result<String, String> {
        d.bound(self)?;
        let c = d
            .config_context(practice)
            .ok_or("No supplied configuration task")?;
        let mut next = self.clone();
        let runtime = next
            .config_scan_runtime
            .as_mut()
            .ok_or("Missing configuration work")?;
        let live = if practice {
            &mut runtime.example
        } else {
            &mut runtime.assignment
        };
        let result = live.apply(c, op);
        // Failed scans retain their actual saved input/diagnostic for retry. A
        // rejected edit/save is atomic and leaves the whole session unchanged.
        if result.is_err() && !matches!(op, flow::Operation::Scan) {
            return result.map(|_| String::new());
        }
        let output=match &result {
            Err(e)=>format!("Scan needs attention: {e}"),
            Ok(())=>match op {
                flow::Operation::Scan=>format!("Scanned {} supplied virtual inventories. Results are actual simulated device reads; Check answers compares the requested selections.",c.cases.len()),
                flow::Operation::SaveConfig{..}=>"Saved simulated LVM configuration. Run scan to observe its effect.".into(),
                flow::Operation::SaveSelection{..}=>"Saved simulated devname selection. Real hardware-ID file serialization is not modeled. Run scan to observe its effect.".into(),
            }
        };
        let rows = if practice {
            &mut next.example_transcript
        } else {
            &mut next.transcript
        };
        if rows.len() == 32 {
            rows.remove(0);
        }
        rows.push(Row {
            input: op.label().into(),
            output: output.clone(),
        });
        if practice {
            next.example_checked = false;
        } else {
            next.checked = false;
        }
        d.bound(&next)?;
        *self = next;
        result.map(|_| output)
    }
}
