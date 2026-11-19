//! Caller-held catalog admission. Persisted work cannot replace its task definition.
use crate::{
    autofs_workspace::{Definition, Session},
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
};
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::Autofs { simulation }, Goal::Autofs { definition }) = (&s.initial, &s.goal) else {
        return Err("AutoFS task and workspace must agree".into());
    };
    definition.validate()?;
    let actions = definition
        .reference
        .iter()
        .map(|s| s.operation.label())
        .chain(std::iter::once("Check work".into()))
        .collect::<Vec<_>>();
    if s.solution != actions {
        return Err("AutoFS reference descriptions must match their typed native actions; no Save packet is a Linux command".into());
    }

    if definition.source_id != s.id || **simulation != Session::new(definition)? {
        return Err(
            "AutoFS source must declare its identifier and independent starting work".into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::Autofs { simulation } = &a.state else {
        return Err("Saved AutoFS work belongs to another workspace; prior data retained".into());
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
        return Err("Incompatible AutoFS attempt metadata; prior data retained".into());
    }
    d.bound(simulation)
}
