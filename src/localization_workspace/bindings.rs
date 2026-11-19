//! Admit the authored source, never infer objectives from a saved world.
use super::*;
pub const HELP: &str = crate::localization::command::HELP;
pub const PREFIX: &str = "LOCALIZATION WORK\n";
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::Localization { simulation },
        crate::engine::Goal::Localization { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Locale/encoding state and objective must use the same supplied task.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id {
        return Err("The independent locale/encoding task ID must match its scenario ID.".into());
    }
    if **simulation != Session::new(definition) {
        return Err(
            "Locale/encoding work must start with separate declared example and assignment drafts."
                .into(),
        );
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::Localization { .. })
        || matches!(s.initial, crate::engine::State::Localization { .. })
        || matches!(s.goal, crate::engine::Goal::Localization { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let definition = source(s)?;
    let crate::engine::State::Localization { simulation } = &a.state else {
        return Err("Saved locale/encoding work belongs to another activity.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible locale/encoding metadata; original retained.".into());
    }
    simulation.validate(definition)
}
impl Definition {
    pub fn checks(&self, s: &Session, practice: bool) -> Vec<(bool, String)> {
        if let Err(error) = self.validate().and_then(|()| s.validate(self)) {
            return vec![(false, error)];
        }
        let (draft, task) = if practice {
            (&s.practice, &self.example)
        } else {
            (&s.assignment, &self.initial)
        };
        if !draft.attempted {
            return vec![(
                false,
                "Not checked. Investigate the supplied work, repair it, then choose Check work."
                    .into(),
            )];
        }
        match draft.checks(task) {
            Ok(rows) => rows
                .into_iter()
                .map(|r| (r.correct, format!("{}: {}", r.label, r.feedback)))
                .collect(),
            Err(error) => vec![(false, error)],
        }
    }
}
impl Session {
    /// Developer-only source reachability route. Native controls perform typed
    /// actions directly; the terminal never teaches/accepts these JSON helpers.
    pub fn execute_reference(&mut self, d: &Definition, raw: &str) -> Result<String> {
        d.validate()?;
        self.validate(d)?;
        if raw.len() > 8192 {
            return Err("Source action exceeds its supported bounds.".into());
        }
        let mut next = self.clone();
        let message = if raw == "@locale-check" {
            next.assignment.check(&d.initial)?;
            "Checked actual current locale/encoding outcomes.".into()
        } else {
            let output = if let Some(body) = raw.strip_prefix(PREFIX) {
                let action: Action = serde_json::from_str(body)
                    .map_err(|e| format!("Unsupported typed locale/encoding action: {e}"))?;
                next.assignment
                    .perform(&d.initial, action)?
                    .map_err(|e| e.to_string())?
            } else {
                if raw.len() > 1024 {
                    return Err("Keep one simulated terminal command within 1,024 bytes.".into());
                }
                next.assignment.input = raw.into();
                next.assignment
                    .run(&d.initial)?
                    .map_err(|e| e.to_string())?
            };
            format!(
                "Exit status {}\n{}{}",
                output.status,
                crate::localization::readable(&output.stdout),
                output.stderr
            )
        };
        next.validate(d)?;
        *self = next;
        Ok(message)
    }
}
