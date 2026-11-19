//! Authored-source admission. No guessing a task from a saved state.
use super::*;
pub fn source(s: &crate::pack::Scenario) -> Result<&Definition> {
    let (
        crate::engine::State::PrintQueues { simulation },
        crate::engine::Goal::PrintQueues { definition },
    ) = (&s.initial, &s.goal)
    else {
        return Err("Print state and objective must use the same supplied task.".into());
    };
    definition.validate()?;
    if definition.initial.id != s.id {
        return Err(
            "The independent print task identifier must match its scenario identifier.".into(),
        );
    }
    if **simulation != Session::new(definition) {
        return Err(
            "Print work must begin with separate declared practice and assignment drafts.".into(),
        );
    }
    Ok(definition)
}
pub fn relevant(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> bool {
    matches!(a.state, crate::engine::State::PrintQueues { .. })
        || matches!(s.initial, crate::engine::State::PrintQueues { .. })
        || matches!(s.goal, crate::engine::Goal::PrintQueues { .. })
}
pub fn context(a: &crate::progress::Attempt, s: &crate::pack::Scenario) -> Result<()> {
    let definition = source(s)?;
    let crate::engine::State::PrintQueues { simulation } = &a.state else {
        return Err("Saved print work belongs to another activity.".into());
    };
    if a.scenario_version != s.version
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible print-work metadata; original retained.".into());
    }
    simulation.validate(definition)
}
