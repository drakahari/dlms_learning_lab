//! Caller-held source admission. Saved data never supplies its grading definition.
use crate::{
    directory_policy_workspace::{Definition, Session},
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
};
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::DirectoryPolicy { simulation }, Goal::DirectoryPolicy { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Directory-policy task and workspace must agree.".into());
    };
    definition.validate()?;
    if definition.source_id != s.id || **simulation != Session::new(definition) {
        return Err("Directory-policy source needs its declared independent starting files and matching identifier.".into());
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::DirectoryPolicy { simulation } = &a.state else {
        return Err(
            "Saved directory-policy work belongs to another workspace; prior data retained.".into(),
        );
    };
    if a.scenario_version != s.version
        || a.step >= s.steps.len()
        || a.hints > s.hints.len()
        || a.transcript.len() > 120
        || a.ipv4.is_some()
        || a.workspace.as_ref().is_some_and(|w| {
            serde_json::to_value(w).ok()
                != serde_json::to_value(crate::workspace::WorkspaceData::new()).ok()
        })
    {
        return Err("Incompatible directory-policy attempt metadata; prior data retained.".into());
    }
    d.bound(simulation)
}
