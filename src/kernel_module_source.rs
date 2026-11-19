//! Authored module metadata/goals are supplied by the catalogue, never saved work.
use crate::{
    engine::{Goal, State},
    kernel_module_workspace::{Definition, Session},
    pack::Scenario,
    progress::Attempt,
};
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::KernelModules { simulation }, Goal::KernelModules { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Module task and supplied starting workspace must agree.".into());
    };
    definition.validate()?;
    if definition.source_id != s.id || **simulation != Session::new(definition)? {
        return Err(
            "Module source needs its declared independent initial state and matching identifier."
                .into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::KernelModules { simulation } = &a.state else {
        return Err("Saved module work belongs to another workspace; original retained.".into());
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
        return Err("Incompatible module attempt metadata; original retained.".into());
    }
    d.bound(simulation)
}
