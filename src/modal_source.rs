//! Authored modal-editor binding; no learner command dispatch outside simulation.
use crate::{
    engine::{Goal, State},
    modal_workspace::{Definition, Session},
    pack::Scenario,
};
pub const HELP: &str = "This is a bounded native vi/Vim-style simulator, not a host editor. NORMAL: h j k l (or arrows), 0 $, gg G, i a o, x dd yy p P u, / ? n. INSERT: printable ASCII, Enter, Backspace, Escape; paste is text only. Colon commands: :w :wq :q :q!, followed by Enter. Tab navigates app controls. Plain case-sensitive search wraps; no regular expressions, counts, visual mode, named registers, redo, options, shell escapes or arbitrary files. Insert Backspace uses Vim-style permissive deletion, including joining lines; editor configurations can differ. Undo is bounded to32 changes, with one Insert session grouped. Activities use up to4 supplied4 KiB virtual files. Actual saved files and unsaved buffers are distinct; Check reads saved outcomes and requires clean buffers.";
impl Session {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.id.is_empty() || self.id.len() > 80 {
            return Err("Invalid modal activity identity.".into());
        }
        for a in [&self.assignment, &self.practice] {
            if a.editors.is_empty() || a.editors.len() > 4 || !a.editors.contains_key(&a.selected) {
                return Err("Invalid bounded modal activity files.".into());
            }
            for (p, e) in &a.editors {
                if p.len() > 120
                    || !p.starts_with('/')
                    || e.contents().len() > 4096
                    || e.file_bytes().len() > 4096
                {
                    return Err("Invalid bounded modal virtual file.".into());
                }
                e.validate()?;
            }
        }
        Ok(())
    }
    pub fn execute_reference(&mut self, d: &Definition, input: &str) -> Result<String, String> {
        d.context(self)?;
        if input.len() > 16384 {
            return Err("Modal authoring event exceeds its bound.".into());
        }
        let raw=input.strip_prefix("modal-lab ").ok_or("Use the native editor. modal-lab JSON is a simulator authoring format, not a Linux or vi command.")?;
        let event = serde_json::from_str(raw)
            .map_err(|e| format!("Invalid declarative modal event: {e}"))?;
        self.apply(d, false, &event)?;
        Ok("Virtual editing action applied.".into())
    }
}
impl Definition {
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(e) = self.context(s) {
            return vec![(false, e)];
        }
        let (a, f) = if practice {
            (&s.practice, &self.example)
        } else {
            (&s.assignment, &self.initial)
        };
        let mut rows = vec![(
            a.checked,
            "Use Check saved files to evaluate the current saved outcomes.".into(),
        )];
        rows.extend(a.checks(f));
        rows
    }
}
fn admitted(s: &Scenario) -> Result<&Definition, String> {
    let (State::ModalEditing { simulation }, Goal::ModalEditing { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Modal editing task and workspace do not match.".into());
    };
    if s.track != "LPIC-1"
        || s.id != definition.id
        || s.version != definition.version
        || **simulation != definition.new_session()?
    {
        return Err("Match the supplied editing lesson, version and initial session.".into());
    }
    Ok(definition)
}
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let d = admitted(s)?;
    d.validate()?;
    Ok(d)
}
pub fn context(a: &crate::progress::Attempt, s: &Scenario) -> Result<(), String> {
    let d = admitted(s)?;
    let State::ModalEditing { simulation } = &a.state else {
        return Err("Saved work belongs to another workspace.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Saved modal progress metadata is incompatible.".into());
    }
    simulation.validate()?;
    d.context(simulation)
}
