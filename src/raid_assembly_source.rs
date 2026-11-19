//! Caller-held catalog admission. Persisted work cannot replace its task definition.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
    raid_assembly_workspace::{Definition, Session},
};
pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::RaidAssembly { simulation }, Goal::RaidAssembly { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("RAID assembly task and workspace must agree".into());
    };
    definition.validate()?;
    let actions = definition
        .reference
        .iter()
        .map(|s| s.operation.label())
        .chain(std::iter::once("Check work".into()))
        .collect::<Vec<_>>();
    if s.solution != actions {
        return Err("RAID assembly reference descriptions must match their typed native actions; Save file is a native button, not a Linux command".into());
    }

    if definition.source_id != s.id || **simulation != Session::new(definition)? {
        return Err(
            "RAID assembly source must declare its identifier and independent starting work".into(),
        );
    }
    Ok(definition)
}
pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::RaidAssembly { simulation } = &a.state else {
        return Err(
            "Saved RAID assembly work belongs to another workspace; prior data retained".into(),
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
        return Err("Incompatible RAID assembly attempt metadata; prior data retained".into());
    }
    d.bound(simulation)
}
