//! Authored task authority stays in the bundled catalog, never saved progress.
use crate::{
    engine::{Goal, State},
    pack::Scenario,
    progress::Attempt,
    recursive_tree_workspace::{CHECK_ACTION, Definition, Session},
};

pub fn source(s: &Scenario) -> Result<&Definition, String> {
    let (State::RecursiveTree { simulation }, Goal::RecursiveTree { definition }) =
        (&s.initial, &s.goal)
    else {
        return Err("Recursive file task and supplied workspace must agree.".into());
    };
    definition.validate()?;
    if definition.source_id != s.id || **simulation != Session::new(definition)? {
        return Err("Recursive source needs its declared independent starting files and matching identifier.".into());
    }
    Ok(definition)
}

pub fn context(a: &Attempt, s: &Scenario) -> Result<(), String> {
    let d = source(s)?;
    let State::RecursiveTree { simulation } = &a.state else {
        return Err(
            "Saved recursive file work belongs to another workspace; original retained.".into(),
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
        return Err("Incompatible recursive file attempt metadata; original retained.".into());
    }
    d.bound(simulation)
}

pub fn execute(s: &mut Session, d: &Definition, input: &str) -> Result<String, String> {
    if input == CHECK_ACTION {
        s.check(d, false)?;
        Ok("Current simulated tree checked.".into())
    } else {
        s.run(d, input, false)
    }
}
